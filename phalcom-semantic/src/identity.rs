pub use crate::types::id::{ProperTypeId, TypeId, TypeStoreId};
use phalcom_common::selector::Selector;
pub use phalcom_modules::{
    DeclarationId, ModuleComponent, ModuleId, ModulePath, ProjectIdentity, ProjectRevisionFingerprint, ProjectSourceIdentity, ResolvedProjectId,
    StableModuleKey, StableProjectKey, SyntheticProjectId,
};

/// Stable identifier for a workspace instance.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorkspaceId(pub u64);

impl WorkspaceId {
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Monotonically increasing revision counter within a workspace.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticRevision(pub u64);

impl SemanticRevision {
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

/// Unique snapshot identifier combining workspace, semantic revision, and type store identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SnapshotId {
    pub workspace: WorkspaceId,
    pub revision: SemanticRevision,
    pub store: TypeStoreId,
}

impl SnapshotId {
    pub const fn new(workspace: WorkspaceId, revision: SemanticRevision, store: TypeStoreId) -> Self {
        Self { workspace, revision, store }
    }

    pub const fn workspace(self) -> WorkspaceId {
        self.workspace
    }

    pub const fn revision(self) -> SemanticRevision {
        self.revision
    }

    pub const fn store(self) -> TypeStoreId {
        self.store
    }
}

/// Store-relative type handle ensuring cross-store handles are never accidentally conflated.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SnapshotTypeRef {
    pub store: TypeStoreId,
    pub id: TypeId,
}

impl SnapshotTypeRef {
    pub const fn new(store: TypeStoreId, id: TypeId) -> Self {
        Self { store, id }
    }

    pub const fn store(self) -> TypeStoreId {
        self.store
    }

    pub const fn id(self) -> TypeId {
        self.id
    }
}

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
