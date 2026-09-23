//! Draws an SVG file in a hand drawn style.
//!
//! roughr's `path` takes SVG path data (the `d` attribute). A real SVG file
//! also has shapes like `<rect>` and `<circle>`, transforms and styles, so
//! this example lets `usvg` normalize the file into plain paths first, then
//! sketches every path with its own fill and stroke color.
//!
//! ```sh
//! cargo run -p rough_tiny_skia --example svg -- rough_tiny_skia/assets/house.svg house.png
//! ```

use palette::Srgba;
use rough_tiny_skia::SkiaGenerator;
use roughr::core::{FillStyle, OptionsBuilder};
use tiny_skia::{Paint, Pixmap, Rect, Transform};
use usvg::tiny_skia_path::PathSegment;

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args
        .next()
        .unwrap_or_else(|| concat!(env!("CARGO_MANIFEST_DIR"), "/assets/house.svg").to_owned());
    let output = args.next().unwrap_or_else(|| "svg.png".to_owned());

    let data = std::fs::read(&input).expect("failed to read svg file");
    let tree = usvg::Tree::from_data(&data, &usvg::Options::default()).expect("invalid svg");
    let size = tree.size();

    let mut pixmap = Pixmap::new(size.width() as u32, size.height() as u32).unwrap();
    let mut background = Paint::default();
    background.set_color_rgba8(150, 192, 183, 255);
    pixmap.fill_rect(
        Rect::from_xywh(0.0, 0.0, size.width(), size.height()).unwrap(),
        &background,
        Transform::identity(),
        None,
    );

    draw_group(tree.root(), &mut pixmap);

    pixmap.save_png(&output).unwrap();
    println!("wrote {output}");
}

fn draw_group(group: &usvg::Group, pixmap: &mut Pixmap) {
    for node in group.children() {
        match node {
            usvg::Node::Group(group) => draw_group(group, pixmap),
            usvg::Node::Path(path) => draw_path(path, pixmap),
            _ => {}
        }
    }
}

fn draw_path(path: &usvg::Path, pixmap: &mut Pixmap) {
    // Bake the transforms of all parent groups into the points
    let Some(data) = path.data().clone().transform(path.abs_transform()) else {
        return;
    };

    let mut options = OptionsBuilder::default();
    options
        .fill_style(FillStyle::Hachure)
        .fill_weight(1.0)
        .hachure_gap(4.0);
    match path.stroke() {
        Some(stroke) => {
            options
                .stroke(to_srgba(stroke.paint(), stroke.opacity().get()))
                .stroke_width(stroke.width().get());
        }
        // Hide the outline, but keep the default stroke width for the fill lines
        None => {
            options.stroke(Srgba::new(0.0, 0.0, 0.0, 0.0));
        }
    }
    if let Some(fill) = path.fill() {
        options.fill(to_srgba(fill.paint(), fill.opacity().get()));
    }

    let generator = SkiaGenerator::new(options.build().unwrap());
    generator
        .path::<f32>(to_svg_path_data(&data))
        .draw(&mut pixmap.as_mut());
}

/// Converts a usvg path back to SVG path data for roughr.
fn to_svg_path_data(data: &usvg::tiny_skia_path::Path) -> String {
    let mut d = String::new();
    for segment in data.segments() {
        let part = match segment {
            PathSegment::MoveTo(p) => format!("M {} {} ", p.x, p.y),
            PathSegment::LineTo(p) => format!("L {} {} ", p.x, p.y),
            PathSegment::QuadTo(c, p) => format!("Q {} {} {} {} ", c.x, c.y, p.x, p.y),
            PathSegment::CubicTo(c1, c2, p) => {
                format!("C {} {} {} {} {} {} ", c1.x, c1.y, c2.x, c2.y, p.x, p.y)
            }
            PathSegment::Close => "Z ".to_owned(),
        };
        d.push_str(&part);
    }
    d
}

/// Only plain colors are supported; gradients and patterns fall back to gray.
fn to_srgba(paint: &usvg::Paint, opacity: f32) -> Srgba {
    match paint {
        usvg::Paint::Color(c) => Srgba::new(
            c.red as f32 / 255.0,
            c.green as f32 / 255.0,
            c.blue as f32 / 255.0,
            opacity,
        ),
        _ => Srgba::new(0.5, 0.5, 0.5, opacity),
    }
}
