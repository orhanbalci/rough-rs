use kurbo::{CubicBez, ParamCurve, ParamCurveDeriv, Point, Vec2};
use svgtypes::PathSegment;

use crate::measure::{PathMeasure, Piece};

/// How many tolerances apart samples are at most, along the path. Half a
/// tolerance keeps the curves within it between samples too.
const SPACING: f64 = 0.5;

/// Fewest samples taken along a curve or an arc.
const CURVE_SAMPLES: usize = 4;

/// Newton steps that move the samples' parameters closer to the curve
/// before a stretch is given up on.
const REPARAMETERIZE: usize = 4;

/// A point along the path and the unit tangent of the path there.
#[derive(Clone, Copy, Debug)]
struct Sample {
    point: Point,
    tangent: Vec2,
}

/// A segment the fit draws.
enum Fitted {
    Line(Point),
    Cubic(CubicBez),
}

impl PathMeasure {
    /// The path redrawn with fewer segments, cubic curves that stay within
    /// `tolerance` of it: many short segments, as a freehand stroke, a
    /// traced outline or the lines of [`flatten`](Self::flatten) have,
    /// become a few long curves.
    ///
    /// The path keeps its subpaths, a closed subpath stays closed, and it
    /// keeps its corners: the joins where its direction turns by more than
    /// `corner_angle` degrees. Between corners it is fitted with cubic
    /// curves, except where it is straight, which stays a line. A stretch
    /// between corners that curves would not draw with fewer segments, as a
    /// single segment or a path drawn by hand, keeps its segments.
    ///
    /// A `corner_angle` of 180 smooths every join, as Paper.js does. Noisy
    /// input, as a freehand stroke, turns sharply from sample to sample and
    /// wants a large one; a path with real corners, as an outline, wants
    /// one around 60.
    ///
    /// The result is in absolute coordinates.
    ///
    /// # Algorithm
    ///
    /// Each stretch between corners is sampled into points no more than half
    /// a `tolerance` apart, each with the path's direction there, and fitted
    /// with Philip J. Schneider's method from *An Algorithm for
    /// Automatically Fitting Digitized Curves* (Graphics Gems, 1990), the
    /// one Paper.js's `simplify` uses:
    ///
    /// 1. Give every sample a parameter by its distance along the polyline
    ///    through the samples, from 0 at the first to 1 at the last.
    /// 2. With the curve's ends on the first and last samples and its
    ///    control points along the path's direction there, pick the two
    ///    control point distances by least squares, so the curve at each
    ///    sample's parameter is as close to the sample as it can be.
    /// 3. If every sample is within `tolerance` of the curve, keep it.
    ///    Otherwise move each parameter to the curve's nearest point to its
    ///    sample with a Newton step and fit again, up to four times.
    ///
    /// Schneider splits a stretch that does not fit at its furthest sample
    /// and fits both halves, which cuts curves at uneven places and takes
    /// more of them than needed. Instead, each curve here takes the longest
    /// stretch from where the last one ended that one curve fits: doubling
    /// the stretch until a fit fails, then a binary search between the last
    /// that fit and the first that did not. The curves on both sides of a
    /// cut end along the path's direction there, so they join smoothly, as
    /// they do at the start of a closed subpath with no corner there.
    ///
    /// A sample is checked against the curve point at its parameter, which
    /// is never nearer than the curve's nearest point, so every sample is
    /// within `tolerance` of the result. Between samples half a tolerance
    /// apart the curve stays close too; the tests measure both the result's
    /// distance from the path and the path's from the result.
    ///
    /// ```
    /// use svg_path_ops::svgtypes::PathParser;
    /// use svg_path_ops::PathMeasure;
    ///
    /// // A circle drawn with 64 lines
    /// let mut path = String::from("M 100 0");
    /// for i in 1..64 {
    ///     let angle = i as f64 / 64.0 * std::f64::consts::TAU;
    ///     path += &format!(" L {} {}", 100.0 * angle.cos(), 100.0 * angle.sin());
    /// }
    /// path += " Z";
    /// let segments: Vec<_> = PathParser::from(path.as_str()).collect::<Result<_, _>>()?;
    ///
    /// let simplified = PathMeasure::new(&segments).simplify(1.0, 60.0);
    /// // M, three curves and Z
    /// assert_eq!(simplified.len(), 5);
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn simplify(&self, tolerance: f64, corner_angle: f64) -> Vec<PathSegment> {
        let tolerance = tolerance.max(1e-9);
        let corner = corner_angle.to_radians();
        let mut path = Vec::new();
        for subpath in self.subpaths() {
            let start = subpath[0].from;
            path.push(PathSegment::MoveTo { abs: true, x: start.x, y: start.y });
            let closed = subpath.last().is_some_and(|piece| piece.closes);
            let drawing: Vec<&Piece> = subpath.iter().filter(|p| p.length > 0.0).collect();
            let turns = |a: &Piece, b: &Piece| turn(a, b) > corner;

            // The runs between corners, as ranges of `drawing`
            let mut runs = Vec::new();
            let mut first = 0;
            for i in 1..drawing.len() {
                if turns(drawing[i - 1], drawing[i]) {
                    runs.push(first..i);
                    first = i;
                }
            }
            if !drawing.is_empty() {
                runs.push(first..drawing.len());
            }

            // Where a closed subpath turns smoothly at its start, the runs
            // on both sides of it leave and arrive along one tangent: a run
            // of one segment keeps its own, otherwise it is halfway between
            // the path's directions there
            let seam = match (drawing.first(), drawing.last()) {
                (Some(first), Some(last)) if closed && !turns(last, first) => {
                    let out = unit(first.shape.direction(0.0));
                    let into = unit(last.shape.direction(1.0));
                    Some(if runs[0].len() == 1 {
                        out
                    } else if runs[runs.len() - 1].len() == 1 {
                        into
                    } else {
                        unit(Some(into + out))
                    })
                }
                _ => None,
            };

            for run in runs {
                let pieces = &drawing[run.clone()];
                let original = |piece: &&Piece| {
                    if piece.closes {
                        PathSegment::ClosePath { abs: true }
                    } else {
                        match piece.shape.segment(0.0, 1.0) {
                            PathSegment::EllipticalArc {
                                rx,
                                ry,
                                x_axis_rotation,
                                large_arc,
                                sweep,
                                ..
                            } => {
                                // The arc's own end, not the one its shape
                                // comes back to
                                PathSegment::EllipticalArc {
                                    abs: true,
                                    rx,
                                    ry,
                                    x_axis_rotation,
                                    large_arc,
                                    sweep,
                                    x: piece.to.x,
                                    y: piece.to.y,
                                }
                            }
                            segment => segment,
                        }
                    }
                };
                if pieces.len() == 1 {
                    path.extend(pieces.iter().map(original));
                    continue;
                }
                let mut samples = sample(pieces, tolerance);
                if let Some(seam) = seam {
                    if run.start == 0 {
                        samples[0].tangent = seam;
                    }
                    if run.end == drawing.len() {
                        samples.last_mut().expect("a run has samples").tangent = seam;
                    }
                }
                let mut fitted = Vec::new();
                let n = samples.len();
                fit(
                    &samples,
                    samples[0].tangent,
                    -samples[n - 1].tangent,
                    tolerance,
                    &mut fitted,
                );
                // A run drawn well already, as by hand, keeps its segments
                if fitted.len() >= pieces.len() {
                    path.extend(pieces.iter().map(original));
                    continue;
                }
                path.extend(fitted.into_iter().map(|segment| match segment {
                    Fitted::Line(p) => PathSegment::LineTo { abs: true, x: p.x, y: p.y },
                    Fitted::Cubic(c) => PathSegment::CurveTo {
                        abs: true,
                        x1: c.p1.x,
                        y1: c.p1.y,
                        x2: c.p2.x,
                        y2: c.p2.y,
                        x: c.p3.x,
                        y: c.p3.y,
                    },
                }));
            }
            if closed {
                // A fitted line back to the start is the close path's line
                if let Some(PathSegment::LineTo { x, y, .. }) = path.last() {
                    if (*x, *y) == (start.x, start.y) {
                        path.pop();
                    }
                }
                if !matches!(path.last(), Some(PathSegment::ClosePath { .. })) {
                    path.push(PathSegment::ClosePath { abs: true });
                }
            }
        }
        path
    }
}

