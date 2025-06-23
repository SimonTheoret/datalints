use std::fmt::Display;
use std::hash::Hash;

use ahash::{HashSet, HashSetExt};
use arrow::{
    array::{AsArray, Float32Array, Float64Array},
    compute::CastOptions,
    datatypes::{DataType, Schema},
};

use crate::document::Document;
use crate::lints::{Lints, QueryType, StreamingLinter};

use super::AggregateLinter;

type EnumThreshold = u16;

pub struct EnumAsFloatLinter {
    /// How many examples we've seen.
    nbr_exemples_seen: usize,
    /// How many positives examples must we collect, at most.
    max_number_of_positives: usize,
    /// How many different floats  must be seen before treating them as if they were not enum.
    enum_categories_threshold: EnumThreshold,
    /// List of counters. Each counter is associated with a single column. Each FloatCounter has a
    /// name, which is the same as the associated column's name.
    counters: Vec<FloatCounter>,
    /// Vec of boolean flags. `true` means we alreaady have found more than
    /// `enum_categories_threshold` unique values for a column. Therefore, we can skip aggregating
    /// for the given field. It is of the same length as the `counters` attribute.
    completed: Vec<bool>,
    /// boolean flag. If true, it means we have more enum categories than
    /// `enum_categories_threshold`. Therefore, the whole aggregating function can be skipped.
    all_completed: bool,
}

// TODO: Clean up
impl EnumAsFloatLinter {
    fn find_counter<S: PartialEq<String>>(&self, field_name: S) -> &FloatCounter {
        // We can safely unwarp due to the fact we already take all the fields of the
        // normalized BatchRecord and creates FloatCounters with the columns' names.
        self.counters.iter().find(|c| field_name == c.name).unwrap()
    }

    fn find_counter_mut<S: PartialEq<String>>(&mut self, field_name: S) -> &mut FloatCounter {
        // We can safely unwarp due to the fact we already take all the fields of the
        // normalized BatchRecord and creates FloatCounters with the columns' names.
        self.counters
            .iter_mut()
            .find(|c| field_name == c.name)
            .unwrap()
    }
    fn insert<S: PartialEq<String>>(&mut self, field_name: S, float: f64) {
        self.find_counter_mut(field_name).insert(float)
    }
    fn column_is_completed<S: PartialEq<String>>(&self, field_name: S) -> bool {
        let idx = self
            .counters
            .iter()
            .position(|c| field_name == c.name)
            .unwrap();
        self.completed[idx]
    }
}

/// This struct is a f64 in disguise. Internally, it is represented as u64. It is mainly used by
/// the FloatCounter.
#[derive(Debug, Hash, PartialEq, Eq)]
struct F64AsBits(u64);

impl From<f64> for F64AsBits {
    fn from(value: f64) -> Self {
        Self(value.to_bits())
    }
}

impl From<F64AsBits> for f64 {
    fn from(value: F64AsBits) -> Self {
        f64::from_bits(value.0)
    }
}

impl Display for F64AsBits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", f64::from_bits(self.0))
    }
}

#[derive(Debug)]
struct FloatCounter {
    map: HashSet<F64AsBits>,
    name: String,
}

impl FloatCounter {
    fn new<S>(col_name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            map: HashSet::with_capacity(10),
            name: col_name.into(),
        }
    }
    fn len(&self) -> usize {
        self.map.len()
    }
}

impl FloatCounter {
    fn insert(&mut self, float: f64) {
        let converted_float = F64AsBits::from(float);
        self.map.insert(converted_float);
    }
}

impl EnumAsFloatLinter {
    fn new(schema: &Schema) -> Self {
        let counters: Vec<_> = schema
            .fields()
            .iter()
            .map(|f| f.name())
            .map(|n| FloatCounter::new(n.clone()))
            .collect();
        let completed: Vec<_> = vec![false; counters.len()];
        Self {
            nbr_exemples_seen: 0,
            max_number_of_positives: 10,
            enum_categories_threshold: 10,
            counters,
            completed,
            all_completed: false,
        }
    }

