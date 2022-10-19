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
//!
#[macro_use]
extern crate derive_builder;

pub mod core;
pub mod filler;
pub mod generator;
pub mod geometry;
pub mod points_on_path;
pub mod renderer;

pub use palette::{Pixel, Srgb};
