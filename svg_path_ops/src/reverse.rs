use std::borrow::Borrow;

use euclid::default::Point2D;
use svgtypes::PathSegment;

use crate::context::{segments_with_context, SegmentContext};

/// Reverses the drawing direction of every subpath, keeping the subpaths in
/// their original order.
///
/// The result draws the same shape. Relative segments stay relative and
/// absolute ones stay absolute. Arcs stay arcs with their sweep flag flipped.
/// A smooth segment (`S`/`T`) stays smooth when its reversed neighbour allows
/// it and becomes a full `C`/`Q` otherwise. An open subpath starts from its
/// old end; a closed one starts from its last point and still ends with a
/// close path.
///
/// ```
/// use svg_path_ops::svgtypes::PathParser;
/// use svg_path_ops::{reverse, write_path, WriteOptions};
///
/// let segments: Vec<_> =
///     PathParser::from("M 0 0 L 10 0 l 0 10 A 5 5 0 0 1 0 10 Z").collect::<Result<_, _>>()?;
///
/// assert_eq!(
///     write_path(reverse(&segments), &WriteOptions::default()),
///     "M 0 10 A 5 5 0 0 0 10 10 l 0 -10 L 0 0 Z"
/// );
/// # Ok::<(), svg_path_ops::svgtypes::Error>(())
/// ```
pub fn reverse(segments: impl IntoIterator<Item = impl Borrow<PathSegment>>) -> Vec<PathSegment> {
    let segments: Vec<PathSegment> = segments.into_iter().map(|s| *s.borrow()).collect();
    let contexts: Vec<SegmentContext<'_>> = segments_with_context(&segments).collect();

    let mut reversed = Vec::with_capacity(segments.len());
    // Where the reversed path currently is, for writing relative segments
    let mut current = Point2D::zero();

    for subpath in split_subpaths(&contexts) {
        let (end, drawing, move_abs, close) = subpath.parts();

        let (x, y) = if move_abs {
            (end.x, end.y)
        } else {
            (end.x - current.x, end.y - current.y)
        };
        reversed.push(PathSegment::MoveTo { abs: move_abs, x, y });
        current = end;

        for (i, context) in drawing.iter().enumerate().rev() {
            let next = drawing.get(i + 1).map(|next| next.segment);
            reversed.push(reverse_segment(context, next));
            current = context.start;
        }

        if let Some(abs) = close {
            reversed.push(PathSegment::ClosePath { abs });
            current = end;
        }
    }

    reversed
}

/// One subpath: an optional move, the drawing segments and an optional close.
struct Subpath<'s, 'a> {
    contexts: &'s [SegmentContext<'a>],
}

impl<'a> Subpath<'_, 'a> {
    /// Returns the point the reversed subpath starts from, the drawing
    /// segments, whether its move is absolute and, when it is closed, whether
    /// its close path is absolute.
    fn parts(&self) -> (Point2D<f64>, &[SegmentContext<'a>], bool, Option<bool>) {
        let mut contexts = self.contexts;

        let mut move_abs = true;
        let mut start = contexts[0].start;
        if let PathSegment::MoveTo { abs, .. } = *contexts[0].segment {
            move_abs = abs;
            start = contexts[0].end;
            contexts = &contexts[1..];
        }

        let mut close = None;
        if let Some((last, rest)) = contexts.split_last() {
            if let PathSegment::ClosePath { abs } = *last.segment {
                close = Some(abs);
                contexts = rest;
            }
        }

        let end = contexts.last().map_or(start, |last| last.end);
        (end, contexts, move_abs, close)
    }
}

/// Splits a path into subpaths. A subpath starts at a move, or at a drawing
/// segment that follows a close path without a move of its own.
fn split_subpaths<'s, 'a>(contexts: &'s [SegmentContext<'a>]) -> Vec<Subpath<'s, 'a>> {
    let mut subpaths = Vec::new();
    let mut start = 0;
    for (i, context) in contexts.iter().enumerate() {
        let after_close = i > 0 && matches!(contexts[i - 1].segment, PathSegment::ClosePath { .. });
        let starts_subpath = matches!(context.segment, PathSegment::MoveTo { .. }) || after_close;
        if i > start && starts_subpath {
            subpaths.push(Subpath { contexts: &contexts[start..i] });
            start = i;
        }
    }
    if start < contexts.len() {
        subpaths.push(Subpath { contexts: &contexts[start..] });
    }
    subpaths
}

