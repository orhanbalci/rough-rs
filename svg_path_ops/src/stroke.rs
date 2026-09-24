use euclid::default::{Box2D, Point2D};
use kurbo::{Point, Vec2};

use crate::measure::{PathMeasure, Piece, PieceShape};

/// How the ends of open subpaths are drawn, as SVG's `stroke-linecap`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineCap {
    /// The stroke stops square at the end. SVG's default.
    #[default]
    Butt,
    /// A half circle beyond the end.
    Round,
    /// Half a square beyond the end.
    Square,
}

/// How the corners where segments meet are drawn, as SVG's
/// `stroke-linejoin`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineJoin {
    /// The outer edges run on until they meet, unless that is further than
    /// the miter limit allows, when the corner is beveled. SVG's default.
    #[default]
    Miter,
    /// A circular arc round the corner.
    Round,
    /// A straight line across the corner.
    Bevel,
}

/// How a path is stroked, as SVG's stroke properties.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrokeStyle {
    /// `stroke-width`
    pub width: f64,
    /// `stroke-linecap`
    pub cap: LineCap,
    /// `stroke-linejoin`
    pub join: LineJoin,
    /// `stroke-miterlimit`: how many times the width a miter may reach
    /// from the corner before it is beveled
    pub miter_limit: f64,
}

impl Default for StrokeStyle {
    /// SVG's defaults: width 1, butt caps and miter joins with a limit of 4.
    fn default() -> Self {
        StrokeStyle {
            width: 1.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            miter_limit: 4.0,
        }
    }
}

/// Samples of the curvature along a curve or arc, looking for where the
/// stroke's inner edge turns back on itself.
const CURVATURE_SAMPLES: usize = 64;

impl PathMeasure {
    /// The smallest box around the path, or `None` when it draws nothing.
    ///
    /// Curves and arcs are bounded exactly, at their ends and where they run
    /// vertically or horizontally, not by their control points.
    ///
    /// ```
    /// use svg_path_ops::euclid::default::{Box2D, Point2D};
    /// use svg_path_ops::pt::PathTransformer;
    ///
    /// let bump = PathTransformer::parse("M 0 0 C 0 -20 20 -20 20 0")?.measure();
    /// assert_eq!(
    ///     bump.bounds(),
    ///     Some(Box2D::new(
    ///         Point2D::new(0.0, -15.0),
    ///         Point2D::new(20.0, 0.0)
    ///     ))
    /// );
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn bounds(&self) -> Option<Box2D<f64>> {
        let mut bounds = Bounds::default();
        for piece in &self.pieces {
            bounds.add(piece.from);
            bounds.add(piece.to);
            for t in piece.shape.axis_turns() {
                bounds.add(piece.shape.eval(t));
            }
        }
        bounds.into_box()
    }

