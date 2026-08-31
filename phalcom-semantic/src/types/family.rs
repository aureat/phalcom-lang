use phalcom_common::selector::{SelectorKind, SelectorSlot};

use super::id::TypeId;

/// Compact identifier for an interned canonical associated family type.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FamilyTypeId(pub u32);

impl FamilyTypeId {
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// Base-erased operation shape identifying a member within an associated family.
/// Does not contain the selector base name (which is denotation/provenance).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FamilyOperationShape {
    pub kind: SelectorKind,
    pub slots: Box<[SelectorSlot]>,
}

impl FamilyOperationShape {
    pub fn new(kind: SelectorKind, slots: impl Into<Box<[SelectorSlot]>>) -> Self {
        Self { kind, slots: slots.into() }
    }

    pub fn method(slots: impl Into<Box<[SelectorSlot]>>) -> Self {
        Self::new(SelectorKind::Method, slots)
    }

    pub fn getter() -> Self {
        Self::new(SelectorKind::Getter, Box::new([]) as Box<[SelectorSlot]>)
    }

    pub fn setter() -> Self {
        Self::new(SelectorKind::Setter, Box::new([SelectorSlot::Positional]) as Box<[SelectorSlot]>)
    }
}

/// Kind of member contained within an associated family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FamilyMemberTypeKind {
    /// Member is a materialized/stored value.
    Value,
    /// Member is an invocable callable.
    Callable,
}

/// Canonical type entry for a single member of an associated family.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FamilyMemberType {
    pub operation: FamilyOperationShape,
    pub member_kind: FamilyMemberTypeKind,
    pub ty: TypeId,
}

impl FamilyMemberType {
    pub fn value(operation: FamilyOperationShape, ty: TypeId) -> Self {
        Self {
            operation,
            member_kind: FamilyMemberTypeKind::Value,
            ty,
        }
    }

    pub fn callable(operation: FamilyOperationShape, callable_ty: TypeId) -> Self {
        Self {
            operation,
            member_kind: FamilyMemberTypeKind::Callable,
            ty: callable_ty,
        }
    }
}

/// Structural family type definition holding canonically ordered member shapes.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FamilyType {
    pub members: Box<[FamilyMemberType]>,
}

impl FamilyType {
    pub fn new(members: Box<[FamilyMemberType]>) -> Self {
        Self { members }
    }

    pub fn find_operation(&self, shape: &FamilyOperationShape) -> Option<&FamilyMemberType> {
        self.members.iter().find(|m| &m.operation == shape)
    }
}

/// Error encountered when canonicalizing or constructing a family type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FamilyTypeError {
    CallableMemberNotCallable { operation: FamilyOperationShape, ty: TypeId },
    DuplicateOperationShape { operation: FamilyOperationShape },
}
