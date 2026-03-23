use crate::data::MissionDefinition;

/// Simple snapshot-based undo/redo.
/// Keeps up to MAX_HISTORY full mission snapshots.
pub struct UndoStack {
    history: Vec<MissionDefinition>,
    cursor:  usize, // points to current state
}

const MAX_HISTORY: usize = 64;

impl UndoStack {
    pub fn new(initial: MissionDefinition) -> Self {
        Self {
            history: vec![initial],
            cursor:  0,
        }
    }

    /// Push a new snapshot (truncates any redo future)
    pub fn push(&mut self, state: MissionDefinition) {
        // Drop redo states
        self.history.truncate(self.cursor + 1);

        // Trim oldest if at capacity
        if self.history.len() >= MAX_HISTORY {
            self.history.remove(0);
        } else {
            self.cursor += 1;
        }

        self.history.push(state);
    }

    /// Current mission (read-only view)
    pub fn current(&self) -> &MissionDefinition {
        &self.history[self.cursor]
    }

    pub fn can_undo(&self) -> bool { self.cursor > 0 }
    pub fn can_redo(&self) -> bool { self.cursor + 1 < self.history.len() }

    /// Returns the previous snapshot
    pub fn undo(&mut self) -> &MissionDefinition {
        if self.can_undo() { self.cursor -= 1; }
        &self.history[self.cursor]
    }

    /// Returns the next snapshot
    pub fn redo(&mut self) -> &MissionDefinition {
        if self.can_redo() { self.cursor += 1; }
        &self.history[self.cursor]
    }
}
