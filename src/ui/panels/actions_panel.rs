use egui::{Color32, RichText, ScrollArea, Stroke, Ui, Window};
use egui_extras::{Column, TableBuilder};
use crate::config::Theme;
use crate::data::{Action, BitField};
use crate::logic::EditorState;

// ── Constants ─────────────────────────────────────────────────────────────────

const COST_RANGE:      std::ops::RangeInclusive<u32> = 1..=100;
const COMBO_WIDTH:     f32 = 130.0;
const HEADER_HEIGHT:   f32 = 22.0;
const ROW_HEIGHT:      f32 = 28.0;
const TABLE_MAX_H:     f32 = 400.0;

// ── Bit condition types ───────────────────────────────────────────────────────

/// What the precondition requires for a field.
#[derive(Clone, PartialEq)]
enum PreCondition {
    DontCare,
    MustBe(u64),
}

/// What the effect does to a field.
#[derive(Clone, PartialEq)]
enum EffectCondition {
    NoChange,
    SetTo(u64),
}

// ── State ─────────────────────────────────────────────────────────────────────

pub struct ActionDialog {
    pub action: Action,
    pub mode:   DialogMode,
}

pub enum DialogMode {
    Add,
    Edit(String),
}

#[derive(Default)]
pub struct ActionsPanelState {
    pub selected_id: Option<String>,
    pub dialog:      Option<ActionDialog>,
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn show_actions_panel(ui: &mut Ui, editor: &mut EditorState, state: &mut ActionsPanelState) {
    let mut to_delete: Option<String> = None;

    ui.vertical(|ui| {
        ui.heading("⚡ GOAP Actions");
        show_toolbar(ui, editor, state);
        ui.separator();
        show_action_list(ui, editor, state, &mut to_delete);
    });

    if let Some(id) = to_delete {
        editor.delete_action(&id);
        if state.selected_id.as_deref() == Some(&id) {
            state.selected_id = None;
        }
    }

    show_action_dialog(ui, editor, state);
}

// ── Toolbar ───────────────────────────────────────────────────────────────────

fn show_toolbar(ui: &mut Ui, editor: &EditorState, state: &mut ActionsPanelState) {
    ui.horizontal(|ui| {
        if ui.button("➕ Add Action").clicked() {
            state.dialog = Some(ActionDialog {
                action: Action::new("New Action"),
                mode:   DialogMode::Add,
            });
        }
        if ui.button("✎ Edit Selected").clicked() {
            open_edit_dialog_for_selected(editor, state);
        }
        if ui.button("🗑 Delete Selected").clicked() {
            // handled via to_delete in caller
        }
    });
}

fn open_edit_dialog_for_selected(editor: &EditorState, state: &mut ActionsPanelState) {
    let Some(id) = &state.selected_id else { return };
    let Some(action) = editor.mission().actions.iter().find(|a| &a.id == id) else { return };
    state.dialog = Some(ActionDialog {
        action: action.clone(),
        mode:   DialogMode::Edit(id.clone()),
    });
}

fn open_edit_dialog_for_action(action: &Action, state: &mut ActionsPanelState) {
    state.dialog = Some(ActionDialog {
        action: action.clone(),
        mode:   DialogMode::Edit(action.id.clone()),
    });
}

// ── Action list ───────────────────────────────────────────────────────────────

fn show_action_list(
    ui:        &mut Ui,
    editor:    &EditorState,
    state:     &mut ActionsPanelState,
    to_delete: &mut Option<String>,
) {
    ScrollArea::vertical().id_salt("actions_scroll").show(ui, |ui| {
        for action in &editor.mission().actions {
            let is_sel = state.selected_id.as_deref() == Some(&action.id);
            show_action_row(ui, action, is_sel, state, to_delete);
        }
    });
}

fn show_action_row(
    ui:        &mut Ui,
    action:    &Action,
    is_sel:    bool,
    state:     &mut ActionsPanelState,
    to_delete: &mut Option<String>,
) {
    let row_response = ui.horizontal(|ui| {
        let response = ui.selectable_label(is_sel, &action.name);

        if response.clicked()        { state.selected_id = Some(action.id.clone()); }
        if response.double_clicked() { open_edit_dialog_for_action(action, state); }

        response.context_menu(|ui| {
            if ui.button("Edit").clicked() {
                open_edit_dialog_for_action(action, state);
                ui.close();
            }
            if ui.button("Delete").clicked() {
                *to_delete = Some(action.id.clone());
                ui.close();
            }
        });

        ui.label(
            RichText::new(format!("cost: {}", action.cost))
                .color(Theme::TEXT_MUTED)
                .size(10.0),
        );
    });

    if ui.rect_contains_pointer(row_response.response.rect) {
        ui.label(
            RichText::new(format!(
                "pre: 0x{:X}=0x{:X}  eff: 0x{:X}→0x{:X}",
                action.pre_mask, action.pre_value,
                action.effect_mask, action.effect_value,
            ))
                .monospace()
                .color(Theme::TEXT_MUTED)
                .size(9.0),
        );
    }
}

// ── Action dialog ─────────────────────────────────────────────────────────────

fn show_action_dialog(ui: &mut Ui, editor: &mut EditorState, state: &mut ActionsPanelState) {
    let Some(dialog) = &mut state.dialog else { return };

    let mut is_open = true;
    let mut save    = false;
    let mut cancel  = false;

    let title = match &dialog.mode {
        DialogMode::Add     => "Add Action",
        DialogMode::Edit(_) => "Edit Action",
    };

    Window::new(title)
        .open(&mut is_open)
        .resizable(true)
        .default_width(620.0)
        .show(ui.ctx(), |ui| {
            show_dialog_name_cost(ui, dialog);
            ui.separator();
            show_dialog_condition_table(ui, dialog, editor);
            ui.separator();
            show_dialog_mask_summary(ui, dialog);
            ui.separator();
            show_dialog_buttons(ui, &mut save, &mut cancel);
        });

    if save {
        commit_dialog(editor, state);
    } else if cancel || !is_open {
        state.dialog = None;
    }
}

fn show_dialog_name_cost(ui: &mut Ui, dialog: &mut ActionDialog) {
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut dialog.action.name);
    });
    ui.add(egui::Slider::new(&mut dialog.action.cost, COST_RANGE).text("Cost"));
}

