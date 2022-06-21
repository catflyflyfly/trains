use clap::Parser;

pub mod args;

fn main() {
    let network = args::Network::parse();

    println!("{:#?}", network);
}
