use serde::{Deserialize, Serialize};

/// Runtime collection level for one isolated profiling run.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservabilityLevel {
    Off,
    Summary,
    Detailed,
    Trace,
}

impl ObservabilityLevel {
    pub fn collects_summary(self) -> bool {
        self >= Self::Summary
    }

    pub fn collects_details(self) -> bool {
        self >= Self::Detailed
    }
}

/// Stable top-level and canonical-work phases.
///
/// Serialized spellings are part of the report schema. Additions are compatible;
/// renames require an explicit schema decision.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum PerfPhase {
    #[serde(rename = "compilation")]
    Compilation,
    #[serde(rename = "source_discovery")]
    SourceDiscovery,
    #[serde(rename = "source_load")]
    SourceLoad,
    #[serde(rename = "parse")]
    Parse,
    #[serde(rename = "interface_build")]
    InterfaceBuild,
    #[serde(rename = "import_resolution")]
    ImportResolution,
    #[serde(rename = "link")]
    Link,
    #[serde(rename = "canonical_image.cache_lookup")]
    CanonicalImageCacheLookup,
    #[serde(rename = "canonical_image.build")]
    CanonicalImageBuild,
    #[serde(rename = "canonical_image.bootstrap_semantic_update")]
    CanonicalImageBootstrap,
    #[serde(rename = "canonical_image.transaction_commit")]
    CanonicalImageTransactionCommit,
    #[serde(rename = "canonical_image.snapshot_validate")]
    CanonicalImageValidate,
    #[serde(rename = "canonical_image.query_freeze")]
    CanonicalImageQueryFreeze,
    #[serde(rename = "canonical_image.core_consumer")]
    CanonicalImageCoreConsumer,
    #[serde(rename = "canonical_universe.total")]
    CanonicalUniverse,
    #[serde(rename = "canonical_universe.build")]
    CanonicalUniverseBuild,
    #[serde(rename = "canonical_universe.source_index")]
    CanonicalSourceIndex,
    #[serde(rename = "canonical_universe.native_verification")]
    CanonicalNativeVerification,
    #[serde(rename = "canonical_universe.interface_loading")]
    CanonicalInterfaceLoading,
    #[serde(rename = "canonical_universe.import_resolution")]
    CanonicalImportResolution,
    #[serde(rename = "canonical_universe.linking")]
    CanonicalLinking,
    #[serde(rename = "canonical_universe.semantic.total")]
    CanonicalSemanticAnalysis,
    #[serde(rename = "canonical_universe.match_validation")]
    CanonicalMatchValidation,
    #[serde(rename = "canonical_universe.compiler_projection.total")]
    CanonicalCompilerProjection,
    #[serde(rename = "canonical_universe.bootstrap_planning")]
    CanonicalBootstrapPlanning,
    #[serde(rename = "canonical_universe.product_validation")]
    CanonicalProductValidation,
    #[serde(rename = "semantic.canonical_baseline.total")]
    CanonicalSemanticBaseline,
    #[serde(rename = "semantic.canonical_baseline.linked_world")]
    CanonicalBaselineLinkedWorld,
    #[serde(rename = "semantic.canonical_baseline.native_surfaces")]
    CanonicalBaselineNativeSurfaces,
    #[serde(rename = "semantic.canonical_baseline.predeclaration")]
    CanonicalBaselinePredeclaration,
    #[serde(rename = "semantic.canonical_baseline.trait_census")]
    CanonicalBaselineTraitCensus,
    #[serde(rename = "semantic.canonical_baseline.hierarchy")]
    CanonicalBaselineHierarchy,
    #[serde(rename = "semantic.canonical_baseline.class_surfaces")]
    CanonicalBaselineClassSurfaces,
    #[serde(rename = "semantic.canonical_baseline.callable_signatures")]
    CanonicalBaselineCallableSignatures,
    #[serde(rename = "semantic.canonical_baseline.data")]
    CanonicalBaselineData,
    #[serde(rename = "semantic.canonical_baseline.enums")]
    CanonicalBaselineEnums,
    #[serde(rename = "semantic.canonical_baseline.inherent_impls")]
    CanonicalBaselineInherentImpls,
    #[serde(rename = "semantic.canonical_baseline.effective_surfaces")]
    CanonicalBaselineEffectiveSurfaces,
    #[serde(rename = "semantic.canonical_baseline.enum_behavior")]
    CanonicalBaselineEnumBehavior,
    #[serde(rename = "semantic.canonical_baseline.enum_requirements")]
    CanonicalBaselineEnumRequirements,
    #[serde(rename = "semantic.canonical_baseline.associated_surfaces")]
    CanonicalBaselineAssociatedSurfaces,
    #[serde(rename = "semantic.canonical_baseline.type_members")]
    CanonicalBaselineTypeMembers,
    #[serde(rename = "semantic.session_init")]
    SemanticSessionInit,
    #[serde(rename = "semantic.update.total")]
    SemanticUpdate,
    #[serde(rename = "semantic.update.inputs")]
    SemanticUpdateInputs,
    #[serde(rename = "semantic.update.invalidation")]
    SemanticUpdateInvalidation,
    #[serde(rename = "semantic.update.structural_shards")]
    SemanticUpdateStructuralShards,
    #[serde(rename = "semantic.update.dependency_graphs")]
    SemanticUpdateDependencyGraphs,
    #[serde(rename = "semantic.update.generic_headers")]
    SemanticUpdateGenericHeaders,
    #[serde(rename = "semantic.update.declarations")]
    SemanticUpdateDeclarations,
    #[serde(rename = "semantic.update.aliases")]
    SemanticUpdateAliases,
    #[serde(rename = "semantic.update.hierarchy")]
    SemanticUpdateHierarchy,
    #[serde(rename = "semantic.update.callable_signatures")]
    SemanticUpdateCallableSignatures,
    #[serde(rename = "semantic.update.field_signatures")]
    SemanticUpdateFieldSignatures,
    #[serde(rename = "semantic.update.declaration_surfaces")]
    SemanticUpdateDeclarationSurfaces,
    #[serde(rename = "semantic.update.data")]
    SemanticUpdateData,
    #[serde(rename = "semantic.update.enums")]
    SemanticUpdateEnums,
    #[serde(rename = "semantic.update.conformances")]
    SemanticUpdateConformances,
    #[serde(rename = "semantic.update.associated")]
    SemanticUpdateAssociated,
    #[serde(rename = "semantic.update.type_members")]
    SemanticUpdateTypeMembers,
    #[serde(rename = "semantic.update.callable_bodies.initial")]
    SemanticUpdateCallableBodiesInitial,
    #[serde(rename = "semantic.update.callable_bodies.downstream")]
    SemanticUpdateCallableBodiesDownstream,
    #[serde(rename = "semantic.update.callable_bodies.cycle")]
    SemanticUpdateCallableBodiesCycle,
    #[serde(rename = "semantic.update.callable_bodies.conformance")]
    SemanticUpdateCallableBodiesConformance,
    #[serde(rename = "semantic.update.callable_bodies.final_revalidation")]
    SemanticUpdateCallableBodiesFinalRevalidation,
    #[serde(rename = "semantic.update.module_bodies")]
    SemanticUpdateModuleBodies,
    #[serde(rename = "semantic.update.static_definitions")]
    SemanticUpdateStaticDefinitions,
    #[serde(rename = "semantic.update.static_constants")]
    SemanticUpdateStaticConstants,
    #[serde(rename = "semantic.update.source_index")]
    SemanticUpdateSourceIndex,
    #[serde(rename = "semantic.update.formal_projection")]
    SemanticUpdateFormalProjection,
    #[serde(rename = "semantic.update.advisory")]
    SemanticUpdateAdvisory,
    #[serde(rename = "semantic.update.diagnostics")]
    SemanticUpdateDiagnostics,
    #[serde(rename = "semantic.update.snapshot_publication")]
    SemanticSnapshotPublication,
    #[serde(rename = "semantic.update.effects")]
    SemanticUpdateEffects,
    #[serde(rename = "semantic.db.scheduler_snapshot")]
    SemanticDbSchedulerSnapshot,
    #[serde(rename = "semantic.type_store.baseline_fork")]
    TypeStoreBaselineFork,
    #[serde(rename = "compiler_projection.total")]
    CompilerProjection,
    #[serde(rename = "compiler_projection.runtime_root_discovery")]
    CompilerProjectionRuntimeRootDiscovery,
    #[serde(rename = "compiler_projection.metadata_export")]
    CompilerProjectionMetadataExport,
    #[serde(rename = "compiler_projection.runtime_type_recipes")]
    CompilerProjectionRuntimeTypeRecipes,
    #[serde(rename = "compiler_projection.generic_evidence")]
    CompilerProjectionGenericEvidence,
    #[serde(rename = "compiler_projection.conditional_calls")]
    CompilerProjectionConditionalCalls,
    #[serde(rename = "compiler_projection.trait_environments")]
    CompilerProjectionTraitEnvironments,
    #[serde(rename = "compiler_projection.module_lowering")]
    CompilerProjectionModuleLowering,
    #[serde(rename = "compiler_projection.materialization_plans")]
    CompilerProjectionMaterializationPlans,
    #[serde(rename = "metadata_export.total")]
    MetadataExport,
    #[serde(rename = "metadata_export.project_identity_validation")]
    MetadataExportProjectIdentityValidation,
    #[serde(rename = "metadata_export.declarations")]
    MetadataExportDeclarations,
    #[serde(rename = "metadata_export.aliases")]
    MetadataExportAliases,
    #[serde(rename = "metadata_export.callables")]
    MetadataExportCallables,
    #[serde(rename = "metadata_export.fields")]
    MetadataExportFields,
    #[serde(rename = "metadata_export.runtime_roots")]
    MetadataExportRuntimeRoots,
    #[serde(rename = "metadata_export.static_domains")]
    MetadataExportStaticDomains,
    #[serde(rename = "metadata_export.fingerprints")]
    MetadataExportFingerprints,
    #[serde(rename = "semantic_lowering.total")]
    SemanticLowering,
    #[serde(rename = "vm.bootstrap")]
    VmBootstrap,
    #[serde(rename = "vm.kernel")]
    VmKernel,
    #[serde(rename = "vm.native")]
    VmNative,
    #[serde(rename = "universe.vm_compilation")]
    UniverseVmCompilation,
    #[serde(rename = "universe.initializer_execution")]
    UniverseInitializerExecution,
    #[serde(rename = "user_execution")]
    UserExecution,
    #[serde(rename = "other")]
    Other,
}

/// Result of one measured phase.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerfOutcome {
    Success,
    SemanticError,
    ParseError,
    LinkError,
    BootstrapError,
    Cancelled,
    BudgetExceeded,
    Blocked,
    PanicBoundary,
    OtherFailure,
}

/// Workload scope for attribution without depending on compiler-owned IDs.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerfScope {
    Universe,
    CurrentProject,
    DependencyProject,
    SyntheticStandalone,
    Workspace,
}

/// Lightweight attribution keys supplied by the owning compiler layer.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PerfAttribution {
    pub scope: PerfScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,
}

impl PerfAttribution {
    pub fn workspace() -> Self {
        Self {
            scope: PerfScope::Workspace,
            module: None,
            entity: None,
        }
    }

    pub fn module(scope: PerfScope, module: impl Into<String>) -> Self {
        Self {
            scope,
            module: Some(module.into()),
            entity: None,
        }
    }

    pub fn entity(scope: PerfScope, module: Option<String>, entity: impl Into<String>) -> Self {
        Self {
            scope,
            module,
            entity: Some(entity.into()),
        }
    }
}
