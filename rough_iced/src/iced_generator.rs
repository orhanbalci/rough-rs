use std::fmt::Display;
use std::ops::MulAssign;

use euclid::default::Point2D;
use euclid::Trig;

use iced::widget::canvas::fill::Rule;
use iced::widget::canvas::path::lyon_path::PathEvent;
use iced::widget::canvas::{Fill, Frame, LineDash, Path, Stroke, Style};
use iced::{Color, Point};
use num_traits::{Float, FromPrimitive};
use palette::rgb::Rgba;
use palette::Srgba;
use roughr::core::{Drawable, OpSet, OpSetType, OpType, Options};
use roughr::generator::Generator;
use roughr::PathSegment;

#[derive(Default)]
pub struct IcedGenerator {
    gen: Generator,
    options: Option<Options>,
}

#[derive(Clone)]
pub struct IcedOpset<F: Float + Trig> {
    pub op_set_type: OpSetType,
    pub ops: Path,
    pub size: Option<Point2D<F>>,
    pub path: Option<String>,
}

pub trait ToIcedOpset<F: Float + Trig> {
    fn to_iced_opset(self) -> IcedOpset<F>;
}

impl<F: Float + Trig + FromPrimitive> ToIcedOpset<F> for OpSet<F> {
    fn to_iced_opset(self) -> IcedOpset<F> {
        IcedOpset {
            op_set_type: self.op_set_type.clone(),
            size: self.size,
            path: self.path.clone(),
            ops: opset_to_shape(&self),
        }
    }
}

pub struct IcedDrawable<F: Float + Trig> {
    pub shape: String,
    pub options: Options,
    pub sets: Vec<IcedOpset<F>>,
}

pub trait ToIcedDrawable<F: Float + Trig> {
    fn to_iced_drawable(self) -> IcedDrawable<F>;
}

impl<F: Float + Trig + FromPrimitive> ToIcedDrawable<F> for Drawable<F> {
    fn to_iced_drawable(self) -> IcedDrawable<F> {
        IcedDrawable {
            shape: self.shape,
            options: self.options,
            sets: self.sets.into_iter().map(|s| s.to_iced_opset()).collect(),
        }
    }
}

impl IcedGenerator {
    pub fn new(options: Options) -> Self {
        IcedGenerator { gen: Generator::default(), options: Some(options) }
    }
}