    /// The smallest box around the path stroked with `style`, caps, joins
    /// and all, or `None` when the stroke draws nothing. Dashes are not
    /// taken into account, so a dashed stroke lies within the box.
    ///
    /// As SVG draws them, a subpath that draws nothing but a point has a
    /// round or square cap there, and one that only moves draws nothing.
    ///
    /// # Algorithm
    ///
    /// Along a segment the stroke covers the lines across the path, half
    /// the width to each side, so its edges are the points `p ± w/2 n` for
    /// the points `p` of the segment with their normals `n`. The stroke
    /// reaches furthest in a direction `u` where `p·u + w/2 |n·u|` is
    /// largest. Its derivative is the speed times `(t·u)(1 − κ w/2)`, with
    /// `t` the tangent and `κ` the curvature, so along the segment the
    /// largest value is at an end, where the segment runs across `u`
    /// (`t·u = 0`), or where the radius of curvature is half the width
    /// (`κ w/2 = 1`), where the inner edge turns back on itself. The first
    /// kind is found exactly, as for [`bounds`](Self::bounds); the second
    /// by sampling the curvature and bisecting where it crosses `2/w`. The
    /// edge points at all of these bound the segments' part of the stroke.
    ///
    /// Joins and caps add what reaches past the segments: a round join or
    /// cap reaches half the width in the directions of its arc, a miter
    /// join reaches its tip when that is within the miter limit, and a
    /// square cap reaches its two outer corners. A bevel join and a butt
    /// cap add nothing.
    ///
    /// ```
    /// use svg_path_ops::euclid::default::{Box2D, Point2D};
    /// use svg_path_ops::pt::PathTransformer;
    /// use svg_path_ops::{LineCap, StrokeStyle};
    ///
    /// let line = PathTransformer::parse("M 0 0 H 20")?.measure();
    /// let style = StrokeStyle {
    ///     width: 4.0,
    ///     cap: LineCap::Square,
    ///     ..StrokeStyle::default()
    /// };
    /// assert_eq!(
    ///     line.stroke_bounds(&style),
    ///     Some(Box2D::new(
    ///         Point2D::new(-2.0, -2.0),
    ///         Point2D::new(22.0, 2.0)
    ///     ))
    /// );
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn stroke_bounds(&self, style: &StrokeStyle) -> Option<Box2D<f64>> {
        let half = style.width.max(0.0) / 2.0;
        let mut bounds = Bounds::default();
        for subpath in self.subpaths() {
            let drawing: Vec<&Piece> = subpath.iter().filter(|p| p.length > 0.0).collect();
            let closed = subpath.last().is_some_and(|piece| piece.closes);
            let (Some(first), Some(last)) = (drawing.first(), drawing.last()) else {
                // A point: a round cap is a dot, a square one a square
                // along the x axis
                let at = subpath[0].from;
                if style.cap != LineCap::Butt {
                    bounds.add(at - Vec2::new(half, half));
                    bounds.add(at + Vec2::new(half, half));
                }
                continue;
            };

            for piece in &drawing {
                for t in edge_params(&piece.shape, half) {
                    let (p, n) = (piece.shape.eval(t), normal(&piece.shape, t));
                    bounds.add(p + n * half);
                    bounds.add(p - n * half);
                }
            }
            for pair in drawing.windows(2) {
                join(
                    &mut bounds,
                    pair[0].to,
                    direction(pair[0], 1.0),
                    direction(pair[1], 0.0),
                    style,
                );
            }
            if closed {
                join(
                    &mut bounds,
                    first.from,
                    direction(last, 1.0),
                    direction(first, 0.0),
                    style,
                );
            } else {
                cap(&mut bounds, first.from, -direction(first, 0.0), style);
                cap(&mut bounds, last.to, direction(last, 1.0), style);
            }
        }
        bounds.into_box()
    }
}

/// A box grown point by point.
#[derive(Default)]
struct Bounds(Option<(Point, Point)>);

impl Bounds {
    fn add(&mut self, p: Point) {
        self.0 = Some(match self.0 {
            None => (p, p),
            Some((min, max)) => (
                Point::new(min.x.min(p.x), min.y.min(p.y)),
                Point::new(max.x.max(p.x), max.y.max(p.y)),
            ),
        });
    }

    fn into_box(self) -> Option<Box2D<f64>> {
        self.0
            .map(|(min, max)| Box2D::new(Point2D::new(min.x, min.y), Point2D::new(max.x, max.y)))
    }
}

/// The unit direction of `piece` at `t`, or zero where it does not move.
fn direction(piece: &Piece, t: f64) -> Vec2 {
    unit(piece.shape.direction(t))
}

/// The unit normal of `shape` at `t`: its direction turned a quarter turn.
fn normal(shape: &PieceShape, t: f64) -> Vec2 {
    let d = unit(shape.direction(t));
    Vec2::new(-d.y, d.x)
}

fn unit(v: Option<Vec2>) -> Vec2 {
    match v {
        Some(v) if v.hypot() > 0.0 => v / v.hypot(),
        _ => Vec2::ZERO,
    }
}

