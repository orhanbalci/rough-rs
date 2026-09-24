use i_overlay::core::fill_rule::FillRule as OverlayFill;
use i_overlay::core::overlay_rule::OverlayRule;
use kurbo::{Point, Vec2};
use svgtypes::PathSegment;

use crate::boolean::{combine, finish, flatten_tolerance, polygons};
use crate::measure::{Piece, PieceShape};
use crate::{BooleanOptions, FillRule, LineCap, LineJoin, PathMeasure, StrokeStyle};

/// How many times a stretch of a curve is halved at most while following
/// its offsets.
const MAX_DEPTH: u32 = 16;

impl PathMeasure {
    /// The area the path's stroke covers, as a path: what SVG draws when it
    /// strokes the path with `style`, caps, joins and all, as closed
    /// outlines to fill. It is what "stroke to path" does in a drawing
    /// program, and what a cutter or plotter needs to follow a thick line.
    ///
    /// `options` sets the tolerance and whether the outline is drawn with
    /// curves, as for [`boolean`](crate::boolean); its fill rule is not
    /// used. Dashes are not applied.
    ///
    /// # Algorithm
    ///
    /// Along each segment, the stroke covers the lines across it, half the
    /// width to each side. Those lines are taken at points close enough
    /// together that the edges between their ends are within a quarter of
    /// the tolerance of the stroke's edges, halving a stretch until its
    /// middle and quarter points are, and the four-sided pieces between
    /// neighbouring lines, cut into triangles, cover the segment's part.
    /// So a curve tighter than half the width is covered right up to where
    /// its inner edge turns back on itself. Joins are added only where
    /// segments meet, as the style says: the triangle a bevel fills, the
    /// arc of a round join, or a miter's point when it is within the miter
    /// limit, and caps at the ends of open subpaths. All the pieces are
    /// united by [i_overlay] and, as for [`boolean`](crate::boolean), curves
    /// are fitted to the outline.
    ///
    /// [i_overlay]: https://docs.rs/i_overlay
    ///
    /// ```
    /// use svg_path_ops::pt::PathTransformer;
    /// use svg_path_ops::{BooleanOptions, LineCap, StrokeStyle};
    ///
    /// let line = PathTransformer::parse("M 0 0 H 20")?.measure();
    /// let style = StrokeStyle {
    ///     width: 4.0,
    ///     cap: LineCap::Square,
    ///     ..StrokeStyle::default()
    /// };
    /// let outline = line.stroke_to_path(&style, &BooleanOptions::default());
    /// // A 24 by 4 rectangle
    /// assert!((svg_path_ops::PathMeasure::new(&outline).area() - 96.0).abs() < 1e-9);
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn stroke_to_path(
        &self,
        style: &StrokeStyle,
        options: &BooleanOptions,
    ) -> Vec<PathSegment> {
        let tolerance = options.tolerance.max(1e-9);
        let pieces = stroke_polygons(self, style, flatten_tolerance(options));
        let shapes = combine(
            &pieces,
            &Vec::new(),
            OverlayRule::Subject,
            OverlayFill::NonZero,
            tolerance,
        );
        finish(&shapes, tolerance, options.curves)
    }

