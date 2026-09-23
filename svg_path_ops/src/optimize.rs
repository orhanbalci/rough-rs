use std::borrow::Borrow;

use euclid::default::Point2D;
use svgtypes::PathSegment;

use crate::context::{segments_with_context, SegmentContext};
use crate::subpaths::subpath_ranges;
use crate::write::{round, write_path, WriteOptions};

type Point = Point2D<f64>;

/// Rewrites path data to draw the same shape in as few characters as
/// possible, for writing with [`write_path`] and
/// `WriteOptions { compact: true, precision }`.
///
/// With a `precision`, every point is first rounded to that many decimal
/// places in absolute coordinates, and relative coordinates are taken
/// between rounded points, so rounding errors do not add up along a path.
/// Without one, no point moves: numbers keep at most as many decimal places
/// as the input uses, which also clears the float noise of converting
/// between relative and absolute coordinates.
///
/// Then each segment is written in its shortest form:
///
/// - absolute or relative, whichever is shorter; a tie keeps the previous
///   segment's command, so compact output can leave its letter out
/// - horizontal and vertical lines as `H`/`V`
/// - curves whose first control mirrors the previous one as `S`/`T`
///
/// - curves whose control points lie on the line between their ends as
///   lines, when no control point strays further than half a unit of the
///   precision
/// - runs of cubic curves that follow one circle as a single arc, when
///   that is shorter and no point of the curves strays from the circle by
///   more than half a unit of the precision or 0.03% of the radius, which
///   the usual cubic approximations of a circle stay within; this needs a
///   precision, and without one only exactly straight curves become lines
///
/// and segments that draw nothing are removed: zero-length segments (unless
/// a subpath draws nothing else, since a round line cap still draws a dot),
/// arcs that end where they start, a line that runs back to the subpath
/// start right before a close path, which closes it anyway, and moves
/// followed by another move or by nothing. Arcs with a zero radius become
/// lines, as SVG draws them.
///
/// ```
/// use svg_path_ops::svgtypes::PathParser;
/// use svg_path_ops::{optimize, write_path, WriteOptions};
///
/// let segments: Vec<_> = PathParser::from(
///     "M 100 100 L 110 100 L 110 110.004 C 110 120 120 120 120 110 \
///      C 120 100 130 100 130 110 L 100 100 Z",
/// )
/// .collect::<Result<_, _>>()?;
///
/// let options = WriteOptions { precision: Some(1), compact: true };
/// assert_eq!(
///     write_path(optimize(&segments, Some(1)), &options),
///     "M100 100h10v10c0 10 10 10 10 0s10-10 10 0z"
/// );
/// # Ok::<(), svg_path_ops::svgtypes::Error>(())
/// ```
pub fn optimize(
    segments: impl IntoIterator<Item = impl Borrow<PathSegment>>,
    precision: Option<u8>,
) -> Vec<PathSegment> {
    let segments: Vec<PathSegment> = segments.into_iter().map(|s| *s.borrow()).collect();
    let contexts: Vec<SegmentContext<'_>> = segments_with_context(&segments).collect();
    let rounder = Rounder::new(precision, &segments);
    let geometry: Vec<Geometry> = contexts.iter().map(|c| Geometry::of(c, rounder)).collect();

    // Whether each segment's subpath draws something of non-zero length
    let mut subpath_draws = vec![false; geometry.len()];
    for range in subpath_ranges(&contexts) {
        let mut draws = false;
        for (context, g) in contexts[range.clone()].iter().zip(&geometry[range.clone()]) {
            if !matches!(g, Geometry::Move(_) | Geometry::Close) {
                draws |= !g.is_zero_length(rounder.point(context.start));
            }
        }
        subpath_draws[range].fill(draws);
    }

    let mut writer = Writer::new(rounder);
    let mut kept_dot = false;
    let mut i = 0;
    while i < geometry.len() {
        let index = i;
        let g = &geometry[index];
        let next = geometry.get(index + 1);
        i += 1;
        match *g {
            Geometry::Move(to) => {
                kept_dot = false;
                // A move followed by another move, or by nothing, draws nothing
                if matches!(next, None | Some(Geometry::Move(_))) {
                    continue;
                }
                writer.move_to(to);
            }
            Geometry::Close => writer.close(),
            _ => {
                let current = writer.current;
                if g.is_zero_length(current) {
                    // Keep one zero-length segment when the subpath draws
                    // nothing else, as a line cap still draws a dot for it
                    if subpath_draws[index] || kept_dot {
                        continue;
                    }
                    kept_dot = true;
                }
                // Close path draws the line back to the subpath start anyway
                if let (Geometry::Line(to), Some(Geometry::Close)) = (*g, next) {
                    if to == writer.subpath_start && !g.is_zero_length(current) {
                        continue;
                    }
                }
                if let Some((arc, following)) = arc_run(current, &geometry[index..], rounder) {
                    // Write the curves as one arc only when that is shorter
                    let (mut as_arc, mut as_curves) = (writer.fork(), writer.fork());
                    as_arc.draw(arc);
                    for curve in &geometry[index..=index + following] {
                        as_curves.draw(*curve);
                    }
                    if as_arc.text_len() < as_curves.text_len() {
                        writer.draw(arc);
                        i += following;
                        continue;
                    }
                }
                writer.draw(*g);
            }
        }
    }
    writer.segments
}