impl<F: Float + Trig> IcedDrawable<F> {
    pub fn draw(&self, frame: &mut Frame) {
        let stroke_line_dash = self
            .options
            .stroke_line_dash
            .clone()
            .unwrap_or(Vec::new())
            .iter()
            .map(|&a| a as f32)
            .collect::<Vec<f32>>();

        let fill_line_dash = self
            .options
            .fill_line_dash
            .clone()
            .unwrap_or(Vec::new())
            .iter()
            .map(|&a| a as f32)
            .collect::<Vec<f32>>();

        for set in self.sets.iter() {
            match set.op_set_type {
                OpSetType::Path => {
                    //ctx.save().expect("Failed to save render context");
                    frame.with_save(|f| {
                        if self.options.stroke_line_dash.is_some() {
                            let mut stroke_style = Stroke::default();
                            stroke_style.line_dash = LineDash {
                                segments: stroke_line_dash.as_slice(),
                                offset: self
                                    .options
                                    .stroke_line_dash_offset
                                    .map(|a| a as usize)
                                    .unwrap_or(1),
                            };
                            stroke_style = stroke_style
                                .with_line_cap(convert_line_cap_from_roughr_to_iced(
                                    self.options.line_cap,
                                ))
                                .with_line_join(convert_line_join_from_roughr_to_iced(
                                    self.options.line_join,
                                ));

                            let stroke_color = self
                                .options
                                .stroke
                                .unwrap_or_else(|| Srgba::from_components((1.0, 1.0, 1.0, 1.0)));
                            let rgb: (f32, f32, f32, f32) = stroke_color.into_components();
                            f.stroke(
                                &set.ops,
                                stroke_style
                                    .with_color(Color::from_rgba(rgb.0, rgb.1, rgb.2, rgb.3))
                                    .with_width(self.options.stroke_width.unwrap_or(1.0)),
                            );
                            //ctx.restore().expect("Failed to restore render context");
                        } else {
                            let stroke_style = Stroke::default();
                            let stroke_color = self
                                .options
                                .stroke
                                .unwrap_or_else(|| Srgba::new(1.0, 1.0, 1.0, 1.0));
                            let rgb: (f32, f32, f32, f32) = stroke_color.into_components();
                            f.stroke(
                                &set.ops,
                                stroke_style
                                    .with_color(Color::from_rgba(rgb.0, rgb.1, rgb.2, rgb.3))
                                    .with_width(self.options.stroke_width.unwrap_or(1.0)),
                            );
                        }
                    })
                }
                OpSetType::FillPath => {
                    frame.with_save(|f| match self.shape.as_str() {
                        "curve" | "polygon" | "path" => {
                            let fill_color =
                                self.options.fill.unwrap_or(Rgba::new(1.0, 1.0, 1.0, 1.0));
                            let rgb: (f32, f32, f32, f32) = fill_color.into_components();

                            f.fill(
                                &set.ops,
                                Fill {
                                    style: Style::Solid(Color::from_rgba(
                                        rgb.0, rgb.1, rgb.2, rgb.3,
                                    )),
                                    rule: Rule::EvenOdd,
                                },
                            )
                        }
                        _ => {
                            let fill_color =
                                self.options.fill.unwrap_or(Rgba::new(1.0, 1.0, 1.0, 1.0));
                            let rgb: (f32, f32, f32, f32) = fill_color.into_components();
                            f.fill(
                                &set.ops,
                                Fill {
                                    style: Style::Solid(Color::from_rgba(
                                        rgb.0, rgb.1, rgb.2, rgb.3,
                                    )),
                                    rule: Rule::EvenOdd,
                                },
                            )
                        }
                    });
                }
                OpSetType::FillSketch => {
                    let mut fweight = self.options.fill_weight.unwrap_or_default();
                    if fweight < 0.0 {
                        fweight = self.options.stroke_width.unwrap_or(1.0) / 2.0;
                    }
                    frame.with_save(|f| {
                        if self.options.fill_line_dash.is_some() {
                            let mut stroke_style = Stroke::default();
                            stroke_style.line_dash = LineDash {
                                segments: fill_line_dash.as_slice(),
                                offset: self
                                    .options
                                    .fill_line_dash_offset
                                    .map(|a| a as usize)
                                    .unwrap_or(0),
                            };
                            stroke_style = stroke_style
                                .with_line_cap(convert_line_cap_from_roughr_to_iced(
                                    self.options.line_cap,
                                ))
                                .with_line_join(convert_line_join_from_roughr_to_iced(
                                    self.options.line_join,
                                ));
                            let fill_color = self
                                .options
                                .fill
                                .unwrap_or_else(|| Rgba::new(1.0, 1.0, 1.0, 1.0));
                            let rgb: (f32, f32, f32, f32) = fill_color.into_components();
                            f.stroke(
                                &set.ops,
                                stroke_style
                                    .with_color(Color::from_rgba(rgb.0, rgb.1, rgb.2, rgb.3))
                                    .with_width(fweight),
                            );
                        } else {
                            let fill_color = self
                                .options
                                .fill
                                .unwrap_or_else(|| Rgba::new(1.0, 1.0, 1.0, 1.0));
                            let rgb: (f32, f32, f32, f32) = fill_color.into_components();
                            let stroke_style = Stroke::default();
                            f.stroke(
                                &set.ops,
                                stroke_style
                                    .with_color(Color::from_rgba(rgb.0, rgb.1, rgb.2, rgb.3))
                                    .with_width(fweight),
                            );
                        }
                    });
                }
            }
        }
    }
}

fn opset_to_shape<F: Trig + Float + FromPrimitive>(op_set: &OpSet<F>) -> Path {
    let path: Path = Path::new(|b| {
        for item in op_set.ops.iter() {
            match item.op {
                OpType::Move => b.move_to(Point::new(
                    item.data[0].to_f32().unwrap(),
                    item.data[1].to_f32().unwrap(),
                )),
                OpType::BCurveTo => b.bezier_curve_to(
                    Point::new(
                        item.data[0].to_f32().unwrap(),
                        item.data[1].to_f32().unwrap(),
                    ),
                    Point::new(
                        item.data[2].to_f32().unwrap(),
                        item.data[3].to_f32().unwrap(),
                    ),
                    Point::new(
                        item.data[4].to_f32().unwrap(),
                        item.data[5].to_f32().unwrap(),
                    ),
                ),
                OpType::LineTo => {
                    b.line_to(Point::new(
                        item.data[0].to_f32().unwrap(),
                        item.data[1].to_f32().unwrap(),
                    ));
                }
            }
        }
    });

    path
}

impl IcedGenerator {
    pub fn line<F: Trig + Float + FromPrimitive>(
        &self,
        x1: F,
        y1: F,
        x2: F,
        y2: F,
    ) -> IcedDrawable<F> {
        let drawable = self.gen.line(x1, y1, x2, y2, &self.options);
        drawable.to_iced_drawable()
    }

