use serde::{Deserialize, Serialize};

/// A GOAP action: fires when preconditions match, then applies effects.
/// Both precondition and effect use bitwise mask+value pairs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id:           String,
    pub name:         String,
    pub description:  String,
    pub pre_mask:     u64,   // Bits that must match
    pub pre_value:    u64,   // Required bit pattern
    pub effect_mask:  u64,   // Bits that will change
    pub effect_value: u64,   // New bit pattern
    pub cost:         u32,
}

impl Action {
    pub fn new(name: &str) -> Self {
        Self {
            id:           uuid::Uuid::new_v4().to_string(),
            name:         name.to_string(),
            description:  String::new(),
            pre_mask:     0,
            pre_value:    0,
            effect_mask:  0,
            effect_value: 0,
            cost:         1,
        }
    }

    /// Can this action execute given the current state?
    #[inline(always)]
    pub fn is_applicable(&self, state: u64) -> bool {
        (state & self.pre_mask) == self.pre_value
    }

    /// Apply this action to a state, returning the new state.
    #[inline(always)]
    pub fn apply(&self, state: u64) -> u64 {
        (state & !self.effect_mask) | self.effect_value
    }

    /// Basic sanity checks
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("Action name cannot be empty".into());
        }
        if self.cost == 0 {
            errors.push("Cost must be at least 1".into());
        }
        if self.effect_value & !self.effect_mask != 0 {
            errors.push("Effect value has bits set outside effect mask".into());
        }
        if self.pre_value & !self.pre_mask != 0 {
            errors.push("Pre value has bits set outside pre mask".into());
        }
        errors
    }
}
