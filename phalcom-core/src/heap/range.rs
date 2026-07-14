//! A native, heap-backed lazy numeric interval.
//!
//! Realizes [ADR-0032 §1](../../../docs/adr/accepted/0032-collections-representation-and-literals.md)
//! (native heap-arm representation) and
//! [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)
//! (the raw-primitive floor amendment): `Range` is a dedicated
//! [`crate::heap::Object::Range`] heap variant, mirroring
//! [`crate::heap::ListObject`] — **not** an [`crate::heap::InstanceObject`].
//! Unlike every other native collection arm this unit builds, `Range` holds
//! **no element storage at all** — just its three bound fields
//! (`docs/spec/v0.2/core/tuple-and-range.md` §2 RG-2, laziness): `each`/
//! `toList` *generate* `start, start+1, …` in `.ph` over these getters +
//! `Number` arithmetic, so `Range.new(1, 1000000, true)` is O(1) to construct
//! regardless of its logical size.

use crate::value::Value;

/// A native, lazy numeric interval — three fields, no element buffer.
///
/// The three VM-blessed floor primitives
/// ([ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md),
/// `phalcom-core/src/primitive/range.rs`) are raw field reads; the surfaced
/// `size`/`at(_)`/`includes(_)`/`first`/`last`/`each(_)`/`toList` protocol is
/// defined in `.ph` over them (`tuple-and-range.md` §2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RangeObject {
    /// The range's start bound (`Range#first`).
    start: Value,
    /// The range's end bound — inclusive or exclusive per [`Self::inclusive`]
    /// (RG-1, the `a..b`/`a...b` bound convention).
    end: Value,
    /// `true` for `a..b` (inclusive of `end`), `false` for `a...b` (exclusive).
    inclusive: bool,
}

impl RangeObject {
    /// Builds a range from its three bound fields.
    pub fn new(start: Value, end: Value, inclusive: bool) -> Self {
        Self { start, end, inclusive }
    }

    /// Returns the start bound.
    pub fn start(&self) -> Value {
        self.start
    }

    /// Returns the end bound.
    pub fn end(&self) -> Value {
        self.end
    }

    /// Returns whether `end` is included (`a..b`) or excluded (`a...b`).
    pub fn inclusive(&self) -> bool {
        self.inclusive
    }
}
