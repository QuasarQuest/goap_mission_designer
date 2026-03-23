use egui::{Color32, Context, Pos2, Rect, RichText, ScrollArea, Sense, Stroke, Ui, Vec2};
use crate::config::Theme;
use crate::logic::EditorState;
use crate::data::BitField;
use crate::ui::widgets::{heading, icon_btn};
use crate::ui::dialogs::FieldDialog;

const TOTAL_BITS:  u8  = 64;
const GRID_COLS:   u8  = 16;
const CELL_SIZE:   f32 = 18.0;
const CELL_GAP:    f32 = 3.0;
const CELL_STRIDE: f32 = CELL_SIZE + CELL_GAP;

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct FieldsPanelState {
    pub selected_id: Option<String>,
    pub add_dialog:  Option<FieldDialog>,
    pub edit_dialog: Option<(String, FieldDialog)>,
}

// ── Public entry point ────────────────────────────────────────────────────────

pub fn show_fields_panel(ui: &mut Ui, editor: &mut EditorState, state: &mut FieldsPanelState) {
    let fields: Vec<_> = editor.mission().sorted_fields().into_iter().cloned().collect();

    ui.vertical(|ui| {
        heading(ui, "⬛ Bit Fields");
        show_toolbar(ui, editor, state);
        ui.add_space(4.0);
        show_field_list(ui, &fields, state);
    });

    if let Some(new_field) = handle_add_dialog(ui.ctx(), &mut state.add_dialog, editor) {
        editor.add_field(new_field);
    }

    if let Some((id, name, values)) = handle_edit_dialog(ui.ctx(), &mut state.edit_dialog, editor) {
        editor.update_field_name(&id, name);
        editor.update_field_values(&id, values);
    }
}

// ── Toolbar ───────────────────────────────────────────────────────────────────

fn show_toolbar(ui: &mut Ui, editor: &mut EditorState, state: &mut FieldsPanelState) {
    ui.horizontal(|ui| {
        if icon_btn(ui, "➕", "Add field") {
            state.add_dialog = Some(FieldDialog::default());
        }
        if icon_btn(ui, "✏️", "Edit selected") {
            if let Some(id) = &state.selected_id {
                if let Some(field) = editor.mission().field_by_id(id) {
                    state.edit_dialog = Some((id.clone(), FieldDialog::from_field(field)));
                }
            }
        }
        if icon_btn(ui, "🗑", "Delete selected") {
            if let Some(id) = state.selected_id.take() {
                editor.delete_field(&id);
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{}/{TOTAL_BITS} bits", editor.mission().bits_used()))
                    .color(Theme::TEXT_SECONDARY)
                    .size(11.0),
            );
        });
    });
}

// ── Bit colour map ────────────────────────────────────────────────────────────

fn build_bit_map(fields: &[BitField]) -> ([Color32; 64], [Option<usize>; 64]) {
    let mut colors:    [Color32;       64] = [Theme::UNUSED_BIT_COLOR; 64];
    let mut field_idx: [Option<usize>; 64] = [None; 64];
    for (fi, field) in fields.iter().enumerate() {
        let color = Theme::bit_color(fi);
        for bit in field.bit_offset..=field.end_bit() {
            let b = bit as usize;
            if b < 64 { colors[b] = color; field_idx[b] = Some(fi); }
        }
    }
    (colors, field_idx)
}

/// Lighten a colour towards white by `amount` (0.0 = unchanged, 1.0 = white).
fn lighten(c: Color32, amount: f32) -> Color32 {
    let blend = |ch: u8| (ch as f32 + (255.0 - ch as f32) * amount) as u8;
    Color32::from_rgb(blend(c.r()), blend(c.g()), blend(c.b()))
}

/// Darken a colour by `amount` (0.0 = unchanged, 1.0 = black).
fn darken(c: Color32, amount: f32) -> Color32 {
    let blend = |ch: u8| (ch as f32 * (1.0 - amount)) as u8;
    Color32::from_rgb(blend(c.r()), blend(c.g()), blend(c.b()))
}

// ── Interactive bit-range picker ──────────────────────────────────────────────

