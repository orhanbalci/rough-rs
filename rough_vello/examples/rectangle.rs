mod common;

use palette::Srgba;
use rough_vello::VelloGenerator;
use roughr::core::{FillStyle, OptionsBuilder};
use vello::kurbo::Affine;
use vello::peniko::Color;

const CANVAS_WIDTH: f64 = 800.0;
const CANVAS_HEIGHT: f64 = 600.0;

fn main() {
    // Create rough rectangle using Vello
    let options = OptionsBuilder::default()
        .stroke(Srgba::from_components((114u8, 87u8, 82u8, 255u8)).into_format())
        .fill(Srgba::from_components((254u8, 246u8, 201u8, 255)).into_format())
        .fill_style(FillStyle::ZigZagLine)
        .fill_weight(96.0 * 0.01)
        .bowing(0.8)
        .build()
        .unwrap();

    let generator = VelloGenerator::new(options);
    let rect_width = 300.0;
    let rect_height = 200.0;
    // Position rectangle at the center of the canvas
    // For centering: (canvas_size - rect_size) / 2
    let rect = generator.rectangle::<f32>(
        (CANVAS_WIDTH as f32 - rect_width) / 2.0,
        (CANVAS_HEIGHT as f32 - rect_height) / 2.0,
        rect_width,
        rect_height,
    );

    // Draw the rough rectangle into its own scene once
    let mut rect_scene = vello::Scene::new();
    rect.draw(&mut rect_scene);

    common::run(
        "rough_vello: rectangle",
        Color::from_rgb8(150, 192, 183),
        false,
        move |scene, frame| {
            // Keep the canvas centered in the window
            let offset = Affine::translate((
                (frame.width - CANVAS_WIDTH) / 2.0,
                (frame.height - CANVAS_HEIGHT) / 2.0,
            ));
            scene.append(&rect_scene, Some(offset));
        },
    );
}