/// The parameters where the edges of the stroke of `shape`, `half` its
/// width to each side, may reach furthest in x or y: its ends, where it
/// runs vertically or horizontally, and where its radius of curvature is
/// `half`.
fn edge_params(shape: &PieceShape, half: f64) -> Vec<f64> {
    let mut params = vec![0.0, 1.0];
    params.extend(shape.axis_turns());
    if matches!(shape, PieceShape::Line(_)) || half == 0.0 {
        return params;
    }
    // Where |κ| − 1/half changes sign, found between samples by bisection
    let excess = |t: f64| shape.curvature(t).map(|k| k.abs() - 1.0 / half);
    let mut before = (0.0, excess(0.0));
    for k in 1..=CURVATURE_SAMPLES {
        let t = k as f64 / CURVATURE_SAMPLES as f64;
        let now = (t, excess(t));
        if let ((mut lo, Some(a)), (mut hi, Some(b))) = (before, now) {
            if (a < 0.0) != (b < 0.0) {
                for _ in 0..60 {
                    let mid = (lo + hi) / 2.0;
                    match excess(mid) {
                        Some(m) if (m < 0.0) == (a < 0.0) => lo = mid,
                        _ => hi = mid,
                    }
                }
                params.push((lo + hi) / 2.0);
            }
        }
        before = now;
    }
    params
}

/// Adds what a join at `at`, from direction `into` to direction `out`,
/// reaches past the segments' edges.
fn join(bounds: &mut Bounds, at: Point, into: Vec2, out: Vec2, style: &StrokeStyle) {
    let half = style.width.max(0.0) / 2.0;
    let turn = into.cross(out).atan2(into.dot(out));
    if turn == 0.0 || half == 0.0 {
        return;
    }
    // The outer side is the one the path turns away from
    let side = -turn.signum();
    let (n_into, n_out) = (
        Vec2::new(-into.y, into.x) * side,
        Vec2::new(-out.y, out.x) * side,
    );
    match style.join {
        LineJoin::Bevel => {}
        LineJoin::Round => arc_extremes(bounds, at, half, n_into, turn),
        LineJoin::Miter => {
            // The outer edges meet at 1 / cos(turn / 2) half widths along
            // the bisector of the normals
            let cos = (turn / 2.0).cos();
            if cos > 0.0 && 1.0 / cos <= style.miter_limit {
                let bisector = unit(Some(n_into + n_out));
                bounds.add(at + bisector * (half / cos));
            }
        }
    }
}

/// Adds what a cap at `at`, where the path leaves in direction `outward`,
/// reaches past the segment's edges.
fn cap(bounds: &mut Bounds, at: Point, outward: Vec2, style: &StrokeStyle) {
    let half = style.width.max(0.0) / 2.0;
    let side = Vec2::new(-outward.y, outward.x);
    match style.cap {
        LineCap::Butt => {}
        // Half a turn from one side, through the outward direction
        LineCap::Round => arc_extremes(bounds, at, half, -side, std::f64::consts::PI),
        LineCap::Square => {
            bounds.add(at + (outward + side) * half);
            bounds.add(at + (outward - side) * half);
        }
    }
}

/// Adds the points furthest in x and y of the circular arc about `center`
/// of radius `radius`, from direction `from` turning by `sweep` radians.
fn arc_extremes(bounds: &mut Bounds, center: Point, radius: f64, from: Vec2, sweep: f64) {
    let start = from.y.atan2(from.x);
    for axis in [
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(-1.0, 0.0),
        Vec2::new(0.0, -1.0),
    ] {
        // How far round from `from` the axis direction is, the way the arc
        // turns
        let angle = axis.y.atan2(axis.x) - start;
        let along = (angle * sweep.signum()).rem_euclid(std::f64::consts::TAU);
        if along <= sweep.abs() {
            bounds.add(center + axis * radius);
        }
    }
}