/// Reverses one drawing segment, so it runs from `context.end` back to
/// `context.start`. `next` is the segment that followed it in the original
/// path, which decides whether a curve can be written in its smooth form.
fn reverse_segment(context: &SegmentContext<'_>, next: Option<&PathSegment>) -> PathSegment {
    let (start, end) = (context.start, context.end);
    let absolute = |abs: bool, x: f64, y: f64| {
        if abs {
            Point2D::new(x, y)
        } else {
            Point2D::new(start.x + x, start.y + y)
        }
    };
    // Coordinates of `point` for a reversed segment starting at `end`
    let to = |abs: bool, point: Point2D<f64>| {
        if abs {
            (point.x, point.y)
        } else {
            (point.x - end.x, point.y - end.y)
        }
    };
    let next_is_smooth_cubic = matches!(next, Some(PathSegment::SmoothCurveTo { .. }));
    let next_is_smooth_quadratic = matches!(next, Some(PathSegment::SmoothQuadratic { .. }));

    match *context.segment {
        PathSegment::LineTo { abs, .. } => {
            let (x, y) = to(abs, start);
            PathSegment::LineTo { abs, x, y }
        }
        PathSegment::HorizontalLineTo { abs, .. } => {
            let (x, _) = to(abs, start);
            PathSegment::HorizontalLineTo { abs, x }
        }
        PathSegment::VerticalLineTo { abs, .. } => {
            let (_, y) = to(abs, start);
            PathSegment::VerticalLineTo { abs, y }
        }
        PathSegment::CurveTo { abs, x1, y1, x2, y2, .. } => reverse_cubic(
            abs,
            absolute(abs, x1, y1),
            absolute(abs, x2, y2),
            start,
            next_is_smooth_cubic,
            to,
        ),
        PathSegment::SmoothCurveTo { abs, x2, y2, .. } => reverse_cubic(
            abs,
            context
                .implied_control
                .expect("smooth curves have an implied control"),
            absolute(abs, x2, y2),
            start,
            next_is_smooth_cubic,
            to,
        ),
        PathSegment::Quadratic { abs, x1, y1, .. } => reverse_quadratic(
            abs,
            absolute(abs, x1, y1),
            start,
            next_is_smooth_quadratic,
            to,
        ),
        PathSegment::SmoothQuadratic { abs, .. } => reverse_quadratic(
            abs,
            context
                .implied_control
                .expect("smooth quadratics have an implied control"),
            start,
            next_is_smooth_quadratic,
            to,
        ),
        PathSegment::EllipticalArc { abs, rx, ry, x_axis_rotation, large_arc, sweep, .. } => {
            let (x, y) = to(abs, start);
            PathSegment::EllipticalArc {
                abs,
                rx,
                ry,
                x_axis_rotation,
                large_arc,
                sweep: !sweep,
                x,
                y,
            }
        }
        PathSegment::MoveTo { .. } | PathSegment::ClosePath { .. } => {
            unreachable!("moves and close paths are not drawing segments")
        }
    }
}

/// Reverses a cubic curve with controls `first` and `second`. It can be
/// written smooth when the following original segment was a smooth cubic,
/// since that one's implied control mirrors `second`.
fn reverse_cubic(
    abs: bool,
    first: Point2D<f64>,
    second: Point2D<f64>,
    start: Point2D<f64>,
    smooth: bool,
    to: impl Fn(bool, Point2D<f64>) -> (f64, f64),
) -> PathSegment {
    let (x, y) = to(abs, start);
    let (x2, y2) = to(abs, first);
    if smooth {
        PathSegment::SmoothCurveTo { abs, x2, y2, x, y }
    } else {
        let (x1, y1) = to(abs, second);
        PathSegment::CurveTo { abs, x1, y1, x2, y2, x, y }
    }
}

/// Reverses a quadratic curve with `control`. It can be written smooth when
/// the following original segment was a smooth quadratic.
fn reverse_quadratic(
    abs: bool,
    control: Point2D<f64>,
    start: Point2D<f64>,
    smooth: bool,
    to: impl Fn(bool, Point2D<f64>) -> (f64, f64),
) -> PathSegment {
    let (x, y) = to(abs, start);
    if smooth {
        PathSegment::SmoothQuadratic { abs, x, y }
    } else {
        let (x1, y1) = to(abs, control);
        PathSegment::Quadratic { abs, x1, y1, x, y }
    }
}

#[cfg(test)]
mod test {
    use svgtypes::PathParser;

    use super::reverse;
    use crate::{normalize, write_path, PathSegment, WriteOptions};

    fn parse(path: &str) -> Vec<PathSegment> {
        PathParser::from(path).map(Result::unwrap).collect()
    }

    fn reversed(path: &str) -> String {
        write_path(reverse(parse(path)), &WriteOptions::default())
    }

