use clap::Parser;

#[derive(Parser, Debug)]
#[clap()]
struct Args {
    #[clap(long, value_parser = args_parser::parse_train)]
    pub train: Option<Vec<Train>>,
}

#[derive(Debug, Clone)]
pub struct Train {
    pub name: String,
    pub cap: u32,
    pub initial_station_name: String,
}

pub mod args_parser {
    use anyhow::{anyhow, bail};
    use itertools::Itertools;

    use crate::Train;

    pub fn parse_train(input: &str) -> Result<Train, anyhow::Error> {
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

fn main() {
    let args = Args::parse();

    println!("{:#?}", args);
}
