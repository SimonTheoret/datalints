use crate::reader::NormalizedRecordBatch;
use arrow::array::Array;
use arrow::datatypes::SchemaRef;
use arrow_schema::DataType;
use std::sync::Arc;

pub struct Document {
    array: NormalizedRecordBatch,
    string_type_columns: Vec<usize>,
    float_type_columns: Vec<usize>,
}

impl From<NormalizedRecordBatch> for Document {
    fn from(value: NormalizedRecordBatch) -> Self {
        let mut string_type_columns: Vec<usize> = Vec::with_capacity(10);
        let mut float_type_columns: Vec<usize> = Vec::with_capacity(10);
        let schema = value.schema();
        let fields = schema.fields(); // Normlized fields
        for (idx, field) in fields.iter().enumerate() {
            match field.data_type() {
                DataType::Utf8 => string_type_columns.push(idx),
                DataType::LargeUtf8 => string_type_columns.push(idx),
                DataType::Utf8View => string_type_columns.push(idx),
                DataType::Float16 => float_type_columns.push(idx),
                DataType::Float32 => float_type_columns.push(idx),
                DataType::Float64 => float_type_columns.push(idx),
                DataType::Decimal128(_, _) => float_type_columns.push(idx),
                DataType::Decimal256(_, _) => float_type_columns.push(idx),
                _ => todo!(),
            }
        }
        Self {
            array: value,
            string_type_columns,
            float_type_columns,
        }
    }
}

impl Document {
    pub fn field_name_by_idx(&self, idx: usize) -> &str {
        self.array.schema_ref().field(idx).name().as_str()
    }
    pub fn normalized_schema(&self) -> &SchemaRef {
        self.array.schema_ref()
    }
}

pub enum QueryType {
    StringArray,
    FloatArray,
}

pub trait Queryable {
    fn query(&self, query_type: QueryType) -> Vec<&Arc<dyn Array>>;
}

impl Queryable for Document {
    fn query(&self, query_type: QueryType) -> Vec<&Arc<dyn Array>> {
        match query_type {
            QueryType::StringArray => self
                .string_type_columns
                .iter()
                .map(|i| self.array.column(*i))
                .collect(),
            QueryType::FloatArray => self
                .float_type_columns
                .iter()
                .map(|i| self.array.column(*i))
                .collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests_common::{setup_document, setup_normalized_record_batch};
    use arrow::array::{LargeStringArray, StringArray, StringViewArray};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_document_from_record_batch() {
        let record_batch = setup_normalized_record_batch();
        let _doc: Document = Document::from(record_batch);
    }

    #[test]
    fn test_queryable_for_document() {
        let doc = setup_document();
        let arrays = doc.query(QueryType::StringArray);
        for arr in arrays.into_iter() {
            let array_ref = arr.as_ref().as_any();
            let is_string_type = array_ref.is::<StringArray>()
                || array_ref.is::<LargeStringArray>()
                || array_ref.is::<StringViewArray>();
            assert_eq!(is_string_type, true)
        }
    }
}
