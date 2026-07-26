# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/orhanbalci/rough-rs/compare/svg_path_ops@0.11.1...HEAD
[0.11.1]: https://github.com/orhanbalci/rough-rs/compare/svg_path_ops@0.11.0...svg_path_ops@0.11.1
[0.11.0]: https://github.com/orhanbalci/rough-rs/releases/tag/svg_path_ops@0.11.0
