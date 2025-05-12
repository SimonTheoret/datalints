use std::{
    ffi::{OsString, OsStr},
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use log;
use thiserror::{self, Error};

use arrow::json::Reader;
use arrow_csv::reader::infer_schema_from_files;
use arrow_json::reader::{ReaderBuilder, infer_json_schema_from_seekable};

#[derive(derive_more::FromStr)]
pub enum DatasetFormat {
    JSONL,
    CSV,
    TXT,
    Parquet
}

#[derive(Error, Debug )]
#[error("Error while parsing the file extension")]
pub struct ExtensionParsingError(OsString);

impl TryFrom<&OsStr> for DatasetFormat {
    type Error = ExtensionParsingError;
    fn try_from(value: &OsStr) -> Result<Self,Self::Error > {

        match &value.to_owned().to_ascii_lowercase().to_str() {
            Some(s) => {
                match *s {
                    "jsonl" | "json" => Ok(DatasetFormat::JSONL),
                    "csv" => Ok(DatasetFormat::CSV),
                    "txt" => Ok(DatasetFormat::TXT),
                    "parquet" => Ok(DatasetFormat::Parquet),
                    _ => Err(ExtensionParsingError(value.to_owned()))
                }
            },
            None => Err(ExtensionParsingError(value.to_owned()))
        }
    }
}

pub struct DatasetReader<P: AsRef<Path>, > {
    path: P,
    format: DatasetFormat,
    len: Option<usize>,
}

impl<P: AsRef<Path>> DatasetReader<P> {
    const DEFAULT_NUM_LINES: usize = 128;
    fn new<>(path: P, dataset_type: Option<impl Into<DatasetFormat>>, len: Option<usize>, extension: Option<OsString>) {
        let dataset_type = dataset_type.unwrap_or_else(|| DatasetFormat::from(path.as_ref().extension()));
        Self{
             path,
            len,
        }
    }

    fn get_jsonl_iter(&self) -> Result<Reader<BufReader<File>>, arrow::error::ArrowError> {
        let file = File::open(self.path.as_ref())?;
        let mut reader = BufReader::new(file);
        let const num_lines_for_infer= self.len.map_or_else(
            || Self::DEFAULT_NUM_LINES,
            |l| l.min(Self::DEFAULT_NUM_LINES),
        );
        let (schema, _) = infer_json_schema_from_seekable(&mut reader, Some(num_lines_for_infer))?;
        ReaderBuilder::new(std::sync::Arc::new(schema)).build(reader)
    }
    fn get_csv_iter(&self) -> Result<(), arrow::error::ArrowError> {}
}

#[cfg(test)]
mod test {
    #[test]
    fn test_build_jsonl_iter() {
        todo!();
    }
}
