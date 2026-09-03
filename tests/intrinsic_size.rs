//! A barcode has a size of its own: its output size.
//!
//! These are the layout claims behind `SceneContent::intrinsic_size` for a
//! barcode, checked through the semantic runtime: a vertical scroll view names
//! the width and leaves the height open, which is exactly the case that used
//! to collapse to zero.

use hydrolysis_m3::install as install_m3;
use waterui::ViewExt as _;
use waterui::layout::scroll::ScrollView;
use waterui_barcode::Barcode;
use waterui_testing::{Role, SemanticApp, ui};

const QR_LABEL: &str = "QR code: https://waterui.dev";

fn bounds(app: &mut SemanticApp, label: &str) -> (f32, f32) {
    let bounds = app.query().role(Role::IMAGE).label(label).single().bounds();
    (bounds.width(), bounds.height())
}

fn assert_close(actual: (f32, f32), expected: (f32, f32)) {
    assert!(
        (actual.0 - expected.0).abs() < 0.5 && (actual.1 - expected.1).abs() < 0.5,
        "expected {}x{}, got {}x{}",
        expected.0,
        expected.1,
        actual.0,
        actual.1
    );
}

/// A QR code is square, so the named width carries to the open height.
#[test]
fn a_qr_code_stays_square_on_an_unconstrained_axis() {
    let mut app = ui()
        .theme(install_m3)
        .viewport(300, 200)
        .mount(|| ScrollView::vertical(Barcode::qr("https://waterui.dev")));
    assert_close(bounds(&mut app, QR_LABEL), (300.0, 300.0));
}

/// Given a box, a barcode still fills it: the output size is what layout falls
/// back to, never a cap on what a container may ask for.
#[test]
fn a_barcode_still_fills_a_frame() {
    let mut app = ui()
        .theme(install_m3)
        .viewport(400, 400)
        .mount(|| Barcode::code128("HELLO-WATERUI-128").size(180.0, 80.0));
    assert_close(
        bounds(&mut app, "Code 128 barcode: HELLO-WATERUI-128"),
        (180.0, 80.0),
    );
}
