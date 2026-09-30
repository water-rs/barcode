//! Render the two styled barcode paths — a gradient module fill and a masked
//! scene fill — and prove both still decode as the payload they encode.

use std::path::PathBuf;

use rxing::BarcodeFormat;
use waterui_barcode::{BarcodeFill, BarcodeMask, BarcodeRenderer, BarcodeSource};
use waterui_core::{Environment, layout::UnitPoint};
use waterui_graphics::{
    OffscreenImage, OffscreenRenderer, OffscreenSize, SceneContent, SceneResources,
    cherenkov::{
        Color as EngineColor, Draw as _, Fixed, Interpolation, LinearGradient, Recorder,
        Srgb as EngineSrgb,
    },
    cherenkov_gpu::Gpu,
    color::{Color, Srgb},
};

mod support;

const CONTENT: &str = "https://waterui.dev/styled";

/// Ink that paints a diagonal teal-to-navy gradient across the whole surface.
///
/// The mask clips it to the dark modules, so a barcode drawn with it stays
/// decodable only when the clip geometry is right.
#[derive(Debug)]
struct GradientInk;

impl SceneContent for GradientInk {
    fn build_scene(
        &mut self,
        recorder: &mut Recorder,
        _resources: &SceneResources,
        width: f32,
        height: f32,
    ) -> bool {
        let (width, height) = (f64::from(width), f64::from(height));
        let rect = kurbo::Rect::new(0.0, 0.0, width, height);
        recorder.fill(
            Fixed(rect),
            LinearGradient::new((0.0, 0.0), (width, height))
                .stop(0.0, EngineColor::<EngineSrgb>::new([0.0, 0.35, 0.4, 1.0]))
                .stop(1.0, EngineColor::<EngineSrgb>::new([0.05, 0.0, 0.3, 1.0]))
                .interpolation(Interpolation::SrgbEncoded),
        );
        false
    }
}

fn render(mut content: impl SceneContent, pixels: u32) -> OffscreenImage {
    let size = OffscreenSize::try_from_pixels(pixels, pixels).expect("valid output size");
    let gpu = OffscreenRenderer::<Gpu>::new()
        .expect("styled barcode tests require a working GPU renderer");
    gpu.render(&mut content, size, 1.0)
        .expect("offscreen render should succeed")
}

fn save(output: &OffscreenImage, variable: &str, default: &str) {
    let path = std::env::var(variable).map_or_else(|_| PathBuf::from(default), PathBuf::from);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("output directory should be creatable");
    }
    output.save_png(&path).expect("png should be writable");
}

#[test]
fn gradient_filled_qr_still_decodes() {
    let env = Environment::new();
    let renderer = BarcodeRenderer::new(
        BarcodeSource::qr(CONTENT).expect("static test payload must encode"),
        &env,
    )
    .with_fill(BarcodeFill::linear_gradient(
        Color::from(Srgb::new(0.0, 0.25, 0.3)),
        Color::from(Srgb::new(0.35, 0.0, 0.25)),
        UnitPoint::TOP_LEADING,
        UnitPoint::BOTTOM_TRAILING,
    ));

    let output = render(renderer, 768);

    assert_eq!(support::decode(&output, BarcodeFormat::QR_CODE), CONTENT);
    save(
        &output,
        "WATERUI_QR_GRADIENT_OUT",
        "target/generated_qr_gradient.png",
    );
}

#[test]
fn scene_masked_qr_still_decodes() {
    let env = Environment::new();
    let mask = BarcodeMask::new(
        BarcodeSource::qr(CONTENT).expect("static test payload must encode"),
        Color::from(Srgb::WHITE),
        GradientInk,
        &env,
    );

    let output = render(mask, 768);

    assert_eq!(support::decode(&output, BarcodeFormat::QR_CODE), CONTENT);
    save(
        &output,
        "WATERUI_QR_MASKED_OUT",
        "target/generated_qr_masked.png",
    );
}
