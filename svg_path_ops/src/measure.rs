use std::borrow::Borrow;

use euclid::default::{Point2D, Vector2D};
use kurbo::common::GAUSS_LEGENDRE_COEFFS_16;
use kurbo::{
    Arc,
    BezPath,
    CubicBez,
    Line,
    ParamCurve,
    ParamCurveArclen,
    ParamCurveArea,
    ParamCurveDeriv,
    ParamCurveNearest,
    QuadBez,
    Shape,
    SvgArc,
    Vec2,
};
use svgtypes::PathSegment;

use crate::context::{segments_with_context, SegmentContext};
use crate::subpaths::subpath_ranges;

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
    pub(crate) pieces: Vec<Piece>,
    total: f64,
}

/// A drawing segment, and where along the path it starts.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Piece {
    pub(crate) index: usize,
    /// Which subpath the segment belongs to, counting from 0
    pub(crate) subpath: usize,
    /// Whether the segment is a close path
    pub(crate) closes: bool,
    pub(crate) shape: PieceShape,
    /// Where the segment starts and ends, exactly as the path gives them;
    /// an arc's shape only gets there to rounding
    pub(crate) from: kurbo::Point,
    pub(crate) to: kurbo::Point,
    pub(crate) start: f64,
    pub(crate) length: f64,
}

/// How [`PathMeasure::contains`] decides what is inside a path, as SVG's
/// `fill-rule` does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FillRule {
    /// Inside when the path winds around the point in total, counting
    /// clockwise and counterclockwise turns against each other. SVG's
    /// default.
    #[default]
    NonZero,
    /// Inside when a ray from the point crosses the path an odd number of
    /// times.
    EvenOdd,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum PieceShape {
    Line(Line),
    Quadratic(QuadBez),
    Cubic(CubicBez),
    Arc(Arc),
}

