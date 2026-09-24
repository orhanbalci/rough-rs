use kurbo::Point;
use svgtypes::PathSegment;

use crate::context::{segments_with_context, SegmentContext};
use crate::PathMeasure;

impl PathMeasure {
    /// The path with the segment at `length` divided there into two, as
    /// Paper.js's `divideAt` does: the path draws the same shape, with one
    /// more point where segments meet.
    ///
    /// The two segments are of the kind the divided one was, and absolute or
    /// relative as it was: a line becomes two lines, a horizontal or vertical
    /// line two of the same, a curve two curves and an arc two arcs. A close
    /// path becomes a line to the point and the close path. A smooth curve
    /// (`S`/`T`) is divided into two full curves; so is a smooth curve right
    /// after the divided one, which took its first control point from the
    /// control point the division moved. Every other segment stays exactly
    /// as it is.
    ///
    /// A `length` at a point where segments already meet, at either end of
    /// the path or outside it changes nothing.
    ///
    /// ```
    /// use svg_path_ops::svgtypes::PathParser;
    /// use svg_path_ops::{write_path, PathMeasure, WriteOptions};
    ///
    /// let segments: Vec<_> = PathParser::from("M 0 0 h 10 v 10 z").collect::<Result<_, _>>()?;
    /// let divided = PathMeasure::new(&segments).divide_at(4.0);
    /// assert_eq!(
    ///     write_path(&divided, &WriteOptions::default()),
    ///     "M 0 0 h 4 h 6 v 10 z"
    /// );
    /// # Ok::<(), svg_path_ops::svgtypes::Error>(())
    /// ```
    pub fn divide_at(&self, length: f64) -> Vec<PathSegment> {
        let mut segments = self.segments.clone();
        let Some((i, t)) = self.locate(length) else {
            return segments;
        };
        let piece = &self.pieces[i];
        if t <= 0.0 || t >= 1.0 || piece.length == 0.0 {
            return segments;
        }
        let contexts: Vec<SegmentContext<'_>> = segments_with_context(&self.segments).collect();
        let context = &contexts[piece.index];
        let split = piece.shape.eval(t);
        let before = in_form(
            piece.shape.segment(0.0, t),
            context.segment,
            piece.from,
            split,
        );
        let after = match context.segment {
            PathSegment::ClosePath { .. } => *context.segment,
            original => in_form(piece.shape.segment(t, 1.0), original, split, piece.to),
        };

        // A smooth curve after the divided one reflected the control point
        // the division moved: write the control point it had in full
        let next = contexts.get(piece.index + 1).and_then(|next| {
            let control = next.implied_control?;
            let at = |abs: bool| {
                if abs {
                    control
                } else {
                    (control - next.start).to_point()
                }
            };
            match *next.segment {
                PathSegment::SmoothCurveTo { abs, x2, y2, x, y } => {
                    let c = at(abs);
                    Some(PathSegment::CurveTo { abs, x1: c.x, y1: c.y, x2, y2, x, y })
                }
                PathSegment::SmoothQuadratic { abs, x, y } => {
                    let c = at(abs);
                    Some(PathSegment::Quadratic { abs, x1: c.x, y1: c.y, x, y })
                }
                _ => None,
            }
        });
        if let Some(next) = next {
            segments[piece.index + 1] = next;
        }
        segments.splice(piece.index..=piece.index, [before, after]);
        segments
    }
}

/// `part`, an absolute segment of the shape `original` draws, from `start`
/// to `end`, written the way `original` is: absolute or relative, and as a
/// horizontal or vertical line where it was one. A smooth curve is written
/// in full, and a close path becomes a line.
fn in_form(part: PathSegment, original: &PathSegment, start: Point, end: Point) -> PathSegment {
    let abs = match *original {
        PathSegment::MoveTo { abs, .. }
        | PathSegment::LineTo { abs, .. }
        | PathSegment::HorizontalLineTo { abs, .. }
        | PathSegment::VerticalLineTo { abs, .. }
        | PathSegment::CurveTo { abs, .. }
        | PathSegment::SmoothCurveTo { abs, .. }
        | PathSegment::Quadratic { abs, .. }
        | PathSegment::SmoothQuadratic { abs, .. }
        | PathSegment::EllipticalArc { abs, .. }
        | PathSegment::ClosePath { abs } => abs,
    };
    // A point written the way the segment writes its points
    let at = |x: f64, y: f64| {
        if abs {
            (x, y)
        } else {
            (x - start.x, y - start.y)
        }
    };
    let (x, y) = at(end.x, end.y);
    match (part, original) {
        (_, PathSegment::HorizontalLineTo { .. }) => PathSegment::HorizontalLineTo { abs, x },
        (_, PathSegment::VerticalLineTo { .. }) => PathSegment::VerticalLineTo { abs, y },
        (PathSegment::CurveTo { x1, y1, x2, y2, .. }, _) => {
            let ((x1, y1), (x2, y2)) = (at(x1, y1), at(x2, y2));
            PathSegment::CurveTo { abs, x1, y1, x2, y2, x, y }
        }
        (PathSegment::Quadratic { x1, y1, .. }, _) => {
            let (x1, y1) = at(x1, y1);
            PathSegment::Quadratic { abs, x1, y1, x, y }
        }
        (PathSegment::EllipticalArc { rx, ry, x_axis_rotation, large_arc, sweep, .. }, _) => {
            PathSegment::EllipticalArc {
                abs,
                rx,
                ry,
                x_axis_rotation,
                large_arc,
                sweep,
                x,
                y,
            }
        }
        _ => PathSegment::LineTo { abs, x, y },
    }
}

