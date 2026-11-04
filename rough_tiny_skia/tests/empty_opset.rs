use roughr::core::{Drawable, Op, OpSet, OpSetType, OpType, Options};
use rough_tiny_skia::{ToSkiaDrawable, ToSkiaOpset};
use tiny_skia::Pixmap;

fn empty() -> OpSet<f32> {
    OpSet { op_set_type: OpSetType::Path, ops: vec![], size: None, path: None }
}

fn lone_move() -> OpSet<f32> {
    OpSet {
        op_set_type: OpSetType::Path,
        ops: vec![Op { op: OpType::Move, data: vec![1.0, 2.0] }],
        size: None,
        path: None,
    }
}

fn line() -> OpSet<f32> {
    OpSet {
        op_set_type: OpSetType::Path,
        ops: vec![
            Op { op: OpType::Move, data: vec![1.0, 1.0] },
            Op { op: OpType::LineTo, data: vec![20.0, 20.0] },
        ],
        size: None,
        path: None,
    }
}

#[test]
fn try_to_skia_opset_is_none_for_empty() {
    assert!(empty().try_to_skia_opset().is_none());
    assert!(lone_move().try_to_skia_opset().is_none());
    assert!(line().try_to_skia_opset().is_some());
}

#[test]
fn drawable_skips_empty_sets_and_draws() {
    let d = Drawable {
        shape: "line".into(),
        options: Options::default(),
        sets: vec![empty(), line(), lone_move()],
    }
    .to_skia_drawable();
    assert_eq!(d.sets.len(), 1);
    let mut pm = Pixmap::new(32, 32).unwrap();
    d.draw(&mut pm.as_mut());
    assert!(pm.data().iter().any(|&b| b != 0));
}

#[test]
fn drawable_with_only_empty_sets_draws_nothing() {
    let d = Drawable { shape: "line".into(), options: Options::default(), sets: vec![empty()] }
        .to_skia_drawable();
    assert!(d.sets.is_empty());
    let mut pm = Pixmap::new(8, 8).unwrap();
    d.draw(&mut pm.as_mut());
    assert!(pm.data().iter().all(|&b| b == 0));
}
