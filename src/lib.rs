//! Barcode and QR code rendering for `WaterUI`.
//!
//! Barcodes record signal-bound paths and paints into Cherenkov through
//! [`SceneContent`](waterui_graphics::SceneContent). The engine retains their
//! geometry, clip, and color bindings across frames.
//!
//! Encoders produce a module matrix; adjacent dark modules form path runs.
//! `BarcodeSource::generate` renders a standalone image through the engine's
//! offscreen target when the `gpu` feature is enabled.
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
