use clap::Parser;
use datalints::{Args, inner_main};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    inner_main(args)
}
