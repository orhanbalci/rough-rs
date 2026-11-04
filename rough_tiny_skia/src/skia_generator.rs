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
    LineJoin,
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
    /// Converts to a [`SkiaOpset`], panicking if the op set produces no
    /// drawable geometry (see [`try_to_skia_opset`](Self::try_to_skia_opset)).
    fn to_skia_opset(self) -> SkiaOpset<F>;

    /// Converts to a [`SkiaOpset`], returning `None` when the op set is empty
    /// or otherwise yields no valid path (tiny-skia cannot represent an empty
    /// [`Path`]).
    fn try_to_skia_opset(self) -> Option<SkiaOpset<F>>;
}

impl<F: Float + Trig + FromPrimitive> ToSkiaOpset<F> for OpSet<F> {
    fn to_skia_opset(self) -> SkiaOpset<F> {
        self.try_to_skia_opset()
            .expect("op set has no drawable geometry; use try_to_skia_opset")
    }

    fn try_to_skia_opset(self) -> Option<SkiaOpset<F>> {
        let ops = opset_to_shape(&self)?;
        Some(SkiaOpset {
            op_set_type: self.op_set_type.clone(),
            size: self.size,
            path: self.path.clone(),
            ops,
        })
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
    /// Converts every op set, silently dropping those with no drawable
    /// geometry (empty sets), so the result never panics on degenerate input.
    fn to_skia_drawable(self) -> SkiaDrawable<F> {
        SkiaDrawable {
            shape: self.shape,
            options: self.options,
            sets: self.sets.into_iter().filter_map(|s| s.try_to_skia_opset()).collect(),
        }
    }
}

impl SkiaGenerator {
    pub fn new(options: Options) -> Self {
        SkiaGenerator { gen: Generator::default(), options: Some(options) }
    }
}

impl<F: Float + Trig> SkiaDrawable<F> {
    /// Draws the drawable onto `ctx` with no transform applied.
    ///
    /// Equivalent to [`draw_with_transform`](Self::draw_with_transform) with
    /// [`Transform::identity`].
    pub fn draw(&self, ctx: &mut PixmapMut) {
        self.draw_with_transform(ctx, Transform::identity());
    }

    /// Draws the drawable onto `ctx`, applying `transform` during rasterization.
    ///
    /// The transform maps the drawable's own coordinates (the ones passed to
    /// the generator) to pixmap coordinates, so canvas pan, zoom and HiDPI
    /// scaling can be applied without rebuilding the drawable.
    ///
    /// The transform is applied *after* stroking, as tiny-skia does: stroke
    /// widths, dash lengths and dash offsets are expressed in drawable space
    /// and scale together with the geometry. A 2x zoom therefore doubles line
    /// thickness, exactly as if the whole scene had been generated at 2x.
    pub fn draw_with_transform(&self, ctx: &mut PixmapMut, transform: Transform) {
        for set in self.sets.iter() {
            match set.op_set_type {
                OpSetType::Path => {
                    if self.options.stroke_line_dash.is_some() {
                        let mut stroke = Stroke {
                            width: self.options.stroke_width.unwrap_or(1.0),
                            line_cap: convert_line_cap_from_roughr_to_piet(self.options.line_cap),
                            line_join: convert_line_join_from_roughr_to_piet(
                                self.options.line_join,
                            ),
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

                        ctx.stroke_path(&set.ops, &paint, &stroke, transform, None);
                    } else {
                        let mut stroke = Stroke::default();
                        stroke.width = self.options.stroke_width.unwrap_or(1.0);
                        // Only override tiny-skia's defaults (butt cap, miter join)
                        // when the caller set them explicitly.
                        if let Some(cap) = self.options.line_cap {
                            stroke.line_cap = convert_line_cap_from_roughr_to_piet(Some(cap));
                        }
                        if let Some(join) = self.options.line_join {
                            stroke.line_join = convert_line_join_from_roughr_to_piet(Some(join));
                            if let roughr::core::LineJoin::Miter { limit } = join {
                                stroke.miter_limit = limit as f32;
                            }
                        }

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

                        ctx.stroke_path(&set.ops, &paint, &stroke, transform, None);
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
                                transform,
                                None,
                            );
                        }
                        _ => {
                            ctx.fill_path(
                                &set.ops,
                                &paint,
                                FillRule::Winding,
                                transform,
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
                        stroke.width = fweight;
                        stroke.line_cap =
                            convert_line_cap_from_roughr_to_piet(self.options.line_cap);
                        stroke.line_join =
                            convert_line_join_from_roughr_to_piet(self.options.line_join);
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
                        ctx.stroke_path(&set.ops, &paint, &stroke, transform, None);
                    } else {
                        let mut stroke = Stroke::default();
                        stroke.width = fweight;
                        stroke.line_cap =
                            convert_line_cap_from_roughr_to_piet(self.options.line_cap);
                        stroke.line_join =
                            convert_line_join_from_roughr_to_piet(self.options.line_join);

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
                        ctx.stroke_path(&set.ops, &paint, &stroke, transform, None);
                    }
                }
            }
        }
    }
}

fn opset_to_shape<F: Trig + Float + FromPrimitive>(op_set: &OpSet<F>) -> Option<Path> {
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
    path.finish()
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

fn convert_line_cap_from_roughr_to_piet(roughr_line_cap: Option<roughr::core::LineCap>) -> LineCap {
    match roughr_line_cap {
        Some(roughr::core::LineCap::Butt) => LineCap::Butt,
        Some(roughr::core::LineCap::Round) => LineCap::Round,
        Some(roughr::core::LineCap::Square) => LineCap::Square,
        None => LineCap::Round,
    }
}

fn convert_line_join_from_roughr_to_piet(
    roughr_line_join: Option<roughr::core::LineJoin>,
) -> LineJoin {
    match roughr_line_join {
        Some(roughr::core::LineJoin::Miter { limit: _ }) => LineJoin::Miter,
        Some(roughr::core::LineJoin::Round) => LineJoin::Round,
        Some(roughr::core::LineJoin::Bevel) => LineJoin::Bevel,
        None => LineJoin::Miter,
    }
}
