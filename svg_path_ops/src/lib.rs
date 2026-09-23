// This crate is entirely safe
#![forbid(unsafe_code)]

//! Read, transform and write SVG path data.
//!
//! The crate works on [`PathSegment`] values from [`svgtypes`], and keeps them
//! as they are unless you ask otherwise: arcs stay arcs, relative commands
//! stay relative and shorthand commands stay shorthand. [`svgtypes`] and
//! [`euclid`] are re-exported, so you don't need matching dependencies of your
//! own.
//!
//! The reference images were drawn with
//! [rough_tiny_skia](https://github.com/orhanbalci/rough-rs/tree/main/rough_tiny_skia)'s
//! `path_ops_gallery` example, using Ferris the crab by Karen Rustad Tölva
//! (CC0, <https://rustacean.net>).
//!
//! ## 📦 Cargo.toml
//!
//! ```toml
//! [dependencies]
//! svg_path_ops = "0.11"
//! ```
//!
//! ## 🔧 Usage
//!
//! ### Transforming a path
//!
//! [`PathTransformer`] applies transforms in the order
//! they are added, and writes the result with its `Display` implementation.
//! In the images below the original path is dashed in purple.
//!
//! ![translate](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/translate.png)
//!
//! ![rotate](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/rotate.png)
//!
//! ![scale](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/scale.png)
//!
//! ![skew_x](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/skew_x.png)
//!
//! ![skew_y](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/skew_y.png)
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//!
//! let mut path = PathTransformer::parse("M 10 10 L 50 10 L 30 40 Z")?;
//! path.scale(2.0, 2.0).translate(10.0, 0.0);
//! assert_eq!(path.to_string(), "M 30 20 L 110 20 L 70 80 Z");
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! It also takes the value of an SVG `transform` attribute:
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//!
//! let mut path = PathTransformer::parse("M 10 10 L 50 10")?;
//! path.transform("translate(5 5) scale(2)".into());
//! assert_eq!(path.to_string(), "M 25 25 L 105 25");
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! [`flip_x`] and [`flip_y`] mirror a path in place, about the center of its
//! bounding box:
//!
//! ![flip](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/flip.png)
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//!
//! let mut triangle = PathTransformer::parse("M 0 0 L 16 0 L 8 16")?;
//! assert_eq!(triangle.flip_y().to_string(), "M 0 16 L 16 16 L 8 0");
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! ### Writing path data
//!
//! [`WriteOptions`] rounds numbers and writes compact output. Use it with
//! [`to_string_with`], or with
//! [`write_path`] for any sequence of segments.
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//! use svg_path_ops::WriteOptions;
//!
//! let mut path = PathTransformer::parse("M 10 10 L 50 10")?;
//! path.rotate(90.0, 10.0, 10.0);
//! assert_eq!(path.to_string(), "M 10 10 L 10.000000000000004 50");
//!
//! let rounded = WriteOptions { precision: Some(3), ..WriteOptions::default() };
//! assert_eq!(path.to_string_with(&rounded), "M 10 10 L 10 50");
//!
//! let compact = WriteOptions { precision: Some(3), compact: true };
//! assert_eq!(path.to_string_with(&compact), "M10 10 10 50");
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! [`optimize`] goes further and rewrites each segment in its shortest
//! form: relative or absolute, `H`/`V` for straight lines, `S`/`T` for
//! mirrored curves, lines for straight curves and arcs for curves that
//! follow a circle, dropping segments that draw nothing. Relative
//! coordinates are taken between rounded absolute points, so rounding does
//! not drift along a path:
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//! use svg_path_ops::WriteOptions;
//!
//! let mut path = PathTransformer::parse(
//!     "M 100 100 L 110 100 L 110 110 C 110 120 120 120 120 110 \
//!      C 120 100 130 100 130 110 L 100 100 Z",
//! )?;
//! let options = WriteOptions { precision: Some(2), compact: true };
//! assert_eq!(
//!     path.optimize(Some(2)).to_string_with(&options),
//!     "M100 100h10v10c0 10 10 10 10 0s10-10 10 0z"
//! );
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! ### Converting commands
//!
//! [`PathTransformer`] converts between absolute and
//! relative commands, expands shorthand commands and replaces arcs with cubic
//! curves:
//!
//! ![unarc](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/unarc.png)
//!
//! ![unshort](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/unshort.png)
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//!
//! let mut path = PathTransformer::parse("M 10 10 l 40 0 l -20 30 z")?;
//! assert_eq!(path.abs().to_string(), "M 10 10 L 50 10 L 30 40 Z");
//! assert_eq!(path.rel().to_string(), "M 10 10 l 40 0 l -20 30 z");
//!
//! let mut smooth = PathTransformer::parse("M 0 0 C 10 0 20 10 30 10 S 50 20 60 20")?;
//! assert_eq!(
//!     smooth.unshort().to_string(),
//!     "M 0 0 C 10 0 20 10 30 10 C 40 10 50 20 60 20"
//! );
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! [`normalize`] reduces absolute segments to move, line, cubic curve and
//! close commands, for targets that support nothing else:
//!
//! ```
//! use svg_path_ops::svgtypes::PathParser;
//! use svg_path_ops::{normalize, write_path, PathSegment, WriteOptions};
//!
//! let segments: Vec<PathSegment> =
//!     PathParser::from("M 10 10 H 50 Q 50 40 20 40 Z").collect::<Result<_, _>>()?;
//! let normalized: Vec<PathSegment> = normalize(segments.iter()).collect();
//!
//! let options = WriteOptions { precision: Some(2), ..WriteOptions::default() };
//! assert_eq!(
//!     write_path(&normalized, &options),
//!     "M 10 10 L 50 10 C 50 30 40 40 20 40 Z"
//! );
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! ### Converting shapes
//!
//! [`Shape`] turns the SVG basic shapes into the paths SVG 2 defines for
//! them, starting where a browser starts and going the same way, so markers
//! and dashes land in the same places:
//!
//! ![shapes](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/shapes.png)
//!
//! ```
//! use svg_path_ops::shapes::Shape;
//! use svg_path_ops::{write_path, WriteOptions};
//!
//! let circle = Shape::Circle { cx: 50.0, cy: 50.0, r: 10.0 };
//! assert_eq!(
//!     write_path(circle.to_path(), &WriteOptions::default()),
//!     "M 60 50 A 10 10 0 0 1 50 60 A 10 10 0 0 1 40 50 \
//!      A 10 10 0 0 1 50 40 A 10 10 0 0 1 60 50 Z"
//! );
//! ```
//!
//! ### Reversing a path
//!
//! [`reverse`] draws every subpath in the opposite direction, keeping
//! relative segments relative and arcs as arcs. The shape stays the same;
//! the direction matters for holes under the default nonzero fill rule, for
//! the order a pen plotter draws in, for stroke animations and for where
//! markers and text on a path go. Below, the inner square only cuts a hole
//! once it is reversed:
//!
//! ![reverse](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/reverse.png)
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//!
//! let mut path = PathTransformer::parse("M 0 0 L 10 0 l 0 10 A 5 5 0 0 1 0 10 Z")?;
//! assert_eq!(
//!     path.reverse().to_string(),
//!     "M 0 10 A 5 5 0 0 0 10 10 l 0 -10 L 0 0 Z"
//! );
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! ### Subpaths
//!
//! A path can hold several subpaths, each started by a move.
//! [`split_subpaths`] returns them as paths of their own, and [`is_closed`]
//! tells whether every subpath ends with a close path:
//!
//! ![split_subpaths](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/split_subpaths.png)
//!
//! ```
//! use svg_path_ops::svgtypes::PathParser;
//! use svg_path_ops::{is_closed, split_subpaths, write_path, WriteOptions};
//!
//! let segments: Vec<_> =
//!     PathParser::from("M 0 0 h 10 v 10 z m 20 0 h 10").collect::<Result<_, _>>()?;
//! assert!(!is_closed(&segments));
//!
//! let subpaths = split_subpaths(&segments);
//! assert!(is_closed(&subpaths[0]));
//! // The relative move depended on the first subpath, so it becomes absolute
//! assert_eq!(
//!     write_path(&subpaths[1], &WriteOptions::default()),
//!     "M 20 0 h 10"
//! );
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! Returning to the start is not the same as closing: without a close path
//! the corner where the path starts gets two line ends instead of a join.
//!
//! ![is_closed](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/is_closed.png)
//!
//! ### Bounding boxes
//!
//! [`to_box`] measures a path, and [`inbox`] fits it into a box:
//!
//! ![to_box](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/to_box.png)
//!
//! ![inbox](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/inbox.png)
//!
//! ![inbox_alignment](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/inbox_alignment.png)
//!
//! ```
//! use svg_path_ops::bbox::{BBox, InboxParameters};
//! use svg_path_ops::pt::PathTransformer;
//!
//! let mut path = PathTransformer::parse("M 10 10 L 50 10 L 30 40 Z")?;
//! let bbox = path.to_box(None);
//! assert_eq!((bbox.width(), bbox.height()), (40.0, 30.0));
//!
//! path.inbox(InboxParameters {
//!     destination: BBox::from("0 0 100 100"),
//!     ..InboxParameters::default()
//! });
//! assert_eq!(path.to_string(), "M 0 12.5 L 100 12.5 L 50 87.5 Z");
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! ### Walking segments
//!
//! [`segments_with_context`] gives each segment its absolute start and end
//! points, so relative and shorthand segments can be handled without
//! tracking the current point:
//!
//! ![segments_with_context](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/segments_with_context.png)
//!
//! ```
//! use svg_path_ops::euclid::default::Point2D;
//! use svg_path_ops::segments_with_context;
//! use svg_path_ops::svgtypes::PathParser;
//!
//! let segments: Vec<_> =
//!     PathParser::from("M 10 10 l 5 0 s 5 5 10 0").collect::<Result<_, _>>()?;
//! let last = segments_with_context(&segments).last().unwrap();
//!
//! assert_eq!(last.start, Point2D::new(15.0, 10.0));
//! assert_eq!(last.end, Point2D::new(25.0, 10.0));
//! assert_eq!(last.implied_control, Some(Point2D::new(15.0, 10.0)));
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! ### Invalid path data
//!
//! [`PathTransformer::parse`] returns an error for invalid path data.
//! [`PathTransformer::new`] keeps the segments before the first error instead, as SVG renderers do:
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//!
//! assert!(PathTransformer::parse("M 10 10 L 20 20 L 5").is_err());
//!
//! let lenient = PathTransformer::new("M 10 10 L 20 20 L 5".into());
//! assert_eq!(lenient.to_string(), "M 10 10 L 20 20");
//! ```
//!
//! [`PathSegment`]: https://docs.rs/svgtypes/0.16/svgtypes/enum.PathSegment.html
//! [`svgtypes`]: https://docs.rs/svgtypes/0.16
//! [`euclid`]: https://docs.rs/euclid/0.22
//! [`PathTransformer`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html
//! [`PathTransformer::parse`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.parse
//! [`PathTransformer::new`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.new
//! [`to_string_with`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.to_string_with
//! [`to_box`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.to_box
//! [`inbox`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.inbox
//! [`WriteOptions`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.WriteOptions.html
//! [`write_path`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.write_path.html
//! [`normalize`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.normalize.html
//! [`segments_with_context`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.segments_with_context.html
//! [`reverse`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.reverse.html
//! [`optimize`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.optimize.html
//! [`Shape`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/shapes/enum.Shape.html
//! [`flip_x`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.flip_x
//! [`flip_y`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.flip_y
//! [`split_subpaths`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.split_subpaths.html
//! [`is_closed`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.is_closed.html

