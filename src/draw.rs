//! The canvas a barcode is drawn on.
//!
//! The matrix is encoded on CPU once and drawn as one filled path of dark
//! modules. The path, the colors and the gradient endpoints are live operands
//! of the canvas: a payload, color or box-size change updates the retained
//! command in place, and only the intrinsic size — which layout must hear
//! about — follows the payload through the canvas' own size signal.

use nami::{Computed, SignalExt as _, signal::IntoComputed};
use waterui_core::layout::{Affine2, Point, Rect, Size, UnitPoint};
use waterui_core::{Environment, flatten_signal};
use waterui_graphics::{
    canvas::{Canvas, Cx, Drawing, Interpolation, LinearGradient, Paint, Path},
    color::{Color, WorkingColor},
};

use crate::BarcodeSource;
use crate::geometry::{content_rect, dark_module_path, natural_size};
use crate::view::BarcodeFill;

/// What fills a barcode's dark modules when it is not a plain color or
/// gradient: a [`Drawing`], or a closure that draws on the barcode's canvas.
///
/// The ink draws across the whole box and survives only where a dark module
/// is, so a gradient, a photo or an animation becomes the barcode's ink
/// without either side knowing about the other.
pub trait Ink: 'static {
    /// Draws the ink into `cx`, the barcode's canvas, clipped to its modules.
    fn draw(self, cx: &mut Cx);
}

impl<F: FnOnce(&mut Cx) + 'static> Ink for F {
    fn draw(self, cx: &mut Cx) {
        self(cx);
    }
}

impl Ink for Drawing {
    /// The drawing scaled to cover the barcode's box.
    fn draw(self, cx: &mut Cx) {
        let natural = self.size();
        let scale = cx.size().map(move |size: Size| {
            Affine2::scale(size.width / natural.width, size.height / natural.height)
        });
        cx.draw(&self, scale);
    }
}

/// A [`BarcodeFill`] whose colors are already resolved against an environment.
pub enum ResolvedFill {
    Solid(Computed<WorkingColor>),
    LinearGradient {
        start: Computed<WorkingColor>,
        end: Computed<WorkingColor>,
        start_point: UnitPoint,
        end_point: UnitPoint,
    },
}

impl ResolvedFill {
    /// Resolves every color of `fill` against `env`.
    pub fn new(fill: BarcodeFill, env: &Environment) -> Self {
        match fill {
            BarcodeFill::Solid(color) => Self::Solid(resolve_color(color, env)),
            BarcodeFill::LinearGradient {
                start_color,
                end_color,
                start_point,
                end_point,
            } => Self::LinearGradient {
                start: resolve_color(start_color, env),
                end: resolve_color(end_color, env),
                start_point,
                end_point,
            },
        }
    }

    /// The paint covering dark modules of `source` laid out in a box of `size`.
    ///
    /// Gradient endpoints are unit coordinates of the barcode square, so they
    /// are mapped onto the area the symbol occupies rather than onto the whole
    /// box.
    fn paint(&self, source: &Computed<BarcodeSource>, size: &Computed<Size>) -> Computed<Paint> {
        match self {
            Self::Solid(color) => color.clone().map(Paint::from).computed(),
            Self::LinearGradient {
                start,
                end,
                start_point,
                end_point,
            } => {
                let (start_point, end_point) = (*start_point, *end_point);
                let area = source
                    .zip(size)
                    .map(|(source, size)| content_rect(&source, size))
                    .computed();
                start
                    .zip(end)
                    .zip(&area)
                    .map(move |((start, end), area)| -> Paint {
                        // Gradients interpolate in the encoded space, as CSS
                        // gradients do.
                        LinearGradient::new(anchor(start_point, area), anchor(end_point, area))
                            .stop(0.0, start)
                            .stop(1.0, end)
                            .interpolation(Interpolation::Encoded)
                            .into()
                    })
                    .computed()
            }
        }
    }
}

const fn anchor(point: UnitPoint, area: Rect) -> Point {
    Point::new(
        point.x.mul_add(area.width(), area.x()),
        point.y.mul_add(area.height(), area.y()),
    )
}

/// Resolves `color` against `env`, keeping the result reactive.
pub fn resolve_color(color: impl IntoComputed<Color>, env: &Environment) -> Computed<WorkingColor> {
    let env = env.clone();
    flatten_signal(color.into_computed().map(move |color| color.resolve(&env)))
}

/// A canvas drawing `source` over `light`, whose dark modules are painted by
/// `modules`.
///
/// `modules` receives the canvas, its box as a signal and the dark-module path
/// laid out in that box. The canvas measures as the symbol's output size.
pub fn barcode_canvas(
    source: Computed<BarcodeSource>,
    light: Computed<WorkingColor>,
    modules: impl FnOnce(&mut Cx, &Computed<Size>, Computed<Path>) + 'static,
) -> Canvas {
    let natural = source.map(|source| Some(natural_size(&source))).computed();
    Canvas::new(move |cx| {
        let size = cx.size();
        cx.fill(size.map(Rect::from_size), light);
        let path = source
            .zip(&size)
            .map(|(source, size)| dark_module_path(&source, content_rect(&source, size)))
            .computed();
        modules(cx, &size, path);
    })
    .intrinsic_size(natural)
}

/// A canvas painting the dark modules with `fill`.
pub fn filled_canvas(
    source: Computed<BarcodeSource>,
    light: Computed<WorkingColor>,
    fill: ResolvedFill,
) -> Canvas {
    let paint_source = source.clone();
    barcode_canvas(source, light, move |cx, size, path| {
        cx.fill(path, fill.paint(&paint_source, size));
    })
}

/// A canvas drawing `ink` clipped to the dark modules.
pub fn inked_canvas(
    source: Computed<BarcodeSource>,
    light: Computed<WorkingColor>,
    ink: impl Ink,
) -> Canvas {
    barcode_canvas(source, light, move |cx, _, path| {
        cx.clip(path, |cx| ink.draw(cx));
    })
}
