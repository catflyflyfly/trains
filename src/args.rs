use clap::Parser;

#[derive(Parser, Debug)]
#[clap()]
pub struct Network {
    #[clap(name = "station", long, value_parser = parser::parse_station)]
    pub stations: Vec<Station>,

    #[clap(name = "route", long, value_parser = parser::parse_route)]
    pub routes: Vec<Route>,

    #[clap(name = "package", long, value_parser = parser::parse_package)]
    pub packages: Vec<Package>,

    #[clap(name = "train", long, value_parser = parser::parse_train)]
    pub trains: Vec<Train>,
}

#[derive(Debug, Clone)]
pub struct Station {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub name: String,
    pub station1_name: String,
    pub station2_name: String,
    pub duration_mins: u32,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub weight: u32,
    pub start_station_name: String,
    pub destination_station_name: String,
}

#[derive(Debug, Clone)]
pub struct Train {
    pub name: String,
    pub cap: u32,
    pub initial_station_name: String,
}

pub mod parser {
    use anyhow::{anyhow, bail, Result};
    use itertools::Itertools;

    use crate::args::{Package, Route, Station, Train};

    pub fn parse_station(input: &str) -> Result<Station> {
        if let [name] = input.split(",").collect_vec()[..] {
            Ok(Station {
                name: name.to_string(),
            })
        } else {
            bail!("[NAME]")
        }
    }

    pub fn parse_route(input: &str) -> Result<Route> {
        if let [name, station1_name, station2_name, duration_mins] =
            input.split(",").collect_vec()[..]
        {
            Ok(Route {
                name: name.to_string(),
                station1_name: station1_name.to_string(),
                station2_name: station2_name.to_string(),
                duration_mins: duration_mins.parse().map_err(|error| {
                    anyhow!("parse duration_mins `{duration_mins}` fail with error `{error}`")
                })?,
            })
        } else {
            bail!("[NAME],[STATION1],[STATION2],[DURATION_MINS]")
        }
    }

    pub fn parse_package(input: &str) -> Result<Package> {
        if let [name, weight, start_station_name, destination_station_name] =
            input.split(",").collect_vec()[..]
        {
            Ok(Package {
                name: name.to_string(),
                weight: weight.parse().map_err(|error| {
                    anyhow!("parse weight `{weight}` fail with error `{error}`")
                })?,
                start_station_name: start_station_name.to_string(),
                destination_station_name: destination_station_name.to_string(),
            })
        } else {
            bail!("[NAME],[WEIGHT],[START],[DESTINATION]")
        }
    }

    pub fn parse_train(input: &str) -> Result<Train> {
        if let [name, capacity, initial_station_name] = input.split(",").collect_vec()[..] {
            Ok(Train {
                name: name.to_string(),
                cap: capacity.parse().map_err(|error| {
                    anyhow!("parse capacity `{capacity}` fail with error `{error}`")
                })?,
                initial_station_name: initial_station_name.to_string(),
            })
        } else {
            bail!("[NAME],[CAPACITY],[INITIAL_STATION_NAME]")
        }
    }
}
