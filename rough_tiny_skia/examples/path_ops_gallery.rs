//! Generates the operation reference images used in the svg_path_ops
//! documentation.
//!
//! Each image is a strip of cells that apply one operation with a different
//! value from left to right. The original path is drawn dashed in purple,
//! the result filled. Run it from the workspace root after changing an operation:
//!
//! ```sh
//! cargo run -p rough_tiny_skia --example path_ops_gallery
//! ```
//!
//! The images are written to `svg_path_ops/assets/ops/`.

mod common;

use std::path::{Path, PathBuf};

use common::{Canvas, Layout, BROWN, CREAM};
use palette::Srgba;
use rough_tiny_skia::SkiaGenerator;
use roughr::core::{FillStyle, OptionsBuilder};
use svg_path_ops::bbox::{Alignment, BBox, BoxAlignment, InboxParameters, ScaleType};
use svg_path_ops::euclid::default::Point2D;
use svg_path_ops::pt::PathTransformer;
use svg_path_ops::shapes::Shape;
use svg_path_ops::svgtypes::PathParser;
use svg_path_ops::{
    is_closed,
    reverse,
    segments_with_context,
    split_subpaths,
    write_path,
    FillRule,
    PathMeasure,
    PathSegment,
    WriteOptions,
};
use tiny_skia::{Paint, PathBuilder, Stroke, StrokeDash, Transform};

const CELL_WIDTH: f32 = 170.0;
const CELL_HEIGHT: f32 = 130.0;

const LAYOUT: Layout = Layout {
    cell_width: CELL_WIDTH,
    cell_height: CELL_HEIGHT,
    wordmark: "SVG_PATH_OPS",
};

/// Side of the square Ferris is fitted into before an operation, leaving
/// room in the cell for the transformed result.
const SUBJECT_SIZE: f64 = 84.0;

/// Color of the dashed outline marking where the path was before an
/// operation: the purple from the original svg_path_ops images.
const GHOST_COLOR: (u8, u8, u8) = (156, 1, 188);

/// Ferris the crab, the Rust mascot by Karen Rustad Tölva, dedicated to the
/// public domain (CC0) at <https://rustacean.net>. The parts come from
/// `rustacean-flat-noshadow.svg` in drawing order, moved to absolute
/// coordinates and written compactly with svg_path_ops itself. Each part is
/// a single subpath starting with `M`.
const FERRIS_PARTS: [(Part, &str); 8] = [
    (
        Part::Back,
        "M597.3 357.5C476.1 357.5 366 372 284.4 395.8L284.4 598.7C366 622.4 \
         476.1 637 597.3 637 736.1 637 860.3 617.9 943.8 587.8L943.8 \
         406.7C860.3 376.6 736.1 357.5 597.3 357.5",
    ),
    (
        Part::Back,
        "M1068.8 522.3L1054.5 492.9C1054.6 491.8 1054.7 490.6 1054.7 489.5 \
         1054.7 456.1 1020 425.4 961.6 400.8L961.6 578.3C988.8 566.8 1010.9 554 \
         1026.7 540.2 1022.1 558.9 1006.2 596.7 993.5 623.3 972.6 661.4 965.1 \
         694.5 966 696.2 966.7 697.2 973.8 686.2 984.3 668.5 1008.7 633.7 \
         1054.9 567.3 1064.2 550.4 1074.6 531.2 1068.8 522.3 1068.8 522.3",
    ),
    (
        Part::Back,
        "M149.1 491.5C149.1 497.9 150.4 504.2 152.9 510.4L144.3 525.6C144.3 \
         525.6 137.5 534.4 149.5 553.1 160.1 569.5 213.2 634 241.2 667.7 253.2 \
         685 261.3 695.6 262.1 694.6 263.2 693 254.6 660.5 230.7 623.5 219.6 \
         603.6 206.2 577.2 198.3 557.7 220.6 571.9 249.7 584.7 284.4 \
         595.7L284.4 387.3C200.9 413.8 149.1 450.7 149.1 491.5",
    ),
    (
        Part::Body,
        "M1151.3 522.2L1057.9 453.3C1057 450.3 1056.1 447.2 1055.2 444.2L1085.9 \
         399.7C1089 395.2 1089.6 389.3 1087.6 384.2 1085.6 379 1081.1 375.4 \
         1075.8 374.5L1024 365.7C1022 361.6 1019.8 357.5 1017.7 353.5L1039.5 \
         303.6C1041.8 298.5 1041.3 292.6 1038.3 288 1035.4 283.4 1030.3 280.7 \
         1025 280.9L972.3 282.8C969.6 279.2 966.8 275.7 964 272.3L976.1 \
         218.8C977.3 213.3 975.8 207.7 972 203.7 968.2 199.8 962.8 198.1 957.6 \
         199.4L906.3 212C903 209.1 899.6 206.2 896.2 203.4L898 148.4C898.2 \
         142.9 895.6 137.6 891.2 134.5 886.7 131.4 881.1 130.9 876.2 \
         133.3L828.4 156C824.5 153.8 820.6 151.6 816.7 149.5L808.3 95.4C807.4 \
         89.9 803.9 85.2 798.9 83.1 794 81 788.4 81.7 784.1 84.9L741.4 \
         116.9C737.2 115.5 733 114.2 728.7 112.9L710.3 61.6C708.4 56.4 704.1 \
         52.5 698.9 51.4 693.6 50.4 688.2 52.2 684.6 56.2L648.7 96.4C644.4 95.9 \
         640 95.5 635.7 95.1L607.9 48.4C605.1 43.6 600.2 40.8 594.8 40.8 589.5 \
         40.8 584.5 43.6 581.7 48.4L554 95.1C549.6 95.5 545.3 95.9 540.9 \
         96.4L505 56.2C501.4 52.2 496 50.4 490.8 51.4 485.5 52.5 481.2 56.4 \
         479.3 61.6L460.9 112.9C456.7 114.2 452.5 115.5 448.2 116.9L405.6 \
         84.9C401.3 81.6 395.6 81 390.7 83.1 385.8 85.2 382.2 89.9 381.4 \
         95.4L372.9 149.5C369 151.6 365.1 153.8 361.3 156L313.4 133.3C308.6 \
         130.9 302.9 131.4 298.5 134.5 294 137.6 291.4 142.9 291.6 148.4L293.5 \
         203.4C290 206.2 286.7 209.1 283.3 212L232.1 199.4C226.9 198.2 221.4 \
         199.8 217.6 203.7 213.8 207.7 212.3 213.3 213.5 218.8L225.6 \
         272.3C222.8 275.7 220 279.2 217.3 282.8L164.6 280.9C159.3 280.7 154.3 \
         283.4 151.3 288 148.3 292.6 147.9 298.5 150.1 303.6L171.9 353.5C169.8 \
         357.5 167.7 361.6 165.6 365.7L113.8 374.5C108.5 375.4 104 379 102 \
         384.2 100 389.3 100.6 395.2 103.7 399.7L134.4 444.2C134.2 445 134 \
         445.8 133.7 446.5L47 538.7C47 538.7 33.7 549.1 53 573.6 70.1 595.2 \
         157.7 680.9 204 725.8 223.6 748.5 237 762.6 238.4 761.4 240.6 759.5 \
         229 718 190.5 669.4 160.8 625.8 122.3 558.7 131.3 550.9 131.3 550.9 \
         141.6 537.9 162.2 528.5 163 529.1 161.4 527.9 162.2 528.5 162.2 528.5 \
         597.4 729.2 1001 531.9 1047.1 523.6 1075 548.3 1075 548.3 1084.7 553.9 \
         1059.7 622.9 1039.2 668.6 1011.3 720.5 1007.3 760.9 1009.6 762.3 \
         1011.1 763.2 1021.3 747.8 1035.7 723.2 1071.4 673.5 1138.9 578.6 \
         1151.3 555.6 1165.3 529.5 1151.3 522.2 1151.3 522.2",
    ),
    (
        Part::Front,
        "M966.1 497C966.1 497 950.5 559.1 853.6 624.3L826.5 630.6C826.5 630.6 \
         738.6 470.4 614.1 651 614.1 651 652.9 628.4 756.9 655.9 756.9 655.9 \
         709.1 729.1 612.7 726.9 612.7 726.9 705 837.7 845.6 677.6 845.6 677.6 \
         994.2 620.2 1006.3 497L966.1 497Z",
    ),
    (
        Part::Front,
        "M441.4 662.5C533.2 651 586.3 649.9 586.3 649.9 463.4 487.8 363.7 634.5 \
         363.7 634.5 338.8 625.6 313.3 590.9 294.3 558.9L218 536.4C305.9 685.5 \
         371.3 687.8 371.3 687.8 507.5 862.4 572.1 722.1 572.1 722.1 495.6 \
         713.4 441.4 662.5 441.4 662.5",
    ),
    (
        Part::Eye,
        "M677.4 417.5C677.4 417.5 720.9 369.9 764.4 417.5 764.4 417.5 798.5 481 \
         764.4 512.8 764.4 512.8 708.5 557.2 677.4 512.8 677.4 512.8 640.1 \
         477.9 677.4 417.5",
    ),
    (
        Part::Eye,
        "M483.3 404.5C483.3 404.5 557.9 371.5 578.3 445.2 578.3 445.2 599.6 \
         531.2 517 536 517 536 411.7 515.7 483.3 404.5",
    ),
];

