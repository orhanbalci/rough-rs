// This crate is entirely safe
#![forbid(unsafe_code)]
// Ensures that `pub` means published in the public API.
// This property is useful for reasoning about breaking API changes.
#![deny(unreachable_pub)]

//!
//! This crate is a rustlang port of [Rough.js](https://github.com/rough-stuff/rough) npm package written by
//! [@pshihn](https://github.com/pshihn).
//!
//! This package exposes functions to generate rough drawing primitives which looks like hand drawn sketches.
//! This is the core create of operations to create rough drawings. It exposes its own primitive drawing types for lines
//! curves, arcs, polygons, circles, ellipses and even svg paths.
//! Works on [Point2D](https://docs.rs/euclid/0.22.7/euclid/struct.Point2D.html) type from [euclid](https://github.com/servo/euclid) crate
//! On its own this crate can not draw on any context. One needs to use existing drawing libraries such as [piet](https://github.com/linebender/piet)
//! [raqote](https://github.com/jrmuizel/raqote) [tiny-skia](https://github.com/RazrFalcon/tiny-skia) etc in combination with
//! roughr. In this workspace an example adapter is implemented for [piet](https://github.com/linebender/piet). Below examples are
//! output of [rough_piet](https://github.com/orhanbalci/rough-rs/tree/main/rough_piet) adapter.

//! ## 📦 Cargo.toml

//! ```toml
//! [dependencies]
//! roughr = "0.1"
//! ```

//! ## 🔧 Example

//! ```rust
//! use euclid::{default, point2};
//! use points_on_curve::points_on_bezier_curves;

//! let input = vec![
//!        point2(70.0,  240.0),
//!        point2(145.0,  60.0),
//!        point2(275.0,  90.0),
//!        point2(300.0,  230.0),
//! ];
//! let result_015 = points_on_bezier_curves(&input, 0.2, Some(0.15));

//! ```

//! ## 🖨️ Output
//! This picture shows computed points with 4 different distance values 0.15, 0.75, 1.5 and 3.0 with tolerance 2.0.

//! [tolerance](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/points_on_curve/assets/tolerance.png)

//! ## 🔭 Examples

//! For more examples have a look at the
//! [examples](https://github.com/orhanbalci/rough-rs/blob/main/points_on_curve/examples) folder.

#[macro_use]
extern crate derive_builder;

pub mod core;
pub mod filler;
pub mod generator;
pub mod geometry;
pub mod points_on_path;
pub mod renderer;

pub use palette::{Pixel, Srgb};
