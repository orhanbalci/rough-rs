use plotters::prelude::*;
use rough_plotters_svg::{RoughSVGBackend, RoughOptions};
use roughr::core::FillStyle;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use rand_xorshift::XorShiftRng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate some sample data
    let data: Vec<_> = {
        let norm_dist = Normal::new(500.0, 100.0).unwrap();
        let mut x_rand = XorShiftRng::from_seed(*b"MyFragileSeed123");
        let x_iter = norm_dist.sample_iter(&mut x_rand);
        x_iter
            .filter(|x| *x < 1500.0)
            .take(100)
            .zip(0..)
            .map(|(x, b)| x + (b as f64).powf(1.2))
            .collect()
    };

    // Configure rough options for area chart
    let mut rough_options = RoughOptions::default();
    rough_options.roughness = Some(1.8);
    rough_options.stroke_width = Some(2.5);
    rough_options.stroke = Some(palette::Srgba::from_components((114u8, 87u8, 82u8, 255u8)).into_format()); // Brown color
    rough_options.fill_style = Some(FillStyle::ZigZag);
    rough_options.hachure_gap = Some(5.0);
    rough_options.zigzag_offset = Some(3.0);
    
    let mut svg_string = String::new();
    
    {
        let backend = RoughSVGBackend::with_string_and_options(&mut svg_string, (1024, 768), rough_options);
        let root = backend.into_drawing_area();

        // Fill with solid cream background (from rough_piet examples)
        root.fill(&RGBColor(254, 246, 201))?; // Cream color from rough_piet

        let mut chart = ChartBuilder::on(&root)
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 60)
            .caption("Rough Area Chart Demo", ("sans-serif", 40))
            .build_cartesian_2d(0..(data.len() - 1), 0.0..1500.0)?;

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .axis_style(&RGBColor(114, 87, 82))
            .label_style(("sans-serif", 12).into_font().color(&RGBColor(114, 87, 82)))
            .draw()?;

        chart.draw_series(
            AreaSeries::new(
                (0..).zip(data.iter()).map(|(x, y)| (x, *y)),
                0.0,
                RGBColor(70, 130, 180).mix(0.4), // Steel blue with transparency
            )
            .border_style(&RGBColor(114, 87, 82)), // Brown border from rough_piet
        )?;

        root.present()?;
    }
    
    // Save the SVG to a file
    std::fs::write("area_chart.svg", &svg_string)?;
    println!("Rough area chart example saved to area_chart.svg");
    
    Ok(())
}

#[test]
fn entry_point() {
    main().unwrap()
}
