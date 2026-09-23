# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Breaking:** Upgrade `svgtypes` from 0.11 to 0.16. roughr re-exports
  `svgtypes::*`, so all of those re-exported types (including `PathSegment`
  and `PathParser`) now come from 0.16. Code that mixes them with types from
  its own `svgtypes` 0.11 dependency must upgrade to 0.16.
- Arcs in SVG paths are sketched from curves of at most 90 degrees instead
  of 120, as `svg_path_ops::normalize()` now converts them more accurately.
  The same seed sketches paths with arcs slightly differently.

## [0.13.1] - 2026-09-23

### Added

- Documentation for every `Options` field, `FillStyle` variant and `LineCap`,
  including its default and a reference image showing its effect.
- "Options" and "Drawing SVG files" sections in the crate docs and README.

## [0.13.0] - 2026-09-11

### Added

- Re-export `euclid::Trig` and `num_traits::{Float, FromPrimitive}`. These
  traits bound the generic parameter of `OpSet` and other public types, so
  consumers can now name them without depending on `euclid` or `num-traits`
  directly ([#30](https://github.com/orhanbalci/rough-rs/pull/30)).

### Changed

- **Breaking:** `roughr::Point2D` now re-exports the one-parameter alias
  `euclid::default::Point2D<T>` (`euclid::Point2D<T, UnknownUnit>`) instead of
  the two-parameter `euclid::Point2D<T, U>`, matching the type used throughout
  roughr's API. Code that wrote `roughr::Point2D<T, U>` must switch to
  `roughr::Point2D<T>` or use `euclid::Point2D` directly
  ([#30](https://github.com/orhanbalci/rough-rs/pull/30)).

## [0.12.0] - 2025-06-10

- Previous releases; see the
  [git history](https://github.com/orhanbalci/rough-rs/commits/main/roughr) for details.

[Unreleased]: https://github.com/orhanbalci/rough-rs/compare/roughr@0.13.1...HEAD
[0.13.1]: https://github.com/orhanbalci/rough-rs/compare/roughr@0.13.0...roughr@0.13.1
[0.13.0]: https://github.com/orhanbalci/rough-rs/compare/roughr@0.12.0...roughr@0.13.0
[0.12.0]: https://github.com/orhanbalci/rough-rs/releases/tag/roughr@0.12.0