#[derive(Clone, Copy)]
enum Part {
    Back,
    Body,
    Front,
    Eye,
}

/// All Ferris parts as one path, so operations move them together.
fn ferris() -> String {
    FERRIS_PARTS
        .iter()
        .map(|(_, part)| *part)
        .collect::<Vec<_>>()
        .join(" ")
}

fn rgba((r, g, b): (u8, u8, u8)) -> Srgba<f32> {
    Srgba::from_components((r, g, b, 255u8)).into_format()
}

/// Draws `path` sketched with a hachure fill.
fn draw_filled(canvas: &mut Canvas, path: &str) {
    let options = OptionsBuilder::default()
        .stroke(rgba(BROWN))
        .fill(rgba(CREAM))
        .fill_style(FillStyle::Hachure)
        .roughness(0.6)
        .build()
        .unwrap();
    SkiaGenerator::new(options)
        .path::<f32>(path.to_owned())
        .draw(&mut canvas.pixmap.as_mut());
}

/// Draws `path` as a thin sketched outline.
fn draw_outline(canvas: &mut Canvas, path: &str) {
    let options = OptionsBuilder::default()
        .stroke(rgba(BROWN))
        .roughness(0.6)
        .build()
        .unwrap();
    SkiaGenerator::new(options)
        .path::<f32>(path.to_owned())
        .draw(&mut canvas.pixmap.as_mut());
}

/// Draws a Ferris path produced from [`ferris`], giving each part its own
/// style.
fn draw_ferris(canvas: &mut Canvas, path: &str) {
    for ((part, _), data) in FERRIS_PARTS.iter().zip(ferris_parts(path)) {
        match part {
            Part::Back => draw_outline(canvas, &data),
            Part::Body | Part::Front => draw_filled(canvas, &data),
            Part::Eye => draw_solid(canvas, &data),
        }
    }
}

/// Splits a path produced from [`ferris`] back into its parts.
fn ferris_parts(path: &str) -> Vec<String> {
    split_subpaths(parse(path))
        .iter()
        .map(|subpath| write_path(subpath, &WriteOptions::default()))
        .collect()
}

/// Draws the body outline of a Ferris path dashed, to mark where it was.
fn draw_ferris_ghost(canvas: &mut Canvas, path: &str) {
    for ((part, _), data) in FERRIS_PARTS.iter().zip(ferris_parts(path)) {
        if matches!(part, Part::Body) {
            draw_dashed(canvas, &data, GHOST_COLOR, 2.0);
        }
    }
}

/// Draws `path` filled with the stroke color.
fn draw_solid(canvas: &mut Canvas, path: &str) {
    let options = OptionsBuilder::default()
        .stroke(rgba(BROWN))
        .fill(rgba(BROWN))
        .fill_style(FillStyle::Solid)
        .roughness(0.4)
        .build()
        .unwrap();
    SkiaGenerator::new(options)
        .path::<f32>(path.to_owned())
        .draw(&mut canvas.pixmap.as_mut());
}

/// Strokes `path` with an exact dashed line, for originals and guides.
fn draw_dashed(canvas: &mut Canvas, path: &str, color: (u8, u8, u8), width: f32) {
    let Some(tiny_path) = tiny_path(path) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color_rgba8(color.0, color.1, color.2, 255);
    let stroke = Stroke {
        width,
        dash: StrokeDash::new(vec![5.0, 3.0], 0.0),
        ..Stroke::default()
    };
    canvas
        .pixmap
        .stroke_path(&tiny_path, &paint, &stroke, Transform::identity(), None);
}

/// Absolute move, line, cubic curve and close segments drawing `path`.
fn normalized(path: &str) -> Vec<PathSegment> {
    let segments = parse(path);
    svg_path_ops::normalize(svg_path_ops::absolutize(segments.iter())).collect()
}

