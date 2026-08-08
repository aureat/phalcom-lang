//! Shared sequence indexing and negative-coordinate normalization.
//!
//! Provides the AST/Runtime index representation validation and relative-to-end
//! conversion helpers required by Spec C.1.

use crate::error::{PhResult, RuntimeError};
use crate::heap::Heap;
use crate::value::Value;

pub(crate) enum NormalizedIndex {
    Valid(usize),
    OutOfRange,
}

/// Validates that `value` is a finite integral number (Int or Float with fract == 0.0),
/// and normalizes any negative coordinate against sequence length `len`.
///
/// Returns type errors if the value's type or float representation is invalid.
pub(crate) fn normalize_element_index(heap: &Heap, value: &Value, len: usize) -> PhResult<NormalizedIndex> {
    let index_val: i64 = match value {
        Value::Int(n) => *n,
        Value::Float(n) => {
            // TODO(NUMERIC-TOWER): require Int; Float 1.0 must be rejected.
            if !n.is_finite() || n.fract() != 0.0 {
                return Err(RuntimeError::Type {
                    expected: "an integer index",
                    found: "float",
                }
                .into());
            }
            // Avoid overflow when casting huge floats.
            // float values outside exact integer range [-2^53, 2^53] are treated as out of bounds
            if *n < -9007199254740992.0 || *n > 9007199254740992.0 {
                return Ok(NormalizedIndex::OutOfRange);
            }
            *n as i64
        }
        Value::Obj(oid) if heap.as_large_int(*oid).is_some() => {
            // LargeInt is an integer, so it's a valid index representation but guaranteed to be OutOfRange
            return Ok(NormalizedIndex::OutOfRange);
        }
        other => {
            return Err(RuntimeError::Type {
                expected: "an integer index",
                found: other.type_name(),
            }
            .into());
        }
    };

    let len_i = len as i64;
    let normalized = if index_val >= 0 {
        index_val
    } else {
        // Safe relative-to-end lookup.
        len_i.checked_add(index_val).unwrap_or(i64::MIN)
    };

    if normalized >= 0 && normalized < len_i {
        Ok(NormalizedIndex::Valid(normalized as usize))
    } else {
        Ok(NormalizedIndex::OutOfRange)
    }
}
