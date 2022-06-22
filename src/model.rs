use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;

use anyhow::{anyhow, bail, Error, Result};
use itertools::zip;
use itertools::Itertools;
use pathfinding::prelude::{build_path, dijkstra, dijkstra_all};

use crate::args;

#[derive(Debug, Clone)]
pub struct Network {
    pub stations: Vec<Station>,
    pub routes: Vec<Route>,
    pub packages: Vec<Package>,
    pub trains: Vec<Train>,
}

impl Network {
    pub fn optimal_time_mins(&self) -> u32 {
        self.solve().1
    }

    pub fn optimal_instructions(&self) -> Vec<Instruction> {
        self.solve().0.last().unwrap().instructions()
    }

    pub fn all_shortest_route_paths_map(&self) -> HashMap<(Station, Station), RoutePath> {
        let all_shortest_route_paths = self.all_shortest_route_paths();

        HashMap::from_iter(zip(
            all_shortest_route_paths
                .iter()
                .map(|r| r.station_pair.clone()),
            all_shortest_route_paths.iter().map(|r| r.clone()),
        ))
    }

    pub fn all_shortest_route_paths(&self) -> Vec<RoutePath> {
        let self_route_paths = self
            .stations
            .iter()
            .map(|station| RoutePath {
                station_pair: (station.clone(), station.clone()),
                routes: vec![Route {
                    name: format!("iden#{}", station.name),
                    station_pair: (station.clone(), station.clone()),
                    duration_mins: 0,
                }],
            })
            .collect_vec();

        let out_route_paths = self
            .stations
            .iter()
            .map(|station| self.shortest_route_paths(station))
            .flatten()
            .unique()
            .collect_vec();

        vec![self_route_paths, out_route_paths].concat()
    }

    fn shortest_route_paths(&self, from: &Station) -> Vec<RoutePath> {
        let reachable_stations = dijkstra_all(from, |to| self.reachable_stations(to));

        reachable_stations
            .keys()
            .map(|to| build_path(to, &reachable_stations))
            .map(|station_seq| {
                RoutePath::try_from((station_seq.deref(), self.routes.deref())).unwrap()
            })
            .collect_vec()
    }

    fn routes_from(&self, station: &Station) -> Vec<&Route> {
        self.routes
            .iter()
            .filter(|route| route.is_involve_station(station))
            .collect_vec()
    }

    fn reachable_stations(&self, station: &Station) -> Vec<(Station, u32)> {
        let involved_routes = self.routes_from(station);

        let available_stations = involved_routes
            .iter()
            .map(|route| route.corresponding_station(station))
            .collect::<Result<Vec<_>>>()
            .unwrap()
            .into_iter()
            .map(|st| st.to_owned())
            .collect_vec();

        let duration_mins = involved_routes.into_iter().map(|route| route.duration_mins);

        zip(available_stations, duration_mins).collect_vec()
    }

    fn possible_actions(&self) -> HashSet<state::Action> {
        self.packages
            .iter()
            .flat_map(|package| package.actions())
            .collect::<HashSet<_>>()
    }

    fn solve(&self) -> (Vec<state::Network>, u32) {
        dijkstra(
            &state::Network::new(self),
            |state| state.successor_states(),
            |state| state.is_success(),
        )
        .unwrap()
    }
}

impl TryFrom<args::Network> for Network {
    type Error = Error;

