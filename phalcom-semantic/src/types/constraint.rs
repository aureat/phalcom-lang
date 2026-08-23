//! Type constraints representation.

use super::id::TypeId;
use phalcom_common::selector::Selector;

/// A static type constraint generated during expression analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeConstraint {
    /// Two types must be identical (unification).
    Equal(TypeId, TypeId),
    /// Left type must be a subtype of right type.
    Subtype(TypeId, TypeId),
    /// Receiver type must support a member/selector.
    HasMember(TypeId, Selector),
}

/// A set of accumulated type constraints.
#[derive(Clone, Debug, Default)]
pub struct ConstraintSet {
    pub constraints: Vec<TypeConstraint>,
}

impl ConstraintSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, constraint: TypeConstraint) {
        self.constraints.push(constraint);
    }

    pub fn extend(&mut self, other: ConstraintSet) {
        self.constraints.extend(other.constraints);
    }

    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty()
    }

    pub fn len(&self) -> usize {
        self.constraints.len()
    }
}
