//! Intersections between two paths.
//!
//! Every pair of drawing segments, one from each path, is intersected with
//! the method that suits the pair:
//!
//! - **Line and line** — solved directly: the two parameters come out of a
//!   2×2 linear system.
//! - **Line and quadratic or cubic curve** — kurbo turns the curve's
//!   distance to the line into a polynomial and finds its real roots.
//! - **Line and elliptical arc** — the line is moved into the frame where
//!   the ellipse is the unit circle, where meeting it is a quadratic
//!   equation; the roots are mapped back to angles on the arc.
//! - **Two curves** (quadratic, cubic or arc, in any mix) — found by
//!   subdivision, then made exact by Newton's method. See [`subdivide`].
//!
//! Points found twice, where one path crosses the joint of two segments of
//! the other, are merged. Segments that overlap along a stretch, like two
//! collinear lines, meet at infinitely many points and are left out.

use euclid::default::Point2D;
use kurbo::{Arc, Line, ParamCurve, PathSeg, Rect, Vec2};

use crate::crossing::is_crossing;
use crate::measure::{PathMeasure, Piece, PieceShape, Position};

/// Where a point lies on one path: its length along the path, and the
/// segment and parameter it falls on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    pub length: f64,
    pub position: Position,
}

/// A point where two paths meet, as found by [`PathMeasure::intersections`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Intersection {
    pub point: Point2D<f64>,
    /// Where the point lies on the path `intersections` was called on
    pub this: Location,
    /// Where it lies on the other path
    pub other: Location,
    /// Whether the other path passes from one side of this one to the
    /// other there, rather than touching it and turning back, as a line
    /// touching a circle does. A point at an open end of either path is
    /// not a crossing.
    pub crossing: bool,
}

/// Subdivision stops once both parts fit in a box this wide, and Newton's
/// method takes over.
const LEAF_SIZE: f64 = 1e-6;
/// How many leaves one pair of curves may produce. Curves that keep
/// overlapping past this share a stretch rather than crossing at points.
const LEAF_BUDGET: usize = 4096;
/// Newton's method stops once the two points are this close.
const CONVERGED: f64 = 1e-10;
/// Points closer than this, or than this fraction of the paths' size, are
/// the same intersection. Where curves touch, Newton's method only gets
/// to about the square root of the rounding error, so the same point can
/// be found a little apart from two pairs of segments.
const SAME_POINT: f64 = 1e-7;