impl PathMeasure {
    /// Measures `segments`.
    pub fn new(segments: impl IntoIterator<Item = impl Borrow<PathSegment>>) -> Self {
        let segments: Vec<PathSegment> = segments.into_iter().map(|s| *s.borrow()).collect();
        let contexts: Vec<SegmentContext<'_>> = segments_with_context(&segments).collect();
        let mut pieces = Vec::new();
        let mut total = 0.0;
        for (subpath, range) in subpath_ranges(&contexts).into_iter().enumerate() {
            for context in &contexts[range] {
                let Some(shape) = PieceShape::of(context) else {
                    continue;
                };
                let length = shape.length();
                pieces.push(Piece {
                    index: context.index,
                    subpath,
                    closes: matches!(context.segment, PathSegment::ClosePath { .. }),
                    shape,
                    from: kurbo::Point::new(context.start.x, context.start.y),
                    to: kurbo::Point::new(context.end.x, context.end.y),
                    start: total,
                    length,
                });
                total += length;
            }
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

    /// The unit normal at `length`: the tangent turned a quarter turn
    /// counterclockwise on screen, where y points down, so it points to the
    /// left of the way the path runs. Paper.js and svgpathtools use the same
    /// direction. `None` where [`tangent_at`](Self::tangent_at) is.
    ///
    /// ```
    /// use svg_path_ops::euclid::default::Vector2D;
    /// use svg_path_ops::svgtypes::PathParser;
    /// use svg_path_ops::PathMeasure;
    ///
    /// let segments: Vec<_> = PathParser::from("M 0 0 H 10").collect::<Result<_, _>>()?;
    /// let normal = PathMeasure::new(&segments).normal_at(5.0);
    /// // Running right, the left is up the screen
    /// assert_eq!(normal, Some(Vector2D::new(0.0, -1.0)));
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn normal_at(&self, length: f64) -> Option<Vector2D<f64>> {
        let tangent = self.tangent_at(length)?;
        Some(Vector2D::new(tangent.y, -tangent.x))
    }

    /// The curvature at `length`: one over the radius of the circle that
    /// best fits the path there, positive where the path turns clockwise on
    /// screen and negative where it turns counterclockwise. A line has no
    /// curvature, and a circle of radius r has 1/r all round. `None` where
    /// the path draws nothing.
    ///
    /// At a join, the curvature is the start of the second segment's, as
    /// for [`position_at`](Self::position_at).
    ///
    /// ```
    /// use svg_path_ops::svgtypes::PathParser;
    /// use svg_path_ops::PathMeasure;
    ///
    /// // Half a circle of radius 10, drawn clockwise on screen
    /// let segments: Vec<_> =
    ///     PathParser::from("M 10 0 A 10 10 0 0 1 -10 0").collect::<Result<_, _>>()?;
    /// let curvature = PathMeasure::new(&segments).curvature_at(3.0).unwrap();
    /// assert!((curvature - 0.1).abs() < 1e-12);
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn curvature_at(&self, length: f64) -> Option<f64> {
        let (piece, t) = self.locate(length)?;
        self.pieces[piece].shape.curvature(t).or_else(|| {
            // A zero-length piece does not bend; use the nearest piece
            // before it that moves
            self.pieces[..=piece]
                .iter()
                .rev()
                .find_map(|piece| piece.shape.curvature(1.0))
        })
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

    /// The area the path encloses, signed by the direction it is drawn in:
    /// positive when drawn clockwise on screen, where y points down, and
    /// negative counterclockwise. Areas of subpaths drawn in opposite
    /// directions cancel.
    ///
    /// An open subpath is closed with a line back to its start, as filling
    /// it does. Arcs contribute their exact area.
    ///
    /// ```
    /// use svg_path_ops::svgtypes::PathParser;
    /// use svg_path_ops::PathMeasure;
    ///
    /// let square: Vec<_> = PathParser::from("M 0 0 h 10 v 10 h -10 z").collect::<Result<_, _>>()?;
    /// assert_eq!(PathMeasure::new(&square).area(), 100.0);
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn area(&self) -> f64 {
        let mut area = 0.0;
        for subpath in self.subpaths() {
            area += subpath
                .iter()
                .map(|piece| piece.shape.signed_area())
                .sum::<f64>();
            let (start, end) = subpath_ends(subpath);
            area += Line::new(end, start).signed_area();
        }
        area
    }

    /// Whether filling the path with `rule` covers `point`. An open subpath
    /// is filled as if closed with a line back to its start. Points right on
    /// the outline may land on either side.
    ///
    /// ```
    /// use svg_path_ops::euclid::default::Point2D;
    /// use svg_path_ops::svgtypes::PathParser;
    /// use svg_path_ops::{FillRule, PathMeasure};
    ///
    /// // A square with a smaller square inside, drawn the same way round
    /// let frame: Vec<_> = PathParser::from("M 0 0 h 30 v 30 h -30 z M 10 10 h 10 v 10 h -10 z")
    ///     .collect::<Result<_, _>>()?;
    /// let measure = PathMeasure::new(&frame);
    ///
    /// let middle = Point2D::new(15.0, 15.0);
    /// assert!(measure.contains(middle, FillRule::NonZero));
    /// assert!(!measure.contains(middle, FillRule::EvenOdd));
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn contains(&self, point: Point2D<f64>, rule: FillRule) -> bool {
        let mut outline = BezPath::new();
        for subpath in self.subpaths() {
            let (start, _) = subpath_ends(subpath);
            outline.move_to(start);
            for piece in subpath {
                piece.shape.append_to(&mut outline);
            }
            outline.close_path();
        }
        let winding = outline.winding(kurbo::Point::new(point.x, point.y));
        match rule {
            FillRule::NonZero => winding != 0,
            FillRule::EvenOdd => winding % 2 != 0,
        }
    }

    /// The pieces of each subpath that draws something.
    pub(crate) fn subpaths(&self) -> impl Iterator<Item = &[Piece]> {
        self.pieces.chunk_by(|a, b| a.subpath == b.subpath)
    }

    /// The part of the path between lengths `from` and `to`, as a path of
    /// its own in absolute coordinates, or an empty path when `from` is not
    /// before `to`. Lengths outside the path are clamped.
    ///
    /// Each segment keeps its kind: a cut arc is still an arc, a cut curve
    /// the same kind of curve. A part that spans subpaths moves to each
    /// one, and a close path stays one only when the part includes its
    /// whole subpath up to it; otherwise it becomes the line it draws.
    ///
    /// ```
    /// use svg_path_ops::svgtypes::PathParser;
    /// use svg_path_ops::{write_path, PathMeasure, WriteOptions};
    ///
    /// let segments: Vec<_> = PathParser::from("M 0 0 h 10 v 10").collect::<Result<_, _>>()?;
    /// let part = PathMeasure::new(&segments).crop(5.0, 15.0);
    /// assert_eq!(
    ///     write_path(&part, &WriteOptions::default()),
    ///     "M 5 0 L 10 0 L 10 5"
    /// );
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn crop(&self, from: f64, to: f64) -> Vec<PathSegment> {
        let clamp = |length: f64| {
            if length.is_nan() {
                0.0
            } else {
                length.clamp(0.0, self.total)
            }
        };
        let (from, to) = (clamp(from), clamp(to));
        let mut part = Vec::new();
        if from >= to {
            return part;
        }

        // The subpath being written, and whether the part includes it from
        // its start, which a close path needs to close it
        let mut current: Option<(usize, bool)> = None;
        for (i, piece) in self.pieces.iter().enumerate() {
            let end = piece.start + piece.length;
            let overlaps = piece.length > 0.0 && end > from && piece.start < to;
            let closes_in_range =
                piece.closes && piece.length == 0.0 && piece.start > from && piece.start <= to;
            if !overlaps && !closes_in_range {
                continue;
            }
            let t0 = if from > piece.start {
                piece.shape.inv_arclen(from - piece.start, piece.length)
            } else {
                0.0
            };
            let t1 = if to < end {
                piece.shape.inv_arclen(to - piece.start, piece.length)
            } else {
                1.0
            };

            if current.is_none_or(|(subpath, _)| subpath != piece.subpath) {
                let start = piece.shape.eval(t0);
                part.push(PathSegment::MoveTo { abs: true, x: start.x, y: start.y });
                let first_of_subpath = i == 0 || self.pieces[i - 1].subpath != piece.subpath;
                current = Some((piece.subpath, first_of_subpath && t0 == 0.0));
            }
            let from_subpath_start = current.is_some_and(|(_, whole)| whole);
            if piece.closes && t0 == 0.0 && t1 == 1.0 && from_subpath_start {
                part.push(PathSegment::ClosePath { abs: true });
            } else {
                part.push(piece.shape.segment(t0, t1));
            }
        }
        part
    }

    /// The path split at `length` into the part before it and the part
    /// after it. See [`crop`](Self::crop).
    pub fn split_at(&self, length: f64) -> (Vec<PathSegment>, Vec<PathSegment>) {
        (self.crop(0.0, length), self.crop(length, self.total))
    }

    /// The path made of straight lines only, none of them further than
    /// `tolerance` from the curve or arc it replaces. Close paths stay close
    /// paths.
    ///
    /// ```
    /// use svg_path_ops::svgtypes::PathParser;
    /// use svg_path_ops::{PathMeasure, PathSegment};
    ///
    /// let circle: Vec<_> = PathParser::from("M 10 0 A 10 10 0 0 1 -10 0 A 10 10 0 0 1 10 0 Z")
    ///     .collect::<Result<_, _>>()?;
    /// let polygon = PathMeasure::new(&circle).flatten(0.1);
    /// assert!(polygon.iter().all(|segment| matches!(
    ///     segment,
    ///     PathSegment::MoveTo { .. } | PathSegment::LineTo { .. } | PathSegment::ClosePath { .. }
    /// )));
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn flatten(&self, tolerance: f64) -> Vec<PathSegment> {
        let tolerance = tolerance.max(1e-9);
        let mut polygon = Vec::new();
        for subpath in self.subpaths() {
            let (start, _) = subpath_ends(subpath);
            polygon.push(PathSegment::MoveTo { abs: true, x: start.x, y: start.y });
            for piece in subpath {
                match piece.shape {
                    _ if piece.closes => polygon.push(PathSegment::ClosePath { abs: true }),
                    PieceShape::Line(line) => polygon.push(line_to(line.p1)),
                    PieceShape::Quadratic(quad) => {
                        flatten_bezier(&[quad.p0, quad.p1, quad.p2], tolerance, 0, &mut polygon)
                    }
                    PieceShape::Cubic(cubic) => flatten_bezier(
                        &[cubic.p0, cubic.p1, cubic.p2, cubic.p3],
                        tolerance,
                        0,
                        &mut polygon,
                    ),
                    PieceShape::Arc(arc) => {
                        polygon.extend(arc_polyline(&arc, tolerance).map(line_to))
                    }
                }
            }
        }
        polygon
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

impl Piece {
    /// The segment as the path gives it, in absolute coordinates: a close
    /// path, or its shape ending exactly where the path says.
    pub(crate) fn as_given(&self) -> PathSegment {
        if self.closes {
            return PathSegment::ClosePath { abs: true };
        }
        match self.shape.segment(0.0, 1.0) {
            // The arc's own end, not the one its shape comes back to
            PathSegment::EllipticalArc { rx, ry, x_axis_rotation, large_arc, sweep, .. } => {
                PathSegment::EllipticalArc {
                    abs: true,
                    rx,
                    ry,
                    x_axis_rotation,
                    large_arc,
                    sweep,
                    x: self.to.x,
                    y: self.to.y,
                }
            }
            segment => segment,
        }
    }

    /// The angle the path turns by, in radians from 0 to π, where this
    /// piece ends and `next` starts.
    pub(crate) fn turn_to(&self, next: &Piece) -> f64 {
        match (self.shape.direction(1.0), next.shape.direction(0.0)) {
            (Some(into), Some(out)) => into.cross(out).atan2(into.dot(out)).abs(),
            _ => 0.0,
        }
    }
}

/// The runs of `pieces`, which follow each other, between the joins that
/// turn by more than `corner` radians, as ranges of `pieces`.
pub(crate) fn corner_runs(pieces: &[&Piece], corner: f64) -> Vec<std::ops::Range<usize>> {
    let mut runs = Vec::new();
    let mut first = 0;
    for i in 1..pieces.len() {
        if pieces[i - 1].turn_to(pieces[i]) > corner {
            runs.push(first..i);
            first = i;
        }
    }
    if !pieces.is_empty() {
        runs.push(first..pieces.len());
    }
    runs
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
                match center_arc(&arc) {
                    Some(arc) => PieceShape::Arc(arc),
                    None => PieceShape::Line(Line::new(start, end)),
                }
            }
        })
    }

    pub(crate) fn eval(&self, t: f64) -> kurbo::Point {
        match self {
            PieceShape::Line(line) => line.eval(t),
            PieceShape::Quadratic(quad) => quad.eval(t),
            PieceShape::Cubic(cubic) => cubic.eval(t),
            PieceShape::Arc(arc) => arc.eval(t),
        }
    }

    pub(crate) fn length(&self) -> f64 {
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
            PieceShape::Quadratic(quad) => {
                let nearest = quad.nearest(target, ACCURACY);
                return polish_nearest(quad, target, nearest.t);
            }
            PieceShape::Cubic(cubic) => {
                let nearest = cubic.nearest(target, ACCURACY);
                return polish_nearest(cubic, target, nearest.t);
            }
            PieceShape::Arc(arc) => return arc_nearest(arc, target),
        };
        (nearest.t, nearest.distance_sq)
    }

