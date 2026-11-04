use roughr::core::{LineCap, LineJoin, Options, OptionsBuilder};
use roughr::Srgba;
use rough_tiny_skia::SkiaGenerator;
use tiny_skia::Pixmap;

fn opts(cap: Option<LineCap>, join: Option<LineJoin>) -> Options {
    let mut b = OptionsBuilder::default();
    b.stroke(Srgba::from_components((0.0, 0.0, 0.0, 1.0)))
        .stroke_width(10.0)
        .roughness(0.0)
        .seed(1u64);
    if let Some(c) = cap {
        b.line_cap(c);
    }
    if let Some(j) = join {
        b.line_join(j);
    }
    b.build().unwrap()
}

fn render_line(o: Options) -> Pixmap {
    let mut pm = Pixmap::new(80, 64).unwrap();
    SkiaGenerator::new(o)
        .line::<f32>(20.0, 32.0, 60.0, 32.0)
        .draw(&mut pm.as_mut());
    pm
}

fn alpha(pm: &Pixmap, x: u32, y: u32) -> u8 {
    pm.pixel(x, y).unwrap().alpha()
}

#[test]
fn butt_cap_stays_inside_endpoint() {
    let pm = render_line(opts(Some(LineCap::Butt), None));
    assert_eq!(alpha(&pm, 17, 32), 0);
}

#[test]
fn round_and_square_caps_extend_past_endpoint() {
    let round = render_line(opts(Some(LineCap::Round), None));
    let square = render_line(opts(Some(LineCap::Square), None));
    assert!(alpha(&round, 17, 32) > 0);
    assert!(alpha(&square, 17, 32) > 0);
    // Round caps curve away at the corner, square caps fill it.
    assert_eq!(alpha(&round, 15, 27), 0);
    assert!(alpha(&square, 15, 27) > 0);
}

#[test]
fn unset_cap_matches_previous_butt_default() {
    let unset = render_line(opts(None, None));
    let butt = render_line(opts(Some(LineCap::Butt), None));
    assert_eq!(unset.data(), butt.data());
}

fn render_corner(join: Option<LineJoin>) -> Pixmap {
    use roughr::core::OpSetType;
    use rough_tiny_skia::{SkiaDrawable, SkiaOpset};
    use tiny_skia::PathBuilder;

    // One continuous subpath, so the corner is a real join.
    let mut pb = PathBuilder::new();
    pb.move_to(20.0, 60.0);
    pb.line_to(40.0, 20.0);
    pb.line_to(60.0, 60.0);
    let d = SkiaDrawable::<f32> {
        shape: "linear_path".into(),
        options: opts(Some(LineCap::Butt), join),
        sets: vec![SkiaOpset {
            op_set_type: OpSetType::Path,
            ops: pb.finish().unwrap(),
            size: None,
            path: None,
        }],
    };
    let mut pm = Pixmap::new(80, 80).unwrap();
    d.draw(&mut pm.as_mut());
    pm
}

#[test]
fn join_style_changes_apex() {
    let miter = render_corner(Some(LineJoin::Miter { limit: 10.0 }));
    let bevel = render_corner(Some(LineJoin::Bevel));
    let round = render_corner(Some(LineJoin::Round));
    let top = |pm: &Pixmap| (0..80).find(|&y| (0..80).any(|x| alpha(pm, x, y) > 0)).unwrap();
    assert!(top(&miter) < top(&round));
    assert!(top(&round) < top(&bevel));
}

#[test]
fn miter_limit_falls_back_to_bevel() {
    let low = render_corner(Some(LineJoin::Miter { limit: 1.0 }));
    let bevel = render_corner(Some(LineJoin::Bevel));
    assert_eq!(low.data(), bevel.data());
}

#[test]
fn unset_join_matches_previous_miter_default() {
    let unset = render_corner(None);
    let miter = render_corner(Some(LineJoin::Miter { limit: 4.0 }));
    assert_eq!(unset.data(), miter.data());
}
