use std::thread::current;

use arrow::{array::Array, array::AsArray, array::StringArrayType, datatypes::DataType};

use crate::{
    document::{FieldIdx, QueryType},
    lints::StreamingLinter,
};

use ahash::HashMap;

pub struct TokenizableStringLinter {
    number_of_example_over_min_byte_len: Vec<(FieldIdx, usize)>,
    min_byte_len_for_lint: usize,
    min_count_before_lint: usize,
}

impl Default for TokenizableStringLinter {
    fn default() -> Self {
        Self {
            number_of_example_over_min_byte_len: Vec::default(),
            min_byte_len_for_lint: 30,
            min_count_before_lint: 3,
        }
    }
}

impl TokenizableStringLinter {
    fn increment_count_based_on_field(&mut self, field: FieldIdx) {
        for index in 0..self.number_of_example_over_min_byte_len.len() {
            let (field_idx_current_value, current_count) =
                self.number_of_example_over_min_byte_len[index];
            if field_idx_current_value == field {
                self.number_of_example_over_min_byte_len[index] = (field, current_count + 1);
                return;
            }
        }
        self.number_of_example_over_min_byte_len.push((field, 1));
    }

    fn compute_number_of_example_over_min_byte_len<'a, A: StringArrayType<'a>>(
        &'a mut self,
        array: &'a A,
        field_idx: FieldIdx,
    ) {
        for index in 0..array.len() {
            let elem_length = array.value(index).len();
            if elem_length >= self.min_byte_len_for_lint {
                self.increment_count_based_on_field(field_idx);
            }
        }
    }
}

impl StreamingLinter for TokenizableStringLinter {
    const QUERY_TYPE: crate::document::QueryType = QueryType::StringArray;
    fn lint(&mut self, doc: &crate::document::Document) -> Option<super::Lints> {
        let lints: Vec<String> = vec![];
        let arrays = self.get_arrays(doc);
        for (field_idx, arr) in arrays {
            let field_name = doc.field_name_by_idx(field_idx);
            match arr.data_type() {
                DataType::Utf8 => {
                    let arr_string = arr.as_string::<i32>();
                    self.compute_number_of_example_over_min_byte_len(&arr_string, field_idx);
                }
                DataType::LargeUtf8 => {
                    let arr_string = arr.as_string::<i64>();
                    self.compute_number_of_example_over_min_byte_len(&arr_string, field_idx);
                }
                DataType::Utf8View => {
                    let arr_string = arr.as_string_view();
                    self.compute_number_of_example_over_min_byte_len(&arr_string, field_idx);
                }
                _ => unreachable!(),
            };
        }
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests_common::{
        setup_document_with_lots_of_numbers_and_string,
        setup_document_with_numbers_and_long_strings, setup_document_with_some_numbers_and_string,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_lint_for_tokenizable_string() {
        let doc = setup_document_with_numbers_and_long_strings();
        let linter = TokenizableStringLinter::default();
        let actual_lints = linter.lint(&doc).unwrap().lint;
        let expected_lints = vec![String::from(
            "A feature (column a) has very long strings (3 strings over 30 byte) that are likely all unique.",
        )];
        assert_eq!(actual_lints, expected_lints);
    }
}
