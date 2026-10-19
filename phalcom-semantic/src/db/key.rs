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
    TraitDispatchBucket(crate::trait_dispatch::TraitDispatchBucketKey),
    /// Context-independent canonical proof query for one exact generic
    /// conformance proposition. Lexical assumptions are never part of this
    /// cached product; callers retain contextual evidence on their owner.
    GenericConformance(crate::generic_evidence::GenericConformanceKey),
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
    /// Canonical semantic evidence for one executable attribute argument.
    AttributeArgument(crate::attribute_analysis::AttributeArgumentId),
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

/// Stable observability classification for every semantic database query.
///
/// Keep this mapping exhaustive with `QueryKey::kind`: adding a query key must
/// require an explicit classification here.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum QueryKind {
    ParsedModule,
    UnlinkedInterface,
    ResolvedImports,
    LinkedInterface,
    DeclarationShell,
    SemanticComponent,
    DeclarationSurface,
    TraitHeader,
    TraitSurface,
    ConformanceDispatch,
    TraitDispatchBucket,
    GenericConformance,
    CallableDefinition,
    HierarchyEdge,
    FieldSignature,
    ConstructorInitialization,
    CallableSignature,
    CallableBody,
    CallableEffects,
    CallableControl,
    CallableTermination,
    CallableContracts,
    AttributeArgument,
    VerificationConditions,
    SourceStructure,
    SourceFormalAttachment,
    AdvisoryCallable,
    AdvisoryModule,
    ModuleDiagnostics,
    ModuleMetadata,
    DataDeclaration,
    EnumDeclaration,
    EnumRequirements,
    TypeMemberDeclaration,
    StaticDomainCaseCatalog,
    StaticCaseSignature,
    AssociatedStaticDomainDeclaration,
    AssociatedSurface,
    ResolvedImport,
    LinkedName,
    PublicExport,
    StaticDefinition,
    StaticConstant,
    ModuleStaticBindingCatalog,
    ModuleStaticBindings,
    CheckedStaticBody,
}

impl QueryKind {
    /// Stable inventory used by artifact policy and query-metrics coverage.
    ///
    /// Keep this list in one-to-one correspondence with [`QueryKey::kind`].
    /// The artifact policy is also an exhaustive match, so adding a new query
    /// requires choosing its persistence disposition explicitly.
    pub const ALL: [Self; 46] = [
        Self::ParsedModule,
        Self::UnlinkedInterface,
        Self::ResolvedImports,
        Self::LinkedInterface,
        Self::DeclarationShell,
        Self::SemanticComponent,
        Self::DeclarationSurface,
        Self::TraitHeader,
        Self::TraitSurface,
        Self::ConformanceDispatch,
        Self::TraitDispatchBucket,
        Self::GenericConformance,
        Self::CallableDefinition,
        Self::HierarchyEdge,
        Self::FieldSignature,
        Self::ConstructorInitialization,
        Self::CallableSignature,
        Self::CallableBody,
        Self::CallableEffects,
        Self::CallableControl,
        Self::CallableTermination,
        Self::CallableContracts,
        Self::AttributeArgument,
        Self::VerificationConditions,
        Self::SourceStructure,
        Self::SourceFormalAttachment,
        Self::AdvisoryCallable,
        Self::AdvisoryModule,
        Self::ModuleDiagnostics,
        Self::ModuleMetadata,
        Self::DataDeclaration,
        Self::EnumDeclaration,
        Self::EnumRequirements,
        Self::TypeMemberDeclaration,
        Self::StaticDomainCaseCatalog,
        Self::StaticCaseSignature,
        Self::AssociatedStaticDomainDeclaration,
        Self::AssociatedSurface,
        Self::ResolvedImport,
        Self::LinkedName,
        Self::PublicExport,
        Self::StaticDefinition,
        Self::StaticConstant,
        Self::ModuleStaticBindingCatalog,
        Self::ModuleStaticBindings,
        Self::CheckedStaticBody,
    ];

    pub const fn count() -> usize {
        Self::ALL.len()
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ParsedModule => "parsed_module",
            Self::UnlinkedInterface => "unlinked_interface",
            Self::ResolvedImports => "resolved_imports",
            Self::LinkedInterface => "linked_interface",
            Self::DeclarationShell => "declaration_shell",
            Self::SemanticComponent => "semantic_component",
            Self::DeclarationSurface => "declaration_surface",
            Self::TraitHeader => "trait_header",
            Self::TraitSurface => "trait_surface",
            Self::ConformanceDispatch => "conformance_dispatch",
            Self::TraitDispatchBucket => "trait_dispatch_bucket",
            Self::GenericConformance => "generic_conformance",
            Self::CallableDefinition => "callable_definition",
            Self::HierarchyEdge => "hierarchy_edge",
            Self::FieldSignature => "field_signature",
            Self::ConstructorInitialization => "constructor_initialization",
            Self::CallableSignature => "callable_signature",
            Self::CallableBody => "callable_body",
            Self::CallableEffects => "callable_effects",
            Self::CallableControl => "callable_control",
            Self::CallableTermination => "callable_termination",
            Self::CallableContracts => "callable_contracts",
            Self::AttributeArgument => "attribute_argument",
            Self::VerificationConditions => "verification_conditions",
            Self::SourceStructure => "source_structure",
            Self::SourceFormalAttachment => "source_formal_attachment",
            Self::AdvisoryCallable => "advisory_callable",
            Self::AdvisoryModule => "advisory_module",
            Self::ModuleDiagnostics => "module_diagnostics",
            Self::ModuleMetadata => "module_metadata",
            Self::DataDeclaration => "data_declaration",
            Self::EnumDeclaration => "enum_declaration",
            Self::EnumRequirements => "enum_requirements",
            Self::TypeMemberDeclaration => "type_member_declaration",
            Self::StaticDomainCaseCatalog => "static_domain_case_catalog",
            Self::StaticCaseSignature => "static_case_signature",
            Self::AssociatedStaticDomainDeclaration => "associated_static_domain_declaration",
            Self::AssociatedSurface => "associated_surface",
            Self::ResolvedImport => "resolved_import",
            Self::LinkedName => "linked_name",
            Self::PublicExport => "public_export",
            Self::StaticDefinition => "static_definition",
            Self::StaticConstant => "static_constant",
            Self::ModuleStaticBindingCatalog => "module_static_binding_catalog",
            Self::ModuleStaticBindings => "module_static_bindings",
            Self::CheckedStaticBody => "checked_static_body",
        }
    }
}