impl PathMeasure {
    /// The points where this path meets `other`, in order along this path.
    ///
    /// Each [`Intersection`] tells where the point lies on both paths, by
    /// length and by segment, so it can go straight into [`crop`] or
    /// [`point_at`]. Arcs are intersected as the ellipse arcs they are.
    ///
    /// Segments that overlap along a stretch, rather than cross or touch,
    /// have no single meeting point and are not reported.
    ///
    /// # Algorithm
    ///
    /// Every drawing segment of this path is paired with every one of
    /// `other` whose bounding box it overlaps, and each pair is solved by
    /// the method that suits it:
    ///
    /// - two lines: directly, from a 2×2 linear system;
    /// - a line and a quadratic or cubic curve: kurbo's root finding on the
    ///   polynomial of the curve's distance to the line;
    /// - a line and an arc: a quadratic equation, after moving the line
    ///   into the frame where the arc's ellipse is the unit circle;
    /// - two curves, arcs included: recursive subdivision. Each curve is
    ///   bounded by a box: a Bézier curve by its control points, which
    ///   enclose it, and an arc exactly, by its ends and the points where
    ///   it turns in x or y. Where the boxes overlap, the larger curve is
    ///   cut in half and both halves are tried again; where they do not,
    ///   the parts cannot meet. Once both parts fit in a box a millionth
    ///   of a unit wide, Newton's method on the two exact curves turns the
    ///   pair into a point accurate to about 1e-10.
    ///
    /// Points found more than once, as where a path crosses the joint of
    /// two segments of the other, are merged.
    ///
    /// [`crop`]: PathMeasure::crop
    /// [`point_at`]: PathMeasure::point_at
    ///
    /// ```
    /// use svg_path_ops::euclid::default::Point2D;
    /// use svg_path_ops::svgtypes::PathParser;
    /// use svg_path_ops::PathMeasure;
    ///
    /// let circle: Vec<_> = PathParser::from("M 10 0 A 10 10 0 0 1 -10 0 A 10 10 0 0 1 10 0")
    ///     .collect::<Result<_, _>>()?;
    /// let line: Vec<_> = PathParser::from("M -20 6 H 20").collect::<Result<_, _>>()?;
    ///
    /// let meets = PathMeasure::new(&circle).intersections(&PathMeasure::new(&line));
    /// assert_eq!(meets.len(), 2);
    /// assert!((meets[0].point - Point2D::new(8.0, 6.0)).length() < 1e-9);
    /// assert!((meets[1].point - Point2D::new(-8.0, 6.0)).length() < 1e-9);
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn intersections(&self, other: &PathMeasure) -> Vec<Intersection> {
        let mut found: Vec<Intersection> = Vec::new();
        for a in self.pieces.iter().filter(|piece| piece.length > 0.0) {
            let a_bounds = a.shape.bounds(0.0, 1.0);
            for b in other.pieces.iter().filter(|piece| piece.length > 0.0) {
                if !overlap(a_bounds, b.shape.bounds(0.0, 1.0)) {
                    continue;
                }
                for (t, s) in intersect_pieces(&a.shape, &b.shape) {
                    let point = a.shape.eval(t);
                    found.push(Intersection {
                        point: Point2D::new(point.x, point.y),
                        this: location(a, t),
                        other: location(b, s),
                        crossing: false,
                    });
                }
            }
        }

        let size = match (self.bounds(), other.bounds()) {
            (Some(a), Some(b)) => (a.union(&b).max - a.union(&b).min).length(),
            _ => 0.0,
        };
        let same = SAME_POINT.max(SAME_POINT * size);
        found.sort_by(|p, q| {
            p.this
                .length
                .total_cmp(&q.this.length)
                .then(p.other.length.total_cmp(&q.other.length))
        });
        let mut merged: Vec<Intersection> = Vec::with_capacity(found.len());
        for intersection in found {
            let seen = merged
                .iter()
                .any(|kept| (kept.point - intersection.point).length() <= same);
            if !seen {
                merged.push(intersection);
            }
        }

        // Crossing or touching, probing no further along either path than
        // a third of the way to the nearest other intersection
        let gap = |i: usize, length: fn(&Intersection) -> f64| {
            merged
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, x)| (length(x) - length(&merged[i])).abs())
                .fold(f64::INFINITY, f64::min)
        };
        let crossings: Vec<bool> = (0..merged.len())
            .map(|i| {
                let x = &merged[i];
                is_crossing(
                    self,
                    other,
                    kurbo::Point::new(x.point.x, x.point.y),
                    &x.this,
                    &x.other,
                    gap(i, |x| x.this.length),
                    gap(i, |x| x.other.length),
                    size,
                )
            })
            .collect();
        for (x, crossing) in merged.iter_mut().zip(crossings) {
            x.crossing = crossing;
        }
        merged
    }
}

fn location(piece: &Piece, t: f64) -> Location {
    Location {
        length: piece.start + piece.shape.length_to(t),
        position: Position { index: piece.index, t },
    }
}

/// Whether two boxes share a point, edges included.
fn overlap(a: Rect, b: Rect) -> bool {
    a.x0 <= b.x1 && b.x0 <= a.x1 && a.y0 <= b.y1 && b.y0 <= a.y1
}

