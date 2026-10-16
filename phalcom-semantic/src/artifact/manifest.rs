//! Exhaustive persistence coverage manifest (`PERF003.C2.P7.T0/T5`).
//!
//! Every `SemanticSnapshot` field, `CanonicalProductFamily`, and `CompilerSourceAttachment`
//! has an explicit `Wire`, `Derived`, or `Omitted` disposition with named compiler readers.

use crate::canonical_image::{CanonicalProductFamily, CanonicalQueryPersistence};
use crate::db::QueryKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldDisposition {
    /// Directly serialized in a portable `universe.telosc` section.
    Wire { section: &'static str },
    /// Reconstructed deterministically on load from validated `Wire` authorities in O(N).
    Derived { owner: &'static str },
    /// Intentionally excluded from `universe.telosc`; never required by source-free user compilation.
    Omitted { reason: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapshotFieldManifestRow {
    pub field_name: &'static str,
    pub disposition: FieldDisposition,
    pub compiler_readers: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallableAnalysisFieldManifestRow {
    pub field_name: &'static str,
    pub disposition: FieldDisposition,
}

/// Field-level wire contract for `checker::analysis::CallableAnalysis`.
/// This list is intentionally explicit: adding a required owner field needs a
/// matching DTO/import disposition before canonical image publication.
pub const CALLABLE_ANALYSIS_FIELD_MANIFEST: &[CallableAnalysisFieldManifestRow] = &[
    CallableAnalysisFieldManifestRow { field_name: "callable", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "body_range", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "body_source_owner", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "expressions", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "expression.conditional_dispatch", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "expression.denotation", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "bindings", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "binding.denotation", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "binding_origins", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "for_iteration_lanes", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "positional_expansions", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "known_invalid_positional_expansions", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "sequence_literals", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "associated_resolutions", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "callable_reference_resolutions", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "family_applications", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "match_resolutions", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "pattern_site_resolutions", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "flow_graph", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "entry_flow", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "exits", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "return_validation", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "diagnostics", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "internal_incidents", disposition: FieldDisposition::Omitted { reason: "Canonical compiler image rejects nonempty internal incidents" } },
    CallableAnalysisFieldManifestRow { field_name: "explanations", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "return_explanation", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "dependencies", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "semantic_dependencies", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "constructor_super_initializer", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
    CallableAnalysisFieldManifestRow { field_name: "dependency_fingerprint", disposition: FieldDisposition::Derived { owner: "P7.T8::callable_body_product_fingerprint_after_relocation" } },
    CallableAnalysisFieldManifestRow { field_name: "status", disposition: FieldDisposition::Wire { section: "SEMANTIC_WORLD::callable_analyses" } },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductFamilyDispositionRow {
    pub family: CanonicalProductFamily,
    pub disposition: FieldDisposition,
    pub compiler_readers: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryArtifactDisposition {
    ReusableRoot,
    AggregateDependencyEvidence,
    OmittedSourceDependent,
    OmittedOpenWorld,
    OmittedNonCompilerProduct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryKindDispositionRow {
    pub kind: QueryKind,
    pub in_memory_persistence: CanonicalQueryPersistence,
    pub artifact_disposition: QueryArtifactDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerAttachmentFamily {
    AcceptedCallableDefinition,
    CheckedStaticDefinitionProgram,
    CheckedStaticConstantProgram,
    NormalizedGenericConstraints,
    LogicalSourceLocations,
    SourceFormalFacts,
    ParsedModuleSyntax,
    EditorSourceIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompilerAttachmentDispositionRow {
    pub family: CompilerAttachmentFamily,
    pub disposition: FieldDisposition,
    pub compiler_readers: &'static [&'static str],
}

pub const SNAPSHOT_FIELD_MANIFEST: &[SnapshotFieldManifestRow] = &[
    SnapshotFieldManifestRow {
        field_name: "id",
        disposition: FieldDisposition::Omitted {
            reason: "Session-local snapshot ID generated per-process",
        },
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "generation",
        disposition: FieldDisposition::Omitted {
            reason: "Session-local generation counter",
        },
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "store",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::type_graph",
        },
        compiler_readers: &["TypeChecker", "StaticEvaluator", "CallableLowering", "LspTypeProvider"],
    },
    SnapshotFieldManifestRow {
        field_name: "sources",
        disposition: FieldDisposition::Omitted {
            reason: "Source-free canonical image; raw AST excluded",
        },
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "presentation_sources",
        disposition: FieldDisposition::Omitted {
            reason: "Tooling presentation text; optional presentation capability",
        },
        compiler_readers: &["LspFormattingProvider", "LspHoverProvider"],
    },
    SnapshotFieldManifestRow {
        field_name: "surfaces",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["CallChecker", "SelectorDispatcher", "OverrideVerifier"],
    },
    SnapshotFieldManifestRow {
        field_name: "dispatch",
        disposition: FieldDisposition::Derived {
            owner: "P7.T8::SurfaceDispatchResolver::from_surfaces",
        },
        compiler_readers: &["SelectorDispatcher", "MethodResolutionEngine"],
    },
    SnapshotFieldManifestRow {
        field_name: "callable_signatures",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["CallChecker", "TypeChecker", "DiagnosticEmitter"],
    },
    SnapshotFieldManifestRow {
        field_name: "constructor_initializations",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["ConstructorChecker", "BytecodeCompiler"],
    },
    SnapshotFieldManifestRow {
        field_name: "callable_definitions",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["BytecodeCompiler", "CallableLowering"],
    },
    SnapshotFieldManifestRow {
        field_name: "callable_definitions_by_module",
        disposition: FieldDisposition::Derived {
            owner: "P7.T8::module_indexes",
        },
        compiler_readers: &["ModuleLoweringInputView", "BytecodeCompiler"],
    },
    SnapshotFieldManifestRow {
        field_name: "conformance_index",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["TraitSolver", "WitnessSelector"],
    },
    SnapshotFieldManifestRow {
        field_name: "conformance_witness_plans",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["WitnessSelector", "BytecodeCompiler"],
    },
    SnapshotFieldManifestRow {
        field_name: "derived_conformance_rules",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["DeriveExpander", "TraitSolver"],
    },
    SnapshotFieldManifestRow {
        field_name: "trait_dispatch_index",
        disposition: FieldDisposition::Derived {
            owner: "P7.T8::TraitDispatchIndex::from_conformance",
        },
        compiler_readers: &["TraitDispatcher", "SelectorDispatcher"],
    },
    SnapshotFieldManifestRow {
        field_name: "field_signatures",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["FieldChecker", "TypeChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "declarations",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::structural_contracts",
        },
        compiler_readers: &["NameResolver", "SubtypingEngine", "ClassHierarchyGuard"],
    },
    SnapshotFieldManifestRow {
        field_name: "type_aliases",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::structural_contracts",
        },
        compiler_readers: &["TypeResolver", "AliasExpander"],
    },
    SnapshotFieldManifestRow {
        field_name: "trait_headers",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["TraitSolver", "GenericSignatureResolver"],
    },
    SnapshotFieldManifestRow {
        field_name: "trait_surfaces",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["TraitSolver", "CallChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "hierarchy",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::structural_contracts",
        },
        compiler_readers: &["SubtypingEngine", "ClassHierarchyGuard"],
    },
    SnapshotFieldManifestRow {
        field_name: "diagnostics",
        disposition: FieldDisposition::Omitted {
            reason: "Diagnostic collection; empty for valid Universe build",
        },
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "semantic_graph",
        disposition: FieldDisposition::Derived {
            owner: "P7.T8::reconstructed_semantic_graph",
        },
        compiler_readers: &["IncrementalAnalyzer", "InvalidationEngine"],
    },
    SnapshotFieldManifestRow {
        field_name: "callable_analyses",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        // Required reader contract is the complete owner graph, including the
        // binding/site facts lowering needs; see PERF003.C2.P7's adopted
        // callable-analysis owner-graph amendment.
        compiler_readers: &["BytecodeCompiler", "ProverIntegration", "DiagnosticsRenderer"],
    },
    SnapshotFieldManifestRow {
        field_name: "internal_incidents",
        disposition: FieldDisposition::Omitted {
            reason: "Internal analyzer telemetry; empty for canonical image",
        },
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "formal_projection",
        disposition: FieldDisposition::Derived {
            owner: "P7.T8::formal_projection_from_snapshot",
        },
        compiler_readers: &["ProgramCompiler", "CompilationOverlay"],
    },
    SnapshotFieldManifestRow {
        field_name: "source_index",
        disposition: FieldDisposition::Omitted {
            reason: "Broad source-backed tooling index; compiler-required callable binding/site facts are rebuilt from persisted callable owners",
        },
        compiler_readers: &["LspDefinitionProvider", "LspReferencesProvider"],
    },
    SnapshotFieldManifestRow {
        field_name: "advisory",
        disposition: FieldDisposition::Omitted {
            reason: "Advisory workspace analysis; omitted in runtime-minimal and source-free builds",
        },
        compiler_readers: &["LspAdvisoryProvider"],
    },
    SnapshotFieldManifestRow {
        field_name: "module_products",
        disposition: FieldDisposition::Derived {
            owner: "P7.T8::module_query_products",
        },
        compiler_readers: &["ModuleResolver", "ProgramCompiler"],
    },
    SnapshotFieldManifestRow {
        field_name: "data_semantics",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::structural_contracts",
        },
        compiler_readers: &["PatternExhaustiveness", "StaticValueStore", "ConstructorChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "enum_semantics",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::structural_contracts",
        },
        compiler_readers: &["PatternExhaustiveness", "StaticValueStore", "ConstructorChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "enum_requirements",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::structural_contracts",
        },
        compiler_readers: &["TraitSolver", "PatternExhaustiveness"],
    },
    SnapshotFieldManifestRow {
        field_name: "associated_surfaces",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["AssociatedResolver", "TypeChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "type_member_declarations",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["TypeMemberResolver", "TypeChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "associated_static_domains",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["StaticDomainResolver", "TypeChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "static_case_catalogs",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["StaticCaseResolver", "ConstructorChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "static_case_signatures",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["StaticCaseResolver", "CallChecker"],
    },
    SnapshotFieldManifestRow {
        field_name: "static_definitions",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["StaticEvalMachine", "StaticApplicationEvaluator"],
    },
    SnapshotFieldManifestRow {
        field_name: "static_constants",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["ConstantFolder", "BytecodeCompiler"],
    },
    SnapshotFieldManifestRow {
        field_name: "static_values",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::static_values",
        },
        compiler_readers: &["StaticEvaluator", "ConstantFolder", "TypeLevelTermNormalizer"],
    },
    SnapshotFieldManifestRow {
        field_name: "module_static_bindings",
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["ModuleImporter", "UserOverlayBinder"],
    },
    SnapshotFieldManifestRow {
        field_name: "semantic_structure_shards",
        disposition: FieldDisposition::Omitted {
            reason: "Source incremental shards; raw AST excluded",
        },
        compiler_readers: &[],
    },
    SnapshotFieldManifestRow {
        field_name: "status",
        disposition: FieldDisposition::Omitted {
            reason: "Session-local snapshot state; loaded image is always committed",
        },
        compiler_readers: &[],
    },
];

pub const CANONICAL_PRODUCT_FAMILY_MANIFEST: &[ProductFamilyDispositionRow] = &[
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::ModulesAndInterfaces,
        disposition: FieldDisposition::Wire { section: "MODULE_WORLD" },
        compiler_readers: &["ProgramCompiler", "SemanticWorkspaceSession", "ModuleResolver"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::ImportsAndTopology,
        disposition: FieldDisposition::Wire { section: "MODULE_WORLD" },
        compiler_readers: &["ModuleResolver", "TopologySorter", "ImportBinder"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::TypeArena,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::type_graph",
        },
        compiler_readers: &["TypeChecker", "StaticEvaluator", "CallableLowering", "LspTypeProvider"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::StaticValueArena,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::static_values",
        },
        compiler_readers: &["StaticEvaluator", "ConstantFolder", "TypeLevelTermNormalizer"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::DeclarationsAndAliases,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::structural_contracts",
        },
        compiler_readers: &["NameResolver", "SubtypingEngine", "ClassHierarchyGuard"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::HierarchyAndDispatch,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::structural_contracts",
        },
        compiler_readers: &["SubtypingEngine", "SelectorDispatcher", "MethodResolutionEngine"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::CallableSignaturesAndDefinitions,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["CallChecker", "BytecodeCompiler", "CallableLowering"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::CallableAnalyses,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["BytecodeCompiler", "ProverIntegration", "DiagnosticsRenderer"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::TraitsAndConformance,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["TraitSolver", "WitnessSelector", "DeriveExpander"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::ConformanceWitnessesAndDispatch,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["WitnessSelector", "TraitDispatcher", "BytecodeCompiler"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::DataAndEnums,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::structural_contracts",
        },
        compiler_readers: &["PatternExhaustiveness", "StaticValueStore", "ConstructorChecker"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::AssociatedAndStaticSemantics,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["AssociatedResolver", "StaticDomainResolver", "StaticEvalMachine"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::FrozenStructuralQueries,
        disposition: FieldDisposition::Wire { section: "QUERY_IMAGE" },
        compiler_readers: &["CanonicalQueryReader", "IncrementalAnalyzer"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::RuntimePublicMetadata,
        disposition: FieldDisposition::Wire { section: "METADATA" },
        compiler_readers: &["ProgramCompiler", "RuntimeMetadataLoader"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::CompilerSourceAttachments,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::primary",
        },
        compiler_readers: &["DiagnosticEmitter", "LspDefinitionProvider", "LspHoverProvider", "LspReferencesProvider"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::EditorPresentation,
        disposition: FieldDisposition::Omitted {
            reason: "Tooling presentation text; optional presentation capability",
        },
        compiler_readers: &["LspFormattingProvider", "LspHoverProvider"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::AdvisoryProducts,
        disposition: FieldDisposition::Omitted {
            reason: "Advisory analysis is omitted in source-free compiler execution",
        },
        compiler_readers: &["LspAdvisoryProvider"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::SourceMaps,
        disposition: FieldDisposition::Omitted {
            reason: "Optional logical source mapping capability",
        },
        compiler_readers: &["Debugger", "CrashReporter"],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::Diagnostics,
        disposition: FieldDisposition::Omitted {
            reason: "Never persist diagnostics in compiled canonical image",
        },
        compiler_readers: &[],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::WorkspaceRevisionHistory,
        disposition: FieldDisposition::Omitted {
            reason: "Process-local workspace revision history",
        },
        compiler_readers: &[],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::MutableSemanticDatabase,
        disposition: FieldDisposition::Omitted {
            reason: "Process-local mutable database; loaded image is strictly immutable",
        },
        compiler_readers: &[],
    },
    ProductFamilyDispositionRow {
        family: CanonicalProductFamily::RuntimeHandles,
        disposition: FieldDisposition::Omitted {
            reason: "Process-local runtime object and VM handles",
        },
        compiler_readers: &[],
    },
];

pub const fn family_disposition(family: CanonicalProductFamily) -> ProductFamilyDispositionRow {
    match family {
        CanonicalProductFamily::ModulesAndInterfaces => CANONICAL_PRODUCT_FAMILY_MANIFEST[0],
        CanonicalProductFamily::ImportsAndTopology => CANONICAL_PRODUCT_FAMILY_MANIFEST[1],
        CanonicalProductFamily::TypeArena => CANONICAL_PRODUCT_FAMILY_MANIFEST[2],
        CanonicalProductFamily::StaticValueArena => CANONICAL_PRODUCT_FAMILY_MANIFEST[3],
        CanonicalProductFamily::DeclarationsAndAliases => CANONICAL_PRODUCT_FAMILY_MANIFEST[4],
        CanonicalProductFamily::HierarchyAndDispatch => CANONICAL_PRODUCT_FAMILY_MANIFEST[5],
        CanonicalProductFamily::CallableSignaturesAndDefinitions => CANONICAL_PRODUCT_FAMILY_MANIFEST[6],
        CanonicalProductFamily::CallableAnalyses => CANONICAL_PRODUCT_FAMILY_MANIFEST[7],
        CanonicalProductFamily::TraitsAndConformance => CANONICAL_PRODUCT_FAMILY_MANIFEST[8],
        CanonicalProductFamily::ConformanceWitnessesAndDispatch => CANONICAL_PRODUCT_FAMILY_MANIFEST[9],
        CanonicalProductFamily::DataAndEnums => CANONICAL_PRODUCT_FAMILY_MANIFEST[10],
        CanonicalProductFamily::AssociatedAndStaticSemantics => CANONICAL_PRODUCT_FAMILY_MANIFEST[11],
        CanonicalProductFamily::FrozenStructuralQueries => CANONICAL_PRODUCT_FAMILY_MANIFEST[12],
        CanonicalProductFamily::RuntimePublicMetadata => CANONICAL_PRODUCT_FAMILY_MANIFEST[13],
        CanonicalProductFamily::CompilerSourceAttachments => CANONICAL_PRODUCT_FAMILY_MANIFEST[14],
        CanonicalProductFamily::EditorPresentation => CANONICAL_PRODUCT_FAMILY_MANIFEST[15],
        CanonicalProductFamily::AdvisoryProducts => CANONICAL_PRODUCT_FAMILY_MANIFEST[16],
        CanonicalProductFamily::SourceMaps => CANONICAL_PRODUCT_FAMILY_MANIFEST[17],
        CanonicalProductFamily::Diagnostics => CANONICAL_PRODUCT_FAMILY_MANIFEST[18],
        CanonicalProductFamily::WorkspaceRevisionHistory => CANONICAL_PRODUCT_FAMILY_MANIFEST[19],
        CanonicalProductFamily::MutableSemanticDatabase => CANONICAL_PRODUCT_FAMILY_MANIFEST[20],
        CanonicalProductFamily::RuntimeHandles => CANONICAL_PRODUCT_FAMILY_MANIFEST[21],
    }
}

/// Exhaustive query policy. Adding a `QueryKind` requires an explicit artifact
/// disposition here even when the family is not a reusable query root.
pub const fn query_kind_disposition(kind: QueryKind) -> QueryKindDispositionRow {
    let in_memory_persistence = crate::canonical_image::canonical_query_persistence(kind);
    let artifact_disposition = match kind {
        QueryKind::ParsedModule => QueryArtifactDisposition::OmittedSourceDependent,
        QueryKind::UnlinkedInterface
        | QueryKind::ResolvedImports
        | QueryKind::LinkedInterface
        | QueryKind::DeclarationShell
        | QueryKind::DeclarationSurface
        | QueryKind::TraitHeader
        | QueryKind::TraitSurface
        | QueryKind::CallableDefinition
        | QueryKind::HierarchyEdge
        | QueryKind::FieldSignature
        | QueryKind::CallableSignature
        | QueryKind::DataDeclaration
        | QueryKind::EnumDeclaration
        | QueryKind::TypeMemberDeclaration
        | QueryKind::AssociatedSurface
        | QueryKind::ModuleStaticBindingCatalog
        | QueryKind::ModuleStaticBindings => QueryArtifactDisposition::ReusableRoot,
        QueryKind::ConformanceDispatch
        | QueryKind::TraitDispatchBucket
        | QueryKind::GenericConformance
        | QueryKind::CallableBody
        | QueryKind::CheckedStaticBody => QueryArtifactDisposition::OmittedOpenWorld,
        QueryKind::SemanticComponent
        | QueryKind::ConstructorInitialization
        | QueryKind::CallableEffects
        | QueryKind::CallableControl
        | QueryKind::CallableTermination
        | QueryKind::CallableContracts
        | QueryKind::VerificationConditions
        | QueryKind::EnumRequirements
        | QueryKind::StaticDomainCaseCatalog
        | QueryKind::StaticCaseSignature
        | QueryKind::AssociatedStaticDomainDeclaration
        | QueryKind::ResolvedImport
        | QueryKind::LinkedName
        | QueryKind::PublicExport
        | QueryKind::StaticDefinition
        | QueryKind::StaticConstant => QueryArtifactDisposition::AggregateDependencyEvidence,
        QueryKind::SourceStructure | QueryKind::SourceFormalAttachment => QueryArtifactDisposition::OmittedSourceDependent,
        QueryKind::AdvisoryCallable | QueryKind::AdvisoryModule | QueryKind::ModuleDiagnostics | QueryKind::ModuleMetadata => {
            QueryArtifactDisposition::OmittedNonCompilerProduct
        }
    };
    QueryKindDispositionRow {
        kind,
        in_memory_persistence,
        artifact_disposition,
    }
}

/// Required source-independent replacements and deliberately excluded source
/// products. The checked evaluator programs remain required even though their
/// current in-process representation includes AST nodes.
pub const COMPILER_ATTACHMENT_MANIFEST: &[CompilerAttachmentDispositionRow] = &[
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::AcceptedCallableDefinition,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::callable_definitions",
        },
        compiler_readers: &["ModuleLoweringInputView", "ProgramCompiler::compile_analyzed"],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::CheckedStaticDefinitionProgram,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::checked_static_programs",
        },
        compiler_readers: &["EvaluatorBody::root_work", "StaticEvalMachine::run"],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::CheckedStaticConstantProgram,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::checked_static_programs",
        },
        compiler_readers: &["EvaluatorBody::root_work", "StaticEvalMachine::run"],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::NormalizedGenericConstraints,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::generic_constraints",
        },
        compiler_readers: &["TypeResolver", "ConformanceResolver", "StaticEvalMachine"],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::LogicalSourceLocations,
        disposition: FieldDisposition::Wire {
            section: "SEMANTIC_WORLD::logical_locations",
        },
        compiler_readers: &["SemanticDiagnostic", "StaticEvalTraceFrame"],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::SourceFormalFacts,
        disposition: FieldDisposition::Derived {
            owner: "P7.T8::relocated_callable_analysis_and_formal_projection",
        },
        compiler_readers: &["ModuleLoweringInputView", "StaticEvalMachine"],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::ParsedModuleSyntax,
        disposition: FieldDisposition::Omitted {
            reason: "raw ParsedModuleUnit and AST syntax are replaced by checked operational templates",
        },
        compiler_readers: &[],
    },
    CompilerAttachmentDispositionRow {
        family: CompilerAttachmentFamily::EditorSourceIndex,
        disposition: FieldDisposition::Omitted {
            reason: "editor presentation requires a separate optional source capability",
        },
        compiler_readers: &["SourceSemanticIndex"],
    },
];

/// Source-independent attachment contract for `CompilerSourceAttachments`,
/// checked-static-body consumers, and normalized constraint syntax consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicalSourceAttachmentDto {
    pub owner_module_stable_id: u32,
    pub logical_file_symbol: u32,
    pub byte_start: u32,
    pub byte_end: u32,
    pub line_one_based: u32,
    pub column_one_based: u32,
    pub attachment_capability_flags: u16,
}

pub const CHECKED_STATIC_BODY_CONSUMERS: &[&str] = &[
    "StaticEvaluator",
    "StaticApplicationEvaluator",
    "StaticConstantDemandEvaluator",
    "TypeChecker",
    "DiagnosticEmitter",
];

pub const CONSTRAINT_SYNTAX_CONSUMERS: &[&str] = &["TypeFormation", "SubtypingEngine", "TraitSolver", "GenericSignatureResolver"];

pub const COMPILER_SOURCE_ATTACHMENT_CONSUMERS: &[&str] = &["DiagnosticEmitter", "LspDefinitionProvider", "LspHoverProvider", "LspReferencesProvider"];
