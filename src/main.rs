use std::any::Any;
use ahash::HashMap;

mod reader;

fn main() {
    println!("Hello, world!");
}

pub struct Document<T> {
    inner: T,
    data: HashMap<String, Box<dyn Any>>,
}

// #[derive(Debug, Clone, derive_more::Display, derive_more::DerefMut, derive_more::Deref)]
// pub struct DataString(CompactString);
//
// pub type Metadata = AHashMap<&'static str, Box<dyn Any>>;
//
// #[derive(Debug)]
// pub struct Document {
//     text: DataString,
//     metadata: Metadata,
// }
//
// pub enum DatasetFileType {
//     JSON,
//     JSONL,
//     TXT,
// }
//
// pub struct FileDataset<I>
// where
//     I: Iterator,
// {
//     handle: File,
//     filetype: DatasetFileType,
//     iter: Option<I>,
// }
//
//
