//! Scene content clipped by signal-bound barcode module geometry.

use core::fmt;

use nami::{SignalExt as _, signal::IntoComputed};
use waterui_core::layout::Size;
use waterui_core::reactive::watcher::BoxWatcherGuard;
use waterui_core::{Computed, Environment, Signal as _, Str};
use waterui_graphics::{
    Scene, SceneContent, SceneInvalidator,
    cherenkov::Draw as _,
    color::{Color, WorkingColor},
};

use crate::geometry::{content_rect, dark_module_path, natural_size};
use crate::renderer::{resolve_color, surface_rect};
use crate::{BarcodeSource, BarcodeSymbology};

/// Clips `C` to the dark modules over a reactive background color.
pub struct BarcodeMask<C: SceneContent> {
    source: Computed<BarcodeSource>,
    light_color: Computed<WorkingColor>,
    ink: C,
    layout_guard: Option<BoxWatcherGuard>,
}

impl<C: SceneContent> fmt::Debug for BarcodeMask<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BarcodeMask").finish_non_exhaustive()
    }
}

impl<C: SceneContent> BarcodeMask<C> {
    /// Masks `ink` into `source`, resolving the background against `env`.
    #[must_use]
    pub fn new(
        source: BarcodeSource,
        light_color: impl IntoComputed<Color>,
        ink: C,
        env: &Environment,
    ) -> Self {
        Self {
            source: source.into_computed(),
            light_color: resolve_color(light_color, env),
            ink,
            layout_guard: None,
        }
    }

    /// Binds the clipping geometry to encoded payload changes.
    ///
    /// # Panics
    /// When a payload cannot be encoded for `symbology`.
    #[must_use]
    pub fn reactive(
        symbology: BarcodeSymbology,
        content: impl IntoComputed<Str>,
        light_color: impl IntoComputed<Color>,
        ink: C,
        env: &Environment,
    ) -> Self {
        Self {
            source: crate::qr::reactive_source(symbology, content.into_computed()),
            light_color: resolve_color(light_color, env),
            ink,
            layout_guard: None,
        }
    }
}

impl<C: SceneContent> SceneContent for BarcodeMask<C> {
    fn intrinsic_size(&self) -> Option<Size> {
        Some(natural_size(&self.source.snapshot()))
    }

    fn record(&mut self, scene: &mut Scene<'_>) {
        let (width, height) = (scene.width(), scene.height());
        let Some(surface) = surface_rect(width, height) else {
            return;
        };
        let modules = self.source.clone().map(move |source| {
            dark_module_path(
                &source,
                content_rect(&source, surface.width(), surface.height()),
            )
        });
        let resources = scene.resources().clone();
        scene.recorder().fill(surface, self.light_color.clone());
        scene.recorder().clip(modules, |recorder| {
            self.ink
                .record(&mut Scene::new(recorder, &resources, width, height));
        });
    }

    fn set_invalidator(&mut self, invalidator: Option<SceneInvalidator>) {
        self.ink.set_invalidator(invalidator.clone());
        self.layout_guard = invalidator.map(|invalidate| self.source.watch(move |_| invalidate()));
    }
}
