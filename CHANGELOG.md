# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/water-rs/barcode/compare/v0.1.0...v0.2.0) - 2026-09-11

### Added

- bring the barcode demonstration home
- [**breaking**] give the barcode scene contents an intrinsic size
- [**breaking**] draw barcodes through Scene2D instead of a wgpu pipeline

### Fixed

- *(ci)* install the same Linux packages for the release preflight
- *(release)* verify registry-only package graph
- finish the audit sweep — dead gesture payload variant and lint debt
- *(barcode)* [**breaking**] reactive content and typed encode errors
- *(barcode)* render linear codes correctly

### Other

- link the test graph to the waterui 0.4 release commit
- *(deps)* waterui-graphics 0.4 (and waterui-text/-testing 0.4 where used)
- keep the example off the framework's dynamic-linking build
- refresh git dependency revisions after the upstream history rewrite
- update Linux package matrix and add dxc on Windows
- setup standalone crate files, CI workflows, and release-plz
- ship the licence texts in every published crate
- reformat the build scripts shortened by the shaderloom switch
- depend on shaderloom directly, and give the icon codegen its own name
- format audit-fix files with the workspace rustfmt
- Fix workspace CI failures
- Format workspace
- upgrade workspace dependencies
- Add cross-platform shader AOT with Shaderloom
- refactor native backends and GPU surface integration
- clean up clippy warnings across the workspace
- SubView: Send + Sync; decouple GpuView from SubView
- Lean dependency graph for embedded: gpu/widgets/gestures features
- Restore WaterUI CI gates and reactive map API
- reorganize the project
