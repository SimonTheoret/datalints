use crate::document::{Document, QueryType, Queryable};
use crate::lints::{Lints, StreamingLinter};
use arrow::array::{Array, AsArray, StringArrayType};
use arrow::datatypes::DataType;
use regex::Regex;

/// This struct is linter responsible for detecting numbers encoded as strings.
pub struct NumberAsStringLinter {
    /// How many examples we've seen.
    nbr_exemples_seen: usize,
    /// Regex engine used for matching numbers.
    regex: Regex,
    /// How many positives matches must we collect, at most.
    max_number_of_positives: usize,
}

impl Default for NumberAsStringLinter {
    fn default() -> Self {
        Self {
            nbr_exemples_seen: 0,
            regex: Regex::new(NumberAsStringLinter::RE).unwrap(),
            max_number_of_positives: 10,
        }
    }
}

impl NumberAsStringLinter {
    const RE: &'static str = r"^[0-9\.\,]+$";
    const QUERY_TYPE: QueryType = QueryType::StringArray;

    fn lint_inner(&mut self, doc: &Document) -> Option<Lints> {
        let arrays = doc.query(NumberAsStringLinter::QUERY_TYPE);
        let mut lints = Lints::default();
        for (idx_field, arr) in arrays.iter().enumerate() {
            if lints.len() >= self.max_number_of_positives {
                break;
            };

            let field_name = doc.field_name_by_idx(idx_field);
            let array_len = arr.len();
            match arr.data_type() {
                DataType::Utf8 => {
                    let arr_string = arr.as_string::<i32>();
                    self.compute_matches_and_add_to_lints(&arr_string, &mut lints, field_name);
                }
                DataType::LargeUtf8 => {
                    let arr_string = arr.as_string::<i64>();
                    self.compute_matches_and_add_to_lints(&arr_string, &mut lints, field_name);
                }
                DataType::Utf8View => {
                    let arr_string = arr.as_string_view();
                    self.compute_matches_and_add_to_lints(&arr_string, &mut lints, field_name);
                }
                _ => unreachable!(),
            };
            self.nbr_exemples_seen += array_len;
        }
        if lints.is_empty() { None } else { Some(lints) }
    }

    fn compute_matches_and_add_to_lints<'a, A: StringArrayType<'a>>(
        &'a self,
        array: &'a A,
        lints: &'a mut Lints,
        field_name: &'a str,
    ) {
        for value_idx in 0..array.len() {
            let value = array.value(value_idx);

            if self.regex.is_match(value) {
                let lint_exemple: String = format!(
                    "Number represented as string ({}) at example index {} and at column '{}'",
                    value,
                    value_idx + self.nbr_exemples_seen,
                    field_name
                );
                lints.push(lint_exemple)
            }
        }
    }
}

impl StreamingLinter for NumberAsStringLinter {
    fn lint(&mut self, doc: &Document) -> Option<Lints> {
        self.lint_inner(doc)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests_common::{
        setup_document_with_lots_of_numbers_and_string, setup_document_with_some_numbers_and_string,
    };

    use super::NumberAsStringLinter;

    #[test]
    fn test_lint_for_number_as_string_linter() {
        let doc = setup_document_with_some_numbers_and_string();
        let mut linter = NumberAsStringLinter::default();
        let lints = linter.lint(&doc).unwrap();
        let expected_lints = vec![
            String::from(
                "Number represented as string (1278) at example index 6 and at column 'anum'",
            ),
            String::from(
                "Number represented as string (718872) at example index 8 and at column 'anum'",
            ),
            String::from(
                "Number represented as string (12718) at example index 9 and at column 'num'",
            ),
            String::from(
                "Number represented as string (18389) at example index 10 and at column 'num'",
            ),
            String::from(
                "Number represented as string (7188782) at example index 11 and at column 'num'",
            ),
        ];
        assert_eq!(lints, expected_lints);
    }

    #[test]
    fn test_maximum_number_of_lints() {
        let doc = setup_document_with_lots_of_numbers_and_string();
        let mut linter = NumberAsStringLinter::default();
        let lints = linter.lint(&doc).unwrap().lint;
        assert_eq!(lints.len(), linter.max_number_of_positives);
    }
    // TODO: uncomment this test once we are able to treat documents with Int8, Int16, and float
    // content
    // #[test]
    // fn test_document_without_string_content() {
    //     let doc = setup_document_with_no_string_content();
    //     let mut linter = NumberAsStringLinter::default();
    //     let lints = linter.lint(&doc).unwrap().lint;
    //     assert!(lints.is_empty())
    // }
}