fn show_dialog_condition_table(ui: &mut Ui, dialog: &mut ActionDialog, editor: &EditorState) {
    ui.label(RichText::new("Bit State Configuration").strong());
    ui.label(
        RichText::new("Configure preconditions and effects per field")
            .color(Theme::TEXT_MUTED)
            .size(10.0),
    );
    ui.add_space(4.0);
    ScrollArea::vertical()
        .max_height(TABLE_MAX_H)
        .show(ui, |ui| show_field_condition_table(ui, &mut dialog.action, editor));
}

fn show_dialog_mask_summary(ui: &mut Ui, dialog: &ActionDialog) {
    ui.group(|ui| {
        ui.label(RichText::new("Generated Masks").strong().size(11.0));
        ui.horizontal(|ui| {
            show_mask_column(ui, "Precondition", Theme::ACCENT_BLUE,
                             dialog.action.pre_mask, dialog.action.pre_value);
            ui.add_space(16.0);
            show_mask_column(ui, "Effect", Theme::ACCENT_ORANGE,
                             dialog.action.effect_mask, dialog.action.effect_value);
        });
    });
}

fn show_mask_column(ui: &mut Ui, label: &str, color: Color32, mask: u64, value: u64) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).color(color).size(10.0));
        ui.label(RichText::new(format!("Mask:  0x{mask:016X}")).monospace().size(9.0));
        ui.label(RichText::new(format!("Value: 0x{value:016X}")).monospace().size(9.0));
    });
}

fn show_dialog_buttons(ui: &mut Ui, save: &mut bool, cancel: &mut bool) {
    ui.horizontal(|ui| {
        if ui.button("💾 Save").clicked()   { *save   = true; }
        if ui.button("✕ Cancel").clicked() { *cancel = true; }
    });
}

fn commit_dialog(editor: &mut EditorState, state: &mut ActionsPanelState) {
    let Some(dialog) = &state.dialog else { return };
    match &dialog.mode {
        DialogMode::Add => editor.add_action(dialog.action.clone()),
        DialogMode::Edit(id) => {
            let src = dialog.action.clone();
            editor.update_action(id, |a| {
                a.name         = src.name.clone();
                a.cost         = src.cost;
                a.pre_mask     = src.pre_mask;
                a.pre_value    = src.pre_value;
                a.effect_mask  = src.effect_mask;
                a.effect_value = src.effect_value;
            });
        }
    }
    state.dialog = None;
}

// ── Field-level condition table ───────────────────────────────────────────────

