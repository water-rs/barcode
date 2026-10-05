//! Barcode and QR code rendering for `WaterUI`.
//!
//! Barcodes are drawn as vector geometry through `waterui-graphics`'
//! render-target-neutral recording contract, so one implementation renders
//! on the GPU engine, the CPU engine used on embedded targets, and any
//! backend that hosts a scene.
//!
//! # Architecture
//!
//! 1. **Matrix generation**: encoders produce the module matrix on CPU.
//! 2. **Geometry**: dark modules become one filled path, with horizontally
//!    adjacent modules collapsed into a single rectangle per bar.
//! 3. **Recording**: the path and its paints are recorded through
//!    [`waterui_graphics::draw::Recorder`], bound as live operands so a
//!    payload or color signal update reaches the retained scene in place —
//!    leaving resolution, anti-aliasing, and rasterization to the render
//!    target.
//!
//! Rasterizing a barcode into a standalone image needs a GPU device, so
//! `BarcodeSource::generate` sits behind the non-default `gpu` feature.
//! Drawing a barcode into a view does not.
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
#[doc(hidden)]
pub use qr::reactive_source;
pub use qr::{BarcodeError, BarcodeMatrix, BarcodeSource, BarcodeSymbology};
pub use renderer::BarcodeRenderer;
pub use view::{Barcode, BarcodeFill, BarcodeSceneFill, code128, qr_code};
