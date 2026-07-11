//! The boolean immediate values.
//!
//! Booleans are a `Value` arm ([ADR-0010](../../../docs/adr/0010-tagged-value-enum.md));
//! `True`/`False` are a later dispatch refinement, not distinct representations
//! ([ADR-0004](../../../docs/adr/0004-boolean-as-abstract-bool-with-true-false.md)).

use crate::value::Value;

/// The `true` value.
pub const TRUE: Value = Value::Bool(true);
/// The `false` value.
pub const FALSE: Value = Value::Bool(false);
