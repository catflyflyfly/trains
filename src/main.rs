use anyhow::Result;
use clap::Parser;

pub mod args;
pub mod model;

fn main() -> Result<()> {
    let network_args = args::Network::parse();
    let network = model::Network::try_from(network_args)?;

    println!("{:#?}", network);

    Ok(())
}