/// The most decimal places any number in `segments` is written with. Sums
/// and differences of such numbers need no more, so rounding to it only
/// removes float noise.
fn decimal_places(segments: &[PathSegment]) -> u8 {
    let places = |value: f64| {
        let text = value.to_string();
        text.split_once('.')
            .map_or(0, |(_, fraction)| fraction.len())
    };
    let numbers = segments.iter().flat_map(|segment| match *segment {
        PathSegment::MoveTo { x, y, .. }
        | PathSegment::LineTo { x, y, .. }
        | PathSegment::SmoothQuadratic { x, y, .. } => vec![x, y],
        PathSegment::HorizontalLineTo { x, .. } => vec![x],
        PathSegment::VerticalLineTo { y, .. } => vec![y],
        PathSegment::CurveTo { x1, y1, x2, y2, x, y, .. } => vec![x1, y1, x2, y2, x, y],
        PathSegment::SmoothCurveTo { x2, y2, x, y, .. } => vec![x2, y2, x, y],
        PathSegment::Quadratic { x1, y1, x, y, .. } => vec![x1, y1, x, y],
        PathSegment::EllipticalArc { rx, ry, x_axis_rotation, x, y, .. } => {
            vec![rx, ry, x_axis_rotation, x, y]
        }
        PathSegment::ClosePath { .. } => vec![],
    });
    let most = numbers.map(places).max().unwrap_or(0);
    u8::try_from(most).unwrap_or(u8::MAX)
}

/// Rounds coordinates to a number of decimal places: the precision asked
/// for, or the input's own when there is none.
#[derive(Clone, Copy)]
struct Rounder {
    places: u8,
    /// Whether the places came from the caller. Without that, rounding only
    /// clears float noise and no point may move at all.
    requested: bool,
}

impl Rounder {
    fn new(precision: Option<u8>, segments: &[PathSegment]) -> Self {
        match precision {
            Some(places) => Rounder { places, requested: true },
            None => Rounder { places: decimal_places(segments), requested: false },
        }
    }

    fn write_options(self) -> WriteOptions {
        WriteOptions { precision: Some(self.places), compact: true }
    }

    fn value(self, value: f64) -> f64 {
        round(value, self.places)
    }

    fn point(self, point: Point) -> Point {
        Point::new(self.value(point.x), self.value(point.y))
    }

    /// How far a point may move: half a unit of the requested precision,
    /// or nothing without one.
    fn tolerance(self) -> f64 {
        if self.requested {
            0.5 * 10f64.powi(-i32::from(self.places))
        } else {
            0.0
        }
    }
}

/// Whether a curve from `start` to `end` with `controls` draws exactly the
/// line between its ends: every control lies on that line, within
/// `tolerance`, and between the ends, so the curve does not overshoot them.
fn is_straight(start: Point, controls: &[Point], end: Point, tolerance: f64) -> bool {
    let direction = end - start;
    let length = direction.length();
    if length == 0.0 {
        return false;
    }
    controls.iter().all(|control| {
        let offset = *control - start;
        let distance = direction.cross(offset).abs() / length;
        let along = direction.dot(offset) / length;
        distance <= tolerance && along >= -tolerance && along <= length + tolerance
    })
}

