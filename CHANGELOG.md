# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/water-rs/barcode/releases/tag/v0.1.0) - 2026-08-29

### Fixed

- *(release)* verify registry-only package graph
- finish the audit sweep — dead gesture payload variant and lint debt
- *(barcode)* [**breaking**] reactive content and typed encode errors
- *(barcode)* render linear codes correctly

### Other

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
