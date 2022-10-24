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
On its own this crate can not draw on any context. One needs to use existing drawing libraries such as [piet](https://github.com/linebender/piet)
[raqote](https://github.com/jrmuizel/raqote) [tiny-skia](https://github.com/RazrFalcon/tiny-skia) etc in combination with
roughr. In this workspace an example adapter is implemented for [piet](https://github.com/linebender/piet). Below examples are
output of [rough_piet](https://github.com/orhanbalci/rough-rs/tree/main/rough_piet) adapter.

<!-- cargo-sync-readme end -->

## 📝 License

Licensed under MIT License ([LICENSE](LICENSE)).

### 🚧 Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the MIT license, shall be licensed as above, without any additional terms or conditions.
