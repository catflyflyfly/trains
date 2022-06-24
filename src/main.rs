#[macro_use]
extern crate derive_builder;

use anyhow::Result;
use clap::Parser;

pub mod args;
pub mod model;

fn main() -> Result<()> {
    model::Network::try_from(args::Network::parse())?
        .optimal_itinerary()
        .print_output();

    Ok(())
}
