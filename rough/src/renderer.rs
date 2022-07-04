use std::fmt::Display;

use euclid::default::Point2D;
use euclid::Trig;
use num_traits::{Float, FloatConst, FromPrimitive};

use super::core::{Options, _c};
use crate::core::{Op, OpSet, OpSetType, OpType};

pub struct EllipseParams<F: Float> {
    pub rx: F,
    pub ry: F,
    pub increment: F,
}

pub struct EllipseResult<F: Float + FromPrimitive + Trig> {
    pub opset: OpSet<F>,
    pub estimated_points: Vec<Point2D<F>>,
}

pub fn line<F: Float + Trig + FromPrimitive>(
    x1: F,
    y1: F,
    x2: F,
    y2: F,
    o: &mut Options,
) -> OpSet<F> {
    OpSet {
        op_set_type: OpSetType::Path,
        ops: _double_line(x1, y1, x2, y2, o, false),
        size: None,
        path: None,
    }
}

pub fn linear_path<F: Float + Trig + FromPrimitive>(
    points: &[Point2D<F>],
    close: bool,
    o: &mut Options,
) -> OpSet<F> {
    let len = points.len();
    if len > 2 {
        let mut ops: Vec<Op<F>> = Vec::new();
        let mut i = 0;
        while i < (len - 1) {
            ops.append(&mut _double_line(
                points[i].x,
                points[i].y,
                points[i + 1].x,
                points[i + 1].y,
                o,
                false,
            ));
            i += 1;
        }
        if close {
            ops.append(&mut _double_line(
                points[len - 1].x,
                points[len - 1].y,
                points[0].x,
                points[0].y,
                o,
                false,
            ));
        }
        OpSet {
            op_set_type: OpSetType::Path,
            ops: ops,
            path: None,
            size: None,
        }
    } else if len == 2 {
        line(points[0].x, points[0].y, points[1].x, points[1].y, o)
    } else {
        OpSet {
            op_set_type: OpSetType::Path,
            ops: Vec::new(),
            path: None,
            size: None,
        }
    }
}

pub fn polygon<F: Float + Trig + FromPrimitive>(
    points: &[Point2D<F>],
    o: &mut Options,
) -> OpSet<F> {
    linear_path(points, true, o)
}

pub fn rectangle<F: Float + Trig + FromPrimitive>(
    x: F,
    y: F,
    width: F,
    height: F,
    o: &mut Options,
) -> OpSet<F> {
    let points: Vec<Point2D<F>> = vec![
        Point2D::new(x, y),
        Point2D::new(x + width, y),
        Point2D::new(x + width, y + height),
        Point2D::new(x, y + height),
    ];
    polygon(&points, o)
}

pub fn curve<F: Float + Trig + FromPrimitive>(points: &[Point2D<F>], o: &mut Options) -> OpSet<F> {
    let mut o1 = _curve_with_offset(
        points,
        _c::<F>(1.0) * _c(1.0 + o.roughness.unwrap_or(0.0) * 0.2),
        o,
    );
    if !o.disable_multi_stroke.unwrap_or(false) {
        let mut o2 = _curve_with_offset(
            points,
            _c::<F>(1.5) * _c(1.0 + o.roughness.unwrap_or(0.0) * 0.22),
            &mut clone_options_alter_seed(o),
        );
        o1.append(&mut o2);
    }
    OpSet {
        op_set_type: OpSetType::Path,
        ops: o1,
        path: None,
        size: None,
    }
}

pub fn ellipse<F: Float + Trig + FromPrimitive>(
    x: F,
    y: F,
    width: F,
    height: F,
    o: &mut Options,
) -> OpSet<F> {
    let params = generate_ellipse_params(width, height, o);
    ellipse_with_params(x, y, o, &params).opset
}

