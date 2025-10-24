# rough_plotters_svg

[![Crates.io](https://img.shields.io/crates/v/rough_plotters_svg.svg)](https://crates.io/crates/rough_plotters_svg)
[![Documentation](https://docs.rs/rough_plotters_svg/badge.svg)](https://docs.rs/rough_plotters_svg)
[![License](https://img.shields.io/github/license/orhanbalci/rough-rs.svg)](https://github.com/orhanbalci/rough-rs/LICENSE)

<!-- cargo-sync-readme start -->

# Rough Plotters SVG

A rough/sketchy style wrapper around the `plotters-svg` backend for the [Plotters](https://github.com/plotters-rs/plotters) 
plotting library. This crate provides a `DrawingBackend` implementation that intercepts drawing calls and applies 
rough, hand-drawn style transformations to geometric primitives like lines, rectangles, circles, and paths.

## Features

- **Drop-in replacement** for `plotters-svg::SVGBackend`
- **Rough styling** applied to all geometric primitives
- **Configurable roughness** with full control over rough options
- **Multiple fill styles** including hachure, cross-hatch, zigzag, and more
- **Compatible** with the entire Plotters ecosystem

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
rough_plotters_svg = "0.0.0"
plotters = "0.3"
roughr = "0.13"
```

### Basic Usage

```rust
use rough_plotters_svg::RoughSVGBackend;
use plotters::prelude::*;
use roughr::core::{FillStyle, Options};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure rough styling
    let mut options = Options::default();
    options.fill_style = Some(FillStyle::Hachure);
    options.roughness = Some(2.0);
    options.stroke_width = Some(1.5);

    // Create backend with rough styling
    let backend = RoughSVGBackend::with_options("chart.svg", (800, 600), options);
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;

    // Use with plotters as normal - all shapes will be rough styled
    let mut chart = ChartBuilder::on(&root)
        .caption("Rough Chart", ("serif", 40))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0f32..10f32, 0f32..100f32)?;

    chart.configure_mesh().draw()?;

    chart.draw_series(
        (0..10).map(|x| Rectangle::new([(x as f32, 0f32), (x as f32 + 0.8, x as f32 * 10f32)], RED.filled()))
    )?;

    root.present()?;
    Ok(())
}
```

![Basic Chart Example](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/rough_plotters_svg/assets/chart.svg)

### Fill Styles Showcase

![Fill Styles Showcase](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/rough_plotters_svg/assets/showcase.svg)

### Working with Fill Styles

```rust
use rough_plotters_svg::RoughSVGBackend;
use plotters::prelude::*;
use roughr::core::{FillStyle, Options};

fn create_chart_with_fill_style(fill_style: FillStyle, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut options = Options::default();
    options.fill_style = Some(fill_style);
    options.stroke_width = Some(2.0);
    options.roughness = Some(1.0);
    
    let backend = RoughSVGBackend::with_options(filename, (400, 300), options);
    let root = backend.into_drawing_area();
    root.fill(&RGBColor(254, 246, 201))?; // Cream background

    let mut chart = ChartBuilder::on(&root)
        .caption(&format!("Fill Style: {:?}", fill_style), ("Arial", 20))
        .margin(10)
        .build_cartesian_2d(0..10, 0..100)?;

    chart.configure_mesh().draw()?;

    // Draw some shapes with the fill style
    chart.draw_series(
        [(2, 30), (4, 50), (6, 70), (8, 90)]
            .iter()
            .map(|(x, y)| Rectangle::new([(*x, 0), (*x + 1, *y)], BLUE.filled()))
    )?;

    chart.draw_series(
        std::iter::once(Circle::new((5, 50), 15, RED.filled()))
    )?;

    root.present()?;
    Ok(())
}
```

![Fill Style Example](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/rough_plotters_svg/assets/hachure.svg)

## Available Fill Styles

- `FillStyle::Solid` - Solid color fill
- `FillStyle::Hachure` - Parallel line hatching (default)
- `FillStyle::ZigZag` - Zigzag pattern fill
- `FillStyle::CrossHatch` - Cross-hatched pattern
- `FillStyle::Dots` - Dotted pattern fill
- `FillStyle::Dashed` - Dashed line pattern
- `FillStyle::ZigZagLine` - Zigzag line pattern

![CrossHatch Fill Style](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/rough_plotters_svg/assets/crosshatch.svg)

### Stock Chart Example

Rough styling works great with financial charts:

![Stock Chart Example](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/rough_plotters_svg/assets/stock.svg)

## String-based Backend

For in-memory SVG generation:

```rust
use rough_plotters_svg::RoughSVGBackend;
use roughr::core::Options;

let mut svg_string = String::new();
let backend = RoughSVGBackend::with_string_and_options(&mut svg_string, (640, 480), Options::default());
// ... use backend normally ...
// svg_string now contains the generated SVG
```

<!-- cargo-sync-readme end -->

## 🔭 Examples

For more examples have a look at the [examples](https://github.com/orhanbalci/rough-rs/tree/main/rough_plotters_svg/examples) folder.

You can run examples with:
```bash
cargo run --example chart
cargo run --example showcase
cargo run --example all_fill_styles
cargo run --example stock
```

## 📝 License

Licensed under MIT License ([LICENSE](LICENSE)).

### 🚧 Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the MIT license, shall be licensed as above, without any additional terms or conditions.
