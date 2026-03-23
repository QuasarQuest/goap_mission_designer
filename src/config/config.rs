pub struct FieldsPanelConfig {
    pub total_bits: u8,
    pub heading:    &'static str,
    pub scroll_id:  &'static str,
}

impl Default for FieldsPanelConfig {
    fn default() -> Self {
        Self {
            total_bits: 48,
            heading:    "⬛ Bit Fields",
            scroll_id:  "fields_scroll",
        }
    }
}