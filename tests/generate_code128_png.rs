//! Generate a Code128 PNG by rasterizing the barcode scene offscreen.

use std::path::PathBuf;

use rxing::BarcodeFormat;
use waterui_barcode::{BarcodeRenderer, BarcodeSource};
use waterui_graphics::{
    cherenkov_gpu::Gpu,
    offscreen::{OffscreenRenderer, OffscreenSize},
};

mod support;

#[test]
fn generate_code128_png_offscreen() {
    let content = std::env::var("WATERUI_CODE128_CONTENT")
        .unwrap_or_else(|_| "HELLO-WATERUI-128".to_string());
    let out_path = std::env::var("WATERUI_CODE128_OUT").map_or_else(
        |_| PathBuf::from("/tmp/generated_code128.png"),
        PathBuf::from,
    );

    let env = waterui_core::Environment::new();
    let mut renderer = BarcodeRenderer::new(
        BarcodeSource::code128(content.clone()).expect("static test payload must encode"),
        &env,
    );
    let size = OffscreenSize::try_from_pixels(1024, 256).expect("valid output size");
    let runtime = OffscreenRenderer::<Gpu>::new().expect("barcode tests require a GPU engine");
    let output = runtime
        .render(&mut renderer, size, 1.0)
        .expect("barcode offscreen rendering failed");
    assert_eq!(
        output.rgba8.len(),
        (output.width * output.height * 4) as usize
    );
    assert_eq!(support::decode(&output, BarcodeFormat::CODE_128), content);

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).expect("output directory should be creatable");
    }
    output.save_png(&out_path).expect("png should be writable");
    assert!(out_path.exists(), "png file should exist at {out_path:?}");
}
