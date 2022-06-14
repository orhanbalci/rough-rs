//! This example implements a generative algorithm for making pictures like
//! Piet Mondrian's squares.
// TODO: Remove all the wasm32 cfg guards once this compiles with piet-web

use kurbo_rough::KurboGenerator;
use piet::{Color, RenderContext};
use piet_common::kurbo::{Point, Size};
use piet_common::Device;
use rand::{prelude::*, random};
use rand_distr::Normal;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
/// For now, assume pixel density (dots per inch)
const DPI: f64 = 96.;

/// Feature "png" needed for save_to_file() and it's disabled by default for optional dependencies
/// cargo run --example mondrian --features png
fn main() {
    let mut device = Device::new().unwrap();
    let mut bitmap = device.bitmap_target(WIDTH, HEIGHT, 1.0).unwrap();
    let mut rc = bitmap.render_context();
    let generator = KurboGenerator::default();
    let rect = generator.rectangle::<f32>(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0, 200.0, 100.0);

    for path in rect.iter() {
        rc.stroke(path, &Color::BLACK, 0.01 * DPI);
        //rc.fill(path, &Color::RED);
    }

    rc.finish().unwrap();
    std::mem::drop(rc);

    bitmap
        .save_to_file("temp-image.png")
        .expect("file save error");
}
