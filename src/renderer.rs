//! Retained vector content for encoded barcode matrices.

use core::fmt;

use kurbo::{Point, Rect};
use nami::{SignalExt as _, signal::IntoComputed};
use waterui_core::layout::{Size, UnitPoint};
use waterui_core::reactive::watcher::BoxWatcherGuard;
use waterui_core::{Computed, Environment, Signal as _, Str, flatten_signal};
use waterui_graphics::{
    Scene, SceneContent, SceneInvalidator,
    cherenkov::{Draw as _, LinearGradient, Paint},
    color::{Color, Srgb, WorkingColor},
};

use crate::geometry::{content_rect, dark_module_path, natural_size};
use crate::{BarcodeSource, BarcodeSymbology, view::BarcodeFill};

/// Barcode geometry and paints bound directly to a retained engine recording.
pub struct BarcodeRenderer {
    environment: Environment,
    source: Computed<BarcodeSource>,
    fill: ResolvedFill,
    light_color: Computed<WorkingColor>,
    layout_guard: Option<BoxWatcherGuard>,
}

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
    fn paint(&self, area: Rect) -> Computed<Paint> {
        match self {
            Self::Solid(color) => color.clone().map(Paint::Solid).into_computed(),
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
                let (from, to) = (anchor(*start_point), anchor(*end_point));
                start
                    .clone()
                    .zip(end)
                    .map(move |(start, end)| -> Paint {
                        LinearGradient::new(from, to)
                            .stop(0.0, start)
                            .stop(1.0, end)
                            .into()
                    })
                    .into_computed()
            }
        }
    }
}

/// Resolves a color without sampling away its reactive binding.
pub fn resolve_color(color: impl IntoComputed<Color>, env: &Environment) -> Computed<WorkingColor> {
    let env = env.clone();
    flatten_signal(color.into_computed().map(move |color| color.resolve(&env)))
}

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

/// The surface rectangle, or `None` for an empty layout offer.
pub fn surface_rect(width: f32, height: f32) -> Option<Rect> {
    let (width, height) = (f64::from(width), f64::from(height));
    (width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0)
        .then(|| Rect::new(0.0, 0.0, width, height))
}

impl fmt::Debug for BarcodeRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BarcodeRenderer").finish_non_exhaustive()
    }
}

impl BarcodeRenderer {
    /// Creates vector barcode content with black modules and a white background.
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

    /// Binds encoded module geometry directly to `content`.
    ///
    /// # Panics
    /// When the current or a later payload cannot be encoded for `symbology`.
    #[must_use]
    pub fn reactive(
        symbology: BarcodeSymbology,
        content: impl IntoComputed<Str>,
        env: &Environment,
    ) -> Self {
        let source = crate::qr::reactive_source(symbology, content.into_computed());
        let mut renderer = Self::new(source.snapshot(), env);
        renderer.source = source;
        renderer
    }

    /// Sets the module paint.
    #[must_use]
    pub fn with_fill(mut self, fill: BarcodeFill) -> Self {
        self.fill = resolve_fill(fill, &self.environment);
        self
    }

    /// Sets the background color.
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

    fn record(&mut self, scene: &mut Scene<'_>) -> bool {
        let Some(surface) = surface_rect(scene.width(), scene.height()) else {
            return false;
        };
        let area = content_rect(&self.source.snapshot(), surface.width(), surface.height());
        scene.recorder().fill(surface, self.light_color.clone());
        scene.recorder().fill(
            self.source.clone().map(move |source| {
                dark_module_path(
                    &source,
                    content_rect(&source, surface.width(), surface.height()),
                )
            }),
            self.fill.paint(area),
        );
        false
    }

    fn set_invalidator(&mut self, invalidator: Option<SceneInvalidator>) {
        // Payload changes may change intrinsic size. Geometry and paint changes
        // remain bound to the recorder; this notification requests layout.
        self.layout_guard = invalidator.map(|invalidate| self.source.watch(move |_| invalidate()));
    }
}