pub fn generate_ellipse_params<F: Float + Trig + FromPrimitive>(
    width: F,
    height: F,
    o: &mut Options,
) -> EllipseParams<F> {
    let psq: F = Float::sqrt(
        _c::<F>(f32::PI())
            * _c(2.0)
            * Float::sqrt(
                (Float::powi(width / _c(2.0), 2) + Float::powi(height / _c(2.0), 2)) / _c(2.0),
            ),
    );
    let step_count: F = Float::ceil(Float::max(
        _c(o.curve_step_count.unwrap_or(1.0)),
        _c::<F>(o.curve_step_count.unwrap_or(1.0) / Float::sqrt(200.0)) * psq,
    ));
    let increment: F = (_c::<F>(f32::PI()) * _c(2.0)) / step_count;
    let mut rx = Float::abs(width / _c(2.0));
    let mut ry = Float::abs(height / _c(2.0));
    let curve_fit_randomness: F = _c::<F>(1.0) - _c(o.curve_fitting.unwrap_or(0.0));
    rx = rx + _offset_opt(rx * curve_fit_randomness, o, None);
    ry = ry + _offset_opt(ry * curve_fit_randomness, o, None);
    EllipseParams { increment, rx, ry }
}

pub fn ellipse_with_params<F: Float + Trig + FromPrimitive>(
    x: F,
    y: F,
    o: &mut Options,
    ellipse_params: &EllipseParams<F>,
) -> EllipseResult<F> {
    let ellipse_points = _compute_ellipse_points(
        ellipse_params.increment,
        x,
        y,
        ellipse_params.rx,
        ellipse_params.ry,
        _c(1.0),
        ellipse_params.increment
            * _offset(
                _c(0.1),
                _offset(_c::<F>(0.4), _c::<F>(1.0), o, None),
                o,
                None,
            ),
        o,
    );
    let ap1 = ellipse_points[0].clone();
    let cp1 = ellipse_points[1].clone();
    let mut o1 = _curve(&ap1, None, o);
    if (!o.disable_multi_stroke.unwrap_or(false)) && (o.roughness.unwrap_or(0.0) != 0.0) {
        let inner_ellipse_points = _compute_ellipse_points(
            ellipse_params.increment,
            x,
            y,
            ellipse_params.rx,
            ellipse_params.ry,
            _c::<F>(1.5),
            _c::<F>(0.0),
            o,
        );
        let ap2 = inner_ellipse_points[0].clone();
        let _cp2 = inner_ellipse_points[1].clone();
        let mut o2 = _curve(&ap2, None, o);
        o1.append(&mut o2);
    }
    EllipseResult {
        estimated_points: cp1,
        opset: OpSet {
            op_set_type: OpSetType::Path,
            ops: o1,
            size: None,
            path: None,
        },
    }
}

pub fn arc<F: Float + Trig + FromPrimitive>(
    x: F,
    y: F,
    width: F,
    height: F,
    start: F,
    stop: F,
    closed: bool,
    rough_closure: bool,
    o: &mut Options,
) -> OpSet<F> {
    let cx = x;
    let cy = y;
    let mut rx = Float::abs(width / _c(2.0));
    let mut ry = Float::abs(height / _c(2.0));
    rx = rx + _offset_opt(rx * _c(0.01), o, None);
    ry = ry + _offset_opt(ry * _c(0.01), o, None);
    let mut strt: F = start;
    let mut stp: F = stop;
    while strt < _c(0.0) {
        strt = strt + _c(f32::PI() * 2.0);
        stp = stp + _c(f32::PI() * 2.0);
    }
    if (stp - strt) > _c(f32::PI() * 2.0) {
        strt = _c(0.0);
        stp = _c(f32::PI() * 2.0);
    }
    let ellipse_inc: F = _c::<F>(f32::PI() * 2.0) / _c(o.curve_step_count.unwrap_or(1.0));
    let arc_inc = Float::min(ellipse_inc / _c(2.0), (stp - strt) / _c(2.0));
    let mut ops = _arc(arc_inc, cx, cy, rx, ry, strt, stp, _c(1.0), o);
    if !o.disable_multi_stroke.unwrap_or(false) {
        let mut o2 = _arc(arc_inc, cx, cy, rx, ry, strt, stp, _c(1.5), o);
        ops.append(&mut o2);
    }
    if closed {
        if rough_closure {
            ops.append(&mut _double_line(
                cx,
                cy,
                cx + rx * Float::cos(strt),
                cy + ry * Float::sin(strt),
                o,
                false,
            ));
            ops.append(&mut _double_line(
                cx,
                cy,
                cx + rx * Float::cos(stp),
                cy + ry * Float::sin(stp),
                o,
                false,
            ));
        } else {
            ops.push(Op {
                op: OpType::LineTo,
                data: vec![cx, cy],
            });
            ops.push(Op {
                op: OpType::LineTo,
                data: vec![cx + rx * Float::cos(strt), cy + ry * Float::sin(strt)],
            });
        }
    }
    OpSet {
        op_set_type: OpSetType::Path,
        ops,
        path: None,
        size: None,
    }
}