/// The parameters, on `a` and on `b`, of the points where they meet.
fn intersect_pieces(a: &PieceShape, b: &PieceShape) -> Vec<(f64, f64)> {
    let swap = |pairs: Vec<(f64, f64)>| pairs.into_iter().map(|(s, t)| (t, s)).collect();
    let hits: Vec<(f64, f64)> = match (a, b) {
        (PieceShape::Line(a), PieceShape::Line(b)) => return line_line(a, b).into_iter().collect(),
        (PieceShape::Line(line), curve) => swap(curve_line(curve, line)),
        (curve, PieceShape::Line(line)) => curve_line(curve, line),
        (a, b) => return curve_curve(a, b),
    };
    // Closed-form roots lose digits far from the origin; a few Newton steps
    // on the exact segments win them back
    hits.into_iter()
        .map(|(t, s)| newton(a, b, t, s).unwrap_or((t, s)))
        .collect()
}

/// Where two line segments cross, if they do at a single point.
fn line_line(a: &Line, b: &Line) -> Option<(f64, f64)> {
    let (da, db) = (a.p1 - a.p0, b.p1 - b.p0);
    let denominator = da.cross(db);
    // Parallel lines meet nowhere, or along a stretch when collinear
    if denominator.abs() <= f64::EPSILON * da.hypot() * db.hypot() {
        return None;
    }
    let offset = b.p0 - a.p0;
    let t = offset.cross(db) / denominator;
    let s = offset.cross(da) / denominator;
    let on_both = |u: f64| (-1e-12..=1.0 + 1e-12).contains(&u);
    (on_both(t) && on_both(s)).then(|| (t.clamp(0.0, 1.0), s.clamp(0.0, 1.0)))
}

/// Where a curve meets a line segment, as (curve parameter, line parameter).
fn curve_line(curve: &PieceShape, line: &Line) -> Vec<(f64, f64)> {
    match curve {
        PieceShape::Line(other) => line_line(other, line).into_iter().collect(),
        PieceShape::Quadratic(quad) => PathSeg::Quad(*quad)
            .intersect_line(*line)
            .iter()
            .map(|hit| (hit.segment_t, hit.line_t))
            .collect(),
        PieceShape::Cubic(cubic) => PathSeg::Cubic(*cubic)
            .intersect_line(*line)
            .iter()
            .map(|hit| (hit.segment_t, hit.line_t))
            .collect(),
        PieceShape::Arc(arc) => arc_line(arc, line),
    }
}

/// Where an arc meets a line segment, as (arc parameter, line parameter).
///
/// In the frame where the arc's ellipse is the unit circle, a point
/// `p + u·d` of the line is on the ellipse when `|p + u·d|² = 1`, a
/// quadratic equation in `u`. Each root on the segment gives an angle, which
/// is on the arc when it falls within the arc's sweep.
fn arc_line(arc: &Arc, line: &Line) -> Vec<(f64, f64)> {
    let (sin, cos) = arc.x_rotation.sin_cos();
    let to_unit = |v: Vec2| {
        Vec2::new(
            (v.x * cos + v.y * sin) / arc.radii.x,
            (-v.x * sin + v.y * cos) / arc.radii.y,
        )
    };
    let p = to_unit(line.p0 - arc.center);
    let d = to_unit(line.p1 - line.p0);
    let (a, b, c) = (d.dot(d), 2.0 * p.dot(d), p.dot(p) - 1.0);
    let discriminant = b * b - 4.0 * a * c;
    // A line touching the ellipse has a zero discriminant, which rounding
    // turns into a tiny one of either sign
    let touching = discriminant.abs() <= 1e-12 * (b * b + (4.0 * a * c).abs());
    if a == 0.0 || (discriminant < 0.0 && !touching) {
        return Vec::new();
    }
    let root = if touching { 0.0 } else { discriminant.sqrt() };
    let mut hits = Vec::new();
    for u in [(-b - root) / (2.0 * a), (-b + root) / (2.0 * a)] {
        if !(-1e-12..=1.0 + 1e-12).contains(&u) {
            continue;
        }
        let on_circle = p + d * u;
        let angle = on_circle.y.atan2(on_circle.x);
        if let Some(t) = arc_parameter(arc, angle) {
            hits.push((t, u.clamp(0.0, 1.0)));
        }
        // A line touching the ellipse gives the same root twice
        if root == 0.0 {
            break;
        }
    }
    hits
}

