use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

use itertools::Either;

use super::route_path::RouteMap;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Pick(Package, Station),
    Drop(Package, Station),
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

#[derive(Clone, Eq)]
pub struct Network<'a> {
    pub train_states: Vec<Train<'a>>,
    required_actions: Vec<Action>,
}

impl<'a> Network<'a> {
    pub(super) fn new(network: &'a super::Network) -> Self {
        let route_map = Rc::new(network.all_shortest_route_paths_map());

        Self {
            train_states: network
                .trains
                .iter()
                .map(|train| Train {
                    train,
                    taken_actions: vec![],
                    route_map: route_map.clone(),
                })
                .collect_vec(),
            required_actions: network.actions(),
        }
    }

    pub(super) fn is_success(&self) -> bool {
        self.available_actions().is_empty()
    }

    pub fn instructions(&self) -> Vec<Instruction> {
        self.train_states
            .iter()
            .flat_map(|state| state.instructions())
            .collect_vec()
    }

    pub(super) fn take_available_actions(&self) -> Vec<(Network<'a>, u32)> {
        let untaken_actions = self.untaken_actions();
        let current_total_durations = self.optimal_duration_mins();

        self.clone()
            .train_states
            .iter_mut()
            .enumerate()
            .flat_map(|(index, each_train_state)| {
                let actions = each_train_state.available_actions(&untaken_actions);

                actions
                    .iter()
                    .map(|action| {
                        let mut new_train_states = self.train_states.clone();

                        let mut new_train_state = each_train_state.clone();

                        new_train_state.take_action(action);

                        new_train_states[index] = new_train_state;

                        new_train_states
                    })
                    .map(|train_states| Network {
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
            })
            .collect_vec()
    }

    fn available_actions(&self) -> Vec<Action> {
        let untaken_actions = self.untaken_actions();

        self.train_states
            .iter()
            .flat_map(|train| train.available_actions(&untaken_actions))
            .cloned()
            .unique()
            .collect_vec()
    }

    fn taken_actions(&self) -> Vec<Action> {
        self.train_states
            .iter()
            .flat_map(|state| state.taken_actions.clone())
            .collect()
    }

    fn untaken_actions(&self) -> Vec<Action> {
        let taken_actions = self.taken_actions();

        self.required_actions
            .iter()
            .filter(|action| !taken_actions.contains(action))
            .cloned()
            .collect_vec()
    }

    fn optimal_duration_mins(&self) -> u32 {
        self.train_states
            .iter()
            .map(|state| state.optimal_duration_mins())
            .max()
            .unwrap()
    }
}

impl<'a> Debug for Network<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Network")
            .field("train_states", &self.train_states)
            .finish()
    }
}

impl<'a> PartialEq for Network<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.train_states == other.train_states
    }
}

impl<'a> Hash for Network<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.train_states.hash(state);
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Train<'a> {
    pub train: &'a super::Train,
    pub taken_actions: Vec<Action>,
    route_map: Rc<RouteMap>,
}

impl<'a> PartialEq for Train<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.train == other.train && self.taken_actions == other.taken_actions
    }
}

impl<'a> Hash for Train<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.train.hash(state);
        self.taken_actions.hash(state);
    }
}

impl<'a> Train<'a> {
    fn take_action(&mut self, action: &Action) {
        self.taken_actions.push(action.clone());
    }

