use std::borrow::Borrow;

use svgtypes::PathSegment;

pub fn absolutize<'a>(
    path_segments: impl Iterator<Item = impl Borrow<PathSegment>>,
) -> impl Iterator<Item = PathSegment> {
    let mut result = vec![];
    let (mut cx, mut cy, mut subx, mut suby) = (0.0, 0.0, 0.0, 0.0);
    for segment in path_segments {
        match segment.borrow() {
            &PathSegment::MoveTo { abs: true, x, y } => {
                cx = x;
                cy = y;
                subx = x;
                suby = y;
                result.push(segment.borrow().clone())
            }
            &PathSegment::MoveTo { abs: false, x, y } => {
                cx += x;
                cy += y;
                result.push(PathSegment::MoveTo {
                    abs: true,
                    x: cx,
                    y: cy,
                });
                subx = cx;
                suby = cy;
            }
            &PathSegment::LineTo { abs: true, x, y } => {
                result.push(segment.borrow().clone());
                cx = x;
                cy = y;
            }
            &PathSegment::LineTo { abs: false, x, y } => {
                cx += x;
                cy += y;
                result.push(PathSegment::LineTo {
                    abs: true,
                    x: cx,
                    y: cy,
                });
            }
            &PathSegment::CurveTo {
                abs: true,
                x1: _,
                y1: _,
                x2: _,
                y2: _,
                x,
                y,
            } => {
                result.push(segment.borrow().clone());
                cx = x;
                cy = y;
            }
            &PathSegment::CurveTo {
                abs: false,
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                result.push(PathSegment::CurveTo {
                    abs: true,
                    x1: x1 + cx,
                    y1: y1 + cy,
                    x2: x2 + cx,
                    y2: y2 + cy,
                    x: x + cx,
                    y: y + cy,
                });
                cx += x;
                cy += y;
            }
            &PathSegment::Quadratic {
                abs: true,
                x1: _,
                y1: _,
                x,
                y,
            } => {
                result.push(segment.borrow().clone());
                cx = x;
                cy = y;
            }
            &PathSegment::Quadratic {
                abs: false,
                x1,
                y1,
                x,
                y,
            } => {
                result.push(PathSegment::Quadratic {
                    abs: true,
                    x1: x1 + cx,
                    y1: y1 + cy,
                    x: x + cx,
                    y: y + cy,
                });
                cx += x;
                cy += y;
            }
            &PathSegment::EllipticalArc {
                abs: true,
                rx: _,
                ry: _,
                x_axis_rotation: _,
                large_arc: _,
                sweep: _,
                x,
                y,
            } => {
                result.push(segment.borrow().clone());
                cx = x;
                cy = y;
            }
            &PathSegment::EllipticalArc {
                abs: false,
                rx,
                ry,
                x_axis_rotation,
                large_arc,
                sweep,
                x,
                y,
            } => {
                cx += x;
                cy += y;
                result.push(PathSegment::EllipticalArc {
                    abs: true,
                    rx,
                    ry,
                    x_axis_rotation,
                    large_arc,
                    sweep,
                    x: cx,
                    y: cy,
                });
            }
            &PathSegment::HorizontalLineTo { abs: true, x } => {
                result.push(segment.borrow().clone());
                cx = x;
            }
            &PathSegment::HorizontalLineTo { abs: false, x } => {
                cx += x;
                result.push(PathSegment::HorizontalLineTo { abs: true, x: cx });
            }
            &PathSegment::VerticalLineTo { abs: true, y } => {
                result.push(segment.borrow().clone());
                cy = y;
            }
            &PathSegment::VerticalLineTo { abs: false, y } => {
                cy += y;
                result.push(PathSegment::VerticalLineTo { abs: true, y: cy });
            }
            &PathSegment::SmoothCurveTo {
                abs: true,
                x2: _,
                y2: _,
                x,
                y,
            } => {
                result.push(segment.borrow().clone());
                cx = x;
                cy = y;
            }
            &PathSegment::SmoothCurveTo {
                abs: false,
                x2,
                y2,
                x,
                y,
            } => {
                result.push(PathSegment::SmoothCurveTo {
                    abs: true,
                    x2: x2 + cx,
                    y2: y2 + cy,
                    x: x + cx,
                    y: y + cy,
                });
                cx += x;
                cy += y;
            }
            &PathSegment::SmoothQuadratic { abs: true, x, y } => {
                result.push(segment.borrow().clone());
                cx = x;
                cy = y;
            }
            &PathSegment::SmoothQuadratic { abs: false, x, y } => {
                cx += x;
                cy += y;
                result.push(PathSegment::SmoothQuadratic {
                    abs: true,
                    x: cx,
                    y: cy,
                });
            }
            &PathSegment::ClosePath { .. } => {
                result.push(segment.borrow().clone());
                cx = subx;
                cy = suby;
            }
        }
    }

    return result.into_iter();
}
