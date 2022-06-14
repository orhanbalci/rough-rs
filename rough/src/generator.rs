use euclid::default::Point2D;
use euclid::Trig;
use num_traits::Float;
use num_traits::FromPrimitive;

use crate::core::Drawable;
use crate::core::OpSet;
use crate::renderer::rectangle;
use crate::renderer::solid_fill_polygon;

use super::core::Options;
use super::core::OptionsBuilder;

use super::renderer::line;

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
                .fill(None)
                .fill_style(Some("hachure".to_owned()))
                .fill_weight(Some(-1.0))
                .hachure_angle(Some(-41.0))
                .hachure_gap(Some(-1.0))
                .dash_offset(Some(-1.0))
                .dash_gap(Some(-1.0))
                .zigzag_offset(Some(-1.0))
                .seed(Some(0_u64))
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
    fn d<T, F>(&self, name: T, op_sets: Vec<OpSet<F>>) -> Drawable<F>
    where
        T: Into<String>,
        F: Float + Trig + FromPrimitive,
    {
        Drawable {
            shape: name.into(),
            options: self.default_options.clone(),
            sets: op_sets,
        }
    }

    pub fn line<F>(&self, x1: F, y1: F, x2: F, y2: F) -> Drawable<F>
    where
        F: Float + Trig + FromPrimitive,
    {
        self.d(
            "line",
            vec![line(x1, y1, x2, y2, &mut self.default_options.clone())],
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
            if options.fill_style == Some("solid".into()) {
                paths.push(solid_fill_polygon(&vec![points], &mut options));
            } else {
                //paths.push(patternFillPolygons([points], o));
                todo!("pattern_fill_polygons is not implemented yet");
            }
        }
        if options.stroke.is_some() {
            paths.push(outline);
        }

        self.d("rectangle", paths)
    }
}
