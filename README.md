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

## 📦 Crate Versions

The table below shows the current version of each crate and the versions of the
other workspace crates it depends on. A `—` means there is no dependency.

| Crate | Version | points_on_curve | svg_path_ops | roughr |
|-------|---------|:---------------:|:------------:|:------:|
| points_on_curve | 0.7.0 | — | — | — |
| svg_path_ops | 0.11.1 | — | — | — |
| roughr | 0.12.0 | 0.7.0 | 0.11.0 | — |
| rough_piet | 0.13.0 | — | 0.11.0 | 0.12.0 |
| rough_tiny_skia | 0.12.0 | — | — | 0.12.0 |
| rough_vello | 0.13.0 | — | 0.11.0 | 0.12.0 |
| rough_iced | 0.13.0 | — | 0.11.0 | 0.12.0 |
| rough_plotters_svg | 0.1.0 | — | — | 0.12.0 |

> Dependency requirements use Cargo's default caret semantics, so a requirement
> of `0.11.0` resolves to any `0.11.x` — the `svg_path_ops 0.11.1` bug-fix
> release is picked up automatically without republishing the dependent crates.

## 📝 License

Licensed under MIT License ([LICENSE](LICENSE)).

### 🚧 Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the MIT license, shall be licensed as above, without any additional terms or conditions.