    /// The piece's part of `½∮(x dy − y dx)`, the signed area a closed path
    /// made of such pieces encloses.
    fn signed_area(&self) -> f64 {
        match self {
            PieceShape::Line(line) => line.signed_area(),
            PieceShape::Quadratic(quad) => quad.signed_area(),
            PieceShape::Cubic(cubic) => cubic.signed_area(),
            PieceShape::Arc(arc) => arc_signed_area(arc),
        }
    }

    /// Appends the piece to `outline`, which must end where the piece
    /// starts. Arcs become cubic curves within 1e-7 of them.
    fn append_to(&self, outline: &mut BezPath) {
        match self {
            PieceShape::Line(line) => outline.line_to(line.p1),
            PieceShape::Quadratic(quad) => outline.quad_to(quad.p1, quad.p2),
            PieceShape::Cubic(cubic) => outline.curve_to(cubic.p1, cubic.p2, cubic.p3),
            PieceShape::Arc(arc) => outline.extend(arc.append_iter(1e-7)),
        }
    }

    /// The part of the piece between parameters `t0` and `t1` as an absolute
    /// segment of the same kind, starting at the point at `t0`.
    pub(crate) fn segment(&self, t0: f64, t1: f64) -> PathSegment {
        match self {
            PieceShape::Line(line) => {
                let end = line.eval(t1);
                PathSegment::LineTo { abs: true, x: end.x, y: end.y }
            }
            PieceShape::Quadratic(quad) => {
                let part = quad.subsegment(t0..t1);
                PathSegment::Quadratic {
                    abs: true,
                    x1: part.p1.x,
                    y1: part.p1.y,
                    x: part.p2.x,
                    y: part.p2.y,
                }
            }
            PieceShape::Cubic(cubic) => {
                let part = cubic.subsegment(t0..t1);
                PathSegment::CurveTo {
                    abs: true,
                    x1: part.p1.x,
                    y1: part.p1.y,
                    x2: part.p2.x,
                    y2: part.p2.y,
                    x: part.p3.x,
                    y: part.p3.y,
                }
            }
            PieceShape::Arc(arc) => {
                let part = arc.subsegment(t0..t1);
                let end = part.eval(1.0);
                PathSegment::EllipticalArc {
                    abs: true,
                    rx: part.radii.x,
                    ry: part.radii.y,
                    x_axis_rotation: part.x_rotation.to_degrees(),
                    large_arc: part.sweep_angle.abs() > std::f64::consts::PI,
                    sweep: part.sweep_angle > 0.0,
                    x: end.x,
                    y: end.y,
                }
            }
        }
    }

