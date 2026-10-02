//! Crate-boundary checks for the `barcode` component.
//!
//! These complement the mounted tests: a `Barcode` view's `body` must wrap
//! its `SceneView` leaf in `AccessibilityRole::Image` and an
//! `AccessibilityLabel` carrying the encoded payload, and
//! `SceneView::intrinsic_size` must report the barcode's output size while
//! `resolve_scene_proposal` — the rule the mounted scroll-view case
//! exercises — carries a named axis to an open one and honours a fully
//! constrained box.

use waterui_barcode::{Barcode, BarcodeRenderer, BarcodeSource};
use waterui_core::accessibility::{AccessibilityLabel, AccessibilityRole};
use waterui_core::layout::{ProposalSize, Size};
use waterui_core::metadata::IgnorableMetadata;
use waterui_core::{AnyView, Environment, Signal as _, View as _};
use waterui_graphics::{SceneView, resolve_scene_proposal};

/// Peels `body`'s metadata wrappers, asserting the inner `SceneView` leaf
/// after checking the label and role on the way down.
fn unwrap_scene_leaf(view: AnyView, expected_label: &str) {
    let label = view
        .downcast::<IgnorableMetadata<AccessibilityLabel>>()
        .unwrap_or_else(|view| panic!("expected AccessibilityLabel metadata, got {}", view.name()));
    assert_eq!(
        label.value.signal().snapshot().as_ref() as &str,
        expected_label
    );

    let role = label
        .content
        .downcast::<IgnorableMetadata<AccessibilityRole>>()
        .unwrap_or_else(|view| panic!("expected AccessibilityRole metadata, got {}", view.name()));
    assert!(matches!(role.value, AccessibilityRole::Image));

    role.content
        .downcast::<SceneView>()
        .unwrap_or_else(|view| panic!("expected a SceneView leaf, got {}", view.name()));
}

#[test]
fn qr_barcode_exposes_accessible_image_semantics() {
    let env = Environment::new();
    let view = AnyView::new(Barcode::qr("https://waterui.dev/testing").body(&env));
    unwrap_scene_leaf(view, "QR code: https://waterui.dev/testing");
}

#[test]
fn code128_barcode_exposes_accessible_image_semantics() {
    let env = Environment::new();
    let view = AnyView::new(Barcode::code128("HELLO-WATERUI-128").body(&env));
    unwrap_scene_leaf(view, "Code 128 barcode: HELLO-WATERUI-128");
}

fn qr_view() -> SceneView {
    let env = Environment::new();
    SceneView::new(BarcodeRenderer::new(
        BarcodeSource::qr("https://waterui.dev").expect("static payload must encode"),
        &env,
    ))
}

fn code128_view() -> SceneView {
    let env = Environment::new();
    SceneView::new(BarcodeRenderer::new(
        BarcodeSource::code128("HELLO-WATERUI-128").expect("static payload must encode"),
        &env,
    ))
}

/// The view reports the barcode's output size as its own.
#[test]
fn a_scene_view_reports_the_output_size() {
    let view = qr_view();
    assert_eq!(view.intrinsic_size(), Some(Size::new(256.0, 256.0)));
}

/// A QR code is square, so the named width carries to the open height — the
/// exact case a vertical scroll view hits, which used to collapse to zero.
#[test]
fn a_qr_code_stays_square_on_an_unconstrained_axis() {
    let proposal =
        resolve_scene_proposal(qr_view().intrinsic_size(), ProposalSize::new(300.0, None));
    assert_eq!(proposal, ProposalSize::new(300.0, 300.0));
}

/// A linear barcode's height likewise carries to the open height when only
/// the width is named, and vice versa.
#[test]
fn a_code128_carries_the_named_axis_to_the_open_one() {
    let intrinsic = code128_view().intrinsic_size();
    let natural = intrinsic.expect("a barcode has an intrinsic size");

    let proposal = resolve_scene_proposal(intrinsic, ProposalSize::new(None, 80.0));
    assert_eq!(
        proposal,
        ProposalSize::new(natural.width * (80.0 / natural.height), 80.0)
    );
}

/// Given a box, a barcode still fills it: the output size is what layout falls
/// back to, never a cap on what a container may ask for.
#[test]
fn a_barcode_still_fills_a_frame() {
    let proposal = resolve_scene_proposal(
        code128_view().intrinsic_size(),
        ProposalSize::new(180.0, 80.0),
    );
    assert_eq!(proposal, ProposalSize::new(180.0, 80.0));
}
