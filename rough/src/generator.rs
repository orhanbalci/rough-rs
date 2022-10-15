use std::fmt::{Display, Write};
use std::ops::MulAssign;

use euclid::default::Point2D;
use euclid::Trig;
use num_traits::{Float, FromPrimitive};
use palette::Srgb;
use points_on_curve::{curve_to_bezier, points_on_bezier_curves};

use crate::core::{
    Drawable,
    FillStyle,
    OpSet,
    OpSetType,
    OpType,
    Options,
    OptionsBuilder,
    PathInfo,
    _c,
};
use crate::points_on_path::points_on_path;
use crate::renderer::{
    curve,
    ellipse_with_params,
    generate_ellipse_params,
    line,
    linear_path,
    pattern_fill_arc,
    pattern_fill_polygons,
    rectangle,
    solid_fill_polygon,
    svg_path,
};

pub struct Generator {
    default_options: Options,
}

impl Default for Generator {
    fn default() -> Self {
        Self {
            default_options: OptionsBuilder::default()
                .max_randomness_offset(Some(2.0))
                .roughness(Some(1.0))
                .bowing(Some(2.0))
                .stroke(Some(Srgb::new(0.0, 0.0, 0.0)))
                .stroke_width(Some(1.0))
                .curve_tightness(Some(0.0))
                .curve_fitting(Some(0.95))
                .curve_step_count(Some(9.0))
                .fill(Some(Srgb::new(0.0, 0.0, 0.0)))
                .fill_style(Some(FillStyle::Hachure))
                .fill_weight(Some(-1.0))
                .hachure_angle(Some(-41.0))
                .hachure_gap(Some(-1.0))
                .dash_offset(Some(-1.0))
                .dash_gap(Some(-1.0))
                .zigzag_offset(Some(-1.0))
                .seed(Some(345_u64))
                .disable_multi_stroke(Some(false))
                .disable_multi_stroke_fill(Some(false))
                .preserve_vertices(Some(false))
                .simplification(Some(1.0))
                .stroke_line_dash(None)
                .stroke_line_dash_offset(None)
                .fill_line_dash(None)
                .fill_line_dash_offset(None)
                .fixed_decimal_place_digits(None)
                .randomizer(None)
                .build()
                .expect("failed to build default options"),
        }
    }
}

impl Generator {
    fn d<T, F>(&self, name: T, op_sets: &[OpSet<F>]) -> Drawable<F>
    where
        T: Into<String>,
        F: Float + Trig + FromPrimitive,
    {
        Drawable {
            shape: name.into(),
            options: self.default_options.clone(),
            sets: Vec::from_iter(op_sets.iter().cloned()),
        }
    }

    pub fn line<F>(&self, x1: F, y1: F, x2: F, y2: F) -> Drawable<F>
    where
        F: Float + Trig + FromPrimitive,
    {
        self.d(
            "line",
            &[line(x1, y1, x2, y2, &mut self.default_options.clone())],
        )
    }

    pub fn rectangle<F>(&self, x: F, y: F, width: F, height: F) -> Drawable<F>
    where
        F: Float + Trig + FromPrimitive,
    {
        let mut paths = vec![];
        let mut options = self.default_options.clone();
        let outline = rectangle(x, y, width, height, &mut options);
        if options.fill.is_some() {
            let points = vec![
                Point2D::new(x, y),
                Point2D::new(x + width, y),
                Point2D::new(x + width, y + height),
                Point2D::new(x, y + height),
            ];
            if options.fill_style == Some(FillStyle::Solid) {
                paths.push(solid_fill_polygon(&vec![points], &mut options));
            } else {
                paths.push(pattern_fill_polygons(vec![points], &mut options));
            }
        }
        if options.stroke.is_some() {
            paths.push(outline);
        }

        self.d("rectangle", &paths)
    }