    fn available_actions<'b>(&'b self, actions: &'b [Action]) -> Vec<&Action> {
        actions
            .iter()
            .filter(|action| self.can_take(action))
            .collect_vec()
    }

    fn can_take(&self, action: &Action) -> bool {
        match action {
            Action::Pick(package, _) => self.can_pick(package),
            Action::Drop(package, _) => self.can_drop(package),
        }
    }

    fn can_pick(&self, package: &Package) -> bool {
        let is_route_exist = self
            .route_map
            .get(&(self.train.initial_station.clone(), package.from().clone()))
            .is_some();

        let is_enough_room = package.weight + self.current_weight() <= self.train.capacity;

        is_route_exist && is_enough_room
    }

    fn can_drop(&self, package: &Package) -> bool {
        self.taken_actions.iter().any(|action| match action {
            Action::Pick(_, _) => action.package() == package.clone(),
            Action::Drop(_, _) => false,
        })
    }

    fn current_weight(&self) -> u32 {
        self.current_packages()
            .iter()
            .map(|package| package.weight)
            .sum()
    }

    fn current_packages(&self) -> Vec<Package> {
        let (picked_packages, dropped_packages): (Vec<_>, Vec<_>) =
            self.taken_actions.iter().partition_map(|r| match r {
                Action::Pick(p, _) => Either::Left(p),
                Action::Drop(p, _) => Either::Right(p),
            });

        picked_packages
            .into_iter()
            .filter(|package| !dropped_packages.contains(package))
            .cloned()
            .collect()
    }

    fn optimal_duration_mins(&self) -> u32 {
        self.optimal_route_paths()
            .iter()
            .map(|state| state.total_duration_mins())
            .sum()
    }

    fn optimal_route_paths(&self) -> Vec<RoutePath> {
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
            .map(|pair| self.route_map.get(&pair).unwrap().clone())
            .collect_vec()
    }

    fn sub_instructions(
        &self,
        route_path: &RoutePath,
        action: &Action,
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
                    (true, Action::Pick(p, _)) => builder.picked_package(p.clone()),
                    (true, Action::Drop(p, _)) => builder.dropped_package(p.clone()),
                };

                let instruction = builder.build().unwrap();

                begin_at += route.duration_mins;

                instruction
            })
            .collect_vec()
    }

    fn instructions(&self) -> Vec<Instruction> {
        let route_paths = self.optimal_route_paths();
        let taken_actions = &self.taken_actions;

        let mut begin_at = 0;

        zip(route_paths, taken_actions)
            .flat_map(|(route_path, action)| {
                let instructions = self.sub_instructions(&route_path, action, begin_at);

                begin_at += route_path.total_duration_mins();

                instructions
            })
            .collect_vec()
    }
}

// #[cfg(test)]
// pub mod case {
//     use super::*;
//     use crate::model;

//     macro_rules! from_model {
//         ($case_name:ident) => {
//             pub fn $case_name() -> Network {
//                 Network::new(&model::case::$case_name())
//             }
//         };
//     }

//     from_model!(diverge);
//     from_model!(multiple_packages_small_train);
// }

#[cfg(test)]
pub mod test {
    use super::*;

    use crate::model::case;

    #[test]
    fn train_take_action_diverge() {
        let network = case::diverge();

        let mut state = Network::new(&network);

        let possible_actions = &state.required_actions;

        let (pick_p1, drop_p1, pick_p2, drop_p2) = possible_actions.iter().collect_tuple().unwrap();

        assert_state_eq(
            &state,
            vec![],
            vec![
                pick_p1.clone(),
                drop_p1.clone(),
                pick_p2.clone(),
                drop_p2.clone(),
            ],
            vec![pick_p1.clone(), pick_p2.clone()],
            0,
            0,
            false,
            0,
            0,
            0,
        );

        state.train_states[0].take_action(pick_p1);

        assert_state_eq(
            &state,
            vec![pick_p1.clone()],
            vec![drop_p1.clone(), pick_p2.clone(), drop_p2.clone()],
            vec![drop_p1.clone(), pick_p2.clone()],
            50,
            1,
            false,
            5,
            50,
            1,
        );

        state.train_states[0].take_action(drop_p1);

        assert_state_eq(
            &state,
            vec![pick_p1.clone(), drop_p1.clone()],
            vec![pick_p2.clone(), drop_p2.clone()],
            vec![pick_p2.clone()],
            60,
            2,
            false,
            0,
            60,
            2,
        );

        state.train_states[0].take_action(pick_p2);

        assert_state_eq(
            &state,
            vec![pick_p1.clone(), drop_p1.clone(), pick_p2.clone()],
            vec![drop_p2.clone()],
            vec![drop_p2.clone()],
            160,
            5,
            false,
            5,
            160,
            5,
        );

        state.train_states[0].take_action(drop_p2);

        assert_state_eq(
            &state,
            vec![
                pick_p1.clone(),
                drop_p1.clone(),
                pick_p2.clone(),
                drop_p2.clone(),
            ],
            vec![],
            vec![],
            170,
            6,
            true,
            0,
            170,
            6,
        );
    }

