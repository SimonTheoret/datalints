use ahash::AHashMap;
use compact_str::CompactString;
use std::{any::Any, fs::File};

fn main() {
    println!("Hello, world!");
}

#[derive(Debug, Clone, derive_more::Display, derive_more::DerefMut, derive_more::Deref)]
pub struct DataString(CompactString);

pub type Metadata = AHashMap<&'static str, Box<dyn Any>>;

#[derive(Debug)]
pub struct Document {
    text: DataString,
    metadata: Metadata,
}

pub enum DatasetFileType {
    JSON,
    JSONL,
    TXT,
}

pub struct FileDataset<I>
where
    I: Iterator,
{
    handle: File,
    filetype: DatasetFileType,
    iter: Option<I>,
}


