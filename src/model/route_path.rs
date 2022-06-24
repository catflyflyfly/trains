use std::collections::HashMap;
use std::ops::Deref;

use anyhow::Result;
use itertools::zip;
use itertools::Itertools;
use pathfinding::prelude::{build_path, dijkstra_all};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoutePath {
    pub station_pair: (Station, Station),
    pub routes: Vec<Route>,
}

impl RoutePath {
    pub fn total_duration_mins(&self) -> u32 {
        self.routes.iter().map(|route| route.duration_mins).sum()
    }
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
                    .find(|route| route.is_from(from) && route.is_to(to))
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

pub type RouteMap = HashMap<(Station, Station), RoutePath>;

impl Network {
    pub fn all_shortest_route_paths_map(&self) -> RouteMap {
        let all_shortest_route_paths = self.all_shortest_route_paths();

        HashMap::from_iter(zip(
            all_shortest_route_paths
                .iter()
                .map(|r| r.station_pair.clone()),
            all_shortest_route_paths.iter().cloned(),
        ))
    }

    fn all_shortest_route_paths(&self) -> Vec<RoutePath> {
        let self_route_paths = self
            .stations
            .iter()
            .map(|station| RoutePath {
                station_pair: (station.clone(), station.clone()),
                routes: vec![Route {
                    name: format!("{}#id", station.name),
                    station_pair: (station.clone(), station.clone()),
                    duration_mins: 0,
                }],
            })
            .collect_vec();

        let out_route_paths = self
            .stations
            .iter()
            .flat_map(|station| self.shortest_route_paths(station))
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
}
