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
    pub from_to: (String, String),
    pub travel_time: u32,
}

impl Route {
    pub fn reverse(&self) -> Self {
        let Self {
            name,
            from_to: (from, to),
            travel_time,
        } = self.clone();

        Self {
            name,
            from_to: (to, from),
            travel_time,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub weight: u32,
    pub from_to: (String, String),
}

#[derive(Debug, Clone)]
pub struct Train {
    pub name: String,
    pub capacity: u32,
    pub initial_station_name: String,
}

pub mod parser {
    use anyhow::{anyhow, bail, Result};
    use itertools::Itertools;

    use crate::args::{Package, Route, Station, Train};

    pub fn parse_station(input: &str) -> Result<Station> {
        if let [name] = input.split(',').collect_vec()[..] {
            Ok(Station {
                name: name.to_string(),
            })
        } else {
            bail!("[NAME]")
        }
    }

    pub fn parse_route(input: &str) -> Result<Route> {
        if let [name, station1_name, station2_name, travel_time] =
            input.split(',').collect_vec()[..]
        {
            Ok(Route {
                name: name.to_string(),
                from_to: (station1_name.to_string(), station2_name.to_string()),
                travel_time: travel_time.parse().map_err(|error| {
                    anyhow!("parse travel_time `{travel_time}` fail with error `{error}`")
                })?,
            })
        } else {
            bail!("[NAME],[STATION1],[STATION2],[TRAVEL_TIME]")
        }
    }

    pub fn parse_package(input: &str) -> Result<Package> {
        if let [name, weight, start_station_name, destination_station_name] =
            input.split(',').collect_vec()[..]
        {
            Ok(Package {
                name: name.to_string(),
                weight: weight.parse().map_err(|error| {
                    anyhow!("parse weight `{weight}` fail with error `{error}`")
                })?,
                from_to: (
                    start_station_name.to_string(),
                    destination_station_name.to_string(),
                ),
            })
        } else {
            bail!("[NAME],[WEIGHT],[START],[DESTINATION]")
        }
    }

    pub fn parse_train(input: &str) -> Result<Train> {
        if let [name, capacity, initial_station_name] = input.split(',').collect_vec()[..] {
            Ok(Train {
                name: name.to_string(),
                capacity: capacity.parse().map_err(|error| {
                    anyhow!("parse capacity `{capacity}` fail with error `{error}`")
                })?,
                initial_station_name: initial_station_name.to_string(),
            })
        } else {
            bail!("[NAME],[CAPACITY],[INITIAL_STATION_NAME]")
        }
    }
}

// #[cfg(test)]
pub mod case {
    use super::*;

    //   10   10
    // A----B----C
    //
    // T, 5, A
    // P, 5, A -> C
    //
    // Solution: 20     A-A(Pick)-C
    //
    pub fn direct() -> Network {
        Network {
            stations: vec![
                Station { name: "A".into() },
                Station { name: "B".into() },
                Station { name: "C".into() },
            ],
            routes: vec![
                Route {
                    name: "AB".into(),
                    from_to: ("A".into(), "B".into()),
                    travel_time: 10,
                },
                Route {
                    name: "BC".into(),
                    from_to: ("B".into(), "C".into()),
                    travel_time: 10,
                },
            ],
            packages: vec![Package {
                name: "P".into(),
                weight: 5,
                from_to: ("A".into(), "C".into()),
            }],
            trains: vec![Train {
                name: "T".into(),
                capacity: 5,
                initial_station_name: "A".into(),
            }],
        }
    }

    //   10   10
    // /----B----\
    // A         D
    // \----C----/
    //   10   50
    //
    // T, 5, A
    // P, 5, A -> D
    //
    // Solution: 20     A-A(Pick)-D
    pub fn choice() -> Network {
        Network {
            stations: vec![
                Station { name: "A".into() },
                Station { name: "B".into() },
                Station { name: "C".into() },
                Station { name: "D".into() },
            ],
            routes: vec![
                Route {
                    name: "AB".into(),
                    from_to: ("A".into(), "B".into()),
                    travel_time: 10,
                },
                Route {
                    name: "AC".into(),
                    from_to: ("A".into(), "C".into()),
                    travel_time: 10,
                },
                Route {
                    name: "BD".into(),
                    from_to: ("B".into(), "D".into()),
                    travel_time: 10,
                },
                Route {
                    name: "CD".into(),
                    from_to: ("C".into(), "D".into()),
                    travel_time: 50,
                },
            ],
            packages: vec![Package {
                name: "P".into(),
                weight: 5,
                from_to: ("A".into(), "D".into()),
            }],
            trains: vec![Train {
                name: "T".into(),
                capacity: 5,
                initial_station_name: "A".into(),
            }],
        }
    }

    //   10
    // A----B    C
    //
    // T, 5, A
    // P, 5, A -> B
    //
    // Solution: 10     A-A(Pick)-B
    //
    pub fn islands() -> Network {
        Network {
            stations: vec![
                Station { name: "A".into() },
                Station { name: "B".into() },
                Station { name: "C".into() },
            ],
            routes: vec![Route {
                name: "AB".into(),
                from_to: ("A".into(), "B".into()),
                travel_time: 10,
            }],
            packages: vec![Package {
                name: "P".into(),
                weight: 5,
                from_to: ("A".into(), "B".into()),
            }],
            trains: vec![Train {
                name: "T".into(),
                capacity: 5,
                initial_station_name: "A".into(),
            }],
        }
    }

    //   10   50   40   10
    // A----B----C----D----E
    //
    // T, 10, C
    // P1, 5, B -> A
    // P2, 5, D -> E
    //
    // Solution: 160    C-D-E-B-A
    //
    pub fn diverge() -> Network {
        Network {
            stations: vec![
                Station { name: "A".into() },
                Station { name: "B".into() },
                Station { name: "C".into() },
                Station { name: "D".into() },
                Station { name: "E".into() },
            ],
            routes: vec![
                Route {
                    name: "AB".into(),
                    from_to: ("A".into(), "B".into()),
                    travel_time: 10,
                },
                Route {
                    name: "BC".into(),
                    from_to: ("B".into(), "C".into()),
                    travel_time: 50,
                },
                Route {
                    name: "CD".into(),
                    from_to: ("C".into(), "D".into()),
                    travel_time: 40,
                },
                Route {
                    name: "DE".into(),
                    from_to: ("D".into(), "E".into()),
                    travel_time: 10,
                },
            ],
            packages: vec![
                Package {
                    name: "P1".into(),
                    weight: 5,
                    from_to: ("B".into(), "A".into()),
                },
                Package {
                    name: "P2".into(),
                    weight: 5,
                    from_to: ("D".into(), "E".into()),
                },
            ],
            trains: vec![Train {
                name: "T".into(),
                capacity: 10,
                initial_station_name: "C".into(),
            }],
        }
    }

    //   10
    // A----B
    //
    // T, 5, A
    // P1, 5, A -> B
    // P2, 5, A -> B
    //
    // Solution: 30     A-A(Pick)-B-A-B
    //
    pub fn multiple_packages_small_train() -> Network {
        Network {
            stations: vec![Station { name: "A".into() }, Station { name: "B".into() }],
            routes: vec![Route {
                name: "AB".into(),
                from_to: ("A".into(), "B".into()),
                travel_time: 10,
            }],
            packages: vec![
                Package {
                    name: "P1".into(),
                    weight: 5,
                    from_to: ("A".into(), "B".into()),
                },
                Package {
                    name: "P2".into(),
                    weight: 5,
                    from_to: ("A".into(), "B".into()),
                },
            ],
            trains: vec![Train {
                name: "T".into(),
                capacity: 5,
                initial_station_name: "A".into(),
            }],
        }
    }

    //   10
    // A----B
    //
    // T, 10, A
    // P1, 5, A -> B
    // P2, 5, A -> B
    //
    // Solution: 10     A-A(Pick)-B
    //
    pub fn multiple_packages_big_train() -> Network {
        Network {
            stations: vec![Station { name: "A".into() }, Station { name: "B".into() }],
            routes: vec![Route {
                name: "AB".into(),
                from_to: ("A".into(), "B".into()),
                travel_time: 10,
            }],
            packages: vec![
                Package {
                    name: "P1".into(),
                    weight: 5,
                    from_to: ("A".into(), "B".into()),
                },
                Package {
                    name: "P2".into(),
                    weight: 5,
                    from_to: ("A".into(), "B".into()),
                },
            ],
            trains: vec![Train {
                name: "T".into(),
                capacity: 10,
                initial_station_name: "A".into(),
            }],
        }
    }

    //    10
    // A1----B1
    //
    // A2----B2
    //    20
    //
    // T1, 5, A
    // T2, 5, A
    // P1, 5, A1 -> B1
    // P2, 5, A2 -> B2
    //
    // Solution: 20     A1-B1 + A2-B2
    //
    pub fn multiple_packages_islands() -> Network {
        Network {
            stations: vec![
                Station { name: "A1".into() },
                Station { name: "B1".into() },
                Station { name: "A2".into() },
                Station { name: "B2".into() },
            ],
            routes: vec![
                Route {
                    name: "AB1".into(),
                    from_to: ("A1".into(), "B1".into()),
                    travel_time: 10,
                },
                Route {
                    name: "AB2".into(),
                    from_to: ("A2".into(), "B2".into()),
                    travel_time: 20,
                },
            ],
            packages: vec![
                Package {
                    name: "P1".into(),
                    weight: 5,
                    from_to: ("A1".into(), "B1".into()),
                },
                Package {
                    name: "P2".into(),
                    weight: 5,
                    from_to: ("A2".into(), "B2".into()),
                },
            ],
            trains: vec![
                Train {
                    name: "T1".into(),
                    capacity: 5,
                    initial_station_name: "A1".into(),
                },
                Train {
                    name: "T2".into(),
                    capacity: 5,
                    initial_station_name: "A2".into(),
                },
            ],
        }
    }
}
