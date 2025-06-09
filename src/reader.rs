use arrow_json::{
    Reader,
    reader::{ReaderBuilder, infer_json_schema_from_seekable},
};

#[cfg(test)]
use enum_iterator::Sequence;

use eyre::{Result, WrapErr, eyre};
use memmap2::Mmap;
use std::{
    fs::File,
    io::{BufReader, Cursor},
    path::Path,
    str::FromStr,
    sync::Arc,
};

pub struct BoxedMmap(Box<Mmap>);
impl AsRef<[u8]> for BoxedMmap {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
// #[clippy]
#[cfg_attr(test, derive(Sequence))]
#[derive(Debug, Default, derive_more::FromStr, derive_more::Display)]
pub enum DatasetType {
    #[default]
    Jsonl,
    Json,
}

impl DatasetType {
    /// Deduce the dataset type from the given argument or directly from the path.
    fn parse_from_arg_or_path<P: AsRef<Path>>(arg: Option<String>, path: P) -> Result<Self> {
        arg.map_or_else(
            // If self.arg is None
            || {
                let ds_type = match path.as_ref().extension() {
                    Some(s) => s.to_str().map_or_else(DatasetType::default, |ext| {
                        DatasetType::from_str(ext).unwrap_or_default()
                    }),
                    None => DatasetType::default(),
                };
                Ok(ds_type)
            },
            // If self.arg is Some(..)
            |v| {
                DatasetType::from_str(v.as_ref()).map_err(|_| {
                    eyre!(
                        "could not parse the dataset type. Supported dataset types are {:#?}",
                        SUPPORTED_DATASET_TYPES
                    )
                })
            },
        )
    }

    /// Returns `true` if the dataset type is [`JSONL`].
    ///
    /// [`JSONL`]: DatasetType::JSONL
    #[must_use]
    pub fn is_jsonl(&self) -> bool {
        matches!(self, Self::Jsonl)
    }

    /// Returns `true` if the dataset type is [`JSON`].
    ///
    /// [`JSON`]: DatasetType::JSON
    #[must_use]
    pub fn is_json(&self) -> bool {
        matches!(self, Self::Json)
    }
}

#[derive(Debug)]
pub enum DatasetReaderIter {
    Jsonl(Reader<BufReader<Cursor<BoxedMmap>>>),
}

impl Iterator for DatasetReaderIter {
    type Item = Result<arrow::array::RecordBatch>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Jsonl(iter) => iter.next().map(|r| r.map_err(|e| eyre!(e))),
        }
    }
}

const SUPPORTED_DATASET_TYPES: [&str; 2] = ["Jsonl", "Json"];

pub struct DatasetReaderBuilder<P>
where
    P: AsRef<Path>,
{
    path: P,
    arg: Option<String>,
}
impl<P> DatasetReaderBuilder<P>
where
    P: AsRef<Path>,
{
    fn new(path: P) -> Self {
        Self { path, arg: None }
    }

    fn build(self) -> Result<DatasetReaderIter> {
        let path = self.path;
        let path_ref = path.as_ref();
        let dataset_type = DatasetType::parse_from_arg_or_path(self.arg, path_ref)?;
        match dataset_type {
            DatasetType::Json | DatasetType::Jsonl => {
                let file = File::open(path_ref).wrap_err_with(|| {
                    format!("could not open the dataset {:?}", path_ref.to_owned())
                })?;
                let mmap = unsafe {
                    Box::new(Mmap::map(&file).wrap_err_with(|| {
                        format!("could not memmap the dataset {:?}", path_ref.to_owned())
                    })?)
                };
                let cursor = Cursor::new(BoxedMmap(mmap));
                let mut reader = BufReader::new(cursor);
                let schema = infer_json_schema_from_seekable(&mut reader, Some(100))
                    .wrap_err_with(|| "could not infer the schema")?
                    .0;
                let schema = Arc::new(
                    schema
                        .normalize(".", None)
                        .wrap_err("Could not normalize the schema")?,
                );
                let builder = ReaderBuilder::new(schema);
                let json_reader = builder
                    .build(reader)
                    .wrap_err_with(|| "could not build the JSON reader")?;
                Ok(DatasetReaderIter::Jsonl(json_reader))
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn memmap_jsonl() {
        let path = "./tests/test_data/jsonl_test.jsonl";
        let json_reader = DatasetReaderBuilder::new(path).build().unwrap();
        let expected = 1; // we expect the dataset to fit into a single batch
        assert_eq!(json_reader.count(), expected)
    }

    #[test]
    fn err_msg_is_up_to_date() {
        let cardinality_dataset_types_enum = DatasetType::CARDINALITY;
        let expected_cardinality = SUPPORTED_DATASET_TYPES.len();
        assert_eq!(
            cardinality_dataset_types_enum, expected_cardinality,
            "update the SUPPORTED_DATASET_TYPES constant."
        )
    }
}