/// Converts SVG path data to an exact tiny-skia path.
fn tiny_path(path: &str) -> Option<tiny_skia::Path> {
    let mut builder = PathBuilder::new();
    for segment in normalized(path) {
        match segment {
            PathSegment::MoveTo { x, y, .. } => builder.move_to(x as f32, y as f32),
            PathSegment::LineTo { x, y, .. } => builder.line_to(x as f32, y as f32),
            PathSegment::CurveTo { x1, y1, x2, y2, x, y, .. } => builder.cubic_to(
                x1 as f32, y1 as f32, x2 as f32, y2 as f32, x as f32, y as f32,
            ),
            PathSegment::ClosePath { .. } => builder.close(),
            _ => unreachable!("normalize only emits M, L, C and Z"),
        }
    }
    builder.finish()
}

/// Fills `path` with the nonzero rule, the SVG default: a region is filled
/// unless the contours around it wind in opposite directions.
fn fill_nonzero(canvas: &mut Canvas, path: &str) {
    let Some(tiny_path) = tiny_path(path) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color_rgba8(CREAM.0, CREAM.1, CREAM.2, 255);
    canvas.pixmap.fill_path(
        &tiny_path,
        &paint,
        tiny_skia::FillRule::Winding,
        Transform::identity(),
        None,
    );
}

/// The point halfway along each segment of `path`, with the direction the
/// path is drawn in there.
fn segment_midpoints(path: &str) -> Vec<((f64, f64), (f64, f64))> {
    let mut midpoints = Vec::new();
    let (mut current, mut subpath_start) = ((0.0, 0.0), (0.0, 0.0));
    let line = |from: (f64, f64), to: (f64, f64)| {
        (
            ((from.0 + to.0) / 2.0, (from.1 + to.1) / 2.0),
            (to.0 - from.0, to.1 - from.1),
        )
    };
    for segment in normalized(path) {
        match segment {
            PathSegment::MoveTo { x, y, .. } => {
                current = (x, y);
                subpath_start = current;
            }
            PathSegment::LineTo { x, y, .. } => {
                midpoints.push(line(current, (x, y)));
                current = (x, y);
            }
            PathSegment::CurveTo { x1, y1, x2, y2, x, y, .. } => {
                // A cubic curve at t = 0.5 and its derivative there
                let (p0, p1, p2, p3) = (current, (x1, y1), (x2, y2), (x, y));
                let at = |a: f64, b: f64, c: f64, d: f64| (a + 3.0 * b + 3.0 * c + d) / 8.0;
                let slope = |a: f64, b: f64, c: f64, d: f64| 0.75 * (c + d - a - b);
                midpoints.push((
                    (at(p0.0, p1.0, p2.0, p3.0), at(p0.1, p1.1, p2.1, p3.1)),
                    (slope(p0.0, p1.0, p2.0, p3.0), slope(p0.1, p1.1, p2.1, p3.1)),
                ));
                current = p3;
            }
            PathSegment::ClosePath { .. } => {
                if current != subpath_start {
                    midpoints.push(line(current, subpath_start));
                }
                current = subpath_start;
            }
            _ => unreachable!("normalize only emits M, L, C and Z"),
        }
    }
    midpoints
}

/// Draws an arrowhead halfway along each segment of `path`, pointing the way
/// the path is drawn.
fn draw_direction(canvas: &mut Canvas, path: &str) {
    for (at, direction) in segment_midpoints(path) {
        draw_arrowhead(canvas, at, direction);
    }
}

/// Draws an arrowhead centered on `(x, y)`, pointing along `(dx, dy)`.
fn draw_arrowhead(canvas: &mut Canvas, (x, y): (f64, f64), (dx, dy): (f64, f64)) {
    let length = dx.hypot(dy);
    if length == 0.0 {
        return;
    }
    let (ux, uy) = (dx / length, dy / length);
    let tip = (x + ux * 5.0, y + uy * 5.0);
    let base = (x - ux * 4.0, y - uy * 4.0);
    let side = (-uy * 4.5, ux * 4.5);

    let mut builder = PathBuilder::new();
    builder.move_to(tip.0 as f32, tip.1 as f32);
    builder.line_to((base.0 + side.0) as f32, (base.1 + side.1) as f32);
    builder.line_to((base.0 - side.0) as f32, (base.1 - side.1) as f32);
    builder.close();
    let Some(arrow) = builder.finish() else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color_rgba8(BROWN.0, BROWN.1, BROWN.2, 255);
    canvas.pixmap.fill_path(
        &arrow,
        &paint,
        tiny_skia::FillRule::Winding,
        Transform::identity(),
        None,
    );
}