pub(crate) mod a2c;
pub mod bbox;
mod context;
pub(crate) mod ellipse;
mod optimize;
pub mod pt;
mod reverse;
pub mod shapes;
mod subpaths;
mod write;

use std::borrow::Borrow;

use a2c::a2c;
pub use context::{segments_with_context, SegmentContext};
pub use optimize::optimize;
pub use reverse::reverse;
pub use subpaths::{is_closed, split_subpaths};
pub use svgtypes::PathSegment;
pub use write::{write_path, WriteOptions};
pub use {euclid, svgtypes};

/// Translates relative commands to absolute commands. All commands that use relative positions (lower-case ones),
/// turns into absolute position commands (upper-case ones).
pub fn absolutize(
    path_segments: impl Iterator<Item = impl Borrow<PathSegment>>,
) -> impl Iterator<Item = PathSegment> {
    let mut result = vec![];
    let (mut cx, mut cy, mut subx, mut suby) = (0.0, 0.0, 0.0, 0.0);
    for segment in path_segments {
        match *segment.borrow() {
            PathSegment::MoveTo { abs: true, x, y } => {
                result.push(*segment.borrow());
                cx = x;
                cy = y;
                subx = x;
                suby = y;
            }
            PathSegment::MoveTo { abs: false, x, y } => {
                cx += x;
                cy += y;
                result.push(PathSegment::MoveTo { abs: true, x: cx, y: cy });
                subx = cx;
                suby = cy;
            }
            PathSegment::LineTo { abs: true, x, y } => {
                result.push(*segment.borrow());
                cx = x;
                cy = y;
            }
            PathSegment::LineTo { abs: false, x, y } => {
                cx += x;
                cy += y;
                result.push(PathSegment::LineTo { abs: true, x: cx, y: cy });
            }
            PathSegment::CurveTo { abs: true, x1: _, y1: _, x2: _, y2: _, x, y } => {
                result.push(*segment.borrow());
                cx = x;
                cy = y;
            }
            PathSegment::CurveTo { abs: false, x1, y1, x2, y2, x, y } => {
                result.push(PathSegment::CurveTo {
                    abs: true,
                    x1: x1 + cx,
                    y1: y1 + cy,
                    x2: x2 + cx,
                    y2: y2 + cy,
                    x: x + cx,
                    y: y + cy,
                });
                cx += x;
                cy += y;
            }
            PathSegment::Quadratic { abs: true, x1: _, y1: _, x, y } => {
                result.push(*segment.borrow());
                cx = x;
                cy = y;
            }
            PathSegment::Quadratic { abs: false, x1, y1, x, y } => {
                result.push(PathSegment::Quadratic {
                    abs: true,
                    x1: x1 + cx,
                    y1: y1 + cy,
                    x: x + cx,
                    y: y + cy,
                });
                cx += x;
                cy += y;
            }
            PathSegment::EllipticalArc {
                abs: true,
                rx: _,
                ry: _,
                x_axis_rotation: _,
                large_arc: _,
                sweep: _,
                x,
                y,
            } => {
                result.push(*segment.borrow());
                cx = x;
                cy = y;
            }
            PathSegment::EllipticalArc {
                abs: false,
                rx,
                ry,
                x_axis_rotation,
                large_arc,
                sweep,
                x,
                y,
            } => {
                cx += x;
                cy += y;
                result.push(PathSegment::EllipticalArc {
                    abs: true,
                    rx,
                    ry,
                    x_axis_rotation,
                    large_arc,
                    sweep,
                    x: cx,
                    y: cy,
                });
            }
            PathSegment::HorizontalLineTo { abs: true, x } => {
                result.push(*segment.borrow());
                cx = x;
            }
            PathSegment::HorizontalLineTo { abs: false, x } => {
                cx += x;
                result.push(PathSegment::HorizontalLineTo { abs: true, x: cx });
            }
            PathSegment::VerticalLineTo { abs: true, y } => {
                result.push(*segment.borrow());
                cy = y;
            }
            PathSegment::VerticalLineTo { abs: false, y } => {
                cy += y;
                result.push(PathSegment::VerticalLineTo { abs: true, y: cy });
            }
            PathSegment::SmoothCurveTo { abs: true, x2: _, y2: _, x, y } => {
                result.push(*segment.borrow());
                cx = x;
                cy = y;
            }
            PathSegment::SmoothCurveTo { abs: false, x2, y2, x, y } => {
                result.push(PathSegment::SmoothCurveTo {
                    abs: true,
                    x2: x2 + cx,
                    y2: y2 + cy,
                    x: x + cx,
                    y: y + cy,
                });
                cx += x;
                cy += y;
            }
            PathSegment::SmoothQuadratic { abs: true, x, y } => {
                result.push(*segment.borrow());
                cx = x;
                cy = y;
            }
            PathSegment::SmoothQuadratic { abs: false, x, y } => {
                cx += x;
                cy += y;
                result.push(PathSegment::SmoothQuadratic { abs: true, x: cx, y: cy });
            }
            PathSegment::ClosePath { .. } => {
                result.push(PathSegment::ClosePath { abs: true });
                cx = subx;
                cy = suby;
            }
        }
    }

    result.into_iter()
}

