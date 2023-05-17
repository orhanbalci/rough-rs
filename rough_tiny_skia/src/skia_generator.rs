use std::fmt::Display;
use std::ops::MulAssign;

use euclid::default::Point2D;
use euclid::Trig;
use num_traits::{Float, FromPrimitive};
use palette::Srgba;
use roughr::core::{Drawable, OpSet, OpSetType, OpType, Options};
use roughr::generator::Generator;
use tiny_skia::{
    FillRule,
    LineCap,
    Paint,
    Path,
    PathBuilder,
    PixmapMut,
    Stroke,
    StrokeDash,
    Transform,
};

#[derive(Default)]
pub struct SkiaGenerator {
    gen: Generator,
    options: Option<Options>,
}

#[derive(Clone)]
pub struct SkiaOpset<F: Float + Trig> {
    pub op_set_type: OpSetType,
    pub ops: Path,
    pub size: Option<Point2D<F>>,
    pub path: Option<String>,
}

pub trait ToSkiaOpset<F: Float + Trig> {
    fn to_skia_opset(self) -> SkiaOpset<F>;
}

impl<F: Float + Trig + FromPrimitive> ToSkiaOpset<F> for OpSet<F> {
    fn to_skia_opset(self) -> SkiaOpset<F> {
        SkiaOpset {
            op_set_type: self.op_set_type.clone(),
            size: self.size,
            path: self.path.clone(),
            ops: opset_to_shape(&self),
        }
    }
}

pub struct SkiaDrawable<F: Float + Trig> {
    pub shape: String,
    pub options: Options,
    pub sets: Vec<SkiaOpset<F>>,
}

pub trait ToSkiaDrawable<F: Float + Trig> {
    fn to_skia_drawable(self) -> SkiaDrawable<F>;
}

impl<F: Float + Trig + FromPrimitive> ToSkiaDrawable<F> for Drawable<F> {
    fn to_skia_drawable(self) -> SkiaDrawable<F> {
        SkiaDrawable {
            shape: self.shape,
            options: self.options,
            sets: self.sets.into_iter().map(|s| s.to_skia_opset()).collect(),
        }
    }
}

impl SkiaGenerator {
    pub fn new(options: Options) -> Self {
        SkiaGenerator { gen: Generator::default(), options: Some(options) }
    }
}

impl<F: Float + Trig> SkiaDrawable<F> {
    pub fn draw(&self, ctx: &mut PixmapMut) {
        for set in self.sets.iter() {
            match set.op_set_type {
                OpSetType::Path => {
                    if self.options.stroke_line_dash.is_some() {
                        let mut stroke = Stroke {
                            width: self.options.stroke_width.unwrap_or(1.0),
                            line_cap: LineCap::Round,
                            ..Stroke::default()
                        };
                        let stroke_line_dash = self
                            .options
                            .stroke_line_dash
                            .clone()
                            .unwrap_or(Vec::new())
                            .iter()
                            .map(|&a| a as f32)
                            .collect();

                        stroke.dash = StrokeDash::new(
                            stroke_line_dash,
                            self.options.stroke_line_dash_offset.unwrap_or(1.0f64) as f32,
                        );

                        let stroke_color = self
                            .options
                            .stroke
                            .unwrap_or_else(|| Srgba::from_components((1.0, 1.0, 1.0, 1.0)));
                        let stroke_color_components: (u8, u8, u8, u8) =
                            stroke_color.into_format().into_components();

                        let mut paint = Paint::default();
                        paint.set_color_rgba8(
                            stroke_color_components.0,
                            stroke_color_components.1,
                            stroke_color_components.2,
                            stroke_color_components.3,
                        );
                        paint.anti_alias = true;

                        ctx.stroke_path(&set.ops, &paint, &stroke, Transform::identity(), None);
                    } else {
                        let mut stroke = Stroke::default();
                        stroke.width = self.options.stroke_width.unwrap_or(1.0);

                        let stroke_color = self
                            .options
                            .stroke
                            .unwrap_or_else(|| Srgba::from_components((1.0, 1.0, 1.0, 1.0)));
                        let stroke_color_components: (u8, u8, u8, u8) =
                            stroke_color.into_format().into_components();

                        let mut paint = Paint::default();
                        paint.set_color_rgba8(
                            stroke_color_components.0,
                            stroke_color_components.1,
                            stroke_color_components.2,
                            stroke_color_components.3,
                        );
                        paint.anti_alias = true;

                        ctx.stroke_path(&set.ops, &paint, &stroke, Transform::identity(), None);
                    }
                }
                OpSetType::FillPath => {
                    let fill_color = self
                        .options
                        .fill
                        .unwrap_or(Srgba::from_components((1.0, 1.0, 1.0, 1.0)));
                    let fill_color_components: (u8, u8, u8, u8) =
                        fill_color.into_format().into_components();

                    let mut paint = Paint::default();
                    paint.set_color_rgba8(
                        fill_color_components.0,
                        fill_color_components.1,
                        fill_color_components.2,
                        fill_color_components.3,
                    );
                    paint.anti_alias = true;
                    match self.shape.as_str() {
                        "curve" | "polygon" | "path" => {
                            ctx.fill_path(
                                &set.ops,
                                &paint,
                                FillRule::EvenOdd,
                                Transform::identity(),
                                None,
                            );
                        }
                        _ => {
                            ctx.fill_path(
                                &set.ops,
                                &paint,
                                FillRule::Winding,
                                Transform::identity(),
                                None,
                            );
                        }
                    }
                }
                OpSetType::FillSketch => {
                    let mut fweight = self.options.fill_weight.unwrap_or_default();
                    if fweight < 0.0 {
                        fweight = self.options.stroke_width.unwrap_or(1.0) / 2.0;
                    }

                    if self.options.fill_line_dash.is_some() {
                        let mut stroke = Stroke::default();
                        stroke.width = self.options.fill_weight.unwrap_or(1.0);
                        stroke.line_cap = LineCap::Round;
                        let fill_line_dash = self
                            .options
                            .fill_line_dash
                            .clone()
                            .unwrap_or(Vec::new())
                            .iter()
                            .map(|&a| a as f32)
                            .collect();

                        stroke.dash = StrokeDash::new(
                            fill_line_dash,
                            self.options.fill_line_dash_offset.unwrap_or(1.0f64) as f32,
                        );

                        let fill_color = self
                            .options
                            .fill
                            .unwrap_or(Srgba::from_components((1.0, 1.0, 1.0, 1.0)));
                        let fill_color_components: (u8, u8, u8, u8) =
                            fill_color.into_format().into_components();

                        let mut paint = Paint::default();
                        paint.set_color_rgba8(
                            fill_color_components.0,
                            fill_color_components.1,
                            fill_color_components.2,
                            fill_color_components.3,
                        );
                        paint.anti_alias = true;
                        ctx.stroke_path(&set.ops, &paint, &stroke, Transform::identity(), None);
                    } else {
                        let mut stroke = Stroke::default();
                        stroke.width = self.options.fill_weight.unwrap_or(1.0);
                        stroke.line_cap = LineCap::Round;

                        let fill_color = self
                            .options
                            .fill
                            .unwrap_or(Srgba::from_components((1.0, 1.0, 1.0, 1.0)));
                        let fill_color_components: (u8, u8, u8, u8) =
                            fill_color.into_format().into_components();

                        let mut paint = Paint::default();
                        paint.set_color_rgba8(
                            fill_color_components.0,
                            fill_color_components.1,
                            fill_color_components.2,
                            fill_color_components.3,
                        );
                        paint.anti_alias = true;
                        ctx.stroke_path(&set.ops, &paint, &stroke, Transform::identity(), None);
                    }
                }
            }
        }
    }
}