pub fn solid_fill_polygon<F: Float + Trig + FromPrimitive>(
    polygon_list: &Vec<Vec<Point2D<F>>>,
    options: &mut Options,
) -> OpSet<F> {
    let mut ops = vec![];
    for polygon in polygon_list {
        if polygon.len() > 2 {
            let rand_offset = _c(options.max_randomness_offset.unwrap_or(0.0));
            polygon.iter().enumerate().for_each(|(ind, point)| {
                if ind == 0 {
                    ops.push(Op {
                        op: OpType::Move,
                        data: vec![
                            point.x + _offset_opt(rand_offset, options, None),
                            point.y + _offset_opt(rand_offset, options, None),
                        ],
                    });
                } else {
                    ops.push(Op {
                        op: OpType::LineTo,
                        data: vec![
                            point.x + _offset_opt(rand_offset, options, None),
                            point.y + _offset_opt(rand_offset, options, None),
                        ],
                    });
                }
            })
        }
    }
    return OpSet {
        op_set_type: OpSetType::FillPath,
        ops,
        size: None,
        path: None,
    };
}

pub fn rand_offset<F: Float + Trig + FromPrimitive>(x: F, o: &mut Options) -> F {
    _offset_opt(x, o, None)
}

pub fn rand_offset_with_range<F: Float + Trig + FromPrimitive>(
    min: F,
    max: F,
    o: &mut Options,
) -> F {
    _offset(min, max, o, None)
}

pub fn double_line_fill_ops<F: Float + Trig + FromPrimitive>(
    x1: F,
    y1: F,
    x2: F,
    y2: F,
    o: &mut Options,
) -> Vec<Op<F>> {
    _double_line(x1, y1, x2, y2, o, true)
}

pub fn clone_options_alter_seed(ops: &mut Options) -> Options {
    let mut result: Options = ops.clone();
    if let Some(seed) = ops.seed {
        result.seed = Some(seed + 1);
    }
    result
}

fn _offset<F: Float + Trig + FromPrimitive>(
    min: F,
    max: F,
    ops: &mut Options,
    roughness_gain: Option<F>,
) -> F {
    let rg: F = roughness_gain.unwrap_or(_c(1.0));
    _c::<F>(ops.roughness.unwrap_or(1.0))
        * rg
        * ((_c::<F>(ops.random() as f32) * (max - min)) + min)
}

fn _offset_opt<F: Float + Trig + FromPrimitive>(
    x: F,
    ops: &mut Options,
    roughness_gain: Option<F>,
) -> F {
    _offset(-x, x, ops, roughness_gain)
}