#[cfg(test)]
mod test {
    use euclid::default::{Box2D, Point2D};
    use kurbo::{CubicBez, ParamCurve, ParamCurveDeriv, Point, Vec2};
    use svgtypes::{PathParser, PathSegment};

    use super::{LineCap, LineJoin, StrokeStyle};
    use crate::PathMeasure;

    fn measure(path: &str) -> PathMeasure {
        let segments: Vec<PathSegment> = PathParser::from(path).collect::<Result<_, _>>().unwrap();
        PathMeasure::new(segments)
    }

    fn close(a: Box2D<f64>, b: Box2D<f64>, tolerance: f64) -> bool {
        (a.min - b.min).length() <= tolerance && (a.max - b.max).length() <= tolerance
    }

    fn boxed(x0: f64, y0: f64, x1: f64, y1: f64) -> Box2D<f64> {
        Box2D::new(Point2D::new(x0, y0), Point2D::new(x1, y1))
    }

    fn style(width: f64, cap: LineCap, join: LineJoin, miter_limit: f64) -> StrokeStyle {
        StrokeStyle { width, cap, join, miter_limit }
    }

    #[test]
    fn bounds_are_exact() {
        let circle = measure("M 10 0 A 10 10 0 0 1 -10 0 A 10 10 0 0 1 10 0");
        assert!(close(
            circle.bounds().unwrap(),
            boxed(-10.0, -10.0, 10.0, 10.0),
            1e-12
        ));
        let two = measure("M 0 0 L 1 1 M 5 -2 Q 6 2 7 -2");
        assert!(close(
            two.bounds().unwrap(),
            boxed(0.0, -2.0, 7.0, 1.0),
            1e-12
        ));
        assert_eq!(measure("").bounds(), None);
        assert_eq!(measure("M 1 1").bounds(), None);
    }

    #[test]
    fn caps() {
        let line = measure("M 0 0 H 20");
        let at = |cap| {
            line.stroke_bounds(&style(4.0, cap, LineJoin::Miter, 4.0))
                .unwrap()
        };
        assert!(close(at(LineCap::Butt), boxed(0.0, -2.0, 20.0, 2.0), 1e-12));
        assert!(close(
            at(LineCap::Round),
            boxed(-2.0, -2.0, 22.0, 2.0),
            1e-12
        ));
        assert!(close(
            at(LineCap::Square),
            boxed(-2.0, -2.0, 22.0, 2.0),
            1e-12
        ));

        // A diagonal line: a square cap reaches its corners, a round one
        // half the width straight out
        let diagonal = measure("M 0 0 L 10 10");
        let square = diagonal.stroke_bounds(&style(2.0, LineCap::Square, LineJoin::Miter, 4.0));
        let reach = 2f64.sqrt();
        assert!(close(
            square.unwrap(),
            boxed(-reach, -reach, 10.0 + reach, 10.0 + reach),
            1e-12
        ));
        let round = diagonal.stroke_bounds(&style(2.0, LineCap::Round, LineJoin::Miter, 4.0));
        assert!(close(round.unwrap(), boxed(-1.0, -1.0, 11.0, 11.0), 1e-12));
    }

    #[test]
    fn joins() {
        // A sharp turn at 10 2: the miter reaches about 5.1 half widths out
        let sharp = measure("M 0 0 L 10 2 L 0 4");
        let right = |join, limit| {
            sharp
                .stroke_bounds(&style(2.0, LineCap::Butt, join, limit))
                .unwrap()
                .max
                .x
        };
        let miter = 1.0 / (2f64 / 10f64.hypot(2.0)).asin().sin().abs();
        assert!((right(LineJoin::Miter, 10.0) - (10.0 + miter)).abs() < 1e-9);
        // Past the limit, and beveled, the corner stops short of the round
        assert!(right(LineJoin::Miter, 4.0) < 11.0);
        assert!(right(LineJoin::Bevel, 10.0) < 11.0);
        assert!((right(LineJoin::Round, 10.0) - 11.0).abs() < 1e-12);
    }

