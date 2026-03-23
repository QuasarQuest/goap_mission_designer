use std::collections::{HashSet, VecDeque};
use crate::data::MissionDefinition;

pub struct StateNode {
    pub state:      u64,
    pub is_initial: bool,
    pub is_goal:    bool,
}

pub struct StateEdge {
    pub from:        u64,
    pub to:          u64,
    pub action_name: String,
    pub cost:        u32,
}

pub struct StateGraph {
    pub nodes: Vec<StateNode>,
    pub edges: Vec<StateEdge>,
}

impl StateGraph {
    pub fn build(mission: &MissionDefinition) -> Self {
        const MAX_NODES: usize = 60;

        let mut visited: HashSet<u64> = HashSet::new();
        let mut queue: VecDeque<u64> = VecDeque::new();
        let mut edges: Vec<StateEdge> = Vec::new();

        queue.push_back(mission.initial_state);
        visited.insert(mission.initial_state);

        while let Some(state) = queue.pop_front() {
            if visited.len() >= MAX_NODES { break; }

            for action in &mission.actions {
                if action.is_applicable(state) {
                    let next = action.apply(state);
                    edges.push(StateEdge {
                        from: state,
                        to: next,
                        action_name: action.name.clone(),
                        cost: action.cost,
                    });
                    if !visited.contains(&next) {
                        visited.insert(next);
                        queue.push_back(next);
                    }
                }
            }
        }

        let nodes = visited.into_iter()
            .map(|state| {
                let is_goal = if mission.goal_state == 0 {
                    false
                } else {
                    (state & mission.goal_state) == mission.goal_state
                };

                StateNode {
                    state,
                    is_initial: state == mission.initial_state,
                    is_goal,
                }
            })
            .collect();

        Self { nodes, edges }
    }
}