/// Renders a 16×4 grid where:
/// - All fields are shown in their theme colour (like the toolbar bar)
/// - The current selection is shown **lighter/raised** with a bright border
/// - Taken bits belonging to *other* fields cannot be selected
///
/// Returns `true` if the selection changed.
fn show_bit_range_picker(
    ui:           &mut Ui,
    dialog:       &mut FieldDialog,
    other_fields: &[BitField],  // fields that should be shown as taken (not the one being edited)
) -> bool {
    let (taken_colors, taken_field_idx) = build_bit_map(other_fields);

    let is_free = |bit: usize| taken_colors[bit] == Theme::UNUSED_BIT_COLOR;

    let current_offset = dialog.bit_offset.parse::<u8>().unwrap_or(0);
    let current_width  = dialog.bit_width.parse::<u8>().unwrap_or(1).max(1);
    let current_end    = (current_offset + current_width).saturating_sub(1).min(63);

    let rows   = (TOTAL_BITS as f32 / GRID_COLS as f32).ceil() as usize;
    let width  = GRID_COLS as f32 * CELL_STRIDE - CELL_GAP;
    let height = rows as f32 * CELL_STRIDE - CELL_GAP + 14.0;

    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click_and_drag());
    let painter = ui.painter_at(rect);

    // Persist drag-start across frames
    let drag_id                  = resp.id;
    let drag_start: Option<u8>   = ui.memory(|m| m.data.get_temp(drag_id));

    let pos_to_bit = |pos: Pos2| -> Option<u8> {
        let rel_x = pos.x - rect.left();
        let rel_y = pos.y - rect.top() - 12.0;
        if rel_x < 0.0 || rel_y < 0.0 { return None; }
        let col = (rel_x / CELL_STRIDE) as usize;
        let row = (rel_y / CELL_STRIDE) as usize;
        if col >= GRID_COLS as usize { return None; }
        let bit = row * GRID_COLS as usize + col;
        if bit < 64 { Some(bit as u8) } else { None }
    };

    // Clamp [lo, hi] so no taken bit is included, keeping lo fixed.
    let clamp_range = |lo: u8, hi: u8| -> Option<(u8, u8)> {
        if !is_free(lo as usize) { return None; }
        let mut clear_end = lo;
        for b in lo..=hi {
            if b as usize >= 64 || !is_free(b as usize) { break; }
            clear_end = b;
        }
        let w = (clear_end - lo + 1).min(8);
        Some((lo, lo + w - 1))
    };

    let mut new_offset = current_offset;
    let mut new_end    = current_end;
    let mut changed    = false;

    if resp.drag_started() {
        if let Some(pos) = ui.input(|i| i.pointer.press_origin()) {
            if let Some(bit) = pos_to_bit(pos) {
                if is_free(bit as usize) {
                    ui.memory_mut(|m| m.data.insert_temp(drag_id, bit));
                    new_offset = bit; new_end = bit; changed = true;
                }
            }
        }
    } else if resp.dragged() {
        if let (Some(start), Some(pos)) = (drag_start, ui.input(|i| i.pointer.hover_pos())) {
            if let Some(bit) = pos_to_bit(pos) {
                let (lo, hi) = (start.min(bit), start.max(bit));
                if let Some((o, e)) = clamp_range(lo, hi) {
                    new_offset = o; new_end = e; changed = true;
                }
            }
        }
    } else if resp.drag_stopped() {
        ui.memory_mut(|m| m.data.remove::<u8>(drag_id));
    } else if resp.clicked() {
        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            if let Some(bit) = pos_to_bit(pos) {
                if is_free(bit as usize) {
                    new_offset = bit; new_end = bit; changed = true;
                }
            }
        }
    }

    if changed {
        let w = (new_end - new_offset + 1).min(8);
        dialog.bit_offset = new_offset.to_string();
        dialog.bit_width  = w.to_string();
    }

    // Resolve highlight from (potentially updated) dialog
    let hi_offset = dialog.bit_offset.parse::<u8>().unwrap_or(0);
    let hi_width  = dialog.bit_width.parse::<u8>().unwrap_or(1).max(1);
    let hi_end    = (hi_offset + hi_width).saturating_sub(1).min(63);

    // ── Draw ─────────────────────────────────────────────────────────────────
    // Determine a colour for the "new field being created" — pick the next
    // palette index after all existing other_fields.
    let new_field_color = Theme::bit_color(other_fields.len());

    for bit in 0..TOTAL_BITS as usize {
        let col  = (bit % GRID_COLS as usize) as f32;
        let row  = (bit / GRID_COLS as usize) as f32;
        let x    = rect.left() + col * CELL_STRIDE;
        let y    = rect.top()  + row * CELL_STRIDE + 12.0;
        let cell = Rect::from_min_size(Pos2::new(x, y), Vec2::splat(CELL_SIZE));

        let in_sel = bit as u8 >= hi_offset && bit as u8 <= hi_end;
        let taken  = !is_free(bit);

        // Colour logic — matches toolbar bar aesthetic:
        //   taken other field  → field colour, darkened (locked, recessed)
        //   in selection       → new field colour, lightened (raised, bright)
        //   free, not selected → UNUSED_BIT_COLOR (dim background)
        let bg = if taken {
            darken(taken_colors[bit], 0.25)
        } else if in_sel {
            lighten(new_field_color, 0.45)
        } else {
            Theme::UNUSED_BIT_COLOR
        };

        painter.rect_filled(cell, 2.0, bg);

        // "Raised" highlight border on selected cells
        if in_sel {
            // Bright top/left edge to simulate elevation
            painter.rect_stroke(
                cell, 2.0,
                Stroke::new(2.0, lighten(new_field_color, 0.7)),
                egui::StrokeKind::Outside,
            );
        }

        // Subtle lock indicator on taken cells (no emoji — just a dim overlay line)

        // Column index labels on top row
        if row == 0.0 {
            painter.text(
                Pos2::new(x + CELL_SIZE * 0.5, rect.top()),
                egui::Align2::CENTER_TOP,
                bit.to_string(),
                egui::FontId::proportional(7.0),
                Theme::TEXT_MUTED,
            );
        }
    }

    // Hover tooltip
    if let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
        if rect.contains(hover_pos) {
            if let Some(bit) = pos_to_bit(hover_pos) {
                let tip = match taken_field_idx[bit as usize] {
                    Some(i) => format!("bit {} — {}", bit, other_fields[i].name),
                    None    => format!("bit {} — free", bit),
                };
                resp.on_hover_text(tip);
            }
        }
    }

    changed
}

