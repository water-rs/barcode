//! Vector scene content for encoded barcode matrices.

use core::fmt;

use kurbo::{BezPath, Point, Rect};
use nami::{SignalExt as _, signal::IntoComputed};
use waterui_core::layout::{Size, UnitPoint};
use waterui_core::reactive::watcher::BoxWatcherGuard;
use waterui_core::{Computed, Environment, Signal, Str, flatten_signal};
use waterui_graphics::{
    RecordingResources, SceneContent, SceneInvalidator,
    color::{Color, Srgb, WorkingColor},
    draw::{Draw as _, Fixed, Interpolation, LinearGradient, Paint, Recorder},
};

use crate::geometry::{content_rect, dark_module_path, natural_size};
use crate::{BarcodeSource, BarcodeSymbology, view::BarcodeFill};

/// A [`SceneContent`] that records a barcode as vector geometry.
///
/// The matrix is encoded on CPU once and emitted as one filled path of dark
/// modules recorded through the engine's [`Recorder`]. Geometry and colors
/// stay bound as live operands: a signal change updates the retained command
/// in place without re-recording, and only a payload change — which can
/// alter the intrinsic size — wakes the surface through the invalidator.
pub struct BarcodeRenderer {
    environment: Environment,
    source: Computed<BarcodeSource>,
    fill: ResolvedFill,
    light_color: Computed<WorkingColor>,
    layout_guard: Option<BoxWatcherGuard>,
}

/// A [`BarcodeFill`] whose colors are already resolved against an environment.
enum ResolvedFill {
    Solid(Computed<WorkingColor>),
    LinearGradient {
        start: Computed<WorkingColor>,
        end: Computed<WorkingColor>,
        start_point: UnitPoint,
        end_point: UnitPoint,
    },
}

impl ResolvedFill {
    /// The paint covering dark modules laid out inside `area`.
    ///
    /// Gradient endpoints are unit coordinates of the barcode square, so they
    /// are mapped onto `area` rather than onto the whole surface.
    fn paint(&self, area: Rect) -> Computed<Paint> {
        match self {
            Self::Solid(color) => color.clone().map(Paint::Solid).into_computed(),
            Self::LinearGradient {
                start,
                end,
                start_point,
                end_point,
            } => {
                let anchor = |point: &UnitPoint| {
                    Point::new(
                        f64::from(point.x).mul_add(area.width(), area.x0),
                        f64::from(point.y).mul_add(area.height(), area.y0),
                    )
                };
                let (from, to) = (anchor(start_point), anchor(end_point));
                start
                    .clone()
                    .zip(end)
                    .map(move |(start, end)| -> Paint {
                        // Gradients interpolate in the encoded space, matching
                        // the peniko default this mapping replaced.
                        LinearGradient::new(from, to)
                            .stop(0.0, start)
                            .stop(1.0, end)
                            .interpolation(Interpolation::SrgbEncoded)
                            .into()
                    })
                    .into_computed()
            }
        }
    }
}

/// Resolves `color` against `env`, keeping the result reactive.
pub fn resolve_color(color: impl IntoComputed<Color>, env: &Environment) -> Computed<WorkingColor> {
    let env = env.clone();
    flatten_signal(color.into_computed().map(move |color| color.resolve(&env)))
}

/// Resolves every color of `fill` against `env`.
fn resolve_fill(fill: BarcodeFill, env: &Environment) -> ResolvedFill {
    match fill {
        BarcodeFill::Solid(color) => ResolvedFill::Solid(resolve_color(color, env)),
        BarcodeFill::LinearGradient {
            start_color,
            end_color,
            start_point,
            end_point,
        } => ResolvedFill::LinearGradient {
            start: resolve_color(start_color, env),
            end: resolve_color(end_color, env),
            start_point,
            end_point,
        },
    }
}

/// The surface rectangle, or `None` when the surface has no drawable area.
pub fn surface_rect(width: f32, height: f32) -> Option<Rect> {
    let (width, height) = (f64::from(width), f64::from(height));
    (width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0)
        .then(|| Rect::new(0.0, 0.0, width, height))
}

/// The dark-module path of `source` laid out in `surface`, kept bound to the
/// payload signal: a re-encode updates the recorded operand in place.
pub fn module_path(
    source: &Computed<BarcodeSource>,
    surface: Rect,
) -> impl Signal<Output = BezPath> + use<> {
    source.map(move |source| {
        dark_module_path(
            &source,
            content_rect(&source, surface.width(), surface.height()),
        )
    })
}

impl fmt::Debug for BarcodeRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BarcodeRenderer").finish_non_exhaustive()
    }
}

impl BarcodeRenderer {
    /// Creates scene content drawing `source`, resolving colors against `env`.
    ///
    /// Defaults to a solid black fill on a white background. Use
    /// [`Self::with_fill`] and [`Self::with_light_color`] to override.
    #[must_use]
    pub fn new(source: BarcodeSource, env: &Environment) -> Self {
        Self {
            environment: env.clone(),
            source: source.into_computed(),
            fill: resolve_fill(BarcodeFill::default(), env),
            light_color: resolve_color(Color::from(Srgb::WHITE), env),
            layout_guard: None,
        }
    }

    /// Creates content whose barcode follows a signal.
    ///
    /// Content changes re-encode the matrix as a live operand of the recorded
    /// scene: the next frame draws the new modules without rebuilding the
    /// content.
    ///
    /// # Panics
    ///
    /// Panics when the signal's current or any later value cannot be encoded
    /// for `symbology` (QR capacity exceeded, Code128-unencodable characters).
    /// Pre-validate runtime user input with [`BarcodeSource::qr`] /
    /// [`BarcodeSource::code128`].
    #[must_use]
    pub fn reactive(
        symbology: BarcodeSymbology,
        content: impl IntoComputed<Str>,
        env: &Environment,
    ) -> Self {
        let source = crate::qr::reactive_source(symbology, &content.into_computed());
        let mut renderer = Self::new(source.snapshot(), env);
        renderer.source = source;
        renderer
    }

    /// Sets the fill style for dark modules.
    #[must_use]
    pub fn with_fill(mut self, fill: BarcodeFill) -> Self {
        self.fill = resolve_fill(fill, &self.environment);
        self
    }

    /// Sets the light module/background color.
    #[must_use]
    pub fn with_light_color(mut self, color: impl IntoComputed<Color>) -> Self {
        self.light_color = resolve_color(color, &self.environment);
        self
    }
}

impl SceneContent for BarcodeRenderer {
    fn intrinsic_size(&self) -> Option<Size> {
        Some(natural_size(&self.source.snapshot()))
    }

    fn build_scene(
        &mut self,
        recorder: &mut Recorder,
        _resources: &mut RecordingResources<'_>,
        width: f32,
        height: f32,
    ) -> bool {
        let Some(surface) = surface_rect(width, height) else {
            return false;
        };

        recorder.fill(Fixed(surface), self.light_color.clone());

        let area = content_rect(&self.source.snapshot(), surface.width(), surface.height());
        recorder.fill(module_path(&self.source, surface), self.fill.paint(area));
        false
    }

    /// Signals and resolved colors are all this content holds; nothing it
    /// owns is bound to the engine generation that recorded it.
    fn rebuild_for_engine(&mut self) {}

    fn set_invalidator(&mut self, invalidator: Option<SceneInvalidator>) {
        // Geometry and paint signal changes reach the recorded scene through
        // their live operands without a re-record; only a payload change can
        // move the intrinsic size, which is what layout has to be told about.
        self.layout_guard = invalidator.map(|invalidate| self.source.watch(move |_| invalidate()));
    }
}
