use euclid::default::Point2D;
use svgtypes::PathSegment;

/// An SVG basic shape, with the attributes of its element.
///
/// [`Shape::to_path`] returns the equivalent path that SVG 2 defines for it:
/// the same start point, direction and arcs a browser uses, so markers and
/// dashes land in the same places.
///
/// ```
/// use svg_path_ops::shapes::Shape;
/// use svg_path_ops::svgtypes::PointsParser;
/// use svg_path_ops::{write_path, WriteOptions};
///
/// let rect = Shape::Rect {
///     x: 0.0,
///     y: 0.0,
///     width: 20.0,
///     height: 10.0,
///     rx: Some(2.0),
///     ry: None,
/// };
/// assert_eq!(
///     write_path(rect.to_path(), &WriteOptions::default()),
///     "M 2 0 H 18 A 2 2 0 0 1 20 2 V 8 A 2 2 0 0 1 18 10 \
///      H 2 A 2 2 0 0 1 0 8 V 2 A 2 2 0 0 1 2 0 Z"
/// );
///
/// // The points attribute of a <polygon>
/// let triangle = Shape::Polygon(PointsParser::from("0,0 10,0 5,8").map(Into::into).collect());
/// assert_eq!(
///     write_path(triangle.to_path(), &WriteOptions::default()),
///     "M 0 0 L 10 0 L 5 8 Z"
/// );
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    /// A `<rect>`. `rx` and `ry` are `None` when the attribute is absent
    /// (`auto`): one missing radius takes the other's value, and both
    /// missing means square corners.
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        rx: Option<f64>,
        ry: Option<f64>,
    },
    /// A `<circle>`.
    Circle { cx: f64, cy: f64, r: f64 },
    /// An `<ellipse>`. A missing radius (`None`, `auto`) takes the other's
    /// value.
    Ellipse { cx: f64, cy: f64, rx: Option<f64>, ry: Option<f64> },
    /// A `<line>`.
    Line { x1: f64, y1: f64, x2: f64, y2: f64 },
    /// A `<polyline>`: its points joined by lines.
    Polyline(Vec<Point2D<f64>>),
    /// A `<polygon>`: its points joined by lines and closed.
    Polygon(Vec<Point2D<f64>>),
}

impl Shape {
    /// Returns the equivalent path of the shape as SVG 2 defines it. A shape
    /// SVG does not render, such as a rect without width or a circle
    /// without radius, gives an empty path.
    pub fn to_path(&self) -> Vec<PathSegment> {
        match *self {
            Shape::Rect { x, y, width, height, rx, ry } => rect(x, y, width, height, rx, ry),
            Shape::Circle { cx, cy, r } => ellipse(cx, cy, r, r),
            Shape::Ellipse { cx, cy, rx, ry } => {
                let (rx, ry) = auto_radii(rx, ry);
                ellipse(cx, cy, rx, ry)
            }
            Shape::Line { x1, y1, x2, y2 } => vec![
                PathSegment::MoveTo { abs: true, x: x1, y: y1 },
                PathSegment::LineTo { abs: true, x: x2, y: y2 },
            ],
            Shape::Polyline(ref points) => poly(points, false),
            Shape::Polygon(ref points) => poly(points, true),
        }
    }
}

/// Resolves `auto` radii: a missing or negative one takes the other's value,
/// and both missing is zero.
fn auto_radii(rx: Option<f64>, ry: Option<f64>) -> (f64, f64) {
    let valid = |r: Option<f64>| r.filter(|r| *r >= 0.0);
    match (valid(rx), valid(ry)) {
        (Some(rx), Some(ry)) => (rx, ry),
        (Some(r), None) | (None, Some(r)) => (r, r),
        (None, None) => (0.0, 0.0),
    }
}

fn rect(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    rx: Option<f64>,
    ry: Option<f64>,
) -> Vec<PathSegment> {
    if width <= 0.0 || height <= 0.0 {
        return Vec::new();
    }
    let (rx, ry) = auto_radii(rx, ry);
    let (rx, ry) = (rx.min(width / 2.0), ry.min(height / 2.0));
    let rounded = rx > 0.0 && ry > 0.0;
    // A corner with one zero radius is square, so neither radius may
    // shorten the edges
    let (rx, ry) = if rounded { (rx, ry) } else { (0.0, 0.0) };
    let arc = |x, y| PathSegment::EllipticalArc {
        abs: true,
        rx,
        ry,
        x_axis_rotation: 0.0,
        large_arc: false,
        sweep: true,
        x,
        y,
    };

    let mut path = vec![
        PathSegment::MoveTo { abs: true, x: x + rx, y },
        PathSegment::HorizontalLineTo { abs: true, x: x + width - rx },
    ];
    if rounded {
        path.push(arc(x + width, y + ry));
    }
    path.push(PathSegment::VerticalLineTo { abs: true, y: y + height - ry });
    if rounded {
        path.push(arc(x + width - rx, y + height));
    }
    path.push(PathSegment::HorizontalLineTo { abs: true, x: x + rx });
    if rounded {
        path.push(arc(x, y + height - ry));
    }
    path.push(PathSegment::VerticalLineTo { abs: true, y: y + ry });
    if rounded {
        path.push(arc(x + rx, y));
    }
    path.push(PathSegment::ClosePath { abs: true });
    path
}

