//! The original Bullshism calculator, now behind frosted glass.

use super::BullshitApp;
use crate::liar::{Rng, VERDICTS, format_number, wild_guess};
use crate::parser;
use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontFamily, FontId, Layout, Pos2, Rect, Sense,
    Stroke, StrokeKind, Vec2,
    epaint::{Mesh, RectShape, Shadow, Tessellator},
};
use std::sync::Arc;

/// One answered sum, kept for the history list.
struct Entry {
    input: String,
    /// Kept so the answer can be re-rolled when confess mode flips.
    truth: f64,
    answer: String,
    verdict: &'static str,
}

pub struct Calculator {
    input: String,
    rng: Rng,
    ans: f64,
    confess: bool,
    error: Option<String>,
    history: Vec<Entry>,
}

// The Velvet Glass palette.
const BACKDROP: Color32 = Color32::from_rgb(18, 15, 28);
const TEXT: Color32 = Color32::from_rgb(251, 246, 255);
const PINK: Color32 = Color32::from_rgb(244, 185, 255);
const ROSE: Color32 = Color32::from_rgb(255, 194, 209);
const EQUALS_FROM: Color32 = Color32::from_rgb(226, 92, 240);
const EQUALS_TO: Color32 = Color32::from_rgb(122, 92, 255);
const SWITCH_ON: Color32 = Color32::from_rgb(198, 79, 216);

/// Glows painted behind the glass: colour, centre and radius as fractions of the panel.
const GLOWS: [(Color32, [f32; 2], [f32; 2]); 3] = [
    (Color32::from_rgb(193, 63, 208), [0.18, 0.20], [0.60, 0.55]),
    (Color32::from_rgb(58, 78, 224), [0.85, 0.30], [0.55, 0.60]),
    (Color32::from_rgb(18, 163, 154), [0.60, 0.95], [0.70, 0.55]),
];

const CARD_WIDTH: f32 = 384.0;
const KEY_HEIGHT: f32 = 48.0;
const FN_HEIGHT: f32 = 32.0;
const GAP: f32 = 8.0;

fn white(alpha: u8) -> Color32 {
    Color32::from_white_alpha(alpha)
}

fn sora(weight: &str, size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(format!("sora-{weight}").into()))
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Function,
    Digit,
    Operator,
    Clear,
    Equals,
}

/// What a key shows, what it types (or does), and how it looks.
struct Key(&'static str, &'static str, Kind);

const FUNCTIONS: [Key; 6] = [
    Key("(", "(", Kind::Function),
    Key(")", ")", Kind::Function),
    Key("^", "^", Kind::Function),
    Key("√", "sqrt(", Kind::Function),
    Key("!", "!", Kind::Function),
    Key("π", "pi", Kind::Function),
];

/// Four columns; "0" is the one key that is two columns wide.
const KEYPAD: [Key; 19] = [
    Key("C", "C", Kind::Clear),
    Key("Del", "Del", Kind::Clear),
    Key("ans", "ans", Kind::Operator),
    Key("÷", "/", Kind::Operator),
    Key("7", "7", Kind::Digit),
    Key("8", "8", Kind::Digit),
    Key("9", "9", Kind::Digit),
    Key("×", "*", Kind::Operator),
    Key("4", "4", Kind::Digit),
    Key("5", "5", Kind::Digit),
    Key("6", "6", Kind::Digit),
    Key("−", "-", Kind::Operator),
    Key("1", "1", Kind::Digit),
    Key("2", "2", Kind::Digit),
    Key("3", "3", Kind::Digit),
    Key("+", "+", Kind::Operator),
    Key("0", "0", Kind::Digit),
    Key(".", ".", Kind::Digit),
    Key("=", "=", Kind::Equals),
];

