use std::borrow::Borrow;

use euclid::default::Point2D;
use svgtypes::PathSegment;

use crate::context::{segments_with_context, SegmentContext};
use crate::{is_closed, reverse, split_subpaths};

/// Joins path `b` onto path `a`: the last subpath of `a` and the first of
/// `b` become one subpath, as Paper.js's `join` does.
///
/// The two are joined where their ends meet, within `tolerance`, trying in
/// turn the end of `a` with the start of `b`, the end of `a` with the end
/// of `b`, the start of `a` with the end of `b`, and the start of `a` with
/// the start of `b`, and reversing `b`'s subpath where that makes the ends
/// follow on. When no ends meet, a line joins the end of `a` to the start
/// of `b`. When the joined subpath then ends where it starts, within
/// `tolerance`, it is closed.
///
/// Where the ends meet only within the tolerance, the segment drawn from the
/// joint starts at the point of the subpath before it, and is written in
/// absolute coordinates so the rest of the path does not move. Every other
/// segment keeps its form. The other subpaths of `a` come first and the
/// other subpaths of `b` last, unchanged. When either of the two subpaths is
/// closed or draws nothing, there is nothing to join and the result is `a`
/// followed by `b`.
///
/// ```
/// use svg_path_ops::svgtypes::PathParser;
/// use svg_path_ops::{join, write_path, WriteOptions};
///
/// let parse = |data| PathParser::from(data).collect::<Result<Vec<_>, _>>();
/// let (a, b) = (parse("M 0 0 L 10 0")?, parse("M 10 10 L 10 0")?);
///
/// // The ends at 10 0 meet: b is reversed to follow on from a
/// assert_eq!(
///     write_path(join(&a, &b, 0.0), &WriteOptions::default()),
///     "M 0 0 L 10 0 L 10 10"
/// );
/// # Ok::<(), svg_path_ops::svgtypes::Error>(())
/// ```
pub fn join(
    a: impl IntoIterator<Item = impl Borrow<PathSegment>>,
    b: impl IntoIterator<Item = impl Borrow<PathSegment>>,
    tolerance: f64,
) -> Vec<PathSegment> {
    let mut a = split_subpaths(a);
    let mut b = split_subpaths(b);
    let (Some(a_last), Some(b_first)) = (a.last(), b.first()) else {
        return a.into_iter().chain(b).flatten().collect();
    };
    let (Some(a_ends), Some(b_ends)) = (ends(a_last), ends(b_first)) else {
        return a.into_iter().chain(b).flatten().collect();
    };
    if is_closed(a_last) || is_closed(b_first) {
        return a.into_iter().chain(b).flatten().collect();
    }

    let meets = |p: Point2D<f64>, q: Point2D<f64>| (p - q).length() <= tolerance;
    let (a_last, b_first) = (a.pop().expect("a has a subpath"), b.remove(0));
    let ((a_start, a_end), (b_start, b_end)) = (a_ends, b_ends);
    let mut joined = if meets(a_end, b_start) {
        follow_on(a_last, &b_first)
    } else if meets(a_end, b_end) {
        follow_on(a_last, &reverse(&b_first))
    } else if meets(a_start, b_end) {
        follow_on(b_first, &a_last)
    } else if meets(a_start, b_start) {
        follow_on(reverse(&b_first), &a_last)
    } else {
        // A line from the end of `a` to the start of `b`, after which `b`
        // goes on from exactly where it started
        let mut joined = a_last;
        joined.push(PathSegment::LineTo { abs: true, x: b_start.x, y: b_start.y });
        joined.extend(&b_first[1..]);
        joined
    };
    if let Some((start, end)) = ends(&joined) {
        if meets(start, end) {
            joined.push(PathSegment::ClosePath { abs: true });
        }
    }

    a.into_iter()
        .flatten()
        .chain(joined)
        .chain(b.into_iter().flatten())
        .collect()
}

/// Where a single subpath starts and ends, or `None` when it draws nothing.
fn ends(subpath: &[PathSegment]) -> Option<(Point2D<f64>, Point2D<f64>)> {
    let contexts: Vec<SegmentContext<'_>> = segments_with_context(subpath).collect();
    let mut drawing = contexts
        .iter()
        .filter(|context| !matches!(context.segment, PathSegment::MoveTo { .. }));
    let (first, last) = (drawing.clone().next()?, drawing.next_back()?);
    Some((first.start, last.end))
}

/// `first` followed by `then`, a subpath starting where `first` ends,
/// within the tolerance: `then` without its move, its first drawing
/// segment written in absolute coordinates, starting from the end of
/// `first`.
fn follow_on(mut first: Vec<PathSegment>, then: &[PathSegment]) -> Vec<PathSegment> {
    let contexts: Vec<SegmentContext<'_>> = segments_with_context(then).collect();
    let mut rest = contexts
        .iter()
        .skip_while(|context| matches!(context.segment, PathSegment::MoveTo { .. }));
    if let Some(context) = rest.next() {
        first.push(absolute(context));
    }
    first.extend(rest.map(|context| *context.segment));
    first
}

