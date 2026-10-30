//! Every bullshit app lives here. To add one, write a type that implements
//! `BullshitApp` and list it in `all()`; the launcher does the rest.

mod calculator;

use eframe::egui;

/// One app inside the launcher.
pub trait BullshitApp {
    /// Shown on the tile and in the title bar.
    fn name(&self) -> &'static str;
    /// One line under the name on the tile.
    fn tagline(&self) -> &'static str;
    /// A single glyph for the tile.
    fn icon(&self) -> &'static str;
    /// Draw the app. Called every frame while it is open.
    fn ui(&mut self, ui: &mut egui::Ui);
}

/// Load anything the apps need before the first frame, such as fonts.
pub fn setup(ctx: &egui::Context) {
    calculator::install_fonts(ctx);
}

/// Every app the launcher shows, in tile order.
pub fn all() -> Vec<Box<dyn BullshitApp>> {
    vec![Box::new(calculator::Calculator::new())]
}