    fn try_from(input: args::Network) -> Result<Self, Self::Error> {
        let stations = input.stations.into_iter().map(Station::from).collect_vec();

        let routes = input
            .routes
            .into_iter()
            .map(|route| Route::try_from((route, stations.deref())))
            .collect::<Result<Vec<_>>>()?;

        let packages = input
            .packages
            .into_iter()
            .map(|package| Package::try_from((package, stations.deref())))
            .collect::<Result<Vec<_>>>()?;

        let trains = input
            .trains
            .into_iter()
            .map(|train| Train::try_from((train, stations.deref())))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            stations,
            routes,
            packages,
            trains,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Station {
    pub name: String,
}

impl From<args::Station> for Station {
    fn from(station: args::Station) -> Self {
        Self { name: station.name }
    }
}

#[derive(Debug, Clone)]
pub struct Route {
    pub name: String,
    pub station_pair: (Station, Station),
    pub duration_mins: u32,
}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Route {}

impl std::hash::Hash for Route {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Route {
    pub fn from(&self) -> &Station {
        &self.station_pair.0
    }

    pub fn to(&self) -> &Station {
        &self.station_pair.1
    }

    fn is_involve_station(&self, station: &Station) -> bool {
        let (from, to) = &self.station_pair;

        return from == station || to == station;
    }

    fn corresponding_station(&self, station: &Station) -> Result<&Station> {
        match &self.station_pair {
            (from, to) if from == station => Ok(&to),
            (from, to) if to == station => Ok(&from),
            _ => bail!(
                "this station {} is not the part of this route",
                station.name
            ),
        }
    }
}

impl TryFrom<(args::Route, &[Station])> for Route {
    type Error = Error;

    fn try_from((route, stations): (args::Route, &[Station])) -> Result<Self, Self::Error> {
        let args::Route {
            name,
            station_pair_name: (from_name, to_name),
            duration_mins,
        } = route;

        let station_pair = (
            find_station(stations, from_name)?,
            find_station(stations, to_name)?,
        );

        Ok(Self {
            name,
            station_pair,
            duration_mins,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Package {
    pub name: String,
    pub weight: u32,
    pub station_pair: (Station, Station),
}

impl Package {
    pub fn from(&self) -> &Station {
        &self.station_pair.0
    }

    pub fn to(&self) -> &Station {
        &self.station_pair.1
    }

    fn actions(&self) -> Vec<state::Action> {
        vec![
            state::Action::Pick(self.clone(), self.from().clone()),
            state::Action::Drop(self.clone(), self.to().clone()),
        ]
    }
}

impl TryFrom<(args::Package, &[Station])> for Package {
    type Error = Error;

    fn try_from((package, stations): (args::Package, &[Station])) -> Result<Self, Self::Error> {
        let args::Package {
            name,
            weight,
            station_pair_name: (from_name, to_name),
        } = package;

        let station_pair = (
            find_station(stations, from_name)?,
            find_station(stations, to_name)?,
        );

        Ok(Self {
            name,
            weight,
            station_pair,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Train {
    pub name: String,
    pub capacity: u32,
    pub initial_station: Station,
}

impl TryFrom<(args::Train, &[Station])> for Train {
    type Error = Error;

    fn try_from((train, stations): (args::Train, &[Station])) -> Result<Self, Self::Error> {
        let args::Train {
            name,
            capacity,
            initial_station_name,
        } = train;

        let initial_station = find_station(stations, initial_station_name)?;

        Ok(Self {
            name,
            capacity,
            initial_station,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoutePath {
    pub station_pair: (Station, Station),
    pub routes: Vec<Route>,
}

impl RoutePath {
    fn total_duration_mins(&self) -> u32 {
        self.routes.iter().map(|route| route.duration_mins).sum()
    }
}

#[derive(Debug, Clone, Builder)]
pub struct Instruction {
    pub begin_at: u32,
    pub train: Train,
    pub route: Route,
    #[builder(setter(into, strip_option), default)]
    pub picked_package: Option<Package>,
    #[builder(setter(into, strip_option), default)]
    pub dropped_package: Option<Package>,
}

impl TryFrom<(&[Station], &[Route])> for RoutePath {
    type Error = Error;

    fn try_from((stations, all_routes): (&[Station], &[Route])) -> Result<Self, Self::Error> {
        let first = stations.first().unwrap();
        let last = stations.last().unwrap();

        let stations_except_first = stations.iter().skip(1);
        let stations_except_last = stations.iter().take(stations.len() - 1);

        let station_pairs_chain = zip(stations_except_last, stations_except_first);

        let routes = station_pairs_chain
            .map(|(from, to)| {
                all_routes
                    .iter()
                    .find(|route| route.is_involve_station(from) && route.is_involve_station(to))
                    .unwrap()
                    .clone()
            })
            .collect_vec();

        Ok(Self {
            station_pair: (first.clone(), last.clone()),
            routes,
        })
    }
}

pub mod state {
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
                (Action::Pick(_, _), Action::Pick(_, _))
                | (Action::Drop(_, _), Action::Drop(_, _)) => Ordering::Equal,
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
        possible_actions: HashSet<Action>,
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
                possible_actions: network.possible_actions(),
                optimal_route_paths_map: network.all_shortest_route_paths_map(),
            }
        }

        pub(super) fn successor_states(&self) -> Vec<(state::Network, u32)> {
            self.available_actions()
                .iter()
                .flat_map(|action| self.action_successor_states(action))
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

        fn action_successor_states(&self, action: &state::Action) -> Vec<(state::Network, u32)> {
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
            self.possible_actions
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
}

fn find_station(stations: &[Station], station_name: String) -> Result<Station> {
    Ok(stations
        .iter()
        .find(|station| station.name == station_name)
        .ok_or_else(|| anyhow!("station not found: {station_name}"))?
        .clone())
}

#[cfg(test)]
pub mod test {
    use super::*;

    use crate::args::case;

    #[test]
    fn simple_choice() {
        let args = case::simple_choice();

        let network = Network::try_from(args).unwrap();

        println!("{:#?}", network.optimal_instructions());
        println!("{:#?}", network.optimal_time_mins());

        assert_eq!(network.optimal_time_mins(), 30);
    }
}