/// The angle the path turns by, in radians from 0 to π, where piece `a`
/// ends and piece `b` starts.
fn turn(a: &Piece, b: &Piece) -> f64 {
    match (a.shape.direction(1.0), b.shape.direction(0.0)) {
        (Some(into), Some(out)) => into.cross(out).atan2(into.dot(out)).abs(),
        _ => 0.0,
    }
}

/// `v` scaled to length 1, or zero when there is no direction.
fn unit(v: Option<Vec2>) -> Vec2 {
    match v {
        Some(v) if v.hypot() > 0.0 => v / v.hypot(),
        _ => Vec2::ZERO,
    }
}

/// Points along `pieces`, which follow each other, no more than
/// `SPACING × tolerance` apart, with the unit tangent at each. Where two
/// pieces meet, the tangent is halfway between theirs.
fn sample(pieces: &[&Piece], tolerance: f64) -> Vec<Sample> {
    let mut samples = Vec::new();
    for (i, piece) in pieces.iter().enumerate() {
        let straight = matches!(piece.shape, crate::measure::PieceShape::Line(_));
        let by_length = (piece.length / (SPACING * tolerance)).ceil() as usize;
        let count = if straight {
            by_length.max(1)
        } else {
            by_length.max(CURVE_SAMPLES)
        };
        let tangent = |t: f64| unit(piece.shape.direction(t));
        for k in 0..count {
            let t = k as f64 / count as f64;
            let point = if k == 0 {
                piece.from
            } else {
                piece.shape.eval(t)
            };
            let mut sample = Sample { point, tangent: tangent(t) };
            if k == 0 && i > 0 {
                let before = unit(pieces[i - 1].shape.direction(1.0));
                sample.tangent = unit(Some(before + sample.tangent));
            }
            samples.push(sample);
        }
    }
    let last = pieces.last().expect("a run has pieces");
    samples.push(Sample {
        point: last.to,
        tangent: unit(last.shape.direction(1.0)),
    });
    samples
}

