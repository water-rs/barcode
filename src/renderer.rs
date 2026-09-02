//! Vector scene content for encoded barcode matrices.

use core::fmt;

use kurbo::{Affine, BezPath, Point, Rect};
use nami::{SignalExt as _, signal::IntoComputed};
use peniko::{Brush, ColorStop, Fill, Gradient};
use waterui_core::reactive::watcher::BoxWatcherGuard;
use waterui_core::{Computed, Environment, Signal as _, Str, flatten_signal, layout::UnitPoint};
use waterui_graphics::{
    Scene2D, SceneContent, SceneInvalidator,
    color::{Color, ResolvedColor, Srgb},
};

use crate::geometry::{content_rect, dark_module_path};
use crate::qr::ReactiveBarcodeContent;
use crate::{BarcodeSource, BarcodeSymbology, view::BarcodeFill};

/// A [`SceneContent`] that draws a barcode as vector geometry.
///
/// The matrix is encoded on CPU once and emitted as one filled path of dark
/// modules, so the same content draws through any [`Scene2D`] — the GPU
/// compute renderer, the CPU sparse-strip renderer, or a recording. Colors stay
/// reactive for the content's lifetime: a change resolves through the
/// environment the content was built in and invalidates the surface precisely.
pub struct BarcodeRenderer {
    environment: Environment,
    source: BarcodeSource,
    reactive_content: Option<ReactiveBarcodeContent>,
    fill: ResolvedFill,
    light_color: Computed<ResolvedColor>,
    color_guards: Vec<BoxWatcherGuard>,
}

/// A [`BarcodeFill`] whose colors are already resolved against an environment.
enum ResolvedFill {
    Solid(Computed<ResolvedColor>),
    LinearGradient {
        start: Computed<ResolvedColor>,
        end: Computed<ResolvedColor>,
        start_point: UnitPoint,
        end_point: UnitPoint,
    },
}

impl ResolvedFill {
    /// The brush painting dark modules laid out inside `area`.
    ///
    /// Gradient endpoints are unit coordinates of the barcode square, so they
    /// are mapped onto `area` rather than onto the whole surface.
    fn brush(&self, area: Rect) -> Brush {
        match self {
            Self::Solid(color) => Brush::Solid(to_peniko(&color.get())),
            Self::LinearGradient {
                start,
                end,
                start_point,
                end_point,
            } => {
                let anchor = |point: UnitPoint| {
                    Point::new(
                        f64::from(point.x).mul_add(area.width(), area.x0),
                        f64::from(point.y).mul_add(area.height(), area.y0),
                    )
                };
                let stops = [
                    ColorStop {
                        offset: 0.0,
                        color: to_peniko(&start.get()).into(),
                    },
                    ColorStop {
                        offset: 1.0,
                        color: to_peniko(&end.get()).into(),
                    },
                ];
                Brush::Gradient(
                    Gradient::new_linear(anchor(*start_point), anchor(*end_point))
                        .with_stops(stops.as_slice()),
                )
            }
        }
    }

    /// Every signal this fill reads, in the order it reads them.
    fn colors(&self) -> impl Iterator<Item = &Computed<ResolvedColor>> {
        let (first, second) = match self {
            Self::Solid(color) => (color, None),
            Self::LinearGradient { start, end, .. } => (start, Some(end)),
        };
        core::iter::once(first).chain(second)
    }
}

/// Resolves `color` against `env`, keeping the result reactive.
pub fn resolve_color(
    color: impl IntoComputed<Color>,
    env: &Environment,
) -> Computed<ResolvedColor> {
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

/// Converts a resolved linear-RGB color into the sRGB-encoded color peniko takes.
pub fn to_peniko(color: &ResolvedColor) -> peniko::Color {
    let srgb = color.to_srgb_with_headroom();
    peniko::Color::new([
        srgb.red,
        srgb.green,
        srgb.blue,
        color.opacity.clamp(0.0, 1.0),
    ])
}

/// Fills `rect` with `brush` through `scene`.
pub fn fill_rect(scene: &mut dyn Scene2D, rect: Rect, brush: &Brush) {
    let mut path = BezPath::new();
    path.move_to((rect.x0, rect.y0));
    path.line_to((rect.x1, rect.y0));
    path.line_to((rect.x1, rect.y1));
    path.line_to((rect.x0, rect.y1));
    path.close_path();
    scene.fill(Fill::NonZero, Affine::IDENTITY, brush, None, &path);
}

/// The surface rectangle, or `None` when the surface has no drawable area.
pub fn surface_rect(width: f32, height: f32) -> Option<Rect> {
    let (width, height) = (f64::from(width), f64::from(height));
    (width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0)
        .then(|| Rect::new(0.0, 0.0, width, height))
}

impl fmt::Debug for BarcodeRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BarcodeRenderer")
            .field("source", &self.source)
            .finish_non_exhaustive()
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
            source,
            reactive_content: None,
            fill: resolve_fill(BarcodeFill::default(), env),
            light_color: resolve_color(Color::from(Srgb::WHITE), env),
            color_guards: Vec::new(),
        }
    }

    /// Creates content whose barcode follows a signal.
    ///
    /// Every content change re-encodes the matrix before the next frame,
    /// without recreating the content.
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
        let reactive_content = ReactiveBarcodeContent::new(symbology, content.into_computed());
        let source = reactive_content.initial_source();
        let mut renderer = Self::new(source, env);
        renderer.reactive_content = Some(reactive_content);
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
    fn build_scene(&mut self, scene: &mut dyn Scene2D, width: f32, height: f32) -> bool {
        if let Some(source) = self
            .reactive_content
            .as_mut()
            .and_then(ReactiveBarcodeContent::take_reencoded)
        {
            self.source = source;
        }
        let Some(surface) = surface_rect(width, height) else {
            return false;
        };

        fill_rect(
            scene,
            surface,
            &Brush::Solid(to_peniko(&self.light_color.get())),
        );

        let area = content_rect(&self.source, surface.width(), surface.height());
        let modules = dark_module_path(&self.source, area);
        if !modules.is_empty() {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &self.fill.brush(area),
                None,
                &modules,
            );
        }
        false
    }

    fn set_invalidator(&mut self, invalidator: Option<SceneInvalidator>) {
        self.color_guards.clear();
        let Some(invalidator) = invalidator else {
            return;
        };

        let mut guards = Vec::new();
        for color in core::iter::once(&self.light_color).chain(self.fill.colors()) {
            let invalidator = SceneInvalidator::clone(&invalidator);
            guards.push(color.watch(move |_| invalidator()));
        }
        self.color_guards = guards;

        if let Some(reactive_content) = &mut self.reactive_content {
            reactive_content.install(move || invalidator());
        }
    }
}