/// Normalize takes a list of absolute segments and outputs a list of segments with only four commands: M, L, C, Z. So every segment is described as move, line, or a bezier curve (cubic).
/// This is useful when translating SVG paths to non SVG mediums - Canvas, or some other graphics platform. Most such platforms will support lines and bezier curves.
/// It also simplifies the cases to consider when modifying these segments.
pub fn normalize(
    path_segments: impl Iterator<Item = impl Borrow<PathSegment>>,
) -> impl Iterator<Item = PathSegment> {
    let mut out = vec![];

    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut subx = 0.0;
    let mut suby = 0.0;
    let mut lcx = 0.0;
    let mut lcy = 0.0;
    let mut last_type: Option<PathSegment> = None;

    for segment in path_segments {
        match *segment.borrow() {
            PathSegment::MoveTo { abs: true, x, y } => {
                out.push(*segment.borrow());
                cx = x;
                cy = y;
                subx = x;
                suby = y;
            }
            PathSegment::CurveTo { abs: true, x1: _, y1: _, x2, y2, x, y } => {
                out.push(*segment.borrow());
                cx = x;
                cy = y;
                lcx = x2;
                lcy = y2;
            }
            PathSegment::LineTo { abs: true, x, y } => {
                out.push(*segment.borrow());
                cx = x;
                cy = y;
            }
            PathSegment::HorizontalLineTo { abs: true, x } => {
                cx = x;
                out.push(PathSegment::LineTo { abs: true, x: cx, y: cy });
            }
            PathSegment::VerticalLineTo { abs: true, y } => {
                cy = y;
                out.push(PathSegment::LineTo { abs: true, x: cx, y: cy });
            }
            PathSegment::SmoothCurveTo { abs: true, x2, y2, x, y } => {
                let cx1;
                let cy1;
                if let Some(lt) = last_type {
                    if matches!(lt, PathSegment::CurveTo { .. })
                        || matches!(lt, PathSegment::SmoothCurveTo { .. })
                    {
                        cx1 = cx + (cx - lcx);
                        cy1 = cy + (cy - lcy);
                    } else {
                        cx1 = cx;
                        cy1 = cy;
                    }
                } else {
                    cx1 = cx;
                    cy1 = cy;
                }
                out.push(PathSegment::CurveTo { abs: true, x1: cx1, y1: cy1, x2, y2, x, y });
                lcx = x2;
                lcy = y2;
                cx = x;
                cy = y;
            }
            PathSegment::SmoothQuadratic { abs: true, x, y } => {
                let x1;
                let y1;
                if let Some(lt) = last_type {
                    if matches!(lt, PathSegment::Quadratic { .. })
                        || matches!(lt, PathSegment::SmoothQuadratic { .. })
                    {
                        x1 = cx + (cx - lcx);
                        y1 = cy + (cy - lcy);
                    } else {
                        x1 = cx;
                        y1 = cy;
                    }
                } else {
                    x1 = cx;
                    y1 = cy;
                }
                let cx1 = cx + 2.0 * (x1 - cx) / 3.0;
                let cy1 = cy + 2.0 * (y1 - cy) / 3.0;
                let cx2 = x + 2.0 * (x1 - x) / 3.0;
                let cy2 = y + 2.0 * (y1 - y) / 3.0;
                out.push(PathSegment::CurveTo {
                    abs: true,
                    x1: cx1,
                    y1: cy1,
                    x2: cx2,
                    y2: cy2,
                    x,
                    y,
                });
                lcx = x1;
                lcy = y1;
                cx = x;
                cy = y;
            }
            PathSegment::Quadratic { abs: true, x1, y1, x, y } => {
                let cx1 = cx + 2.0 * (x1 - cx) / 3.0;
                let cy1 = cy + 2.0 * (y1 - cy) / 3.0;
                let cx2 = x + 2.0 * (x1 - x) / 3.0;
                let cy2 = y + 2.0 * (y1 - y) / 3.0;
                out.push(PathSegment::CurveTo {
                    abs: true,
                    x1: cx1,
                    y1: cy1,
                    x2: cx2,
                    y2: cy2,
                    x,
                    y,
                });
                lcx = x1;
                lcy = y1;
                cx = x;
                cy = y;
            }
            PathSegment::EllipticalArc {
                abs: true,
                rx,
                ry,
                x_axis_rotation,
                large_arc,
                sweep,
                x,
                y,
            } => {
                let r1 = rx.abs();
                let r2 = ry.abs();
                let angle = x_axis_rotation;
                let large_arc_flag = large_arc;
                let sweep_flag = sweep;
                if r1 == 0.0 || r2 == 0.0 {
                    out.push(PathSegment::CurveTo {
                        abs: true,
                        x1: cx,
                        y1: cy,
                        x2: x,
                        y2: y,
                        x,
                        y,
                    });
                    cx = x;
                    cy = y;
                } else if cx != x || cy != y {
                    let curves = a2c(cx, cy, x, y, large_arc_flag, sweep_flag, r1, r2, angle);
                    let last = curves.len().saturating_sub(1);
                    for (i, curve) in curves.iter().enumerate() {
                        // End exactly on the arc's end point, so float error
                        // does not carry into the segments after it
                        let (end_x, end_y) = if i == last {
                            (x, y)
                        } else {
                            (curve[6], curve[7])
                        };
                        out.push(PathSegment::CurveTo {
                            abs: true,
                            x1: curve[2],
                            y1: curve[3],
                            x2: curve[4],
                            y2: curve[5],
                            x: end_x,
                            y: end_y,
                        })
                    }
                    cx = x;
                    cy = y;
                }
            }
            PathSegment::ClosePath { abs: true } => {
                out.push(*segment.borrow());
                cx = subx;
                cy = suby;
            }
            _ => panic!("Not expecting none absolute path!"),
        }
        last_type = Some(*segment.borrow());
    }

    out.into_iter()
}

