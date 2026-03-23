use regex::Regex;
use std::collections::HashMap;

use crate::data::{Action, BitField, MissionDefinition};

pub fn parse_rust(src: &str) -> Result<MissionDefinition, String> {
    let mut mission = MissionDefinition::default();

    parse_header(src, &mut mission)?;
    parse_states(src, &mut mission)?;
    parse_bit_fields(src, &mut mission)?;
    parse_actions(src, &mut mission)?;

    Ok(mission)
}

// ── Header ───────────────────────────────────────────────────────────────────

fn parse_header(src: &str, m: &mut MissionDefinition) -> Result<(), String> {
    let re_name = Regex::new(r"// GOAP Mission:\s*(.+)").unwrap();
    m.name = re_name
        .captures(src)
        .and_then(|c| c.get(1))
        .map(|v| v.as_str().trim().to_string())
        .ok_or("Could not find '// GOAP Mission:' header line")?;

    let re_type = Regex::new(r"// Mission Type\s*:\s*(.+)").unwrap();
    m.mission_type = re_type
        .captures(src)
        .and_then(|c| c.get(1))
        .map(|v| v.as_str().trim().to_string())
        .unwrap_or_else(|| "ELS".to_string());

    let re_var = Regex::new(r"// Variant:\s*(\d+)").unwrap();
    if let Some(caps) = re_var.captures(src) {
        m.variant = caps[1]
            .parse::<u8>()
            .map_err(|_| "Variant is not a valid u8")?;
    }

    Ok(())
}

// ── States ───────────────────────────────────────────────────────────────────

fn parse_states(src: &str, m: &mut MissionDefinition) -> Result<(), String> {
    let re_start = Regex::new(r"pub const \w+_START\s*:\s*WorldState\s*=\s*WorldState\(0x([0-9A-Fa-f]+)\)").unwrap();
    let re_goal  = Regex::new(r"pub const \w+_GOAL\s*:\s*GoalState\s*=\s*GoalState\s*\(0x([0-9A-Fa-f]+)\)").unwrap();

    if let Some(caps) = re_start.captures(src) {
        m.initial_state = u64::from_str_radix(&caps[1], 16)
            .map_err(|_| "Could not parse START state hex value")?;
    }
    if let Some(caps) = re_goal.captures(src) {
        m.goal_state = u64::from_str_radix(&caps[1], 16)
            .map_err(|_| "Could not parse GOAL state hex value")?;
    }

    Ok(())
}

// ── Bit fields ───────────────────────────────────────────────────────────────
//
// Processes longest field name first to avoid prefix ambiguity
// (e.g. REPORT_READY must claim its consts before REPORT does).

fn parse_bit_fields(src: &str, m: &mut MissionDefinition) -> Result<(), String> {
    let re_field = Regex::new(
        r"// (\w[\w ]*?)\s*:\s*(\d+)-bit field\s+bits\s+(\d+)-(\d+)"
    ).unwrap();

    let re_val = Regex::new(
        r"pub const ([A-Z][A-Z0-9_]*)\s*:\s*u64\s*=\s*0x([0-9A-Fa-f]+);"
    ).unwrap();

    let mut all_consts: HashMap<String, u64> = HashMap::new();
    for caps in re_val.captures_iter(src) {
        let cname = caps[1].to_string();
        if cname.ends_with("_MASK")
            || cname.ends_with("_SHIFT")
            || cname.ends_with("_START")
            || cname.ends_with("_GOAL")
            || cname.ends_with("_ACTIONS")
        {
            continue;
        }
        let val = u64::from_str_radix(&caps[2], 16)
            .map_err(|_| format!("Bad hex for const {}", cname))?;
        all_consts.insert(cname, val);
    }

    struct FieldDesc { name: String, bit_width: u8, bit_offset: u8 }
    let mut descs: Vec<FieldDesc> = Vec::new();
    for caps in re_field.captures_iter(src) {
        descs.push(FieldDesc {
            name:       caps[1].trim().to_string(),
            bit_width:  caps[2].parse::<u8>().map_err(|_| "Bad bit_width")?,
            bit_offset: caps[3].parse::<u8>().map_err(|_| "Bad bit_offset")?,
        });
    }

    descs.sort_by(|a, b| b.name.len().cmp(&a.name.len()));

    let mut claimed: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut fields: Vec<BitField> = Vec::new();

    for desc in &descs {
        let num_values  = 1usize << desc.bit_width;
        let field_upper = desc.name.to_uppercase();
        let prefix      = format!("{}_", field_upper);

        let mut indexed: Vec<(usize, String)> = Vec::new();

        for (cname, raw_val) in &all_consts {
            if claimed.contains(cname) { continue; }
            if !cname.starts_with(&prefix) { continue; }
            let value_label = cname[prefix.len()..].to_string();
            let index = (raw_val >> desc.bit_offset) as usize;
            if index < num_values {
                indexed.push((index, value_label));
            }
        }

        indexed.sort_by_key(|(i, _)| *i);

        if indexed.len() != num_values {
            return Err(format!(
                "Field '{}': expected {} value names, found {}",
                desc.name, num_values, indexed.len()
            ));
        }

        for (_, label) in &indexed {
            claimed.insert(format!("{}_{}", field_upper, label));
        }

        let value_names: Vec<String> = indexed.into_iter().map(|(_, n)| n).collect();

        fields.push(BitField {
            id:         uuid::Uuid::new_v4().to_string(),
            name:       desc.name.clone(),
            bit_offset: desc.bit_offset,
            bit_width:  desc.bit_width,
            value_names,
        });
    }

    fields.sort_by_key(|f| f.bit_offset);
    m.bit_fields = fields;

    Ok(())
}