fn show_field_condition_table(ui: &mut Ui, action: &mut Action, editor: &EditorState) {
    let sep    = Stroke::new(1.0, Theme::TEXT_MUTED);
    let fields = &editor.mission().bit_fields;

    // Collect unassigned single bits for display after named fields
    let assigned_bits: std::collections::HashSet<u8> = fields
        .iter()
        .flat_map(|f| f.bit_offset..=f.end_bit())
        .collect();

    TableBuilder::new(ui)
        .striped(false)
        .column(Column::auto().at_least(100.0))   // Field name
        .column(Column::auto().at_least(60.0))    // Bits
        .column(Column::remainder().at_least(COMBO_WIDTH)) // Precondition
        .column(Column::remainder().at_least(COMBO_WIDTH)) // Effect
        .header(HEADER_HEIGHT, |mut header| {
            header.col(|ui| table_header_cell(ui, "Field",        None,                        sep, false));
            header.col(|ui| table_header_cell(ui, "Bits",         None,                        sep, true));
            header.col(|ui| table_header_cell(ui, "Precondition", Some(Theme::ACCENT_BLUE),    sep, true));
            header.col(|ui| table_header_cell(ui, "Effect",       Some(Theme::ACCENT_ORANGE),  sep, true));
        })
        .body(|mut body| {
            // Named fields
            for (field_idx, field) in fields.iter().enumerate() {
                let color = Theme::bit_color(field_idx);
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        sep_hline(ui, sep);
                        ui.label(RichText::new(&field.name).color(color).strong().size(11.0));
                    });
                    row.col(|ui| {
                        sep_vline(ui, sep); sep_hline(ui, sep);
                        let range_text = if field.bit_width == 1 {
                            format!("{}", field.bit_offset)
                        } else {
                            format!("{}-{}", field.bit_offset, field.end_bit())
                        };
                        ui.label(RichText::new(range_text).monospace().color(Theme::TEXT_MUTED).size(10.0));
                    });
                    row.col(|ui| {
                        sep_vline(ui, sep); sep_hline(ui, sep);
                        show_field_precondition(ui, action, field, color);
                    });
                    row.col(|ui| {
                        sep_vline(ui, sep); sep_hline(ui, sep);
                        show_field_effect(ui, action, field, color);
                    });
                });
            }

            // Raw unassigned bits
            for bit_idx in 0u8..64 {
                if assigned_bits.contains(&bit_idx) { continue; }
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        sep_hline(ui, sep);
                        ui.label(RichText::new(format!("bit {bit_idx}")).color(Theme::TEXT_MUTED).size(10.0));
                    });
                    row.col(|ui| {
                        sep_vline(ui, sep); sep_hline(ui, sep);
                        ui.label(RichText::new(bit_idx.to_string()).monospace().color(Theme::TEXT_MUTED).size(10.0));
                    });
                    row.col(|ui| {
                        sep_vline(ui, sep); sep_hline(ui, sep);
                        show_raw_bit_precondition(ui, action, bit_idx);
                    });
                    row.col(|ui| {
                        sep_vline(ui, sep); sep_hline(ui, sep);
                        show_raw_bit_effect(ui, action, bit_idx);
                    });
                });
            }
        });
}

// ── Field-level condition combos ──────────────────────────────────────────────

/// Reads the current precondition for an entire multi-bit field.
/// Returns `DontCare` if any bit in the field is unmasked, otherwise `MustBe(value_index)`.
fn field_pre_condition(action: &Action, field: &BitField) -> PreCondition {
    let field_mask = field.mask();
    if (action.pre_mask & field_mask) != field_mask {
        return PreCondition::DontCare;
    }
    let raw_value = (action.pre_value & field_mask) >> field.bit_offset;
    PreCondition::MustBe(raw_value)
}

/// Reads the current effect for an entire multi-bit field.
fn field_effect_condition(action: &Action, field: &BitField) -> EffectCondition {
    let field_mask = field.mask();
    if (action.effect_mask & field_mask) != field_mask {
        return EffectCondition::NoChange;
    }
    let raw_value = (action.effect_value & field_mask) >> field.bit_offset;
    EffectCondition::SetTo(raw_value)
}

fn show_field_precondition(ui: &mut Ui, action: &mut Action, field: &BitField, color: Color32) {
    let field_mask  = field.mask();
    let current     = field_pre_condition(action, field);
    let current_txt = match &current {
        PreCondition::DontCare   => "Don't Care".to_string(),
        PreCondition::MustBe(v)  => field.value_name(*v).to_string(),
    };

    egui::ComboBox::from_id_salt(format!("pre_field_{}", field.id))
        .selected_text(current_txt)
        .width(COMBO_WIDTH)
        .show_ui(ui, |ui| {
            // Don't Care
            if ui.selectable_label(current == PreCondition::DontCare,
                                   RichText::new("Don't Care").color(Theme::TEXT_MUTED)
            ).clicked() {
                action.pre_mask &= !field_mask;
            }

            ui.separator();

            // One entry per named value
            for (idx, name) in field.value_names.iter().enumerate() {
                let idx     = idx as u64;
                let opt     = PreCondition::MustBe(idx);
                let label   = RichText::new(format!("{name}")).color(color);
                if ui.selectable_label(current == opt, label).clicked() {
                    action.pre_mask  |= field_mask;
                    // clear field bits then write new value
                    action.pre_value &= !field_mask;
                    action.pre_value |= (idx << field.bit_offset) & field_mask;
                }
            }
        });
}

