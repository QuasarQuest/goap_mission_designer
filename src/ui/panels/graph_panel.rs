use egui::{Color32, FontId, Pos2, Sense, Stroke, Ui, Vec2};
use crate::config::Theme;
use crate::logic::EditorState;
use crate::ui::widgets::icon_btn;
use crate::utils::StateGraph;
use std::collections::HashMap;

const NODE_R:        f32 = 22.0;
const ARROW_SIZE:    f32 = 8.0;
const ARROW_ANGLE:   f32 = 0.5;
const EDGE_OFFSET:   f32 = 6.0;
const LEGEND_OFFSET: f32 = 60.0;
const LEGEND_STEP:   f32 = 18.0;

pub fn show_graph_panel(ui: &mut Ui, editor: &mut EditorState) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            if icon_btn(ui, "🔄", "Rebuild graph") {
                editor.graph_dirty = true;
            }
            let node_count = editor.graph.as_ref().map(|g| g.nodes.len()).unwrap_or(0);
            let edge_count = editor.graph.as_ref().map(|g| g.edges.len()).unwrap_or(0);
            ui.label(format!("{} nodes  •  {} edges  (max 60)", node_count, edge_count));
        });

        ui.separator();
        editor.rebuild_graph_if_needed();

        let available = ui.available_size();
        let (resp, painter) = ui.allocate_painter(available, Sense::hover());
        let origin = resp.rect.min.to_vec2();

        let graph = match &editor.graph {
            Some(g) => g,
            None    => return,
        };

        let positions = compute_tree_layout(graph, resp.rect.size());

        draw_edges(  &painter, &graph, &positions, origin);
        draw_nodes(  &painter, &graph, &positions, origin);
        draw_legend( &painter, &resp);
    });
}

// --- Layout ---

fn compute_tree_layout(graph: &StateGraph, size: Vec2) -> HashMap<u64, Pos2> {
    use std::collections::VecDeque;

    let Some(initial) = graph.nodes.iter().find(|n| n.is_initial).map(|n| n.state) else {
        return HashMap::new();
    };

    // Build adjacency
    let mut adjacency: HashMap<u64, Vec<u64>> = HashMap::new();
    for edge in &graph.edges {
        adjacency.entry(edge.from).or_default().push(edge.to);
    }

    // BFS to assign depth layers
    let mut depth: HashMap<u64, usize> = HashMap::new();
    let mut queue = VecDeque::new();
    depth.insert(initial, 0);
    queue.push_back(initial);

    while let Some(state) = queue.pop_front() {
        let d = depth[&state];
        for &next in adjacency.get(&state).into_iter().flatten() {
            if !depth.contains_key(&next) {
                depth.insert(next, d + 1);
                queue.push_back(next);
            }
        }
    }

    // Unreachable nodes go to last layer
    let max_depth = depth.values().copied().max().unwrap_or(0);
    for node in &graph.nodes {
        depth.entry(node.state).or_insert(max_depth + 1);
    }

    // Group by layer
    let mut layers: HashMap<usize, Vec<u64>> = HashMap::new();
    for node in &graph.nodes {
        layers.entry(depth[&node.state]).or_default().push(node.state);
    }

    let total_layers = layers.len().max(1);
    let layer_h      = size.y / total_layers as f32;

    let mut positions = HashMap::new();
    for (layer_idx, states) in &layers {
        let count  = states.len().max(1);
        let cell_w = size.x / count as f32;
        let y      = layer_h * *layer_idx as f32 + layer_h * 0.5;

        for (i, &state) in states.iter().enumerate() {
            let x = cell_w * i as f32 + cell_w * 0.5;
            positions.insert(state, Pos2::new(x, y));
        }
    }

    positions
}

// --- Drawing ---

fn draw_edges(
    painter:   &egui::Painter,
    graph:     &StateGraph,
    positions: &HashMap<u64, Pos2>,
    origin:    Vec2,
) {
    for edge in &graph.edges {
        let Some(&from) = positions.get(&edge.from) else { continue };
        let Some(&to)   = positions.get(&edge.to)   else { continue };
        if from == to { continue; }

        let from = from + origin;
        let to   = to   + origin;

        let dir  = (to - from).normalized();
        let perp = Vec2::new(-dir.y, dir.x) * EDGE_OFFSET;
        let p0   = from + dir * NODE_R + perp;
        let p1   = to   - dir * NODE_R + perp;

        let stroke = Stroke::new(1.0, Theme::EDGE_COLOR);
        painter.line_segment([p0, p1], stroke);
        painter.line_segment([p1, p1 - rotated(dir * ARROW_SIZE,  ARROW_ANGLE)], stroke);
        painter.line_segment([p1, p1 - rotated(dir * ARROW_SIZE, -ARROW_ANGLE)], stroke);

        let mid = Pos2::new((p0.x + p1.x) / 2.0, (p0.y + p1.y) / 2.0);
        painter.text(mid, egui::Align2::CENTER_CENTER,
                     edge.cost.to_string(),
                     FontId::proportional(9.0), Theme::TEXT_MUTED);
    }
}

fn draw_nodes(
    painter:   &egui::Painter,
    graph:     &StateGraph,
    positions: &HashMap<u64, Pos2>,
    origin:    Vec2,
) {
    for node in &graph.nodes {
        let Some(&pos) = positions.get(&node.state) else { continue };
        let pos = pos + origin;

        let fill = if node.is_initial    { Theme::NODE_INITIAL }
                            else if node.is_goal { Theme::NODE_GOAL }
                            else                 { Theme::NODE_NORMAL };

        painter.circle_filled(pos, NODE_R, fill);
        painter.circle_stroke(pos, NODE_R, Stroke::new(1.5, Theme::NODE_BORDER));
        painter.text(pos, egui::Align2::CENTER_CENTER,
                     format!("{:04X}", node.state & 0xFFFF),
                     FontId::monospace(9.0), Color32::WHITE);
    }
}

fn draw_legend(painter: &egui::Painter, resp: &egui::Response) {
    let lx = resp.rect.left() + 10.0;
    let ly = resp.rect.bottom() - LEGEND_OFFSET;

    for (i, (color, label)) in [
        (Theme::NODE_INITIAL, "Initial"),
        (Theme::NODE_GOAL,    "Goal"),
        (Theme::NODE_NORMAL,  "Intermediate"),
    ].iter().enumerate() {
        let y = ly + i as f32 * LEGEND_STEP;
        painter.circle_filled(Pos2::new(lx + 7.0, y), 6.0, *color);
        painter.text(Pos2::new(lx + 18.0, y), egui::Align2::LEFT_CENTER,
                     *label, FontId::proportional(11.0), Theme::TEXT_SECONDARY);
    }
}

fn rotated(v: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}