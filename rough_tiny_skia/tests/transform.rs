use roughr::core::{FillStyle, OptionsBuilder};
use roughr::Srgba;
use rough_tiny_skia::SkiaGenerator;
use tiny_skia::{Pixmap, Transform};

fn opts() -> roughr::core::Options {
    OptionsBuilder::default()
        .stroke(Srgba::from_components((1.0, 0.0, 0.0, 1.0)))
        .fill(Srgba::from_components((0.0, 0.0, 1.0, 1.0)))
        .fill_style(FillStyle::Solid)
        .roughness(0.0)
        .seed(1u64)
        .build()
        .unwrap()
}

fn painted(pm: &Pixmap) -> Vec<(u32, u32)> {
    let mut v = vec![];
    for y in 0..pm.height() {
        for x in 0..pm.width() {
            if pm.pixel(x, y).unwrap().alpha() > 0 {
                v.push((x, y));
            }
        }
    }
    v
}

#[test]
fn identity_matches_draw() {
    let d = SkiaGenerator::new(opts()).rectangle::<f32>(10.0, 10.0, 30.0, 20.0);
    let mut a = Pixmap::new(100, 100).unwrap();
    let mut b = Pixmap::new(100, 100).unwrap();
    d.draw(&mut a.as_mut());
    d.draw_with_transform(&mut b.as_mut(), Transform::identity());
    assert_eq!(a.data(), b.data());
}

#[test]
fn translate_shifts_pixels() {
    let d = SkiaGenerator::new(opts()).rectangle::<f32>(10.0, 10.0, 30.0, 20.0);
    let mut a = Pixmap::new(100, 100).unwrap();
    let mut b = Pixmap::new(100, 100).unwrap();
    d.draw(&mut a.as_mut());
    d.draw_with_transform(&mut b.as_mut(), Transform::from_translate(25.0, 15.0));
    let shifted: Vec<_> = painted(&a).into_iter().map(|(x, y)| (x + 25, y + 15)).collect();
    assert_eq!(shifted, painted(&b));
}

#[test]
fn scale_grows_bounds() {
    let d = SkiaGenerator::new(opts()).rectangle::<f32>(10.0, 10.0, 20.0, 20.0);
    let mut a = Pixmap::new(100, 100).unwrap();
    let mut b = Pixmap::new(100, 100).unwrap();
    d.draw(&mut a.as_mut());
    d.draw_with_transform(&mut b.as_mut(), Transform::from_scale(2.0, 2.0));
    let max = |p: &Pixmap| painted(p).iter().map(|&(x, _)| x).max().unwrap();
    assert!(max(&b) > max(&a) + 20);
}
