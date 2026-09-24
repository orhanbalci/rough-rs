use std::borrow::Borrow;

use i_overlay::core::fill_rule::FillRule as OverlayFill;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::scale::FixedScaleFloatOverlay;
use i_overlay::float::single::SingleFloatOverlay;
use svgtypes::PathSegment;

use crate::context::segments_with_context;
use crate::{FillRule, PathMeasure};

/// Which boolean operation [`boolean`] applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BooleanOp {
    /// Everything either path covers.
    Union,
    /// What both paths cover.
    Intersect,
    /// What the first path covers and the second does not.
    Difference,
    /// What exactly one of the paths covers.
    Xor,
}

/// How [`boolean`] reads its paths and writes the result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BooleanOptions {
    /// How both paths are filled, and so what they cover.
    pub fill_rule: FillRule,
    /// How far the result may be from the exact outline of the operation.
    pub tolerance: f64,
    /// Whether the result is drawn with cubic curves fitted to it, rather
    /// than the straight lines the operation works with.
    pub curves: bool,
}

impl Default for BooleanOptions {
    /// Nonzero fill, a tolerance of 0.01 and curves.
    fn default() -> Self {
        BooleanOptions {
            fill_rule: FillRule::NonZero,
            tolerance: 0.01,
            curves: true,
        }
    }
}

/// The share of the tolerance spent on turning the paths into lines.
pub(crate) const FLATTEN_SHARE: f64 = 0.25;

/// The share of the tolerance spent on fitting curves to the result. The
/// two leave a twentieth spare, since curve fitting checks its distance at
/// samples along the lines.
pub(crate) const FIT_SHARE: f64 = 0.7;

/// Joins that turn more than this many degrees stay corners when curves
/// are fitted to the result.
pub(crate) const CORNER_ANGLE: f64 = 30.0;

/// The area `op` makes of the areas paths `a` and `b` cover, as a path of
/// closed subpaths within `options.tolerance` of its exact outline.
///
/// Outlines are drawn clockwise on screen and the holes in them the other
/// way, so the result fills the same with either fill rule. Open subpaths
/// are filled as if closed, as SVG fills them. Specks and slivers smaller
/// than the tolerance, which touching outlines can leave, are left out.
///
/// # Algorithm
///
/// Both paths are turned into straight lines with
/// [`PathMeasure::flatten`], none further than a quarter of the tolerance
/// from the curve it replaces, and the polygons are combined by
/// [i_overlay], which snaps them to an integer grid a thousandth of the
/// tolerance fine and finds the result exactly on it. Its outline is made of pieces of the two polygons,
/// so it is within a quarter of the tolerance of the exact outline.
///
/// With `options.curves`, the outline is then redrawn by
/// [`PathMeasure::simplify`] within 0.7 of the tolerance, keeping as
/// corners the joins that turn more than 30 degrees, where the two paths
/// meet or their own corners are. So the curves are within the tolerance
/// of the exact outline, with a little to spare, and straight edges stay
/// straight lines.
///
/// Every point of the result's outline is within the tolerance of the
/// outlines of `a` or `b`, and every point the result fills differently
/// from the exact operation is within the tolerance of its outline. Where
/// the two outlines touch or run along each other, slivers thinner than
/// the tolerance may come or go.
///
/// [i_overlay]: https://docs.rs/i_overlay
///
/// ```
/// use svg_path_ops::svgtypes::PathParser;
/// use svg_path_ops::{boolean, BooleanOp, BooleanOptions, PathMeasure};
///
/// let parse = |data| PathParser::from(data).collect::<Result<Vec<_>, _>>();
/// let square = parse("M 0 0 H 20 V 20 H 0 Z")?;
/// let circle = parse("M 30 10 A 10 10 0 0 1 10 10 A 10 10 0 0 1 30 10 Z")?;
///
/// let union = boolean(
///     &square,
///     &circle,
///     BooleanOp::Union,
///     &BooleanOptions::default(),
/// );
/// let area = PathMeasure::new(&union).area();
/// // The square, and the half of the circle outside it
/// let expected = 400.0 + std::f64::consts::PI * 100.0 / 2.0;
/// assert!((area - expected).abs() < 0.5);
/// # Ok::<(), svg_path_ops::svgtypes::Error>(())
/// ```
pub fn boolean(
    a: impl IntoIterator<Item = impl Borrow<PathSegment>>,
    b: impl IntoIterator<Item = impl Borrow<PathSegment>>,
    op: BooleanOp,
    options: &BooleanOptions,
) -> Vec<PathSegment> {
    let tolerance = options.tolerance.max(1e-9);
    let flatten = flatten_tolerance(options);
    let a = polygons(&PathMeasure::new(a), flatten);
    let b = polygons(&PathMeasure::new(b), flatten);
    let rule = match op {
        BooleanOp::Union => OverlayRule::Union,
        BooleanOp::Intersect => OverlayRule::Intersect,
        BooleanOp::Difference => OverlayRule::Difference,
        BooleanOp::Xor => OverlayRule::Xor,
    };
    let fill = match options.fill_rule {
        FillRule::NonZero => OverlayFill::NonZero,
        FillRule::EvenOdd => OverlayFill::EvenOdd,
    };
    let shapes = combine(&a, &b, rule, fill, tolerance);
    finish(&shapes, tolerance, options.curves)
}

