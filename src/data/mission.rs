use serde::{Deserialize, Serialize};
use crate::data::{Action, BitField};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionDefinition {
    pub name:          String,
    pub mission_type:  String,
    pub variant:       u8,   // 1-255
    pub bit_fields:    Vec<BitField>,
    pub initial_state: u64,
    pub goal_state:    u64,
    pub actions:       Vec<Action>,
}

impl Default for MissionDefinition {
    fn default() -> Self {
        Self {
            name:          "New Mission".into(),
            mission_type:  "ELS".into(),
            variant:       1,
            bit_fields:    Vec::new(),
            initial_state: 0,
            goal_state:    0,
            actions:       Vec::new(),
        }
    }
}

impl MissionDefinition {


    /// Sorted fields (lowest offset first)
    pub fn sorted_fields(&self) -> Vec<&BitField> {
        let mut sorted: Vec<&BitField> = self.bit_fields.iter().collect();
        sorted.sort_by_key(|f| f.bit_offset);
        sorted
    }

    /// Return any overlapping field pairs
    pub fn overlapping_fields(&self) -> Vec<(&BitField, &BitField)> {
        let mut pairs = Vec::new();
        for (i, a) in self.bit_fields.iter().enumerate() {
            for b in self.bit_fields.iter().skip(i + 1) {
                if a.overlaps(b) {
                    pairs.push((a, b));
                }
            }
        }
        pairs
    }

    /// Find field by id
    pub fn field_by_id(&self, id: &str) -> Option<&BitField> {
        self.bit_fields.iter().find(|f| f.id == id)
    }

    pub fn field_by_id_mut(&mut self, id: &str) -> Option<&mut BitField> {
        self.bit_fields.iter_mut().find(|f| f.id == id)
    }

    pub fn action_by_id_mut(&mut self, id: &str) -> Option<&mut Action> {
        self.actions.iter_mut().find(|a| a.id == id)
    }

    /// Bits used by defined fields (out of 48 available)
    pub fn bits_used(&self) -> u8 {
        self.bit_fields.iter().map(|f| f.bit_width).sum()
    }

    /// Validate whole mission
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.name.is_empty() {
            errors.push("Mission name is empty".into());
        }
        for field in &self.bit_fields {
            for e in field.validate() {
                errors.push(format!("[{}] {}", field.name, e));
            }
        }
        for (a, b) in self.overlapping_fields() {
            errors.push(format!("Fields '{}' and '{}' overlap", a.name, b.name));
        }
        for action in &self.actions {
            for e in action.validate() {
                errors.push(format!("[{}] {}", action.name, e));
            }
        }

        errors
    }

    /// Serialize to pretty JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
