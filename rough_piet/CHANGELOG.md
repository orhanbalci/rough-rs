# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.15.0] - 2026-09-24

### Changed

- **Breaking:** Require `roughr` 0.14. This crate's public API exposes
  `roughr` types (`Options`, `Point2D`, `OpSetType`), so they now come from
  `roughr` 0.14, which uses `svgtypes` 0.16 and `svg_path_ops` 0.12; see the
  [roughr changelog](../roughr/CHANGELOG.md).

## [0.14.0] - 2026-09-11

### Changed

- **Breaking:** Require `roughr` 0.13. This crate's public API exposes `roughr`
  types (`Options`, `Point2D`, `OpSetType`), so they now come from `roughr`
  0.13; see the [roughr changelog](../roughr/CHANGELOG.md).

## [0.13.0] - 2025-08-29

- Previous releases; see the
  [git history](https://github.com/orhanbalci/rough-rs/commits/main/rough_piet) for details.

[Unreleased]: https://github.com/orhanbalci/rough-rs/compare/rough_piet@0.15.0...HEAD
[0.15.0]: https://github.com/orhanbalci/rough-rs/compare/rough_piet@0.14.0...rough_piet@0.15.0
[0.14.0]: https://github.com/orhanbalci/rough-rs/compare/rough_piet@0.13.0...rough_piet@0.14.0
[0.13.0]: https://github.com/orhanbalci/rough-rs/releases/tag/rough_piet@0.13.0