/// The arc's parameter at `angle`, if the arc sweeps over it.
fn arc_parameter(arc: &Arc, angle: f64) -> Option<f64> {
    let turn = (angle - arc.start_angle) * arc.sweep_angle.signum();
    let t = turn.rem_euclid(std::f64::consts::TAU) / arc.sweep_angle.abs();
    // Just past the end, rounding may have wrapped the turn around
    let t = if t > 1.0 && (std::f64::consts::TAU / arc.sweep_angle.abs() - t).abs() < 1e-9 {
        0.0
    } else {
        t
    };
    (t <= 1.0 + 1e-9).then(|| t.clamp(0.0, 1.0))
}

/// Where two curves meet, found by subdivision and made exact by Newton's
/// method. Curves that overlap along a stretch produce too many leaves and
/// give nothing.
///
/// Every meeting point ends up in a few neighbouring leaves: one or two
/// where the curves cross at an angle, a whole run of them where they
/// touch or cross while running side by side, closer than a leaf for a
/// while. So the leaves are first grouped into runs of neighbours, and each
/// run gives one point: Newton's method from the leaf where the curves come
/// closest, or that leaf itself when the curves run too nearly parallel for
/// Newton's method to settle.
fn curve_curve(a: &PieceShape, b: &PieceShape) -> Vec<(f64, f64)> {
    let mut leaves = Vec::new();
    subdivide(a, (0.0, 1.0), b, (0.0, 1.0), &mut leaves);
    if leaves.len() > LEAF_BUDGET {
        return Vec::new();
    }
    leaves.sort_by(|p, q| p.0.total_cmp(&q.0).then(p.1.total_cmp(&q.1)));

    let gap = |&(t, s): &(f64, f64)| (a.eval(t) - b.eval(s)).hypot();
    let mut hits = Vec::new();
    for run in leaves.chunk_by(|p, q| (a.eval(p.0) - a.eval(q.0)).hypot() <= 8.0 * LEAF_SIZE) {
        // Where the curves run side by side, many leaves may be as close as
        // floating point can tell; the middle one of those is the best guess
        let least = run.iter().map(gap).fold(f64::INFINITY, f64::min);
        let closest: Vec<&(f64, f64)> = run
            .iter()
            .filter(|leaf| gap(leaf) <= (2.0 * least).max(CONVERGED))
            .collect();
        let closest = closest[closest.len() / 2];
        match newton(a, b, closest.0, closest.1) {
            Some(hit) => hits.push(hit),
            None if gap(closest) <= LEAF_SIZE => hits.push(*closest),
            None => {}
        }
    }
    hits
}

/// Collects pairs of parameter ranges, one on each curve, that are too
/// small to cut further and whose boxes still overlap.
///
/// A curve's part over a parameter range lies inside its box (see
/// [`PieceShape::bounds`]), so parts with disjoint boxes cannot meet and
/// are dropped. Otherwise the part with the larger box is cut in half and
/// each half is tried against the other part. Each cut halves a box, so a
/// crossing ends in a leaf a millionth wide after about 20 cuts per curve,
/// while every part away from a crossing is soon dropped.
fn subdivide(
    a: &PieceShape,
    (a0, a1): (f64, f64),
    b: &PieceShape,
    (b0, b1): (f64, f64),
    leaves: &mut Vec<(f64, f64)>,
) {
    if leaves.len() > LEAF_BUDGET {
        return;
    }
    let (box_a, box_b) = (a.bounds(a0, a1), b.bounds(b0, b1));
    if !overlap(box_a, box_b) {
        return;
    }
    let size = |r: Rect| r.width().max(r.height());
    let (size_a, size_b) = (size(box_a), size(box_b));
    let (a_middle, b_middle) = ((a0 + a1) / 2.0, (b0 + b1) / 2.0);
    // Parameters that no longer split also end the search
    let a_splits = a_middle > a0 && a_middle < a1;
    let b_splits = b_middle > b0 && b_middle < b1;
    if (size_a <= LEAF_SIZE && size_b <= LEAF_SIZE) || (!a_splits && !b_splits) {
        leaves.push((a_middle, b_middle));
        return;
    }
    if a_splits && (size_a >= size_b || !b_splits) {
        subdivide(a, (a0, a_middle), b, (b0, b1), leaves);
        subdivide(a, (a_middle, a1), b, (b0, b1), leaves);
    } else {
        subdivide(a, (a0, a1), b, (b0, b_middle), leaves);
        subdivide(a, (a0, a1), b, (b_middle, b1), leaves);
    }
}

