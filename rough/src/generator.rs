use std::fmt::Display;
use std::ops::MulAssign;

use euclid::default::Point2D;
use euclid::Trig;
use num_traits::{Float, FromPrimitive};
use points_on_curve::{curve_to_bezier, points_on_bezier_curves};

use super::core::{Options, OptionsBuilder};
use super::renderer::line;
use crate::core::{Drawable, FillStyle, OpSet, OpSetType, _c};
use crate::renderer::{
    curve,
    ellipse_with_params,
    generate_ellipse_params,
    linear_path,
    pattern_fill_arc,
    pattern_fill_polygons,
    rectangle,
    solid_fill_polygon,
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
                .stroke(Some("#000".to_owned()))
                .stroke_width(Some(1.0))
                .curve_tightness(Some(0.0))
                .curve_fitting(Some(0.95))
                .curve_step_count(Some(9.0))
                .fill(true)
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
        if options.fill {
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
        if options.fill {
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
        if closed && options.fill {
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
        if options.fill && points.len() >= 3 {
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

    // TODO add polygon function
    // TODO add path function
    // TODO add ops_to_path function
    // TODO add to_paths function
    // TODO add fill_sketch function
    // TODO path function needs points_on_path (dependencies are implemented)
}
