//! End-to-end accessibility-semantics tests for the `barcode` component.

use waterui::ViewExt as _;
use waterui_barcode::Barcode;
use waterui_testing::{Role, SemanticApp};

fn qr_barcode_view() -> impl waterui::View {
    Barcode::qr("https://waterui.dev/testing").size(180.0, 180.0)
}

#[waterui::test(qr_barcode_view)]
fn qr_barcode_exposes_accessible_image_node(app: &mut SemanticApp) {
    // The semantic tree answers identity, not geometry: the size a barcode
    // takes is `tests/intrinsic_size.rs`' business on the rendered runtime.
    app.query()
        .role(Role::IMAGE)
        .label("QR code: https://waterui.dev/testing")
        .assert_exists();
}

fn code128_barcode_view() -> impl waterui::View {
    Barcode::code128("HELLO-WATERUI-128").size(180.0, 80.0)
}

#[waterui::test(code128_barcode_view)]
fn code128_barcode_exposes_accessible_image_node(app: &mut SemanticApp) {
    app.query()
        .role(Role::IMAGE)
        .label("Code 128 barcode: HELLO-WATERUI-128")
        .assert_exists();
}
