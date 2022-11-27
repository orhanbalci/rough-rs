//! This example shows painting cubic bezier curve using common-piet crate and
//! kurbo rough shape generator

use euclid::Point2D;
use palette::{Pixel, Srgb};
use piet::{Color, RenderContext};
use piet_common::kurbo::Rect;
use piet_common::Device;
use rough_piet::KurboGenerator;
use roughr::core::{FillStyle, OptionsBuilder};

const WIDTH: usize = 250;
const HEIGHT: usize = 200;
/// For now, assume pixel density (dots per inch)
const DPI: f32 = 96.;

fn main() {
    let mut device = Device::new().unwrap();
    let mut bitmap = device.bitmap_target(WIDTH, HEIGHT, 1.0).unwrap();
    let mut rc = bitmap.render_context();
    let options = OptionsBuilder::default()
        .stroke(Srgb::from_raw(&[114u8, 87u8, 82u8]).into_format())
        .fill(Srgb::from_raw(&[254u8, 246u8, 201u8]).into_format())
        .fill_style(FillStyle::Hachure)
        .fill_weight(DPI * 0.01)
        .build()
        .unwrap();
    let generator = KurboGenerator::new(options);
    let bezier_quadratic = generator.bezier_cubic::<f32>(
        Point2D::new(WIDTH as f32, (1.0 / 3.0) * HEIGHT as f32),
        Point2D::new((2.0 / 3.0) * WIDTH as f32, (5.0 / 3.0) * HEIGHT as f32),
        Point2D::new((1.0 / 3.0) * WIDTH as f32, -(1.0 / 3.0) * HEIGHT as f32),
        Point2D::new(0.0, (2.0 / 3.0) * HEIGHT as f32),
    );
    let background_color = Color::from_hex_str("96C0B7").unwrap();

    rc.fill(
        Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
        &background_color,
    );
    bezier_quadratic.draw(&mut rc);
    rc.finish().unwrap();
    std::mem::drop(rc);

    bitmap
        .save_to_file("bezier_cubic.png")
        .expect("file save error");
}
