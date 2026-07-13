//! The private `nil` sentinel.
//!
//! `nil` is the VM's uninitialized-slot marker. It has no surface class and can
//! never be produced or observed by user code (Invariant 4,
//! [ADR-0010](../../../docs/adr/0010-tagged-value-enum.md)).

use crate::value::Value;

/// The private `nil` sentinel value.
pub const NIL: Value = Value::Nil;
