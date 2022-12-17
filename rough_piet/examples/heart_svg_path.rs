//! This example shows painting a rough svg heart path using common-piet crate and
//! kurbo rough shape generator

use palette::{Pixel, Srgba};
use piet::{Color, RenderContext};
use piet_common::kurbo::Rect;
use piet_common::Device;
use rough_piet::KurboGenerator;
use roughr::core::{FillStyle, OptionsBuilder};

const WIDTH: usize = 500;
const HEIGHT: usize = 500;
/// For now, assume pixel density (dots per inch)
const DPI: f32 = 96.;

/// cargo run --example heart_svg_path
fn main() {
    let mut device = Device::new().unwrap();
    let mut bitmap = device.bitmap_target(WIDTH, HEIGHT, 1.0).unwrap();
    let mut rc = bitmap.render_context();
    let options = OptionsBuilder::default()
        .stroke(Srgba::from_raw(&[114u8, 87u8, 82u8, 255u8]).into_format())
        .fill(Srgba::from_raw(&[254u8, 246u8, 201u8, 255u8]).into_format())
        .fill_style(FillStyle::Dots)
        .fill_weight(DPI * 0.01)
        .build()
        .unwrap();
    let generator = KurboGenerator::new(options);
    let heart_svg_path  = "M140 20C73 20 20 74 20 140c0 135 136 170 228 303 88-132 229-173 229-303 0-66-54-120-120-120-48 0-90 28-109 69-19-41-60-69-108-69z".into();
    let heart_svg_path_drawing = generator.path::<f32>(heart_svg_path);
    let background_color = Color::from_hex_str("96C0B7").unwrap();

    rc.fill(
        Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
        &background_color,
    );
    heart_svg_path_drawing.draw(&mut rc);
    rc.finish().unwrap();
    std::mem::drop(rc);

    bitmap
        .save_to_file("heart_svg_path.png")
        .expect("file save error");
}
