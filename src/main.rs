#[macro_use]
extern crate derive_builder;

use anyhow::Result;
use clap::Parser;

pub mod args;
pub mod model;

fn main() -> Result<()> {
    let network_args = args::Network::parse();
    let network = model::Network::try_from(network_args)?;

    network.print_optimal_instructions();
    println!("{:#?}", network.optimal_time_mins());

    Ok(())
}
