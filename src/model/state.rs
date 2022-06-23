use std::cmp::Ordering;
use std::collections::HashSet;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Pick(Package, Station),
    Drop(Package, Station),
}

impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Action {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Action::Pick(_, _), Action::Pick(_, _)) | (Action::Drop(_, _), Action::Drop(_, _)) => {
                Ordering::Equal
            }
            (Action::Pick(_, _), Action::Drop(_, _)) => Ordering::Less,
            (Action::Drop(_, _), Action::Pick(_, _)) => Ordering::Greater,
        }
    }
}

impl Action {
    pub(super) fn package(&self) -> Package {
        match self {
            Action::Pick(p, _) => p.clone(),
            Action::Drop(p, _) => p.clone(),
        }
    }

    pub(super) fn station(&self) -> Station {
        match self {
            Action::Pick(_, s) => s.clone(),
            Action::Drop(_, s) => s.clone(),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Network {
    pub train_states: Vec<Train>,
    required_actions: Vec<Action>,
    optimal_route_paths_map: HashMap<(Station, Station), RoutePath>,
}

impl Network {
    pub(super) fn new(network: &super::Network) -> Self {
        state::Network {
            train_states: network
                .trains
                .iter()
                .map(|train| Train {
                    train: train.clone(),
                    taken_actions: vec![],
                })
                .collect_vec(),
            required_actions: network.actions(),
            optimal_route_paths_map: network.all_shortest_route_paths_map(),
        }
    }

    pub(super) fn take_available_actions(&self) -> Vec<(state::Network, u32)> {
        self.available_actions()
            .iter()
            .flat_map(|action| self.take_action(action))
            .collect_vec()
    }

    pub(super) fn is_success(&self) -> bool {
        self.available_actions().is_empty()
    }

    pub fn instructions(&self) -> Vec<Instruction> {
        self.train_states
            .iter()
            .flat_map(|state| state.instructions(&self.optimal_route_paths_map))
            .collect_vec()
    }

    fn take_action(&self, action: &state::Action) -> Vec<(state::Network, u32)> {
        let current_total_durations = self.optimal_duration_mins();

        self.clone()
            .train_states
            .iter_mut()
            .enumerate()
            .map(|(index, train_state)| {
                let mut new_train_states = self.train_states.clone();

                train_state.take_action(action);

                new_train_states[index] = train_state.clone();

                new_train_states
            })
            .map(|train_states| state::Network {
                train_states,
                ..self.clone()
            })
            .map(|new_state| {
                (
                    new_state.clone(),
                    new_state.optimal_duration_mins() - current_total_durations,
                )
            })
            .collect_vec()
    }

    fn available_actions(&self) -> Vec<state::Action> {
        self.required_actions
            .iter()
            .map(|x| x.clone())
            .collect::<HashSet<_>>()
            .difference(&self.taken_actions())
            .group_by(|action| action.package())
            .into_iter()
            .map(|(_, actions)| {
                actions
                    .sorted()
                    .collect_vec()
                    .first()
                    .unwrap()
                    .clone()
                    .clone()
            })
            .collect_vec()
    }

    fn taken_actions(&self) -> HashSet<Action> {
        self.train_states
            .iter()
            .flat_map(|state| state.taken_actions.clone())
            .collect()
    }

    fn optimal_duration_mins(&self) -> u32 {
        self.train_states
            .iter()
            .map(|state| state.optimal_duration_mins(&self.optimal_route_paths_map))
            .max()
            .unwrap()
    }
}

impl std::fmt::Debug for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Network")
            .field("train_states", &self.train_states)
            .finish()
    }
}

impl std::hash::Hash for Network {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.train_states.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Train {
    pub train: super::Train,
    pub taken_actions: Vec<Action>,
}

impl Train {
    fn take_action(&mut self, action: &Action) {
        self.taken_actions.push(action.clone())
    }

    fn optimal_duration_mins(
        &self,
        optimal_route_paths_map: &HashMap<(Station, Station), RoutePath>,
    ) -> u32 {
        self.optimal_route_paths(optimal_route_paths_map)
            .iter()
            .map(|state| state.total_duration_mins())
            .sum()
    }

    fn optimal_route_paths(
        &self,
        optimal_route_paths_map: &HashMap<(Station, Station), RoutePath>,
    ) -> Vec<RoutePath> {
        if self.taken_actions.is_empty() {
            return vec![];
        }

        let froms = vec![
            vec![self.train.initial_station.clone()],
            self.taken_actions
                .iter()
                .take(self.taken_actions.len() - 1)
                .map(|a| a.station())
                .collect_vec(),
        ]
        .concat();

        let tos = self
            .taken_actions
            .iter()
            .take(self.taken_actions.len())
            .map(|a| a.station());

        let pairs = zip(froms, tos);

        pairs
            .map(|pair| optimal_route_paths_map.get(&pair).unwrap().clone())
            .collect_vec()
    }

    fn sub_instructions(
        &self,
        route_path: &RoutePath,
        action: &state::Action,
        begin_at: u32,
    ) -> Vec<Instruction> {
        let route_len = route_path.routes.len();

        let is_last = |index: usize| route_len - 1 == index;

        let mut begin_at = begin_at;

        route_path
            .routes
            .iter()
            .enumerate()
            .map(|(index, route)| {
                let mut builder = InstructionBuilder::default();

                let _ = &builder
                    .begin_at(begin_at)
                    .train(self.train.clone())
                    .route(route.clone());

                let _ = match (is_last(index), action) {
                    (false, _) => &builder,
                    (true, Action::Pick(p, _)) => &builder.picked_package(p.clone()),
                    (true, Action::Drop(p, _)) => &builder.dropped_package(p.clone()),
                };

                let instruction = builder.build().unwrap();

                begin_at += route.duration_mins;

                instruction
            })
            .collect_vec()
    }

    fn instructions(
        &self,
        optimal_route_paths_map: &HashMap<(Station, Station), RoutePath>,
    ) -> Vec<Instruction> {
        let route_paths = self.optimal_route_paths(optimal_route_paths_map);
        let taken_actions = &self.taken_actions;

        let mut begin_at = 0;

        zip(route_paths, taken_actions)
            .flat_map(|(route_path, action)| {
                let instructions = self.sub_instructions(&route_path, &action, begin_at);

                begin_at += route_path.total_duration_mins();

                instructions
            })
            .collect_vec()
    }
}