// ── Actions ──────────────────────────────────────────────────────────────────
//
// Handles both formats:
//
// Multi-line (new):                    Single-line (old):
//   Action {                             Action { name: "Foo", pre_mask: 0x01, ... },
//       name         : "Foo",
//       pre_mask     : 0x00000001,
//       ...
//   },

fn parse_actions(src: &str, m: &mut MissionDefinition) -> Result<(), String> {
    // Matches both single-line and multi-line Action { ... } blocks by
    // scanning for the opening brace and collecting until the closing brace.
    let lines: Vec<&str> = src.lines().collect();
    let mut blocks: Vec<(String, String)> = Vec::new(); // (block_text, description)

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("Action {") {
            // Grab optional preceding description comment
            let desc = if i > 0 {
                let prev = lines[i - 1].trim();
                if prev.starts_with("// ") { prev[3..].to_string() } else { String::new() }
            } else {
                String::new()
            };

            // Accumulate lines until braces balance
            let mut block = String::new();
            let mut depth = 0usize;
            let mut j = i;
            while j < lines.len() {
                let l = lines[j];
                block.push_str(l);
                block.push('\n');
                depth += l.chars().filter(|&c| c == '{').count();
                depth = depth.saturating_sub(l.chars().filter(|&c| c == '}').count());
                j += 1;
                if depth == 0 { break; }
            }
            blocks.push((block, desc));
            i = j;
        } else {
            i += 1;
        }
    }

    // Extract each named field from the collapsed block.
    // Handles both  `name: "Foo"`  and  `name : "Foo"`.
    let re_name = Regex::new(r#"name\s*:\s*"([^"]+)""#).unwrap();
    let re_hex  = |field: &str| -> Regex {
        Regex::new(&format!(r"(?i){}\s*:\s*0x([0-9A-Fa-f]+)", regex::escape(field))).unwrap()
    };
    let re_cost = Regex::new(r"cost\s*:\s*(\d+)").unwrap();

    for (block, desc) in &blocks {
        let name = re_name.captures(block)
            .and_then(|c| c.get(1))
            .map(|v| v.as_str().to_string())
            .ok_or("Action block missing 'name' field")?;

        let get_hex = |field: &str| -> Result<u64, String> {
            re_hex(field)
                .captures(block)
                .and_then(|c| c.get(1))
                .ok_or_else(|| format!("Action '{}': missing field '{}'", name, field))
                .and_then(|v| u64::from_str_radix(v.as_str(), 16)
                    .map_err(|_| format!("Action '{}': bad hex in '{}'", name, field)))
        };

        let mut action      = Action::new(&name);
        action.pre_mask     = get_hex("pre_mask")?;
        action.pre_value    = get_hex("pre_value")?;
        action.effect_mask  = get_hex("effect_mask")?;
        action.effect_value = get_hex("effect_value")?;
        action.cost         = re_cost.captures(block)
            .and_then(|c| c.get(1))
            .and_then(|v| v.as_str().parse::<u32>().ok())
            .ok_or_else(|| format!("Action '{}': missing 'cost'", name))?;
        action.description  = desc.clone();
        m.actions.push(action);
    }

    Ok(())
}