//! This example shows painting a rough rectangle using Vello crate and
//! the rough Vello generator

use palette::Srgba;
use rough_vello::VelloGenerator;
use roughr::core::{FillStyle, OptionsBuilder};
use vello::Scene;

const WIDTH: usize = 192;
const HEIGHT: usize = 108;

fn main() {
    let options = OptionsBuilder::default()
        .stroke(Srgba::from_components((114u8, 87u8, 82u8, 255u8)).into_format())
        .fill(Srgba::from_components((254u8, 246u8, 201u8, 255)).into_format())
        .fill_style(FillStyle::ZigZagLine)
        .fill_weight(1.0)
        .build()
        .unwrap();
    
    let generator = VelloGenerator::new(options);
    let rect_width = 100.0;
    let rect_height = 50.0;
    let rect = generator.rectangle::<f32>(
        (WIDTH as f32 - rect_width) / 2.0,
        (HEIGHT as f32 - rect_height) / 2.0,
        rect_width,
        rect_height,
    );

    let mut scene = Scene::new();
    rect.draw(&mut scene);

    println!("Successfully created a rough rectangle using Vello!");
}
