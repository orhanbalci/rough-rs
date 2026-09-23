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
//!
//! The 8x8 bitmap font below is `font8x8_basic` by Daniel Hepper (public
//! domain, based on the public-domain IBM VGA font by Marcel Sondaar), see
//! <https://github.com/dhepper/font8x8>. Only the glyphs used by the labels
//! are included, and every label is drawn in uppercase.

use std::path::{Path, PathBuf};

use euclid::default::Point2D;
use palette::Srgba;
use rough_tiny_skia::SkiaGenerator;
use roughr::core::{FillStyle, Options, OptionsBuilder};
use tiny_skia::{Paint, Pixmap, Rect, Transform};

const CELL_WIDTH: f32 = 160.0;
const CELL_HEIGHT: f32 = 120.0;
const CELL_GAP: f32 = 12.0;
const SHAPE_PADDING: f32 = 24.0;

const MARGIN: f32 = 22.0;
const INSET: f32 = 24.0;
const TITLE_H: f32 = 76.0;
const LABEL_H: f32 = 28.0;
const FOOTER_H: f32 = 34.0;

// The palette used by every rough-rs example.
const TEAL: (u8, u8, u8) = (150, 192, 183);
const BROWN: (u8, u8, u8) = (114, 87, 82);
const CREAM: (u8, u8, u8) = (254, 246, 201);

const CANVAS_BG: (u8, u8, u8) = CREAM;
const CELL_BG: (u8, u8, u8) = TEAL;
const ACCENT: (u8, u8, u8) = BROWN;
const TITLE_COLOR: (u8, u8, u8) = BROWN;
const SUBTITLE_COLOR: (u8, u8, u8) = (157, 136, 120);
const LABEL_COLOR: (u8, u8, u8) = BROWN;
const WORDMARK_COLOR: (u8, u8, u8) = TEAL;

/// A heart made of cubic curves, centered on the origin. Simplification only
/// affects curves, so straight-edged paths would not show it.
const HEART: &str = "M 0 -18 C -8 -42 -48 -36 -44 -6 C -40 18 -4 30 0 38 \
                     C 4 30 40 18 44 -6 C 48 -36 8 -42 0 -18 Z";

