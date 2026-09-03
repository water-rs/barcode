//! Module geometry shared by the barcode scene contents.
//!
//! Both the plain renderer and the masked fill lay a barcode out the same way,
//! so the placement rules — quiet zone, square-module preservation, module size
//! — live here once and are read as vector geometry rather than sampled per
//! pixel.

use kurbo::{BezPath, Rect};
use waterui_core::layout::Size;

use crate::qr::{BarcodeSource, ReactiveBarcodeContent};

/// The size a barcode *is*, for `SceneContent::intrinsic_size`.
///
/// It is [`BarcodeSource::output_size`] at one unit per pixel — the same box
/// the image generator rasterizes — taken from the source `reactive` has
/// encoded but no frame has adopted yet when there is one, and from `current`
/// otherwise. Layout measures between a content change and the frame that
/// draws it, and wants the symbol that is about to be shown.
pub fn natural_size(current: &BarcodeSource, reactive: Option<&ReactiveBarcodeContent>) -> Size {
    let (width, height) = reactive
        .and_then(ReactiveBarcodeContent::pending_output_size)
        .unwrap_or_else(|| current.output_size());
    Size::new(extent_to_units(width), extent_to_units(height))
}

#[expect(
    clippy::cast_precision_loss,
    reason = "an extent past 2^24 units is not a barcode anyone lays out; below it the cast is exact"
)]
const fn extent_to_units(extent: u32) -> f32 {
    extent as f32
}

/// The rectangle a barcode occupies inside a `width` x `height` surface,
/// quiet zone included.
///
/// Symbologies whose modules must stay square (QR) are centred inside the
/// largest square the surface holds; linear symbologies stretch to fill it.
pub fn content_rect(source: &BarcodeSource, width: f64, height: f64) -> Rect {
    if source.preserves_square_modules() {
        let side = width.min(height);
        let x = (width - side) / 2.0;
        let y = (height - side) / 2.0;
        Rect::new(x, y, x + side, y + side)
    } else {
        Rect::new(0.0, 0.0, width, height)
    }
}

/// One path covering every dark module of `source`, laid out inside `area`.
///
/// Horizontally adjacent dark modules become a single rectangle: the run is
/// exactly the shape they would union into, and collapsing it keeps the path
/// proportional to the barcode's bar count rather than its module count.
pub fn dark_module_path(source: &BarcodeSource, area: Rect) -> BezPath {
    let matrix = source.matrix();
    let quiet_x = source.quiet_zone();
    let quiet_y = source.vertical_quiet_zone();
    let cell_width = area.width() / f64::from(matrix.width + quiet_x * 2);
    let cell_height = area.height() / f64::from(matrix.height + quiet_y * 2);

    let mut path = BezPath::new();
    for row in 0..matrix.height {
        let top = f64::from(row + quiet_y).mul_add(cell_height, area.y0);
        let bottom = top + cell_height;
        let mut run_start: Option<u32> = None;
        // One column past the last closes a run that reaches the right edge.
        for column in 0..=matrix.width {
            let dark = column < matrix.width && matrix.is_dark(column, row);
            match (dark, run_start) {
                (true, None) => run_start = Some(column),
                (false, Some(start)) => {
                    let left = f64::from(start + quiet_x).mul_add(cell_width, area.x0);
                    let right = f64::from(column + quiet_x).mul_add(cell_width, area.x0);
                    push_rect(&mut path, Rect::new(left, top, right, bottom));
                    run_start = None;
                }
                (true, Some(_)) | (false, None) => {}
            }
        }
    }
    path
}

/// Appends `rect` to `path` as its own closed, clockwise subpath.
fn push_rect(path: &mut BezPath, rect: Rect) {
    path.move_to((rect.x0, rect.y0));
    path.line_to((rect.x1, rect.y0));
    path.line_to((rect.x1, rect.y1));
    path.line_to((rect.x0, rect.y1));
    path.close_path();
}