    #[test]
    fn train_take_action_multiple_packages_small_train() {
        let network = case::multiple_packages_small_train();

        let mut state = Network::new(&network);
        let possible_actions = &state.required_actions;

        let (pick_p1, drop_p1, pick_p2, drop_p2) = possible_actions.iter().collect_tuple().unwrap();

        assert_state_eq(
            &state,
            vec![],
            vec![
                pick_p1.clone(),
                drop_p1.clone(),
                pick_p2.clone(),
                drop_p2.clone(),
            ],
            vec![pick_p1.clone(), pick_p2.clone()],
            0,
            0,
            false,
            0,
            0,
            0,
        );

        state.train_states[0].take_action(pick_p1);

        assert_state_eq(
            &state,
            vec![pick_p1.clone()],
            vec![drop_p1.clone(), pick_p2.clone(), drop_p2.clone()],
            vec![drop_p1.clone()],
            0,
            1,
            false,
            5,
            0,
            1,
        );

        state.train_states[0].take_action(drop_p1);

        assert_state_eq(
            &state,
            vec![pick_p1.clone(), drop_p1.clone()],
            vec![pick_p2.clone(), drop_p2.clone()],
            vec![pick_p2.clone()],
            10,
            2,
            false,
            0,
            10,
            2,
        );

        state.train_states[0].take_action(pick_p2);

        assert_state_eq(
            &state,
            vec![pick_p1.clone(), drop_p1.clone(), pick_p2.clone()],
            vec![drop_p2.clone()],
            vec![drop_p2.clone()],
            20,
            3,
            false,
            5,
            20,
            3,
        );

        state.train_states[0].take_action(drop_p2);

        assert_state_eq(
            &state,
            vec![
                pick_p1.clone(),
                drop_p1.clone(),
                pick_p2.clone(),
                drop_p2.clone(),
            ],
            vec![],
            vec![],
            30,
            4,
            true,
            0,
            30,
            4,
        );
    }

    #[test]
    fn network_take_available_actions_diverge() {
        let network = case::diverge();

        let state = Network::new(&network);

        let successor_states = state.take_available_actions();
        assert_eq!(successor_states.len(), 2);
        let (state1, _) = successor_states.iter().collect_tuple().unwrap();

        let state = &state1.0;
        let successor_states = state.take_available_actions();
        assert_eq!(successor_states.len(), 2);
        let (state1, _) = successor_states.iter().collect_tuple().unwrap();

        let state = &state1.0;
        let successor_states = state.take_available_actions();
        assert_eq!(successor_states.len(), 1);
        let (state1,) = successor_states.iter().collect_tuple().unwrap();

        let state = &state1.0;
        let successor_states = state.take_available_actions();
        assert_eq!(successor_states.len(), 1);
        let (state1,) = successor_states.iter().collect_tuple().unwrap();

        let state = &state1.0;
        let successor_states = state.take_available_actions();
        assert_eq!(successor_states.len(), 0);
    }

    fn assert_state_eq(
        state: &Network,
        taken_actions: Vec<Action>,
        untaken_actions: Vec<Action>,
        available_actions: Vec<Action>,
        optimal_duration_mins: u32,
        instructions_len: usize,
        is_success: bool,
        train_current_weight: u32,
        train_optimal_duration_mins: u32,
        train_instructions_len: usize,
    ) {
        assert_eq!(state.taken_actions(), taken_actions);
        assert_eq!(state.untaken_actions(), untaken_actions);
        assert_eq!(state.available_actions(), available_actions);
        assert_eq!(state.optimal_duration_mins(), optimal_duration_mins);
        assert_eq!(state.instructions().len(), instructions_len);
        assert_eq!(state.is_success(), is_success);
        assert_eq!(state.train_states[0].current_weight(), train_current_weight);
        assert_eq!(
            state.train_states[0].optimal_duration_mins(),
            train_optimal_duration_mins
        );
        assert_eq!(
            state.train_states[0].instructions().len(),
            train_instructions_len
        );
    }
}