#[cfg(test)]
mod test {

    use svgtypes::{PathParser, PathSegment};

    use super::{absolutize, segments_with_context};

    /// Assert two path segments are equal, comparing coordinates with a small
    /// tolerance so tests stay robust against last-ULP differences in the
    /// arc-to-cubic conversion across platforms/compilers.
    fn assert_segment_close(actual: PathSegment, expected: PathSegment) {
        let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
        match (actual, expected) {
            (
                PathSegment::MoveTo { abs: aa, x: ax, y: ay },
                PathSegment::MoveTo { abs: ea, x: ex, y: ey },
            )
            | (
                PathSegment::LineTo { abs: aa, x: ax, y: ay },
                PathSegment::LineTo { abs: ea, x: ex, y: ey },
            ) => {
                assert_eq!(aa, ea);
                assert!(close(ax, ex) && close(ay, ey), "{ax},{ay} vs {ex},{ey}");
            }
            (
                PathSegment::CurveTo {
                    abs: aa,
                    x1: ax1,
                    y1: ay1,
                    x2: ax2,
                    y2: ay2,
                    x: ax,
                    y: ay,
                },
                PathSegment::CurveTo {
                    abs: ea,
                    x1: ex1,
                    y1: ey1,
                    x2: ex2,
                    y2: ey2,
                    x: ex,
                    y: ey,
                },
            ) => {
                assert_eq!(aa, ea);
                assert!(
                    close(ax1, ex1)
                        && close(ay1, ey1)
                        && close(ax2, ex2)
                        && close(ay2, ey2)
                        && close(ax, ex)
                        && close(ay, ey),
                    "CurveTo {ax1},{ay1},{ax2},{ay2},{ax},{ay} vs \
                     {ex1},{ey1},{ex2},{ey2},{ex},{ey}"
                );
            }
            // ClosePath and any mismatched variants: fall back to exact equality.
            (a, e) => assert_eq!(a, e),
        }
    }

