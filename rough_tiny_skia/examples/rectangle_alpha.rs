use palette::{Pixel, Srgb, Srgba};
use rough_tiny_skia::SkiaGenerator;
use roughr::core::{FillStyle, OptionsBuilder};
use tiny_skia::*;

const WIDTH: f32 = 192.0;
const HEIGHT: f32 = 108.0;
/// For now, assume pixel density (dots per inch)
const DPI: f32 = 96.;

fn main() {
    let options = OptionsBuilder::default()
        .stroke(Srgba::from_raw(&[114u8, 87u8, 82u8, 100]).into_format())
        .fill(Srgba::from_raw(&[254u8, 246u8, 201u8, 100]).into_format())
        .fill_style(FillStyle::Hachure)
        .fill_weight(DPI * 0.01)
        .build()
        .unwrap();
    let generator = SkiaGenerator::new(options);
    let rect_width = 100.0;
    let rect_height = 50.0;
    let rect = generator.rectangle::<f32>(
        (WIDTH - rect_width) / 2.0,
        (HEIGHT - rect_height) / 2.0,
        rect_width,
        rect_height,
    );

    let mut pixmap = Pixmap::new(WIDTH as u32, HEIGHT as u32).unwrap();
    let mut background_paint = Paint::default();
    background_paint.set_color_rgba8(150, 192, 183, 255);

    pixmap.fill_rect(
        Rect::from_xywh(0.0, 0.0, WIDTH, HEIGHT).unwrap(),
        &background_paint,
        Transform::identity(),
        None,
    );

    rect.draw(&mut pixmap.as_mut());

    pixmap.save_png("skia_rectangle_alpha.png").unwrap();
}
