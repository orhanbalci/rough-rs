# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Re-export the `svgtypes` crate as `svg_path_ops::svgtypes`, so consumers can
  name its types without adding a matching `svgtypes` dependency.

### Changed

- **Breaking:** Upgrade `svgtypes` from 0.11 to 0.16. The re-exported
  `PathSegment`, and every function that takes or returns it, now uses the
  0.16 type. Code that passes segments from its own `svgtypes` 0.11
  dependency must upgrade to 0.16 or use `svg_path_ops::svgtypes`.
- Replace the `cgmath` dependency with `kurbo` 0.13 for the transform
  matrix math. `svgtypes` 0.16 already depends on `kurbo` 0.13, so this
  removes `cgmath`, `approx` and `num-traits` from the dependency tree
  without adding a new crate. The public API and output are unchanged.

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