    #[test]
    pub fn absolutize_happy_path() {
        let path: String = "m 0 0 c 3 -0.6667 6 -1.3333 9 -2 a 1 1 0 0 0 -8 -1 a 1 1 0 0 0 -2 0 l 0 4 v 2 h 8 q 4 -10 9 -5 t -6 8 z".into();
        let path_parser = PathParser::from(path.as_ref());
        let path_segments: Vec<PathSegment> = path_parser.flatten().collect();
        let mut absolute = absolutize(path_segments.iter());
        assert_eq!(
            absolute.next().unwrap(),
            PathSegment::MoveTo { abs: true, x: 0.0, y: 0.0 }
        );
        assert_eq!(
            absolute.next().unwrap(),
            PathSegment::CurveTo {
                abs: true,
                x1: 3.0,
                y1: -0.6667,
                x2: 6.0,
                y2: -1.3333,
                x: 9.0,
                y: -2.0
            }
        );
        assert_eq!(
            absolute.next().unwrap(),
            PathSegment::EllipticalArc {
                abs: true,
                rx: 1.0,
                ry: 1.0,
                x_axis_rotation: 0.0,
                large_arc: false,
                sweep: false,
                x: 1.0,
                y: -3.0
            }
        );

        assert_eq!(
            absolute.next().unwrap(),
            PathSegment::EllipticalArc {
                abs: true,
                rx: 1.0,
                ry: 1.0,
                x_axis_rotation: 0.0,
                large_arc: false,
                sweep: false,
                x: -1.0,
                y: -3.0
            }
        );
        assert_eq!(
            absolute.next().unwrap(),
            PathSegment::LineTo { abs: true, x: -1.0, y: 1.0 }
        );
        assert_eq!(
            absolute.next().unwrap(),
            PathSegment::VerticalLineTo { abs: true, y: 3.0 }
        );
        assert_eq!(
            absolute.next().unwrap(),
            PathSegment::HorizontalLineTo { abs: true, x: 7.0 }
        );
        assert_eq!(
            absolute.next().unwrap(),
            PathSegment::Quadratic { abs: true, x1: 11.0, y1: -7.0, x: 16.0, y: -2.0 }
        );
        assert_eq!(
            absolute.next().unwrap(),
            PathSegment::SmoothQuadratic { abs: true, x: 10.0, y: 6.0 }
        );
        assert_eq!(
            absolute.next().unwrap(),
            PathSegment::ClosePath { abs: true }
        );
    }

