/*
# Linter implementation
Most of the data linters are taken from http://learningsys.org/nips17/assets/papers/paper_19.pdf.

## Miscoding data linters
Identify data that should be transformed to improve the likelihood
that a model can learn from the data.

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

// PROTOTYPE

use arrow::{
    array::{RecordBatch, StringArray, StringArrayType},
    datatypes::Schema,
};
use eyre::Result;

// PROTOTYPE
pub enum ArrayType {}

/// PORS
/// We should reuse the vecs, and only switch the record_batch when creating a new Doc.
pub struct Document {
    record_batch: RecordBatch,
    iint_type_columns: Vec<(usize, ArrayType)>,
    uint_type_columns: Vec<(usize, ArrayType)>,
    /// Includes the Decimal128 and Decimal256 types.
    float_type_columns: Vec<(usize, ArrayType)>,
    timestamp_type_columns: Vec<(usize, ArrayType)>,
    date_type_columns: Vec<(usize, ArrayType)>,
    time_type_columns: Vec<(usize, ArrayType)>,
    duration_type_columns: Vec<(usize, ArrayType)>,
    interval_type_columns: Vec<(usize, ArrayType)>,
    utf8_type_columns: Vec<(usize, ArrayType)>,
    boolean_type_columns: Vec<(usize, ArrayType)>,
    binary_type_columns: Vec<(usize, ArrayType)>,
}

impl Document {
    // ///Expects a normalized schema
    // fn new(record_batch: RecordBatch, normalized_schema: Schema) -> Self {
    //
    // }
}

// PROTOTYPE
pub trait Linter {
    fn lint(&mut self, doc: &Document) -> Result<()>;
}

/// Detects if a number is represented as a string
pub struct NumberAsStringLinter {
    /// Change this field into a struct. This struct will probably initialize itself with only
    /// negative integers to mark empty places in its array.
    exemples_index: [isize; 3],
}

// PROTOTYPE
impl NumberAsStringLinter {
    fn lint(&mut self, doc: &Document) {
        for col_idx in doc.utf8_type_columns.iter() {
            let col = doc.record_batch.column(*col_idx);
            let col_utf8 = col.as_any().downcast_ref::<&dyn StringArrayType>();
        }
    }
}