impl QueryKey {
    pub const fn kind(&self) -> QueryKind {
        match self {
            Self::ParsedModule(_) => QueryKind::ParsedModule,
            Self::UnlinkedInterface(_) => QueryKind::UnlinkedInterface,
            Self::ResolvedImports(_) => QueryKind::ResolvedImports,
            Self::LinkedInterface(_) => QueryKind::LinkedInterface,
            Self::DeclarationShell(_) => QueryKind::DeclarationShell,
            Self::SemanticComponent(_) => QueryKind::SemanticComponent,
            Self::DeclarationSurface(_) => QueryKind::DeclarationSurface,
            Self::TraitHeader(_) => QueryKind::TraitHeader,
            Self::TraitSurface(_) => QueryKind::TraitSurface,
            Self::ConformanceDispatch(_) => QueryKind::ConformanceDispatch,
            Self::TraitDispatchBucket(_) => QueryKind::TraitDispatchBucket,
            Self::GenericConformance(_) => QueryKind::GenericConformance,
            Self::CallableDefinition(_) => QueryKind::CallableDefinition,
            Self::HierarchyEdge(_) => QueryKind::HierarchyEdge,
            Self::FieldSignature(_) => QueryKind::FieldSignature,
            Self::ConstructorInitialization(_) => QueryKind::ConstructorInitialization,
            Self::CallableSignature(_) => QueryKind::CallableSignature,
            Self::CallableBody(_) => QueryKind::CallableBody,
            Self::CallableEffects(_) => QueryKind::CallableEffects,
            Self::CallableControl(_) => QueryKind::CallableControl,
            Self::CallableTermination(_) => QueryKind::CallableTermination,
            Self::CallableContracts(_) => QueryKind::CallableContracts,
            Self::AttributeArgument(_) => QueryKind::AttributeArgument,
            Self::VerificationConditions(_) => QueryKind::VerificationConditions,
            Self::SourceStructure(_) => QueryKind::SourceStructure,
            Self::SourceFormalAttachment(_) => QueryKind::SourceFormalAttachment,
            Self::AdvisoryCallable(_) => QueryKind::AdvisoryCallable,
            Self::AdvisoryModule(_) => QueryKind::AdvisoryModule,
            Self::ModuleDiagnostics(_) => QueryKind::ModuleDiagnostics,
            Self::ModuleMetadata(_) => QueryKind::ModuleMetadata,
            Self::DataDeclaration(_) => QueryKind::DataDeclaration,
            Self::EnumDeclaration(_) => QueryKind::EnumDeclaration,
            Self::EnumRequirements(_) => QueryKind::EnumRequirements,
            Self::TypeMemberDeclaration(_) => QueryKind::TypeMemberDeclaration,
            Self::StaticDomainCaseCatalog(_) => QueryKind::StaticDomainCaseCatalog,
            Self::StaticCaseSignature(_) => QueryKind::StaticCaseSignature,
            Self::AssociatedStaticDomainDeclaration(_) => QueryKind::AssociatedStaticDomainDeclaration,
            Self::AssociatedSurface(_) => QueryKind::AssociatedSurface,
            Self::ResolvedImport(_) => QueryKind::ResolvedImport,
            Self::LinkedName(_, _) => QueryKind::LinkedName,
            Self::PublicExport(_, _) => QueryKind::PublicExport,
            Self::StaticDefinition(_) => QueryKind::StaticDefinition,
            Self::StaticConstant(_) => QueryKind::StaticConstant,
            Self::ModuleStaticBindingCatalog(_) => QueryKind::ModuleStaticBindingCatalog,
            Self::ModuleStaticBindings(_) => QueryKind::ModuleStaticBindings,
            Self::CheckedStaticBody(_) => QueryKind::CheckedStaticBody,
        }
    }

    pub fn clone_with_allocation_budget(
        &self,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, phalcom_type_meta::allocation::AllocationError> {
        allocations.reserve_bytes(std::mem::size_of::<Self>())?;
        Ok(self.clone())
    }
}
