// This crate is entirely safe
#![forbid(unsafe_code)]
// Ensures that `pub` means published in the public API.
// This property is useful for reasoning about breaking API changes.
#![deny(unreachable_pub)]

//!
//! This crate is an adapter crate between [roughr](https://github.com/orhanbalci/rough-rs/main/roughr) and
//! [piet](https://github.com/linebender/piet) crates. Converts from roughr drawing
//! primitives to piets path types. Also has convenience traits for drawing onto piet contexts. For more detailed
//! information you can check roughr crate.
//!
//! Below examples are output of [rough_piet](https://github.com/orhanbalci/rough-rs/tree/main/rough_piet) adapter.
//!
//! ## 📦 Cargo.toml
//!
//! ```toml
//! [dependencies]
//! rough_piet = "0.1"
//! ```
//!
//! ## 🔧 Example
//!
//! ### Rectangle
//!
//! ```ignore
//! let options = OptionsBuilder::default()
//!     .stroke(Srgb::from_raw(&[114u8, 87u8, 82u8]).into_format())
//!     .fill(Srgb::from_raw(&[254u8, 246u8, 201u8]).into_format())
//!     .fill_style(FillStyle::Hachure)
//!     .fill_weight(DPI * 0.01)
//!     .build()
//!     .unwrap();
//! let generator = KurboGenerator::new(options);
//! let rect_width = 100.0;
//! let rect_height = 50.0;
//! let rect = generator.rectangle::<f32>(
//!     (WIDTH as f32 - rect_width) / 2.0,
//!     (HEIGHT as f32 - rect_height) / 2.0,
//!     rect_width,
//!     rect_height,
//! );
//! let background_color = Color::from_hex_str("96C0B7").unwrap();
//!
//! rc.fill(
//!     Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
//!     &background_color,
//! );
//! rect.draw(&mut rc);
//! ```
//!
//! ### 🖨️ Output Rectangle
//! ![rectangle](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/rectangle.png)
//!
//! ### Circle
//!
//! ```ignore
//! let options = OptionsBuilder::default()
//!     .stroke(Srgb::from_raw(&[114u8, 87u8, 82u8]).into_format())
//!     .fill(Srgb::from_raw(&[254u8, 246u8, 201u8]).into_format())
//!     .fill_style(FillStyle::Hachure)
//!     .fill_weight(DPI * 0.01)
//!     .build()
//!     .unwrap();
//! let generator = KurboGenerator::new(options);
//! let circle_paths = generator.circle::<f32>(
//!     (WIDTH as f32) / 2.0,
//!     (HEIGHT as f32) / 2.0,
//!     HEIGHT as f32 - 10.0f32,
//! );
//! let background_color = Color::from_hex_str("96C0B7").unwrap();
//!
//! rc.fill(
//!     Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
//!     &background_color,
//! );
//! circle_paths.draw(&mut rc);
//! ```
//!
//! ### 🖨️ Output Circle
//! ![circle](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/circle.png)
//!
//!
//! ### Ellipse
//!
//! ```ignore
//! let options = OptionsBuilder::default()
//!     .stroke(Srgb::from_raw(&[114u8, 87u8, 82u8]).into_format())
//!     .fill(Srgb::from_raw(&[254u8, 246u8, 201u8]).into_format())
//!     .fill_style(FillStyle::Hachure)
//!     .fill_weight(DPI * 0.01)
//!     .build()
//!     .unwrap();
//! let generator = KurboGenerator::new(options);
//! let ellipse_paths = generator.ellipse::<f32>(
//!     (WIDTH as f32) / 2.0,
//!     (HEIGHT as f32) / 2.0,
//!     WIDTH as f32 - 10.0,
//!     HEIGHT as f32 - 10.0,
//! );
//! let background_color = Color::from_hex_str("96C0B7").unwrap();
//!
//! rc.fill(
//!     Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
//!     &background_color,
//! );
//! ellipse_paths.draw(&mut rc);
//! ```
//!
//! ### 🖨️ Output Ellipse
//! ![ellipse](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/ellipse.png)
//!
//!
//! ### Svg Path
//!
//! ```ignore
//! let options = OptionsBuilder::default()
//!     .stroke(Srgb::from_raw(&[114u8, 87u8, 82u8]).into_format())
//!     .fill(Srgb::from_raw(&[254u8, 246u8, 201u8]).into_format())
//!     .fill_style(FillStyle::Hachure)
//!     .fill_weight(DPI * 0.01)
//!     .build()
//!     .unwrap();
//! let generator = KurboGenerator::new(options);
//! let heart_svg_path  = "M140 20C73 20 20 74 20 140c0 135 136 170 228 303 88-132 229-173 229-303 0-66-54-120-120-120-48 0-90 28-109 69-19-41-60-69-108-69z".into();
//! let heart_svg_path_drawing = generator.path::<f32>(heart_svg_path);
//! let background_color = Color::from_hex_str("96C0B7").unwrap();
//!
//! rc.fill(
//!     Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
//!     &background_color,
//! );
//! heart_svg_path_drawing.draw(&mut rc);
//! ```
//!
//! ### 🖨️ Output Svg Path
//! ![svgheart](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/roughr/assets/heart_svg_path.png)
//!
//! ## Filler Implementation Status
//! - [x] Hachure
//! - [ ] Zigzag
//! - [ ] Cross-Hatch
//! - [ ] Dots
//! - [x] Dashed
//! - [ ] Zigzag-Line
//!
//! ## 🔭 Examples
//!
//! For more examples have a look at the
//! [examples](https://github.com/orhanbalci/rough-rs/tree/main/rough_piet/examples) folder.

pub mod kurbo_generator;
pub use kurbo_generator::*;
