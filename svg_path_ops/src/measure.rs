use std::borrow::Borrow;

use euclid::default::{Point2D, Vector2D};
use kurbo::common::GAUSS_LEGENDRE_COEFFS_16;
use kurbo::{
    Arc,
    CubicBez,
    Line,
    ParamCurve,
    ParamCurveArclen,
    ParamCurveDeriv,
    ParamCurveNearest,
    QuadBez,
    SvgArc,
    Vec2,
};
use svgtypes::PathSegment;

use crate::context::{segments_with_context, SegmentContext};

/// Absolute accuracy of the lengths a [`PathMeasure`] works with.
const ACCURACY: f64 = 1e-9;

/// Where a length along a path falls: the segment at `index` in the path, at
/// parameter `t` of it, from 0 at its start to 1 at its end.
///
/// For a curve, `t` is the curve's own parameter; for an arc, the fraction of
/// its angle; for a line or a close path, the fraction of its length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub index: usize,
    pub t: f64,
}

/// The point of a path nearest to another point, as found by
/// [`PathMeasure::nearest`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nearest {
    /// The nearest point on the path.
    pub point: Point2D<f64>,
    /// Its distance from the point asked about.
    pub distance: f64,
    /// Its length along the path, for [`PathMeasure::point_at`] and the
    /// other queries.
    pub length: f64,
    /// The segment it is on and the parameter within it.
    pub position: Position,
}

/// Measures lengths along a path and finds the point at a given length.
///
/// Segment lengths are computed once, when the measure is made, so queries
/// are cheap. Arcs are measured as the ellipse arcs they are, not as the
/// curves that approximate them. A close path counts as the line back to
/// its subpath's start.
///
/// Lengths outside the path are clamped to its start or end. A path that
/// draws nothing has a length of 0 and no points.
///
/// ```
/// use svg_path_ops::euclid::default::{Point2D, Vector2D};
/// use svg_path_ops::svgtypes::PathParser;
/// use svg_path_ops::{PathMeasure, Position};
///
/// let segments: Vec<_> = PathParser::from("M 0 0 L 30 40 h 50").collect::<Result<_, _>>()?;
/// let measure = PathMeasure::new(&segments);
///
/// assert_eq!(measure.total_length(), 100.0);
/// assert_eq!(measure.point_at(25.0), Some(Point2D::new(15.0, 20.0)));
/// assert_eq!(measure.tangent_at(75.0), Some(Vector2D::new(1.0, 0.0)));
/// assert_eq!(
///     measure.position_at(75.0),
///     Some(Position { index: 2, t: 0.5 })
/// );
/// # Ok::<(), svg_path_ops::svgtypes::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct PathMeasure {
    pieces: Vec<Piece>,
    total: f64,
}

/// A drawing segment, and where along the path it starts.
#[derive(Clone, Copy, Debug)]
struct Piece {
    index: usize,
    shape: PieceShape,
    start: f64,
    length: f64,
}

#[derive(Clone, Copy, Debug)]
enum PieceShape {
    Line(Line),
    Quadratic(QuadBez),
    Cubic(CubicBez),
    Arc(Arc),
}

impl PathMeasure {
    /// Measures `segments`.
    pub fn new(segments: impl IntoIterator<Item = impl Borrow<PathSegment>>) -> Self {
        let segments: Vec<PathSegment> = segments.into_iter().map(|s| *s.borrow()).collect();
        let mut pieces = Vec::new();
        let mut total = 0.0;
        for context in segments_with_context(&segments) {
            let Some(shape) = PieceShape::of(&context) else {
                continue;
            };
            let length = shape.length();
            pieces.push(Piece { index: context.index, shape, start: total, length });
            total += length;
        }
        PathMeasure { pieces, total }
    }

    /// The length of the whole path.
    pub fn total_length(&self) -> f64 {
        self.total
    }

    /// The segment and parameter at `length` along the path, or `None` when
    /// the path draws nothing.
    ///
    /// At the join of two segments this is the start of the second one, and
    /// at the end of the path the end of its last segment.
    pub fn position_at(&self, length: f64) -> Option<Position> {
        let (piece, t) = self.locate(length)?;
        Some(Position { index: self.pieces[piece].index, t })
    }

