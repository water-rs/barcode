# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/water-rs/barcode/compare/v0.3.0...v0.4.0) - 2026-11-14

### Added

- *(gpu)* rasterize a barcode through render_drawing
- [**breaking**] draw through the waterui Canvas API
- port to the render-target-neutral recording contract
- implement SceneContent::rebuild_for_engine engine-recovery contract
- [**breaking**] port to the Cherenkov recording API

### Fixed

- raise the MSRV to the framework floor and write generated images under the test tmpdir
- repin hydrolysis to 14d5e231, m3 to fcb8b9e8; patch all registry waterkit crates to 123b0581
- *(ci)* pin the toolchain without disarming the MSRV leg ([#29](https://github.com/water-rs/barcode/pull/29))

### Other

- *(deps)* point migrated Batch 2 git dependencies to main ([#58](https://github.com/water-rs/barcode/pull/58))
- update release-plz rev for Rust 1.100 compatibility
- release from main and retire dev ([#53](https://github.com/water-rs/barcode/pull/53))
- restore the Hydrolysis test fonts from the Actions cache ([#52](https://github.com/water-rs/barcode/pull/52))
- *(deps)* point migrated Batch 1 git dependencies to main ([#49](https://github.com/water-rs/barcode/pull/49)) ([#50](https://github.com/water-rs/barcode/pull/50))
- name the offscreen renderer through waterui_graphics::gpu
- consume the framework from its dev branch
- lock the framework at its rebased nami 0.13 head
- lock the sibling components at their nami 0.13 heads
- [**breaking**] move to nami 0.13 and the migrated framework
- pass the release pull request's gate without CI
- [**breaking**] move the patch table to nami 0.12
- require nami 0.12
- install Hydrolysis test fonts before tests
- *(deps)* repin water-rs/waterui to 1e359bca
- *(deps)* repin the engine stack to in-tree Cherenkov at waterui 4dacae48
- *(deps)* pin hydrolysis, hydrolysis-m3 and video-gpu at the repinned heads
- pin cherenkov to 52236561
- install SPIRV-Tools through the shared Linux deps action
- install SPIRV-Tools for cherenkov-gpu's shader build on non-Apple legs
- repin to the Cherenkov cutover base
- repin cherenkov to its rewritten history
- repin the vello fork to the glifo canonical-location fix
- repin water-rs dependencies to merged dev heads
- repin water-rs dependencies to unified migration heads
- repin water-rs dependencies to unified migration heads
- repin water-rs dependencies to unified migration heads
- repin hydrolysis to accessibility import fix
- refresh lockfile for final branch revs
- repin migrated dependency branch heads
- [**breaking**] migrate to nami snapshot API
- consume the 0.5.0 framework wave from crates.io ([#28](https://github.com/water-rs/barcode/pull/28))

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
