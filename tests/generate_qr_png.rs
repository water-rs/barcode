//! Generate a QR PNG by rasterizing the barcode scene offscreen.

use std::path::PathBuf;

use rxing::BarcodeFormat;
use waterui_barcode::{BarcodeRenderer, BarcodeSource};
use waterui_graphics::{
    cherenkov_gpu::Gpu,
    offscreen::{OffscreenRenderer, OffscreenSize},
};

mod support;

#[test]
fn generate_qr_png_offscreen() {
    let content =
        std::env::var("WATERUI_QR_CONTENT").unwrap_or_else(|_| "https://waterui.dev".to_string());
    let out_path = std::env::var("WATERUI_QR_OUT")
        .map_or_else(|_| PathBuf::from("target/generated_qr.png"), PathBuf::from);

    let env = waterui_core::Environment::new();
    let mut renderer = BarcodeRenderer::new(
        BarcodeSource::qr(content.clone()).expect("static test payload must encode"),
        &env,
    );
    let size = OffscreenSize::try_from_pixels(768, 768).expect("valid output size");
    let runtime = OffscreenRenderer::<Gpu>::new().expect("barcode tests require a GPU engine");
    let output = runtime
        .render(&mut renderer, size, 1.0)
        .expect("barcode offscreen rendering failed");
    assert_eq!(
        output.rgba8.len(),
        (output.width * output.height * 4) as usize
    );
    assert_eq!(support::decode(&output, BarcodeFormat::QR_CODE), content);

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).expect("output directory should be creatable");
    }
    output.save_png(&out_path).expect("png should be writable");
    assert!(out_path.exists(), "png file should exist at {out_path:?}");
}
