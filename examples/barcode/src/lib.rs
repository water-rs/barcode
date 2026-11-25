use waterui::{
    Environment,
    app::{App, Windows},
    prelude::*,
    preview,
};
use waterui_barcode::{Barcode, BarcodeSymbology};

#[preview]
pub fn demo() -> impl View {
    vstack((
        text("Scan me!")
            .title()
            .foreground(Color::srgb(245, 247, 250)),
        Barcode::new(BarcodeSymbology::Qr, "https://waterui.dev").size(300.0, 300.0),
    ))
    .spacing(20.0)
    .background(Color::srgb(31, 35, 38))
    .padding()
}

pub fn app(env: Environment) -> App<impl Windows> {
    App::new(demo, env)
}
