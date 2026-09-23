use std::borrow::Borrow;
use std::ops::Range;

use svgtypes::PathSegment;

use crate::context::{segments_with_context, SegmentContext};

/// Splits a path into its subpaths, each one a path of its own.
///
/// A subpath starts at a move, or at a drawing segment that follows a close
/// path without a move of its own. Each returned path draws exactly what its
/// subpath drew: a relative move that depended on the previous subpath
/// becomes absolute, and a subpath that started without a move gets one.
/// Every other segment is kept as it is.
///
/// ```
/// use svg_path_ops::svgtypes::PathParser;
/// use svg_path_ops::{split_subpaths, write_path, WriteOptions};
///
/// let segments: Vec<_> =
///     PathParser::from("M 0 0 l 10 0 z m 5 5 l 1 0").collect::<Result<_, _>>()?;
///
/// let subpaths: Vec<String> = split_subpaths(&segments)
///     .iter()
///     .map(|subpath| write_path(subpath, &WriteOptions::default()))
///     .collect();
/// assert_eq!(subpaths, ["M 0 0 l 10 0 z", "M 5 5 l 1 0"]);
/// # Ok::<(), svg_path_ops::svgtypes::Error>(())
/// ```
pub fn split_subpaths(
    segments: impl IntoIterator<Item = impl Borrow<PathSegment>>,
) -> Vec<Vec<PathSegment>> {
    let segments: Vec<PathSegment> = segments.into_iter().map(|s| *s.borrow()).collect();
    let contexts: Vec<SegmentContext<'_>> = segments_with_context(&segments).collect();

    subpath_ranges(&contexts)
        .into_iter()
        .map(|range| {
            let first = &contexts[range.start];
            let mut subpath = Vec::with_capacity(range.len() + 1);
            match *first.segment {
                // Only the path's first move is relative to the origin, as a
                // standalone path's would be
                PathSegment::MoveTo { abs: false, .. } if first.index > 0 => {
                    subpath.push(PathSegment::MoveTo { abs: true, x: first.end.x, y: first.end.y });
                }
                PathSegment::MoveTo { .. } => subpath.push(*first.segment),
                _ => {
                    subpath.push(PathSegment::MoveTo {
                        abs: true,
                        x: first.start.x,
                        y: first.start.y,
                    });
                    subpath.push(*first.segment);
                }
            }
            subpath.extend(
                contexts[range.start + 1..range.end]
                    .iter()
                    .map(|c| *c.segment),
            );
            subpath
        })
        .collect()
}

/// Returns whether every subpath that draws something ends with a close
/// path (`Z`/`z`). An empty path, or one that only moves, is not closed.
///
/// This follows SVG: a subpath that returns to its start without a close
/// path is still open, and a stroke gets line caps there instead of a join.
///
/// ```
/// use svg_path_ops::is_closed;
/// use svg_path_ops::svgtypes::PathParser;
///
/// let closed: Vec<_> = PathParser::from("M 0 0 L 10 0 L 10 10 Z").collect::<Result<_, _>>()?;
/// assert!(is_closed(&closed));
///
/// let back_to_start: Vec<_> =
///     PathParser::from("M 0 0 L 10 0 L 10 10 L 0 0").collect::<Result<_, _>>()?;
/// assert!(!is_closed(&back_to_start));
/// # Ok::<(), svg_path_ops::svgtypes::Error>(())
/// ```
pub fn is_closed(segments: impl IntoIterator<Item = impl Borrow<PathSegment>>) -> bool {
    let mut draws_anything = false;
    for subpath in split_subpaths(segments) {
        let drawing = subpath
            .iter()
            .any(|segment| !matches!(segment, PathSegment::MoveTo { .. }));
        if !drawing {
            continue;
        }
        draws_anything = true;
        if !matches!(subpath.last(), Some(PathSegment::ClosePath { .. })) {
            return false;
        }
    }
    draws_anything
}

/// Index ranges of the subpaths in `contexts`. A subpath starts at a move,
/// or at a drawing segment that follows a close path without a move.
pub(crate) fn subpath_ranges(contexts: &[SegmentContext<'_>]) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for (i, context) in contexts.iter().enumerate() {
        let after_close = i > 0 && matches!(contexts[i - 1].segment, PathSegment::ClosePath { .. });
        let starts_subpath = matches!(context.segment, PathSegment::MoveTo { .. }) || after_close;
        if i > start && starts_subpath {
            ranges.push(start..i);
            start = i;
        }
    }
    if start < contexts.len() {
        ranges.push(start..contexts.len());
    }
    ranges
}

