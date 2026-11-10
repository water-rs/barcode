//! Barcode and QR code rendering for `WaterUI`.
//!
//! Barcodes are drawn as vector geometry on a [`waterui_graphics::canvas`]
//! canvas, so whichever renderer draws the screen around a barcode draws it.
//!
//! # Architecture
//!
//! 1. **Matrix generation**: encoders produce the module matrix on CPU.
//! 2. **Geometry**: dark modules become one path, with horizontally adjacent
//!    modules collapsed into a single rectangle per bar.
//! 3. **Drawing**: the canvas fills the light background and the module path.
//!    The path, the colors and the gradient endpoints are live operands, so a
//!    payload, color or box-size change reaches the retained drawing in place —
//!    leaving resolution, anti-aliasing and rasterization to the renderer. The
//!    canvas' intrinsic size follows the payload.
//!
//! Rasterizing a barcode into a standalone image needs a GPU device, so
//! `BarcodeSource::generate` sits behind the non-default `gpu` feature. It
//! records the same geometry as a [`Drawing`](waterui_graphics::canvas::Drawing)
//! and renders it offscreen. Drawing a barcode on a canvas does not need it.
//!
//! A barcode's dark modules are filled by a color, a gradient, or any
//! [`Ink`] — a [`Drawing`](waterui_graphics::canvas::Drawing) or a canvas
//! closure — through [`Barcode::fill_scene`].
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

mod draw;
mod geometry;
mod qr;
mod view;

pub use draw::Ink;
#[doc(hidden)]
pub use qr::reactive_source;
pub use qr::{BarcodeError, BarcodeMatrix, BarcodeSource, BarcodeSymbology};
pub use view::{Barcode, BarcodeFill, BarcodeSceneFill, code128, qr_code};