fn _line<F: Float + Trig + FromPrimitive>(
    x1: F,
    y1: F,
    x2: F,
    y2: F,
    o: &mut Options,
    mover: bool,
    overlay: bool,
) -> Vec<Op<F>> {
    // println!(
    //     "Drawing line {},{} to {},{}",
    //     x1.to_f32().unwrap(),
    //     y1.to_f32().unwrap(),
    //     x2.to_f32().unwrap(),
    //     y2.to_f32().unwrap()
    // );
    let length_sq = (x1 - x2).powi(2) + (y1 - y2).powi(2);
    let length = length_sq.sqrt();
    let mut roughness_gain = _c(1.0);
    if length < _c(200.0_f32) {
        roughness_gain = _c(1.0);
    } else if length > _c(500.0) {
        roughness_gain = _c(0.4);
    } else {
        roughness_gain = _c::<F>(-0.0016668) * length + _c(1.233334);
    }

    let mut offset = _c(o.max_randomness_offset.unwrap_or(1.0) as f32);
    if (offset * offset * _c(100.0)) > length_sq {
        offset = length / _c(10.0);
    }
    let half_offset = offset / _c(2.0);
    let diverge_point = _c::<F>(0.2) + _c::<F>(o.random() as f32) * _c(0.2);
    let mut mid_disp_x = _c::<F>(o.bowing.unwrap_or(1.0) as f32)
        * _c(o.max_randomness_offset.unwrap_or(1.0) as f32)
        * (y2 - y1)
        / _c(200.0);
    let mut mid_disp_y = _c::<F>(o.bowing.unwrap_or(1.0) as f32)
        * _c(o.max_randomness_offset.unwrap_or(1.0) as f32)
        * (x1 - x2)
        / _c(200.0);
    mid_disp_x = _offset_opt(mid_disp_x, o, Some(roughness_gain));
    mid_disp_y = _offset_opt(mid_disp_y, o, Some(roughness_gain));
    let mut ops: Vec<Op<F>> = Vec::new();

    let preserve_vertices = o.preserve_vertices.unwrap_or(false);
    if mover {
        if overlay {
            ops.push(Op {
                op: OpType::Move,
                data: vec![
                    x1 + if preserve_vertices {
                        _c(0.0)
                    } else {
                        _offset_opt(half_offset, o, Some(roughness_gain))
                    },
                    y1 + if preserve_vertices {
                        _c(0.0)
                    } else {
                        _offset_opt(half_offset, o, Some(roughness_gain))
                    },
                ],
            });
        } else {
            ops.push(Op {
                op: OpType::Move,
                data: vec![
                    x1 + if preserve_vertices {
                        _c(0.0)
                    } else {
                        _offset_opt(offset, o, Some(roughness_gain))
                    },
                    y1 + if preserve_vertices {
                        _c(0.0)
                    } else {
                        _offset_opt(offset, o, Some(roughness_gain))
                    },
                ],
            });
        }
    }
    if overlay {
        ops.push(Op {
            op: OpType::BCurveTo,
            data: vec![
                mid_disp_x
                    + x1
                    + (x2 - x1) * diverge_point
                    + _offset_opt(half_offset, o, Some(roughness_gain)),
                mid_disp_y
                    + y1
                    + (y2 - y1) * diverge_point
                    + _offset_opt(half_offset, o, Some(roughness_gain)),
                mid_disp_x
                    + x1
                    + _c::<F>(2.0) * (x2 - x1) * diverge_point
                    + _offset_opt(half_offset, o, Some(roughness_gain)),
                mid_disp_y
                    + y1
                    + _c::<F>(2.0) * (y2 - y1) * diverge_point
                    + _offset_opt(half_offset, o, Some(roughness_gain)),
                x2 + if preserve_vertices {
                    _c(0.0)
                } else {
                    _offset_opt(half_offset, o, Some(roughness_gain))
                },
                y2 + if preserve_vertices {
                    _c(0.0)
                } else {
                    _offset_opt(half_offset, o, Some(roughness_gain))
                },
            ],
        });
    } else {
        ops.push(Op {
            op: OpType::BCurveTo,
            data: vec![
                mid_disp_x
                    + x1
                    + (x2 - x1) * diverge_point
                    + _offset_opt(offset, o, Some(roughness_gain)),
                mid_disp_y
                    + y1
                    + (y2 - y1) * diverge_point
                    + _offset_opt(offset, o, Some(roughness_gain)),
                mid_disp_x
                    + x1
                    + _c::<F>(2.0) * (x2 - x1) * diverge_point
                    + _offset_opt(offset, o, Some(roughness_gain)),
                mid_disp_y
                    + y1
                    + _c::<F>(2.0) * (y2 - y1) * diverge_point
                    + _offset_opt(offset, o, Some(roughness_gain)),
                x2 + if preserve_vertices {
                    _c(0.0)
                } else {
                    _offset_opt(offset, o, Some(roughness_gain))
                },
                y2 + if preserve_vertices {
                    _c(0.0)
                } else {
                    _offset_opt(offset, o, Some(roughness_gain))
                },
            ],
        });
    }
    // for op in ops.iter() {
    //     for data in op.data.iter(){
    //         print!("data point {} ", data.to_f32().unwrap());
    //     }
    //     println!("");
    // }
    ops
}

