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
//! It also draws regular polygons and stars, laid out as Paper.js lays
//! them out:
//!
//! ![polygons and stars](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/polygons.png)
//!
//! ```
//! use svg_path_ops::shapes::Shape;
//! use svg_path_ops::PathMeasure;
//!
//! let star = Shape::Star {
//!     cx: 0.0,
//!     cy: 0.0,
//!     points: 5,
//!     outer_radius: 10.0,
//!     inner_radius: 4.0,
//! };
//! let path = star.to_path();
//! // A move to the first point, lines to the other nine and a close path
//! assert_eq!(path.len(), 11);
//! assert!(PathMeasure::new(&path).is_clockwise());
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
//! [`PathMeasure::is_clockwise`] tells which way a path runs, and
//! [`reorient`] turns every subpath of a shape with holes the right way
//! round: outlines one way and the holes in them the other, as fonts and
//! icon sets expect, so the shape fills the same under both fill rules.
//! [`join`] joins two open paths where their ends meet, reversing one when
//! it runs the other way:
//!
//! ![reorient and join](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/reorient_join.png)
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//! use svg_path_ops::svgtypes::PathParser;
//! use svg_path_ops::{join, write_path, WriteOptions};
//!
//! // Both squares clockwise: the inner one fills instead of cutting a hole
//! let mut frame = PathTransformer::parse("M 0 0 H 30 V 30 H 0 Z M 10 10 H 20 V 20 H 10 Z")?;
//! assert_eq!(frame.measure().area(), 1000.0);
//! frame.reorient(true);
//! assert_eq!(frame.measure().area(), 800.0);
//!
//! let parse = |data| PathParser::from(data).collect::<Result<Vec<_>, _>>();
//! let joined = join(&parse("M 0 0 L 10 0")?, &parse("M 20 0 L 10 0")?, 0.0);
//! assert_eq!(
//!     write_path(&joined, &WriteOptions::default()),
//!     "M 0 0 L 10 0 L 20 0"
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
//! ### Measuring a path
//!
//! [`PathMeasure`] finds the length of a path and the point, direction and
//! segment at any length along it. Arcs are measured as arcs, not as the
//! curves that approximate them:
//!
//! ![measure](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/measure.png)
//!
//! ```
//! use svg_path_ops::euclid::default::Point2D;
//! use svg_path_ops::pt::PathTransformer;
//!
//! let path = PathTransformer::parse("M 10 0 A 10 10 0 0 1 -10 0")?;
//! let measure = path.measure();
//!
//! // A half circle of radius 10
//! let length = measure.total_length();
//! assert!((length - 10.0 * std::f64::consts::PI).abs() < 1e-9);
//!
//! let middle = measure.point_at(length / 2.0).unwrap();
//! assert!((middle - Point2D::new(0.0, 10.0)).length() < 1e-9);
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! Along with the point, it gives the tangent, the normal and the curvature
//! at any length. A curvature comb draws normals as long as the curvature,
//! showing where a path bends hard and where it turns the other way:
//!
//! ![curvature](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/curvature.png)
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//!
//! // Half an ellipse with radii 20 and 10, drawn clockwise on screen
//! let measure = PathTransformer::parse("M 20 0 A 20 10 0 0 1 -20 0")?.measure();
//!
//! // Curvature is a / b² at the end of the long axis, b / a² at the short
//! assert!((measure.curvature_at(0.0).unwrap() - 0.2).abs() < 1e-9);
//! let middle = measure.total_length() / 2.0;
//! assert!((measure.curvature_at(middle).unwrap() - 0.025).abs() < 1e-9);
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! [`PathMeasure::classify`] tells what kind of curve a segment is: a line,
//! an arch that bends one way, or a cubic curve with inflections, a cusp or
//! a loop, and where they are:
//!
//! ![classify](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/classify.png)
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//! use svg_path_ops::CurveKind;
//!
//! let measure = PathTransformer::parse("M 0 0 C 40 30 -10 30 30 0")?.measure();
//! let Some(CurveKind::Loop { first, second }) = measure.classify(1) else {
//!     panic!("a loop");
//! };
//! assert!(0.0 < first && first < second && second < 1.0);
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! It also finds the point of the path nearest to another point, and
//! whether a point is on the path's stroke:
//!
//! ![nearest](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/nearest.png)
//!
//! ```
//! use svg_path_ops::euclid::default::Point2D;
//! use svg_path_ops::pt::PathTransformer;
//!
//! let measure = PathTransformer::parse("M 10 0 A 10 10 0 0 1 -10 0")?.measure();
//!
//! let nearest = measure.nearest(Point2D::new(0.0, 20.0)).unwrap();
//! assert!((nearest.distance - 10.0).abs() < 1e-9);
//! assert!(measure.is_point_in_stroke(Point2D::new(0.0, 11.0), 4.0));
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! And the area a path encloses, signed by the direction it is drawn in,
//! and whether a point is inside it under either [`FillRule`]:
//!
//! ![contains](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/contains.png)
//!
//! ```
//! use svg_path_ops::euclid::default::Point2D;
//! use svg_path_ops::pt::PathTransformer;
//! use svg_path_ops::FillRule;
//!
//! // A square with a hole: the inner square runs the other way round
//! let frame = PathTransformer::parse("M 0 0 h 30 v 30 h -30 z M 10 10 v 10 h 10 v -10 z")?;
//! let measure = frame.measure();
//!
//! assert_eq!(measure.area(), 800.0);
//! assert!(measure.contains(Point2D::new(5.0, 5.0), FillRule::NonZero));
//! assert!(!measure.contains(Point2D::new(15.0, 15.0), FillRule::NonZero));
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! It cuts out the part between two lengths, keeping arcs as arcs, splits
//! the path at a length, and turns it into straight lines within a
//! tolerance, for pen plotters and anything else that only draws lines:
//!
//! ![crop](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/crop.png)
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//! use svg_path_ops::{write_path, PathSegment, WriteOptions};
//!
//! let measure = PathTransformer::parse("M 0 0 h 10 A 5 5 0 0 1 10 10")?.measure();
//!
//! let part = measure.crop(5.0, 12.0);
//! assert!(matches!(part[2], PathSegment::EllipticalArc { .. }));
//!
//! let lines = measure.flatten(0.01);
//! assert!(lines
//!     .iter()
//!     .skip(1)
//!     .all(|segment| matches!(segment, PathSegment::LineTo { .. })));
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! [`PathMeasure::interior_point`] finds a point inside a shape, away from
//! its outline, for a label or to tell which shape lies inside which, and
//! [`PathMeasure::divide_at`] divides the segment at a length in two of the
//! same kind, leaving the rest of the path as it is:
//!
//! ![interior_point and divide_at](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/interior_divide.png)
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//! use svg_path_ops::{write_path, FillRule, WriteOptions};
//!
//! let measure = PathTransformer::parse("M 0 0 h 10 v 10 h -10 z")?.measure();
//! let inside = measure.interior_point(FillRule::NonZero).unwrap();
//! assert!(measure.contains(inside, FillRule::NonZero));
//!
//! let divided = measure.divide_at(15.0);
//! assert_eq!(
//!     write_path(&divided, &WriteOptions::default()),
//!     "M 0 0 h 10 v 5 v 5 h -10 z"
//! );
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! ### Simplifying
//!
//! [`PathMeasure::simplify`] redraws a path of many short segments, as a
//! freehand stroke, a traced outline or flattened lines, with a few cubic
//! curves that stay within a tolerance of it. Joins that turn more than a
//! given angle stay corners, and straight stretches stay lines:
//!
//! ![simplify](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/simplify.png)
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//!
//! // A quarter circle drawn with 90 lines
//! let mut data = String::from("M 100 0");
//! for degree in 1..=90 {
//!     let angle = f64::from(degree).to_radians();
//!     data += &format!(" L {} {}", 100.0 * angle.cos(), 100.0 * angle.sin());
//! }
//! let simplified = PathTransformer::parse(&data)?.measure().simplify(0.5, 60.0);
//! // A move and one curve
//! assert_eq!(simplified.len(), 2);
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! The curves are fitted with Philip J. Schneider's algorithm, the one
//! Paper.js uses: control points along the path's direction, placed by
//! least squares, then refined with Newton's method. Each curve takes the
//! longest stretch of the path one curve can follow, so the result has few
//! curves, and neighbouring curves join smoothly.
//!
//! ### Smoothing
//!
//! [`PathMeasure::smooth`] draws smooth curves through the points where a
//! path's segments meet, rounding a polygon or a path of lines. A
//! [`Smoothing::Continuous`] spline keeps the curvature continuous
//! everywhere; a [`Smoothing::CatmullRom`] spline shapes each curve from
//! its neighbouring points only. Joins that turn more than a given angle
//! stay corners:
//!
//! ![smooth](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/smooth.png)
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//! use svg_path_ops::{PathSegment, Smoothing};
//!
//! let zigzag = PathTransformer::parse("M 0 0 L 10 10 L 20 0 L 30 10")?;
//! let wave = zigzag
//!     .measure()
//!     .smooth(Smoothing::CatmullRom { alpha: 0.5 }, 180.0);
//!
//! // Three curves through the same points
//! assert!(wave[1..]
//!     .iter()
//!     .all(|segment| matches!(segment, PathSegment::CurveTo { .. })));
//! assert!(matches!(
//!     wave[3],
//!     PathSegment::CurveTo { x: 30.0, y: 10.0, .. }
//! ));
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! ### Morphing
//!
//! [`Morph`] brings two paths to the same shape of path data, so a path
//! part of the way from one to the other is a matter of mixing their
//! numbers, for animations; [`interpolate`] gives one such path. Subpaths
//! without a partner grow from a point, and closed ones are lined up so
//! the shape does not twist on its way:
//!
//! ![morph](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/morph.png)
//!
//! ```
//! use svg_path_ops::svgtypes::PathParser;
//! use svg_path_ops::{Morph, PathMeasure};
//!
//! let parse = |data| PathParser::from(data).collect::<Result<Vec<_>, _>>();
//! let square = parse("M 0 0 H 20 V 20 H 0 Z")?;
//! let circle = parse("M 20 10 A 10 10 0 0 1 0 10 A 10 10 0 0 1 20 10 Z")?;
//! let morph = Morph::new(&square, &circle);
//! let frames: Vec<f64> = (0..=10)
//!     .map(|i| PathMeasure::new(morph.at(f64::from(i) / 10.0)).area())
//!     .collect();
//! // From the square's area down to the circle's
//! assert!((frames[0] - 400.0).abs() < 1e-6);
//! assert!(frames.windows(2).all(|pair| pair[1] < pair[0]));
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! ### Intersections
//!
//! [`PathMeasure::intersections`] finds the points where two paths meet,
//! arcs included, where each point lies on both paths, and whether the
//! paths cross there or only touch:
//!
//! ![intersections](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/intersections.png)
//!
//! ```
//! use svg_path_ops::euclid::default::Point2D;
//! use svg_path_ops::pt::PathTransformer;
//!
//! let circle = PathTransformer::parse("M 10 0 A 10 10 0 0 1 -10 0 A 10 10 0 0 1 10 0")?;
//! let line = PathTransformer::parse("M -20 6 H 20")?;
//!
//! let meets = circle.measure().intersections(&line.measure());
//! assert_eq!(meets.len(), 2);
//! assert!((meets[0].point - Point2D::new(8.0, 6.0)).length() < 1e-9);
//! // Where the second point lies on the line, by length
//! assert!((meets[1].other.length - 12.0).abs() < 1e-9);
//! assert!(meets.iter().all(|meet| meet.crossing));
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! Each pair of segments is solved the way that suits it. Lines meet lines
//! in a linear system, and lines meet curves and arcs at the roots of a
//! polynomial. Two curves are cut in half again and again, dropping the
//! halves whose bounding boxes do not overlap, until the pieces around each
//! meeting point are a millionth of a unit wide; Newton's method on the
//! exact curves then makes each point accurate to about 1e-10. Where two
//! curves cross while running side by side, the pieces around the crossing
//! are gathered into one point. Segments that overlap along a stretch have
//! no single meeting point and are not reported.
//!
//! To tell a crossing from a touch, each path is followed a little way back
//! and on from the point; the other path crosses when its two ends fall on
//! different sides of this one. That tells a line touching a circle from
//! one crossing it, and a curve crossing a line where it runs along it, at
//! an inflection, from one touching it.
//!
//! [`PathMeasure::self_intersections`] finds where a path meets itself:
//! one part crossing or touching another, or a cubic curve looping across
//! itself, each point once, with its two places along the path:
//!
//! ![self_intersections](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/self_intersections.png)
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//!
//! let bow = PathTransformer::parse("M 0 0 L 20 20 L 20 0 L 0 20 Z")?.measure();
//! let meets = bow.self_intersections();
//! assert_eq!(meets.len(), 1);
//! assert!((meets[0].point.x - 10.0).abs() < 1e-9 && meets[0].crossing);
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! ### Boolean operations
//!
//! [`boolean`] combines the areas two paths cover: their union,
//! intersection, difference or exclusive or, drawn with curves within a
//! tolerance of the exact outline. [`PathTransformer`] has them too, as
//! `union`, `intersect`, `difference` and `xor`, applying both paths'
//! pending transforms first. They need the `boolean` feature, on by
//! default, which brings in [i_overlay] for the polygon work:
//!
//! ![boolean](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/boolean.png)
//!
//! ```
//! # #[cfg(feature = "boolean")]
//! # {
//! use svg_path_ops::pt::PathTransformer;
//! use svg_path_ops::svgtypes::PathParser;
//! use svg_path_ops::{boolean, BooleanOp, BooleanOptions, PathMeasure};
//!
//! let parse = |data| PathParser::from(data).collect::<Result<Vec<_>, _>>();
//! let square = parse("M 0 0 H 20 V 20 H 0 Z")?;
//! let hole = parse("M 15 10 A 5 5 0 0 1 5 10 A 5 5 0 0 1 15 10 Z")?;
//! let options = BooleanOptions::default();
//!
//! let frame = boolean(&square, &hole, BooleanOp::Difference, &options);
//! let area = PathMeasure::new(&frame).area();
//! assert!((area - (400.0 - std::f64::consts::PI * 25.0)).abs() < 0.2);
//!
//! // The same with transformers, the hole moved into place
//! let mut square = PathTransformer::parse("M 0 0 H 20 V 20 H 0 Z")?;
//! let mut hole = PathTransformer::parse("M 5 0 A 5 5 0 0 1 -5 0 A 5 5 0 0 1 5 0 Z")?;
//! hole.translate(10.0, 10.0);
//! square.difference(&hole, &options);
//! assert!((square.measure().area() - area).abs() < 0.2);
//! # }
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! The paths are turned into lines within a quarter of the tolerance and
//! combined by i_overlay, and curves are fitted to the result with
//! [`PathMeasure::simplify`], keeping the corners where the outlines meet.
//! Every point of the result's outline is within the tolerance of the
//! original outlines, and so is every point it fills differently from the
//! exact operation.
//!
//! [i_overlay]: https://docs.rs/i_overlay
//!
//! ### Stroke to path and offset
//!
//! [`PathMeasure::stroke_to_path`] turns a stroke into the outline of the
//! area it covers, caps, joins and miter limit included, for cutters,
//! plotters and "stroke to path" in a drawing program, and
//! [`PathMeasure::offset`] grows or shrinks the area a path covers. Both
//! use the `boolean` feature and work within a tolerance, as [`boolean`]
//! does:
//!
//! ![outline](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/outline.png)
//!
//! ```
//! # #[cfg(feature = "boolean")]
//! # {
//! use svg_path_ops::pt::PathTransformer;
//! use svg_path_ops::{BooleanOptions, LineCap, LineJoin, PathMeasure, StrokeStyle};
//!
//! let options = BooleanOptions::default();
//! let line = PathTransformer::parse("M 0 0 H 20")?.measure();
//! let style = StrokeStyle {
//!     width: 4.0,
//!     cap: LineCap::Square,
//!     ..StrokeStyle::default()
//! };
//! let outline = line.stroke_to_path(&style, &options);
//! assert!((PathMeasure::new(&outline).area() - 96.0).abs() < 1e-9);
//!
//! let square = PathTransformer::parse("M 0 0 H 20 V 20 H 0 Z")?.measure();
//! let grown = square.offset(2.0, LineJoin::Miter, 4.0, &options);
//! assert!((PathMeasure::new(&grown).area() - 576.0).abs() < 1e-9);
//! # }
//! # Ok::<(), svg_path_ops::svgtypes::Error>(())
//! ```
//!
//! The stroke is built from the lines across the path at each point, half
//! the width to each side, so a curve tighter than the stroke is covered
//! exactly as SVG defines it; joins are added only where segments meet.
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
//! [`PathMeasure::bounds`] gives the exact box around a path, and
//! [`PathMeasure::stroke_bounds`] the box around its stroke, caps, joins
//! and miter limit included:
//!
//! ![stroke_bounds](https://raw.githubusercontent.com/orhanbalci/rough-rs/main/svg_path_ops/assets/ops/stroke_bounds.png)
//!
//! ```
//! use svg_path_ops::pt::PathTransformer;
//! use svg_path_ops::{LineJoin, StrokeStyle};
//!
//! let corner = PathTransformer::parse("M 0 10 L 10 0 L 20 10")?.measure();
//! let style = StrokeStyle {
//!     width: 2.0,
//!     join: LineJoin::Round,
//!     ..StrokeStyle::default()
//! };
//! let bounds = corner.stroke_bounds(&style).unwrap();
//! // The round join reaches one unit above the corner
//! assert!((bounds.min.y + 1.0).abs() < 1e-12);
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
//! [`boolean`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.boolean.html
//! [`reorient`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.reorient.html
//! [`join`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.join.html
//! [`PathMeasure::is_clockwise`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.is_clockwise
//! [`PathMeasure`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html
//! [`FillRule`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/enum.FillRule.html
//! [`PathMeasure::intersections`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.intersections
//! [`PathMeasure::interior_point`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.interior_point
//! [`PathMeasure::divide_at`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.divide_at
//! [`PathMeasure::classify`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.classify
//! [`PathMeasure::bounds`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.bounds
//! [`PathMeasure::stroke_bounds`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.stroke_bounds
//! [`PathMeasure::stroke_to_path`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.stroke_to_path
//! [`PathMeasure::offset`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.offset
//! [`Morph`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.Morph.html
//! [`interpolate`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.interpolate.html
//! [`PathMeasure::self_intersections`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.self_intersections
//! [`PathMeasure::simplify`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.simplify
//! [`PathMeasure::smooth`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/struct.PathMeasure.html#method.smooth
//! [`Smoothing::Continuous`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/enum.Smoothing.html#variant.Continuous
//! [`Smoothing::CatmullRom`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/enum.Smoothing.html#variant.CatmullRom
//! [`optimize`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.optimize.html
//! [`Shape`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/shapes/enum.Shape.html
//! [`flip_x`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.flip_x
//! [`flip_y`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/pt/struct.PathTransformer.html#method.flip_y
//! [`split_subpaths`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.split_subpaths.html
//! [`is_closed`]: https://docs.rs/svg_path_ops/latest/svg_path_ops/fn.is_closed.html

pub(crate) mod a2c;
pub mod bbox;
#[cfg(feature = "boolean")]
mod boolean;
mod classify;
mod context;
mod crossing;
mod divide;
pub(crate) mod ellipse;
mod intersect;
mod join;
mod measure;
mod morph;
mod optimize;
mod orient;
#[cfg(feature = "boolean")]
mod outline;
pub mod pt;
mod reverse;
pub mod shapes;
mod simplify;
mod smooth;
mod stroke;
mod subpaths;
mod write;

use std::borrow::Borrow;

use a2c::a2c;
#[cfg(feature = "boolean")]
pub use boolean::{boolean, BooleanOp, BooleanOptions};
pub use classify::CurveKind;
pub use context::{segments_with_context, SegmentContext};
pub use intersect::{Intersection, Location};
pub use join::join;
pub use measure::{FillRule, Nearest, PathMeasure, Position};
pub use morph::{interpolate, Morph};
pub use optimize::optimize;
pub use orient::reorient;
pub use reverse::reverse;
pub use smooth::Smoothing;
pub use stroke::{LineCap, LineJoin, StrokeStyle};
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
