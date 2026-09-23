use euclid::default::Point2D;
use svgtypes::PathSegment;

/// A path segment with the absolute positions it depends on, as yielded by
/// [`segments_with_context`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegmentContext<'a> {
    /// Position of the segment in the path.
    pub index: usize,
    /// The segment as it appears in the path, relative or absolute.
    pub segment: &'a PathSegment,
    /// Absolute point where the segment starts.
    pub start: Point2D<f64>,
    /// Absolute point where the segment ends. For a close path, this is the
    /// start of its subpath.
    pub end: Point2D<f64>,
    /// Absolute start of the subpath the segment belongs to.
    pub subpath_start: Point2D<f64>,
    /// For a smooth curve (`S`/`s`) or smooth quadratic (`T`/`t`), the
    /// absolute first control point it implies: the previous segment's
    /// control point reflected about `start`, or `start` itself when the
    /// previous segment is not a curve of the same kind. `None` for every
    /// other segment.
    pub implied_control: Option<Point2D<f64>>,
}

/// Iterates over path segments together with the absolute positions each
/// one depends on, so relative and shorthand segments can be handled without
/// tracking the current point by hand.
///
/// ```
/// use svg_path_ops::euclid::default::Point2D;
/// use svg_path_ops::segments_with_context;
/// use svg_path_ops::svgtypes::PathParser;
///
/// let segments: Vec<_> = PathParser::from("M10 10 l5 0 v5 z")
///     .collect::<Result<_, _>>()
///     .unwrap();
///
/// let ends: Vec<Point2D<f64>> = segments_with_context(&segments)
///     .map(|context| context.end)
///     .collect();
///
/// assert_eq!(
///     ends,
///     [
///         Point2D::new(10.0, 10.0),
///         Point2D::new(15.0, 10.0),
///         Point2D::new(15.0, 15.0),
///         Point2D::new(10.0, 10.0),
///     ]
/// );
/// ```
pub fn segments_with_context<'a>(
    segments: impl IntoIterator<Item = &'a PathSegment>,
) -> impl Iterator<Item = SegmentContext<'a>> {
    let mut current = Point2D::zero();
    let mut subpath_start = Point2D::zero();
    // Second control point of the previous segment, if it was a cubic curve.
    let mut cubic_control: Option<Point2D<f64>> = None;
    // Control point of the previous segment, if it was a quadratic curve.
    let mut quadratic_control: Option<Point2D<f64>> = None;

    segments
        .into_iter()
        .enumerate()
        .map(move |(index, segment)| {
            let start = current;
            let absolute = |abs: bool, x: f64, y: f64| {
                if abs {
                    Point2D::new(x, y)
                } else {
                    Point2D::new(start.x + x, start.y + y)
                }
            };
            let reflect = |control: Option<Point2D<f64>>| {
                control.map_or(start, |control| start + (start - control))
            };

            let mut implied_control = None;
            let mut next_cubic_control = None;
            let mut next_quadratic_control = None;

            let end = match *segment {
                PathSegment::MoveTo { abs, x, y } => {
                    let end = absolute(abs, x, y);
                    subpath_start = end;
                    end
                }
                PathSegment::LineTo { abs, x, y } => absolute(abs, x, y),
                PathSegment::HorizontalLineTo { abs, x } => {
                    Point2D::new(if abs { x } else { start.x + x }, start.y)
                }
                PathSegment::VerticalLineTo { abs, y } => {
                    Point2D::new(start.x, if abs { y } else { start.y + y })
                }
                PathSegment::CurveTo { abs, x2, y2, x, y, .. } => {
                    next_cubic_control = Some(absolute(abs, x2, y2));
                    absolute(abs, x, y)
                }
                PathSegment::SmoothCurveTo { abs, x2, y2, x, y } => {
                    implied_control = Some(reflect(cubic_control));
                    next_cubic_control = Some(absolute(abs, x2, y2));
                    absolute(abs, x, y)
                }
                PathSegment::Quadratic { abs, x1, y1, x, y } => {
                    next_quadratic_control = Some(absolute(abs, x1, y1));
                    absolute(abs, x, y)
                }
                PathSegment::SmoothQuadratic { abs, x, y } => {
                    let control = reflect(quadratic_control);
                    implied_control = Some(control);
                    next_quadratic_control = Some(control);
                    absolute(abs, x, y)
                }
                PathSegment::EllipticalArc { abs, x, y, .. } => absolute(abs, x, y),
                PathSegment::ClosePath { .. } => subpath_start,
            };

            current = end;
            cubic_control = next_cubic_control;
            quadratic_control = next_quadratic_control;

            SegmentContext {
                index,
                segment,
                start,
                end,
                subpath_start,
                implied_control,
            }
        })
}