// ── Field list ────────────────────────────────────────────────────────────────

fn show_field_list(ui: &mut Ui, fields: &[BitField], state: &mut FieldsPanelState) {
    ScrollArea::vertical().id_salt("fields_scroll").show(ui, |ui| {
        for (idx, field) in fields.iter().enumerate() {
            let is_sel = state.selected_id.as_deref() == Some(&field.id);
            let (clicked, double_clicked) = show_field_item(ui, field, idx, is_sel);
            if clicked        { state.selected_id = Some(field.id.clone()); }
            if double_clicked {
                state.edit_dialog = Some((field.id.clone(), FieldDialog::from_field(field)));
            }
            ui.add_space(3.0);
        }
    });
}

fn show_field_item(ui: &mut Ui, field: &BitField, idx: usize, is_sel: bool) -> (bool, bool) {
    let color = Theme::bit_color(idx);
    let bg    = if is_sel { Theme::BG_SELECTED } else { Theme::BG_CARD };

    let fr = egui::Frame::default()
        .fill(bg)
        .inner_margin(egui::Margin::symmetric(8, 5))
        .corner_radius(Theme::CARD_ROUNDING)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                show_color_swatch(ui, color);
                ui.add_space(4.0);
                show_field_info(ui, field, color);
            });
        });

    let response = fr.response.interact(Sense::click());
    response.context_menu(|ui| {
        if ui.button("Edit").clicked() { ui.close(); }
    });
    (response.clicked(), response.double_clicked())
}

fn show_color_swatch(ui: &mut Ui, color: Color32) {
    let (swatch, _) = ui.allocate_exact_size(Vec2::new(5.0, 18.0), Sense::hover());
    ui.painter().rect_filled(swatch, 2.0, color);
}

fn show_field_info(ui: &mut Ui, field: &BitField, color: Color32) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(&field.name).strong().color(color).size(12.0));
            ui.label(
                RichText::new(format!(
                    "bits {}-{}  ({}b)",
                    field.bit_offset, field.end_bit(), field.bit_width
                ))
                    .color(Theme::TEXT_MUTED)
                    .size(10.0),
            );
        });
        let value_summary = field.value_names.iter().enumerate()
            .map(|(i, name)| {
                let bits = format!("{:0>width$b}", i, width = field.bit_width as usize);
                format!("{bits}={name}")
            })
            .collect::<Vec<_>>()
            .join("  ");
        ui.label(RichText::new(value_summary).color(Theme::TEXT_SECONDARY).size(10.0));
    });
}

// ── Dialogs ───────────────────────────────────────────────────────────────────