    /// The length from the piece's start to parameter `t`.
    pub(crate) fn length_to(&self, t: f64) -> f64 {
        match self {
            PieceShape::Line(line) => line.arclen(ACCURACY) * t,
            PieceShape::Quadratic(quad) => quad.subsegment(0.0..t).arclen(ACCURACY),
            PieceShape::Cubic(cubic) => cubic.subsegment(0.0..t).arclen(ACCURACY),
            PieceShape::Arc(arc) => arc_length(arc, 0.0, t),
        }
    }

    /// The derivative at `t`, or `None` where the piece does not move.
    pub(crate) fn direction(&self, t: f64) -> Option<Vec2> {
        self.moving_at(t).map(|t| self.derivative(t))
    }

    /// `t`, or a parameter just inside the piece when it does not move at
    /// `t`: a curve whose control point sits on its end has no derivative
    /// there, and just inside it the curve already runs along its tangent.
    /// `None` when it does not move there either.
    fn moving_at(&self, t: f64) -> Option<f64> {
        if self.derivative(t).hypot() > 1e-12 {
            return Some(t);
        }
        let inside = if t < 0.5 { t + 1e-6 } else { t - 1e-6 };
        (self.derivative(inside).hypot() > 1e-12).then_some(inside)
    }

    /// The derivative of the piece's point by its parameter.
    pub(crate) fn derivative(&self, t: f64) -> Vec2 {
        match self {
            PieceShape::Line(line) => line.p1 - line.p0,
            PieceShape::Quadratic(quad) => quad.deriv().eval(t).to_vec2(),
            PieceShape::Cubic(cubic) => cubic.deriv().eval(t).to_vec2(),
            PieceShape::Arc(arc) => arc_derivative(arc, t),
        }
    }

    /// The second derivative of the piece's point by its parameter.
    fn second_derivative(&self, t: f64) -> Vec2 {
        match self {
            PieceShape::Line(_) => Vec2::ZERO,
            PieceShape::Quadratic(quad) => quad.deriv().deriv().eval(t).to_vec2(),
            PieceShape::Cubic(cubic) => cubic.deriv().deriv().eval(t).to_vec2(),
            PieceShape::Arc(arc) => {
                // The point is C + R(rx cos θ, ry sin θ) with θ moving by
                // the sweep angle per unit of t
                let angle = arc.start_angle + arc.sweep_angle * t;
                let (sin, cos) = angle.sin_cos();
                let local = Vec2::new(-arc.radii.x * cos, -arc.radii.y * sin);
                let (rsin, rcos) = arc.x_rotation.sin_cos();
                Vec2::new(
                    local.x * rcos - local.y * rsin,
                    local.x * rsin + local.y * rcos,
                ) * (arc.sweep_angle * arc.sweep_angle)
            }
        }
    }

    /// The signed curvature at `t`, or `None` where the piece does not move.
    fn curvature(&self, t: f64) -> Option<f64> {
        let t = self.moving_at(t)?;
        let (velocity, acceleration) = (self.derivative(t), self.second_derivative(t));
        Some(velocity.cross(acceleration) / velocity.hypot().powi(3))
    }
}

