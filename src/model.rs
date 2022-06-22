use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;

use anyhow::{anyhow, bail, Error, Result};
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
