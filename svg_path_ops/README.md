# svg_path_ops

[![Crates.io](https://img.shields.io/crates/v/svg_path_ops.svg)](https://crates.io/crates/svg_path_ops)
[![Documentation](https://docs.rs/svg_path_ops/badge.svg)](https://docs.rs/svg_path_ops)
[![License](https://img.shields.io/github/license/orhanbalci/rough-rs/tree/main/svg_path_ops.svg)](https://github.com/orhanbalci/rough-rs/blob/main/svg_path_ops/LICENSE)

<!-- cargo-sync-readme start -->


This crate includes utility functions to work with svg paths. Works on types from [svgtypes](https://github.com/RazrFalcon/svgtypes)
crate.

This package exposes functions to manipulate svg paths with simplification purposes.


## 📦 Cargo.toml

```toml
[dependencies]
svg_path_ops = "0.1"
```

## 🔧 Example

```rust
use svgtypes::{PathParser, PathSegment};

let path: String = "m 0 0 c 3 -0.6667 6 -1.3333 9 -2 a 1 1 0 0 0 -8 -1 a 1 1 0 0 0 -2 0 l 0 4 v 2 h 8 q 4 -10 9 -5 t -6 8 z".into();
let path_parser = PathParser::from(path.as_ref());
let path_segments: Vec<PathSegment> = path_parser.flatten().collect();
let mut absolute = absolutize(path_segments.iter());
assert_eq!(
    absolute.next().unwrap(),
    PathSegment::MoveTo { abs: true, x: 0.0, y: 0.0 }
);
assert_eq!(
    absolute.next().unwrap(),
    PathSegment::CurveTo {
        abs: true,
        x1: 3.0,
        y1: -0.6667,
        x2: 6.0,
        y2: -1.3333,
        x: 9.0,
        y: -2.0
    }
);
assert_eq!(
    absolute.next().unwrap(),
    PathSegment::EllipticalArc {
        abs: true,
        rx: 1.0,
        ry: 1.0,
        x_axis_rotation: 0.0,
        large_arc: false,
        sweep: false,
        x: 1.0,
        y: -3.0
    }
);

assert_eq!(
    absolute.next().unwrap(),
    PathSegment::EllipticalArc {
        abs: true,
        rx: 1.0,
        ry: 1.0,
        x_axis_rotation: 0.0,
        large_arc: false,
        sweep: false,
        x: -1.0,
        y: -3.0
    }
);
assert_eq!(
    absolute.next().unwrap(),
    PathSegment::LineTo { abs: true, x: -1.0, y: 1.0 }
);
assert_eq!(
    absolute.next().unwrap(),
    PathSegment::VerticalLineTo { abs: true, y: 3.0 }
);
assert_eq!(
    absolute.next().unwrap(),
    PathSegment::HorizontalLineTo { abs: true, x: 7.0 }
);
assert_eq!(
    absolute.next().unwrap(),
    PathSegment::Quadratic { abs: true, x1: 11.0, y1: -7.0, x: 16.0, y: -2.0 }
);
assert_eq!(
    absolute.next().unwrap(),
    PathSegment::SmoothQuadratic { abs: true, x: 10.0, y: 6.0 }
);
assert_eq!(
    absolute.next().unwrap(),
    PathSegment::ClosePath { abs: true }
);
```

<!-- cargo-sync-readme end -->

## 📝 License

Licensed under MIT License ([LICENSE](LICENSE)).

### 🚧 Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the MIT license, shall be licensed as above, without any additional terms or conditions.