/// Polygons closer than this share of the tolerance are the same: i_overlay
/// snaps every point to a grid this fine, so edges that should coincide,
/// computed two ways, meet instead of leaving a hairline gap.
const GRID_SHARE: f64 = 1e-3;

/// `a` and `b` combined by i_overlay with `rule`, on a grid a thousandth of
/// `tolerance` fine, or as fine as i_overlay can go for paths too large
/// for that: about two billionths of their size.
///
/// This uses i_overlay's 32-bit integer engine. Its 64-bit one leaves
/// polygons that meet along a shared edge, with a vertex of one inside an
/// edge of the other, apart instead of united.
pub(crate) fn combine(
    a: &Vec<Vec<[f64; 2]>>,
    b: &Vec<Vec<[f64; 2]>>,
    rule: OverlayRule,
    fill: OverlayFill,
    tolerance: f64,
) -> Vec<Vec<Vec<[f64; 2]>>> {
    let scale = 1.0 / (tolerance * GRID_SHARE);
    a.overlay_with_fixed_scale_as::<i32>(b, rule, fill, scale)
        .unwrap_or_else(|_| a.overlay_as::<i32>(b, rule, fill))
}

/// The tolerance spent on turning paths into lines, from the whole.
pub(crate) fn flatten_tolerance(options: &BooleanOptions) -> f64 {
    let tolerance = options.tolerance.max(1e-9);
    if options.curves {
        tolerance * FLATTEN_SHARE
    } else {
        tolerance
    }
}

/// The shapes i_overlay made as a path, without specks smaller than
/// `tolerance`, and with curves fitted to it when `curves` is set.
pub(crate) fn finish(
    shapes: &[Vec<Vec<[f64; 2]>>],
    tolerance: f64,
    curves: bool,
) -> Vec<PathSegment> {
    let mut path = Vec::new();
    for contour in shapes.iter().flatten() {
        // A speck smaller than the tolerance, left where outlines touch,
        // is within the tolerance of not being there
        let (low, high) = contour.iter().fold(
            ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]),
            |(low, high), [x, y]| {
                (
                    [low[0].min(*x), low[1].min(*y)],
                    [high[0].max(*x), high[1].max(*y)],
                )
            },
        );
        if (high[0] - low[0]).hypot(high[1] - low[1]) <= tolerance {
            continue;
        }
        for (i, [x, y]) in contour.iter().copied().enumerate() {
            path.push(if i == 0 {
                PathSegment::MoveTo { abs: true, x, y }
            } else {
                PathSegment::LineTo { abs: true, x, y }
            });
        }
        path.push(PathSegment::ClosePath { abs: true });
    }
    if curves {
        PathMeasure::new(&path).simplify(tolerance * FIT_SHARE, CORNER_ANGLE)
    } else {
        path
    }
}

/// The subpaths of the path measured by `measure` as closed polygons, none
/// of their lines further than `tolerance` from the path.
pub(crate) fn polygons(measure: &PathMeasure, tolerance: f64) -> Vec<Vec<[f64; 2]>> {
    let lines = measure.flatten(tolerance);
    let mut polygons: Vec<Vec<[f64; 2]>> = Vec::new();
    for context in segments_with_context(&lines) {
        let point = [context.end.x, context.end.y];
        match context.segment {
            PathSegment::MoveTo { .. } => polygons.push(vec![point]),
            // The polygon closes itself
            PathSegment::ClosePath { .. } => {}
            _ => {
                if let Some(polygon) = polygons.last_mut() {
                    polygon.push(point);
                }
            }
        }
    }
    // A polygon of fewer than three points covers nothing
    polygons.retain(|polygon| polygon.len() >= 3);
    polygons
}

#[cfg(test)]
mod test {
    use euclid::default::Point2D;
    use svgtypes::{PathParser, PathSegment};

    use super::{boolean, BooleanOp, BooleanOptions};
    use crate::{split_subpaths, FillRule, PathMeasure};

    fn parse(path: &str) -> Vec<PathSegment> {
        PathParser::from(path).collect::<Result<_, _>>().unwrap()
    }

    const SQUARE: &str = "M 0 0 H 40 V 40 H 0 Z";
    /// A circle of radius 15 about 40 20, half inside the square.
    const CIRCLE: &str = "M 55 20 A 15 15 0 0 1 25 20 A 15 15 0 0 1 55 20 Z";

    fn options(tolerance: f64) -> BooleanOptions {
        BooleanOptions { tolerance, ..BooleanOptions::default() }
    }

