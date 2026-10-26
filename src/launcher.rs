//! The Bullshism launcher: a window of tiles, one per bullshit app.

use crate::apps::{self, BullshitApp};
use eframe::egui::{self, Color32, RichText, Vec2};

/// The magenta the terminal uses for titles, so both sides feel like one product.
pub const ACCENT: Color32 = Color32::from_rgb(214, 92, 214);

const TILE: Vec2 = Vec2::new(190.0, 150.0);

struct Launcher {
    apps: Vec<Box<dyn BullshitApp>>,
    open: Option<usize>,
}

/// Open the launcher window and block until it is closed.
pub fn run() -> Result<(), String> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Bullshism")
            .with_app_id("bullshism")
            .with_inner_size([720.0, 660.0])
            .with_min_inner_size([420.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Bullshism",
        options,
        Box::new(|cc| {
            style(&cc.egui_ctx);
            Ok(Box::new(Launcher {
                apps: apps::all(),
                open: None,
            }))
        }),
    )
    .map_err(|e| e.to_string())
}

fn style(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::dark());
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        style.spacing.button_padding = Vec2::new(10.0, 6.0);
        style.visuals.selection.bg_fill = ACCENT.linear_multiply(0.6);
        style.visuals.hyperlink_color = ACCENT;
    });
}

impl eframe::App for Launcher {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("bar").show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if let Some(i) = self.open {
                    if ui.button("< All apps").clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        self.open = None;
                    }
                    ui.label(RichText::new(self.apps[i].name()).heading().strong());
                } else {
                    ui.label(RichText::new("Bullshism").heading().strong().color(ACCENT));
                    ui.label(RichText::new("apps you absolutely should not trust").weak());
                }
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ui, |ui| match self.open {
            Some(i) => self.apps[i].ui(ui),
            None => self.home(ui),
        });
    }
}

impl Launcher {
    fn home(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            for (i, app) in self.apps.iter().enumerate() {
                if tile(ui, app.icon(), app.name(), app.tagline(), true).clicked() {
                    self.open = Some(i);
                }
            }
            tile(ui, "?", "More nonsense", "Coming soon", false);
        });
    }
}

/// A rounded tile with a big glyph badge, a name and a tagline.
fn tile(ui: &mut egui::Ui, icon: &str, name: &str, tagline: &str, enabled: bool) -> egui::Response {
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(TILE, sense);
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let visuals = ui.visuals();
    let hovered = enabled && response.hovered();
    let fill = if hovered {
        visuals.widgets.hovered.weak_bg_fill
    } else {
        visuals.widgets.inactive.weak_bg_fill
    };
    let accent = if enabled {
        ACCENT
    } else {
        visuals.weak_text_color()
    };
    let text = if enabled {
        visuals.strong_text_color()
    } else {
        visuals.weak_text_color()
    };
    let painter = ui.painter();

    painter.rect_filled(rect, 16.0, fill);
    if hovered {
        painter.rect_stroke(rect, 16.0, (1.5, ACCENT), egui::StrokeKind::Inside);
    }
    let badge = rect.center_top() + Vec2::new(0.0, 46.0);
    painter.circle_filled(badge, 26.0, accent.linear_multiply(0.25));
    painter.text(
        badge,
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(28.0),
        accent,
    );
    painter.text(
        rect.center_top() + Vec2::new(0.0, 96.0),
        egui::Align2::CENTER_CENTER,
        name,
        egui::FontId::proportional(17.0),
        text,
    );
    painter.text(
        rect.center_top() + Vec2::new(0.0, 120.0),
        egui::Align2::CENTER_CENTER,
        tagline,
        egui::FontId::proportional(13.0),
        visuals.weak_text_color(),
    );
    if enabled {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        response
    }
}