    #[test]
    fn a_closed_subpath_joins_at_its_start() {
        let diamond = measure("M 10 0 L 20 10 L 10 20 L 0 10 Z");
        let reach = 2f64.sqrt();
        let miter = diamond.stroke_bounds(&style(2.0, LineCap::Round, LineJoin::Miter, 4.0));
        assert!(close(
            miter.unwrap(),
            boxed(-reach, -reach, 20.0 + reach, 20.0 + reach),
            1e-12
        ));
        let bevel = diamond.stroke_bounds(&style(2.0, LineCap::Round, LineJoin::Bevel, 4.0));
        let edge = 0.5f64.sqrt();
        assert!(close(
            bevel.unwrap(),
            boxed(-edge, -edge, 20.0 + edge, 20.0 + edge),
            1e-12
        ));
    }

    #[test]
    fn a_point_has_a_cap_of_its_own() {
        let dot = measure("M 5 5 Z");
        let round = dot.stroke_bounds(&style(4.0, LineCap::Round, LineJoin::Miter, 4.0));
        assert!(close(round.unwrap(), boxed(3.0, 3.0, 7.0, 7.0), 1e-12));
        assert_eq!(dot.stroke_bounds(&StrokeStyle::default()), None);
        assert_eq!(
            measure("M 5 5").stroke_bounds(&style(4.0, LineCap::Round, LineJoin::Round, 4.0)),
            None
        );
    }

    /// The stroke edges of `curve` sampled finely by parameter.
    fn sampled(curve: CubicBez, half: f64) -> Box2D<f64> {
        let (mut min, mut max) = (
            Point::new(f64::MAX, f64::MAX),
            Point::new(f64::MIN, f64::MIN),
        );
        let n = 200_000;
        for k in 0..=n {
            let t = f64::from(k) / f64::from(n);
            let v = curve.deriv().eval(t).to_vec2();
            if v.hypot() == 0.0 {
                continue;
            }
            let normal = Vec2::new(-v.y, v.x) / v.hypot();
            for p in [curve.eval(t) + normal * half, curve.eval(t) - normal * half] {
                min = Point::new(min.x.min(p.x), min.y.min(p.y));
                max = Point::new(max.x.max(p.x), max.y.max(p.y));
            }
        }
        boxed(min.x, min.y, max.x, max.y)
    }

    #[test]
    fn curves_reach_where_their_inner_edge_turns_back() {
        for (c, width) in [
            // Gentle, and turning tighter than half the width
            ([(0.0, 0.0), (10.0, -20.0), (30.0, -20.0), (40.0, 0.0)], 6.0),
            (
                [(0.0, 0.0), (10.0, -20.0), (30.0, -20.0), (40.0, 0.0)],
                50.0,
            ),
            // Nearly a cusp, and a loop
            ([(0.0, 0.0), (30.0, 10.0), (0.1, 10.0), (30.0, 0.0)], 20.0),
            ([(0.0, 0.0), (40.0, 30.0), (-10.0, 30.0), (30.0, 0.0)], 12.0),
        ] {
            let points = c.map(|(x, y)| Point::new(x, y));
            let curve = CubicBez::new(points[0], points[1], points[2], points[3]);
            let path = format!(
                "M {} {} C {} {} {} {} {} {}",
                c[0].0, c[0].1, c[1].0, c[1].1, c[2].0, c[2].1, c[3].0, c[3].1
            );
            let bounds = measure(&path)
                .stroke_bounds(&style(width, LineCap::Butt, LineJoin::Bevel, 4.0))
                .unwrap();
            let expected = sampled(curve, width / 2.0);
            assert!(
                close(bounds, expected, 1e-6),
                "{path}: {bounds:?} {expected:?}"
            );
        }
    }
}