    /// The point at `length` along the path.
    pub fn point_at(&self, length: f64) -> Option<Point2D<f64>> {
        let (piece, t) = self.locate(length)?;
        let point = self.pieces[piece].shape.eval(t);
        Some(Point2D::new(point.x, point.y))
    }

    /// The direction the path is drawn in at `length`, as a unit vector, or
    /// `None` when the path draws nothing or only zero-length segments.
    pub fn tangent_at(&self, length: f64) -> Option<Vector2D<f64>> {
        let (piece, t) = self.locate(length)?;
        let direction = self.pieces[piece].shape.direction(t).or_else(|| {
            // A zero-length piece has no direction; use the nearest piece
            // before it that has one
            self.pieces[..=piece]
                .iter()
                .rev()
                .find_map(|piece| piece.shape.direction(1.0))
        })?;
        let unit = direction / direction.hypot();
        Some(Vector2D::new(unit.x, unit.y))
    }

    /// The point of the path nearest to `point`, or `None` when the path
    /// draws nothing. When several are equally near, the first along the
    /// path wins.
    ///
    /// ```
    /// use svg_path_ops::euclid::default::Point2D;
    /// use svg_path_ops::svgtypes::PathParser;
    /// use svg_path_ops::PathMeasure;
    ///
    /// let segments: Vec<_> = PathParser::from("M 0 0 L 10 0").collect::<Result<_, _>>()?;
    /// let nearest = PathMeasure::new(&segments)
    ///     .nearest(Point2D::new(4.0, 3.0))
    ///     .unwrap();
    ///
    /// assert_eq!(nearest.point, Point2D::new(4.0, 0.0));
    /// assert_eq!(nearest.distance, 3.0);
    /// assert_eq!(nearest.length, 4.0);
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn nearest(&self, point: Point2D<f64>) -> Option<Nearest> {
        let target = kurbo::Point::new(point.x, point.y);
        let (piece, t, distance_sq) = self
            .pieces
            .iter()
            .map(|piece| {
                let (t, distance_sq) = piece.shape.nearest(target);
                (piece, t, distance_sq)
            })
            .reduce(|best, next| if next.2 < best.2 { next } else { best })?;
        let on_path = piece.shape.eval(t);
        Some(Nearest {
            point: Point2D::new(on_path.x, on_path.y),
            distance: distance_sq.sqrt(),
            length: piece.start + piece.shape.length_to(t),
            position: Position { index: piece.index, t },
        })
    }

    /// Whether `point` is on the path's stroke when it is drawn `width`
    /// wide: within half the width of the path.
    ///
    /// Ends and corners count as round, whatever the stroke's line caps and
    /// joins; a path that draws nothing has no stroke.
    pub fn is_point_in_stroke(&self, point: Point2D<f64>, width: f64) -> bool {
        self.nearest(point)
            .is_some_and(|nearest| nearest.distance <= width / 2.0)
    }

    /// The index of the piece at `length` and the parameter within it.
    fn locate(&self, length: f64) -> Option<(usize, f64)> {
        let last = self.pieces.len().checked_sub(1)?;
        let length = if length.is_nan() {
            0.0
        } else {
            length.clamp(0.0, self.total)
        };
        if length >= self.total {
            return Some((last, 1.0));
        }
        // The last piece with length that starts at or before `length`
        let after = self.pieces.partition_point(|piece| piece.start <= length);
        let index = self.pieces[..after]
            .iter()
            .rposition(|piece| piece.length > 0.0)
            .unwrap_or(0);
        let piece = &self.pieces[index];
        let t = piece.shape.inv_arclen(length - piece.start, piece.length);
        Some((index, t))
    }
}

