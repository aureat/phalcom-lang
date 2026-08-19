//! A native, heap-backed range bound descriptor.
//!
//! Realizes [ADR-0032 §1](../../../docs/adr/accepted/0032-collections-representation-and-literals.md)
//! (native heap-arm representation) and
//! [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)
//! (the raw-primitive floor amendment): `Range` is a dedicated
//! [`crate::heap::Object::Range`] heap variant, mirroring
//! [`crate::heap::ListObject`] — **not** an [`crate::heap::InstanceObject`].
//! It records optional bounds and upper inclusion only. Progression and
//! iteration semantics are deliberately deferred.

use crate::value::Value;

/// A native range descriptor. `Value::nil()` is private and means an omitted
/// endpoint, avoiding an arena-wide size increase from two `Option<Value>`s.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RangeObject {
    lower: Value,
    upper: Value,
    upper_inclusive: bool,
}

impl RangeObject {
    /// Builds a range, canonicalizing omitted endpoints to `Value::nil()`.
    pub fn new(lower: Option<Value>, upper: Option<Value>, upper_inclusive: bool) -> Self {
        assert!(!upper_inclusive || upper.is_some(), "an inclusive upper bound must be present");
        if let Some(lower) = lower {
            assert!(!lower.is_nil(), "a present lower bound cannot be Value::nil()");
        }
        if let Some(upper) = upper {
            assert!(!upper.is_nil(), "a present upper bound cannot be Value::nil()");
        }
        Self {
            lower: lower.unwrap_or(Value::nil()),
            upper: upper.unwrap_or(Value::nil()),
            upper_inclusive,
        }
    }

    pub fn lower(&self) -> Option<Value> {
        (!self.lower.is_nil()).then_some(self.lower)
    }

    pub fn upper(&self) -> Option<Value> {
        (!self.upper.is_nil()).then_some(self.upper)
    }

    pub fn upper_inclusive(&self) -> bool {
        self.upper_inclusive
    }
}
