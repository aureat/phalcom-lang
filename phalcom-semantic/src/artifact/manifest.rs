//! Exhaustive persistence coverage manifest (`ARTF001.C1.P2.R3.T0`).
//!
//! Every `SemanticSnapshot` field, `SemanticWorkspaceSession` base family,
//! `QueryKind`, compiler attachment family, and imported artifact family has
//! an explicit `ArtifactAuthorityDisposition` with named compiler readers and
//! concrete provenance.

use crate::artifact::query_policy::{CanonicalQueryPersistence, CanonicalQueryRootRequirement, canonical_query_persistence, canonical_query_root_requirement};
use crate::db::key::QueryKind;

/// Canonical classification of an artifact or semantic family's authority source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArtifactAuthorityDisposition {
    /// Directly serialized/deserialized in a portable `.telosc` container section.
    Wire,
    /// Reconstructed deterministically on load from validated `Wire` authorities in O(N).
    DerivedOnLoad,
    /// Maintained strictly as a mutable/incremental local source overlay.
    LocalOverlayOnly,
    /// Pertains exclusively to source compilation (e.g. raw AST, source syntax).
    SourceOnly,
    /// Optional tooling presentation text or formatting aid; never an analysis input.
    PresentationOnly,
    /// Advisory runtime-shape facts; omitted in canonical runtime and source-free builds.
    AdvisoryOnly,
    /// Process-local ephemeral state (e.g. process counters, caches, mutexes).
    ProcessLocal,
    /// Intentionally excluded from `.telosc`; never required for source-free compilation.
    IntentionallyOmitted,
}

impl ArtifactAuthorityDisposition {
    /// Returns true if this disposition requires a validated wire representation.
    pub const fn is_wire(self) -> bool {
        matches!(self, Self::Wire)
    }

    /// Returns true if this disposition is derived during artifact loading.
    pub const fn is_derived_on_load(self) -> bool {
        matches!(self, Self::DerivedOnLoad)
    }

    /// Returns true if this disposition is excluded from source-free canonical images.
    pub const fn is_omitted_or_source_only(self) -> bool {
        matches!(
            self,
            Self::SourceOnly | Self::PresentationOnly | Self::AdvisoryOnly | Self::IntentionallyOmitted
        )
    }
}

/// Manifest row classifying a single field of [`crate::snapshot::SemanticSnapshot`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotFieldManifestRow {
    pub field_name: &'static str,
    pub disposition: ArtifactAuthorityDisposition,
    pub section_or_source: &'static str,
    pub compiler_readers: &'static [&'static str],
}

/// Manifest row classifying a base field / family of [`crate::session::SemanticWorkspaceSession`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionBaseFamilyManifestRow {
    pub field_name: &'static str,
    pub disposition: ArtifactAuthorityDisposition,
    pub description: &'static str,
}

/// Artifact persistence disposition for a query root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryArtifactDisposition {
    /// Reusable closed-world root published as `QueryState::Ready` in `SemanticDb`.
    ReusableRoot,
    /// Aggregated dependency evidence; validated but not published as a standalone root.
    AggregateDependencyEvidence,
    /// Source-dependent query; omitted in source-free canonical base.
    OmittedSourceDependent,
    /// Open-world query; never published as unconditional `Ready`.
    OmittedOpenWorld,
    /// Non-compiler or advisory product; omitted from canonical image.
    OmittedNonCompilerProduct,
    /// Unsupported query family; rejected from reusable profile.
    Unsupported,
}

/// Manifest row classifying a [`QueryKind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryKindDispositionRow {
    pub kind: QueryKind,
    pub in_memory_persistence: CanonicalQueryPersistence,
    pub root_requirement: CanonicalQueryRootRequirement,
    pub artifact_disposition: QueryArtifactDisposition,
    pub authority_disposition: ArtifactAuthorityDisposition,
}

/// Canonical compiler attachment families that bridge source to compiler authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompilerAttachmentFamily {
    AcceptedCallableDefinition,
    CheckedStaticDefinitionProgram,
    CheckedStaticConstantProgram,
    NormalizedGenericConstraints,
    LogicalSourceLocations,
    SourceFormalFacts,
    ParsedModuleSyntax,
    EditorSourceIndex,
    ConstructorSuperInitializer,
    ExplanationArena,
}

/// Manifest row classifying a [`CompilerAttachmentFamily`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompilerAttachmentDispositionRow {
    pub family: CompilerAttachmentFamily,
    pub disposition: ArtifactAuthorityDisposition,
    pub section_or_source: &'static str,
    pub compiler_readers: &'static [&'static str],
}

