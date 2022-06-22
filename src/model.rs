use std::ops::Deref;

use anyhow::{anyhow, bail, Error, Result};
use itertools::zip;
use itertools::Itertools;

use crate::args;

#[derive(Debug, Clone)]
pub struct Network {
    pub stations: Vec<Station>,
    pub routes: Vec<Route>,
    pub packages: Vec<Package>,
    pub trains: Vec<Train>,
}

impl Network {
    pub fn shortest_schedules_time(&self) -> u32 {
        self.shortest_schedules()
            .iter()
            .map(|schedule| schedule.end_at())
            .max()
            .unwrap_or(0)
    }

    pub fn shortest_schedules(&self) -> Vec<Schedule> {
        vec![]
    }

    pub fn print_all_shortest_routes(&self) {
        let all = self
            .stations
            .iter()
            .map(|station| {
                (
                    station.clone(),
                    pathfinding::prelude::dijkstra_all(station, |s| self.reachable_stations(s)),
                )
            })
            .collect_vec();

        println!("{:#?}", all)
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

impl Route {
    fn is_involve_station(&self, station: &Station) -> bool {
        let (from, to) = &self.station_pair;

        return from.name == station.name || to.name == station.name;
    }

    fn corresponding_station(&self, station: &Station) -> Result<&Station> {
        match &self.station_pair {
            (from, to) if from.name == station.name => Ok(&to),
            (from, to) if to.name == station.name => Ok(&from),
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

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub weight: u32,
    pub station_pair: (Station, Station),
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

#[derive(Debug, Clone)]
pub struct Train {
    pub name: String,
    pub cap: u32,
    pub initial_station: Station,
}

impl TryFrom<(args::Train, &[Station])> for Train {
    type Error = Error;

    fn try_from((train, stations): (args::Train, &[Station])) -> Result<Self, Self::Error> {
        let args::Train {
            name,
            cap,
            initial_station_name,
        } = train;

        let initial_station = find_station(stations, initial_station_name)?;

        Ok(Self {
            name,
            cap,
            initial_station,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Schedule {
    pub begin_at: u32,
    pub train: Train,
    pub route: Route,
    pub destination: Station,
    pub picked_packages: Vec<Package>,
    pub dropped_packages: Vec<Package>,
}

impl Schedule {
    fn end_at(&self) -> u32 {
        self.begin_at + self.route.duration_mins
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

        assert_eq!(network.shortest_schedules_time(), 30);
    }
}
