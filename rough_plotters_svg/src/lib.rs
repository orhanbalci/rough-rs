//! # Rough Plotters SVG
//!
//! A rough/sketchy style wrapper around the `plotters-svg` backend for the [Plotters](https://github.com/plotters-rs/plotters) 
//! plotting library. This crate provides a `DrawingBackend` implementation that intercepts drawing calls and applies 
//! rough, hand-drawn style transformations to geometric primitives like lines, rectangles, circles, and paths.
//!
//! ## Features
//!
//! - **Drop-in replacement** for `plotters-svg::SVGBackend`
//! - **Rough styling** applied to all geometric primitives
//! - **Configurable roughness** with full control over rough options
//! - **Multiple fill styles** including hachure, cross-hatch, zigzag, and more
//! - **Compatible** with the entire Plotters ecosystem
//!
//! ## Quick Start
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rough_plotters_svg = "0.0.0"
//! plotters = "0.3"
//! roughr = "0.13"
//! ```
//!
//! ### Basic Usage
//!
//! ```rust
//! use rough_plotters_svg::RoughSVGBackend;
//! use plotters::prelude::*;
//! use roughr::core::{FillStyle, Options};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Configure rough styling
//!     let mut options = Options::default();
//!     options.fill_style = Some(FillStyle::Hachure);
//!     options.roughness = Some(2.0);
//!     options.stroke_width = Some(1.5);
//!
//!     // Create backend with rough styling
//!     let backend = RoughSVGBackend::with_options("chart.svg", (800, 600), options);
//!     let root = backend.into_drawing_area();
//!     root.fill(&WHITE)?;
//!
//!     // Use with plotters as normal - all shapes will be rough styled
//!     let mut chart = ChartBuilder::on(&root)
//!         .caption("Rough Chart", ("serif", 40))
//!         .margin(20)
//!         .x_label_area_size(40)
//!         .y_label_area_size(50)
//!         .build_cartesian_2d(0f32..10f32, 0f32..100f32)?;
//!
//!     chart.configure_mesh().draw()?;
//!
//!     chart.draw_series(
//!         (0..10).map(|x| Rectangle::new([(x as f32, 0f32), (x as f32 + 0.8, x as f32 * 10f32)], RED.filled()))
//!     )?;
//!
//!     root.present()?;
//!     Ok(())
//! }
//! ```
//!
//! ![Basic Chart Example](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/rough_plotters_svg/assets/chart.svg)
//!
//! ### Fill Styles Showcase
//!
//! ![Fill Styles Showcase](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/rough_plotters_svg/assets/showcase.svg)
//!
//! ### Working with Fill Styles
//!
//! ```rust
//! use rough_plotters_svg::RoughSVGBackend;
//! use plotters::prelude::*;
//! use roughr::core::{FillStyle, Options};
//!
//! fn create_chart_with_fill_style(fill_style: FillStyle, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
//!     let mut options = Options::default();
//!     options.fill_style = Some(fill_style);
//!     options.stroke_width = Some(2.0);
//!     options.roughness = Some(1.0);
//!     
//!     let backend = RoughSVGBackend::with_options(filename, (400, 300), options);
//!     let root = backend.into_drawing_area();
//!     root.fill(&RGBColor(254, 246, 201))?; // Cream background
//!
//!     let mut chart = ChartBuilder::on(&root)
//!         .caption(&format!("Fill Style: {:?}", fill_style), ("Arial", 20))
//!         .margin(10)
//!         .build_cartesian_2d(0..10, 0..100)?;
//!
//!     chart.configure_mesh().draw()?;
//!
//!     // Draw some shapes with the fill style
//!     chart.draw_series(
//!         [(2, 30), (4, 50), (6, 70), (8, 90)]
//!             .iter()
//!             .map(|(x, y)| Rectangle::new([(*x, 0), (*x + 1, *y)], BLUE.filled()))
//!     )?;
//!
//!     chart.draw_series(
//!         std::iter::once(Circle::new((5, 50), 15, RED.filled()))
//!     )?;
//!
//!     root.present()?;
//!     Ok(())
//! }
//! ```
//!
//! ![Fill Style Example](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/rough_plotters_svg/assets/hachure.svg)
//!
//! ## Available Fill Styles
//!
//! - `FillStyle::Solid` - Solid color fill
//! - `FillStyle::Hachure` - Parallel line hatching (default)
//! - `FillStyle::ZigZag` - Zigzag pattern fill
//! - `FillStyle::CrossHatch` - Cross-hatched pattern
//! - `FillStyle::Dots` - Dotted pattern fill
//! - `FillStyle::Dashed` - Dashed line pattern
//! - `FillStyle::ZigZagLine` - Zigzag line pattern
//!
//! ![CrossHatch Fill Style](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/rough_plotters_svg/assets/crosshatch.svg)
//!
//! ### Stock Chart Example
//!
//! Rough styling works great with financial charts:
//!
//! ![Stock Chart Example](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/rough_plotters_svg/assets/stock.svg)
//!
//! ## String-based Backend
//!
//! For in-memory SVG generation:
//!
//! ```rust
//! use rough_plotters_svg::RoughSVGBackend;
//! use roughr::core::Options;
//!
//! let mut svg_string = String::new();
//! let backend = RoughSVGBackend::with_string_and_options(&mut svg_string, (640, 480), Options::default());
//! // ... use backend normally ...
//! // svg_string now contains the generated SVG
//! ```