fn handle_add_dialog(
    ctx:          &Context,
    dialog_state: &mut Option<FieldDialog>,
    editor:       &EditorState,
) -> Option<BitField> {
    let dialog = dialog_state.as_mut()?;
    // All existing fields are "taken" when adding a new one
    let all_fields: Vec<BitField> = editor.mission().sorted_fields().into_iter().cloned().collect();

    let mut open   = true;
    let mut result = None;
    let mut close  = false;

    egui::Window::new("Add Bit Field")
        .collapsible(false)
        .resizable(true)
        .default_width(390.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut dialog.name);
            });
            ui.add_space(6.0);
            ui.label(
                RichText::new("Click or drag to select a free bit range:")
                    .color(Theme::TEXT_SECONDARY).size(11.0),
            );
            let changed = show_bit_range_picker(ui, dialog, &all_fields);
            show_selection_summary(ui, dialog);
            if changed { sync_value_names(dialog); }
            ui.add_space(4.0);
            show_value_name_editor(ui, dialog);
            ui.separator();
            for e in &dialog.errors {
                ui.colored_label(Color32::from_rgb(248, 113, 113), e);
            }
            ui.horizontal(|ui| {
                if ui.button("Add").clicked() && dialog.validate(editor.mission()).is_empty() {
                    result = Some(dialog.build_field());
                    close  = true;
                }
                if ui.button("Cancel").clicked() { close = true; }
            });
        });

    if result.is_some() || close || !open { *dialog_state = None; }
    result
}

fn handle_edit_dialog(
    ctx:          &Context,
    dialog_state: &mut Option<(String, FieldDialog)>,
    editor:       &EditorState,
) -> Option<(String, String, Vec<String>)> {
    let (id, mut dialog) = dialog_state.take()?;

    // All fields *except* the one being edited count as taken
    let other_fields: Vec<BitField> = editor.mission()
        .sorted_fields()
        .into_iter()
        .filter(|f| f.id != id)
        .cloned()
        .collect();

    let mut open   = true;
    let mut result = None;
    let mut close  = false;

    egui::Window::new("Edit Bit Field")
        .collapsible(false)
        .resizable(true)
        .default_width(390.0)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut dialog.name);
            });
            ui.add_space(6.0);
            ui.label(
                RichText::new("Click or drag to change bit range:")
                    .color(Theme::TEXT_SECONDARY).size(11.0),
            );
            let changed = show_bit_range_picker(ui, &mut dialog, &other_fields);
            show_selection_summary(ui, &dialog);
            if changed { sync_value_names(&mut dialog); }
            ui.add_space(4.0);
            show_value_name_editor(ui, &mut dialog);
            ui.separator();
            for e in &dialog.errors {
                ui.colored_label(Color32::from_rgb(248, 113, 113), e);
            }
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    result = Some((id.clone(), dialog.name.clone(), dialog.value_names_vec()));
                    close  = true;
                }
                if ui.button("Cancel").clicked() { close = true; }
            });
        });

    if open && !close { *dialog_state = Some((id, dialog)); }
    result
}

// ── Dialog helpers ────────────────────────────────────────────────────────────

fn show_selection_summary(ui: &mut Ui, dialog: &FieldDialog) {
    let offset = dialog.bit_offset.parse::<u8>().unwrap_or(0);
    let width  = dialog.bit_width.parse::<u8>().unwrap_or(1).max(1);
    let end    = (offset + width).saturating_sub(1);
    let slots  = 1u32 << width.min(8);
    ui.label(
        RichText::new(format!("bits {offset}–{end}  ({width}-bit → {slots} values)"))
            .color(Theme::TEXT_SECONDARY).size(11.0),
    );
}

fn sync_value_names(dialog: &mut FieldDialog) {
    let width      = dialog.bit_width().max(1).min(8);
    let slot_count = 1usize << width;
    let names      = dialog.value_names_mut();
    names.resize_with(slot_count, String::new);
    for (i, name) in names.iter_mut().enumerate() {
        if name.is_empty() { *name = default_value_name(i, width); }
    }
}

fn default_value_name(index: usize, bit_width: u8) -> String {
    if bit_width == 1 { return if index == 0 { "Off".into() } else { "On".into() }; }
    format!("{:0>width$b}", index, width = bit_width as usize)
}

fn show_value_name_editor(ui: &mut Ui, dialog: &mut FieldDialog) {
    let bit_width:  u8    = dialog.bit_width().max(1).min(8);
    let slot_count: usize = 1 << bit_width;

    ui.separator();
    ui.label(RichText::new(format!("Value names  ({slot_count} slots)")).strong().size(11.0));
    ui.add_space(2.0);

    let mut names: Vec<String> = std::mem::take(dialog.value_names_mut());
    names.resize_with(slot_count, String::new);

    ScrollArea::vertical()
        .max_height(150.0)
        .id_salt("val_names_scroll")
        .show(ui, |ui| {
            for (i, name) in names.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    let pat = format!("{:0>width$b}", i, width = bit_width as usize);
                    ui.label(
                        RichText::new(format!("{pat} ({i})"))
                            .monospace().color(Theme::TEXT_MUTED).size(10.0),
                    );
                    ui.text_edit_singleline(name);
                });
            }
        });

    *dialog.value_names_mut() = names;
}