/// 8x8 bitmap glyphs. Each row is one scanline, LSB-first.
const FONT: &[(char, [u8; 8])] = &[
    (' ', [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
    (',', [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C, 0x06]),
    ('-', [0x00, 0x00, 0x00, 0x3F, 0x00, 0x00, 0x00, 0x00]),
    ('.', [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C, 0x00]),
    ('/', [0x60, 0x30, 0x18, 0x0C, 0x06, 0x03, 0x01, 0x00]),
    ('0', [0x3E, 0x63, 0x73, 0x7B, 0x6F, 0x67, 0x3E, 0x00]),
    ('1', [0x0C, 0x0E, 0x0C, 0x0C, 0x0C, 0x0C, 0x3F, 0x00]),
    ('2', [0x1E, 0x33, 0x30, 0x1C, 0x06, 0x33, 0x3F, 0x00]),
    ('3', [0x1E, 0x33, 0x30, 0x1C, 0x30, 0x33, 0x1E, 0x00]),
    ('4', [0x38, 0x3C, 0x36, 0x33, 0x7F, 0x30, 0x78, 0x00]),
    ('5', [0x3F, 0x03, 0x1F, 0x30, 0x30, 0x33, 0x1E, 0x00]),
    ('6', [0x1C, 0x06, 0x03, 0x1F, 0x33, 0x33, 0x1E, 0x00]),
    ('7', [0x3F, 0x33, 0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x00]),
    ('8', [0x1E, 0x33, 0x33, 0x1E, 0x33, 0x33, 0x1E, 0x00]),
    ('9', [0x1E, 0x33, 0x33, 0x3E, 0x30, 0x18, 0x0E, 0x00]),
    (':', [0x00, 0x0C, 0x0C, 0x00, 0x00, 0x0C, 0x0C, 0x00]),
    ('(', [0x18, 0x0C, 0x06, 0x06, 0x06, 0x0C, 0x18, 0x00]),
    (')', [0x06, 0x0C, 0x18, 0x18, 0x18, 0x0C, 0x06, 0x00]),
    ('>', [0x06, 0x0C, 0x18, 0x30, 0x18, 0x0C, 0x06, 0x00]),
    ('A', [0x0C, 0x1E, 0x33, 0x33, 0x3F, 0x33, 0x33, 0x00]),
    ('B', [0x3F, 0x66, 0x66, 0x3E, 0x66, 0x66, 0x3F, 0x00]),
    ('C', [0x3C, 0x66, 0x03, 0x03, 0x03, 0x66, 0x3C, 0x00]),
    ('D', [0x1F, 0x36, 0x66, 0x66, 0x66, 0x36, 0x1F, 0x00]),
    ('E', [0x7F, 0x46, 0x16, 0x1E, 0x16, 0x46, 0x7F, 0x00]),
    ('F', [0x7F, 0x46, 0x16, 0x1E, 0x16, 0x06, 0x0F, 0x00]),
    ('G', [0x3C, 0x66, 0x03, 0x03, 0x73, 0x66, 0x7C, 0x00]),
    ('H', [0x33, 0x33, 0x33, 0x3F, 0x33, 0x33, 0x33, 0x00]),
    ('I', [0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00]),
    ('J', [0x78, 0x30, 0x30, 0x30, 0x33, 0x33, 0x1E, 0x00]),
    ('K', [0x67, 0x66, 0x36, 0x1E, 0x36, 0x66, 0x67, 0x00]),
    ('L', [0x0F, 0x06, 0x06, 0x06, 0x46, 0x66, 0x7F, 0x00]),
    ('M', [0x63, 0x77, 0x7F, 0x7F, 0x6B, 0x63, 0x63, 0x00]),
    ('N', [0x63, 0x67, 0x6F, 0x7B, 0x73, 0x63, 0x63, 0x00]),
    ('O', [0x1C, 0x36, 0x63, 0x63, 0x63, 0x36, 0x1C, 0x00]),
    ('P', [0x3F, 0x66, 0x66, 0x3E, 0x06, 0x06, 0x0F, 0x00]),
    ('Q', [0x1E, 0x33, 0x33, 0x33, 0x3B, 0x1E, 0x38, 0x00]),
    ('R', [0x3F, 0x66, 0x66, 0x3E, 0x36, 0x66, 0x67, 0x00]),
    ('S', [0x1E, 0x33, 0x07, 0x0E, 0x38, 0x33, 0x1E, 0x00]),
    ('T', [0x3F, 0x2D, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00]),
    ('U', [0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x3F, 0x00]),
    ('V', [0x33, 0x33, 0x33, 0x33, 0x33, 0x1E, 0x0C, 0x00]),
    ('W', [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00]),
    ('X', [0x63, 0x63, 0x36, 0x1C, 0x1C, 0x36, 0x63, 0x00]),
    ('Y', [0x33, 0x33, 0x33, 0x1E, 0x0C, 0x0C, 0x1E, 0x00]),
    ('Z', [0x7F, 0x63, 0x31, 0x18, 0x4C, 0x66, 0x7F, 0x00]),
    ('_', [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF]),
];

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

/// Framed canvas shared by every image: cream background, a hand-drawn
/// accent border, title/subtitle top-left and a wordmark bottom-right.
struct Canvas {
    pixmap: Pixmap,
    width: f32,
    height: f32,
    cells_x: f32,
}

impl Canvas {
    fn new(title: &str, subtitle: &str, cell_count: usize) -> Self {
        let cells_width = CELL_WIDTH * cell_count as f32 + CELL_GAP * (cell_count - 1) as f32;
        let content_width = cells_width
            .max(text_width(title, 3.0))
            .max(text_width(subtitle, 1.0));
        let width = content_width + 2.0 * (MARGIN + INSET);
        let height = MARGIN + TITLE_H + CELL_HEIGHT + LABEL_H + FOOTER_H + MARGIN;
        let mut canvas = Canvas {
            pixmap: Pixmap::new(width as u32, height as u32).unwrap(),
            width,
            height,
            // Center the cells when the title is wider than they are
            cells_x: MARGIN + INSET + (content_width - cells_width) / 2.0,
        };

        canvas.fill(0.0, 0.0, width, height, CANVAS_BG);

        let border = OptionsBuilder::default()
            .stroke(Srgba::from_components((ACCENT.0, ACCENT.1, ACCENT.2, 255u8)).into_format())
            .stroke_width(2.5)
            .roughness(1.5)
            .bowing(1.0)
            .disable_multi_stroke(true)
            .build()
            .unwrap();
        SkiaGenerator::new(border)
            .rectangle::<f32>(MARGIN, MARGIN, width - 2.0 * MARGIN, height - 2.0 * MARGIN)
            .draw(&mut canvas.pixmap.as_mut());

        let text_x = MARGIN + INSET;
        canvas.text(title, text_x, MARGIN + 18.0, 3.0, TITLE_COLOR);
        canvas.text(
            subtitle,
            text_x,
            MARGIN + 18.0 + 27.0 + 10.0,
            1.0,
            SUBTITLE_COLOR,
        );
        let wordmark = "ROUGH-RS";
        canvas.text(
            wordmark,
            width - MARGIN - INSET - text_width(wordmark, 2.0),
            height - MARGIN - 12.0 - 16.0,
            2.0,
            WORDMARK_COLOR,
        );
        canvas
    }

    /// Top-left corner of the `i`th cell.
    fn cell_origin(&self, i: usize) -> (f32, f32) {
        (
            self.cells_x + i as f32 * (CELL_WIDTH + CELL_GAP),
            MARGIN + TITLE_H,
        )
    }

    fn fill(&mut self, x: f32, y: f32, w: f32, h: f32, color: (u8, u8, u8)) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.0, color.1, color.2, 255);
        if let Some(rect) = Rect::from_xywh(x, y, w, h) {
            self.pixmap
                .fill_rect(rect, &paint, Transform::identity(), None);
        }
    }

    /// Draws `text` at `(x, y)` with each font pixel scaled up by `scale`.
    fn text(&mut self, text: &str, x: f32, y: f32, scale: f32, color: (u8, u8, u8)) {
        let mut cursor = x;
        for ch in text.chars() {
            let bits = glyph(ch);
            for (row, byte) in bits.iter().enumerate() {
                for col in 0..8 {
                    if byte & (1 << col) != 0 {
                        self.fill(
                            cursor + col as f32 * scale,
                            y + row as f32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
            cursor += 9.0 * scale;
        }
    }

    fn save(&self, out_dir: &Path, name: &str) {
        let path = out_dir.join(format!("{name}.png"));
        self.pixmap.save_png(&path).unwrap();
        println!("wrote {} ({}x{})", path.display(), self.width, self.height);
    }
}

fn glyph(c: char) -> [u8; 8] {
    let c = c.to_ascii_uppercase();
    FONT.iter()
        .find(|(g, _)| *g == c)
        .map(|(_, bits)| *bits)
        .unwrap_or([0; 8])
}

fn text_width(text: &str, scale: f32) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    text.chars().count() as f32 * 9.0 * scale - scale
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
    let mut canvas = Canvas::new(name, subtitle, values.len());

    for (i, &value) in values.iter().enumerate() {
        let mut builder = base();
        set(&mut builder, value);
        let options = builder.build().expect("valid options");

        let (cx, cy) = canvas.cell_origin(i);
        canvas.fill(cx, cy, CELL_WIDTH, CELL_HEIGHT, CELL_BG);
        draw_shape(&mut canvas, options, shape, cx, cy);

        let label = value.label();
        let label_x = cx + (CELL_WIDTH - text_width(&label, 1.0)) / 2.0;
        canvas.text(&label, label_x, cy + CELL_HEIGHT + 10.0, 1.0, LABEL_COLOR);
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
