# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/orhanbalci/rough-rs/compare/rough_tiny_skia@0.13.1...HEAD
[0.13.1]: https://github.com/orhanbalci/rough-rs/compare/rough_tiny_skia@0.13.0...rough_tiny_skia@0.13.1
[0.13.0]: https://github.com/orhanbalci/rough-rs/compare/rough_tiny_skia@0.12.0...rough_tiny_skia@0.13.0
[0.12.0]: https://github.com/orhanbalci/rough-rs/releases/tag/rough_tiny_skia@0.12.0