/// Refines where `a` and `b` meet from parameters close to it, by Newton's
/// method on `a(t) − b(s) = 0`. Returns `None` when it does not settle on a
/// point where the curves meet.
///
/// Each step solves the 2×2 system with the curves' derivatives as columns.
/// Where the curves run parallel, as where they touch, the system is
/// singular and the step fails; the caller then decides from the leaf.
fn newton(a: &PieceShape, b: &PieceShape, mut t: f64, mut s: f64) -> Option<(f64, f64)> {
    for _ in 0..32 {
        let gap = a.eval(t) - b.eval(s);
        if gap.hypot() <= CONVERGED {
            return Some((t, s));
        }
        let (da, db) = (a.derivative(t), b.derivative(s));
        let determinant = da.cross(db);
        if determinant.abs() < 1e-18 {
            break;
        }
        // Solve da·Δt − db·Δs = −gap
        let dt = gap.cross(db) / determinant;
        let ds = gap.cross(da) / determinant;
        t = (t - dt).clamp(0.0, 1.0);
        s = (s - ds).clamp(0.0, 1.0);
    }
    ((a.eval(t) - b.eval(s)).hypot() <= 10.0 * CONVERGED).then_some((t, s))
}

impl PieceShape {
    /// A box around the piece's part between parameters `t0` and `t1`.
    ///
    /// A Bézier curve's part lies inside the convex hull of its control
    /// points, so their box encloses it. An arc's box is exact: the part
    /// reaches its furthest in x or y either at its ends or where it turns,
    /// at the angles where the derivative of x or y is zero.
    pub(crate) fn bounds(&self, t0: f64, t1: f64) -> Rect {
        let hull = |points: &[kurbo::Point]| {
            points
                .iter()
                .skip(1)
                .fold(Rect::from_points(points[0], points[0]), |r, p| {
                    r.union_pt(*p)
                })
        };
        match self {
            PieceShape::Line(line) => hull(&[line.eval(t0), line.eval(t1)]),
            PieceShape::Quadratic(quad) => {
                let part = quad.subsegment(t0..t1);
                hull(&[part.p0, part.p1, part.p2])
            }
            PieceShape::Cubic(cubic) => {
                let part = cubic.subsegment(t0..t1);
                hull(&[part.p0, part.p1, part.p2, part.p3])
            }
            PieceShape::Arc(arc) => {
                let (low, high) = (t0.min(t1), t0.max(t1));
                let mut points = vec![arc.eval(t0), arc.eval(t1)];
                points.extend(
                    self.axis_turns()
                        .into_iter()
                        .filter(|t| (low..=high).contains(t))
                        .map(|t| arc.eval(t)),
                );
                hull(&points)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use euclid::default::Point2D;
    use svgtypes::PathParser;

    use crate::{PathMeasure, PathSegment};

    fn measure(path: &str) -> PathMeasure {
        let segments: Vec<PathSegment> = PathParser::from(path).map(Result::unwrap).collect();
        PathMeasure::new(&segments)
    }

    fn points(a: &str, b: &str) -> Vec<Point2D<f64>> {
        measure(a)
            .intersections(&measure(b))
            .iter()
            .map(|i| i.point)
            .collect()
    }

    fn close(a: Point2D<f64>, b: Point2D<f64>) -> bool {
        (a - b).length() < 1e-9
    }

    fn same_points(found: &[Point2D<f64>], expected: &[Point2D<f64>]) -> bool {
        found.len() == expected.len() && found.iter().zip(expected).all(|(a, b)| close(*a, *b))
    }

    const CIRCLE: &str = "M 10 0 A 10 10 0 0 1 -10 0 A 10 10 0 0 1 10 0";

    #[test]
    fn crossing_lines() {
        assert!(same_points(
            &points("M 0 0 L 10 10", "M 0 10 L 10 0"),
            &[Point2D::new(5.0, 5.0)]
        ));
        // Lines that stop short of each other do not meet
        assert!(points("M 0 0 L 4 4", "M 0 10 L 10 0").is_empty());
    }

    #[test]
    fn parallel_and_overlapping_lines_meet_nowhere_single() {
        assert!(points("M 0 0 L 10 0", "M 0 1 L 10 1").is_empty());
        assert!(points("M 0 0 L 10 0", "M 5 0 L 15 0").is_empty());
    }

    #[test]
    fn a_crossing_at_a_joint_is_found_once() {
        let found = points("M 0 0 L 10 0 L 20 0", "M 10 -5 V 5");
        assert!(same_points(&found, &[Point2D::new(10.0, 0.0)]));
    }

    #[test]
    fn line_and_circle() {
        // x² + 6² = 10² at x = ±8; this path runs right to left
        let found = points("M 20 6 H -20", CIRCLE);
        assert!(same_points(
            &found,
            &[Point2D::new(8.0, 6.0), Point2D::new(-8.0, 6.0)]
        ));
        // A tangent line touches once
        let touching = points("M -20 10 H 20", CIRCLE);
        assert!(same_points(&touching, &[Point2D::new(0.0, 10.0)]));
    }

    #[test]
    fn line_and_rotated_ellipse() {
        let ellipse = "M 20 0 A 20 10 30 0 1 -20 0 A 20 10 30 0 1 20 0";
        let found = measure("M -30 3 L 30 -2").intersections(&measure(ellipse));
        assert_eq!(found.len(), 2);
        for hit in &found {
            // On the line, and on the ellipse
            let on_ellipse = measure(ellipse).nearest(hit.point).unwrap().distance;
            let on_line = measure("M -30 3 L 30 -2")
                .nearest(hit.point)
                .unwrap()
                .distance;
            assert!(on_ellipse < 1e-9 && on_line < 1e-9);
        }
    }

    #[test]
    fn two_circles() {
        // Radius 10 around (0, 0) and around (12, 0) meet at x = 6, y = ±8
        let other = "M 22 0 A 10 10 0 0 1 2 0 A 10 10 0 0 1 22 0";
        let mut found = points(CIRCLE, other);
        found.sort_by(|a, b| a.y.total_cmp(&b.y));
        assert!(same_points(
            &found,
            &[Point2D::new(6.0, -8.0), Point2D::new(6.0, 8.0)]
        ));
    }

    #[test]
    fn cubic_curves_cross_where_both_run() {
        // A wave and its mirror image across y = 0 meet where the wave does:
        // at both ends and in the middle, crossing at an angle each time
        let a = "M 0 0 C 50 150 50 -150 100 0";
        let b = "M 0 0 C 50 -150 50 150 100 0";
        let found = measure(a).intersections(&measure(b));
        let expected = [
            Point2D::new(0.0, 0.0),
            Point2D::new(50.0, 0.0),
            Point2D::new(100.0, 0.0),
        ];
        let found_points: Vec<_> = found.iter().map(|hit| hit.point).collect();
        assert!(same_points(&found_points, &expected), "{found_points:?}");
        for hit in &found {
            let on_a = measure(a).point_at(hit.this.length).unwrap();
            let on_b = measure(b).point_at(hit.other.length).unwrap();
            assert!(close(on_a, hit.point) && close(on_b, hit.point));
        }
        // In order along the first path
        assert!(found
            .windows(2)
            .all(|pair| pair[0].this.length < pair[1].this.length));
    }

    #[test]
    fn curves_crossing_side_by_side_meet_once() {
        // Both run straight up or down through (50, 50), where they cross
        // with parallel tangents, lying within a millionth of each other for
        // a stretch; that is one meeting point, not a run of them
        let a = "M 0 0 C 100 0 0 100 100 100";
        let b = "M 0 100 C 100 100 0 0 100 0";
        let found = points(a, b);
        assert_eq!(found.len(), 1);
        assert!((found[0] - Point2D::new(50.0, 50.0)).length() < 1e-5);
    }

    #[test]
    fn curve_and_arc() {
        let curve = "M -15 -5 C -5 30 5 -30 15 5";
        let found = measure(curve).intersections(&measure(CIRCLE));
        assert!(!found.is_empty());
        for hit in &found {
            assert!((hit.point.to_vector().length() - 10.0).abs() < 1e-9);
            assert!(measure(curve).nearest(hit.point).unwrap().distance < 1e-9);
        }
    }

    #[test]
    fn positions_name_the_segments() {
        let found = measure("M 0 5 H 4 L 10 5").intersections(&measure("M 7 0 V 10"));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].this.position.index, 2);
        assert_eq!(found[0].other.position.index, 1);
        assert!((found[0].this.position.t - 0.5).abs() < 1e-12);
    }

    #[test]
    fn identical_curves_overlap_and_give_no_points() {
        let curve = "M 0 0 C 30 80 60 -40 100 20";
        assert!(points(curve, curve).is_empty());
    }

    #[test]
    fn paths_that_draw_nothing_meet_nothing() {
        assert!(points("M 5 5", CIRCLE).is_empty());
        assert!(points(CIRCLE, "").is_empty());
    }

    /// Whether each meeting point of `a` with `b` is a crossing.
    fn crossings(a: &str, b: &str) -> Vec<bool> {
        measure(a)
            .intersections(&measure(b))
            .iter()
            .map(|x| x.crossing)
            .collect()
    }

    const CLOSED_CIRCLE: &str = "M 10 0 A 10 10 0 0 1 -10 0 A 10 10 0 0 1 10 0 Z";

    #[test]
    fn crossing_or_touching() {
        assert_eq!(crossings("M 0 0 L 10 10", "M 0 10 L 10 0"), [true]);
        assert_eq!(crossings(CLOSED_CIRCLE, "M -20 5 H 20"), [true, true]);
        // A line touching a circle, and two circles touching
        assert_eq!(crossings(CLOSED_CIRCLE, "M -20 10 H 20"), [false]);
        assert_eq!(
            crossings(
                CLOSED_CIRCLE,
                "M 30 0 A 10 10 0 0 1 10 0 A 10 10 0 0 1 30 0 Z"
            ),
            [false]
        );
    }

    #[test]
    fn a_curve_crosses_a_line_it_is_tangent_to_at_an_inflection() {
        // y = x³, which runs along the x axis at the origin
        let cubic = "M -10 -10 C -3.3333333333333335 10 3.3333333333333335 -10 10 10";
        let meets = measure(cubic).intersections(&measure("M -20 0 H 20"));
        assert!(!meets.is_empty());
        assert!(meets.iter().all(|x| x.crossing), "{meets:?}");
    }

    #[test]
    fn corners_touch_or_cross() {
        let line = "M -5 0 H 25";
        assert_eq!(crossings("M 0 10 L 10 0 L 20 10", line), [false]);
        assert_eq!(crossings("M 0 10 L 10 0 L 20 -10", line), [true]);
        // Through the corners of a square, its start among them
        assert_eq!(
            crossings("M 0 0 H 10 V 10 H 0 Z", "M -5 -5 L 15 15"),
            [true, true]
        );
        assert_eq!(
            crossings("M 0 0 H 10 V 10 H 0 Z", "M -5 5 L 0 0 L -5 -5"),
            [false]
        );
    }

    #[test]
    fn an_open_end_does_not_cross() {
        assert_eq!(crossings("M 0 0 L 10 0", "M 5 0 L 5 10"), [false]);
        assert_eq!(crossings("M 5 0 L 5 10", "M 0 0 L 10 0"), [false]);
    }
}
