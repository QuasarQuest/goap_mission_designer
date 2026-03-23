use egui::{RichText, Ui, Button, Layout, Align};
use crate::config::Theme;
use crate::logic::EditorState;

#[derive(PartialEq)]
pub enum ToolbarAction {
    None,
    New, Open, Save, SaveAs, Export, Import,
    Undo, Redo
}

pub fn show_toolbar(ui: &mut Ui, editor: &mut EditorState) -> ToolbarAction {
    let mut action = ToolbarAction::None;

    egui::MenuBar::new().ui(ui, |ui| {
        show_file_menu(ui, &mut action);
        show_edit_menu(ui, editor, &mut action);

        ui.separator();
        show_mission_fields(ui, editor);

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            show_right_toolbar(ui, editor, &mut action);
        });
    });

    action
}

fn show_file_menu(ui: &mut Ui, action: &mut ToolbarAction) {
    ui.menu_button("File", |ui| {
        if ui.button("⭐ New").clicked()          { *action = ToolbarAction::New;    ui.close(); }
        if ui.button("📂 Open…").clicked()        { *action = ToolbarAction::Open;   ui.close(); }
        ui.separator();
        if ui.button("💾 Save").clicked()         { *action = ToolbarAction::Save;   ui.close(); }
        if ui.button("💾 Save As…").clicked()     { *action = ToolbarAction::SaveAs; ui.close(); }
        ui.separator();
        if ui.button("📤 Export Rust…").clicked() { *action = ToolbarAction::Export; ui.close(); }
        if ui.button("📥 Import Rust…").clicked() { *action = ToolbarAction::Import; ui.close(); }
    });
}

fn show_edit_menu(ui: &mut Ui, editor: &EditorState, action: &mut ToolbarAction) {
    ui.menu_button("Edit", |ui| {
        if ui.add_enabled(editor.undo.can_undo(), Button::new("↩ Undo")).clicked() {
            *action = ToolbarAction::Undo;
            ui.close();
        }
        if ui.add_enabled(editor.undo.can_redo(), Button::new("↪ Redo")).clicked() {
            *action = ToolbarAction::Redo;
            ui.close();
        }
    });
}

fn show_mission_fields(ui: &mut Ui, editor: &mut EditorState) {
    ui.label(RichText::new("Mission:").color(Theme::TEXT_SECONDARY));
    let mut name = editor.mission().name.clone();
    ui.add(egui::TextEdit::singleline(&mut name).desired_width(200.0));
    if name != editor.mission().name {
        editor.set_name(name);
    }
}

fn show_right_toolbar(ui: &mut Ui, editor: &EditorState, action: &mut ToolbarAction) {
    if editor.is_dirty {
        ui.label(RichText::new("Modified").color(Theme::TEXT_WARNING))
            .on_hover_text("There are unsaved changes in the editor");
    }

    ui.separator();

    if ui.add_enabled(editor.undo.can_redo(), Button::new("↪ Redo")).clicked() {
        *action = ToolbarAction::Redo;
    }
    if ui.add_enabled(editor.undo.can_undo(), Button::new("↩ Undo")).clicked() {
        *action = ToolbarAction::Undo;
    }
}