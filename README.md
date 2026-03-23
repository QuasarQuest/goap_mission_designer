# GOAP Mission Designer

A desktop tool for designing, visualising, and exporting GOAP (Goal-Oriented Action Planning) missions for autonomous UAV operations. Built with Rust and egui.

## What it does

GOAP models an agent's world as a single 64-bit integer. Each bit (or group of bits) represents a named state variable — a sensor being active, a data collection phase, an engagement state. Actions declare which bit pattern they require to fire (*preconditions*) and which bits they flip when they do (*effects*). A planner then finds the cheapest sequence of actions that moves the system from an initial state to a goal state.

This tool lets you define those fields and actions visually, inspect the resulting state graph, and export the whole thing as ready-to-compile Rust code.

---

## Concepts

**Bit Fields** encode state variables into the 64-bit world state word. A 1-bit field is a boolean flag; a 3-bit field can hold 8 named values. Fields are packed into bits 0–63.

**Actions** are defined by two mask/value pairs:
- *Precondition* — `(state & pre_mask) == pre_value` must hold for the action to be applicable.
- *Effect* — `state = (state & !effect_mask) | effect_value` is the resulting state after the action fires.
- *Cost* — used by the A* planner to find the optimal sequence.

**Initial / Goal State** are the starting and target world states, expressed as the named field values you select in the States panel.

---

## Interface

The tool has three tabs.

**Editor** is split into three columns. The left panel manages bit fields — add, edit, remove, and reorder them. A colour-coded bit layout bar at the top of the screen shows exactly which bits each field occupies. The centre panel lets you set the initial and goal states by selecting named values from combo boxes for each field. The right panel manages actions, showing their precondition and effect masks decoded against the defined fields.

**State Graph** runs a bounded BFS from the initial state (capped at 60 nodes) and renders the reachable state space as a directed graph. Nodes are colour-coded: green for initial, amber for goal, grey for intermediate. Edge labels show action cost.

**Rust Code** generates a self-contained `.rs` module with all fields, actions, initial state, and goal state as typed constants, ready to drop into your planner crate.

---

## File formats

**JSON** missions are saved as plain JSON. Open and save via File or Ctrl+S. The format is stable and version-controlled — suitable for committing alongside mission source code.

**Rust** each mission can be exported as a .rs file via File > Export Rust. The output is a self-contained module with typed constants for every bit field, value name, and action — no runtime dependency on the designer. Intended to be dropped directly into a planner crate.

---

## Building

Requires Rust and a native windowing backend (X11 or Wayland on Linux).

```bash
cargo build --release
cargo run --release
```

Dependencies: `eframe`, `egui`, `egui_extras`, `serde_json`, `uuid`, `rfd`.

---

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| Ctrl+S | Save |
| Ctrl+Z | Undo |
| Ctrl+Y / Ctrl+Shift+Z | Redo |

Undo/redo covers all structural changes to fields, actions, and state values. The toolbar shows "Modified" when there are unsaved changes.