pub(crate) fn _double_line<F: Float + Trig + FromPrimitive>(
    x1: F,
    y1: F,
    x2: F,
    y2: F,
    o: &mut Options,
    filling: bool,
) -> Vec<Op<F>> {
    let single_stroke = if filling {
        o.disable_multi_stroke_fill.unwrap_or(false)
    } else {
        o.disable_multi_stroke.unwrap_or(false)
    };
    let mut o1 = _line(x1, y1, x2, y2, o, true, false);
    if single_stroke {
        o1
    } else {
        let mut o2 = _line(x1, y1, x2, y2, o, true, true);
        o1.append(&mut o2);
        o1
    }
}

fn _curve<F: Float + Trig + FromPrimitive>(
    points: &[Point2D<F>],
    close_point: Option<Point2D<F>>,
    o: &mut Options,
) -> Vec<Op<F>> {
    let len = points.len();
    let mut ops: Vec<Op<F>> = vec![];
    if len > 3 {
        let mut b: [[F; 2]; 4] = [[_c(0.0); 2]; 4];
        let s: F = _c::<F>(1.0) - _c(o.curve_tightness.unwrap_or(0.0));
        ops.push(Op {
            op: OpType::Move,
            data: vec![points[1].x, points[1].y],
        });
        let mut i = 1;
        while (i + 2) < len {
            let cached_vert_array = points[i];
            b[0] = [cached_vert_array.x, cached_vert_array.y];
            b[1] = [
                cached_vert_array.x + (s * points[i + 1].x - s * points[i - 1].x) / _c(6.0),
                cached_vert_array.y + (s * points[i + 1].y - s * points[i - 1].y) / _c(6.0),
            ];
            b[2] = [
                points[i + 1].x + (s * points[i].x - s * points[i + 2].x) / _c(6.0),
                points[i + 1].y + (s * points[i].y - s * points[i + 2].y) / _c(6.0),
            ];
            b[3] = [points[i + 1].x, points[i + 1].x];
            ops.push(Op {
                op: OpType::BCurveTo,
                data: vec![b[1][0], b[1][1], b[2][0], b[2][1], b[3][0], b[3][1]],
            });
            i += 1;
        }
        if let Some(cp) = close_point {
            let ro = _c(o.max_randomness_offset.unwrap_or(0.0));
            ops.push(Op {
                op: OpType::LineTo,
                data: vec![
                    cp.x + _offset_opt(ro, o, None),
                    cp.y + _offset_opt(ro, o, None),
                ],
            });
        }
    } else if len == 3 {
        ops.push(Op {
            op: OpType::Move,
            data: vec![points[1].x, points[1].y],
        });
        ops.push(Op {
            op: OpType::BCurveTo,
            data: vec![
                points[1].x,
                points[1].y,
                points[2].x,
                points[2].y,
                points[2].x,
                points[2].y,
            ],
        });
    } else if len == 2 {
        ops.append(&mut _double_line(
            points[0].x,
            points[0].y,
            points[1].x,
            points[1].y,
            o,
            false,
        ));
    }
    ops
}

