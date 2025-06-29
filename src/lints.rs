/*
# Linter implementation
Most of the data linters are taken from http://learningsys.org/nips17/assets/papers/paper_19.pdf.

## Miscoding data linters
Identify data that should be transformed to improve the likelihood that a model can learn from the
data.

* Number as string: A number is encoded as a string. Consider whether it should be represented as a number.

* Enum as real: An enum (a categorical value) is encoded as a real number. Consider converting to an integer and using an embedding or one-hot vector.

* Tokenizable string: A feature has very long strings that are likely all unique. Consider tokenizing and using word embeddings.

* Circular domain as linear: A circular feature was identified (e.g., latitude, longitude, day of the week). Consider bucketing the value or transforming it via a trigonometric function.

* Date/time as a string: Consider encoding as a timestamp.

* Zip code as number: A zip code should probably be bucketed or represented as an embedding.

* Integer as float: Integers are encoded as floats. Consider whether they may actually be enums.

* Pattern matching: Validate entries against specific patterns (e.g., email, phone numbers).

## Outlier and scaling data linters
Identify likely outliers and scaling issues in data.

* Unnormalized feature: The values of this feature vary widely. Consider normalizing them.

* Tailed distribution detector: Extreme values that significantly affect the mean were detected. Examine histograms of the data to ensure they follow expected distributions.

* Uncommon list length: A feature is composed of a list of elements, with most instances consisting of a specific list length. However, some instances have a different length. Ensure that the data are materialized as expected and the model can handle data lists of varying length.

* Uncommon sign detector: The data includes some values that have a different sign (+/-) from the rest of the data (e.g., -9999), which can affect training. If these are special markers in the data, consider replacing them with a more neutral value (e.g., an empty or average value).

* Skewness and kurtosis: Check skewness and kurtosis to identify highly skewed distributions in the features.

* Correlation beetween features: Check for highly correlated features that might introduce multicollinearity, which can be mitigated through scaling or transformation.


## Packing error  data linters

* Duplicate value detector: Duplicate rows of data have been identified.

* Empty examples: Empty examples have been detected. The whole row must be empty, not a single feature.

 */

use crate::document::{Document, FieldIdx, QueryType, Queryable};
use arrow::array::Array;
use std::sync::Arc;

mod enum_as_float;
mod number_as_string;
mod tokenizable_string;

pub trait StreamingLinter {
    const QUERY_TYPE: QueryType = QueryType::StringArray;

    fn lint(&mut self, doc: &Document) -> Option<Lints>;

    fn get_arrays<'a>(&self, doc: &'a Document) -> Vec<(FieldIdx, &'a Arc<dyn Array>)> {
        doc.query(Self::QUERY_TYPE)
    }
}

pub trait AggregateLinter {
    const QUERY_TYPE: QueryType = QueryType::StringArray;

    fn aggregate(&mut self, doc: &Document);

    fn lint(self) -> Option<Lints>;

    fn get_arrays<'a>(&self, doc: &'a Document) -> Vec<(FieldIdx, &'a Arc<dyn Array>)> {
        doc.query(Self::QUERY_TYPE)
    }
}

#[derive(Debug, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct Lints {
    lint: Vec<String>,
}

impl PartialEq<Vec<String>> for Lints {
    fn eq(&self, other: &Vec<String>) -> bool {
        self.lint.eq(other)
    }
}
