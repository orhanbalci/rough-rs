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

[`flip_x`] and [`flip_y`] mirror a path in place, about the center of its
bounding box:

![flip](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/flip.png)

```rust
use svg_path_ops::pt::PathTransformer;

let mut triangle = PathTransformer::parse("M 0 0 L 16 0 L 8 16")?;
assert_eq!(triangle.flip_y().to_string(), "M 0 16 L 16 16 L 8 0");
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
assert_eq!(path.to_string_with(&compact), "M10 10 10 50");
```

[`optimize`] goes further and rewrites each segment in its shortest
form: relative or absolute, `H`/`V` for straight lines, `S`/`T` for
mirrored curves, lines for straight curves and arcs for curves that
follow a circle, dropping segments that draw nothing. Relative
coordinates are taken between rounded absolute points, so rounding does
not drift along a path:

```rust
use svg_path_ops::pt::PathTransformer;
use svg_path_ops::WriteOptions;

let mut path = PathTransformer::parse(
    "M 100 100 L 110 100 L 110 110 C 110 120 120 120 120 110 \
     C 120 100 130 100 130 110 L 100 100 Z",
)?;
let options = WriteOptions { precision: Some(2), compact: true };
assert_eq!(
    path.optimize(Some(2)).to_string_with(&options),
    "M100 100h10v10c0 10 10 10 10 0s10-10 10 0z"
);
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

### Converting shapes

[`Shape`] turns the SVG basic shapes into the paths SVG 2 defines for
them, starting where a browser starts and going the same way, so markers
and dashes land in the same places:

![shapes](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/shapes.png)

```rust
use svg_path_ops::shapes::Shape;
use svg_path_ops::{write_path, WriteOptions};