    #[test]
    pub fn normalize_happy_path() {
        // A Bootstrap icon made mostly of arcs
        let path = "M5.5.5A.5.5 0 016 0h4a.5.5 0 010 1H9v1.07a7.002 7.002 0 013.537 12.26l.817.816a.5.5 0 01-.708.708l-.924-.925A6.967 6.967 0 018 16a6.967 6.967 0 01-3.722-1.07l-.924.924a.5.5 0 01-.708-.708l.817-.816A7.002 7.002 0 017 2.07V1H5.999a.5.5 0 01-.5-.5zM.86 5.387A2.5 2.5 0 114.387 1.86 8.035 8.035 0 00.86 5.387zM13.5 1c-.753 0-1.429.333-1.887.86a8.035 8.035 0 013.527 3.527A2.5 2.5 0 0013.5 1zm-5 4a.5.5 0 00-1 0v3.882l-1.447 2.894a.5.5 0 10.894.448l1.5-3A.5.5 0 008.5 9V5z";
        let segments: Vec<PathSegment> = PathParser::from(path).flatten().collect();
        let absolute: Vec<PathSegment> = absolutize(segments.iter()).collect();
        let normalized: Vec<PathSegment> = super::normalize(absolute.iter()).collect();

        // Only moves, lines, cubic curves and close paths come out
        assert!(normalized.iter().all(|segment| matches!(
            segment,
            PathSegment::MoveTo { .. }
                | PathSegment::LineTo { .. }
                | PathSegment::CurveTo { .. }
                | PathSegment::ClosePath { .. }
        )));

        // Every input segment still ends where it did, in the same order
        let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
        let ends = |segments: &[PathSegment]| -> Vec<(f64, f64)> {
            segments_with_context(segments)
                .map(|context| (context.end.x, context.end.y))
                .collect()
        };
        let output_ends = ends(&normalized);
        let mut remaining = output_ends.iter();
        for (x, y) in ends(&absolute) {
            assert!(
                remaining.any(|(ox, oy)| close(*ox, x) && close(*oy, y)),
                "({x}, {y}) is missing from the output"
            );
        }

        // Arcs become cubic curves that stay on the arc's ellipse
        for context in segments_with_context(&absolute) {
            let PathSegment::EllipticalArc { rx, ry, x_axis_rotation, large_arc, sweep, .. } =
                *context.segment
            else {
                continue;
            };
            let (start, end) = (context.start, context.end);
            let arc = kurbo::Arc::from_svg_arc(&kurbo::SvgArc {
                from: kurbo::Point::new(start.x, start.y),
                to: kurbo::Point::new(end.x, end.y),
                radii: kurbo::Vec2::new(rx, ry),
                x_rotation: x_axis_rotation.to_radians(),
                large_arc,
                sweep,
            })
            .expect("a drawable arc");
            // The distance from the ellipse's center in units of its radii,
            // which is 1 on the ellipse
            let unit_radius = |x: f64, y: f64| {
                let (dx, dy) = (x - arc.center.x, y - arc.center.y);
                let (sin, cos) = (-arc.x_rotation).sin_cos();
                let (ux, uy) = (dx * cos - dy * sin, dx * sin + dy * cos);
                (ux / arc.radii.x).hypot(uy / arc.radii.y)
            };

            let alone = [
                PathSegment::MoveTo { abs: true, x: start.x, y: start.y },
                *context.segment,
            ];
            let mut current = (start.x, start.y);
            for segment in super::normalize(alone.iter()).skip(1) {
                let PathSegment::CurveTo { x1, y1, x2, y2, x, y, .. } = segment else {
                    panic!("an arc became {segment:?}");
                };
                for k in 0..=16 {
                    let t = f64::from(k) / 16.0;
                    let u = 1.0 - t;
                    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                    let px = a * current.0 + b * x1 + c * x2 + d * x;
                    let py = a * current.1 + b * y1 + c * y2 + d * y;
                    let error = (unit_radius(px, py) - 1.0).abs();
                    assert!(error <= 3e-4, "{segment:?} strays {error} from the arc");
                }
                current = (x, y);
            }
        }
    }

