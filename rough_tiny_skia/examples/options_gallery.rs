//! Generates the option reference images used in the roughr documentation.
//!
//! Each image is a strip of cells that draw the same shape while one option
//! changes from left to right; every other option keeps its default. Run it
//! from the workspace root after changing how shapes are rendered:
//!
//! ```sh
//! cargo run -p rough_tiny_skia --example options_gallery
//! ```
//!
//! The images are written to `roughr/assets/options/`.

mod common;

use std::path::{Path, PathBuf};

use common::{Canvas, Layout};
use euclid::default::Point2D;
use palette::Srgba;
use rough_tiny_skia::SkiaGenerator;
use roughr::core::{FillStyle, Options, OptionsBuilder};

const CELL_WIDTH: f32 = 160.0;
const CELL_HEIGHT: f32 = 120.0;
const SHAPE_PADDING: f32 = 24.0;

const LAYOUT: Layout = Layout {
    cell_width: CELL_WIDTH,
    cell_height: CELL_HEIGHT,
    wordmark: "ROUGH-RS",
};

/// A heart made of cubic curves, centered on the origin. Simplification only
/// affects curves, so straight-edged paths would not show it.
const HEART: &str = "M 0 -18 C -8 -42 -48 -36 -44 -6 C -40 18 -4 30 0 38 \
                     C 4 30 40 18 44 -6 C 48 -36 8 -42 0 -18 Z";

#[derive(Clone, Copy)]
enum Shape {
    Rectangle,
    Ellipse,
    Curve,
    Triangle,
    Heart,
}

/// How a strip value is printed under its cell.
trait Label {
    fn label(&self) -> String;
}

impl Label for f32 {
    fn label(&self) -> String {
        self.to_string()
    }
}

impl Label for u64 {
    fn label(&self) -> String {
        self.to_string()
    }
}

impl Label for bool {
    fn label(&self) -> String {
        self.to_string()
    }
}

impl Label for FillStyle {
    fn label(&self) -> String {
        self.to_string()
    }
}

