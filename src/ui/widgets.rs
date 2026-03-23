use egui::{Color32, RichText, Ui, Vec2};
use crate::config::Theme;

/// Small colored badge
pub fn badge(ui: &mut Ui, text: &str, color: Color32) {
    let label = RichText::new(text)
        .size(10.0)
        .color(Color32::WHITE)
        .strong();
    ui.label(label);
    let _ = color; // used for framing later if needed
}

/// Styled section heading
pub fn heading(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).strong().color(Theme::TEXT_ACCENT).size(13.0));
    ui.separator();
}

/// Monospace hex label
pub fn hex_label(ui: &mut Ui, value: u64) {
    ui.label(
        RichText::new(format!("0x{:016X}", value))
            .monospace()
            .color(Theme::TEXT_SECONDARY)
            .size(11.0),
    );
}

/// A compact icon-button that returns true if clicked.
pub fn icon_btn(ui: &mut Ui, icon: &str, tooltip: &str) -> bool {
    ui.add(egui::Button::new(icon).min_size(Vec2::new(24.0, 22.0)))
        .on_hover_text(tooltip)
        .clicked()
}

/// A thin coloured divider line
pub fn color_sep(ui: &mut Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), 1.0),
        egui::Sense::hover(),
    );
    ui.painter().rect_filled(rect, 0.0, color);
}
