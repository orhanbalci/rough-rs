//! This example shows painting a rough ellipse using common-piet crate and
//! kurbo rough shape generator

use palette::{Pixel, Srgba};
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
        .stroke(Srgba::from_raw(&[114u8, 87u8, 82u8, 255u8]).into_format())
        .fill(Srgba::from_raw(&[254u8, 246u8, 201u8, 255u8]).into_format())
        .fill_style(FillStyle::Hachure)
        .fill_weight(DPI * 0.01)
        .build()
        .unwrap();
    let generator = KurboGenerator::new(options);
    let ellipse_paths = generator.ellipse::<f32>(
        (WIDTH as f32) / 2.0,
        (HEIGHT as f32) / 2.0,
        WIDTH as f32 - 10.0,
        HEIGHT as f32 - 10.0,
    );
    let background_color = Color::from_hex_str("96C0B7").unwrap();

    rc.fill(
        Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
        &background_color,
    );
    ellipse_paths.draw(&mut rc);
    rc.finish().unwrap();
    std::mem::drop(rc);

    bitmap.save_to_file("ellipse.png").expect("file save error");
}
