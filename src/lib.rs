use crate::reader::DatasetReaderBuilder;
use clap::Parser;
use std::path::PathBuf;

mod document;
mod lints;
mod pipeline;
mod reader;

#[cfg(test)]
mod tests_common;

#[derive(Debug, Parser)]
pub struct Args {
    pub path: PathBuf,
}

pub fn inner_main(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let reader_builder = DatasetReaderBuilder::new(args.path.as_path());
    let reader = reader_builder.build();
    reader.into_iter().for_each(|rb| println!("{:?}", rb));
    Ok(())
}
