//! Query keys, input fingerprints, and product fingerprints.

use crate::identity::{CallableId, DeclarationId, FieldId, ModuleId};
use crate::types::type_member::TypeMemberDeclarationId;

/// Fingerprint representing the input parameters to a query evaluation.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InputFingerprint(pub u64);

impl InputFingerprint {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Fingerprint representing the content or result of a computed query product.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProductFingerprint(pub u64);

impl ProductFingerprint {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Strongly-typed key identifying a semantic database query product.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum QueryKey {
    ParsedModule(ModuleId),
    UnlinkedInterface(ModuleId),
    ResolvedImports(ModuleId),
    LinkedInterface(ModuleId),
    DeclarationShell(DeclarationId),
    SemanticComponent(ModuleId),
    DeclarationSurface(DeclarationId),
    TraitHeader(DeclarationId),
    TraitSurface(DeclarationId),
    ConformanceDispatch(crate::identity::ImplId),
    /// Canonical accepted source definition for one callable.
    CallableDefinition(CallableId),
    HierarchyEdge(DeclarationId),
    FieldSignature(FieldId),
    /// Canonical owner-scoped constructor/initializer path set.
    ConstructorInitialization(DeclarationId),
    CallableSignature(CallableId),
    CallableBody(CallableId),
    CallableEffects(CallableId),
    CallableControl(CallableId),
    CallableTermination(CallableId),
    CallableContracts(CallableId),
    VerificationConditions(CallableId),
    SourceStructure(ModuleId),
    SourceFormalAttachment(CallableId),
    AdvisoryCallable(CallableId),
    AdvisoryModule(ModuleId),
    ModuleDiagnostics(ModuleId),
    ModuleMetadata(ModuleId),
    DataDeclaration(DeclarationId),
    EnumDeclaration(DeclarationId),
    EnumRequirements(DeclarationId),
    /// Checked source declaration for one type member, keyed by stable owner/selector identity.
    TypeMemberDeclaration(TypeMemberDeclarationId),
    /// Source-structural case membership, available before checked signatures.
    StaticDomainCaseCatalog(crate::identity::StaticDomainDeclarationId),
    /// Checked constructor contract for one stable static-case identity.
    StaticCaseSignature(crate::identity::StaticCaseId),
    /// Checked associated static-domain declaration, keyed independently of owner specialization.
    AssociatedStaticDomainDeclaration(crate::identity::AssociatedStaticDomainDeclarationId),
    AssociatedSurface(DeclarationId),
    ResolvedImport(phalcom_modules::identity::ImportSiteId),
    LinkedName(ModuleId, String),
    PublicExport(ModuleId, String),
    /// Canonical checked static definition product.
    StaticDefinition(DeclarationId),
    /// Canonical checked compile-time constant product.
    StaticConstant(crate::static_constant::StaticConstantId),
    /// Pre-type, source-only module binding identities and phases.
    ModuleStaticBindingCatalog(ModuleId),
    /// Checked ordinary module binding facts used by static-body analysis.
    ModuleStaticBindings(ModuleId),
    /// Ordinary-checker facts and evaluator layout for one static body.
    CheckedStaticBody(crate::static_definition::StaticBodyId),
}