fn _curve_with_offset<F: Float + Trig + FromPrimitive>(
    points: &[Point2D<F>],
    offset: F,
    o: &mut Options,
) -> Vec<Op<F>> {
    let mut ps: Vec<Point2D<F>> = vec![
        Point2D::new(
            points[0].x + _offset_opt(offset, o, None),
            points[0].y + _offset_opt(offset, o, None),
        ),
        Point2D::new(
            points[0].x + _offset_opt(offset, o, None),
            points[0].y + _offset_opt(offset, o, None),
        ),
    ];
    let mut i = 1;
    while i < points.len() {
        ps.push(Point2D::new(
            points[i].x + _offset_opt(offset, o, None),
            points[i].y + _offset_opt(offset, o, None),
        ));
        if i == (points.len() - 1) {
            ps.push(Point2D::new(
                points[i].x + _offset_opt(offset, o, None),
                points[i].y + _offset_opt(offset, o, None),
            ));
        }
        i += 1;
    }
    _curve(&ps, None, o)
}

fn _compute_ellipse_points<F: Float + Trig + FromPrimitive>(
    increment: F,
    cx: F,
    cy: F,
    rx: F,
    ry: F,
    offset: F,
    overlap: F,
    o: &mut Options,
) -> Vec<Vec<Point2D<F>>> {
    let core_only = o.roughness.unwrap_or(0.0) == 0.0;
    let mut core_points: Vec<Point2D<F>> = Vec::new();
    let mut all_points: Vec<Point2D<F>> = Vec::new();

    if core_only {
        let increment_inner = increment / _c(4.0);
        all_points.push(Point2D::new(
            cx + rx * Float::cos(-increment_inner),
            cy + ry * Float::sin(-increment_inner),
        ));

        let mut angle = _c(0.0);
        while angle <= _c(f32::PI() * 2.0) {
            let p = Point2D::new(cx + rx * Float::cos(angle), cy + ry * Float::sin(angle));
            core_points.push(p);
            all_points.push(p);
            angle = angle + increment_inner;
        }
        all_points.push(Point2D::new(
            cx + rx * Float::cos(_c(0.0)),
            cy + ry * Float::sin(_c(0.0)),
        ));
        all_points.push(Point2D::new(
            cx + rx * Float::cos(increment_inner),
            cy + ry * Float::sin(increment_inner),
        ));
    } else {
        let rad_offset: F = _offset_opt::<F>(_c(0.5), o, None) - (_c::<F>(f32::PI()) / _c(2.0));
        all_points.push(Point2D::new(
            _offset_opt(offset, o, None)
                + cx
                + _c::<F>(0.9) * rx * Float::cos(rad_offset - increment),
            _offset_opt(offset, o, None)
                + cy
                + _c::<F>(0.9) * ry * Float::sin(rad_offset - increment),
        ));
        let end_angle = _c::<F>(f32::PI()) * _c(2.0) + rad_offset - _c(0.01);
        let mut angle = rad_offset;
        while angle < end_angle {
            let p = Point2D::new(
                _offset_opt(offset, o, None) + cx + rx * Float::cos(angle),
                _offset_opt(offset, o, None) + cy + ry * Float::sin(angle),
            );
            core_points.push(p);
            all_points.push(p);
            angle = angle + increment;
        }

        all_points.push(Point2D::new(
            _offset_opt(offset, o, None)
                + cx
                + rx * Float::cos(rad_offset + _c::<F>(f32::PI()) * _c(2.0) + overlap * _c(0.5)),
            _offset_opt(offset, o, None)
                + cy
                + ry * Float::sin(rad_offset + _c::<F>(f32::PI()) * _c(2.0) + overlap * _c(0.5)),
        ));
        all_points.push(Point2D::new(
            _offset_opt(offset, o, None)
                + cx
                + _c::<F>(0.98) * rx * Float::cos(rad_offset + overlap),
            _offset_opt(offset, o, None)
                + cy
                + _c::<F>(0.98) * ry * Float::sin(rad_offset + overlap),
        ));
        all_points.push(Point2D::new(
            _offset_opt(offset, o, None)
                + cx
                + _c::<F>(0.9) * rx * Float::cos(rad_offset + overlap * _c(0.5)),
            _offset_opt(offset, o, None)
                + cy
                + _c::<F>(0.9) * ry * Float::sin(rad_offset + overlap * _c(0.5)),
        ));
    }
    return vec![all_points, core_points];
}

