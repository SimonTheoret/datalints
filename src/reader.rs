use ahash::HashMap;
use eyre::Result;
use log;
use std::cmp::max;
use std::iter::Skip;
use std::thread::current;
use std::{
    any::Any,
    ffi::{OsStr, OsString},
    fs::File,
    io::{BufRead, BufReader, Lines},
    path::Path,
    sync::Arc,
};
use thiserror::{self, Error};

use crate::Document;
use arrow::datatypes::Schema;
use arrow_csv::reader::Format;

#[derive(derive_more::FromStr)]
pub enum DatasetFormat {
    JsonlFormat,
    CsvFormat,
    TxtFormat,
    ParquetFormat,
}

#[derive(Error, Debug)]
#[error("Error while parsing the file extension")]
pub struct ExtensionParsingError(OsString);

impl TryFrom<&OsStr> for DatasetFormat {
    type Error = ExtensionParsingError;
    fn try_from(value: &OsStr) -> Result<Self, Self::Error> {
        match &value.to_owned().to_ascii_lowercase().to_str() {
            Some(s) => match *s {
                "jsonl" | "json" => Ok(DatasetFormat::JsonlFormat),
                "csv" => Ok(DatasetFormat::CsvFormat),
                "txt" => Ok(DatasetFormat::TxtFormat),
                "parquet" => Ok(DatasetFormat::ParquetFormat),
                _ => Err(ExtensionParsingError(value.to_owned())),
            },
            None => Err(ExtensionParsingError(value.to_owned())),
        }
    }
}

impl DatasetFormat {
    const DEFAULT_NUM_LINES: usize = 128;
    fn build<P: AsRef<Path> + Clone>(
        self,
        path: P,
        skip_n_record: usize,
        max_n_records: Option<usize>,
        schema_num_lines: Option<usize>,
        len: Option<usize>,
        csv_has_header: Option<bool>,
    ) -> Result<(DatasetReader<P>, Option<Schema>)> {
        let file = File::open(path.as_ref())?;
        let num_record_to_read = schema_num_lines.unwrap_or(Self::DEFAULT_NUM_LINES);
        match self {
            Self::JsonlFormat => {
                let mut reader = BufReader::new(file);
                let (schema, _) = arrow_json::reader::infer_json_schema_from_seekable(
                    &mut reader,
                    Some(num_record_to_read),
                )?;
                let iter = arrow_json::ReaderBuilder::new(Arc::new(schema.clone()))
                    .with_batch_size(1)
                    .build(reader)?;
                let iter = iter.skip(skip_n_record);
                Ok((
                    DatasetReader::Jsonl {
                        path,
                        len,
                        iter,
                        max_n_records,
                        current_count: 0,
                    },
                    Some(schema),
                ))
            }
            Self::CsvFormat => {
                let reader = BufReader::new(file);
                let mut csv_reader = csv::Reader::from_reader(reader);
                csv_reader.deserialize()

                let format = Format::default().with_header(csv_has_header.unwrap());
                let copied_reader = BufReader::new(File::open(path.as_ref())?);
                let (schema, _) = format.infer_schema(copied_reader, Some(num_record_to_read))?;
                let iter = arrow_csv::ReaderBuilder::new(Arc::new(schema.clone()))
                    .with_batch_size(1)
                    .build(reader)?;
                let skipped = iter.skip(skip_n_record);
                Ok((
                    DatasetReader::Csv {
                        path,
                        len,
                        iter: skipped,
                        max_n_records,
                        current_count: 0,
                    },
                    Some(schema),
                ))
            }
            Self::TxtFormat => {
                let reader = BufReader::new(file);
                let lines_iter = reader.lines();
                let lines_iter = lines_iter.skip(skip_n_record);
                let dr = DatasetReader::Txt {
                    path,
                    len,
                    iter: lines_iter,
                    max_n_records,
                    current_count: 0,
                };
                Ok((dr, None))
            }
            Self::ParquetFormat => {
                let iter = parquet::file::serialized_reader::SerializedFileReader::new(file)?
                    .into_iter()
                    .with_batch_size(1);
                let skipped = iter.skip(skip_n_record);
                Ok((
                    DatasetReader::Parquet {
                        path,
                        len,
                        iter: skipped,
                        max_n_records,
                        current_count: 0,
                    },
                    None,
                ))
            }
        }
    }
}

