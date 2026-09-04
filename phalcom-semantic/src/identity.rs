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

/// Stable variant identity across the compilation lifecycle.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantId {
    pub owner: DeclarationId,
    pub selector: Selector,
}

impl VariantId {
    pub fn new(owner: DeclarationId, selector: Selector) -> Self {
        Self { owner, selector }
    }

    pub fn family(&self) -> Option<VariantFamilyId> {
        let base = self.selector.base.clone();
        match base {
            phalcom_common::selector::SelectorBase::Named(name) => Some(VariantFamilyId {
                owner: self.owner.clone(),
                base_name: name.into_boxed_str(),
            }),
            phalcom_common::selector::SelectorBase::Subscript => None,
        }
    }
}

/// Stable variant family identity (grouping overloaded variants sharing a base name).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantFamilyId {
    pub owner: DeclarationId,
    pub base_name: Box<str>,
}

impl VariantFamilyId {
    pub fn new(owner: DeclarationId, base_name: impl Into<Box<str>>) -> Self {
        Self {
            owner,
            base_name: base_name.into(),
        }
    }

    pub fn associated(&self) -> AssociatedFamilyId {
        AssociatedFamilyId {
            owner: self.owner.clone(),
            base: phalcom_common::selector::SelectorBase::Named(self.base_name.to_string()),
        }
    }
}

/// Universal associated family identity (covering variants and class-side callables).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssociatedFamilyId {
    pub owner: DeclarationId,
    pub base: phalcom_common::selector::SelectorBase,
}

impl AssociatedFamilyId {
    pub fn new(owner: DeclarationId, base: phalcom_common::selector::SelectorBase) -> Self {
        Self { owner, base }
    }
}

/// Payload field identity within a variant.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantFieldId {
    pub variant: VariantId,
    pub index: u32,
}

impl VariantFieldId {
    pub fn new(variant: VariantId, index: u32) -> Self {
        Self { variant, index }
    }
}

/// Variant constructor identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantConstructorId {
    pub variant: VariantId,
}

impl VariantConstructorId {
    pub fn new(variant: VariantId) -> Self {
        Self { variant }
    }
}

/// Owner identity of a callable member (class declaration or exact enum variant).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CallableOwnerId {
    Declaration(DeclarationId),
    Variant(VariantId),
}

impl CallableOwnerId {
    pub fn declaration(&self) -> &DeclarationId {
        match self {
            Self::Declaration(decl) => decl,
            Self::Variant(var) => &var.owner,
        }
    }

    pub fn module(&self) -> &ModuleId {
        match self {
            Self::Declaration(decl) => &decl.module,
            Self::Variant(var) => &var.owner.module,
        }
    }
}

impl std::ops::Deref for CallableOwnerId {
    type Target = DeclarationId;
    fn deref(&self) -> &Self::Target {
        self.declaration()
    }
}

impl From<DeclarationId> for CallableOwnerId {
    fn from(decl: DeclarationId) -> Self {
        Self::Declaration(decl)
    }
}

impl From<VariantId> for CallableOwnerId {
    fn from(var: VariantId) -> Self {
        Self::Variant(var)
    }
}

impl PartialEq<DeclarationId> for CallableOwnerId {
    fn eq(&self, other: &DeclarationId) -> bool {
        match self {
            CallableOwnerId::Declaration(decl) => decl == other,
            CallableOwnerId::Variant(var) => &var.owner == other,
        }
    }
}

impl PartialEq<CallableOwnerId> for DeclarationId {
    fn eq(&self, other: &CallableOwnerId) -> bool {
        other == self
    }
}

/// Canonical callable identity across module/class/variant boundaries.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CallableId {
    pub owner: CallableOwnerId,
    pub selector: Selector,
    pub side: DispatchSide,
}

impl CallableId {
    pub fn new(owner: impl Into<CallableOwnerId>, selector: Selector, side: DispatchSide) -> Self {
        Self {
            owner: owner.into(),
            selector,
            side,
        }
    }

    pub fn method(owner: DeclarationId, selector: Selector, side: DispatchSide) -> Self {
        Self {
            owner: CallableOwnerId::Declaration(owner),
            selector,
            side,
        }
    }

    pub fn case_method(owner: VariantId, selector: Selector) -> Self {
        Self {
            owner: CallableOwnerId::Variant(owner),
            selector,
            side: DispatchSide::Instance,
        }
    }

    /// Returns the canonical class-side callable identity that owns generic
    /// parameters declared directly on a variant constructor.
    pub fn variant_constructor(variant: VariantId) -> Self {
        Self {
            selector: variant.selector.clone(),
            owner: CallableOwnerId::Variant(variant),
            side: DispatchSide::Class,
        }
    }

    pub fn declaration_owner(&self) -> &DeclarationId {
        self.owner.declaration()
    }

    pub fn module(&self) -> &ModuleId {
        self.owner.module()
    }
}

/// Canonical target of an invocation (behavioral callable or variant constructor).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum InvocationTargetId {
    Behavioral(CallableId),
    VariantConstructor(VariantConstructorId),
}

impl InvocationTargetId {
    pub fn behavioral(callable: CallableId) -> Self {
        Self::Behavioral(callable)
    }

    pub fn variant_constructor(variant: VariantId) -> Self {
        Self::VariantConstructor(VariantConstructorId::new(variant))
    }

    pub fn callable_id(&self) -> Option<&CallableId> {
        match self {
            Self::Behavioral(c) => Some(c),
            Self::VariantConstructor(_) => None,
        }
    }
}

impl From<CallableId> for InvocationTargetId {
    fn from(callable: CallableId) -> Self {
        Self::Behavioral(callable)
    }
}

impl From<VariantConstructorId> for InvocationTargetId {
    fn from(ctor: VariantConstructorId) -> Self {
        Self::VariantConstructor(ctor)
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

pub use crate::diagnostic::SemanticSourceSpan;

/// Canonical semantic target attached to source sites and occurrences.
///
/// Local bindings target their declaration [`SourceSiteId`]; declarations,
/// callables, fields, modules, and variants use cross-revision compiler identities.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SemanticTargetId {
    Binding(SourceSiteId),
    Declaration(DeclarationId),
    Callable(CallableId),
    Field(FieldId),
    Module(ModuleId),
    Variant(VariantId),
    VariantFamily(VariantFamilyId),
    VariantField(VariantFieldId),
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