impl PieceShape {
    /// The shape a drawing segment draws, or `None` for a move or for an arc
    /// SVG leaves out because it ends where it starts.
    fn of(context: &SegmentContext<'_>) -> Option<Self> {
        let point = |p: Point2D<f64>| kurbo::Point::new(p.x, p.y);
        let (start, end) = (point(context.start), point(context.end));
        let absolute = |abs: bool, x: f64, y: f64| {
            if abs {
                kurbo::Point::new(x, y)
            } else {
                kurbo::Point::new(start.x + x, start.y + y)
            }
        };
        let implied = || point(context.implied_control.expect("smooth segment"));

        Some(match *context.segment {
            PathSegment::MoveTo { .. } => return None,
            PathSegment::LineTo { .. }
            | PathSegment::HorizontalLineTo { .. }
            | PathSegment::VerticalLineTo { .. }
            | PathSegment::ClosePath { .. } => PieceShape::Line(Line::new(start, end)),
            PathSegment::CurveTo { abs, x1, y1, x2, y2, .. } => PieceShape::Cubic(CubicBez::new(
                start,
                absolute(abs, x1, y1),
                absolute(abs, x2, y2),
                end,
            )),
            PathSegment::SmoothCurveTo { abs, x2, y2, .. } => {
                PieceShape::Cubic(CubicBez::new(start, implied(), absolute(abs, x2, y2), end))
            }
            PathSegment::Quadratic { abs, x1, y1, .. } => {
                PieceShape::Quadratic(QuadBez::new(start, absolute(abs, x1, y1), end))
            }
            PathSegment::SmoothQuadratic { .. } => {
                PieceShape::Quadratic(QuadBez::new(start, implied(), end))
            }
            PathSegment::EllipticalArc { rx, ry, x_axis_rotation, large_arc, sweep, .. } => {
                if start == end {
                    return None;
                }
                let arc = SvgArc {
                    from: start,
                    to: end,
                    radii: Vec2::new(rx, ry),
                    x_rotation: x_axis_rotation.to_radians(),
                    large_arc,
                    sweep,
                };
                // An arc with a zero radius is drawn as a line
                match Arc::from_svg_arc(&arc) {
                    Some(arc) => PieceShape::Arc(arc),
                    None => PieceShape::Line(Line::new(start, end)),
                }
            }
        })
    }

    fn eval(&self, t: f64) -> kurbo::Point {
        match self {
            PieceShape::Line(line) => line.eval(t),
            PieceShape::Quadratic(quad) => quad.eval(t),
            PieceShape::Cubic(cubic) => cubic.eval(t),
            PieceShape::Arc(arc) => arc.eval(t),
        }
    }

    fn length(&self) -> f64 {
        match self {
            PieceShape::Line(line) => line.arclen(ACCURACY),
            PieceShape::Quadratic(quad) => quad.arclen(ACCURACY),
            PieceShape::Cubic(cubic) => cubic.arclen(ACCURACY),
            PieceShape::Arc(arc) => arc_length(arc, 0.0, 1.0),
        }
    }

    /// The parameter at which the length from the start is `length`, given
    /// the piece's whole length.
    fn inv_arclen(&self, length: f64, whole: f64) -> f64 {
        if whole <= 0.0 {
            return 0.0;
        }
        match self {
            PieceShape::Line(_) => length / whole,
            PieceShape::Quadratic(quad) => quad.inv_arclen(length, ACCURACY),
            PieceShape::Cubic(cubic) => cubic.inv_arclen(length, ACCURACY),
            PieceShape::Arc(arc) => arc_inv_length(arc, length, whole),
        }
    }

    /// The parameter of the piece's point nearest to `target`, and the
    /// squared distance to it.
    fn nearest(&self, target: kurbo::Point) -> (f64, f64) {
        let nearest = match self {
            PieceShape::Line(line) => line.nearest(target, ACCURACY),
            PieceShape::Quadratic(quad) => quad.nearest(target, ACCURACY),
            PieceShape::Cubic(cubic) => cubic.nearest(target, ACCURACY),
            PieceShape::Arc(arc) => return arc_nearest(arc, target),
        };
        (nearest.t, nearest.distance_sq)
    }

    /// The length from the piece's start to parameter `t`.
    fn length_to(&self, t: f64) -> f64 {
        match self {
            PieceShape::Line(line) => line.arclen(ACCURACY) * t,
            PieceShape::Quadratic(quad) => quad.subsegment(0.0..t).arclen(ACCURACY),
            PieceShape::Cubic(cubic) => cubic.subsegment(0.0..t).arclen(ACCURACY),
            PieceShape::Arc(arc) => arc_length(arc, 0.0, t),
        }
    }