/// A circle a run of cubic curves follows, and how far along it they go.
#[derive(Clone, Copy, Debug)]
struct CircleFit {
    center: Point,
    radius: f64,
    /// SVG's sweep flag: true for increasing angles, clockwise on screen
    sweep: bool,
    /// Angle covered, in radians
    angle: f64,
}

fn cubic_at(p0: Point, p1: Point, p2: Point, p3: Point, t: f64) -> Point {
    let u = 1.0 - t;
    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    Point::new(
        a * p0.x + b * p1.x + c * p2.x + d * p3.x,
        a * p0.y + b * p1.y + c * p2.y + d * p3.y,
    )
}

/// The circle through three points, if they are not on one line.
fn circle_through(a: Point, b: Point, c: Point) -> Option<(Point, f64)> {
    let (b, c) = (b - a, c - a);
    let d = 2.0 * b.cross(c);
    if d.abs() < 1e-12 {
        return None;
    }
    let (b2, c2) = (b.square_length(), c.square_length());
    let center = Point::new(
        a.x + (c.y * b2 - b.y * c2) / d,
        a.y + (b.x * c2 - c.x * b2) / d,
    );
    Some((center, (center - a).length()))
}

/// How closely a curve must follow a circle of `radius` to become an arc.
fn arc_tolerance(radius: f64, rounder: Rounder) -> f64 {
    rounder.tolerance().max(3e-4 * radius)
}

/// Fits the cubic curve from `start` to a circle, if every sampled point of
/// it lies on one.
fn fit_circle(
    start: Point,
    c1: Point,
    c2: Point,
    end: Point,
    rounder: Rounder,
) -> Option<CircleFit> {
    if start == end {
        return None;
    }
    let middle = cubic_at(start, c1, c2, end, 0.5);
    let (center, radius) = circle_through(start, middle, end)?;
    let fit = CircleFit { center, radius, sweep: false, angle: 0.0 };
    follow_circle(&fit, start, c1, c2, end, rounder)
}

/// If the cubic curve from `start` stays on the circle of `fit`, returns the
/// circle with the curve's direction and the angle it covers.
fn follow_circle(
    fit: &CircleFit,
    start: Point,
    c1: Point,
    c2: Point,
    end: Point,
    rounder: Rounder,
) -> Option<CircleFit> {
    let tolerance = arc_tolerance(fit.radius, rounder);
    let on_circle = (1..=16).all(|k| {
        let point = cubic_at(start, c1, c2, end, f64::from(k) / 16.0);
        ((point - fit.center).length() - fit.radius).abs() <= tolerance
    });
    if start == end || !on_circle {
        return None;
    }
    // Going start, middle, end turns right on screen for increasing angles
    let middle = cubic_at(start, c1, c2, end, 0.5);
    let sweep = (middle - start).cross(end - start) > 0.0;
    let angle_of = |p: Point| (p.y - fit.center.y).atan2(p.x - fit.center.x);
    let turn = angle_of(end) - angle_of(start);
    let angle = if sweep { turn } else { -turn }.rem_euclid(std::f64::consts::TAU);
    Some(CircleFit { sweep, angle, ..*fit })
}