    #[test]
    fn reverses_open_lines() {
        assert_eq!(reversed("M0 0 L10 0 L10 10"), "M 10 10 L 10 0 L 0 0");
    }

    #[test]
    fn keeps_relative_segments_relative() {
        assert_eq!(reversed("M0 0 l10 0 l0 10"), "M 10 10 l 0 -10 l -10 0");
    }

    #[test]
    fn keeps_horizontal_and_vertical_lines() {
        assert_eq!(reversed("M0 0 H10 V10"), "M 10 10 V 0 H 0");
        assert_eq!(reversed("M0 0 h10 v10"), "M 10 10 v -10 h -10");
    }

    #[test]
    fn closed_subpath_starts_from_its_last_point() {
        assert_eq!(reversed("M0 0 L10 0 L10 10 Z"), "M 10 10 L 10 0 L 0 0 Z");
        assert_eq!(reversed("M0 0 L10 0 L10 10 z"), "M 10 10 L 10 0 L 0 0 z");
    }

    #[test]
    fn swaps_cubic_controls() {
        assert_eq!(reversed("M0 0 C1 2 3 4 5 6"), "M 5 6 C 3 4 1 2 0 0");
        assert_eq!(reversed("M0 0 c1 2 3 4 5 6"), "M 5 6 c -2 -2 -4 -4 -5 -6");
    }

    #[test]
    fn keeps_smooth_cubic_when_its_neighbour_allows() {
        // The S becomes a C, and the C before it becomes the S
        assert_eq!(
            reversed("M0 0 C0 10 10 10 10 0 S20 -10 20 0"),
            "M 20 0 C 20 -10 10 -10 10 0 S 0 10 0 0"
        );
    }

    #[test]
    fn keeps_smooth_quadratic_when_its_neighbour_allows() {
        assert_eq!(
            reversed("M0 0 Q10 10 20 0 T40 0"),
            "M 40 0 Q 30 -10 20 0 T 0 0"
        );
    }

    #[test]
    fn flips_arc_sweep() {
        assert_eq!(reversed("M0 0 A5 5 0 0 1 10 0"), "M 10 0 A 5 5 0 0 0 0 0");
        assert_eq!(
            reversed("M0 0 a5 5 30 1 0 10 0"),
            "M 10 0 a 5 5 30 1 1 -10 0"
        );
    }

    #[test]
    fn keeps_subpath_order() {
        assert_eq!(
            reversed("M0 0 L10 0 M20 0 L30 0"),
            "M 10 0 L 0 0 M 30 0 L 20 0"
        );
    }

    #[test]
    fn relative_moves_follow_the_reversed_path() {
        assert_eq!(reversed("m5 5 l1 0"), "m 6 5 l -1 0");
        assert_eq!(
            reversed("M0 0 L10 0 m5 5 l1 0"),
            "M 10 0 L 0 0 m 16 5 l -1 0"
        );
    }

    #[test]
    fn drawing_after_close_path_starts_a_new_subpath() {
        assert_eq!(
            reversed("M0 0 L10 0 Z L0 10"),
            "M 10 0 L 0 0 Z M 0 10 L 0 0"
        );
    }

    #[test]
    fn lone_moves_are_kept() {
        assert_eq!(reversed("M1 1 M0 0 L5 0"), "M 1 1 M 5 0 L 0 0");
    }

    #[test]
    fn empty_path_stays_empty() {
        assert!(reverse(Vec::<PathSegment>::new()).is_empty());
    }

    #[test]
    fn reversing_twice_restores_the_path() {
        let path = "M0 0 C0 10 10 10 10 0 S20 -10 20 0 q5 5 10 0 t10 0 \
                    a5 5 0 0 1 10 0 h5 v5 l-5 5 Z m 50 0 l 10 10";
        let segments = parse(path);
        assert_eq!(reverse(reverse(&segments)), segments);
    }

    #[test]
    fn reversed_path_covers_the_same_geometry() {
        // Normalized to absolute cubic curves, the reversed path visits the
        // same points in the opposite order
        let path = "M0 0 C0 10 10 10 10 0 S20 -10 20 0 L30 10";
        let forward: Vec<PathSegment> = normalize(parse(path).iter()).collect();
        let backward: Vec<PathSegment> = normalize(reverse(parse(path)).iter()).collect();
        assert_eq!(forward.len(), backward.len());
        assert_eq!(
            write_path(&backward, &WriteOptions::default()),
            "M 30 10 L 20 0 C 20 -10 10 -10 10 0 C 10 10 0 10 0 0"
        );
    }
}