fn opset_to_shape<F: Trig + Float + FromPrimitive>(op_set: &OpSet<F>) -> Path {
    let mut path: PathBuilder = PathBuilder::new();
    for item in op_set.ops.iter() {
        match item.op {
            OpType::Move => path.move_to(
                item.data[0].to_f32().unwrap(),
                item.data[1].to_f32().unwrap(),
            ),
            OpType::BCurveTo => path.cubic_to(
                item.data[0].to_f32().unwrap(),
                item.data[1].to_f32().unwrap(),
                item.data[2].to_f32().unwrap(),
                item.data[3].to_f32().unwrap(),
                item.data[4].to_f32().unwrap(),
                item.data[5].to_f32().unwrap(),
            ),
            OpType::LineTo => {
                path.line_to(
                    item.data[0].to_f32().unwrap(),
                    item.data[1].to_f32().unwrap(),
                );
            }
        }
    }
    path.finish().unwrap()
}

impl SkiaGenerator {
    pub fn line<F: Trig + Float + FromPrimitive>(
        &self,
        x1: F,
        y1: F,
        x2: F,
        y2: F,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.line(x1, y1, x2, y2, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn rectangle<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.rectangle(x, y, width, height, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn ellipse<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.ellipse(x, y, width, height, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn circle<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        diameter: F,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.circle(x, y, diameter, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn linear_path<F: Trig + Float + FromPrimitive>(
        &self,
        points: &[Point2D<F>],
        close: bool,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.linear_path(points, close, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn polygon<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        points: &[Point2D<F>],
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.polygon(points, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn arc<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
        start: F,
        stop: F,
        closed: bool,
    ) -> SkiaDrawable<F> {
        let drawable = self
            .gen
            .arc(x, y, width, height, start, stop, closed, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn bezier_quadratic<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        start: Point2D<F>,
        cp: Point2D<F>,
        end: Point2D<F>,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.bezier_quadratic(start, cp, end, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn bezier_cubic<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        start: Point2D<F>,
        cp1: Point2D<F>,
        cp2: Point2D<F>,
        end: Point2D<F>,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.bezier_cubic(start, cp1, cp2, end, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn curve<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        points: &[Point2D<F>],
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.curve(points, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn path<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        svg_path: String,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.path(svg_path, &self.options);
        drawable.to_skia_drawable()
    }
}
