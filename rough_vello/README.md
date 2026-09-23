# rough_vello

[![Crates.io](https://img.shields.io/crates/v/rough_vello.svg)](https://crates.io/crates/rough_vello)
[![Documentation](https://docs.rs/rough_vello/badge.svg)](https://docs.rs/rough_vello)
[![License](https://img.shields.io/github/license/orhanbalci/rough-rs.svg)](https://github.com/orhanbalci/rough-rs/LICENSE)

<!-- cargo-sync-readme start -->


This crate is an adapter crate between [roughr](https://github.com/orhanbalci/rough-rs/main/roughr) and
[vello](https://github.com/linebender/vello) crates. Converts from roughr drawing
primitives to vello's Scene types. Also has convenience traits for drawing onto vello scenes. For more detailed
information you can check roughr crate.

Below examples are output of [rough_vello](https://github.com/orhanbalci/rough-rs/tree/main/rough_vello) adapter.

## 📦 Cargo.toml

```toml
[dependencies]
rough_vello = "0.15"
```

## 🧩 Vello Compatibility

`rough_vello` exposes `vello::Scene` in its public API, so your project must use the same
vello version as `rough_vello`.

| rough_vello        | vello |
|--------------------|-------|
| 0.15               | 0.10  |
| 0.14               | 0.5   |
| 0.13               | 0.5   |
| 0.1                | 0.5   |

## 🔧 Example

### Rust Logo

```rust
use rough_vello::VelloGenerator;
use vello::Scene;
use palette::Srgba;
use roughr::core::{FillStyle, OptionsBuilder};

let options = OptionsBuilder::default()
    .stroke(Srgba::from_components((114u8, 87u8, 82u8, 255u8)).into_format())
    .fill(Srgba::from_components((254u8, 246u8, 201u8, 255)).into_format())
    .fill_style(FillStyle::Hachure)
    .fill_weight(1.0)
    .bowing(0.8)
    .build()
    .unwrap();

let generator = VelloGenerator::new(options);
let rust_logo_svg_path = "..."; // SVG path data for the Rust logo
let rust_logo_drawing = generator.path::<f32>(rust_logo_svg_path);

let mut scene = Scene::new();
rust_logo_drawing.draw(&mut scene);
```

### 🖨️ Output Rust Logo
![rust_logo](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/rough_vello/assets/rust_logo.png)

## Filler Implementation Status
- [x] Hachure
- [x] Zigzag
- [x] Cross-Hatch
- [x] Dots
- [x] Dashed
- [x] Zigzag-Line

## 🔭 Examples

For more examples have a look at the
[examples](https://github.com/orhanbalci/rough-rs/tree/main/rough_vello/examples) folder.

## 🔌 Integration

### Windowing

`rough_vello` only produces `vello::Scene`s, so it works with any setup that can render
vello scenes. The examples open a plain [winit](https://github.com/rust-windowing/winit)
window and render with `vello::util::RenderContext`; see `examples/common/mod.rs`.

### Bevy Integration

For Bevy, [bevy_vello](https://github.com/linebender/bevy_vello) can render vello scenes.
Pick a `bevy_vello` release that depends on the same vello version as `rough_vello`
(currently vello 0.10), otherwise the `Scene` types will not match.

<!-- cargo-sync-readme end -->

## 📝 License

Licensed under MIT License ([LICENSE](LICENSE)).

### 🚧 Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the MIT license, shall be licensed as above, without any additional terms or conditions.