    /// Checks `op` on `a` and `b` against point by point membership and the
    /// original outlines, and returns the result.
    fn check(a: &str, b: &str, op: BooleanOp, options: &BooleanOptions) -> Vec<PathSegment> {
        let (a, b) = (parse(a), parse(b));
        let result = boolean(&a, &b, op, options);
        let tolerance = options.tolerance;
        let outlines: Vec<PathMeasure> = split_subpaths(a.iter().chain(&b))
            .iter()
            .map(PathMeasure::new)
            .collect();
        let near_outline = |p: Point2D<f64>| {
            outlines
                .iter()
                .filter_map(|m| m.nearest(p))
                .any(|n| n.distance <= tolerance)
        };

        // The outline lies along the original ones
        for subpath in split_subpaths(&result) {
            let measure = PathMeasure::new(&subpath);
            let n = (measure.total_length() / 0.05).ceil() as usize;
            for k in 0..=n {
                let p = measure
                    .point_at(measure.total_length() * k as f64 / n as f64)
                    .unwrap();
                assert!(near_outline(p), "{p:?} is off the outlines");
            }
        }
        // It fills what the operation does, away from the outlines
        let (ma, mb, mr) = (
            PathMeasure::new(&a),
            PathMeasure::new(&b),
            PathMeasure::new(&result),
        );
        for i in 0..80 {
            for j in 0..60 {
                let p = Point2D::new(
                    -10.0 + f64::from(i) * 0.9 + 0.13,
                    -10.0 + f64::from(j) + 0.37,
                );
                let (in_a, in_b) = (
                    ma.contains(p, options.fill_rule),
                    mb.contains(p, options.fill_rule),
                );
                let expected = match op {
                    BooleanOp::Union => in_a || in_b,
                    BooleanOp::Intersect => in_a && in_b,
                    BooleanOp::Difference => in_a && !in_b,
                    BooleanOp::Xor => in_a != in_b,
                };
                if mr.contains(p, FillRule::NonZero) != expected {
                    assert!(near_outline(p), "{p:?} filled wrongly");
                }
            }
        }
        result
    }

    #[test]
    fn areas_of_the_four_operations() {
        let half = std::f64::consts::PI * 225.0 / 2.0;
        for (op, area) in [
            (BooleanOp::Union, 1600.0 + half),
            (BooleanOp::Intersect, half),
            (BooleanOp::Difference, 1600.0 - half),
            (BooleanOp::Xor, 1600.0),
        ] {
            let result = check(SQUARE, CIRCLE, op, &options(0.01));
            let measured = PathMeasure::new(&result).area();
            // Within the tolerance all round the outline
            assert!(
                (measured - area).abs() < 0.01 * 250.0,
                "{op:?} {measured} {area}"
            );
        }
    }

    #[test]
    fn holes_run_against_their_outlines() {
        let result = check(
            SQUARE,
            "M 30 20 A 10 10 0 0 1 10 20 A 10 10 0 0 1 30 20 Z",
            BooleanOp::Difference,
            &options(0.01),
        );
        let directions: Vec<bool> = split_subpaths(&result)
            .iter()
            .map(|subpath| PathMeasure::new(subpath).is_clockwise())
            .collect();
        assert_eq!(directions, [true, false]);
    }

    #[test]
    fn curves_or_lines() {
        let curves = check(SQUARE, CIRCLE, BooleanOp::Union, &options(0.01));
        let lines = check(
            SQUARE,
            CIRCLE,
            BooleanOp::Union,
            &BooleanOptions { curves: false, ..options(0.01) },
        );
        assert!(lines.iter().all(|s| matches!(
            s,
            PathSegment::MoveTo { .. } | PathSegment::LineTo { .. } | PathSegment::ClosePath { .. }
        )));
        assert!(curves
            .iter()
            .any(|s| matches!(s, PathSegment::CurveTo { .. })));
        assert!(curves.len() * 2 < lines.len());
        // The square's edges stay straight
        assert!(curves
            .iter()
            .any(|s| matches!(s, PathSegment::LineTo { .. })));
    }

    #[test]
    fn follows_the_fill_rule() {
        // Two squares drawn the same way, overlapping in the middle
        let squares = "M 0 0 H 30 V 30 H 0 Z M 15 0 H 45 V 30 H 15 Z";
        let strip = "M -5 10 H 50 V 20 H -5 Z";
        for fill_rule in [FillRule::NonZero, FillRule::EvenOdd] {
            let options = BooleanOptions { fill_rule, ..options(0.01) };
            check(squares, strip, BooleanOp::Intersect, &options);
        }
    }

    #[test]
    fn empty_paths() {
        let options = options(0.01);
        let square = parse(SQUARE);
        let none: Vec<PathSegment> = Vec::new();
        assert!(boolean(&square, &none, BooleanOp::Intersect, &options).is_empty());
        let union = boolean(&square, &none, BooleanOp::Union, &options);
        assert!((PathMeasure::new(&union).area() - 1600.0).abs() < 1e-9);
        assert!(boolean(&none, &none, BooleanOp::Xor, &options).is_empty());
    }

    #[test]
    fn a_coarse_tolerance_stays_within_it() {
        for op in [
            BooleanOp::Union,
            BooleanOp::Intersect,
            BooleanOp::Difference,
            BooleanOp::Xor,
        ] {
            check(SQUARE, CIRCLE, op, &options(1.0));
        }
    }
}
