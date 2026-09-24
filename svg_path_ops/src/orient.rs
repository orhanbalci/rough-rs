use std::borrow::Borrow;

use euclid::default::Point2D;
use svgtypes::PathSegment;

use crate::{reverse, split_subpaths, FillRule, PathMeasure};

/// Turns each subpath so that outlines run one way and the holes in them
/// the other, as fonts and icon sets expect: outlines clockwise on screen
/// when `clockwise` is true, counterclockwise otherwise, the holes in them
/// the other way, the outlines inside those holes the way of the outlines,
/// and so on.
///
/// Whether a subpath is an outline or a hole depends on how many others
/// it lies inside, as the even-odd fill rule counts them. So the result
/// fills with SVG's default nonzero rule what the path filled with the
/// even-odd rule, and fills the same with both, however its subpaths were
/// drawn. Subpaths are expected not to cross each other.
///
/// A subpath is reversed with [`reverse`] or kept as it is, and the
/// subpaths stay in their order. One that encloses no area, as a line, is
/// kept and counts as inside nothing.
///
/// # Algorithm
///
/// Each subpath's direction is the sign of its [`PathMeasure::area`]. To
/// tell whether it lies inside another, a point inside it and away from its
/// outline is checked against the other with [`PathMeasure::contains`]. As
/// Paper.js does, it is found with [`PathMeasure::interior_point`]. Two
/// subpaths that do not
/// cross lie one inside the other or apart, so that point is inside
/// another exactly when the whole subpath is, even where the two touch. Only a subpath with a larger area
/// can contain another, which keeps two copies of one outline from both
/// counting as inside each other. Checking every pair takes time quadratic
/// in the number of subpaths.
///
/// ```
/// use svg_path_ops::svgtypes::PathParser;
/// use svg_path_ops::{reorient, write_path, PathMeasure, WriteOptions};
///
/// // A square with a hole, both drawn clockwise: filled with nonzero, the
/// // hole is filled too
/// let frame: Vec<_> = PathParser::from("M 0 0 H 30 V 30 H 0 Z M 10 10 H 20 V 20 H 10 Z")
///     .collect::<Result<_, _>>()?;
/// assert_eq!(PathMeasure::new(&frame).area(), 1000.0);
///
/// let fixed = reorient(&frame, true);
/// assert_eq!(
///     write_path(&fixed, &WriteOptions::default()),
///     "M 0 0 H 30 V 30 H 0 Z M 10 20 H 20 V 10 H 10 Z"
/// );
/// assert_eq!(PathMeasure::new(&fixed).area(), 800.0);
/// # Ok::<(), svg_path_ops::svgtypes::Error>(())
/// ```
pub fn reorient(
    segments: impl IntoIterator<Item = impl Borrow<PathSegment>>,
    clockwise: bool,
) -> Vec<PathSegment> {
    let subpaths = split_subpaths(segments);
    let measures: Vec<PathMeasure> = subpaths.iter().map(PathMeasure::new).collect();
    let areas: Vec<f64> = measures.iter().map(PathMeasure::area).collect();

    let interior: Vec<Option<Point2D<f64>>> = measures
        .iter()
        .map(|measure| measure.interior_point(FillRule::NonZero))
        .collect();
    let inside = |i: usize, j: usize| {
        interior[i].is_some_and(|point| measures[j].contains(point, FillRule::NonZero))
    };

    subpaths
        .iter()
        .enumerate()
        .flat_map(|(i, subpath)| {
            if areas[i] == 0.0 {
                return subpath.clone();
            }
            let depth = (0..areas.len())
                .filter(|&j| areas[j].abs() > areas[i].abs() && inside(i, j))
                .count();
            let wanted = clockwise != (depth % 2 == 1);
            if (areas[i] > 0.0) == wanted {
                subpath.clone()
            } else {
                reverse(subpath)
            }
        })
        .collect()
}

#[cfg(test)]
mod test {
    use euclid::default::Point2D;
    use svgtypes::{PathParser, PathSegment};

    use super::reorient;
    use crate::pt::PathTransformer;
    use crate::{split_subpaths, FillRule, PathMeasure};

    fn parse(data: &str) -> Vec<PathSegment> {
        PathParser::from(data).collect::<Result<_, _>>().unwrap()
    }

    /// Whether each subpath is drawn clockwise.
    fn directions(segments: &[PathSegment]) -> Vec<bool> {
        split_subpaths(segments)
            .iter()
            .map(|subpath| PathMeasure::new(subpath).is_clockwise())
            .collect()
    }

    /// An outline, a hole in it and an island in the hole, all clockwise,
    /// and a separate outline counterclockwise.
    const NESTED: &str = "M 0 0 H 60 V 60 H 0 Z M 10 10 H 50 V 50 H 10 Z \
                          M 20 20 H 40 V 40 H 20 Z M 70 0 V 20 H 90 V 0 Z";

    #[test]
    fn outlines_and_holes_alternate() {
        let path = parse(NESTED);
        assert_eq!(
            directions(&reorient(&path, true)),
            [true, false, true, true]
        );
        assert_eq!(
            directions(&reorient(&path, false)),
            [false, true, false, false]
        );
    }

    #[test]
    fn nonzero_fills_what_evenodd_did() {
        let path = parse(NESTED);
        let before = PathMeasure::new(&path);
        let after = PathMeasure::new(reorient(&path, true));
        for x in 0..95 {
            for y in 0..65 {
                let point = Point2D::new(f64::from(x) + 0.5, f64::from(y) + 0.5);
                let evenodd = before.contains(point, FillRule::EvenOdd);
                assert_eq!(after.contains(point, FillRule::NonZero), evenodd);
                assert_eq!(after.contains(point, FillRule::EvenOdd), evenodd);
            }
        }
    }

    #[test]
    fn a_hole_touching_its_outline_is_a_hole() {
        let path = parse("M 0 0 H 60 V 60 H 0 Z M 0 0 H 30 V 30 H 0 Z");
        assert_eq!(directions(&reorient(&path, true)), [true, false]);
    }

    #[test]
    fn a_hole_sharing_three_sides_with_its_outline_is_a_hole() {
        let path = parse("M 0 0 H 60 V 60 H 0 Z M 0 0 H 60 V 30 H 0 Z");
        assert_eq!(directions(&reorient(&path, true)), [true, false]);
    }

    #[test]
    fn keeps_subpaths_that_enclose_nothing() {
        let path = parse("M 0 0 L 10 0 M 20 0 V 10 H 30 V 0 Z");
        let reoriented = reorient(&path, true);
        assert_eq!(&reoriented[..2], &path[..2]);
        assert_eq!(directions(&reoriented), [false, true]);
    }

    #[test]
    fn transformer_applies_mirroring_first() {
        let mut shape = PathTransformer::parse("M 0 0 H 10 V 10 H 0 Z").unwrap();
        shape.scale(-1.0, 1.0).reorient(true);
        assert!(shape.measure().is_clockwise());
    }
}
