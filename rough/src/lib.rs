// This crate is entirely safe
#![forbid(unsafe_code)]
// Ensures that `pub` means published in the public API.
// This property is useful for reasoning about breaking API changes.
#![deny(unreachable_pub)]

#[macro_use]
extern crate derive_builder;

pub mod core;
pub mod filler;
pub mod generator;
pub mod geometry;
pub mod points_on_path;
pub mod renderer;