    /// The area the path covers, grown by `distance` all round, or shrunk
    /// when `distance` is negative, as a path. Corners on the outside of
    /// the growth are drawn with `join`, mitered up to `miter_limit` times
    /// the distance; the outside of a curve stays a curve.
    ///
    /// `options` sets the fill rule that says what the path covers, the
    /// tolerance and whether the outline is drawn with curves, as for
    /// [`boolean`](crate::boolean). The path's subpaths are expected not to
    /// cross each other or themselves: when shrinking, every outline eats
    /// into the area around it, even one that lies inside another.
    ///
    /// # Algorithm
    ///
    /// Growing by `d` is the union of the area with the stroke of its
    /// outline `2|d|` wide, and shrinking is the area less that stroke,
    /// each found as in [`stroke_to_path`](Self::stroke_to_path).
    ///
    /// ```
    /// use svg_path_ops::pt::PathTransformer;
    /// use svg_path_ops::{BooleanOptions, LineJoin, PathMeasure};
    ///
    /// let square = PathTransformer::parse("M 0 0 H 20 V 20 H 0 Z")?.measure();
    /// let options = BooleanOptions::default();
    /// let grown = square.offset(2.0, LineJoin::Miter, 4.0, &options);
    /// let shrunk = square.offset(-2.0, LineJoin::Miter, 4.0, &options);
    /// assert!((PathMeasure::new(&grown).area() - 576.0).abs() < 1e-9);
    /// assert!((PathMeasure::new(&shrunk).area() - 256.0).abs() < 1e-9);
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn offset(
        &self,
        distance: f64,
        join: LineJoin,
        miter_limit: f64,
        options: &BooleanOptions,
    ) -> Vec<PathSegment> {
        let tolerance = options.tolerance.max(1e-9);
        let flatten = flatten_tolerance(options);
        let fill = match options.fill_rule {
            FillRule::NonZero => OverlayFill::NonZero,
            FillRule::EvenOdd => OverlayFill::EvenOdd,
        };
        // The area as clean outlines, which nonzero fills as `fill` did
        let area = combine(
            &polygons(self, flatten),
            &Vec::new(),
            OverlayRule::Subject,
            fill,
            tolerance,
        );
        let area: Vec<Vec<[f64; 2]>> = area.into_iter().flatten().collect();
        if distance == 0.0 {
            return finish(&[area], tolerance, options.curves);
        }

        // The stroke of every subpath, closed as filling closes it
        let style = StrokeStyle {
            width: 2.0 * distance.abs(),
            cap: LineCap::Butt,
            join,
            miter_limit,
        };
        let band = stroke_polygons_closed(self, &style, flatten);
        let rule = if distance > 0.0 {
            OverlayRule::Union
        } else {
            OverlayRule::Difference
        };
        let shapes = combine(&area, &band, rule, OverlayFill::NonZero, tolerance);
        finish(&shapes, tolerance, options.curves)
    }
}

/// The polygons, all counterclockwise in the math sense, whose union is
/// the stroke of the path, none of their edges further than `tolerance`
/// inside it.
fn stroke_polygons(
    measure: &PathMeasure,
    style: &StrokeStyle,
    tolerance: f64,
) -> Vec<Vec<[f64; 2]>> {
    let mut out = Vec::new();
    for subpath in measure.subpaths() {
        let closed = subpath.last().is_some_and(|piece| piece.closes);
        subpath_polygons(subpath, closed, style, tolerance, &mut out);
    }
    out
}

/// As [`stroke_polygons`], but every subpath is stroked closed, as its
/// outline when filled.
fn stroke_polygons_closed(
    measure: &PathMeasure,
    style: &StrokeStyle,
    tolerance: f64,
) -> Vec<Vec<[f64; 2]>> {
    let mut out = Vec::new();
    for subpath in measure.subpaths() {
        let closed = subpath.last().is_some_and(|piece| piece.closes);
        if closed {
            subpath_polygons(subpath, true, style, tolerance, &mut out);
        } else {
            // The line back to the start that filling draws
            let (start, end) = (subpath[0].from, subpath[subpath.len() - 1].to);
            let mut closing = *subpath.last().expect("a subpath has pieces");
            closing.shape = PieceShape::Line(kurbo::Line::new(end, start));
            closing.from = end;
            closing.to = start;
            closing.length = (start - end).hypot();
            closing.closes = true;
            let mut pieces: Vec<Piece> = subpath.to_vec();
            pieces.push(closing);
            subpath_polygons(&pieces, true, style, tolerance, &mut out);
        }
    }
    out
}

