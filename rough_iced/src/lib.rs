// This crate is entirely safe
#![forbid(unsafe_code)]
// Ensures that `pub` means published in the public API.
// This property is useful for reasoning about breaking API changes.
#![deny(unreachable_pub)]

//!
//! This crate is an adapter crate between [roughr](https://github.com/orhanbalci/rough-rs/tree/main/roughr) and
//! [iced](https://github.com/iced-rs/iced) crates. Converts from roughr drawing
//! primitives to iced path types. Also has convenience traits for drawing onto iced frames. For more detailed
//! information you can check roughr crate.
//!
//! Below examples are output of [rough_iced](https://github.com/orhanbalci/rough-rs/tree/main/rough_iced) adapter.
//!
//! ## 📦 Cargo.toml
//!
//! ```toml
//! [dependencies]
//! rough_iced = "0.1"
//! ```
//!
//! ## 🔧 Configuration Tool
//! ![rectangle](https://raw.githubusercontent.com/orhanbalci/rough-rs/refs/heads/main/rough_iced/assets/conf.png)
//!
//! ## 🔭 Examples
//!
//! For more examples have a look at the
//! [examples](https://github.com/orhanbalci/rough-rs/tree/main/rough_iced/examples) folder.

pub mod iced_generator;
pub use iced_generator::*;
