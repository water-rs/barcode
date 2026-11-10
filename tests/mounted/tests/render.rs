//! Rendered barcodes: every fill the component draws, rasterized through the
//! offscreen Hydrolysis renderer and decoded back to the payload it encodes.
//! The PNGs are written for review by eye.

use std::path::PathBuf;

use hydrolysis_m3::Material3;
use rxing::{
    BarcodeFormat, BinaryBitmap, DecodeHints, Luma8LuminanceSource, MultiFormatReader, Reader,
    common::HybridBinarizer,
};
use waterui::SignalExt as _;
use waterui::ViewExt as _;
use waterui::graphics::canvas::{Drawing, Interpolation, LinearGradient};
use waterui::graphics::color::{Srgb, WorkingColor};
use waterui_barcode::Barcode;
use waterui_core::layout::{Point, Rect, Size, UnitPoint};
use waterui_testing::{Snapshot, ui};

const QR_CONTENT: &str = "https://waterui.dev";
const STYLED_CONTENT: &str = "https://waterui.dev/styled";
const CODE128_CONTENT: &str = "HELLO-WATERUI-128";

fn teal() -> WorkingColor {
    Srgb::new(0.0, 0.35, 0.4).into()
}

fn navy() -> WorkingColor {
    Srgb::new(0.05, 0.0, 0.3).into()
}

/// A barcode filling a `width` x `height` viewport.
fn render(width: u16, height: u16, view: impl Fn() -> Barcode + 'static) -> Snapshot {
    render_with(width, height, move || {
        view().size(f32::from(width), f32::from(height))
    })
}

fn render_with<V: waterui::View + 'static>(
    width: u16,
    height: u16,
    view: impl Fn() -> V + 'static,
) -> Snapshot {
    let mut app = ui()
        .theme(Material3::defaults())
        .viewport(u32::from(width), u32::from(height))
        .mount_offscreen(view);
    app.snapshot()
}

fn decode(snapshot: &Snapshot, format: BarcodeFormat) -> String {
    let luminance = snapshot
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
    let source = Luma8LuminanceSource::new(luminance, snapshot.width(), snapshot.height())
        .expect("snapshot dimensions must match the pixel buffer");
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
        .expect("the rendered symbol must decode as the requested barcode format")
        .getText()
        .to_owned()
}

fn save(snapshot: &Snapshot, variable: &str, default: &str) {
    let path = std::env::var(variable).map_or_else(|_| PathBuf::from(default), PathBuf::from);
    snapshot.save_png(&path).expect("the png is writable");
}

#[test]
fn qr_renders_and_decodes() {
    let snapshot = render(768, 768, || Barcode::qr(QR_CONTENT));
    assert_eq!(decode(&snapshot, BarcodeFormat::QR_CODE), QR_CONTENT);
    save(&snapshot, "WATERUI_QR_OUT", "target/generated_qr.png");
}

#[test]
fn code128_renders_and_decodes() {
    let snapshot = render(1024, 256, || Barcode::code128(CODE128_CONTENT));
    assert_eq!(decode(&snapshot, BarcodeFormat::CODE_128), CODE128_CONTENT);
    save(
        &snapshot,
        "WATERUI_CODE128_OUT",
        "target/generated_code128.png",
    );
}

#[test]
fn gradient_filled_qr_still_decodes() {
    let snapshot = render(768, 768, || {
        Barcode::qr(STYLED_CONTENT).linear_gradient(
            waterui::graphics::color::Color::from(Srgb::new(0.0, 0.25, 0.3)),
            waterui::graphics::color::Color::from(Srgb::new(0.35, 0.0, 0.25)),
            UnitPoint::TOP_LEADING,
            UnitPoint::BOTTOM_TRAILING,
        )
    });
    assert_eq!(decode(&snapshot, BarcodeFormat::QR_CODE), STYLED_CONTENT);
    save(
        &snapshot,
        "WATERUI_QR_GRADIENT_OUT",
        "target/generated_qr_gradient.png",
    );
}

/// A canvas closure as ink: the diagonal gradient is clipped to the dark
/// modules, so the symbol stays decodable only when the clip geometry is
/// right.
#[test]
fn closure_ink_qr_still_decodes() {
    let snapshot = render_with(768, 768, || {
        Barcode::qr(STYLED_CONTENT)
            .fill_scene(|cx: &mut waterui::graphics::canvas::Cx| {
                let size = cx.size();
                cx.fill(
                    size.clone().map(Rect::from_size),
                    size.map(|size: Size| {
                        LinearGradient::new(Point::zero(), Point::new(size.width, size.height))
                            .stop(0.0, teal())
                            .stop(1.0, navy())
                            .interpolation(Interpolation::Encoded)
                    }),
                );
            })
            .size(768.0, 768.0)
    });
    assert_eq!(decode(&snapshot, BarcodeFormat::QR_CODE), STYLED_CONTENT);
    save(
        &snapshot,
        "WATERUI_QR_MASKED_OUT",
        "target/generated_qr_masked.png",
    );
}

/// A recorded drawing as ink, scaled over the whole box.
#[test]
fn drawing_ink_qr_still_decodes() {
    let ink = Drawing::record(Size::new(100.0, 100.0), |cx| {
        cx.fill(
            Rect::new(Point::zero(), Size::new(100.0, 100.0)),
            LinearGradient::new(Point::zero(), Point::new(100.0, 100.0))
                .stop(0.0, teal())
                .stop(1.0, navy())
                .interpolation(Interpolation::Encoded),
        );
    });
    let snapshot = render_with(768, 768, move || {
        Barcode::qr(STYLED_CONTENT)
            .fill_scene(ink.clone())
            .size(768.0, 768.0)
    });
    assert_eq!(decode(&snapshot, BarcodeFormat::QR_CODE), STYLED_CONTENT);
    save(
        &snapshot,
        "WATERUI_QR_DRAWING_OUT",
        "target/generated_qr_drawing.png",
    );
}