/// Register the Sora weights the calculator draws with. Call once at startup.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    // Sora has no √ or π, so the default fonts stay behind it as a fallback.
    let fallback = fonts.families[&FontFamily::Proportional].clone();
    for (weight, bytes) in [
        (
            "200",
            &include_bytes!("../../assets/fonts/Sora-ExtraLight.ttf")[..],
        ),
        (
            "300",
            &include_bytes!("../../assets/fonts/Sora-Light.ttf")[..],
        ),
        (
            "400",
            &include_bytes!("../../assets/fonts/Sora-Regular.ttf")[..],
        ),
    ] {
        let name = format!("sora-{weight}");
        fonts
            .font_data
            .insert(name.clone(), Arc::new(egui::FontData::from_static(bytes)));
        let mut chain = vec![name.clone()];
        chain.extend(fallback.iter().cloned());
        fonts.families.insert(FontFamily::Name(name.into()), chain);
    }
    ctx.set_fonts(fonts);
}

/// Show sums the way people write them, not the way the parser reads them.
fn pretty(input: &str) -> String {
    input
        .replace("sqrt", "√")
        .replace("pi", "π")
        .replace('*', "×")
        .replace('/', "÷")
        .replace('-', "−")
}

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
                let answer = wild_guess(truth, self.confess, &mut self.rng);
                // ans remembers what we said, not what was true. Lies compound.
                self.ans = answer;
                self.error = None;
                self.history.push(Entry {
                    input,
                    truth,
                    answer: format_number(answer),
                    verdict: self.rng.pick(&VERDICTS),
                });
                self.input.clear();
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }

    /// Confess mode flipped: the latest answer changes its story to match.
    fn reroll_last(&mut self) {
        if let Some(last) = self.history.last_mut() {
            let answer = wild_guess(last.truth, self.confess, &mut self.rng);
            last.answer = format_number(answer);
            last.verdict = self.rng.pick(&VERDICTS);
            self.ans = answer;
        }
    }

    fn press(&mut self, typed: &str) {
        match typed {
            "=" => self.calculate(),
            "C" => {
                self.input.clear();
                self.error = None;
            }
            "Del" => {
                self.input.pop();
            }
            _ => self.input.push_str(typed),
        }
    }

    /// The expression, the big answer and its verdict, all right-aligned.
    fn display(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let hint = match self.history.last() {
            Some(last) => format!("{} =", pretty(&last.input)),
            None => "Type a sum, e.g. 2+2".into(),
        };
        let field = ui.add(
            egui::TextEdit::singleline(&mut self.input)
                .hint_text(
                    egui::RichText::new(hint)
                        .font(sora("300", 15.0))
                        .color(white(150)),
                )
                .font(sora("300", 15.0))
                .text_color(TEXT)
                .frame(egui::Frame::NONE)
                .background_color(Color32::TRANSPARENT)
                .align(Align2::RIGHT_CENTER)
                .desired_width(f32::INFINITY),
        );
        if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.calculate();
            field.request_focus();
        }

        let (answer, verdict) = match self.history.last() {
            Some(last) => (last.answer.as_str(), last.verdict),
            None => ("0", "Ask me anything. I'll be sure of it."),
        };
        let size = match answer.chars().count() {
            0..=7 => 60.0,
            8..=11 => 44.0,
            _ => 32.0,
        };
        ui.label(
            egui::RichText::new(answer)
                .font(sora("200", size))
                .color(TEXT),
        );
        ui.label(
            egui::RichText::new(verdict)
                .font(sora("300", 13.0))
                .color(PINK),
        );
        if let Some(err) = &self.error {
            ui.label(
                egui::RichText::new(format!("error: {err}"))
                    .font(sora("300", 12.0))
                    .color(ROSE),
            );
        }
        field
    }

    /// The function strip and the keypad, laid out on a fixed grid.
    fn keypad(&mut self, ui: &mut egui::Ui) {
        let width = ui.available_width();
        let height = FN_HEIGHT + GAP + KEY_HEIGHT * 5.0 + GAP * 4.0;
        let (area, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());

        let fn_width = (width - GAP * 5.0) / 6.0;
        for (i, key) in FUNCTIONS.iter().enumerate() {
            let min = area.min + Vec2::new(i as f32 * (fn_width + GAP), 0.0);
            if glass_key(
                ui,
                Rect::from_min_size(min, Vec2::new(fn_width, FN_HEIGHT)),
                key,
            ) {
                self.press(key.1);
            }
        }

        let col = (width - GAP * 3.0) / 4.0;
        let top = area.min.y + FN_HEIGHT + GAP;
        let mut slot = 0;
        for key in &KEYPAD {
            let span = if key.0 == "0" { 2 } else { 1 };
            let (row, column) = (slot / 4, slot % 4);
            let min = Pos2::new(
                area.min.x + column as f32 * (col + GAP),
                top + row as f32 * (KEY_HEIGHT + GAP),
            );
            let size = Vec2::new(col * span as f32 + GAP * (span - 1) as f32, KEY_HEIGHT);
            if glass_key(ui, Rect::from_min_size(min, size), key) {
                self.press(key.1);
            }
            slot += span;
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
        backdrop(ui);

        let card = egui::Frame::new()
            .fill(white(23))
            .stroke(Stroke::new(1.0, white(50)))
            .corner_radius(28.0)
            .inner_margin(18.0);

        let before = self.input.clone();
        let mut field = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                ui.set_max_width(CARD_WIDTH);
                card.show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(GAP, 4.0);
                    ui.with_layout(Layout::top_down(Align::Max), |ui| {
                        field = Some(self.display(ui));
                    });
                    ui.add_space(12.0);
                    self.keypad(ui);
                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        if switch(ui, &mut self.confess).changed() {
                            self.reroll_last();
                        }
                        ui.label(
                            egui::RichText::new("Confess")
                                .font(sora("300", 14.0))
                                .color(white(200)),
                        );
                    });

                    if self.history.len() > 1 {
                        ui.add_space(8.0);
                        let line = ui.available_rect_before_wrap();
                        ui.painter()
                            .hline(line.x_range(), line.top(), Stroke::new(1.0, white(30)));
                        ui.add_space(10.0);
                        ui.with_layout(Layout::top_down(Align::Min), |ui| {
                            for e in self.history.iter().rev().skip(1).take(5) {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} = {}",
                                        pretty(&e.input),
                                        e.answer
                                    ))
                                    .font(sora("300", 12.5))
                                    .color(white(140)),
                                );
                            }
                        });
                    }
                });
            });
            ui.add_space(6.0);
        });

        // Keep the keyboard on the sum, with the cursor after anything the keypad added.
        let Some(field) = field else { return };
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
    }
}