/// Four quarter arcs clockwise from the rightmost point, as SVG 2 draws
/// circles and ellipses.
fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64) -> Vec<PathSegment> {
    if rx <= 0.0 || ry <= 0.0 {
        return Vec::new();
    }
    let arc = |x, y| PathSegment::EllipticalArc {
        abs: true,
        rx,
        ry,
        x_axis_rotation: 0.0,
        large_arc: false,
        sweep: true,
        x,
        y,
    };
    vec![
        PathSegment::MoveTo { abs: true, x: cx + rx, y: cy },
        arc(cx, cy + ry),
        arc(cx - rx, cy),
        arc(cx, cy - ry),
        arc(cx + rx, cy),
        PathSegment::ClosePath { abs: true },
    ]
}

fn poly(points: &[Point2D<f64>], closed: bool) -> Vec<PathSegment> {
    let mut path: Vec<PathSegment> = points
        .iter()
        .enumerate()
        .map(|(i, point)| {
            if i == 0 {
                PathSegment::MoveTo { abs: true, x: point.x, y: point.y }
            } else {
                PathSegment::LineTo { abs: true, x: point.x, y: point.y }
            }
        })
        .collect();
    if closed && !path.is_empty() {
        path.push(PathSegment::ClosePath { abs: true });
    }
    path
}

#[cfg(test)]
mod test {
    use euclid::default::Point2D;

    use super::Shape;
    use crate::{write_path, WriteOptions};

    fn path(shape: Shape) -> String {
        write_path(shape.to_path(), &WriteOptions::default())
    }

    fn rect(width: f64, height: f64, rx: Option<f64>, ry: Option<f64>) -> Shape {
        Shape::Rect { x: 10.0, y: 20.0, width, height, rx, ry }
    }

    #[test]
    fn square_cornered_rect() {
        assert_eq!(
            path(rect(30.0, 40.0, None, None)),
            "M 10 20 H 40 V 60 H 10 V 20 Z"
        );
    }

    #[test]
    fn rounded_rect_goes_clockwise_from_the_top_left_corner() {
        assert_eq!(
            path(rect(30.0, 40.0, Some(5.0), Some(8.0))),
            "M 15 20 H 35 A 5 8 0 0 1 40 28 V 52 A 5 8 0 0 1 35 60 \
             H 15 A 5 8 0 0 1 10 52 V 28 A 5 8 0 0 1 15 20 Z"
        );
    }

    #[test]
    fn missing_rect_radius_takes_the_other() {
        assert_eq!(
            path(rect(30.0, 40.0, None, Some(4.0))),
            path(rect(30.0, 40.0, Some(4.0), Some(4.0)))
        );
        assert_eq!(
            path(rect(30.0, 40.0, Some(4.0), None)),
            path(rect(30.0, 40.0, Some(4.0), Some(4.0)))
        );
        // A negative radius is an error and counts as missing
        assert_eq!(
            path(rect(30.0, 40.0, Some(-1.0), Some(4.0))),
            path(rect(30.0, 40.0, Some(4.0), Some(4.0)))
        );
    }

    #[test]
    fn rect_radii_are_clamped_to_half_the_size() {
        assert_eq!(
            path(rect(30.0, 40.0, Some(100.0), None)),
            path(rect(30.0, 40.0, Some(15.0), Some(20.0)))
        );
    }

    #[test]
    fn rect_with_a_zero_radius_has_square_corners() {
        assert_eq!(
            path(rect(30.0, 40.0, Some(0.0), Some(5.0))),
            path(rect(30.0, 40.0, None, None))
        );
    }

    #[test]
    fn rect_without_area_is_not_drawn() {
        assert_eq!(path(rect(0.0, 40.0, None, None)), "");
        assert_eq!(path(rect(30.0, -1.0, None, None)), "");
    }

    #[test]
    fn circle_is_four_arcs_from_the_rightmost_point() {
        assert_eq!(
            path(Shape::Circle { cx: 50.0, cy: 50.0, r: 10.0 }),
            "M 60 50 A 10 10 0 0 1 50 60 A 10 10 0 0 1 40 50 \
             A 10 10 0 0 1 50 40 A 10 10 0 0 1 60 50 Z"
        );
        assert_eq!(path(Shape::Circle { cx: 50.0, cy: 50.0, r: 0.0 }), "");
    }

    #[test]
    fn ellipse_radii() {
        assert_eq!(
            path(Shape::Ellipse { cx: 0.0, cy: 0.0, rx: Some(20.0), ry: Some(10.0) }),
            "M 20 0 A 20 10 0 0 1 0 10 A 20 10 0 0 1 -20 0 \
             A 20 10 0 0 1 0 -10 A 20 10 0 0 1 20 0 Z"
        );
        assert_eq!(
            path(Shape::Ellipse { cx: 0.0, cy: 0.0, rx: None, ry: Some(10.0) }),
            path(Shape::Circle { cx: 0.0, cy: 0.0, r: 10.0 })
        );
        assert_eq!(
            path(Shape::Ellipse { cx: 0.0, cy: 0.0, rx: None, ry: None }),
            ""
        );
    }

    #[test]
    fn line() {
        assert_eq!(
            path(Shape::Line { x1: 1.0, y1: 2.0, x2: 3.0, y2: 4.0 }),
            "M 1 2 L 3 4"
        );
    }

    #[test]
    fn polyline_and_polygon() {
        let points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(5.0, 8.0),
        ];
        assert_eq!(path(Shape::Polyline(points.clone())), "M 0 0 L 10 0 L 5 8");
        assert_eq!(path(Shape::Polygon(points)), "M 0 0 L 10 0 L 5 8 Z");
        assert_eq!(path(Shape::Polygon(Vec::new())), "");
    }
}