#[cfg(test)]
mod tests {
    use super::{content_rect, dark_module_path, extent_to_units, natural_size};
    use crate::qr::ReactiveBarcodeContent;
    use crate::{BarcodeSource, BarcodeSymbology};
    use kurbo::{Rect, Shape as _};
    use nami::signal::IntoComputed as _;
    use waterui_core::layout::Size;
    use waterui_core::{Binding, Str};

    /// A static barcode is its output size, at one unit per pixel.
    #[test]
    fn a_static_barcode_measures_at_its_output_size() {
        let mut source = BarcodeSource::code128("HELLO").expect("static payload must encode");
        source.set_size(40);
        let (width, height) = source.output_size();

        assert_eq!(
            natural_size(&source, None),
            Size::new(extent_to_units(width), extent_to_units(height))
        );
    }

    /// Between a content change and the frame that adopts it, the size is the
    /// symbol about to be drawn, not the one on screen: layout asks in that gap.
    #[test]
    fn a_pending_symbol_wins_over_the_one_on_screen() {
        let content = Binding::container(Str::from_static("A"));
        let mut reactive =
            ReactiveBarcodeContent::new(BarcodeSymbology::Code128, content.clone().into_computed());
        let mut current = reactive.initial_source();
        current.set_size(1);
        let (narrow, _) = current.output_size();
        reactive.install(|| {});

        assert_eq!(
            natural_size(&current, Some(&reactive)),
            natural_size(&current, None),
            "with no pending symbol the one on screen is the answer"
        );

        content.set(Str::from_static("A MUCH LONGER PAYLOAD"));
        let measured = natural_size(&current, Some(&reactive));
        let adopted = reactive
            .take_reencoded()
            .expect("a content change must leave a source to adopt");
        let (wide, _) = adopted.output_size();
        assert!(wide > narrow, "the fixture must change the bar count");
        assert_eq!(measured, natural_size(&adopted, None));
        assert_eq!(natural_size(&adopted, Some(&reactive)), measured);
    }

    #[test]
    fn qr_content_is_the_centred_square() {
        let source = BarcodeSource::qr("https://waterui.dev").expect("static payload must encode");
        let area = content_rect(&source, 400.0, 200.0);

        assert_eq!(area, Rect::new(100.0, 0.0, 300.0, 200.0));
    }

    #[test]
    fn code128_content_fills_the_surface() {
        let source = BarcodeSource::code128("HELLO-WATERUI").expect("static payload must encode");
        let area = content_rect(&source, 400.0, 200.0);

        assert_eq!(area, Rect::new(0.0, 0.0, 400.0, 200.0));
    }

    #[test]
    fn dark_modules_stay_inside_the_quiet_zone() {
        let source = BarcodeSource::qr("https://waterui.dev").expect("static payload must encode");
        let area = content_rect(&source, 256.0, 256.0);
        let modules = dark_module_path(&source, area);
        let bounds = modules.bounding_box();

        let quiet = area.width() * f64::from(source.quiet_zone())
            / f64::from(source.matrix().width + source.quiet_zone() * 2);
        assert!(
            bounds.x0 >= area.x0 + quiet - f64::EPSILON,
            "left edge {} must clear the {quiet}pt quiet zone",
            bounds.x0
        );
        assert!(
            bounds.x1 <= area.x1 - quiet + f64::EPSILON,
            "right edge {} must clear the {quiet}pt quiet zone",
            bounds.x1
        );
        assert!(bounds.y0 >= area.y0 + quiet - f64::EPSILON);
        assert!(bounds.y1 <= area.y1 - quiet + f64::EPSILON);
    }

    #[test]
    fn code128_bars_span_the_full_height() {
        let source = BarcodeSource::code128("HELLO-WATERUI").expect("static payload must encode");
        let area = content_rect(&source, 512.0, 128.0);
        let bounds = dark_module_path(&source, area).bounding_box();

        assert!((bounds.y0 - 0.0).abs() < f64::EPSILON);
        assert!((bounds.y1 - 128.0).abs() < f64::EPSILON);
    }
}