/// The dark wallpaper with soft colour glows that shows through the glass.
fn backdrop(ui: &egui::Ui) {
    let rect = ui.clip_rect();
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, BACKDROP);

    let mut mesh = Mesh::default();
    for (color, [cx, cy], [rx, ry]) in GLOWS {
        let centre = Pos2::new(
            rect.left() + rect.width() * cx,
            rect.top() + rect.height() * cy,
        );
        let radius = Vec2::new(rect.width() * rx, rect.height() * ry);
        // Three rings fading out, so the glow has no visible edge.
        let rings = [(0.0, 0.85), (0.45, 0.45), (1.0, 0.0)];
        let steps = 48;
        let base = mesh.vertices.len() as u32;
        for &(r, strength) in &rings {
            let count = if r == 0.0 { 1 } else { steps };
            for s in 0..count {
                let angle = s as f32 / steps as f32 * std::f32::consts::TAU;
                let pos = centre + Vec2::new(angle.cos() * radius.x, angle.sin() * radius.y) * r;
                mesh.colored_vertex(pos, color.gamma_multiply(strength));
            }
        }
        // Centre fan into the first ring, then a strip between the two rings.
        for s in 0..steps {
            let (a, b) = (base + 1 + s, base + 1 + (s + 1) % steps);
            mesh.add_triangle(base, a, b);
            mesh.add_triangle(a, b, a + steps);
            mesh.add_triangle(b, a + steps, b + steps);
        }
    }
    painter.add(mesh);
}

