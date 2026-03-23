use serde::{Deserialize, Serialize};

/// A multi-bit field in the u64 state word.
/// Width 1 = boolean flag, Width 2 = 4 values, Width 3 = 8 values, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitField {
    pub id:          String,
    pub name:        String,
    pub bit_offset:  u8,
    pub bit_width:   u8,
    pub value_names: Vec<String>,
}

impl BitField {
    pub fn new(name: &str, bit_offset: u8, bit_width: u8, value_names: Vec<&str>) -> Self {
        Self {
            id:          uuid::Uuid::new_v4().to_string(),
            name:        name.to_string(),
            bit_offset,
            bit_width,
            value_names: value_names.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// 2^bit_width – 1
    pub fn max_value(&self) -> u64 {
        (1u64 << self.bit_width) - 1
    }

    /// Mask covering this field's bits
    pub fn mask(&self) -> u64 {
        self.max_value() << self.bit_offset
    }

    /// Inclusive last bit index
    pub fn end_bit(&self) -> u8 {
        self.bit_offset + self.bit_width - 1
    }

    /// Extract this field's value from a full state word
    pub fn get_value(&self, state: u64) -> u64 {
        (state >> self.bit_offset) & self.max_value()
    }

    /// Return a new state word with this field set to `value`
    pub fn set_value(&self, state: u64, value: u64) -> u64 {
        (state & !self.mask()) | ((value << self.bit_offset) & self.mask())
    }

    /// Human-readable name for a numeric value
    pub fn value_name(&self, value: u64) -> &str {
        self.value_names
            .get(value as usize)
            .map(String::as_str)
            .unwrap_or("UNKNOWN")
    }

    /// True if the two fields share any bits
    pub fn overlaps(&self, other: &BitField) -> bool {
        !(self.end_bit() < other.bit_offset || self.bit_offset > other.end_bit())
    }

    /// Validate the field definition
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.name.is_empty() {
            errors.push("Name cannot be empty".into());
        }
        if self.bit_width == 0 || self.bit_width > 8 {
            errors.push(format!("Bit width {} is out of range 1-8", self.bit_width));
        }
        if self.bit_offset > 63 {
            errors.push(format!("Bit offset {} exceeds 63", self.bit_offset));
        }
        if self.bit_offset as u16 + self.bit_width as u16 > 64 {
            errors.push("Field extends beyond bit 63".into());
        }
        let expected = 1usize << self.bit_width;
        if self.value_names.len() != expected {
            errors.push(format!(
                "Expected {} value names for {}-bit field, got {}",
                expected,
                self.bit_width,
                self.value_names.len()
            ));
        }

        errors
    }
}
