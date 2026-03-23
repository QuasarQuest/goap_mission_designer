use egui::{FontId, ScrollArea, TextEdit, Ui};
use crate::logic::EditorState;
use crate::utils::generate_rust;
use crate::ui::widgets::icon_btn;

pub struct CodePanelState {
    pub code:  String,
    pub dirty: bool,
}

impl Default for CodePanelState {
    fn default() -> Self {
        Self { code: String::new(), dirty: true }
    }
}

pub fn show_code_panel(ui: &mut Ui, editor: &EditorState, state: &mut CodePanelState) {
    if state.dirty || editor.is_dirty {
        state.code  = generate_rust(editor.mission());
        state.dirty = false;
    }

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            if icon_btn(ui, "🔄", "Regenerate") {
                state.code = generate_rust(editor.mission());
            }
            if icon_btn(ui, "📋", "Copy to clipboard") {
                ui.ctx().copy_text(state.code.clone());
            }
        });

        ui.separator();

        // TextEdit::multiline needs &mut String; we pass a mutable clone
        // and discard changes (read-only display with syntax styling)
        let mut display = state.code.clone();
        ScrollArea::both().id_salt("code_scroll").show(ui, |ui| {
            ui.add(
                TextEdit::multiline(&mut display)
                    .font(FontId::monospace(12.0))
                    .desired_width(f32::INFINITY)
                    .code_editor(),
            );
        });
    });
}