use euclid::default::Point2D;
use palette::Srgba;
use plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
};
use plotters_svg::SVGBackend;
use roughr::core::{Options, FillStyle};
use roughr::generator::Generator;
use std::error::Error;
use std::fmt;
use std::path::Path;

/// A rough wrapper around plotters-svg that applies sketchy styling to geometric primitives
pub struct RoughSVGBackend<'a> {
    inner: SVGBackend<'a>,
    rough_generator: Generator,
    rough_options: Options,
}

/// Error type for RoughSVGBackend
#[derive(Debug)]
pub struct RoughSVGError {
    inner: Box<dyn Error + Send + Sync>,
}

impl fmt::Display for RoughSVGError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RoughSVG error: {}", self.inner)
    }
}

impl Error for RoughSVGError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.inner.as_ref())
    }
}

impl<'a> RoughSVGBackend<'a> {
    /// Create a new RoughSVGBackend with default rough options
    pub fn new<T: AsRef<Path> + ?Sized>(path: &'a T, size: (u32, u32)) -> Self {
        Self::with_options(path, size, Options::default())
    }

    /// Create a new RoughSVGBackend with custom rough options
    pub fn with_options<T: AsRef<Path> + ?Sized>(path: &'a T, size: (u32, u32), options: Options) -> Self {
        Self {
            inner: SVGBackend::new(path, size),
            rough_generator: Generator::default(),
            rough_options: options,
        }
    }

    /// Create a new RoughSVGBackend that writes to a String buffer
    pub fn with_string(buf: &'a mut String, size: (u32, u32)) -> Self {
        Self::with_string_and_options(buf, size, Options::default())
    }

    /// Create a new RoughSVGBackend that writes to a String buffer with custom rough options
    pub fn with_string_and_options(buf: &'a mut String, size: (u32, u32), options: Options) -> Self {
        Self {
            inner: SVGBackend::with_string(buf, size),
            rough_generator: Generator::default(),
            rough_options: options,
        }
    }

    /// Get a mutable reference to the rough options
    pub fn rough_options_mut(&mut self) -> &mut Options {
        &mut self.rough_options
    }

    /// Convert a BackendCoord to a Point2D for rough generation
    fn coord_to_point(coord: BackendCoord) -> Point2D<f64> {
        Point2D::new(coord.0 as f64, coord.1 as f64)
    }

    /// Convert a BackendColor to an Srgba color for rough styling
    fn backend_color_to_srgba(color: &BackendColor) -> Srgba {
        Srgba::new(
            color.rgb.0 as f32 / 255.0,
            color.rgb.1 as f32 / 255.0,
            color.rgb.2 as f32 / 255.0,
            color.alpha as f32,
        )
    }

