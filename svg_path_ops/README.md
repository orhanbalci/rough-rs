# svg_path_ops

[![Crates.io](https://img.shields.io/crates/v/svg_path_ops.svg)](https://crates.io/crates/svg_path_ops)
[![Documentation](https://docs.rs/svg_path_ops/badge.svg)](https://docs.rs/svg_path_ops)
[![License](https://img.shields.io/github/license/orhanbalci/rough-rs.svg)](https://github.com/orhanbalci/rough-rs/blob/main/svg_path_ops/LICENSE)

<!-- cargo-sync-readme start -->

Read, transform and write SVG path data.

The crate works on [`PathSegment`] values from [`svgtypes`], and keeps them
as they are unless you ask otherwise: arcs stay arcs, relative commands
stay relative and shorthand commands stay shorthand. [`svgtypes`] and
[`euclid`] are re-exported, so you don't need matching dependencies of your
own.

The reference images were drawn with
[rough_tiny_skia](https://github.com/orhanbalci/rough-rs/tree/main/rough_tiny_skia)'s
`path_ops_gallery` example, using Ferris the crab by Karen Rustad Tölva
(CC0, <https://rustacean.net>).

## 📦 Cargo.toml

```toml
[dependencies]
svg_path_ops = "0.11"
```

## 🔧 Usage

### Transforming a path

[`PathTransformer`] applies transforms in the order
they are added, and writes the result with its `Display` implementation.
In the images below the original path is dashed in purple.

![translate](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/translate.png)

![rotate](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/rotate.png)

![scale](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/scale.png)

![skew_x](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/skew_x.png)

![skew_y](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/skew_y.png)

```rust
use svg_path_ops::pt::PathTransformer;

let mut path = PathTransformer::parse("M 10 10 L 50 10 L 30 40 Z")?;
path.scale(2.0, 2.0).translate(10.0, 0.0);
assert_eq!(path.to_string(), "M 30 20 L 110 20 L 70 80 Z");
```

It also takes the value of an SVG `transform` attribute:

```rust
use svg_path_ops::pt::PathTransformer;

let mut path = PathTransformer::parse("M 10 10 L 50 10")?;
path.transform("translate(5 5) scale(2)".into());
assert_eq!(path.to_string(), "M 25 25 L 105 25");
```

### Writing path data

[`WriteOptions`] rounds numbers and writes compact output. Use it with
[`to_string_with`], or with
[`write_path`] for any sequence of segments.

```rust
use svg_path_ops::pt::PathTransformer;
use svg_path_ops::WriteOptions;

let mut path = PathTransformer::parse("M 10 10 L 50 10")?;
path.rotate(90.0, 10.0, 10.0);
assert_eq!(path.to_string(), "M 10 10 L 10.000000000000004 50");

let rounded = WriteOptions { precision: Some(3), ..WriteOptions::default() };
assert_eq!(path.to_string_with(&rounded), "M 10 10 L 10 50");

let compact = WriteOptions { precision: Some(3), compact: true };
assert_eq!(path.to_string_with(&compact), "M10 10L10 50");
```

### Converting commands

[`PathTransformer`] converts between absolute and
relative commands, expands shorthand commands and replaces arcs with cubic
curves:

![unarc](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/unarc.png)

![unshort](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/unshort.png)

```rust
use svg_path_ops::pt::PathTransformer;

let mut path = PathTransformer::parse("M 10 10 l 40 0 l -20 30 z")?;
assert_eq!(path.abs().to_string(), "M 10 10 L 50 10 L 30 40 Z");
assert_eq!(path.rel().to_string(), "M 10 10 l 40 0 l -20 30 z");

let mut smooth = PathTransformer::parse("M 0 0 C 10 0 20 10 30 10 S 50 20 60 20")?;
assert_eq!(
    smooth.unshort().to_string(),
    "M 0 0 C 10 0 20 10 30 10 C 40 10 50 20 60 20"
);
```

[`normalize`] reduces absolute segments to move, line, cubic curve and
close commands, for targets that support nothing else:

```rust
use svg_path_ops::svgtypes::PathParser;
use svg_path_ops::{normalize, write_path, PathSegment, WriteOptions};

let segments: Vec<PathSegment> =
    PathParser::from("M 10 10 H 50 Q 50 40 20 40 Z").collect::<Result<_, _>>()?;
let normalized: Vec<PathSegment> = normalize(segments.iter()).collect();

let options = WriteOptions { precision: Some(2), ..WriteOptions::default() };
assert_eq!(
    write_path(&normalized, &options),
    "M 10 10 L 50 10 C 50 30 40 40 20 40 Z"
);
```

### Reversing a path

[`reverse`] draws every subpath in the opposite direction, keeping
relative segments relative and arcs as arcs. The shape stays the same;
the direction matters for holes under the default nonzero fill rule, for
the order a pen plotter draws in, for stroke animations and for where
markers and text on a path go. Below, the inner square only cuts a hole
once it is reversed:

![reverse](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/reverse.png)

```rust
use svg_path_ops::pt::PathTransformer;

let mut path = PathTransformer::parse("M 0 0 L 10 0 l 0 10 A 5 5 0 0 1 0 10 Z")?;
assert_eq!(
    path.reverse().to_string(),
    "M 0 10 A 5 5 0 0 0 10 10 l 0 -10 L 0 0 Z"
);
```

### Bounding boxes

[`to_box`] measures a path, and [`inbox`] fits it into a box:

![to_box](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/to_box.png)

![inbox](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/inbox.png)

![inbox_alignment](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/inbox_alignment.png)

```rust
use svg_path_ops::bbox::{BBox, InboxParameters};
use svg_path_ops::pt::PathTransformer;

let mut path = PathTransformer::parse("M 10 10 L 50 10 L 30 40 Z")?;
let bbox = path.to_box(None);
assert_eq!((bbox.width(), bbox.height()), (40.0, 30.0));

path.inbox(InboxParameters {
    destination: BBox::from("0 0 100 100"),
    ..InboxParameters::default()
});
assert_eq!(path.to_string(), "M 0 12.5 L 100 12.5 L 50 87.5 Z");
```

### Walking segments

[`segments_with_context`] gives each segment its absolute start and end
points, so relative and shorthand segments can be handled without
tracking the current point:

![segments_with_context](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/segments_with_context.png)

```rust
use svg_path_ops::euclid::default::Point2D;
use svg_path_ops::segments_with_context;
use svg_path_ops::svgtypes::PathParser;

let segments: Vec<_> =
    PathParser::from("M 10 10 l 5 0 s 5 5 10 0").collect::<Result<_, _>>()?;
let last = segments_with_context(&segments).last().unwrap();

assert_eq!(last.start, Point2D::new(15.0, 10.0));
assert_eq!(last.end, Point2D::new(25.0, 10.0));
assert_eq!(last.implied_control, Some(Point2D::new(15.0, 10.0)));
```

### Invalid path data

[`PathTransformer::parse`] returns an error for invalid path data.
[`PathTransformer::new`] keeps the segments before the first error instead, as SVG renderers do:

```rust
use svg_path_ops::pt::PathTransformer;

assert!(PathTransformer::parse("M 10 10 L 20 20 L 5").is_err());

let lenient = PathTransformer::new("M 10 10 L 20 20 L 5".into());
assert_eq!(lenient.to_string(), "M 10 10 L 20 20");
```

[`PathSegment`]: https://docs.rs/svgtypes/0.16/svgtypes/enum.PathSegment.html
[`svgtypes`]: https://docs.rs/svgtypes/0.16
[`euclid`]: https://docs.rs/euclid/0.22
[`PathTransformer`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html
[`PathTransformer::parse`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.parse
[`PathTransformer::new`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.new
[`to_string_with`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.to_string_with
[`to_box`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.to_box
[`inbox`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.inbox
[`WriteOptions`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.WriteOptions.html
[`write_path`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.write_path.html
[`normalize`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.normalize.html
[`segments_with_context`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.segments_with_context.html
[`reverse`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.reverse.html

<!-- cargo-sync-readme end -->

## 📝 License

Licensed under MIT License ([LICENSE](LICENSE)).

### 🚧 Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the MIT license, shall be licensed as above, without any additional terms or conditions.
