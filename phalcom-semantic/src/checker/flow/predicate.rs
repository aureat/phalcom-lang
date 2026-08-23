//! Flow predicates for branch filtering and type refinement (Spec 04.5).

use crate::identity::{BindingId, PredicateId};
use crate::types::id::TypeId;

/// A formal predicate asserted on a control flow path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FlowPredicate {
    /// Binding is known to be an instance of `target`.
    IsInstance { binding: BindingId, target: TypeId },
    /// Binding is known not to be nil.
    NotNil { binding: BindingId },
    /// Binding is known to equal a specific value/type.
    Equal { binding: BindingId, target: TypeId },
    /// Condition is true on this branch.
    Truthy { binding: BindingId },
    /// Condition is false on this branch.
    Falsy { binding: BindingId },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredicateEntry {
    pub id: PredicateId,
    pub predicate: FlowPredicate,
}
