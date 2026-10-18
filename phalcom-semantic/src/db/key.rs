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

    pub(crate) fn clone_with_allocation_budget(
        &self,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, phalcom_type_meta::allocation::AllocationError> {
        use crate::generic_evidence::GenericConformanceKey;
        use crate::trait_dispatch::{TraitDispatchBucketKey, TraitDispatchTargetFamily};
        use phalcom_common::selector::{Selector, SelectorBase, SelectorSlot};

        Ok(match self {
            Self::CallableSignature(id) => Self::CallableSignature(id.clone_with_allocation_budget(allocations)?),
            Self::FieldSignature(id) => Self::FieldSignature(id.clone_with_allocation_budget(allocations)?),
            Self::GenericConformance(key) => Self::GenericConformance(GenericConformanceKey::new(
                key.subject.clone(),
                key.trait_ref.clone_with_allocation_budget(allocations)?,
            )),
            Self::TraitDispatchBucket(key) => {
                let target_family = match &key.target_family {
                    TraitDispatchTargetFamily::Declaration(decl) => {
                        TraitDispatchTargetFamily::Declaration(decl.clone_with_allocation_budget(allocations)?)
                    }
                    TraitDispatchTargetFamily::ExactEnumCase(var) => {
                        TraitDispatchTargetFamily::ExactEnumCase(var.clone_with_allocation_budget(allocations)?)
                    }
                };
                let base = match &key.selector.base {
                    SelectorBase::Named(name) => SelectorBase::Named(allocations.string(name)?),
                    SelectorBase::Subscript => SelectorBase::Subscript,
                };
                let mut slots = allocations.vector(key.selector.slots.len())?;
                for slot in key.selector.slots.iter() {
                    slots.push(match slot {
                        SelectorSlot::Positional => SelectorSlot::Positional,
                        SelectorSlot::Label(label) => SelectorSlot::Label(allocations.string(label)?),
                    });
                }
                let selector = Selector {
                    base,
                    kind: key.selector.kind,
                    slots: allocations.boxed_slice(slots)?,
                };
                Self::TraitDispatchBucket(TraitDispatchBucketKey {
                    target_family,
                    selector,
                    side: key.side,
                })
            }
            Self::ParsedModule(id) => Self::ParsedModule(id.clone_with_allocation_budget(allocations)?),
            Self::UnlinkedInterface(id) => Self::UnlinkedInterface(id.clone_with_allocation_budget(allocations)?),
            Self::ResolvedImports(id) => Self::ResolvedImports(id.clone_with_allocation_budget(allocations)?),
            Self::LinkedInterface(id) => Self::LinkedInterface(id.clone_with_allocation_budget(allocations)?),
            Self::DeclarationShell(id) => Self::DeclarationShell(id.clone_with_allocation_budget(allocations)?),
            Self::SemanticComponent(id) => Self::SemanticComponent(id.clone_with_allocation_budget(allocations)?),
            Self::DeclarationSurface(id) => Self::DeclarationSurface(id.clone_with_allocation_budget(allocations)?),
            Self::TraitHeader(id) => Self::TraitHeader(id.clone_with_allocation_budget(allocations)?),
            Self::TraitSurface(id) => Self::TraitSurface(id.clone_with_allocation_budget(allocations)?),
            Self::ConformanceDispatch(id) => Self::ConformanceDispatch(id.clone_with_allocation_budget(allocations)?),
            Self::CallableDefinition(id) => Self::CallableDefinition(id.clone_with_allocation_budget(allocations)?),
            Self::HierarchyEdge(id) => Self::HierarchyEdge(id.clone_with_allocation_budget(allocations)?),
            Self::ConstructorInitialization(id) => Self::ConstructorInitialization(id.clone_with_allocation_budget(allocations)?),
            Self::CallableBody(id) => Self::CallableBody(id.clone_with_allocation_budget(allocations)?),
            Self::CallableEffects(id) => Self::CallableEffects(id.clone_with_allocation_budget(allocations)?),
            Self::CallableControl(id) => Self::CallableControl(id.clone_with_allocation_budget(allocations)?),
            Self::CallableTermination(id) => Self::CallableTermination(id.clone_with_allocation_budget(allocations)?),
            Self::CallableContracts(id) => Self::CallableContracts(id.clone_with_allocation_budget(allocations)?),
            Self::VerificationConditions(id) => Self::VerificationConditions(id.clone_with_allocation_budget(allocations)?),
            Self::SourceStructure(id) => Self::SourceStructure(id.clone_with_allocation_budget(allocations)?),
            Self::SourceFormalAttachment(id) => Self::SourceFormalAttachment(id.clone_with_allocation_budget(allocations)?),
            Self::AdvisoryCallable(id) => Self::AdvisoryCallable(id.clone_with_allocation_budget(allocations)?),
            Self::AdvisoryModule(id) => Self::AdvisoryModule(id.clone_with_allocation_budget(allocations)?),
            Self::ModuleDiagnostics(id) => Self::ModuleDiagnostics(id.clone_with_allocation_budget(allocations)?),
            Self::ModuleMetadata(id) => Self::ModuleMetadata(id.clone_with_allocation_budget(allocations)?),
            Self::DataDeclaration(id) => Self::DataDeclaration(id.clone_with_allocation_budget(allocations)?),
            Self::EnumDeclaration(id) => Self::EnumDeclaration(id.clone_with_allocation_budget(allocations)?),
            Self::EnumRequirements(id) => Self::EnumRequirements(id.clone_with_allocation_budget(allocations)?),
            Self::TypeMemberDeclaration(id) => Self::TypeMemberDeclaration(id.clone()),
            Self::StaticDomainCaseCatalog(id) => Self::StaticDomainCaseCatalog(id.clone()),
            Self::StaticCaseSignature(id) => Self::StaticCaseSignature(id.clone()),
            Self::AssociatedStaticDomainDeclaration(id) => Self::AssociatedStaticDomainDeclaration(id.clone()),
            Self::AssociatedSurface(id) => Self::AssociatedSurface(id.clone_with_allocation_budget(allocations)?),
            Self::ResolvedImport(id) => Self::ResolvedImport(id.clone()),
            Self::LinkedName(id, name) => Self::LinkedName(id.clone_with_allocation_budget(allocations)?, allocations.string(name)?),
            Self::PublicExport(id, name) => Self::PublicExport(id.clone_with_allocation_budget(allocations)?, allocations.string(name)?),
            Self::StaticDefinition(id) => Self::StaticDefinition(id.clone_with_allocation_budget(allocations)?),
            Self::StaticConstant(id) => Self::StaticConstant(id.clone()),
            Self::ModuleStaticBindingCatalog(id) => Self::ModuleStaticBindingCatalog(id.clone_with_allocation_budget(allocations)?),
            Self::ModuleStaticBindings(id) => Self::ModuleStaticBindings(id.clone_with_allocation_budget(allocations)?),
            Self::CheckedStaticBody(id) => Self::CheckedStaticBody(id.clone()),
        })
    }
}
