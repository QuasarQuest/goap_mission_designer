use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use crate::config::Theme;
use crate::data::MissionDefinition;

const TOTAL_BITS:     usize   = 64;
const BAR_HEIGHT:     f32     = 28.0;
const LABEL_MARGIN:   f32     = 20.0;
const MIN_LABEL_WIDTH: f32    = 16.0;
const FONT_SIZE:      f32     = 9.0;
const BIT_MARKER_EVERY: usize = 8;

pub fn show_bit_layout(ui: &mut Ui, mission: &MissionDefinition) {
    let available = ui.available_width();
    let bit_w     = available / TOTAL_BITS as f32;

    let (rect, _) = ui.allocate_exact_size(Vec2::new(available, BAR_HEIGHT + LABEL_MARGIN), Sense::hover());
    let painter   = ui.painter_at(rect);

    let mut bit_color: [Color32; TOTAL_BITS] = [Theme::UNUSED_BIT_COLOR; TOTAL_BITS];

    for (field_idx, field) in mission.sorted_fields().iter().enumerate() {
        let color = Theme::bit_color(field_idx);
        for bit in field.bit_offset..=field.end_bit() {
            if (bit as usize) < TOTAL_BITS {
                bit_color[bit as usize] = color;
            }
        }
    }

    for bit in 0..TOTAL_BITS {
        let x    = rect.left() + bit as f32 * bit_w;
        let cell = Rect::from_min_size(Pos2::new(x + 1.0, rect.top()), Vec2::new(bit_w - 2.0, BAR_HEIGHT));
        painter.rect_filled(cell, 2.0, bit_color[bit]);

        if bit % BIT_MARKER_EVERY == 0 {
            painter.text(
                Pos2::new(x + bit_w * 0.5, rect.top() + BAR_HEIGHT + LABEL_MARGIN * 0.5),
                egui::Align2::CENTER_CENTER,
                bit.to_string(),
                egui::FontId::proportional(FONT_SIZE),
                Theme::TEXT_MUTED,
            );
        }
    }

    for field in mission.sorted_fields() {
        let x_start = rect.left() + field.bit_offset as f32 * bit_w;
        let width   = field.bit_width as f32 * bit_w;

        if width > MIN_LABEL_WIDTH {
            let label = if field.bit_width == 1 {
                field.name.chars().next().map(|c| c.to_string()).unwrap_or_default()
            } else {
                field.name.clone()
            };

            painter.text(
                Pos2::new(x_start + width * 0.5, rect.top() + BAR_HEIGHT * 0.5),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(FONT_SIZE),
                Color32::WHITE,
            );
        }
    }

    painter.rect_stroke(
        Rect::from_min_size(rect.min, Vec2::new(available, BAR_HEIGHT)),
        2.0,
        Stroke::new(1.0, Theme::TEXT_MUTED), egui::StrokeKind::Outside,
    );
}