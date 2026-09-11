//! Barcode and QR code rendering for `WaterUI`.
//!
//! Barcodes are drawn as vector geometry through `waterui-graphics`'
//! engine-neutral [`Scene2D`] contract, so one implementation renders on the
//! GPU compute renderer, the CPU sparse-strip renderer used on embedded
//! targets, and any backend that owns its own scene.
//!
//! [`Scene2D`]: waterui_graphics::Scene2D
//!
//! # Architecture
//!
//! 1. **Matrix generation**: encoders produce the module matrix on CPU.
//! 2. **Geometry**: dark modules become one filled path, with horizontally
//!    adjacent modules collapsed into a single rectangle per bar.
//! 3. **Scene**: that path is filled through [`Scene2D::fill`], leaving
//!    resolution, anti-aliasing, and rasterization to the renderer.
//!
//! [`Scene2D::fill`]: waterui_graphics::Scene2D::fill
//!
//! Rasterizing a barcode into a standalone image needs a GPU device, so the
//! `ImageGenerator` implementation for [`BarcodeSource`] sits behind the
//! non-default `gpu` feature. Drawing a barcode into a view does not.
//!
//! # Example
//!
//! ```ignore
//! use waterui_barcode::Barcode;
//!
//! // Create a QR code view
//! Barcode::qr("https://waterui.dev")
//!
//! // Create a Code128 barcode view
//! Barcode::code128("HELLO-WATERUI")
//! ```

mod geometry;
mod mask;
mod qr;
mod renderer;
mod view;

pub use mask::BarcodeMask;
pub use qr::{BarcodeError, BarcodeMatrix, BarcodeSource, BarcodeSymbology};
pub use renderer::BarcodeRenderer;
pub use view::{Barcode, BarcodeFill, BarcodeSceneFill, code128, qr_code};