let circle = Shape::Circle { cx: 50.0, cy: 50.0, r: 10.0 };
assert_eq!(
    write_path(circle.to_path(), &WriteOptions::default()),
    "M 60 50 A 10 10 0 0 1 50 60 A 10 10 0 0 1 40 50 \
     A 10 10 0 0 1 50 40 A 10 10 0 0 1 60 50 Z"
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

[`PathMeasure::is_clockwise`] tells which way a path runs, and
[`reorient`] turns every subpath of a shape with holes the right way
round: outlines one way and the holes in them the other, as fonts and
icon sets expect, so the shape fills the same under both fill rules.
[`join`] joins two open paths where their ends meet, reversing one when
it runs the other way:

![reorient and join](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/reorient_join.png)

```rust
use svg_path_ops::pt::PathTransformer;
use svg_path_ops::svgtypes::PathParser;
use svg_path_ops::{join, write_path, WriteOptions};

// Both squares clockwise: the inner one fills instead of cutting a hole
let mut frame = PathTransformer::parse("M 0 0 H 30 V 30 H 0 Z M 10 10 H 20 V 20 H 10 Z")?;
assert_eq!(frame.measure().area(), 1000.0);
frame.reorient(true);
assert_eq!(frame.measure().area(), 800.0);

let parse = |data| PathParser::from(data).collect::<Result<Vec<_>, _>>();
let joined = join(&parse("M 0 0 L 10 0")?, &parse("M 20 0 L 10 0")?, 0.0);
assert_eq!(
    write_path(&joined, &WriteOptions::default()),
    "M 0 0 L 10 0 L 20 0"
);
```

### Subpaths

A path can hold several subpaths, each started by a move.
[`split_subpaths`] returns them as paths of their own, and [`is_closed`]
tells whether every subpath ends with a close path:

![split_subpaths](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/split_subpaths.png)

```rust
use svg_path_ops::svgtypes::PathParser;
use svg_path_ops::{is_closed, split_subpaths, write_path, WriteOptions};

let segments: Vec<_> =
    PathParser::from("M 0 0 h 10 v 10 z m 20 0 h 10").collect::<Result<_, _>>()?;
assert!(!is_closed(&segments));

let subpaths = split_subpaths(&segments);
assert!(is_closed(&subpaths[0]));
// The relative move depended on the first subpath, so it becomes absolute
assert_eq!(
    write_path(&subpaths[1], &WriteOptions::default()),
    "M 20 0 h 10"
);
```

Returning to the start is not the same as closing: without a close path
the corner where the path starts gets two line ends instead of a join.

![is_closed](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/is_closed.png)

### Measuring a path

[`PathMeasure`] finds the length of a path and the point, direction and
segment at any length along it. Arcs are measured as arcs, not as the
curves that approximate them:

![measure](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/measure.png)

```rust
use svg_path_ops::euclid::default::Point2D;
use svg_path_ops::pt::PathTransformer;

let path = PathTransformer::parse("M 10 0 A 10 10 0 0 1 -10 0")?;
let measure = path.measure();

// A half circle of radius 10
let length = measure.total_length();
assert!((length - 10.0 * std::f64::consts::PI).abs() < 1e-9);

let middle = measure.point_at(length / 2.0).unwrap();
assert!((middle - Point2D::new(0.0, 10.0)).length() < 1e-9);
```

Along with the point, it gives the tangent, the normal and the curvature
at any length. A curvature comb draws normals as long as the curvature,
showing where a path bends hard and where it turns the other way:

![curvature](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/curvature.png)

```rust
use svg_path_ops::pt::PathTransformer;

// Half an ellipse with radii 20 and 10, drawn clockwise on screen
let measure = PathTransformer::parse("M 20 0 A 20 10 0 0 1 -20 0")?.measure();

// Curvature is a / b² at the end of the long axis, b / a² at the short
assert!((measure.curvature_at(0.0).unwrap() - 0.2).abs() < 1e-9);
let middle = measure.total_length() / 2.0;
assert!((measure.curvature_at(middle).unwrap() - 0.025).abs() < 1e-9);
```

It also finds the point of the path nearest to another point, and
whether a point is on the path's stroke:

![nearest](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/nearest.png)

```rust
use svg_path_ops::euclid::default::Point2D;
use svg_path_ops::pt::PathTransformer;

let measure = PathTransformer::parse("M 10 0 A 10 10 0 0 1 -10 0")?.measure();

let nearest = measure.nearest(Point2D::new(0.0, 20.0)).unwrap();
assert!((nearest.distance - 10.0).abs() < 1e-9);
assert!(measure.is_point_in_stroke(Point2D::new(0.0, 11.0), 4.0));
```

And the area a path encloses, signed by the direction it is drawn in,
and whether a point is inside it under either [`FillRule`]:

![contains](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/contains.png)

```rust
use svg_path_ops::euclid::default::Point2D;
use svg_path_ops::pt::PathTransformer;
use svg_path_ops::FillRule;

// A square with a hole: the inner square runs the other way round
let frame = PathTransformer::parse("M 0 0 h 30 v 30 h -30 z M 10 10 v 10 h 10 v -10 z")?;
let measure = frame.measure();

assert_eq!(measure.area(), 800.0);
assert!(measure.contains(Point2D::new(5.0, 5.0), FillRule::NonZero));
assert!(!measure.contains(Point2D::new(15.0, 15.0), FillRule::NonZero));
```

It cuts out the part between two lengths, keeping arcs as arcs, splits
the path at a length, and turns it into straight lines within a
tolerance, for pen plotters and anything else that only draws lines:

![crop](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/crop.png)

```rust
use svg_path_ops::pt::PathTransformer;
use svg_path_ops::{write_path, PathSegment, WriteOptions};

let measure = PathTransformer::parse("M 0 0 h 10 A 5 5 0 0 1 10 10")?.measure();

let part = measure.crop(5.0, 12.0);
assert!(matches!(part[2], PathSegment::EllipticalArc { .. }));

let lines = measure.flatten(0.01);
assert!(lines
    .iter()
    .skip(1)
    .all(|segment| matches!(segment, PathSegment::LineTo { .. })));
```

### Simplifying

[`PathMeasure::simplify`] redraws a path of many short segments, as a
freehand stroke, a traced outline or flattened lines, with a few cubic
curves that stay within a tolerance of it. Joins that turn more than a
given angle stay corners, and straight stretches stay lines:

![simplify](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/simplify.png)

```rust
use svg_path_ops::pt::PathTransformer;

// A quarter circle drawn with 90 lines
let mut data = String::from("M 100 0");
for degree in 1..=90 {
    let angle = f64::from(degree).to_radians();
    data += &format!(" L {} {}", 100.0 * angle.cos(), 100.0 * angle.sin());
}
let simplified = PathTransformer::parse(&data)?.measure().simplify(0.5, 60.0);
// A move and one curve
assert_eq!(simplified.len(), 2);
```

The curves are fitted with Philip J. Schneider's algorithm, the one
Paper.js uses: control points along the path's direction, placed by
least squares, then refined with Newton's method. Each curve takes the
longest stretch of the path one curve can follow, so the result has few
curves, and neighbouring curves join smoothly.

### Smoothing

[`PathMeasure::smooth`] draws smooth curves through the points where a
path's segments meet, rounding a polygon or a path of lines. A
[`Smoothing::Continuous`] spline keeps the curvature continuous
everywhere; a [`Smoothing::CatmullRom`] spline shapes each curve from
its neighbouring points only. Joins that turn more than a given angle
stay corners:

![smooth](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/smooth.png)

```rust
use svg_path_ops::pt::PathTransformer;
use svg_path_ops::{PathSegment, Smoothing};

let zigzag = PathTransformer::parse("M 0 0 L 10 10 L 20 0 L 30 10")?;
let wave = zigzag
    .measure()
    .smooth(Smoothing::CatmullRom { alpha: 0.5 }, 180.0);

// Three curves through the same points
assert!(wave[1..]
    .iter()
    .all(|segment| matches!(segment, PathSegment::CurveTo { .. })));
assert!(matches!(
    wave[3],
    PathSegment::CurveTo { x: 30.0, y: 10.0, .. }
));
```

### Intersections

[`PathMeasure::intersections`] finds the points where two paths meet,
arcs included, and where each point lies on both paths:

![intersections](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/intersections.png)

```rust
use svg_path_ops::euclid::default::Point2D;
use svg_path_ops::pt::PathTransformer;

let circle = PathTransformer::parse("M 10 0 A 10 10 0 0 1 -10 0 A 10 10 0 0 1 10 0")?;
let line = PathTransformer::parse("M -20 6 H 20")?;

let meets = circle.measure().intersections(&line.measure());
assert_eq!(meets.len(), 2);
assert!((meets[0].point - Point2D::new(8.0, 6.0)).length() < 1e-9);
// Where the second point lies on the line, by length
assert!((meets[1].other.length - 12.0).abs() < 1e-9);
```

Each pair of segments is solved the way that suits it. Lines meet lines
in a linear system, and lines meet curves and arcs at the roots of a
polynomial. Two curves are cut in half again and again, dropping the
halves whose bounding boxes do not overlap, until the pieces around each
meeting point are a millionth of a unit wide; Newton's method on the
exact curves then makes each point accurate to about 1e-10. Where two
curves cross while running side by side, the pieces around the crossing
are gathered into one point. Segments that overlap along a stretch have
no single meeting point and are not reported.

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
[`reorient`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.reorient.html
[`join`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.join.html
[`PathMeasure::is_clockwise`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.is_clockwise
[`PathMeasure`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html
[`FillRule`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/enum.FillRule.html
[`PathMeasure::intersections`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.intersections
[`PathMeasure::simplify`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.simplify
[`PathMeasure::smooth`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.smooth
[`Smoothing::Continuous`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/enum.Smoothing.html#variant.Continuous
[`Smoothing::CatmullRom`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/enum.Smoothing.html#variant.CatmullRom
[`optimize`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.optimize.html
[`Shape`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/shapes/enum.Shape.html
[`flip_x`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.flip_x
[`flip_y`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.flip_y
[`split_subpaths`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.split_subpaths.html
[`is_closed`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.is_closed.html

<!-- cargo-sync-readme end -->

## 📝 License

Licensed under MIT License ([LICENSE](LICENSE)).

### 🚧 Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the MIT license, shall be licensed as above, without any additional terms or conditions.