/// The segment in `context` in absolute coordinates, in a form that does
/// not depend on where it starts or what comes before it: a line for `H`
/// and `V`, and a full curve for `S` and `T`.
fn absolute(context: &SegmentContext<'_>) -> PathSegment {
    let end = context.end;
    let absolute = |abs: bool, x: f64, y: f64| {
        if abs {
            (x, y)
        } else {
            (context.start.x + x, context.start.y + y)
        }
    };
    match *context.segment {
        PathSegment::CurveTo { abs, x1, y1, x2, y2, .. } => {
            let ((x1, y1), (x2, y2)) = (absolute(abs, x1, y1), absolute(abs, x2, y2));
            PathSegment::CurveTo { abs: true, x1, y1, x2, y2, x: end.x, y: end.y }
        }
        PathSegment::SmoothCurveTo { abs, x2, y2, .. } => {
            let control = context.implied_control.expect("smooth segment");
            let (x2, y2) = absolute(abs, x2, y2);
            PathSegment::CurveTo {
                abs: true,
                x1: control.x,
                y1: control.y,
                x2,
                y2,
                x: end.x,
                y: end.y,
            }
        }
        PathSegment::Quadratic { abs, x1, y1, .. } => {
            let (x1, y1) = absolute(abs, x1, y1);
            PathSegment::Quadratic { abs: true, x1, y1, x: end.x, y: end.y }
        }
        PathSegment::SmoothQuadratic { .. } => {
            let control = context.implied_control.expect("smooth segment");
            PathSegment::Quadratic {
                abs: true,
                x1: control.x,
                y1: control.y,
                x: end.x,
                y: end.y,
            }
        }
        PathSegment::EllipticalArc { rx, ry, x_axis_rotation, large_arc, sweep, .. } => {
            PathSegment::EllipticalArc {
                abs: true,
                rx,
                ry,
                x_axis_rotation,
                large_arc,
                sweep,
                x: end.x,
                y: end.y,
            }
        }
        PathSegment::ClosePath { .. } => PathSegment::ClosePath { abs: true },
        _ => PathSegment::LineTo { abs: true, x: end.x, y: end.y },
    }
}

#[cfg(test)]
mod test {
    use svgtypes::{PathParser, PathSegment};

    use super::join;
    use crate::write::{write_path, WriteOptions};

    fn joined(a: &str, b: &str, tolerance: f64) -> String {
        let parse = |data: &str| {
            PathParser::from(data)
                .collect::<Result<Vec<PathSegment>, _>>()
                .unwrap()
        };
        write_path(
            join(parse(a), parse(b), tolerance),
            &WriteOptions::default(),
        )
    }

    #[test]
    fn joins_whichever_ends_meet() {
        let a = "M 0 0 L 10 0";
        assert_eq!(joined(a, "M 10 0 L 10 10", 0.0), "M 0 0 L 10 0 L 10 10");
        assert_eq!(joined(a, "M 10 10 L 10 0", 0.0), "M 0 0 L 10 0 L 10 10");
        assert_eq!(joined(a, "M 0 10 L 0 0", 0.0), "M 0 10 L 0 0 L 10 0");
        assert_eq!(joined(a, "M 0 0 L 0 10", 0.0), "M 0 10 L 0 0 L 10 0");
    }

    #[test]
    fn ends_within_the_tolerance_meet_without_moving_the_rest() {
        assert_eq!(
            joined("M 0 0 L 10 0", "m 10.05 0 l 5 0 l 0 5", 0.1),
            "M 0 0 L 10 0 L 15.05 0 l 0 5"
        );
        assert_eq!(
            joined("M 0 0 L 10 0", "M 10.05 0 L 15 0", 0.01),
            "M 0 0 L 10 0 L 10.05 0 L 15 0"
        );
    }

    #[test]
    fn the_segment_after_the_joint_does_not_depend_on_what_came_before() {
        // A smooth curve after a move takes its start as its first control
        assert_eq!(
            joined("M 0 0 C 0 5 10 5 10 0", "M 10 0 S 20 10 30 0", 0.0),
            "M 0 0 C 0 5 10 5 10 0 C 10 0 20 10 30 0"
        );
        assert_eq!(
            joined("M 0 0 L 10 0", "M 10 0 h 5 q 5 5 10 0 t 10 0", 0.0),
            "M 0 0 L 10 0 L 15 0 q 5 5 10 0 t 10 0"
        );
    }

    #[test]
    fn a_line_joins_ends_that_do_not_meet() {
        assert_eq!(
            joined("M 0 0 L 10 0", "M 20 5 l 5 0", 0.0),
            "M 0 0 L 10 0 L 20 5 l 5 0"
        );
    }

    #[test]
    fn closes_a_joined_subpath_that_ends_where_it_starts() {
        assert_eq!(
            joined("M 0 0 L 10 0 L 10 10", "M 10 10 L 0 10 L 0 0.01", 0.1),
            "M 0 0 L 10 0 L 10 10 L 0 10 L 0 0.01 Z"
        );
    }

    #[test]
    fn keeps_the_other_subpaths() {
        assert_eq!(
            joined("M 0 0 L 1 0 M 5 5 L 6 5", "M 6 5 L 7 5 m 2 2 l 1 0", 0.0),
            "M 0 0 L 1 0 M 5 5 L 6 5 L 7 5 M 9 7 l 1 0"
        );
    }

    #[test]
    fn closed_or_empty_paths_are_put_together() {
        assert_eq!(
            joined("M 0 0 L 10 0 Z", "M 10 0 L 20 0", 0.0),
            "M 0 0 L 10 0 Z M 10 0 L 20 0"
        );
        assert_eq!(joined("", "M 10 0 L 20 0", 0.0), "M 10 0 L 20 0");
        assert_eq!(joined("M 0 0 L 10 0", "", 0.0), "M 0 0 L 10 0");
    }
}
