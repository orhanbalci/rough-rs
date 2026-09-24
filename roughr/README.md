# roughr

[![Crates.io](https://img.shields.io/crates/v/roughr.svg)](https://crates.io/crates/roughr)
[![Documentation](https://docs.rs/roughr/badge.svg)](https://docs.rs/roughr)
[![License](https://img.shields.io/github/license/orhanbalci/rough-rs.svg)](https://github.com/orhanbalci/rough-rs/LICENSE)

<!-- cargo-sync-readme start -->


This crate is a rustlang port of [Rough.js](https://github.com/rough-stuff/rough) npm package written by
[@pshihn](https://github.com/pshihn).

This package exposes functions to generate rough drawing primitives which looks like hand drawn sketches.
This is the core create of operations to create rough drawings. It exposes its own primitive drawing types for lines
curves, arcs, polygons, circles, ellipses and even svg paths.
Works on [Point2D](https://docs.rs/euclid/0.22.7/euclid/struct.Point2D.html) type from [euclid](https://github.com/servo/euclid) crate

On its own this crate can not draw on any context. One needs to use existing drawing libraries such as [piet](https://github.com/linebender/piet),
[raqote](https://github.com/jrmuizel/raqote), [tiny-skia](https://github.com/RazrFalcon/tiny-skia) etc in combination with
roughr. In this workspace an example adapter is implemented for [piet](https://github.com/linebender/piet). Below examples are
output of [rough_piet](https://github.com/orhanbalci/rough-rs/tree/roughr@0.14.0/rough_piet) adapter.

## 📦 Cargo.toml

```toml
[dependencies]
roughr = "0.14"
```

## 🔧 Example

### Rectangle

```rust
let options = OptionsBuilder::default()
    .stroke(Srgba::from_raw(&[114u8, 87u8, 82u8, 255u8]).into_format())
    .fill(Srgba::from_raw(&[254u8, 246u8, 201u8, 255u8]).into_format())
    .fill_style(FillStyle::Hachure)
    .fill_weight(DPI * 0.01)
    .build()
    .unwrap();
let generator = KurboGenerator::new(options);
let rect_width = 100.0;
let rect_height = 50.0;
let rect = generator.rectangle::<f32>(
    (WIDTH as f32 - rect_width) / 2.0,
    (HEIGHT as f32 - rect_height) / 2.0,
    rect_width,
    rect_height,
);
let background_color = Color::from_hex_str("96C0B7").unwrap();

rc.fill(
    Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
    &background_color,
);
rect.draw(&mut rc);
```

### 🖨️ Output Rectangle
![rectangle](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/rectangle.png)

### Circle

```rust
let options = OptionsBuilder::default()
    .stroke(Srgba::from_raw(&[114u8, 87u8, 82u8, 255u8]).into_format())
    .fill(Srgba::from_raw(&[254u8, 246u8, 201u8, 255u8]).into_format())
    .fill_style(FillStyle::Hachure)
    .fill_weight(DPI * 0.01)
    .build()
    .unwrap();
let generator = KurboGenerator::new(options);
let circle_paths = generator.circle::<f32>(
    (WIDTH as f32) / 2.0,
    (HEIGHT as f32) / 2.0,
    HEIGHT as f32 - 10.0f32,
);
let background_color = Color::from_hex_str("96C0B7").unwrap();

rc.fill(
    Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
    &background_color,
);
circle_paths.draw(&mut rc);
```

### 🖨️ Output Circle
![circle](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/circle.png)


### Ellipse

```rust
let options = OptionsBuilder::default()
    .stroke(Srgba::from_raw(&[114u8, 87u8, 82u8, 255u8]).into_format())
    .fill(Srgba::from_raw(&[254u8, 246u8, 201u8, 255u8]).into_format())
    .fill_style(FillStyle::Hachure)
    .fill_weight(DPI * 0.01)
    .build()
    .unwrap();
let generator = KurboGenerator::new(options);
let ellipse_paths = generator.ellipse::<f32>(
    (WIDTH as f32) / 2.0,
    (HEIGHT as f32) / 2.0,
    WIDTH as f32 - 10.0,
    HEIGHT as f32 - 10.0,
);
let background_color = Color::from_hex_str("96C0B7").unwrap();

rc.fill(
    Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
    &background_color,
);
ellipse_paths.draw(&mut rc);
```

### 🖨️ Output Ellipse
![ellipse](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/ellipse.png)


### Svg Path

```rust
let options = OptionsBuilder::default()
    .stroke(Srgba::from_raw(&[114u8, 87u8, 82u8, 255u8]).into_format())
    .fill(Srgba::from_raw(&[254u8, 246u8, 201u8, 255u8]).into_format())
    .fill_style(FillStyle::Hachure)
    .fill_weight(DPI * 0.01)
    .build()
    .unwrap();
let generator = KurboGenerator::new(options);
let heart_svg_path  = "M140 20C73 20 20 74 20 140c0 135 136 170 228 303 88-132 229-173 229-303 0-66-54-120-120-120-48 0-90 28-109 69-19-41-60-69-108-69z".into();
let heart_svg_path_drawing = generator.path::<f32>(heart_svg_path);
let background_color = Color::from_hex_str("96C0B7").unwrap();

rc.fill(
    Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
    &background_color,
);
heart_svg_path_drawing.draw(&mut rc);
```

### 🖨️ Output Svg Path
![svgheart](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/heart_svg_path.png)

## Filler Implementation Status
- [x] Hachure
- [x] Zigzag
- [x] Cross-Hatch
- [x] Dots
- [x] Dashed
- [x] Zigzag-Line

## 🎛️ Options

Every drawing is controlled by an `Options` value, built with `OptionsBuilder`. All fields
have defaults, so set only what you want to change. The most useful ones are `roughness`,
`bowing`, `stroke`, `stroke_width`, `fill`, `fill_style`, `hachure_gap` and `seed`.

Each image below draws the same shape with only one option changed, from left to right.
The field docs of [`Options`](https://docs.rs/roughr/latest/roughr/core/struct.Options.html)
explain every option and its default. To try options interactively, run the
[`drawing_app`](https://github.com/orhanbalci/rough-rs/tree/roughr@0.14.0/rough_iced/examples) example
of `rough_iced`.

<details>
<summary><b>Outline</b></summary>

![roughness](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/roughness.png)

![bowing](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/bowing.png)

![max_randomness_offset](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/max_randomness_offset.png)

![stroke_width](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/stroke_width.png)

![disable_multi_stroke](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/disable_multi_stroke.png)

![preserve_vertices](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/preserve_vertices.png)

![stroke_line_dash](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/stroke_line_dash.png)

![seed](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/seed.png)

</details>

<details>
<summary><b>Curves and SVG paths</b></summary>

![curve_fitting](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/curve_fitting.png)

![curve_step_count](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/curve_step_count.png)

![curve_tightness](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/curve_tightness.png)

![simplification](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/simplification.png)

</details>

<details>
<summary><b>Fill</b></summary>

![fill_style](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/fill_style.png)

![hachure_angle](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/hachure_angle.png)

![hachure_gap](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/hachure_gap.png)

![fill_weight](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/fill_weight.png)

![dash_offset](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/dash_offset.png)

![dash_gap](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/dash_gap.png)

![zigzag_offset](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/zigzag_offset.png)

![disable_multi_stroke_fill](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/roughr/assets/options/disable_multi_stroke_fill.png)

</details>

## 🧾 Drawing SVG files

`path` takes SVG path data, the `d` attribute of a `<path>` element. To sketch a whole SVG
file, which may also contain shapes like `<rect>` and `<circle>`, transforms and styles,
first convert it to plain paths with [usvg](https://crates.io/crates/usvg), then draw every
path with its own colors:

```ignore
let tree = usvg::Tree::from_data(&svg_bytes, &usvg::Options::default())?;
// For each usvg::Path in the tree:
let data = path.data().clone().transform(path.abs_transform()).unwrap();
let options = OptionsBuilder::default()
    .fill(/* the path's fill color */)
    .stroke(/* the path's stroke color */)
    .build()?;
let drawing = SkiaGenerator::new(options).path::<f32>(to_svg_path_data(&data));
drawing.draw(&mut pixmap.as_mut());
```

The complete, runnable version is the
[`svg` example](https://github.com/orhanbalci/rough-rs/blob/roughr@0.14.0/rough_tiny_skia/examples/svg.rs)
of `rough_tiny_skia`:

```sh
cargo run -p rough_tiny_skia --example svg -- input.svg output.png
```

![svg](https://raw.githubusercontent.com/orhanbalci/rough-rs/roughr@0.14.0/rough_tiny_skia/assets/house.png)

## 🔭 Examples

For more examples have a look at the
[examples](https://github.com/orhanbalci/rough-rs/tree/roughr@0.14.0/rough_piet/examples) folder.

<!-- cargo-sync-readme end -->

## 📝 License

Licensed under MIT License ([LICENSE](LICENSE)).

### 🚧 Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the MIT license, shall be licensed as above, without any additional terms or conditions.