    /// Apply rough styling to line drawing
    fn draw_rough_line(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &impl BackendStyle,
    ) -> Result<(), DrawingErrorKind<RoughSVGError>> {
        let from_point = Self::coord_to_point(from);
        let to_point = Self::coord_to_point(to);
        
        // Create rough options from style
        let mut rough_opts = self.rough_options.clone();
        rough_opts.stroke = Some(Self::backend_color_to_srgba(&style.color()));
        rough_opts.stroke_width = Some(style.stroke_width() as f32);

        // Generate rough line
        let drawable = self.rough_generator.line(
            from_point.x,
            from_point.y,
            to_point.x,
            to_point.y,
            &Some(rough_opts),
        );

        // Convert to SVG path and draw using inner backend
        self.draw_rough_drawable(&drawable, style)
    }

    /// Apply rough styling to rectangle drawing
    fn draw_rough_rect(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &impl BackendStyle,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<RoughSVGError>> {
        let width = (bottom_right.0 - upper_left.0) as f64;
        let height = (bottom_right.1 - upper_left.1) as f64;
        let canvas_size = self.get_size();
        
        // Check if this is a full canvas background fill
        let is_background_fill = fill && 
            upper_left.0 == 0 && upper_left.1 == 0 &&
            bottom_right.0 >= canvas_size.0 as i32 && 
            bottom_right.1 >= canvas_size.1 as i32;
        
        // For background fills, use the inner backend directly for solid color
        if is_background_fill {
            return self.inner
                .draw_rect(upper_left, bottom_right, style, fill)
                .map_err(|e| match e {
                    DrawingErrorKind::DrawingError(err) => DrawingErrorKind::DrawingError(RoughSVGError { inner: Box::new(err) }),
                    DrawingErrorKind::FontError(err) => DrawingErrorKind::FontError(err),
                });
        }
        
        let mut rough_opts = self.rough_options.clone();
        rough_opts.stroke = Some(Self::backend_color_to_srgba(&style.color()));
        rough_opts.stroke_width = Some(style.stroke_width() as f32);
        
        if fill {
            rough_opts.fill = Some(Self::backend_color_to_srgba(&style.color()));
            // Use the configured fill style from rough options, or default to Hachure
            if rough_opts.fill_style.is_none() {
                rough_opts.fill_style = Some(FillStyle::Hachure);
            }
        }

        let drawable = self.rough_generator.rectangle(
            upper_left.0 as f64,
            upper_left.1 as f64,
            width,
            height,
            &Some(rough_opts),
        );

        self.draw_rough_drawable(&drawable, style)
    }

    /// Apply rough styling to circle drawing
    fn draw_rough_circle(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &impl BackendStyle,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<RoughSVGError>> {
        let mut rough_opts = self.rough_options.clone();
        rough_opts.stroke = Some(Self::backend_color_to_srgba(&style.color()));
        rough_opts.stroke_width = Some(style.stroke_width() as f32);
        
        if fill {
            rough_opts.fill = Some(Self::backend_color_to_srgba(&style.color()));
            // Use the configured fill style from rough options, or default to Hachure
            if rough_opts.fill_style.is_none() {
                rough_opts.fill_style = Some(FillStyle::Hachure);
            }
        }

        let drawable = self.rough_generator.circle(
            center.0 as f64,
            center.1 as f64,
            radius as f64 * 2.0, // diameter
            &Some(rough_opts),
        );

        self.draw_rough_drawable(&drawable, style)
    }

    /// Apply rough styling to path drawing
    fn draw_rough_path<I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &impl BackendStyle,
    ) -> Result<(), DrawingErrorKind<RoughSVGError>> {
        let points: Vec<Point2D<f64>> = path
            .into_iter()
            .map(Self::coord_to_point)
            .collect();

        if points.len() < 2 {
            return Ok(());
        }

        let mut rough_opts = self.rough_options.clone();
        rough_opts.stroke = Some(Self::backend_color_to_srgba(&style.color()));
        rough_opts.stroke_width = Some(style.stroke_width() as f32);

        let drawable = self.rough_generator.linear_path(&points, false, &Some(rough_opts));

        self.draw_rough_drawable(&drawable, style)
    }

    /// Apply rough styling to polygon filling
    fn fill_rough_polygon<I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &impl BackendStyle,
    ) -> Result<(), DrawingErrorKind<RoughSVGError>> {
        let points: Vec<Point2D<f64>> = vert
            .into_iter()
            .map(Self::coord_to_point)
            .collect();

        if points.len() < 3 {
            return Ok(());
        }

        let mut rough_opts = self.rough_options.clone();
        rough_opts.fill = Some(Self::backend_color_to_srgba(&style.color()));
        // Use the configured fill style from rough options, or default to Hachure
        if rough_opts.fill_style.is_none() {
            rough_opts.fill_style = Some(FillStyle::Hachure);
        }

        let drawable = self.rough_generator.polygon(&points, &Some(rough_opts));

        self.draw_rough_drawable(&drawable, style)
    }

