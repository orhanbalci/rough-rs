//! Framed reference-image canvas shared by the gallery examples: a cream
//! background, a hand-drawn border, a title and subtitle top-left, a strip of
//! labelled cells and a wordmark bottom-right.
//!
//! The 8x8 bitmap font below is `font8x8_basic` by Daniel Hepper (public
//! domain, based on the public-domain IBM VGA font by Marcel Sondaar), see
//! <https://github.com/dhepper/font8x8>. Only the glyphs used by the labels
//! are included, and every label is drawn in uppercase.

// Each example uses a different part of this module.
#![allow(dead_code)]

use std::path::Path;

use palette::Srgba;
use rough_tiny_skia::SkiaGenerator;
use roughr::core::OptionsBuilder;
use tiny_skia::{Paint, Pixmap, Rect, Transform};

pub const CELL_GAP: f32 = 12.0;

pub const MARGIN: f32 = 22.0;
pub const INSET: f32 = 24.0;
pub const TITLE_H: f32 = 76.0;
pub const LABEL_H: f32 = 28.0;
pub const FOOTER_H: f32 = 34.0;

// The palette used by every rough-rs example.
pub const TEAL: (u8, u8, u8) = (150, 192, 183);
pub const BROWN: (u8, u8, u8) = (114, 87, 82);
pub const CREAM: (u8, u8, u8) = (254, 246, 201);

pub const CANVAS_BG: (u8, u8, u8) = CREAM;
pub const CELL_BG: (u8, u8, u8) = TEAL;
pub const ACCENT: (u8, u8, u8) = BROWN;
pub const TITLE_COLOR: (u8, u8, u8) = BROWN;
pub const SUBTITLE_COLOR: (u8, u8, u8) = (157, 136, 120);
pub const LABEL_COLOR: (u8, u8, u8) = BROWN;
pub const WORDMARK_COLOR: (u8, u8, u8) = TEAL;

/// 8x8 bitmap glyphs. Each row is one scanline, LSB-first.
pub const FONT: &[(char, [u8; 8])] = &[
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

/// Size of the cells in a [`Canvas`] strip, and the wordmark it shows.
#[derive(Clone, Copy)]
pub struct Layout {
    pub cell_width: f32,
    pub cell_height: f32,
    pub wordmark: &'static str,
}

/// Framed canvas shared by every image: cream background, a hand-drawn
/// accent border, title/subtitle top-left and a wordmark bottom-right.
pub struct Canvas {
    pub pixmap: Pixmap,
    width: f32,
    height: f32,
    cells_x: f32,
    layout: Layout,
}

impl Canvas {
    pub fn new(layout: Layout, title: &str, subtitle: &str, cell_count: usize) -> Self {
        let Layout { cell_width, cell_height, wordmark } = layout;
        let cells_width = cell_width * cell_count as f32 + CELL_GAP * (cell_count - 1) as f32;
        let content_width = cells_width
            .max(text_width(title, 3.0))
            .max(text_width(subtitle, 1.0));
        let width = content_width + 2.0 * (MARGIN + INSET);
        let height = MARGIN + TITLE_H + cell_height + LABEL_H + FOOTER_H + MARGIN;
        let mut canvas = Canvas {
            pixmap: Pixmap::new(width as u32, height as u32).unwrap(),
            width,
            height,
            // Center the cells when the title is wider than they are
            cells_x: MARGIN + INSET + (content_width - cells_width) / 2.0,
            layout,
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
    pub fn cell_origin(&self, i: usize) -> (f32, f32) {
        (
            self.cells_x + i as f32 * (self.layout.cell_width + CELL_GAP),
            MARGIN + TITLE_H,
        )
    }

    /// Fills the `i`th cell's background and returns its top-left corner.
    pub fn cell(&mut self, i: usize) -> (f32, f32) {
        let (x, y) = self.cell_origin(i);
        self.fill(
            x,
            y,
            self.layout.cell_width,
            self.layout.cell_height,
            CELL_BG,
        );
        (x, y)
    }

    /// Prints `label` centered under the `i`th cell.
    pub fn label(&mut self, i: usize, label: &str) {
        let (x, y) = self.cell_origin(i);
        let label_x = x + (self.layout.cell_width - text_width(label, 1.0)) / 2.0;
        self.text(
            label,
            label_x,
            y + self.layout.cell_height + 10.0,
            1.0,
            LABEL_COLOR,
        );
    }

    pub fn fill(&mut self, x: f32, y: f32, w: f32, h: f32, color: (u8, u8, u8)) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.0, color.1, color.2, 255);
        if let Some(rect) = Rect::from_xywh(x, y, w, h) {
            self.pixmap
                .fill_rect(rect, &paint, Transform::identity(), None);
        }
    }

    /// Draws `text` at `(x, y)` with each font pixel scaled up by `scale`.
    pub fn text(&mut self, text: &str, x: f32, y: f32, scale: f32, color: (u8, u8, u8)) {
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

    pub fn save(&self, out_dir: &Path, name: &str) {
        let path = out_dir.join(format!("{name}.png"));
        self.pixmap.save_png(&path).unwrap();
        println!("wrote {} ({}x{})", path.display(), self.width, self.height);
    }
}

pub fn glyph(c: char) -> [u8; 8] {
    let c = c.to_ascii_uppercase();
    FONT.iter()
        .find(|(g, _)| *g == c)
        .map(|(_, bits)| *bits)
        .unwrap_or([0; 8])
}

pub fn text_width(text: &str, scale: f32) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    text.chars().count() as f32 * 9.0 * scale - scale
}
