//! Scene content that masks arbitrary scene content into barcode modules.

use core::fmt;

use nami::signal::IntoComputed;
use waterui_core::layout::Size;
use waterui_core::reactive::watcher::BoxWatcherGuard;
use waterui_core::{Computed, Environment, Signal, Str};
use waterui_graphics::{
    RecordingResources, SceneContent, SceneInvalidator,
    cherenkov::{Draw as _, Fixed, Recorder},
    color::{Color, WorkingColor},
};

use crate::geometry::natural_size;
use crate::renderer::{module_path, resolve_color, surface_rect};
use crate::{BarcodeSource, BarcodeSymbology};

/// Draws `C` clipped to a barcode's dark modules, over the light module color.
///
/// This is the vector counterpart of painting a barcode with an image: the
/// inner content draws across the whole surface and only survives where a dark
/// module is, so a gradient, a photo, or an animation becomes the barcode's
/// ink without either side knowing about the other.
///
/// The clip shape is bound to the payload as a live operand of the recorded
/// scene: a signal change re-encodes the matrix and updates the clip in place
/// without re-recording, and only the intrinsic size — which layout must hear
/// about — goes through the invalidator.
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
    /// Masks `ink` into `source`, resolving the light color against `env`.
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
        let source = crate::qr::reactive_source(symbology, &content.into_computed());
        let mut mask = Self::new(source.snapshot(), light_color, ink, env);
        mask.source = source;
        mask
    }
}

impl<C: SceneContent> SceneContent for BarcodeMask<C> {
    /// The barcode's own size, not the ink's: the ink draws across whatever
    /// box the symbol is given.
    fn intrinsic_size(&self) -> Option<Size> {
        Some(natural_size(&self.source.snapshot()))
    }

    fn build_scene(
        &mut self,
        recorder: &mut Recorder,
        resources: &mut RecordingResources<'_>,
        width: f32,
        height: f32,
    ) -> bool {
        let Some(surface) = surface_rect(width, height) else {
            return false;
        };

        recorder.fill(Fixed(surface), self.light_color.clone());

        let mut wants_another_frame = false;
        recorder.clip(module_path(&self.source, surface), |recorder| {
            wants_another_frame = self.ink.build_scene(recorder, resources, width, height);
        });
        wants_another_frame
    }

    fn set_invalidator(&mut self, invalidator: Option<SceneInvalidator>) {
        self.ink.set_invalidator(invalidator.clone());
        // Clip shape and light color reach the recorded scene through live
        // operands; only a payload change can move the intrinsic size, which
        // is what layout has to be told about.
        self.layout_guard = invalidator.map(|invalidate| self.source.watch(move |_| invalidate()));
    }
}