#[cfg(test)]
mod test {
    use euclid::default::Point2D;
    use svgtypes::PathParser;

    use super::{segments_with_context, SegmentContext};
    use crate::PathSegment;

    fn parse(path: &str) -> Vec<PathSegment> {
        PathParser::from(path).map(Result::unwrap).collect()
    }

    fn contexts(segments: &[PathSegment]) -> Vec<SegmentContext<'_>> {
        segments_with_context(segments).collect()
    }

    fn pt(x: f64, y: f64) -> Point2D<f64> {
        Point2D::new(x, y)
    }

    #[test]
    fn tracks_absolute_and_relative_points() {
        let segments = parse("M10 20 L30 40 l5 -5 H0 h2 V1 v3");
        let ends: Vec<_> = contexts(&segments).iter().map(|c| c.end).collect();
        assert_eq!(
            ends,
            [
                pt(10.0, 20.0),
                pt(30.0, 40.0),
                pt(35.0, 35.0),
                pt(0.0, 35.0),
                pt(2.0, 35.0),
                pt(2.0, 1.0),
                pt(2.0, 4.0)
            ]
        );

        let contexts = contexts(&segments);
        for pair in contexts.windows(2) {
            assert_eq!(pair[1].start, pair[0].end);
        }
        assert_eq!(contexts[0].start, pt(0.0, 0.0));
        assert!(contexts.iter().enumerate().all(|(i, c)| c.index == i));
    }

    #[test]
    fn first_relative_move_starts_from_origin() {
        let segments = parse("m5 5 l1 1");
        let contexts = contexts(&segments);
        assert_eq!(contexts[0].end, pt(5.0, 5.0));
        assert_eq!(contexts[1].subpath_start, pt(5.0, 5.0));
    }

    #[test]
    fn close_path_returns_to_subpath_start() {
        let segments = parse("M10 10 l10 0 l0 10 z m5 5 l1 1");
        let contexts = contexts(&segments);
        assert_eq!(contexts[3].end, pt(10.0, 10.0));
        // A relative move after a close path starts from the subpath start.
        assert_eq!(contexts[4].end, pt(15.0, 15.0));
        assert_eq!(contexts[5].subpath_start, pt(15.0, 15.0));
    }

    #[test]
    fn arc_ends_at_its_endpoint() {
        let segments = parse("M10 10 a5 5 0 0 1 10 0 A5 5 0 0 1 0 0");
        let ends: Vec<_> = contexts(&segments).iter().map(|c| c.end).collect();
        assert_eq!(ends, [pt(10.0, 10.0), pt(20.0, 10.0), pt(0.0, 0.0)]);
    }

    #[test]
    fn smooth_curve_reflects_previous_cubic_control() {
        let segments = parse("M0 0 C10 0 20 10 30 10 s10 10 20 0 S70 0 80 0");
        let contexts = contexts(&segments);
        assert_eq!(contexts[1].implied_control, None);
        assert_eq!(contexts[2].implied_control, Some(pt(40.0, 10.0)));
        // The relative s has its second control at (40, 20) and ends at
        // (50, 10), so S reflects (40, 20) about (50, 10).
        assert_eq!(contexts[3].implied_control, Some(pt(60.0, 0.0)));
    }

    #[test]
    fn smooth_curve_after_non_cubic_uses_start() {
        let segments = parse("M0 0 Q10 10 20 0 S30 10 40 0");
        let contexts = contexts(&segments);
        assert_eq!(contexts[2].implied_control, Some(pt(20.0, 0.0)));
    }

    #[test]
    fn smooth_quadratic_chains_reflections() {
        let segments = parse("M0 0 Q10 10 20 0 T40 0 t20 0");
        let contexts = contexts(&segments);
        assert_eq!(contexts[2].implied_control, Some(pt(30.0, -10.0)));
        assert_eq!(contexts[3].implied_control, Some(pt(50.0, 10.0)));
    }

    #[test]
    fn smooth_quadratic_after_non_quadratic_uses_start() {
        let segments = parse("M0 0 C1 1 2 2 3 3 T10 0");
        let contexts = contexts(&segments);
        assert_eq!(contexts[2].implied_control, Some(pt(3.0, 3.0)));
    }

    #[test]
    fn empty_path_yields_nothing() {
        assert!(contexts(&[]).is_empty());
    }
}