/// Fits curves to `samples` from the first to the last, starting along
/// `tan1` and arriving along `-tan2`, and appends them to `out`.
///
/// Each curve covers the longest stretch from where the last one ended that
/// one curve fits, found by galloping and then binary search, and ends along the path's
/// direction there, so the next one joins it smoothly.
fn fit(samples: &[Sample], tan1: Vec2, tan2: Vec2, tolerance: f64, out: &mut Vec<Fitted>) {
    let last = samples.len() - 1;
    let mut start = 0;
    while start < last {
        let leaving = if start == 0 {
            tan1
        } else {
            samples[start].tangent
        };
        let arriving = |end: usize| {
            if end == last {
                tan2
            } else {
                -samples[end].tangent
            }
        };
        let attempt =
            |end: usize| fit_one(&samples[start..=end], leaving, arriving(end), tolerance);

        // Gallop: double the stretch until one curve no longer fits it,
        // then binary search between the last that fit and the first that
        // did not. Two samples always fit.
        let mut lo = start + 1;
        let mut best = attempt(lo).expect("two samples fit");
        let mut hi = None;
        let mut step = 1;
        while lo < last {
            let end = (lo + step).min(last);
            match attempt(end) {
                Some(fitted) => (lo, best) = (end, fitted),
                None => {
                    hi = Some(end);
                    break;
                }
            }
            step *= 2;
        }
        if let Some(mut hi) = hi {
            while hi - lo > 1 {
                let mid = lo + (hi - lo) / 2;
                match attempt(mid) {
                    Some(fitted) => (lo, best) = (mid, fitted),
                    None => hi = mid,
                }
            }
        }
        let (end, fitted) = (lo, best);
        out.push(fitted);
        start = end;
    }
}