#[cfg(test)]
mod test {
    use svgtypes::PathParser;

    use super::{is_closed, split_subpaths};
    use crate::{absolutize, normalize, write_path, PathSegment, WriteOptions};

    fn parse(path: &str) -> Vec<PathSegment> {
        PathParser::from(path).map(Result::unwrap).collect()
    }

    fn split(path: &str) -> Vec<String> {
        split_subpaths(parse(path))
            .iter()
            .map(|subpath| write_path(subpath, &WriteOptions::default()))
            .collect()
    }

    #[test]
    fn splits_at_moves() {
        assert_eq!(
            split("M0 0 L10 0 M20 0 L30 0"),
            ["M 0 0 L 10 0", "M 20 0 L 30 0"]
        );
    }

    #[test]
    fn relative_move_becomes_absolute() {
        assert_eq!(
            split("M0 0 l10 0 m5 5 l1 0"),
            ["M 0 0 l 10 0", "M 15 5 l 1 0"]
        );
        // After a close path, the current point is back at the subpath start
        assert_eq!(
            split("m0 0 l10 0 z m5 5 l1 0"),
            ["m 0 0 l 10 0 z", "M 5 5 l 1 0"]
        );
    }

    #[test]
    fn first_relative_move_is_kept() {
        assert_eq!(split("m5 5 l1 0"), ["m 5 5 l 1 0"]);
    }

    #[test]
    fn subpath_after_close_without_move_gets_one() {
        assert_eq!(
            split("M0 0 L10 0 Z L0 10"),
            ["M 0 0 L 10 0 Z", "M 0 0 L 0 10"]
        );
        assert_eq!(
            split("M5 5 l10 0 z l0 10"),
            ["M 5 5 l 10 0 z", "M 5 5 l 0 10"]
        );
    }

    #[test]
    fn lone_moves_are_their_own_subpaths() {
        assert_eq!(split("M1 1 M0 0 L5 0"), ["M 1 1", "M 0 0 L 5 0"]);
    }

    #[test]
    fn empty_path_has_no_subpaths() {
        assert!(split_subpaths(Vec::<PathSegment>::new()).is_empty());
    }

    #[test]
    fn subpaths_draw_the_same_geometry() {
        let path = "M0 0 c5 5 10 5 15 0 s10 -5 15 0 z m20 20 q5 5 10 0 t10 0 \
                    L50 50 Z h5 a5 5 0 0 1 5 5 M 80 80 v5";
        // Compare what is drawn; the split adds an explicit move where the
        // original started a subpath after a close path without one
        let drawn = |segments: Vec<PathSegment>| -> Vec<PathSegment> {
            segments
                .into_iter()
                .filter(|segment| !matches!(segment, PathSegment::MoveTo { .. }))
                .collect()
        };
        let whole = drawn(normalize(absolutize(parse(path).iter())).collect());
        let parts = drawn(
            split_subpaths(parse(path))
                .iter()
                .flat_map(|subpath| normalize(absolutize(subpath.iter())).collect::<Vec<_>>())
                .collect(),
        );
        assert_eq!(parts, whole);
    }

    #[test]
    fn closed_when_every_subpath_is_closed() {
        assert!(is_closed(parse("M0 0 L10 0 L10 10 Z")));
        assert!(is_closed(parse("M0 0 L1 0 Z m5 5 l1 0 z")));
    }

    #[test]
    fn open_when_any_subpath_is_open() {
        assert!(!is_closed(parse("M0 0 L10 0 Z M20 0 L30 0")));
        // Returning to the start is not closing
        assert!(!is_closed(parse("M0 0 L10 0 L10 10 L0 0")));
    }

    #[test]
    fn moves_alone_do_not_count() {
        assert!(is_closed(parse("M0 0 L10 0 L0 10 Z M5 5")));
        assert!(!is_closed(parse("M1 1")));
        assert!(!is_closed(Vec::<PathSegment>::new()));
    }
}
