use plotters::prelude::*;
use rough_plotters_svg::{RoughSVGBackend, RoughOptions};
use roughr::core::FillStyle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure rough options for histogram
    let mut rough_options = RoughOptions::default();
    rough_options.roughness = Some(1.2);
    rough_options.stroke_width = Some(1.5);
    rough_options.stroke = Some(palette::Srgba::from_components((114u8, 87u8, 82u8, 255u8)).into_format()); // Brown color
    rough_options.fill_style = Some(FillStyle::CrossHatch);
    rough_options.hachure_gap = Some(4.0);
    rough_options.hachure_angle = Some(45.0);
    
    let mut svg_string = String::new();
    
    {
        let backend = RoughSVGBackend::with_string_and_options(&mut svg_string, (640, 480), rough_options);
        let root = backend.into_drawing_area();

        // Fill with solid teal background (from rough_piet examples)
        root.fill(&RGBColor(150, 192, 183))?; // #96C0B7

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(35)
            .y_label_area_size(40)
            .margin(5)
            .caption("Rough Histogram Demo", ("sans-serif", 50.0))
            .build_cartesian_2d((0u32..10u32).into_segmented(), 0u32..10u32)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .bold_line_style(&RGBColor(114, 87, 82).mix(0.3)) // Brown grid lines
            .y_desc("Count")
            .x_desc("Bucket")
            .axis_desc_style(("sans-serif", 15).into_font().color(&RGBColor(114, 87, 82)))
            .axis_style(&RGBColor(114, 87, 82))
            .label_style(("sans-serif", 12).into_font().color(&RGBColor(114, 87, 82)))
            .draw()?;

        let data = [
            0u32, 1, 1, 1, 4, 2, 5, 7, 8, 6, 4, 2, 1, 8, 3, 3, 3, 4, 4, 3, 3, 3,
        ];

        chart.draw_series(
            Histogram::vertical(&chart)
                .style(RGBColor(114, 87, 82).mix(0.8).filled()) // Brown/rust color from rough_piet
                .data(data.iter().map(|x: &u32| (*x, 1))),
        )?;

        root.present()?;
    }
    
    // Save the SVG to a file
    std::fs::write("histogram.svg", &svg_string)?;
    println!("Rough histogram example saved to histogram.svg");
    
    Ok(())
}

#[test]
fn entry_point() {
    main().unwrap()
}