/// One segment through `samples`, leaving along `tan1` and arriving along
/// `-tan2`, when one is within `tolerance` of every sample.
fn fit_one(samples: &[Sample], tan1: Vec2, tan2: Vec2, tolerance: f64) -> Option<Fitted> {
    let n = samples.len();
    let (p0, p3) = (samples[0].point, samples[n - 1].point);

    // A straight run stays a line
    if is_straight(samples) {
        return Some(Fitted::Line(p3));
    }
    if n == 2 {
        let third = (p3 - p0).hypot() / 3.0;
        return Some(Fitted::Cubic(CubicBez::new(
            p0,
            p0 + tan1 * third,
            p3 + tan2 * third,
            p3,
        )));
    }

    let mut u = chord_parameters(samples);
    for iteration in 0..=REPARAMETERIZE {
        let curve = generate(samples, &u, tan1, tan2);
        let error = max_error(samples, &curve, &u);
        if error <= tolerance * tolerance {
            return Some(Fitted::Cubic(curve));
        }
        // Newton steps help when the curve is nearly right, not when it is
        // far off
        if iteration == REPARAMETERIZE || error > 16.0 * tolerance * tolerance {
            break;
        }
        if !reparameterize(samples, &curve, &mut u) {
            break;
        }
    }
    None
}

/// Whether the samples lie on the line from the first to the last and run
/// along it: the path there is a straight line, drawn in one or more
/// segments.
fn is_straight(samples: &[Sample]) -> bool {
    let (p0, p3) = (samples[0].point, samples[samples.len() - 1].point);
    let chord = p3 - p0;
    let length = chord.hypot();
    if length == 0.0 {
        return false;
    }
    let direction = chord / length;
    let scale = 1e-9 * length.max(1.0);
    let mut along = 0.0;
    samples.iter().all(|sample| {
        let offset = sample.point - p0;
        let at = offset.dot(direction);
        let ahead = at >= along - scale;
        along = at;
        offset.cross(direction).abs() <= scale
            && ahead
            && sample.tangent.cross(direction).abs() <= 1e-9
            && sample.tangent.dot(direction) > 0.0
    })
}

/// Each sample's distance along the polyline through the samples, as a
/// fraction of the polyline's length.
fn chord_parameters(samples: &[Sample]) -> Vec<f64> {
    let mut u = Vec::with_capacity(samples.len());
    let mut total = 0.0;
    u.push(0.0);
    for pair in samples.windows(2) {
        total += (pair[1].point - pair[0].point).hypot();
        u.push(total);
    }
    if total > 0.0 {
        u.iter_mut().for_each(|value| *value /= total);
    }
    u
}

/// The cubic from the first sample to the last, leaving along `tan1` and
/// arriving along `-tan2`, whose control points are as far along them as
/// puts it closest to the samples at parameters `u`, by least squares.
fn generate(samples: &[Sample], u: &[f64], tan1: Vec2, tan2: Vec2) -> CubicBez {
    let n = samples.len();
    let (p0, p3) = (samples[0].point, samples[n - 1].point);

    // The normal equations for the control point distances α1 and α2:
    // the curve at u is B0 p0 + B1 (p0 + α1 tan1) + B2 (p3 + α2 tan2) + B3 p3
    let (mut c00, mut c01, mut c11, mut x0, mut x1) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for (sample, &t) in samples.iter().zip(u) {
        let mt = 1.0 - t;
        let (b0, b1, b2, b3) = (mt * mt * mt, 3.0 * t * mt * mt, 3.0 * t * t * mt, t * t * t);
        let a1 = tan1 * b1;
        let a2 = tan2 * b2;
        c00 += a1.dot(a1);
        c01 += a1.dot(a2);
        c11 += a2.dot(a2);
        let rest = sample.point.to_vec2() - (p0.to_vec2() * (b0 + b1) + p3.to_vec2() * (b2 + b3));
        x0 += a1.dot(rest);
        x1 += a2.dot(rest);
    }
    let det = c00 * c11 - c01 * c01;
    let chord = (p3 - p0).hypot();
    let (mut alpha1, mut alpha2) = if det.abs() > 1e-12 * c00.max(c11).powi(2) {
        ((x0 * c11 - x1 * c01) / det, (c00 * x1 - c01 * x0) / det)
    } else {
        // The tangents are parallel: share one distance
        let alpha = (x0 + x1) / (c00 + c11 + 2.0 * c01).max(f64::MIN_POSITIVE);
        (alpha, alpha)
    };
    // A negative or tiny distance, or control points past the far end,
    // make a loop or a cusp: fall back to a third of the chord, the
    // distance that draws a straight line straight
    let epsilon = 1e-12 * chord;
    let overshoots = |alpha: f64| alpha > 3.0 * chord;
    if alpha1 < epsilon || alpha2 < epsilon || overshoots(alpha1) || overshoots(alpha2) {
        alpha1 = chord / 3.0;
        alpha2 = chord / 3.0;
    }
    CubicBez::new(p0, p0 + tan1 * alpha1, p3 + tan2 * alpha2, p3)
}

