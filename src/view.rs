use crate::draw::{Ink, ResolvedFill, filled_canvas, inked_canvas, resolve_color};
use crate::{BarcodeSymbology, reactive_source};
use core::fmt;
use nami::{Computed, SignalExt as _, signal::IntoComputed};
use waterui_core::{
    AnyView, Environment, Str, View,
    accessibility::{AccessibilityLabel, AccessibilityRole},
    layout::UnitPoint,
    metadata::IgnorableMetadata,
};
use waterui_graphics::color::Color;

/// Fill style for dark barcode modules.
///
/// Colors are kept as the unresolved [`Color`] type so that barcode views
/// participate in the same theme/HDR resolution flow as the rest of the
/// component library. The renderer observes those resolved colors for its
/// entire lifetime.
#[derive(Clone, Debug)]
pub enum BarcodeFill {
    /// Solid color fill.
    Solid(Computed<Color>),
    /// Linear gradient fill in normalized barcode-space coordinates.
    LinearGradient {
        /// Gradient start color.
        start_color: Computed<Color>,
        /// Gradient end color.
        end_color: Computed<Color>,
        /// Normalized gradient start point in barcode space.
        start_point: UnitPoint,
        /// Normalized gradient end point in barcode space.
        end_point: UnitPoint,
    },
}

impl BarcodeFill {
    /// Creates a reactive solid module fill.
    #[must_use]
    pub fn solid(color: impl IntoComputed<Color>) -> Self {
        Self::Solid(color.into_computed())
    }

    /// Creates a reactive linear-gradient module fill.
    #[must_use]
    pub fn linear_gradient(
        start_color: impl IntoComputed<Color>,
        end_color: impl IntoComputed<Color>,
        start_point: impl Into<UnitPoint>,
        end_point: impl Into<UnitPoint>,
    ) -> Self {
        Self::LinearGradient {
            start_color: start_color.into_computed(),
            end_color: end_color.into_computed(),
            start_point: start_point.into(),
            end_point: end_point.into(),
        }
    }
}

impl Default for BarcodeFill {
    fn default() -> Self {
        Self::solid(Color::from(waterui_graphics::color::Srgb::BLACK))
    }
}

fn default_light_color() -> Computed<Color> {
    Computed::constant(Color::from(waterui_graphics::color::Srgb::WHITE))
}

fn barcode_label(symbology: BarcodeSymbology, content: &Computed<Str>) -> AccessibilityLabel {
    let kind = match symbology {
        BarcodeSymbology::Qr => "QR code",
        BarcodeSymbology::Code128 => "Code 128 barcode",
    };
    AccessibilityLabel::new(content.map(move |content| Str::from(format!("{kind}: {content}"))))
}

fn apply_barcode_semantics(
    env: &Environment,
    symbology: BarcodeSymbology,
    content: &Computed<Str>,
    view: impl View,
) -> AnyView {
    let mut view = AnyView::new(view);
    if env.get::<AccessibilityRole>().is_none() {
        view = AnyView::new(IgnorableMetadata::new(view, AccessibilityRole::Image));
    }
    if env.get::<AccessibilityLabel>().is_none() {
        view = AnyView::new(IgnorableMetadata::new(
            view,
            barcode_label(symbology, content),
        ));
    }
    view
}

/// A view that renders a barcode.
///
/// `Barcode` draws QR and linear barcodes as vector geometry on a `Canvas`,
/// so whichever renderer draws the screen around it draws the symbol.
///
/// # Example
///
/// ```rust
/// use waterui_barcode::Barcode;
///
/// Barcode::qr("https://waterui.dev");
/// Barcode::code128("HELLO-WATERUI");
/// ```
#[derive(Clone, Debug)]
pub struct Barcode {
    symbology: BarcodeSymbology,
    content: Computed<Str>,
    fill: BarcodeFill,
    light_color: Computed<Color>,
}

/// Barcode view whose dark modules are filled by an [`Ink`].
pub struct BarcodeSceneFill<I: Ink> {
    symbology: BarcodeSymbology,
    content: Computed<Str>,
    ink: I,
    light_color: Computed<Color>,
}

impl<I: Ink> fmt::Debug for BarcodeSceneFill<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BarcodeSceneFill").finish_non_exhaustive()
    }
}

