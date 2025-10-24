use plotters::prelude::*;
use rough_plotters_svg::RoughSVGBackend;
use roughr::core::{FillStyle, Options};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a showcase demonstrating rough styling with various shapes and colors
    let mut options = Options::default();
    options.fill_style = Some(FillStyle::Hachure); // Use hachure fill style
    options.stroke_width = Some(2.0);
    options.roughness = Some(1.5);
    options.bowing = Some(1.0);
    
    let backend = RoughSVGBackend::with_options("showcase_test.svg", (800, 400), options);
    let root = backend.into_drawing_area();
    root.fill(&RGBColor(254, 246, 201))?; // Cream background

    let mut chart = ChartBuilder::on(&root)
        .caption("Rough Plotters SVG - Shapes & Colors Showcase", ("serif", 30).into_font().color(&RGBColor(114, 87, 82)))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0..8, 0..100)?;

    chart
        .configure_mesh()
        .x_desc("Different Shapes & Colors")
        .y_desc("Values")
        .axis_style(&RGBColor(114, 87, 82))
        .label_style(("serif", 12).into_font().color(&RGBColor(114, 87, 82)))
        .draw()?;

    // Draw rectangles with different colors - all will use hachure fill
    chart.draw_series(
        [
            (1, 60, RGBColor(220, 57, 18)),   // Red
            (2, 75, RGBColor(255, 153, 0)),   // Orange  
            (3, 45, RGBColor(16, 150, 24)),   // Green
            (4, 85, RGBColor(153, 0, 153)),   // Purple
            (5, 55, RGBColor(52, 152, 219)),  // Blue
            (6, 70, RGBColor(241, 196, 15)),  // Yellow
            (7, 40, RGBColor(231, 76, 60)),   // Crimson
        ]
        .iter()
        .map(|(x, y, color)| {
            Rectangle::new([(*x, 10), (*x + 1, *y)], color.filled())
        }),
    )?;

    // Add circles with rough styling
    chart.draw_series(
        [(2, 30), (4, 35), (6, 25)]
        .iter()
        .map(|(x, y)| Circle::new((*x, *y), 8, RGBColor(46, 125, 50).filled()))
    )?;

    root.present()?;
    println!("Shapes & colors showcase saved to showcase_test.svg");

    Ok(())
}
