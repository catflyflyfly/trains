use std::ops::Deref;

use anyhow::{anyhow, Error, Result};
use itertools::Itertools;

use crate::args;

#[derive(Debug, Clone)]
pub struct Network {
    pub stations: Vec<Station>,
    pub routes: Vec<Route>,
    pub packages: Vec<Package>,
    pub trains: Vec<Train>,
}

impl TryFrom<args::Network> for Network {
    type Error = Error;

    fn try_from(input: args::Network) -> Result<Self, Self::Error> {
        let stations = input.stations.into_iter().map(Station::from).collect_vec();

        let routes = input
            .routes
            .into_iter()
            .map(|route| Route::try_from((route, stations.deref())))
            .collect::<Result<Vec<_>, _>>()?;

        let packages = input
            .packages
            .into_iter()
            .map(|package| Package::try_from((package, stations.deref())))
            .collect::<Result<Vec<_>, _>>()?;

        let trains = input
            .trains
            .into_iter()
            .map(|train| Train::try_from((train, stations.deref())))
            .collect::<Result<Vec<_>, _>>()?;

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
    pub station1: Station,
    pub station2: Station,
    pub duration_mins: u32,
}

impl TryFrom<(args::Route, &[Station])> for Route {
    type Error = Error;

    fn try_from((route, stations): (args::Route, &[Station])) -> Result<Self, Self::Error> {
        let args::Route {
            name,
            station1_name,
            station2_name,
            duration_mins,
        } = route;

        let station1 = find_station(stations, station1_name)?;
        let station2 = find_station(stations, station2_name)?;

        Ok(Self {
            name,
            station1,
            station2,
            duration_mins,
        })
    }
}

fn find_station(stations: &[Station], station_name: String) -> Result<Station> {
    Ok(stations
        .iter()
        .find(|station| station.name == station_name)
        .ok_or_else(|| anyhow!("station not found: {station_name}"))?
        .clone())
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub weight: u32,
    pub start_station: Station,
    pub destination_station: Station,
}

impl TryFrom<(args::Package, &[Station])> for Package {
    type Error = Error;

    fn try_from((package, stations): (args::Package, &[Station])) -> Result<Self, Self::Error> {
        let args::Package {
            name,
            weight,
            start_station_name,
            destination_station_name,
        } = package;

        let start_station = find_station(stations, start_station_name)?;
        let destination_station = find_station(stations, destination_station_name)?;

        Ok(Self {
            name,
            weight,
            start_station,
            destination_station,
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
