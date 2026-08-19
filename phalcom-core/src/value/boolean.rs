//! The boolean immediate values.
//!
//! Booleans are an immediate [`Value`] tag.
//! `True`/`False` are a later dispatch refinement, not distinct representations
//! ([ADR-0004](../../../docs/adr/accepted/0004-boolean-as-abstract-bool-with-true-false.md)).

use crate::value::Value;

/// The `true` value.
pub const TRUE: Value = Value::bool(true);
/// The `false` value.
pub const FALSE: Value = Value::bool(false);