    pub fn ellipse<F>(&self, x: F, y: F, width: F, height: F) -> Drawable<F>
    where
        F: Float + Trig + FromPrimitive,
    {
        let mut paths = vec![];
        let mut options = self.default_options.clone();
        let ellipse_params = generate_ellipse_params(width, height, &mut options);
        let ellipse_response = ellipse_with_params(x, y, &mut options, &ellipse_params);
        if options.fill.is_some() {
            if options.fill_style == Some(FillStyle::Solid) {
                let mut shape = ellipse_with_params(x, y, &mut options, &ellipse_params).opset;
                shape.op_set_type = OpSetType::FillPath;
                paths.push(shape);
            } else {
                paths.push(pattern_fill_polygons(
                    vec![ellipse_response.estimated_points],
                    &mut options,
                ));
            }
        }
        if options.stroke.is_some() {
            paths.push(ellipse_response.opset);
        }
        self.d("ellipse", &paths)
    }

    pub fn circle<F>(&self, x: F, y: F, diameter: F) -> Drawable<F>
    where
        F: Float + Trig + FromPrimitive,
    {
        let mut shape = self.ellipse(x, y, diameter, diameter);
        shape.shape = "circle".into();
        shape
    }

    pub fn linear_path<F>(&self, points: &[Point2D<F>]) -> Drawable<F>
    where
        F: Float + Trig + FromPrimitive,
    {
        let mut options = self.default_options.clone();
        self.d("linear_path", &[linear_path(points, false, &mut options)])
    }

