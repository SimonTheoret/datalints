use std::{
    ffi::OsStr,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use arrow::json::Reader;
use arrow_csv::reader::infer_schema_from_files;
use arrow_json::reader::{ReaderBuilder, infer_json_schema_from_seekable};

fn get_jsonl_iter<P: AsRef<Path>>(
    path: P,
) -> Result<Reader<BufReader<File>>, arrow::error::ArrowError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let (schema, _) = infer_json_schema_from_seekable(&mut reader, Some(96))?;
    ReaderBuilder::new(std::sync::Arc::new(schema)).build(reader)
}

fn get_csv_iter<P: AsRef<Path>>(path: P) -> Result<(), arrow::error::ArrowError> {
    let delimiter = match path.as_ref().extension(){
        
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_build_jsonl_iter() {
        todo!();
    }
}
