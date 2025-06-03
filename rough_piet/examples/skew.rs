//! This example shows painting a rough svg rust logo using common-piet crate and
//! kurbo rough shape generator

use palette::Srgba;
use piet::{Color, RenderContext};
use piet_common::kurbo::Rect;
use piet_common::Device;
use rough_piet::KurboGenerator;
use roughr::core::{FillStyle, OptionsBuilder};
use svg_path_ops::pt::PathTransformer;

const WIDTH: usize = 460;
const HEIGHT: usize = 250;
/// For now, assume pixel density (dots per inch)
const DPI: f32 = 96.;

/// cargo run --example rust_logo
fn main() {
    let mut device = Device::new().unwrap();
    let mut bitmap = device.bitmap_target(WIDTH, HEIGHT, 1.0).unwrap();
    let mut rc = bitmap.render_context();
    let options = OptionsBuilder::default()
        .stroke(Srgba::from_components((114u8, 87, 82, 255u8)).into_format())
        .fill(Srgba::from_components((254u8, 246, 201, 255u8)).into_format())
        .fill_style(FillStyle::Hachure)
        .fill_weight(DPI * 0.01)
        .build()
        .unwrap();

    let generator = KurboGenerator::new(options);
    let cat_svg_path: String  = "M 201.7208 78.236 c -4.2472 -2.5448 -11.884 -17.4056 -13.5836 -19.102 c -1.6996 -1.7024 -11.4648 -5.524 -11.4648 -5.524 l -5.518 -17.4056 c -10.6168 1.274 -16.9796 18.2536 -17.4052 19.5276 c 0 0 -1.0644 29.5056 -48.188 18.044 c -33.7748 -8.216 -54.088 -16.1628 -71.1488 -0.144 c -1.0672 -1.8312 -1.7964 -3.6936 -2.3288 -5.7028 c -0.9512 -3.6248 -1.1672 -7.784 -1.164 -12.5232 c 0 -2.3224 0.0436 -4.7696 0.0468 -7.336 c -0.0032 -5.3744 -0.1972 -11.2992 -1.6184 -17.606 C 27.94 24.1544 25.2544 17.4972 20.5376 10.8648 c -2.0344 -2.8672 -6.0096 -3.5432 -8.8796 -1.5056 c -2.8668 2.0376 -3.5432 6.0124 -1.5056 8.8796 c 3.7216 5.258 5.6648 10.1472 6.7728 15.0204 c 1.0956 4.8668 1.3084 9.7588 1.3056 14.8108 c 0 2.4096 -0.0472 4.8544 -0.0472 7.336 c 0.0064 5.0584 0.1752 10.3196 1.5712 15.728 c 1.0984 4.294 3.0888 8.632 6.2032 12.6824 C 12.772 104.5832 20.0148 126.6336 20.0148 126.6336 l -13.6372 21.6964 c -0.2472 0.3912 -0.4132 0.8264 -0.4976 1.2768 L 0.0584 181.6292 c -0.2564 1.4084 0.3448 2.832 1.5276 3.6336 l 16.3916 11.1052 c 0.3504 0.2408 0.7856 0.31 1.1924 0.1972 c 0.4104 -0.1096 0.7452 -0.3976 0.9232 -0.7796 l 3.3304 -7.13 c 0.2472 -0.5352 0.1472 -1.1672 -0.2536 -1.5992 l -4.2912 -4.5792 c -0.8728 -0.9296 -1.1892 -2.2536 -0.8324 -3.4804 l 7.136 -24.4132 c 0.2316 -0.7792 0.7136 -1.4584 1.374 -1.928 l 9.5932 -6.8232 c 1.302 -0.9328 3.064 -0.8888 4.3224 0.1004 c 1.2584 0.9888 1.7152 2.6884 1.1236 4.1752 L 37.364 160.7432 c -0.488 1.2204 -0.2724 2.61 0.5604 3.624 l 24.1192 29.346 h 18.0872 c 0.7984 0 1.4464 -0.648 1.4464 -1.4428 v -9.0016 c 0 -0.7984 -0.648 -1.446 -1.4464 -1.446 h -5.7212 c -1.3368 0 -2.5636 -0.7416 -3.1928 -1.9216 l -9.7936 -16.4196 c -0.6572 -1.2424 -0.532 -2.748 0.3132 -3.8652 l 8.7792 -11.5996 c 0.6824 -0.898 1.7468 -1.43 2.8768 -1.43 h 24.4788 c 1.1488 0 2.2316 0.5504 2.9108 1.4804 c 0.6792 0.9296 0.8764 2.1252 0.532 3.2236 l -7.8904 24.864 c -0.2504 0.7828 -0.2256 1.6308 0.072 2.3976 l 5.0612 13.0392 h 15.7088 v -3.396 l -2.5976 -8.166 c -0.244 -0.7636 -0.2252 -1.5868 0.0532 -2.3408 l 10.5884 -28.7388 c 0.526 -1.4212 1.8776 -2.3632 3.3896 -2.3632 h 10.8044 c 0.9796 0 1.9184 0.4008 2.6008 1.1108 l 29.9252 41.346 c 0 0 15.2832 0 20.3788 0 c 5.0952 0 3.1236 -10.1656 -1.6964 -10.6168 c -9.1304 -0.848 -20.8016 -26.3192 -25.8968 -35.236 c -5.0956 -8.914 9.8656 -36.5196 21.2268 -37.7812 c 11.4616 -1.274 17.4056 -5.9468 19.1052 -12.316 C 205.5424 89.2748 205.968 80.7836 201.7208 78.236 z".into();
    let cat_svg_path_drawing = generator.path::<f32>(cat_svg_path.clone());
    let background_color = Color::from_hex_str("96C0B7").unwrap();

    // rotate and translate given path
    let mut translated_path = PathTransformer::new(cat_svg_path);
    translated_path.skew_x(20.0).translate(180.0, 0.0);
    let translated_path_string = translated_path.to_string();
    let translated_path_bbox = translated_path.to_box(None);

    // change colors etc of translated path to distinguish it from original
    let translated_options = OptionsBuilder::default()
        .stroke(Srgba::from_components((114u8, 87, 82, 255u8)).into_format())
        .fill(Srgba::from_components((156u8, 1, 188, 255u8)).into_format())
        .fill_style(FillStyle::Hachure)
        .fill_weight(DPI * 0.01)
        .build()
        .unwrap();
    let translated_generator = KurboGenerator::new(translated_options);
    let translated_cat_path_drawing = translated_generator.path::<f32>(translated_path_string);

    let bbox_options = OptionsBuilder::default()
        .stroke(Srgba::from_components((114u8, 87, 82, 255u8)).into_format())
        .fill_weight(DPI * 0.01)
        .build()
        .unwrap();
    let bbox_generator = KurboGenerator::new(bbox_options);
    let bbox = bbox_generator.rectangle(
        translated_path_bbox.min_x.unwrap_or(0.0),
        translated_path_bbox.min_y.unwrap_or(0.0),
        translated_path_bbox.width(),
        translated_path_bbox.height(),
    );

    rc.fill(
        Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
        &background_color,
    );
    cat_svg_path_drawing.draw(&mut rc);
    translated_cat_path_drawing.draw(&mut rc);
    bbox.draw(&mut rc);
    rc.finish().unwrap();
    std::mem::drop(rc);

    bitmap
        .save_to_file("skewed_cat.png")
        .expect("file save error");
}