fn _arc<F: Float + Trig + FromPrimitive>(
    increment: F,
    cx: F,
    cy: F,
    rx: F,
    ry: F,
    strt: F,
    stp: F,
    offset: F,
    o: &mut Options,
) -> Vec<Op<F>> {
    let rad_offset = strt + _offset_opt(_c(0.1), o, None);
    let mut points: Vec<Point2D<F>> = vec![Point2D::new(
        _offset_opt(offset, o, None) + cx + _c::<F>(0.9) * rx * Float::cos(rad_offset - increment),
        _offset_opt(offset, o, None) + cy + _c::<F>(0.9) * ry * Float::sin(rad_offset - increment),
    )];
    let mut angle = rad_offset;
    while angle <= stp {
        points.push(Point2D::new(
            _offset_opt(offset, o, None) + cx + rx * Float::cos(angle),
            _offset_opt(offset, o, None) + cy + ry * Float::sin(angle),
        ));
        angle = angle + increment;
    }
    points.push(Point2D::new(
        cx + rx * Float::cos(stp),
        cy + ry * Float::sin(stp),
    ));
    points.push(Point2D::new(
        cx + rx * Float::cos(stp),
        cy + ry * Float::sin(stp),
    ));
    _curve(&points, None, o)
}

fn _bezier_to<F: Float + Trig + FromPrimitive>(
    x1: F,
    y1: F,
    x2: F,
    y2: F,
    x: F,
    y: F,
    current: &Point2D<F>,
    o: &mut Options,
) -> Vec<Op<F>> {
    let mut ops: Vec<Op<F>> = Vec::new();
    let ros = [
        _c(o.max_randomness_offset.unwrap_or(1.0)),
        _c(o.max_randomness_offset.unwrap_or(1.0) + 0.3),
    ];
    let mut f: Point2D<F> = Point2D::new(_c(0.0), _c(0.0));
    let iterations = if o.disable_multi_stroke.unwrap_or(false) {
        1
    } else {
        2
    };
    let preserve_vertices = o.preserve_vertices.unwrap_or(false);
    let mut i = 0;
    while i < iterations {
        if i == 0 {
            ops.push(Op {
                op: OpType::Move,
                data: vec![current.x, current.y],
            });
        } else {
            ops.push(Op {
                op: OpType::Move,
                data: vec![
                    current.x
                        + (if preserve_vertices {
                            _c(0.0)
                        } else {
                            _offset_opt(ros[0], o, None)
                        }),
                    current.y
                        + (if preserve_vertices {
                            _c(0.0)
                        } else {
                            _offset_opt(ros[0], o, None)
                        }),
                ],
            });
        }
        f = if preserve_vertices {
            Point2D::new(x, y)
        } else {
            Point2D::new(
                x + _offset_opt(ros[i], o, None),
                y + _offset_opt(ros[i], o, None),
            )
        };
        ops.push(Op {
            op: OpType::BCurveTo,
            data: vec![
                x1 + _offset_opt(ros[i], o, None),
                y1 + _offset_opt(ros[i], o, None),
                x2 + _offset_opt(ros[i], o, None),
                y2 + _offset_opt(ros[i], o, None),
                f.x,
                f.y,
            ],
        });
        i += 1;
    }
    ops
}