    /// Convert rough drawable to SVG and draw using inner backend
    fn draw_rough_drawable(
        &mut self,
        drawable: &roughr::core::Drawable<f64>,
        style: &impl BackendStyle,
    ) -> Result<(), DrawingErrorKind<RoughSVGError>> {
        // Convert rough drawable to plotters-compatible drawing operations
        for set in &drawable.sets {
            match set.op_set_type {
                roughr::core::OpSetType::Path => {
                    // Draw stroke paths
                    self.draw_opset_as_path(set, style)?;
                }
                roughr::core::OpSetType::FillPath => {
                    // Draw filled areas
                    self.draw_opset_as_filled_path(set, style)?;
                }
                roughr::core::OpSetType::FillSketch => {
                    // Draw fill hatching/sketching
                    self.draw_opset_as_path(set, style)?;
                }
            }
        }
        Ok(())
    }

    /// Draw an OpSet as a path using the inner backend
    fn draw_opset_as_path(
        &mut self,
        opset: &roughr::core::OpSet<f64>,
        style: &impl BackendStyle,
    ) -> Result<(), DrawingErrorKind<RoughSVGError>> {
        let mut path_points = Vec::new();
        let mut current_pos = (0i32, 0i32);

        for op in &opset.ops {
            match op.op {
                roughr::core::OpType::Move => {
                    if !path_points.is_empty() {
                        // Draw the current path segment
                        self.draw_path_segment(&path_points, style)?;
                        path_points.clear();
                    }
                    current_pos = (op.data[0] as i32, op.data[1] as i32);
                    path_points.push(current_pos);
                }
                roughr::core::OpType::LineTo => {
                    current_pos = (op.data[0] as i32, op.data[1] as i32);
                    path_points.push(current_pos);
                }
                roughr::core::OpType::BCurveTo => {
                    // For Bezier curves, we'll approximate with line segments
                    // In a full implementation, you'd want proper curve support
                    let end_point = (op.data[4] as i32, op.data[5] as i32);
                    path_points.push(end_point);
                    current_pos = end_point;
                }
            }
        }

        // Draw any remaining path
        if !path_points.is_empty() {
            self.draw_path_segment(&path_points, style)?;
        }

        Ok(())
    }

    /// Draw an OpSet as a filled path using the inner backend
    fn draw_opset_as_filled_path(
        &mut self,
        opset: &roughr::core::OpSet<f64>,
        style: &impl BackendStyle,
    ) -> Result<(), DrawingErrorKind<RoughSVGError>> {
        let mut path_points = Vec::new();

        for op in &opset.ops {
            match op.op {
                roughr::core::OpType::Move => {
                    path_points.push((op.data[0] as i32, op.data[1] as i32));
                }
                roughr::core::OpType::LineTo => {
                    path_points.push((op.data[0] as i32, op.data[1] as i32));
                }
                roughr::core::OpType::BCurveTo => {
                    // Approximate bezier curve with end point
                    path_points.push((op.data[4] as i32, op.data[5] as i32));
                }
            }
        }

        if path_points.len() >= 3 {
            // Use the style color for the fill
            let fill_color = style.color();
            self.inner
                .fill_polygon(path_points, &fill_color)
                .map_err(|e| match e {
                    DrawingErrorKind::DrawingError(err) => DrawingErrorKind::DrawingError(RoughSVGError { inner: Box::new(err) }),
                    DrawingErrorKind::FontError(err) => DrawingErrorKind::FontError(err),
                })?;
        }

        Ok(())
    }

