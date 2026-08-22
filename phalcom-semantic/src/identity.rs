//! Canonical semantic identities for Phalcom programs and declarations.

pub use phalcom_modules::{DeclarationId, ModuleId};
use phalcom_common::selector::Selector;

/// The dispatch side of a member declaration or lookup (instance vs class).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DispatchSide {
    Instance,
    Class,
}

impl DispatchSide {
    pub fn is_instance(self) -> bool {
        matches!(self, Self::Instance)
    }

    pub fn is_class(self) -> bool {
        matches!(self, Self::Class)
    }
}

/// Canonical callable identity across module/class boundaries.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CallableId {
    pub owner: DeclarationId,
    pub selector: Selector,
    pub side: DispatchSide,
}

impl CallableId {
    pub fn new(owner: DeclarationId, selector: Selector, side: DispatchSide) -> Self {
        Self { owner, selector, side }
    }
}

/// Canonical field identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FieldId {
    pub owner: DeclarationId,
    pub name: Box<str>,
    pub side: DispatchSide,
}

impl FieldId {
    pub fn new(owner: DeclarationId, name: impl Into<Box<str>>, side: DispatchSide) -> Self {
        Self {
            owner,
            name: name.into(),
            side,
        }
    }
}

/// Snapshot-local binding identity for local variables/parameters.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BindingId(pub u32);
