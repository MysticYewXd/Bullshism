//! The original Bullshism calculator, now with buttons.

use super::BullshitApp;
use crate::launcher::ACCENT;
use crate::liar::{Rng, VERDICTS, format_number, lie};
use crate::parser;
use eframe::egui::{self, Color32, RichText, Vec2};

/// One answered sum, kept for the history list.
struct Entry {
    input: String,
    answer: String,
    verdict: &'static str,
    /// The real answer and how it was ruined, shown only in confess mode.
    confession: String,
}

pub struct Calculator {
    input: String,
    rng: Rng,
    ans: f64,
    confess: bool,
    error: Option<String>,
    history: Vec<Entry>,
}

const KEYS: [[&str; 5]; 4] = [
    ["7", "8", "9", "/", "("],
    ["4", "5", "6", "*", ")"],
    ["1", "2", "3", "-", "^"],
    ["0", ".", "ans", "+", "sqrt("],
];

impl Calculator {
    pub fn new() -> Self {
        Calculator {
            input: String::new(),
            rng: Rng::from_time(),
            ans: 0.0,
            confess: false,
            error: None,
            history: Vec::new(),
        }
    }

    fn calculate(&mut self) {
        let input = self.input.trim().to_string();
        if input.is_empty() {
            return;
        }
        match parser::evaluate(&input, self.ans) {
            Ok(truth) => {
                let (answer, strategy) = lie(truth, &mut self.rng);
                // ans remembers what we said, not what was true. Lies compound.
                self.ans = answer;
                self.error = None;
                self.history.push(Entry {
                    input,
                    answer: format_number(answer),
                    verdict: self.rng.pick(&VERDICTS),
                    confession: format!("really {}; {}", format_number(truth), strategy.name()),
                });
                self.input.clear();
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }
}

impl BullshitApp for Calculator {
    fn name(&self) -> &'static str {
        "Calculator"
    }

    fn tagline(&self) -> &'static str {
        "Confidently, reliably wrong"
    }

    fn icon(&self) -> &'static str {
        "="
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        // The latest answer, big and proud.
        egui::Frame::group(ui.style())
            .corner_radius(12.0)
            .inner_margin(14.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                // Room for every line up front, so the keypad never jumps about.
                ui.set_min_height(118.0);
                match self.history.last() {
                    Some(last) => {
                        ui.label(RichText::new(&last.input).weak());
                        ui.label(
                            RichText::new(format!("= {}", last.answer))
                                .size(34.0)
                                .strong()
                                .color(Color32::from_rgb(110, 220, 120)),
                        );
                        ui.label(RichText::new(last.verdict).italics().weak());
                        if self.confess && self.error.is_none() {
                            ui.label(RichText::new(format!("({})", last.confession)).color(ACCENT));
                        }
                    }
                    None => {
                        ui.label(RichText::new("Ask me anything. I'll be sure of it.").weak());
                        ui.label(RichText::new("= ?").size(34.0).strong());
                    }
                }
                if let Some(err) = &self.error {
                    ui.colored_label(Color32::from_rgb(235, 90, 90), format!("error: {err}"));
                }
            });

        let field = ui.add(
            egui::TextEdit::singleline(&mut self.input)
                .hint_text("Type a sum, e.g. 2+2, then press Enter")
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY),
        );
        if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.calculate();
            field.request_focus();
        }

        // The keypad.
        let before = self.input.clone();
        let gap = ui.spacing().item_spacing.x;
        let key = Vec2::new(((ui.available_width() - gap * 4.0) / 5.0).min(140.0), 40.0);
        for row in KEYS {
            ui.horizontal(|ui| {
                for k in row {
                    if ui
                        .add_sized(key, egui::Button::new(RichText::new(k).size(18.0)))
                        .clicked()
                    {
                        self.input.push_str(k);
                    }
                }
            });
        }
        ui.horizontal(|ui| {
            if ui
                .add_sized(key, egui::Button::new(RichText::new("C").size(18.0)))
                .clicked()
            {
                self.input.clear();
                self.error = None;
            }
            if ui
                .add_sized(key, egui::Button::new(RichText::new("Del").size(18.0)))
                .clicked()
            {
                self.input.pop();
            }
            if ui
                .add_sized(key, egui::Button::new(RichText::new("!").size(18.0)))
                .clicked()
            {
                self.input.push('!');
            }
            if ui
                .add_sized(key, egui::Button::new(RichText::new("pi").size(18.0)))
                .clicked()
            {
                self.input.push_str("pi");
            }
            let equals = egui::Button::new(RichText::new("=").size(18.0).strong())
                .fill(ACCENT.linear_multiply(0.7));
            if ui.add_sized(key, equals).clicked() {
                self.calculate();
            }
        });

        // Keep the keyboard on the sum, with the cursor after anything the keypad added.
        if self.input != before || ui.memory(|m| m.focused().is_none()) {
            field.request_focus();
            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), field.id) {
                let end = egui::text::CCursor::new(self.input.chars().count());
                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::one(end)));
                state.store(ui.ctx(), field.id);
            }
        }

        ui.add_space(4.0);
        ui.checkbox(&mut self.confess, "Confess (show the real answer)");

        if self.history.len() > 1 {
            ui.separator();
            ui.label(RichText::new("Previous lies").weak());
            egui::ScrollArea::vertical().show(ui, |ui| {
                for e in self.history.iter().rev().skip(1) {
                    let mut line = format!("{} = {}", e.input, e.answer);
                    if self.confess {
                        line.push_str(&format!("   ({})", e.confession));
                    }
                    ui.label(RichText::new(line).monospace());
                }
            });
        }
    }
}
