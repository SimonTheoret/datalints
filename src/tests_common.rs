/// This module contains useful functions used during tests.
/// It is deactivated when cfg(test) is false.
use arrow::array::record_batch;
use arrow_schema::{Field, Schema};

use crate::document::Document;
use crate::reader::NormalizedRecordBatch;
use arrow::array::RecordBatch;
use arrow::array::{Decimal128Array, Decimal256Array};
use arrow_buffer::i256;
use std::sync::Arc;

pub fn setup_normalized_record_batch() -> NormalizedRecordBatch {
    let batch = record_batch!(
        ("a", Utf8, ["alpha", "beta", "gamma"]),
        ("b", Utf8, [Some("alpha"), None, Some("gamma")])
    )
    .unwrap();
    NormalizedRecordBatch(batch)
}

pub fn setup_document() -> Document {
    Document::from(setup_normalized_record_batch())
}

pub fn setup_document_with_some_numbers_and_string() -> Document {
    let batch = record_batch!(
        ("a", Utf8, ["alpha", "beta", "gamma"]),
        ("b", Utf8, [Some("alpha"), None, Some("gamma")]),
        ("anum", Utf8, [Some("1278"), None, Some("718872")]),
        ("num", Utf8, ["12718", "18389", "7188782"])
    )
    .unwrap();
    Document::from(NormalizedRecordBatch(batch))
}

pub fn setup_document_with_lots_of_numbers_and_string() -> Document {
    let batch = record_batch!(
        ("a", Utf8, ["alpha", "beta", "gamma"]),
        ("b", Utf8, [Some("alpha"), None, Some("gamma")]),
        ("num", Utf8, [Some("1278"), None, Some("718872")]),
        ("num", Utf8, ["12718", "18389", "7188782"]),
        ("a", Utf8, ["alpha", "beta", "gamma"]),
        ("b", Utf8, [Some("alpha"), None, Some("gamma")]),
        ("num", Utf8, [Some("1278"), None, Some("718872")]),
        ("num", Utf8, ["12718", "18389", "7188782"]),
        ("a", Utf8, ["alpha", "beta", "gamma"]),
        ("b", Utf8, [Some("alpha"), None, Some("gamma")]),
        ("num", Utf8, [Some("1278"), None, Some("718872")]),
        ("num", Utf8, ["12718", "18389", "7188782"])
    )
    .unwrap();
    Document::from(NormalizedRecordBatch(batch))
}

pub fn setup_document_with_numbers_and_long_strings() -> Document {
    let batch = record_batch!(
        ("a", Utf8, ["This is a very long string, manually written by me, although it could be a bit longerThis is a very long string, manually written by me, although it could be a bit longer.This is a very long string, manually written by me, although it could be a bit longer.This is a very long string, manually written by me, although it could be a bit longer.   . ", "This is another loong string, although a bit shorter.This is another loong string, although a bit shorter.This is another loong string, although a bit shorter.This is another loong string, although a bit shorter.", "This is the final long string.This is the final long string.This is the final long string.This is the final long string.", "This is a smaller one", "smoll", "VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG VERY LONG."]),

        ("b", Utf8, [Some("alpha"), None, Some("gamma")]),
        ("num", Utf8, [Some("1278"), None, Some("718872")])
    )
    .unwrap();
    Document::from(NormalizedRecordBatch(batch))
}

pub fn setup_document_with_no_string_content() -> Document {
    let batch = record_batch!(
        ("a", Int32, [1, 2, 3]),
        ("b", Float64, [Some(4.0), None, Some(5.0)]),
        ("a", Int8, [1, 2, 3]),
        ("b", Float32, [Some(4.0), None, Some(5.0)]),
        ("b", Int16, [1, 2, 3])
    )
    .unwrap();
    Document::from(NormalizedRecordBatch(batch))
}

pub fn setup_document_with_some_floats() -> Document {
    let batch = record_batch!(
        ("a", Float32, [4.0, 4.0, 5.0]),
        ("b", Float32, [Some(4.0), None, Some(5.0)]),
        ("c", Float64, [Some(4.0), None, Some(5.0)]),
        ("d", Float64, [4.0, 4.0, 5.0])
    )
    .unwrap();
    Document::from(NormalizedRecordBatch(batch))
}

pub fn setup_document_with_some_decimals() -> Document {
    let arr_128 = Decimal128Array::from(vec![Some(1), None, Some(2)]);
    let arr = Decimal256Array::from(vec![Some(i256::from(1)), None, Some(i256::from(2))]);
    let schema = Schema::new(vec![
        Field::new("a", arrow_schema::DataType::Decimal128(38, 10), true),
        Field::new("b", arrow_schema::DataType::Decimal256(76, 10), true),
    ]);
    let batch =
        RecordBatch::try_new(Arc::new(schema), vec![Arc::new(arr_128), Arc::new(arr)]).unwrap();
    Document::from(NormalizedRecordBatch(batch))
}
