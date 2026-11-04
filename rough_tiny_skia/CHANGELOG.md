# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `SkiaDrawable::draw_with_transform(&mut PixmapMut, Transform)` applies a
  tiny-skia `Transform` during rasterization, so pan, zoom and HiDPI scaling
  need no rebuilt drawable. Stroke widths and dash lengths scale with the
  transform. `draw` is unchanged and delegates with the identity transform.
- `ToSkiaOpset::try_to_skia_opset` returns `None` for an op set with no
  drawable geometry instead of panicking.

### Changed

- `ToSkiaDrawable::to_skia_drawable` drops empty op sets, so converting and
  drawing degenerate input no longer panics. `SkiaDrawable::sets` can
  therefore be shorter than the source `Drawable`'s sets.

### Fixed

- Non-dashed path strokes now honor `Options::line_cap`, `line_join` and the
  miter limit. They were previously ignored unless a dash pattern was set.
  When the options are unset the output is unchanged.

## [0.14.0] - 2026-09-24

### Changed

- **Breaking:** Require `roughr` 0.14. This crate's public API exposes
  `roughr` types (`Options`, `Point2D`, `OpSetType`), so they now come from
  `roughr` 0.14, which uses `svgtypes` 0.16 and `svg_path_ops` 0.12; see the
  [roughr changelog](../roughr/CHANGELOG.md).

### Added

- `path_ops_gallery` example: generates the operation reference images used
  in the `svg_path_ops` docs, drawing Ferris the crab. It shares the framed
  canvas, font and labels with `options_gallery` through `examples/common`.

## [0.13.1] - 2026-09-23

### Fixed

- Sketched fills (hachure, zigzag, dots, ...) were invisible unless
  `fill_weight` was set explicitly: the default `-1` was used as the line
  width instead of `stroke_width / 2`.

### Added

- `svg` example: sketches a whole SVG file by converting it to plain paths
  with `usvg`.
- `options_gallery` example: generates the option reference images used in
  the `roughr` docs.

## [0.13.0] - 2026-09-11

### Changed

- **Breaking:** Require `roughr` 0.13. This crate's public API exposes `roughr`
  types (`Options`, `Point2D`, `OpSetType`), so they now come from `roughr`
  0.13; see the [roughr changelog](../roughr/CHANGELOG.md).

## [0.12.0] - 2025-06-10

- Previous releases; see the
  [git history](https://github.com/orhanbalci/rough-rs/commits/main/rough_tiny_skia) for details.

[Unreleased]: https://github.com/orhanbalci/rough-rs/compare/rough_tiny_skia@0.14.0...HEAD
[0.14.0]: https://github.com/orhanbalci/rough-rs/compare/rough_tiny_skia@0.13.1...rough_tiny_skia@0.14.0
[0.13.1]: https://github.com/orhanbalci/rough-rs/compare/rough_tiny_skia@0.13.0...rough_tiny_skia@0.13.1
[0.13.0]: https://github.com/orhanbalci/rough-rs/compare/rough_tiny_skia@0.12.0...rough_tiny_skia@0.13.0
[0.12.0]: https://github.com/orhanbalci/rough-rs/releases/tag/rough_tiny_skia@0.12.0
