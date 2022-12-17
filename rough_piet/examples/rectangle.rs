//! This example shows painting a rough rectangle using common-piet crate and
//! kurbo rough shape generator

use palette::{Pixel, Srgb, Srgba};
use piet::{Color, RenderContext};
use piet_common::kurbo::Rect;
use piet_common::Device;
use rough_piet::KurboGenerator;
use roughr::core::{FillStyle, OptionsBuilder};

const WIDTH: usize = 192;
const HEIGHT: usize = 108;
/// For now, assume pixel density (dots per inch)
const DPI: f32 = 96.;

/// Feature "png" needed for save_to_file() and it's disabled by default for optional dependencies
/// cargo run --example mondrian --features png
fn main() {
    let mut device = Device::new().unwrap();
    let mut bitmap = device.bitmap_target(WIDTH, HEIGHT, 1.0).unwrap();
    let mut rc = bitmap.render_context();
    let options = OptionsBuilder::default()
        .stroke(Srgba::from_raw(&[114u8, 87u8, 82u8, 255]).into_format())
        .fill(Srgba::from_raw(&[254u8, 246u8, 201u8, 255]).into_format())
        .fill_style(FillStyle::ZigZagLine)
        .fill_weight(DPI * 0.01)
        .build()
        .unwrap();
    let generator = KurboGenerator::new(options);
    let rect_width = 100.0;
    let rect_height = 50.0;
    let rect = generator.rectangle::<f32>(
        (WIDTH as f32 - rect_width) / 2.0,
        (HEIGHT as f32 - rect_height) / 2.0,
        rect_width,
        rect_height,
    );
    let background_color = Color::from_hex_str("96C0B7").unwrap();

    rc.fill(
        Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
        &background_color,
    );
    rect.draw(&mut rc);

    rc.finish().unwrap();
    std::mem::drop(rc);

    bitmap
        .save_to_file("rectangle.png")
        .expect("file save error");
}
