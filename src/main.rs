
use clap::Parser;
use datalints::{inner_main, Args};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    inner_main(args)
}

