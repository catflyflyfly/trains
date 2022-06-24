use std::hash::Hash;
use std::ops::Deref;

use anyhow::{anyhow, Error, Result};
use itertools::zip;
use itertools::Itertools;
use pathfinding::prelude::dijkstra;

use crate::args;

pub mod route_path;
pub mod state;

pub use route_path::RoutePath;

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

    pub fn print_optimal_instructions(&self) {
        self.optimal_instructions()
            .iter()
            .for_each(|i| println!("{i}"));
    }

    fn actions(&self) -> Vec<state::Action> {
        self.packages
            .iter()
            .flat_map(|package| package.actions())
            .collect_vec()
    }

    fn solve(&self) -> (Vec<state::Network>, u32) {
        dijkstra(
            &state::Network::new(self),
            |state| state.take_available_actions(),
            |state| state.is_success(),
        )
        .unwrap()
    }
}

impl TryFrom<args::Network> for Network {
    type Error = Error;

    fn try_from(input: args::Network) -> Result<Self, Self::Error> {
        let stations = input.stations.into_iter().map(Station::from).collect_vec();

        let reversed_routes = input
            .routes
            .iter()
            .map(|route| Route::try_from((route.reverse(), stations.deref())))
            .collect::<Result<Vec<_>>>()?;

        let routes = input
            .routes
            .into_iter()
            .map(|route| Route::try_from((route, stations.deref())))
            .collect::<Result<Vec<_>>>()?;

        let routes = vec![reversed_routes, routes].concat();

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
    pub from_to: (Station, Station),
    pub travel_time: u32,
}

impl PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Route {}

impl Hash for Route {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Route {
    pub fn from(&self) -> &Station {
        &self.from_to.0
    }

    pub fn to(&self) -> &Station {
        &self.from_to.1
    }

    pub fn is_from(&self, station: &Station) -> bool {
        self.from().name == station.name
    }

    pub fn is_to(&self, station: &Station) -> bool {
        self.to().name == station.name
    }
}

impl TryFrom<(args::Route, &[Station])> for Route {
    type Error = Error;

    fn try_from((route, stations): (args::Route, &[Station])) -> Result<Self, Self::Error> {
        let args::Route {
            name,
            from_to: (from, to),
            travel_time,
        } = route;

        let from_to = (find_station(stations, from)?, find_station(stations, to)?);

        Ok(Self {
            name,
            from_to,
            travel_time,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Package {
    pub name: String,
    pub weight: u32,
    pub from_to: (Station, Station),
}

impl Package {
    pub fn from(&self) -> &Station {
        &self.from_to.0
    }

    pub fn to(&self) -> &Station {
        &self.from_to.1
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
            from_to: (from, to),
        } = package;

        let from_to = (find_station(stations, from)?, find_station(stations, to)?);

        Ok(Self {
            name,
            weight,
            from_to,
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

fn find_station(stations: &[Station], station_name: String) -> Result<Station> {
    Ok(stations
        .iter()
        .find(|station| station.name == station_name)
        .ok_or_else(|| anyhow!("station not found: {station_name}"))?
        .clone())
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let picked_package_name = format!(
            "[{}]",
            self.picked_package
                .as_ref()
                .map(|package| package.name.clone())
                .unwrap_or_else(|| "".to_string())
        );

        let dropped_package_name = format!(
            "[{}]",
            self.dropped_package
                .as_ref()
                .map(|package| package.name.clone())
                .unwrap_or_else(|| "".to_string())
        );
        let val = vec![
            ("W", self.begin_at.to_string()),
            ("T", self.train.name.clone()),
            ("N1", self.route.from().name.clone()),
            ("P1", picked_package_name),
            ("N2", self.route.to().name.clone()),
            ("N2", dropped_package_name),
        ];

        let mut str = "";
        for (field, value) in val {
            fmt.write_str(str)?;
            fmt.write_str(field)?;
            fmt.write_str(" = ")?;
            fmt.write_str(&value)?;
            str = ", ";
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod case {
    use super::*;
    use crate::args;

    macro_rules! from_args {
        ($case_name:ident) => {
            pub fn $case_name() -> Network {
                Network::try_from(args::case::$case_name()).unwrap()
            }
        };
    }

    from_args!(direct);
    from_args!(choice);
    from_args!(islands);
    from_args!(diverge);
    from_args!(multiple_packages_small_train);
    from_args!(multiple_packages_big_train);
    from_args!(multiple_packages_islands);
}

#[cfg(test)]
pub mod test {
    use super::*;

    macro_rules! test_solve_train_network {
        ($case_name:ident, $expected_time:literal) => {
            #[test]
            fn $case_name() {
                let network = case::$case_name();

                println!("{:#?}", network.optimal_instructions());
                println!("{:#?}", network.optimal_time_mins());

                assert_eq!(network.optimal_time_mins(), $expected_time);
            }
        };
    }

    test_solve_train_network!(direct, 20);
    test_solve_train_network!(choice, 20);
    test_solve_train_network!(islands, 10);
    test_solve_train_network!(diverge, 160);
    test_solve_train_network!(multiple_packages_small_train, 30);
    test_solve_train_network!(multiple_packages_big_train, 10);
    test_solve_train_network!(multiple_packages_islands, 20);
}
