//! Flow-analysis helpers.

use super::facts::{InferredValue, ValueShape};

/// Joins a sequence of facts while preserving bounded-shape semantics.
pub fn join_values(values: impl IntoIterator<Item = InferredValue>) -> InferredValue {
    let mut values = values.into_iter();
    let Some(mut joined) = values.next() else {
        return InferredValue::exact(ValueShape::Unknown, Default::default());
    };
    for value in values {
        joined = joined.join(&value);
    }
    joined
}