    pub fn rectangle<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
    ) -> IcedDrawable<F> {
        let drawable = self.gen.rectangle(x, y, width, height, &self.options);
        drawable.to_iced_drawable()
    }

    pub fn ellipse<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
    ) -> IcedDrawable<F> {
        let drawable = self.gen.ellipse(x, y, width, height, &self.options);
        drawable.to_iced_drawable()
    }

    pub fn circle<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        diameter: F,
    ) -> IcedDrawable<F> {
        let drawable = self.gen.circle(x, y, diameter, &self.options);
        drawable.to_iced_drawable()
    }

    pub fn linear_path<F: Trig + Float + FromPrimitive>(
        &self,
        points: &[Point2D<F>],
        close: bool,
    ) -> IcedDrawable<F> {
        let drawable = self.gen.linear_path(points, close, &self.options);
        drawable.to_iced_drawable()
    }

    pub fn polygon<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        points: &[Point2D<F>],
    ) -> IcedDrawable<F> {
        let drawable = self.gen.polygon(points, &self.options);
        drawable.to_iced_drawable()
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
    ) -> IcedDrawable<F> {
        let drawable = self
            .gen
            .arc(x, y, width, height, start, stop, closed, &self.options);
        drawable.to_iced_drawable()
    }

    pub fn bezier_quadratic<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        start: Point2D<F>,
        cp: Point2D<F>,
        end: Point2D<F>,
    ) -> IcedDrawable<F> {
        let drawable = self.gen.bezier_quadratic(start, cp, end, &self.options);
        drawable.to_iced_drawable()
    }

    pub fn bezier_cubic<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        start: Point2D<F>,
        cp1: Point2D<F>,
        cp2: Point2D<F>,
        end: Point2D<F>,
    ) -> IcedDrawable<F> {
        let drawable = self.gen.bezier_cubic(start, cp1, cp2, end, &self.options);
        drawable.to_iced_drawable()
    }

    pub fn curve<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        points: &[Point2D<F>],
    ) -> IcedDrawable<F> {
        let drawable = self.gen.curve(points, &self.options);
        drawable.to_iced_drawable()
    }

    pub fn path<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        svg_path: String,
    ) -> IcedDrawable<F> {
        let drawable = self.gen.path(svg_path, &self.options);
        drawable.to_iced_drawable()
    }

    pub fn bez_path<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        path: Path,
    ) -> IcedDrawable<F> {
        let segments = path_to_svg_segments(&path);
        self.gen
            .path_from_segments(segments, &self.options)
            .to_iced_drawable()
    }
}

fn convert_line_cap_from_roughr_to_iced(
    roughr_line_cap: Option<roughr::core::LineCap>,
) -> iced::widget::canvas::LineCap {
    match roughr_line_cap {
        Some(roughr::core::LineCap::Butt) => iced::widget::canvas::LineCap::Butt,
        Some(roughr::core::LineCap::Round) => iced::widget::canvas::LineCap::Round,
        Some(roughr::core::LineCap::Square) => iced::widget::canvas::LineCap::Square,
        None => iced::widget::canvas::LineCap::Butt,
    }
}

fn convert_line_join_from_roughr_to_iced(
    roughr_line_join: Option<roughr::core::LineJoin>,
) -> iced::widget::canvas::LineJoin {
    match roughr_line_join {
        Some(roughr::core::LineJoin::Miter { limit: _ }) => iced::widget::canvas::LineJoin::Miter,
        Some(roughr::core::LineJoin::Round) => iced::widget::canvas::LineJoin::Round,
        Some(roughr::core::LineJoin::Bevel) => iced::widget::canvas::LineJoin::Bevel,
        None => iced::widget::canvas::LineJoin::Miter,
    }
}

pub fn path_to_svg_segments(path: &Path) -> Vec<PathSegment> {
    let mut segments = Vec::new();

    for elem in path.raw() {
        match elem {
            PathEvent::Begin { at: p } => {
                segments.push(PathSegment::MoveTo { abs: true, x: p.x as f64, y: p.y as f64 });
            }
            PathEvent::Line { from: _, to: p } => {
                segments.push(PathSegment::LineTo { abs: true, x: p.x as f64, y: p.y as f64 });
            }
            PathEvent::Quadratic { from: p1, ctrl: _, to: p2 } => {
                segments.push(PathSegment::Quadratic {
                    abs: true,
                    x1: p1.x as f64,
                    y1: p1.y as f64,
                    x: p2.x as f64,
                    y: p2.y as f64,
                });
            }
            PathEvent::Cubic { from: p1, ctrl1: p2, ctrl2: _, to: p3 } => {
                segments.push(PathSegment::CurveTo {
                    abs: true,
                    x1: p1.x as f64,
                    y1: p1.y as f64,
                    x2: p2.x as f64,
                    y2: p2.y as f64,
                    x: p3.x as f64,
                    y: p3.y as f64,
                });
            }
            PathEvent::End { last: _, first: _, close: _ } => {
                segments.push(PathSegment::ClosePath { abs: true });
            }
        }
    }

    segments
}