/// One frosted key. Returns true when it was clicked.
fn glass_key(ui: &mut egui::Ui, rect: Rect, key: &Key) -> bool {
    let Key(label, typed, kind) = *key;
    let response = ui
        .interact(rect, ui.scope_id().with(("key", typed)), Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if !ui.is_rect_visible(rect) {
        return response.clicked();
    }
    let painter = ui.painter();
    let radius = if kind == Kind::Function { 11.0 } else { 16.0 };
    let pressed = response.is_pointer_button_down_on();
    let rect = if pressed { rect.shrink(1.0) } else { rect };

    if kind == Kind::Equals {
        let glow = Shadow {
            offset: [0, 10],
            blur: 24,
            spread: 0,
            color: EQUALS_FROM.gamma_multiply(0.45),
        };
        painter.add(glow.as_shape(rect, radius));
        let lift = if response.hovered() { 1.12 } else { 1.0 };
        painter.add(gradient(ui, rect, radius, EQUALS_FROM, EQUALS_TO, lift));
    } else {
        let alpha = match (kind, response.hovered(), pressed) {
            (_, _, true) => 56,
            (_, true, _) => 40,
            (Kind::Function, ..) => 13,
            _ => 20,
        };
        painter.rect_filled(rect, radius, white(alpha));
    }

    let (color, font) = match kind {
        Kind::Function => (white(190), sora("300", 13.0)),
        Kind::Operator => (PINK, sora("400", 18.0)),
        Kind::Clear => (ROSE, sora("300", 18.0)),
        Kind::Equals => (TEXT, sora("400", 20.0)),
        Kind::Digit => (TEXT, sora("300", 18.0)),
    };
    painter.text(rect.center(), Align2::CENTER_CENTER, label, font, color);
    response.clicked()
}

/// A rounded rect filled with a left-to-right gradient.
fn gradient(ui: &egui::Ui, rect: Rect, radius: f32, from: Color32, to: Color32, lift: f32) -> Mesh {
    let mut tessellator = Tessellator::new(
        ui.ctx().pixels_per_point(),
        Default::default(),
        [1, 1],
        vec![],
    );
    let mut mesh = Mesh::default();
    let shape = RectShape::filled(rect, CornerRadius::same(radius as u8), Color32::WHITE);
    tessellator.tessellate_rect(&shape, &mut mesh);
    for v in &mut mesh.vertices {
        let t = ((v.pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        let mixed = Color32::from_rgb(
            lerp(from.r(), to.r(), t, lift),
            lerp(from.g(), to.g(), t, lift),
            lerp(from.b(), to.b(), t, lift),
        );
        // The feathered edge vertices carry the coverage in their alpha.
        v.color = mixed.gamma_multiply(v.color.a() as f32 / 255.0);
    }
    mesh
}

fn lerp(a: u8, b: u8, t: f32, lift: f32) -> u8 {
    ((a as f32 + (b as f32 - a as f32) * t) * lift).min(255.0) as u8
}

/// A small pill switch. Its response is marked changed when it flips.
fn switch(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(34.0, 20.0), Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let t = ui.ctx().animate_bool(response.id, *on);
    let track = if *on { SWITCH_ON } else { white(46) };
    let painter = ui.painter();
    painter.rect_filled(rect, 10.0, track);
    let knob = Pos2::new(
        egui::lerp(rect.left() + 10.0..=rect.right() - 10.0, t),
        rect.center().y,
    );
    painter.circle_filled(knob, 7.0, Color32::WHITE);
    if response.has_focus() {
        painter.rect_stroke(
            rect.expand(2.0),
            12.0,
            Stroke::new(1.5, PINK),
            StrokeKind::Outside,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
