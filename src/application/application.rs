use eframe::egui;
use egui::{Context, Key};

use crate::config::Theme;
use crate::logic::EditorState;

use crate::ui::{
    self,
    bit_layout::show_bit_layout,
    panels,
    toolbar::ToolbarAction,
};

#[derive(Default, PartialEq, Clone, Copy)]
enum Tab { #[default] Editor, Graph, Code }

pub struct GOAPApp {
    editor:        EditorState,
    fields_state:  panels::FieldsPanelState,
    actions_state: panels::ActionsPanelState,
    code_state:    panels::CodePanelState,
    active_tab:    Tab,
    open_error:    Option<String>,
}

impl Default for GOAPApp {
    fn default() -> Self {
        Self {
            editor:        EditorState::new(MissionDefinition::default()),
            fields_state:  panels::FieldsPanelState::default(),
            actions_state: panels::ActionsPanelState::default(),
            code_state:    panels::CodePanelState::default(),
            active_tab:    Tab::Editor,
            open_error:    None,
        }
    }
}

use crate::data::MissionDefinition;

impl eframe::App for GOAPApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill       = Theme::BG_PRIMARY;
        visuals.window_fill      = Theme::BG_PANEL;
        visuals.extreme_bg_color = Theme::BG_CARD;
        visuals.faint_bg_color   = Theme::BG_CARD;
        ctx.set_visuals(visuals);

        self.handle_shortcuts(ctx);
        self.show_open_error(ctx);

        // Toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            let action = ui::toolbar::show_toolbar(ui, &mut self.editor);
            self.handle_toolbar_action(action);
        });

        // Tabs
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Editor, "📝 Editor");
                ui.selectable_value(&mut self.active_tab, Tab::Graph,  "📊 State Graph");
                ui.selectable_value(&mut self.active_tab, Tab::Code,   "🦀 Rust Code");
            });
        });

        // Status bar
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let m = self.editor.mission();
                ui.label(format!(
                    "{}  |  type {} |  variant {}  |  {} fields  |  {} actions  |  {}/64 bits",
                    m.name, m.mission_type, m.variant,
                    m.bit_fields.len(), m.actions.len(), m.bits_used(),
                ));
                let errors = m.validate();
                if !errors.is_empty() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.colored_label(Theme::TEXT_ERROR,
                                         format!("⚠ {} error(s)", errors.len()),
                        ).on_hover_text(errors.join("\n"));
                    });
                }
            });
        });

        // Bit layout bar (editor only)
        if self.active_tab == Tab::Editor {
            egui::TopBottomPanel::top("bit_layout").show(ctx, |ui| {
                show_bit_layout(ui, self.editor.mission());
            });
        }

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Editor => {
                    ui.columns(3, |cols| {
                        panels::show_fields_panel(&mut cols[0], &mut self.editor, &mut self.fields_state);
                        panels::show_states_panel(&mut cols[1], &mut self.editor);
                        panels::show_actions_panel(&mut cols[2], &mut self.editor, &mut self.actions_state);
                    });
                }
                Tab::Graph => panels::show_graph_panel(ui, &mut self.editor),
                Tab::Code  => panels::show_code_panel(ui, &self.editor, &mut self.code_state),
            }
        });

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.editor.window_title()));
    }
}

impl GOAPApp {
    fn handle_toolbar_action(&mut self, action: ToolbarAction) {
        match action {
            ToolbarAction::New    => {
                self.editor.replace_mission(MissionDefinition::default());
                self.code_state.dirty = true;
            }
            ToolbarAction::Open   => self.open_file(),
            ToolbarAction::Import => self.import_rust(),
            ToolbarAction::Save   => self.save_file(false),
            ToolbarAction::SaveAs => self.save_file(true),
            ToolbarAction::Export => self.export_rust(),
            ToolbarAction::Undo   => { self.editor.undo(); self.code_state.dirty = true; }
            ToolbarAction::Redo   => { self.editor.redo(); self.code_state.dirty = true; }
            ToolbarAction::None   => {}
        }
    }

    fn handle_shortcuts(&mut self, ctx: &Context) {
        ctx.input(|i| {
            let ctrl = i.modifiers.ctrl || i.modifiers.mac_cmd;
            if ctrl && i.key_pressed(Key::Z) && !i.modifiers.shift {
                self.editor.undo();
                self.code_state.dirty = true;
            }
            if ctrl && (i.key_pressed(Key::Y) ||
                (i.modifiers.shift && i.key_pressed(Key::Z))) {
                self.editor.redo();
                self.code_state.dirty = true;
            }
        });
    }

    fn open_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("GOAP Mission", &["json"])
            .pick_file()
        else { return; };

        let Ok(content) = std::fs::read_to_string(&path) else {
            self.open_error = Some(format!("Could not read file:\n{}", path.display()));
            return;
        };

        match MissionDefinition::from_json(&content).map_err(|e| e.to_string()) {
            Ok(mission) => {
                self.editor.replace_mission(mission);
                self.editor.current_file = Some(path.to_string_lossy().into_owned());
                self.code_state.dirty = true;
            }
            Err(msg) => {
                self.open_error = Some(format!(
                    "Failed to open {}:\n\n{}",
                    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
                    msg,
                ));
            }
        }
    }

    fn import_rust(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Rust Source", &["rs"])
            .pick_file()
        else { return; };

        let Ok(content) = std::fs::read_to_string(&path) else {
            self.open_error = Some(format!("Could not read file:\n{}", path.display()));
            return;
        };

        match crate::utils::parse_rust(&content) {
            Ok(mission) => {
                self.editor.replace_mission(mission);
                self.code_state.dirty = true;
            }
            Err(msg) => {
                self.open_error = Some(format!(
                    "Failed to import {}:\n\n{}",
                    path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
                    msg,
                ));
            }
        }
    }

    fn save_file(&mut self, force_dialog: bool) {
        let path = if force_dialog || self.editor.current_file.is_none() {
            rfd::FileDialog::new()
                .add_filter("GOAP Mission", &["json"])
                .set_file_name(format!("{}.json", self.editor.mission().name))
                .save_file()
                .map(|p| p.to_string_lossy().into_owned())
        } else {
            self.editor.current_file.clone()
        };
        if let Some(path) = path {
            if let Ok(json) = self.editor.mission().to_json() {
                let _ = std::fs::write(&path, json);
                self.editor.mark_saved(path);
            }
        }
    }

    fn export_rust(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Rust Source", &["rs"])
            .set_file_name(format!("{}.rs", self.editor.mission().name.to_lowercase()))
            .save_file()
        {
            let code = crate::utils::generate_rust(self.editor.mission());
            let _    = std::fs::write(path, code);
        }
    }

    fn show_open_error(&mut self, ctx: &Context) {
        if self.open_error.is_none() { return; }
        let mut open = true;
        egui::Window::new("⚠ Open failed")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(self.open_error.as_deref().unwrap_or_default());
                ui.add_space(8.0);
                if ui.button("OK").clicked() { self.open_error = None; }
            });
        if !open { self.open_error = None; }
    }
}