/// Imported artifact section / authority families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImportedArtifactFamily {
    ModuleWorld,
    TypeGraph,
    StaticValues,
    StructuralContracts,
    EnumRequirements,
    StaticDefinitions,
    StaticConstants,
    ModuleStaticBindings,
    WitnessPlans,
    DerivedConformanceRules,
    CallableSignatures,
    CallableDefinitions,
    CallableAnalyses,
    TraitSurfaces,
    DispatchSurfaces,
    ConditionalInherentSets,
    ConformanceHeads,
    ConstructorInitializations,
    TypeMemberDeclarations,
    AssociatedStaticDomains,
    StaticCaseCatalogs,
    StaticCaseSignatures,
    AssociatedSurfaces,
    ModuleExecutionAuthority,
    QueryImage,
    MetadataBundle,
}

/// Manifest row classifying an [`ImportedArtifactFamily`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportedArtifactFamilyDispositionRow {
    pub family: ImportedArtifactFamily,
    pub disposition: ArtifactAuthorityDisposition,
    pub target_native_product: &'static str,
}

// ============================================================================
// Manifest Definitions
// ============================================================================

/// Compile-adjacent field inventory for [`crate::snapshot::SemanticSnapshot`].
/// Must contain exactly the 44 fields of `SemanticSnapshot`.
pub const SNAPSHOT_FIELD_MANIFEST: &[SnapshotFieldManifestRow] = &[
    SnapshotFieldManifestRow {
        field_name: "id",
        disposition: ArtifactAuthorityDisposition::ProcessLocal,
        section_or_source: "process-local session snapshot revision",
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "generation",
        disposition: ArtifactAuthorityDisposition::ProcessLocal,
        section_or_source: "process-local generation counter",
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "store",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::type_graph",
        compiler_readers: &["TypeChecker", "StaticEvaluator", "CallableLowering", "LspTypeProvider"],
    },
    SnapshotFieldManifestRow {
        field_name: "sources",
        disposition: ArtifactAuthorityDisposition::SourceOnly,
        section_or_source: "omitted for source-free imported base",
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "presentation_sources",
        disposition: ArtifactAuthorityDisposition::PresentationOnly,
        section_or_source: "tooling presentation text; not analysis input",
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "surfaces",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::dispatch_surfaces",
        compiler_readers: &["CallChecker", "SelectorDispatcher", "OverrideVerifier"],
    },
    SnapshotFieldManifestRow {
        field_name: "dispatch",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        section_or_source: "SurfaceDispatchResolver::from_surfaces",
        compiler_readers: &["SelectorDispatcher", "MethodResolutionEngine"],
    },
    SnapshotFieldManifestRow {
        field_name: "callable_signatures",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::callable_signatures",
        compiler_readers: &["CallChecker", "TypeChecker", "DiagnosticEmitter"],
    },
    SnapshotFieldManifestRow {
        field_name: "constructor_initializations",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::constructor_initializations",
        compiler_readers: &["ConstructorChecker", "BytecodeCompiler"],
    },
    SnapshotFieldManifestRow {
        field_name: "callable_definitions",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::callable_definitions",
        compiler_readers: &["BytecodeCompiler", "CallableLowering"],
    },
    SnapshotFieldManifestRow {
        field_name: "conformance_index",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::conformance_heads",
        compiler_readers: &["TraitSolver", "WitnessSelector"],
    },
    SnapshotFieldManifestRow {
        field_name: "conformance_witness_plans",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::witness_plans",
        compiler_readers: &["WitnessSelector", "BytecodeCompiler"],
    },
    SnapshotFieldManifestRow {
        field_name: "derived_conformance_rules",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::derived_conformance_rules",
        compiler_readers: &["DeriveExpander", "TraitSolver"],
    },
    SnapshotFieldManifestRow {
        field_name: "trait_dispatch_index",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        section_or_source: "TraitDispatchIndex::from_conformance",
        compiler_readers: &["TraitDispatcher", "SelectorDispatcher"],
    },
    SnapshotFieldManifestRow {
        field_name: "field_signatures",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::dispatch_surfaces",
        compiler_readers: &["FieldChecker", "TypeChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "declarations",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::structural_contracts",
        compiler_readers: &["NameResolver", "SubtypingEngine", "ClassHierarchyGuard"],
    },
    SnapshotFieldManifestRow {
        field_name: "type_aliases",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::structural_contracts",
        compiler_readers: &["TypeResolver", "AliasExpander"],
    },
    SnapshotFieldManifestRow {
        field_name: "trait_headers",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::trait_surfaces",
        compiler_readers: &["TraitSolver", "GenericSignatureResolver"],
    },
    SnapshotFieldManifestRow {
        field_name: "trait_surfaces",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::trait_surfaces",
        compiler_readers: &["TraitSolver", "CallChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "hierarchy",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::structural_contracts",
        compiler_readers: &["SubtypingEngine", "ClassHierarchyGuard"],
    },
    SnapshotFieldManifestRow {
        field_name: "diagnostics",
        disposition: ArtifactAuthorityDisposition::IntentionallyOmitted,
        section_or_source: "empty for valid canonical Universe build",
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "semantic_graph",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        section_or_source: "exact dependency graph from module/query/semantic deps",
        compiler_readers: &["IncrementalAnalyzer", "InvalidationEngine"],
    },
    SnapshotFieldManifestRow {
        field_name: "callable_analyses",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::callable_analyses",
        compiler_readers: &["BytecodeCompiler", "ProverIntegration", "DiagnosticsRenderer"],
    },
    SnapshotFieldManifestRow {
        field_name: "internal_incidents",
        disposition: ArtifactAuthorityDisposition::IntentionallyOmitted,
        section_or_source: "canonical compiler image rejects internal failure incidents",
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "formal_projection",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        section_or_source: "formal_projection_from_callable_analyses",
        compiler_readers: &["ProgramCompiler", "CompilationOverlay"],
    },
    SnapshotFieldManifestRow {
        field_name: "source_index",
        disposition: ArtifactAuthorityDisposition::SourceOnly,
        section_or_source: "broad source tooling index; compiler reads native callable products",
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "advisory",
        disposition: ArtifactAuthorityDisposition::AdvisoryOnly,
        section_or_source: "advisory analysis; empty in runtime-minimal builds",
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "module_products",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        section_or_source: "CanonicalExternalModuleWorld::module_query_products",
        compiler_readers: &["ModuleResolver", "ProgramCompiler"],
    },
    SnapshotFieldManifestRow {
        field_name: "data_semantics",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::structural_contracts",
        compiler_readers: &["PatternExhaustiveness", "StaticValueStore", "ConstructorChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "enum_semantics",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::structural_contracts",
        compiler_readers: &["PatternExhaustiveness", "StaticValueStore", "ConstructorChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "enum_requirements",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::enum_requirements",
        compiler_readers: &["TraitSolver", "PatternExhaustiveness"],
    },
    SnapshotFieldManifestRow {
        field_name: "associated_surfaces",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::associated_surfaces",
        compiler_readers: &["AssociatedResolver", "TypeChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "type_member_declarations",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::type_member_declarations",
        compiler_readers: &["TypeMemberResolver", "TypeChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "associated_static_domains",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::associated_static_domains",
        compiler_readers: &["StaticDomainResolver", "TypeChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "static_case_catalogs",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::static_case_catalogs",
        compiler_readers: &["StaticCaseResolver", "ConstructorChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "static_case_signatures",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::static_case_signatures",
        compiler_readers: &["StaticCaseResolver", "CallChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "static_definitions",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::static_definitions",
        compiler_readers: &["StaticEvalMachine", "StaticApplicationEvaluator"],
    },
    SnapshotFieldManifestRow {
        field_name: "static_constants",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::static_constants",
        compiler_readers: &["ConstantFolder", "BytecodeCompiler"],
    },
    SnapshotFieldManifestRow {
        field_name: "static_values",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::static_values",
        compiler_readers: &["StaticEvaluator", "ConstantFolder", "TypeLevelTermNormalizer"],
    },
    SnapshotFieldManifestRow {
        field_name: "module_static_bindings",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::module_static_bindings",
        compiler_readers: &["ModuleImporter", "UserOverlayBinder"],
    },
    SnapshotFieldManifestRow {
        field_name: "semantic_structure_shards",
        disposition: ArtifactAuthorityDisposition::SourceOnly,
        section_or_source: "source incremental shards; raw AST excluded",
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "import_phase_products",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::module_execution_authority",
        compiler_readers: &["ModuleResolver", "ExecutableLinker"],
    },
    SnapshotFieldManifestRow {
        field_name: "generated_runtime_requirements",
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::module_execution_authority",
        compiler_readers: &["ExecutableLinker", "RuntimeDependencyVerifier"],
    },
    SnapshotFieldManifestRow {
        field_name: "status",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        section_or_source: "Complete only after all P2 verification gates pass",
        compiler_readers: &["SemanticWorkspaceSession", "ProgramCompiler"],
    },
];

/// Session base field / family manifest for [`crate::session::SemanticWorkspaceSession`].
pub const SESSION_BASE_FAMILY_MANIFEST: &[SessionBaseFamilyManifestRow] = &[
    SessionBaseFamilyManifestRow {
        field_name: "workspace",
        disposition: ArtifactAuthorityDisposition::ProcessLocal,
        description: "Process-local workspace identifier",
    },
    SessionBaseFamilyManifestRow {
        field_name: "module_session",
        disposition: ArtifactAuthorityDisposition::ProcessLocal,
        description: "Retained caller project graph / session state",
    },
    SessionBaseFamilyManifestRow {
        field_name: "universe_source_provider",
        disposition: ArtifactAuthorityDisposition::SourceOnly,
        description: "Source AST provider; denied/bypassed for imported session",
    },
    SessionBaseFamilyManifestRow {
        field_name: "db",
        disposition: ArtifactAuthorityDisposition::Wire,
        description: "Fresh SemanticDb with published imported Ready roots + local overlay",
    },
    SessionBaseFamilyManifestRow {
        field_name: "store",
        disposition: ArtifactAuthorityDisposition::Wire,
        description: "Reconstructed current-main TypeStore",
    },
    SessionBaseFamilyManifestRow {
        field_name: "static_values",
        disposition: ArtifactAuthorityDisposition::Wire,
        description: "Reconstructed current-main StaticValueStore",
    },
    SessionBaseFamilyManifestRow {
        field_name: "base_declarations",
        disposition: ArtifactAuthorityDisposition::Wire,
        description: "Reconstructed native DeclarationTypeTable",
    },
    SessionBaseFamilyManifestRow {
        field_name: "base_hierarchy",
        disposition: ArtifactAuthorityDisposition::Wire,
        description: "Reconstructed native MapTypeHierarchy",
    },
    SessionBaseFamilyManifestRow {
        field_name: "base_dispatch",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Derived native SurfaceDispatchResolver",
    },
    SessionBaseFamilyManifestRow {
        field_name: "base_callable_signatures",
        disposition: ArtifactAuthorityDisposition::Wire,
        description: "Reconstructed native CallableSignatureTable",
    },
    SessionBaseFamilyManifestRow {
        field_name: "base_data_semantics",
        disposition: ArtifactAuthorityDisposition::Wire,
        description: "Reconstructed native DataSemanticTable",
    },
    SessionBaseFamilyManifestRow {
        field_name: "base_data_products",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Native data declaration query products",
    },
    SessionBaseFamilyManifestRow {
        field_name: "base_enum_semantics",
        disposition: ArtifactAuthorityDisposition::Wire,
        description: "Reconstructed native EnumSemanticTable",
    },
    SessionBaseFamilyManifestRow {
        field_name: "base_enum_products",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Native enum declaration query products",
    },
    SessionBaseFamilyManifestRow {
        field_name: "base_associated_surfaces",
        disposition: ArtifactAuthorityDisposition::Wire,
        description: "Reconstructed native AssociatedFamilyTable",
    },
    SessionBaseFamilyManifestRow {
        field_name: "base_associated_surface_products",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Native associated surface query products",
    },
    SessionBaseFamilyManifestRow {
        field_name: "base_type_member_declarations",
        disposition: ArtifactAuthorityDisposition::Wire,
        description: "Reconstructed native type-member declaration table",
    },
    SessionBaseFamilyManifestRow {
        field_name: "base_type_member_products",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Native type-member declaration query products",
    },
    SessionBaseFamilyManifestRow {
        field_name: "base_enum_requirements",
        disposition: ArtifactAuthorityDisposition::Wire,
        description: "Reconstructed native EnumRequirementTable",
    },
    SessionBaseFamilyManifestRow {
        field_name: "base_enum_requirement_products",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Native enum requirement query products",
    },
    SessionBaseFamilyManifestRow {
        field_name: "sources",
        disposition: ArtifactAuthorityDisposition::LocalOverlayOnly,
        description: "Local overlay parsed sources; empty for imported canonical base",
    },
    SessionBaseFamilyManifestRow {
        field_name: "source_fingerprints",
        disposition: ArtifactAuthorityDisposition::LocalOverlayOnly,
        description: "Local overlay source fingerprints",
    },
    SessionBaseFamilyManifestRow {
        field_name: "field_lifecycle_fingerprints",
        disposition: ArtifactAuthorityDisposition::LocalOverlayOnly,
        description: "Local overlay field lifecycle fingerprints",
    },
    SessionBaseFamilyManifestRow {
        field_name: "default_field_lifecycle",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Builtin field lifecycle defaults",
    },
    SessionBaseFamilyManifestRow {
        field_name: "semantic_structure_shards",
        disposition: ArtifactAuthorityDisposition::LocalOverlayOnly,
        description: "Local overlay semantic structure shards",
    },
    SessionBaseFamilyManifestRow {
        field_name: "generic_header_dependencies",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Imported base generic header dependencies",
    },
    SessionBaseFamilyManifestRow {
        field_name: "generic_header_reverse_dependencies",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Imported base generic header reverse dependencies",
    },
    SessionBaseFamilyManifestRow {
        field_name: "declaration_signature_dependencies",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Imported base declaration signature dependencies",
    },
    SessionBaseFamilyManifestRow {
        field_name: "declaration_signature_reverse_dependencies",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Imported base declaration signature reverse dependencies",
    },
    SessionBaseFamilyManifestRow {
        field_name: "declaration_signature_fingerprints",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Imported base declaration signature fingerprints",
    },
    SessionBaseFamilyManifestRow {
        field_name: "inherent_impl_surface_owners",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Imported base inherent impl surface owners",
    },
    SessionBaseFamilyManifestRow {
        field_name: "alias_sources",
        disposition: ArtifactAuthorityDisposition::LocalOverlayOnly,
        description: "Local overlay alias AST; imported canonical aliases operate from native TypeAliasInfo",
    },
    SessionBaseFamilyManifestRow {
        field_name: "alias_dependencies",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Derived from imported native TypeAliasInfo dependencies",
    },
    SessionBaseFamilyManifestRow {
        field_name: "alias_reverse_dependencies",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Derived from imported native TypeAliasInfo dependencies",
    },
    SessionBaseFamilyManifestRow {
        field_name: "alias_contract_dependencies",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Derived from imported native TypeAliasInfo contract dependencies",
    },
    SessionBaseFamilyManifestRow {
        field_name: "alias_contract_reverse_dependencies",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Derived from imported native TypeAliasInfo contract dependencies",
    },
    SessionBaseFamilyManifestRow {
        field_name: "alias_forms",
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        description: "Derived from imported native TypeAliasInfo forms",
    },
    SessionBaseFamilyManifestRow {
        field_name: "module_diagnostics",
        disposition: ArtifactAuthorityDisposition::LocalOverlayOnly,
        description: "Local overlay module diagnostics",
    },
    SessionBaseFamilyManifestRow {
        field_name: "canonical_native_diagnostics",
        disposition: ArtifactAuthorityDisposition::LocalOverlayOnly,
        description: "Local overlay canonical native diagnostics",
    },
    SessionBaseFamilyManifestRow {
        field_name: "semantic_diagnostic_contributions",
        disposition: ArtifactAuthorityDisposition::LocalOverlayOnly,
        description: "Local overlay semantic diagnostic contributions",
    },
    SessionBaseFamilyManifestRow {
        field_name: "last_snapshot",
        disposition: ArtifactAuthorityDisposition::ProcessLocal,
        description: "Retained session snapshot state",
    },
    SessionBaseFamilyManifestRow {
        field_name: "last_known_good",
        disposition: ArtifactAuthorityDisposition::ProcessLocal,
        description: "Retained session last-known-good snapshot",
    },
    SessionBaseFamilyManifestRow {
        field_name: "pending_semantic_delta",
        disposition: ArtifactAuthorityDisposition::LocalOverlayOnly,
        description: "Cancelled delta awaiting next local update replay",
    },
    SessionBaseFamilyManifestRow {
        field_name: "requested_deep_modules",
        disposition: ArtifactAuthorityDisposition::LocalOverlayOnly,
        description: "Interactive client deep-analysis frontier",
    },
];

/// Exhaustive classification for all 46 [`QueryKind`] variants.
pub const fn query_kind_disposition(kind: QueryKind) -> QueryKindDispositionRow {
    let in_memory_persistence = canonical_query_persistence(kind);
    let root_requirement = canonical_query_root_requirement(kind);
    let (artifact_disposition, authority_disposition) = match kind {
        QueryKind::ParsedModule => (QueryArtifactDisposition::OmittedSourceDependent, ArtifactAuthorityDisposition::SourceOnly),
        QueryKind::UnlinkedInterface
        | QueryKind::ResolvedImports
        | QueryKind::LinkedInterface
        | QueryKind::DeclarationShell
        | QueryKind::CallableDefinition
        | QueryKind::CallableSignature
        | QueryKind::FieldSignature
        | QueryKind::DeclarationSurface
        | QueryKind::TraitHeader
        | QueryKind::TraitSurface
        | QueryKind::DataDeclaration
        | QueryKind::EnumDeclaration
        | QueryKind::AssociatedSurface
        | QueryKind::TypeMemberDeclaration
        | QueryKind::ModuleStaticBindingCatalog
        | QueryKind::ModuleStaticBindings => (QueryArtifactDisposition::ReusableRoot, ArtifactAuthorityDisposition::Wire),
        QueryKind::HierarchyEdge => (QueryArtifactDisposition::AggregateDependencyEvidence, ArtifactAuthorityDisposition::Wire),
        QueryKind::GenericConformance
        | QueryKind::ConformanceDispatch
        | QueryKind::TraitDispatchBucket
        | QueryKind::CallableBody
        | QueryKind::CheckedStaticBody => (QueryArtifactDisposition::OmittedOpenWorld, ArtifactAuthorityDisposition::IntentionallyOmitted),
        QueryKind::SemanticComponent
        | QueryKind::ConstructorInitialization
        | QueryKind::CallableEffects
        | QueryKind::CallableControl
        | QueryKind::CallableTermination
        | QueryKind::CallableContracts
        | QueryKind::AttributeArgument
        | QueryKind::VerificationConditions
        | QueryKind::SourceStructure
        | QueryKind::SourceFormalAttachment
        | QueryKind::AdvisoryCallable
        | QueryKind::AdvisoryModule
        | QueryKind::ModuleDiagnostics
        | QueryKind::ModuleMetadata
        | QueryKind::EnumRequirements
        | QueryKind::StaticDomainCaseCatalog
        | QueryKind::StaticCaseSignature
        | QueryKind::AssociatedStaticDomainDeclaration
        | QueryKind::ResolvedImport
        | QueryKind::LinkedName
        | QueryKind::PublicExport
        | QueryKind::StaticDefinition
        | QueryKind::StaticConstant => (QueryArtifactDisposition::Unsupported, ArtifactAuthorityDisposition::IntentionallyOmitted),
    };
    QueryKindDispositionRow {
        kind,
        in_memory_persistence,
        root_requirement,
        artifact_disposition,
        authority_disposition,
    }
}

/// Static array of query kind dispositions in [`QueryKind::ALL`] order.
pub const QUERY_KIND_MANIFEST: [QueryKindDispositionRow; 46] = [
    query_kind_disposition(QueryKind::ParsedModule),
    query_kind_disposition(QueryKind::UnlinkedInterface),
    query_kind_disposition(QueryKind::ResolvedImports),
    query_kind_disposition(QueryKind::LinkedInterface),
    query_kind_disposition(QueryKind::DeclarationShell),
    query_kind_disposition(QueryKind::SemanticComponent),
    query_kind_disposition(QueryKind::DeclarationSurface),
    query_kind_disposition(QueryKind::TraitHeader),
    query_kind_disposition(QueryKind::TraitSurface),
    query_kind_disposition(QueryKind::ConformanceDispatch),
    query_kind_disposition(QueryKind::TraitDispatchBucket),
    query_kind_disposition(QueryKind::GenericConformance),
    query_kind_disposition(QueryKind::CallableDefinition),
    query_kind_disposition(QueryKind::HierarchyEdge),
    query_kind_disposition(QueryKind::FieldSignature),
    query_kind_disposition(QueryKind::ConstructorInitialization),
    query_kind_disposition(QueryKind::CallableSignature),
    query_kind_disposition(QueryKind::CallableBody),
    query_kind_disposition(QueryKind::CallableEffects),
    query_kind_disposition(QueryKind::CallableControl),
    query_kind_disposition(QueryKind::CallableTermination),
    query_kind_disposition(QueryKind::CallableContracts),
    query_kind_disposition(QueryKind::AttributeArgument),
    query_kind_disposition(QueryKind::VerificationConditions),
    query_kind_disposition(QueryKind::SourceStructure),
    query_kind_disposition(QueryKind::SourceFormalAttachment),
    query_kind_disposition(QueryKind::AdvisoryCallable),
    query_kind_disposition(QueryKind::AdvisoryModule),
    query_kind_disposition(QueryKind::ModuleDiagnostics),
    query_kind_disposition(QueryKind::ModuleMetadata),
    query_kind_disposition(QueryKind::DataDeclaration),
    query_kind_disposition(QueryKind::EnumDeclaration),
    query_kind_disposition(QueryKind::EnumRequirements),
    query_kind_disposition(QueryKind::TypeMemberDeclaration),
    query_kind_disposition(QueryKind::StaticDomainCaseCatalog),
    query_kind_disposition(QueryKind::StaticCaseSignature),
    query_kind_disposition(QueryKind::AssociatedStaticDomainDeclaration),
    query_kind_disposition(QueryKind::AssociatedSurface),
    query_kind_disposition(QueryKind::ResolvedImport),
    query_kind_disposition(QueryKind::LinkedName),
    query_kind_disposition(QueryKind::PublicExport),
    query_kind_disposition(QueryKind::StaticDefinition),
    query_kind_disposition(QueryKind::StaticConstant),
    query_kind_disposition(QueryKind::ModuleStaticBindingCatalog),
    query_kind_disposition(QueryKind::ModuleStaticBindings),
    query_kind_disposition(QueryKind::CheckedStaticBody),
];

/// Compiler attachment family manifest.
pub const COMPILER_ATTACHMENT_MANIFEST: &[CompilerAttachmentDispositionRow] = &[
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::AcceptedCallableDefinition,
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::callable_definitions",
        compiler_readers: &["ModuleLoweringInputView", "ProgramCompiler::compile_analyzed"],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::CheckedStaticDefinitionProgram,
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::static_definitions",
        compiler_readers: &["StaticEvalMachine", "StaticApplicationEvaluator"],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::CheckedStaticConstantProgram,
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::static_constants",
        compiler_readers: &["ConstantFolder", "BytecodeCompiler"],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::NormalizedGenericConstraints,
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::structural_contracts / generic signatures",
        compiler_readers: &["TypeResolver", "ConformanceResolver", "StaticEvalMachine"],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::LogicalSourceLocations,
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::structural_contracts / logical spans",
        compiler_readers: &["SemanticDiagnostic", "StaticEvalTraceFrame"],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::SourceFormalFacts,
        disposition: ArtifactAuthorityDisposition::DerivedOnLoad,
        section_or_source: "formal_projection_from_snapshot",
        compiler_readers: &["ModuleLoweringInputView", "ProgramCompiler"],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::ParsedModuleSyntax,
        disposition: ArtifactAuthorityDisposition::SourceOnly,
        section_or_source: "raw ParsedModuleUnit and AST syntax are replaced by checked operational authority",
        compiler_readers: &[],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::EditorSourceIndex,
        disposition: ArtifactAuthorityDisposition::SourceOnly,
        section_or_source: "editor presentation requires an optional source capability",
        compiler_readers: &["SourceSemanticIndex"],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::ConstructorSuperInitializer,
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::callable_analyses",
        compiler_readers: &["ConstructorChecker", "BytecodeCompiler"],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::ExplanationArena,
        disposition: ArtifactAuthorityDisposition::Wire,
        section_or_source: "SEMANTIC_WORLD::callable_analyses",
        compiler_readers: &["DiagnosticsRenderer", "ProverIntegration"],
    },
];

/// Imported artifact authority family manifest.
pub const IMPORTED_ARTIFACT_FAMILY_MANIFEST: &[ImportedArtifactFamilyDispositionRow] = &[
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::ModuleWorld,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "CanonicalExternalModuleWorld",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::TypeGraph,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "TypeStore",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::StaticValues,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "StaticValueStore",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::StructuralContracts,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "DeclarationTypeTable, TypeAliasTable, MapTypeHierarchy, DataSemanticTable, EnumSemanticTable",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::EnumRequirements,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "EnumRequirementTable",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::StaticDefinitions,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "StaticDefinitionTable (Imported execution authority)",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::StaticConstants,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "StaticConstantTable",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::ModuleStaticBindings,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "BTreeMap<ModuleId, Arc<ModuleStaticBindingIndex>>",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::WitnessPlans,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "BTreeMap<ImplId, Arc<ConformanceWitnessPlan>>",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::DerivedConformanceRules,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "DerivedConformanceRuleIndex",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::CallableSignatures,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "CallableSignatureTable",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::CallableDefinitions,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "BTreeMap<CallableId, EffectiveCallableDefinition>",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::CallableAnalyses,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "HashMap<CallableId, Arc<CallableAnalysis>>",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::TraitSurfaces,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "TraitHeaderTable, TraitSurfaceTable",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::DispatchSurfaces,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "DeclarationSurface map, FieldSignatureTable",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::ConditionalInherentSets,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "Conditional inherent authority on DeclarationSurface",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::ConformanceHeads,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "ConformanceIndex",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::ConstructorInitializations,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "ConstructorInitializationTable",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::TypeMemberDeclarations,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "BTreeMap<TypeMemberDeclarationId, Arc<TypeMemberDeclarationProduct>>",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::AssociatedStaticDomains,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "BTreeMap<AssociatedStaticDomainDeclarationId, Arc<AssociatedStaticDomainDeclarationProduct>>",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::StaticCaseCatalogs,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "BTreeMap<StaticDomainDeclarationId, Arc<StaticDomainCaseCatalogProduct>>",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::StaticCaseSignatures,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "BTreeMap<StaticCaseId, Arc<StaticCaseSignatureProduct>>",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::AssociatedSurfaces,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "AssociatedFamilyTable",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::ModuleExecutionAuthority,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "BTreeMap<ModuleId, ModuleImportPhaseProduct>, Arc<[RuntimeDependencyRequirement]>",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::QueryImage,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "SemanticDb ready query products with closed dependency edges",
    },
    ImportedArtifactFamilyDispositionRow {
        family: ImportedArtifactFamily::MetadataBundle,
        disposition: ArtifactAuthorityDisposition::Wire,
        target_native_product: "SemanticMetadataBundle",
    },
];

/// Looks up the manifest disposition for a snapshot field.
pub fn snapshot_field_disposition(name: &str) -> Option<&'static SnapshotFieldManifestRow> {
    SNAPSHOT_FIELD_MANIFEST.iter().find(|row| row.field_name == name)
}

/// Looks up the manifest disposition for a session base family.
pub fn session_base_family_disposition(name: &str) -> Option<&'static SessionBaseFamilyManifestRow> {
    SESSION_BASE_FAMILY_MANIFEST.iter().find(|row| row.field_name == name)
}

/// Looks up the manifest disposition for a compiler attachment family.
pub fn compiler_attachment_disposition(family: CompilerAttachmentFamily) -> &'static CompilerAttachmentDispositionRow {
    COMPILER_ATTACHMENT_MANIFEST
        .iter()
        .find(|row| row.family == family)
        .expect("exhaustive attachment manifest")
}

/// Looks up the manifest disposition for an imported artifact family.
pub fn imported_family_disposition(family: ImportedArtifactFamily) -> &'static ImportedArtifactFamilyDispositionRow {
    IMPORTED_ARTIFACT_FAMILY_MANIFEST
        .iter()
        .find(|row| row.family == family)
        .expect("exhaustive imported family manifest")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::query_policy::query_root_is_reusable;

    #[test]
    fn artifact_manifest_covers_every_snapshot_field() {
        // Exactly 44 fields in SemanticSnapshot
        assert_eq!(SNAPSHOT_FIELD_MANIFEST.len(), 44);

        let expected_fields = [
            "id",
            "generation",
            "store",
            "sources",
            "presentation_sources",
            "surfaces",
            "dispatch",
            "callable_signatures",
            "constructor_initializations",
            "callable_definitions",
            "conformance_index",
            "conformance_witness_plans",
            "derived_conformance_rules",
            "trait_dispatch_index",
            "field_signatures",
            "declarations",
            "type_aliases",
            "trait_headers",
            "trait_surfaces",
            "hierarchy",
            "diagnostics",
            "semantic_graph",
            "callable_analyses",
            "internal_incidents",
            "formal_projection",
            "source_index",
            "advisory",
            "module_products",
            "data_semantics",
            "enum_semantics",
            "enum_requirements",
            "associated_surfaces",
            "type_member_declarations",
            "associated_static_domains",
            "static_case_catalogs",
            "static_case_signatures",
            "static_definitions",
            "static_constants",
            "static_values",
            "module_static_bindings",
            "semantic_structure_shards",
            "import_phase_products",
            "generated_runtime_requirements",
            "status",
        ];

        for field in &expected_fields {
            assert!(
                snapshot_field_disposition(field).is_some(),
                "snapshot field '{field}' must be classified in SNAPSHOT_FIELD_MANIFEST"
            );
        }
    }

    #[test]
    fn artifact_manifest_covers_every_query_kind() {
        assert_eq!(QueryKind::ALL.len(), 46);
        for kind in QueryKind::ALL {
            let row = query_kind_disposition(kind);
            assert_eq!(row.kind, kind);
            // Verify consistency with query_root_is_reusable
            if row.artifact_disposition == QueryArtifactDisposition::ReusableRoot {
                assert!(
                    query_root_is_reusable(kind),
                    "query kind {kind:?} marked ReusableRoot in manifest must be accepted by query_root_is_reusable"
                );
            }
        }
    }

    #[test]
    fn artifact_manifest_covers_every_compiler_attachment_family() {
        let all_families = [
            CompilerAttachmentFamily::AcceptedCallableDefinition,
            CompilerAttachmentFamily::CheckedStaticDefinitionProgram,
            CompilerAttachmentFamily::CheckedStaticConstantProgram,
            CompilerAttachmentFamily::NormalizedGenericConstraints,
            CompilerAttachmentFamily::LogicalSourceLocations,
            CompilerAttachmentFamily::SourceFormalFacts,
            CompilerAttachmentFamily::ParsedModuleSyntax,
            CompilerAttachmentFamily::EditorSourceIndex,
            CompilerAttachmentFamily::ConstructorSuperInitializer,
            CompilerAttachmentFamily::ExplanationArena,
        ];
        assert_eq!(COMPILER_ATTACHMENT_MANIFEST.len(), all_families.len());
        for family in all_families {
            let row = compiler_attachment_disposition(family);
            assert_eq!(row.family, family);
        }
    }

    #[test]
    fn artifact_manifest_has_no_unknown_compiler_required_family() {
        for row in SNAPSHOT_FIELD_MANIFEST {
            if !row.compiler_readers.is_empty() {
                assert!(
                    !row.disposition.is_omitted_or_source_only(),
                    "compiler-required field '{}' read by {:?} cannot be omitted without replacement",
                    row.field_name,
                    row.compiler_readers
                );
            }
        }
    }

    #[test]
    fn artifact_manifest_covers_session_base_families() {
        assert!(SESSION_BASE_FAMILY_MANIFEST.len() >= 40);
        for row in SESSION_BASE_FAMILY_MANIFEST {
            assert!(session_base_family_disposition(row.field_name).is_some());
        }
    }
}