    fn insert_array_values<T: Iterator<Item = Option<I>>, I: Into<f64>>(
        &mut self,
        iter: T,
        field_name: &str,
    ) {
        for float_opt in iter {
            if float_opt.is_none() {
                continue;
            } else {
                let float: f64 = float_opt.unwrap().into();
                // let float = f64::from(float_opt.unwrap());
                self.insert(field_name, float);
            }
        }
    }
}

impl AggregateLinter for EnumAsFloatLinter {
    const QUERY_TYPE: QueryType = QueryType::FloatArray;
    fn lint(self) -> Option<Lints> {
        let mut lints = Lints::default();
        for counter in self.counters {
            if counter.len() < usize::from(self.enum_categories_threshold) {
                let lint = format!(
                    "Enum represented as floating point number at column '{}'",
                    counter.name
                );
                lints.push(lint)
            };
        }
        if lints.is_empty() { None } else { Some(lints) }
    }

    fn aggregate(&mut self, doc: &Document) {
        if self.all_completed {
            return;
        }
        let arrays = self.get_arrays(doc);
        for (idx_field, arr) in arrays.iter().enumerate() {
            let field_name = doc.field_name_by_idx(idx_field);
            if self.column_is_completed(field_name) {
                continue;
            }
            match arr.data_type() {
                DataType::Float32 => {
                    let arr_float: &Float32Array = arr.as_primitive();
                    self.insert_array_values(arr_float.iter(), field_name);
                }
                DataType::Float64 => {
                    let arr_float: &Float64Array = arr.as_primitive();
                    self.insert_array_values(arr_float.iter(), field_name);
                }
                DataType::Float16 => {
                    let arr_float_dyn = arrow::compute::cast_with_options(
                        arr,
                        &DataType::Float32,
                        &CastOptions::default(),
                    )
                    .unwrap();
                    let arr_float: &Float32Array = arr_float_dyn.as_primitive();
                    self.insert_array_values(arr_float.iter(), field_name);
                }
                DataType::Decimal128(_, _) => {
                    let arr_float_dyn = arrow::compute::cast_with_options(
                        arr,
                        &DataType::Float32,
                        &CastOptions::default(),
                    )
                    .unwrap();
                    let arr_float: &Float32Array = arr_float_dyn.as_primitive();
                    self.insert_array_values(arr_float.iter(), field_name);
                }
                DataType::Decimal256(_, _) => {
                    let arr_float_dyn = arrow::compute::cast_with_options(
                        arr,
                        &DataType::Float32,
                        &CastOptions::default(),
                    )
                    .unwrap();
                    let arr_float: &Float32Array = arr_float_dyn.as_primitive();
                    self.insert_array_values(arr_float.iter(), field_name);
                }
                _ => unreachable!(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lints::enum_as_float::EnumAsFloatLinter;
    use crate::tests_common::{setup_document_with_some_decimals, setup_document_with_some_floats};

    #[test]
    fn test_lint_for_enum_as_float_linter() {
        let doc = setup_document_with_some_floats();
        let mut linter = EnumAsFloatLinter::new(doc.normalized_schema());
        linter.aggregate(&doc);
        let lints = linter.lint().unwrap();

        let expected_lints = vec![
            String::from("Enum represented as floating point number at column 'a'"),
            String::from("Enum represented as floating point number at column 'b'"),
            String::from("Enum represented as floating point number at column 'c'"),
            String::from("Enum represented as floating point number at column 'd'"),
        ];
        assert_eq!(lints, expected_lints);
    }

    #[test]
    fn test_lint_for_enum_as_float_linter_many_decimal() {
        let doc = setup_document_with_some_decimals();
        let mut linter = EnumAsFloatLinter::new(doc.normalized_schema());
        linter.aggregate(&doc);
        let lints = linter.lint().unwrap();

        let expected_lints = vec![
            String::from("Enum represented as floating point number at column 'a'"),
            String::from("Enum represented as floating point number at column 'b'"),
        ];
        assert_eq!(lints, expected_lints);
    }
}