pub enum DatasetReader<P: AsRef<Path> + Clone> {
    Jsonl {
        path: P,
        len: Option<usize>,
        iter: Skip<arrow_json::Reader<BufReader<File>>>,
        max_n_records: Option<usize>,
        current_count: usize,
    },
    Txt {
        path: P,
        len: Option<usize>,
        iter: Skip<Lines<BufReader<File>>>,
        max_n_records: Option<usize>,
        current_count: usize,
    },
    Csv {
        path: P,
        len: Option<usize>,
        iter: Skip<arrow_csv::Reader<BufReader<File>>>,
        max_n_records: Option<usize>,
        current_count: usize,
    },
    Parquet {
        path: P,
        len: Option<usize>,
        iter: Skip<parquet::record::reader::RowIter<'static>>,
        max_n_records: Option<usize>,
        current_count: usize,
    },
}

impl<P: AsRef<Path> + Clone> DatasetReader<P> {
    fn current_count_f(&mut self) -> usize {
        match self {
            Self::Csv {
                path: _,
                len: _,
                iter: _,
                max_n_records: _,
                current_count,
            } => *current_count,
            Self::Jsonl {
                path: _,
                len: _,
                iter: _,
                max_n_records: _,
                current_count,
            } => *current_count,
            Self::Txt {
                path: _,
                len: _,
                iter: _,
                max_n_records: _,
                current_count,
            } => *current_count,
            Self::Parquet {
                path: _,
                len: _,
                iter: _,
                max_n_records: _,
                current_count,
            } => *current_count,
        }
    }
    fn increment_current_count(&mut self) {
        match self {
            Self::Csv {
                path: _,
                len: _,
                iter: _,
                max_n_records: _,
                current_count,
            } => *current_count += 1,
            Self::Jsonl {
                path: _,
                len: _,
                iter: _,
                max_n_records: _,
                current_count,
            } => *current_count += 1,
            Self::Txt {
                path: _,
                len: _,
                iter: _,
                max_n_records: _,
                current_count,
            } => *current_count += 1,
            Self::Parquet {
                path: _,
                len: _,
                iter: _,
                max_n_records: _,
                current_count,
            } => *current_count += 1,
        }
    }
    fn max_n_records(&self) -> &Option<usize> {
        match self {
            Self::Csv {
                path: _,
                len: _,
                iter: _,
                max_n_records,
                current_count: _,
            } => max_n_records,
            Self::Jsonl {
                path: _,
                len: _,
                iter: _,
                max_n_records,
                current_count: _,
            } => max_n_records,
            Self::Txt {
                path: _,
                len: _,
                iter: _,
                max_n_records,
                current_count: _,
            } => max_n_records,
            Self::Parquet {
                path: _,
                len: _,
                iter: _,
                max_n_records,
                current_count: _,
            } => max_n_records,
        }
    }
}

impl<P: AsRef<Path> + Clone> Iterator for DatasetReader<P> {
    type Item = Result<Document>;
    fn next(&mut self) -> Option<Self::Item> {
        self.increment_current_count();
        if self
            .max_n_records()
            .is_some_and(|max| self.current_count_f() >= max)
        {
            return None;
        }
        match self {
            Self::Txt {
                path,
                len,
                iter,
                max_n_records,
                current_count,
            } => {
                let current = iter.next();
                if let Some(res) = current {
                    match res {
                        Ok(s) => Some(Ok(Document {
                            text: s,
                            data: HashMap::default(),
                        })),
                        Err(e) => Some(Err(e.into())),
                    }
                } else {
                    None
                }
            }
            Self::Csv {
                path,
                len,
                iter,
                max_n_records,
                current_count,
            } => {
                let current = iter.next();
                if let Some(res) = current {
                    match res {
                        Ok(rb) => match rb.normalize(".", None) {
                            Err(e) => Some(Err(e.into())),
                            Ok(rb) => {
                                let mut map = HashMap::default();
                                let value: Box<dyn Any> = Box::new(rb);
                                map.insert(String::from("record_batch"), value);
                                Some(Ok(Document {
                                    text: String::default(),
                                    data: map,
                                }))
                            }
                        },
                        Err(e) => Some(Err(e.into())),
                    }
                } else {
                    None
                }
            }
            _ => {
                todo!()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_csv_iter() {
        let path = "./tests/test_data/csv_test.csv";
        let format = DatasetFormat::CsvFormat;
        let iter = format
            .build(path, 0, Some(100), Some(100), Some(4), Some(true))
            .unwrap()
            .0;
        let values: Vec<Document> = iter.map(|r| r.unwrap()).collect();
        assert_eq!(values.first().unwrap().text, "this is text 1");
        assert_eq!(values.last().unwrap().text, "My name is momo i am a cat")
    }

    #[test]
    fn test_build_txt_iter() {
        let path = "./tests/test_data/txt_test.txt";
        let format = DatasetFormat::TxtFormat;
        let iter = format
            .build(path, 0, Some(100), Some(100), Some(4), Some(true))
            .unwrap()
            .0;
        let values: Vec<Document> = iter.map(|r| r.unwrap()).collect();
        assert_eq!(values.first().unwrap().text, "this is text 1")
    }
}