#[cfg(test)]
mod test {
    use svgtypes::{PathParser, PathSegment};

    use crate::write::{write_path, WriteOptions};
    use crate::PathMeasure;

    fn parse(path: &str) -> Vec<PathSegment> {
        PathParser::from(path).collect::<Result<_, _>>().unwrap()
    }

    fn divided(path: &str, length: f64) -> String {
        write_path(
            PathMeasure::new(parse(path)).divide_at(length),
            &WriteOptions::default(),
        )
    }

    /// Checks that dividing `path` at `length` draws the same shape with
    /// one more segment, meeting at the point at `length`.
    fn check(path: &str, length: f64) -> Vec<PathSegment> {
        let measure = PathMeasure::new(parse(path));
        let segments = measure.divide_at(length);
        assert_eq!(segments.len(), measure.segments.len() + 1);
        let after = PathMeasure::new(&segments);
        assert!((after.total_length() - measure.total_length()).abs() < 1e-9);
        for k in 0..=50 {
            let at = measure.total_length() * f64::from(k) / 50.0;
            let (p, q) = (measure.point_at(at).unwrap(), after.point_at(at).unwrap());
            // Lengths are measured to 1e-9, points at them a little less
            assert!((p - q).length() < 1e-7, "{p:?} {q:?} at {at}");
        }
        segments
    }

    #[test]
    fn divides_lines_keeping_their_form() {
        assert_eq!(divided("M 0 0 L 10 0", 4.0), "M 0 0 L 4 0 L 10 0");
        assert_eq!(divided("M 1 1 l 10 0", 4.0), "M 1 1 l 4 0 l 6 0");
        assert_eq!(divided("M 0 0 V 10", 4.0), "M 0 0 V 4 V 10");
        assert_eq!(divided("M 0 0 h 10 v 10", 14.0), "M 0 0 h 10 v 4 v 6");
    }

    #[test]
    fn divides_a_close_path_into_a_line_and_the_close_path() {
        let segments = check("M 0 0 h 10 v 10 z", 24.0);
        assert!(matches!(
            segments[3],
            PathSegment::LineTo { abs: false, .. }
        ));
        assert!(matches!(segments[4], PathSegment::ClosePath { abs: false }));
        assert_eq!(
            divided("M 0 0 H 10 V 10 H 0 Z", 33.0),
            "M 0 0 H 10 V 10 H 0 L 0 7 Z"
        );
    }

    #[test]
    fn divides_curves_into_curves() {
        for path in [
            "M 0 0 C 0 10 20 10 20 0",
            "M 5 5 c 0 10 20 10 20 0",
            "M 0 0 Q 10 20 20 0",
            "M 5 5 q 10 20 20 0",
        ] {
            let segments = check(path, 12.0);
            assert_eq!(
                std::mem::discriminant(&segments[1]),
                std::mem::discriminant(&segments[2])
            );
        }
    }

    #[test]
    fn divides_arcs_into_arcs() {
        let segments = check("M 10 0 A 10 10 0 1 1 -10 0", 20.0);
        assert!(matches!(segments[1], PathSegment::EllipticalArc { .. }));
        assert!(matches!(segments[2], PathSegment::EllipticalArc { .. }));
        // A large arc divided into two small ones
        check("M 10 0 a 10 5 30 1 0 -20 0", 30.0);
    }

    #[test]
    fn smooth_curves_are_written_in_full_where_they_must_be() {
        // Dividing the S, and dividing the C before an S
        let path = "M 0 0 C 0 10 10 10 10 0 S 20 -10 20 0";
        let divided = check(path, 25.0);
        assert!(matches!(divided[2], PathSegment::CurveTo { .. }));
        assert!(matches!(divided[3], PathSegment::CurveTo { .. }));
        let divided = check(path, 5.0);
        assert!(matches!(divided[3], PathSegment::CurveTo { .. }));
        let divided = check("M 0 0 Q 5 10 10 0 t 10 0 t 10 0", 5.0);
        assert!(matches!(divided[3], PathSegment::Quadratic { .. }));
        assert!(matches!(divided[4], PathSegment::SmoothQuadratic { .. }));
    }

    #[test]
    fn nothing_changes_at_joins_or_outside() {
        let path = "M 0 0 h 10 v 10";
        for length in [-5.0, 0.0, 10.0, 20.0, 30.0] {
            assert_eq!(divided(path, length), "M 0 0 h 10 v 10");
        }
    }
}