/// The center form of an SVG arc, or `None` when it has a zero radius.
///
/// When the radii are too small to reach from one end to the other, SVG
/// scales them up until the ends are opposite ends of a diameter, so the
/// arc is exactly half the ellipse around their midpoint. kurbo finds that
/// center through a square root of what is then zero plus rounding noise,
/// which moves it by about 1e-8 of the radius; this case is built directly.
fn center_arc(arc: &SvgArc) -> Option<Arc> {
    let exact = Arc::from_svg_arc(arc)?;
    let (rx, ry) = (arc.radii.x.abs(), arc.radii.y.abs());
    let (sin, cos) = arc.x_rotation.sin_cos();
    let half = (arc.from - arc.to) * 0.5;
    let (px, py) = (cos * half.x + sin * half.y, -sin * half.x + cos * half.y);
    let reach = (px / rx).powi(2) + (py / ry).powi(2);
    if reach < 1.0 {
        return Some(exact);
    }
    let scale = reach.sqrt();
    let (rx, ry) = (rx * scale, ry * scale);
    let sweep_angle = if arc.sweep {
        std::f64::consts::PI
    } else {
        -std::f64::consts::PI
    };
    Some(Arc {
        center: arc.from.midpoint(arc.to),
        radii: Vec2::new(rx, ry),
        start_angle: (py / ry).atan2(px / rx),
        sweep_angle,
        x_rotation: arc.x_rotation,
    })
}

/// How far the Bézier curve with `controls` can stray from its chord.
///
/// The curve stays inside the convex hull of its control points, and the
/// distance to the chord is convex, so it strays no more than the furthest
/// control point. When every control point lies alongside the chord, so
/// does the curve, and its distance to the chord's line is the weighted sum
/// of the controls' distances with Bernstein weights. The end points weigh
/// nothing there, and the inner weights add up to at most 1/2 for a
/// quadratic curve and 3/4 for a cubic one, both at t = 1/2.
fn chord_distance_bound(controls: &[kurbo::Point]) -> f64 {
    let (first, last) = (controls[0], controls[controls.len() - 1]);
    let inner = &controls[1..controls.len() - 1];
    let chord = last - first;
    let length_sq = chord.hypot2();
    let alongside = length_sq > 0.0
        && inner.iter().all(|control| {
            let along = chord.dot(*control - first) / length_sq;
            (0.0..=1.0).contains(&along)
        });
    if alongside {
        let furthest = inner
            .iter()
            .map(|control| chord.cross(*control - first).abs())
            .fold(0.0, f64::max)
            / length_sq.sqrt();
        let weight = if inner.len() == 1 { 0.5 } else { 0.75 };
        weight * furthest
    } else {
        let segment = Line::new(first, last);
        inner
            .iter()
            .map(|control| segment.nearest(*control, ACCURACY).distance_sq.sqrt())
            .fold(0.0, f64::max)
    }
}

fn line_to(p: kurbo::Point) -> PathSegment {
    PathSegment::LineTo { abs: true, x: p.x, y: p.y }
}

/// Appends lines to `out` that follow the Bézier curve with `controls`,
/// none further than `tolerance` from it, splitting the curve in half until
/// its chord is close enough.
fn flatten_bezier(
    controls: &[kurbo::Point],
    tolerance: f64,
    depth: u32,
    out: &mut Vec<PathSegment>,
) {
    let last = controls[controls.len() - 1];
    if chord_distance_bound(controls) <= tolerance || depth >= 32 {
        out.push(line_to(last));
        return;
    }
    // De Casteljau's split at t = 0.5
    let (mut left, mut right) = (vec![controls[0]], vec![last]);
    let mut level = controls.to_vec();
    while level.len() > 1 {
        level = level
            .windows(2)
            .map(|pair| pair[0].midpoint(pair[1]))
            .collect();
        left.push(level[0]);
        right.push(level[level.len() - 1]);
    }
    right.reverse();
    flatten_bezier(&left, tolerance, depth + 1, out);
    flatten_bezier(&right, tolerance, depth + 1, out);
}

/// Points along the arc after its start, ending at its end, close enough
/// that the lines between them stay within `tolerance` of it. A chord over
/// an angle `a` of a circle with radius `r` strays `r(1 − cos(a / 2))` from
/// it; the larger radius of the ellipse bounds that.
fn arc_polyline(arc: &Arc, tolerance: f64) -> impl Iterator<Item = kurbo::Point> + '_ {
    let radius = arc.radii.x.abs().max(arc.radii.y.abs());
    let step = if tolerance >= radius {
        std::f64::consts::FRAC_PI_2
    } else {
        2.0 * (1.0 - tolerance / radius).acos()
    };
    let count = (arc.sweep_angle.abs() / step).ceil().max(1.0) as u32;
    (1..=count).map(move |k| arc.eval(f64::from(k) / f64::from(count)))
}