/// The largest squared distance between a sample and the curve at its
/// parameter.
fn max_error(samples: &[Sample], curve: &CubicBez, u: &[f64]) -> f64 {
    samples
        .iter()
        .zip(u)
        .map(|(sample, &t)| (curve.eval(t) - sample.point).hypot2())
        .fold(0.0, f64::max)
}

/// Moves each parameter a Newton step towards the curve's nearest point to
/// its sample. `false` when the parameters stop increasing along the
/// samples, which means the fit is poor and the samples need splitting.
fn reparameterize(samples: &[Sample], curve: &CubicBez, u: &mut [f64]) -> bool {
    let d1 = curve.deriv();
    let d2 = d1.deriv();
    for (sample, t) in samples.iter().zip(u.iter_mut()) {
        // Minimise |Q(t) − P|²: f(t) = (Q − P)·Q′ = 0
        let diff = curve.eval(*t) - sample.point;
        let q1 = d1.eval(*t).to_vec2();
        let q2 = d2.eval(*t).to_vec2();
        let f = diff.dot(q1);
        let df = q1.dot(q1) + diff.dot(q2);
        if df.abs() > f64::EPSILON {
            *t = (*t - f / df).clamp(0.0, 1.0);
        }
    }
    u.windows(2).all(|pair| pair[0] <= pair[1])
}

#[cfg(test)]
mod test {
    use svgtypes::{PathParser, PathSegment};

    use crate::write::{write_path, WriteOptions};
    use crate::PathMeasure;

    fn parse(path: &str) -> Vec<PathSegment> {
        PathParser::from(path).collect::<Result<_, _>>().unwrap()
    }

    fn written(segments: &[PathSegment]) -> String {
        write_path(segments, &WriteOptions::default())
    }

    /// A circle of radius 100 drawn with `sides` lines.
    fn polygon(sides: usize) -> String {
        let mut path = String::from("M 100 0");
        for i in 1..sides {
            let angle = i as f64 / sides as f64 * std::f64::consts::TAU;
            path += &format!(" L {} {}", 100.0 * angle.cos(), 100.0 * angle.sin());
        }
        path + " Z"
    }

    /// The furthest any point of `a` is from `b`, both ways.
    fn distance(a: &PathMeasure, b: &PathMeasure) -> f64 {
        let one_way = |a: &PathMeasure, b: &PathMeasure| {
            let n = (a.total_length() / 0.05).ceil() as usize;
            (0..=n)
                .map(|i| {
                    let point = a.point_at(a.total_length() * i as f64 / n as f64).unwrap();
                    b.nearest(point).unwrap().distance
                })
                .fold(0.0, f64::max)
        };
        one_way(a, b).max(one_way(b, a))
    }

    fn check(path: &str, tolerance: f64, corner_angle: f64) -> Vec<PathSegment> {
        let measure = PathMeasure::new(parse(path));
        let simplified = measure.simplify(tolerance, corner_angle);
        let error = distance(&measure, &PathMeasure::new(&simplified));
        assert!(error <= tolerance, "{error} > {tolerance}");
        simplified
    }

    #[test]
    fn lines_of_a_circle_become_a_few_curves() {
        let simplified = check(&polygon(64), 0.5, 60.0);
        assert!(matches!(simplified[0], PathSegment::MoveTo { .. }));
        assert!(matches!(
            simplified.last(),
            Some(PathSegment::ClosePath { .. })
        ));
        let curves = simplified
            .iter()
            .filter(|s| matches!(s, PathSegment::CurveTo { .. }))
            .count();
        assert_eq!(curves, simplified.len() - 2);
        assert!(curves <= 4, "{}", written(&simplified));
    }

