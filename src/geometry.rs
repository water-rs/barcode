//! Module geometry shared by the plain and the ink-filled barcode.
//!
//! Both lay a barcode out the same way, so the placement rules — quiet zone,
//! square-module preservation, module size — live here once and are read as
//! vector geometry rather than sampled per pixel.

use waterui_core::layout::{Point, Rect, Size};
use waterui_graphics::canvas::{Path, PathBuilder};

use crate::qr::BarcodeSource;

/// The size a barcode *is*, for `Canvas::intrinsic_size`.
///
/// It is [`BarcodeSource::output_size`] at one unit per pixel — the box a
/// standalone rasterization of the same symbol would have. On a reactive
/// source, `snapshot` already reflects the latest payload, so layout always
/// measures the symbol that is about to be drawn.
pub fn natural_size(source: &BarcodeSource) -> Size {
    let (width, height) = source.output_size();
    Size::new(units(width), units(height))
}

#[expect(
    clippy::cast_precision_loss,
    reason = "an extent past 2^24 units is not a barcode anyone lays out; below it the cast is exact"
)]
const fn units(count: u32) -> f32 {
    count as f32
}

/// The rectangle a barcode occupies inside a surface of `size`, quiet zone
/// included.
///
/// Symbologies whose modules must stay square (QR) are centred inside the
/// largest square the surface holds; linear symbologies stretch to fill it.
pub fn content_rect(source: &BarcodeSource, surface: Size) -> Rect {
    if source.preserves_square_modules() {
        let side = surface.width.min(surface.height);
        Rect::new(
            Point::new((surface.width - side) / 2.0, (surface.height - side) / 2.0),
            Size::new(side, side),
        )
    } else {
        Rect::from_size(surface)
    }
}

/// One path covering every dark module of `source`, laid out inside `area`.
///
/// Horizontally adjacent dark modules become a single rectangle: the run is
/// exactly the shape they would union into, and collapsing it keeps the path
/// proportional to the barcode's bar count rather than its module count.
pub fn dark_module_path(source: &BarcodeSource, area: Rect) -> Path {
    let matrix = source.matrix();
    let quiet_x = source.quiet_zone();
    let quiet_y = source.vertical_quiet_zone();
    let cell_width = area.width() / units(matrix.width + quiet_x * 2);
    let cell_height = area.height() / units(matrix.height + quiet_y * 2);

    let mut path = Path::builder();
    for row in 0..matrix.height {
        let top = units(row + quiet_y).mul_add(cell_height, area.y());
        let bottom = top + cell_height;
        let mut run_start: Option<u32> = None;
        // One column past the last closes a run that reaches the right edge.
        for column in 0..=matrix.width {
            let dark = column < matrix.width && matrix.is_dark(column, row);
            match (dark, run_start) {
                (true, None) => run_start = Some(column),
                (false, Some(start)) => {
                    let left = units(start + quiet_x).mul_add(cell_width, area.x());
                    let right = units(column + quiet_x).mul_add(cell_width, area.x());
                    path = push_rect(path, left, top, right, bottom);
                    run_start = None;
                }
                (true, Some(_)) | (false, None) => {}
            }
        }
    }
    path.build()
}

/// Appends the rectangle to `path` as its own closed, clockwise subpath.
fn push_rect(path: PathBuilder, left: f32, top: f32, right: f32, bottom: f32) -> PathBuilder {
    path.move_to(Point::new(left, top))
        .line_to(Point::new(right, top))
        .line_to(Point::new(right, bottom))
        .line_to(Point::new(left, bottom))
        .close()
}

#[cfg(test)]
mod tests {
    use super::{content_rect, dark_module_path, natural_size, units};
    use crate::qr::reactive_source;
    use crate::{BarcodeSource, BarcodeSymbology};
    use nami::signal::IntoComputed as _;
    use waterui_core::layout::{Point, Rect, Size};
    use waterui_core::{Binding, Signal as _, Str};

    /// A static barcode is its output size, at one unit per pixel.
    #[test]
    fn a_static_barcode_measures_at_its_output_size() {
        let mut source = BarcodeSource::code128("HELLO").expect("static payload must encode");
        source.set_size(40);
        let (width, height) = source.output_size();

        assert_eq!(
            natural_size(&source),
            Size::new(units(width), units(height))
        );
    }

    /// Between a content change and the frame that draws it, the size is the
    /// symbol about to be drawn: the source signal's snapshot is already
    /// re-encoded when layout asks.
    #[test]
    fn a_reencoded_symbol_wins_over_the_one_on_screen() {
        let content = Binding::container(Str::from_static("A"));
        let source = reactive_source(BarcodeSymbology::Code128, &content.clone().into_computed());
        let (narrow, _) = source.snapshot().output_size();

        content.set(Str::from_static("A MUCH LONGER PAYLOAD"));
        let (wide, _) = source.snapshot().output_size();

        assert!(wide > narrow, "the fixture must change the bar count");
        assert_eq!(
            natural_size(&source.snapshot()),
            Size::new(units(wide), 256.0)
        );
    }

    #[test]
    fn qr_content_is_the_centred_square() {
        let source = BarcodeSource::qr("https://waterui.dev").expect("static payload must encode");
        let area = content_rect(&source, Size::new(400.0, 200.0));

        assert_eq!(
            area,
            Rect::new(Point::new(100.0, 0.0), Size::new(200.0, 200.0))
        );
    }

    #[test]
    fn code128_content_fills_the_surface() {
        let source = BarcodeSource::code128("HELLO-WATERUI").expect("static payload must encode");
        let area = content_rect(&source, Size::new(400.0, 200.0));

        assert_eq!(area, Rect::from_size(Size::new(400.0, 200.0)));
    }

    #[test]
    fn dark_modules_stay_inside_the_quiet_zone() {
        let source = BarcodeSource::qr("https://waterui.dev").expect("static payload must encode");
        let area = content_rect(&source, Size::new(256.0, 256.0));
        let bounds = dark_module_path(&source, area).bounds();

        let quiet = area.width() * units(source.quiet_zone())
            / units(source.matrix().width + source.quiet_zone() * 2);
        assert!(
            bounds.min_x() >= area.min_x() + quiet - 1e-3,
            "left edge {} must clear the {quiet}pt quiet zone",
            bounds.min_x()
        );
        assert!(
            bounds.max_x() <= area.max_x() - quiet + 1e-3,
            "right edge {} must clear the {quiet}pt quiet zone",
            bounds.max_x()
        );
        assert!(bounds.min_y() >= area.min_y() + quiet - 1e-3);
        assert!(bounds.max_y() <= area.max_y() - quiet + 1e-3);
    }

    #[test]
    fn code128_bars_span_the_full_height() {
        let source = BarcodeSource::code128("HELLO-WATERUI").expect("static payload must encode");
        let area = content_rect(&source, Size::new(512.0, 128.0));
        let bounds = dark_module_path(&source, area).bounds();

        assert!(bounds.min_y().abs() < 1e-3);
        assert!((bounds.max_y() - 128.0).abs() < 1e-3);
    }
}
