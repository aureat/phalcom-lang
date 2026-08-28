pub use crate::types::id::{InferVarId, ProperTypeId, TypeId, TypeStoreId};
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

/// Canonical identity of a declared callable parameter.
///
/// Names and source ranges are presentation metadata; declaration-order index
/// within a canonical callable is the stable semantic identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CallableParameterId {
    pub callable: CallableId,
    pub index: u32,
}

impl CallableParameterId {
    pub fn new(callable: CallableId, index: u32) -> Self {
        Self { callable, index }
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

/// Owner namespace for snapshot-local source sites.
///
/// Site ordinals are allocated independently for modules and callable bodies.
/// This keeps source-site identity local to one immutable snapshot while
/// avoiding accidental renumbering across unrelated callable owners.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceOwner {
    /// A module-level source structure and its top-level sites.
    Module(ModuleId),
    /// A callable body and its local source sites.
    Callable(CallableId),
}

/// Dense source-site ordinal within one [`SourceOwner`] namespace.
///
/// This ID is snapshot-local. It must not be reused after a source snapshot is
/// replaced without carrying the owning [`SnapshotId`] through [`SourceSiteRef`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceSiteLocalId(pub u32);

/// Snapshot-local source-site identity.
///
/// `SourceRange` is deliberately not part of this identity: ranges are
/// attachment metadata and may change after edits.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceSiteId {
    pub owner: SourceOwner,
    pub local: SourceSiteLocalId,
}

/// Externally carryable source-site handle guarded by its owning snapshot.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceSiteRef {
    snapshot: SnapshotId,
    site: SourceSiteId,
}

impl SourceSiteRef {
    /// Creates a reference whose site is valid only for `snapshot`.
    pub fn new(snapshot: SnapshotId, site: SourceSiteId) -> Self {
        Self { snapshot, site }
    }

    /// Returns snapshot that owns this source-site reference.
    pub const fn snapshot(&self) -> SnapshotId {
        self.snapshot
    }

    /// Returns site identity carried by this reference.
    pub fn site(&self) -> &SourceSiteId {
        &self.site
    }

    /// Resolves site only when queried against owning snapshot.
    pub fn resolve_for(&self, snapshot: SnapshotId) -> Option<&SourceSiteId> {
        (self.snapshot == snapshot).then_some(&self.site)
    }
}

/// Canonical semantic target attached to source sites and occurrences.
///
/// Local bindings target their declaration [`SourceSiteId`]; declarations,
/// callables, fields, and modules use cross-revision compiler identities.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticTargetId {
    Binding(SourceSiteId),
    Declaration(DeclarationId),
    Callable(CallableId),
    Field(FieldId),
    Module(ModuleId),
}

/// Snapshot-local binding identity for local variables/parameters.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BindingId(pub u32);

/// Snapshot-local identifier for a callable or top-level body.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BodyId(pub u32);

/// Snapshot-local identifier for an expression within a body.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LocalExpressionId(pub u32);

/// Stable expression identifier within a callable-body analysis product.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExpressionId {
    pub owner: BodyId,
    pub local: LocalExpressionId,
}

impl ExpressionId {
    pub const fn new(owner: BodyId, local: LocalExpressionId) -> Self {
        Self { owner, local }
    }
}

/// Snapshot-local flow node identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FlowNodeId(pub u32);

/// Snapshot-local flow edge identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FlowEdgeId(pub u32);

/// Snapshot-local flow predicate identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PredicateId(pub u32);

/// Snapshot-local type explanation identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExplanationId(pub u32);

/// Snapshot-local diagnostic cause identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DiagnosticCauseId(pub u32);

/// Snapshot-local call resolution identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CallResolutionId(pub u32);

/// Snapshot-local internal semantic incident identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InternalSemanticIncidentId(pub u32);

/// Compatibility alias for older checker/query callers.
pub type AnalysisIncidentId = InternalSemanticIncidentId;