    #[test]
    fn a_tighter_tolerance_takes_more_curves() {
        let loose = check(&polygon(256), 1.0, 60.0);
        let tight = check(&polygon(256), 0.01, 60.0);
        assert!(tight.len() > loose.len());
    }

    #[test]
    fn a_smooth_closed_path_joins_smoothly_where_it_closes() {
        let simplified = PathMeasure::new(parse(&polygon(64))).simplify(0.5, 60.0);
        let measure = PathMeasure::new(&simplified);
        let start = measure.tangent_at(0.0).unwrap();
        let end = measure.tangent_at(measure.total_length() - 1e-9).unwrap();
        assert!((start - end).length() < 1e-6, "{start:?} {end:?}");
    }

    #[test]
    fn keeps_corners() {
        // A square with each side drawn in four lines
        let mut path = String::from("M 0 0");
        for (x, y) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
            for _ in 0..4 {
                path += &format!(" l {} {}", 25 * x, 25 * y);
            }
        }
        let simplified = check(&path, 0.5, 60.0);
        assert_eq!(
            written(&simplified),
            "M 0 0 L 100 0 L 100 100 L 0 100 L 0 0"
        );
        let closed = check(&(path + " Z"), 0.5, 60.0);
        assert_eq!(written(&closed), "M 0 0 L 100 0 L 100 100 L 0 100 Z");
    }

    #[test]
    fn a_large_corner_angle_rounds_corners() {
        let path = "M 0 0 L 50 0 L 100 0 L 100 50 L 100 100";
        let kept = check(path, 1.0, 60.0);
        assert_eq!(written(&kept), "M 0 0 L 100 0 L 100 100");
        let rounded = check(path, 1.0, 180.0);
        assert!(rounded
            .iter()
            .any(|s| matches!(s, PathSegment::CurveTo { .. })));
    }

    #[test]
    fn a_noisy_stroke_becomes_a_few_curves() {
        // A sine wave with a pixel of noise, sampled every pixel
        let mut path = String::new();
        let mut seed = 1u64;
        for i in 0..300 {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let noise = (seed >> 33) as f64 / (1u64 << 31) as f64 - 0.5;
            let x = i as f64;
            let command = if i == 0 { "M" } else { "L" };
            path += &format!("{command} {x} {} ", 40.0 * (x / 30.0).sin() + noise);
        }
        let simplified = check(&path, 2.0, 180.0);
        assert!(simplified.len() <= 8, "{}", written(&simplified));
    }

    #[test]
    fn a_flattened_path_becomes_curves_again() {
        let ellipse = "M 0 0 A 100 50 30 1 1 0 1 Z M 300 0 C 350 -100 450 100 500 0";
        let lines = PathMeasure::new(parse(ellipse)).flatten(0.01);
        let simplified = check(&written(&lines), 0.1, 60.0);
        assert!(simplified.len() * 10 < lines.len(), "{}", simplified.len());
    }

    #[test]
    fn keeps_segments_that_are_already_simple() {
        for path in [
            "M 0 0 C 10 20 30 20 40 0",
            "M 0 0 A 10 10 0 0 1 20 0 Q 30 20 40 0",
            "M 0 0 L 10 0 L 10 10 Z M 20 0 L 30 0",
        ] {
            let simplified = check(path, 0.5, 60.0);
            let absolute: Vec<_> = crate::absolutize(parse(path).iter()).collect();
            assert_eq!(written(&simplified), written(&absolute));
        }
    }

    #[test]
    fn keeps_subpaths() {
        let path = format!("{} M 300 0 L 400 0 L 500 1 L 600 3", polygon(32));
        let simplified = check(&path, 0.5, 60.0);
        let moves = simplified
            .iter()
            .filter(|s| matches!(s, PathSegment::MoveTo { .. }))
            .count();
        assert_eq!(moves, 2);
    }

    #[test]
    fn a_path_that_draws_nothing() {
        assert!(PathMeasure::new(parse("")).simplify(1.0, 60.0).is_empty());
        let dot = PathMeasure::new(parse("M 5 5 L 5 5 Z")).simplify(1.0, 60.0);
        assert_eq!(written(&dot), "M 5 5 Z");
    }
}