/// Refines the parameter of the curve's point nearest to `target` from
/// `t`, and returns it with the squared distance.
///
/// kurbo finds the nearest point of a curve through quadratic pieces that
/// approximate it, which leaves `t` off by up to about 1e-7 of the curve,
/// too much when the target lies on the curve. At the nearest point the
/// offset `C(t) − target` is at right angles to the curve, so Newton's
/// method on `(C(t) − target) · C′(t) = 0` moves `t` onto it; a step is
/// kept only while it brings the point closer.
fn polish_nearest<C>(curve: &C, target: kurbo::Point, mut t: f64) -> (f64, f64)
where
    C: ParamCurve + ParamCurveDeriv,
    C::DerivResult: ParamCurve + ParamCurveDeriv,
    <C::DerivResult as ParamCurveDeriv>::DerivResult: ParamCurve,
{
    let (first, second) = (curve.deriv(), curve.deriv().deriv());
    let distance_sq = |t: f64| (curve.eval(t) - target).hypot2();
    let mut best = distance_sq(t);
    for _ in 0..8 {
        let offset = curve.eval(t) - target;
        let velocity = first.eval(t).to_vec2();
        let slope = offset.dot(velocity);
        let curvature = velocity.hypot2() + offset.dot(second.eval(t).to_vec2());
        if curvature <= 0.0 {
            break;
        }
        let next = (t - slope / curvature).clamp(0.0, 1.0);
        let next_distance = distance_sq(next);
        if next_distance >= best {
            break;
        }
        (t, best) = (next, next_distance);
    }
    (t, best)
}

/// Where a subpath's first piece starts and its last piece ends.
fn subpath_ends(subpath: &[Piece]) -> (kurbo::Point, kurbo::Point) {
    let first = subpath.first().expect("a subpath has pieces");
    let last = subpath.last().expect("a subpath has pieces");
    (first.shape.eval(0.0), last.shape.eval(1.0))
}