/// If the cubic curves at the start of `geometry`, drawn from `current`,
/// follow one circle, returns them as a single arc along with how many
/// curves after the first it covers.
fn arc_run(current: Point, geometry: &[Geometry], rounder: Rounder) -> Option<(Geometry, usize)> {
    // An arc's radius is derived, not added up, so it only fits a precision
    // the caller asked for
    if !rounder.requested {
        return None;
    }
    let mut curves = Vec::new();
    let mut start = current;
    for g in geometry {
        let Geometry::Cubic(c1, c2, to) = *g else {
            break;
        };
        curves.push((start, c1, c2, to));
        start = to;
    }
    let &(run_start, c1, c2, first_end) = curves.first()?;
    let mut best = (fit_circle(run_start, c1, c2, first_end, rounder)?, 1);

    // Curve ends lie on the circle even when the curves only approximate
    // it, so a longer run takes its circle from three ends far apart and
    // checks every curve against it
    for count in 2..=curves.len() {
        let run = &curves[..count];
        let junction = run[(count - 1) / 2].3;
        let Some((center, radius)) = circle_through(run_start, junction, run[count - 1].3) else {
            break;
        };
        let sweep = best.0.sweep;
        let mut fit = CircleFit { center, radius, sweep, angle: 0.0 };
        let mut follows = true;
        for &(from, a, b, to) in run {
            match follow_circle(&fit, from, a, b, to, rounder) {
                Some(part) if part.sweep == sweep => fit.angle += part.angle,
                _ => {
                    follows = false;
                    break;
                }
            }
        }
        // A full turn would end where it starts, and SVG skips such an arc
        if !follows || fit.angle >= std::f64::consts::TAU - 1e-9 {
            break;
        }
        best = (fit, count);
    }

    let (fit, count) = best;
    let radius = rounder.value(fit.radius);
    let arc = Geometry::Arc {
        rx: radius,
        ry: radius,
        x_axis_rotation: 0.0,
        large_arc: fit.angle > std::f64::consts::PI,
        sweep: fit.sweep,
        to: curves[count - 1].3,
    };
    Some((arc, count - 1))
}

/// A segment in absolute coordinates, with shorthand and horizontal or
/// vertical lines expanded, so its shortest form can be chosen afresh.
#[derive(Clone, Copy, Debug)]
enum Geometry {
    Move(Point),
    Line(Point),
    Cubic(Point, Point, Point),
    Quadratic(Point, Point),
    Arc {
        rx: f64,
        ry: f64,
        x_axis_rotation: f64,
        large_arc: bool,
        sweep: bool,
        to: Point,
    },
    Close,
}

impl Geometry {
    fn of(context: &SegmentContext<'_>, rounder: Rounder) -> Self {
        let geometry = Self::exact(context, rounder);
        let start = rounder.point(context.start);
        let tolerance = rounder.tolerance();
        match geometry {
            Geometry::Cubic(c1, c2, to) if is_straight(start, &[c1, c2], to, tolerance) => {
                Geometry::Line(to)
            }
            Geometry::Quadratic(c, to) if is_straight(start, &[c], to, tolerance) => {
                Geometry::Line(to)
            }
            geometry => geometry,
        }
    }

    fn exact(context: &SegmentContext<'_>, rounder: Rounder) -> Self {
        let start = context.start;
        let absolute = |abs: bool, x: f64, y: f64| {
            let point = if abs {
                Point::new(x, y)
            } else {
                Point::new(start.x + x, start.y + y)
            };
            rounder.point(point)
        };
        let end = rounder.point(context.end);
        let implied = || rounder.point(context.implied_control.expect("smooth segment"));

        match *context.segment {
            PathSegment::MoveTo { .. } => Geometry::Move(end),
            PathSegment::LineTo { .. }
            | PathSegment::HorizontalLineTo { .. }
            | PathSegment::VerticalLineTo { .. } => Geometry::Line(end),
            PathSegment::CurveTo { abs, x1, y1, x2, y2, .. } => {
                Geometry::Cubic(absolute(abs, x1, y1), absolute(abs, x2, y2), end)
            }
            PathSegment::SmoothCurveTo { abs, x2, y2, .. } => {
                Geometry::Cubic(implied(), absolute(abs, x2, y2), end)
            }
            PathSegment::Quadratic { abs, x1, y1, .. } => {
                Geometry::Quadratic(absolute(abs, x1, y1), end)
            }
            PathSegment::SmoothQuadratic { .. } => Geometry::Quadratic(implied(), end),
            PathSegment::EllipticalArc { rx, ry, x_axis_rotation, large_arc, sweep, .. } => {
                Geometry::Arc {
                    rx: rounder.value(rx),
                    ry: rounder.value(ry),
                    x_axis_rotation: rounder.value(x_axis_rotation),
                    large_arc,
                    sweep,
                    to: end,
                }
            }
            PathSegment::ClosePath { .. } => Geometry::Close,
        }
    }