fn subpath_polygons(
    subpath: &[Piece],
    closed: bool,
    style: &StrokeStyle,
    tolerance: f64,
    out: &mut Vec<Vec<[f64; 2]>>,
) {
    let half = style.width.max(0.0) / 2.0;
    if half == 0.0 {
        return;
    }
    let drawing: Vec<&Piece> = subpath.iter().filter(|p| p.length > 0.0).collect();
    let (Some(first), Some(last)) = (drawing.first(), drawing.last()) else {
        // A point: a round cap is a dot, a square one a square along the
        // x axis
        let at = subpath[0].from;
        match style.cap {
            LineCap::Butt => {}
            LineCap::Round => push(
                out,
                arc(
                    at,
                    half,
                    Vec2::new(1.0, 0.0),
                    std::f64::consts::TAU,
                    tolerance,
                ),
            ),
            LineCap::Square => {
                let (x, y) = (at.x, at.y);
                push(
                    out,
                    [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
                        .map(|(dx, dy)| Point::new(x + dx * half, y + dy * half))
                        .to_vec(),
                );
            }
        }
        return;
    };

    for piece in &drawing {
        body(&piece.shape, half, tolerance, out);
    }
    let across = |piece: &Piece, t: f64| across(&piece.shape, t, half);
    for pair in drawing.windows(2) {
        let (into, out_dir) = (direction(pair[0], 1.0), direction(pair[1], 0.0));
        join(
            out,
            across(pair[0], 1.0),
            across(pair[1], 0.0),
            into,
            out_dir,
            style,
            tolerance,
        );
    }
    if closed {
        let (into, out_dir) = (direction(last, 1.0), direction(first, 0.0));
        join(
            out,
            across(last, 1.0),
            across(first, 0.0),
            into,
            out_dir,
            style,
            tolerance,
        );
    } else {
        // Round each end from one side of the path to the other
        let start = across(first, 0.0);
        cap(
            out,
            start.0,
            start.1,
            start.2,
            -direction(first, 0.0),
            style,
            tolerance,
        );
        let end = across(last, 1.0);
        cap(
            out,
            end.0,
            end.2,
            end.1,
            direction(last, 1.0),
            style,
            tolerance,
        );
    }
}

fn direction(piece: &Piece, t: f64) -> Vec2 {
    unit(piece.shape.direction(t))
}

fn unit(v: Option<Vec2>) -> Vec2 {
    match v {
        Some(v) if v.hypot() > 0.0 => v / v.hypot(),
        _ => Vec2::ZERO,
    }
}

/// The unit normal of `shape` at `t`.
fn normal(shape: &PieceShape, t: f64) -> Vec2 {
    let d = unit(shape.direction(t));
    Vec2::new(-d.y, d.x)
}

/// Adds the pieces covering the stroke of one segment.
fn body(shape: &PieceShape, half: f64, tolerance: f64, out: &mut Vec<Vec<[f64; 2]>>) {
    let at = |t: f64| across(shape, t, half);
    if let PieceShape::Line(_) = shape {
        band(at(0.0), at(1.0), out);
        return;
    }
    // Start from quarters, so a stretch is never judged by points that
    // happen to line up
    for k in 0..4 {
        let (t0, t1) = (f64::from(k) / 4.0, f64::from(k + 1) / 4.0);
        stretch(&at, (t0, at(t0)), (t1, at(t1)), tolerance, 0, out);
    }
}

type Across = (Point, Point, Point);

/// Adds the pieces between the lines across the segment at `a` and `b`,
/// halving the stretch until the edges between them follow the stroke's
/// edges within `tolerance`.
fn stretch(
    at: &impl Fn(f64) -> Across,
    a: (f64, Across),
    b: (f64, Across),
    tolerance: f64,
    depth: u32,
    out: &mut Vec<Vec<[f64; 2]>>,
) {
    let (t0, t1) = (a.0, b.0);
    let close = [0.25, 0.5, 0.75].iter().all(|f| {
        let (p, left, right) = at(t0 + (t1 - t0) * f);
        distance_to_segment(left, a.1 .1, b.1 .1) <= tolerance
            && distance_to_segment(right, a.1 .2, b.1 .2) <= tolerance
            && distance_to_segment(p, a.1 .0, b.1 .0) <= tolerance
    });
    if close || depth >= MAX_DEPTH {
        band(a.1, b.1, out);
        return;
    }
    let tm = (t0 + t1) / 2.0;
    let m = (tm, at(tm));
    stretch(at, a, m, tolerance, depth + 1, out);
    stretch(at, m, b, tolerance, depth + 1, out);
}

/// Adds the pieces between the lines across the path at `a` and `b`: on
/// each side of the path, two triangles. A four-sided piece would cross
/// itself where the stroke's inner edge turns back on itself, and fill
/// part of it the wrong way round; a triangle never does.
fn band(a: Across, b: Across, out: &mut Vec<Vec<[f64; 2]>>) {
    push(out, vec![a.0, a.1, b.1]);
    push(out, vec![a.0, b.1, b.0]);
    push(out, vec![a.0, b.0, b.2]);
    push(out, vec![a.0, b.2, a.2]);
}

/// Adds `points` as a polygon, turned counterclockwise in the math sense,
/// unless it covers nothing.
fn push(out: &mut Vec<Vec<[f64; 2]>>, mut points: Vec<Point>) {
    let area: f64 = points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .map(|(p, q)| p.to_vec2().cross(q.to_vec2()))
        .sum();
    if area == 0.0 {
        return;
    }
    if area < 0.0 {
        points.reverse();
    }
    out.push(points.into_iter().map(|p| [p.x, p.y]).collect());
}

fn distance_to_segment(p: Point, a: Point, b: Point) -> f64 {
    let ab = b - a;
    let length = ab.hypot2();
    if length == 0.0 {
        return (p - a).hypot();
    }
    let t = ((p - a).dot(ab) / length).clamp(0.0, 1.0);
    (p - (a + ab * t)).hypot()
}

/// The point of `shape` at `t` and the ends of the line across it there,
/// `half` to its left and to its right. Joins and caps take their points
/// from here too, so the edges they share with the segments' pieces are
/// the very same and unite without a hairline gap.
fn across(shape: &PieceShape, t: f64, half: f64) -> Across {
    let (p, n) = (shape.eval(t), normal(shape, t));
    (p, p + n * half, p - n * half)
}

/// Adds what a join covers outside the segments' pieces, between the end
/// `a` of one segment, arriving in direction `into`, and the start `b` of
/// the next, leaving in direction `out_dir`.
fn join(
    out: &mut Vec<Vec<[f64; 2]>>,
    a: Across,
    b: Across,
    into: Vec2,
    out_dir: Vec2,
    style: &StrokeStyle,
    tolerance: f64,
) {
    let half = style.width.max(0.0) / 2.0;
    let turn = into.cross(out_dir).atan2(into.dot(out_dir));
    if turn == 0.0 {
        return;
    }
    // The outer side is the one the path turns away from: the left when
    // it turns right
    let (from, to) = if turn < 0.0 { (a.1, b.1) } else { (a.2, b.2) };
    let (n_into, n_out) = ((from - a.0) / half, (to - b.0) / half);
    match style.join {
        LineJoin::Bevel => push(out, vec![a.0, from, to, b.0]),
        LineJoin::Round => {
            let mut points = arc(a.0, half, n_into, turn, tolerance);
            // The ends are the segments' own points
            points[0] = from;
            *points.last_mut().expect("an arc has points") = to;
            points.insert(0, a.0);
            points.push(b.0);
            push(out, points);
        }
        LineJoin::Miter => {
            let cos = (turn / 2.0).cos();
            if cos > 0.0 && 1.0 / cos <= style.miter_limit {
                let tip = a.0 + unit(Some(n_into + n_out)) * (half / cos);
                push(out, vec![a.0, from, tip, to, b.0]);
            } else {
                push(out, vec![a.0, from, to, b.0]);
            }
        }
    }
}

/// Adds what a cap covers beyond the end `at` of an open subpath, where
/// the path leaves in direction `outward`, going round from `from` on one
/// side of the path to `to` on the other.
fn cap(
    out: &mut Vec<Vec<[f64; 2]>>,
    at: Point,
    from: Point,
    to: Point,
    outward: Vec2,
    style: &StrokeStyle,
    tolerance: f64,
) {
    let half = style.width.max(0.0) / 2.0;
    match style.cap {
        LineCap::Butt => {}
        LineCap::Round => {
            let mut points = arc(
                at,
                half,
                (from - at) / half,
                std::f64::consts::PI,
                tolerance,
            );
            points[0] = from;
            *points.last_mut().expect("an arc has points") = to;
            points.push(at);
            push(out, points);
        }
        LineCap::Square => {
            let reach = outward * half;
            push(out, vec![from, from + reach, to + reach, to, at]);
        }
    }
}

/// Points along the circular arc about `center` of radius `radius`, from
/// unit direction `from` turning by `sweep` radians, counterclockwise in
/// the math sense when `sweep` is positive, close enough together that
/// the chords between them are within `tolerance` of the arc.
fn arc(center: Point, radius: f64, from: Vec2, sweep: f64, tolerance: f64) -> Vec<Point> {
    let step = if tolerance >= radius {
        std::f64::consts::FRAC_PI_2
    } else {
        (2.0 * (1.0 - tolerance / radius).acos()).min(std::f64::consts::FRAC_PI_4)
    };
    let count = (sweep.abs() / step).ceil().max(1.0) as usize;
    (0..=count)
        .map(|k| {
            let angle = sweep * k as f64 / count as f64;
            let (sin, cos) = angle.sin_cos();
            center + Vec2::new(from.x * cos - from.y * sin, from.x * sin + from.y * cos) * radius
        })
        .collect()
}

#[cfg(test)]
mod test {
    use std::f64::consts::PI;

    use svgtypes::{PathParser, PathSegment};

    use crate::{
        split_subpaths,
        BooleanOptions,
        FillRule,
        LineCap,
        LineJoin,
        PathMeasure,
        StrokeStyle,
    };

    fn measure(path: &str) -> PathMeasure {
        let segments: Vec<PathSegment> = PathParser::from(path).collect::<Result<_, _>>().unwrap();
        PathMeasure::new(segments)
    }

    fn style(width: f64, cap: LineCap, join: LineJoin) -> StrokeStyle {
        StrokeStyle { width, cap, join, miter_limit: 4.0 }
    }

    const TOLERANCE: f64 = 0.01;

    fn options() -> BooleanOptions {
        BooleanOptions { tolerance: TOLERANCE, ..BooleanOptions::default() }
    }

    fn area(path: &[PathSegment]) -> f64 {
        PathMeasure::new(path).area()
    }

    /// Whether `a` is `b` within the tolerance all along an outline
    /// `length` long.
    fn close_area(a: f64, b: f64, length: f64) -> bool {
        (a - b).abs() <= TOLERANCE * length
    }

    #[test]
    fn caps_of_a_line() {
        let line = measure("M 0 0 H 20");
        let with = |cap| area(&line.stroke_to_path(&style(4.0, cap, LineJoin::Miter), &options()));
        assert!((with(LineCap::Butt) - 80.0).abs() < 1e-9);
        assert!((with(LineCap::Square) - 96.0).abs() < 1e-9);
        assert!(close_area(with(LineCap::Round), 80.0 + 4.0 * PI, 60.0));
    }

    #[test]
    fn joins_of_a_right_angle() {
        // Two bands that overlap in a 2 by 2 square, and the outer corner
        let corner = measure("M 0 0 H 20 V 20");
        let with =
            |join| area(&corner.stroke_to_path(&style(4.0, LineCap::Butt, join), &options()));
        assert!((with(LineJoin::Miter) - 160.0).abs() < 1e-9);
        assert!((with(LineJoin::Bevel) - 158.0).abs() < 1e-9);
        assert!(close_area(with(LineJoin::Round), 156.0 + PI, 90.0));
        // Past the miter limit, a miter is a bevel
        let sharp = measure("M 0 0 L 20 1 L 0 2");
        let limited = StrokeStyle {
            miter_limit: 1.5,
            ..style(2.0, LineCap::Butt, LineJoin::Miter)
        };
        let beveled = style(2.0, LineCap::Butt, LineJoin::Bevel);
        assert!(
            (area(&sharp.stroke_to_path(&limited, &options()))
                - area(&sharp.stroke_to_path(&beveled, &options())))
            .abs()
                < 1e-9
        );
    }

    #[test]
    fn a_closed_path_strokes_to_a_ring() {
        let square = measure("M 0 0 H 20 V 20 H 0 Z");
        let ring = square.stroke_to_path(&style(4.0, LineCap::Round, LineJoin::Miter), &options());
        assert_eq!(split_subpaths(&ring).len(), 2);
        assert!((area(&ring) - (576.0 - 256.0)).abs() < 1e-9);
    }

    #[test]
    fn a_round_stroke_is_everything_within_half_its_width() {
        let path = measure("M 0 0 C 30 -40 60 40 90 0 A 30 20 0 1 1 60 30");
        let half = 6.0;
        let stroke = path.stroke_to_path(
            &style(2.0 * half, LineCap::Round, LineJoin::Round),
            &options(),
        );
        for subpath in split_subpaths(&stroke) {
            let outline = PathMeasure::new(&subpath);
            let n = (outline.total_length() / 0.1).ceil() as usize;
            for k in 0..=n {
                let p = outline
                    .point_at(outline.total_length() * k as f64 / n as f64)
                    .unwrap();
                let distance = path.nearest(p).unwrap().distance;
                assert!(
                    (distance - half).abs() <= TOLERANCE,
                    "{p:?} is {distance} away"
                );
            }
        }
    }

    #[test]
    fn points_have_caps() {
        let dot = measure("M 5 5 Z");
        let round = dot.stroke_to_path(&style(4.0, LineCap::Round, LineJoin::Round), &options());
        assert!(close_area(area(&round), 4.0 * PI, 13.0));
        let square = dot.stroke_to_path(&style(4.0, LineCap::Square, LineJoin::Round), &options());
        assert!((area(&square) - 16.0).abs() < 1e-9);
        assert!(dot
            .stroke_to_path(&style(4.0, LineCap::Butt, LineJoin::Round), &options())
            .is_empty());
    }

    #[test]
    fn offsets_grow_and_shrink() {
        let square = measure("M 0 0 H 20 V 20 H 0 Z");
        let at = |distance, join| area(&square.offset(distance, join, 4.0, &options()));
        assert!((at(2.0, LineJoin::Miter) - 576.0).abs() < 1e-9);
        assert!(close_area(
            at(2.0, LineJoin::Round),
            400.0 + 160.0 + 4.0 * PI,
            100.0
        ));
        assert!((at(-2.0, LineJoin::Round) - 256.0).abs() < 1e-9);
        assert!((at(0.0, LineJoin::Round) - 400.0).abs() < 1e-9);
        // Shrunk past its middle, nothing is left
        assert!(square
            .offset(-11.0, LineJoin::Miter, 4.0, &options())
            .is_empty());

        let circle = measure("M 10 0 A 10 10 0 0 1 -10 0 A 10 10 0 0 1 10 0 Z");
        assert!(close_area(
            area(&circle.offset(5.0, LineJoin::Round, 4.0, &options())),
            225.0 * PI,
            95.0
        ));
        assert!(close_area(
            area(&circle.offset(-5.0, LineJoin::Round, 4.0, &options())),
            25.0 * PI,
            32.0
        ));
    }

    #[test]
    fn offsets_follow_the_fill_rule_and_close_open_paths() {
        // A frame whose hole is only a hole with the even-odd rule
        let frame = measure("M 0 0 H 30 V 30 H 0 Z M 10 10 H 20 V 20 H 10 Z");
        let evenodd = BooleanOptions { fill_rule: FillRule::EvenOdd, ..options() };
        let grown = frame.offset(2.0, LineJoin::Miter, 4.0, &evenodd);
        // The outside grows to 34, the hole shrinks to 6
        assert!((area(&grown) - (34.0 * 34.0 - 36.0)).abs() < 1e-9);
        let filled = frame.offset(2.0, LineJoin::Miter, 4.0, &options());
        assert!((area(&filled) - 34.0 * 34.0).abs() < 1e-9);

        // An open path is grown as the shape filling it covers
        let open = measure("M 0 0 H 20 V 20 H 0");
        assert!((area(&open.offset(2.0, LineJoin::Miter, 4.0, &options())) - 576.0).abs() < 1e-9);
    }
}