    /// The derivative at `t`, or `None` where the piece does not move.
    fn direction(&self, t: f64) -> Option<Vec2> {
        let derivative = |t: f64| match self {
            PieceShape::Line(line) => line.p1 - line.p0,
            PieceShape::Quadratic(quad) => quad.deriv().eval(t).to_vec2(),
            PieceShape::Cubic(cubic) => cubic.deriv().eval(t).to_vec2(),
            PieceShape::Arc(arc) => arc_derivative(arc, t),
        };
        let direction = derivative(t);
        if direction.hypot() > 1e-12 {
            return Some(direction);
        }
        // A curve whose control point sits on its end has no derivative
        // there; the direction just inside the curve is the tangent
        let inside = if t < 0.5 { t + 1e-6 } else { t - 1e-6 };
        let direction = derivative(inside);
        (direction.hypot() > 1e-12).then_some(direction)
    }
}

/// How fast the arc's point moves per radian at angle `theta`.
fn arc_speed(arc: &Arc, theta: f64) -> f64 {
    let (sin, cos) = theta.sin_cos();
    (arc.radii.x * sin).hypot(arc.radii.y * cos)
}

/// The length of the arc between parameters `t0` and `t1`.
fn arc_length(arc: &Arc, t0: f64, t1: f64) -> f64 {
    let a = arc.start_angle + arc.sweep_angle * t0;
    let b = arc.start_angle + arc.sweep_angle * t1;
    integrate(|theta| arc_speed(arc, theta), a.min(b), a.max(b), 0)
}

/// Integrates `f` from `a` to `b` with 16-point Gauss–Legendre quadrature,
/// halving the interval until the halves agree with the whole.
fn integrate(f: impl Fn(f64) -> f64 + Copy, a: f64, b: f64, depth: u32) -> f64 {
    let gauss = |a: f64, b: f64| {
        let (half, middle) = ((b - a) / 2.0, (a + b) / 2.0);
        half * GAUSS_LEGENDRE_COEFFS_16
            .iter()
            .map(|&(weight, x)| weight * f(middle + half * x))
            .sum::<f64>()
    };
    let middle = (a + b) / 2.0;
    let (whole, halves) = (gauss(a, b), gauss(a, middle) + gauss(middle, b));
    if (whole - halves).abs() <= ACCURACY || depth >= 20 {
        halves
    } else {
        integrate(f, a, middle, depth + 1) + integrate(f, middle, b, depth + 1)
    }
}

/// The arc's parameter at `length` from its start, given its whole length.
fn arc_inv_length(arc: &Arc, length: f64, whole: f64) -> f64 {
    let (mut low, mut high) = (0.0, 1.0);
    let mut t = length / whole;
    for _ in 0..100 {
        let error = arc_length(arc, 0.0, t) - length;
        if error.abs() <= ACCURACY {
            break;
        }
        if error > 0.0 {
            high = t;
        } else {
            low = t;
        }
        // Newton's step, kept inside the bracket; halving when it leaves
        let speed = arc_speed(arc, arc.start_angle + arc.sweep_angle * t) * arc.sweep_angle.abs();
        let next = t - error / speed;
        t = if speed > 0.0 && next > low && next < high {
            next
        } else {
            (low + high) / 2.0
        };
    }
    t
}

/// The arc's parameter nearest to `target`, and the squared distance to it:
/// the best of evenly spaced samples, narrowed down by golden-section search
/// between its neighbours.
fn arc_nearest(arc: &Arc, target: kurbo::Point) -> (f64, f64) {
    const SAMPLES: u32 = 64;
    let distance_sq = |t: f64| (arc.eval(t) - target).hypot2();
    let best = (0..=SAMPLES)
        .map(|k| f64::from(k) / f64::from(SAMPLES))
        .min_by(|a, b| distance_sq(*a).total_cmp(&distance_sq(*b)))
        .expect("samples");

    let step = 1.0 / f64::from(SAMPLES);
    let (mut low, mut high) = ((best - step).max(0.0), (best + step).min(1.0));
    let ratio = (5f64.sqrt() - 1.0) / 2.0;
    while high - low > 1e-12 {
        let (a, b) = (high - ratio * (high - low), low + ratio * (high - low));
        if distance_sq(a) < distance_sq(b) {
            high = b;
        } else {
            low = a;
        }
    }
    let t = (low + high) / 2.0;
    (t, distance_sq(t))
}

