use plotters::prelude::*;
use rough_plotters_svg::{RoughSVGBackend, RoughOptions};
use roughr::core::FillStyle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure rough options for sketchy appearance
    let mut rough_options = RoughOptions::default();
    rough_options.roughness = Some(1.5);
    rough_options.stroke_width = Some(2.0);
    rough_options.stroke = Some(palette::Srgba::from_components((114u8, 87u8, 82u8, 255u8)).into_format()); // Brown color
    rough_options.fill_style = Some(FillStyle::Hachure);
    rough_options.hachure_gap = Some(6.0);
    
    let mut svg_string = String::new();
    
    {
        let backend = RoughSVGBackend::with_string_and_options(&mut svg_string, (1024, 768), rough_options);
        let root_area = backend.into_drawing_area();

        // Fill with solid teal background (from rough_piet examples)
        root_area.fill(&RGBColor(150, 192, 183))?; // #96C0B7

        let root_area = root_area.titled("Rough Chart Demo", ("sans-serif", 60))?;

        let (upper, lower) = root_area.split_vertically(400);

        let x_axis = (-3.4f32..3.4).step(0.1);

        let mut cc = ChartBuilder::on(&upper)
            .margin(5)
            .set_all_label_area_size(50)
            .caption("Rough Sine and Cosine", ("sans-serif", 40))
            .build_cartesian_2d(-3.4f32..3.4, -1.2f32..1.2f32)?;

        cc.configure_mesh()
            .x_labels(20)
            .y_labels(10)
            .disable_mesh()
            .x_label_formatter(&|v| format!("{:.1}", v))
            .y_label_formatter(&|v| format!("{:.1}", v))
            .axis_style(&RGBColor(114, 87, 82)) // Brown axis lines from rough_piet
            .label_style(("sans-serif", 15).into_font().color(&RGBColor(114, 87, 82)))
            .draw()?;

        cc.draw_series(LineSeries::new(x_axis.values().map(|x| (x, x.sin())), &RGBColor(220, 20, 60)))?  // Crimson
            .label("Sine")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RGBColor(220, 20, 60)));

        cc.draw_series(LineSeries::new(
            x_axis.values().map(|x| (x, x.cos())),
            &RGBColor(30, 144, 255), // Dodger blue
        ))?
        .label("Cosine")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RGBColor(30, 144, 255)));

        cc.configure_series_labels()
            .border_style(&RGBColor(114, 87, 82))
            .background_style(&RGBColor(254, 246, 201).mix(0.8))
            .label_font(("sans-serif", 15).into_font().color(&RGBColor(114, 87, 82)))
            .draw()?;

        // Draw point series with rough styling
        cc.draw_series(PointSeries::of_element(
            (-3.0f32..2.1f32).step(1.0).values().map(|x| (x, x.sin())),
            5,
            ShapeStyle::from(&RGBColor(255, 140, 0)).filled(), // Dark orange
            &|coord, size, style| {
                EmptyElement::at(coord)
                    + Circle::new((0, 0), size, style)
                    + Text::new(format!("{:.1}", coord.0), (0, 15), ("sans-serif", 15))
            },
        ))?;

        let drawing_areas = lower.split_evenly((1, 2));

        for (drawing_area, idx) in drawing_areas.iter().zip(1..) {
            let mut cc = ChartBuilder::on(drawing_area)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .margin_right(20)
                .caption(format!("y = x^{}", 1 + 2 * idx), ("sans-serif", 40))
                .build_cartesian_2d(-1f32..1f32, -1f32..1f32)?;
            
            cc.configure_mesh()
                .x_labels(5)
                .y_labels(3)
                .max_light_lines(4)
                .axis_style(&RGBColor(114, 87, 82))
                .label_style(("sans-serif", 12).into_font().color(&RGBColor(114, 87, 82)))
                .draw()?;

            cc.draw_series(LineSeries::new(
                (-1f32..1f32)
                    .step(0.01)
                    .values()
                    .map(|x| (x, x.powf(idx as f32 * 2.0 + 1.0))),
                &RGBColor(138, 43, 226), // Blue violet
            ))?;
        }

        root_area.present()?;
    }
    
    // Save the SVG to a file
    std::fs::write("chart.svg", &svg_string)?;
    println!("Rough chart example saved to chart.svg");
    
    Ok(())
}

#[test]
fn entry_point() {
    main().unwrap()
}
