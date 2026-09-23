# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Re-export the `svgtypes` crate as `svg_path_ops::svgtypes`, so consumers can
  name its types without adding a matching `svgtypes` dependency.
- Add a dependency on `euclid` 0.22 and re-export it as
  `svg_path_ops::euclid`. `euclid::default::Point2D<f64>` is the point type
  for geometry results, shared with roughr, and converts to and from
  `kurbo::Point` with `.into()`.
- `pt::PathTransformer::parse(&str)` and a `FromStr` implementation. Both
  return an `svgtypes::Error` for invalid path data instead of dropping it
  silently. `PathTransformer::new` keeps its lenient behavior, now
  documented: it keeps the segments before the first error, as SVG
  renderers do.
- `pt::PathTransformer` implements `Display`.
- `write_path()` and `WriteOptions` write any sequence of path segments as
  path data. Options round numbers to a precision and write compact output
  (`M10 10l.5-5 2 0`) without redundant spaces, leading zeros or command
  letters SVG implies (a repeated command, or a line right after a move),
  and with arc flags written without separators (`a5 5 0 0110 0`).
- `pt::PathTransformer::to_string_with()` writes the transformed path with
  `WriteOptions`.
- `segments_with_context()` iterates over segments together with their
  index and absolute start, end and subpath start points, plus the control
  point implied by smooth curves (`S`/`T`). Relative and shorthand segments
  can be handled without tracking the current point by hand.
- `reverse()` and `pt::PathTransformer::reverse()` reverse the drawing
  direction of every subpath, keeping subpath order. Relative segments stay
  relative, arcs keep their shape with the sweep flag flipped, and smooth
  `S`/`T` segments stay smooth where their reversed neighbour allows it.
- `pt::PathTransformer::flip_x()` and `flip_y()` mirror the path in place,
  about the center of its bounding box after pending transforms.
- `shapes::Shape` converts the SVG basic shapes (`rect`, `circle`,
  `ellipse`, `line`, `polyline`, `polygon`) to the equivalent paths SVG 2
  defines, including its rules for missing and oversized `rx`/`ry`.
- `optimize()` and `pt::PathTransformer::optimize()` rewrite a path in its
  shortest form: absolute or relative per segment, `H`/`V` for straight
  lines, `S`/`T` for mirrored curves, lines for curves whose control points
  lie on the line between their ends, a single arc for a run of cubic curves
  that follows one circle, and no segments that draw nothing.
  With a precision, relative coordinates are taken between rounded absolute
  points, so rounding does not drift along the path; without one, no point
  moves.
- `split_subpaths()` returns each subpath as a path of its own, turning a
  relative move that depended on the previous subpath into an absolute one
  and giving a subpath that started after a close path its own move.
- `is_closed()` tells whether every subpath that draws something ends with a
  close path, as SVG defines it: returning to the start without one is open.
- Crate documentation and README with a runnable example for each feature:
  transforming, writing, converting commands, bounding boxes, walking
  segments and handling invalid path data, illustrated by reference images
  for translate, rotate, scale, skew, flip, shapes, reverse, split_subpaths,
  is_closed, unarc, unshort, to_box, inbox and segments_with_context. They
  replace the examples that only linked to rough_piet programs.

### Changed

- **Breaking:** Upgrade `svgtypes` from 0.11 to 0.16. The re-exported
  `PathSegment`, and every function that takes or returns it, now uses the
  0.16 type. Code that passes segments from its own `svgtypes` 0.11
  dependency must upgrade to 0.16 or use `svg_path_ops::svgtypes`.
- Replace the `cgmath` dependency with `kurbo` 0.13 for the transform
  matrix math. `svgtypes` 0.16 already depends on `kurbo` 0.13, so this
  removes `cgmath` and `approx` from the dependency tree without adding a
  new crate. The public API and output are unchanged.
- `normalize()` turns arcs into cubic curves of at most 90 degrees, like
  `pt::PathTransformer::unarc()`, instead of 120 degrees. The curves now
  stay within 0.03% of the arc's radius instead of about 0.15%, and there
  is one arc converter in the crate instead of two.
- **Breaking:** `pt::PathTransformer::to_string()` is now provided by
  `Display` and takes `&self` instead of `&mut self`. It no longer applies
  pending transforms to the transformer itself; it writes them applied and
  leaves the transformer unchanged. The output string is the same.

- The package no longer ships the `assets/` images; the docs load them from
  GitHub.

### Removed

- **Breaking:** `print_line_segment()`, a debugging helper that printed a
  segment to stdout. Format the path with `PathTransformer`'s `Display`
  instead.

## [0.11.2] - 2026-09-11

### Fixed

- `pt::PathTransformer::to_string()` no longer panics on an empty or
  unparseable path; it now returns an empty string.
- `pt::PathTransformer::round()` no longer overflows `u8` when rounding an
  elliptical arc's x-axis rotation with a precision near `u8::MAX`. Debug
  builds panicked; release builds silently rounded the rotation to the wrong
  number of decimal places.

## [0.11.1] - 2026-07-26

### Fixed

- `normalize()` no longer returns `NaN` control points for well-formed
  elliptical arcs. Floating-point error could push the `asin` argument in the
  arc-to-cubic conversion slightly outside `[-1, 1]`; the argument is now
  clamped.
- `bbox::minmax_q` no longer drops the maximum for a rising quadratic whose
  control coordinate equals the start coordinate (`a[1] == a[0]`). This also
  fixes `minmax_c`, `to_box`, and the `add_*_q` / `add_*_c` helpers that rely on
  it.
- `pt::PathTransformer::unshort()` now resolves a smooth segment that directly
  follows another smooth segment against its predecessor's expanded form,
  instead of the pre-pass snapshot that collapsed the reflected control point to
  `(0, 0)`.

## [0.11.0]

- Previous releases; see the
  [git history](https://github.com/orhanbalci/rough-rs/commits/main) for details.

[Unreleased]: https://github.com/orhanbalci/rough-rs/compare/svg_path_ops@0.11.2...HEAD
[0.11.2]: https://github.com/orhanbalci/rough-rs/compare/svg_path_ops@0.11.1...svg_path_ops@0.11.2
[0.11.1]: https://github.com/orhanbalci/rough-rs/compare/svg_path_ops@0.11.0...svg_path_ops@0.11.1
[0.11.0]: https://github.com/orhanbalci/rough-rs/releases/tag/svg_path_ops@0.11.0