/// The derivative of the arc's point by its parameter.
fn arc_derivative(arc: &Arc, t: f64) -> Vec2 {
    let theta = arc.start_angle + arc.sweep_angle * t;
    let (sin, cos) = theta.sin_cos();
    let local = Vec2::new(-arc.radii.x * sin, arc.radii.y * cos);
    let (rsin, rcos) = arc.x_rotation.sin_cos();
    Vec2::new(
        local.x * rcos - local.y * rsin,
        local.x * rsin + local.y * rcos,
    ) * arc.sweep_angle
}

#[cfg(test)]
mod test {
    use std::f64::consts::PI;

    use euclid::default::{Point2D, Vector2D};
    use svgtypes::PathParser;

    use super::{PathMeasure, Position};
    use crate::PathSegment;

    fn measure(path: &str) -> PathMeasure {
        let segments: Vec<PathSegment> = PathParser::from(path).map(Result::unwrap).collect();
        PathMeasure::new(&segments)
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn close_point(a: Point2D<f64>, b: Point2D<f64>) -> bool {
        close(a.x, b.x) && close(a.y, b.y)
    }

    fn close_vector(a: Vector2D<f64>, b: Vector2D<f64>) -> bool {
        close(a.x, b.x) && close(a.y, b.y)
    }

    #[test]
    fn measures_lines() {
        let m = measure("M 0 0 L 3 4");
        assert_eq!(m.total_length(), 5.0);
        assert_eq!(m.point_at(2.5), Some(Point2D::new(1.5, 2.0)));
        assert!(close_vector(
            m.tangent_at(2.5).unwrap(),
            Vector2D::new(0.6, 0.8)
        ));
        assert_eq!(m.position_at(2.5), Some(Position { index: 1, t: 0.5 }));
    }

    #[test]
    fn close_path_counts_as_the_line_back() {
        let m = measure("M 0 0 h 10 v 10 z");
        assert!(close(m.total_length(), 20.0 + 200f64.sqrt()));
        let back = m.position_at(m.total_length() - 1.0).unwrap();
        assert_eq!(back.index, 3);
    }

    #[test]
    fn lengths_outside_the_path_are_clamped() {
        let m = measure("M 1 1 L 11 1");
        assert_eq!(m.point_at(-5.0), Some(Point2D::new(1.0, 1.0)));
        assert_eq!(m.point_at(100.0), Some(Point2D::new(11.0, 1.0)));
        assert_eq!(m.point_at(f64::NAN), Some(Point2D::new(1.0, 1.0)));
    }

    #[test]
    fn measures_circular_arcs() {
        // A half circle of radius 10 through (0, 10)
        let m = measure("M 10 0 A 10 10 0 0 1 -10 0");
        assert!(close(m.total_length(), 10.0 * PI));
        assert!(close_point(
            m.point_at(5.0 * PI).unwrap(),
            Point2D::new(0.0, 10.0)
        ));
        assert!(close_vector(
            m.tangent_at(5.0 * PI).unwrap(),
            Vector2D::new(-1.0, 0.0)
        ));
        let middle = m.position_at(5.0 * PI).unwrap();
        assert_eq!(middle.index, 1);
        assert!(close(middle.t, 0.5));
    }

    #[test]
    fn measures_elliptical_arcs() {
        // The perimeter of an ellipse with radii 20 and 10, from its
        // complete elliptic integral: 4 * 20 * E(sqrt(0.75))
        let m = measure("M 20 0 A 20 10 0 0 1 -20 0 A 20 10 0 0 1 20 0");
        assert!(close(m.total_length(), 96.884_482_205_5));
        // Rotated, with its ends turned along, it has the same length
        let (x, y) = (
            20.0 * 30f64.to_radians().cos(),
            20.0 * 30f64.to_radians().sin(),
        );
        let rotated = measure(&format!(
            "M {x} {y} A 20 10 30 0 1 {} {} A 20 10 30 0 1 {x} {y}",
            -x, -y
        ));
        assert!(close(rotated.total_length(), m.total_length()));
    }

    #[test]
    fn arc_points_are_at_their_lengths() {
        let m = measure("M 20 0 A 20 10 25 0 1 -20 0");
        // Walking half the length and measuring back gives the same length
        let half = m.total_length() / 2.0;
        let point = m.point_at(half).unwrap();
        let t = m.position_at(half).unwrap().t;
        assert!(close(super::arc_length(arc_of(&m), 0.0, t), half));
        assert!(point.x.is_finite() && point.y.is_finite());
    }

    fn arc_of(m: &PathMeasure) -> &kurbo::Arc {
        match &m.pieces[0].shape {
            super::PieceShape::Arc(arc) => arc,
            _ => panic!("not an arc"),
        }
    }

    #[test]
    fn measures_curves_with_implied_controls() {
        // The S mirrors the C, so both halves have the same length
        let m = measure("M 0 0 C 0 10 10 10 10 0 S 20 -10 20 0");
        let half = measure("M 0 0 C 0 10 10 10 10 0").total_length();
        assert!(close(m.total_length(), 2.0 * half));
        assert!(close_point(
            m.point_at(half).unwrap(),
            Point2D::new(10.0, 0.0)
        ));

        let q = measure("M 0 0 Q 10 10 20 0 T 40 0");
        let q_half = measure("M 0 0 Q 10 10 20 0").total_length();
        assert!(close(q.total_length(), 2.0 * q_half));
    }

    #[test]
    fn curve_points_are_at_their_lengths() {
        let m = measure("M 0 0 C 30 80 60 -40 100 20");
        for k in 1..10 {
            let length = m.total_length() * f64::from(k) / 10.0;
            let t = m.position_at(length).unwrap().t;
            let partial = measure(&format!(
                "M 0 0 {}",
                // The curve cut at t, from kurbo's subsegment
                {
                    let cubic = kurbo::CubicBez::new(
                        (0.0, 0.0),
                        (30.0, 80.0),
                        (60.0, -40.0),
                        (100.0, 20.0),
                    );
                    let part = kurbo::ParamCurve::subsegment(&cubic, 0.0..t);
                    format!(
                        "C {} {} {} {} {} {}",
                        part.p1.x, part.p1.y, part.p2.x, part.p2.y, part.p3.x, part.p3.y
                    )
                }
            ));
            assert!(close(partial.total_length(), length));
        }
    }

    #[test]
    fn tangent_where_a_control_point_sits_on_the_end() {
        let m = measure("M 0 0 C 0 0 10 10 10 0");
        let tangent = m.tangent_at(0.0).unwrap();
        assert!(close(tangent.x, 1.0 / 2f64.sqrt()) && close(tangent.y, 1.0 / 2f64.sqrt()));
    }

    #[test]
    fn a_join_belongs_to_the_segment_that_starts_there() {
        let m = measure("M 0 0 L 10 0 L 10 10");
        assert_eq!(m.position_at(10.0), Some(Position { index: 2, t: 0.0 }));
        assert_eq!(m.tangent_at(10.0), Some(Vector2D::new(0.0, 1.0)));
        // The end of the path is the end of its last segment
        assert_eq!(m.position_at(20.0), Some(Position { index: 2, t: 1.0 }));
    }

    #[test]
    fn subpaths_follow_each_other() {
        let m = measure("M 0 0 L 10 0 M 100 100 L 110 100");
        assert_eq!(m.total_length(), 20.0);
        assert_eq!(m.point_at(9.0), Some(Point2D::new(9.0, 0.0)));
        assert_eq!(m.point_at(15.0), Some(Point2D::new(105.0, 100.0)));
    }

    #[test]
    fn zero_length_segments_take_the_tangent_before_them() {
        let m = measure("M 0 0 L 10 0 L 10 0");
        assert_eq!(m.tangent_at(10.0), Some(Vector2D::new(1.0, 0.0)));
    }

    #[test]
    fn arcs_that_draw_nothing_or_are_lines() {
        // SVG leaves out an arc that ends where it starts
        assert_eq!(measure("M 5 5 A 10 10 0 0 1 5 5").total_length(), 0.0);
        // and draws one with a zero radius as a line
        assert_eq!(measure("M 0 0 A 0 10 0 0 1 10 0").total_length(), 10.0);
    }

    #[test]
    fn nearest_point_on_a_line() {
        let m = measure("M 0 0 L 10 0");
        let nearest = m.nearest(Point2D::new(5.0, 5.0)).unwrap();
        assert_eq!(nearest.point, Point2D::new(5.0, 0.0));
        assert_eq!(nearest.distance, 5.0);
        assert_eq!(nearest.length, 5.0);
        assert_eq!(nearest.position, Position { index: 1, t: 0.5 });
        // Past the end, the end is nearest
        let past = m.nearest(Point2D::new(15.0, 0.0)).unwrap();
        assert_eq!((past.point, past.distance), (Point2D::new(10.0, 0.0), 5.0));
    }

    #[test]
    fn nearest_point_on_an_arc() {
        // A half circle of radius 10 through (0, 10)
        let m = measure("M 10 0 A 10 10 0 0 1 -10 0");
        let nearest = m.nearest(Point2D::new(0.0, 20.0)).unwrap();
        assert!(close_point(nearest.point, Point2D::new(0.0, 10.0)));
        assert!(close(nearest.distance, 10.0));
        assert!(close(nearest.length, 5.0 * PI));
        // Seen from above, the half circle's ends are nearest
        let above = m.nearest(Point2D::new(3.0, -4.0)).unwrap();
        assert!(close_point(above.point, Point2D::new(10.0, 0.0)));
    }

    #[test]
    fn nearest_point_on_a_curve_beats_every_sample() {
        let m = measure("M 0 0 C 30 80 60 -40 100 20 S 150 60 170 0");
        for target in [
            Point2D::new(40.0, 40.0),
            Point2D::new(120.0, -10.0),
            Point2D::new(90.0, 10.0),
        ] {
            let nearest = m.nearest(target).unwrap();
            let sampled = (0..=2000)
                .map(|k| {
                    m.point_at(m.total_length() * f64::from(k) / 2000.0)
                        .unwrap()
                })
                .map(|p| (p - target).length())
                .fold(f64::INFINITY, f64::min);
            assert!(nearest.distance <= sampled + 1e-9);
            // Its length leads back to the same point
            assert!(close_point(
                m.point_at(nearest.length).unwrap(),
                nearest.point
            ));
        }
    }

    #[test]
    fn nearest_point_across_subpaths() {
        let m = measure("M 0 0 L 10 0 M 0 100 L 10 100");
        let nearest = m.nearest(Point2D::new(5.0, 90.0)).unwrap();
        assert_eq!(nearest.point, Point2D::new(5.0, 100.0));
        assert_eq!(nearest.position.index, 3);
        assert_eq!(nearest.length, 15.0);
    }

    #[test]
    fn point_in_stroke_is_within_half_the_width() {
        let m = measure("M 0 0 L 10 0");
        assert!(m.is_point_in_stroke(Point2D::new(5.0, 0.0), 0.0));
        assert!(m.is_point_in_stroke(Point2D::new(5.0, 1.0), 2.0));
        assert!(!m.is_point_in_stroke(Point2D::new(5.0, 1.1), 2.0));
        // Ends count as round
        assert!(m.is_point_in_stroke(Point2D::new(10.6, 0.6), 2.0));
    }

    #[test]
    fn a_path_that_draws_nothing_has_no_points() {
        let m = measure("M 5 5");
        assert_eq!(m.total_length(), 0.0);
        assert_eq!(m.point_at(0.0), None);
        assert_eq!(m.tangent_at(0.0), None);
        assert_eq!(measure("").position_at(0.0), None);
        assert_eq!(m.nearest(Point2D::new(5.0, 5.0)), None);
        assert!(!m.is_point_in_stroke(Point2D::new(5.0, 5.0), 10.0));
    }
}
