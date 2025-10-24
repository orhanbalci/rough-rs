use plotters::prelude::*;
use rough_plotters_svg::RoughSVGBackend;
use roughr::core::{FillStyle, Options};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fill_styles = vec![
        (FillStyle::Solid, "solid"),
        (FillStyle::Hachure, "hachure"),
        (FillStyle::ZigZag, "zigzag"),
        (FillStyle::CrossHatch, "crosshatch"),
        (FillStyle::Dots, "dots"),
        (FillStyle::Dashed, "dashed"),
        (FillStyle::ZigZagLine, "zigzagline"),
    ];

    for (fill_style, name) in fill_styles {
        let mut options = Options::default();
        options.fill_style = Some(fill_style);
        options.stroke_width = Some(2.0);
        options.roughness = Some(1.0);
        
        let filename = format!("{}.svg", name);
        let backend = RoughSVGBackend::with_options(&filename, (640, 480), options);
        let root = backend.into_drawing_area();
        root.fill(&RGBColor(254, 246, 201))?; // Cream background

        let mut chart = ChartBuilder::on(&root)
            .caption(&format!("Fill Style: {:?}", fill_style), ("Arial", 30).into_font().color(&RGBColor(114, 87, 82)))
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(0..10, 0..100)?;

        chart
            .configure_mesh()
            .x_desc("X Axis")
            .y_desc("Y Axis")
            .axis_style(&RGBColor(114, 87, 82))
            .draw()?;

        // Draw some filled rectangles
        chart.draw_series(
            [
                (1, 30, RGBColor(220, 57, 18)),   // Red
                (3, 50, RGBColor(255, 153, 0)),   // Orange  
                (5, 70, RGBColor(16, 150, 24)),   // Green
                (7, 90, RGBColor(153, 0, 153)),   // Purple
            ]
            .iter()
            .map(|(x, y, color)| {
                Rectangle::new([(*x, 0), (*x + 1, *y)], color.filled())
            }),
        )?;

        // Draw a filled circle
        chart.draw_series(std::iter::once(Circle::new((9, 50), 20, RGBColor(0, 100, 200).filled())))?;

        root.present()?;
        println!("Fill style example saved to {}", filename);
    }

    println!("All fill style examples generated!");
    Ok(())
}
