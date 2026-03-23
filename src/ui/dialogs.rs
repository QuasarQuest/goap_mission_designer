use crate::data::{Action, BitField, MissionDefinition};

// ─────────────────────────────────────────────────────────────────────────────
// Field dialog
// ─────────────────────────────────────────────────────────────────────────────

pub struct FieldDialog {
    pub name:        String,
    pub bit_offset:  String,
    pub bit_width:   String,
    pub value_names: Vec<String>,
    pub errors:      Vec<String>,
}

impl Default for FieldDialog {
    fn default() -> Self {
        Self {
            name:        String::new(),
            bit_offset:  "0".into(),
            bit_width:   "1".into(),
            value_names: vec!["OFF".into(), "ON".into()],
            errors:      Vec::new(),
        }
    }
}

impl FieldDialog {
    pub fn from_field(field: &BitField) -> Self {
        Self {
            name:        field.name.clone(),
            bit_offset:  field.bit_offset.to_string(),
            bit_width:   field.bit_width.to_string(),
            value_names: field.value_names.clone(),
            errors:      Vec::new(),
        }
    }

    /// Parsed bit width, clamped to 1–8, defaulting to 1.
    pub fn bit_width(&self) -> u8 {
        self.bit_width.parse::<u8>().unwrap_or(1).max(1).min(8)
    }

    /// Mutable access to the value names vec (for the smart editor in bit_fields_panel).
    pub fn value_names_mut(&mut self) -> &mut Vec<String> {
        &mut self.value_names
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("field_dialog_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.name);
                ui.end_row();

                ui.label("Bit Offset (0-63):");
                ui.text_edit_singleline(&mut self.bit_offset);
                ui.end_row();

                ui.label("Bit Width (1-8):");
                ui.text_edit_singleline(&mut self.bit_width);
                ui.end_row();
            });

        // Show errors
        for e in &self.errors {
            ui.colored_label(egui::Color32::from_rgb(248, 113, 113), e);
        }
    }

    pub fn validate(&mut self, mission: &MissionDefinition) -> Vec<String> {
        let mut errors = Vec::new();

        let offset = match self.bit_offset.parse::<u8>() {
            Ok(v) if v <= 63 => v,
            _ => { errors.push("Bit offset must be 0-63".into()); 0 }
        };
        let width = match self.bit_width.parse::<u8>() {
            Ok(v) if (1..=8).contains(&v) => v,
            _ => { errors.push("Bit width must be 1-8".into()); 1 }
        };

        if offset as u16 + width as u16 > 64 {
            errors.push("Field extends beyond bit 63".into());
        }

        let expected = 1usize << width;
        if self.value_names.len() != expected {
            errors.push(format!("Need exactly {expected} value names for {width}-bit field"));
        }

        if self.name.is_empty() {
            errors.push("Name is required".into());
        }

        // Check overlaps against existing fields
        let names_ref: Vec<&str> = self.value_names.iter().map(String::as_str).collect();
        let candidate = BitField::new(&self.name, offset, width, names_ref);
        for f in &mission.bit_fields {
            if f.overlaps(&candidate) {
                errors.push(format!("Overlaps with field '{}'", f.name));
            }
        }

        self.errors = errors.clone();
        errors
    }

    pub fn value_names_vec(&self) -> Vec<String> {
        self.value_names.iter().map(|s| s.trim().to_uppercase()).collect()
    }

    pub fn build_field(&self) -> BitField {
        let offset    = self.bit_offset.parse().unwrap_or(0);
        let width     = self.bit_width.parse().unwrap_or(1);
        let names_ref: Vec<&str> = self.value_names.iter().map(String::as_str).collect();
        BitField::new(&self.name.to_uppercase(), offset, width, names_ref)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Action dialog
// ─────────────────────────────────────────────────────────────────────────────

pub struct ActionDialog {
    pub name:         String,
    pub description:  String,
    pub cost:         String,
    pub pre_mask:     String,
    pub pre_value:    String,
    pub effect_mask:  String,
    pub effect_value: String,
    pub errors:       Vec<String>,
}

impl Default for ActionDialog {
    fn default() -> Self {
        Self {
            name:         String::new(),
            description:  String::new(),
            cost:         "1".into(),
            pre_mask:     "0x0".into(),
            pre_value:    "0x0".into(),
            effect_mask:  "0x0".into(),
            effect_value: "0x0".into(),
            errors:       Vec::new(),
        }
    }
}

impl ActionDialog {
    pub fn from_action(action: &Action) -> Self {
        Self {
            name:         action.name.clone(),
            description:  action.description.clone(),
            cost:         action.cost.to_string(),
            pre_mask:     format!("0x{:X}", action.pre_mask),
            pre_value:    format!("0x{:X}", action.pre_value),
            effect_mask:  format!("0x{:X}", action.effect_mask),
            effect_value: format!("0x{:X}", action.effect_value),
            errors:       Vec::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("action_dialog_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.name);
                ui.end_row();

                ui.label("Description:");
                ui.text_edit_singleline(&mut self.description);
                ui.end_row();

                ui.label("Cost:");
                ui.text_edit_singleline(&mut self.cost);
                ui.end_row();

                ui.separator(); ui.separator(); ui.end_row();

                ui.label("Pre Mask (hex):");
                ui.text_edit_singleline(&mut self.pre_mask);
                ui.end_row();

                ui.label("Pre Value (hex):");
                ui.text_edit_singleline(&mut self.pre_value);
                ui.end_row();

                ui.separator(); ui.separator(); ui.end_row();

                ui.label("Effect Mask (hex):");
                ui.text_edit_singleline(&mut self.effect_mask);
                ui.end_row();

                ui.label("Effect Value (hex):");
                ui.text_edit_singleline(&mut self.effect_value);
                ui.end_row();
            });

        for e in &self.errors {
            ui.colored_label(egui::Color32::from_rgb(248, 113, 113), e);
        }
    }

    fn parse_hex(s: &str) -> Result<u64, ()> {
        let s = s.trim().trim_start_matches("0x").trim_start_matches("0X");
        u64::from_str_radix(s, 16).map_err(|_| ())
    }

    pub fn validate(&mut self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty()                          { errors.push("Name is required".into()); }
        if self.cost.parse::<u32>().is_err()             { errors.push("Cost must be a positive integer".into()); }
        if Self::parse_hex(&self.pre_mask).is_err()      { errors.push("Invalid pre mask hex".into()); }
        if Self::parse_hex(&self.pre_value).is_err()     { errors.push("Invalid pre value hex".into()); }
        if Self::parse_hex(&self.effect_mask).is_err()   { errors.push("Invalid effect mask hex".into()); }
        if Self::parse_hex(&self.effect_value).is_err()  { errors.push("Invalid effect value hex".into()); }
        self.errors = errors.clone();
        errors
    }

    pub fn build_action(&self) -> Action {
        let mut a      = Action::new(&self.name);
        a.description  = self.description.clone();
        a.cost         = self.cost.parse().unwrap_or(1);
        a.pre_mask     = Self::parse_hex(&self.pre_mask).unwrap_or(0);
        a.pre_value    = Self::parse_hex(&self.pre_value).unwrap_or(0);
        a.effect_mask  = Self::parse_hex(&self.effect_mask).unwrap_or(0);
        a.effect_value = Self::parse_hex(&self.effect_value).unwrap_or(0);
        a
    }
}