impl Label for &[f64] {
    fn label(&self) -> String {
        if self.is_empty() {
            "none".to_owned()
        } else {
            self.iter()
                .map(f64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }
    }
}

/// Starting point for every cell: stroke only, in the colors used across the
/// rough-rs READMEs.
fn stroke_only() -> OptionsBuilder {
    let mut builder = OptionsBuilder::default();
    builder.stroke(Srgba::from_components((114u8, 87u8, 82u8, 255u8)).into_format());
    builder
}

/// Like [`stroke_only`] with a hachure fill.
fn filled() -> OptionsBuilder {
    let mut builder = stroke_only();
    builder
        .fill(Srgba::from_components((254u8, 246u8, 201u8, 255u8)).into_format())
        .fill_style(FillStyle::Hachure);
    builder
}

fn strip<V: Copy + Label>(
    out_dir: &Path,
    name: &str,
    subtitle: &str,
    shape: Shape,
    base: fn() -> OptionsBuilder,
    values: &[V],
    set: impl Fn(&mut OptionsBuilder, V),
) {
    let mut canvas = Canvas::new(LAYOUT, name, subtitle, values.len());

    for (i, &value) in values.iter().enumerate() {
        let mut builder = base();
        set(&mut builder, value);
        let options = builder.build().expect("valid options");

        let (cx, cy) = canvas.cell(i);
        draw_shape(&mut canvas, options, shape, cx, cy);

        canvas.label(i, &value.label());
    }

    canvas.save(out_dir, name);
}

fn draw_shape(canvas: &mut Canvas, options: Options, shape: Shape, cx: f32, cy: f32) {
    let generator = SkiaGenerator::new(options);
    let x = cx + SHAPE_PADDING;
    let y = cy + SHAPE_PADDING;
    let w = CELL_WIDTH - 2.0 * SHAPE_PADDING;
    let h = CELL_HEIGHT - 2.0 * SHAPE_PADDING;
    let drawable = match shape {
        Shape::Rectangle => generator.rectangle::<f32>(x, y, w, h),
        Shape::Ellipse => generator.ellipse::<f32>(x + w / 2.0, y + h / 2.0, w, h),
        Shape::Curve => {
            let points = [
                Point2D::new(x, y + h),
                Point2D::new(x + w * 0.3, y),
                Point2D::new(x + w * 0.6, y + h * 0.8),
                Point2D::new(x + w, y + h * 0.1),
            ];
            generator.curve::<f32>(&points)
        }
        Shape::Triangle => generator.polygon::<f32>(&[
            Point2D::new(x + w / 2.0, y),
            Point2D::new(x + w, y + h),
            Point2D::new(x, y + h),
        ]),
        Shape::Heart => {
            let center = (x + w / 2.0, y + h / 2.0);
            generator.path::<f32>(translate_path(HEART, center))
        }
    };
    drawable.draw(&mut canvas.pixmap.as_mut());
}

/// Offsets every coordinate pair of an absolute `M`/`L`/`C` path.
fn translate_path(d: &str, (dx, dy): (f32, f32)) -> String {
    let mut is_x = true;
    d.split_whitespace()
        .map(|token| match token.parse::<f32>() {
            Ok(value) => {
                let offset = if is_x { dx } else { dy };
                is_x = !is_x;
                (value + offset).to_string()
            }
            Err(_) => token.to_owned(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn main() {
    let out_dir: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "..",
        "roughr",
        "assets",
        "options",
    ]
    .iter()
    .collect();
    std::fs::create_dir_all(&out_dir).unwrap();
    let out = out_dir.as_path();

    // Stroke options
    strip(
        out,
        "roughness",
        "how far lines wander from the true shape (default 1)",
        Shape::Rectangle,
        stroke_only,
        &[0.0f32, 0.5, 1.0, 2.0, 4.0],
        |b, v| {
            b.roughness(v);
        },
    );
    strip(
        out,
        "bowing",
        "how much straight lines bend in the middle (default 2)",
        Shape::Rectangle,
        stroke_only,
        &[0.0f32, 1.0, 2.0, 5.0, 10.0],
        |b, v| {
            b.bowing(v);
        },
    );
    strip(
        out,
        "max_randomness_offset",
        "upper bound for random end point offsets, in px (default 2)",
        Shape::Rectangle,
        stroke_only,
        &[0.0f32, 1.0, 2.0, 4.0, 8.0],
        |b, v| {
            b.max_randomness_offset(v);
        },
    );
    strip(
        out,
        "stroke_width",
        "outline width in px (default 1)",
        Shape::Rectangle,
        stroke_only,
        &[0.5f32, 1.0, 2.0, 3.0, 5.0],
        |b, v| {
            b.stroke_width(v);
        },
    );
    strip(
        out,
        "disable_multi_stroke",
        "draw each outline once instead of twice (default false)",
        Shape::Rectangle,
        stroke_only,
        &[false, true],
        |b, v| {
            b.disable_multi_stroke(v);
        },
    );
    strip(
        out,
        "seed",
        "same seed, same drawing (default 345)",
        Shape::Rectangle,
        filled,
        &[1u64, 2, 3, 4, 5],
        |b, v| {
            b.seed(v);
        },
    );
    strip(
        out,
        "preserve_vertices",
        "keep corners in place (roughness 3)",
        Shape::Triangle,
        stroke_only,
        &[false, true],
        |b, v| {
            b.roughness(3.0).preserve_vertices(v);
        },
    );
    strip(
        out,
        "stroke_line_dash",
        "dash pattern for outlines, in px (default none)",
        Shape::Rectangle,
        stroke_only,
        &[&[][..], &[8.0, 4.0], &[2.0, 4.0], &[12.0, 4.0, 2.0, 4.0]],
        |b, v| {
            if !v.is_empty() {
                b.stroke_line_dash(v.to_vec());
            }
        },
    );

    // Curve options
    strip(
        out,
        "curve_fitting",
        "how closely ellipses follow the true radius (default 0.95)",
        Shape::Ellipse,
        stroke_only,
        &[0.5f32, 0.75, 0.9, 0.95, 1.0],
        |b, v| {
            b.curve_fitting(v);
        },
    );
    strip(
        out,
        "curve_step_count",
        "points used to approximate ellipses (default 9)",
        Shape::Ellipse,
        stroke_only,
        &[3.0f32, 5.0, 9.0, 15.0, 30.0],
        |b, v| {
            b.curve_step_count(v);
        },
    );
    strip(
        out,
        "curve_tightness",
        "how tightly curves pass through their points (default 0)",
        Shape::Curve,
        stroke_only,
        &[-1.0f32, -0.5, 0.0, 0.5, 1.0],
        |b, v| {
            b.curve_tightness(v);
        },
    );
    strip(
        out,
        "simplification",
        "below 1, drops points from svg paths (default 1)",
        Shape::Heart,
        stroke_only,
        &[1.0f32, 0.75, 0.5, 0.25, 0.0],
        |b, v| {
            b.simplification(v);
        },
    );

    // Fill options
    strip(
        out,
        "fill_style",
        "how the fill color is sketched",
        Shape::Rectangle,
        filled,
        &[
            FillStyle::Hachure,
            FillStyle::Solid,
            FillStyle::ZigZag,
            FillStyle::CrossHatch,
            FillStyle::Dots,
            FillStyle::Dashed,
            FillStyle::ZigZagLine,
        ],
        |b, v| {
            b.fill_style(v);
        },
    );
    strip(
        out,
        "hachure_angle",
        "angle of fill lines in degrees (default -41)",
        Shape::Rectangle,
        filled,
        &[-41.0f32, 0.0, 30.0, 60.0, 90.0],
        |b, v| {
            b.hachure_angle(v);
        },
    );
    strip(
        out,
        "hachure_gap",
        "space between fill lines in px (default: 4 x stroke_width)",
        Shape::Rectangle,
        filled,
        &[2.0f32, 4.0, 8.0, 12.0, 20.0],
        |b, v| {
            b.hachure_gap(v);
        },
    );
    strip(
        out,
        "fill_weight",
        "width of fill lines in px (default: stroke_width / 2)",
        Shape::Rectangle,
        filled,
        &[0.5f32, 1.0, 2.0, 3.0, 5.0],
        |b, v| {
            b.hachure_gap(8.0).fill_weight(v);
        },
    );
    strip(
        out,
        "dash_offset",
        "dash length for dashed fills (default: hachure_gap)",
        Shape::Rectangle,
        filled,
        &[2.0f32, 5.0, 10.0, 15.0, 20.0],
        |b, v| {
            b.fill_style(FillStyle::Dashed)
                .hachure_gap(6.0)
                .dash_offset(v);
        },
    );
    strip(
        out,
        "dash_gap",
        "gap between dashes for dashed fills (default: hachure_gap)",
        Shape::Rectangle,
        filled,
        &[1.0f32, 3.0, 6.0, 10.0, 15.0],
        |b, v| {
            b.fill_style(FillStyle::Dashed).hachure_gap(6.0).dash_gap(v);
        },
    );
    strip(
        out,
        "zigzag_offset",
        "zigzag size for zigzagline fills (default: hachure_gap)",
        Shape::Rectangle,
        filled,
        &[2.0f32, 4.0, 6.0, 8.0, 10.0],
        |b, v| {
            b.fill_style(FillStyle::ZigZagLine)
                .hachure_gap(8.0)
                .zigzag_offset(v);
        },
    );
    strip(
        out,
        "disable_multi_stroke_fill",
        "draw each fill line once instead of twice (default false, gap 10)",
        Shape::Rectangle,
        filled,
        &[false, true],
        |b, v| {
            b.hachure_gap(10.0).disable_multi_stroke_fill(v);
        },
    );
}