    /// Whether a drawing segment starting at `current` draws nothing. Moves
    /// and close paths are never zero-length segments here.
    fn is_zero_length(&self, current: Point) -> bool {
        match *self {
            Geometry::Line(to) => to == current,
            Geometry::Cubic(c1, c2, to) => c1 == current && c2 == current && to == current,
            Geometry::Quadratic(c, to) => c == current && to == current,
            // SVG omits an arc whose end points coincide
            Geometry::Arc { to, .. } => to == current,
            Geometry::Move(_) | Geometry::Close => false,
        }
    }
}

/// Emits the chosen segments and tracks the state of the output path, which
/// decides the relative coordinates and which shorthands are valid.
struct Writer {
    rounder: Rounder,
    segments: Vec<PathSegment>,
    current: Point,
    subpath_start: Point,
    /// Second control of the last emitted segment, if it was a cubic curve
    cubic_control: Option<Point>,
    /// Control of the last emitted segment, if it was a quadratic curve
    quadratic_control: Option<Point>,
    last_command: Option<u8>,
}

impl Writer {
    /// A copy of the writer's state holding only its last segment, to try
    /// out what writing more segments costs.
    fn fork(&self) -> Self {
        Writer {
            rounder: self.rounder,
            segments: self.segments.last().copied().into_iter().collect(),
            current: self.current,
            subpath_start: self.subpath_start,
            cubic_control: self.cubic_control,
            quadratic_control: self.quadratic_control,
            last_command: self.last_command,
        }
    }

    fn text_len(&self) -> usize {
        let options = self.rounder.write_options();
        write_path(&self.segments, &options).len()
    }

    fn new(rounder: Rounder) -> Self {
        Writer {
            rounder,
            segments: Vec::new(),
            current: Point::zero(),
            subpath_start: Point::zero(),
            cubic_control: None,
            quadratic_control: None,
            last_command: None,
        }
    }

    /// `point` relative to the current point, rounded again so float noise
    /// from the subtraction does not lengthen it.
    fn relative(&self, point: Point) -> Point {
        self.rounder.point(Point::new(
            point.x - self.current.x,
            point.y - self.current.y,
        ))
    }

    fn move_to(&mut self, to: Point) {
        let rel = self.relative(to);
        // On a tie the usual absolute move wins
        let candidates = [
            PathSegment::MoveTo { abs: true, x: to.x, y: to.y },
            PathSegment::MoveTo { abs: false, x: rel.x, y: rel.y },
        ];
        self.emit(&candidates);
        self.current = to;
        self.subpath_start = to;
        self.cubic_control = None;
        self.quadratic_control = None;
    }

    fn close(&mut self) {
        // Both cases cost the same; keep the last command's case
        let abs = self.last_command.is_some_and(|c| c.is_ascii_uppercase());
        self.emit(&[PathSegment::ClosePath { abs }]);
        self.current = self.subpath_start;
        self.cubic_control = None;
        self.quadratic_control = None;
    }