impl Barcode {
    /// Creates a QR code view.
    ///
    /// Content is a signal: a rotating token or live pairing URL re-encodes
    /// the matrix in place without rebuilding the view.
    ///
    /// # Panics
    ///
    /// The view panics while rendering when a content value exceeds QR
    /// capacity. Pre-validate runtime user input with [`BarcodeSource::qr`].
    ///
    /// [`BarcodeSource::qr`]: crate::BarcodeSource::qr
    pub fn qr(content: impl IntoComputed<Str>) -> Self {
        Self {
            symbology: BarcodeSymbology::Qr,
            content: content.into_computed(),
            fill: BarcodeFill::default(),
            light_color: default_light_color(),
        }
    }

    /// Creates a Code128 barcode view.
    ///
    /// Content is a signal: a changing order id re-encodes the matrix in
    /// place without rebuilding the view.
    ///
    /// # Panics
    ///
    /// The view panics while rendering when a content value contains
    /// characters Code128 cannot represent. Pre-validate runtime user input
    /// with [`BarcodeSource::code128`].
    ///
    /// [`BarcodeSource::code128`]: crate::BarcodeSource::code128
    pub fn code128(content: impl IntoComputed<Str>) -> Self {
        Self {
            symbology: BarcodeSymbology::Code128,
            content: content.into_computed(),
            fill: BarcodeFill::default(),
            light_color: default_light_color(),
        }
    }

    /// Sets a solid dark module color.
    #[must_use]
    pub fn dark_color(mut self, color: impl IntoComputed<Color>) -> Self {
        self.fill = BarcodeFill::solid(color);
        self
    }

    /// Sets a linear gradient fill for dark modules.
    ///
    /// Gradient coordinates are normalized to the barcode square via
    /// [`UnitPoint`]: `UnitPoint::TOP_LEADING` = top-left,
    /// `UnitPoint::BOTTOM_TRAILING` = bottom-right.
    #[must_use]
    pub fn linear_gradient(
        mut self,
        start_color: impl IntoComputed<Color>,
        end_color: impl IntoComputed<Color>,
        start_point: impl Into<UnitPoint>,
        end_point: impl Into<UnitPoint>,
    ) -> Self {
        self.fill = BarcodeFill::linear_gradient(start_color, end_color, start_point, end_point);
        self
    }

    /// Sets the light module/background color.
    #[must_use]
    pub fn light_color(mut self, color: impl IntoComputed<Color>) -> Self {
        self.light_color = color.into_computed();
        self
    }

    /// Fills dark modules with arbitrary canvas content.
    ///
    /// `ink` is a [`Drawing`](waterui_graphics::canvas::Drawing) or a closure
    /// drawing on the barcode's canvas; it draws across the whole box and is
    /// clipped to the dark modules.
    #[must_use]
    pub fn fill_scene<I: Ink>(self, ink: I) -> BarcodeSceneFill<I> {
        BarcodeSceneFill {
            symbology: self.symbology,
            content: self.content,
            ink,
            light_color: self.light_color,
        }
    }
}

impl<I: Ink> BarcodeSceneFill<I> {
    /// Sets light module/background color for the masked barcode output.
    #[must_use]
    pub fn light_color(mut self, color: impl IntoComputed<Color>) -> Self {
        self.light_color = color.into_computed();
        self
    }
}

impl View for Barcode {
    fn body(self, env: &Environment) -> impl View {
        let Self {
            symbology,
            content,
            fill,
            light_color,
        } = self;
        let source = reactive_source(symbology, &content);
        let canvas = filled_canvas(
            source,
            resolve_color(light_color, env),
            ResolvedFill::new(fill, env),
        );
        apply_barcode_semantics(env, symbology, &content, canvas)
    }
}

impl<I: Ink> View for BarcodeSceneFill<I> {
    fn body(self, env: &Environment) -> impl View {
        let source = reactive_source(self.symbology, &self.content);
        let canvas = inked_canvas(source, resolve_color(self.light_color, env), self.ink);
        apply_barcode_semantics(env, self.symbology, &self.content, canvas)
    }
}

/// Free-function entry point for a QR code view.
///
/// See [`Barcode::qr`] for content semantics and panic conditions.
#[must_use]
pub fn qr_code(content: impl IntoComputed<Str>) -> Barcode {
    Barcode::qr(content)
}

/// Free-function entry point for a Code128 barcode view.
///
/// See [`Barcode::code128`] for content semantics and panic conditions.
#[must_use]
pub fn code128(content: impl IntoComputed<Str>) -> Barcode {
    Barcode::code128(content)
}
