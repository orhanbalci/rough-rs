use piet::{Color, RenderContext};
use piet_common::kurbo::{Affine, BezPath, Rect, Shape};
use piet_common::Device;
use rough_piet::KurboGenerator;
use roughr::core::{FillStyle, OptionsBuilder};
use roughr::Srgba;

const WIDTH: usize = 500;
const HEIGHT: usize = 200;

/// Feature "png" needed for save_to_file() and it's disabled by default for optional dependencies
/// cargo run --example mondrian --features png
fn main() {
    let mut device = Device::new().unwrap();
    let mut bitmap = device.bitmap_target(WIDTH, HEIGHT, 1.0).unwrap();
    let mut rc = bitmap.render_context();

    let background_color = Color::from_hex_str("96C0B7").unwrap();

    rc.fill(
        Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
        &background_color,
    );

    let mut f = text2v::monospace_font();
    let path_iter = f.render("rough-rs", 2000.0);
    let mut combined = BezPath::from_iter(path_iter.0);
    combined.apply_affine(Affine::scale(100.0));
    let bb = combined.bounding_box();
    combined.apply_affine(Affine::translate((
        (WIDTH as f64 / 2.0) - (bb.width() / 2.0),
        (HEIGHT as f64 / 2.0) - (bb.height() / 2.0),
    )));

    const DPI: f32 = 96.0;
    let text_options = OptionsBuilder::default()
        .stroke(Srgba::from_components((114u8, 87, 82, 255u8)).into_format())
        .fill(Srgba::from_components((254u8, 246, 201, 255u8)).into_format())
        .fill_style(FillStyle::CrossHatch)
        .fill_weight(DPI * 0.01)
        .build()
        .unwrap();
    let text_generator = KurboGenerator::new(text_options);
    let text_rough = text_generator.bez_path::<f32>(combined);
    text_rough.draw(&mut rc);
    rc.finish().unwrap();
    std::mem::drop(rc);

    bitmap
        .save_to_file("rough_text.png")
        .expect("file save error");
}