/// The arc's part of `½∮(x dy − y dx)`. With the ellipse point
/// `P(θ) = C + R(rx cos θ, ry sin θ)`, the integrand `P × P'` is
/// `C × P' + rx ry`, so the integral is exact: `½(C × (P₁ − P₀) + rx ry Δθ)`.
fn arc_signed_area(arc: &Arc) -> f64 {
    let (start, end) = (arc.eval(0.0), arc.eval(1.0));
    let center = arc.center.to_vec2();
    0.5 * (center.cross(end - start) + arc.radii.x * arc.radii.y * arc.sweep_angle)
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

    use super::{FillRule, PathMeasure, Position};
    use crate::shapes::Shape;
    use crate::{write_path, PathSegment, WriteOptions};

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
    fn normal_points_left_of_the_way_the_path_runs() {
        let m = measure("M 0 0 H 10 V 10");
        assert!(close_vector(
            m.normal_at(5.0).unwrap(),
            Vector2D::new(0.0, -1.0)
        ));
        // Running down the screen, the left is to the right
        assert!(close_vector(
            m.normal_at(15.0).unwrap(),
            Vector2D::new(1.0, 0.0)
        ));
        // On a circle drawn clockwise it points to the outside
        let circle = measure("M 10 0 A 10 10 0 0 1 -10 0");
        assert!(close_vector(
            circle.normal_at(5.0 * PI).unwrap(),
            Vector2D::new(0.0, 1.0)
        ));
        assert_eq!(measure("M 5 5").normal_at(0.0), None);
    }

    #[test]
    fn curvature_of_lines_and_circles() {
        assert_eq!(measure("M 0 0 L 30 40").curvature_at(10.0), Some(0.0));
        let clockwise = measure("M 10 0 A 10 10 0 0 1 -10 0");
        let counterclockwise = measure("M 10 0 A 10 10 0 0 0 -10 0");
        for length in [0.0, 5.0, 15.0, 10.0 * PI] {
            assert!(close(clockwise.curvature_at(length).unwrap(), 0.1));
            assert!(close(counterclockwise.curvature_at(length).unwrap(), -0.1));
        }
    }

    #[test]
    fn curvature_of_an_ellipse() {
        // a / b² at the end of the long axis, b / a² at the end of the short
        let m = measure("M 20 0 A 20 10 0 0 1 -20 0");
        assert!(close(m.curvature_at(0.0).unwrap(), 20.0 / 100.0));
        assert!(close(
            m.curvature_at(m.total_length() / 2.0).unwrap(),
            10.0 / 400.0
        ));
    }

    #[test]
    fn curvature_matches_the_circle_through_nearby_points() {
        let m = measure("M 0 0 C 30 80 60 -40 100 20 Q 130 60 150 0");
        let h = 1e-3;
        for k in 1..20 {
            let length = m.total_length() * f64::from(k) / 20.0;
            let [a, b, c] = [length - h, length, length + h].map(|l| m.point_at(l).unwrap());
            // The circle through three points has radius |ab| |bc| |ca| / (2 |ab × ac|)
            let cross = (b - a).cross(c - a);
            let radius =
                (b - a).length() * (c - b).length() * (a - c).length() / (2.0 * cross.abs());
            let expected = cross.signum() / radius;
            let curvature = m.curvature_at(length).unwrap();
            assert!(
                (curvature - expected).abs() < 1e-5 * (1.0 + expected.abs()),
                "{curvature} vs {expected}"
            );
        }
    }

    #[test]
    fn curvature_where_a_control_point_sits_on_the_end() {
        let m = measure("M 0 0 C 0 0 10 10 10 0");
        assert!(m.curvature_at(0.0).unwrap().is_finite());
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
    fn nearest_point_on_the_curve_is_at_distance_zero() {
        let m = measure("M 201.72 78.24 C 197.47 75.69 189.84 60.83 188.14 59.13");
        for k in 1..10 {
            let on_curve = m.point_at(m.total_length() * f64::from(k) / 10.0).unwrap();
            assert!(m.nearest(on_curve).unwrap().distance < 1e-12);
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
    fn area_is_signed_by_direction() {
        // Clockwise on screen, where y points down
        assert_eq!(measure("M 0 0 h 10 v 10 h -10 z").area(), 100.0);
        assert_eq!(measure("M 0 0 v 10 h 10 v -10 z").area(), -100.0);
        // An open subpath is closed with a line back to its start
        assert_eq!(measure("M 0 0 h 10 v 10 h -10").area(), 100.0);
        assert_eq!(measure("M 0 0 L 10 0 L 0 10").area(), 50.0);
    }

    #[test]
    fn area_of_arcs_is_exact() {
        let circle = measure("M 10 0 A 10 10 0 0 1 -10 0 A 10 10 0 0 1 10 0");
        assert!(close(circle.area(), PI * 100.0));
        let ellipse = measure("M 20 0 A 20 10 30 0 1 -20 0 A 20 10 30 0 1 20 0");
        // The ends are not on the rotated ellipse's axis, but it still
        // closes into a whole ellipse, scaled to reach them
        let rx = ellipse_radius_through(20.0, 10.0, 30.0, 20.0, 0.0);
        assert!(close(ellipse.area(), PI * rx * rx / 2.0));
        let upright = measure("M 20 0 A 20 10 0 0 1 -20 0 A 20 10 0 0 1 20 0");
        assert!(close(upright.area(), PI * 200.0));
    }

    /// The x radius of an ellipse with radii in the ratio `rx : ry`, turned
    /// by `degrees`, scaled so it passes through `(x, y)` from its center.
    fn ellipse_radius_through(rx: f64, ry: f64, degrees: f64, x: f64, y: f64) -> f64 {
        let (sin, cos) = degrees.to_radians().sin_cos();
        let (u, v) = (x * cos + y * sin, -x * sin + y * cos);
        let scale = ((u / rx).powi(2) + (v / ry).powi(2)).sqrt().max(1.0);
        rx * scale
    }

    #[test]
    fn area_of_a_rounded_rect() {
        let rect = Shape::Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 60.0,
            rx: Some(10.0),
            ry: None,
        };
        let m = PathMeasure::new(rect.to_path());
        assert!(close(m.area(), 100.0 * 60.0 - (4.0 - PI) * 100.0));
    }

    #[test]
    fn areas_drawn_in_opposite_directions_cancel() {
        let frame = measure("M 0 0 h 30 v 30 h -30 z M 10 10 v 10 h 10 v -10 z");
        assert_eq!(frame.area(), 800.0);
    }

    #[test]
    fn contains_follows_the_fill_rule() {
        let middle = Point2D::new(15.0, 15.0);
        let ring = Point2D::new(5.0, 5.0);
        // Inner square drawn the same way round: nonzero fills it, evenodd not
        let same = measure("M 0 0 h 30 v 30 h -30 z M 10 10 h 10 v 10 h -10 z");
        assert!(same.contains(middle, FillRule::NonZero));
        assert!(!same.contains(middle, FillRule::EvenOdd));
        // Drawn the other way round, both leave a hole
        let reversed = measure("M 0 0 h 30 v 30 h -30 z M 10 10 v 10 h 10 v -10 z");
        assert!(!reversed.contains(middle, FillRule::NonZero));
        assert!(!reversed.contains(middle, FillRule::EvenOdd));
        for m in [&same, &reversed] {
            assert!(m.contains(ring, FillRule::NonZero) && m.contains(ring, FillRule::EvenOdd));
            assert!(!m.contains(Point2D::new(40.0, 5.0), FillRule::NonZero));
        }
    }

    #[test]
    fn contains_closes_open_subpaths() {
        let triangle = measure("M 0 0 L 10 0 L 0 10");
        assert!(triangle.contains(Point2D::new(2.0, 2.0), FillRule::NonZero));
        assert!(!triangle.contains(Point2D::new(8.0, 8.0), FillRule::NonZero));
    }

    #[test]
    fn contains_follows_arcs() {
        let circle = measure("M 10 0 A 10 10 0 0 1 -10 0 A 10 10 0 0 1 10 0");
        assert!(circle.contains(Point2D::new(0.0, 9.99), FillRule::NonZero));
        assert!(!circle.contains(Point2D::new(0.0, 10.01), FillRule::NonZero));
        assert!(circle.contains(Point2D::new(7.06, 7.06), FillRule::NonZero));
        assert!(!circle.contains(Point2D::new(7.08, 7.08), FillRule::NonZero));
    }

    fn written(segments: &[PathSegment]) -> String {
        write_path(segments, &WriteOptions::default())
    }

    /// Checks that cropping `path` between `from` and `to` gives a path of
    /// that length, running between the original's points at both lengths.
    fn check_crop(path: &str, from: f64, to: f64) -> Vec<PathSegment> {
        let m = measure(path);
        let part = m.crop(from, to);
        let cropped = PathMeasure::new(&part);
        assert!(
            close(cropped.total_length(), to - from),
            "{} != {}",
            cropped.total_length(),
            to - from
        );
        assert!(close_point(
            cropped.point_at(0.0).unwrap(),
            m.point_at(from).unwrap()
        ));
        assert!(close_point(
            cropped.point_at(to - from).unwrap(),
            m.point_at(to).unwrap()
        ));
        part
    }

    #[test]
    fn crops_lines() {
        assert_eq!(
            written(&measure("M 0 0 L 10 0").crop(2.0, 5.0)),
            "M 2 0 L 5 0"
        );
        assert_eq!(
            written(&measure("M 0 0 h 10 v 10").crop(5.0, 15.0)),
            "M 5 0 L 10 0 L 10 5"
        );
    }

    #[test]
    fn crops_curves_into_curves() {
        let part = check_crop("M 0 0 C 30 80 60 -40 100 20 Q 120 60 140 20", 20.0, 150.0);
        assert!(matches!(part[1], PathSegment::CurveTo { .. }));
        assert!(matches!(part[2], PathSegment::Quadratic { .. }));
    }

    #[test]
    fn crops_arcs_into_arcs() {
        // Three quarters of a circle of radius 10, cropped to more than half
        // of it: 39 long is about 223 degrees
        let path = "M 10 0 A 10 10 0 1 1 0 -10";
        let part = check_crop(path, 1.0, 40.0);
        let PathSegment::EllipticalArc { rx, ry, large_arc, sweep, .. } = part[1] else {
            panic!("{:?} is not an arc", part[1]);
        };
        assert!(close(rx, 10.0) && close(ry, 10.0) && large_arc && sweep);
        // A short piece of it is a small arc
        let part = check_crop(path, 2.0, 5.0);
        assert!(matches!(
            part[1],
            PathSegment::EllipticalArc { large_arc: false, .. }
        ));
        // So is a piece of a rotated ellipse
        check_crop("M 20 0 A 20 10 30 0 1 -20 0", 3.0, 40.0);
    }

    #[test]
    fn crops_across_subpaths() {
        let part = measure("M 0 0 L 10 0 M 100 0 L 110 0").crop(5.0, 15.0);
        assert_eq!(written(&part), "M 5 0 L 10 0 M 100 0 L 105 0");
    }

    #[test]
    fn keeps_close_paths_only_for_whole_subpaths() {
        let square = "M 0 0 h 10 v 10 h -10 z";
        let m = measure(square);
        assert_eq!(
            written(&m.crop(0.0, m.total_length())),
            "M 0 0 L 10 0 L 10 10 L 0 10 Z"
        );
        // Starting inside it, the close path would go back to the wrong place
        assert_eq!(
            written(&m.crop(5.0, m.total_length())),
            "M 5 0 L 10 0 L 10 10 L 0 10 L 0 0"
        );
        // An explicit close path of zero length is kept too
        let closed = measure("M 0 0 h 10 v 10 L 0 0 z");
        assert!(written(&closed.crop(0.0, closed.total_length())).ends_with('Z'));
    }

    #[test]
    fn crop_clamps_and_rejects_empty_ranges() {
        let m = measure("M 0 0 L 10 0");
        assert_eq!(written(&m.crop(-5.0, 50.0)), "M 0 0 L 10 0");
        assert!(m.crop(5.0, 5.0).is_empty());
        assert!(m.crop(6.0, 2.0).is_empty());
        assert!(measure("M 5 5").crop(0.0, 1.0).is_empty());
    }

    #[test]
    fn split_at_gives_both_parts() {
        let path = "M 0 0 C 30 80 60 -40 100 20 A 20 20 0 0 1 140 20";
        let m = measure(path);
        let (before, after) = m.split_at(70.0);
        assert!(close(PathMeasure::new(&before).total_length(), 70.0));
        assert!(close(
            PathMeasure::new(&after).total_length(),
            m.total_length() - 70.0
        ));
    }

    #[test]
    fn flatten_keeps_lines_and_close_paths() {
        assert_eq!(
            written(&measure("M 0 0 h 10 v 10 z").flatten(0.1)),
            "M 0 0 L 10 0 L 10 10 Z"
        );
    }

    #[test]
    fn flatten_stays_within_the_tolerance() {
        for path in [
            "M 10 0 A 10 10 0 0 1 -10 0 A 10 10 0 0 1 10 0 Z",
            "M 0 0 C 30 80 60 -40 100 20 Q 120 60 140 20",
            "M 20 0 A 20 10 30 0 1 -20 0",
        ] {
            let m = measure(path);
            for tolerance in [1.0, 0.1, 0.01] {
                let polygon = PathMeasure::new(m.flatten(tolerance));
                // Every point of the polygon is near the path, and every
                // point of the path near the polygon
                for k in 0..=400 {
                    let at = f64::from(k) / 400.0;
                    let on_polygon = polygon.point_at(polygon.total_length() * at).unwrap();
                    assert!(m.nearest(on_polygon).unwrap().distance <= tolerance + 1e-9);
                    let on_path = m.point_at(m.total_length() * at).unwrap();
                    assert!(polygon.nearest(on_path).unwrap().distance <= tolerance + 1e-9);
                }
            }
        }
    }

    #[test]
    fn flatten_uses_fewer_lines_for_a_looser_tolerance() {
        let m = measure("M 10 0 A 10 10 0 0 1 -10 0 A 10 10 0 0 1 10 0");
        assert!(m.flatten(1.0).len() < m.flatten(0.01).len());
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
        assert_eq!(m.area(), 0.0);
        assert!(!m.contains(Point2D::new(5.0, 5.0), FillRule::NonZero));
    }
}
