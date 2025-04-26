use ahash::AHashMap;
use compact_str::CompactString;
use std::{any::Any, fs::File};

fn main() {
    println!("Hello, world!");
}

#[derive(Debug, Clone, derive_more::Display, derive_more::DerefMut, derive_more::Deref)]
struct DataString(CompactString);

type Metadata = AHashMap<&'static str, Box<dyn Any>>;

#[derive(Debug)]
struct Document {
    text: DataString,
    metadata: Metadata,
}

enum DatasetFileType {
    JSON,
    JSONL,
    TXT,
}

struct FileDataset<I>
where
    I: Iterator<Item = &'static [u8]>,
{
    handle: File,
    filetype: DatasetFileType,
    iter: Option<I>,
}