    fn draw(&mut self, geometry: Geometry) {
        // SVG draws an arc with a zero radius as a straight line
        let geometry = match geometry {
            Geometry::Arc { rx, ry, to, .. } if rx == 0.0 || ry == 0.0 => Geometry::Line(to),
            geometry => geometry,
        };
        let current = self.current;
        let mirror = |control: Option<Point>| {
            self.rounder
                .point(control.map_or(current, |c| current + (current - c)))
        };
        let mut candidates = Vec::with_capacity(6);
        let (mut cubic_control, mut quadratic_control) = (None, None);

        let to = match geometry {
            Geometry::Line(to) => {
                let rel = self.relative(to);
                if to.y == current.y {
                    candidates.push(PathSegment::HorizontalLineTo { abs: false, x: rel.x });
                    candidates.push(PathSegment::HorizontalLineTo { abs: true, x: to.x });
                }
                if to.x == current.x {
                    candidates.push(PathSegment::VerticalLineTo { abs: false, y: rel.y });
                    candidates.push(PathSegment::VerticalLineTo { abs: true, y: to.y });
                }
                candidates.push(PathSegment::LineTo { abs: false, x: rel.x, y: rel.y });
                candidates.push(PathSegment::LineTo { abs: true, x: to.x, y: to.y });
                to
            }
            Geometry::Cubic(c1, c2, to) => {
                let (r2, rel) = (self.relative(c2), self.relative(to));
                if c1 == mirror(self.cubic_control) {
                    candidates.push(PathSegment::SmoothCurveTo {
                        abs: false,
                        x2: r2.x,
                        y2: r2.y,
                        x: rel.x,
                        y: rel.y,
                    });
                    candidates.push(PathSegment::SmoothCurveTo {
                        abs: true,
                        x2: c2.x,
                        y2: c2.y,
                        x: to.x,
                        y: to.y,
                    });
                }
                let r1 = self.relative(c1);
                candidates.push(PathSegment::CurveTo {
                    abs: false,
                    x1: r1.x,
                    y1: r1.y,
                    x2: r2.x,
                    y2: r2.y,
                    x: rel.x,
                    y: rel.y,
                });
                candidates.push(PathSegment::CurveTo {
                    abs: true,
                    x1: c1.x,
                    y1: c1.y,
                    x2: c2.x,
                    y2: c2.y,
                    x: to.x,
                    y: to.y,
                });
                cubic_control = Some(c2);
                to
            }
            Geometry::Quadratic(c, to) => {
                let rel = self.relative(to);
                if c == mirror(self.quadratic_control) {
                    candidates.push(PathSegment::SmoothQuadratic {
                        abs: false,
                        x: rel.x,
                        y: rel.y,
                    });
                    candidates.push(PathSegment::SmoothQuadratic { abs: true, x: to.x, y: to.y });
                }
                let rc = self.relative(c);
                candidates.push(PathSegment::Quadratic {
                    abs: false,
                    x1: rc.x,
                    y1: rc.y,
                    x: rel.x,
                    y: rel.y,
                });
                candidates.push(PathSegment::Quadratic {
                    abs: true,
                    x1: c.x,
                    y1: c.y,
                    x: to.x,
                    y: to.y,
                });
                quadratic_control = Some(c);
                to
            }
            Geometry::Arc { rx, ry, x_axis_rotation, large_arc, sweep, to } => {
                let rel = self.relative(to);
                let arc = |abs: bool, point: Point| PathSegment::EllipticalArc {
                    abs,
                    rx,
                    ry,
                    x_axis_rotation,
                    large_arc,
                    sweep,
                    x: point.x,
                    y: point.y,
                };
                candidates.push(arc(false, rel));
                candidates.push(arc(true, to));
                to
            }
            Geometry::Move(_) | Geometry::Close => {
                unreachable!("moves and close paths are not drawing segments")
            }
        };

        self.emit(&candidates);
        self.current = to;
        self.cubic_control = cubic_control;
        self.quadratic_control = quadratic_control;
    }

    /// Emits the candidate that writes shortest after the last segment.
    fn emit(&mut self, candidates: &[PathSegment]) {
        let options = self.rounder.write_options();
        // The characters a candidate adds after the last segment, so the
        // letters and separators compact output leaves out are counted
        let cost = |segment: &PathSegment| match self.segments.last() {
            Some(last) => {
                write_path([*last, *segment], &options).len() - write_path([*last], &options).len()
            }
            None => write_path([*segment], &options).len(),
        };
        let best = candidates
            .iter()
            .min_by_key(|segment| cost(segment))
            .expect("at least one candidate");
        self.last_command = Some(best.command());
        self.segments.push(*best);
    }
}

#[cfg(test)]
mod test {
    use svgtypes::PathParser;

    use super::optimize;
    use crate::{absolutize, normalize, write_path, PathSegment, WriteOptions};

    fn parse(path: &str) -> Vec<PathSegment> {
        PathParser::from(path).map(Result::unwrap).collect()
    }

    fn optimized(path: &str, precision: Option<u8>) -> String {
        let options = WriteOptions { precision, compact: true };
        write_path(optimize(parse(path), precision), &options)
    }

