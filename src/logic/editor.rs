use crate::data::{Action, BitField, MissionDefinition};
use crate::logic::UndoStack;
use crate::utils::StateGraph;

/// All mutation of the mission goes through here.
/// Every mutating method takes a snapshot for undo.
pub struct EditorState {
    pub undo:          UndoStack,
    pub is_dirty:      bool,
    pub current_file:  Option<String>,
    pub graph:         Option<StateGraph>,
    pub graph_dirty:   bool, // needs rebuild
}

impl EditorState {
    pub fn new(mission: MissionDefinition) -> Self {
        Self {
            undo:         UndoStack::new(mission),
            is_dirty:     false,
            current_file: None,
            graph:        None,
            graph_dirty:  true,
        }
    }

    pub fn mission(&self) -> &MissionDefinition {
        self.undo.current()
    }

    /// Get a mutable clone, mutate it, then push to undo
    fn mutate<F: FnOnce(&mut MissionDefinition)>(&mut self, f: F) {
        let mut next = self.undo.current().clone();
        f(&mut next);
        self.undo.push(next);
        self.is_dirty   = true;
        self.graph_dirty = true;
    }

    /// Mark as modified without creating undo snapshot
    pub fn mark_modified(&mut self) {
        self.is_dirty   = true;
        self.graph_dirty = true;
    }

    // ── Undo / Redo ─────────────────────────────────────────────────────────

    pub fn can_undo(&self) -> bool {
        self.undo.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.undo.can_redo()
    }

    pub fn undo(&mut self) {
        self.undo.undo();
        self.is_dirty    = true;
        self.graph_dirty = true;
    }

    pub fn redo(&mut self) {
        self.undo.redo();
        self.is_dirty    = true;
        self.graph_dirty = true;
    }

    // ── Mission meta ─────────────────────────────────────────────────────────

    pub fn set_name(&mut self, name: String) {
        self.mutate(|m| m.name = name);
    }

    pub fn set_variant(&mut self, variant: u8) {
        self.mutate(|m| m.variant = variant);
    }

    // ── Bit Fields ───────────────────────────────────────────────────────────

    pub fn add_field(&mut self, field: BitField) {
        self.mutate(|m| {
            m.bit_fields.push(field);
            m.bit_fields.sort_by_key(|f| f.bit_offset);
        });
    }

    pub fn delete_field(&mut self, id: &str) {
        self.mutate(|m| m.bit_fields.retain(|f| f.id != id));
    }

    pub fn update_field_name(&mut self, id: &str, name: String) {
        self.mutate(|m| {
            if let Some(f) = m.field_by_id_mut(id) { f.name = name; }
        });
    }

    pub fn update_field_values(&mut self, id: &str, values: Vec<String>) {
        self.mutate(|m| {
            if let Some(f) = m.field_by_id_mut(id) { f.value_names = values; }
        });
    }

    pub fn update_field(&mut self, id: &str, field: BitField) {
        self.mutate(|m| {
            if let Some(idx) = m.bit_fields.iter().position(|f| f.id == id) {
                m.bit_fields[idx] = field;
                m.bit_fields.sort_by_key(|f| f.bit_offset);
            }
        });
    }

    // ── States ───────────────────────────────────────────────────────────────

    pub fn set_initial_state(&mut self, state: u64) {
        self.mutate(|m| m.initial_state = state);
    }

    pub fn set_goal_state(&mut self, state: u64) {
        self.mutate(|m| m.goal_state = state);
    }

    pub fn set_initial_field(&mut self, field_id: &str, value: u64) {
        let field = self.mission()
            .field_by_id(field_id)
            .cloned();
        if let Some(f) = field {
            self.mutate(|m| {
                m.initial_state = f.set_value(m.initial_state, value);
            });
        }
    }

    pub fn set_goal_field(&mut self, field_id: &str, value: u64) {
        let field = self.mission()
            .field_by_id(field_id)
            .cloned();
        if let Some(f) = field {
            self.mutate(|m| {
                m.goal_state = f.set_value(m.goal_state, value);
            });
        }
    }

    // ── Actions ──────────────────────────────────────────────────────────────

    pub fn add_action(&mut self, action: Action) {
        self.mutate(|m| m.actions.push(action));
    }

    pub fn delete_action(&mut self, id: &str) {
        self.mutate(|m| m.actions.retain(|a| a.id != id));
    }

    pub fn update_action<F: FnOnce(&mut Action)>(&mut self, id: &str, f: F) {
        self.mutate(|m| {
            if let Some(a) = m.action_by_id_mut(id) { f(a); }
        });
    }

    // ── Graph ────────────────────────────────────────────────────────────────

    pub fn rebuild_graph_if_needed(&mut self) {
        if self.graph_dirty {
            self.graph       = Some(StateGraph::build(self.mission()));
            self.graph_dirty = false;
        }
    }

    // ── File helpers ─────────────────────────────────────────────────────────

    pub fn mark_saved(&mut self, path: String) {
        self.current_file = Some(path);
        self.is_dirty     = false;
    }

    pub fn replace_mission(&mut self, mission: MissionDefinition) {
        self.undo         = UndoStack::new(mission);
        self.is_dirty     = false;
        self.graph_dirty  = true;
        self.graph        = None;
    }

    pub fn window_title(&self) -> String {
        let base = format!(
            "GOAP Mission Designer — {}",
            self.mission().name
        );
        if self.is_dirty { format!("{base} *") } else { base }
    }
}