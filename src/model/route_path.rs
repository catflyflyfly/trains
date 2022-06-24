use std::collections::HashMap;
use std::ops::Deref;

use anyhow::Result;
use itertools::zip;
use itertools::Itertools;
use pathfinding::prelude::{build_path, dijkstra_all};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoutePath {
    pub from_to: (Station, Station),
    pub routes: Vec<Route>,
}

impl RoutePath {
    pub fn travel_time(&self) -> u32 {
        self.routes.iter().map(|route| route.travel_time).sum()
    }
}

impl TryFrom<(&[Station], &[Route])> for RoutePath {
    type Error = Error;

    fn try_from((stations, all_routes): (&[Station], &[Route])) -> Result<Self, Self::Error> {
        let first = stations.first().unwrap();
        let last = stations.last().unwrap();

        let stations_except_first = stations.iter().skip(1);
        let stations_except_last = stations.iter().take(stations.len() - 1);

        let from_to_chain = zip(stations_except_last, stations_except_first);

        let routes = from_to_chain
            .map(|(from, to)| {
                all_routes
                    .iter()
                    .find(|route| route.is_from(from) && route.is_to(to))
                    .unwrap()
                    .clone()
            })
            .collect_vec();

        Ok(Self {
            from_to: (first.clone(), last.clone()),
            routes,
        })
    }
}

pub type RouteMap = HashMap<(Station, Station), RoutePath>;

impl Network {
    pub fn route_map(&self) -> RouteMap {
        let all_shortest_route_paths = self.shortest_route_paths();

        HashMap::from_iter(zip(
            all_shortest_route_paths
                .iter()
                .map(|route_path| route_path.from_to.clone()),
            all_shortest_route_paths.iter().cloned(),
        ))
    }

    fn shortest_route_paths(&self) -> Vec<RoutePath> {
        let self_route_paths = self
            .stations
            .iter()
            .map(|station| RoutePath {
                from_to: (station.clone(), station.clone()),
                routes: vec![Route {
                    name: format!("{}#id", station.name),
                    from_to: (station.clone(), station.clone()),
                    travel_time: 0,
                }],
            })
            .collect_vec();

        let out_route_paths = self
            .stations
            .iter()
            .flat_map(|station| self.shortest_route_paths_from(station))
            .unique()
            .collect_vec();

        vec![self_route_paths, out_route_paths].concat()
    }

    fn shortest_route_paths_from(&self, from: &Station) -> Vec<RoutePath> {
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
            .filter(|route| route.is_from(station))
            .collect_vec()
    }

    fn reachable_stations(&self, station: &Station) -> Vec<(Station, u32)> {
        let outward_routes = self.routes_from(station);

        let available_stations = outward_routes
            .iter()
            .map(|route| route.to())
            .cloned()
            .collect_vec();

        let travel_time = outward_routes.into_iter().map(|route| route.travel_time);

        zip(available_stations, travel_time).collect_vec()
    }
}
