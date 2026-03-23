use egui::{Color32, RichText, ScrollArea, Stroke, Ui};
use egui_extras::{Column, TableBuilder};
use crate::config::Theme;
use crate::data::BitField;
use crate::logic::EditorState;

// ── Constants ─────────────────────────────────────────────────────────────────

const COL_FIELD_MIN: f32 = 120.0;
const COL_VALUE_MIN: f32 = 140.0;
const COMBO_WIDTH:   f32 = 130.0;
const HEADER_HEIGHT: f32 = 22.0;
const ROW_HEIGHT:    f32 = 28.0;

// ── Public entry point ────────────────────────────────────────────────────────

pub fn show_states_panel(ui: &mut Ui, editor: &mut EditorState) {
    ui.vertical(|ui| {
        ui.heading("⚙ States");
        ui.separator();
        show_hex_summary(ui, editor);
        ui.add_space(8.0);
        ui.separator();
        show_field_table_or_empty(ui, editor);
    });
}

// ── Hex summary ───────────────────────────────────────────────────────────────

fn show_hex_summary(ui: &mut Ui, editor: &EditorState) {
    let mission = editor.mission();
    ui.horizontal(|ui| {
        show_state_hex(ui, "Initial State", mission.initial_state, Theme::NODE_INITIAL);
        ui.add_space(16.0);
        show_state_hex(ui, "Goal State",    mission.goal_state,    Theme::NODE_GOAL);
    });
}

fn show_state_hex(ui: &mut Ui, label: &str, value: u64, color: Color32) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).color(color).strong().size(12.0));
        ui.label(
            RichText::new(format!("0x{value:016X}"))
                .monospace()
                .color(color)
                .size(11.0),
        );
    });
}

// ── Field table or empty state ────────────────────────────────────────────────

fn show_field_table_or_empty(ui: &mut Ui, editor: &mut EditorState) {
    if editor.mission().bit_fields.is_empty() {
        show_empty_hint(ui);
        return;
    }
    ScrollArea::vertical()
        .id_salt("states_panel_field_editors")
        .show(ui, |ui| show_field_table(ui, editor));
}

fn show_empty_hint(ui: &mut Ui) {
    ui.label(
        RichText::new("No bit fields defined. Add fields in the left panel.")
            .color(Theme::TEXT_MUTED)
            .italics(),
    );
}

// ── Field table ───────────────────────────────────────────────────────────────

fn show_field_table(ui: &mut Ui, editor: &mut EditorState) {
    let sep    = Stroke::new(1.0, Theme::TEXT_MUTED);
    let fields = editor.mission().bit_fields.clone();

    TableBuilder::new(ui)
        .striped(false)
        .column(Column::auto().at_least(COL_FIELD_MIN))
        .column(Column::remainder().at_least(COL_VALUE_MIN))
        .column(Column::remainder().at_least(COL_VALUE_MIN))
        .header(HEADER_HEIGHT, |mut header| {
            header.col(|ui| table_header_cell(ui, "Field",   None,                    sep, false));
            header.col(|ui| table_header_cell(ui, "Initial", Some(Theme::NODE_INITIAL), sep, true));
            header.col(|ui| table_header_cell(ui, "Goal",    Some(Theme::NODE_GOAL),    sep, true));
        })
        .body(|mut body| {
            for (field_idx, field) in fields.iter().enumerate() {
                let color = Theme::bit_color(field_idx);
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| show_field_name_cell(ui, field, color, sep));
                    row.col(|ui| show_value_combo_cell(
                        ui, field, sep,
                        field.get_value(editor.mission().initial_state),
                        "initial",
                        |idx| editor.set_initial_field(&field.id, idx),
                    ));
                    row.col(|ui| show_value_combo_cell(
                        ui, field, sep,
                        field.get_value(editor.mission().goal_state),
                        "goal",
                        |idx| editor.set_goal_field(&field.id, idx),
                    ));
                });
            }
        });
}

// ── Table cell helpers ────────────────────────────────────────────────────────

/// Draws a header cell with an optional accent color and optional left divider.
fn table_header_cell(ui: &mut Ui, label: &str, color: Option<Color32>, sep: Stroke, divider: bool) {
    if divider {
        ui.painter().vline(ui.max_rect().left(), ui.max_rect().y_range(), sep);
    }
    let text = RichText::new(label).strong();
    let text = match color {
        Some(c) => text.color(c),
        None    => text,
    };
    ui.label(text);
    ui.painter().hline(ui.max_rect().x_range(), ui.max_rect().bottom(), sep);
}

/// Draws the field name, dot indicator, and bit-range badge.
fn show_field_name_cell(ui: &mut Ui, field: &BitField, color: Color32, sep: Stroke) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("●").color(color));
        ui.label(RichText::new(&field.name).color(color).strong().size(11.0));
        ui.label(
            RichText::new(format!("[{}-{}]", field.bit_offset, field.end_bit()))
                .color(Theme::TEXT_MUTED)
                .size(9.0),
        );
    });
    ui.painter().hline(ui.max_rect().x_range(), ui.max_rect().bottom(), sep);
}

/// Draws a value combo-box cell, shared by the Initial and Goal columns.
fn show_value_combo_cell(
    ui:            &mut Ui,
    field:         &BitField,
    sep:           Stroke,
    current_value: u64,
    id_prefix:     &str,
    mut on_select: impl FnMut(u64),
) {
    ui.painter().vline(ui.max_rect().left(), ui.max_rect().y_range(), sep);

    egui::ComboBox::from_id_salt(format!("{id_prefix}_{}", field.id))
        .selected_text(field.value_name(current_value))
        .width(COMBO_WIDTH)
        .show_ui(ui, |ui| {
            for (idx, name) in field.value_names.iter().enumerate() {
                let idx = idx as u64;
                if ui.selectable_label(idx == current_value, format!("{idx}: {name}")).clicked() {
                    on_select(idx);
                }
            }
        });

    ui.painter().hline(ui.max_rect().x_range(), ui.max_rect().bottom(), sep);
}