    /// Absolute points every segment of a normalized path passes through,
    /// with a close path as the line back to its subpath start.
    fn points(segments: &[PathSegment]) -> Vec<(f64, f64)> {
        let mut subpath_start = (0.0, 0.0);
        normalize(absolutize(segments.iter()))
            .flat_map(|segment| match segment {
                PathSegment::MoveTo { x, y, .. } => {
                    subpath_start = (x, y);
                    vec![(x, y)]
                }
                PathSegment::LineTo { x, y, .. } => vec![(x, y)],
                PathSegment::CurveTo { x1, y1, x2, y2, x, y, .. } => {
                    vec![(x1, y1), (x2, y2), (x, y)]
                }
                PathSegment::ClosePath { .. } => vec![subpath_start],
                _ => vec![],
            })
            .collect()
    }

    #[test]
    fn picks_the_shorter_of_absolute_and_relative() {
        assert_eq!(optimized("M 100 100 L 101 101", None), "M100 100l1 1");
        // A line right after a move needs no letter
        assert_eq!(optimized("m 5 5 l -100 -100", None), "M5 5-95-95");
    }

    #[test]
    fn writes_horizontal_and_vertical_lines() {
        assert_eq!(optimized("M 0 0 L 10 0 L 10 10", None), "M0 0h10v10");
    }

    #[test]
    fn finds_smooth_curves() {
        assert_eq!(
            optimized("M 0 0 C 0 10 10 10 10 0 C 10 -10 20 -10 20 0", None),
            "M0 0c0 10 10 10 10 0s10-10 10 0"
        );
        assert_eq!(
            optimized("M 0 0 Q 10 10 20 0 Q 30 -10 40 0", None),
            "M0 0q10 10 20 0t20 0"
        );
        // With no curve before it, a smooth curve's first control is the start
        assert_eq!(optimized("M 0 0 C 0 0 10 10 20 0", None), "M0 0s10 10 20 0");
    }

    #[test]
    fn smooth_curves_follow_the_output_not_the_input() {
        // The zero-length line is dropped, so the S may not mirror the C
        // before it: it has to become a full curve
        let path = "M 0 0 C 0 10 10 10 10 0 L 10 0 S 20 -10 20 0";
        let result = optimize(parse(path), None);
        assert_eq!(
            write_path(&result, &WriteOptions { precision: None, compact: true }),
            "M0 0c0 10 10 10 10 0 0 0 10-10 10 0"
        );
    }

    #[test]
    fn removes_segments_that_draw_nothing() {
        assert_eq!(optimized("M 0 0 L 0 0 L 5 0", None), "M0 0h5");
        assert_eq!(optimized("M 0 0 A 5 5 0 0 1 0 0 L 5 0", None), "M0 0h5");
        // The line back to the start is drawn by the close path
        assert_eq!(
            optimized("M 0 0 L 10 0 L 10 10 L 0 0 Z", None),
            "M0 0h10v10z"
        );
    }

    #[test]
    fn keeps_a_lone_zero_length_segment_as_a_dot() {
        assert_eq!(optimized("M 5 5 L 5 5", None), "M5 5h0");
        assert_eq!(
            optimized("M 5 5 L 5 5 L 5 5 M 9 9 L 10 9", None),
            "M5 5h0M9 9h1"
        );
    }

    #[test]
    fn drops_moves_that_draw_nothing() {
        assert_eq!(optimized("M 1 1 M 2 2 L 3 3", None), "M2 2l1 1");
        assert_eq!(optimized("M 1 1 L 3 3 M 5 5", None), "M1 1l2 2");
    }

    #[test]
    fn arc_with_zero_radius_becomes_a_line() {
        assert_eq!(optimized("M 0 0 A 0 5 0 0 1 10 0", None), "M0 0h10");
    }

    #[test]
    fn keeps_arcs() {
        assert_eq!(
            optimized("M 10 10 A 5 5 0 0 1 20 10", None),
            "M10 10a5 5 0 0110 0"
        );
    }