    #[test]
    pub fn normalize_arc_no_nan_control_points() {
        // Regression: floating-point error used to push the asin argument
        // slightly outside [-1, 1] for these well-formed arcs, producing NaN
        // control points instead of a finite cubic.
        for path in [
            "M0 0 A7 7 30 0 0 -7 0",
            "M0 0 A6 3 90 0 0 -3 -6",
            "M170 39 A34 5 180 0 0 170 155",
        ] {
            let segments: Vec<PathSegment> = PathParser::from(path).flatten().collect();
            for segment in super::normalize(absolutize(segments.iter())) {
                if let PathSegment::CurveTo { x1, y1, x2, y2, x, y, .. } = segment {
                    assert!(
                        x1.is_finite()
                            && y1.is_finite()
                            && x2.is_finite()
                            && y2.is_finite()
                            && x.is_finite()
                            && y.is_finite(),
                        "non-finite control point in {path}: {segment:?}"
                    );
                }
            }
        }

        // Expected cubic for the reported path (compared with tolerance to
        // stay robust against last-ULP platform differences).
        let segments: Vec<PathSegment> = PathParser::from("M0 0 A7 7 30 0 0 -7 0")
            .flatten()
            .collect();
        let mut normalized = super::normalize(absolutize(segments.iter()));
        assert_segment_close(
            normalized.next().unwrap(),
            PathSegment::MoveTo { abs: true, x: 0.0, y: 0.0 },
        );
        match normalized.next().unwrap() {
            PathSegment::CurveTo { x1, y1, x2, y2, x, y, .. } => {
                let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
                assert!(close(x1, -2.165807537309522), "x1 = {x1}");
                assert!(close(y1, -1.2504295646785737), "y1 = {y1}");
                assert!(close(x2, -4.8341924626904795), "x2 = {x2}");
                assert!(close(y2, -1.2504295646785737), "y2 = {y2}");
                assert!(close(x, -7.0), "x = {x}");
                assert!(close(y, 0.0), "y = {y}");
            }
            other => panic!("expected CurveTo, got {other:?}"),
        }
    }
}
