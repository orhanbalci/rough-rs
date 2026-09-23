# rough-rs

![rustroughlogo](https://github.com/orhanbalci/rough-rs/blob/main/roughr/assets/rust.png?raw=true)

![roughtext](https://github.com/orhanbalci/rough-rs/blob/main/roughr/assets/rough_text.png?raw=true)

This repository contains a set of crates which in result resembles functionality in [Rough.js](https://github.com/rough-stuff/rough)

- [points_on_curve](https://github.com/orhanbalci/rough-rs/tree/main/points_on_curve) rustlang port of [points-on-curve](https://github.com/pshihn/bezier-points) npm package written by
[@pshihn](https://github.com/pshihn).

- [svg_path_ops](https://github.com/orhanbalci/rough-rs/tree/main/svg_path_ops) originates from [path-data-parser](https://github.com/pshihn/path-data-parser) but not limited to this
packages functionality

- [roughr](https://github.com/orhanbalci/rough-rs/tree/main/roughr) core implementation of [Rough.js](https://github.com/rough-stuff/rough) drawing primitives

- [rough_piet](https://github.com/orhanbalci/rough-rs/tree/main/rough_piet) adapter between [roughr](https://github.com/orhanbalci/rough-rs/tree/main/roughr) and [piet](https://github.com/linebender/piet)

- [rough_plotters_svg](https://github.com/orhanbalci/rough-rs/tree/main/rough_plotters_svg) adapter between [roughr](https://github.com/orhanbalci/rough-rs/tree/main/roughr) and [plotters-svg](https://github.com/plotters-rs/plotters)

- [rough_tiny_skia](https://github.com/orhanbalci/rough-rs/tree/main/rough_tiny_skia) adapter between [roughr](https://github.com/orhanbalci/rough-rs/tree/main/roughr) and [tiny-skia](https://github.com/RazrFalcon/tiny-skia)

- [rough_iced](https://github.com/orhanbalci/rough-rs/tree/main/rough_iced) adapter between [roughr](https://github.com/orhanbalci/rough-rs/tree/main/roughr) and [iced](https://github.com/iced-rs/iced)

- [rough_vello](https://github.com/orhanbalci/rough-rs/tree/main/rough_vello) adapter between [roughr](https://github.com/orhanbalci/rough-rs/tree/main/roughr) and [vello](https://github.com/linebender/vello)

## 🚀 Quick Start

`roughr` generates the sketchy shapes; an adapter crate draws them with your graphics
library. With [rough_tiny_skia](https://github.com/orhanbalci/rough-rs/tree/main/rough_tiny_skia),
drawing to a PNG looks like this:

```toml
[dependencies]
roughr = "0.13"
rough_tiny_skia = "0.13"
palette = "0.7"
tiny-skia = "0.11"
```

```rust
use palette::Srgba;
use rough_tiny_skia::SkiaGenerator;
use roughr::core::{FillStyle, OptionsBuilder};
use tiny_skia::Pixmap;

fn main() {
    let options = OptionsBuilder::default()
        .stroke(Srgba::from_components((114u8, 87u8, 82u8, 255u8)).into_format())
        .fill(Srgba::from_components((254u8, 246u8, 201u8, 255u8)).into_format())
        .fill_style(FillStyle::Hachure)
        .fill_weight(1.0)
        .roughness(1.5)
        .build()
        .unwrap();
    let generator = SkiaGenerator::new(options);

    let mut pixmap = Pixmap::new(300, 200).unwrap();
    pixmap.fill(tiny_skia::Color::from_rgba8(150, 192, 183, 255));
    generator.rectangle::<f32>(50.0, 50.0, 200.0, 100.0).draw(&mut pixmap.as_mut());
    generator.circle::<f32>(150.0, 100.0, 60.0).draw(&mut pixmap.as_mut());
    pixmap.save_png("sketch.png").unwrap();
}
```

The other adapters work the same way: build `Options`, create the adapter's generator,
then draw the shapes it returns.

## 🎛️ Options

`Options` controls how every shape is sketched. All fields have defaults, so set only what
you want to change. Each image draws the same shape with one option changed, from left to
right.

![roughness](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/roughness.png)

![fill_style](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/fill_style.png)

<details>
<summary><b>All options</b></summary>

![bowing](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/bowing.png)

![max_randomness_offset](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/max_randomness_offset.png)

![stroke_width](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/stroke_width.png)

![disable_multi_stroke](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/disable_multi_stroke.png)

![preserve_vertices](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/preserve_vertices.png)

![stroke_line_dash](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/stroke_line_dash.png)

![seed](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/seed.png)

![curve_fitting](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/curve_fitting.png)

![curve_step_count](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/curve_step_count.png)

![curve_tightness](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/curve_tightness.png)

![simplification](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/simplification.png)

![hachure_angle](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/hachure_angle.png)

![hachure_gap](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/hachure_gap.png)

![fill_weight](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/fill_weight.png)

![dash_offset](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/dash_offset.png)

![dash_gap](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/dash_gap.png)

![zigzag_offset](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/zigzag_offset.png)

![disable_multi_stroke_fill](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/options/disable_multi_stroke_fill.png)

</details>

Every option and its default is described in the
[`Options` docs](https://docs.rs/roughr/latest/roughr/core/struct.Options.html). To try them
interactively, run the [`drawing_app`](https://github.com/orhanbalci/rough-rs/tree/main/rough_iced/examples)
example: `cargo run -p rough_iced --example drawing_app`.

## 🧾 Drawing SVG Files

roughr can sketch SVG path data, and with [usvg](https://crates.io/crates/usvg) whole SVG
files, including shapes like `<rect>` and `<circle>`, transforms and colors:

![svg](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/rough_tiny_skia/assets/house.png)

```sh
cargo run -p rough_tiny_skia --example svg -- input.svg output.png
```

See the [`svg` example](https://github.com/orhanbalci/rough-rs/blob/main/rough_tiny_skia/examples/svg.rs)
and the [roughr docs](https://github.com/orhanbalci/rough-rs/tree/main/roughr#-drawing-svg-files)
for how it works.

## 📦 Crate Versions

The table below shows the current version of each crate and the versions of the
other workspace crates it depends on. A `—` means there is no dependency.

| Crate | Version | points_on_curve | svg_path_ops | roughr |
|-------|---------|:---------------:|:------------:|:------:|
| points_on_curve | 0.7.0 | — | — | — |
| svg_path_ops | 0.11.2 | — | — | — |
| roughr | 0.13.0 | 0.7.0 | 0.11.0 | — |
| rough_piet | 0.14.0 | — | 0.11.0 | 0.13.0 |
| rough_tiny_skia | 0.13.0 | — | — | 0.13.0 |
| rough_vello | 0.15.0 | — | 0.11.0 | 0.13.0 |
| rough_iced | 0.14.0 | — | 0.11.0 | 0.13.0 |
| rough_plotters_svg | 0.2.0 | — | — | 0.13.0 |

> Dependency requirements use Cargo's default caret semantics, so a requirement
> of `0.11.0` resolves to any `0.11.x` — the `svg_path_ops 0.11.1` bug-fix
> release is picked up automatically without republishing the dependent crates.

## 📝 License

Licensed under MIT License ([LICENSE](LICENSE)).

### 🚧 Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the MIT license, shall be licensed as above, without any additional terms or conditions.
