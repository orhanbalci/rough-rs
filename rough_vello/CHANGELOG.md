# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Breaking:** Upgrade `vello` from 0.5 to 0.10. This crate's public API
  exposes `vello::Scene`, so callers must use vello 0.10 as well.
- Examples no longer depend on Bevy/`bevy_vello`; they render with plain
  `winit` + `vello::util::RenderContext` so vello upgrades are no longer
  blocked on `bevy_vello` releases.

## [0.14.0] - 2026-09-11

### Changed

- **Breaking:** Require `roughr` 0.13. This crate's public API exposes `roughr`
  types (`Options`, `Point2D`, `OpSetType`), so they now come from `roughr`
  0.13; see the [roughr changelog](../roughr/CHANGELOG.md).

## [0.13.0] - 2025-08-29

- Previous releases; see the
  [git history](https://github.com/orhanbalci/rough-rs/commits/main/rough_vello) for details.

[Unreleased]: https://github.com/orhanbalci/rough-rs/compare/rough_vello@0.14.0...HEAD
[0.14.0]: https://github.com/orhanbalci/rough-rs/compare/rough_vello@0.13.0...rough_vello@0.14.0
[0.13.0]: https://github.com/orhanbalci/rough-rs/releases/tag/rough_vello@0.13.0