    /// Draw a path segment using the inner backend
    fn draw_path_segment(
        &mut self,
        points: &[(i32, i32)],
        style: &impl BackendStyle,
    ) -> Result<(), DrawingErrorKind<RoughSVGError>> {
        if points.len() < 2 {
            return Ok(());
        }

        // Use the style color directly for the stroke
        let stroke_color = style.color();

        self.inner
            .draw_path(points.iter().copied(), &stroke_color)
            .map_err(|e| match e {
                DrawingErrorKind::DrawingError(err) => DrawingErrorKind::DrawingError(RoughSVGError { inner: Box::new(err) }),
                DrawingErrorKind::FontError(err) => DrawingErrorKind::FontError(err),
            })
    }
}

impl<'a> DrawingBackend for RoughSVGBackend<'a> {
    type ErrorType = RoughSVGError;

    fn get_size(&self) -> (u32, u32) {
        self.inner.get_size()
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner
            .ensure_prepared()
            .map_err(|e| match e {
                DrawingErrorKind::DrawingError(err) => DrawingErrorKind::DrawingError(RoughSVGError { inner: Box::new(err) }),
                DrawingErrorKind::FontError(err) => DrawingErrorKind::FontError(err),
            })
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner
            .present()
            .map_err(|e| match e {
                DrawingErrorKind::DrawingError(err) => DrawingErrorKind::DrawingError(RoughSVGError { inner: Box::new(err) }),
                DrawingErrorKind::FontError(err) => DrawingErrorKind::FontError(err),
            })
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        // For individual pixels, just pass through to the inner backend
        self.inner
            .draw_pixel(point, color)
            .map_err(|e| match e {
                DrawingErrorKind::DrawingError(err) => DrawingErrorKind::DrawingError(RoughSVGError { inner: Box::new(err) }),
                DrawingErrorKind::FontError(err) => DrawingErrorKind::FontError(err),
            })
    }

    // Override the drawing methods to apply rough styling
    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.draw_rough_line(from, to, style)
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.draw_rough_rect(upper_left, bottom_right, style, fill)
    }

    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.draw_rough_path(path, style)
    }

    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.draw_rough_circle(center, radius, style, fill)
    }

    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.fill_rough_polygon(vert, style)
    }

    fn draw_text<TStyle: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        // Text drawing passes through to the inner backend without roughening
        self.inner
            .draw_text(text, style, pos)
            .map_err(|e| match e {
                DrawingErrorKind::DrawingError(err) => DrawingErrorKind::DrawingError(RoughSVGError { inner: Box::new(err) }),
                DrawingErrorKind::FontError(err) => DrawingErrorKind::FontError(err),
            })
    }

    fn estimate_text_size<TStyle: BackendTextStyle>(
        &self,
        text: &str,
        style: &TStyle,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        self.inner
            .estimate_text_size(text, style)
            .map_err(|e| match e {
                DrawingErrorKind::DrawingError(err) => DrawingErrorKind::DrawingError(RoughSVGError { inner: Box::new(err) }),
                DrawingErrorKind::FontError(err) => DrawingErrorKind::FontError(err),
            })
    }

    fn blit_bitmap(
        &mut self,
        pos: BackendCoord,
        (iw, ih): (u32, u32),
        src: &[u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        // Bitmap blitting passes through without roughening
        self.inner
            .blit_bitmap(pos, (iw, ih), src)
            .map_err(|e| match e {
                DrawingErrorKind::DrawingError(err) => DrawingErrorKind::DrawingError(RoughSVGError { inner: Box::new(err) }),
                DrawingErrorKind::FontError(err) => DrawingErrorKind::FontError(err),
            })
    }
}

pub use roughr::core::Options as RoughOptions;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_rough_svg_backend_creation() {
        let path = Path::new("test.svg");
        let backend = RoughSVGBackend::new(&path, (800, 600));
        assert_eq!(backend.get_size(), (800, 600));
    }

    #[test]
    fn test_rough_svg_with_options() {
        let path = Path::new("test.svg");
        let mut options = Options::default();
        options.roughness = Some(2.0);
        options.stroke_width = Some(3.0);
        
        let backend = RoughSVGBackend::with_options(&path, (800, 600), options);
        assert_eq!(backend.get_size(), (800, 600));
    }

    #[test]
    fn test_rough_svg_with_string() {
        let mut buffer = String::new();
        let backend = RoughSVGBackend::with_string(&mut buffer, (800, 600));
        assert_eq!(backend.get_size(), (800, 600));
    }
}
