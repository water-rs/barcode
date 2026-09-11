//! Scene content that masks arbitrary scene content into barcode modules.

use core::fmt;

use kurbo::Affine;
use nami::signal::IntoComputed;
use peniko::{Brush, Fill};
use waterui_core::layout::Size;
use waterui_core::reactive::watcher::BoxWatcherGuard;
use waterui_core::{Computed, Environment, Signal as _, Str};
use waterui_graphics::{
    Scene2D, SceneContent, SceneInvalidator,
    color::{Color, ResolvedColor},
};

use crate::geometry::{content_rect, dark_module_path, natural_size};
use crate::qr::ReactiveBarcodeContent;
use crate::renderer::{fill_rect, resolve_color, surface_rect, to_peniko};
use crate::{BarcodeSource, BarcodeSymbology};

/// Draws `C` clipped to a barcode's dark modules, over the light module color.
///
/// This is the vector counterpart of painting a barcode with an image: the
/// inner content draws across the whole surface and only survives where a dark
/// module is, so a gradient, a photo, or an animation becomes the barcode's
/// ink without either side knowing about the other.
pub struct BarcodeMask<C: SceneContent> {
    source: BarcodeSource,
    reactive_content: Option<ReactiveBarcodeContent>,
    light_color: Computed<ResolvedColor>,
    ink: C,
    light_color_guard: Option<BoxWatcherGuard>,
}

impl<C: SceneContent> fmt::Debug for BarcodeMask<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BarcodeMask")
            .field("source", &self.source)
            .finish_non_exhaustive()
    }
}

impl<C: SceneContent> BarcodeMask<C> {
    /// Masks `ink` into `source`, resolving the light color against `env`.
    #[must_use]
    pub fn new(
        source: BarcodeSource,
        light_color: impl IntoComputed<Color>,
        ink: C,
        env: &Environment,
    ) -> Self {
        Self {
            source,
            reactive_content: None,
            light_color: resolve_color(light_color, env),
            ink,
            light_color_guard: None,
        }
    }

    /// Masks `ink` into a barcode whose content follows a signal.
    ///
    /// # Panics
    ///
    /// Panics when the signal's current or any later value cannot be encoded
    /// for `symbology`; pre-validate runtime user input with
    /// [`BarcodeSource::qr`] / [`BarcodeSource::code128`].
    #[must_use]
    pub fn reactive(
        symbology: BarcodeSymbology,
        content: impl IntoComputed<Str>,
        light_color: impl IntoComputed<Color>,
        ink: C,
        env: &Environment,
    ) -> Self {
        let reactive_content = ReactiveBarcodeContent::new(symbology, content.into_computed());
        let source = reactive_content.initial_source();
        let mut mask = Self::new(source, light_color, ink, env);
        mask.reactive_content = Some(reactive_content);
        mask
    }
}

impl<C: SceneContent> SceneContent for BarcodeMask<C> {
    /// The barcode's own size, not the ink's: the ink draws across whatever
    /// box the symbol is given.
    fn intrinsic_size(&self) -> Option<Size> {
        Some(natural_size(&self.source, self.reactive_content.as_ref()))
    }

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
        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &modules);
        let wants_another_frame = self.ink.build_scene(scene, width, height);
        scene.pop_layer();
        wants_another_frame
    }

    fn set_invalidator(&mut self, invalidator: Option<SceneInvalidator>) {
        self.light_color_guard = None;
        self.ink.set_invalidator(invalidator.clone());
        let Some(invalidator) = invalidator else {
            return;
        };

        self.light_color_guard = Some(self.light_color.watch({
            let invalidator = SceneInvalidator::clone(&invalidator);
            move |_| invalidator()
        }));

        if let Some(reactive_content) = &mut self.reactive_content {
            reactive_content.install(move || invalidator());
        }
    }
}