    #[test]
    fn rounding_does_not_drift_along_relative_segments() {
        let path = format!("M 0 0{}", " l 0.333 0.333".repeat(30));
        let result = optimize(parse(&path), Some(1));
        let expected = points(&parse(&path));
        for ((x, y), (ex, ey)) in points(&result).into_iter().zip(expected) {
            let close = |a: f64, b: f64| (a - b).abs() <= 0.05 + 1e-9;
            assert!(
                close(x, ex) && close(y, ey),
                "({x}, {y}) drifted from ({ex}, {ey})"
            );
        }
    }

    #[test]
    fn keeps_the_geometry() {
        let path = "M 10 10 h 20 V 40 c 5 5 10 5 15 0 S 60 30 60 40 q 5 -10 10 0 t 10 0 \
                    A 5 5 0 0 1 90 40 L 10 10 Z m 5 5 l 1 1 l 0 0 z M 50 50 L 50 50";
        let original = parse(path);
        let result = optimize(&original, None);
        let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
        let (got, expected) = (points(&result), points(&original));
        // The dropped segments only remove repeated points
        let dedup = |mut points: Vec<(f64, f64)>| {
            points.dedup_by(|a, b| close(a.0, b.0) && close(a.1, b.1));
            points
        };
        let (got, expected) = (dedup(got), dedup(expected));
        assert_eq!(got.len(), expected.len());
        for (a, b) in got.iter().zip(&expected) {
            assert!(close(a.0, b.0) && close(a.1, b.1), "{a:?} != {b:?}");
        }
    }

    #[test]
    fn without_precision_relative_input_does_not_grow() {
        // 201.7208 - 4.2472 is 197.47359999999998 in floating point
        let path = "M 201.7208 78.236 c -4.2472 -2.5448 -11.884 -17.4056 -13.5836 -19.102";
        assert_eq!(
            optimized(path, None),
            "M201.7208 78.236c-4.2472-2.5448-11.884-17.4056-13.5836-19.102"
        );
    }

    #[test]
    fn straight_curves_become_lines() {
        assert_eq!(optimized("M 0 0 C 2 0 8 0 10 0", None), "M0 0h10");
        assert_eq!(optimized("M 0 0 Q 5 5 10 10", None), "M0 0l10 10");
        // Within half a unit of the precision is straight enough
        assert_eq!(
            optimized("M 0 0 C 3 0.001 7 -0.001 10 0", Some(2)),
            "M0 0h10"
        );
        // Without a precision, it is not
        assert_eq!(
            optimized("M 0 0 C 3 0.001 7 -0.001 10 0", None),
            "M0 0c3 .001 7-.001 10 0"
        );
    }

    #[test]
    fn curves_that_overshoot_their_ends_stay_curves() {
        assert_eq!(
            optimized("M 0 0 C -5 0 15 0 10 0", Some(2)),
            "M0 0c-5 0 15 0 10 0"
        );
    }

    /// A circle of radius 10 around (50, 50) drawn with the usual four
    /// cubic curves.
    fn cubic_circle() -> String {
        let (near, far) = (50.0 + 5.523, 50.0 - 5.523);
        format!(
            "M 60 50 C 60 {near} {near} 60 50 60 C {far} 60 40 {near} 40 50 \
             C 40 {far} {far} 40 50 40 C {near} 40 60 {far} 60 50 Z"
        )
    }

    #[test]
    fn curves_on_a_circle_become_arcs() {
        // Three quarters make one large arc; a full turn cannot be one arc
        assert_eq!(
            optimized(&cubic_circle(), Some(2)),
            "M60 50A10 10 0 1150 40a10 10 0 0110 10z"
        );
    }

    #[test]
    fn curves_become_arcs_only_with_a_precision() {
        assert!(!optimized(&cubic_circle(), None).contains(['a', 'A']));
    }

    #[test]
    fn curves_off_the_circle_stay_curves() {
        // A quarter that bulges by 0.5 is no arc
        let path = "M 60 50 C 60 56 56 60.5 50 60.5";
        assert!(!optimized(path, Some(2)).contains(['a', 'A']));
    }

    #[test]
    fn empty_path_stays_empty() {
        assert!(optimize(Vec::<PathSegment>::new(), None).is_empty());
    }
}