    pub fn arc<F>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
        start: F,
        stop: F,
        closed: bool,
    ) -> Drawable<F>
    where
        F: Float + Trig + FromPrimitive,
    {
        let mut options = self.default_options.clone();
        let mut paths = vec![];
        let outline =
            crate::renderer::arc(x, y, width, height, start, stop, closed, true, &mut options);
        if closed && options.fill.is_some() {
            if options.fill_style == Some(FillStyle::Solid) {
                options.disable_multi_stroke = Some(true);
                let mut shape = crate::renderer::arc(
                    x,
                    y,
                    width,
                    height,
                    start,
                    stop,
                    true,
                    false,
                    &mut options,
                );
                shape.op_set_type = OpSetType::FillPath;
                paths.push(shape);
            } else {
                paths.push(pattern_fill_arc(
                    x,
                    y,
                    width,
                    height,
                    start,
                    stop,
                    &mut options,
                ));
            }
        }
        if options.stroke.is_some() {
            paths.push(outline);
        }
        self.d("arc", &paths)
    }

    pub fn curve<F>(&self, points: &[Point2D<F>]) -> Drawable<F>
    where
        F: Float + Trig + FromPrimitive + MulAssign + Display,
    {
        let mut paths = vec![];
        let mut options = self.default_options.clone();
        let outline = curve(points, &mut options);
        if options.fill.is_some() && points.len() >= 3 {
            let curve = curve_to_bezier(points, _c(0.0));
            if let Some(crv) = curve {
                let poly_points = points_on_bezier_curves(
                    &crv,
                    _c(10.0),
                    Some(_c::<F>(1.0) + _c::<F>(options.roughness.unwrap_or(0.0)) / _c(2.0)),
                );
                if options.fill_style == Some(FillStyle::Solid) {
                    paths.push(solid_fill_polygon(&vec![poly_points], &mut options));
                } else {
                    paths.push(pattern_fill_polygons(&mut vec![poly_points], &mut options));
                }
            }
        }

        if options.stroke.is_some() {
            paths.push(outline);
        }

        self.d("curve", &paths)
    }

    pub fn polygon<F>(&self, points: &[Point2D<F>]) -> Drawable<F>
    where
        F: Float + Trig + FromPrimitive + MulAssign + Display,
    {
        let mut options = self.default_options.clone();
        let mut paths = vec![];
        let outline = linear_path(points, true, &mut options);
        if options.fill.is_some() {
            if options.fill_style == Some(FillStyle::Solid) {
                paths.push(solid_fill_polygon(&vec![points.to_vec()], &mut options));
            } else {
                paths.push(pattern_fill_polygons(
                    &mut vec![points.to_vec()],
                    &mut options,
                ));
            }
        }
        if options.stroke.is_some() {
            paths.push(outline);
        }
        self.d("polygon", &paths)
    }

    pub fn path<F>(&self, d: String) -> Drawable<F>
    where
        F: Float + Trig + FromPrimitive + MulAssign + Display,
    {
        let mut options = self.default_options.clone();
        let mut paths = vec![];
        if d.is_empty() {
            self.d("path", &paths)
        } else {
            let simplified = options.simplification.map(|a| a < 1.0).unwrap_or(false);
            let distance = if simplified {
                _c::<F>(4.0) - _c::<F>(4.0) * _c::<F>(options.simplification.unwrap())
            } else {
                (_c::<F>(1.0) + _c::<F>(options.roughness.unwrap_or(1.0))) / _c::<F>(2.0)
            };

            let sets = points_on_path(d.clone(), Some(_c(1.0)), Some(distance));
            if options.fill.is_some() {
                if options.fill_style == Some(FillStyle::Solid) {
                    paths.push(solid_fill_polygon(&sets, &mut options));
                } else {
                    paths.push(pattern_fill_polygons(sets.clone(), &mut options));
                }
            }

            if options.stroke.is_some() {
                if simplified {
                    sets.iter()
                        .for_each(|s| paths.push(linear_path(s, false, &mut options)));
                } else {
                    paths.push(svg_path(d, &mut options));
                }
            }

            self.d("path", &paths)
        }
    }

    pub fn ops_to_path<F>(mut drawing: OpSet<F>, fixed_decimals: Option<u32>) -> String
    where
        F: Float + FromPrimitive + Trig + Display,
    {
        let mut path = String::new();

        for item in drawing.ops.iter_mut() {
            if let Some(fd) = fixed_decimals {
                let pow: u32 = 10u32.pow(fd);
                item.data.iter_mut().for_each(|p| {
                    *p = (*p * F::from(pow).unwrap()).round() / F::from(pow).unwrap();
                });
            }

            match item.op {
                OpType::Move => {
                    write!(&mut path, "L{} {} ", item.data[0], item.data[1])
                        .expect("Failed to write path string");
                }
                OpType::BCurveTo => {
                    write!(
                        &mut path,
                        "C{} {}, {} {}, {} {} ",
                        item.data[0],
                        item.data[1],
                        item.data[2],
                        item.data[3],
                        item.data[4],
                        item.data[5]
                    )
                    .expect("Failed to write path string");
                }
                OpType::LineTo => {
                    write!(&mut path, "L{} {}, ", item.data[0], item.data[1])
                        .expect("Failed to write path string");
                }
            }
        }

        path
    }

    pub fn to_paths<F>(drawable: Drawable<F>) -> Vec<PathInfo>
    where
        F: Float + FromPrimitive + Trig + Display,
    {
        let sets = drawable.sets;
        let o = drawable.options;
        let mut path_infos = vec![];
        for drawing in sets.iter() {
            let path_info = match drawing.op_set_type {
                OpSetType::Path => PathInfo {
                    d: Self::ops_to_path(drawing.clone(), None),
                    stroke: o.stroke,
                    stroke_width: o.stroke_width,
                    fill: None,
                },

                OpSetType::FillPath => PathInfo {
                    d: Self::ops_to_path(drawing.clone(), None),
                    stroke: None,
                    stroke_width: Some(0.0f32),
                    fill: o.fill,
                },
                OpSetType::FillSketch => {
                    let fill_weight = if o.fill_weight.unwrap_or(0.0) < 0.0 {
                        o.stroke_width.unwrap_or(0.0) / 2.0
                    } else {
                        o.fill_weight.unwrap_or(0.0)
                    };
                    PathInfo {
                        d: Self::ops_to_path(drawing.clone(), None),
                        stroke: o.fill,
                        stroke_width: Some(fill_weight),
                        fill: None,
                    }
                }
            };
            path_infos.push(path_info);
        }
        path_infos
    }
}
