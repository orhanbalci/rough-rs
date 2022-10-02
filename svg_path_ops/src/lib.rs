use std::borrow::Borrow;
use std::f64::consts::PI;

use svgtypes::PathSegment;

pub fn absolutize(
    path_segments: impl Iterator<Item = impl Borrow<PathSegment>>,
) -> impl Iterator<Item = PathSegment> {
    let mut result = vec![];
    let (mut cx, mut cy, mut subx, mut suby) = (0.0, 0.0, 0.0, 0.0);
    for segment in path_segments {
        match *segment.borrow() {
            PathSegment::MoveTo { abs: true, x, y } => {
                cx = x;
                cy = y;
                subx = x;
                suby = y;
                result.push(*segment.borrow())
            }
            PathSegment::MoveTo { abs: false, x, y } => {
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
            PathSegment::LineTo { abs: true, x, y } => {
                result.push(*segment.borrow());
                cx = x;
                cy = y;
            }
            PathSegment::LineTo { abs: false, x, y } => {
                cx += x;
                cy += y;
                result.push(PathSegment::LineTo {
                    abs: true,
                    x: cx,
                    y: cy,
                });
            }
            PathSegment::CurveTo {
                abs: true,
                x1: _,
                y1: _,
                x2: _,
                y2: _,
                x,
                y,
            } => {
                result.push(*segment.borrow());
                cx = x;
                cy = y;
            }
            PathSegment::CurveTo {
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
            PathSegment::Quadratic {
                abs: true,
                x1: _,
                y1: _,
                x,
                y,
            } => {
                result.push(*segment.borrow());
                cx = x;
                cy = y;
            }
            PathSegment::Quadratic {
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
            PathSegment::EllipticalArc {
                abs: true,
                rx: _,
                ry: _,
                x_axis_rotation: _,
                large_arc: _,
                sweep: _,
                x,
                y,
            } => {
                result.push(*segment.borrow());
                cx = x;
                cy = y;
            }
            PathSegment::EllipticalArc {
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
            PathSegment::HorizontalLineTo { abs: true, x } => {
                result.push(*segment.borrow());
                cx = x;
            }
            PathSegment::HorizontalLineTo { abs: false, x } => {
                cx += x;
                result.push(PathSegment::HorizontalLineTo { abs: true, x: cx });
            }
            PathSegment::VerticalLineTo { abs: true, y } => {
                result.push(*segment.borrow());
                cy = y;
            }
            PathSegment::VerticalLineTo { abs: false, y } => {
                cy += y;
                result.push(PathSegment::VerticalLineTo { abs: true, y: cy });
            }
            PathSegment::SmoothCurveTo {
                abs: true,
                x2: _,
                y2: _,
                x,
                y,
            } => {
                result.push(*segment.borrow());
                cx = x;
                cy = y;
            }
            PathSegment::SmoothCurveTo {
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
            PathSegment::SmoothQuadratic { abs: true, x, y } => {
                result.push(*segment.borrow());
                cx = x;
                cy = y;
            }
            PathSegment::SmoothQuadratic { abs: false, x, y } => {
                cx += x;
                cy += y;
                result.push(PathSegment::SmoothQuadratic {
                    abs: true,
                    x: cx,
                    y: cy,
                });
            }
            PathSegment::ClosePath { .. } => {
                result.push(*segment.borrow());
                cx = subx;
                cy = suby;
            }
        }
    }

    result.into_iter()
}

pub fn normalize<'a>(
    path_segments: impl Iterator<Item = impl Borrow<PathSegment>>,
) -> impl Iterator<Item = PathSegment> {
    let mut out = vec![];

    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut subx = 0.0;
    let mut suby = 0.0;
    let mut lcx = 0.0;
    let mut lcy = 0.0;
    let mut last_type: PathSegment;

    for segment in path_segments {
        last_type = *segment.borrow();
        match *segment.borrow() {
            PathSegment::MoveTo { abs: true, x, y } => {
                out.push(*segment.borrow());
                cx = x;
                cy = y;
                subx = x;
                suby = y;
            }
            PathSegment::CurveTo {
                abs: true,
                x1: _,
                y1: _,
                x2,
                y2,
                x,
                y,
            } => {
                out.push(*segment.borrow());
                cx = x;
                cy = y;
                lcx = x2;
                lcy = y2;
            }
            PathSegment::LineTo { abs: true, x, y } => {
                out.push(*segment.borrow());
                cx = x;
                cy = y;
            }
            PathSegment::HorizontalLineTo { abs: true, x } => {
                cx = x;
                out.push(PathSegment::LineTo {
                    abs: true,
                    x: cx,
                    y: cy,
                });
            }
            PathSegment::VerticalLineTo { abs: true, y } => {
                cy = y;
                out.push(PathSegment::LineTo {
                    abs: true,
                    x: cx,
                    y: cy,
                });
            }
            PathSegment::SmoothCurveTo {
                abs: true,
                x2,
                y2,
                x,
                y,
            } => {
                let cx1;
                let cy1;
                if matches!(last_type, PathSegment::CurveTo { .. })
                    || matches!(last_type, PathSegment::SmoothCurveTo { .. })
                {
                    cx1 = cx + (cx - lcx);
                    cy1 = cy + (cy - lcy);
                } else {
                    cx1 = cx;
                    cy1 = cy;
                }
                out.push(PathSegment::CurveTo {
                    abs: true,
                    x1: cx1,
                    y1: cy1,
                    x2,
                    y2,
                    x,
                    y,
                });
                lcx = x2;
                lcy = y2;
                cx = x;
                cy = y;
            }
            PathSegment::SmoothQuadratic { abs: true, x, y } => {
                let x1;
                let y1;
                if matches!(last_type, PathSegment::Quadratic { .. })
                    || matches!(last_type, PathSegment::SmoothQuadratic { .. })
                {
                    x1 = cx + (cx - lcx);
                    y1 = cy + (cy - lcy);
                } else {
                    x1 = cx;
                    y1 = cy;
                }
                let cx1 = cx + 2.0 * (x1 - cx) / 3.0;
                let cy1 = cy + 2.0 * (y1 - cy) / 3.0;
                let cx2 = x + 2.0 * (x1 - x) / 3.0;
                let cy2 = y + 2.0 * (y1 - y) / 3.0;
                out.push(PathSegment::CurveTo {
                    abs: true,
                    x1: cx1,
                    y1: cy1,
                    x2: cx2,
                    y2: cy2,
                    x,
                    y,
                });
                lcx = x1;
                lcy = y1;
                cx = x;
                cy = y;
            }
            PathSegment::Quadratic {
                abs: true,
                x1,
                y1,
                x,
                y,
            } => {
                let cx1 = cx + 2.0 * (x1 - cx) / 3.0;
                let cy1 = cy + 2.0 * (y1 - cy) / 3.0;
                let cx2 = x + 2.0 * (x1 - x) / 3.0;
                let cy2 = y + 2.0 * (y1 - y) / 3.0;
                out.push(PathSegment::CurveTo {
                    abs: true,
                    x1: cx1,
                    y1: cy1,
                    x2: cx2,
                    y2: cy2,
                    x,
                    y,
                });
                lcx = x1;
                lcy = y1;
                cx = x;
                cy = y;
            }
            PathSegment::EllipticalArc {
                abs: true,
                rx,
                ry,
                x_axis_rotation,
                large_arc,
                sweep,
                x,
                y,
            } => {
                let r1 = rx.abs();
                let r2 = ry.abs();
                let angle = x_axis_rotation;
                let large_arc_flag = large_arc;
                let sweep_flag = sweep;
                if r1 == 0.0 || r2 == 0.0 {
                    out.push(PathSegment::CurveTo {
                        abs: true,
                        x1: cx,
                        y1: cy,
                        x2: x,
                        y2: y,
                        x,
                        y,
                    });
                    cx = x;
                    cy = y;
                } else {
                    if cx != x || cy != y {
                        let curves = arc_to_cubic_curves(
                            cx,
                            cy,
                            x,
                            y,
                            r1,
                            r2,
                            angle,
                            large_arc_flag,
                            sweep_flag,
                            None,
                        );
                        for curve in curves.iter() {
                            out.push(PathSegment::CurveTo {
                                abs: true,
                                x1: curve[0],
                                y1: curve[1],
                                x2: curve[2],
                                y2: curve[3],
                                x: curve[4],
                                y: curve[5],
                            })
                        }
                        cx = x;
                        cy = y;
                    }
                }
            }
            PathSegment::ClosePath { abs: true } => {
                out.push(*segment.borrow());
                cx = subx;
                cy = suby;
            }
            _ => panic!("Not expecting none absolute path!"),
        }
    }

    out.into_iter()
}

fn rotate(x: f64, y: f64, angle_rad: f64) -> (f64, f64) {
    let rotated_x = x * angle_rad.cos() - y * angle_rad.sin();
    let rotated_y = x * angle_rad.sin() + y * angle_rad.cos();
    (rotated_x, rotated_y)
}

//  porting ts library to rust https://github.com/pshihn/path-data-parser
pub fn arc_to_cubic_curves(
    x1: f64,
    y1: f64,
    mut x2: f64,
    mut y2: f64,
    mut r1: f64,
    mut r2: f64,
    angle: f64,
    large_arc_flag: bool,
    sweep_flag: bool,
    recursive: Option<[f64; 4]>,
) -> Vec<Vec<f64>> {
    let angle_rad = angle.to_radians();
    let mut params: Vec<Vec<f64>> = vec![];
    let (mut f1, mut f2, cx, cy);
    if let Some(rec) = recursive {
        f1 = rec[0];
        f2 = rec[1];
        cx = rec[2];
        cy = rec[3];
    } else {
        let (x1, y1) = rotate(x1, y1, -angle_rad);
        let (x2, y2) = rotate(x2, y2, -angle_rad);
        let x = (x1 - x2) / 2.0;
        let y = (y1 - y2) / 2.0;
        let mut h = (x * x) / (r1 * r1) + (y * y) / (r2 * r2);
        if h > 1.0 {
            h = h.sqrt();
            r1 *= h;
            r2 *= h;
        }
        let sign = if large_arc_flag == sweep_flag {
            -1.0
        } else {
            1.0
        };
        let r1_pow = r1.powi(2);
        let r2_pow = r2.powi(2);
        let left = r1_pow * r2_pow - r1_pow * y * y - r2_pow * x * x;
        let right = r1_pow * y * y + r2_pow * x * x;

        let k = sign * (left / right).abs().sqrt();

        cx = k * r1 * y / r2 + (x1 + x2) / 2.0;
        cy = k * -r2 * x / r1 + (y1 + y2) / 2.0;

        f1 = ((y1 - cy) / r2).asin();
        f2 = ((y2 - cy) / r2).asin();

        if x1 < cx {
            f1 = PI - f1;
        }
        if x2 < cx {
            f2 = PI - f2;
        }

        if f1 < 0.0 {
            f1 += PI * 2.0;
        }
        if f2 < 0.0 {
            f2 += PI * 2.0;
        }

        if sweep_flag && f1 > f2 {
            f1 -= PI * 2.0;
        }
        if !sweep_flag && f2 > f1 {
            f2 -= PI * 2.0;
        }
    }

    let df = f2 - f1;
    if df.abs() > (PI * 120.0 / 180.0) {
        let f2old = f2;
        let x2old = x2;
        let y2old = y2;

        if sweep_flag && f2 > f1 {
            f2 = f1 + (PI * 120.0 / 180.0) * (1.0);
        } else {
            f2 = f1 + (PI * 120.0 / 180.0) * (-1.0);
        }

        x2 = cx + r1 * (f2.cos());
        y2 = cy + r2 * (f2.sin());
        params = arc_to_cubic_curves(
            x2,
            y2,
            x2old,
            y2old,
            r1,
            r2,
            angle,
            false,
            sweep_flag,
            Some([f2, f2old, cx, cy]),
        );
    }

    // let idf = f2 - f1;

    let c1 = f1.sin();
    let s1 = f1.sin();
    let c2 = f2.cos();
    let s2 = f2.sin();
    let t = (df / 4.0).tan();
    let hx = 4.0 / 3.0 * r1 * t;
    let hy = 4.0 / 3.0 * r2 * t;

    let m1 = vec![x1, y1];
    let mut m2 = vec![x1 + hx * s1, y1 - hy * c1];
    let m3 = vec![x2 + hx * s2, y2 - hy * c2];
    let m4 = vec![x2, y2];

    m2[0] = 2.0 * m1[0] - m2[0];
    m2[1] = 2.0 * m1[1] - m2[1];

    if recursive.is_some() {
        let mut ret_val = vec![m2, m3, m4];
        ret_val.append(&mut params);
        ret_val
    } else {
        let mut ret_val = vec![m2, m3, m4];
        ret_val.append(&mut params);
        let mut curves = vec![];
        for i in (0..ret_val.len()).step_by(3) {
            let r1 = rotate(params[i][0], params[i][1], angle_rad);
            let r2 = rotate(params[i + 1][0], params[i + 1][1], angle_rad);
            let r3 = rotate(params[i + 2][0], params[i + 2][1], angle_rad);
            curves.push(vec![r1.0, r1.1, r2.0, r2.1, r3.0, r3.1]);
        }
        curves
    }
}