/// Draws a dot; hollow dots are outlined and filled with the canvas color.
fn draw_dot(canvas: &mut Canvas, (x, y): (f64, f64), hollow: bool) {
    let Some(circle) = PathBuilder::from_circle(x as f32, y as f32, 3.5) else {
        return;
    };
    let mut paint = Paint::default();
    if hollow {
        paint.set_color_rgba8(CREAM.0, CREAM.1, CREAM.2, 255);
        canvas.pixmap.fill_path(
            &circle,
            &paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );
        paint.set_color_rgba8(BROWN.0, BROWN.1, BROWN.2, 255);
        let stroke = Stroke { width: 1.5, ..Stroke::default() };
        canvas
            .pixmap
            .stroke_path(&circle, &paint, &stroke, Transform::identity(), None);
    } else {
        paint.set_color_rgba8(BROWN.0, BROWN.1, BROWN.2, 255);
        canvas.pixmap.fill_path(
            &circle,
            &paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

/// Draws a thin straight line between two points.
fn draw_line(canvas: &mut Canvas, from: (f64, f64), to: (f64, f64)) {
    let mut builder = PathBuilder::new();
    builder.move_to(from.0 as f32, from.1 as f32);
    builder.line_to(to.0 as f32, to.1 as f32);
    let Some(line) = builder.finish() else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color_rgba8(BROWN.0, BROWN.1, BROWN.2, 255);
    let stroke = Stroke { width: 1.0, ..Stroke::default() };
    canvas
        .pixmap
        .stroke_path(&line, &paint, &stroke, Transform::identity(), None);
}

/// Draws the axis-aligned rectangle `(x, y, w, h)` as a dashed guide.
fn draw_box(canvas: &mut Canvas, (x, y, w, h): (f64, f64, f64, f64)) {
    let rect = format!("M {x} {y} h {w} v {h} h {} Z", -w);
    draw_dashed(canvas, &rect, BROWN, 1.2);
}

/// Fits `path` into the box `(x, y, w, h)`, keeping its aspect ratio.
fn fit(path: &str, (x, y, w, h): (f64, f64, f64, f64)) -> String {
    let mut transformer = PathTransformer::new(path.into());
    transformer.inbox(InboxParameters {
        destination: BBox::from(format!("{x} {y} {w} {h}").as_str()),
        ..InboxParameters::default()
    });
    transformer.to_string()
}

/// Center of the cell whose top-left corner is `(cx, cy)`.
fn cell_center((cx, cy): (f32, f32)) -> (f64, f64) {
    (
        f64::from(cx) + f64::from(CELL_WIDTH) / 2.0,
        f64::from(cy) + f64::from(CELL_HEIGHT) / 2.0,
    )
}

/// Ferris fitted into a `SUBJECT_SIZE` square centered on `center`.
fn subject(center: (f64, f64)) -> String {
    let half = SUBJECT_SIZE / 2.0;
    fit(
        &ferris(),
        (center.0 - half, center.1 - half, SUBJECT_SIZE, SUBJECT_SIZE),
    )
}

/// Applies `op` to `path` about `center`.
fn about(path: &str, center: (f64, f64), op: impl FnOnce(&mut PathTransformer)) -> String {
    let mut transformer = PathTransformer::new(path.into());
    transformer.translate(-center.0, -center.1);
    op(&mut transformer);
    transformer.translate(center.0, center.1);
    transformer.to_string()
}

/// A strip where each cell draws Ferris dashed, then the result of `op`
/// with that cell's value filled on top.
fn transform_strip<V: Copy>(
    out_dir: &Path,
    name: &str,
    subtitle: &str,
    values: &[(V, &str)],
    op: impl Fn(&mut PathTransformer, V),
) {
    let mut canvas = Canvas::new(LAYOUT, name, subtitle, values.len());
    for (i, &(value, label)) in values.iter().enumerate() {
        let center = cell_center(canvas.cell(i));
        let original = subject(center);
        let result = about(&original, center, |transformer| op(transformer, value));
        draw_ferris_ghost(&mut canvas, &original);
        draw_ferris(&mut canvas, &result);
        canvas.label(i, label);
    }
    canvas.save(out_dir, name);
}

/// Shows how `inbox` places Ferris into a wide destination box.
fn inbox_strip(
    out_dir: &Path,
    name: &str,
    subtitle: &str,
    values: &[(ScaleType, BoxAlignment, &str)],
) {
    let mut canvas = Canvas::new(LAYOUT, name, subtitle, values.len());
    for (i, &(scale_type, alignment, label)) in values.iter().enumerate() {
        let (cx, cy) = cell_center(canvas.cell(i));
        let (w, h) = (74.0, 104.0);
        let destination = (cx - w / 2.0, cy - h / 2.0, w, h);

        // Start from a small Ferris so the move-only cell stays readable
        let mut transformer = PathTransformer::new(fit(&ferris(), (0.0, 0.0, 40.0, 40.0)));
        transformer.inbox(InboxParameters {
            destination: BBox::from(
                format!("{} {} {} {}", destination.0, destination.1, w, h).as_str(),
            ),
            scale_type,
            alignment,
        });

        draw_ferris(&mut canvas, &transformer.to_string());
        draw_box(&mut canvas, destination);
        canvas.label(i, label);
    }
    canvas.save(out_dir, name);
}

fn to_box_strip(out_dir: &Path) {
    let angles = [0.0, 30.0, 60.0, 90.0];
    let mut canvas = Canvas::new(
        LAYOUT,
        "to_box",
        "the bounding box of the path, curves included",
        angles.len(),
    );
    for (i, angle) in angles.iter().enumerate() {
        let center = cell_center(canvas.cell(i));
        let rotated = about(&subject(center), center, |t| {
            t.rotate(*angle, 0.0, 0.0);
        });
        draw_ferris(&mut canvas, &rotated);

        let bbox = PathTransformer::new(rotated).to_box(None);
        let (min_x, min_y) = (bbox.min_x.unwrap(), bbox.min_y.unwrap());
        draw_box(&mut canvas, (min_x, min_y, bbox.width(), bbox.height()));
        canvas.label(i, &format!("rotated {angle}"));
    }
    canvas.save(out_dir, "to_box");
}

/// Draws the control points of every cubic curve in `path`, with handles
/// from the curve ends.
fn draw_cubic_controls(canvas: &mut Canvas, path: &str) {
    let segments: Vec<PathSegment> = PathParser::from(path).map(Result::unwrap).collect();
    for context in segments_with_context(&segments) {
        if let PathSegment::CurveTo { x1, y1, x2, y2, .. } = *context.segment {
            let (start, end) = (context.start, context.end);
            draw_line(canvas, (start.x, start.y), (x1, y1));
            draw_line(canvas, (x2, y2), (end.x, end.y));
            draw_dot(canvas, (x1, y1), true);
            draw_dot(canvas, (x2, y2), true);
            draw_dot(canvas, (end.x, end.y), false);
        }
    }
}

fn unarc_strip(out_dir: &Path) {
    // A tab shape: two lines joined by a half-circle arc
    let tab = "M 0 0 L 50 0 A 30 30 0 0 1 50 60 L 0 60 Z";
    let mut canvas = Canvas::new(
        LAYOUT,
        "unarc",
        "elliptical arcs become cubic curves (hollow: control points)",
        2,
    );

    let center = cell_center(canvas.cell(0));
    let shape = fit(tab, (center.0 - 40.0, center.1 - 30.0, 80.0, 60.0));
    draw_filled(&mut canvas, &shape);
    for context in segments_with_context(&parse(&shape)) {
        draw_dot(&mut canvas, (context.end.x, context.end.y), false);
    }
    canvas.label(0, "arc");

    let center = cell_center(canvas.cell(1));
    let shape = fit(tab, (center.0 - 40.0, center.1 - 30.0, 80.0, 60.0));
    let mut transformer = PathTransformer::new(shape);
    let unarced = transformer.abs().unarc().to_string();
    draw_filled(&mut canvas, &unarced);
    draw_cubic_controls(&mut canvas, &unarced);
    canvas.label(1, "after unarc");

    canvas.save(out_dir, "unarc");
}

fn parse(path: &str) -> Vec<PathSegment> {
    PathParser::from(path).map(Result::unwrap).collect()
}

/// Draws each smooth segment's implied control point hollow, with a dashed
/// line to the control point it mirrors.
fn draw_implied_controls(canvas: &mut Canvas, path: &str) {
    let segments = parse(path);
    let mut previous_control: Option<(f64, f64)> = None;
    for context in segments_with_context(&segments) {
        let (start, end) = (
            (context.start.x, context.start.y),
            (context.end.x, context.end.y),
        );
        match *context.segment {
            PathSegment::CurveTo { x1, y1, x2, y2, .. } => {
                draw_dot(canvas, (x1, y1), false);
                draw_dot(canvas, (x2, y2), false);
                previous_control = Some((x2, y2));
            }
            PathSegment::Quadratic { x1, y1, .. } => {
                draw_dot(canvas, (x1, y1), false);
                previous_control = Some((x1, y1));
            }
            PathSegment::SmoothCurveTo { x2, y2, .. } => {
                let implied = context.implied_control.unwrap();
                if let Some(mirrored) = previous_control {
                    let guide = format!(
                        "M {} {} L {} {}",
                        mirrored.0, mirrored.1, implied.x, implied.y
                    );
                    draw_dashed(canvas, &guide, BROWN, 1.2);
                }
                draw_dot(canvas, (implied.x, implied.y), true);
                draw_dot(canvas, (x2, y2), false);
                previous_control = Some((x2, y2));
            }
            PathSegment::SmoothQuadratic { .. } => {
                let implied = context.implied_control.unwrap();
                if let Some(mirrored) = previous_control {
                    let guide = format!(
                        "M {} {} L {} {}",
                        mirrored.0, mirrored.1, implied.x, implied.y
                    );
                    draw_dashed(canvas, &guide, BROWN, 1.2);
                }
                draw_dot(canvas, (implied.x, implied.y), true);
                previous_control = Some((implied.x, implied.y));
            }
            _ => previous_control = None,
        }
        let _ = (start, end);
    }
}

fn unshort_strip(out_dir: &Path) {
    let cases = [
        ("M 0 40 C 0 0 40 0 40 40 S 80 80 80 40", "s after c"),
        ("M 0 30 Q 20 0 40 30 T 80 30 T 120 30", "t after q"),
    ];
    let mut canvas = Canvas::new(
        LAYOUT,
        "unshort",
        "smooth segments get their implied control point (hollow)",
        cases.len(),
    );
    for (i, (path, label)) in cases.iter().enumerate() {
        let center = cell_center(canvas.cell(i));
        let shape = fit(path, (center.0 - 60.0, center.1 - 35.0, 120.0, 70.0));
        draw_outline(&mut canvas, &shape);
        draw_implied_controls(&mut canvas, &shape);
        canvas.label(i, label);
    }
    canvas.save(out_dir, "unshort");
}

fn segments_with_context_strip(out_dir: &Path) {
    let cases = [
        ("m 0 0 l 60 0 l 0 40 h -60 z", "relative lines"),
        ("m 0 40 c 0 -40 40 -40 40 0 s 40 40 40 0", "relative curves"),
    ];
    let mut canvas = Canvas::new(
        LAYOUT,
        "segments_with_context",
        "absolute end point of every segment, numbered in order",
        cases.len(),
    );
    for (i, (path, label)) in cases.iter().enumerate() {
        let center = cell_center(canvas.cell(i));
        let shape = fit(path, (center.0 - 50.0, center.1 - 30.0, 100.0, 60.0));
        // Keep the relative commands: fit only wraps them in a transform
        draw_outline(&mut canvas, &shape);
        draw_numbered_ends(&mut canvas, &shape, false);
        canvas.label(i, label);
    }
    canvas.save(out_dir, "segments_with_context");
}

/// Draws a dot on the end point of every segment of `path`, numbered in
/// order. With `hollow_start`, the move that starts the path is hollow.
fn draw_numbered_ends(canvas: &mut Canvas, path: &str, hollow_start: bool) {
    let segments = parse(path);
    for context in segments_with_context(&segments) {
        let end = (context.end.x, context.end.y);
        draw_dot(canvas, end, false);
        // A close path ends on the subpath start, so label it below
        // instead of over the move's label
        let (dx, dy) = match context.segment {
            PathSegment::ClosePath { .. } => (5.0, 5.0),
            _ => (5.0, -13.0),
        };
        let index = context.index.to_string();
        canvas.text(&index, end.0 as f32 + dx, end.1 as f32 + dy, 1.0, BROWN);
    }
    // Drawn last, so a close path's dot on the same point does not cover it
    let start = segments_with_context(&segments)
        .next()
        .map(|start| start.end);
    if let (true, Some(start)) = (hollow_start, start) {
        draw_dot(canvas, (start.x, start.y), true);
    }
}

fn reverse_strip(out_dir: &Path) {
    let mut canvas = Canvas::new(
        LAYOUT,
        "reverse",
        "flips the drawing direction. with nonzero fill, holes need it",
        4,
    );

    // An open path, drawn forward and reversed
    let open = "M 0 40 C 0 0 40 0 40 40 S 80 80 80 40 L 110 40";
    for (i, reversed, label) in [(0, false, "open"), (1, true, "reversed")] {
        let center = cell_center(canvas.cell(i));
        let mut shape = fit(open, (center.0 - 55.0, center.1 - 30.0, 110.0, 60.0));
        if reversed {
            shape = write_path(reverse(parse(&shape)), &WriteOptions::default());
        }
        draw_outline(&mut canvas, &shape);
        draw_direction(&mut canvas, &shape);
        canvas.label(i, label);
    }

    // A frame: the inner square only cuts a hole once it runs the other way
    let outer = "M 0 0 L 90 0 L 90 70 L 0 70 Z";
    let inner = "M 25 20 L 65 20 L 65 50 L 25 50 Z";
    for (i, reversed, label) in [(2, false, "same direction"), (3, true, "inner reversed")] {
        let center = cell_center(canvas.cell(i));
        let origin = (center.0 - 45.0, center.1 - 35.0);
        let place = |path: &str| {
            let mut transformer = PathTransformer::new(path.into());
            transformer.translate(origin.0, origin.1);
            transformer.to_string()
        };
        let mut hole = place(inner);
        if reversed {
            hole = write_path(reverse(parse(&hole)), &WriteOptions::default());
        }
        let frame = format!("{} {}", place(outer), hole);

        fill_nonzero(&mut canvas, &frame);
        draw_outline(&mut canvas, &frame);
        draw_direction(&mut canvas, &frame);
        canvas.label(i, label);
    }

    canvas.save(out_dir, "reverse");
}

fn flip_strip(out_dir: &Path) {
    // Ferris is symmetric, so tilt him first to make a left-right flip show
    let flips = [
        ("tilted 30", false, false),
        ("flip_x", true, false),
        ("flip_y", false, true),
        ("flip_x, flip_y", true, true),
    ];
    let mut canvas = Canvas::new(
        LAYOUT,
        "flip",
        "mirrors the path in place, about the center of its box",
        flips.len(),
    );
    for (i, &(label, flip_x, flip_y)) in flips.iter().enumerate() {
        let center = cell_center(canvas.cell(i));
        let tilted = about(&subject(center), center, |t| {
            t.rotate(30.0, 0.0, 0.0);
        });
        let mut transformer = PathTransformer::new(tilted.clone());
        if flip_x {
            transformer.flip_x();
        }
        if flip_y {
            transformer.flip_y();
        }
        if i > 0 {
            draw_ferris_ghost(&mut canvas, &tilted);
        }
        draw_ferris(&mut canvas, &transformer.to_string());
        canvas.label(i, label);
    }
    canvas.save(out_dir, "flip");
}

fn shapes_strip(out_dir: &Path) {
    let mut canvas = Canvas::new(
        LAYOUT,
        "shapes",
        "svg shapes as the paths svg 2 defines (hollow: start, arrows: direction)",
        6,
    );
    for i in 0..6 {
        let (cx, cy) = cell_center(canvas.cell(i));
        let (shape, label) = match i {
            0 => (
                Shape::Rect {
                    x: cx - 50.0,
                    y: cy - 35.0,
                    width: 100.0,
                    height: 70.0,
                    rx: None,
                    ry: None,
                },
                "rect",
            ),
            1 => (
                Shape::Rect {
                    x: cx - 50.0,
                    y: cy - 35.0,
                    width: 100.0,
                    height: 70.0,
                    rx: Some(18.0),
                    ry: None,
                },
                "rect rx 18",
            ),
            2 => (Shape::Circle { cx, cy, r: 38.0 }, "circle"),
            3 => (
                Shape::Ellipse { cx, cy, rx: Some(55.0), ry: Some(32.0) },
                "ellipse",
            ),
            4 => (
                Shape::Polygon(vec![
                    (cx - 50.0, cy + 30.0).into(),
                    (cx, cy - 38.0).into(),
                    (cx + 50.0, cy + 30.0).into(),
                ]),
                "polygon",
            ),
            _ => (
                Shape::Polyline(vec![
                    (cx - 55.0, cy + 25.0).into(),
                    (cx - 20.0, cy - 30.0).into(),
                    (cx + 15.0, cy + 25.0).into(),
                    (cx + 55.0, cy - 30.0).into(),
                ]),
                "polyline",
            ),
        };
        let path = write_path(shape.to_path(), &WriteOptions::default());
        if matches!(shape, Shape::Polyline(_)) {
            draw_outline(&mut canvas, &path);
        } else {
            draw_filled(&mut canvas, &path);
        }
        draw_direction(&mut canvas, &path);
        let start = segments_with_context(&parse(&path)).next().unwrap().end;
        draw_dot(&mut canvas, (start.x, start.y), true);
        canvas.label(i, label);
    }
    canvas.save(out_dir, "shapes");
}

fn measure_strip(out_dir: &Path) {
    let mut canvas = Canvas::new(
        LAYOUT,
        "measure",
        "points at equal lengths along the path, pointing along its tangent",
        3,
    );
    for i in 0..3 {
        let center = cell_center(canvas.cell(i));
        let (path, label, count) = match i {
            0 => {
                let rect = Shape::Rect {
                    x: center.0 - 55.0,
                    y: center.1 - 38.0,
                    width: 110.0,
                    height: 76.0,
                    rx: Some(22.0),
                    ry: None,
                };
                (
                    write_path(rect.to_path(), &WriteOptions::default()),
                    "rect with arcs",
                    16,
                )
            }
            1 => {
                let ferris = fit(&ferris(), (center.0 - 62.0, center.1 - 46.0, 124.0, 92.0));
                (ferris_parts(&ferris)[3].clone(), "ferris body", 18)
            }
            _ => (
                fit(
                    "M 0 40 C 0 0 40 0 40 40 A 20 20 0 0 0 80 40 Q 100 0 120 40",
                    (center.0 - 60.0, center.1 - 25.0, 120.0, 50.0),
                ),
                "curves and an arc",
                14,
            ),
        };
        draw_outline(&mut canvas, &path);
        let measure = PathMeasure::new(parse(&path));
        let step = measure.total_length() / f64::from(count);
        for k in 0..count {
            let length = step * (f64::from(k) + 0.5);
            let (Some(point), Some(tangent)) =
                (measure.point_at(length), measure.tangent_at(length))
            else {
                continue;
            };
            draw_arrowhead(&mut canvas, (point.x, point.y), (tangent.x, tangent.y));
        }
        canvas.label(i, label);
    }
    canvas.save(out_dir, "measure");
}

fn nearest_strip(out_dir: &Path) {
    let curve = "M 0 40 C 0 0 40 0 40 40 A 20 20 0 0 0 80 40 Q 100 0 120 40";
    let mut canvas = Canvas::new(
        LAYOUT,
        "nearest",
        "nearest point on the path, and is_point_in_stroke (filled)",
        2,
    );

    // Lines from points around the path to their nearest points on it
    let center = cell_center(canvas.cell(0));
    let path = fit(curve, (center.0 - 60.0, center.1 - 25.0, 120.0, 50.0));
    draw_outline(&mut canvas, &path);
    let measure = PathMeasure::new(parse(&path));
    for (dx, dy) in [
        (-50.0, -45.0),
        (-15.0, 40.0),
        (0.0, -40.0),
        (25.0, 45.0),
        (55.0, -40.0),
    ] {
        let target = Point2D::new(center.0 + dx, center.1 + dy);
        let nearest = measure.nearest(target).unwrap();
        let guide = format!(
            "M {} {} L {} {}",
            target.x, target.y, nearest.point.x, nearest.point.y
        );
        draw_dashed(&mut canvas, &guide, BROWN, 1.2);
        draw_dot(&mut canvas, (target.x, target.y), true);
        draw_dot(&mut canvas, (nearest.point.x, nearest.point.y), false);
    }
    canvas.label(0, "nearest");

    // A grid of points, filled where a 14 wide stroke covers them
    let center = cell_center(canvas.cell(1));
    let path = fit(curve, (center.0 - 60.0, center.1 - 25.0, 120.0, 50.0));
    let measure = PathMeasure::new(parse(&path));
    for row in 0..11 {
        for column in 0..16 {
            let x = center.0 - 75.0 + f64::from(column) * 10.0;
            let y = center.1 - 50.0 + f64::from(row) * 10.0;
            let inside = measure.is_point_in_stroke(Point2D::new(x, y), 14.0);
            draw_small_dot(&mut canvas, (x, y), inside);
        }
    }
    draw_outline(&mut canvas, &path);
    canvas.label(1, "stroke width 14");

    canvas.save(out_dir, "nearest");
}

/// A small grid dot: filled when `on`, a faint ring otherwise.
fn draw_small_dot(canvas: &mut Canvas, (x, y): (f64, f64), on: bool) {
    let Some(circle) = PathBuilder::from_circle(x as f32, y as f32, if on { 2.6 } else { 1.8 })
    else {
        return;
    };
    let mut paint = Paint::default();
    if on {
        paint.set_color_rgba8(GHOST_COLOR.0, GHOST_COLOR.1, GHOST_COLOR.2, 255);
        canvas.pixmap.fill_path(
            &circle,
            &paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );
    } else {
        paint.set_color_rgba8(CREAM.0, CREAM.1, CREAM.2, 255);
        let stroke = Stroke { width: 0.8, ..Stroke::default() };
        canvas
            .pixmap
            .stroke_path(&circle, &paint, &stroke, Transform::identity(), None);
    }
}

fn fill_strip(out_dir: &Path) {
    let mut canvas = Canvas::new(
        LAYOUT,
        "contains",
        "points inside the path by fill rule (filled), and its area",
        3,
    );

    // A five-pointed star drawn in one stroke crosses itself, so the fill
    // rules disagree about the pentagon in its middle
    for (i, rule, label) in [
        (0, FillRule::NonZero, "nonzero"),
        (1, FillRule::EvenOdd, "evenodd"),
    ] {
        let center = cell_center(canvas.cell(i));
        let star: String = (0..5)
            .map(|k| {
                let angle = (-90.0 + 144.0 * f64::from(k)).to_radians();
                let command = if k == 0 { "M" } else { "L" };
                format!(
                    "{command} {} {} ",
                    center.0 + 52.0 * angle.cos(),
                    center.1 + 4.0 + 52.0 * angle.sin()
                )
            })
            .collect::<String>()
            + "Z";
        let measure = PathMeasure::new(parse(&star));
        draw_point_grid(&mut canvas, center, |point| measure.contains(point, rule));
        draw_outline(&mut canvas, &star);
        canvas.label(i, label);
    }

    let center = cell_center(canvas.cell(2));
    let circle = write_path(
        Shape::Circle { cx: center.0, cy: center.1, r: 40.0 }.to_path(),
        &WriteOptions::default(),
    );
    let measure = PathMeasure::new(parse(&circle));
    draw_point_grid(&mut canvas, center, |point| {
        measure.contains(point, FillRule::NonZero)
    });
    draw_outline(&mut canvas, &circle);
    canvas.label(2, &format!("r 40, area: {:.2}", measure.area()));

    canvas.save(out_dir, "contains");
}

/// A grid of small dots around `center`, filled where `inside` holds.
fn draw_point_grid(canvas: &mut Canvas, center: (f64, f64), inside: impl Fn(Point2D<f64>) -> bool) {
    for row in 0..12 {
        for column in 0..16 {
            let x = center.0 - 75.0 + f64::from(column) * 10.0;
            let y = center.1 - 55.0 + f64::from(row) * 10.0;
            draw_small_dot(canvas, (x, y), inside(Point2D::new(x, y)));
        }
    }
}

/// Strokes `path` exactly, `width` wide, with round ends.
fn draw_stroke(canvas: &mut Canvas, path: &str, color: (u8, u8, u8), width: f32) {
    let Some(tiny_path) = tiny_path(path) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color_rgba8(color.0, color.1, color.2, 255);
    let stroke = Stroke {
        width,
        line_cap: tiny_skia::LineCap::Round,
        line_join: tiny_skia::LineJoin::Round,
        ..Stroke::default()
    };
    canvas
        .pixmap
        .stroke_path(&tiny_path, &paint, &stroke, Transform::identity(), None);
}

fn crop_strip(out_dir: &Path) {
    let curve = "M 0 40 C 0 0 40 0 40 40 A 20 20 0 0 0 80 40 Q 100 0 120 40";
    let mut canvas = Canvas::new(
        LAYOUT,
        "crop",
        "parts by fraction of the length, and the path in straight lines",
        4,
    );
    let cell = |canvas: &mut Canvas, i: usize| {
        let center = cell_center(canvas.cell(i));
        fit(curve, (center.0 - 60.0, center.1 - 25.0, 120.0, 50.0))
    };

    let path = cell(&mut canvas, 0);
    let measure = PathMeasure::new(parse(&path));
    let part = measure.crop(measure.total_length() * 0.25, measure.total_length() * 0.7);
    draw_dashed(&mut canvas, &path, BROWN, 1.0);
    draw_stroke(
        &mut canvas,
        &write_path(&part, &WriteOptions::default()),
        BROWN,
        3.0,
    );
    canvas.label(0, "crop 0.25 to 0.7");

    let path = cell(&mut canvas, 1);
    let measure = PathMeasure::new(parse(&path));
    let (before, after) = measure.split_at(measure.total_length() * 0.4);
    draw_stroke(
        &mut canvas,
        &write_path(&before, &WriteOptions::default()),
        BROWN,
        3.0,
    );
    draw_stroke(
        &mut canvas,
        &write_path(&after, &WriteOptions::default()),
        GHOST_COLOR,
        3.0,
    );
    canvas.label(1, "split_at 0.4");

    for (i, tolerance) in [(2, 2.0), (3, 0.3)] {
        let path = cell(&mut canvas, i);
        let polygon = PathMeasure::new(parse(&path)).flatten(tolerance);
        draw_dashed(&mut canvas, &path, BROWN, 1.0);
        let written = write_path(&polygon, &WriteOptions::default());
        draw_stroke(&mut canvas, &written, BROWN, 1.5);
        for context in segments_with_context(&polygon) {
            draw_small_dot(&mut canvas, (context.end.x, context.end.y), true);
        }
        canvas.label(i, &format!("flatten {tolerance}"));
    }

    canvas.save(out_dir, "crop");
}

fn intersections_strip(out_dir: &Path) {
    let mut canvas = Canvas::new(
        LAYOUT,
        "intersections",
        "the points where two paths meet (hollow), arcs and curves included",
        3,
    );
    let cells: [(&str, &str); 3] = [
        (
            "M -60 0 C -30 -70 -10 70 0 0 S 30 -70 60 0",
            "M 38 0 A 38 38 0 0 1 -38 0 A 38 38 0 0 1 38 0",
        ),
        (
            "M 55 0 A 55 25 20 0 1 -55 0 A 55 25 20 0 1 55 0",
            "M 55 0 A 55 25 -35 0 1 -55 0 A 55 25 -35 0 1 55 0",
        ),
        ("", "M -65 30 L 65 -32"),
    ];
    let labels = ["curve and circle", "two ellipses", "ferris and a line"];
    for (i, (first, second)) in cells.iter().enumerate() {
        let center = cell_center(canvas.cell(i));
        let place = |path: &str| {
            let mut transformer = PathTransformer::new(path.into());
            transformer.translate(center.0, center.1);
            transformer.to_string()
        };
        let first = if first.is_empty() {
            let ferris = fit(&ferris(), (center.0 - 60.0, center.1 - 44.0, 120.0, 88.0));
            ferris_parts(&ferris)[3].clone()
        } else {
            place(first)
        };
        let second = place(second);
        draw_stroke(&mut canvas, &first, BROWN, 1.5);
        draw_stroke(&mut canvas, &second, GHOST_COLOR, 1.5);
        let meets =
            PathMeasure::new(parse(&first)).intersections(&PathMeasure::new(parse(&second)));
        for meet in meets {
            draw_dot(&mut canvas, (meet.point.x, meet.point.y), true);
        }
        canvas.label(i, labels[i]);
    }
    canvas.save(out_dir, "intersections");
}

fn split_subpaths_strip(out_dir: &Path) {
    // Ferris is one path whose parts are subpaths: 3 is the body, 1 the
    // legs on one side and 6 an eye
    let cases = [
        (None, "whole path"),
        (Some(3), "subpath 3: body"),
        (Some(1), "subpath 1: legs"),
        (Some(6), "subpath 6: eye"),
    ];
    let mut canvas = Canvas::new(
        LAYOUT,
        "split_subpaths",
        "each subpath of a path as a path of its own",
        cases.len(),
    );
    for (i, (subpath, label)) in cases.iter().enumerate() {
        let center = cell_center(canvas.cell(i));
        let ferris = fit(&ferris(), (center.0 - 60.0, center.1 - 44.0, 120.0, 88.0));
        match subpath {
            None => draw_ferris(&mut canvas, &ferris),
            Some(index) => {
                draw_dashed(&mut canvas, &ferris, BROWN, 0.8);
                let part = &split_subpaths(parse(&ferris))[*index];
                let data = write_path(part, &WriteOptions::default());
                if matches!(FERRIS_PARTS[*index].0, Part::Eye) {
                    draw_solid(&mut canvas, &data);
                } else {
                    draw_filled(&mut canvas, &data);
                }
            }
        }
        canvas.label(i, label);
    }
    canvas.save(out_dir, "split_subpaths");
}

/// Strokes `path` exactly with a wide pen, so its corners and ends show.
fn draw_wide_stroke(canvas: &mut Canvas, path: &str) {
    let Some(tiny_path) = tiny_path(path) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color_rgba8(BROWN.0, BROWN.1, BROWN.2, 255);
    let stroke = Stroke {
        width: 16.0,
        line_join: tiny_skia::LineJoin::Miter,
        line_cap: tiny_skia::LineCap::Butt,
        ..Stroke::default()
    };
    canvas
        .pixmap
        .stroke_path(&tiny_path, &paint, &stroke, Transform::identity(), None);
}

fn is_closed_strip(out_dir: &Path) {
    let cases = [
        ("M 0 0 L 70 0 L 70 60 L 0 60 Z", "with z"),
        ("M 0 0 L 70 0 L 70 60 L 0 60 L 0 0", "no z"),
    ];
    let mut canvas = Canvas::new(
        LAYOUT,
        "is_closed",
        "both return to the top left corner, only z joins it",
        cases.len(),
    );
    for (i, (path, label)) in cases.iter().enumerate() {
        let center = cell_center(canvas.cell(i));
        let mut shape = PathTransformer::new((*path).into());
        shape.translate(center.0 - 35.0, center.1 - 30.0);
        let shape = shape.to_string();
        draw_wide_stroke(&mut canvas, &shape);
        let closed = if is_closed(parse(&shape)) {
            "true"
        } else {
            "false"
        };
        canvas.label(i, &format!("{label}: {closed}"));
    }
    canvas.save(out_dir, "is_closed");
}

fn main() {
    let out_dir: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "..",
        "svg_path_ops",
        "assets",
        "ops",
    ]
    .iter()
    .collect();
    std::fs::create_dir_all(&out_dir).unwrap();
    let out = out_dir.as_path();

    transform_strip(
        out,
        "translate",
        "moves the path by x, y",
        &[
            ((0.0, 0.0), "0, 0"),
            ((30.0, 0.0), "30, 0"),
            ((0.0, 25.0), "0, 25"),
            ((-30.0, -25.0), "-30, -25"),
        ],
        |t, (x, y)| {
            t.translate(x, y);
        },
    );
    transform_strip(
        out,
        "rotate",
        "turns the path by degrees about a point",
        &[
            (0.0, "0"),
            (45.0, "45"),
            (90.0, "90"),
            (180.0, "180"),
            (270.0, "270"),
        ],
        |t, angle| {
            t.rotate(angle, 0.0, 0.0);
        },
    );
    transform_strip(
        out,
        "scale",
        "scales x and y separately",
        &[
            ((0.5, 0.5), "0.5, 0.5"),
            ((1.0, 1.0), "1, 1"),
            ((1.4, 1.4), "1.4, 1.4"),
            ((1.6, 0.7), "1.6, 0.7"),
            ((1.0, -1.0), "1, -1"),
        ],
        |t, (x, y)| {
            t.scale(x, y);
        },
    );
    transform_strip(
        out,
        "skew_x",
        "slants the path along x by degrees",
        &[
            (-30.0, "-30"),
            (-15.0, "-15"),
            (0.0, "0"),
            (15.0, "15"),
            (30.0, "30"),
        ],
        |t, angle| {
            t.skew_x(angle);
        },
    );
    transform_strip(
        out,
        "skew_y",
        "slants the path along y by degrees",
        &[
            (-30.0, "-30"),
            (-15.0, "-15"),
            (0.0, "0"),
            (15.0, "15"),
            (30.0, "30"),
        ],
        |t, angle| {
            t.skew_y(angle);
        },
    );

    let mid = BoxAlignment { x: Alignment::Mid, y: Alignment::Mid };
    inbox_strip(
        out,
        "inbox",
        "fits the path into a box (dashed) with a scale type",
        &[
            (ScaleType::Meet, mid, "meet"),
            (ScaleType::Slice, mid, "slice"),
            (ScaleType::Fit, mid, "fit"),
            (ScaleType::Move, mid, "move"),
        ],
    );
    inbox_strip(
        out,
        "inbox_alignment",
        "where inbox puts the path when the box is taller (meet)",
        &[
            (
                ScaleType::Meet,
                BoxAlignment { x: Alignment::Mid, y: Alignment::Min },
                "y min",
            ),
            (ScaleType::Meet, mid, "y mid"),
            (
                ScaleType::Meet,
                BoxAlignment { x: Alignment::Mid, y: Alignment::Max },
                "y max",
            ),
        ],
    );

    to_box_strip(out);
    unarc_strip(out);
    unshort_strip(out);
    segments_with_context_strip(out);
    flip_strip(out);
    measure_strip(out);
    nearest_strip(out);
    fill_strip(out);
    crop_strip(out);
    intersections_strip(out);
    shapes_strip(out);
    reverse_strip(out);
    split_subpaths_strip(out);
    is_closed_strip(out);
}