fn show_field_effect(ui: &mut Ui, action: &mut Action, field: &BitField, color: Color32) {
    let field_mask  = field.mask();
    let current     = field_effect_condition(action, field);
    let current_txt = match &current {
        EffectCondition::NoChange => "No Change".to_string(),
        EffectCondition::SetTo(v) => field.value_name(*v).to_string(),
    };

    egui::ComboBox::from_id_salt(format!("eff_field_{}", field.id))
        .selected_text(current_txt)
        .width(COMBO_WIDTH)
        .show_ui(ui, |ui| {
            // No Change
            if ui.selectable_label(current == EffectCondition::NoChange,
                                   RichText::new("No Change").color(Theme::TEXT_MUTED)
            ).clicked() {
                action.effect_mask &= !field_mask;
            }

            ui.separator();

            // One entry per named value
            for (idx, name) in field.value_names.iter().enumerate() {
                let idx   = idx as u64;
                let opt   = EffectCondition::SetTo(idx);
                let label = RichText::new(format!("{name}")).color(color);
                if ui.selectable_label(current == opt, label).clicked() {
                    action.effect_mask  |= field_mask;
                    action.effect_value &= !field_mask;
                    action.effect_value |= (idx << field.bit_offset) & field_mask;
                }
            }
        });
}

// ── Raw single-bit combos (unassigned bits) ───────────────────────────────────

fn show_raw_bit_precondition(ui: &mut Ui, action: &mut Action, bit_idx: u8) {
    let bit_mask    = 1u64 << bit_idx;
    let is_masked   = (action.pre_mask  & bit_mask) != 0;
    let is_set      = (action.pre_value & bit_mask) != 0;
    let options     = ["Don't Care", "Must be 0", "Must be 1"];
    let current_idx = if !is_masked { 0 } else if is_set { 2 } else { 1 };

    egui::ComboBox::from_id_salt(format!("pre_raw_{bit_idx}"))
        .selected_text(options[current_idx])
        .width(COMBO_WIDTH)
        .show_ui(ui, |ui| {
            if ui.selectable_label(current_idx == 0, options[0]).clicked() {
                action.pre_mask &= !bit_mask;
            }
            if ui.selectable_label(current_idx == 1, options[1]).clicked() {
                action.pre_mask  |= bit_mask;
                action.pre_value &= !bit_mask;
            }
            if ui.selectable_label(current_idx == 2, options[2]).clicked() {
                action.pre_mask  |= bit_mask;
                action.pre_value |= bit_mask;
            }
        });
}

fn show_raw_bit_effect(ui: &mut Ui, action: &mut Action, bit_idx: u8) {
    let bit_mask    = 1u64 << bit_idx;
    let is_changed  = (action.effect_mask  & bit_mask) != 0;
    let new_value   = (action.effect_value & bit_mask) != 0;
    let options     = ["No Change", "Set to 0", "Set to 1"];
    let current_idx = if !is_changed { 0 } else if new_value { 2 } else { 1 };

    egui::ComboBox::from_id_salt(format!("eff_raw_{bit_idx}"))
        .selected_text(options[current_idx])
        .width(COMBO_WIDTH)
        .show_ui(ui, |ui| {
            if ui.selectable_label(current_idx == 0, options[0]).clicked() {
                action.effect_mask &= !bit_mask;
            }
            if ui.selectable_label(current_idx == 1, options[1]).clicked() {
                action.effect_mask  |= bit_mask;
                action.effect_value &= !bit_mask;
            }
            if ui.selectable_label(current_idx == 2, options[2]).clicked() {
                action.effect_mask  |= bit_mask;
                action.effect_value |= bit_mask;
            }
        });
}

// ── Painter helpers ───────────────────────────────────────────────────────────

fn table_header_cell(ui: &mut Ui, label: &str, color: Option<Color32>, sep: Stroke, divider: bool) {
    if divider { sep_vline(ui, sep); }
    let text = RichText::new(label).strong();
    ui.label(match color { Some(c) => text.color(c), None => text });
    sep_hline(ui, sep);
}

fn sep_hline(ui: &Ui, sep: Stroke) {
    ui.painter().hline(ui.max_rect().x_range(), ui.max_rect().bottom(), sep);
}

fn sep_vline(ui: &Ui, sep: Stroke) {
    ui.painter().vline(ui.max_rect().left(), ui.max_rect().y_range(), sep);
}