//! `BarcodeSource::generate`: standalone images decode back to their payload.
#![cfg(feature = "gpu")]

use std::path::PathBuf;

use rxing::{
    BarcodeFormat, BinaryBitmap, DecodeHints, Luma8LuminanceSource, MultiFormatReader, Reader,
    common::HybridBinarizer,
};
use waterui_barcode::BarcodeSource;
use waterui_graphics::{OffscreenImage, OffscreenRenderer, cherenkov_gpu::Gpu};

fn decode(image: &OffscreenImage, format: BarcodeFormat) -> String {
    let luminance = image
        .rgba8()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|&[red, green, blue, _alpha]| {
            u8::try_from(
                (u32::from(red) * 2126 + u32::from(green) * 7152 + u32::from(blue) * 722) / 10_000,
            )
            .expect("weighted RGB luminance must fit into one byte")
        })
        .collect();
    let source = Luma8LuminanceSource::new(luminance, image.width(), image.height())
        .expect("image dimensions must match the pixel buffer");
    let mut bitmap = BinaryBitmap::new(HybridBinarizer::new(source));
    MultiFormatReader::default()
        .decode_with_hints(
            &mut bitmap,
            &DecodeHints {
                PossibleFormats: Some([format].into()),
                TryHarder: Some(true),
                ..DecodeHints::default()
            },
        )
        .expect("the generated image must decode as the requested barcode format")
        .getText()
        .to_owned()
}

fn out_dir() -> PathBuf {
    let dir = PathBuf::from(
        "/private/tmp/claude-501/-Users-lexoliu-Coding-waterui/161051f2-1da9-410b-8d87-b5d503d54de0/scratchpad/barcode44",
    );
    std::fs::create_dir_all(&dir).expect("output directory must be creatable");
    dir
}

fn check(source: &BarcodeSource, format: BarcodeFormat, payload: &str, file: &str) {
    let renderer =
        OffscreenRenderer::<Gpu>::new().expect("this test requires a working GPU renderer");
    let image = source.generate(&renderer);
    let (width, height) = source.output_size();
    assert_eq!((image.width(), image.height()), (width, height));
    assert_eq!(image.rgba8().len(), width as usize * height as usize * 4);
    image
        .save_png(out_dir().join(file))
        .expect("png must be writable");
    assert_eq!(decode(&image, format), payload);
}

#[test]
fn qr_generates_a_decodable_image() {
    let payload = "https://waterui.dev";
    let mut source = BarcodeSource::qr(payload).expect("static test payload must encode");
    source.set_size(768);
    check(&source, BarcodeFormat::QR_CODE, payload, "generate_qr.png");
}

#[test]
fn code128_generates_a_decodable_image() {
    let payload = "HELLO-WATERUI-128";
    let mut source = BarcodeSource::code128(payload).expect("static test payload must encode");
    // A Code128 symbol is its bar count wide at the smallest extent; a whole
    // number of pixels per module keeps the bars sharp enough to decode.
    source.set_size(1);
    let (bars, _) = source.output_size();
    source.set_size(bars * 4);
    check(
        &source,
        BarcodeFormat::CODE_128,
        payload,
        "generate_code128.png",
    );
}
