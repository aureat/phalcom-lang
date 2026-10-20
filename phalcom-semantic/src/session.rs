//! Compiler-owned incremental workspace session (Spec 04.5 / Wave 5 / Tasks 16-18).

use crate::advisory::{
    AdvisoryBuiltins, AdvisoryCallableSummary, AdvisoryConfidence, AdvisoryFact, AdvisoryFlowContext, AdvisoryModuleProduct, AdvisoryOrigin,
    AdvisoryProductStatus, AdvisorySolver, AdvisorySolverBudget, AdvisorySolverNode, AdvisoryTargetResolution, AdvisoryWorkspace, CallableForShapeResolver,
    FormalCallResultResolver, MethodFamilyResolver, ModuleMemberResolver, advisory_shape_from_formal, advisory_shape_from_formal_for_receiver, analyze_expr,
    analyze_statements,
};
use crate::associated::AssociatedFamilyTable;
use crate::checker::analysis::normal_return_summary;
use crate::checker::context::CheckingContext;
use crate::checker::declaration::check_class_field_initializers;
use crate::checker::statement::check_statement;
use crate::data_semantics::DataSemanticTable;
use crate::db::SemanticDb;
use crate::db::budget::{BudgetReport, CancellationToken, QueryBudget};
use crate::db::key::QueryKey;
use crate::db::product::EnumRequirementsProduct;
use crate::db::query::{
    CallableBodyQuery, DeclarationSurfaceQuery, FormalQueryInputs, HierarchyEdgeQueryInputs, SignatureQueryInputs, bootstrap_advisory_callable,
    query_advisory_callable, query_advisory_module, query_associated_surface, query_bootstrap_callable_signature, query_bootstrap_declaration_surface,
    query_bootstrap_hierarchy_edge, query_callable_body_with_formal_inputs, query_callable_definition, query_callable_signature_with_inputs,
    query_constructor_initialization_owner, query_data_declaration, query_declaration_shell, query_declaration_surface, query_enum_declaration,
    query_enum_requirements, query_field_signature_with_inputs, query_hierarchy_edge, query_linked_interface, query_source_formal_attachment,
    query_source_structure, query_static_case_signature, query_static_definition, query_static_domain_case_catalog, query_type_member_declaration,
    query_unlinked_interface,
};
use crate::db::state::QueryOutcome;
use crate::declarations::{DeclarationTypeInfo, DeclarationTypeTable, GenericSupertypeTemplate, NominalDeclarationHeader, TypeDeclarationShell};
use crate::diagnostic::{DiagnosticCode, DiagnosticLabel, SemanticDiagnostic, SemanticSourceSpan};
use crate::dispatch::SurfaceDispatchResolver;
use crate::enum_requirements::{EnumRequirementTable, check_enum_requirements};
use crate::enum_semantics::{EnumSemanticTable, VariantInfo};
use crate::identity::{CallableId, DeclarationId, DispatchSide, FieldId, ModuleId, SemanticTargetId, SourceOwner, SourceSiteId, WorkspaceId};
use crate::presentation::FormalSemanticProjection;
use crate::request::{AdvisoryLevel, PresentationLevel, SemanticProductRequest, SourceSemanticLevel};
use crate::resolver::LinkedTypeResolver;
use crate::semantic_shard::ModuleSemanticStructureShard;
use crate::signature::{
    CallableParameterSemantic, CallableSemanticSignature, CallableSignatureTable, ConstructorInitializationProduct, ConstructorInitializationTable,
    ConstructorOrigin, ConstructorParameterField, FieldSignatureTable, ReturnContractValidation,
};
use crate::snapshot::SemanticSnapshot;
use crate::source::ParsedModuleUnit;
use crate::source_index::{
    SourceIndexContext, SourceSemanticIndex, build_source_scope_index, resolve_type_member_projection_targets, resolve_type_parameter_source_targets,
    resolve_type_reference_targets,
};
use crate::static_definition::{StaticDefinitionTable, resolve_static_definition};
use crate::traits::{TraitHeader, TraitRef, TraitSurfaceTable, build_trait_surface};
use crate::type_alias::TypeAliasTable;
use crate::types::annotation::{
    GenericBinderSite, TypeFormationOutcome, TypeFormationSite, TypeResolver, resolve_generic_signature, resolve_kind_syntax, type_level_binding_for_parameter,
};
use crate::types::id::{KindId, TypeId};
use crate::types::outcome::BlockReason;
use crate::types::parameter::{GenericBinderHeader, GenericSignature, TypeParameterOwner};
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::TypeStore;
use crate::workspace::SemanticWorkspaceInput;
use phalcom_ast::ast::{ClassMember, PackItem, PackLabel, Statement, TraitMember, TypeAnnotation, TypeAnnotationExpr};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::declaration::{DeclarationBlueprint, DeclarationKind, DeclarationRealizationError, DeclarationShellTable};
use phalcom_modules::graph::{SemanticEdge, SemanticEdgeKind, SemanticNodeId};
use phalcom_modules::interface::{InterfaceBuilder, LinkedExportTarget};
use phalcom_modules::linker::{LinkedProgram, SymbolId};
use phalcom_modules::{WorkspaceModuleSession, WorkspaceModuleSessionError, WorkspaceModuleUpdate, WorkspaceSourceBatchMutation, WorkspaceSourceMutation};
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, OnceLock};

static CANONICAL_LINKED_UNIVERSE: OnceLock<Arc<LinkedProgram>> = OnceLock::new();

pub(crate) fn canonical_universe_linked_program(provider: &phalcom_modules::UniverseSourceProvider) -> Arc<LinkedProgram> {
    if provider.is_canonical() {
        Arc::clone(CANONICAL_LINKED_UNIVERSE.get_or_init(|| build_canonical_universe_linked_program(provider)))
    } else {
        build_canonical_universe_linked_program(provider)
    }
}

fn build_canonical_universe_linked_program(provider: &phalcom_modules::UniverseSourceProvider) -> Arc<LinkedProgram> {
    let universe = Arc::new(phalcom_modules::project::ProjectUniverse::new());
    let filesystem = phalcom_modules::FilesystemSourceProvider::new();
    let mut module_resolver = phalcom_modules::ModuleResolver::with_universe_source_provider(&universe, &filesystem, provider.clone());
    let mut interfaces = BTreeMap::new();

    for node in provider.nodes() {
        let path = phalcom_modules::ModulePath::from_components(
            node.path
                .iter()
                .map(|component| phalcom_modules::ModuleComponent::from_identifier(component).expect("canonical Universe component"))
                .collect::<Vec<_>>(),
        );
        let module = phalcom_modules::ModuleId::universe(path);
        let interface = provider
            .load_interface(&module)
            .unwrap_or_else(|error| panic!("failed to load canonical Universe interface {module}: {error}"));
        interfaces.insert(module, interface);
    }

    let mut resolved = BTreeMap::new();
    for (module, interface) in &interfaces {
        for import in &interface.imports {
            let path = match import {
                phalcom_modules::ImportSurface::Module(decl) => &decl.path,
                phalcom_modules::ImportSurface::Selective(decl) => &decl.path,
                phalcom_modules::ImportSurface::ReExport(decl) => &decl.path,
            };
            let target = module_resolver
                .resolve_import(module, path)
                .unwrap_or_else(|error| panic!("failed to resolve canonical Universe import {module}::{path}: {error}"));
            resolved.insert((module.clone(), path.to_string()), target.id);
        }
    }

    Arc::new(
        phalcom_modules::ModuleLinker::new(universe, interfaces)
            .link_all(phalcom_modules::ModuleId::universe_root(), &resolved)
            .unwrap_or_else(|error| panic!("failed to link canonical Universe semantic baseline: {error}")),
    )
}

fn canonical_linked_program_for_session(db: &SemanticDb, provider: &phalcom_modules::UniverseSourceProvider) -> Arc<LinkedProgram> {
    if let Some(image) = db.canonical_image() {
        Arc::clone(image.linked())
    } else {
        canonical_universe_linked_program(provider)
    }
}

/// Performance and recomputation metrics for one semantic workspace update.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SemanticUpdateStats {
    pub modules_recomputed: usize,
    pub callables_recomputed: usize,
    pub callables_reused: usize,
    pub project_graph_rebuilt: bool,
    pub modules_relinked: usize,
    pub source_indexes_recomputed: usize,
    pub advisory_sources_recomputed: usize,
    pub advisory_callables_recomputed: usize,
    pub semantic_structure_shards_recomputed: usize,
    pub semantic_structure_shards_reused: usize,
    pub declaration_products_recomputed: usize,
    pub declaration_products_reused: usize,
    pub hierarchy_edges_recomputed: usize,
    pub hierarchy_edges_reused: usize,
    pub alias_regions_recomputed: usize,
    pub alias_regions_reused: usize,
    pub alias_dependency_nodes_considered: usize,
    pub callable_signatures_recomputed: usize,
    pub callable_signatures_reused: usize,
    pub field_signatures_recomputed: usize,
    pub field_signatures_reused: usize,
    pub callable_bodies_recomputed: usize,
    pub callable_bodies_reused: usize,
    pub query_products_recomputed: usize,
    pub query_products_revalidated: usize,
    pub exact_name_products_recomputed: usize,
    pub exact_name_products_reused: usize,
    /// Number of linked importer modules inspected while seeding semantic work.
    pub linked_importers_considered: usize,
    pub reverse_candidates_considered: usize,
    pub semantic_dependents_recomputed: usize,
    pub semantic_dependents_reused: usize,
    /// Whole snapshot maps materialized while freezing this generation.
    pub snapshot_full_map_materializations: usize,
    /// Snapshot roots retained by pointer identity from the previous
    /// generation.
    pub snapshot_roots_retained: usize,
    /// Snapshot roots replaced for this generation.
    pub snapshot_roots_replaced: usize,
    /// Canonical TypeStore nodes copied while publishing this generation.
    pub type_store_snapshot_deep_copies: usize,
    /// Workspace-wide advisory fact flattening operations.
    pub advisory_flattening_work: usize,
    /// Modules considered for diagnostic publication effects.
    pub diagnostic_frontier_work: usize,
}

fn semantic_diagnostics_from_module_diagnostics(diagnostics: &[phalcom_modules::diagnostic::ModuleDiagnostic]) -> Vec<SemanticDiagnostic> {
    diagnostics
        .iter()
        .map(|diag| {
            let code = match diag.code() {
                phalcom_modules::diagnostic::ModuleDiagnosticCode::ImportUnresolved => DiagnosticCode::ModuleImportUnresolved,
                phalcom_modules::diagnostic::ModuleDiagnosticCode::ExposureRejected => DiagnosticCode::ModuleExposureRejected,
                phalcom_modules::diagnostic::ModuleDiagnosticCode::ExportMissing => DiagnosticCode::ModuleExportMissing,
                phalcom_modules::diagnostic::ModuleDiagnosticCode::RelativeInvalidRoot => DiagnosticCode::ModuleRelativeInvalidRoot,
                phalcom_modules::diagnostic::ModuleDiagnosticCode::LinkFailed => DiagnosticCode::ModuleLinkFailed,
                phalcom_modules::diagnostic::ModuleDiagnosticCode::InterfaceFailed => DiagnosticCode::ModuleInterfaceFailed,
                phalcom_modules::diagnostic::ModuleDiagnosticCode::RuntimeCycle => DiagnosticCode::ModuleRuntimeCycle,
            };
            SemanticDiagnostic::error_in(diag.module.clone(), code, diag.message.clone(), diag.range)
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CallableRevisionDisposition {
    Reused,
    Recomputed,
}

fn retain_generic_signature(
    outcome: TypeFormationOutcome<GenericSignature>,
    module: &ModuleId,
    range: SourceRange,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> Option<GenericSignature> {
    match outcome {
        TypeFormationOutcome::Ready(signature) => Some(signature),
        TypeFormationOutcome::Dynamic => {
            diagnostics.push(SemanticDiagnostic::error_in(
                module.clone(),
                DiagnosticCode::AnnotationUnsupported,
                "generic signature depends on a dynamic type-form boundary",
                range,
            ));
            None
        }
        TypeFormationOutcome::Missing(reason) => {
            diagnostics.push(SemanticDiagnostic::error_in(
                module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                format!("generic signature publication missing: {reason:?}"),
                range,
            ));
            None
        }
        TypeFormationOutcome::Unresolved(_) | TypeFormationOutcome::Invalid(_) => None,
        TypeFormationOutcome::Blocked(reason) => {
            diagnostics.push(SemanticDiagnostic::error_in(
                module.clone(),
                DiagnosticCode::AnalysisBlocked,
                format!("generic signature publication blocked: {reason:?}"),
                range,
            ));
            None
        }
        TypeFormationOutcome::Cancelled => {
            diagnostics.push(SemanticDiagnostic::error_in(
                module.clone(),
                DiagnosticCode::AnalysisBlocked,
                "generic signature publication cancelled",
                range,
            ));
            None
        }
        TypeFormationOutcome::BudgetExceeded(report) => {
            diagnostics.push(SemanticDiagnostic::error_in(
                module.clone(),
                DiagnosticCode::AnalysisBudgetExceeded,
                format!("generic signature publication exceeded budget: {report:?}"),
                range,
            ));
            None
        }
        TypeFormationOutcome::InternalFailure(failure) => {
            diagnostics.push(SemanticDiagnostic::error_in(
                module.clone(),
                DiagnosticCode::AnalysisInternalFailure,
                format!("generic signature publication failed: {failure}"),
                range,
            ));
            None
        }
    }
}

fn predeclare_nominal_binder_header(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    module: &ModuleId,
    declaration: &DeclarationId,
    parameters: &[phalcom_ast::ast::GenericParameterSyntax],
    where_clause: Option<&phalcom_ast::ast::WhereClauseSyntax>,
) -> Option<GenericBinderHeader> {
    if parameters.is_empty() {
        return None;
    }
    let mut diagnostics = Vec::new();
    match crate::types::annotation::resolve_generic_binder_header_with_context(
        store,
        &TypeFormationSite::member(module.clone(), declaration.clone(), DispatchSide::Instance),
        TypeParameterOwner::Declaration(declaration.clone()),
        GenericBinderSite::NominalDeclaration,
        parameters,
        where_clause,
        &mut diagnostics,
        Some(declarations),
        Some(resolver),
    ) {
        TypeFormationOutcome::Ready(header) => Some(header),
        _ => None,
    }
}

fn resolve_missing_nominal_binder_headers(
    store: &mut TypeStore,
    declarations: &mut DeclarationTypeTable,
    resolver: &dyn TypeResolver,
    shards: &BTreeMap<ModuleId, Arc<ModuleSemanticStructureShard>>,
    invalid_headers: &BTreeSet<DeclarationId>,
) -> bool {
    let mut any_progress = false;
    loop {
        let mut pass_progress = false;
        for (module_id, shard) in shards {
            for statement in &shard.source.program.statements {
                let (declaration, parameters, where_clause, result_kind) = match statement {
                    Statement::Class(class_def) => (
                        DeclarationId::new(module_id.clone(), class_def.name.clone().into()),
                        class_def.generic_parameters.as_slice(),
                        class_def.where_clause.as_ref(),
                        KindId::TYPE,
                    ),
                    Statement::Data(data_def) => {
                        let declaration = DeclarationId::new(module_id.clone(), data_def.name.clone().into());
                        let result_kind = if data_def.phase == phalcom_ast::ast::AlgebraicPhase::TypeOnly {
                            let domain = store.nominal_static_domain(declaration.clone());
                            store.static_domain_kind(domain)
                        } else {
                            KindId::TYPE
                        };
                        (declaration, data_def.generic_parameters.as_slice(), data_def.where_clause.as_ref(), result_kind)
                    }
                    _ => continue,
                };
                if parameters.is_empty() || invalid_headers.contains(&declaration) || declarations.generic_binder_header(&declaration).is_some() {
                    continue;
                }
                let Some(mut info) = declarations.get(&declaration).cloned() else {
                    continue;
                };
                let Some(header) = predeclare_nominal_binder_header(store, declarations, resolver, module_id, &declaration, parameters, where_clause) else {
                    continue;
                };
                let kind = store.arrow_kind(header.parameter_kinds.clone(), result_kind);
                info.kind = kind;
                info.form = store.nominal_form(declaration, kind);
                info.generic_binder_header = Some(header);
                declarations.refresh_shell_preserving_registered_projections(info);
                pass_progress = true;
            }
        }
        if !pass_progress {
            break;
        }
        any_progress = true;
    }
    any_progress
}

pub(crate) fn ready_kind_for_predeclaration(store: &mut TypeStore, parameter: &phalcom_ast::ast::GenericParameterSyntax) -> Option<KindId> {
    let kind_resolution = parameter.kind.as_ref().map_or_else(
        || {
            let kind = match parameter.mode {
                None => KindId::TYPE,
                Some(phalcom_ast::ast::ExpansionMode::Positional) => KindId::TYPE_PACK,
                Some(phalcom_ast::ast::ExpansionMode::Labeled) => KindId::RECORD_ROW,
                Some(phalcom_ast::ast::ExpansionMode::Complete) => KindId::COMPLETE_TYPE_PACK,
            };
            crate::types::annotation::KindResolution::Ready(kind)
        },
        |kind| resolve_kind_syntax(store, kind),
    );
    match kind_resolution {
        crate::types::annotation::KindResolution::Ready(kind) => Some(kind),
        crate::types::annotation::KindResolution::Dynamic
        | crate::types::annotation::KindResolution::Missing(_)
        | crate::types::annotation::KindResolution::Unresolved(_)
        | crate::types::annotation::KindResolution::Invalid(_)
        | crate::types::annotation::KindResolution::Blocked(_)
        | crate::types::annotation::KindResolution::Cancelled
        | crate::types::annotation::KindResolution::BudgetExceeded(_)
        | crate::types::annotation::KindResolution::InternalFailure(_) => None,
    }
}

fn canonical_runtime_support_superclass(root: &str, members: &[String]) -> Option<DeclarationId> {
    if !members.is_empty() {
        return None;
    }
    match root {
        "Some" => Some(crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Some)),
        "None" => Some(crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::None)),
        _ => None,
    }
}

/// Product-level effects of one immutable semantic publication.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SemanticPublicationEffects {
    /// Modules whose published diagnostics changed.
    pub diagnostics_changed: BTreeSet<ModuleId>,
    /// Modules whose source-site or occurrence products changed.
    pub source_index_changed: BTreeSet<ModuleId>,
    /// Modules whose formal products changed.
    pub formal_changed: BTreeSet<ModuleId>,
    /// Modules whose advisory products changed.
    pub advisory_changed: BTreeSet<ModuleId>,
    /// Whether the sorted declaration index changed.
    pub declaration_index_changed: bool,
    /// Whether the canonical module graph changed.
    pub module_graph_changed: bool,
}

/// The result of an incremental semantic workspace publication.
#[derive(Clone, Debug)]
pub struct SemanticWorkspacePublication {
    pub snapshot: Arc<SemanticSnapshot>,
    pub invalidated: Arc<[QueryKey]>,
    pub recomputed: Arc<[QueryKey]>,
    pub stats: SemanticUpdateStats,
    /// Module-layer work counts when publication came from module mutations.
    pub module_stats: Option<phalcom_modules::session::WorkspaceModuleStats>,
    pub effects: SemanticPublicationEffects,
}

/// Compatibility name for existing compiler tests and lower-level callers.
pub type SemanticWorkspaceUpdate = SemanticWorkspacePublication;

/// Operational failure that prevented a semantic workspace publication.
///
/// This is distinct from source diagnostics: failures never produce a new
/// authoritative snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SemanticUpdateFailure {
    Cancelled,
    BudgetExceeded(BudgetReport),
    Blocked(Box<BlockReason>),
    Failed(String),
}

impl From<QueryOutcome<()>> for SemanticUpdateFailure {
    fn from(outcome: QueryOutcome<()>) -> Self {
        match outcome {
            QueryOutcome::Cancelled => Self::Cancelled,
            QueryOutcome::BudgetExceeded(report) => Self::BudgetExceeded(report),
            QueryOutcome::Blocked(reason) => Self::Blocked(Box::new(reason)),
            QueryOutcome::Failed(failure) => Self::Failed(failure),
            QueryOutcome::Ready(()) => Self::Failed("unexpected ready outcome in semantic error path".to_string()),
        }
    }
}

impl fmt::Display for SemanticUpdateFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => formatter.write_str("semantic update cancelled"),
            Self::BudgetExceeded(report) => write!(formatter, "semantic update budget exceeded: {report:?}"),
            Self::Blocked(reason) => write!(formatter, "semantic update blocked: {reason:?}"),
            Self::Failed(failure) => write!(formatter, "semantic update failed: {failure}"),
        }
    }
}

impl std::error::Error for SemanticUpdateFailure {}

/// Failure from an accepted module update or its semantic publication.
#[derive(Debug, thiserror::Error)]
pub enum SemanticWorkspaceUpdateError {
    #[error(transparent)]
    Module(#[from] WorkspaceModuleSessionError),
    #[error(transparent)]
    Semantic(#[from] SemanticUpdateFailure),
}

#[derive(Clone, Debug, Default)]
struct SemanticModuleDelta {
    changed_modules: BTreeSet<ModuleId>,
    removed_modules: BTreeSet<ModuleId>,
    identity_changes: BTreeSet<ModuleId>,
    module_stats: phalcom_modules::session::WorkspaceModuleStats,
    module_graph_changed: bool,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SemanticPublicationFailpoint {
    AfterRevisionBegin,
    AfterStructuralShards,
    AfterQueryRetirement,
    AfterSourceFingerprints,
    AfterReadyQueryPublish,
    AfterStaticValueMutation,
    BeforeCommit,
    CancelBeforeCommit,
}

#[derive(Clone, Debug, Default)]
struct SemanticContributionDelta {
    declarations: BTreeSet<DeclarationId>,
    declarations_removed: BTreeSet<DeclarationId>,
    hierarchy_edges: BTreeSet<DeclarationId>,
    callable_signatures: BTreeSet<CallableId>,
    callable_signatures_removed: BTreeSet<CallableId>,
    field_signatures: BTreeSet<FieldId>,
    field_signatures_removed: BTreeSet<FieldId>,
    callable_bodies: BTreeSet<CallableId>,
    callable_bodies_removed: BTreeSet<CallableId>,
    aliases: BTreeSet<DeclarationId>,
    structural_modules: BTreeSet<ModuleId>,
}

fn changed_fingerprinted_keys<K: Clone + Ord>(current: Option<&BTreeMap<K, u64>>, previous: Option<&BTreeMap<K, u64>>) -> BTreeSet<K> {
    let mut changed = BTreeSet::new();
    if let Some(current) = current {
        changed.extend(
            current
                .iter()
                .filter_map(|(key, fingerprint)| (previous.and_then(|old| old.get(key)) != Some(fingerprint)).then_some(key.clone())),
        );
    }
    if let Some(previous) = previous {
        changed.extend(previous.keys().filter(|key| current.is_none_or(|now| !now.contains_key(*key))).cloned());
    }
    changed
}

/// Replaces one forward dependency contribution while keeping its reverse
/// index exact. Both maps are updated as one logical operation so incremental
/// worklists can traverse consumers without scanning unrelated providers.
fn replace_reverse_dependencies<K: Clone + Ord>(
    forward: &mut BTreeMap<K, BTreeSet<K>>,
    reverse: &mut BTreeMap<K, BTreeSet<K>>,
    consumer: K,
    dependencies: BTreeSet<K>,
) {
    if let Some(previous) = forward.remove(&consumer) {
        for dependency in previous {
            if let Some(consumers) = reverse.get_mut(&dependency) {
                consumers.remove(&consumer);
                if consumers.is_empty() {
                    reverse.remove(&dependency);
                }
            }
        }
    }
    if dependencies.is_empty() {
        return;
    }
    for dependency in &dependencies {
        reverse.entry(dependency.clone()).or_default().insert(consumer.clone());
    }
    forward.insert(consumer, dependencies);
}

/// Removes one forward dependency contribution and all corresponding reverse
/// edges without inspecting unrelated consumers.
fn remove_reverse_dependencies<K: Clone + Ord>(forward: &mut BTreeMap<K, BTreeSet<K>>, reverse: &mut BTreeMap<K, BTreeSet<K>>, consumer: &K) {
    if let Some(previous) = forward.remove(consumer) {
        for dependency in previous {
            if let Some(consumers) = reverse.get_mut(&dependency) {
                consumers.remove(consumer);
                if consumers.is_empty() {
                    reverse.remove(&dependency);
                }
            }
        }
    }
}

fn removed_fingerprinted_keys<K: Clone + Ord>(current: Option<&BTreeMap<K, u64>>, previous: Option<&BTreeMap<K, u64>>) -> BTreeSet<K> {
    previous
        .into_iter()
        .flat_map(|old| old.keys())
        .filter(|key| current.is_none_or(|now| !now.contains_key(*key)))
        .cloned()
        .collect()
}

fn contribution_delta(
    current: &BTreeMap<ModuleId, Arc<ModuleSemanticStructureShard>>,
    previous: &BTreeMap<ModuleId, Arc<ModuleSemanticStructureShard>>,
) -> SemanticContributionDelta {
    let mut delta = SemanticContributionDelta::default();
    let modules = current.keys().chain(previous.keys()).cloned().collect::<BTreeSet<_>>();
    for module in modules {
        let now = current.get(&module);
        let old = previous.get(&module);
        if now.map(|shard| shard.source_fingerprint) != old.map(|shard| shard.source_fingerprint) {
            delta.structural_modules.insert(module.clone());
        }
        delta.declarations.extend(changed_fingerprinted_keys(
            now.map(|shard| shard.declaration_header_fingerprints.as_ref()),
            old.map(|shard| shard.declaration_header_fingerprints.as_ref()),
        ));
        delta.declarations_removed.extend(removed_fingerprinted_keys(
            now.map(|shard| shard.declaration_header_fingerprints.as_ref()),
            old.map(|shard| shard.declaration_header_fingerprints.as_ref()),
        ));
        delta.hierarchy_edges.extend(changed_fingerprinted_keys(
            now.map(|shard| shard.hierarchy_edge_fingerprints.as_ref()),
            old.map(|shard| shard.hierarchy_edge_fingerprints.as_ref()),
        ));
        delta.callable_signatures.extend(changed_fingerprinted_keys(
            now.map(|shard| shard.callable_signature_fingerprints.as_ref()),
            old.map(|shard| shard.callable_signature_fingerprints.as_ref()),
        ));
        delta.callable_signatures_removed.extend(removed_fingerprinted_keys(
            now.map(|shard| shard.callable_signature_fingerprints.as_ref()),
            old.map(|shard| shard.callable_signature_fingerprints.as_ref()),
        ));
        delta.field_signatures.extend(changed_fingerprinted_keys(
            now.map(|shard| shard.field_signature_fingerprints.as_ref()),
            old.map(|shard| shard.field_signature_fingerprints.as_ref()),
        ));
        delta.field_signatures_removed.extend(removed_fingerprinted_keys(
            now.map(|shard| shard.field_signature_fingerprints.as_ref()),
            old.map(|shard| shard.field_signature_fingerprints.as_ref()),
        ));
        delta.callable_bodies.extend(changed_fingerprinted_keys(
            now.map(|shard| shard.callable_body_fingerprints.as_ref()),
            old.map(|shard| shard.callable_body_fingerprints.as_ref()),
        ));
        delta.callable_bodies_removed.extend(removed_fingerprinted_keys(
            now.map(|shard| shard.callable_body_fingerprints.as_ref()),
            old.map(|shard| shard.callable_body_fingerprints.as_ref()),
        ));
        let current_aliases = now.map(|shard| shard.alias_sources.keys().cloned().collect::<BTreeSet<_>>());
        let previous_aliases = old.map(|shard| shard.alias_sources.keys().cloned().collect::<BTreeSet<_>>());
        match (current_aliases, previous_aliases) {
            (Some(now), Some(old)) => delta.aliases.extend(now.symmetric_difference(&old).cloned()),
            (Some(now), None) => delta.aliases.extend(now),
            (None, Some(old)) => delta.aliases.extend(old),
            (None, None) => {}
        }
    }
    delta
}

/// Compiler-owned stateful semantic workspace session.
///
/// Owns the canonical `SemanticDb`, interner `TypeStore`, dependency index,
/// and published immutable snapshots across source revisions.
#[derive(Debug)]
pub struct SemanticWorkspaceSession {
    workspace: WorkspaceId,
    module_session: WorkspaceModuleSession,
    universe_source_provider: phalcom_modules::UniverseSourceProvider,
    db: SemanticDb,
    store: Arc<TypeStore>,
    static_values: Arc<crate::static_value::StaticValueStore>,
    base_declarations: DeclarationTypeTable,
    base_hierarchy: MapTypeHierarchy,
    base_dispatch: SurfaceDispatchResolver,
    base_callable_signatures: CallableSignatureTable,
    base_data_semantics: DataSemanticTable,
    base_data_products: Vec<Arc<crate::db::product::DataDeclarationProduct>>,
    base_enum_semantics: EnumSemanticTable,
    base_enum_products: Vec<Arc<crate::db::product::EnumDeclarationProduct>>,
    base_associated_surfaces: AssociatedFamilyTable,
    base_associated_surface_products: Vec<Arc<crate::associated::AssociatedSurface>>,
    base_type_member_declarations: BTreeMap<crate::types::type_member::TypeMemberDeclarationId, Arc<crate::db::product::TypeMemberDeclarationProduct>>,
    base_type_member_products: Vec<Arc<crate::db::product::TypeMemberDeclarationProduct>>,
    base_enum_requirements: EnumRequirementTable,
    base_enum_requirement_products: Vec<(DeclarationId, Arc<crate::db::product::EnumRequirementsProduct>)>,
    sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    source_fingerprints: BTreeMap<ModuleId, u64>,
    field_lifecycle_fingerprints: BTreeMap<ModuleId, u64>,
    default_field_lifecycle: crate::checker::field_lifecycle::FieldLifecycleTable,
    semantic_structure_shards: BTreeMap<ModuleId, Arc<ModuleSemanticStructureShard>>,
    generic_header_dependencies: BTreeMap<DeclarationId, BTreeSet<DeclarationId>>,
    generic_header_reverse_dependencies: BTreeMap<DeclarationId, BTreeSet<DeclarationId>>,
    declaration_signature_dependencies: BTreeMap<TypeSignatureDependencyNode, BTreeSet<TypeSignatureDependencyNode>>,
    declaration_signature_reverse_dependencies: BTreeMap<TypeSignatureDependencyNode, BTreeSet<TypeSignatureDependencyNode>>,
    declaration_signature_fingerprints: BTreeMap<TypeSignatureDependencyNode, u64>,
    inherent_impl_surface_owners: BTreeMap<crate::identity::ImplId, DeclarationId>,
    alias_sources: BTreeMap<DeclarationId, (ModuleId, phalcom_ast::ast::TypeAliasDef)>,
    alias_dependencies: BTreeMap<DeclarationId, BTreeSet<DeclarationId>>,
    alias_reverse_dependencies: BTreeMap<DeclarationId, BTreeSet<DeclarationId>>,
    alias_contract_dependencies: BTreeMap<DeclarationId, BTreeSet<DeclarationId>>,
    alias_contract_reverse_dependencies: BTreeMap<DeclarationId, BTreeSet<DeclarationId>>,
    alias_forms: Arc<BTreeMap<DeclarationId, TypeId>>,
    module_diagnostics: BTreeMap<ModuleId, Vec<phalcom_modules::diagnostic::ModuleDiagnostic>>,
    canonical_native_diagnostics: BTreeMap<ModuleId, Vec<SemanticDiagnostic>>,
    semantic_diagnostic_contributions: BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>,
    last_snapshot: Option<Arc<SemanticSnapshot>>,
    last_known_good: Option<Arc<SemanticSnapshot>>,
    /// Module deltas whose semantic publication was cancelled and must be
    /// replayed with the next newest source update.
    pending_semantic_delta: Option<SemanticModuleDelta>,
    /// Optional deep-analysis frontier supplied by an interactive client.
    /// `None` preserves the normal whole-workspace semantic update behavior.
    requested_deep_modules: Option<BTreeSet<ModuleId>>,
    /// Optional semantic products requested by this consumer. Existing
    /// callers retain the full tooling publication contract by default.
    product_request: SemanticProductRequest,
    #[cfg(test)]
    test_publication_failpoint: std::sync::Mutex<Option<SemanticPublicationFailpoint>>,
    #[cfg(test)]
    test_candidate_type_parameter: std::sync::Mutex<Option<crate::types::parameter::TypeParameterData>>,
}

impl Default for SemanticWorkspaceSession {
    fn default() -> Self {
        Self::new()
    }
}

fn retire_blocked_declaration_shell_queries(db: &mut SemanticDb, blocked_declarations: &BTreeSet<DeclarationId>, type_aliases: &TypeAliasTable) {
    for declaration in blocked_declarations {
        if !type_aliases.contains_key(declaration) {
            db.retire_query(&QueryKey::DeclarationShell(declaration.clone()));
        }
    }
}

fn canonical_topology_for_input(input: &SemanticWorkspaceInput) -> Arc<phalcom_modules::topology::ModuleTopology> {
    if let Some(topology) = &input.topology {
        return topology.clone();
    }

    let mut unlinked = BTreeMap::new();
    for (module, interface) in input.interfaces.as_ref() {
        unlinked.insert(module.clone(), (**interface).clone());
    }
    for (module, unit) in &input.sources {
        if unlinked.contains_key(module) {
            continue;
        }
        if let Ok(interface) = InterfaceBuilder::build(module.clone(), unit.kind, &unit.program) {
            unlinked.insert(module.clone(), interface);
        }
    }
    let sources = input
        .sources
        .iter()
        .filter_map(|(module, unit)| unit.source.clone().map(|source| (module.clone(), source)))
        .collect();
    Arc::new(phalcom_modules::topology::ModuleTopology::from_parts(
        phalcom_modules::stabilization::ResolverGeneration(input.generation),
        &input.linked.universe,
        &unlinked,
        &sources,
    ))
}

impl SemanticWorkspaceSession {
    #[cfg(test)]
    fn apply_test_candidate_type_parameter(&mut self) {
        if let Some(data) = self.test_candidate_type_parameter.lock().expect("test candidate mutex poisoned").take() {
            Arc::make_mut(&mut self.store).intern_type_parameter(data);
        }
    }

    #[cfg(test)]
    fn fail_at_test_publication_point(&self, point: SemanticPublicationFailpoint) -> Result<(), QueryOutcome<()>> {
        let mut failpoint = self.test_publication_failpoint.lock().expect("test failpoint mutex poisoned");
        if *failpoint == Some(point) {
            *failpoint = None;
            return Err(QueryOutcome::Failed(format!("injected semantic publication failure at {point:?}")));
        }
        Ok(())
    }

    #[cfg(test)]
    fn cancel_at_test_publication_point(&self, cancel: &CancellationToken) {
        let mut failpoint = self.test_publication_failpoint.lock().expect("test failpoint mutex poisoned");
        if *failpoint == Some(SemanticPublicationFailpoint::CancelBeforeCommit) {
            *failpoint = None;
            // The caller observes this at the real pre-commit cancellation
            // check, so the test covers the same linearization boundary.
            cancel.cancel();
        }
    }

    /// Creates a new workspace session with universe declarations and native surfaces bootstrapped.
    pub fn new() -> Self {
        Self::with_workspace(WorkspaceId::from_raw(1))
    }

    /// Creates a new workspace session for a specific workspace ID.
    pub fn with_workspace(workspace: WorkspaceId) -> Self {
        crate::canonical_baseline::canonical_universe_semantic_baseline().instantiate(workspace)
    }

    /// Creates an ordinary local workspace session over an immutable loaded
    /// canonical compiler image. The image snapshot remains the canonical base;
    /// local updates publish through this session's normal database overlay.
    pub fn from_image(workspace: WorkspaceId, image: Arc<crate::canonical_image::CanonicalUniverseFormalImage>) -> Self {
        let snapshot = image.snapshot();
        let provider = phalcom_modules::UniverseSourceProvider::new();
        let base_type_member_declarations = snapshot
            .type_member_declarations
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<BTreeMap<_, _>>();
        let canonical_native_diagnostics = snapshot
            .diagnostics
            .iter()
            .filter(|(module, _)| module.project.is_universe())
            .map(|(module, diagnostics)| (module.clone(), diagnostics.to_vec()))
            .collect();
        let snapshot = snapshot.clone();
        Self {
            workspace,
            module_session: WorkspaceModuleSession::with_external_modules(image.modules().clone()),
            universe_source_provider: provider,
            db: SemanticDb::with_canonical_image(workspace, image),
            store: snapshot.store.clone(),
            static_values: snapshot.static_values.clone(),
            base_declarations: (*snapshot.declarations).clone(),
            base_hierarchy: (*snapshot.hierarchy).clone(),
            base_dispatch: (*snapshot.dispatch).clone(),
            base_callable_signatures: (*snapshot.callable_signatures).clone(),
            base_data_semantics: (*snapshot.data_semantics).clone(),
            // The image query table supplies immutable declaration products
            // directly; local session state must not duplicate those roots.
            base_data_products: Vec::new(),
            base_enum_semantics: (*snapshot.enum_semantics).clone(),
            base_enum_products: Vec::new(),
            base_associated_surfaces: (*snapshot.associated_surfaces).clone(),
            base_associated_surface_products: Vec::new(),
            base_type_member_declarations,
            base_type_member_products: Vec::new(),
            base_enum_requirements: (*snapshot.enum_requirements).clone(),
            base_enum_requirement_products: Vec::new(),
            sources: BTreeMap::new(),
            source_fingerprints: BTreeMap::new(),
            field_lifecycle_fingerprints: BTreeMap::new(),
            default_field_lifecycle: crate::checker::field_lifecycle::FieldLifecycleTable::default(),
            semantic_structure_shards: BTreeMap::new(),
            generic_header_dependencies: BTreeMap::new(),
            generic_header_reverse_dependencies: BTreeMap::new(),
            declaration_signature_dependencies: BTreeMap::new(),
            declaration_signature_reverse_dependencies: BTreeMap::new(),
            declaration_signature_fingerprints: BTreeMap::new(),
            inherent_impl_surface_owners: BTreeMap::new(),
            alias_sources: BTreeMap::new(),
            alias_dependencies: BTreeMap::new(),
            alias_reverse_dependencies: BTreeMap::new(),
            alias_contract_dependencies: BTreeMap::new(),
            alias_contract_reverse_dependencies: BTreeMap::new(),
            alias_forms: Arc::new(BTreeMap::new()),
            module_diagnostics: BTreeMap::new(),
            canonical_native_diagnostics,
            semantic_diagnostic_contributions: BTreeMap::new(),
            last_snapshot: Some(snapshot.clone()),
            last_known_good: Some(snapshot),
            pending_semantic_delta: None,
            requested_deep_modules: None,
            product_request: SemanticProductRequest::default(),
            #[cfg(test)]
            test_publication_failpoint: std::sync::Mutex::new(None),
            #[cfg(test)]
            test_candidate_type_parameter: std::sync::Mutex::new(None),
        }
    }

    /// Builds a semantic session with an explicit canonical Universe source provider.
    /// Source overrides themselves are only constructible through the modules
    /// crate's test-support feature; normal production construction uses `new`.
    pub fn with_universe_source_provider(workspace: WorkspaceId, provider: phalcom_modules::UniverseSourceProvider) -> Self {
        if provider.is_canonical() {
            crate::canonical_baseline::canonical_universe_semantic_baseline().instantiate(workspace)
        } else {
            crate::canonical_baseline::CanonicalUniverseSemanticBaseline::build_with_provider(provider).instantiate(workspace)
        }
    }

    pub(crate) fn from_baseline(baseline: &crate::canonical_baseline::CanonicalUniverseSemanticBaseline, workspace: WorkspaceId) -> Self {
        let db = SemanticDb::with_workspace(workspace);
        let store = Arc::new(baseline.store.fork_from_baseline());
        let static_values = Arc::new(crate::static_value::StaticValueStore::new());
        Self {
            workspace,
            module_session: WorkspaceModuleSession::with_universe_source_provider(baseline.provider.clone()),
            universe_source_provider: baseline.provider.clone(),
            db,
            store,
            static_values,
            base_declarations: baseline.base_declarations.clone(),
            base_hierarchy: baseline.base_hierarchy.clone(),
            base_dispatch: baseline.base_dispatch.clone(),
            base_callable_signatures: baseline.base_callable_signatures.clone(),
            base_data_semantics: baseline.base_data_semantics.clone(),
            base_data_products: baseline.base_data_products.clone(),
            base_enum_semantics: baseline.base_enum_semantics.clone(),
            base_enum_products: baseline.base_enum_products.clone(),
            base_associated_surfaces: baseline.base_associated_surfaces.clone(),
            base_associated_surface_products: baseline.base_associated_surface_products.clone(),
            base_type_member_declarations: baseline.base_type_member_declarations.clone(),
            base_type_member_products: baseline.base_type_member_products.clone(),
            base_enum_requirements: baseline.base_enum_requirements.clone(),
            base_enum_requirement_products: baseline.base_enum_requirement_products.clone(),
            sources: BTreeMap::new(),
            source_fingerprints: BTreeMap::new(),
            field_lifecycle_fingerprints: BTreeMap::new(),
            default_field_lifecycle: crate::checker::field_lifecycle::FieldLifecycleTable::default(),
            semantic_structure_shards: BTreeMap::new(),
            generic_header_dependencies: BTreeMap::new(),
            generic_header_reverse_dependencies: BTreeMap::new(),
            declaration_signature_dependencies: BTreeMap::new(),
            declaration_signature_reverse_dependencies: BTreeMap::new(),
            declaration_signature_fingerprints: BTreeMap::new(),
            inherent_impl_surface_owners: BTreeMap::new(),
            alias_sources: baseline.canonical_alias_sources.clone(),
            alias_dependencies: BTreeMap::new(),
            alias_reverse_dependencies: BTreeMap::new(),
            alias_contract_dependencies: BTreeMap::new(),
            alias_contract_reverse_dependencies: BTreeMap::new(),
            alias_forms: Arc::new(BTreeMap::new()),
            module_diagnostics: BTreeMap::new(),
            canonical_native_diagnostics: baseline.canonical_native_diagnostics.clone(),
            semantic_diagnostic_contributions: BTreeMap::new(),
            last_snapshot: None,
            last_known_good: None,
            pending_semantic_delta: None,
            requested_deep_modules: None,
            product_request: SemanticProductRequest::default(),
            #[cfg(test)]
            test_publication_failpoint: std::sync::Mutex::new(None),
            #[cfg(test)]
            test_candidate_type_parameter: std::sync::Mutex::new(None),
        }
    }

    pub fn workspace(&self) -> WorkspaceId {
        self.workspace
    }

    /// Configures the products published by subsequent workspace updates.
    /// Call this before the session's first update. Existing sessions default
    /// to the full tooling product set.
    pub fn with_product_request(mut self, request: SemanticProductRequest) -> Self {
        self.product_request = request;
        self
    }

    /// Returns persistent project/source/module ownership used by compiler updates.
    pub fn module_session(&self) -> &WorkspaceModuleSession {
        &self.module_session
    }

    /// Mutably borrows persistent module ownership for a worker-side batch.
    pub fn module_session_mut(&mut self) -> &mut WorkspaceModuleSession {
        &mut self.module_session
    }

    /// Loads requested project roots into the compiler-owned module session
    /// before source discovery begins.
    pub fn prepare_workspace_roots(&mut self, roots: &[std::path::PathBuf]) -> Result<SemanticWorkspacePublication, SemanticWorkspaceUpdateError> {
        let update = self.module_session.set_workspace_roots(roots, &phalcom_modules::NullDependencyProvider)?;
        self.update_module_workspace(update).map_err(Into::into)
    }

    /// Applies one module lifecycle mutation and publishes its semantic snapshot.
    pub fn apply_module_mutation(&mut self, mutation: WorkspaceSourceMutation) -> Result<SemanticWorkspaceUpdate, SemanticWorkspaceUpdateError> {
        let update = self.module_session.apply(mutation)?;
        self.update_module_workspace(update).map_err(Into::into)
    }

    /// Applies one heterogeneous module/source batch and publishes one
    /// canonical semantic snapshot for the resulting workspace generation.
    pub fn apply_module_mutations<I>(&mut self, mutations: I) -> Result<SemanticWorkspacePublication, SemanticWorkspaceUpdateError>
    where
        I: IntoIterator<Item = WorkspaceSourceBatchMutation>,
    {
        let update = self.module_session.apply_batch(mutations)?;
        self.update_module_workspace(update).map_err(Into::into)
    }

    /// Applies a module batch while restricting first-publication deep work to
    /// the supplied module frontier. Discovery, identity, interfaces, and
    /// linking remain workspace-wide; only formal/body/advisory solving is
    /// narrowed. Workspace callers should pass `None` via the ordinary method.
    pub fn apply_module_mutations_with_deep_modules<I>(
        &mut self,
        mutations: I,
        deep_modules: BTreeSet<ModuleId>,
    ) -> Result<SemanticWorkspacePublication, SemanticWorkspaceUpdateError>
    where
        I: IntoIterator<Item = WorkspaceSourceBatchMutation>,
    {
        self.requested_deep_modules.get_or_insert_with(BTreeSet::new).extend(deep_modules);
        self.apply_module_mutations(mutations)
    }

    /// Applies one module batch while allowing the semantic phase to observe
    /// a caller-owned latest-wins cancellation token. The module transaction
    /// commits its source/topology delta first; if semantic work is cancelled,
    /// the next update still receives the newest module delta from the same
    /// persistent session.
    pub fn apply_module_mutations_with_cancel<I>(
        &mut self,
        mutations: I,
        cancel: &CancellationToken,
    ) -> Result<SemanticWorkspacePublication, SemanticWorkspaceUpdateError>
    where
        I: IntoIterator<Item = WorkspaceSourceBatchMutation>,
    {
        let update = self.module_session.apply_batch(mutations)?;
        // Interactive latest-wins updates must include the changed modules as
        // deep roots; the dependency expansion below adds their transitive
        // linked providers when a Local scan previously published only
        // shallow module products.
        self.requested_deep_modules
            .get_or_insert_with(BTreeSet::new)
            .extend(update.changed_modules.iter().cloned());
        self.update_module_workspace_with_cancel(update, cancel).map_err(Into::into)
    }

    /// Publishes an already-linked update without last-known-good fallback.
    /// Executable consumers must observe failure instead of stale source.
    pub fn update_module_workspace_with_cancel(
        &mut self,
        update: WorkspaceModuleUpdate,
        cancel: &CancellationToken,
    ) -> Result<SemanticWorkspacePublication, SemanticUpdateFailure> {
        self.update_module_workspace_with_execution_mode_and_cancel(update, crate::attribute_analysis::AttributeExecutionMode::Debug, cancel)
    }

    /// Publishes a linked workspace update with the guard policy selected by
    /// its executable consumer. Query checking remains mode-independent.
    pub fn update_module_workspace_with_execution_mode_and_cancel(
        &mut self,
        update: WorkspaceModuleUpdate,
        execution_mode: crate::attribute_analysis::AttributeExecutionMode,
        cancel: &CancellationToken,
    ) -> Result<SemanticWorkspacePublication, SemanticUpdateFailure> {
        self.update_module_workspace_with_budget_and_cancel(update, QueryBudget::default(), execution_mode, cancel)
    }

    fn update_module_workspace_with_budget_and_cancel(
        &mut self,
        update: WorkspaceModuleUpdate,
        budget: QueryBudget,
        execution_mode: crate::attribute_analysis::AttributeExecutionMode,
        cancel: &CancellationToken,
    ) -> Result<SemanticWorkspacePublication, SemanticUpdateFailure> {
        self.update_module_workspace_with_link_and_budget(update, budget, execution_mode, None, cancel)
    }

    /// Publishes a retained module update with the canonical symbolic link
    /// needed to project executable requirements before finalization.
    pub fn update_executable_module_workspace(
        &mut self,
        update: WorkspaceModuleUpdate,
        semantic_linked: Arc<phalcom_modules::SemanticLinkedProgram>,
        execution_mode: crate::attribute_analysis::AttributeExecutionMode,
        cancel: &CancellationToken,
    ) -> Result<SemanticWorkspacePublication, SemanticUpdateFailure> {
        self.update_module_workspace_with_link_and_budget(update, QueryBudget::default(), execution_mode, Some(semantic_linked), cancel)
    }

    fn update_module_workspace_with_link_and_budget(
        &mut self,
        update: WorkspaceModuleUpdate,
        budget: QueryBudget,
        execution_mode: crate::attribute_analysis::AttributeExecutionMode,
        semantic_linked: Option<Arc<phalcom_modules::SemanticLinkedProgram>>,
        cancel: &CancellationToken,
    ) -> Result<SemanticWorkspacePublication, SemanticUpdateFailure> {
        let generation = self.module_session.generation();
        let delta = SemanticModuleDelta {
            changed_modules: update.changed_modules.clone(),
            removed_modules: update.removed_modules.clone(),
            identity_changes: update.identity_changes.clone(),
            module_stats: update.stats.clone(),
            module_graph_changed: update.module_graph_changed,
        };
        let current_sources = update.sources.keys().cloned().collect::<BTreeSet<_>>();
        let delta = self.merge_pending_semantic_delta(delta, &current_sources);
        let result = self
            .update_with_budget_and_cancel_and_delta(
                SemanticWorkspaceInput {
                    linked: update.linked,
                    semantic_linked,
                    sources: update.sources,
                    interfaces: update.interfaces,
                    import_products: update.import_products,
                    module_graph_changed: update.module_graph_changed,
                    import_sites_by_module: update.sites_by_importer,
                    require_canonical_import_products: true,
                    attribute_execution_mode: execution_mode,
                    diagnostics: update.diagnostics,
                    blocked_modules: update.blocked_modules,
                    generation,
                    topology: Some(update.topology),
                    reverse_imports: Some(update.reverse_importers),
                },
                budget,
                cancel,
                Some(delta.clone()),
            )
            .map_err(SemanticUpdateFailure::from);
        self.pending_semantic_delta = result.as_ref().err().map(|_| delta);
        result
    }

    fn merge_pending_semantic_delta(&self, mut current: SemanticModuleDelta, current_sources: &BTreeSet<ModuleId>) -> SemanticModuleDelta {
        if let Some(previous) = &self.pending_semantic_delta {
            current.changed_modules.extend(previous.changed_modules.iter().cloned());
            current.removed_modules.extend(previous.removed_modules.iter().cloned());
            current.identity_changes.extend(previous.identity_changes.iter().cloned());
            current.module_graph_changed |= previous.module_graph_changed;
        }
        current.changed_modules.retain(|module| current_sources.contains(module));
        current.removed_modules.retain(|module| !current_sources.contains(module));
        current.identity_changes.retain(|module| current_sources.contains(module));
        current
    }

    /// Publishes semantic products for an already-linked module workspace update.
    pub fn update_module_workspace(&mut self, update: WorkspaceModuleUpdate) -> Result<SemanticWorkspaceUpdate, SemanticUpdateFailure> {
        let generation = self.module_session.generation();
        let delta = SemanticModuleDelta {
            changed_modules: update.changed_modules.clone(),
            removed_modules: update.removed_modules.clone(),
            identity_changes: update.identity_changes.clone(),
            module_stats: update.stats.clone(),
            module_graph_changed: update.module_graph_changed,
        };
        let current_sources = update.sources.keys().cloned().collect::<BTreeSet<_>>();
        let delta = self.merge_pending_semantic_delta(delta, &current_sources);
        let result = self.update_with_delta(
            SemanticWorkspaceInput {
                linked: update.linked,
                semantic_linked: None,
                sources: update.sources,
                interfaces: update.interfaces,
                import_products: update.import_products,
                module_graph_changed: update.module_graph_changed,
                import_sites_by_module: update.sites_by_importer,
                require_canonical_import_products: true,
                attribute_execution_mode: crate::attribute_analysis::AttributeExecutionMode::Debug,
                diagnostics: update.diagnostics,
                blocked_modules: update.blocked_modules,
                generation,
                topology: Some(update.topology),
                reverse_imports: Some(update.reverse_importers),
            },
            Some(delta.clone()),
        );
        self.pending_semantic_delta = result.as_ref().err().map(|_| delta);
        result
    }

    pub fn db(&self) -> &SemanticDb {
        &self.db
    }

    pub fn db_mut(&mut self) -> &mut SemanticDb {
        &mut self.db
    }

    pub fn store(&self) -> &TypeStore {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut TypeStore {
        Arc::make_mut(&mut self.store)
    }

    pub fn last_snapshot(&self) -> Option<&Arc<SemanticSnapshot>> {
        self.last_snapshot.as_ref()
    }

    pub fn last_known_good_snapshot(&self) -> Option<&Arc<SemanticSnapshot>> {
        self.last_known_good.as_ref()
    }

    /// Returns retained source-local semantic structure shards.
    pub fn semantic_structure_shards(&self) -> &BTreeMap<ModuleId, Arc<ModuleSemanticStructureShard>> {
        &self.semantic_structure_shards
    }

    /// Performs an incremental semantic analysis update on this session.
    pub fn update(&mut self, input: SemanticWorkspaceInput) -> Result<SemanticWorkspaceUpdate, SemanticUpdateFailure> {
        self.update_with_delta(input, None)
    }

    /// Performs an incremental update and returns operational failures without publishing stale snapshots.
    pub fn try_update(&mut self, input: SemanticWorkspaceInput) -> Result<SemanticWorkspaceUpdate, SemanticUpdateFailure> {
        self.update(input)
    }

    /// Fallible form of [`Self::update_module_workspace`].
    pub fn try_update_module_workspace(&mut self, update: WorkspaceModuleUpdate) -> Result<SemanticWorkspaceUpdate, SemanticUpdateFailure> {
        self.update_module_workspace(update)
    }

    fn update_with_delta(
        &mut self,
        input: SemanticWorkspaceInput,
        module_delta: Option<SemanticModuleDelta>,
    ) -> Result<SemanticWorkspaceUpdate, SemanticUpdateFailure> {
        self.update_with_budget_and_cancel_and_delta(input, QueryBudget::default(), &CancellationToken::new(), module_delta)
            .map_err(SemanticUpdateFailure::from)
    }

    /// Performs an incremental update under an explicit query budget and cancellation token.
    pub fn update_with_budget_and_cancel(
        &mut self,
        input: SemanticWorkspaceInput,
        budget: QueryBudget,
        cancel: &CancellationToken,
    ) -> Result<SemanticWorkspaceUpdate, QueryOutcome<()>> {
        self.update_with_budget_and_cancel_and_delta(input, budget, cancel, None)
    }

    #[allow(deprecated)]
    fn update_with_budget_and_cancel_and_delta(
        &mut self,
        input: SemanticWorkspaceInput,
        budget: QueryBudget,
        cancel: &CancellationToken,
        module_delta: Option<SemanticModuleDelta>,
    ) -> Result<SemanticWorkspaceUpdate, QueryOutcome<()>> {
        if cancel.is_cancelled() {
            return Err(QueryOutcome::Cancelled);
        }

        let previous_store = self.store.clone();
        let previous_static_values = self.static_values.clone();
        let previous_structure_shards = self.semantic_structure_shards.clone();
        self.db.begin_transaction();
        let result = self.update_candidate_with_budget_and_cancel_and_delta(input, budget, cancel, module_delta, &previous_structure_shards);
        match result {
            Ok(update) => {
                self.db.commit_transaction();
                Ok(update)
            }
            Err(error) => {
                self.db.rollback_transaction();
                self.store = previous_store;
                self.static_values = previous_static_values;
                self.semantic_structure_shards = previous_structure_shards;
                Err(error)
            }
        }
    }

    #[allow(deprecated)]
    fn update_candidate_with_budget_and_cancel_and_delta(
        &mut self,
        input: SemanticWorkspaceInput,
        budget: QueryBudget,
        cancel: &CancellationToken,
        module_delta: Option<SemanticModuleDelta>,
        previous_structure_shards: &BTreeMap<ModuleId, Arc<ModuleSemanticStructureShard>>,
    ) -> Result<SemanticWorkspaceUpdate, QueryOutcome<()>> {
        if cancel.is_cancelled() {
            return Err(QueryOutcome::Cancelled);
        }

        self.db.begin_revision();
        #[cfg(test)]
        self.fail_at_test_publication_point(SemanticPublicationFailpoint::AfterRevisionBegin)?;
        let semantic_topology = canonical_topology_for_input(&input);
        let source_resolution_input = crate::db::fingerprint::source_resolution_input_fingerprint(&input.interfaces);
        let linked_component_product = crate::db::fingerprint::semantic_component_product_fingerprint(&input.linked);
        let mut stats = SemanticUpdateStats::default();
        let mut invalidated_keys = BTreeSet::new();
        let mut callable_dispositions = BTreeMap::new();
        let previous_snapshot = self.last_snapshot.clone();
        let previous_field_lifecycle_fingerprints = self.field_lifecycle_fingerprints.clone();
        let previous_module_diagnostics = self.module_diagnostics.clone();
        let current_module_diagnostics = input.diagnostics.clone();
        let mut modules_with_changed_module_diagnostics = BTreeSet::new();
        for module in previous_module_diagnostics.keys().chain(current_module_diagnostics.keys()) {
            if previous_module_diagnostics.get(module) != current_module_diagnostics.get(module) {
                modules_with_changed_module_diagnostics.insert(module.clone());
            }
        }
        let linked_interface_changed = previous_snapshot.as_ref().map_or_else(BTreeSet::new, |previous| {
            input
                .linked
                .modules
                .iter()
                .filter(|(module, linked)| previous.module_products.linked_interface(module) != Some(&linked.interface))
                .map(|(module, _)| module.clone())
                .chain(
                    previous
                        .module_products
                        .linked
                        .keys()
                        .filter(|module| !input.linked.modules.contains_key(*module))
                        .cloned(),
                )
                .collect()
        });
        let delta_driven = module_delta.is_some();
        let delta_changed_modules = module_delta.as_ref().map_or_else(BTreeSet::new, |delta| delta.changed_modules.clone());
        let delta_identity_changes = module_delta.as_ref().map_or_else(BTreeSet::new, |delta| delta.identity_changes.clone());
        let delta_removed_modules = module_delta.as_ref().map_or_else(BTreeSet::new, |delta| delta.removed_modules.clone());

        let mut semantic_structure_shards = BTreeMap::new();
        let mut structural_recomputed_modules = BTreeSet::new();
        let mut structural_reused_modules = BTreeSet::new();
        for (module, source) in &input.sources {
            let explicitly_changed = delta_changed_modules.contains(module) || delta_identity_changes.contains(module);
            if let Some(previous) = self.semantic_structure_shards.get(module) {
                let structural_unchanged = ModuleSemanticStructureShard::structural_fingerprint(source) == previous.source_fingerprint;
                let unchanged = if delta_driven && explicitly_changed && structural_unchanged {
                    semantic_structure_shards.insert(module.clone(), ModuleSemanticStructureShard::with_source(previous, source.clone()));
                    structural_reused_modules.insert(module.clone());
                    stats.semantic_structure_shards_reused += 1;
                    continue;
                } else if delta_driven && !explicitly_changed {
                    true
                } else {
                    self.source_fingerprints
                        .get(module)
                        .is_some_and(|fingerprint| *fingerprint == compute_module_fingerprint(source))
                };
                if unchanged {
                    semantic_structure_shards.insert(module.clone(), previous.clone());
                    structural_reused_modules.insert(module.clone());
                    stats.semantic_structure_shards_reused += 1;
                    continue;
                }
            }
            semantic_structure_shards.insert(module.clone(), ModuleSemanticStructureShard::from_source(source.clone()));
            structural_recomputed_modules.insert(module.clone());
            stats.semantic_structure_shards_recomputed += 1;
        }
        let contribution_delta = contribution_delta(&semantic_structure_shards, previous_structure_shards);
        self.semantic_structure_shards = semantic_structure_shards;
        #[cfg(test)]
        self.fail_at_test_publication_point(SemanticPublicationFailpoint::AfterStructuralShards)?;
        let current_declarations = self
            .semantic_structure_shards
            .values()
            .flat_map(|shard| shard.declaration_header_fingerprints.keys().cloned())
            .collect::<BTreeSet<_>>();
        let removed_callable_bodies = previous_snapshot
            .as_ref()
            .into_iter()
            .flat_map(|snapshot| snapshot.callable_analyses.keys())
            .filter(|callable| {
                callable
                    .try_declaration_owner()
                    .is_some_and(|owner| owner.name.as_ref() != "<main>" && !current_declarations.contains(owner))
            })
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut removed_query_roots = BTreeSet::new();
        for declaration in &contribution_delta.declarations_removed {
            removed_query_roots.insert(QueryKey::DeclarationShell(declaration.clone()));
            removed_query_roots.insert(QueryKey::DeclarationSurface(declaration.clone()));
            removed_query_roots.insert(QueryKey::TraitHeader(declaration.clone()));
            removed_query_roots.insert(QueryKey::TraitSurface(declaration.clone()));
            removed_query_roots.insert(QueryKey::HierarchyEdge(declaration.clone()));
            removed_query_roots.insert(QueryKey::LinkedName(declaration.module.clone(), declaration.name.to_string()));
            removed_query_roots.insert(QueryKey::PublicExport(declaration.module.clone(), declaration.name.to_string()));
        }
        for callable in &contribution_delta.callable_signatures_removed {
            removed_query_roots.insert(QueryKey::CallableDefinition(callable.clone()));
            removed_query_roots.insert(QueryKey::CallableSignature(callable.clone()));
            removed_query_roots.insert(QueryKey::AdvisoryCallable(callable.clone()));
        }
        for field in &contribution_delta.field_signatures_removed {
            removed_query_roots.insert(QueryKey::FieldSignature(field.clone()));
        }
        for callable in &contribution_delta.callable_bodies_removed {
            removed_query_roots.insert(QueryKey::CallableBody(callable.clone()));
            removed_query_roots.insert(QueryKey::AdvisoryCallable(callable.clone()));
        }
        for callable in &removed_callable_bodies {
            removed_query_roots.insert(QueryKey::CallableBody(callable.clone()));
            removed_query_roots.insert(QueryKey::AdvisoryCallable(callable.clone()));
        }
        let removed_closure = self.db.index().reverse_closure(removed_query_roots);
        invalidated_keys.extend(removed_closure.iter().cloned());
        let mut retired_body_dependents = BTreeSet::new();
        for key in &removed_closure {
            if let QueryKey::CallableBody(callable) = key {
                if !contribution_delta.callable_bodies_removed.contains(callable) && !removed_callable_bodies.contains(callable) {
                    retired_body_dependents.insert(callable.clone());
                }
            }
        }
        for declaration in &contribution_delta.declarations_removed {
            self.db.retire_query(&QueryKey::DeclarationShell(declaration.clone()));
            self.db.retire_query(&QueryKey::DeclarationSurface(declaration.clone()));
            self.db.retire_query(&QueryKey::TraitHeader(declaration.clone()));
            self.db.retire_query(&QueryKey::TraitSurface(declaration.clone()));
            self.db.retire_query(&QueryKey::HierarchyEdge(declaration.clone()));
        }
        for callable in &contribution_delta.callable_signatures_removed {
            self.db.retire_query(&QueryKey::CallableDefinition(callable.clone()));
            self.db.retire_query(&QueryKey::CallableSignature(callable.clone()));
            self.db.retire_query(&QueryKey::AdvisoryCallable(callable.clone()));
        }
        for field in &contribution_delta.field_signatures_removed {
            self.db.retire_query(&QueryKey::FieldSignature(field.clone()));
        }
        for callable in &contribution_delta.callable_bodies_removed {
            self.db.retire_query(&QueryKey::CallableBody(callable.clone()));
            self.db.retire_query(&QueryKey::AdvisoryCallable(callable.clone()));
        }
        for callable in &removed_callable_bodies {
            self.db.retire_query(&QueryKey::CallableBody(callable.clone()));
            self.db.retire_query(&QueryKey::AdvisoryCallable(callable.clone()));
        }
        let current_modules = self.semantic_structure_shards.keys().cloned().collect::<BTreeSet<_>>();
        let mut removed_modules = delta_removed_modules;
        if !delta_driven {
            // Direct SemanticWorkspaceInput publication is a full-workspace
            // compatibility path. Production module updates carry the exact
            // Plan-A removal worklist above and never rediscover retirement
            // from retained source maps.
            removed_modules.extend(self.sources.keys().filter(|module| !current_modules.contains(module)).cloned());
        }
        structural_recomputed_modules.extend(removed_modules.iter().cloned());
        let retained_sources = self
            .semantic_structure_shards
            .iter()
            .map(|(module, shard)| (module.clone(), shard.source.clone()))
            .collect::<BTreeMap<_, _>>();

        // Publish source identity and binding phase before any type-bearing
        // declaration work. The catalog deliberately contains no checker state
        // or type information; linked aliases keep their remote SymbolId.
        let mut module_static_binding_catalogs = retained_sources
            .iter()
            .map(|(module, source)| {
                (
                    module.clone(),
                    Arc::new(crate::module_bindings::ModuleStaticBindingCatalog::from_source(module, &source.program)),
                )
            })
            .collect::<BTreeMap<_, _>>();
        if let Some(semantic_linked) = input.semantic_linked.as_ref() {
            for (module_id, sem_mod) in &semantic_linked.modules {
                let imported_seeds = sem_mod
                    .imports
                    .bindings
                    .values()
                    .filter_map(|import_binding| {
                        let phalcom_modules::linker::SymbolicImportTarget::Binding(ref symbol) = import_binding.target else {
                            return None;
                        };
                        let seed = module_static_binding_catalogs.get(&symbol.module)?.get(symbol)?.clone();
                        Some((import_binding.local_name.to_string(), seed))
                    })
                    .collect::<Vec<_>>();
                if let Some(catalog) = module_static_binding_catalogs.get_mut(module_id) {
                    for (local_name, seed) in imported_seeds {
                        Arc::make_mut(catalog).insert_imported_binding(&local_name, seed);
                    }
                }
            }
        } else {
            for (module_id, linked_module) in &input.linked.modules {
                let imported_seeds = linked_module
                    .bindings
                    .imports
                    .iter()
                    .filter_map(|(local_name, import_id)| {
                        let Some(phalcom_modules::linker::LinkedReadSpec::Binding(symbol)) = linked_module.linked_reads.get(import_id.0 as usize) else {
                            return None;
                        };
                        let seed = module_static_binding_catalogs.get(&symbol.module)?.get(symbol)?.clone();
                        Some((local_name.to_string(), seed))
                    })
                    .collect::<Vec<_>>();
                if let Some(catalog) = module_static_binding_catalogs.get_mut(module_id) {
                    for (local_name, seed) in imported_seeds {
                        Arc::make_mut(catalog).insert_imported_binding(&local_name, seed);
                    }
                }
            }
        }
        for (module_id, catalog) in module_static_binding_catalogs.clone() {
            let catalog = match crate::db::query::query_module_static_binding_catalog(&mut self.db, catalog) {
                QueryOutcome::Ready(product) => product,
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            };
            module_static_binding_catalogs.insert(module_id, catalog);
        }

        // 1. Refresh source-owned staged products without eager reverse invalidation.
        //
        // A source edit changes ParsedModule input identity. Query-local recomputation
        // preserves downstream cached products, and their dependency fingerprints decide
        // lazily whether semantic propagation stops or continues. UnlinkedInterface is
        // evaluated for every source so an unchanged unlinked semantic product can become
        // current and allow linked/formal/body products to remain reusable.
        let mut new_fingerprints = BTreeMap::new();
        let mut changed_modules = if delta_driven { delta_changed_modules.clone() } else { BTreeSet::new() };
        for (module_id, unit) in &input.sources {
            let fp = if delta_driven && !changed_modules.contains(module_id) {
                self.source_fingerprints
                    .get(module_id)
                    .copied()
                    .unwrap_or_else(|| compute_module_fingerprint(unit))
            } else {
                compute_module_fingerprint(unit)
            };
            new_fingerprints.insert(module_id.clone(), fp);

            let existed = self.source_fingerprints.contains_key(module_id);
            let changed = changed_modules.contains(module_id) || self.source_fingerprints.get(module_id).copied() != Some(fp);

            if changed {
                changed_modules.insert(module_id.clone());
                stats.modules_recomputed += 1;
                if existed {
                    invalidated_keys.insert(QueryKey::ParsedModule(module_id.clone()));
                }
            }

            let needs_unlinked_refresh = !delta_driven || changed || !existed;
            if !needs_unlinked_refresh {
                continue;
            }
            let precomputed = input.interfaces.get(module_id).cloned();
            match query_unlinked_interface(&mut self.db, module_id.clone(), unit.clone(), precomputed) {
                QueryOutcome::Ready(_) => {}
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            }
        }

        // Removal is different from recomputation: the product has no replacement whose
        // fingerprint could prove semantic stability, so its full reverse closure must die.
        for old_module_id in self.sources.keys() {
            if !input.sources.contains_key(old_module_id) {
                changed_modules.insert(old_module_id.clone());
                let parsed_key = QueryKey::ParsedModule(old_module_id.clone());
                let unlinked_key = QueryKey::UnlinkedInterface(old_module_id.clone());
                let linked_key = QueryKey::LinkedInterface(old_module_id.clone());
                let diags_key = QueryKey::ModuleDiagnostics(old_module_id.clone());
                let mut seeds = vec![parsed_key, unlinked_key, linked_key, diags_key, QueryKey::AdvisoryModule(old_module_id.clone())];
                seeds.extend(self.db.query_keys_for_module(old_module_id).cloned());
                if let Some(snapshot) = previous_snapshot.as_ref() {
                    seeds.extend(
                        snapshot
                            .callable_analyses
                            .keys()
                            .filter(|callable| callable.module() == old_module_id)
                            .cloned()
                            .map(QueryKey::AdvisoryCallable),
                    );
                }
                let closure = self.db.invalidate(seeds);
                invalidated_keys.extend(closure);
                self.db.purge_module(old_module_id);
            }
        }
        #[cfg(test)]
        self.fail_at_test_publication_point(SemanticPublicationFailpoint::AfterQueryRetirement)?;

        let mut next_field_lifecycle_fingerprints = BTreeMap::new();
        let mut field_lifecycle_changed_modules = BTreeSet::new();
        for (module, source) in &input.sources {
            let refresh = !delta_driven || delta_changed_modules.contains(module) || !previous_field_lifecycle_fingerprints.contains_key(module);
            let fingerprint = if refresh {
                field_lifecycle_source_fingerprint(source)
            } else {
                previous_field_lifecycle_fingerprints[module]
            };
            if previous_field_lifecycle_fingerprints.get(module).copied() != Some(fingerprint) {
                field_lifecycle_changed_modules.insert(module.clone());
            }
            next_field_lifecycle_fingerprints.insert(module.clone(), fingerprint);
        }
        field_lifecycle_changed_modules.extend(
            previous_field_lifecycle_fingerprints
                .keys()
                .filter(|module| !next_field_lifecycle_fingerprints.contains_key(*module))
                .cloned(),
        );

        #[cfg(test)]
        self.fail_at_test_publication_point(SemanticPublicationFailpoint::AfterSourceFingerprints)?;
        // Generic declaration headers form a second exact dependency graph:
        // a provider header edit must reach only headers whose bounds,
        // constraints, or superclass template mention that provider.
        let mut generic_header_work = contribution_delta.declarations.clone();
        let mut alias_contract_work = BTreeSet::new();
        let mut pending_contract_providers = contribution_delta
            .declarations
            .iter()
            .chain(contribution_delta.declarations_removed.iter())
            .cloned()
            .collect::<Vec<_>>();
        while let Some(provider) = pending_contract_providers.pop() {
            let Some(consumers) = self.alias_contract_reverse_dependencies.get(&provider) else {
                continue;
            };
            for consumer in consumers {
                if alias_contract_work.insert(consumer.clone()) {
                    pending_contract_providers.push(consumer.clone());
                }
            }
        }
        let mut changed_header_dependencies = contribution_delta.declarations.clone();
        for module in contribution_delta.structural_modules.clone() {
            let imports_changed = match (self.semantic_structure_shards.get(&module), previous_structure_shards.get(&module)) {
                (Some(current), Some(previous)) => current.source.program.preamble.dependencies != previous.source.program.preamble.dependencies,
                (Some(_), None) => true,
                (None, Some(_)) => true,
                (None, None) => false,
            };
            if imports_changed {
                if let Some(shard) = self.semantic_structure_shards.get(&module) {
                    generic_header_work.extend(shard.declaration_header_fingerprints.keys().cloned());
                }
            }
        }
        changed_header_dependencies.extend(generic_header_work.iter().cloned());
        let mut pending_header_providers = changed_header_dependencies.iter().cloned().collect::<Vec<_>>();
        while let Some(provider) = pending_header_providers.pop() {
            let Some(consumers) = self.generic_header_reverse_dependencies.get(&provider) else {
                continue;
            };
            for consumer in consumers {
                if generic_header_work.insert(consumer.clone()) {
                    changed_header_dependencies.insert(consumer.clone());
                    pending_header_providers.push(consumer.clone());
                }
            }
        }

        // The declaration-signature graph is also the retained invalidation
        // authority. Compare stable declaration-level source fingerprints, then
        // walk the previous graph's reverse edges before any products are dropped.
        // Only inspect declaration syntax in modules whose semantic-structure
        // shard changed. Reuse retained fingerprints for every other module so
        // a local edit does not turn fingerprint discovery into a workspace
        // declaration scan.
        let mut signature_known_declarations = self
            .base_declarations
            .iter()
            .map(|(declaration, _)| declaration.clone())
            .collect::<HashSet<_>>();
        for shard in self.semantic_structure_shards.values() {
            signature_known_declarations.extend(shard.declarations.iter().map(|declaration| declaration.id.clone()));
            signature_known_declarations.extend(shard.aliases.iter().cloned());
        }
        for kind in [
            crate::checker::proof_predicate::ProofPredicateKind::Equivalent,
            crate::checker::proof_predicate::ProofPredicateKind::Subtype,
            crate::checker::proof_predicate::ProofPredicateKind::Conforms,
        ] {
            signature_known_declarations.insert(crate::checker::proof_predicate::proof_predicate_declaration(kind));
        }
        let canonical_linked = canonical_linked_program_for_session(&self.db, &self.universe_source_provider);
        let signature_resolver = LinkedTypeResolver::with_retained_alias_forms(
            input.linked.clone(),
            signature_known_declarations,
            ModuleId::universe_root(),
            self.alias_forms.clone(),
        )
        .with_canonical_linked(canonical_linked)
        .with_semantic_linked(input.semantic_linked.clone());
        let mut current_signature_fingerprints = self.declaration_signature_fingerprints.clone();
        for module in &structural_recomputed_modules {
            current_signature_fingerprints.retain(|node, _| signature_node_module(node) != module);
            if let Some(shard) = self.semantic_structure_shards.get(module) {
                current_signature_fingerprints.extend(declaration_signature_source_fingerprints(std::iter::once((module, shard)), &signature_resolver));
            }
        }
        let mut signature_change_seeds = current_signature_fingerprints
            .iter()
            .filter(|(node, fingerprint)| self.declaration_signature_fingerprints.get(*node) != Some(*fingerprint))
            .map(|(node, _)| node.clone())
            .collect::<BTreeSet<_>>();
        signature_change_seeds.extend(
            self.declaration_signature_fingerprints
                .keys()
                .filter(|node| !current_signature_fingerprints.contains_key(*node))
                .cloned(),
        );
        signature_change_seeds.extend(generic_header_work.iter().cloned().map(TypeSignatureDependencyNode::Header));
        for module in &structural_recomputed_modules {
            let imports_changed = match (self.semantic_structure_shards.get(module), previous_structure_shards.get(module)) {
                (Some(current), Some(previous)) => current.source.program.preamble.dependencies != previous.source.program.preamble.dependencies,
                (Some(_), None) | (None, Some(_)) => true,
                (None, None) => false,
            };
            if imports_changed {
                signature_change_seeds.extend(
                    current_signature_fingerprints
                        .keys()
                        .filter(|node| signature_node_module(node) == module)
                        .cloned(),
                );
                signature_change_seeds.extend(
                    self.declaration_signature_fingerprints
                        .keys()
                        .filter(|node| signature_node_module(node) == module)
                        .cloned(),
                );
            }
        }
        let signature_dependency_work = reverse_signature_dependency_closure(&self.declaration_signature_reverse_dependencies, &signature_change_seeds);
        let signature_alias_work = signature_dependency_work
            .iter()
            .filter_map(|node| match node {
                TypeSignatureDependencyNode::Alias(declaration) => Some(declaration.clone()),
                TypeSignatureDependencyNode::Header(_) | TypeSignatureDependencyNode::Member(_) | TypeSignatureDependencyNode::StaticCaseSignature(_) => None,
            })
            .collect::<BTreeSet<_>>();
        let signature_header_work = signature_dependency_work
            .iter()
            .filter_map(|node| match node {
                TypeSignatureDependencyNode::Header(declaration) => Some(declaration.clone()),
                TypeSignatureDependencyNode::Alias(_) | TypeSignatureDependencyNode::Member(_) | TypeSignatureDependencyNode::StaticCaseSignature(_) => None,
            })
            .collect::<BTreeSet<_>>();
        let signature_member_work = signature_dependency_work
            .iter()
            .filter_map(|node| match node {
                TypeSignatureDependencyNode::Member(member) => Some(member.clone()),
                TypeSignatureDependencyNode::Alias(_) | TypeSignatureDependencyNode::Header(_) | TypeSignatureDependencyNode::StaticCaseSignature(_) => None,
            })
            .collect::<BTreeSet<_>>();
        let signature_case_work = signature_dependency_work
            .iter()
            .filter_map(|node| match node {
                TypeSignatureDependencyNode::StaticCaseSignature(case) => Some(case.clone()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        generic_header_work.extend(signature_header_work.iter().cloned());

        // Convert source contribution deltas into typed query roots. The
        // dependency index then propagates those roots through exact reverse
        // edges; a changed module no longer seeds every cached query owned by
        // that module.
        let mut hierarchy_edge_work = BTreeSet::new();
        let mut declaration_surface_work = BTreeSet::new();
        let mut callable_signature_work = BTreeSet::new();
        let mut callable_body_work = contribution_delta.callable_bodies.clone();
        callable_body_work.extend(retired_body_dependents);
        // Trait header/surface changes invalidate every default owned by the
        // trait, even when the default statements themselves are unchanged.
        // Trait defaults are contract-relative and therefore must be checked
        // again against the new header/surface product.
        for declaration in &contribution_delta.declarations {
            let Some(shard) = self.semantic_structure_shards.get(&declaration.module) else {
                continue;
            };
            let is_trait = shard.source.program.statements.iter().any(|statement| {
                matches!(statement, Statement::Trait(trait_def) if DeclarationId::new(declaration.module.clone(), trait_def.name.clone().into()) == *declaration)
            });
            if is_trait {
                callable_body_work.extend(
                    shard
                        .callable_body_fingerprints
                        .keys()
                        .filter(|callable| callable.try_declaration_owner() == Some(declaration))
                        .cloned(),
                );
            }
        }
        // A member signature edit changes the contract even when its default
        // body text is unchanged. Seed only bodyful members owned by traits;
        // ordinary class/impl body scheduling remains unchanged.
        for callable in &contribution_delta.callable_signatures {
            let Some(shard) = self.semantic_structure_shards.get(callable.module()) else {
                continue;
            };
            let is_trait = shard.source.program.statements.iter().any(|statement| {
                matches!(statement, Statement::Trait(trait_def) if callable.try_declaration_owner().is_some_and(|owner| DeclarationId::new(callable.module().clone(), trait_def.name.clone().into()) == *owner))
            });
            if is_trait && shard.callable_body_fingerprints.contains_key(callable) {
                callable_body_work.insert(callable.clone());
            }
        }
        let mut field_signature_work = BTreeSet::new();
        let mut declaration_shell_work = BTreeSet::new();
        let requested_deep_modules = self.requested_deep_modules.clone();
        let mut semantic_work_modules = if previous_snapshot.is_none() {
            requested_deep_modules.clone().unwrap_or_else(|| current_modules.clone())
        } else {
            let mut roots = BTreeSet::new();
            for declaration in &generic_header_work {
                roots.insert(QueryKey::DeclarationShell(declaration.clone()));
            }
            // Alias source fingerprints share the declaration-signature graph,
            // but alias changes also publish declaration-shell products that
            // callable signatures depend on. Feed those changed alias roots
            // through the existing query dependency graph so consumers are
            // invalidated after the new transparent form is published.
            for declaration in &signature_alias_work {
                roots.insert(QueryKey::DeclarationShell(declaration.clone()));
            }
            for declaration in &contribution_delta.declarations {
                roots.insert(QueryKey::LinkedName(declaration.module.clone(), declaration.name.to_string()));
                roots.insert(QueryKey::PublicExport(declaration.module.clone(), declaration.name.to_string()));
            }
            for declaration in &contribution_delta.declarations_removed {
                roots.insert(QueryKey::LinkedName(declaration.module.clone(), declaration.name.to_string()));
                roots.insert(QueryKey::PublicExport(declaration.module.clone(), declaration.name.to_string()));
            }
            for declaration in &contribution_delta.aliases {
                roots.insert(QueryKey::DeclarationShell(declaration.clone()));
            }
            for declaration in &contribution_delta.hierarchy_edges {
                roots.insert(QueryKey::HierarchyEdge(declaration.clone()));
            }
            for callable in &contribution_delta.callable_signatures {
                // A callable whose body also changed is already a body root.
                // Let its canonical signature query publish first, then seed
                // exact signature consumers from the actual recomputation
                // event; seeding here would traverse every consumer even when
                // the signature contract is unchanged.
                if !contribution_delta.callable_bodies.contains(callable) {
                    roots.insert(QueryKey::CallableSignature(callable.clone()));
                }
            }
            for field in &contribution_delta.field_signatures {
                roots.insert(QueryKey::FieldSignature(field.clone()));
            }
            for callable in &contribution_delta.callable_bodies {
                roots.insert(QueryKey::CallableBody(callable.clone()));
            }
            for module in &contribution_delta.structural_modules {
                roots.insert(QueryKey::SourceStructure(module.clone()));
                if let Some(previous) = previous_snapshot.as_ref() {
                    for (impl_id, _) in previous.conformance_index.iter() {
                        if &impl_id.module == module {
                            roots.insert(QueryKey::ConformanceDispatch(impl_id.clone()));
                        }
                    }
                    for rule in previous.derived_conformance_rules.iter() {
                        if &rule.impl_id.module == module {
                            roots.insert(QueryKey::ConformanceDispatch(rule.impl_id.clone()));
                        }
                    }
                    for (trait_decl, _) in previous.trait_headers.iter() {
                        if &trait_decl.module == module {
                            roots.insert(QueryKey::TraitHeader(trait_decl.clone()));
                            roots.insert(QueryKey::TraitSurface(trait_decl.clone()));
                        }
                    }
                }
            }
            for module in &changed_modules {
                // Source-only body edits already have a typed CallableBody
                // root. Their parsed/interface products may be refreshed for
                // publication, but must not seed the reverse importer graph.
                // Structural edits still seed those module-layer products.
                if contribution_delta.structural_modules.contains(module) {
                    roots.insert(QueryKey::ParsedModule(module.clone()));
                    roots.insert(QueryKey::UnlinkedInterface(module.clone()));
                }
                if modules_with_changed_module_diagnostics.contains(module) {
                    roots.insert(QueryKey::ModuleDiagnostics(module.clone()));
                }
            }
            callable_body_work.extend(roots.iter().filter(|root| !matches!(root, QueryKey::CallableBody(_))).flat_map(|root| {
                self.db
                    .index()
                    .dependents_of(root)
                    .into_iter()
                    .flatten()
                    .filter_map(|dependent| match dependent {
                        QueryKey::CallableBody(callable) => Some(callable.clone()),
                        _ => None,
                    })
            }));
            let closure = self.db.index().reverse_closure(roots.clone());
            stats.reverse_candidates_considered = closure.len();
            for key in &closure {
                match key {
                    QueryKey::DeclarationShell(declaration) => {
                        declaration_shell_work.insert(declaration.clone());
                    }
                    QueryKey::DeclarationSurface(declaration) => {
                        declaration_surface_work.insert(declaration.clone());
                    }
                    QueryKey::CallableSignature(callable) => {
                        callable_signature_work.insert(callable.clone());
                    }
                    QueryKey::FieldSignature(field) => {
                        field_signature_work.insert(field.clone());
                    }
                    QueryKey::CallableBody(callable) => {
                        callable_body_work.insert(callable.clone());
                    }
                    _ => {}
                }
            }
            hierarchy_edge_work.extend(closure.iter().filter_map(|key| match key {
                QueryKey::HierarchyEdge(declaration) => Some(declaration.clone()),
                _ => None,
            }));
            let mut work = closure
                .iter()
                .filter_map(query_key_module_for_worklist)
                .filter(|module| current_modules.contains(*module))
                .cloned()
                .collect::<BTreeSet<_>>();
            work.extend(changed_modules.iter().filter(|module| current_modules.contains(*module)).cloned());
            work.extend(linked_interface_changed.iter().filter(|module| current_modules.contains(*module)).cloned());
            work
        };
        if let Some(requested) = requested_deep_modules {
            semantic_work_modules.extend(requested.into_iter().filter(|module| current_modules.contains(module)));
        }
        // Complete a requested Local frontier through linked reads. This is
        // the deep dependency closure only; unrelated discovered modules stay
        // available for navigation and linking without body analysis.
        let mut pending_deep = semantic_work_modules.iter().cloned().collect::<Vec<_>>();
        while let Some(module) = pending_deep.pop() {
            let Some(linked_module) = input.linked.modules.get(&module) else {
                continue;
            };
            for dependency in &linked_module.linked_reads {
                let target = match dependency {
                    phalcom_modules::linker::LinkedReadSpec::Binding(symbol) => symbol.module.clone(),
                    phalcom_modules::linker::LinkedReadSpec::Module(target) => target.clone(),
                };
                if current_modules.contains(&target) && semantic_work_modules.insert(target.clone()) {
                    pending_deep.push(target);
                }
            }
        }
        // A body query may be rerun because an upstream product changed while
        // retaining formal products of its own. Revalidate those retained
        // prerequisites from the body's previous typed dependency set. This is
        // intentionally narrower than rebuilding every formal product in the
        // body's module: the set is keyed by the exact semantic queries that
        // are about to be evaluated.
        if let Some(previous) = previous_snapshot.as_ref() {
            for (callable, analysis) in previous.callable_analyses.iter() {
                if !semantic_work_modules.contains(callable.module()) {
                    continue;
                }
                for dependency in analysis.semantic_dependencies.iter() {
                    match dependency {
                        crate::checker::analysis::SemanticDependency::DeclarationShell(declaration) => {
                            declaration_shell_work.insert(declaration.clone());
                        }
                        crate::checker::analysis::SemanticDependency::CallableSignature(callable) => {
                            callable_signature_work.insert(callable.clone());
                        }
                        crate::checker::analysis::SemanticDependency::FieldSignature(field) => {
                            field_signature_work.insert(field.clone());
                        }
                        crate::checker::analysis::SemanticDependency::DeclarationSurface(declaration) => {
                            declaration_surface_work.insert(declaration.clone());
                        }
                        crate::checker::analysis::SemanticDependency::HierarchyEdge(declaration) if current_modules.contains(&declaration.module) => {
                            hierarchy_edge_work.insert(declaration.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
        declaration_shell_work.extend(contribution_delta.declarations.iter().cloned());
        declaration_shell_work.extend(generic_header_work.iter().cloned());
        declaration_surface_work.extend(contribution_delta.declarations.iter().cloned());
        declaration_surface_work.extend(generic_header_work.iter().cloned());
        // Every new or removed declaration can change the module's implicit
        // hierarchy contribution (classes/enums inherit Object even without
        // explicit superclass syntax).  Seed the hierarchy query worklist
        // from the declaration delta as well as explicit superclass edits;
        // otherwise a newly added class is visible in declarations but its
        // Object edge remains stale until a later hierarchy-specific edit.
        hierarchy_edge_work.extend(contribution_delta.declarations.iter().cloned());
        // Import edits can change the target of an unchanged superclass
        // spelling (for example `is Base` after rebinding `Base`).  The shard
        // already owns the exact class/enum edge keys, so add only those keys
        // for structurally recomputed modules rather than reopening every
        // hierarchy product in the workspace.
        for module in &structural_recomputed_modules {
            if let Some(shard) = self.semantic_structure_shards.get(module) {
                hierarchy_edge_work.extend(shard.hierarchy_edge_fingerprints.keys().cloned());
            }
        }
        hierarchy_edge_work.extend(contribution_delta.hierarchy_edges.iter().cloned());
        callable_signature_work.extend(contribution_delta.callable_signatures.iter().cloned());
        field_signature_work.extend(contribution_delta.field_signatures.iter().cloned());
        declaration_surface_work.extend(
            contribution_delta
                .callable_signatures
                .iter()
                .filter_map(|callable| callable.try_declaration_owner().cloned()),
        );
        declaration_surface_work.extend(contribution_delta.field_signatures.iter().map(|field| field.owner.clone()));
        // Rebuilding a declaration surface consumes all its primary member
        // contracts, even when only a linked-name change reopened the surface.
        // Schedule those exact owner prerequisites before surface publication.
        for declaration in &declaration_surface_work {
            let Some(shard) = self.semantic_structure_shards.get(&declaration.module) else {
                continue;
            };
            field_signature_work.extend(shard.field_signature_fingerprints.keys().filter(|field| field.owner == *declaration).cloned());
            callable_signature_work.extend(
                shard
                    .callable_signature_fingerprints
                    .keys()
                    .filter(|callable| callable.try_declaration_owner() == Some(declaration))
                    .cloned(),
            );
        }
        let changed_declarations = hierarchy_edge_work
            .iter()
            .chain(declaration_shell_work.iter())
            .chain(declaration_surface_work.iter())
            .collect::<BTreeSet<_>>();
        // Production module updates carry an exact reverse module-import index.
        // Use it to bound the linked-read inspection to actual importers of a
        // changed declaration's module. Direct compatibility inputs do not
        // have that index, so retain the complete scan there.
        let linked_importer_candidates = if delta_driven {
            let mut candidates = BTreeSet::new();
            if let Some(reverse_imports) = input.reverse_imports.as_ref() {
                for module in changed_declarations.iter().map(|declaration| &declaration.module) {
                    if let Some(importers) = reverse_imports.get(module) {
                        candidates.extend(importers.iter().cloned());
                    }
                }
            } else {
                candidates.extend(input.linked.modules.keys().cloned());
            }
            candidates
        } else {
            input.linked.modules.keys().cloned().collect()
        };
        stats.linked_importers_considered = linked_importer_candidates.len();
        for importer_id in linked_importer_candidates {
            let Some(linked_mod) = input.linked.modules.get(&importer_id) else {
                continue;
            };
            if !current_modules.contains(&importer_id) || semantic_work_modules.contains(&importer_id) {
                continue;
            }
            let imports_changed_decl = linked_mod.linked_reads.iter().any(|read| match read {
                phalcom_modules::linker::LinkedReadSpec::Binding(sym) => changed_declarations
                    .iter()
                    .any(|decl| sym.module == decl.module && sym.name.as_ref() == decl.name.as_ref()),
                phalcom_modules::linker::LinkedReadSpec::Module(target_mod) => changed_declarations.iter().any(|decl| &decl.module == target_mod),
            });
            if imports_changed_decl {
                semantic_work_modules.insert(importer_id.clone());
            }
        }
        let structural_work_modules = semantic_work_modules
            .iter()
            .filter(|module| {
                !structural_reused_modules.contains(*module) || linked_interface_changed.contains(*module) || field_lifecycle_changed_modules.contains(*module)
            })
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut conformance_witness_rebuild_impls = BTreeSet::new();
        if let Some(previous) = previous_snapshot.as_ref() {
            conformance_witness_rebuild_impls.extend(
                previous
                    .conformance_index
                    .iter()
                    .filter(|(impl_id, _)| structural_work_modules.contains(&impl_id.module))
                    .map(|(impl_id, _)| impl_id.clone()),
            );
            conformance_witness_rebuild_impls.extend(
                previous
                    .derived_conformance_rules
                    .iter()
                    .filter(|rule| structural_work_modules.contains(&rule.impl_id.module))
                    .map(|rule| rule.impl_id.clone()),
            );
            conformance_witness_rebuild_impls.extend(previous.callable_definitions.values().filter_map(|definition| {
                let crate::impls::CallableDefinitionOrigin::ConformanceWitness(impl_id) = &definition.origin else {
                    return None;
                };
                structural_work_modules.contains(&impl_id.module).then(|| impl_id.clone())
            }));
        }
        for module in &structural_work_modules {
            let Some(shard) = self.semantic_structure_shards.get(module) else {
                continue;
            };
            conformance_witness_rebuild_impls.extend(shard.source.program.statements.iter().enumerate().filter_map(|(index, statement)| {
                let Statement::Impl(implementation) = statement else { return None };
                matches!(implementation.kind, phalcom_ast::ast::ImplKind::Conformance { .. })
                    .then(|| crate::identity::ImplId::new(module.clone(), crate::identity::ImplLocalId(index as u32)))
            }));
        }
        let hierarchy_work_modules = structural_work_modules
            .iter()
            .cloned()
            .chain(
                hierarchy_edge_work
                    .iter()
                    .map(|declaration| declaration.module.clone())
                    .filter(|module| current_modules.contains(module)),
            )
            .collect::<BTreeSet<_>>();
        // Revalidate only typed hierarchy-edge work. Retained edges are already
        // present in the immutable aggregate and are not replaced merely
        // because a body in their module was edited.
        let mut hierarchy_query_work = hierarchy_edge_work.clone();
        let mut formal_work_modules = structural_work_modules
            .iter()
            .cloned()
            .chain(declaration_surface_work.iter().map(|declaration| declaration.module.clone()))
            .chain(callable_signature_work.iter().map(|callable| callable.module().clone()))
            .chain(field_signature_work.iter().map(|field| field.owner.module.clone()))
            .filter(|module| current_modules.contains(module))
            .collect::<BTreeSet<_>>();
        let mut formal_declarations = if previous_snapshot.is_none() {
            self.semantic_structure_shards
                .values()
                .flat_map(|shard| shard.declaration_header_fingerprints.keys().cloned())
                .collect::<BTreeSet<_>>()
        } else {
            BTreeSet::new()
        };
        formal_declarations.extend(
            declaration_surface_work
                .iter()
                .filter(|declaration| current_modules.contains(&declaration.module))
                .cloned(),
        );
        formal_declarations.extend(
            callable_signature_work
                .iter()
                .filter_map(|callable| callable.try_declaration_owner().cloned())
                .filter(|declaration| current_modules.contains(&declaration.module)),
        );
        formal_declarations.extend(
            field_signature_work
                .iter()
                .map(|field| field.owner.clone())
                .filter(|declaration| current_modules.contains(&declaration.module)),
        );
        let mut formal_declaration_modules = formal_declarations
            .iter()
            .map(|declaration| declaration.module.clone())
            .collect::<BTreeSet<_>>();
        let mut diagnostic_work_modules = semantic_work_modules.clone();
        diagnostic_work_modules.extend(hierarchy_work_modules.iter().cloned());
        diagnostic_work_modules.extend(formal_work_modules.iter().cloned());
        diagnostic_work_modules.extend(field_lifecycle_changed_modules.iter().cloned());
        diagnostic_work_modules.extend(modules_with_changed_module_diagnostics.iter().cloned());
        diagnostic_work_modules.extend(removed_modules.iter().cloned());
        let mut prior_conformance_clients = BTreeSet::new();
        // Seed diagnostic work through the same formal dependency graph used
        // to refresh conformance plans and dispatch clients. This happens
        // before prior diagnostics are retained, so stale plan/body diagnostics
        // are retired by rebuilding their current products, not by deleting
        // entries from an append-only diagnostic accumulator later.
        if let Some(previous) = previous_snapshot.as_ref() {
            let mut changed_contract_products = BTreeSet::new();
            for (callable, _) in previous.callable_signatures.iter() {
                if semantic_work_modules.contains(callable.module()) {
                    changed_contract_products.insert(QueryKey::CallableSignature(callable.clone()));
                }
            }
            for (declaration, _) in previous.trait_surfaces.iter() {
                if semantic_work_modules.contains(&declaration.module) {
                    changed_contract_products.insert(QueryKey::TraitSurface(declaration.clone()));
                }
            }
            for dependent in self.db.index().reverse_closure(changed_contract_products) {
                match dependent {
                    QueryKey::ConformanceDispatch(impl_id) => {
                        conformance_witness_rebuild_impls.insert(impl_id.clone());
                        diagnostic_work_modules.insert(impl_id.module.clone());
                        semantic_work_modules.insert(impl_id.module.clone());
                    }
                    QueryKey::CallableBody(callable) => {
                        diagnostic_work_modules.insert(callable.module().clone());
                        semantic_work_modules.insert(callable.module().clone());
                        prior_conformance_clients.insert(callable);
                    }
                    _ => {}
                }
            }
        }
        let retired_conformance_witnesses = previous_snapshot
            .as_ref()
            .into_iter()
            .flat_map(|previous| previous.callable_definitions.values())
            .filter_map(|definition| {
                let crate::impls::CallableDefinitionOrigin::ConformanceWitness(impl_id) = &definition.origin else {
                    return None;
                };
                conformance_witness_rebuild_impls.contains(impl_id).then(|| definition.callable.clone())
            })
            .collect::<BTreeSet<_>>();
        if !retired_conformance_witnesses.is_empty() {
            let retired_roots = retired_conformance_witnesses
                .iter()
                .flat_map(|callable| {
                    [
                        QueryKey::CallableDefinition(callable.clone()),
                        QueryKey::CallableSignature(callable.clone()),
                        QueryKey::CallableBody(callable.clone()),
                        QueryKey::AdvisoryCallable(callable.clone()),
                    ]
                })
                .collect::<BTreeSet<_>>();
            for dependent in self.db.index().reverse_closure(retired_roots) {
                match dependent {
                    QueryKey::CallableSignature(callable) if !retired_conformance_witnesses.contains(&callable) => {
                        callable_signature_work.insert(callable.clone());
                        semantic_work_modules.insert(callable.module().clone());
                        diagnostic_work_modules.insert(callable.module().clone());
                        formal_work_modules.insert(callable.module().clone());
                        if let Some(declaration) = callable.try_declaration_owner() {
                            formal_declarations.insert(declaration.clone());
                            formal_declaration_modules.insert(declaration.module.clone());
                        }
                    }
                    QueryKey::CallableBody(callable) if !retired_conformance_witnesses.contains(&callable) => {
                        callable_body_work.insert(callable.clone());
                        prior_conformance_clients.insert(callable.clone());
                        semantic_work_modules.insert(callable.module().clone());
                        diagnostic_work_modules.insert(callable.module().clone());
                    }
                    _ => {}
                }
            }
            callable_signature_work.retain(|callable| !retired_conformance_witnesses.contains(callable));
            callable_body_work.retain(|callable| !retired_conformance_witnesses.contains(callable));
        }
        let structural_aggregates_reusable = previous_snapshot.as_ref().is_some_and(|previous| {
            hierarchy_work_modules.is_empty()
                && removed_modules.is_empty()
                && field_lifecycle_changed_modules.is_empty()
                && previous.semantic_graph.module_projection() == input.linked.graphs.semantics
        });

        // 2. Predeclare Every Source Declaration
        let mut declarations = self.base_declarations.clone();
        if let Some(previous) = previous_snapshot.as_ref() {
            for (declaration, info) in previous.declarations.iter() {
                let retained_from_unchanged_module = structural_reused_modules.contains(&declaration.module);
                let retained_header_in_changed_module = structural_recomputed_modules.contains(&declaration.module)
                    && current_modules.contains(&declaration.module)
                    && !generic_header_work.contains(declaration)
                    && self
                        .semantic_structure_shards
                        .get(&declaration.module)
                        .is_some_and(|shard| shard.declaration_header_fingerprints.contains_key(declaration));
                if retained_from_unchanged_module || retained_header_in_changed_module {
                    declarations.refresh_shell_preserving_registered_projections(info.clone());
                }
            }
        }
        let mut hierarchy = previous_snapshot
            .as_ref()
            .map_or_else(|| self.base_hierarchy.clone(), |snapshot| (*snapshot.hierarchy).clone());
        for declaration in &hierarchy_edge_work {
            hierarchy.remove_superclass(declaration);
        }
        for declaration in &contribution_delta.declarations {
            let declaration_still_exists = self
                .semantic_structure_shards
                .get(&declaration.module)
                .is_some_and(|shard| shard.declaration_header_fingerprints.contains_key(declaration));
            if declaration_still_exists {
                hierarchy.remove_template(declaration);
            } else {
                hierarchy.remove(declaration);
            }
        }
        for module in &removed_modules {
            hierarchy.remove_module(module);
        }
        let mut alias_sources = self.alias_sources.clone();
        for module in &structural_recomputed_modules {
            alias_sources.retain(|declaration, _| &declaration.module != module);
            if let Some(shard) = self.semantic_structure_shards.get(module) {
                alias_sources.extend(
                    shard
                        .alias_sources
                        .iter()
                        .map(|(declaration, alias)| (declaration.clone(), (module.clone(), alias.clone()))),
                );
            }
        }
        // A removed source overlay for a canonical Universe module reveals
        // that module's source-backed alias declarations again. Keep these
        // contributions in the same alias worklist as workspace aliases.
        if !removed_modules.is_empty() {
            for module in &removed_modules {
                alias_sources.retain(|declaration, _| &declaration.module != module);
                if module.project.is_universe() && self.db.canonical_image().is_some() {
                    continue;
                }
                let Ok(source) = self.universe_source_provider.load_parsed(module) else {
                    continue;
                };
                for statement in &source.program.statements {
                    let (Statement::TypeAlias(alias) | Statement::TypeDefinition(alias)) = statement else {
                        continue;
                    };
                    let declaration = DeclarationId::new(module.clone(), alias.name.clone().into());
                    alias_sources.insert(declaration, (module.clone(), alias.clone()));
                }
            }
        }
        let mut shell_table = DeclarationShellTable::default();
        let alias_declarations = alias_sources.keys().cloned().collect::<BTreeSet<_>>();
        let mut initial_blueprints: Vec<DeclarationBlueprint> = declarations
            .iter()
            .map(|(decl_id, _)| DeclarationBlueprint {
                id: decl_id.clone(),
                kind: DeclarationKind::Class,
            })
            .collect();

        for module in &structural_recomputed_modules {
            let Some(shard) = self.semantic_structure_shards.get(module) else {
                continue;
            };
            initial_blueprints.extend(
                shard
                    .declarations
                    .iter()
                    .filter(|blueprint| !declarations.is_intrinsic_static_domain(&blueprint.id))
                    .cloned(),
            );
        }

        for module_id in &structural_recomputed_modules {
            let Some(shard) = self.semantic_structure_shards.get(module_id) else {
                continue;
            };
            let parsed_unit = &shard.source;
            for stmt in &parsed_unit.program.statements {
                if let Statement::Class(class_def) = stmt {
                    let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                    let retain_previous_header = previous_snapshot
                        .as_ref()
                        .is_some_and(|previous| !generic_header_work.contains(&decl_id) && previous.declarations.get(&decl_id).is_some());
                    if retain_previous_header {
                        continue;
                    }
                    // Publish every nominal name before resolving any generic
                    // binder domain. Contextual kinds (for example `S: Shape`)
                    // need the complete declaration table and resolver, so
                    // binder headers are attached in a second pass below.
                    let kind = KindId::TYPE;
                    let form = Arc::make_mut(&mut self.store).nominal_form(decl_id.clone(), kind);
                    let class_obj_type = Arc::make_mut(&mut self.store).class_object_type(decl_id.clone());
                    let binding = crate::declarations::intrinsic_binding_for_source(module_id, class_def.name.as_str())
                        .unwrap_or_else(crate::declarations::DeclarationTypeBinding::ordinary);
                    declarations.replace_authoritative(DeclarationTypeInfo {
                        declaration: decl_id,
                        form,
                        class_object_type: class_obj_type,
                        kind,
                        binding,
                        generic_binder_header: None,
                        generic_signature: None,
                        supertype_template: None,
                    });
                } else if let Statement::Data(data_def) = stmt {
                    let decl_id = DeclarationId::new(module_id.clone(), data_def.name.clone().into());
                    if data_def.is_intrinsic_facade() {
                        continue;
                    }
                    let retain_previous_header = previous_snapshot
                        .as_ref()
                        .is_some_and(|previous| !generic_header_work.contains(&decl_id) && previous.declarations.get(&decl_id).is_some());
                    if retain_previous_header {
                        continue;
                    }
                    let result_kind = if data_def.phase == phalcom_ast::ast::AlgebraicPhase::TypeOnly {
                        let domain = Arc::make_mut(&mut self.store).nominal_static_domain(decl_id.clone());
                        Arc::make_mut(&mut self.store).static_domain_kind(domain)
                    } else {
                        KindId::TYPE
                    };
                    let kind = result_kind;
                    let form = Arc::make_mut(&mut self.store).nominal_form(decl_id.clone(), kind);
                    let class_object_type = Arc::make_mut(&mut self.store).class_object_type(decl_id.clone());
                    let binding = crate::declarations::intrinsic_binding_for_source(module_id, data_def.name.as_str())
                        .unwrap_or_else(crate::declarations::DeclarationTypeBinding::ordinary);
                    declarations.replace_authoritative(DeclarationTypeInfo {
                        declaration: decl_id,
                        form,
                        class_object_type,
                        kind,
                        binding,
                        generic_binder_header: None,
                        generic_signature: None,
                        supertype_template: None,
                    });
                }
            }
        }
        shell_table.predeclare(initial_blueprints);

        // 3. Construct LinkedTypeResolver
        let mut known_declarations: HashSet<DeclarationId> = declarations.iter().map(|(decl_id, _)| decl_id.clone()).collect();
        known_declarations.extend(declarations.intrinsic_static_domain_declarations().cloned());
        known_declarations.extend(alias_declarations.iter().cloned());
        for kind in [
            crate::checker::proof_predicate::ProofPredicateKind::Equivalent,
            crate::checker::proof_predicate::ProofPredicateKind::Subtype,
            crate::checker::proof_predicate::ProofPredicateKind::Conforms,
        ] {
            known_declarations.insert(crate::checker::proof_predicate::proof_predicate_declaration(kind));
        }
        let mut trait_associated_types: BTreeMap<DeclarationId, BTreeMap<String, crate::traits::AssociatedTypeRequirementId>> = BTreeMap::new();
        for (module, shard) in &self.semantic_structure_shards {
            for statement in &shard.source.program.statements {
                if let Statement::Trait(trait_def) = statement {
                    let decl = DeclarationId::new(module.clone(), trait_def.name.clone().into());
                    known_declarations.insert(decl.clone());
                    let mut associated_index = 0u32;
                    for member in &trait_def.members {
                        if let TraitMember::AssociatedType(assoc) = member {
                            trait_associated_types.entry(decl.clone()).or_default().insert(
                                assoc.name.clone(),
                                crate::traits::AssociatedTypeRequirementId::new(decl.clone(), associated_index),
                            );
                            associated_index += 1;
                        }
                    }
                }
            }
        }
        let canonical_linked = canonical_linked_program_for_session(&self.db, &self.universe_source_provider);
        let resolver = LinkedTypeResolver::with_retained_alias_forms(
            input.linked.clone(),
            known_declarations.clone(),
            ModuleId::universe_root(),
            self.alias_forms.clone(),
        )
        .with_canonical_linked(canonical_linked)
        .with_semantic_linked(input.semantic_linked.clone())
        .with_trait_associated_types(Arc::new(trait_associated_types.clone()));

        // Resolve contextual binder kinds only after every nominal declaration
        // shell is visible. The legacy predeclaration path handled builtin
        // kinds but rejected named static domains, which meant a declaration
        // such as `data Tensor<T: Type, S: Shape>()` never acquired a shell at
        // all. Full generic signatures are still checked in dependency order
        // below; this pass publishes only their stable binder header.
        for module_id in &structural_recomputed_modules {
            let Some(shard) = self.semantic_structure_shards.get(module_id) else {
                continue;
            };
            for statement in &shard.source.program.statements {
                let (declaration, parameters, where_clause, result_kind) = match statement {
                    Statement::Class(class_def) => (
                        DeclarationId::new(module_id.clone(), class_def.name.clone().into()),
                        class_def.generic_parameters.as_slice(),
                        class_def.where_clause.as_ref(),
                        KindId::TYPE,
                    ),
                    Statement::Data(data_def) => {
                        let declaration = DeclarationId::new(module_id.clone(), data_def.name.clone().into());
                        let result_kind = if data_def.phase == phalcom_ast::ast::AlgebraicPhase::TypeOnly {
                            let domain = Arc::make_mut(&mut self.store).nominal_static_domain(declaration.clone());
                            Arc::make_mut(&mut self.store).static_domain_kind(domain)
                        } else {
                            KindId::TYPE
                        };
                        (declaration, data_def.generic_parameters.as_slice(), data_def.where_clause.as_ref(), result_kind)
                    }
                    _ => continue,
                };
                if parameters.is_empty()
                    || previous_snapshot
                        .as_ref()
                        .is_some_and(|previous| !generic_header_work.contains(&declaration) && previous.declarations.get(&declaration).is_some())
                {
                    continue;
                }
                let Some(mut info) = declarations.get(&declaration).cloned() else {
                    continue;
                };
                let Some(header) = predeclare_nominal_binder_header(
                    Arc::make_mut(&mut self.store),
                    &declarations,
                    &resolver,
                    module_id,
                    &declaration,
                    parameters,
                    where_clause,
                ) else {
                    // The complete signature pass owns diagnostics and may
                    // become resolvable after an alias/member dependency.
                    continue;
                };
                let kind = Arc::make_mut(&mut self.store).arrow_kind(header.parameter_kinds.clone(), result_kind);
                info.kind = kind;
                info.form = Arc::make_mut(&mut self.store).nominal_form(declaration, kind);
                info.generic_binder_header = Some(header);
                declarations.refresh_shell_preserving_registered_projections(info);
            }
        }

        // 4. Enrich Semantic Graph
        let mut semantic_graph = input.linked.graphs.semantics.clone();
        if let Some(previous) = previous_snapshot.as_ref() {
            for module_id in current_modules.iter() {
                for edge in previous.semantic_graph.declaration_edges_from_module(module_id) {
                    let keep = match &edge.from {
                        SemanticNodeId::Declaration { module, name } => {
                            let declaration = DeclarationId::new(module.clone(), name.clone());
                            !hierarchy_edge_work.contains(&declaration)
                        }
                        SemanticNodeId::Module(_) => true,
                    };
                    if keep {
                        semantic_graph.add(edge);
                    }
                }
            }
        }
        if !structural_aggregates_reusable {
            for module_id in &hierarchy_work_modules {
                let Some(shard) = self.semantic_structure_shards.get(module_id) else {
                    continue;
                };
                let parsed_unit = &shard.source;
                for stmt in &parsed_unit.program.statements {
                    if let Statement::Class(class_def) = stmt {
                        let declaration = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                        if !structural_work_modules.contains(module_id) && !hierarchy_edge_work.contains(&declaration) {
                            continue;
                        }
                        let from_node = SemanticNodeId::Declaration {
                            module: module_id.clone(),
                            name: class_def.name.clone().into(),
                        };

                        if let Some(super_ref) = class_def.superclass_ref() {
                            let members: Vec<String> = super_ref.members.iter().map(|m| m.name.clone()).collect();
                            if let Some(target_decl) = resolver
                                .resolve_type_name(module_id, &super_ref.root, &members)
                                .or_else(|| canonical_runtime_support_superclass(&super_ref.root, &members))
                            {
                                let to_node = SemanticNodeId::Declaration {
                                    module: target_decl.module,
                                    name: target_decl.name,
                                };
                                semantic_graph.add(SemanticEdge {
                                    from: from_node.clone(),
                                    to: to_node,
                                    kind: SemanticEdgeKind::Superclass,
                                    range: super_ref.range,
                                });
                            }
                        } else if declaration != crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Object) {
                            let object = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Object);
                            semantic_graph.add(SemanticEdge {
                                from: from_node,
                                to: SemanticNodeId::Declaration {
                                    module: object.module,
                                    name: object.name,
                                },
                                kind: SemanticEdgeKind::Superclass,
                                range: class_def.range,
                            });
                        }
                    }
                }
            }
        }

        // 5. Realize Declaration Shells
        let mut diags_by_module: BTreeMap<ModuleId, Vec<SemanticDiagnostic>> = BTreeMap::new();
        for (module_id, diagnostics) in &self.canonical_native_diagnostics {
            diags_by_module.entry(module_id.clone()).or_default().extend(diagnostics.iter().cloned());
        }
        if let Some(previous) = previous_snapshot.as_ref() {
            for (module_id, diagnostics) in previous.diagnostics.iter() {
                if current_modules.contains(module_id) && !diagnostic_work_modules.contains(module_id) {
                    diags_by_module.insert(module_id.clone(), diagnostics.to_vec());
                } else if current_modules.contains(module_id) && !semantic_work_modules.contains(module_id) {
                    if let Some(semantic_diagnostics) = self.semantic_diagnostic_contributions.get(module_id) {
                        diags_by_module.insert(module_id.clone(), semantic_diagnostics.to_vec());
                    }
                }
            }
        }
        for (module_id, mod_diags) in &input.diagnostics {
            if diagnostic_work_modules.contains(module_id) {
                diags_by_module
                    .entry(module_id.clone())
                    .or_default()
                    .extend(semantic_diagnostics_from_module_diagnostics(mod_diags));
            }
        }
        // Publish the complete namespace from stable declaration shells before
        // checking any alias or source type-member signature. Checked member
        // products are added to the resolver independently as the signature
        // dependency scheduler makes them available.
        let mut associated_surfaces_table = previous_snapshot
            .as_ref()
            .map_or_else(|| self.base_associated_surfaces.clone(), |snapshot| (*snapshot.associated_surfaces).clone());
        let mut type_member_declarations_table = previous_snapshot.as_ref().map_or_else(
            || {
                self.base_type_member_declarations
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect()
            },
            |snapshot| (*snapshot.type_member_declarations).clone(),
        );
        for module in structural_recomputed_modules.iter().chain(removed_modules.iter()) {
            associated_surfaces_table.remove_module(module);
        }
        for member in &signature_member_work {
            type_member_declarations_table.remove(member);
        }
        let mut associated_surface_recompute_modules = structural_recomputed_modules.clone();
        associated_surface_recompute_modules.extend(removed_modules.iter().cloned());
        let (associated_surface_owners, source_associated_surfaces, current_inherent_impl_surface_owners) = build_source_associated_surfaces(
            &self.semantic_structure_shards,
            previous_structure_shards,
            &resolver,
            &declarations,
            Arc::make_mut(&mut self.store),
            &associated_surface_recompute_modules,
            &self.inherent_impl_surface_owners,
        );
        for owner in associated_surface_owners {
            associated_surfaces_table.remove(&owner);
        }
        for (declaration, surface, diagnostics) in source_associated_surfaces {
            diags_by_module
                .entry(declaration.module.clone())
                .or_default()
                .extend(diagnostics.iter().cloned());
            match query_associated_surface(&mut self.db, surface) {
                QueryOutcome::Ready(surface) => {
                    associated_surfaces_table.insert(declaration, surface);
                }
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            }
        }
        resolver.update_associated_type_members(&associated_surfaces_table, &type_member_declarations_table);
        // Case identity is structural source authority. Publish it before the
        // alias/header/member signature scheduler reads any annotation.
        let mut static_case_catalogs = BTreeMap::new();
        for (module_id, shard) in &self.semantic_structure_shards {
            let mut candidates = BTreeMap::<crate::identity::StaticDomainDeclarationId, Vec<&phalcom_ast::ast::AlgebraicDef>>::new();
            for statement in &shard.source.program.statements {
                match statement {
                    Statement::Data(definition) if definition.phase == phalcom_ast::ast::AlgebraicPhase::TypeOnly => {
                        if matches!(&definition.shape, phalcom_ast::ast::AlgebraicShapeSyntax::IntrinsicFacade(_)) {
                            continue;
                        }
                        let declaration = DeclarationId::new(module_id.clone(), definition.name.clone().into());
                        candidates
                            .entry(crate::identity::StaticDomainDeclarationId::Nominal(declaration))
                            .or_default()
                            .push(definition);
                    }
                    Statement::Class(class_def) => {
                        let owner = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                        for definition in &class_def.associated_type_data {
                            let id = crate::identity::AssociatedStaticDomainDeclarationId::new(owner.clone(), definition.name.clone());
                            if associated_surface_admits(
                                &associated_surfaces_table,
                                &owner,
                                &crate::associated::AssociatedMemberId::StaticDomain(id.clone()),
                            ) {
                                candidates
                                    .entry(crate::identity::StaticDomainDeclarationId::Associated(id))
                                    .or_default()
                                    .push(definition);
                            }
                        }
                    }
                    Statement::Impl(impl_def) if matches!(impl_def.kind, phalcom_ast::ast::ImplKind::Inherent) => {
                        if let Some(symbol) = impl_def.target.origin_symbol_ref() {
                            let members = symbol.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
                            if let Some(owner) = resolver.resolve_type_name(module_id, &symbol.root, &members) {
                                for definition in &impl_def.associated_type_data {
                                    let id = crate::identity::AssociatedStaticDomainDeclarationId::new(owner.clone(), definition.name.clone());
                                    if associated_surface_admits(
                                        &associated_surfaces_table,
                                        &owner,
                                        &crate::associated::AssociatedMemberId::StaticDomain(id.clone()),
                                    ) {
                                        candidates
                                            .entry(crate::identity::StaticDomainDeclarationId::Associated(id))
                                            .or_default()
                                            .push(definition);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            for (domain, definitions) in candidates {
                let [definition] = definitions.as_slice() else { continue };
                let product = Arc::new(crate::static_case::catalog_from_declaration(domain.clone(), definition));
                match query_static_domain_case_catalog(&mut self.db, product) {
                    QueryOutcome::Ready(product) => {
                        static_case_catalogs.insert(domain, product);
                    }
                    QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                    QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                    QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                    QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                }
            }
        }
        resolver.update_static_case_catalogs(&static_case_catalogs);
        let static_case_sources = collect_static_case_sources(&self.semantic_structure_shards, &resolver, &static_case_catalogs);
        let mut static_case_signatures_table = previous_snapshot
            .as_ref()
            .map_or_else(crate::ordered_map::OrderedMap::default, |snapshot| (*snapshot.static_case_signatures).clone());
        for case in &signature_case_work {
            static_case_signatures_table.remove(case);
        }
        resolver.update_static_case_signatures(&static_case_signatures_table);
        let mut blocked_declarations = BTreeSet::new();
        let mut type_aliases = previous_snapshot
            .as_ref()
            .map_or_else(TypeAliasTable::new, |snapshot| (*snapshot.type_aliases).clone());
        let mut static_definitions_table = previous_snapshot
            .as_ref()
            .map_or_else(StaticDefinitionTable::new, |snapshot| (*snapshot.static_definitions).clone());
        let mut static_constants_table = previous_snapshot
            .as_ref()
            .map_or_else(crate::static_constant::StaticConstantTable::new, |snapshot| {
                (*snapshot.static_constants).clone()
            });
        for module in &structural_recomputed_modules {
            static_constants_table.remove_module(module);
        }
        let mut alias_dependencies = self.alias_dependencies.clone();
        let mut alias_reverse_dependencies = self.alias_reverse_dependencies.clone();
        let mut alias_contract_dependencies = self.alias_contract_dependencies.clone();
        let mut alias_contract_reverse_dependencies = self.alias_contract_reverse_dependencies.clone();
        let mut alias_recomputed_declarations = declaration_shell_work
            .iter()
            .filter(|declaration| type_aliases.contains_key(declaration) || static_definitions_table.contains_key(declaration))
            .cloned()
            .collect::<BTreeSet<_>>();
        alias_recomputed_declarations.extend(alias_contract_work);
        alias_recomputed_declarations.extend(signature_alias_work);
        for module in &structural_recomputed_modules {
            alias_recomputed_declarations.extend(type_aliases.declarations_for_module(module).cloned());
            if let Some(shard) = self.semantic_structure_shards.get(module) {
                alias_recomputed_declarations.extend(shard.aliases.iter().cloned());
            }
        }
        if previous_snapshot.is_none() {
            // Canonical source aliases are seeded from Universe source AST at
            // session bootstrap, then lowered by this ordinary first-update
            // alias worklist rather than by a parallel prelude form table.
            alias_recomputed_declarations.extend(alias_sources.keys().cloned());
        }
        // Alias source edits are not identity edits, so the source shard delta
        // alone cannot seed retained alias consumers. Walk the compact retained
        // dependency graph in reverse to add exactly those consumers whose
        // lowered forms depend on a changed alias.
        let mut pending_alias_providers = alias_recomputed_declarations.iter().cloned().collect::<Vec<_>>();
        while let Some(provider) = pending_alias_providers.pop() {
            let Some(consumers) = self.alias_reverse_dependencies.get(&provider) else {
                continue;
            };
            for consumer in consumers {
                if alias_sources.contains_key(consumer) && alias_recomputed_declarations.insert(consumer.clone()) {
                    pending_alias_providers.push(consumer.clone());
                }
            }
        }
        let mut alias_form_updates = BTreeMap::new();
        for declaration in &alias_recomputed_declarations {
            resolver.remove_alias_form(declaration.clone());
            alias_form_updates.insert(declaration.clone(), None);
        }
        for module in &structural_recomputed_modules {
            type_aliases.remove_module(module);
            static_definitions_table.remove_module(module);
        }
        for declaration in &alias_recomputed_declarations {
            type_aliases.remove(declaration);
            static_definitions_table.remove(declaration);
            remove_reverse_dependencies(&mut alias_dependencies, &mut alias_reverse_dependencies, declaration);
            remove_reverse_dependencies(&mut alias_contract_dependencies, &mut alias_contract_reverse_dependencies, declaration);
        }
        if let Err(err) = shell_table.realize_semantic_graph(&semantic_graph) {
            match err {
                DeclarationRealizationError::InheritanceCycle { cycle } => {
                    if let Some(first_node) = cycle.first() {
                        let mod_id = match first_node {
                            SemanticNodeId::Module(m) => m.clone(),
                            SemanticNodeId::Declaration { module, .. } => module.clone(),
                        };
                        diags_by_module.entry(mod_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                            mod_id,
                            DiagnosticCode::AnnotationUnresolved,
                            format!("A class cannot extend itself: inheritance cycle detected: {cycle:?}"),
                            SourceRange::default(),
                        ));
                    }
                }
                DeclarationRealizationError::MissingShell(node) => {
                    let mod_id = match &node {
                        SemanticNodeId::Module(m) => m.clone(),
                        SemanticNodeId::Declaration { module, .. } => module.clone(),
                    };
                    diags_by_module.entry(mod_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                        mod_id,
                        DiagnosticCode::AnnotationUnresolved,
                        format!("missing declaration shell for {node:?}"),
                        SourceRange::default(),
                    ));
                }
            }
        }

        // Lower transparent aliases before class signatures so alias references
        // resolve through the same linked declaration resolver.
        for declaration in &alias_recomputed_declarations {
            let Some((module_id, alias)) = alias_sources.get(declaration) else {
                continue;
            };
            if let Some(body) = alias.type_form_body() {
                let mut dependencies = BTreeSet::new();
                collect_alias_dependencies(body, module_id, &resolver, &alias_declarations, &mut dependencies);
                replace_reverse_dependencies(&mut alias_dependencies, &mut alias_reverse_dependencies, declaration.clone(), dependencies);
                let mut contract_dependencies = BTreeSet::new();
                collect_type_annotation_declarations(body, module_id, &resolver, &mut contract_dependencies);
                contract_dependencies.retain(|dependency| declarations.generic_binder_header(dependency).is_some());
                replace_reverse_dependencies(
                    &mut alias_contract_dependencies,
                    &mut alias_contract_reverse_dependencies,
                    declaration.clone(),
                    contract_dependencies,
                );
            }
        }
        stats.alias_dependency_nodes_considered = alias_recomputed_declarations.len();
        let mut member_definitions = BTreeMap::new();
        let mut source_member_ids = BTreeSet::new();
        for (module_id, shard) in &self.semantic_structure_shards {
            for statement in &shard.source.program.statements {
                match statement {
                    Statement::Class(class_def) => {
                        let owner = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                        for member in &class_def.members {
                            let ClassMember::TypeMember(definition) = member else { continue };
                            let id = crate::types::type_member::TypeMemberDeclarationId::new(
                                owner.clone(),
                                crate::types::type_member::TypeMemberSelector::from_syntax(&definition.selector),
                            );
                            source_member_ids.insert(id.clone());
                            member_definitions.insert(id, (module_id.clone(), definition));
                        }
                    }
                    Statement::Impl(implementation) if matches!(implementation.kind, phalcom_ast::ast::ImplKind::Inherent) => {
                        let Some(symbol) = implementation.target.origin_symbol_ref() else { continue };
                        let members = symbol.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
                        let Some(owner) = resolver.resolve_type_name(module_id, &symbol.root, &members) else {
                            continue;
                        };
                        for member in &implementation.members {
                            let phalcom_ast::ast::ImplMember::TypeMember(definition) = member else {
                                continue;
                            };
                            let id = crate::types::type_member::TypeMemberDeclarationId::new(
                                owner.clone(),
                                crate::types::type_member::TypeMemberSelector::from_syntax(&definition.selector),
                            );
                            source_member_ids.insert(id.clone());
                            member_definitions.insert(id, (module_id.clone(), definition));
                        }
                    }
                    _ => {}
                }
            }
        }
        let mut signature_graph = BTreeMap::<TypeSignatureDependencyNode, BTreeSet<TypeSignatureDependencyNode>>::new();
        let mut signature_ranges = BTreeMap::<TypeSignatureDependencyNode, (ModuleId, SourceRange)>::new();
        let mut generic_header_sources = BTreeMap::<DeclarationId, (ModuleId, SourceRange)>::new();
        for (module_id, shard) in &self.semantic_structure_shards {
            for statement in &shard.source.program.statements {
                let header = match statement {
                    Statement::Class(definition) if !definition.generic_parameters.is_empty() => {
                        Some((DeclarationId::new(module_id.clone(), definition.name.clone().into()), definition.range))
                    }
                    Statement::Data(definition) if !definition.generic_parameters.is_empty() => {
                        Some((DeclarationId::new(module_id.clone(), definition.name.clone().into()), definition.range))
                    }
                    Statement::Trait(definition) if !definition.generic_parameters.is_empty() => {
                        Some((DeclarationId::new(module_id.clone(), definition.name.clone().into()), definition.range))
                    }
                    _ => None,
                };
                if let Some((declaration, range)) = header {
                    generic_header_sources.insert(declaration, (module_id.clone(), range));
                }
            }
        }
        let generic_header_nodes = generic_header_sources.keys().cloned().collect::<BTreeSet<_>>();
        let alias_nodes = alias_sources.keys().cloned().collect::<BTreeSet<_>>();
        for (declaration, (module_id, alias)) in &alias_sources {
            let node = TypeSignatureDependencyNode::Alias(declaration.clone());
            let mut dependencies = alias_dependencies
                .get(declaration)
                .into_iter()
                .flatten()
                .map(|dependency| TypeSignatureDependencyNode::Alias(dependency.clone()))
                .collect::<BTreeSet<_>>();
            if let Some(result_kind) = alias.result_kind() {
                let mut kind_dependencies = BTreeSet::new();
                collect_kind_signature_alias_dependencies(&result_kind, module_id, &resolver, &alias_nodes, &mut kind_dependencies);
                dependencies.extend(kind_dependencies.into_iter().map(TypeSignatureDependencyNode::Alias));
            }
            for parameter in alias.generic_parameters() {
                if let Some(kind) = &parameter.kind {
                    let mut kind_dependencies = BTreeSet::new();
                    collect_kind_signature_alias_dependencies(kind, module_id, &resolver, &alias_nodes, &mut kind_dependencies);
                    dependencies.extend(kind_dependencies.into_iter().map(TypeSignatureDependencyNode::Alias));
                }
            }
            if let Some(body) = alias.type_form_body() {
                let mut member_dependencies = BTreeSet::new();
                collect_source_member_projection_dependencies(body, module_id, &resolver, &source_member_ids, &mut member_dependencies);
                dependencies.extend(member_dependencies.into_iter().map(TypeSignatureDependencyNode::Member));
                collect_static_case_signature_dependencies(body, module_id, &resolver, &static_case_catalogs, &mut dependencies);
                let mut referenced_declarations = BTreeSet::new();
                collect_type_annotation_declarations(body, module_id, &resolver, &mut referenced_declarations);
                dependencies.extend(
                    referenced_declarations
                        .into_iter()
                        .filter(|dependency| generic_header_nodes.contains(dependency))
                        .map(TypeSignatureDependencyNode::Header),
                );
            }
            if let Some(result_kind) = alias.result_kind() {
                let mut header_dependencies = BTreeSet::new();
                collect_kind_signature_header_dependencies(&result_kind, module_id, &resolver, &generic_header_nodes, &mut header_dependencies);
                dependencies.extend(header_dependencies.into_iter().map(TypeSignatureDependencyNode::Header));
            }
            for parameter in alias.generic_parameters() {
                if let Some(kind) = &parameter.kind {
                    let mut header_dependencies = BTreeSet::new();
                    collect_kind_signature_header_dependencies(kind, module_id, &resolver, &generic_header_nodes, &mut header_dependencies);
                    dependencies.extend(header_dependencies.into_iter().map(TypeSignatureDependencyNode::Header));
                }
            }
            if let Some(where_clause) = &alias.where_clause {
                for constraint in &where_clause.constraints {
                    let mut annotations = Vec::new();
                    match constraint {
                        phalcom_ast::ast::GenericConstraintSyntax::Subtype { lower, upper, .. }
                        | phalcom_ast::ast::GenericConstraintSyntax::Equivalent { left: lower, right: upper, .. } => {
                            annotations.extend([lower, upper]);
                        }
                        phalcom_ast::ast::GenericConstraintSyntax::Conforms { subject, bounds, .. } => {
                            annotations.push(subject);
                            for bound in bounds {
                                annotations.push(&bound.trait_type);
                                annotations.extend(bound.associated_specializations.iter().map(|specialization| &specialization.value));
                            }
                        }
                        phalcom_ast::ast::GenericConstraintSyntax::KindFact { kind, .. } => {
                            let mut aliases = BTreeSet::new();
                            collect_kind_signature_alias_dependencies(kind, module_id, &resolver, &alias_nodes, &mut aliases);
                            dependencies.extend(aliases.into_iter().map(TypeSignatureDependencyNode::Alias));
                            let mut headers = BTreeSet::new();
                            collect_kind_signature_header_dependencies(kind, module_id, &resolver, &generic_header_nodes, &mut headers);
                            dependencies.extend(headers.into_iter().map(TypeSignatureDependencyNode::Header));
                        }
                        phalcom_ast::ast::GenericConstraintSyntax::Invalid { .. } => {}
                    }
                    for annotation in annotations {
                        let mut referenced = BTreeSet::new();
                        collect_type_annotation_declarations(annotation, module_id, &resolver, &mut referenced);
                        dependencies.extend(
                            referenced
                                .iter()
                                .filter(|dependency| alias_nodes.contains(*dependency))
                                .cloned()
                                .map(TypeSignatureDependencyNode::Alias),
                        );
                        dependencies.extend(
                            referenced
                                .into_iter()
                                .filter(|dependency| generic_header_nodes.contains(dependency))
                                .map(TypeSignatureDependencyNode::Header),
                        );
                        let mut projected = BTreeSet::new();
                        collect_source_member_projection_dependencies(annotation, module_id, &resolver, &source_member_ids, &mut projected);
                        dependencies.extend(projected.into_iter().map(TypeSignatureDependencyNode::Member));
                        collect_static_case_signature_dependencies(annotation, module_id, &resolver, &static_case_catalogs, &mut dependencies);
                    }
                }
            }
            signature_graph.insert(node.clone(), dependencies);
            signature_ranges.insert(node, (module_id.clone(), alias.range));
        }
        for (id, (module_id, definition)) in &member_definitions {
            let node = TypeSignatureDependencyNode::Member(id.clone());
            let mut dependencies = BTreeSet::new();
            let mut annotations = Vec::new();
            for parameter in &definition.parameters {
                match &parameter.domain {
                    phalcom_ast::ast::TypeMemberParameterDomainSyntax::ConstType(annotation) => annotations.push(annotation),
                    phalcom_ast::ast::TypeMemberParameterDomainSyntax::Kind(kind) => {
                        let mut kind_dependencies = BTreeSet::new();
                        collect_kind_signature_alias_dependencies(kind, module_id, &resolver, &alias_nodes, &mut kind_dependencies);
                        dependencies.extend(kind_dependencies.into_iter().map(TypeSignatureDependencyNode::Alias));
                        let mut header_dependencies = BTreeSet::new();
                        collect_kind_signature_header_dependencies(kind, module_id, &resolver, &generic_header_nodes, &mut header_dependencies);
                        dependencies.extend(header_dependencies.into_iter().map(TypeSignatureDependencyNode::Header));
                    }
                }
            }
            match &definition.result {
                phalcom_ast::ast::TypeMemberResultSyntax::Type(annotation) => annotations.push(annotation),
                phalcom_ast::ast::TypeMemberResultSyntax::Kind(kind) => {
                    let mut kind_dependencies = BTreeSet::new();
                    collect_kind_signature_alias_dependencies(kind, module_id, &resolver, &alias_nodes, &mut kind_dependencies);
                    dependencies.extend(kind_dependencies.into_iter().map(TypeSignatureDependencyNode::Alias));
                    let mut header_dependencies = BTreeSet::new();
                    collect_kind_signature_header_dependencies(kind, module_id, &resolver, &generic_header_nodes, &mut header_dependencies);
                    dependencies.extend(header_dependencies.into_iter().map(TypeSignatureDependencyNode::Header));
                }
            }
            for annotation in annotations {
                let mut declarations = BTreeSet::new();
                collect_type_annotation_declarations(annotation, module_id, &resolver, &mut declarations);
                dependencies.extend(
                    declarations
                        .into_iter()
                        .filter(|dependency| alias_nodes.contains(dependency))
                        .map(TypeSignatureDependencyNode::Alias),
                );
                let mut referenced_declarations = BTreeSet::new();
                collect_type_annotation_declarations(annotation, module_id, &resolver, &mut referenced_declarations);
                dependencies.extend(
                    referenced_declarations
                        .into_iter()
                        .filter(|dependency| generic_header_nodes.contains(dependency))
                        .map(TypeSignatureDependencyNode::Header),
                );
                let mut projected_members = BTreeSet::new();
                collect_source_member_projection_dependencies(annotation, module_id, &resolver, &source_member_ids, &mut projected_members);
                dependencies.extend(projected_members.into_iter().map(TypeSignatureDependencyNode::Member));
                collect_static_case_signature_dependencies(annotation, module_id, &resolver, &static_case_catalogs, &mut dependencies);
            }
            signature_graph.insert(node.clone(), dependencies);
            signature_ranges.insert(node, (module_id.clone(), definition.range));
        }
        for (module_id, shard) in &self.semantic_structure_shards {
            for statement in &shard.source.program.statements {
                let (declaration, parameters, where_clause, superclass, range) = match statement {
                    Statement::Class(definition) if !definition.generic_parameters.is_empty() => (
                        DeclarationId::new(module_id.clone(), definition.name.clone().into()),
                        definition.generic_parameters.as_slice(),
                        definition.where_clause.as_ref(),
                        definition.superclass.as_ref(),
                        definition.range,
                    ),
                    Statement::Data(definition) if !definition.generic_parameters.is_empty() => (
                        DeclarationId::new(module_id.clone(), definition.name.clone().into()),
                        definition.generic_parameters.as_slice(),
                        definition.where_clause.as_ref(),
                        None,
                        definition.range,
                    ),
                    Statement::Trait(definition) if !definition.generic_parameters.is_empty() => (
                        DeclarationId::new(module_id.clone(), definition.name.clone().into()),
                        definition.generic_parameters.as_slice(),
                        definition.where_clause.as_ref(),
                        None,
                        definition.range,
                    ),
                    _ => continue,
                };
                let node = TypeSignatureDependencyNode::Header(declaration.clone());
                let mut dependencies = BTreeSet::new();
                for parameter in parameters {
                    if let Some(kind) = &parameter.kind {
                        let mut alias_dependencies = BTreeSet::new();
                        collect_kind_signature_alias_dependencies(kind, module_id, &resolver, &alias_nodes, &mut alias_dependencies);
                        dependencies.extend(alias_dependencies.into_iter().map(TypeSignatureDependencyNode::Alias));
                        let mut header_dependencies = BTreeSet::new();
                        collect_kind_signature_header_dependencies(kind, module_id, &resolver, &generic_header_nodes, &mut header_dependencies);
                        dependencies.extend(header_dependencies.into_iter().map(TypeSignatureDependencyNode::Header));
                    }
                }
                if let Some(superclass) = superclass {
                    let mut referenced = BTreeSet::new();
                    collect_type_annotation_declarations(superclass, module_id, &resolver, &mut referenced);
                    dependencies.extend(
                        referenced
                            .iter()
                            .filter(|dependency| alias_nodes.contains(*dependency))
                            .cloned()
                            .map(TypeSignatureDependencyNode::Alias),
                    );
                    dependencies.extend(
                        referenced
                            .into_iter()
                            .filter(|dependency| generic_header_nodes.contains(dependency))
                            .map(TypeSignatureDependencyNode::Header),
                    );
                    let mut projected_members = BTreeSet::new();
                    collect_source_member_projection_dependencies(superclass, module_id, &resolver, &source_member_ids, &mut projected_members);
                    dependencies.extend(projected_members.into_iter().map(TypeSignatureDependencyNode::Member));
                }
                if let Some(where_clause) = where_clause {
                    for constraint in &where_clause.constraints {
                        let mut annotations = Vec::new();
                        match constraint {
                            phalcom_ast::ast::GenericConstraintSyntax::Subtype { lower, upper, .. }
                            | phalcom_ast::ast::GenericConstraintSyntax::Equivalent { left: lower, right: upper, .. } => {
                                annotations.extend([lower, upper]);
                            }
                            phalcom_ast::ast::GenericConstraintSyntax::Conforms { subject, bounds, .. } => {
                                annotations.push(subject);
                                for bound in bounds {
                                    annotations.push(&bound.trait_type);
                                    annotations.extend(bound.associated_specializations.iter().map(|specialization| &specialization.value));
                                }
                            }
                            phalcom_ast::ast::GenericConstraintSyntax::KindFact { kind, .. } => {
                                let mut aliases = BTreeSet::new();
                                collect_kind_signature_alias_dependencies(kind, module_id, &resolver, &alias_nodes, &mut aliases);
                                dependencies.extend(aliases.into_iter().map(TypeSignatureDependencyNode::Alias));
                                let mut headers = BTreeSet::new();
                                collect_kind_signature_header_dependencies(kind, module_id, &resolver, &generic_header_nodes, &mut headers);
                                dependencies.extend(headers.into_iter().map(TypeSignatureDependencyNode::Header));
                            }
                            phalcom_ast::ast::GenericConstraintSyntax::Invalid { .. } => {}
                        }
                        for annotation in annotations {
                            let mut referenced = BTreeSet::new();
                            collect_type_annotation_declarations(annotation, module_id, &resolver, &mut referenced);
                            dependencies.extend(
                                referenced
                                    .iter()
                                    .filter(|dependency| alias_nodes.contains(*dependency))
                                    .cloned()
                                    .map(TypeSignatureDependencyNode::Alias),
                            );
                            dependencies.extend(
                                referenced
                                    .into_iter()
                                    .filter(|dependency| generic_header_nodes.contains(dependency))
                                    .map(TypeSignatureDependencyNode::Header),
                            );
                            let mut projected_members = BTreeSet::new();
                            collect_source_member_projection_dependencies(annotation, module_id, &resolver, &source_member_ids, &mut projected_members);
                            dependencies.extend(projected_members.into_iter().map(TypeSignatureDependencyNode::Member));
                        }
                    }
                }
                dependencies.remove(&TypeSignatureDependencyNode::Header(declaration.clone()));
                signature_graph.insert(node.clone(), dependencies);
                signature_ranges.insert(node, (module_id.clone(), range));
            }
        }
        for (id, (module_id, owner, definition, header)) in &static_case_sources {
            let node = TypeSignatureDependencyNode::StaticCaseSignature(id.clone());
            let mut dependencies = BTreeSet::new();
            if generic_header_nodes.contains(owner) {
                dependencies.insert(TypeSignatureDependencyNode::Header(owner.clone()));
            }
            for annotation in static_case_payload_annotations(definition, id).unwrap_or_default() {
                let mut referenced = BTreeSet::new();
                collect_type_annotation_declarations(annotation, module_id, &resolver, &mut referenced);
                dependencies.extend(
                    referenced
                        .iter()
                        .filter(|dependency| alias_nodes.contains(*dependency))
                        .cloned()
                        .map(TypeSignatureDependencyNode::Alias),
                );
                dependencies.extend(
                    referenced
                        .iter()
                        .filter(|dependency| generic_header_nodes.contains(*dependency))
                        .cloned()
                        .map(TypeSignatureDependencyNode::Header),
                );
                let mut projected = BTreeSet::new();
                collect_source_member_projection_dependencies(annotation, module_id, &resolver, &source_member_ids, &mut projected);
                dependencies.extend(projected.into_iter().map(TypeSignatureDependencyNode::Member));
                collect_static_case_signature_dependencies(annotation, module_id, &resolver, &static_case_catalogs, &mut dependencies);
            }
            signature_graph.insert(node.clone(), dependencies);
            signature_ranges.insert(node, (module_id.clone(), header.source_range));
        }
        let mut type_signature_cycle_aliases = BTreeSet::new();
        let mut type_signature_cycle_headers = BTreeSet::new();
        let mut type_signature_cycle_members = BTreeSet::new();
        let mut type_signature_cycle_cases = BTreeSet::new();
        for component in type_signature_cycle_components(&signature_graph).into_iter().filter(|component| {
            component.iter().any(|node| {
                matches!(
                    node,
                    TypeSignatureDependencyNode::Member(_) | TypeSignatureDependencyNode::StaticCaseSignature(_)
                )
            })
        }) {
            type_signature_cycle_aliases.extend(component.iter().filter_map(|node| match node {
                TypeSignatureDependencyNode::Alias(declaration) => Some(declaration.clone()),
                TypeSignatureDependencyNode::Header(_) | TypeSignatureDependencyNode::Member(_) | TypeSignatureDependencyNode::StaticCaseSignature(_) => None,
            }));
            type_signature_cycle_headers.extend(component.iter().filter_map(|node| match node {
                TypeSignatureDependencyNode::Header(declaration) => Some(declaration.clone()),
                TypeSignatureDependencyNode::Alias(_) | TypeSignatureDependencyNode::Member(_) | TypeSignatureDependencyNode::StaticCaseSignature(_) => None,
            }));
            type_signature_cycle_members.extend(component.iter().filter_map(|node| match node {
                TypeSignatureDependencyNode::Member(id) => Some(id.clone()),
                TypeSignatureDependencyNode::Alias(_) | TypeSignatureDependencyNode::Header(_) | TypeSignatureDependencyNode::StaticCaseSignature(_) => None,
            }));
            type_signature_cycle_cases.extend(component.iter().filter_map(|node| match node {
                TypeSignatureDependencyNode::StaticCaseSignature(id) => Some(id.clone()),
                _ => None,
            }));
            if let Some(anchor) = component.first()
                && let Some((module_id, range)) = signature_ranges.get(anchor)
            {
                let participants = component.iter().map(|node| format!("{node:?}")).collect::<Vec<_>>().join(" -> ");
                diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                    module_id.clone(),
                    DiagnosticCode::TypeSignatureCycle,
                    format!("type-level declaration signature cycle: {participants}"),
                    *range,
                ));
            }
        }
        // A mixed SCC is invalid as one component. Remove any opportunistically
        // predeclared generic headers from its publication shell before aliases
        // or member signatures can consume them.
        for declaration in &type_signature_cycle_headers {
            blocked_declarations.insert(declaration.clone());
            if let Some(mut info) = declarations.get(declaration).cloned() {
                let result_kind = self
                    .semantic_structure_shards
                    .get(&declaration.module)
                    .and_then(|shard| {
                        shard.source.program.statements.iter().find_map(|statement| match statement {
                            Statement::Data(definition) if definition.name == declaration.name.as_ref() => {
                                Some(if definition.phase == phalcom_ast::ast::AlgebraicPhase::TypeOnly {
                                    let domain = Arc::make_mut(&mut self.store).nominal_static_domain(declaration.clone());
                                    Arc::make_mut(&mut self.store).static_domain_kind(domain)
                                } else {
                                    KindId::TYPE
                                })
                            }
                            Statement::Class(definition) if definition.name == declaration.name.as_ref() => Some(KindId::TYPE),
                            _ => None,
                        })
                    })
                    .unwrap_or(info.kind);
                info.kind = result_kind;
                info.form = Arc::make_mut(&mut self.store).nominal_form(declaration.clone(), result_kind);
                info.generic_binder_header = None;
                info.generic_signature = None;
                declarations.refresh_shell_preserving_registered_projections(info);
            }
        }
        let alias_cycle_seeds = if previous_snapshot.is_none() {
            alias_dependencies.keys().cloned().collect()
        } else {
            alias_recomputed_declarations.clone()
        };
        let mut alias_cycles = find_alias_cycles_from_seeds(&alias_dependencies, &alias_cycle_seeds);
        alias_cycles.retain(|declaration| !type_signature_cycle_aliases.contains(declaration));
        for declaration in &alias_cycles {
            let Some((module_id, alias)) = alias_sources.get(declaration) else {
                continue;
            };
            diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                module_id.clone(),
                DiagnosticCode::TypeAliasCycle,
                format!("type alias `{}` participates in a cycle", alias.name),
                alias.range,
            ));
        }

        // Resolve source members whose own signature prerequisites are already
        // available. This publishes one canonical surface early enough for
        // aliases that depend on those members; members waiting on aliases are
        // completed after alias lowering and before formal field/callable checks.
        let mut pending_members = source_member_ids
            .iter()
            .filter(|id| !type_signature_cycle_members.contains(*id) && !type_member_declarations_table.contains_key(*id))
            .cloned()
            .collect::<BTreeSet<_>>();
        while !pending_members.is_empty() {
            let ready_member = pending_members
                .iter()
                .find(|id| {
                    signature_graph
                        .get(&TypeSignatureDependencyNode::Member((*id).clone()))
                        .is_none_or(|dependencies| {
                            dependencies.iter().all(|dependency| match dependency {
                                TypeSignatureDependencyNode::Alias(alias) => !alias_sources.contains_key(alias) || type_aliases.contains_key(alias),
                                TypeSignatureDependencyNode::Header(header) => declarations.generic_binder_header(header).is_some(),
                                TypeSignatureDependencyNode::Member(member) => type_member_declarations_table.contains_key(member),
                                TypeSignatureDependencyNode::StaticCaseSignature(case) => static_case_signatures_table.contains_key(case),
                            })
                        })
                })
                .cloned();
            let Some(id) = ready_member else { break };
            let admitted = associated_surfaces_table.surfaces.get(&id.owner).is_some_and(|surface| {
                surface.families.values().any(|family| {
                    family
                        .members
                        .iter()
                        .any(|member| matches!(member, crate::associated::AssociatedMemberId::TypeMember(member) if member == &id))
                })
            });
            if !admitted {
                pending_members.remove(&id);
                continue;
            }
            let Some((module_id, definition)) = member_definitions.get(&id) else {
                pending_members.remove(&id);
                continue;
            };
            let Some(shard) = self.semantic_structure_shards.get(module_id) else {
                pending_members.remove(&id);
                continue;
            };
            let parsed_unit = &shard.source;
            let _ = publish_type_member_owner(
                &mut self.db,
                Arc::make_mut(&mut self.store),
                &resolver,
                &associated_surfaces_table,
                &mut type_member_declarations_table,
                &declarations,
                &id,
                module_id,
                definition,
                parsed_unit,
                &mut diags_by_module,
            )?;
            pending_members.remove(&id);
        }

        // Lower aliases after their alias dependencies. This matters for
        // imported aliases and preserves transparent substitution across
        // modules; cycles were removed above and never reach publication.
        let mut pending_aliases = alias_recomputed_declarations.clone();
        for declaration in alias_cycles.iter().chain(type_signature_cycle_aliases.iter()) {
            pending_aliases.remove(declaration);
        }
        let mut pending_static_cases = static_case_sources
            .keys()
            .filter(|case| !static_case_signatures_table.contains_key(*case) || signature_case_work.contains(*case))
            .filter(|case| !type_signature_cycle_cases.contains(*case))
            .cloned()
            .collect::<BTreeSet<_>>();
        loop {
            let ready_case = pending_static_cases
                .iter()
                .find(|case| {
                    signature_graph
                        .get(&TypeSignatureDependencyNode::StaticCaseSignature((*case).clone()))
                        .is_none_or(|dependencies| {
                            dependencies.iter().all(|dependency| match dependency {
                                TypeSignatureDependencyNode::Alias(alias) => !alias_sources.contains_key(alias) || type_aliases.contains_key(alias),
                                TypeSignatureDependencyNode::Header(header) => declarations.generic_binder_header(header).is_some(),
                                TypeSignatureDependencyNode::Member(member) => type_member_declarations_table.contains_key(member),
                                TypeSignatureDependencyNode::StaticCaseSignature(dependency) => static_case_signatures_table.contains_key(dependency),
                            })
                        })
                })
                .cloned();
            if let Some(case_id) = ready_case {
                if let Some((module_id, owner, definition, header)) = static_case_sources.get(&case_id)
                    && let Some(shard) = self.semantic_structure_shards.get(module_id)
                {
                    let (product, diagnostics) =
                        build_static_case_signature(Arc::make_mut(&mut self.store), &declarations, &resolver, module_id, owner, definition, header);
                    diags_by_module.entry(module_id.clone()).or_default().extend(diagnostics);
                    if let Some(product) = product {
                        match query_static_case_signature(&mut self.db, &shard.source, Arc::new(product)) {
                            QueryOutcome::Ready(product) => {
                                static_case_signatures_table.insert(case_id.clone(), product);
                                resolver.update_static_case_signatures(&static_case_signatures_table);
                            }
                            QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                            QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                            QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                            QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                        }
                    }
                }
                pending_static_cases.remove(&case_id);
                continue;
            }
            let ready_alias = pending_aliases
                .iter()
                .find(|declaration| {
                    alias_dependencies
                        .get(*declaration)
                        .is_none_or(|dependencies| dependencies.iter().all(|dependency| !pending_aliases.contains(dependency)))
                        && signature_graph
                            .get(&TypeSignatureDependencyNode::Alias((*declaration).clone()))
                            .is_none_or(|dependencies| {
                                dependencies.iter().all(|dependency| match dependency {
                                    TypeSignatureDependencyNode::Alias(alias) => !pending_aliases.contains(alias),
                                    TypeSignatureDependencyNode::Header(header) => declarations.generic_binder_header(header).is_some(),
                                    TypeSignatureDependencyNode::Member(member) => type_member_declarations_table.contains_key(member),
                                    TypeSignatureDependencyNode::StaticCaseSignature(case) => static_case_signatures_table.contains_key(case),
                                })
                            })
                })
                .cloned();
            let Some(declaration) = ready_alias else {
                let header_progress = resolve_missing_nominal_binder_headers(
                    Arc::make_mut(&mut self.store),
                    &mut declarations,
                    &resolver,
                    &self.semantic_structure_shards,
                    &type_signature_cycle_headers,
                );
                if header_progress {
                    continue;
                }
                let ready_member = pending_members
                    .iter()
                    .find(|id| {
                        associated_surface_admits(
                            &associated_surfaces_table,
                            &id.owner,
                            &crate::associated::AssociatedMemberId::TypeMember((*id).clone()),
                        ) && signature_graph
                            .get(&TypeSignatureDependencyNode::Member((*id).clone()))
                            .is_none_or(|dependencies| {
                                dependencies.iter().all(|dependency| match dependency {
                                    TypeSignatureDependencyNode::Alias(alias) => !alias_sources.contains_key(alias) || type_aliases.contains_key(alias),
                                    TypeSignatureDependencyNode::Header(header) => declarations.generic_binder_header(header).is_some(),
                                    TypeSignatureDependencyNode::Member(member) => type_member_declarations_table.contains_key(member),
                                    TypeSignatureDependencyNode::StaticCaseSignature(case) => static_case_signatures_table.contains_key(case),
                                })
                            })
                    })
                    .cloned();
                let Some(id) = ready_member else { break };
                if let Some((module_id, definition)) = member_definitions.get(&id)
                    && let Some(shard) = self.semantic_structure_shards.get(module_id)
                {
                    let _ = publish_type_member_owner(
                        &mut self.db,
                        Arc::make_mut(&mut self.store),
                        &resolver,
                        &associated_surfaces_table,
                        &mut type_member_declarations_table,
                        &declarations,
                        &id,
                        module_id,
                        definition,
                        &shard.source,
                        &mut diags_by_module,
                    )?;
                }
                pending_members.remove(&id);
                continue;
            };
            let Some((module_id, alias)) = alias_sources.get(&declaration) else {
                pending_aliases.remove(&declaration);
                continue;
            };
            let site = TypeFormationSite::module(module_id.clone());
            let dependencies = alias_dependencies.get(&declaration).cloned().unwrap_or_default();
            let mut diagnostics = Vec::new();
            let product = resolve_static_definition(
                Arc::make_mut(&mut self.store),
                &declarations,
                &resolver,
                &site,
                alias,
                module_id,
                dependencies.into_iter().collect::<Vec<_>>().into_boxed_slice(),
                &mut diagnostics,
            );
            diags_by_module.entry(module_id.clone()).or_default().extend(diagnostics);
            let product = match query_static_definition(&mut self.db, Arc::new(product)) {
                QueryOutcome::Ready(product) => product,
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            };
            if let Some(info) = &product.alias_info {
                resolver.insert_alias_form(declaration.clone(), info.form);
                alias_form_updates.insert(declaration.clone(), Some(info.form));
                type_aliases.insert((**info).clone());
            }
            static_definitions_table.insert(product);
            pending_aliases.remove(&declaration);
        }
        resolver.update_static_definitions(&static_definitions_table);

        // Generic signatures and supertype templates. Retain the dependency
        // contribution for unchanged headers; only structurally changed or
        // exact reverse-dependent headers are rebuilt below.
        let mut next_generic_header_dependencies = self.generic_header_dependencies.clone();
        let mut next_generic_header_reverse_dependencies = self.generic_header_reverse_dependencies.clone();
        for module in &structural_recomputed_modules {
            if let Some(previous_shard) = previous_structure_shards.get(module) {
                for declaration in previous_shard.declaration_header_fingerprints.keys() {
                    let still_declared = self
                        .semantic_structure_shards
                        .get(module)
                        .is_some_and(|shard| shard.declaration_header_fingerprints.contains_key(declaration));
                    if !still_declared || generic_header_work.contains(declaration) {
                        remove_reverse_dependencies(
                            &mut next_generic_header_dependencies,
                            &mut next_generic_header_reverse_dependencies,
                            declaration,
                        );
                    }
                }
            }
        }
        let generic_header_modules = generic_header_work
            .iter()
            .map(|declaration| declaration.module.clone())
            .collect::<BTreeSet<_>>();
        for module_id in generic_header_modules {
            let Some(shard) = self.semantic_structure_shards.get(&module_id) else {
                continue;
            };
            let parsed_unit = &shard.source;
            'source_declaration: for stmt in &parsed_unit.program.statements {
                if let Statement::Class(class_def) = stmt {
                    let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                    if type_signature_cycle_headers.contains(&decl_id) {
                        continue;
                    }
                    if previous_snapshot.is_some() && !generic_header_work.contains(&decl_id) {
                        continue;
                    }
                    let mut header_dependencies = BTreeSet::new();
                    if let Some(superclass) = &class_def.superclass {
                        collect_type_annotation_declarations(superclass, &module_id, &resolver, &mut header_dependencies);
                    }
                    if let Some(where_clause) = &class_def.where_clause {
                        for constraint in &where_clause.constraints {
                            match constraint {
                                phalcom_ast::ast::GenericConstraintSyntax::Subtype { lower, upper, .. }
                                | phalcom_ast::ast::GenericConstraintSyntax::Equivalent { left: lower, right: upper, .. } => {
                                    collect_type_annotation_declarations(lower, &module_id, &resolver, &mut header_dependencies);
                                    collect_type_annotation_declarations(upper, &module_id, &resolver, &mut header_dependencies);
                                }
                                phalcom_ast::ast::GenericConstraintSyntax::Conforms { subject, bounds, .. } => {
                                    collect_type_annotation_declarations(subject, &module_id, &resolver, &mut header_dependencies);
                                    for bound in bounds {
                                        collect_type_annotation_declarations(&bound.trait_type, &module_id, &resolver, &mut header_dependencies);
                                        for spec in &bound.associated_specializations {
                                            collect_type_annotation_declarations(&spec.value, &module_id, &resolver, &mut header_dependencies);
                                        }
                                    }
                                }
                                phalcom_ast::ast::GenericConstraintSyntax::KindFact { .. } => {}
                                phalcom_ast::ast::GenericConstraintSyntax::Invalid { .. } => {}
                            }
                        }
                    }
                    header_dependencies.remove(&decl_id);
                    replace_reverse_dependencies(
                        &mut next_generic_header_dependencies,
                        &mut next_generic_header_reverse_dependencies,
                        decl_id.clone(),
                        header_dependencies,
                    );
                    let formation_site = TypeFormationSite::member(module_id.clone(), decl_id.clone(), DispatchSide::Instance);
                    let mut invalid_generic_signature = false;
                    let generic_signature = if !class_def.generic_parameters.is_empty() {
                        let outcome = resolve_generic_signature(
                            Arc::make_mut(&mut self.store),
                            &declarations,
                            &resolver,
                            &formation_site,
                            TypeParameterOwner::Declaration(decl_id.clone()),
                            GenericBinderSite::NominalDeclaration,
                            &class_def.generic_parameters,
                            class_def.where_clause.as_ref(),
                            diags_by_module.entry(module_id.clone()).or_default(),
                        );
                        invalid_generic_signature = matches!(&outcome, TypeFormationOutcome::Invalid(_));
                        Some(retain_generic_signature(
                            outcome,
                            &module_id,
                            class_def.range,
                            diags_by_module.entry(module_id.clone()).or_default(),
                        ))
                        .flatten()
                    } else {
                        None
                    };

                    if !class_def.generic_parameters.is_empty() && generic_signature.is_none() {
                        if invalid_generic_signature {
                            blocked_declarations.insert(decl_id.clone());
                        }
                        continue;
                    }
                    if let (Some(header), Some(signature)) = (declarations.generic_binder_header(&decl_id), generic_signature.as_ref()) {
                        assert!(
                            header.matches_signature(signature),
                            "final class signature must complete its predeclared binder header"
                        );
                    }
                    let header = NominalDeclarationHeader::from_signature(Arc::make_mut(&mut self.store), decl_id.clone(), generic_signature);

                    let supertype_template = if let Some(super_ann) = &class_def.superclass {
                        let type_params_map = if let Some(ref sig) = header.generic_signature {
                            let mut map = std::collections::HashMap::new();
                            for &param_id in sig.parameters.iter() {
                                let name = self.store.type_parameter(param_id).name.to_string();
                                let binding = type_level_binding_for_parameter(Arc::make_mut(&mut self.store), param_id);
                                map.insert(name, binding);
                            }
                            map
                        } else {
                            std::collections::HashMap::new()
                        };
                        let scoped_resolver = crate::types::annotation::ScopedTypeResolver {
                            parent: &resolver,
                            type_parameters: type_params_map,
                        };
                        let mut diags = Vec::new();
                        let form_res = crate::types::annotation::resolve_type_form(
                            Arc::make_mut(&mut self.store),
                            &declarations,
                            &scoped_resolver,
                            &formation_site,
                            super_ann,
                            &mut diags,
                        );
                        let super_ty = match form_res {
                            crate::types::annotation::TypeFormResolution::Ready(ty) => {
                                // The bundled Universe source uses one erased
                                // generic protocol form for collection
                                // inheritance (`Iterable` rather than
                                // `Iterable<T>`). Keep only that exact
                                // source-backed form available for formal body
                                // queries; user modules retain strict
                                // proper-type validation below.
                                if self.store.kind_of(ty) == KindId::TYPE {
                                    Some(ty)
                                } else if let Some(declaration) = canonical_erased_superclass_declaration(&resolver, &module_id, super_ann) {
                                    // Canonical collection classes expose an
                                    // erased superclass spelling in ordinary
                                    // source (`class X is Collection`). Form
                                    // that spelling as the canonical nominal
                                    // instance type; arbitrary generic
                                    // constructors remain rejected below.
                                    Some(Arc::make_mut(&mut self.store).nominal_type(declaration))
                                } else {
                                    diags.push(SemanticDiagnostic::error_in(
                                        module_id.clone(),
                                        DiagnosticCode::KindExpectedType,
                                        "superclass must be a proper type",
                                        super_ann.range,
                                    ));
                                    diags_by_module.entry(module_id.clone()).or_default().extend(diags);
                                    blocked_declarations.insert(decl_id.clone());
                                    continue 'source_declaration;
                                }
                            }
                            crate::types::annotation::TypeFormResolution::Dynamic
                            | crate::types::annotation::TypeFormResolution::Missing(_)
                            | crate::types::annotation::TypeFormResolution::Unresolved(_)
                            | crate::types::annotation::TypeFormResolution::Invalid(_)
                            | crate::types::annotation::TypeFormResolution::Blocked(_)
                            | crate::types::annotation::TypeFormResolution::Cancelled
                            | crate::types::annotation::TypeFormResolution::BudgetExceeded(_)
                            | crate::types::annotation::TypeFormResolution::InternalFailure(_) => {
                                diags_by_module.entry(module_id.clone()).or_default().extend(diags);
                                blocked_declarations.insert(decl_id.clone());
                                continue 'source_declaration;
                            }
                        };
                        diags_by_module.entry(module_id.clone()).or_default().extend(diags);
                        super_ty.map(|ty| GenericSupertypeTemplate::from_type(&self.store, decl_id.clone(), ty))
                    } else {
                        None
                    };

                    let binding = crate::declarations::intrinsic_binding_for_source(&module_id, class_def.name.as_str())
                        .unwrap_or_else(crate::declarations::DeclarationTypeBinding::ordinary);
                    let type_info = header.into_type_info_with_binding(supertype_template, binding);
                    if let Some(template) = type_info.supertype_template.clone() {
                        hierarchy.insert_template(template);
                    }
                    declarations.replace_authoritative(type_info);
                } else if let Statement::Data(data_def) = stmt {
                    if data_def.is_intrinsic_facade() {
                        continue;
                    }
                    let decl_id = DeclarationId::new(module_id.clone(), data_def.name.clone().into());
                    if type_signature_cycle_headers.contains(&decl_id) {
                        continue;
                    }
                    if previous_snapshot.is_some() && !generic_header_work.contains(&decl_id) {
                        continue;
                    }
                    let mut header_dependencies = BTreeSet::new();
                    if let Some(where_clause) = &data_def.where_clause {
                        for constraint in &where_clause.constraints {
                            match constraint {
                                phalcom_ast::ast::GenericConstraintSyntax::Subtype { lower, upper, .. }
                                | phalcom_ast::ast::GenericConstraintSyntax::Equivalent { left: lower, right: upper, .. } => {
                                    collect_type_annotation_declarations(lower, &module_id, &resolver, &mut header_dependencies);
                                    collect_type_annotation_declarations(upper, &module_id, &resolver, &mut header_dependencies);
                                }
                                phalcom_ast::ast::GenericConstraintSyntax::Conforms { subject, bounds, .. } => {
                                    collect_type_annotation_declarations(subject, &module_id, &resolver, &mut header_dependencies);
                                    for bound in bounds {
                                        collect_type_annotation_declarations(&bound.trait_type, &module_id, &resolver, &mut header_dependencies);
                                        for spec in &bound.associated_specializations {
                                            collect_type_annotation_declarations(&spec.value, &module_id, &resolver, &mut header_dependencies);
                                        }
                                    }
                                }
                                phalcom_ast::ast::GenericConstraintSyntax::KindFact { .. } => {}
                                phalcom_ast::ast::GenericConstraintSyntax::Invalid { .. } => {}
                            }
                        }
                    }
                    header_dependencies.remove(&decl_id);
                    replace_reverse_dependencies(
                        &mut next_generic_header_dependencies,
                        &mut next_generic_header_reverse_dependencies,
                        decl_id.clone(),
                        header_dependencies,
                    );
                    let formation_site = TypeFormationSite::member(module_id.clone(), decl_id.clone(), DispatchSide::Instance);
                    let mut invalid_generic_signature = false;
                    let generic_signature = if !data_def.generic_parameters.is_empty() {
                        let outcome = resolve_generic_signature(
                            Arc::make_mut(&mut self.store),
                            &declarations,
                            &resolver,
                            &formation_site,
                            TypeParameterOwner::Declaration(decl_id.clone()),
                            GenericBinderSite::NominalDeclaration,
                            &data_def.generic_parameters,
                            data_def.where_clause.as_ref(),
                            diags_by_module.entry(module_id.clone()).or_default(),
                        );
                        invalid_generic_signature = matches!(&outcome, TypeFormationOutcome::Invalid(_));
                        Some(retain_generic_signature(
                            outcome,
                            &module_id,
                            data_def.range,
                            diags_by_module.entry(module_id.clone()).or_default(),
                        ))
                        .flatten()
                    } else {
                        None
                    };

                    if !data_def.generic_parameters.is_empty() && generic_signature.is_none() {
                        if invalid_generic_signature {
                            blocked_declarations.insert(decl_id.clone());
                        }
                        continue;
                    }
                    if let (Some(header), Some(signature)) = (declarations.generic_binder_header(&decl_id), generic_signature.as_ref()) {
                        assert!(
                            header.matches_signature(signature),
                            "final data signature must complete its predeclared binder header"
                        );
                    }
                    let result_kind = if data_def.phase == phalcom_ast::ast::AlgebraicPhase::TypeOnly {
                        let domain = Arc::make_mut(&mut self.store).nominal_static_domain(decl_id.clone());
                        Arc::make_mut(&mut self.store).static_domain_kind(domain)
                    } else {
                        KindId::TYPE
                    };
                    let header = NominalDeclarationHeader::from_signature_with_result_kind(
                        Arc::make_mut(&mut self.store),
                        decl_id.clone(),
                        generic_signature,
                        result_kind,
                    );

                    let binding = crate::declarations::intrinsic_binding_for_source(&module_id, data_def.name.as_str())
                        .unwrap_or_else(crate::declarations::DeclarationTypeBinding::ordinary);
                    declarations.replace_authoritative(header.into_type_info_with_binding(None, binding));
                }
            }
        }

        // Trait headers are contract metadata, not nominal declaration types.
        // Resolve their stable declaration-owned generic binders through the
        // same canonical generic-signature path, but publish them separately
        // from `DeclarationTypeTable` so no fake form or class object exists.
        let mut trait_headers = previous_snapshot.as_ref().map(|snapshot| (*snapshot.trait_headers).clone()).unwrap_or_default();
        // Trait headers are source-owned formal products. A structural
        // replacement or module removal retires every prior trait contract
        // from that module before current source is rebuilt below.
        for module in structural_recomputed_modules.iter().chain(removed_modules.iter()) {
            trait_headers.remove_module(module);
        }
        for module in type_signature_cycle_headers
            .iter()
            .map(|declaration| declaration.module.clone())
            .collect::<BTreeSet<_>>()
        {
            trait_headers.remove_module(&module);
        }
        for (module_id, shard) in &self.semantic_structure_shards {
            for stmt in &shard.source.program.statements {
                let Statement::Trait(trait_def) = stmt else { continue };
                let decl_id = DeclarationId::new(module_id.clone(), trait_def.name.clone().into());
                // The prior snapshot may contain a valid header for this
                // declaration. Retire it before rebuilding so an invalid
                // edited binder cannot leave stale trait evidence published.
                trait_headers.remove(&decl_id);
                if type_signature_cycle_headers.contains(&decl_id) {
                    continue;
                }
                let generic_signature = if !trait_def.generic_parameters.is_empty() || trait_def.where_clause.is_some() {
                    let associated_type_context = trait_associated_types.get(&decl_id).cloned().unwrap_or_default();
                    let trait_ref = TraitRef::new(decl_id.clone(), Vec::new());
                    let formation_site =
                        TypeFormationSite::trait_member(module_id.clone(), decl_id.clone(), DispatchSide::Instance, trait_ref, associated_type_context);
                    let outcome = resolve_generic_signature(
                        Arc::make_mut(&mut self.store),
                        &declarations,
                        &resolver,
                        &formation_site,
                        TypeParameterOwner::Declaration(decl_id.clone()),
                        GenericBinderSite::Trait,
                        &trait_def.generic_parameters,
                        trait_def.where_clause.as_ref(),
                        diags_by_module.entry(module_id.clone()).or_default(),
                    );
                    Some(retain_generic_signature(
                        outcome,
                        module_id,
                        trait_def.range,
                        diags_by_module.entry(module_id.clone()).or_default(),
                    ))
                    .flatten()
                } else {
                    None
                };
                if (!trait_def.generic_parameters.is_empty() || trait_def.where_clause.is_some()) && generic_signature.is_none() {
                    continue;
                }
                let header = Arc::new(TraitHeader {
                    declaration: decl_id,
                    generic_signature,
                    source: crate::diagnostic::SemanticSourceSpan::new(module_id.clone(), trait_def.range),
                });
                let header = match crate::db::query::query_bootstrap_trait_header(&mut self.db, header) {
                    QueryOutcome::Ready(header) => header,
                    QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                    QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                    QueryOutcome::Blocked(reason) => {
                        return Err(QueryOutcome::Blocked(reason));
                    }
                    QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                };
                trait_headers.insert(header);
            }
        }

        // Trait surfaces are semantic contract products, not declaration
        // surfaces. Publish every member signature before any body-analysis
        // work so defaults cannot influence the contract shape they consume.
        let mut trait_surfaces = previous_snapshot
            .as_ref()
            .map(|snapshot| (*snapshot.trait_surfaces).clone())
            .unwrap_or_default();
        // Recomputed modules must not retain deleted trait surfaces. Retaining
        // one would allow its default-member signatures to be reintroduced
        // after the callable-removal delta has already retired them.
        for module in structural_recomputed_modules.iter().chain(removed_modules.iter()) {
            trait_surfaces.remove_module(module);
        }
        for (module_id, shard) in &self.semantic_structure_shards {
            for stmt in &shard.source.program.statements {
                let Statement::Trait(trait_def) = stmt else { continue };
                let declaration = DeclarationId::new(module_id.clone(), trait_def.name.clone().into());
                let Some(header) = trait_headers.get(&declaration) else { continue };
                let mut context = CheckingContext::new(Arc::make_mut(&mut self.store), &hierarchy, &resolver, &declarations, module_id.clone());
                let surface = build_trait_surface(&mut context, header, trait_def);
                diags_by_module
                    .entry(module_id.clone())
                    .or_default()
                    .extend(surface.diagnostics.iter().cloned());
                let surface = match crate::db::query::query_bootstrap_trait_surface(&mut self.db, Arc::new(surface)) {
                    QueryOutcome::Ready(surface) => surface,
                    QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                    QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                    QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                    QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                };
                trait_surfaces.insert(surface);
            }
        }

        // Ensure declaration query authority before any formal surface,
        // signature, or body query can consume it. An image-backed session
        // consumes the exact frozen shell product rather than reconstructing
        // query input from the final aggregate declaration table: the aggregate
        // may have gained later projections while the immutable query root
        // correctly retains its original checked input fingerprint.
        let mut published_shells = BTreeSet::new();
        for (declaration, info) in self.base_declarations.iter() {
            published_shells.insert(declaration.clone());
            let key = QueryKey::DeclarationShell(declaration.clone());
            if self.db.is_canonical_query(&key) {
                let Some(shell) = self.db.product(&key).and_then(|product| product.as_declaration_shell()) else {
                    return Err(QueryOutcome::Failed(format!(
                        "canonical declaration shell {declaration:?} has no typed DeclarationShell product"
                    )));
                };
                if shell.declaration() != declaration {
                    return Err(QueryOutcome::Failed(format!(
                        "canonical declaration shell key {declaration:?} disagrees with product owner {:?}",
                        shell.declaration()
                    )));
                }
                self.db.metrics().record_hit();
                continue;
            }
            match query_declaration_shell(&mut self.db, Arc::new(TypeDeclarationShell::Nominal(info.clone()))) {
                QueryOutcome::Ready(_) => {}
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            }
        }
        for (declaration, _) in self.base_declarations.iter() {
            let key = QueryKey::DeclarationSurface(declaration.clone());
            if self.db.is_canonical_query(&key) {
                let Some(surface) = self.db.product(&key).and_then(|product| product.as_declaration_surface()) else {
                    return Err(QueryOutcome::Failed(format!(
                        "canonical declaration surface {declaration:?} has no typed DeclarationSurface product"
                    )));
                };
                if surface.id.as_ref() != Some(declaration) {
                    return Err(QueryOutcome::Failed(format!(
                        "canonical declaration surface key {declaration:?} disagrees with product owner {:?}",
                        surface.id
                    )));
                }
                self.db.metrics().record_hit();
                continue;
            }
            let surface = self
                .base_dispatch
                .surface(declaration)
                .cloned()
                .unwrap_or_else(|| crate::surface::DeclarationSurface::new(Some(declaration.clone())));
            match query_bootstrap_declaration_surface(&mut self.db, declaration.clone(), Arc::new(surface)) {
                QueryOutcome::Ready(_) => {}
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            }
        }
        for (_, signature) in self.base_callable_signatures.iter() {
            let key = QueryKey::CallableSignature(signature.callable.clone());
            if self.db.is_canonical_query(&key) {
                let Some(canonical_signature) = self.db.product(&key).and_then(|product| product.as_callable_signature()) else {
                    return Err(QueryOutcome::Failed(format!(
                        "canonical callable signature {:?} has no typed CallableSignature product",
                        signature.callable
                    )));
                };
                if canonical_signature.callable != signature.callable {
                    return Err(QueryOutcome::Failed(format!(
                        "canonical callable signature key {:?} disagrees with product owner {:?}",
                        signature.callable, canonical_signature.callable
                    )));
                }
                self.db.metrics().record_hit();
                continue;
            }
            match query_bootstrap_callable_signature(&mut self.db, Arc::new(signature.clone())) {
                QueryOutcome::Ready(_) => {}
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            }
        }
        // Canonical source declarations (for example Duration) participate in
        // tracked hierarchy dependencies just like native bootstrap classes.
        // Imported images may retain HierarchyEdge as dependency evidence
        // without admitting it as a reusable typed query root. In that case
        // the immutable image fingerprint is the authority and no local query
        // state may be synthesized for the canonical declaration.
        for (class_decl, _) in self.base_declarations.iter() {
            let key = QueryKey::HierarchyEdge(class_decl.clone());
            if self.db.is_dependency_current(&key) {
                continue;
            }
            match query_bootstrap_hierarchy_edge(&mut self.db, class_decl.clone(), self.base_hierarchy.superclasses.get(class_decl).cloned()) {
                QueryOutcome::Ready(_) => {}
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            }
        }

        // Publish alias shells in dependency order so each edge records the
        // dependency's actual structural product fingerprint.
        let alias_shell_work = alias_recomputed_declarations.clone();
        let mut pending_alias_shells = alias_shell_work
            .iter()
            .filter(|declaration| type_aliases.contains_key(declaration))
            .cloned()
            .collect::<BTreeSet<_>>();
        while let Some(declaration) = pending_alias_shells
            .iter()
            .find(|declaration| {
                type_aliases
                    .get(declaration)
                    .is_none_or(|info| info.dependencies.iter().all(|dependency| !pending_alias_shells.contains(dependency)))
            })
            .cloned()
        {
            let Some(info) = type_aliases.get(&declaration).cloned() else {
                pending_alias_shells.remove(&declaration);
                continue;
            };
            if published_shells.insert(declaration.clone()) {
                match query_declaration_shell(&mut self.db, Arc::new(TypeDeclarationShell::Alias(info))) {
                    QueryOutcome::Ready(_) => {}
                    QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                    QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                    QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                    QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                }
            }
            pending_alias_shells.remove(&declaration);
        }
        if !structural_aggregates_reusable {
            let mut shell_work = declaration_shell_work.clone();
            shell_work.extend(alias_shell_work.iter().cloned());
            for declaration in shell_work {
                if blocked_declarations.contains(&declaration) || !current_modules.contains(&declaration.module) {
                    continue;
                }
                let Some(shell) = declarations
                    .get(&declaration)
                    .cloned()
                    .map(TypeDeclarationShell::Nominal)
                    .or_else(|| type_aliases.get(&declaration).cloned().map(TypeDeclarationShell::Alias))
                else {
                    // The worklist also contains deleted declaration identities;
                    // their contribution was removed before publication.
                    continue;
                };
                if published_shells.insert(declaration) {
                    match query_declaration_shell(&mut self.db, Arc::new(shell)) {
                        QueryOutcome::Ready(_) => {}
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Blocked(reason) => {
                            return Err(QueryOutcome::Blocked(reason));
                        }
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    }
                }
            }
        }

        // Publish/validate linked-interface prerequisites before declaration queries.
        for (module_id, linked_mod) in &input.linked.modules {
            match query_linked_interface(&mut self.db, module_id.clone(), Arc::new(linked_mod.interface.clone())) {
                QueryOutcome::Ready(_) => {}
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            }
        }

        // Build the compatibility hierarchy exclusively from DB-owned hierarchy-edge queries.
        // A changed direct edge can expose a new superclass path to a body that
        // is already in the reverse worklist. Revalidate the source-owned
        // edges on that exact path as well; this remains a bounded closure from
        // typed hierarchy roots, not a scan of the workspace hierarchy.
        let mut pending_hierarchy = hierarchy_query_work.iter().cloned().collect::<Vec<_>>();
        let mut visited_hierarchy = BTreeSet::new();
        while let Some(declaration) = pending_hierarchy.pop() {
            if !visited_hierarchy.insert(declaration.clone()) {
                continue;
            }
            let Some(shard) = self.semantic_structure_shards.get(&declaration.module) else {
                continue;
            };
            let Some(super_ref) = shard.source.program.statements.iter().find_map(|statement| match statement {
                Statement::Class(class_def) if class_def.name.as_str() == declaration.name.as_ref() => class_def.superclass_ref(),
                _ => None,
            }) else {
                continue;
            };
            let members = super_ref.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
            let Some(super_decl) = resolver.resolve_type_name(&declaration.module, &super_ref.root, &members) else {
                continue;
            };
            if current_modules.contains(&super_decl.module) && hierarchy_query_work.insert(super_decl.clone()) {
                pending_hierarchy.push(super_decl);
            }
        }

        let hierarchy_query_modules = hierarchy_query_work
            .iter()
            .map(|declaration| declaration.module.clone())
            .collect::<BTreeSet<_>>();
        for module_id in &hierarchy_query_modules {
            let Some(shard) = self.semantic_structure_shards.get(module_id) else {
                continue;
            };
            let parsed_unit = &shard.source;
            let Some(linked_module) = input.linked.modules.get(module_id) else {
                return Err(QueryOutcome::Failed(format!(
                    "linked module prerequisite is missing for semantic source {module_id:?}"
                )));
            };
            let linked_interface = Arc::new(linked_module.interface.clone());

            for stmt in &parsed_unit.program.statements {
                if let Statement::Class(class_def) = stmt {
                    let class_decl = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                    if blocked_declarations.contains(&class_decl) || (previous_snapshot.is_some() && !hierarchy_query_work.contains(&class_decl)) {
                        continue;
                    }
                    let edge = match query_hierarchy_edge(
                        &mut self.db,
                        class_decl.clone(),
                        parsed_unit.clone(),
                        HierarchyEdgeQueryInputs {
                            linked_interface: linked_interface.clone(),
                            resolver: &resolver,
                            linked: input.linked.as_ref(),
                            declarations: &declarations,
                            import_products: &input.import_products,
                        },
                    ) {
                        QueryOutcome::Ready(edge) => edge,
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Blocked(reason) => {
                            return Err(QueryOutcome::Blocked(reason));
                        }
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    };

                    if let Some(super_decl) = &edge.super_decl {
                        hierarchy.insert(class_decl.clone(), super_decl.clone());
                    } else if !edge.diagnostics.is_empty() {
                        diags_by_module.entry(module_id.clone()).or_default().extend(edge.diagnostics.iter().cloned());
                    } else if let Some(super_ref) = class_def.superclass_ref() {
                        diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                            module_id.clone(),
                            DiagnosticCode::AnnotationUnresolved,
                            format!("unresolved superclass `{}`", super_ref.root),
                            super_ref.range,
                        ));
                    }
                } else if let Statement::Data(data_def) = stmt {
                    let decl = DeclarationId::new(module_id.clone(), data_def.name.clone().into());
                    if data_def.is_intrinsic_facade() {
                        continue;
                    }
                    if blocked_declarations.contains(&decl) || (previous_snapshot.is_some() && !hierarchy_query_work.contains(&decl)) {
                        continue;
                    }
                    let edge = match query_hierarchy_edge(
                        &mut self.db,
                        decl.clone(),
                        parsed_unit.clone(),
                        HierarchyEdgeQueryInputs {
                            linked_interface: linked_interface.clone(),
                            resolver: &resolver,
                            linked: input.linked.as_ref(),
                            declarations: &declarations,
                            import_products: &input.import_products,
                        },
                    ) {
                        QueryOutcome::Ready(edge) => edge,
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Blocked(reason) => {
                            return Err(QueryOutcome::Blocked(reason));
                        }
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    };
                    if let Some(super_decl) = &edge.super_decl {
                        hierarchy.insert(decl, super_decl.clone());
                    }
                }
            }
        }

        // 6. Materialize compatibility dispatch/signature tables from DB-owned formal products.
        let mut dispatch = previous_snapshot
            .as_ref()
            .map_or_else(|| self.base_dispatch.clone(), |snapshot| (*snapshot.dispatch).clone());
        let mut callable_signatures = previous_snapshot
            .as_ref()
            .map_or_else(|| self.base_callable_signatures.clone(), |snapshot| (*snapshot.callable_signatures).clone());
        let mut callable_definitions = previous_snapshot
            .as_ref()
            .map_or_else(crate::parent_map::ParentMap::default, |snapshot| (*snapshot.callable_definitions).clone());
        let mut field_signatures = previous_snapshot
            .as_ref()
            .map_or_else(FieldSignatureTable::new, |snapshot| (*snapshot.field_signatures).clone());
        for callable in &retired_conformance_witnesses {
            callable_definitions.remove(callable);
            callable_signatures.remove(callable);
            self.db.retire_query(&QueryKey::CallableDefinition(callable.clone()));
            self.db.retire_query(&QueryKey::CallableSignature(callable.clone()));
            self.db.retire_query(&QueryKey::CallableBody(callable.clone()));
            self.db.retire_query(&QueryKey::AdvisoryCallable(callable.clone()));
        }
        let previous_callable_formal_states = callable_signature_work
            .iter()
            .filter_map(|callable| callable_signatures.get(callable).cloned().map(|signature| (callable.clone(), signature)))
            .collect::<BTreeMap<_, _>>();
        for declaration in declaration_surface_work
            .iter()
            .filter(|declaration| current_modules.contains(&declaration.module))
        {
            dispatch.remove_surface(declaration);
        }
        for declaration in &contribution_delta.declarations_removed {
            dispatch.remove_surface(declaration);
        }
        for callable in callable_signature_work.iter().filter(|callable| current_modules.contains(callable.module())) {
            callable_signatures.remove(callable);
        }
        for callable in &contribution_delta.callable_signatures_removed {
            callable_signatures.remove(callable);
            callable_definitions.remove(callable);
        }
        for field in field_signature_work.iter().filter(|field| current_modules.contains(&field.owner.module)) {
            field_signatures.remove(field);
        }
        for field in &contribution_delta.field_signatures_removed {
            field_signatures.remove(field);
        }
        for module in &removed_modules {
            dispatch.remove_module(module);
            callable_signatures.remove_module(module);
            field_signatures.remove_module(module);
        }

        // A retained effective definition is still the canonical input when
        // its module has no structural work. Signature/body replay must make
        // that input current before recording a CallableDefinition edge. Modules
        // with structural work rebuild their definitions below instead.
        for definition in callable_definitions.values().filter(|definition| {
            current_modules.contains(definition.callable.module())
                && !structural_work_modules.contains(definition.callable.module())
                && (formal_work_modules.contains(definition.callable.module()) || semantic_work_modules.contains(definition.callable.module()))
        }) {
            match query_callable_definition(&mut self.db, Arc::new(definition.clone())) {
                QueryOutcome::Ready(_) => {}
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            }
        }

        // Body-bearing trait defaults use the same canonical signature table
        // as ordinary declaration-owned callables. Their initial contract is
        // the already-published trait surface signature; the existing body
        // refresh below may then publish a normal-return summary into that
        // same table. TraitSurface remains a contract projection, not a
        // second owner of inferred return knowledge.
        let mut trait_default_callables = BTreeSet::new();
        for (_, surface) in trait_surfaces.iter() {
            for (_, member) in surface.iter() {
                if !member.default_present {
                    continue;
                }
                trait_default_callables.insert(member.callable.clone());
                let signature = if let Some(signature) = callable_signatures.get(&member.callable).cloned() {
                    signature
                } else {
                    let mut signature = member.signature.clone();
                    if let Some(previous) = previous_callable_formal_states.get(&member.callable) {
                        if previous.parameters == signature.parameters && previous.declared_return == signature.declared_return {
                            signature.return_validation = previous.return_validation;
                            signature.inferred_return = previous.inferred_return.clone();
                        }
                    }
                    callable_signatures.insert(signature.clone());
                    signature
                };
                match query_bootstrap_callable_signature(&mut self.db, Arc::new(signature)) {
                    QueryOutcome::Ready(_) => {}
                    QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                    QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                    QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                    QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                }
            }
        }

        for base_prod in &self.base_data_products {
            let _ = query_data_declaration(&mut self.db, base_prod.clone());
        }
        for base_prod in &self.base_enum_products {
            let _ = query_enum_declaration(&mut self.db, base_prod.clone());
        }
        for base_assoc in &self.base_associated_surface_products {
            let _ = query_associated_surface(&mut self.db, base_assoc.clone());
        }
        for base_tm in &self.base_type_member_products {
            let _ = query_type_member_declaration(&mut self.db, None, base_tm.clone());
        }
        for (decl_id, base_req) in &self.base_enum_requirement_products {
            let _ = query_enum_requirements(&mut self.db, decl_id.clone(), base_req.clone());
        }

        let static_intrinsics =
            crate::static_intrinsic::StaticIntrinsicRegistry::from_canonical_core().map_err(|error| QueryOutcome::Failed(error.to_string()))?;
        let early_data_semantics = self.base_data_semantics.clone();
        let early_enum_semantics = self.base_enum_semantics.clone();
        let no_deferred_static_definitions = BTreeSet::new();
        for module_id in &formal_declaration_modules {
            let Some(shard) = self.semantic_structure_shards.get(module_id) else {
                continue;
            };
            let parsed_unit = &shard.source;
            let Some(linked_module) = input.linked.modules.get(module_id) else {
                return Err(QueryOutcome::Failed(format!(
                    "linked module prerequisite is missing for semantic source {module_id:?}"
                )));
            };
            let linked_interface = Arc::new(linked_module.interface.clone());

            for stmt in &parsed_unit.program.statements {
                let Statement::Class(class_def) = stmt else {
                    continue;
                };
                let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                if blocked_declarations.contains(&decl_id) || !formal_declarations.contains(&decl_id) {
                    continue;
                }
                for member in &class_def.members {
                    let Some(field_id) = crate::checker::declaration_signature::field_id_for_member(&decl_id, member) else {
                        continue;
                    };
                    if previous_snapshot.is_some() && !field_signature_work.contains(&field_id) {
                        continue;
                    }
                    let key_projections = if let ClassMember::Field(field) = member
                        && let Some(annotation) = field.annotation.as_ref()
                        && !crate::static_key::type_record_key_expressions(annotation, 0).is_empty()
                    {
                        loop {
                            let projection = {
                                let mut driver = crate::static_constant_demand::StaticConstantDemandDriver {
                                    db: &mut self.db,
                                    store: Arc::make_mut(&mut self.store),
                                    values: Arc::make_mut(&mut self.static_values),
                                    constants: &mut static_constants_table,
                                    definitions: &static_definitions_table,
                                    catalogs: &module_static_binding_catalogs,
                                    sources: &retained_sources,
                                    linked: input.linked.as_ref(),
                                    topology: semantic_topology.as_ref(),
                                    import_products: &input.import_products,
                                    source_resolution_input,
                                    linked_component_product,
                                    hierarchy: &hierarchy,
                                    resolver: &resolver,
                                    declarations: &declarations,
                                    type_aliases: &type_aliases,
                                    dispatch: &dispatch,
                                    field_signatures: &field_signatures,
                                    callable_signatures: &callable_signatures,
                                    data_semantics: &early_data_semantics,
                                    enum_semantics: &early_enum_semantics,
                                    associated_families: &associated_surfaces_table,
                                    deferred_static_definitions: &no_deferred_static_definitions,
                                    intrinsics: &static_intrinsics,
                                    budget,
                                    cancel,
                                    active_constants: Vec::new(),
                                };
                                driver.project_annotation(QueryKey::FieldSignature(field_id.clone()), module_id, annotation)
                            };
                            match projection {
                                crate::static_constant_demand::StaticDemandOutcome::Ready(projections) => break Some(projections),
                                crate::static_constant_demand::StaticDemandOutcome::Awaiting(demands) => {
                                    match service_type_member_demands(
                                        &mut self.db,
                                        Arc::make_mut(&mut self.store),
                                        &resolver,
                                        &demands,
                                        &signature_graph,
                                        &alias_sources,
                                        &type_aliases,
                                        &declarations,
                                        &associated_surfaces_table,
                                        &mut type_member_declarations_table,
                                        &static_case_signatures_table,
                                        &type_signature_cycle_members,
                                        &member_definitions,
                                        &retained_sources,
                                        &mut diags_by_module,
                                    )? {
                                        TypeMemberDemandService::Current => continue,
                                        TypeMemberDemandService::Cycle => break None,
                                        TypeMemberDemandService::NoProgress => {
                                            return Err(QueryOutcome::Failed(format!(
                                                "type-member demand for field signature {field_id:?} could not progress: {demands:?}"
                                            )));
                                        }
                                    }
                                }
                                crate::static_constant_demand::StaticDemandOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                                crate::static_constant_demand::StaticDemandOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                                crate::static_constant_demand::StaticDemandOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                                crate::static_constant_demand::StaticDemandOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                            }
                        }
                    } else {
                        None
                    };
                    match query_field_signature_with_inputs(
                        &mut self.db,
                        field_id,
                        parsed_unit.clone(),
                        SignatureQueryInputs {
                            store: Arc::make_mut(&mut self.store),
                            hierarchy: &hierarchy,
                            resolver: &resolver,
                            record_key_projections: key_projections.as_ref(),
                            declarations: &declarations,
                            linked: Some(input.linked.as_ref()),
                            type_aliases: Some(&type_aliases),
                            import_products: Some(&input.import_products),
                        },
                    ) {
                        QueryOutcome::Ready(signature) => field_signatures.insert((*signature).clone()),
                        QueryOutcome::Blocked(reason) => {
                            return Err(QueryOutcome::Blocked(reason));
                        }
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    }
                }
                // Publish declaration-owned callable signatures first. Dispatch
                // surfaces are compatibility projections of these facts.
                for member in &class_def.members {
                    let Some(callable_id) = crate::checker::declaration_signature::callable_id_for_member(&decl_id, member) else {
                        continue;
                    };
                    // A previous snapshot does not prove that this callable already
                    // had a signature: newly imported or newly accepted owners
                    // can enter the structural set during an incremental update.
                    // Skip only when the current canonical projection actually
                    // retains the signature; otherwise publish from source.
                    if previous_snapshot.is_some() && !callable_signature_work.contains(&callable_id) && callable_signatures.get(&callable_id).is_some() {
                        continue;
                    }
                    let annotations = callable_type_annotations(member);
                    let key_projections = if annotations
                        .iter()
                        .any(|annotation| !crate::static_key::type_record_key_expressions(annotation, 0).is_empty())
                    {
                        loop {
                            let projection = {
                                let mut driver = crate::static_constant_demand::StaticConstantDemandDriver {
                                    db: &mut self.db,
                                    store: Arc::make_mut(&mut self.store),
                                    values: Arc::make_mut(&mut self.static_values),
                                    constants: &mut static_constants_table,
                                    definitions: &static_definitions_table,
                                    catalogs: &module_static_binding_catalogs,
                                    sources: &retained_sources,
                                    linked: input.linked.as_ref(),
                                    topology: semantic_topology.as_ref(),
                                    import_products: &input.import_products,
                                    source_resolution_input,
                                    linked_component_product,
                                    hierarchy: &hierarchy,
                                    resolver: &resolver,
                                    declarations: &declarations,
                                    type_aliases: &type_aliases,
                                    dispatch: &dispatch,
                                    field_signatures: &field_signatures,
                                    callable_signatures: &callable_signatures,
                                    data_semantics: &early_data_semantics,
                                    enum_semantics: &early_enum_semantics,
                                    associated_families: &associated_surfaces_table,
                                    deferred_static_definitions: &no_deferred_static_definitions,
                                    intrinsics: &static_intrinsics,
                                    budget,
                                    cancel,
                                    active_constants: Vec::new(),
                                };
                                driver.project_annotations(QueryKey::CallableSignature(callable_id.clone()), module_id, annotations.iter().copied())
                            };
                            match projection {
                                crate::static_constant_demand::StaticDemandOutcome::Ready(projections) => break Some(projections),
                                crate::static_constant_demand::StaticDemandOutcome::Awaiting(demands) => {
                                    match service_type_member_demands(
                                        &mut self.db,
                                        Arc::make_mut(&mut self.store),
                                        &resolver,
                                        &demands,
                                        &signature_graph,
                                        &alias_sources,
                                        &type_aliases,
                                        &declarations,
                                        &associated_surfaces_table,
                                        &mut type_member_declarations_table,
                                        &static_case_signatures_table,
                                        &type_signature_cycle_members,
                                        &member_definitions,
                                        &retained_sources,
                                        &mut diags_by_module,
                                    )? {
                                        TypeMemberDemandService::Current => continue,
                                        TypeMemberDemandService::Cycle => break None,
                                        TypeMemberDemandService::NoProgress => {
                                            return Err(QueryOutcome::Failed(format!(
                                                "type-member demand for callable signature {callable_id:?} could not progress: {demands:?}"
                                            )));
                                        }
                                    }
                                }
                                crate::static_constant_demand::StaticDemandOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                                crate::static_constant_demand::StaticDemandOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                                crate::static_constant_demand::StaticDemandOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                                crate::static_constant_demand::StaticDemandOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                            }
                        }
                    } else {
                        None
                    };
                    match query_callable_signature_with_inputs(
                        &mut self.db,
                        callable_id.clone(),
                        parsed_unit.clone(),
                        SignatureQueryInputs {
                            store: Arc::make_mut(&mut self.store),
                            hierarchy: &hierarchy,
                            resolver: &resolver,
                            record_key_projections: key_projections.as_ref(),
                            declarations: &declarations,
                            linked: Some(input.linked.as_ref()),
                            type_aliases: Some(&type_aliases),
                            import_products: Some(&input.import_products),
                        },
                    ) {
                        QueryOutcome::Ready(signature) => {
                            let mut signature = (*signature).clone();
                            if let Some(previous) = previous_callable_formal_states.get(&callable_id) {
                                if previous.parameters == signature.parameters && previous.declared_return == signature.declared_return {
                                    signature.return_validation = previous.return_validation;
                                    signature.inferred_return = previous.inferred_return.clone();
                                }
                            }
                            callable_signatures.insert(signature)
                        }
                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    }
                }
                let mut delegation_context =
                    crate::checker::CheckingContext::new(Arc::make_mut(&mut self.store), &hierarchy, &resolver, &declarations, module_id.clone());
                delegation_context.attach_field_signatures(&field_signatures);
                for member in &class_def.members {
                    let phalcom_ast::ast::ClassMember::Delegation(delegation) = member else {
                        continue;
                    };
                    if let Ok(generated) = crate::checker::declaration_signature::delegated_accessor_signatures(&mut delegation_context, &decl_id, delegation) {
                        for signature in generated {
                            callable_signatures.insert(signature);
                        }
                    }
                }

                let surface = match query_declaration_surface(
                    &mut self.db,
                    DeclarationSurfaceQuery {
                        decl_id: decl_id.clone(),
                        unit: parsed_unit.clone(),
                        linked_interface: linked_interface.clone(),
                        store: Arc::make_mut(&mut self.store),
                        hierarchy: &hierarchy,
                        resolver: &resolver,
                        declarations: &declarations,
                        field_signatures: Some(&field_signatures),
                        callable_signatures: Some(&callable_signatures),
                        type_aliases: Some(&type_aliases),
                        linked: Some(input.linked.as_ref()),
                        import_products: Some(&input.import_products),
                    },
                ) {
                    QueryOutcome::Ready(surface) => surface,
                    QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                    QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                    QueryOutcome::Blocked(reason) => {
                        return Err(QueryOutcome::Blocked(reason));
                    }
                    QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                };
                if let Some(diagnostics) = self
                    .db
                    .product(&QueryKey::DeclarationSurface(decl_id.clone()))
                    .and_then(|product| product.as_declaration_surface_diagnostics())
                {
                    diags_by_module.entry(module_id.clone()).or_default().extend(diagnostics.iter().cloned());
                }

                dispatch.register_surface(decl_id.clone(), (*surface).clone());
                if let Some(ty) = declarations.form(&decl_id) {
                    dispatch.register_type(ty, decl_id.clone());
                }
                // A declaration-surface rebuild starts from source-formal
                // evidence, which may be weaker than a retained callable
                // signature that has already been validated by its body.
                // Restore only this declaration's exact callable projections
                // before body analysis so a structurally unrelated edit
                // cannot make incremental callers observe a different
                // evidence level from a cold session.
                for member in &class_def.members {
                    let Some(callable) = crate::checker::declaration_signature::callable_id_for_member(&decl_id, member) else {
                        continue;
                    };
                    let Some(signature) = callable_signatures.get(&callable) else {
                        continue;
                    };
                    dispatch.update_callable_return_type(&callable, signature.published_return_knowledge());
                }
            }
        }

        // Constructor identities are semantic products even when the source
        // class does not spell the generated factory or initializer. Rebuild
        // each affected module from its retained source shard and canonical
        // field/callable signatures; neither the source index nor compiler
        // attribute expansion owns these identities.
        let mut constructor_initializations = previous_snapshot.as_ref().map_or_else(ConstructorInitializationTable::default, |snapshot| {
            (*snapshot.constructor_initializations).clone()
        });
        let mut constructor_product_work_modules = formal_declaration_modules
            .iter()
            .cloned()
            .chain(structural_work_modules.iter().cloned())
            .chain(removed_modules.iter().cloned())
            .collect::<BTreeSet<_>>();
        let mut constructor_product_work_owners = self
            .semantic_structure_shards
            .values()
            .filter(|shard| constructor_product_work_modules.contains(&shard.module))
            .flat_map(|shard| shard.declaration_header_fingerprints.keys().cloned())
            .collect::<BTreeSet<_>>();
        loop {
            let mut newly_affected = Vec::new();
            for (child, parent) in &hierarchy.superclasses {
                if constructor_product_work_owners.contains(parent) && !constructor_product_work_owners.contains(child) {
                    constructor_product_work_owners.insert(child.clone());
                    newly_affected.push(child.clone());
                }
            }
            if newly_affected.is_empty() {
                break;
            }
            constructor_product_work_modules.extend(newly_affected.iter().map(|owner| owner.module.clone()));
        }
        for module in &constructor_product_work_modules {
            let prior_products = constructor_initializations
                .iter()
                .filter(|(factory, _)| factory.module() == module)
                .map(|(factory, product)| (factory.clone(), product.initializer.clone(), product.origin))
                .collect::<Vec<_>>();
            for (factory, initializer, origin) in prior_products {
                callable_signatures.remove(&initializer);
                if origin != ConstructorOrigin::Source {
                    callable_signatures.remove(&factory);
                }
            }
            constructor_initializations.remove_module(module);
        }

        for module_id in &constructor_product_work_modules {
            let Some(shard) = self.semantic_structure_shards.get(module_id) else {
                continue;
            };
            for statement in &shard.source.program.statements {
                let Statement::Class(class_def) = statement else {
                    continue;
                };
                let owner = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                let instance_fields = class_def
                    .members
                    .iter()
                    .filter_map(|member| match member {
                        ClassMember::Field(field) if crate::checker::declaration::member_side(member) == DispatchSide::Instance => {
                            crate::checker::declaration_signature::field_id_for_member(&owner, member).map(|field_id| (field, field_id))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                let default_fields = instance_fields
                    .iter()
                    .filter(|(field, _)| field.default.is_some())
                    .map(|(_, field_id)| field_id.clone())
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                let field_input_fingerprint = constructor_field_input_fingerprint(&instance_fields, &shard.source.text);

                let source_constructors = class_def
                    .members
                    .iter()
                    .filter_map(|member| match member {
                        ClassMember::Method(method) if method.is_constructor || method.attributes.iter().any(|attribute| attribute.name == "constructor") => {
                            Some(method)
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                for method in &source_constructors {
                    let Some(factory) = crate::checker::declaration_signature::callable_id_for_member(&owner, &ClassMember::Method((*method).clone())) else {
                        continue;
                    };
                    let Some(factory_signature) = callable_signatures.get(&factory).cloned() else {
                        continue;
                    };
                    let initializer_selector = Selector::method(format!("init {}", method.name), factory.selector.slots.to_vec())
                        .expect("a valid constructor selector remains valid with a generated initializer name");
                    let initializer = CallableId::new(factory.owner.clone(), initializer_selector, DispatchSide::Instance);
                    let initializer_signature = initializer_signature_for_factory(&factory_signature, initializer.clone());
                    callable_signatures.insert(initializer_signature);
                    constructor_initializations.insert(ConstructorInitializationProduct {
                        factory,
                        initializer,
                        origin: ConstructorOrigin::Source,
                        field_input_fingerprint,
                        default_fields: default_fields.clone(),
                        parameter_fields: Box::new([]),
                    });
                }

                let has_construct_attribute = class_def.attributes.iter().any(|attribute| attribute.name == "construct");
                let has_data_attribute = class_def.attributes.iter().any(|attribute| attribute.name == "data");
                let source_new = source_constructors.iter().any(|method| method.name == "new");
                let parent_requires_initialization = hierarchy.superclasses.get(&owner).is_some_and(|parent| {
                    constructor_initializations.iter().any(|(factory, _)| factory.owner == *parent)
                        || self.semantic_structure_shards.get(&parent.module).is_some_and(|parent_shard| {
                            parent_shard.source.program.statements.iter().any(|statement| {
                                matches!(statement, Statement::Class(parent_class) if parent_class.name == parent.name.as_ref() && class_declares_constructor_lifecycle(parent_class))
                            })
                        })
                });
                let generated_origin = if has_construct_attribute {
                    Some(ConstructorOrigin::Construct)
                } else if has_data_attribute && !source_new {
                    Some(ConstructorOrigin::Data)
                } else if source_constructors.is_empty()
                    && !has_construct_attribute
                    && !has_data_attribute
                    && !default_fields.is_empty()
                    && !parent_requires_initialization
                {
                    Some(ConstructorOrigin::DefaultOnly)
                } else {
                    None
                };
                let Some(origin) = generated_origin else {
                    continue;
                };

                let parameter_fields = if matches!(origin, ConstructorOrigin::Construct | ConstructorOrigin::Data) {
                    instance_fields
                        .iter()
                        .filter(|(field, _)| field.default.is_none())
                        .map(|(field, field_id)| {
                            let label = strip_constructor_field_prefix(&field.name);
                            let declared_type = field_signatures
                                .get(field_id)
                                .map(|signature| signature.declared_type.clone())
                                .unwrap_or_else(|| crate::declaration_type::DeclaredTypeFact::unknown(crate::types::evidence::UnknownReason::NoTypeEvidence));
                            (label.clone(), label, field_id.clone(), declared_type)
                        })
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                let slots = parameter_fields
                    .iter()
                    .map(|(_, label, _, _)| SelectorSlot::Label(label.clone()))
                    .collect::<Vec<_>>();
                let Ok(factory_selector) = Selector::method("new", slots) else {
                    continue;
                };
                let factory = CallableId::new(owner.clone(), factory_selector, DispatchSide::Class);
                // A source constructor with the generated selector is already
                // the canonical source path; compiler-side collision handling
                // remains authoritative for invalid duplicate declarations.
                if source_constructors.iter().any(|method| {
                    crate::checker::declaration_signature::callable_id_for_member(&owner, &ClassMember::Method((*method).clone()))
                        .is_some_and(|source_factory| source_factory == factory)
                }) {
                    continue;
                }
                let initializer_selector = Selector::method("init new", factory.selector.slots.to_vec())
                    .expect("a valid generated factory selector remains valid with an initializer name");
                let initializer = CallableId::new(owner.clone(), initializer_selector, DispatchSide::Instance);
                let (factory_signature, initializer_signature, parameter_bindings) =
                    generated_constructor_signatures(&owner, factory.clone(), initializer.clone(), parameter_fields, Arc::make_mut(&mut self.store));
                callable_signatures.insert(factory_signature);
                callable_signatures.insert(initializer_signature);
                constructor_initializations.insert(ConstructorInitializationProduct {
                    factory,
                    initializer,
                    origin,
                    field_input_fingerprint,
                    default_fields: default_fields.clone(),
                    parameter_fields: parameter_bindings.into_boxed_slice(),
                });
            }
        }

        // Owner construction paths are query products too. Publish them before
        // body scheduling so changed parent paths seed their exact semantic
        // dependents, including when the only source edit was a field default.
        let mut constructor_query_owners = constructor_product_work_owners.clone();
        if let Some(previous) = previous_snapshot.as_ref() {
            constructor_query_owners.extend(
                previous
                    .constructor_initializations
                    .iter()
                    .filter(|(factory, _)| constructor_product_work_modules.contains(factory.module()))
                    .filter_map(|(factory, _)| factory.try_declaration_owner().cloned()),
            );
        }
        let mut changed_constructor_owner_queries = BTreeSet::new();
        for owner in constructor_query_owners {
            let key = QueryKey::ConstructorInitialization(owner.clone());
            let previous_fingerprint = self.db.ready_product_fingerprint(&key);
            let paths = constructor_initializations
                .iter()
                .filter(|(factory, _)| factory.owner == owner)
                .map(|(_, product)| product.clone())
                .collect::<Vec<_>>()
                .into_boxed_slice();
            match query_constructor_initialization_owner(
                &mut self.db,
                Arc::new(crate::db::product::ConstructorInitializationOwnerProduct { owner, paths }),
            ) {
                QueryOutcome::Ready(_) => {}
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            }
            let current_fingerprint = self.db.ready_product_fingerprint(&key);
            if previous_fingerprint.is_some() && previous_fingerprint != current_fingerprint {
                changed_constructor_owner_queries.insert(key);
            }
        }
        if !changed_constructor_owner_queries.is_empty() {
            let changed_dependents = self.db.index().reverse_closure(changed_constructor_owner_queries);
            for key in changed_dependents {
                if let QueryKey::CallableBody(callable) = key {
                    if current_modules.contains(callable.module()) {
                        callable_body_work.insert(callable.clone());
                        semantic_work_modules.insert(callable.module().clone());
                    }
                }
            }
        }
        // 6b. Compile and publish data declarations, enum declarations, associated surfaces, and closed-enum requirements.
        let mut data_semantics = previous_snapshot
            .as_ref()
            .map_or_else(|| self.base_data_semantics.clone(), |snapshot| (*snapshot.data_semantics).clone());
        let mut enum_semantics = previous_snapshot
            .as_ref()
            .map_or_else(|| self.base_enum_semantics.clone(), |snapshot| (*snapshot.enum_semantics).clone());
        let mut enum_requirements_table = previous_snapshot
            .as_ref()
            .map_or_else(|| self.base_enum_requirements.clone(), |snapshot| (*snapshot.enum_requirements).clone());
        let mut associated_static_domains_table = previous_snapshot.as_ref().map_or_else(crate::ordered_map::OrderedMap::default, |snapshot| {
            (*snapshot.associated_static_domains).clone()
        });
        for module in structural_work_modules.iter().chain(removed_modules.iter()) {
            data_semantics.remove_module(module);
            enum_semantics.remove_module(module);
            enum_requirements_table.remove_module(module);
            associated_static_domains_table.retain(|id, _| &id.owner.module != module);
        }

        for base_prod in &self.base_data_products {
            let _ = query_data_declaration(&mut self.db, base_prod.clone());
        }
        for base_prod in &self.base_enum_products {
            let _ = query_enum_declaration(&mut self.db, base_prod.clone());
        }
        for base_assoc in &self.base_associated_surface_products {
            let _ = query_associated_surface(&mut self.db, base_assoc.clone());
        }
        for (decl_id, base_req) in &self.base_enum_requirement_products {
            let _ = query_enum_requirements(&mut self.db, decl_id.clone(), base_req.clone());
        }

        for module_id in &structural_work_modules {
            let Some(shard) = self.semantic_structure_shards.get(module_id) else {
                continue;
            };
            let parsed_unit = &shard.source;
            for stmt in &parsed_unit.program.statements {
                if let Statement::Data(data_def) = stmt {
                    if data_def.is_intrinsic_facade() {
                        let declaration = DeclarationId::new(module_id.clone(), data_def.name.clone().into());
                        if !declarations.is_intrinsic_static_domain(&declaration) && declarations.intrinsic_runtime_owner(&declaration).is_none() {
                            diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                module_id.clone(),
                                DiagnosticCode::KindExpectedType,
                                format!("intrinsic data facade `{}` has no compiler authority binding", data_def.name),
                                data_def.range,
                            ));
                        }
                        continue;
                    }
                    let decl_id = DeclarationId::new(module_id.clone(), data_def.name.clone().into());
                    let static_catalog = (data_def.phase == phalcom_ast::ast::AlgebraicPhase::TypeOnly)
                        .then(|| static_case_catalogs.get(&crate::identity::StaticDomainDeclarationId::Nominal(decl_id.clone())))
                        .flatten()
                        .map(Arc::as_ref);
                    if data_def.is_sum() {
                        let Some(enum_product) = crate::checker::enum_declaration::build_enum_semantics_with_static_products(
                            &decl_id,
                            data_def,
                            Arc::make_mut(&mut self.store),
                            &declarations,
                            &resolver,
                            module_id,
                            static_catalog,
                            &static_case_signatures_table,
                        ) else {
                            diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                module_id.clone(),
                                DiagnosticCode::AnnotationUnresolved,
                                format!("enum `{}` has no published declaration type", data_def.name),
                                data_def.range,
                            ));
                            continue;
                        };

                        diags_by_module
                            .entry(module_id.clone())
                            .or_default()
                            .extend(enum_product.diagnostics.iter().cloned());

                        enum_semantics.insert_enum(enum_product.info.clone());
                        for v in enum_product.variants.iter() {
                            enum_semantics.insert_variant(v.clone());
                        }

                        let arc_enum_product = Arc::new(enum_product);
                        let _ = query_enum_declaration(&mut self.db, arc_enum_product);
                    } else {
                        let Some(data_product) = crate::checker::data_declaration::build_data_semantics_with_static_products(
                            &decl_id,
                            data_def,
                            Arc::make_mut(&mut self.store),
                            &declarations,
                            &resolver,
                            module_id,
                            static_catalog,
                            &static_case_signatures_table,
                        ) else {
                            diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                module_id.clone(),
                                DiagnosticCode::AnnotationUnresolved,
                                format!("data `{}` has no published declaration type", data_def.name),
                                data_def.range,
                            ));
                            continue;
                        };

                        diags_by_module
                            .entry(module_id.clone())
                            .or_default()
                            .extend(data_product.diagnostics.iter().cloned());

                        data_semantics.insert_data(data_product.info.clone());

                        let arc_data_product = Arc::new(data_product);
                        let _ = query_data_declaration(&mut self.db, arc_data_product);

                        let surface = dispatch.surface(&decl_id).cloned().unwrap_or_default();
                        dispatch.register_surface(decl_id.clone(), surface);
                        if let Some(ty) = declarations.form(&decl_id) {
                            dispatch.register_type(ty, decl_id.clone());
                        }
                    }
                }
            }
        }

        let mut inherent_contributions_by_module: BTreeMap<ModuleId, Vec<crate::impls::InherentImplContribution>> = BTreeMap::new();
        let mut inherent_callable_record_key_projections: HashMap<(crate::identity::ImplId, usize), crate::static_key::ProjectedRecordKeySet<'_>> =
            HashMap::new();
        let mut conformance_index = previous_snapshot
            .as_ref()
            .map(|snapshot| (*snapshot.conformance_index).clone())
            .unwrap_or_default();
        let mut conformance_witness_plans = previous_snapshot
            .as_ref()
            .map(|snapshot| (*snapshot.conformance_witness_plans).clone())
            .unwrap_or_default();
        let previous_conformance_plan_fingerprints = conformance_witness_plans
            .iter()
            .map(|(impl_id, plan)| (impl_id.clone(), plan.fingerprint))
            .collect::<BTreeMap<_, _>>();
        let mut conformance_witness_bodies = BTreeSet::new();
        let mut conformance_witness_visibilities: BTreeMap<crate::identity::CallableId, crate::surface::MemberVisibility> = BTreeMap::new();
        let mut conformance_invalid_witnesses: BTreeMap<crate::identity::ImplId, BTreeSet<crate::traits::TraitRequirementId>> = BTreeMap::new();
        let mut conformance_invalid_members = BTreeSet::new();
        let mut conformance_associated_type_plans: BTreeMap<crate::identity::ImplId, crate::impls::ConformanceAssociatedTypePlan> = BTreeMap::new();
        let mut derived_conformance_rules = previous_snapshot
            .as_ref()
            .map(|snapshot| (*snapshot.derived_conformance_rules).clone())
            .unwrap_or_default();
        for module in structural_work_modules.iter().chain(removed_modules.iter()) {
            conformance_index.remove_source(module);
            conformance_witness_plans.retain(|impl_id, _| &impl_id.module != module);
            conformance_associated_type_plans.retain(|impl_id, _| &impl_id.module != module);
            derived_conformance_rules.remove_source(module);
        }
        for impl_id in conformance_witness_rebuild_impls
            .iter()
            .filter(|impl_id| !structural_work_modules.contains(&impl_id.module))
        {
            conformance_index.remove(impl_id);
            conformance_witness_plans.remove(impl_id);
            conformance_associated_type_plans.remove(impl_id);
            derived_conformance_rules.remove(impl_id);
        }
        let conformance_rebuild_modules = structural_work_modules
            .iter()
            .cloned()
            .chain(conformance_witness_rebuild_impls.iter().map(|impl_id| impl_id.module.clone()))
            .collect::<BTreeSet<_>>();
        for module_id in &conformance_rebuild_modules {
            let Some(shard) = self.semantic_structure_shards.get(module_id) else {
                continue;
            };
            let parsed_unit = &shard.source;
            for (idx, statement) in parsed_unit.program.statements.iter().enumerate() {
                if let Statement::Impl(impl_def) = statement {
                    let impl_id = crate::identity::ImplId::new(module_id.clone(), crate::identity::ImplLocalId(idx as u32));
                    let is_conformance = matches!(impl_def.kind, phalcom_ast::ast::ImplKind::Conformance { .. });
                    if std::env::var_os("PHALCOM_INFERENCE_TRACE").is_some() && module_id.path.to_string().contains("collections.list") {
                        eprintln!("conformance candidate {:?} module={} is_conformance={is_conformance}", impl_id, module_id);
                    }
                    if is_conformance && !conformance_witness_rebuild_impls.contains(&impl_id) {
                        continue;
                    }
                    if !is_conformance && !structural_work_modules.contains(module_id) {
                        continue;
                    }
                    if is_conformance {
                        let mut context =
                            crate::checker::CheckingContext::new(Arc::make_mut(&mut self.store), &hierarchy, &resolver, &declarations, module_id.clone());
                        context.attach_data_semantics(&data_semantics);
                        context.attach_enum_semantics(&enum_semantics);
                        match crate::impls::resolve_conformance_head(&mut context, &trait_headers, &impl_id, impl_def) {
                            Ok(crate::impls::ResolvedConformanceDeclaration::Nominal(head)) => {
                                if std::env::var_os("PHALCOM_INFERENCE_TRACE").is_some() && module_id.path.to_string().contains("collections.list") {
                                    eprintln!(
                                        "conformance resolved {:?} target={:?} trait={:?} eligible={}",
                                        impl_id, head.target, head.trait_ref, head.eligible
                                    );
                                }
                                let authorized = crate::impls::conformance_is_authorized(&head.source_module, &head.trait_ref, &head.target);
                                let target_owner = head.target.declaration().clone();
                                let impl_signature = head.generic_signature.clone();
                                let contribution = crate::impls::ConformanceContribution::from_resolved(head, authorized);
                                let has_class_side_requirements = trait_surfaces
                                    .get(&contribution.trait_ref.declaration)
                                    .is_some_and(|surface| surface.iter().any(|(requirement, _)| requirement.side == DispatchSide::Class));
                                if has_class_side_requirements && enum_semantics.enum_info(contribution.target.declaration()).is_some() {
                                    diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                        module_id.clone(),
                                        DiagnosticCode::EnumCaseStaticBehaviorUnsupported,
                                        "class-side trait requirements are not supported on enum targets",
                                        impl_def.range,
                                    ));
                                    continue;
                                }
                                let mut source_associated_type_plan = if let Some(surface) = trait_surfaces.get(&contribution.trait_ref.declaration) {
                                    let associated_plan = crate::impls::build_conformance_associated_type_plan(
                                        &mut context,
                                        &contribution.impl_id,
                                        impl_def,
                                        surface,
                                        impl_signature.as_ref(),
                                        &contribution.trait_ref,
                                        contribution.target_head,
                                    );
                                    diags_by_module
                                        .entry(module_id.clone())
                                        .or_default()
                                        .extend(associated_plan.diagnostics.iter().cloned());
                                    Some(associated_plan)
                                } else {
                                    None
                                };
                                let source_formation_site = if let Some(surface) = trait_surfaces.get(&contribution.trait_ref.declaration) {
                                    let associated_type_context = surface
                                        .associated_types
                                        .values()
                                        .map(|requirement| (requirement.name.to_string(), requirement.requirement.clone()))
                                        .collect();
                                    Some(
                                        TypeFormationSite::trait_member(
                                            module_id.clone(),
                                            contribution.trait_ref.declaration.clone(),
                                            DispatchSide::Instance,
                                            contribution.trait_ref.clone(),
                                            associated_type_context,
                                        )
                                        .with_self_type_override(contribution.target_head),
                                    )
                                } else {
                                    None
                                };
                                if !contribution.authorized {
                                    diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                        module_id.clone(),
                                        DiagnosticCode::ImplForeignTarget,
                                        "conformance must be declared in the trait or target owner module",
                                        impl_def.range,
                                    ));
                                }
                                diags_by_module
                                    .entry(module_id.clone())
                                    .or_default()
                                    .extend(contribution.diagnostics.iter().cloned());
                                if contribution.is_lookup_eligible() {
                                    let callable_owner = crate::identity::CallableOwnerId::Conformance(contribution.impl_id.clone());
                                    let mut seen_requirements = BTreeSet::new();
                                    for (source_member_index, impl_member) in impl_def.members.iter().enumerate() {
                                        if let phalcom_ast::ast::ImplMember::Delegation(delegation) = impl_member {
                                            conformance_invalid_members.insert(contribution.impl_id.clone());
                                            diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                                module_id.clone(),
                                                DiagnosticCode::DelegationInConformanceUnsupported,
                                                "direct-field `via` is supported only in class and inherent impl declarations",
                                                delegation.range,
                                            ));
                                            continue;
                                        }
                                        let Some(member) = impl_member.behavior() else {
                                            continue;
                                        };
                                        let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(member);
                                        let side = crate::semantic_shard::behavior_side(member);
                                        let Some(candidate_callable) =
                                            crate::checker::declaration_signature::callable_id_for_syntax(&callable_owner, syntax, side)
                                        else {
                                            continue;
                                        };
                                        let requirement = trait_surfaces
                                            .get(&contribution.trait_ref.declaration)
                                            .and_then(|surface| surface.get_by_selector(&candidate_callable.selector, candidate_callable.side));
                                        let Some(requirement) = requirement else {
                                            conformance_invalid_members.insert(contribution.impl_id.clone());
                                            diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                                module_id.clone(),
                                                DiagnosticCode::ImplMemberConflict,
                                                "conformance member does not match a trait requirement",
                                                syntax.range(),
                                            ));
                                            continue;
                                        };
                                        if !seen_requirements.insert(requirement.requirement.clone()) {
                                            conformance_invalid_witnesses
                                                .entry(contribution.impl_id.clone())
                                                .or_default()
                                                .insert(requirement.requirement.clone());
                                            diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                                module_id.clone(),
                                                DiagnosticCode::ImplMemberConflict,
                                                "multiple conformance members target the same trait requirement",
                                                syntax.range(),
                                            ));
                                            continue;
                                        }
                                        let Some(mut signature) = (if let Some(formation_site) = source_formation_site.as_ref() {
                                            crate::checker::declaration_signature::semantic_signature_for_conformance_syntax_with_site(
                                                &mut context,
                                                &callable_owner,
                                                &target_owner,
                                                impl_signature.as_ref(),
                                                &formation_site.clone().with_self_dispatch_side(side),
                                                syntax,
                                                side,
                                            )
                                        } else {
                                            crate::checker::declaration_signature::semantic_signature_for_conformance_syntax(
                                                &mut context,
                                                &callable_owner,
                                                &target_owner,
                                                impl_signature.as_ref(),
                                                syntax,
                                                side,
                                            )
                                        }) else {
                                            continue;
                                        };
                                        if let Some(associated_plan) = source_associated_type_plan.as_mut() {
                                            signature = crate::impls::normalize_source_signature(
                                                &mut context,
                                                &contribution.trait_ref,
                                                contribution.target_head,
                                                associated_plan,
                                                signature,
                                            );
                                        }
                                        let callable = signature.callable.clone();
                                        conformance_witness_visibilities.insert(callable.clone(), crate::traits::behavior_member_visibility(member));
                                        let has_body = match member {
                                            phalcom_ast::ast::BehaviorMember::Method(method) => method.body.statements().is_some(),
                                            phalcom_ast::ast::BehaviorMember::Getter(getter) => getter.body.statements().is_some(),
                                            phalcom_ast::ast::BehaviorMember::Setter(setter) => setter.body.statements().is_some(),
                                            phalcom_ast::ast::BehaviorMember::Index(index) => index.body.statements().is_some(),
                                        };
                                        if !has_body {
                                            conformance_invalid_witnesses
                                                .entry(contribution.impl_id.clone())
                                                .or_default()
                                                .insert(requirement.requirement.clone());
                                            diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                                module_id.clone(),
                                                DiagnosticCode::ImplBodylessMemberUnsupported,
                                                "explicit conformance witnesses require a body",
                                                syntax.range(),
                                            ));
                                            continue;
                                        }
                                        if has_body {
                                            conformance_witness_bodies.insert(callable.clone());
                                        }
                                        let definition = crate::impls::EffectiveCallableDefinition {
                                            callable: callable.clone(),
                                            origin: crate::impls::CallableDefinitionOrigin::ConformanceWitness(contribution.impl_id.clone()),
                                            source_member_index,
                                            signature: signature.clone(),
                                            body_provenance: crate::impls::CallableDefinitionBodyProvenance::SourceBody,
                                            inherent_body_context: None,
                                        };
                                        callable_definitions.insert(callable.clone(), definition.clone());
                                        let _ = query_callable_definition(&mut self.db, Arc::new(definition));
                                        callable_signatures.insert(signature.clone());
                                        let _ = query_bootstrap_callable_signature(&mut self.db, Arc::new(signature));
                                    }
                                }
                                if let Some(associated_plan) = source_associated_type_plan.take() {
                                    conformance_associated_type_plans.insert(contribution.impl_id.clone(), associated_plan);
                                }
                                conformance_index.insert(contribution);
                            }
                            Ok(crate::impls::ResolvedConformanceDeclaration::GuardedRule(rule_head)) => {
                                if std::env::var_os("PHALCOM_INFERENCE_TRACE").is_some() && module_id.path.to_string().contains("collections.list") {
                                    eprintln!(
                                        "conformance guarded {:?} target={:?} trait={:?}",
                                        impl_id, rule_head.target_template, rule_head.trait_ref
                                    );
                                }
                                let Some(surface) = trait_surfaces.get(&rule_head.trait_ref.declaration) else {
                                    continue;
                                };
                                let target_declaration = context.store.nominal_origin_declaration(rule_head.target_template).cloned();
                                if surface.iter().any(|(requirement, _)| requirement.side == DispatchSide::Class)
                                    && target_declaration
                                        .as_ref()
                                        .is_some_and(|declaration| enum_semantics.enum_info(declaration).is_some())
                                {
                                    diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                        module_id.clone(),
                                        DiagnosticCode::EnumCaseStaticBehaviorUnsupported,
                                        "class-side trait requirements are not supported on enum targets",
                                        impl_def.range,
                                    ));
                                    continue;
                                }
                                let mut source_associated_type_plan = crate::impls::build_conformance_associated_type_plan(
                                    &mut context,
                                    &rule_head.impl_id,
                                    impl_def,
                                    surface,
                                    Some(&rule_head.generic_signature),
                                    &rule_head.trait_ref,
                                    rule_head.target_template,
                                );
                                diags_by_module
                                    .entry(module_id.clone())
                                    .or_default()
                                    .extend(source_associated_type_plan.diagnostics.iter().cloned());

                                let mut rule_requirements = BTreeMap::new();
                                let requirement_views = crate::impls::instantiate_conformance_requirements(
                                    context.store,
                                    Some(&declarations),
                                    surface,
                                    &rule_head.trait_ref,
                                    rule_head.target_template,
                                    &HashMap::new(),
                                );
                                let callable_owner = crate::identity::CallableOwnerId::Conformance(rule_head.impl_id.clone());
                                let mut seen_requirements = BTreeSet::new();
                                for (source_member_index, impl_member) in impl_def.members.iter().enumerate() {
                                    if let phalcom_ast::ast::ImplMember::Delegation(delegation) = impl_member {
                                        diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                            module_id.clone(),
                                            DiagnosticCode::DelegationInConformanceUnsupported,
                                            "direct-field `via` is supported only in class and inherent impl declarations",
                                            delegation.range,
                                        ));
                                        continue;
                                    }
                                    let Some(member) = impl_member.behavior() else {
                                        continue;
                                    };
                                    let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(member);
                                    let side = crate::semantic_shard::behavior_side(member);
                                    let Some(candidate_callable) = crate::checker::declaration_signature::callable_id_for_syntax(&callable_owner, syntax, side)
                                    else {
                                        continue;
                                    };
                                    let requirement = surface.get_by_selector(&candidate_callable.selector, candidate_callable.side);
                                    let Some(requirement) = requirement else {
                                        diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                            module_id.clone(),
                                            DiagnosticCode::ImplMemberConflict,
                                            "conformance member does not match a trait requirement",
                                            syntax.range(),
                                        ));
                                        continue;
                                    };
                                    if !seen_requirements.insert(requirement.requirement.clone()) {
                                        diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                            module_id.clone(),
                                            DiagnosticCode::ImplMemberConflict,
                                            "multiple conformance members target the same trait requirement",
                                            syntax.range(),
                                        ));
                                        continue;
                                    }
                                    let associated_type_context =
                                        surface.associated_types.values().map(|r| (r.name.to_string(), r.requirement.clone())).collect();
                                    let formation_site = TypeFormationSite::trait_member(
                                        module_id.clone(),
                                        rule_head.trait_ref.declaration.clone(),
                                        side,
                                        rule_head.trait_ref.clone(),
                                        associated_type_context,
                                    )
                                    .with_self_type_override(rule_head.target_template);

                                    let Some(mut signature) = crate::checker::declaration_signature::semantic_signature_for_conformance_syntax_with_site(
                                        &mut context,
                                        &callable_owner,
                                        &rule_head.trait_ref.declaration,
                                        Some(&rule_head.generic_signature),
                                        &formation_site,
                                        syntax,
                                        side,
                                    ) else {
                                        continue;
                                    };
                                    signature = crate::impls::normalize_source_signature(
                                        &mut context,
                                        &rule_head.trait_ref,
                                        rule_head.target_template,
                                        &mut source_associated_type_plan,
                                        signature,
                                    );
                                    let callable = signature.callable.clone();
                                    conformance_witness_visibilities.insert(callable.clone(), crate::traits::behavior_member_visibility(member));
                                    let has_body = match member {
                                        phalcom_ast::ast::BehaviorMember::Method(method) => method.body.statements().is_some(),
                                        phalcom_ast::ast::BehaviorMember::Getter(getter) => getter.body.statements().is_some(),
                                        phalcom_ast::ast::BehaviorMember::Setter(setter) => setter.body.statements().is_some(),
                                        phalcom_ast::ast::BehaviorMember::Index(index) => index.body.statements().is_some(),
                                    };
                                    if has_body {
                                        conformance_witness_bodies.insert(callable.clone());
                                    }
                                    let definition = crate::impls::EffectiveCallableDefinition {
                                        callable: callable.clone(),
                                        origin: crate::impls::CallableDefinitionOrigin::ConformanceWitness(rule_head.impl_id.clone()),
                                        source_member_index,
                                        signature: signature.clone(),
                                        body_provenance: crate::impls::CallableDefinitionBodyProvenance::SourceBody,
                                        inherent_body_context: None,
                                    };
                                    callable_definitions.insert(callable.clone(), definition.clone());
                                    let _ = query_callable_definition(&mut self.db, Arc::new(definition));
                                    callable_signatures.insert(signature.clone());
                                    let _ = query_bootstrap_callable_signature(&mut self.db, Arc::new(signature));

                                    rule_requirements.insert(
                                        requirement.requirement.clone(),
                                        crate::impls::RequirementSelectionTemplate::ConformanceCallable { callable: candidate_callable },
                                    );
                                }
                                for (requirement, member) in surface.iter() {
                                    if !rule_requirements.contains_key(requirement) && member.default_present {
                                        rule_requirements.insert(
                                            requirement.clone(),
                                            crate::impls::RequirementSelectionTemplate::TraitDefault {
                                                callable: requirement.source_callable(),
                                            },
                                        );
                                    }
                                }

                                let existing_rules = derived_conformance_rules.rules_for_trait(&rule_head.trait_ref.declaration);
                                if !existing_rules.is_empty() {
                                    diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                        module_id.clone(),
                                        DiagnosticCode::DerivedConformanceOverlap,
                                        format!(
                                            "overlapping derived conformance rules: multiple derived conformance rules found for trait `{}`",
                                            rule_head.trait_ref.declaration.name
                                        ),
                                        impl_def.range,
                                    ));
                                }

                                let rule = crate::derived_conformance::DerivedConformanceRule::new(
                                    rule_head.impl_id.clone(),
                                    rule_head.source_module.clone(),
                                    rule_head.target_parameter,
                                    rule_head.target_template,
                                    rule_head.trait_ref.clone(),
                                    rule_head.generic_signature.clone(),
                                    source_associated_type_plan,
                                    requirement_views,
                                    rule_requirements,
                                    rule_head.source.clone(),
                                );
                                derived_conformance_rules.insert(rule);
                            }
                            Err(diagnostics) => {
                                if std::env::var_os("PHALCOM_INFERENCE_TRACE").is_some() && module_id.path.to_string().contains("collections.list") {
                                    eprintln!("conformance rejected {:?}: {:?}", impl_id, diagnostics);
                                }
                                diags_by_module.entry(module_id.clone()).or_default().extend(diagnostics);
                            }
                        }
                        continue;
                    }
                    if !matches!(impl_def.kind, phalcom_ast::ast::ImplKind::Inherent) {
                        continue;
                    }
                    let mut context =
                        crate::checker::CheckingContext::new(Arc::make_mut(&mut self.store), &hierarchy, &resolver, &declarations, module_id.clone());
                    context.attach_field_signatures(&field_signatures);
                    context.attach_data_semantics(&data_semantics);
                    context.attach_enum_semantics(&enum_semantics);
                    let resolved_target = crate::impls::resolve_inherent_impl_target(&mut context, &impl_id, impl_def);
                    if let Ok(target) = &resolved_target {
                        let target_owner = target.target.to_callable_owner();
                        for (member_index, impl_member) in impl_def.members.iter().enumerate() {
                            let Some(member) = impl_member.behavior() else {
                                continue;
                            };
                            let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(member);
                            let annotations = crate::checker::declaration_signature::callable_syntax_type_annotations(syntax);
                            if !annotations
                                .iter()
                                .any(|annotation| crate::checker::declaration_signature::annotation_has_computed_record_key(annotation))
                            {
                                continue;
                            }
                            let is_class_side = syntax.attributes().iter().any(|attribute| attribute.name == "class")
                                || match member {
                                    phalcom_ast::ast::BehaviorMember::Method(method) => method.is_static,
                                    phalcom_ast::ast::BehaviorMember::Getter(getter) => getter.is_static,
                                    phalcom_ast::ast::BehaviorMember::Setter(setter) => setter.is_static,
                                    phalcom_ast::ast::BehaviorMember::Index(_) => false,
                                };
                            let side = if is_class_side {
                                crate::identity::DispatchSide::Class
                            } else {
                                crate::identity::DispatchSide::Instance
                            };
                            let Some(callable_id) = crate::checker::declaration_signature::callable_id_for_syntax(&target_owner, syntax, side) else {
                                continue;
                            };

                            let projections = loop {
                                let projection = {
                                    let mut driver = crate::static_constant_demand::StaticConstantDemandDriver {
                                        db: &mut self.db,
                                        store: context.store,
                                        values: Arc::make_mut(&mut self.static_values),
                                        constants: &mut static_constants_table,
                                        definitions: &static_definitions_table,
                                        catalogs: &module_static_binding_catalogs,
                                        sources: &retained_sources,
                                        linked: input.linked.as_ref(),
                                        topology: semantic_topology.as_ref(),
                                        import_products: &input.import_products,
                                        source_resolution_input,
                                        linked_component_product,
                                        hierarchy: &hierarchy,
                                        resolver: &resolver,
                                        declarations: &declarations,
                                        type_aliases: &type_aliases,
                                        dispatch: &dispatch,
                                        field_signatures: &field_signatures,
                                        callable_signatures: &callable_signatures,
                                        data_semantics: &early_data_semantics,
                                        enum_semantics: &early_enum_semantics,
                                        associated_families: &associated_surfaces_table,
                                        deferred_static_definitions: &no_deferred_static_definitions,
                                        intrinsics: &static_intrinsics,
                                        budget,
                                        cancel,
                                        active_constants: Vec::new(),
                                    };
                                    driver.project_annotations(QueryKey::CallableSignature(callable_id.clone()), module_id, annotations.iter().copied())
                                };
                                match projection {
                                    crate::static_constant_demand::StaticDemandOutcome::Ready(projections) => break Some(projections),
                                    crate::static_constant_demand::StaticDemandOutcome::Awaiting(demands) => {
                                        match service_type_member_demands(
                                            &mut self.db,
                                            context.store,
                                            &resolver,
                                            &demands,
                                            &signature_graph,
                                            &alias_sources,
                                            &type_aliases,
                                            &declarations,
                                            &associated_surfaces_table,
                                            &mut type_member_declarations_table,
                                            &static_case_signatures_table,
                                            &type_signature_cycle_members,
                                            &member_definitions,
                                            &retained_sources,
                                            &mut diags_by_module,
                                        )? {
                                            TypeMemberDemandService::Current => continue,
                                            TypeMemberDemandService::Cycle => break None,
                                            TypeMemberDemandService::NoProgress => {
                                                return Err(QueryOutcome::Failed(format!(
                                                    "type-member demand for inherent callable signature {callable_id:?} could not progress: {demands:?}"
                                                )));
                                            }
                                        }
                                    }
                                    crate::static_constant_demand::StaticDemandOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                                    crate::static_constant_demand::StaticDemandOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                                    crate::static_constant_demand::StaticDemandOutcome::BudgetExceeded(report) => {
                                        return Err(QueryOutcome::BudgetExceeded(report));
                                    }
                                    crate::static_constant_demand::StaticDemandOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                                }
                            };
                            if let Some(projections) = projections {
                                inherent_callable_record_key_projections.insert((impl_id.clone(), member_index), projections);
                            }
                        }
                    }
                    let projections_by_member = impl_def
                        .members
                        .iter()
                        .enumerate()
                        .filter_map(|(member_index, _)| {
                            inherent_callable_record_key_projections
                                .get(&(impl_id.clone(), member_index))
                                .map(|projections| (member_index, projections))
                        })
                        .collect::<BTreeMap<_, _>>();
                    let contribution = crate::impls::build_inherent_impl_contribution_with_prepared_target_and_record_keys(
                        &mut context,
                        &impl_id,
                        impl_def,
                        resolved_target,
                        &projections_by_member,
                    );
                    if !contribution.diagnostics.is_empty() {
                        diags_by_module
                            .entry(module_id.clone())
                            .or_default()
                            .extend(contribution.diagnostics.iter().cloned());
                    }
                    inherent_contributions_by_module.entry(module_id.clone()).or_default().push(contribution);
                }
            }
        }

        for (first, second) in conformance_index.overlap_conflicts(Arc::make_mut(&mut self.store)) {
            let Some(first_contribution) = conformance_index.get(&first) else { continue };
            let Some(second_contribution) = conformance_index.get(&second) else {
                continue;
            };
            diags_by_module.entry(first.module.clone()).or_default().push(SemanticDiagnostic::error_in(
                first.module.clone(),
                DiagnosticCode::ImplConformanceOverlap,
                format!("conformance heads overlap with `{}`; no specialization precedence is defined", second.module),
                first_contribution.source.range,
            ));
            diags_by_module.entry(second.module.clone()).or_default().push(SemanticDiagnostic::error_in(
                second.module.clone(),
                DiagnosticCode::ImplConformanceOverlap,
                format!("conformance heads overlap with `{}`; no specialization precedence is defined", first.module),
                second_contribution.source.range,
            ));
        }

        for module_id in &structural_work_modules {
            let Some(shard) = self.semantic_structure_shards.get(module_id) else {
                continue;
            };
            let parsed_unit = &shard.source;
            for stmt in &parsed_unit.program.statements {
                if let Statement::Data(enum_def) = stmt {
                    if !enum_def.is_sum() {
                        continue;
                    }
                    let decl_id = DeclarationId::new(module_id.clone(), enum_def.name.clone().into());
                    let empty_contribs = Vec::new();
                    let module_contribs = inherent_contributions_by_module.get(module_id).unwrap_or(&empty_contribs);
                    let enum_contribs: Vec<crate::impls::InherentImplContribution> =
                        module_contribs.iter().filter(|c| c.target.declaration() == &decl_id).cloned().collect();

                    let mut behavior_ctx = crate::checker::CheckingContext::new_with_dispatch_ref(
                        Arc::make_mut(&mut self.store),
                        &hierarchy,
                        &resolver,
                        &declarations,
                        &dispatch,
                        module_id.clone(),
                    );
                    behavior_ctx.attach_enum_semantics(&enum_semantics);
                    let behavior_product = crate::checker::enum_behavior::build_enum_behavior(&mut behavior_ctx, &decl_id, Some(enum_def), &enum_contribs);
                    diags_by_module
                        .entry(module_id.clone())
                        .or_default()
                        .extend(behavior_product.diagnostics.iter().cloned());

                    // Publish root defaults to callable_signatures and dispatch surface
                    let mut surface = dispatch.surface(&decl_id).cloned().unwrap_or_default();
                    for default_sig in behavior_product.root_defaults.iter() {
                        callable_signatures.insert(default_sig.clone());
                        let projection = crate::checker::declaration_signature::project_semantic_signature(default_sig);
                        if default_sig
                            .source
                            .as_ref()
                            .is_some_and(|s| s.range.start >= enum_def.range.start && s.range.end <= enum_def.range.end)
                        {
                            surface.add_callable(default_sig.side, projection);
                        }
                    }
                    dispatch.register_surface(decl_id.clone(), surface);
                    if let Some(ty) = declarations.form(&decl_id) {
                        dispatch.register_type(ty, decl_id.clone());
                    }
                    let Some(linked_module) = input.linked.modules.get(module_id) else {
                        return Err(QueryOutcome::Failed(format!("linked module prerequisite is missing for enum {decl_id:?}")));
                    };
                    match query_declaration_surface(
                        &mut self.db,
                        DeclarationSurfaceQuery {
                            decl_id: decl_id.clone(),
                            unit: parsed_unit.clone(),
                            linked_interface: Arc::new(linked_module.interface.clone()),
                            store: Arc::make_mut(&mut self.store),
                            hierarchy: &hierarchy,
                            resolver: &resolver,
                            declarations: &declarations,
                            field_signatures: None,
                            callable_signatures: Some(&callable_signatures),
                            type_aliases: Some(&type_aliases),
                            linked: Some(input.linked.as_ref()),
                            import_products: Some(&input.import_products),
                        },
                    ) {
                        QueryOutcome::Ready(_) => {}
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    }

                    // Publish case implementations to callable_signatures
                    for case_sigs in behavior_product.case_implementations.values() {
                        for case_sig in case_sigs.iter() {
                            callable_signatures.insert(case_sig.clone());
                        }
                    }

                    for contrib in &enum_contribs {
                        if matches!(contrib.target, crate::impls::InherentImplTarget::ExactEnumCase(_)) {
                            for member in contrib.members.iter() {
                                let definition = crate::impls::EffectiveCallableDefinition {
                                    callable: member.callable.clone(),
                                    origin: crate::impls::CallableDefinitionOrigin::InherentImpl(contrib.id.clone()),
                                    source_member_index: member.source_member,
                                    signature: member.signature.clone(),
                                    body_provenance: crate::impls::CallableDefinitionBodyProvenance::SourceBody,
                                    inherent_body_context: Some(crate::impls::InherentBodyContext {
                                        generic_signature: contrib.generic_signature.clone(),
                                        covering: contrib.covering.clone(),
                                        source_target_head: contrib.source_target_head,
                                    }),
                                };
                                callable_definitions.insert(definition.callable.clone(), definition.clone());
                                let _ = query_callable_definition(&mut self.db, Arc::new(definition.clone()));
                            }
                        }
                    }

                    let mut exact_case_sets = BTreeMap::<crate::impls::InherentImplTarget, crate::impls::ConditionalInherentMemberSet>::new();
                    for contrib in &enum_contribs {
                        let crate::impls::InherentImplTarget::ExactEnumCase(variant) = &contrib.target else {
                            continue;
                        };
                        let set = exact_case_sets
                            .entry(crate::impls::InherentImplTarget::ExactEnumCase(variant.clone()))
                            .or_insert_with(|| crate::impls::ConditionalInherentMemberSet::new(Some(contrib.target.clone())));
                        for member in contrib.members.iter() {
                            let Some(domain) = contrib.domain.clone() else {
                                continue;
                            };
                            set.add_member(crate::impls::ConditionalInherentMember {
                                impl_id: contrib.id.clone(),
                                domain,
                                is_conditionally_applicable: contrib.is_conditionally_applicable,
                                callable: member.callable.clone(),
                                signature_template: member.signature.clone(),
                                visibility: member.visibility,
                                source_member: member.source_member,
                                is_requirement: member.is_requirement,
                            });
                        }
                    }
                    for (target, set) in exact_case_sets {
                        dispatch.register_conditional_members(target, Arc::new(set));
                    }

                    let variants_info: Vec<VariantInfo> = enum_def
                        .cases()
                        .unwrap_or_default()
                        .iter()
                        .filter_map(|v| {
                            let sel = phalcom_ast::selector::selector_from_variant(v);
                            let vid = crate::identity::VariantId::new(decl_id.clone(), sel);
                            enum_semantics.variant_info(&vid).cloned()
                        })
                        .collect();

                    let mut case_methods_map: HashMap<crate::identity::VariantId, Vec<crate::signature::CallableSemanticSignature>> = HashMap::new();
                    for (v_id, sigs) in &behavior_product.case_implementations {
                        case_methods_map.insert(v_id.clone(), sigs.to_vec());
                    }

                    let (case_statuses, req_diags) = check_enum_requirements(
                        &decl_id,
                        enum_semantics.enum_info(&decl_id).unwrap(),
                        &variants_info,
                        &behavior_product.root_requirements,
                        &case_methods_map,
                        Arc::make_mut(&mut self.store),
                        &hierarchy,
                        module_id,
                    );
                    diags_by_module.entry(module_id.clone()).or_default().extend(req_diags.iter().cloned());
                    enum_requirements_table.insert(decl_id.clone(), Arc::from(behavior_product.root_requirements.clone()), case_statuses.clone());
                    let req_product = Arc::new(EnumRequirementsProduct {
                        requirements: Arc::from(behavior_product.root_requirements),
                        case_statuses,
                        diagnostics: req_diags,
                    });
                    let _ = query_enum_requirements(&mut self.db, decl_id.clone(), req_product);
                }
            }
        }

        for module_id in &structural_work_modules {
            let Some(shard) = self.semantic_structure_shards.get(module_id) else {
                continue;
            };
            let parsed_unit = &shard.source;
            let empty_contribs = Vec::new();
            let module_contribs = inherent_contributions_by_module.get(module_id).unwrap_or(&empty_contribs);
            let mut contributions_by_target = BTreeMap::<DeclarationId, Vec<crate::impls::InherentImplContribution>>::new();
            for contribution in module_contribs {
                contributions_by_target
                    .entry(contribution.target.declaration().clone())
                    .or_default()
                    .push(contribution.clone());
            }
            for (target, contributions) in contributions_by_target {
                let Some(primary_surface) = dispatch.get_surface(&target).cloned() else {
                    continue;
                };
                let primary_signatures = callable_signatures
                    .iter()
                    .filter(|(callable, _)| callable.try_declaration_owner() == Some(&target))
                    .map(|(callable, signature)| (callable.clone(), signature.clone()))
                    .collect::<HashMap<_, _>>();
                let effective = crate::impls::build_effective_surface(
                    &target,
                    &primary_surface,
                    &primary_signatures,
                    &contributions,
                    data_semantics.data_info(&target).map(AsRef::as_ref),
                    associated_surfaces_table.surfaces.get(&target).map(AsRef::as_ref),
                );
                diags_by_module
                    .entry(module_id.clone())
                    .or_default()
                    .extend(effective.diagnostics.iter().cloned());
                dispatch.register_surface(target.clone(), (*effective.surface).clone());
                dispatch.register_conditional_members(
                    crate::impls::InherentImplTarget::Declaration(target.clone()),
                    effective.conditional_members.clone(),
                );
                let _ = query_bootstrap_declaration_surface(&mut self.db, target.clone(), effective.surface.clone());

                for signature in effective.native_anchors.values() {
                    match query_bootstrap_callable_signature(&mut self.db, Arc::new(signature.clone())) {
                        QueryOutcome::Ready(signature) => {
                            callable_signatures.insert((*signature).clone());
                        }
                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    }
                }

                for definition in effective.definitions.values() {
                    if matches!(definition.origin, crate::impls::CallableDefinitionOrigin::InherentImpl(_)) {
                        callable_definitions.insert(definition.callable.clone(), definition.clone());
                        let _ = query_callable_definition(&mut self.db, Arc::new(definition.clone()));
                        let callable_id = definition.callable.clone();
                        // Freshly accepted inherent definitions must publish their
                        // signatures even when this module has an older snapshot.
                        // Worklist absence alone is not proof of a retained product.
                        if previous_snapshot.is_some() && !callable_signature_work.contains(&callable_id) && callable_signatures.get(&callable_id).is_some() {
                            continue;
                        }
                        let (impl_id, source_member_index) = match &definition.origin {
                            crate::impls::CallableDefinitionOrigin::InherentImpl(impl_id) => (impl_id, definition.source_member_index),
                            _ => unreachable!("inherent definition branch must retain its impl source"),
                        };
                        let type_params_map = definition
                            .inherent_body_context
                            .as_ref()
                            .and_then(|context| context.generic_signature.as_ref())
                            .map(|signature| {
                                signature
                                    .parameters
                                    .iter()
                                    .map(|parameter_id| {
                                        let name = self.store.type_parameter(*parameter_id).name.to_string();
                                        let binding = type_level_binding_for_parameter(Arc::make_mut(&mut self.store), *parameter_id);
                                        (name, binding)
                                    })
                                    .collect::<HashMap<_, _>>()
                            })
                            .unwrap_or_default();
                        let scoped_resolver = crate::types::annotation::ScopedTypeResolver {
                            parent: &resolver,
                            type_parameters: type_params_map,
                        };
                        let record_key_projections = inherent_callable_record_key_projections.get(&(impl_id.clone(), source_member_index));
                        match query_callable_signature_with_inputs(
                            &mut self.db,
                            callable_id.clone(),
                            parsed_unit.clone(),
                            SignatureQueryInputs {
                                store: Arc::make_mut(&mut self.store),
                                hierarchy: &hierarchy,
                                resolver: &scoped_resolver,
                                record_key_projections,
                                declarations: &declarations,
                                linked: Some(input.linked.as_ref()),
                                type_aliases: Some(&type_aliases),
                                import_products: Some(&input.import_products),
                            },
                        ) {
                            QueryOutcome::Ready(signature) => {
                                let mut signature = (*signature).clone();
                                if let Some(previous) = previous_callable_formal_states.get(&callable_id) {
                                    if previous.parameters == signature.parameters && previous.declared_return == signature.declared_return {
                                        signature.return_validation = previous.return_validation;
                                        signature.inferred_return = previous.inferred_return.clone();
                                    }
                                }
                                callable_signatures.insert(signature);
                            }
                            QueryOutcome::Blocked(reason) => {
                                return Err(QueryOutcome::Blocked(reason));
                            }
                            QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                            QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                            QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                        }
                    }
                }
            }
        }

        for module_id in &structural_work_modules {
            let Some(shard) = self.semantic_structure_shards.get(module_id) else {
                continue;
            };
            let parsed_unit = &shard.source;
            let empty_contribs = Vec::new();
            let module_contribs = inherent_contributions_by_module.get(module_id).unwrap_or(&empty_contribs);
            for statement in &shard.source.program.statements {
                let declaration = match statement {
                    Statement::Class(class_def) => {
                        let declaration = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                        let mut static_domain_candidates = Vec::<crate::db::product::AssociatedStaticDomainDeclarationProduct>::new();
                        for definition in &class_def.associated_type_data {
                            let domain_id = crate::identity::AssociatedStaticDomainDeclarationId::new(declaration.clone(), definition.name.clone());
                            if !associated_surface_admits(
                                &associated_surfaces_table,
                                &declaration,
                                &crate::associated::AssociatedMemberId::StaticDomain(domain_id.clone()),
                            ) {
                                continue;
                            }
                            let catalog = static_case_catalogs
                                .get(&crate::identity::StaticDomainDeclarationId::Associated(domain_id))
                                .map(Arc::as_ref);
                            let (product, diagnostics) = crate::checker::algebraic_declaration::build_associated_static_domain_declaration_with_static_products(
                                &declaration,
                                definition,
                                Arc::make_mut(&mut self.store),
                                &declarations,
                                &resolver,
                                module_id,
                                catalog,
                                &static_case_signatures_table,
                            );
                            diags_by_module.entry(module_id.clone()).or_default().extend(diagnostics);
                            if let Some(product) = product {
                                static_domain_candidates.push(product);
                            }
                        }
                        for (statement_index, statement) in parsed_unit.program.statements.iter().enumerate() {
                            let Statement::Impl(impl_def) = statement else { continue };
                            if impl_def.associated_type_data.is_empty() {
                                continue;
                            }
                            let target_matches_owner = impl_def.target.origin_symbol_ref().is_some_and(|symbol| {
                                let members = symbol.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
                                resolver.resolve_type_name(module_id, &symbol.root, &members).as_ref() == Some(&declaration)
                            });
                            if !target_matches_owner {
                                continue;
                            }
                            let impl_id = crate::identity::ImplId::new(module_id.clone(), crate::identity::ImplLocalId(statement_index as u32));
                            if !matches!(impl_def.kind, phalcom_ast::ast::ImplKind::Inherent) {
                                for definition in &impl_def.associated_type_data {
                                    diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                        module_id.clone(),
                                        DiagnosticCode::KindExpectedType,
                                        "associated type data is not permitted in a conformance impl",
                                        definition.range,
                                    ));
                                }
                                continue;
                            }
                            let Some(contribution) = module_contribs.iter().find(|contribution| contribution.id == impl_id) else {
                                continue;
                            };
                            if contribution.target.declaration() != &declaration {
                                continue;
                            }
                            if contribution.is_conditionally_applicable {
                                for definition in &impl_def.associated_type_data {
                                    diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                        module_id.clone(),
                                        DiagnosticCode::KindExpectedType,
                                        "guarded or partial-target impls cannot introduce associated type data",
                                        definition.range,
                                    ));
                                }
                                continue;
                            }
                            for definition in &impl_def.associated_type_data {
                                let domain_id = crate::identity::AssociatedStaticDomainDeclarationId::new(declaration.clone(), definition.name.clone());
                                if !associated_surface_admits(
                                    &associated_surfaces_table,
                                    &declaration,
                                    &crate::associated::AssociatedMemberId::StaticDomain(domain_id.clone()),
                                ) {
                                    continue;
                                }
                                let catalog = static_case_catalogs
                                    .get(&crate::identity::StaticDomainDeclarationId::Associated(domain_id))
                                    .map(Arc::as_ref);
                                let (product, diagnostics) =
                                    crate::checker::algebraic_declaration::build_associated_static_domain_declaration_with_static_products(
                                        &declaration,
                                        definition,
                                        Arc::make_mut(&mut self.store),
                                        &declarations,
                                        &resolver,
                                        module_id,
                                        catalog,
                                        &static_case_signatures_table,
                                    );
                                diags_by_module.entry(module_id.clone()).or_default().extend(diagnostics);
                                if let Some(product) = product {
                                    static_domain_candidates.push(product);
                                }
                            }
                        }
                        for product in static_domain_candidates {
                            let product = Arc::new(product);
                            match crate::db::query::query_associated_static_domain_declaration(&mut self.db, parsed_unit, product.clone(), &self.store) {
                                QueryOutcome::Ready(product) => {
                                    associated_static_domains_table.insert(product.id.clone(), product);
                                }
                                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                            }
                        }
                        declaration
                    }
                    Statement::Data(data_def) => {
                        let declaration = DeclarationId::new(module_id.clone(), data_def.name.clone().into());
                        let mut candidates = Vec::new();
                        for (statement_index, statement) in parsed_unit.program.statements.iter().enumerate() {
                            let Statement::Impl(impl_def) = statement else { continue };
                            if impl_def.associated_type_data.is_empty() {
                                continue;
                            }
                            let target_matches_owner = impl_def.target.origin_symbol_ref().is_some_and(|symbol| {
                                let members = symbol.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
                                resolver.resolve_type_name(module_id, &symbol.root, &members).as_ref() == Some(&declaration)
                            });
                            if !target_matches_owner {
                                continue;
                            }
                            let impl_id = crate::identity::ImplId::new(module_id.clone(), crate::identity::ImplLocalId(statement_index as u32));
                            if !matches!(impl_def.kind, phalcom_ast::ast::ImplKind::Inherent) {
                                for definition in &impl_def.associated_type_data {
                                    diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                        module_id.clone(),
                                        DiagnosticCode::KindExpectedType,
                                        "associated type data is not permitted in a conformance impl",
                                        definition.range,
                                    ));
                                }
                                continue;
                            }
                            let Some(contribution) = module_contribs.iter().find(|contribution| contribution.id == impl_id) else {
                                continue;
                            };
                            if contribution.target.declaration() != &declaration {
                                continue;
                            }
                            if contribution.is_conditionally_applicable {
                                for definition in &impl_def.associated_type_data {
                                    diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                        module_id.clone(),
                                        DiagnosticCode::KindExpectedType,
                                        "guarded or partial-target impls cannot introduce associated type data",
                                        definition.range,
                                    ));
                                }
                                continue;
                            }
                            for definition in &impl_def.associated_type_data {
                                let domain_id = crate::identity::AssociatedStaticDomainDeclarationId::new(declaration.clone(), definition.name.clone());
                                if !associated_surface_admits(
                                    &associated_surfaces_table,
                                    &declaration,
                                    &crate::associated::AssociatedMemberId::StaticDomain(domain_id.clone()),
                                ) {
                                    continue;
                                }
                                let catalog = static_case_catalogs
                                    .get(&crate::identity::StaticDomainDeclarationId::Associated(domain_id))
                                    .map(Arc::as_ref);
                                let (product, diagnostics) =
                                    crate::checker::algebraic_declaration::build_associated_static_domain_declaration_with_static_products(
                                        &declaration,
                                        definition,
                                        Arc::make_mut(&mut self.store),
                                        &declarations,
                                        &resolver,
                                        module_id,
                                        catalog,
                                        &static_case_signatures_table,
                                    );
                                diags_by_module.entry(module_id.clone()).or_default().extend(diagnostics);
                                if let Some(product) = product {
                                    candidates.push(product);
                                }
                            }
                        }
                        for product in candidates {
                            let product = Arc::new(product);
                            match crate::db::query::query_associated_static_domain_declaration(&mut self.db, parsed_unit, product.clone(), &self.store) {
                                QueryOutcome::Ready(product) => {
                                    associated_static_domains_table.insert(product.id.clone(), product);
                                }
                                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                            }
                        }
                        declaration
                    }
                    _ => continue,
                };
                if !associated_surfaces_table.surfaces.contains_key(&declaration) {
                    let assoc_surface = Arc::new(crate::associated::AssociatedSurface::new(declaration.clone()));
                    associated_surfaces_table.insert(declaration, assoc_surface.clone());
                    let _ = query_associated_surface(&mut self.db, assoc_surface);
                }
            }
            let retained_type_member_ids: BTreeSet<_> = type_member_declarations_table
                .keys()
                .filter(|id| &id.owner.module == module_id)
                .cloned()
                .collect();
            let stale_type_member_keys: Vec<_> = self
                .db
                .query_keys_for_module(module_id)
                .filter(|key| matches!(key, QueryKey::TypeMemberDeclaration(id) if !retained_type_member_ids.contains(id)))
                .cloned()
                .collect();
            for key in stale_type_member_keys {
                self.db.retire_query(&key);
            }
        }

        resolver.update_associated_type_members(&associated_surfaces_table, &type_member_declarations_table);

        // Override safety is a hierarchy-wide publication invariant. Validate
        // only after every formal signature and dispatch surface is available.
        for (module_id, shard) in &self.semantic_structure_shards {
            // Canonical Universe support declarations are compiled against
            // long-standing native-facing contracts. Identify them by both
            // their Universe module identity and the canonical provider URI;
            // synthetic test fixtures may validly use a Universe module ID.
            let canonical_universe_source = shard
                .source
                .source
                .as_ref()
                .is_some_and(|source| phalcom_modules::universe_module_from_uri(source.source_id.0.as_ref()).as_ref() == Some(module_id));
            if canonical_universe_source {
                continue;
            }
            let mut context = CheckingContext::new_with_dispatch_ref(
                Arc::make_mut(&mut self.store),
                &hierarchy,
                &resolver,
                &declarations,
                &dispatch,
                module_id.clone(),
            );
            for statement in &shard.source.program.statements {
                if let Statement::Class(class_def) = statement {
                    crate::checker::declaration::validate_class_overrides(&mut context, class_def, &callable_signatures);
                }
            }
            diags_by_module.entry(module_id.clone()).or_default().extend(context.diagnostics);
        }

        // 7. Check field defaults, constructors, then ordinary callable bodies.
        // Constructor-first ordering makes lifecycle publication independent of
        // source member order.
        let default_field_lifecycle = if structural_aggregates_reusable {
            self.default_field_lifecycle.clone()
        } else {
            let mut table = if previous_snapshot.is_some() {
                self.default_field_lifecycle.clone()
            } else {
                crate::checker::field_lifecycle::FieldLifecycleTable::default()
            };
            if previous_snapshot.is_some() {
                for module in structural_work_modules.iter().chain(removed_modules.iter()) {
                    table.remove_module(module);
                }
            }
            let modules = if previous_snapshot.is_some() {
                structural_work_modules.iter().cloned().collect::<Vec<_>>()
            } else {
                self.semantic_structure_shards.keys().cloned().collect::<Vec<_>>()
            };
            for module_id in modules {
                let Some(shard) = self.semantic_structure_shards.get(&module_id) else {
                    continue;
                };
                let parsed_unit = &shard.source;
                let mut ctx = CheckingContext::new_with_dispatch_ref(
                    Arc::make_mut(&mut self.store),
                    &hierarchy,
                    &resolver,
                    &declarations,
                    &dispatch,
                    module_id.clone(),
                );
                ctx.attach_field_signatures(&field_signatures);
                ctx.attach_enum_semantics(&enum_semantics);
                ctx.attach_data_semantics(&data_semantics);
                ctx.attach_associated_families(&associated_surfaces_table);
                for stmt in &parsed_unit.program.statements {
                    if let Statement::Class(class_def) = stmt {
                        let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                        if blocked_declarations.contains(&decl_id) {
                            continue;
                        }
                        table.extend(crate::checker::field_lifecycle::default_field_seeds(&mut ctx, class_def));
                    }
                }
            }
            table
        };
        let mut field_lifecycle = default_field_lifecycle.clone();
        let mut callable_analyses = previous_snapshot.as_ref().map_or_else(HashMap::new, |snapshot| {
            snapshot
                .callable_analyses
                .iter()
                .filter(|(callable, _)| {
                    current_modules.contains(callable.module())
                        && !contribution_delta.callable_bodies_removed.contains(*callable)
                        && !removed_callable_bodies.contains(*callable)
                        && !retired_conformance_witnesses.contains(*callable)
                })
                .map(|(callable, analysis)| (callable.clone(), analysis.clone()))
                .collect()
        });

        // Retained conformance definitions still have source bodies and access
        // modifiers even when their modules need no structural reconstruction.
        // Recover these source facts through the accepted definition coordinates,
        // not through the set of bodies that happened to be rechecked this turn.
        for definition in callable_definitions.values() {
            let crate::impls::CallableDefinitionOrigin::ConformanceWitness(impl_id) = &definition.origin else {
                continue;
            };
            let Some(unit) = retained_sources.get(&impl_id.module) else { continue };
            let Some(Statement::Impl(implementation)) = unit.program.statements.get(impl_id.local.0 as usize) else {
                continue;
            };
            let Some(member) = implementation.members.get(definition.source_member_index).and_then(|member| member.behavior()) else {
                continue;
            };
            conformance_witness_visibilities.insert(definition.callable.clone(), crate::traits::behavior_member_visibility(member));
            let has_body = match member {
                phalcom_ast::ast::BehaviorMember::Method(member) => member.body.statements().is_some(),
                phalcom_ast::ast::BehaviorMember::Getter(member) => member.body.statements().is_some(),
                phalcom_ast::ast::BehaviorMember::Setter(member) => member.body.statements().is_some(),
                phalcom_ast::ast::BehaviorMember::Index(member) => member.body.statements().is_some(),
            };
            if has_body {
                conformance_witness_bodies.insert(definition.callable.clone());
            }
        }

        // Conformance signatures and body presence are already canonical at
        // this point. Publish witness/default plans before ordinary callable
        // bodies are checked so the canonical dispatch resolver can consume
        // exact P2 evidence during those bodies.
        let _ = publish_conformance_witness_plans(
            &mut self.db,
            Arc::make_mut(&mut self.store),
            &hierarchy,
            &dispatch,
            &declarations,
            &conformance_index,
            &callable_signatures,
            &mut conformance_witness_plans,
            &conformance_witness_bodies,
            &conformance_witness_visibilities,
            &conformance_invalid_witnesses,
            &conformance_invalid_members,
            &conformance_associated_type_plans,
            &data_semantics,
            &trait_surfaces,
            &mut diags_by_module,
            false,
            &diagnostic_work_modules,
        )?;
        let trait_dispatch_index = crate::trait_dispatch::TraitDispatchIndex::build(&conformance_index, &trait_surfaces);
        let conformance_semantics = crate::trait_dispatch::ConformanceSemanticView {
            trait_dispatch: &trait_dispatch_index,
            conformance_index: &conformance_index,
            witness_plans: &conformance_witness_plans,
            trait_surfaces: &trait_surfaces,
            derived_conformance_rules: &derived_conformance_rules,
            callable_signatures: None,
        };

        // Refresh changed dispatch fingerprint owners before any cached body
        // replays its previous semantic dependencies. Include removed buckets:
        // their current fingerprint is the canonical empty-bucket fingerprint.
        let changed_bucket_keys = previous_snapshot.as_ref().map_or_else(BTreeSet::new, |previous| {
            let bucket_keys = trait_dispatch_index
                .buckets()
                .keys()
                .chain(previous.trait_dispatch_index.buckets().keys())
                .cloned()
                .collect::<BTreeSet<_>>();
            bucket_keys
                .into_iter()
                .filter(|bucket_key| trait_dispatch_index.bucket_fingerprint(bucket_key) != previous.trait_dispatch_index.bucket_fingerprint(bucket_key))
                .map(QueryKey::TraitDispatchBucket)
                .collect()
        });
        for key in &changed_bucket_keys {
            let QueryKey::TraitDispatchBucket(bucket_key) = key else {
                unreachable!("changed bucket set contains only trait dispatch buckets");
            };
            ready_or_error(crate::db::query::refresh_trait_dispatch_bucket_root(
                &mut self.db,
                bucket_key,
                trait_dispatch_index.bucket_fingerprint(bucket_key),
            ))?;
        }

        // Generic-conformance roots include negative proof rows and nested
        // obligations whose own candidate slices may have changed. Refresh
        // every previously observed exact proposition against the rebuilt
        // conformance view before cached bodies replay their dependencies.
        // This is necessary for negative-to-positive transitions: a missing
        // proof has no selected ConformanceDispatch edge to seed its clients.
        let generic_conformance_roots = self
            .db
            .query_keys()
            .filter_map(|key| match key {
                QueryKey::GenericConformance(conformance_key) => Some(conformance_key.clone()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let mut changed_generic_conformance_roots = BTreeSet::new();
        for conformance_key in generic_conformance_roots {
            let query_key = QueryKey::GenericConformance(conformance_key.clone());
            let previous_fingerprint = self.db.ready_product_fingerprint(&query_key);
            ready_or_error(crate::db::query::refresh_generic_conformance_root(
                &mut self.db,
                &conformance_key,
                &conformance_semantics,
                &declarations,
                &hierarchy,
                Arc::make_mut(&mut self.store),
            ))?;
            if previous_fingerprint != self.db.ready_product_fingerprint(&query_key) {
                changed_generic_conformance_roots.insert(query_key);
            }
        }

        // Trait defaults are checked once against the complete abstract
        // contract. They receive the trait header directly instead of
        // looking for a nominal declaration entry, and their `Self` receiver
        // remains a semantic SelfType with no executable dispatch target.
        for (module_id, shard) in &self.semantic_structure_shards {
            let parsed_unit = &shard.source;
            for stmt in &parsed_unit.program.statements {
                let Statement::Trait(trait_def) = stmt else { continue };
                let declaration = DeclarationId::new(module_id.clone(), trait_def.name.clone().into());
                let (Some(header), Some(surface)) = (trait_headers.get(&declaration), trait_surfaces.get(&declaration)) else {
                    continue;
                };
                for trait_member in &trait_def.members {
                    let Some(member) = trait_member.behavior() else {
                        continue;
                    };
                    let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(member);
                    let side = crate::semantic_shard::behavior_side(member);
                    let Some(callable) = crate::checker::declaration_signature::callable_id_for_syntax(
                        &crate::identity::CallableOwnerId::Declaration(declaration.clone()),
                        syntax,
                        side,
                    ) else {
                        continue;
                    };
                    let body = match member {
                        phalcom_ast::ast::BehaviorMember::Method(method) => method.body.statements().map(|body| (body, method.range)),
                        phalcom_ast::ast::BehaviorMember::Getter(getter) => getter.body.statements().map(|body| (body, getter.range)),
                        phalcom_ast::ast::BehaviorMember::Setter(setter) => setter.body.statements().map(|body| (body, setter.range)),
                        phalcom_ast::ast::BehaviorMember::Index(index) => index.body.statements().map(|body| (body, index.range)),
                    };
                    let Some((body, body_range)) = body else { continue };
                    let Some(surface_member) = surface.get_by_selector(&callable.selector, side) else {
                        continue;
                    };
                    let query_key = QueryKey::CallableBody(callable.clone());
                    let forced = callable_body_work.contains(&callable);
                    if forced {
                        self.db.discard_for_recompute(&query_key);
                    }
                    let previous_computation_revision = self.db.query_state(&query_key).and_then(|state| state.revision());
                    let formal_inputs = FormalQueryInputs {
                        sources: &retained_sources,
                        source_resolution_input,
                        linked_component_product,
                        linked: &input.linked,
                        topology: semantic_topology.as_ref(),
                        import_products: &input.import_products,
                        hierarchy: &hierarchy,
                        base_resolver: &resolver,
                        declarations: &declarations,
                        type_aliases: &type_aliases,
                        field_signatures: None,
                        field_lifecycle: None,
                        constructor_initializations: Some(&constructor_initializations),
                        callable_signatures: Some(&callable_signatures),
                        enum_semantics: Some(&enum_semantics),
                        data_semantics: Some(&data_semantics),
                        associated_families: Some(&associated_surfaces_table),
                        type_member_declarations: Some(&type_member_declarations_table),
                        conformance_semantics: Some(&conformance_semantics),
                        module_values: None,
                        semantic_linked: input.semantic_linked.as_deref(),
                    };
                    let outcome = query_callable_body_with_formal_inputs(
                        &mut self.db,
                        CallableBodyQuery {
                            callable: callable.clone(),
                            body,
                            body_range,
                            declared_signature: Some((&surface_member.callable, &surface_member.signature)),
                            owner_generic_signature: header.generic_signature.as_ref(),
                            receiver_value_override: None,
                            instance_self_override: None,
                            trait_surface: Some(surface),
                            store: Arc::make_mut(&mut self.store),
                            hierarchy: &hierarchy,
                            resolver: &resolver,
                            declarations: &declarations,
                            dispatch: &dispatch,
                            module: module_id.clone(),
                            budget,
                            cancel,
                            formal_inputs: Some(&formal_inputs),
                        },
                    );
                    match outcome {
                        QueryOutcome::Ready(analysis) => {
                            if forced
                                || previous_snapshot.is_none()
                                || self.db.query_state(&query_key).is_some_and(|state| {
                                    state.revision() == Some(self.db.revision()) && previous_computation_revision != Some(self.db.revision())
                                })
                            {
                                callable_dispositions.insert(callable.clone(), CallableRevisionDisposition::Recomputed);
                            } else {
                                callable_dispositions.entry(callable.clone()).or_insert(CallableRevisionDisposition::Reused);
                            }
                            callable_analyses.insert(callable, analysis);
                        }
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    }
                }
            }
        }
        for constructors_only in [true, false] {
            for (module_id, shard) in &self.semantic_structure_shards {
                if !semantic_work_modules.contains(module_id) {
                    continue;
                }
                let parsed_unit = &shard.source;
                for (stmt_idx, stmt) in parsed_unit.program.statements.iter().enumerate() {
                    if let Statement::Class(class_def) = stmt {
                        let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                        if blocked_declarations.contains(&decl_id) {
                            continue;
                        }
                        let type_params_map = if let Some(sig) = declarations.generic_signature(&decl_id) {
                            let mut map = std::collections::HashMap::new();
                            for &param_id in sig.parameters.iter() {
                                let name = self.store.type_parameter(param_id).name.to_string();
                                let binding = type_level_binding_for_parameter(Arc::make_mut(&mut self.store), param_id);
                                map.insert(name, binding);
                            }
                            map
                        } else {
                            std::collections::HashMap::new()
                        };
                        let scoped_resolver = crate::types::annotation::ScopedTypeResolver {
                            parent: &resolver,
                            type_parameters: type_params_map,
                        };

                        // A retained declaration surface may still contain a
                        // weaker source-formal return fact than the canonical
                        // callable signature retained for this module.  Sync
                        // this module's exact callable projections before
                        // checking its bodies so callers see the same formal
                        // evidence that a cold pass will publish.
                        for member in &class_def.members {
                            let Some(callable) = crate::checker::declaration_signature::callable_id_for_member(&decl_id, member) else {
                                continue;
                            };
                            let Some(signature) = callable_signatures.get(&callable) else {
                                continue;
                            };
                            dispatch.update_callable_return_type(&callable, signature.published_return_knowledge());
                        }

                        for member in &class_def.members {
                            let is_constructor =
                                matches!(member, ClassMember::Method(m) if m.is_constructor || m.attributes.iter().any(|a| a.name == "constructor"));
                            if is_constructor != constructors_only {
                                continue;
                            }
                            let side = if is_constructor {
                                crate::identity::DispatchSide::Instance
                            } else {
                                crate::checker::declaration::member_side(member)
                            };

                            let (selector_opt, body_opt, range_opt) = match member {
                                ClassMember::Method(m) => {
                                    let slots = m
                                        .params
                                        .iter()
                                        .filter(|p| p.rest_mode == phalcom_ast::ast::RestMode::None)
                                        .map(|p| {
                                            if let Some(ref l) = p.label {
                                                if l == "_" {
                                                    phalcom_common::selector::SelectorSlot::Positional
                                                } else {
                                                    phalcom_common::selector::SelectorSlot::Label(l.clone())
                                                }
                                            } else {
                                                phalcom_common::selector::SelectorSlot::Positional
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    (Selector::method(&m.name, slots).ok(), m.body.statements(), Some(m.range))
                                }
                                ClassMember::Getter(g) => (Selector::getter(&g.name).ok(), g.body.statements(), Some(g.range)),
                                ClassMember::Setter(s) => (Selector::setter(&s.name).ok(), s.body.statements(), Some(s.range)),
                                ClassMember::Index(index) => (
                                    crate::checker::declaration_signature::callable_id_for_member(&decl_id, member).map(|callable| callable.selector),
                                    index.body.statements(),
                                    Some(index.range),
                                ),
                                _ => (None, None, None),
                            };

                            if let (Some(selector), Some(body), Some(range)) = (selector_opt, body_opt, range_opt) {
                                let callable_id = crate::identity::CallableId::new(decl_id.clone(), selector, side);
                                let query_key = QueryKey::CallableBody(callable_id.clone());

                                let formal_inputs = FormalQueryInputs {
                                    sources: &retained_sources,
                                    source_resolution_input,
                                    linked_component_product,
                                    linked: &input.linked,
                                    topology: semantic_topology.as_ref(),
                                    import_products: &input.import_products,
                                    hierarchy: &hierarchy,
                                    base_resolver: &resolver,
                                    declarations: &declarations,
                                    type_aliases: &type_aliases,
                                    field_signatures: Some(&field_signatures),
                                    field_lifecycle: Some(if is_constructor { &default_field_lifecycle } else { &field_lifecycle }),
                                    constructor_initializations: Some(&constructor_initializations),
                                    callable_signatures: Some(&callable_signatures),
                                    enum_semantics: Some(&enum_semantics),
                                    data_semantics: Some(&data_semantics),
                                    associated_families: Some(&associated_surfaces_table),
                                    type_member_declarations: Some(&type_member_declarations_table),
                                    conformance_semantics: Some(&conformance_semantics),
                                    module_values: None,
                                    semantic_linked: input.semantic_linked.as_deref(),
                                };

                                let forced = callable_body_work.contains(&callable_id);
                                if forced {
                                    self.db.discard_for_recompute(&query_key);
                                } else {
                                    if previous_snapshot.is_some()
                                        && callable_analyses.contains_key(&callable_id)
                                        && refresh_cached_body_dependencies(&mut self.db, &query_key, &formal_inputs, Arc::make_mut(&mut self.store)).is_ok()
                                        && self.db.validate_ready(&query_key)
                                    {
                                        callable_dispositions.entry(callable_id.clone()).or_insert(CallableRevisionDisposition::Reused);
                                        continue;
                                    }

                                    refresh_cached_body_dependencies(&mut self.db, &query_key, &formal_inputs, Arc::make_mut(&mut self.store))?;
                                }
                                let previous_computation_revision = self.db.query_state(&query_key).and_then(|state| state.revision());
                                let outcome = query_callable_body_with_formal_inputs(
                                    &mut self.db,
                                    CallableBodyQuery {
                                        callable: callable_id.clone(),
                                        body,
                                        body_range: range,
                                        declared_signature: None,
                                        owner_generic_signature: None,
                                        receiver_value_override: None,
                                        instance_self_override: None,
                                        trait_surface: None,
                                        store: Arc::make_mut(&mut self.store),
                                        hierarchy: &hierarchy,
                                        resolver: &scoped_resolver,
                                        declarations: &declarations,
                                        dispatch: &dispatch,
                                        module: module_id.clone(),
                                        budget,
                                        cancel,
                                        formal_inputs: Some(&formal_inputs),
                                    },
                                );

                                match outcome {
                                    QueryOutcome::Ready(analysis) => {
                                        if self.db.query_state(&query_key).is_some_and(|state| {
                                            state.revision() == Some(self.db.revision()) && previous_computation_revision != Some(self.db.revision())
                                        }) {
                                            callable_dispositions.insert(callable_id.clone(), CallableRevisionDisposition::Recomputed);
                                        } else {
                                            callable_dispositions.entry(callable_id.clone()).or_insert(CallableRevisionDisposition::Reused);
                                        }
                                        if let Some(sig) = callable_signatures.get_mut(&callable_id) {
                                            if sig.return_validation != analysis.return_validation {
                                                sig.return_validation = analysis.return_validation;
                                                dispatch.update_callable_return_type(&callable_id, sig.published_return_knowledge());
                                            }
                                        }
                                        callable_analyses.insert(callable_id.clone(), analysis);
                                        if is_constructor {
                                            let finalized = crate::checker::field_lifecycle::finalize_instance_field_lifecycle(
                                                &default_field_lifecycle,
                                                callable_analyses
                                                    .values()
                                                    .filter(|analysis| {
                                                        analysis.callable.try_declaration_owner() == Some(&decl_id)
                                                            && analysis.callable.side == DispatchSide::Instance
                                                    })
                                                    .filter(|analysis| {
                                                        callable_signatures
                                                            .get_for_body(&analysis.callable)
                                                            .is_some_and(|signature| signature.is_constructor())
                                                    })
                                                    .map(AsRef::as_ref),
                                            );
                                            for (field, fact) in finalized.fields {
                                                if field.owner == decl_id {
                                                    field_lifecycle.insert(field, fact);
                                                }
                                            }
                                        }
                                    }
                                    QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                                    QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                                    QueryOutcome::Blocked(reason) => {
                                        return Err(QueryOutcome::Blocked(reason));
                                    }
                                    QueryOutcome::Failed(err) => return Err(QueryOutcome::Failed(err)),
                                }
                            }
                        }
                    } else if let Statement::Data(_data_def) = stmt {
                        // Data and enums are structural; their behavior is declared in Statement::Impl.
                    } else if let Statement::Impl(impl_def) = stmt {
                        if constructors_only {
                            continue;
                        }
                        if matches!(impl_def.kind, phalcom_ast::ast::ImplKind::Conformance { .. }) {
                            let impl_id = crate::identity::ImplId::new(module_id.clone(), crate::identity::ImplLocalId(stmt_idx as u32));
                            let (contribution_generic_signature, contribution_target_head, trait_decl) =
                                if let Some(contribution) = conformance_index.get(&impl_id).cloned() {
                                    if !contribution.is_lookup_eligible() {
                                        continue;
                                    }
                                    (
                                        contribution.generic_signature,
                                        Some(contribution.target_head),
                                        contribution.trait_ref.declaration,
                                    )
                                } else if let Some(rule) = derived_conformance_rules.iter().find(|r| r.impl_id == impl_id) {
                                    (
                                        Some(rule.generic_signature.clone()),
                                        Some(rule.target_template),
                                        rule.trait_ref.declaration.clone(),
                                    )
                                } else {
                                    continue;
                                };
                            let Some(trait_surface) = trait_surfaces.get(&trait_decl) else {
                                continue;
                            };
                            let type_params_map = if let Some(sig) = contribution_generic_signature.as_ref() {
                                let mut map = std::collections::HashMap::new();
                                for &param_id in sig.parameters.iter() {
                                    let name = self.store.type_parameter(param_id).name.to_string();
                                    let binding = type_level_binding_for_parameter(Arc::make_mut(&mut self.store), param_id);
                                    map.insert(name, binding);
                                }
                                map
                            } else {
                                std::collections::HashMap::new()
                            };
                            let scoped_resolver = crate::types::annotation::ScopedTypeResolver {
                                parent: &resolver,
                                type_parameters: type_params_map,
                            };
                            let callable_owner = crate::identity::CallableOwnerId::Conformance(impl_id.clone());
                            for impl_member in &impl_def.members {
                                let Some(member) = impl_member.behavior() else {
                                    continue;
                                };
                                let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(member);
                                let side = crate::semantic_shard::behavior_side(member);
                                let Some(callable) = crate::checker::declaration_signature::callable_id_for_syntax(&callable_owner, syntax, side) else {
                                    continue;
                                };
                                let Some(signature) = callable_signatures.get(&callable) else {
                                    continue;
                                };
                                let Some((body, body_range)) = (match member {
                                    phalcom_ast::ast::BehaviorMember::Method(method) => method.body.statements().map(|body| (body, method.range)),
                                    phalcom_ast::ast::BehaviorMember::Getter(getter) => getter.body.statements().map(|body| (body, getter.range)),
                                    phalcom_ast::ast::BehaviorMember::Setter(setter) => setter.body.statements().map(|body| (body, setter.range)),
                                    phalcom_ast::ast::BehaviorMember::Index(index) => index.body.statements().map(|body| (body, index.range)),
                                }) else {
                                    continue;
                                };
                                let query_key = QueryKey::CallableBody(callable.clone());
                                let formal_inputs = FormalQueryInputs {
                                    sources: &retained_sources,
                                    source_resolution_input,
                                    linked_component_product,
                                    linked: &input.linked,
                                    topology: semantic_topology.as_ref(),
                                    import_products: &input.import_products,
                                    hierarchy: &hierarchy,
                                    base_resolver: &resolver,
                                    declarations: &declarations,
                                    type_aliases: &type_aliases,
                                    field_signatures: Some(&field_signatures),
                                    field_lifecycle: Some(&field_lifecycle),
                                    constructor_initializations: Some(&constructor_initializations),
                                    callable_signatures: Some(&callable_signatures),
                                    enum_semantics: Some(&enum_semantics),
                                    data_semantics: Some(&data_semantics),
                                    associated_families: Some(&associated_surfaces_table),
                                    type_member_declarations: Some(&type_member_declarations_table),
                                    conformance_semantics: Some(&conformance_semantics),
                                    module_values: None,
                                    semantic_linked: input.semantic_linked.as_deref(),
                                };
                                let forced = callable_body_work.contains(&callable);
                                if forced {
                                    self.db.discard_for_recompute(&query_key);
                                } else {
                                    if previous_snapshot.is_some()
                                        && callable_analyses.contains_key(&callable)
                                        && refresh_cached_body_dependencies(&mut self.db, &query_key, &formal_inputs, Arc::make_mut(&mut self.store)).is_ok()
                                        && self.db.validate_ready(&query_key)
                                    {
                                        callable_dispositions.entry(callable).or_insert(CallableRevisionDisposition::Reused);
                                        continue;
                                    }
                                    refresh_cached_body_dependencies(&mut self.db, &query_key, &formal_inputs, Arc::make_mut(&mut self.store))?;
                                }
                                let previous_computation_revision = self.db.query_state(&query_key).and_then(|state| state.revision());
                                let outcome = query_callable_body_with_formal_inputs(
                                    &mut self.db,
                                    CallableBodyQuery {
                                        callable: callable.clone(),
                                        body,
                                        body_range,
                                        declared_signature: Some((&signature.callable, signature)),
                                        owner_generic_signature: contribution_generic_signature.as_ref(),
                                        receiver_value_override: contribution_target_head,
                                        instance_self_override: None,
                                        trait_surface: Some(trait_surface),
                                        store: Arc::make_mut(&mut self.store),
                                        hierarchy: &hierarchy,
                                        resolver: &scoped_resolver,
                                        declarations: &declarations,
                                        dispatch: &dispatch,
                                        module: module_id.clone(),
                                        budget,
                                        cancel,
                                        formal_inputs: Some(&formal_inputs),
                                    },
                                );
                                match outcome {
                                    QueryOutcome::Ready(analysis) => {
                                        if self.db.query_state(&query_key).is_some_and(|state| {
                                            state.revision() == Some(self.db.revision()) && previous_computation_revision != Some(self.db.revision())
                                        }) {
                                            callable_dispositions.insert(callable.clone(), CallableRevisionDisposition::Recomputed);
                                        } else {
                                            callable_dispositions.entry(callable.clone()).or_insert(CallableRevisionDisposition::Reused);
                                        }
                                        if let Some(sig) = callable_signatures.get_mut(&callable)
                                            && sig.return_validation != analysis.return_validation
                                        {
                                            sig.return_validation = analysis.return_validation;
                                        }
                                        callable_analyses.insert(callable, analysis);
                                    }
                                    QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                                    QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                                    QueryOutcome::Blocked(reason) => {
                                        return Err(QueryOutcome::Blocked(reason));
                                    }
                                    QueryOutcome::Failed(err) => return Err(QueryOutcome::Failed(err)),
                                }
                            }
                            continue;
                        }
                        if !matches!(impl_def.kind, phalcom_ast::ast::ImplKind::Inherent) {
                            continue;
                        }
                        let impl_id = crate::identity::ImplId::new(module_id.clone(), crate::identity::ImplLocalId(stmt_idx as u32));
                        let mut ctx =
                            crate::checker::CheckingContext::new(Arc::make_mut(&mut self.store), &hierarchy, &resolver, &declarations, module_id.clone());
                        ctx.attach_field_signatures(&field_signatures);
                        ctx.attach_data_semantics(&data_semantics);
                        ctx.attach_enum_semantics(&enum_semantics);
                        let Ok(resolved_impl_target) = crate::impls::resolve_inherent_impl_target(&mut ctx, &impl_id, impl_def) else {
                            continue;
                        };
                        let target_owner = resolved_impl_target.target.to_callable_owner();

                        let type_params_map = if let Some(sig) = resolved_impl_target.generic_signature.as_ref() {
                            let mut map = std::collections::HashMap::new();
                            for &param_id in sig.parameters.iter() {
                                let name = self.store.type_parameter(param_id).name.to_string();
                                let binding = type_level_binding_for_parameter(Arc::make_mut(&mut self.store), param_id);
                                map.insert(name, binding);
                            }
                            map
                        } else {
                            std::collections::HashMap::new()
                        };
                        let scoped_resolver = crate::types::annotation::ScopedTypeResolver {
                            parent: &resolver,
                            type_parameters: type_params_map,
                        };

                        for (member_index, impl_member) in impl_def.members.iter().enumerate() {
                            let Some(member) = impl_member.behavior() else {
                                continue;
                            };
                            let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(member);
                            let is_class_side = syntax.attributes().iter().any(|a| a.name == "class")
                                || match member {
                                    phalcom_ast::ast::BehaviorMember::Method(m) => m.is_static,
                                    phalcom_ast::ast::BehaviorMember::Getter(g) => g.is_static,
                                    phalcom_ast::ast::BehaviorMember::Setter(s) => s.is_static,
                                    phalcom_ast::ast::BehaviorMember::Index(_) => false,
                                };
                            let side = if is_class_side {
                                crate::identity::DispatchSide::Class
                            } else {
                                crate::identity::DispatchSide::Instance
                            };

                            let (selector_opt, body_opt, range_opt) = match member {
                                phalcom_ast::ast::BehaviorMember::Method(m) => {
                                    let slots = m
                                        .params
                                        .iter()
                                        .filter(|p| p.rest_mode == phalcom_ast::ast::RestMode::None)
                                        .map(|p| {
                                            if let Some(ref l) = p.label {
                                                if l == "_" {
                                                    phalcom_common::selector::SelectorSlot::Positional
                                                } else {
                                                    phalcom_common::selector::SelectorSlot::Label(l.clone())
                                                }
                                            } else {
                                                phalcom_common::selector::SelectorSlot::Positional
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    (Selector::method(&m.name, slots).ok(), m.body.statements(), Some(m.range))
                                }
                                phalcom_ast::ast::BehaviorMember::Getter(g) => (Selector::getter(&g.name).ok(), g.body.statements(), Some(g.range)),
                                phalcom_ast::ast::BehaviorMember::Setter(s) => (Selector::setter(&s.name).ok(), s.body.statements(), Some(s.range)),
                                phalcom_ast::ast::BehaviorMember::Index(i) => {
                                    let slots = i
                                        .params
                                        .iter()
                                        .map(|p| {
                                            if let Some(ref l) = p.label {
                                                if l == "_" {
                                                    phalcom_common::selector::SelectorSlot::Positional
                                                } else {
                                                    phalcom_common::selector::SelectorSlot::Label(l.clone())
                                                }
                                            } else {
                                                phalcom_common::selector::SelectorSlot::Positional
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    let sel = match &i.accessor {
                                        phalcom_ast::ast::IndexAccessor::Get => Selector::subscript_get(slots).ok(),
                                        phalcom_ast::ast::IndexAccessor::Set { .. } => Selector::subscript_set(slots).ok(),
                                    };
                                    (sel, i.body.statements(), Some(i.range))
                                }
                            };

                            if let (Some(selector), Some(body), Some(range)) = (selector_opt, body_opt, range_opt) {
                                let callable_id = crate::identity::CallableId::new(target_owner.clone(), selector, side);
                                let accepted_definition = self
                                    .db
                                    .product(&QueryKey::CallableDefinition(callable_id.clone()))
                                    .and_then(|product| product.as_callable_definition())
                                    .cloned();
                                if !accepted_definition.as_ref().is_some_and(|definition| {
                                    definition.source_member_index == member_index
                                        && definition.origin == crate::impls::CallableDefinitionOrigin::InherentImpl(impl_id.clone())
                                }) {
                                    continue;
                                }
                                let body_context = accepted_definition.as_ref().and_then(|definition| definition.inherent_body_context.clone());
                                let canonical_signature = if let Some(signature) = callable_signatures.get(&callable_id).cloned() {
                                    signature
                                } else {
                                    // An accepted current source definition may enter body work
                                    // without an earlier signature-worklist seed after an
                                    // incremental module replacement. Requery the canonical
                                    // signature from that source and its checked prerequisites;
                                    // never synthesize a signature from the body or reuse an
                                    // unchecked last-known-good projection.
                                    let record_key_projections = inherent_callable_record_key_projections.get(&(impl_id.clone(), member_index));
                                    let signature = match query_callable_signature_with_inputs(
                                        &mut self.db,
                                        callable_id.clone(),
                                        parsed_unit.clone(),
                                        SignatureQueryInputs {
                                            store: Arc::make_mut(&mut self.store),
                                            hierarchy: &hierarchy,
                                            resolver: &scoped_resolver,
                                            record_key_projections,
                                            declarations: &declarations,
                                            linked: Some(input.linked.as_ref()),
                                            type_aliases: Some(&type_aliases),
                                            import_products: Some(&input.import_products),
                                        },
                                    ) {
                                        QueryOutcome::Ready(signature) => (*signature).clone(),
                                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                                    };
                                    callable_signatures.insert(signature.clone());
                                    signature
                                };
                                let body_signature = body_context
                                    .as_ref()
                                    .map(|context| {
                                        context
                                            .signature_for_body(Arc::make_mut(&mut self.store), &declarations, canonical_signature.clone())
                                            .expect("validated covering substitution must materialize its accepted callable signature")
                                    })
                                    .unwrap_or(canonical_signature);
                                let query_key = QueryKey::CallableBody(callable_id.clone());

                                let formal_inputs = FormalQueryInputs {
                                    sources: &retained_sources,
                                    source_resolution_input,
                                    linked_component_product,
                                    linked: &input.linked,
                                    topology: semantic_topology.as_ref(),
                                    import_products: &input.import_products,
                                    hierarchy: &hierarchy,
                                    base_resolver: &resolver,
                                    declarations: &declarations,
                                    type_aliases: &type_aliases,
                                    field_signatures: Some(&field_signatures),
                                    field_lifecycle: Some(&field_lifecycle),
                                    constructor_initializations: Some(&constructor_initializations),
                                    callable_signatures: Some(&callable_signatures),
                                    enum_semantics: Some(&enum_semantics),
                                    data_semantics: Some(&data_semantics),
                                    associated_families: Some(&associated_surfaces_table),
                                    type_member_declarations: Some(&type_member_declarations_table),
                                    conformance_semantics: Some(&conformance_semantics),
                                    module_values: None,
                                    semantic_linked: input.semantic_linked.as_deref(),
                                };

                                let forced = callable_body_work.contains(&callable_id);
                                if forced {
                                    self.db.discard_for_recompute(&query_key);
                                } else {
                                    if previous_snapshot.is_some()
                                        && callable_analyses.contains_key(&callable_id)
                                        && refresh_cached_body_dependencies(&mut self.db, &query_key, &formal_inputs, Arc::make_mut(&mut self.store)).is_ok()
                                        && self.db.validate_ready(&query_key)
                                    {
                                        callable_dispositions.entry(callable_id.clone()).or_insert(CallableRevisionDisposition::Reused);
                                        continue;
                                    }

                                    refresh_cached_body_dependencies(&mut self.db, &query_key, &formal_inputs, Arc::make_mut(&mut self.store))?;
                                }
                                let previous_computation_revision = self.db.query_state(&query_key).and_then(|state| state.revision());
                                let outcome = query_callable_body_with_formal_inputs(
                                    &mut self.db,
                                    CallableBodyQuery {
                                        callable: callable_id.clone(),
                                        body,
                                        body_range: range,
                                        declared_signature: Some((&callable_id, &body_signature)),
                                        owner_generic_signature: body_context.as_ref().and_then(|context| context.generic_signature.as_ref()),
                                        receiver_value_override: body_context.as_ref().and_then(|context| context.runtime_receiver_override(callable_id.side)),
                                        instance_self_override: body_context.as_ref().map(|context| context.instance_self_override()),
                                        trait_surface: None,
                                        store: Arc::make_mut(&mut self.store),
                                        hierarchy: &hierarchy,
                                        resolver: &scoped_resolver,
                                        declarations: &declarations,
                                        dispatch: &dispatch,
                                        module: module_id.clone(),
                                        budget,
                                        cancel,
                                        formal_inputs: Some(&formal_inputs),
                                    },
                                );

                                match outcome {
                                    QueryOutcome::Ready(analysis) => {
                                        if self.db.query_state(&query_key).is_some_and(|state| {
                                            state.revision() == Some(self.db.revision()) && previous_computation_revision != Some(self.db.revision())
                                        }) {
                                            callable_dispositions.insert(callable_id.clone(), CallableRevisionDisposition::Recomputed);
                                        } else {
                                            callable_dispositions.entry(callable_id.clone()).or_insert(CallableRevisionDisposition::Reused);
                                        }
                                        if let Some(sig) = callable_signatures.get_mut(&callable_id) {
                                            if sig.return_validation != analysis.return_validation {
                                                sig.return_validation = analysis.return_validation;
                                                dispatch.update_callable_return_type(&callable_id, sig.published_return_knowledge());
                                            }
                                        }
                                        callable_analyses.insert(callable_id.clone(), analysis);
                                    }
                                    QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                                    QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                                    QueryOutcome::Blocked(reason) => {
                                        return Err(QueryOutcome::Blocked(reason));
                                    }
                                    QueryOutcome::Failed(err) => return Err(QueryOutcome::Failed(err)),
                                }
                            }
                        }
                    }
                }
            }
        }

        // Only a recomputed callable signature can publish a changed contract
        // to callers. A body-only recomputation with an unchanged declared
        // signature must stop at the provider.
        let downstream_body_roots = self
            .db
            .revision_recomputed_keys()
            .filter_map(|key| match key {
                QueryKey::CallableSignature(callable) => Some(QueryKey::CallableSignature(callable.clone())),
                QueryKey::CallableBody(callable) => Some(QueryKey::CallableSignature(callable.clone())),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let downstream_body_work = self
            .db
            .index()
            .reverse_closure(downstream_body_roots)
            .into_iter()
            .filter_map(|key| match key {
                QueryKey::CallableBody(callable)
                    if current_modules.contains(callable.module())
                        && callable_dispositions.get(&callable) != Some(&CallableRevisionDisposition::Recomputed) =>
                {
                    Some(callable)
                }
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        semantic_work_modules.extend(downstream_body_work.iter().map(|callable| callable.module().clone()));
        for callable in downstream_body_work {
            let Some(unit) = retained_sources.get(callable.module()).cloned() else {
                continue;
            };
            let callable_signature_snapshot = callable_signatures.clone();
            let is_constructor = callable_signatures.get_for_body(&callable).is_some_and(|signature| signature.is_constructor());
            let formal_inputs = FormalQueryInputs {
                sources: &retained_sources,
                source_resolution_input,
                linked_component_product,
                linked: &input.linked,
                topology: semantic_topology.as_ref(),
                import_products: &input.import_products,
                hierarchy: &hierarchy,
                base_resolver: &resolver,
                declarations: &declarations,
                type_aliases: &type_aliases,
                field_signatures: Some(&field_signatures),
                field_lifecycle: Some(if is_constructor { &default_field_lifecycle } else { &field_lifecycle }),
                constructor_initializations: Some(&constructor_initializations),
                callable_signatures: Some(&callable_signature_snapshot),
                enum_semantics: Some(&enum_semantics),
                data_semantics: Some(&data_semantics),
                associated_families: Some(&associated_surfaces_table),
                type_member_declarations: Some(&type_member_declarations_table),
                conformance_semantics: Some(&conformance_semantics),
                module_values: None,
                semantic_linked: input.semantic_linked.as_deref(),
            };
            let has_declared_return = callable_signatures
                .get_for_body(&callable)
                .is_some_and(|signature| !signature.declared_return.is_unknown())
                || callable_analyses
                    .get(&callable)
                    .is_some_and(|analysis| matches!(analysis.return_validation, crate::signature::ReturnContractValidation::Satisfied(_)));
            if has_declared_return
                && callable_dispositions.get(&callable) != Some(&CallableRevisionDisposition::Recomputed)
                && callable_analyses.contains_key(&callable)
            {
                refresh_cached_body_dependencies(
                    &mut self.db,
                    &QueryKey::CallableBody(callable.clone()),
                    &formal_inputs,
                    Arc::make_mut(&mut self.store),
                )?;
                if self.db.validate_ready(&QueryKey::CallableBody(callable.clone())) {
                    continue;
                }
            }
            revalidate_downstream_callable_body(
                &mut self.db,
                &callable,
                &unit,
                &formal_inputs,
                &default_field_lifecycle,
                DownstreamCallableBodyState {
                    definitions: &callable_definitions,
                    store: Arc::make_mut(&mut self.store),
                    hierarchy: &hierarchy,
                    dispatch: &mut dispatch,
                    callable_signatures: &mut callable_signatures,
                    callable_analyses: &mut callable_analyses,
                    callable_dispositions: &mut callable_dispositions,
                    budget,
                    cancel,
                },
            )?;
        }

        // 7. Retract stale inferred results for current unseeded direct cycles
        // before allowing the ordinary inferred-result refresh to consume
        // them. The returned SCC set is withheld from that generic loop.
        let (withheld_cycle_callables, cycle_reanalysis_roots) =
            retract_stale_unseeded_callable_result_cycles(&mut self.db, &callable_analyses, &mut callable_signatures, &mut dispatch)?;
        if !cycle_reanalysis_roots.is_empty() {
            let signature_roots = cycle_reanalysis_roots.iter().cloned().map(QueryKey::CallableSignature).collect::<BTreeSet<_>>();
            let downstream_bodies = self
                .db
                .index()
                .reverse_closure(signature_roots)
                .into_iter()
                .filter_map(|key| match key {
                    QueryKey::CallableBody(callable) if retained_sources.contains_key(callable.module()) && callable_analyses.contains_key(&callable) => {
                        Some(callable)
                    }
                    _ => None,
                })
                .collect::<BTreeSet<_>>();
            semantic_work_modules.extend(downstream_bodies.iter().map(|callable| callable.module().clone()));
            diagnostic_work_modules.extend(downstream_bodies.iter().map(|callable| callable.module().clone()));
            semantic_work_modules.extend(cycle_reanalysis_roots.iter().map(|callable| callable.module().clone()));
            diagnostic_work_modules.extend(cycle_reanalysis_roots.iter().map(|callable| callable.module().clone()));

            // The root bodies were already queried earlier in this session
            // revision, while their old inferred signatures were still
            // available. Invalidate only those body roots so their one
            // authorized reanalysis consumes the retracted dispatch result.
            // Incoming consumers remain cached and are refreshed by the
            // ordinary inferred-result propagation below.
            self.db.invalidate_roots(cycle_reanalysis_roots.iter().cloned().map(QueryKey::CallableBody));

            let reanalysis_signature_snapshot = callable_signatures.clone();
            let reanalysis_conformance_semantics = crate::trait_dispatch::ConformanceSemanticView {
                trait_dispatch: &trait_dispatch_index,
                conformance_index: &conformance_index,
                witness_plans: &conformance_witness_plans,
                trait_surfaces: &trait_surfaces,
                derived_conformance_rules: &derived_conformance_rules,
                callable_signatures: Some(&reanalysis_signature_snapshot),
            };
            let reanalysis_formal_inputs = FormalQueryInputs {
                sources: &retained_sources,
                source_resolution_input,
                linked_component_product,
                linked: &input.linked,
                topology: semantic_topology.as_ref(),
                import_products: &input.import_products,
                hierarchy: &hierarchy,
                base_resolver: &resolver,
                declarations: &declarations,
                type_aliases: &type_aliases,
                field_signatures: Some(&field_signatures),
                field_lifecycle: Some(&field_lifecycle),
                constructor_initializations: Some(&constructor_initializations),
                callable_signatures: Some(&reanalysis_signature_snapshot),
                data_semantics: Some(&data_semantics),
                enum_semantics: Some(&enum_semantics),
                associated_families: Some(&associated_surfaces_table),
                type_member_declarations: Some(&type_member_declarations_table),
                conformance_semantics: Some(&reanalysis_conformance_semantics),
                module_values: None,
                semantic_linked: input.semantic_linked.as_deref(),
            };
            for callable in &cycle_reanalysis_roots {
                let Some(unit) = retained_sources.get(callable.module()) else {
                    continue;
                };
                revalidate_downstream_callable_body(
                    &mut self.db,
                    callable,
                    unit,
                    &reanalysis_formal_inputs,
                    &default_field_lifecycle,
                    DownstreamCallableBodyState {
                        definitions: &callable_definitions,
                        store: Arc::make_mut(&mut self.store),
                        hierarchy: &hierarchy,
                        dispatch: &mut dispatch,
                        callable_signatures: &mut callable_signatures,
                        callable_analyses: &mut callable_analyses,
                        callable_dispositions: &mut callable_dispositions,
                        budget,
                        cancel,
                    },
                )?;
            }
        }

        // 8. Refine Callable Return Interfaces and Reach Fixed Point
        //
        // This is a specialized session-level loop: body inference and
        // interface publication iterate until return contracts stabilize.
        // During each iteration, we only run inference and update dispatch; we
        // do not run full checking or re-emit diagnostics.
        refresh_inferred_callable_results(InferredCallableRefreshInputs {
            db: &mut self.db,
            sources: &retained_sources,
            store: Arc::make_mut(&mut self.store),
            hierarchy: &hierarchy,
            resolver: &resolver,
            declarations: &declarations,
            dispatch: &mut dispatch,
            callable_signatures: &mut callable_signatures,
            constructor_initializations: &constructor_initializations,
            callable_analyses: &mut callable_analyses,
            previous_callable_analyses: previous_snapshot.as_ref().map(|snapshot| snapshot.callable_analyses.as_ref()),
            field_signatures: &field_signatures,
            field_lifecycle: &field_lifecycle,
            enum_semantics: &enum_semantics,
            data_semantics: &data_semantics,
            associated_surfaces: &associated_surfaces_table,
            topology: semantic_topology.as_ref(),
            callable_dispositions: &mut callable_dispositions,
            semantic_work_modules: &semantic_work_modules,
            previous_callable_signatures: previous_snapshot.as_ref().map(|snapshot| snapshot.callable_signatures.as_ref()),
            trait_default_callables: &trait_default_callables,
            conformance_semantics: &conformance_semantics,
            withheld_result_callables: &withheld_cycle_callables,
            budget,
            cancel,
        })?;

        refresh_unresolved_callable_result_cycle_diagnostics(&callable_analyses, &callable_signatures, &mut diags_by_module);

        // The first witness plans are provisional for body analysis. Once
        // canonical callable results stabilize, replace their derived
        // requirement projections before publishing the final semantic view.
        let _ = publish_conformance_witness_plans(
            &mut self.db,
            Arc::make_mut(&mut self.store),
            &hierarchy,
            &dispatch,
            &declarations,
            &conformance_index,
            &callable_signatures,
            &mut conformance_witness_plans,
            &conformance_witness_bodies,
            &conformance_witness_visibilities,
            &conformance_invalid_witnesses,
            &conformance_invalid_members,
            &conformance_associated_type_plans,
            &data_semantics,
            &trait_surfaces,
            &mut diags_by_module,
            true,
            &diagnostic_work_modules,
        )?;
        // Compare the settled plans to the prior published snapshot, not to
        // the provisional plans used to analyze bodies earlier in this pass.
        // Otherwise a plan that changed during the provisional publication
        // could appear unchanged here and leave retained dispatch clients stale.
        let changed_conformance_plans = previous_conformance_plan_fingerprints
            .iter()
            .filter_map(|(impl_id, previous_fingerprint)| {
                conformance_witness_plans
                    .get(impl_id)
                    .is_none_or(|plan| plan.fingerprint != *previous_fingerprint)
                    .then_some(impl_id.clone())
            })
            .chain(
                conformance_witness_plans
                    .keys()
                    .filter(|impl_id| !previous_conformance_plan_fingerprints.contains_key(*impl_id))
                    .cloned(),
            )
            .collect::<BTreeSet<_>>();
        diagnostic_work_modules.extend(changed_conformance_plans.iter().map(|impl_id| impl_id.module.clone()));
        let changed_plan_keys = changed_conformance_plans
            .iter()
            .cloned()
            .map(QueryKey::ConformanceDispatch)
            .chain(changed_bucket_keys)
            .chain(changed_generic_conformance_roots)
            .collect::<BTreeSet<_>>();
        let changed_plan_clients = self
            .db
            .index()
            .reverse_closure(changed_plan_keys)
            .into_iter()
            .filter_map(|key| match key {
                QueryKey::CallableBody(callable) if current_modules.contains(callable.module()) => Some(callable),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let affected_plan_clients = prior_conformance_clients
            .into_iter()
            .chain(changed_plan_clients)
            .filter(|callable| current_modules.contains(callable.module()))
            .collect::<BTreeSet<_>>();
        if !affected_plan_clients.is_empty() {
            let reanalysis_signature_snapshot = callable_signatures.clone();
            let reanalysis_conformance_semantics = crate::trait_dispatch::ConformanceSemanticView {
                trait_dispatch: &trait_dispatch_index,
                conformance_index: &conformance_index,
                witness_plans: &conformance_witness_plans,
                trait_surfaces: &trait_surfaces,
                derived_conformance_rules: &derived_conformance_rules,
                callable_signatures: Some(&reanalysis_signature_snapshot),
            };
            let formal_inputs = FormalQueryInputs {
                sources: &retained_sources,
                source_resolution_input,
                linked_component_product,
                linked: &input.linked,
                topology: semantic_topology.as_ref(),
                import_products: &input.import_products,
                hierarchy: &hierarchy,
                base_resolver: &resolver,
                declarations: &declarations,
                type_aliases: &type_aliases,
                field_signatures: Some(&field_signatures),
                field_lifecycle: Some(&field_lifecycle),
                constructor_initializations: Some(&constructor_initializations),
                callable_signatures: Some(&reanalysis_signature_snapshot),
                data_semantics: Some(&data_semantics),
                enum_semantics: Some(&enum_semantics),
                associated_families: Some(&associated_surfaces_table),
                type_member_declarations: Some(&type_member_declarations_table),
                conformance_semantics: Some(&reanalysis_conformance_semantics),
                module_values: None,
                semantic_linked: input.semantic_linked.as_deref(),
            };
            for callable in &affected_plan_clients {
                let Some(unit) = retained_sources.get(callable.module()) else {
                    continue;
                };
                semantic_work_modules.insert(callable.module().clone());
                diagnostic_work_modules.insert(callable.module().clone());
                revalidate_downstream_callable_body(
                    &mut self.db,
                    callable,
                    unit,
                    &formal_inputs,
                    &default_field_lifecycle,
                    DownstreamCallableBodyState {
                        definitions: &callable_definitions,
                        store: Arc::make_mut(&mut self.store),
                        hierarchy: &hierarchy,
                        dispatch: &mut dispatch,
                        callable_signatures: &mut callable_signatures,
                        callable_analyses: &mut callable_analyses,
                        callable_dispositions: &mut callable_dispositions,
                        budget,
                        cancel,
                    },
                )?;
            }
        }
        let conformance_semantics = crate::trait_dispatch::ConformanceSemanticView {
            trait_dispatch: &trait_dispatch_index,
            conformance_index: &conformance_index,
            witness_plans: &conformance_witness_plans,
            trait_surfaces: &trait_surfaces,
            derived_conformance_rules: &derived_conformance_rules,
            callable_signatures: Some(&callable_signatures),
        };

        for (module_id, shard) in &self.semantic_structure_shards {
            if !semantic_work_modules.contains(module_id) {
                continue;
            }
            let parsed_unit = &shard.source;
            let mut ctx = CheckingContext::new_with_dispatch_ref(
                Arc::make_mut(&mut self.store),
                &hierarchy,
                &resolver,
                &declarations,
                &dispatch,
                module_id.clone(),
            );
            ctx.set_caller_package(semantic_topology.owning_package(module_id));
            ctx.set_access_topology(semantic_topology.as_ref());
            ctx.attach_field_signatures(&field_signatures);
            ctx.attach_field_lifecycle(&field_lifecycle);
            ctx.attach_enum_semantics(&enum_semantics);
            ctx.attach_data_semantics(&data_semantics);
            ctx.attach_associated_families(&associated_surfaces_table);
            ctx.attach_conformance_semantics(&conformance_semantics);

            for stmt in &parsed_unit.program.statements {
                match stmt {
                    Statement::Class(class_def) => {
                        let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                        if blocked_declarations.contains(&decl_id) {
                            continue;
                        }
                        check_class_field_initializers(&mut ctx, class_def);
                    }
                    _ => {
                        check_statement(&mut ctx, stmt);
                    }
                }
            }

            let body_range = parsed_unit.program.preamble.range;
            let module_decl = DeclarationId::new(module_id.clone(), "<main>".into());
            let callable_id = CallableId::new(module_decl, Selector::getter("<main>").unwrap(), DispatchSide::Instance);
            let analysis = ctx.finalize(callable_id.clone(), body_range, crate::checker::CallableAnalysisStatus::Complete);
            callable_analyses.insert(callable_id, Arc::new(analysis));
        }

        // Module value exports are linker-owned public-name projections. Their
        // value facts live in the canonical source binding index and must be
        // available to ordinary top-level checking just like local bindings.
        let module_value_exports = input
            .linked
            .modules
            .iter()
            .flat_map(|(module, linked_module)| {
                linked_module.interface.exports.iter().filter_map(move |(name, export)| match &export.target {
                    LinkedExportTarget::Binding(symbol) => Some(((module.clone(), name.to_string()), symbol.clone())),
                    LinkedExportTarget::Module(_) => None,
                })
            })
            .collect::<BTreeMap<_, _>>();

        // The first ordinary pass establishes local module binding facts. Seed
        // a second pass from those canonical facts so imported values retain
        // their exact structural type and denotation during normal checking.
        let mut module_static_bindings = self
            .semantic_structure_shards
            .iter()
            .filter_map(|(module_id, shard)| {
                let module_decl = DeclarationId::new(module_id.clone(), "<main>".into());
                let main_callable = CallableId::new(module_decl, Selector::getter("<main>").unwrap(), DispatchSide::Instance);
                let analysis = callable_analyses.get(&main_callable)?;
                Some((
                    module_id.clone(),
                    Arc::new(crate::module_bindings::ModuleStaticBindingIndex::from_module_analysis(
                        module_id,
                        &shard.source.program,
                        analysis,
                    )),
                ))
            })
            .collect::<crate::parent_map::ParentMap<_, _>>();
        if let Some(semantic_linked) = input.semantic_linked.as_ref() {
            for (module_id, sem_mod) in &semantic_linked.modules {
                let imported_facts = sem_mod
                    .imports
                    .bindings
                    .values()
                    .filter_map(|import_binding| {
                        let phalcom_modules::linker::SymbolicImportTarget::Binding(ref symbol) = import_binding.target else {
                            return None;
                        };
                        let fact = module_static_bindings.get(&symbol.module)?.get(symbol)?.clone();
                        Some((import_binding.local_name.to_string(), fact))
                    })
                    .collect::<Vec<_>>();
                if let Some(bindings) = module_static_bindings.get_mut(module_id) {
                    for (local_name, fact) in imported_facts {
                        Arc::make_mut(bindings).insert_imported_binding(&local_name, fact);
                    }
                }
            }
        } else {
            for (module_id, linked_module) in &input.linked.modules {
                let imported_facts = linked_module
                    .bindings
                    .imports
                    .iter()
                    .filter_map(|(local_name, import_id)| {
                        let phalcom_modules::linker::LinkedReadSpec::Binding(symbol) = linked_module.linked_reads.get(import_id.0 as usize)? else {
                            return None;
                        };
                        let fact = module_static_bindings.get(&symbol.module)?.get(symbol)?.clone();
                        Some((local_name.to_string(), fact))
                    })
                    .collect::<Vec<_>>();
                if let Some(bindings) = module_static_bindings.get_mut(module_id) {
                    for (local_name, fact) in imported_facts {
                        Arc::make_mut(bindings).insert_imported_binding(&local_name, fact);
                    }
                }
            }
        }
        for (module_id, bindings) in module_static_bindings
            .iter()
            .map(|(module, bindings)| (module.clone(), bindings.clone()))
            .collect::<Vec<_>>()
        {
            let bindings = match crate::db::query::query_module_static_bindings(&mut self.db, bindings) {
                QueryOutcome::Ready(product) => product,
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            };
            module_static_bindings.insert(module_id, bindings);
        }

        for (module_id, shard) in &self.semantic_structure_shards {
            if !semantic_work_modules.contains(module_id) {
                continue;
            }
            let parsed_unit = &shard.source;
            let mut ctx = CheckingContext::new_with_dispatch_ref(
                Arc::make_mut(&mut self.store),
                &hierarchy,
                &resolver,
                &declarations,
                &dispatch,
                module_id.clone(),
            );
            ctx.set_caller_package(semantic_topology.owning_package(module_id));
            ctx.set_access_topology(semantic_topology.as_ref());
            ctx.attach_module_value_exports(&module_static_bindings, &module_value_exports);
            ctx.attach_field_signatures(&field_signatures);
            ctx.attach_field_lifecycle(&field_lifecycle);
            ctx.attach_enum_semantics(&enum_semantics);
            ctx.attach_data_semantics(&data_semantics);
            ctx.attach_associated_families(&associated_surfaces_table);
            ctx.attach_conformance_semantics(&conformance_semantics);

            if let Some(semantic_module) = input.semantic_linked.as_ref().and_then(|sl| sl.modules.get(module_id)) {
                let linked_module = input
                    .linked
                    .modules
                    .get(module_id)
                    .expect("semantic module import authority must include its executable linked layout");
                ctx.seed_symbolic_module_import_values(semantic_module, linked_module, &module_static_bindings);
            } else if let Some(linked_module) = input.linked.modules.get(module_id) {
                ctx.seed_linked_module_import_values(linked_module, &module_static_bindings);
            }

            for stmt in &parsed_unit.program.statements {
                match stmt {
                    Statement::Class(class_def) => {
                        let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                        if !blocked_declarations.contains(&decl_id) {
                            check_class_field_initializers(&mut ctx, class_def);
                        }
                    }
                    _ => {
                        check_statement(&mut ctx, stmt);
                    }
                }
            }

            let body_range = parsed_unit.program.preamble.range;
            let module_decl = DeclarationId::new(module_id.clone(), "<main>".into());
            let callable_id = CallableId::new(module_decl, Selector::getter("<main>").unwrap(), DispatchSide::Instance);
            let analysis = ctx.finalize(callable_id.clone(), body_range, crate::checker::CallableAnalysisStatus::Complete);
            callable_analyses.insert(callable_id, Arc::new(analysis));
        }

        // The top-level checker pass above publishes canonical checked module
        // bindings. Revalidate declaration bodies against that same value
        // environment so selective and qualified imports share one semantic
        // owner and their exact dependencies are recorded on CallableBody.
        let module_body_signature_snapshot = callable_signatures.clone();
        let module_body_conformance_semantics = crate::trait_dispatch::ConformanceSemanticView {
            trait_dispatch: &trait_dispatch_index,
            conformance_index: &conformance_index,
            witness_plans: &conformance_witness_plans,
            trait_surfaces: &trait_surfaces,
            derived_conformance_rules: &derived_conformance_rules,
            callable_signatures: Some(&module_body_signature_snapshot),
        };
        let module_body_inputs = FormalQueryInputs {
            sources: &retained_sources,
            source_resolution_input,
            linked_component_product,
            linked: &input.linked,
            topology: semantic_topology.as_ref(),
            import_products: &input.import_products,
            hierarchy: &hierarchy,
            base_resolver: &resolver,
            declarations: &declarations,
            type_aliases: &type_aliases,
            field_signatures: Some(&field_signatures),
            field_lifecycle: Some(&field_lifecycle),
            constructor_initializations: Some(&constructor_initializations),
            callable_signatures: Some(&module_body_signature_snapshot),
            data_semantics: Some(&data_semantics),
            enum_semantics: Some(&enum_semantics),
            associated_families: Some(&associated_surfaces_table),
            type_member_declarations: Some(&type_member_declarations_table),
            conformance_semantics: Some(&module_body_conformance_semantics),
            module_values: Some(crate::module_bindings::ModuleValueEnvironment {
                bindings: &module_static_bindings,
                exports: &module_value_exports,
            }),
            semantic_linked: input.semantic_linked.as_deref(),
        };
        let module_body_callables = callable_analyses.keys().cloned().collect::<BTreeSet<_>>();
        for callable in module_body_callables {
            let Some(unit) = retained_sources.get(callable.module()) else {
                continue;
            };
            let definition = self
                .db
                .product(&QueryKey::CallableDefinition(callable.clone()))
                .and_then(|product| product.as_callable_definition())
                .cloned();
            if callable_signatures.get(&callable).is_none() || source_body_for_callable(&callable, definition.as_deref(), unit).is_none() {
                continue;
            }
            semantic_work_modules.insert(callable.module().clone());
            diagnostic_work_modules.insert(callable.module().clone());
            revalidate_downstream_callable_body(
                &mut self.db,
                &callable,
                unit,
                &module_body_inputs,
                &default_field_lifecycle,
                DownstreamCallableBodyState {
                    definitions: &callable_definitions,
                    store: Arc::make_mut(&mut self.store),
                    hierarchy: &hierarchy,
                    dispatch: &mut dispatch,
                    callable_signatures: &mut callable_signatures,
                    callable_analyses: &mut callable_analyses,
                    callable_dispositions: &mut callable_dispositions,
                    budget,
                    cancel,
                },
            )?;
        }
        let conformance_semantics = crate::trait_dispatch::ConformanceSemanticView {
            trait_dispatch: &trait_dispatch_index,
            conformance_index: &conformance_index,
            witness_plans: &conformance_witness_plans,
            trait_surfaces: &trait_surfaces,
            derived_conformance_rules: &derived_conformance_rules,
            callable_signatures: Some(&callable_signatures),
        };

        // Body diagnostics are published from the final analyses, once per
        // rebuilt module. Intermediate inferred-result and conformance-plan
        // refreshes may replace those analyses, so accumulating diagnostics
        // during each pass can retain stale or duplicated diagnostics.
        for analysis in callable_analyses.values() {
            let module = analysis.callable.module();
            if !semantic_work_modules.contains(module)
                || !current_modules.contains(module)
                || analysis
                    .callable
                    .try_declaration_owner()
                    .is_some_and(|owner| owner.name.as_ref() != "<main>" && !current_declarations.contains(owner))
            {
                continue;
            }
            diags_by_module.entry(module.clone()).or_default().extend(analysis.diagnostics.iter().cloned());
        }

        // 8. Freeze and Publish Immutable Snapshot
        let mut diagnostics_map: BTreeMap<ModuleId, Vec<SemanticDiagnostic>> = BTreeMap::new();
        for (module_id, diags) in diags_by_module {
            if !diags.is_empty() {
                diagnostics_map.insert(module_id, diags);
            }
        }

        let mut unlinked_map = BTreeMap::new();
        let mut linked_map = BTreeMap::new();
        let mut resolved_imports_map = self.module_session.resolved_imports().clone();
        let mut sources_loc_map = BTreeMap::new();

        for (mod_id, unit) in &input.sources {
            if let Some(unlinked) = input.interfaces.get(mod_id) {
                unlinked_map.insert(mod_id.clone(), (**unlinked).clone());
            } else if let Ok(unlinked) = InterfaceBuilder::build(mod_id.clone(), unit.kind, &unit.program) {
                unlinked_map.insert(mod_id.clone(), unlinked);
            }
            if let Some(ref loc) = unit.source {
                sources_loc_map.insert(mod_id.clone(), loc.clone());
            }
        }

        if let Some(semantic_linked) = input.semantic_linked.as_ref() {
            for (mod_id, sem_mod) in &semantic_linked.modules {
                linked_map.insert(mod_id.clone(), sem_mod.interface.clone());
                for import_binding in sem_mod.imports.bindings.values() {
                    let local_name = import_binding.local_name.to_string();
                    match &import_binding.target {
                        phalcom_modules::linker::SymbolicImportTarget::Binding(sym) => {
                            resolved_imports_map.insert((mod_id.clone(), local_name), sym.module.clone());
                        }
                        phalcom_modules::linker::SymbolicImportTarget::Module(target_mod) => {
                            resolved_imports_map.insert((mod_id.clone(), local_name), target_mod.clone());
                        }
                    }
                }
            }
        } else {
            for (mod_id, linked_mod) in &input.linked.modules {
                linked_map.insert(mod_id.clone(), linked_mod.interface.clone());
                for (name, import_id) in &linked_mod.bindings.imports {
                    let Some(read_spec) = linked_mod.linked_reads.get(import_id.0 as usize) else {
                        continue;
                    };
                    match read_spec {
                        phalcom_modules::linker::LinkedReadSpec::Binding(sym) => {
                            resolved_imports_map.insert((mod_id.clone(), name.to_string()), sym.module.clone());
                        }
                        phalcom_modules::linker::LinkedReadSpec::Module(target_mod) => {
                            resolved_imports_map.insert((mod_id.clone(), name.to_string()), target_mod.clone());
                        }
                    }
                }
            }
        }

        let topology = semantic_topology.clone();
        let reverse_imports = input.reverse_imports.unwrap_or_else(|| {
            let mut rev: BTreeMap<ModuleId, BTreeSet<ModuleId>> = BTreeMap::new();
            for ((importer, _), target) in &resolved_imports_map {
                rev.entry(target.clone()).or_default().insert(importer.clone());
            }
            Arc::new(rev)
        });

        let module_products = Arc::new(crate::snapshot::ModuleQueryProducts::new(crate::snapshot::ModuleQueryProductsInputs {
            universe: input.linked.universe.clone(),
            unlinked: Arc::new(unlinked_map),
            linked: Arc::new(linked_map),
            import_products: input.import_products.clone(),
            resolved_imports: Arc::new(resolved_imports_map.clone()),
            sources: Arc::new(sources_loc_map),
            topology,
            reverse_imports,
        }));

        let mut source_index_rebuild_modules = changed_modules.clone();
        source_index_rebuild_modules.extend(
            callable_dispositions
                .iter()
                .filter_map(|(callable, disposition)| (*disposition == CallableRevisionDisposition::Recomputed).then_some(callable.module().clone())),
        );
        let mut source_index_analysis_callables = BTreeMap::new();
        for module in &source_index_rebuild_modules {
            let Some(shard) = self.semantic_structure_shards.get(module) else {
                continue;
            };
            let mut callables = BTreeSet::new();
            callables.extend(shard.callable_signature_fingerprints.keys().cloned());
            callables.extend(shard.callable_body_fingerprints.keys().cloned());
            callables.insert(CallableId::new(
                DeclarationId::new(module.clone(), "<main>".into()),
                Selector::getter("<main>").unwrap(),
                DispatchSide::Instance,
            ));
            source_index_analysis_callables.insert(module.clone(), callables);
        }
        let (mut source_index, presentation_sources) = build_source_semantic_index(SourceSemanticIndexInputs {
            sources: &input.sources,
            universe_source_provider: &self.universe_source_provider,
            include_workspace_source_semantics: self.product_request.source_semantics != SourceSemanticLevel::None,
            include_canonical_source_semantics: self.product_request.source_semantics == SourceSemanticLevel::Full && self.db.canonical_image().is_none(),
            include_canonical_presentation_sources: self.product_request.presentation == PresentationLevel::Full && self.db.canonical_image().is_none(),
            callable_analyses: &callable_analyses,
            callable_signatures: &callable_signatures,
            import_products: &input.import_products,
            require_canonical_import_products: input.require_canonical_import_products,
            linked: input.linked.as_ref(),
            type_resolver: &resolver,
            nominal_declarations: &known_declarations,
            previous: previous_snapshot.as_deref().map(|snapshot| snapshot.source_index.as_ref()),
            rebuild_modules: &source_index_rebuild_modules,
            retired_modules: &removed_modules,
            current_modules: &current_modules,
            analysis_callables: &source_index_analysis_callables,
            import_sites_by_module: &input.import_sites_by_module,
            trait_surfaces: &trait_surfaces,
        });
        // Presentation-only Universe source shards provide provenance and
        // navigation. They are deliberately not workspace query inputs.
        for module in &source_index_rebuild_modules {
            let Some(module_index) = source_index.module_arc(module) else {
                continue;
            };
            match query_source_structure(&mut self.db, module.clone(), module_index.clone()) {
                QueryOutcome::Ready(_) => {}
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            }
            for attachment in module_index.attachments.values() {
                match query_source_formal_attachment(&mut self.db, attachment.callable.clone(), attachment.clone()) {
                    QueryOutcome::Ready(_) => {}
                    QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                    QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                    QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                    QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                }
            }
        }
        let advisory = if self.product_request.advisory == AdvisoryLevel::Full {
            let mut advisory = build_advisory_workspace(AdvisoryWorkspaceInputs {
                sources: &retained_sources,
                source_index: &source_index,
                callable_analyses: &callable_analyses,
                store: &self.store,
                declarations: &declarations,
                callable_signatures: &callable_signatures,
                dispatch: &dispatch,
                hierarchy: &hierarchy,
                linked: input.linked.as_ref(),
                previous: self.last_snapshot.as_deref().map(|snapshot| snapshot.advisory.as_ref()),
                budget,
                cancel,
            });
            // Revalidate retained advisory summaries as well as summaries rebuilt
            // from the current formal analyses. Reused module shards still expose
            // their prior callable evidence, so their exact advisory dependencies
            // must be current before the shard can publish again.
            let mut advisory_query_callables = advisory.callables.as_ref().clone();
            if let Some(previous) = previous_snapshot.as_ref() {
                for (callable, summary) in previous.advisory.callables.iter() {
                    if current_modules.contains(callable.module()) {
                        advisory_query_callables.entry(callable.clone()).or_insert_with(|| summary.clone());
                    }
                }
            }
            let mut advisory_query_failed = None;
            for summary in advisory_query_callables.values() {
                if let QueryOutcome::Failed(error) = bootstrap_advisory_callable(&mut self.db, summary.clone()) {
                    advisory_query_failed = Some(error);
                    break;
                }
            }
            if advisory_query_failed.is_none() {
                for summary in advisory_query_callables.values() {
                    self.db.discard_for_recompute(&QueryKey::AdvisoryCallable(summary.callable.clone()));
                    if let QueryOutcome::Failed(error) = query_advisory_callable(&mut self.db, summary.clone()) {
                        advisory_query_failed = Some(error);
                        break;
                    }
                }
            }
            if advisory_query_failed.is_none() {
                for (module, module_product) in advisory.modules.iter() {
                    let callables = advisory
                        .callables
                        .keys()
                        .filter(|callable| callable.module() == module)
                        .cloned()
                        .chain(advisory_query_callables.keys().filter(|callable| callable.module() == module).cloned())
                        .collect::<BTreeSet<_>>();
                    let Some(source_structure) = source_index.module(module).cloned() else {
                        continue;
                    };
                    if let QueryOutcome::Failed(error) = query_advisory_module(&mut self.db, module_product.clone(), Arc::new(source_structure), callables) {
                        advisory_query_failed = Some(error);
                        break;
                    }
                }
            }
            if let Some(error) = advisory_query_failed {
                advisory = advisory.with_status(AdvisoryProductStatus::InternalFailure(error.into_boxed_str()));
            }
            advisory
        } else {
            if let Some(previous) = previous_snapshot.as_ref() {
                for module in previous.advisory.modules.keys() {
                    self.db.discard_for_recompute(&QueryKey::AdvisoryModule(module.clone()));
                }
                for callable in previous.advisory.callables.keys() {
                    self.db.discard_for_recompute(&QueryKey::AdvisoryCallable(callable.clone()));
                }
            }
            AdvisoryWorkspace::default()
        };
        for declaration in &blocked_declarations {
            declarations.remove(declaration);
        }
        retire_blocked_declaration_shell_queries(&mut self.db, &blocked_declarations, &type_aliases);
        callable_analyses.retain(|callable, _| {
            callable
                .try_declaration_owner()
                .is_none_or(|owner| owner.name.as_ref() == "<main>" || current_declarations.contains(owner))
        });
        let previous_formal = self.last_snapshot.as_ref().map(|s| s.formal_projection.as_ref());
        let mut formal_projection = previous_formal.cloned().unwrap_or_default();
        if previous_formal.is_none() {
            for (mod_id, _) in source_index.modules() {
                let callable_ids = source_index
                    .module(mod_id)
                    .map(|module| module.attachments.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                let mod_proj = FormalSemanticProjection::build_module_projection(mod_id, callable_ids, &callable_analyses, Some(&source_index));
                formal_projection.replace_module(mod_id.clone(), Arc::new(mod_proj));
            }
        } else {
            for mod_id in &source_index_rebuild_modules {
                if !current_modules.contains(mod_id) {
                    continue;
                }
                let callable_ids = source_index
                    .module(mod_id)
                    .map(|module| module.attachments.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                let mod_proj = FormalSemanticProjection::build_module_projection(mod_id, callable_ids, &callable_analyses, Some(&source_index));
                formal_projection.replace_module(mod_id.clone(), Arc::new(mod_proj));
            }
            // Plan A already computed the exact retirement set. Reusing it
            // avoids comparing every retained formal module on each update.
            for module in &removed_modules {
                formal_projection.retire_module(module);
            }
        }

        // Publish formal lifecycle statistics alongside the source shard
        // statistics. These counts are module deltas, not per-callable
        // attachment operations.
        let mut source_index_stats = source_index.stats();
        if previous_formal.is_none() {
            source_index_stats.formal_modules_rebuilt = source_index.len();
        } else {
            source_index_stats.formal_modules_rebuilt = source_index_rebuild_modules.iter().filter(|module| current_modules.contains(*module)).count();
            let formal_modules_rebuilt_retained = source_index_rebuild_modules
                .iter()
                .filter(|module| current_modules.contains(*module) && previous_formal.is_some_and(|projection| projection.module(module).is_some()))
                .count();
            let formal_modules_retired = removed_modules
                .iter()
                .filter(|module| previous_formal.is_some_and(|projection| projection.module(module).is_some()))
                .count();
            source_index_stats.formal_modules_retired = formal_modules_retired;
            source_index_stats.formal_modules_reused = previous_formal
                .map(|projection| {
                    projection
                        .module_count()
                        .saturating_sub(formal_modules_rebuilt_retained)
                        .saturating_sub(formal_modules_retired)
                })
                .unwrap_or_default();
        }
        source_index.set_stats(source_index_stats);

        // Static constant checking keeps ordinary checker facts only for
        // runtime bindings. Constant names get unknown types during dependency
        // discovery and checked types from published constant products during
        // the final initializer check.
        let mut module_static_binding_runtime_states = BTreeMap::new();
        for (module_id, catalog) in &module_static_binding_catalogs {
            let module_decl = DeclarationId::new(module_id.clone(), "<main>".into());
            let main_callable = CallableId::new(module_decl, Selector::getter("<main>").unwrap(), DispatchSide::Instance);
            let Some(analysis) = callable_analyses.get(&main_callable) else {
                continue;
            };
            for (symbol, seed) in catalog
                .iter()
                .filter(|(symbol, seed)| &symbol.module == module_id && seed.phase == crate::module_bindings::ModuleBindingPhase::Runtime)
            {
                if let Some(state) = analysis
                    .bindings
                    .values()
                    .find(|state| state.name.as_str() == symbol.name.as_ref() && state.range == seed.binding_range)
                {
                    module_static_binding_runtime_states.insert(symbol.clone(), state.clone());
                }
            }
        }

        module_static_bindings = self
            .semantic_structure_shards
            .iter()
            .filter_map(|(module_id, shard)| {
                let module_decl = DeclarationId::new(module_id.clone(), "<main>".into());
                let main_callable = CallableId::new(module_decl, Selector::getter("<main>").unwrap(), DispatchSide::Instance);
                let analysis = callable_analyses.get(&main_callable)?;
                Some((
                    module_id.clone(),
                    Arc::new(crate::module_bindings::ModuleStaticBindingIndex::from_module_analysis(
                        module_id,
                        &shard.source.program,
                        analysis,
                    )),
                ))
            })
            .collect::<crate::parent_map::ParentMap<_, _>>();
        if let Some(semantic_linked) = input.semantic_linked.as_ref() {
            for (module_id, sem_mod) in &semantic_linked.modules {
                let imported_facts = sem_mod
                    .imports
                    .bindings
                    .values()
                    .filter_map(|import_binding| {
                        let phalcom_modules::linker::SymbolicImportTarget::Binding(ref symbol) = import_binding.target else {
                            return None;
                        };
                        let fact = module_static_bindings.get(&symbol.module)?.get(symbol)?.clone();
                        Some((import_binding.local_name.to_string(), fact))
                    })
                    .collect::<Vec<_>>();
                if let Some(module_bindings) = module_static_bindings.get_mut(module_id) {
                    for (local_name, fact) in imported_facts {
                        Arc::make_mut(module_bindings).insert_imported_binding(&local_name, fact);
                    }
                }
            }
        } else {
            for (module_id, linked_module) in &input.linked.modules {
                let imported_facts = linked_module
                    .bindings
                    .imports
                    .iter()
                    .filter_map(|(local_name, import_id)| {
                        let Some(phalcom_modules::linker::LinkedReadSpec::Binding(symbol)) = linked_module.linked_reads.get(import_id.0 as usize) else {
                            return None;
                        };
                        let fact = module_static_bindings.get(&symbol.module)?.get(symbol)?.clone();
                        Some((local_name.to_string(), fact))
                    })
                    .collect::<Vec<_>>();
                if let Some(module_bindings) = module_static_bindings.get_mut(module_id) {
                    for (local_name, fact) in imported_facts {
                        Arc::make_mut(module_bindings).insert_imported_binding(&local_name, fact);
                    }
                }
            }
        }
        for (module_id, module_bindings) in module_static_bindings
            .iter()
            .map(|(module, bindings)| (module.clone(), bindings.clone()))
            .collect::<Vec<_>>()
        {
            let module_bindings = match crate::db::query::query_module_static_bindings(&mut self.db, module_bindings) {
                QueryOutcome::Ready(product) => product,
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            };
            module_static_bindings.insert(module_id, module_bindings);
        }

        let static_body_formal_inputs = FormalQueryInputs {
            sources: &retained_sources,
            source_resolution_input,
            linked_component_product,
            linked: &input.linked,
            topology: semantic_topology.as_ref(),
            import_products: &input.import_products,
            hierarchy: &hierarchy,
            base_resolver: &resolver,
            declarations: &declarations,
            type_aliases: &type_aliases,
            field_signatures: Some(&field_signatures),
            field_lifecycle: Some(&field_lifecycle),
            constructor_initializations: Some(&constructor_initializations),
            callable_signatures: Some(&callable_signatures),
            data_semantics: Some(&data_semantics),
            enum_semantics: Some(&enum_semantics),
            associated_families: Some(&associated_surfaces_table),
            type_member_declarations: Some(&type_member_declarations_table),
            conformance_semantics: Some(&conformance_semantics),
            module_values: None,
            semantic_linked: input.semantic_linked.as_deref(),
        };
        let static_definitions = static_definitions_table
            .iter()
            .filter(|(id, _)| current_modules.contains(&id.module))
            .map(|(id, product)| (id.clone(), product.clone()))
            .collect::<Vec<_>>();
        for (definition_id, definition_product) in static_definitions {
            let Some(module_bindings) = module_static_bindings.get(&definition_id.module) else {
                return Err(QueryOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency));
            };
            let definition = &definition_product.definition;
            let module_binding_fingerprint = module_bindings.product_fingerprint();
            let input_fingerprint = crate::db::fingerprint::checked_static_body_source_input_fingerprint(definition, Some(module_binding_fingerprint));
            let body_id = crate::static_definition::StaticBodyId(definition_id.clone());
            let checked_body = if let Some(cached) = crate::db::query::cached_checked_static_body(&mut self.db, &body_id, input_fingerprint) {
                cached
            } else {
                let mut type_parameters = HashMap::new();
                for parameter in definition.signature.parameters.iter() {
                    if let Some(type_parameter) = parameter.type_parameter_id {
                        let binding = type_level_binding_for_parameter(Arc::make_mut(&mut self.store), type_parameter);
                        type_parameters.insert(parameter.name.clone(), binding);
                    }
                }
                let scoped_resolver = crate::types::annotation::ScopedTypeResolver {
                    parent: &resolver,
                    type_parameters,
                };
                let mut context = CheckingContext::new_with_dispatch_ref(
                    Arc::make_mut(&mut self.store),
                    &hierarchy,
                    &scoped_resolver,
                    &declarations,
                    &dispatch,
                    definition_id.module.clone(),
                );
                context.set_caller_package(semantic_topology.owning_package(&definition_id.module));
                context.set_access_topology(semantic_topology.as_ref());
                context.attach_field_signatures(&field_signatures);
                context.attach_field_lifecycle(&field_lifecycle);
                context.attach_enum_semantics(&enum_semantics);
                context.attach_data_semantics(&data_semantics);
                context.attach_associated_families(&associated_surfaces_table);
                context.attach_conformance_semantics(&conformance_semantics);
                let checked = Arc::new(crate::checker::static_body::analyze_static_body_with_module_bindings(
                    context,
                    definition,
                    module_bindings,
                    &static_intrinsics,
                ));
                match crate::db::query::query_checked_static_body(&mut self.db, checked, &static_body_formal_inputs, Arc::make_mut(&mut self.store)) {
                    QueryOutcome::Ready(product) => product,
                    QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                    QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                    QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                    QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                }
            };
            diagnostics_map
                .entry(definition_id.module.clone())
                .or_default()
                .extend(checked_body.diagnostics.iter().cloned());
            let canonical_program = if self.product_request.canonical_static_programs {
                Some(
                    definition_product
                        .canonical_program
                        .as_ref()
                        .filter(|program| {
                            program.input_fingerprint == checked_body.input_fingerprint
                                && program.module_binding_fingerprint == checked_body.module_binding_fingerprint
                        })
                        .cloned()
                        .map(Ok)
                        .unwrap_or_else(|| {
                            crate::static_definition::CanonicalStaticProgram::from_checked_body(
                                &checked_body,
                                Arc::make_mut(&mut self.static_values),
                                &self.store,
                            )
                            .map(Arc::new)
                            .map_err(|error| QueryOutcome::Failed(format!("failed to publish canonical static program for {definition_id:?}: {error:?}")))
                        })?,
                )
            } else {
                definition_product.canonical_program.clone()
            };
            let mut composed = (*definition_product).clone();
            composed.canonical_program = canonical_program;
            composed.checked_body = Some(checked_body);
            static_definitions_table.insert(Arc::new(composed));
        }

        struct ConstantDeclInfo<'a> {
            id: crate::static_constant::StaticConstantId,
            module_id: ModuleId,
            declaration_range: SourceRange,
            initializer_range: Option<SourceRange>,
            initializer: &'a phalcom_ast::ast::Expr,
            annotation: Option<&'a phalcom_ast::ast::TypeAnnotation>,
            expected_type: Option<crate::types::id::TypeId>,
            has_computed_record_keys: bool,
            input_fingerprint: crate::db::key::InputFingerprint,
        }

        let mut constant_declarations: BTreeMap<crate::static_constant::StaticConstantId, ConstantDeclInfo<'_>> = BTreeMap::new();
        for module_id in &current_modules {
            let Some(catalog) = module_static_binding_catalogs.get(module_id) else {
                continue;
            };
            let Some(shard) = self.semantic_structure_shards.get(module_id) else {
                continue;
            };
            for (_name, seed) in catalog.visible_bindings() {
                if seed.phase == crate::module_bindings::ModuleBindingPhase::StaticConstant && &seed.symbol.module == module_id {
                    let id = crate::static_constant::StaticConstantId(seed.symbol.clone());
                    if let Some(binding) =
                        crate::module_bindings::find_constant_declaration(&shard.source.program, &seed.symbol, seed.declaration_range, seed.initializer_range)
                    {
                        if let Some(initializer) = &binding.value {
                            let has_computed_record_keys = binding
                                .annotation
                                .as_ref()
                                .is_some_and(|annotation| !crate::static_key::type_record_key_expressions(annotation, 0).is_empty());
                            let expected_type = (!has_computed_record_keys)
                                .then_some(binding.annotation.as_ref())
                                .flatten()
                                .and_then(|annotation| {
                                    if crate::checker::annotation_elaboration::annotation_contains_inference_hole(annotation) {
                                        return None;
                                    }
                                    let mut annotation_diagnostics = Vec::new();
                                    let knowledge = crate::types::annotation::resolve_type_annotation(
                                        Arc::make_mut(&mut self.store),
                                        &declarations,
                                        &resolver,
                                        &TypeFormationSite::module(module_id.clone()),
                                        annotation,
                                        &mut annotation_diagnostics,
                                    );
                                    let module_diagnostics = diagnostics_map.entry(module_id.clone()).or_default();
                                    for diagnostic in annotation_diagnostics {
                                        if !module_diagnostics.contains(&diagnostic) {
                                            module_diagnostics.push(diagnostic);
                                        }
                                    }
                                    knowledge.ty()
                                });
                            let annotation = binding.annotation.as_ref();
                            let input_fingerprint = crate::db::fingerprint::static_constant_input_fingerprint(&id, initializer, annotation, expected_type);
                            constant_declarations.insert(
                                id.clone(),
                                ConstantDeclInfo {
                                    id,
                                    module_id: module_id.clone(),
                                    declaration_range: seed.declaration_range,
                                    initializer_range: seed.initializer_range,
                                    initializer,
                                    annotation,
                                    expected_type,
                                    has_computed_record_keys,
                                    input_fingerprint,
                                },
                            );
                        }
                    }
                }
            }
        }

        let mut checked_constants: BTreeMap<crate::static_constant::StaticConstantId, Arc<crate::static_constant::CheckedStaticConstantInitializer>> =
            BTreeMap::new();
        let mut constant_graph: BTreeMap<crate::static_constant::StaticConstantId, BTreeSet<crate::static_constant::StaticConstantId>> = BTreeMap::new();
        // The graph remains the authority for reachability. Preserve the
        // narrower computed-key edge origin separately so a mixed formation
        // cycle is diagnosed causally, including when an unchanged producer is
        // reused from the previous revision.
        let mut constant_record_key_dependencies: BTreeMap<crate::static_constant::StaticConstantId, BTreeSet<crate::static_constant::StaticConstantId>> =
            BTreeMap::new();

        for (id, decl) in &constant_declarations {
            let key = crate::db::QueryKey::StaticConstant(id.clone());
            let is_input_matching = match self.db.query_state(&key) {
                Some(crate::db::state::QueryState::Ready { input_fingerprint, .. }) => *input_fingerprint == decl.input_fingerprint,
                _ => false,
            };
            if is_input_matching {
                if let Some(deps) = self.db.index().dependencies_of(&key) {
                    let const_deps = deps
                        .iter()
                        .filter_map(|edge| match &edge.dependency {
                            crate::db::QueryKey::StaticConstant(dep_id) => Some(dep_id.clone()),
                            _ => None,
                        })
                        .collect::<BTreeSet<_>>();
                    constant_graph.insert(id.clone(), const_deps);
                    let record_key_deps = self
                        .db
                        .product(&key)
                        .and_then(|product| product.as_static_constant())
                        .into_iter()
                        .flat_map(|product| {
                            product
                                .checked
                                .as_ref()
                                .map(|c| c.record_key_static_constant_reads.as_ref())
                                .unwrap_or(&[])
                                .iter()
                        })
                        .cloned()
                        .map(crate::static_constant::StaticConstantId)
                        .collect();
                    constant_record_key_dependencies.insert(id.clone(), record_key_deps);
                    continue;
                }
            }

            let Some(catalog) = module_static_binding_catalogs.get(&decl.module_id) else {
                continue;
            };
            let mut context = CheckingContext::new_with_dispatch_ref(
                Arc::make_mut(&mut self.store),
                &hierarchy,
                &resolver,
                &declarations,
                &dispatch,
                decl.module_id.clone(),
            );
            context.set_caller_package(semantic_topology.owning_package(&decl.module_id));
            context.set_access_topology(semantic_topology.as_ref());
            context.attach_field_signatures(&field_signatures);
            context.attach_field_lifecycle(&field_lifecycle);
            context.attach_enum_semantics(&enum_semantics);
            context.attach_data_semantics(&data_semantics);
            context.attach_associated_families(&associated_surfaces_table);
            context.attach_conformance_semantics(&conformance_semantics);

            let binding_states = static_constant_binding_states(catalog, &module_static_binding_runtime_states, None);
            let checked = Arc::new(crate::checker::static_body::analyze_static_constant_initializer(
                context,
                &decl.id,
                decl.declaration_range,
                decl.initializer_range.unwrap_or_else(|| decl.initializer.range()),
                decl.initializer,
                decl.annotation,
                decl.expected_type,
                catalog,
                &binding_states,
                &static_intrinsics,
            ));

            let mut const_deps = checked
                .static_constant_reads
                .iter()
                .map(|sym| crate::static_constant::StaticConstantId(sym.clone()))
                .collect::<BTreeSet<_>>();
            let mut record_key_deps = BTreeSet::new();
            if decl.has_computed_record_keys {
                let binding_states = static_constant_binding_states(catalog, &module_static_binding_runtime_states, None);
                for site in crate::static_key::type_record_key_expressions(decl.annotation.expect("computed key annotation"), 0) {
                    let mut context = CheckingContext::new_with_dispatch_ref(
                        Arc::make_mut(&mut self.store),
                        &hierarchy,
                        &resolver,
                        &declarations,
                        &dispatch,
                        decl.module_id.clone(),
                    );
                    context.set_caller_package(semantic_topology.owning_package(&decl.module_id));
                    context.set_access_topology(semantic_topology.as_ref());
                    context.attach_field_signatures(&field_signatures);
                    context.attach_field_lifecycle(&field_lifecycle);
                    context.attach_enum_semantics(&enum_semantics);
                    context.attach_data_semantics(&data_semantics);
                    context.attach_associated_families(&associated_surfaces_table);
                    context.attach_conformance_semantics(&conformance_semantics);
                    let key_expression =
                        crate::checker::static_body::analyze_static_expression(context, site.expression, None, catalog, &binding_states, &static_intrinsics);
                    const_deps.extend(
                        key_expression
                            .static_constant_reads
                            .iter()
                            .cloned()
                            .map(crate::static_constant::StaticConstantId),
                    );
                    record_key_deps.extend(
                        key_expression
                            .static_constant_reads
                            .iter()
                            .cloned()
                            .map(crate::static_constant::StaticConstantId),
                    );
                }
            }
            constant_graph.insert(id.clone(), const_deps);
            constant_record_key_dependencies.insert(id.clone(), record_key_deps);
            checked_constants.insert(id.clone(), checked);
        }

        struct ConstantTarjan<'a> {
            graph: &'a BTreeMap<crate::static_constant::StaticConstantId, BTreeSet<crate::static_constant::StaticConstantId>>,
            next: usize,
            indices: HashMap<crate::static_constant::StaticConstantId, usize>,
            lowlinks: HashMap<crate::static_constant::StaticConstantId, usize>,
            stack: Vec<crate::static_constant::StaticConstantId>,
            on_stack: BTreeSet<crate::static_constant::StaticConstantId>,
            components: Vec<BTreeSet<crate::static_constant::StaticConstantId>>,
        }

        fn visit_constant(node: crate::static_constant::StaticConstantId, state: &mut ConstantTarjan<'_>) {
            let index = state.next;
            state.next += 1;
            state.indices.insert(node.clone(), index);
            state.lowlinks.insert(node.clone(), index);
            state.stack.push(node.clone());
            state.on_stack.insert(node.clone());

            if let Some(dependencies) = state.graph.get(&node) {
                for dependency in dependencies {
                    if state.graph.contains_key(dependency) {
                        if !state.indices.contains_key(dependency) {
                            visit_constant(dependency.clone(), state);
                            let lowlink = state.lowlinks[&node].min(state.lowlinks[dependency]);
                            state.lowlinks.insert(node.clone(), lowlink);
                        } else if state.on_stack.contains(dependency) {
                            let lowlink = state.lowlinks[&node].min(state.indices[dependency]);
                            state.lowlinks.insert(node.clone(), lowlink);
                        }
                    }
                }
            }

            if state.lowlinks[&node] == state.indices[&node] {
                let mut component = BTreeSet::new();
                while let Some(member) = state.stack.pop() {
                    state.on_stack.remove(&member);
                    component.insert(member.clone());
                    if member == node {
                        break;
                    }
                }
                state.components.push(component);
            }
        }

        let mut tarjan_state = ConstantTarjan {
            graph: &constant_graph,
            next: 0,
            indices: HashMap::new(),
            lowlinks: HashMap::new(),
            stack: Vec::new(),
            on_stack: BTreeSet::new(),
            components: Vec::new(),
        };

        for id in constant_graph.keys() {
            if !tarjan_state.indices.contains_key(id) {
                visit_constant(id.clone(), &mut tarjan_state);
            }
        }

        for component in tarjan_state.components {
            let is_cycle = component.len() > 1 || component.iter().any(|node| constant_graph.get(node).is_some_and(|deps| deps.contains(node)));
            let includes_computed_key_edge = component.iter().any(|source| {
                constant_record_key_dependencies
                    .get(source)
                    .is_some_and(|dependencies| dependencies.iter().any(|target| component.contains(target)))
            });

            if is_cycle {
                for id in component {
                    let Some(decl) = constant_declarations.get(&id) else { continue };
                    let Some(catalog) = module_static_binding_catalogs.get(&decl.module_id) else {
                        continue;
                    };
                    let checked = match checked_constants.remove(&id) {
                        Some(c) => c,
                        None => {
                            let mut context = CheckingContext::new_with_dispatch_ref(
                                Arc::make_mut(&mut self.store),
                                &hierarchy,
                                &resolver,
                                &declarations,
                                &dispatch,
                                decl.module_id.clone(),
                            );
                            context.set_caller_package(semantic_topology.owning_package(&decl.module_id));
                            context.set_access_topology(semantic_topology.as_ref());
                            context.attach_field_signatures(&field_signatures);
                            context.attach_field_lifecycle(&field_lifecycle);
                            context.attach_enum_semantics(&enum_semantics);
                            context.attach_data_semantics(&data_semantics);
                            context.attach_associated_families(&associated_surfaces_table);
                            context.attach_conformance_semantics(&conformance_semantics);
                            let binding_states = static_constant_binding_states(catalog, &module_static_binding_runtime_states, None);
                            Arc::new(crate::checker::static_body::analyze_static_constant_initializer(
                                context,
                                &decl.id,
                                decl.declaration_range,
                                decl.initializer_range.unwrap_or_else(|| decl.initializer.range()),
                                decl.initializer,
                                decl.annotation,
                                decl.expected_type,
                                catalog,
                                &binding_states,
                                &static_intrinsics,
                            ))
                        }
                    };

                    let cycle_diag = SemanticDiagnostic::error_in(
                        decl.module_id.clone(),
                        DiagnosticCode::StaticConstantCycle,
                        if includes_computed_key_edge {
                            format!(
                                "static constant `{}` participates in a static-formation cycle through a computed Record key",
                                id.name()
                            )
                        } else {
                            format!("static constant `{}` participates in a cycle", id.name())
                        },
                        decl.declaration_range,
                    );
                    let mut diags = checked.diagnostics.to_vec();
                    diags.push(cycle_diag);
                    let authority = crate::static_constant::StaticConstantAuthority::from_checked(&checked);

                    let product = Arc::new(crate::static_constant::StaticConstantProduct {
                        id: id.clone(),
                        declaration_range: decl.declaration_range,
                        initializer_range: decl.initializer_range.unwrap_or_else(|| decl.initializer.range()),
                        checked: Some(checked),
                        authority: Some(authority),
                        outcome: crate::static_eval::StaticEvalOutcome::Recursive(crate::static_eval::StaticCycle::empty()),
                        residuals: None,
                        value: None,
                        lkg_value: previous_snapshot
                            .as_ref()
                            .and_then(|prev| prev.static_constant(&id))
                            .and_then(|p| p.value.or(p.lkg_value)),
                        diagnostics: Arc::from(diags.into_boxed_slice()),
                        imported: None,
                    });

                    let published = match crate::db::query::query_static_constant(
                        &mut self.db,
                        product,
                        &self.static_values,
                        &static_body_formal_inputs,
                        Arc::make_mut(&mut self.store),
                    ) {
                        QueryOutcome::Ready(p) => p,
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    };
                    diagnostics_map
                        .entry(decl.module_id.clone())
                        .or_default()
                        .extend(published.diagnostics.iter().cloned());
                    static_constants_table.insert(published);
                }
            } else {
                for id in component {
                    let Some(decl) = constant_declarations.get(&id) else { continue };

                    if !decl.has_computed_record_keys && !checked_constants.contains_key(&id) {
                        if let Some(cached) = crate::db::query::cached_static_constant(&mut self.db, &id, decl.input_fingerprint) {
                            diagnostics_map
                                .entry(decl.module_id.clone())
                                .or_default()
                                .extend(cached.diagnostics.iter().cloned());
                            static_constants_table.insert(cached);
                            continue;
                        }
                    }

                    let Some(catalog) = module_static_binding_catalogs.get(&decl.module_id) else {
                        continue;
                    };
                    // The first check discovers exact constant reads without
                    // granting unresolved constants a provisional type. Recheck
                    // the initializer now that dependency products are current,
                    // and seed constant names only from their published checked
                    // result types.
                    checked_constants.remove(&id);
                    let binding_states = static_constant_binding_states(catalog, &module_static_binding_runtime_states, Some(&static_constants_table));
                    let (expected, key_expressions) = if decl.has_computed_record_keys {
                        let annotation = decl.annotation.expect("computed key annotation");
                        let owner = QueryKey::StaticConstant(id.clone());
                        let mut projections = crate::static_key::ProjectedRecordKeySet::new(owner);
                        for site in crate::static_key::type_record_key_expressions(annotation, 0) {
                            let mut key_context = CheckingContext::new_with_dispatch_ref(
                                Arc::make_mut(&mut self.store),
                                &hierarchy,
                                &resolver,
                                &declarations,
                                &dispatch,
                                decl.module_id.clone(),
                            );
                            key_context.set_caller_package(semantic_topology.owning_package(&decl.module_id));
                            key_context.set_access_topology(semantic_topology.as_ref());
                            key_context.attach_field_signatures(&field_signatures);
                            key_context.attach_field_lifecycle(&field_lifecycle);
                            key_context.attach_enum_semantics(&enum_semantics);
                            key_context.attach_data_semantics(&data_semantics);
                            key_context.attach_associated_families(&associated_surfaces_table);
                            key_context.attach_conformance_semantics(&conformance_semantics);
                            let checked_key = Arc::new(crate::checker::static_body::analyze_static_expression(
                                key_context,
                                site.expression,
                                None,
                                catalog,
                                &binding_states,
                                &static_intrinsics,
                            ));
                            let mut key_budget = budget;
                            let projection = match crate::static_key::project_checked_static_key(
                                &checked_key,
                                &static_constants_table,
                                &static_definitions_table,
                                &self.store,
                                &data_semantics,
                                &enum_semantics,
                                Arc::make_mut(&mut self.static_values),
                                &mut key_budget,
                                cancel,
                            ) {
                                QueryOutcome::Ready(projection) => projection,
                                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                            };
                            diagnostics_map
                                .entry(decl.module_id.clone())
                                .or_default()
                                .extend(checked_key.diagnostics.iter().cloned());
                            if projections.insert(site.syntax, site.ordinal, checked_key, projection).is_err() {
                                return Err(QueryOutcome::Failed("duplicate computed Record key identity".to_string()));
                            }
                        }
                        let key_expressions = projections.facts().map(|fact| fact.checked.clone()).collect::<Vec<_>>();
                        let projected_resolver = crate::types::annotation::ProjectedRecordKeyResolver {
                            parent: &resolver,
                            projections: &projections,
                        };
                        let mut annotation_diagnostics = Vec::new();
                        let expected_type = crate::types::annotation::resolve_type_annotation(
                            Arc::make_mut(&mut self.store),
                            &declarations,
                            &projected_resolver,
                            &TypeFormationSite::module(decl.module_id.clone()),
                            annotation,
                            &mut annotation_diagnostics,
                        )
                        .ty();
                        diagnostics_map.entry(decl.module_id.clone()).or_default().extend(annotation_diagnostics);
                        (
                            expected_type.map_or(crate::checker::expected::ExpectedType::None, |ty| {
                                crate::checker::expected::ExpectedType::proper_from(ty, crate::checker::expected::ExpectationOrigin::DeclarationContract)
                            }),
                            key_expressions,
                        )
                    } else {
                        // Partial annotations are elaborated below in the checker
                        // that owns initializer constraints and publication.
                        let expected = decl.expected_type.map_or(crate::checker::expected::ExpectedType::None, |ty| {
                            crate::checker::expected::ExpectedType::proper_from(ty, crate::checker::expected::ExpectationOrigin::DeclarationContract)
                        });
                        (expected, Vec::new())
                    };
                    let mut context = CheckingContext::new_with_dispatch_ref(
                        Arc::make_mut(&mut self.store),
                        &hierarchy,
                        &resolver,
                        &declarations,
                        &dispatch,
                        decl.module_id.clone(),
                    );
                    context.set_caller_package(semantic_topology.owning_package(&decl.module_id));
                    context.set_access_topology(semantic_topology.as_ref());
                    context.attach_field_signatures(&field_signatures);
                    context.attach_field_lifecycle(&field_lifecycle);
                    context.attach_enum_semantics(&enum_semantics);
                    context.attach_data_semantics(&data_semantics);
                    context.attach_associated_families(&associated_surfaces_table);
                    context.attach_conformance_semantics(&conformance_semantics);
                    let diagnostic_start = context.diagnostics.len();
                    let expected = if let Some(annotation) = decl.annotation.filter(|annotation| {
                        !decl.has_computed_record_keys && crate::checker::annotation_elaboration::annotation_contains_inference_hole(annotation)
                    }) {
                        let (elaboration, _) = context.elaborate_type_annotation(&resolver, annotation);
                        match elaboration {
                            Some(crate::checker::annotation_elaboration::AnnotationElaboration::Inference { context: inf_ctx, term, .. }) => {
                                crate::checker::expected::ExpectedType::inference_from(
                                    inf_ctx,
                                    term,
                                    crate::checker::expected::ExpectationOrigin::DeclarationContract,
                                )
                            }
                            Some(crate::checker::annotation_elaboration::AnnotationElaboration::Canonical(ty)) => {
                                crate::checker::expected::ExpectedType::proper_from(ty, crate::checker::expected::ExpectationOrigin::DeclarationContract)
                            }
                            None => crate::checker::expected::ExpectedType::None,
                        }
                    } else {
                        expected
                    };
                    let checked = crate::checker::static_body::analyze_static_constant_initializer_with_expected_and_diagnostic_start(
                        context,
                        &decl.id,
                        decl.declaration_range,
                        decl.initializer_range.unwrap_or_else(|| decl.initializer.range()),
                        decl.initializer,
                        decl.annotation,
                        expected,
                        catalog,
                        &binding_states,
                        &static_intrinsics,
                        diagnostic_start,
                    );
                    let mut checked = checked;
                    if !key_expressions.is_empty() {
                        let mut dependencies = checked.dependencies.to_vec();
                        let mut reads = checked.static_constant_reads.to_vec();
                        let mut record_key_reads = Vec::new();
                        for key_expression in key_expressions {
                            dependencies.extend(key_expression.dependencies.iter().cloned());
                            reads.extend(key_expression.static_constant_reads.iter().cloned());
                            record_key_reads.extend(key_expression.static_constant_reads.iter().cloned());
                        }
                        dependencies.sort();
                        dependencies.dedup();
                        reads.sort();
                        reads.dedup();
                        record_key_reads.sort();
                        record_key_reads.dedup();
                        checked.expression.dependencies = Arc::from(dependencies.into_boxed_slice());
                        checked.expression.static_constant_reads = Arc::from(reads.into_boxed_slice());
                        checked.record_key_static_constant_reads = Arc::from(record_key_reads.into_boxed_slice());
                    }
                    let checked = Arc::new(checked);

                    let mut eval_budget = budget;
                    let type_store = Arc::make_mut(&mut self.store);
                    let mut evaluator =
                        crate::static_eval::StaticEvaluator::for_constant(&checked, Arc::make_mut(&mut self.static_values), &mut eval_budget, cancel)
                            .with_static_constants(&static_constants_table)
                            .with_static_definitions(&static_definitions_table)
                            .with_semantic_tables_mut(type_store, &data_semantics, &enum_semantics);

                    let outcome = evaluator.evaluate();
                    let residuals = if matches!(outcome, crate::static_eval::StaticEvalOutcome::Symbolic(_)) {
                        evaluator.take_residuals().map(Arc::new)
                    } else {
                        None
                    };
                    drop(evaluator);
                    #[cfg(test)]
                    self.fail_at_test_publication_point(SemanticPublicationFailpoint::AfterStaticValueMutation)?;
                    let value = match &outcome {
                        crate::static_eval::StaticEvalOutcome::Reduced(val_ref) => Some(*val_ref),
                        _ => None,
                    };
                    let lkg_value = if value.is_some() {
                        value
                    } else {
                        previous_snapshot
                            .as_ref()
                            .and_then(|prev| prev.static_constant(&id))
                            .and_then(|p| p.value.or(p.lkg_value))
                    };

                    let product = Arc::new(crate::static_constant::StaticConstantProduct {
                        id: id.clone(),
                        declaration_range: decl.declaration_range,
                        initializer_range: decl.initializer_range.unwrap_or_else(|| decl.initializer.range()),
                        checked: Some(checked.clone()),
                        authority: Some(crate::static_constant::StaticConstantAuthority::from_checked(&checked)),
                        outcome,
                        residuals,
                        value,
                        lkg_value,
                        diagnostics: checked.diagnostics.clone(),
                        imported: None,
                    });

                    let published = match crate::db::query::query_static_constant(
                        &mut self.db,
                        product,
                        &self.static_values,
                        &static_body_formal_inputs,
                        Arc::make_mut(&mut self.store),
                    ) {
                        QueryOutcome::Ready(p) => p,
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    };
                    diagnostics_map
                        .entry(decl.module_id.clone())
                        .or_default()
                        .extend(published.diagnostics.iter().cloned());
                    static_constants_table.insert(published);
                }
            }
        }

        let attribute_formal_inputs = FormalQueryInputs {
            sources: &retained_sources,
            source_resolution_input,
            linked_component_product,
            linked: &input.linked,
            topology: semantic_topology.as_ref(),
            import_products: &input.import_products,
            hierarchy: &hierarchy,
            base_resolver: &resolver,
            declarations: &declarations,
            type_aliases: &type_aliases,
            field_signatures: Some(&field_signatures),
            field_lifecycle: Some(&field_lifecycle),
            constructor_initializations: Some(&constructor_initializations),
            callable_signatures: Some(&callable_signatures),
            data_semantics: Some(&data_semantics),
            enum_semantics: Some(&enum_semantics),
            associated_families: Some(&associated_surfaces_table),
            type_member_declarations: Some(&type_member_declarations_table),
            conformance_semantics: Some(&conformance_semantics),
            module_values: Some(crate::module_bindings::ModuleValueEnvironment {
                bindings: &module_static_bindings,
                exports: &module_value_exports,
            }),
            semantic_linked: input.semantic_linked.as_deref(),
        };
        let mut attribute_analyses = BTreeMap::new();
        if input.semantic_linked.is_some() {
            for (module_id, parsed) in &retained_sources {
                for site in crate::attribute_analysis::collect_supported_attribute_argument_sites(module_id, &parsed.program) {
                    let callable = site.callable_context.clone().unwrap_or_else(|| {
                        CallableId::new(
                            DeclarationId::new(module_id.clone(), "<main>".into()),
                            Selector::getter("<main>").expect("module initializer selector"),
                            DispatchSide::Instance,
                        )
                    });
                    let declared_signature = match site.callable_context.as_ref() {
                        Some(callable) => {
                            let Some(signature) = callable_signatures.get(callable) else {
                                continue;
                            };
                            Some((callable, signature))
                        }
                        None => None,
                    };
                    let owner_generic_signature = site
                        .callable_context
                        .as_ref()
                        .and_then(CallableId::try_declaration_owner)
                        .and_then(|owner| declarations.generic_signature(owner));
                    let body = [Statement::Expr {
                        expr: site.expression.clone(),
                        range: site.expression.range(),
                    }];
                    let body_range = site.expression.range();
                    let query = crate::db::CallableBodyQuery {
                        callable,
                        body: &body,
                        body_range,
                        declared_signature,
                        owner_generic_signature,
                        receiver_value_override: None,
                        instance_self_override: None,
                        trait_surface: None,
                        store: Arc::make_mut(&mut self.store),
                        hierarchy: &hierarchy,
                        resolver: &resolver,
                        declarations: &declarations,
                        dispatch: &dispatch,
                        module: module_id.clone(),
                        budget,
                        cancel,
                        formal_inputs: Some(&attribute_formal_inputs),
                    };
                    match crate::db::query_attribute_argument(&mut self.db, site.id.clone(), query, site.id.context) {
                        QueryOutcome::Ready(product) => {
                            diagnostics_map
                                .entry(module_id.clone())
                                .or_default()
                                .extend(product.analysis.diagnostics.iter().cloned());
                            attribute_analyses.insert(site.id, product);
                        }
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Blocked(_) => {}
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    }
                }
            }
        }

        let mut semantic_diagnostic_contributions = BTreeMap::new();
        for (module_id, diagnostics) in &diagnostics_map {
            let mut semantic_diagnostics = diagnostics.clone();
            if let Some(module_diagnostics) = current_module_diagnostics.get(module_id) {
                for module_diagnostic in semantic_diagnostics_from_module_diagnostics(module_diagnostics) {
                    if let Some(index) = semantic_diagnostics.iter().position(|diagnostic| diagnostic == &module_diagnostic) {
                        semantic_diagnostics.remove(index);
                    }
                }
            }
            semantic_diagnostics.retain(|diagnostic| diagnostic.code != DiagnosticCode::ImplConformanceIncomplete);
            if !semantic_diagnostics.is_empty() {
                semantic_diagnostic_contributions.insert(module_id.clone(), Arc::from(semantic_diagnostics.into_boxed_slice()));
            }
        }
        let diagnostics_map = diagnostics_map
            .into_iter()
            .map(|(module, diagnostics)| (module, Arc::from(diagnostics.into_boxed_slice())))
            .collect();
        let trait_dispatch_index = crate::trait_dispatch::TraitDispatchIndex::build(&conformance_index, &trait_surfaces);
        #[cfg(test)]
        self.apply_test_candidate_type_parameter();
        let mut snapshot_obj = SemanticSnapshot::new_with_callable_analyses(
            self.workspace,
            self.db.revision(),
            input.generation,
            self.store.clone(),
            Arc::new(retained_sources.clone()),
            dispatch.surfaces().clone(),
            Arc::new(dispatch),
            Arc::new(callable_signatures),
            Arc::new(declarations),
            Arc::new(hierarchy),
            Arc::new(diagnostics_map),
            Arc::new(semantic_graph),
            callable_analyses,
        );

        snapshot_obj = snapshot_obj.with_field_signatures(Arc::new(field_signatures));
        snapshot_obj = snapshot_obj.with_constructor_initializations(Arc::new(constructor_initializations));
        snapshot_obj = snapshot_obj.with_callable_definitions(Arc::new(callable_definitions));
        snapshot_obj = snapshot_obj.with_conformance_index(Arc::new(conformance_index));
        snapshot_obj = snapshot_obj.with_conformance_witness_plans(Arc::new(conformance_witness_plans));
        snapshot_obj = snapshot_obj.with_derived_conformance_rules(Arc::new(derived_conformance_rules));
        snapshot_obj = snapshot_obj.with_trait_dispatch_index(Arc::new(trait_dispatch_index));
        snapshot_obj = snapshot_obj.with_presentation_sources(Arc::new(presentation_sources));
        snapshot_obj = snapshot_obj.with_source_index(Arc::new(source_index));
        snapshot_obj = snapshot_obj.with_formal_projection(Arc::new(formal_projection));
        snapshot_obj = snapshot_obj.with_data_semantics(Arc::new(data_semantics));
        snapshot_obj = snapshot_obj.with_enum_semantics(Arc::new(enum_semantics));
        snapshot_obj = snapshot_obj.with_enum_requirements(Arc::new(enum_requirements_table));
        snapshot_obj = snapshot_obj.with_associated_surfaces(Arc::new(associated_surfaces_table));
        snapshot_obj = snapshot_obj.with_type_member_declarations(Arc::new(type_member_declarations_table));
        snapshot_obj = snapshot_obj.with_associated_static_domains(Arc::new(associated_static_domains_table));
        snapshot_obj = snapshot_obj.with_static_case_catalogs(Arc::new(crate::ordered_map::OrderedMap::from(static_case_catalogs)));
        snapshot_obj = snapshot_obj.with_static_case_signatures(Arc::new(static_case_signatures_table));
        snapshot_obj = snapshot_obj.with_type_aliases(Arc::new(type_aliases));
        snapshot_obj = snapshot_obj.with_static_definitions(Arc::new(static_definitions_table));
        snapshot_obj = snapshot_obj.with_static_constants(Arc::new(static_constants_table));
        snapshot_obj = snapshot_obj.with_static_values(self.static_values.clone());
        snapshot_obj = snapshot_obj.with_module_static_bindings(Arc::new(module_static_bindings));
        snapshot_obj = snapshot_obj.with_trait_headers(Arc::new(trait_headers));
        snapshot_obj = snapshot_obj.with_trait_surfaces(Arc::new(trait_surfaces));
        snapshot_obj = snapshot_obj.with_semantic_structure_shards(Arc::new(self.semantic_structure_shards.clone()));
        snapshot_obj.advisory = Arc::new(advisory);
        snapshot_obj.module_products = module_products;
        if !input.blocked_modules.is_empty() {
            snapshot_obj.status = crate::snapshot::SnapshotStatus::Partial {
                blocked_modules: input.blocked_modules.len() as u32,
            };
        }
        if let Some(semantic_linked) = input.semantic_linked.as_ref() {
            let is_complete = input.blocked_modules.is_empty();
            let (import_phase_products, generated_runtime_requirements) =
                crate::phase_evidence::compute_module_import_phase_products_with_attribute_analyses_and_mode(
                    semantic_linked,
                    &input.sources,
                    snapshot_obj.source_index.as_ref(),
                    snapshot_obj.callable_analyses.as_ref(),
                    &attribute_analyses,
                    Some(snapshot_obj.store.as_ref()),
                    Some(&snapshot_obj.static_constants),
                    is_complete,
                    input.attribute_execution_mode,
                );
            snapshot_obj.import_phase_products = Arc::new(import_phase_products.into());
            snapshot_obj.generated_runtime_requirements = Arc::from(generated_runtime_requirements);
        }
        let snapshot = Arc::new(snapshot_obj);

        // These are observational counters. The current snapshot builder
        // still freezes several BTreeMap roots wholesale; recording that fact
        // keeps C0 honest while the remaining persistent-root migration is
        // completed. TypeStore and advisory flattening are already shallow at
        // this seam.
        stats.snapshot_full_map_materializations += 8;
        stats.snapshot_roots_replaced += 8;
        stats.type_store_snapshot_deep_copies = 0;
        stats.advisory_flattening_work = 0;
        stats.diagnostic_frontier_work = diagnostic_work_modules.len();

        // Query evaluation records computation and validation events directly.
        // Consume only this revision's event set instead of walking every cached
        // product and every dependency edge after each update.
        let mut query_work = BTreeMap::new();
        for key in self.db.revision_revalidated_keys() {
            query_work.entry(key.clone()).or_insert(false);
        }
        for key in self.db.revision_recomputed_keys() {
            query_work.insert(key.clone(), true);
        }
        for (key, recomputed) in query_work {
            if recomputed {
                stats.query_products_recomputed += 1;
            } else {
                stats.query_products_revalidated += 1;
            }

            let exact_name = matches!(key, QueryKey::LinkedName(_, _) | QueryKey::PublicExport(_, _));
            if exact_name {
                if recomputed {
                    stats.exact_name_products_recomputed += 1;
                } else {
                    stats.exact_name_products_reused += 1;
                }
            }

            if self.db.index().dependencies_of(&key).is_some_and(|edges| !edges.is_empty()) {
                if recomputed {
                    stats.semantic_dependents_recomputed += 1;
                } else {
                    stats.semantic_dependents_reused += 1;
                }
            }

            match &key {
                QueryKey::DeclarationShell(declaration) => {
                    if snapshot.type_aliases.contains_key(declaration) {
                        if recomputed {
                            stats.alias_regions_recomputed += 1;
                        } else {
                            stats.alias_regions_reused += 1;
                        }
                    } else if snapshot.sources.contains_key(&declaration.module) {
                        if recomputed {
                            stats.declaration_products_recomputed += 1;
                        } else {
                            stats.declaration_products_reused += 1;
                        }
                    }
                }
                QueryKey::HierarchyEdge(declaration) if snapshot.sources.contains_key(&declaration.module) => {
                    if recomputed {
                        stats.hierarchy_edges_recomputed += 1;
                    } else {
                        stats.hierarchy_edges_reused += 1;
                    }
                }
                QueryKey::CallableSignature(callable) if snapshot.sources.contains_key(callable.module()) => {
                    if recomputed {
                        stats.callable_signatures_recomputed += 1;
                    } else {
                        stats.callable_signatures_reused += 1;
                    }
                }
                QueryKey::FieldSignature(field) if snapshot.sources.contains_key(&field.owner.module) => {
                    if recomputed {
                        stats.field_signatures_recomputed += 1;
                    } else {
                        stats.field_signatures_reused += 1;
                    }
                }
                QueryKey::CallableBody(callable) if snapshot.sources.contains_key(callable.module()) => {
                    if recomputed {
                        stats.callable_bodies_recomputed += 1;
                    } else {
                        stats.callable_bodies_reused += 1;
                    }
                }
                _ => {}
            }
        }

        let mut diagnostics_changed = if previous_snapshot.is_none() {
            current_modules.clone()
        } else {
            BTreeSet::new()
        };
        let previous_diagnostics = previous_snapshot.as_ref().map(|snapshot| snapshot.diagnostics.as_ref());
        for module in &diagnostic_work_modules {
            let before = previous_diagnostics.and_then(|diagnostics| diagnostics.get(module));
            let after = snapshot.diagnostics.get(module);
            if before != after {
                diagnostics_changed.insert(module.clone());
            }
        }
        // A source or dependency update can leave the diagnostic value equal
        // while still invalidating the publication's source/version pairing.
        // Keep the exact semantic diagnostic frontier in the effects so the
        // LSP can republish only affected open documents and never reuse a
        // stale diagnostic publication for the new source revision.
        diagnostics_changed.extend(diagnostic_work_modules.iter().cloned());

        // Module-session publications carry the exact graph/import delta. Do
        // not infer it by comparing retained workspace-sized import maps.
        let module_graph_changed = previous_snapshot.as_deref().is_none_or(|previous| {
            input.module_graph_changed
                || module_delta.as_ref().is_some_and(|delta| delta.module_graph_changed)
                || previous.semantic_graph != snapshot.semantic_graph
                || !previous.module_products.linked_module_keys().eq(snapshot.module_products.linked_module_keys())
        });
        let declaration_index_changed = previous_snapshot.as_deref().is_none_or(|previous| {
            let previous_callables = previous.callable_signatures.iter().map(|(callable, _)| callable).collect::<BTreeSet<_>>();
            let current_callables = snapshot.callable_signatures.iter().map(|(callable, _)| callable).collect::<BTreeSet<_>>();
            let previous_fields = previous.field_signatures.iter().map(|(field, _)| field).collect::<BTreeSet<_>>();
            let current_fields = snapshot.field_signatures.iter().map(|(field, _)| field).collect::<BTreeSet<_>>();
            !previous.surfaces.keys().eq(snapshot.surfaces.keys()) || previous_callables != current_callables || previous_fields != current_fields
        });
        let effects = SemanticPublicationEffects {
            diagnostics_changed,
            source_index_changed: changed_modules.clone(),
            formal_changed: changed_modules.clone(),
            advisory_changed: if self.product_request.advisory == AdvisoryLevel::Full {
                changed_modules.clone()
            } else {
                BTreeSet::new()
            },
            declaration_index_changed,
            module_graph_changed,
        };

        // These counters describe compiler-owned product work, not protocol
        // requests. A source mutation refreshes the affected source shard and
        // its advisory/callable products; graph/declaration work is tracked
        // separately so callers can distinguish a body edit from a project
        // lifecycle event.
        stats.source_indexes_recomputed = changed_modules.len();
        stats.advisory_sources_recomputed = if self.product_request.advisory == AdvisoryLevel::Full {
            changed_modules.len()
        } else {
            0
        };
        stats.modules_relinked = if module_graph_changed { changed_modules.len() } else { 0 };
        stats.project_graph_rebuilt = effects.module_graph_changed;
        if previous_snapshot.is_some() {
            for callable in snapshot.callable_analyses.keys() {
                if callable.try_declaration_owner().is_some_and(|owner| owner.name.as_ref() != "<main>") {
                    callable_dispositions.entry(callable.clone()).or_insert(CallableRevisionDisposition::Reused);
                }
            }
        }
        stats.callables_recomputed = callable_dispositions
            .values()
            .filter(|disposition| **disposition == CallableRevisionDisposition::Recomputed)
            .count();
        stats.callables_reused = callable_dispositions
            .values()
            .filter(|disposition| **disposition == CallableRevisionDisposition::Reused)
            .count();
        stats.advisory_callables_recomputed = if self.product_request.advisory == AdvisoryLevel::Full {
            stats.callables_recomputed
        } else {
            0
        };
        let recomputed_keys = callable_dispositions
            .iter()
            .filter(|(_, disposition)| **disposition == CallableRevisionDisposition::Recomputed)
            .map(|(callable, _)| QueryKey::CallableBody(callable.clone()))
            .collect::<Vec<_>>();

        #[cfg(test)]
        self.fail_at_test_publication_point(SemanticPublicationFailpoint::AfterReadyQueryPublish)?;
        #[cfg(test)]
        self.cancel_at_test_publication_point(cancel);
        #[cfg(test)]
        self.fail_at_test_publication_point(SemanticPublicationFailpoint::BeforeCommit)?;
        if cancel.is_cancelled() {
            return Err(QueryOutcome::Cancelled);
        }
        self.last_snapshot = Some(snapshot.clone());
        self.last_known_good = Some(snapshot.clone());
        self.field_lifecycle_fingerprints = next_field_lifecycle_fingerprints;
        self.generic_header_dependencies = next_generic_header_dependencies;
        self.generic_header_reverse_dependencies = next_generic_header_reverse_dependencies;
        self.declaration_signature_dependencies = signature_graph.clone();
        self.declaration_signature_reverse_dependencies = reverse_signature_dependency_index(&signature_graph);
        self.declaration_signature_fingerprints = current_signature_fingerprints;
        self.inherent_impl_surface_owners = current_inherent_impl_surface_owners;
        self.alias_sources = alias_sources;
        self.alias_dependencies = alias_dependencies;
        self.alias_reverse_dependencies = alias_reverse_dependencies;
        self.alias_contract_dependencies = alias_contract_dependencies;
        self.alias_contract_reverse_dependencies = alias_contract_reverse_dependencies;
        if !alias_form_updates.is_empty() {
            let mut next_alias_forms = (*self.alias_forms).clone();
            for (declaration, form) in alias_form_updates {
                match form {
                    Some(form) => {
                        next_alias_forms.insert(declaration, form);
                    }
                    None => {
                        next_alias_forms.remove(&declaration);
                    }
                }
            }
            self.alias_forms = Arc::new(next_alias_forms);
        }
        self.module_diagnostics = current_module_diagnostics;
        self.semantic_diagnostic_contributions = semantic_diagnostic_contributions;
        self.default_field_lifecycle = default_field_lifecycle;
        self.sources = input.sources.clone();
        self.source_fingerprints = new_fingerprints;
        self.requested_deep_modules = None;

        Ok(SemanticWorkspaceUpdate {
            snapshot,
            invalidated: Arc::from(invalidated_keys.into_iter().collect::<Vec<_>>()),
            recomputed: Arc::from(recomputed_keys),
            stats,
            module_stats: module_delta.as_ref().map(|delta| delta.module_stats.clone()),
            effects,
        })
    }
}

pub(crate) fn canonical_erased_superclass_declaration(resolver: &dyn TypeResolver, module: &ModuleId, annotation: &TypeAnnotation) -> Option<DeclarationId> {
    let TypeAnnotationExpr::Reference(reference) = &annotation.expr else {
        return None;
    };
    if !reference.members.is_empty() {
        return None;
    }
    let key = phalcom_native_meta::UniverseKey::from_name(&reference.root)?;
    if !matches!(
        key,
        phalcom_native_meta::UniverseKey::Collection
            | phalcom_native_meta::UniverseKey::List
            | phalcom_native_meta::UniverseKey::Array
            | phalcom_native_meta::UniverseKey::Map
            | phalcom_native_meta::UniverseKey::Set
            | phalcom_native_meta::UniverseKey::Tuple
    ) {
        return None;
    }
    let declaration = resolver.resolve_type_name(module, &reference.root, &[])?;
    (declaration == crate::core_surface::universe_declaration(key)).then_some(declaration)
}

fn collect_alias_dependencies(
    annotation: &TypeAnnotation,
    module: &ModuleId,
    resolver: &dyn TypeResolver,
    aliases: &BTreeSet<DeclarationId>,
    dependencies: &mut BTreeSet<DeclarationId>,
) {
    match &annotation.expr {
        TypeAnnotationExpr::Reference(reference) => {
            let members = reference.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
            if let Some(declaration) = resolver.resolve_type_name(module, &reference.root, &members) {
                if aliases.contains(&declaration) {
                    dependencies.insert(declaration);
                }
            }
        }
        TypeAnnotationExpr::Application { origin, arguments, .. } => {
            collect_alias_dependencies(origin, module, resolver, aliases, dependencies);
            for argument in arguments {
                match argument {
                    phalcom_ast::ast::GenericArgumentSyntax::Single(annotation) => {
                        collect_alias_dependencies(annotation, module, resolver, aliases, dependencies);
                    }
                    phalcom_ast::ast::GenericArgumentSyntax::Expansion(expansion) => {
                        collect_alias_dependencies(&expansion.operand, module, resolver, aliases, dependencies);
                    }
                    phalcom_ast::ast::GenericArgumentSyntax::Remainder(remainder) => {
                        if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &remainder.remainder {
                            collect_alias_dependencies(constraint.as_ref(), module, resolver, aliases, dependencies);
                        }
                    }
                    phalcom_ast::ast::GenericArgumentSyntax::Value(_) | phalcom_ast::ast::GenericArgumentSyntax::InferenceHole { .. } => {}
                }
            }
        }
        TypeAnnotationExpr::Union { members, .. } => {
            for member in members {
                collect_alias_dependencies(member, module, resolver, aliases, dependencies);
            }
        }
        TypeAnnotationExpr::Tuple { elements, .. } => {
            for element in elements {
                match element {
                    phalcom_ast::ast::TypeTupleItem::Element(element) => {
                        collect_alias_dependencies(&element.ty, module, resolver, aliases, dependencies);
                    }
                    phalcom_ast::ast::TypeTupleItem::Expansion(expansion) => {
                        collect_alias_dependencies(&expansion.operand, module, resolver, aliases, dependencies);
                    }
                    phalcom_ast::ast::TypeTupleItem::Remainder(remainder) => {
                        if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &remainder.remainder {
                            collect_alias_dependencies(constraint, module, resolver, aliases, dependencies);
                        }
                    }
                }
            }
        }
        TypeAnnotationExpr::Callable { parameters, result, .. } => {
            for parameter in parameters {
                match parameter {
                    phalcom_ast::ast::TypeCallableItem::Parameter(parameter) => {
                        collect_alias_dependencies(&parameter.ty, module, resolver, aliases, dependencies);
                    }
                    phalcom_ast::ast::TypeCallableItem::Expansion(expansion) => {
                        collect_alias_dependencies(&expansion.operand, module, resolver, aliases, dependencies);
                    }
                    phalcom_ast::ast::TypeCallableItem::Remainder(_) => {}
                }
            }
            collect_alias_dependencies(result, module, resolver, aliases, dependencies);
        }
        TypeAnnotationExpr::Record { fields, tail, .. } => {
            for field in fields {
                collect_alias_dependencies(&field.ty, module, resolver, aliases, dependencies);
            }
            if let Some(phalcom_ast::ast::RecordRowTailSyntax::Anonymous(rem)) = tail {
                match rem {
                    phalcom_ast::ast::AnonymousRecordRemainderSyntax::TypedValue { value_type, .. } => {
                        collect_alias_dependencies(value_type, module, resolver, aliases, dependencies);
                    }
                    phalcom_ast::ast::AnonymousRecordRemainderSyntax::TypedKey { key_type, .. } => {
                        collect_alias_dependencies(key_type, module, resolver, aliases, dependencies);
                    }
                    phalcom_ast::ast::AnonymousRecordRemainderSyntax::Typed { key_type, value_type, .. } => {
                        collect_alias_dependencies(key_type, module, resolver, aliases, dependencies);
                        collect_alias_dependencies(value_type, module, resolver, aliases, dependencies);
                    }
                    phalcom_ast::ast::AnonymousRecordRemainderSyntax::Unconstrained(_) => {}
                }
            }
        }
        TypeAnnotationExpr::TypeLambda { body, .. } => collect_alias_dependencies(body, module, resolver, aliases, dependencies),
        TypeAnnotationExpr::ExactEnumCase {
            enum_target,
            generic_arguments,
            ..
        } => {
            collect_alias_dependencies(enum_target, module, resolver, aliases, dependencies);
            for argument in generic_arguments {
                collect_alias_dependencies(argument, module, resolver, aliases, dependencies);
            }
        }
        TypeAnnotationExpr::AssociatedTypeProjection { subject, .. } => {
            collect_alias_dependencies(subject, module, resolver, aliases, dependencies);
        }
        TypeAnnotationExpr::QualifiedAssociatedTypeProjection { subject, trait_ref, .. } => {
            collect_alias_dependencies(subject, module, resolver, aliases, dependencies);
            collect_alias_dependencies(&trait_ref.trait_type, module, resolver, aliases, dependencies);
            for spec in &trait_ref.associated_specializations {
                collect_alias_dependencies(&spec.value, module, resolver, aliases, dependencies);
            }
        }
        TypeAnnotationExpr::TypeMemberProjection { subject, arguments, .. } => {
            collect_alias_dependencies(subject, module, resolver, aliases, dependencies);
            for argument in arguments {
                if let phalcom_ast::ast::TypeMemberArgumentSyntax::TypeForm(ty) = argument {
                    collect_alias_dependencies(ty, module, resolver, aliases, dependencies);
                }
            }
        }
        TypeAnnotationExpr::Unit { .. }
        | TypeAnnotationExpr::Dynamic { .. }
        | TypeAnnotationExpr::Never { .. }
        | TypeAnnotationExpr::SelfType { .. }
        | TypeAnnotationExpr::Invalid { .. } => {}
    }
}

fn collect_associated_projection_targets(
    annotation: &TypeAnnotation,
    module: &ModuleId,
    surface: &crate::traits::TraitSurface,
    targets: &mut BTreeMap<(ModuleId, SourceRange), crate::traits::AssociatedTypeRequirementId>,
) {
    match &annotation.expr {
        TypeAnnotationExpr::AssociatedTypeProjection { subject, name, name_range, .. } => {
            collect_associated_projection_targets(subject, module, surface, targets);
            if matches!(subject.expr, TypeAnnotationExpr::SelfType { .. })
                && let Some(requirement) = surface.associated_type_by_name(name)
            {
                targets.insert((module.clone(), *name_range), requirement.requirement.clone());
            }
        }
        TypeAnnotationExpr::QualifiedAssociatedTypeProjection {
            subject,
            trait_ref,
            name,
            name_range,
            ..
        } => {
            collect_associated_projection_targets(subject, module, surface, targets);
            collect_associated_projection_targets(&trait_ref.trait_type, module, surface, targets);
            for spec in &trait_ref.associated_specializations {
                collect_associated_projection_targets(&spec.value, module, surface, targets);
            }
            if let Some(requirement) = surface.associated_type_by_name(name) {
                targets.insert((module.clone(), *name_range), requirement.requirement.clone());
            }
        }
        TypeAnnotationExpr::Application { origin, arguments, .. } => {
            collect_associated_projection_targets(origin, module, surface, targets);
            for argument in arguments {
                match argument {
                    phalcom_ast::ast::GenericArgumentSyntax::Single(annotation) => {
                        collect_associated_projection_targets(annotation, module, surface, targets);
                    }
                    phalcom_ast::ast::GenericArgumentSyntax::Expansion(expansion) => {
                        collect_associated_projection_targets(&expansion.operand, module, surface, targets);
                    }
                    phalcom_ast::ast::GenericArgumentSyntax::Remainder(remainder) => {
                        if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &remainder.remainder {
                            collect_associated_projection_targets(constraint.as_ref(), module, surface, targets);
                        }
                    }
                    phalcom_ast::ast::GenericArgumentSyntax::Value(_) | phalcom_ast::ast::GenericArgumentSyntax::InferenceHole { .. } => {}
                }
            }
        }
        TypeAnnotationExpr::Union { members, .. } => {
            for member in members {
                collect_associated_projection_targets(member, module, surface, targets);
            }
        }
        TypeAnnotationExpr::Tuple { elements, .. } => {
            for element in elements {
                match element {
                    phalcom_ast::ast::TypeTupleItem::Element(element) => {
                        collect_associated_projection_targets(&element.ty, module, surface, targets);
                    }
                    phalcom_ast::ast::TypeTupleItem::Expansion(expansion) => {
                        collect_associated_projection_targets(&expansion.operand, module, surface, targets);
                    }
                    phalcom_ast::ast::TypeTupleItem::Remainder(remainder) => {
                        if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &remainder.remainder {
                            collect_associated_projection_targets(constraint, module, surface, targets);
                        }
                    }
                }
            }
        }
        TypeAnnotationExpr::Record { fields, tail, .. } => {
            for field in fields {
                collect_associated_projection_targets(&field.ty, module, surface, targets);
            }
            if let Some(phalcom_ast::ast::RecordRowTailSyntax::Anonymous(rem)) = tail {
                match rem {
                    phalcom_ast::ast::AnonymousRecordRemainderSyntax::TypedValue { value_type, .. } => {
                        collect_associated_projection_targets(value_type, module, surface, targets);
                    }
                    phalcom_ast::ast::AnonymousRecordRemainderSyntax::TypedKey { key_type, .. } => {
                        collect_associated_projection_targets(key_type, module, surface, targets);
                    }
                    phalcom_ast::ast::AnonymousRecordRemainderSyntax::Typed { key_type, value_type, .. } => {
                        collect_associated_projection_targets(key_type, module, surface, targets);
                        collect_associated_projection_targets(value_type, module, surface, targets);
                    }
                    phalcom_ast::ast::AnonymousRecordRemainderSyntax::Unconstrained(_) => {}
                }
            }
        }
        TypeAnnotationExpr::Callable { parameters, result, .. } => {
            for parameter in parameters {
                match parameter {
                    phalcom_ast::ast::TypeCallableItem::Parameter(parameter) => {
                        collect_associated_projection_targets(&parameter.ty, module, surface, targets);
                    }
                    phalcom_ast::ast::TypeCallableItem::Expansion(expansion) => {
                        collect_associated_projection_targets(&expansion.operand, module, surface, targets);
                    }
                    phalcom_ast::ast::TypeCallableItem::Remainder(_) => {}
                }
            }
            collect_associated_projection_targets(result, module, surface, targets);
        }
        TypeAnnotationExpr::TypeLambda { body, .. } => collect_associated_projection_targets(body, module, surface, targets),
        TypeAnnotationExpr::ExactEnumCase {
            enum_target,
            generic_arguments,
            ..
        } => {
            collect_associated_projection_targets(enum_target, module, surface, targets);
            for argument in generic_arguments {
                collect_associated_projection_targets(argument, module, surface, targets);
            }
        }
        TypeAnnotationExpr::TypeMemberProjection { subject, arguments, .. } => {
            collect_associated_projection_targets(subject, module, surface, targets);
            for argument in arguments {
                if let phalcom_ast::ast::TypeMemberArgumentSyntax::TypeForm(ty) = argument {
                    collect_associated_projection_targets(ty, module, surface, targets);
                }
            }
        }
        TypeAnnotationExpr::Reference(_)
        | TypeAnnotationExpr::Unit { .. }
        | TypeAnnotationExpr::Dynamic { .. }
        | TypeAnnotationExpr::Never { .. }
        | TypeAnnotationExpr::SelfType { .. }
        | TypeAnnotationExpr::Invalid { .. } => {}
    }
}

fn collect_associated_projection_targets_from_behavior(
    member: &phalcom_ast::ast::BehaviorMember,
    module: &ModuleId,
    surface: &crate::traits::TraitSurface,
    targets: &mut BTreeMap<(ModuleId, SourceRange), crate::traits::AssociatedTypeRequirementId>,
) {
    match member {
        phalcom_ast::ast::BehaviorMember::Method(method) => {
            for parameter in &method.params {
                if let Some(annotation) = parameter.type_annotation() {
                    collect_associated_projection_targets(annotation, module, surface, targets);
                } else if let Some(lane) = parameter.rest_lane_annotation() {
                    collect_associated_projection_targets(&lane.operand, module, surface, targets);
                }
            }
            if let Some(annotation) = &method.return_annotation {
                collect_associated_projection_targets(annotation, module, surface, targets);
            }
        }
        phalcom_ast::ast::BehaviorMember::Getter(getter) => {
            if let Some(annotation) = &getter.return_annotation {
                collect_associated_projection_targets(annotation, module, surface, targets);
            }
        }
        phalcom_ast::ast::BehaviorMember::Setter(setter) => {
            if let Some(annotation) = setter.param.type_annotation() {
                collect_associated_projection_targets(annotation, module, surface, targets);
            } else if let Some(lane) = setter.param.rest_lane_annotation() {
                collect_associated_projection_targets(&lane.operand, module, surface, targets);
            }
            if let Some(annotation) = &setter.return_annotation {
                collect_associated_projection_targets(annotation, module, surface, targets);
            }
        }
        phalcom_ast::ast::BehaviorMember::Index(index) => {
            for parameter in &index.params {
                if let Some(annotation) = parameter.type_annotation() {
                    collect_associated_projection_targets(annotation, module, surface, targets);
                } else if let Some(lane) = parameter.rest_lane_annotation() {
                    collect_associated_projection_targets(&lane.operand, module, surface, targets);
                }
            }
            if let phalcom_ast::ast::IndexAccessor::Set { value } = &index.accessor {
                if let Some(annotation) = value.type_annotation() {
                    collect_associated_projection_targets(annotation, module, surface, targets);
                } else if let Some(lane) = value.rest_lane_annotation() {
                    collect_associated_projection_targets(&lane.operand, module, surface, targets);
                }
            }
            if let Some(annotation) = &index.return_annotation {
                collect_associated_projection_targets(annotation, module, surface, targets);
            }
        }
    }
}

fn collect_type_annotation_declarations(
    annotation: &TypeAnnotation,
    module: &ModuleId,
    resolver: &dyn TypeResolver,
    dependencies: &mut BTreeSet<DeclarationId>,
) {
    match &annotation.expr {
        TypeAnnotationExpr::Reference(reference) => {
            let members = reference.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
            if let Some(declaration) = resolver.resolve_type_name(module, &reference.root, &members) {
                dependencies.insert(declaration);
            }
        }
        TypeAnnotationExpr::Application { origin, arguments, .. } => {
            collect_type_annotation_declarations(origin, module, resolver, dependencies);
            for argument in arguments {
                match argument {
                    phalcom_ast::ast::GenericArgumentSyntax::Single(annotation) => {
                        collect_type_annotation_declarations(annotation, module, resolver, dependencies);
                    }
                    phalcom_ast::ast::GenericArgumentSyntax::Expansion(expansion) => {
                        collect_type_annotation_declarations(&expansion.operand, module, resolver, dependencies);
                    }
                    phalcom_ast::ast::GenericArgumentSyntax::Remainder(remainder) => {
                        if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &remainder.remainder {
                            collect_type_annotation_declarations(constraint.as_ref(), module, resolver, dependencies);
                        }
                    }
                    phalcom_ast::ast::GenericArgumentSyntax::Value(_) | phalcom_ast::ast::GenericArgumentSyntax::InferenceHole { .. } => {}
                }
            }
        }
        TypeAnnotationExpr::Union { members, .. } => {
            for member in members {
                collect_type_annotation_declarations(member, module, resolver, dependencies);
            }
        }
        TypeAnnotationExpr::Tuple { elements, .. } => {
            for element in elements {
                match element {
                    phalcom_ast::ast::TypeTupleItem::Element(element) => {
                        collect_type_annotation_declarations(&element.ty, module, resolver, dependencies);
                    }
                    phalcom_ast::ast::TypeTupleItem::Expansion(expansion) => {
                        collect_type_annotation_declarations(&expansion.operand, module, resolver, dependencies);
                    }
                    phalcom_ast::ast::TypeTupleItem::Remainder(rem) => {
                        if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &rem.remainder {
                            collect_type_annotation_declarations(constraint, module, resolver, dependencies);
                        }
                    }
                }
            }
        }
        TypeAnnotationExpr::Callable { parameters, result, .. } => {
            for parameter in parameters {
                match parameter {
                    phalcom_ast::ast::TypeCallableItem::Parameter(parameter) => {
                        collect_type_annotation_declarations(&parameter.ty, module, resolver, dependencies);
                    }
                    phalcom_ast::ast::TypeCallableItem::Expansion(expansion) => {
                        collect_type_annotation_declarations(&expansion.operand, module, resolver, dependencies);
                    }
                    phalcom_ast::ast::TypeCallableItem::Remainder(rem) => {
                        if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &rem.remainder {
                            collect_type_annotation_declarations(constraint, module, resolver, dependencies);
                        }
                    }
                }
            }
            collect_type_annotation_declarations(result, module, resolver, dependencies);
        }
        TypeAnnotationExpr::Record { fields, .. } => {
            for field in fields {
                collect_type_annotation_declarations(&field.ty, module, resolver, dependencies);
            }
        }
        TypeAnnotationExpr::TypeLambda { body, .. } => collect_type_annotation_declarations(body, module, resolver, dependencies),
        TypeAnnotationExpr::ExactEnumCase {
            enum_target,
            generic_arguments,
            ..
        } => {
            collect_type_annotation_declarations(enum_target, module, resolver, dependencies);
            for argument in generic_arguments {
                collect_type_annotation_declarations(argument, module, resolver, dependencies);
            }
        }
        TypeAnnotationExpr::AssociatedTypeProjection { subject, .. } => {
            collect_type_annotation_declarations(subject, module, resolver, dependencies);
        }
        TypeAnnotationExpr::QualifiedAssociatedTypeProjection { subject, trait_ref, .. } => {
            collect_type_annotation_declarations(subject, module, resolver, dependencies);
            collect_type_annotation_declarations(&trait_ref.trait_type, module, resolver, dependencies);
            for spec in &trait_ref.associated_specializations {
                collect_type_annotation_declarations(&spec.value, module, resolver, dependencies);
            }
        }
        TypeAnnotationExpr::TypeMemberProjection { subject, arguments, .. } => {
            collect_type_annotation_declarations(subject, module, resolver, dependencies);
            for argument in arguments {
                match argument {
                    phalcom_ast::ast::TypeMemberArgumentSyntax::TypeForm(ty) => {
                        collect_type_annotation_declarations(ty, module, resolver, dependencies);
                    }
                    phalcom_ast::ast::TypeMemberArgumentSyntax::IntLiteral { .. } | phalcom_ast::ast::TypeMemberArgumentSyntax::Name { .. } => {}
                }
            }
        }
        TypeAnnotationExpr::Unit { .. }
        | TypeAnnotationExpr::Dynamic { .. }
        | TypeAnnotationExpr::Never { .. }
        | TypeAnnotationExpr::SelfType { .. }
        | TypeAnnotationExpr::Invalid { .. } => {}
    }
}

fn find_alias_cycles(graph: &BTreeMap<DeclarationId, BTreeSet<DeclarationId>>) -> BTreeSet<DeclarationId> {
    fn visit(
        node: &DeclarationId,
        graph: &BTreeMap<DeclarationId, BTreeSet<DeclarationId>>,
        stack: &mut Vec<DeclarationId>,
        visiting: &mut BTreeSet<DeclarationId>,
        visited: &mut BTreeSet<DeclarationId>,
        cycles: &mut BTreeSet<DeclarationId>,
    ) {
        if let Some(index) = stack.iter().position(|current| current == node) {
            cycles.extend(stack[index..].iter().cloned());
            return;
        }
        if !visited.insert(node.clone()) || !visiting.insert(node.clone()) {
            return;
        }
        stack.push(node.clone());
        if let Some(dependencies) = graph.get(node) {
            for dependency in dependencies {
                visit(dependency, graph, stack, visiting, visited, cycles);
            }
        }
        stack.pop();
        visiting.remove(node);
    }

    let mut cycles = BTreeSet::new();
    let mut stack = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for declaration in graph.keys() {
        visit(declaration, graph, &mut stack, &mut visiting, &mut visited, &mut cycles);
    }
    cycles
}

fn find_alias_cycles_from_seeds(graph: &BTreeMap<DeclarationId, BTreeSet<DeclarationId>>, seeds: &BTreeSet<DeclarationId>) -> BTreeSet<DeclarationId> {
    let mut reachable = BTreeSet::new();
    let mut pending = seeds.iter().cloned().collect::<Vec<_>>();
    while let Some(declaration) = pending.pop() {
        if !reachable.insert(declaration.clone()) {
            continue;
        }
        if let Some(dependencies) = graph.get(&declaration) {
            pending.extend(dependencies.iter().cloned());
        }
    }

    let affected_graph = reachable
        .iter()
        .map(|declaration| {
            let dependencies = graph
                .get(declaration)
                .map(|dependencies| dependencies.intersection(&reachable).cloned().collect())
                .unwrap_or_default();
            (declaration.clone(), dependencies)
        })
        .collect::<BTreeMap<_, _>>();
    find_alias_cycles(&affected_graph)
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum TypeSignatureDependencyNode {
    Alias(DeclarationId),
    Header(DeclarationId),
    Member(crate::types::type_member::TypeMemberDeclarationId),
    StaticCaseSignature(crate::identity::StaticCaseId),
}

type StaticCaseSource = (ModuleId, DeclarationId, phalcom_ast::ast::AlgebraicDef, crate::static_case::StaticCaseHeader);

fn collect_static_case_sources(
    shards: &BTreeMap<ModuleId, Arc<ModuleSemanticStructureShard>>,
    resolver: &LinkedTypeResolver,
    catalogs: &BTreeMap<crate::identity::StaticDomainDeclarationId, Arc<crate::static_case::StaticDomainCaseCatalogProduct>>,
) -> BTreeMap<crate::identity::StaticCaseId, StaticCaseSource> {
    fn add_definition(
        module: &ModuleId,
        owner: &DeclarationId,
        definition: &phalcom_ast::ast::AlgebraicDef,
        catalogs: &BTreeMap<crate::identity::StaticDomainDeclarationId, Arc<crate::static_case::StaticDomainCaseCatalogProduct>>,
        sources: &mut BTreeMap<crate::identity::StaticCaseId, StaticCaseSource>,
    ) {
        let domain = match definition.phase {
            phalcom_ast::ast::AlgebraicPhase::TypeOnly if owner.name.as_ref() == definition.name => {
                crate::identity::StaticDomainDeclarationId::Nominal(owner.clone())
            }
            phalcom_ast::ast::AlgebraicPhase::TypeOnly => crate::identity::StaticDomainDeclarationId::Associated(
                crate::identity::AssociatedStaticDomainDeclarationId::new(owner.clone(), definition.name.clone()),
            ),
            phalcom_ast::ast::AlgebraicPhase::Runtime => return,
        };
        let Some(catalog) = catalogs.get(&domain) else { return };
        for case in catalog.cases.iter() {
            if matches!(case.shape, crate::static_case::StaticCaseShape::Constructor { .. }) {
                sources.insert(case.id.clone(), (module.clone(), owner.clone(), definition.clone(), case.clone()));
            }
        }
    }

    let mut sources = BTreeMap::new();
    for (module, shard) in shards {
        for statement in &shard.source.program.statements {
            match statement {
                Statement::Data(definition) if definition.phase == phalcom_ast::ast::AlgebraicPhase::TypeOnly => {
                    let owner = DeclarationId::new(module.clone(), definition.name.clone().into());
                    add_definition(module, &owner, definition, catalogs, &mut sources);
                }
                Statement::Class(class_def) => {
                    let owner = DeclarationId::new(module.clone(), class_def.name.clone().into());
                    for definition in &class_def.associated_type_data {
                        add_definition(module, &owner, definition, catalogs, &mut sources);
                    }
                }
                Statement::Impl(impl_def) if matches!(impl_def.kind, phalcom_ast::ast::ImplKind::Inherent) => {
                    if let Some(symbol) = impl_def.target.origin_symbol_ref() {
                        let members = symbol.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
                        if let Some(owner) = resolver.resolve_type_name(module, &symbol.root, &members) {
                            for definition in &impl_def.associated_type_data {
                                add_definition(module, &owner, definition, catalogs, &mut sources);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    sources
}

fn static_case_payload_annotations<'a>(definition: &'a phalcom_ast::ast::AlgebraicDef, id: &crate::identity::StaticCaseId) -> Option<Vec<&'a TypeAnnotation>> {
    match &definition.shape {
        phalcom_ast::ast::AlgebraicShapeSyntax::Product(product) => {
            let components = product.components();
            Some(components.iter().map(|component| &component.annotation).collect())
        }
        phalcom_ast::ast::AlgebraicShapeSyntax::Sum(sum) => {
            let case = sum
                .cases
                .iter()
                .find(|case| phalcom_ast::selector::selector_from_variant(case) == id.selector)?;
            if !case.generic_parameters.is_empty() || case.where_clause.is_some() || case.result_annotation.is_some() {
                return None;
            }
            match &case.payload {
                phalcom_ast::ast::CasePayloadSyntax::Singleton => Some(Vec::new()),
                phalcom_ast::ast::CasePayloadSyntax::Tuple { parameters, .. } => {
                    Some(parameters.iter().map(|parameter| parameter.type_annotation()).collect::<Option<Vec<_>>>()?)
                }
                phalcom_ast::ast::CasePayloadSyntax::Record { components, .. } => Some(components.iter().map(|component| &component.annotation).collect()),
            }
        }
        phalcom_ast::ast::AlgebraicShapeSyntax::IntrinsicFacade(_) => None,
    }
}

fn build_static_case_signature(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    resolver: &LinkedTypeResolver,
    module: &ModuleId,
    owner: &DeclarationId,
    definition: &phalcom_ast::ast::AlgebraicDef,
    header: &crate::static_case::StaticCaseHeader,
) -> (Option<crate::static_case::StaticCaseSignatureProduct>, Vec<SemanticDiagnostic>) {
    let mut diagnostics = Vec::new();
    let Some(annotations) = static_case_payload_annotations(definition, &header.id) else {
        diagnostics.push(SemanticDiagnostic::error_in(
            module.clone(),
            DiagnosticCode::KindExpectedType,
            "static-data constructors cannot declare case-local generics, constraints, or specialized results",
            header.source_range,
        ));
        return (None, diagnostics);
    };
    let mut type_parameters = HashMap::new();
    if let Some(binder) = declarations.generic_binder_header(owner) {
        for parameter in binder.parameters.iter().copied() {
            let data = store.type_parameter(parameter);
            let name = data.name.to_string();
            let binding = type_level_binding_for_parameter(store, parameter);
            type_parameters.insert(name, binding);
        }
    }
    let scoped = crate::types::annotation::ScopedTypeResolver {
        parent: resolver,
        type_parameters,
    };
    let site = TypeFormationSite::member(module.clone(), owner.clone(), DispatchSide::Class);
    let mut parameter_kinds = Vec::with_capacity(annotations.len());
    for annotation in annotations {
        let form = crate::types::annotation::resolve_type_form(store, declarations, &scoped, &site, annotation, &mut diagnostics);
        let crate::types::annotation::TypeFormationOutcome::Ready(form) = form else {
            return (None, diagnostics);
        };
        let kind = crate::types::annotation::static_domain_kind_of_form(store, form);
        if kind != KindId::TYPE && !matches!(store.get_kind(kind), crate::types::kind::KindData::StaticDomain(_)) {
            diagnostics.push(SemanticDiagnostic::error_in(
                module.clone(),
                DiagnosticCode::KindExpectedType,
                "type-data payload must have kind Type or a supported static domain",
                annotation.range,
            ));
            return (None, diagnostics);
        }
        parameter_kinds.push(kind);
    }
    (
        Some(crate::static_case::StaticCaseSignatureProduct {
            id: header.id.clone(),
            parameter_kinds: parameter_kinds.into_boxed_slice(),
            source_range: header.source_range,
        }),
        diagnostics,
    )
}

fn collect_static_case_signature_dependencies(
    annotation: &TypeAnnotation,
    module: &ModuleId,
    resolver: &dyn TypeResolver,
    catalogs: &BTreeMap<crate::identity::StaticDomainDeclarationId, Arc<crate::static_case::StaticDomainCaseCatalogProduct>>,
    dependencies: &mut BTreeSet<TypeSignatureDependencyNode>,
) {
    use phalcom_ast::ast::{GenericArgumentSyntax, TypeCallableItem, TypeMemberArgumentSyntax, TypeTupleItem};

    fn domain_of_form(form: &TypeAnnotation, module: &ModuleId, resolver: &dyn TypeResolver) -> Option<crate::identity::StaticDomainDeclarationId> {
        match &form.expr {
            TypeAnnotationExpr::Reference(reference) => {
                let members = reference.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
                resolver
                    .resolve_type_name(module, &reference.root, &members)
                    .map(crate::identity::StaticDomainDeclarationId::Nominal)
            }
            TypeAnnotationExpr::Application { origin, .. } => domain_of_form(origin, module, resolver),
            TypeAnnotationExpr::ExactEnumCase {
                enum_target,
                variant_name,
                generic_arguments,
                payload_shape,
                ..
            } if generic_arguments.is_empty() && payload_shape.is_none() => {
                let symbol = enum_target.origin_symbol_ref()?;
                let members = symbol.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
                let owner = resolver.resolve_type_name(module, &symbol.root, &members)?;
                resolver
                    .resolve_associated_static_domain(&owner, variant_name)
                    .map(crate::identity::StaticDomainDeclarationId::Associated)
            }
            _ => None,
        }
    }

    match &annotation.expr {
        TypeAnnotationExpr::ExactEnumCase {
            enum_target,
            variant_name,
            generic_arguments,
            payload_shape,
            ..
        } => {
            if let Some(domain) = domain_of_form(enum_target, module, resolver)
                && let Some(catalog) = catalogs.get(&domain)
            {
                let selected = if payload_shape.is_none() && !generic_arguments.is_empty() {
                    catalog.select_constructor(variant_name, generic_arguments.len())
                } else {
                    let selector = phalcom_ast::selector::selector_from_exact_case_target(variant_name, payload_shape.as_ref());
                    catalog.select(&selector)
                };
                if let Some(case) = selected
                    && matches!(case.shape, crate::static_case::StaticCaseShape::Constructor { .. })
                {
                    dependencies.insert(TypeSignatureDependencyNode::StaticCaseSignature(case.id.clone()));
                }
            }
            collect_static_case_signature_dependencies(enum_target, module, resolver, catalogs, dependencies);
            for argument in generic_arguments {
                collect_static_case_signature_dependencies(argument, module, resolver, catalogs, dependencies);
            }
        }
        TypeAnnotationExpr::Application { origin, arguments, .. } => {
            collect_static_case_signature_dependencies(origin, module, resolver, catalogs, dependencies);
            for argument in arguments {
                match argument {
                    GenericArgumentSyntax::Single(argument) => collect_static_case_signature_dependencies(argument, module, resolver, catalogs, dependencies),
                    GenericArgumentSyntax::Expansion(expansion) => {
                        collect_static_case_signature_dependencies(&expansion.operand, module, resolver, catalogs, dependencies)
                    }
                    GenericArgumentSyntax::Remainder(remainder) => {
                        if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &remainder.remainder {
                            collect_static_case_signature_dependencies(constraint.as_ref(), module, resolver, catalogs, dependencies);
                        }
                    }
                    GenericArgumentSyntax::Value(_) | GenericArgumentSyntax::InferenceHole { .. } => {}
                }
            }
        }
        TypeAnnotationExpr::Union { members, .. } => {
            for item in members {
                collect_static_case_signature_dependencies(item, module, resolver, catalogs, dependencies);
            }
        }
        TypeAnnotationExpr::Tuple { elements, .. } => {
            for item in elements {
                match item {
                    TypeTupleItem::Element(element) => collect_static_case_signature_dependencies(&element.ty, module, resolver, catalogs, dependencies),
                    TypeTupleItem::Expansion(expansion) => {
                        collect_static_case_signature_dependencies(&expansion.operand, module, resolver, catalogs, dependencies)
                    }
                    TypeTupleItem::Remainder(rem) => {
                        if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &rem.remainder {
                            collect_static_case_signature_dependencies(constraint, module, resolver, catalogs, dependencies);
                        }
                    }
                }
            }
        }
        TypeAnnotationExpr::Callable { parameters, result, .. } => {
            for item in parameters {
                match item {
                    TypeCallableItem::Parameter(parameter) => {
                        collect_static_case_signature_dependencies(&parameter.ty, module, resolver, catalogs, dependencies)
                    }
                    TypeCallableItem::Expansion(expansion) => {
                        collect_static_case_signature_dependencies(&expansion.operand, module, resolver, catalogs, dependencies)
                    }
                    TypeCallableItem::Remainder(rem) => {
                        if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &rem.remainder {
                            collect_static_case_signature_dependencies(constraint, module, resolver, catalogs, dependencies);
                        }
                    }
                }
            }
            collect_static_case_signature_dependencies(result, module, resolver, catalogs, dependencies);
        }
        TypeAnnotationExpr::Record { fields, .. } => {
            for field in fields {
                collect_static_case_signature_dependencies(&field.ty, module, resolver, catalogs, dependencies);
            }
        }
        TypeAnnotationExpr::TypeLambda { body, .. } => collect_static_case_signature_dependencies(body, module, resolver, catalogs, dependencies),
        TypeAnnotationExpr::TypeMemberProjection { subject, arguments, .. } => {
            collect_static_case_signature_dependencies(subject, module, resolver, catalogs, dependencies);
            for argument in arguments {
                if let TypeMemberArgumentSyntax::TypeForm(argument) = argument {
                    collect_static_case_signature_dependencies(argument, module, resolver, catalogs, dependencies);
                }
            }
        }
        TypeAnnotationExpr::AssociatedTypeProjection { subject, .. } => {
            collect_static_case_signature_dependencies(subject, module, resolver, catalogs, dependencies)
        }
        TypeAnnotationExpr::QualifiedAssociatedTypeProjection { subject, trait_ref, .. } => {
            collect_static_case_signature_dependencies(subject, module, resolver, catalogs, dependencies);
            collect_static_case_signature_dependencies(&trait_ref.trait_type, module, resolver, catalogs, dependencies);
            for specialization in &trait_ref.associated_specializations {
                collect_static_case_signature_dependencies(&specialization.value, module, resolver, catalogs, dependencies);
            }
        }
        TypeAnnotationExpr::Reference(_)
        | TypeAnnotationExpr::Unit { .. }
        | TypeAnnotationExpr::Dynamic { .. }
        | TypeAnnotationExpr::Never { .. }
        | TypeAnnotationExpr::SelfType { .. }
        | TypeAnnotationExpr::Invalid { .. } => {}
    }
}

fn signature_node_module(node: &TypeSignatureDependencyNode) -> &ModuleId {
    match node {
        TypeSignatureDependencyNode::Alias(declaration) | TypeSignatureDependencyNode::Header(declaration) => &declaration.module,
        TypeSignatureDependencyNode::Member(member) => &member.owner.module,
        TypeSignatureDependencyNode::StaticCaseSignature(case) => match &case.domain {
            crate::identity::StaticDomainDeclarationId::Nominal(declaration) => &declaration.module,
            crate::identity::StaticDomainDeclarationId::Associated(declaration) => &declaration.owner.module,
        },
    }
}

fn reverse_signature_dependency_index(
    dependencies: &BTreeMap<TypeSignatureDependencyNode, BTreeSet<TypeSignatureDependencyNode>>,
) -> BTreeMap<TypeSignatureDependencyNode, BTreeSet<TypeSignatureDependencyNode>> {
    let mut reverse = BTreeMap::<TypeSignatureDependencyNode, BTreeSet<TypeSignatureDependencyNode>>::new();
    for (consumer, providers) in dependencies {
        for provider in providers {
            reverse.entry(provider.clone()).or_default().insert(consumer.clone());
        }
    }
    reverse
}

fn reverse_signature_dependency_closure(
    reverse: &BTreeMap<TypeSignatureDependencyNode, BTreeSet<TypeSignatureDependencyNode>>,
    seeds: &BTreeSet<TypeSignatureDependencyNode>,
) -> BTreeSet<TypeSignatureDependencyNode> {
    let mut affected = seeds.clone();
    let mut pending = seeds.iter().cloned().collect::<Vec<_>>();
    while let Some(provider) = pending.pop() {
        for consumer in reverse.get(&provider).into_iter().flatten() {
            if affected.insert(consumer.clone()) {
                pending.push(consumer.clone());
            }
        }
    }
    affected
}

fn declaration_signature_source_fingerprints<'a>(
    shards: impl IntoIterator<Item = (&'a ModuleId, &'a Arc<ModuleSemanticStructureShard>)>,
    resolver: &dyn TypeResolver,
) -> BTreeMap<TypeSignatureDependencyNode, u64> {
    fn hash_text(text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }

    fn range_text(shard: &ModuleSemanticStructureShard, range: SourceRange) -> &str {
        shard.source.text.get(range.start..range.end).unwrap_or_default()
    }

    fn header_hash<T: std::fmt::Debug>(value: T) -> u64 {
        hash_text(&format!("{value:?}"))
    }

    let mut fingerprints = BTreeMap::new();
    for (module, shard) in shards {
        for (declaration, alias) in shard.alias_sources.iter() {
            fingerprints.insert(
                TypeSignatureDependencyNode::Alias(declaration.clone()),
                hash_text(range_text(shard, alias.range)),
            );
        }
        for statement in &shard.source.program.statements {
            match statement {
                Statement::Class(class_def) => {
                    let owner = DeclarationId::new(module.clone(), class_def.name.clone().into());
                    if !class_def.generic_parameters.is_empty() {
                        fingerprints.insert(
                            TypeSignatureDependencyNode::Header(owner.clone()),
                            header_hash((&class_def.generic_parameters, &class_def.where_clause, &class_def.superclass)),
                        );
                    }
                    for member in &class_def.members {
                        let ClassMember::TypeMember(definition) = member else { continue };
                        let id = crate::types::type_member::TypeMemberDeclarationId::new(
                            owner.clone(),
                            crate::types::type_member::TypeMemberSelector::from_syntax(&definition.selector),
                        );
                        fingerprints.insert(TypeSignatureDependencyNode::Member(id), hash_text(range_text(shard, definition.range)));
                    }
                    for definition in &class_def.associated_type_data {
                        if definition.phase != phalcom_ast::ast::AlgebraicPhase::TypeOnly {
                            continue;
                        }
                        let domain = crate::identity::StaticDomainDeclarationId::Associated(crate::identity::AssociatedStaticDomainDeclarationId::new(
                            owner.clone(),
                            definition.name.clone(),
                        ));
                        let catalog = crate::static_case::catalog_from_declaration(domain, definition);
                        for source_case in match &definition.shape {
                            phalcom_ast::ast::AlgebraicShapeSyntax::Sum(sum) => sum.cases.iter().collect::<Vec<_>>(),
                            phalcom_ast::ast::AlgebraicShapeSyntax::Product(_) => Vec::new(),
                            phalcom_ast::ast::AlgebraicShapeSyntax::IntrinsicFacade(_) => Vec::new(),
                        } {
                            let selector = phalcom_ast::selector::selector_from_variant(source_case);
                            let id = crate::identity::StaticCaseId::with_selector(catalog.domain.clone(), selector);
                            if catalog
                                .select(&id.selector)
                                .is_some_and(|case| matches!(case.shape, crate::static_case::StaticCaseShape::Constructor { .. }))
                            {
                                fingerprints.insert(
                                    TypeSignatureDependencyNode::StaticCaseSignature(id),
                                    hash_text(range_text(shard, source_case.range)),
                                );
                            }
                        }
                    }
                }
                Statement::Data(data_def) => {
                    let declaration = DeclarationId::new(module.clone(), data_def.name.clone().into());
                    if !data_def.generic_parameters.is_empty() {
                        fingerprints.insert(
                            TypeSignatureDependencyNode::Header(declaration.clone()),
                            header_hash((&data_def.generic_parameters, &data_def.where_clause)),
                        );
                    }
                    if data_def.phase == phalcom_ast::ast::AlgebraicPhase::TypeOnly {
                        if matches!(&data_def.shape, phalcom_ast::ast::AlgebraicShapeSyntax::IntrinsicFacade(_)) {
                            continue;
                        }
                        let domain = crate::identity::StaticDomainDeclarationId::Nominal(declaration);
                        let catalog = crate::static_case::catalog_from_declaration(domain, data_def);
                        match &data_def.shape {
                            phalcom_ast::ast::AlgebraicShapeSyntax::Sum(sum) => {
                                for source_case in &sum.cases {
                                    let selector = phalcom_ast::selector::selector_from_variant(source_case);
                                    let id = crate::identity::StaticCaseId::with_selector(catalog.domain.clone(), selector);
                                    if catalog
                                        .select(&id.selector)
                                        .is_some_and(|case| matches!(case.shape, crate::static_case::StaticCaseShape::Constructor { .. }))
                                    {
                                        fingerprints.insert(
                                            TypeSignatureDependencyNode::StaticCaseSignature(id),
                                            hash_text(range_text(shard, source_case.range)),
                                        );
                                    }
                                }
                            }
                            phalcom_ast::ast::AlgebraicShapeSyntax::Product(_) => {
                                if let Some(case) = catalog
                                    .cases
                                    .iter()
                                    .find(|case| matches!(case.shape, crate::static_case::StaticCaseShape::Constructor { .. }))
                                {
                                    fingerprints.insert(
                                        TypeSignatureDependencyNode::StaticCaseSignature(case.id.clone()),
                                        hash_text(range_text(shard, data_def.range)),
                                    );
                                }
                            }
                            phalcom_ast::ast::AlgebraicShapeSyntax::IntrinsicFacade(_) => {}
                        }
                    }
                }
                Statement::Trait(trait_def) if !trait_def.generic_parameters.is_empty() => {
                    let declaration = DeclarationId::new(module.clone(), trait_def.name.clone().into());
                    fingerprints.insert(
                        TypeSignatureDependencyNode::Header(declaration),
                        header_hash((&trait_def.generic_parameters, &trait_def.where_clause)),
                    );
                }
                Statement::Impl(impl_def) if matches!(impl_def.kind, phalcom_ast::ast::ImplKind::Inherent) => {
                    if let Some(symbol) = impl_def.target.origin_symbol_ref()
                        && let Some(owner) = resolver.resolve_type_name(
                            module,
                            &symbol.root,
                            &symbol.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>(),
                        )
                    {
                        for definition in &impl_def.associated_type_data {
                            if definition.phase != phalcom_ast::ast::AlgebraicPhase::TypeOnly {
                                continue;
                            }
                            let domain = crate::identity::StaticDomainDeclarationId::Associated(crate::identity::AssociatedStaticDomainDeclarationId::new(
                                owner.clone(),
                                definition.name.clone(),
                            ));
                            let catalog = crate::static_case::catalog_from_declaration(domain, definition);
                            match &definition.shape {
                                phalcom_ast::ast::AlgebraicShapeSyntax::Sum(sum) => {
                                    for source_case in &sum.cases {
                                        let selector = phalcom_ast::selector::selector_from_variant(source_case);
                                        let id = crate::identity::StaticCaseId::with_selector(catalog.domain.clone(), selector);
                                        if catalog
                                            .select(&id.selector)
                                            .is_some_and(|case| matches!(case.shape, crate::static_case::StaticCaseShape::Constructor { .. }))
                                        {
                                            fingerprints.insert(
                                                TypeSignatureDependencyNode::StaticCaseSignature(id),
                                                hash_text(range_text(shard, source_case.range)),
                                            );
                                        }
                                    }
                                }
                                phalcom_ast::ast::AlgebraicShapeSyntax::Product(_) => {
                                    if let Some(case) = catalog
                                        .cases
                                        .iter()
                                        .find(|case| matches!(case.shape, crate::static_case::StaticCaseShape::Constructor { .. }))
                                    {
                                        fingerprints.insert(
                                            TypeSignatureDependencyNode::StaticCaseSignature(case.id.clone()),
                                            hash_text(range_text(shard, definition.range)),
                                        );
                                    }
                                }
                                phalcom_ast::ast::AlgebraicShapeSyntax::IntrinsicFacade(_) => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    fingerprints
}

fn collect_kind_signature_header_dependencies(
    syntax: &phalcom_ast::ast::KindSyntax,
    module: &ModuleId,
    resolver: &dyn TypeResolver,
    headers: &BTreeSet<DeclarationId>,
    dependencies: &mut BTreeSet<DeclarationId>,
) {
    match syntax {
        phalcom_ast::ast::KindSyntax::StaticDomain { form, .. } => {
            let mut referenced = BTreeSet::new();
            collect_type_annotation_declarations(form, module, resolver, &mut referenced);
            dependencies.extend(referenced.into_iter().filter(|declaration| headers.contains(declaration)));
        }
        phalcom_ast::ast::KindSyntax::Arrow { parameter, result, .. } => {
            collect_kind_signature_header_dependencies(parameter, module, resolver, headers, dependencies);
            collect_kind_signature_header_dependencies(result, module, resolver, headers, dependencies);
        }
        phalcom_ast::ast::KindSyntax::Grouped { inner, .. } => {
            collect_kind_signature_header_dependencies(inner, module, resolver, headers, dependencies);
        }
        phalcom_ast::ast::KindSyntax::Type(_)
        | phalcom_ast::ast::KindSyntax::RecordRow(_)
        | phalcom_ast::ast::KindSyntax::RecordFields(_)
        | phalcom_ast::ast::KindSyntax::TypePack(_)
        | phalcom_ast::ast::KindSyntax::LabeledTypePack(_)
        | phalcom_ast::ast::KindSyntax::CompleteTypePack(_)
        | phalcom_ast::ast::KindSyntax::Invalid { .. } => {}
    }
}

fn collect_kind_signature_alias_dependencies(
    syntax: &phalcom_ast::ast::KindSyntax,
    module: &ModuleId,
    resolver: &dyn TypeResolver,
    aliases: &BTreeSet<DeclarationId>,
    dependencies: &mut BTreeSet<DeclarationId>,
) {
    match syntax {
        phalcom_ast::ast::KindSyntax::StaticDomain { form, .. } => {
            let mut referenced = BTreeSet::new();
            collect_type_annotation_declarations(form, module, resolver, &mut referenced);
            dependencies.extend(referenced.into_iter().filter(|declaration| aliases.contains(declaration)));
        }
        phalcom_ast::ast::KindSyntax::Arrow { parameter, result, .. } => {
            collect_kind_signature_alias_dependencies(parameter, module, resolver, aliases, dependencies);
            collect_kind_signature_alias_dependencies(result, module, resolver, aliases, dependencies);
        }
        phalcom_ast::ast::KindSyntax::Grouped { inner, .. } => {
            collect_kind_signature_alias_dependencies(inner, module, resolver, aliases, dependencies);
        }
        phalcom_ast::ast::KindSyntax::Type(_)
        | phalcom_ast::ast::KindSyntax::RecordRow(_)
        | phalcom_ast::ast::KindSyntax::RecordFields(_)
        | phalcom_ast::ast::KindSyntax::TypePack(_)
        | phalcom_ast::ast::KindSyntax::LabeledTypePack(_)
        | phalcom_ast::ast::KindSyntax::CompleteTypePack(_)
        | phalcom_ast::ast::KindSyntax::Invalid { .. } => {}
    }
}

fn collect_source_member_projection_dependencies(
    annotation: &TypeAnnotation,
    module: &ModuleId,
    resolver: &dyn TypeResolver,
    members: &BTreeSet<crate::types::type_member::TypeMemberDeclarationId>,
    dependencies: &mut BTreeSet<crate::types::type_member::TypeMemberDeclarationId>,
) {
    use phalcom_ast::ast::{GenericArgumentSyntax, TypeCallableItem, TypeMemberArgumentSyntax, TypeTupleItem};

    match &annotation.expr {
        TypeAnnotationExpr::TypeMemberProjection {
            subject, selector, arguments, ..
        } => {
            if let Some(owner) = type_member_subject_owner(subject, module, resolver) {
                let member =
                    crate::types::type_member::TypeMemberDeclarationId::new(owner, crate::types::type_member::TypeMemberSelector::from_syntax(selector));
                if members.contains(&member) {
                    dependencies.insert(member);
                }
            }
            collect_source_member_projection_dependencies(subject, module, resolver, members, dependencies);
            for argument in arguments {
                if let TypeMemberArgumentSyntax::TypeForm(argument) = argument {
                    collect_source_member_projection_dependencies(argument, module, resolver, members, dependencies);
                }
            }
        }
        TypeAnnotationExpr::Application { origin, arguments, .. } => {
            collect_source_member_projection_dependencies(origin, module, resolver, members, dependencies);
            for argument in arguments {
                match argument {
                    GenericArgumentSyntax::Single(argument) => {
                        collect_source_member_projection_dependencies(argument, module, resolver, members, dependencies);
                    }
                    GenericArgumentSyntax::Expansion(expansion) => {
                        collect_source_member_projection_dependencies(&expansion.operand, module, resolver, members, dependencies);
                    }
                    GenericArgumentSyntax::Remainder(remainder) => {
                        if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &remainder.remainder {
                            collect_source_member_projection_dependencies(constraint.as_ref(), module, resolver, members, dependencies);
                        }
                    }
                    GenericArgumentSyntax::Value(_) | GenericArgumentSyntax::InferenceHole { .. } => {}
                }
            }
        }
        TypeAnnotationExpr::Union { members: items, .. } => {
            for item in items {
                collect_source_member_projection_dependencies(item, module, resolver, members, dependencies);
            }
        }
        TypeAnnotationExpr::Tuple { elements, .. } => {
            for item in elements {
                match item {
                    TypeTupleItem::Element(element) => {
                        collect_source_member_projection_dependencies(&element.ty, module, resolver, members, dependencies);
                    }
                    TypeTupleItem::Expansion(expansion) => {
                        collect_source_member_projection_dependencies(&expansion.operand, module, resolver, members, dependencies);
                    }
                    TypeTupleItem::Remainder(rem) => {
                        if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &rem.remainder {
                            collect_source_member_projection_dependencies(constraint, module, resolver, members, dependencies);
                        }
                    }
                }
            }
        }
        TypeAnnotationExpr::Callable { parameters, result, .. } => {
            for item in parameters {
                match item {
                    TypeCallableItem::Parameter(parameter) => {
                        collect_source_member_projection_dependencies(&parameter.ty, module, resolver, members, dependencies);
                    }
                    TypeCallableItem::Expansion(expansion) => {
                        collect_source_member_projection_dependencies(&expansion.operand, module, resolver, members, dependencies);
                    }
                    TypeCallableItem::Remainder(rem) => {
                        if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &rem.remainder {
                            collect_source_member_projection_dependencies(constraint, module, resolver, members, dependencies);
                        }
                    }
                }
            }
            collect_source_member_projection_dependencies(result, module, resolver, members, dependencies);
        }
        TypeAnnotationExpr::Record { fields, .. } => {
            for field in fields {
                collect_source_member_projection_dependencies(&field.ty, module, resolver, members, dependencies);
            }
        }
        TypeAnnotationExpr::TypeLambda { body, .. } => {
            collect_source_member_projection_dependencies(body, module, resolver, members, dependencies);
        }
        TypeAnnotationExpr::ExactEnumCase {
            enum_target,
            generic_arguments,
            ..
        } => {
            collect_source_member_projection_dependencies(enum_target, module, resolver, members, dependencies);
            for argument in generic_arguments {
                collect_source_member_projection_dependencies(argument, module, resolver, members, dependencies);
            }
        }
        TypeAnnotationExpr::AssociatedTypeProjection { subject, .. } => {
            collect_source_member_projection_dependencies(subject, module, resolver, members, dependencies);
        }
        TypeAnnotationExpr::QualifiedAssociatedTypeProjection { subject, trait_ref, .. } => {
            collect_source_member_projection_dependencies(subject, module, resolver, members, dependencies);
            collect_source_member_projection_dependencies(&trait_ref.trait_type, module, resolver, members, dependencies);
            for specialization in &trait_ref.associated_specializations {
                collect_source_member_projection_dependencies(&specialization.value, module, resolver, members, dependencies);
            }
        }
        TypeAnnotationExpr::Reference(_)
        | TypeAnnotationExpr::Unit { .. }
        | TypeAnnotationExpr::Dynamic { .. }
        | TypeAnnotationExpr::Never { .. }
        | TypeAnnotationExpr::SelfType { .. }
        | TypeAnnotationExpr::Invalid { .. } => {}
    }
}

fn type_member_subject_owner(annotation: &TypeAnnotation, module: &ModuleId, resolver: &dyn TypeResolver) -> Option<DeclarationId> {
    match &annotation.expr {
        TypeAnnotationExpr::Reference(reference) => {
            let members = reference.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
            resolver.resolve_type_name(module, &reference.root, &members)
        }
        TypeAnnotationExpr::Application { origin, .. } => type_member_subject_owner(origin, module, resolver),
        _ => None,
    }
}

fn type_signature_cycle_components(
    graph: &BTreeMap<TypeSignatureDependencyNode, BTreeSet<TypeSignatureDependencyNode>>,
) -> Vec<BTreeSet<TypeSignatureDependencyNode>> {
    struct Tarjan<'a> {
        graph: &'a BTreeMap<TypeSignatureDependencyNode, BTreeSet<TypeSignatureDependencyNode>>,
        next: usize,
        indices: HashMap<TypeSignatureDependencyNode, usize>,
        lowlinks: HashMap<TypeSignatureDependencyNode, usize>,
        stack: Vec<TypeSignatureDependencyNode>,
        on_stack: BTreeSet<TypeSignatureDependencyNode>,
        components: Vec<BTreeSet<TypeSignatureDependencyNode>>,
    }

    fn visit(node: TypeSignatureDependencyNode, state: &mut Tarjan<'_>) {
        let index = state.next;
        state.next += 1;
        state.indices.insert(node.clone(), index);
        state.lowlinks.insert(node.clone(), index);
        state.stack.push(node.clone());
        state.on_stack.insert(node.clone());

        if let Some(dependencies) = state.graph.get(&node) {
            for dependency in dependencies {
                if !state.indices.contains_key(dependency) {
                    visit(dependency.clone(), state);
                    let lowlink = state.lowlinks[&node].min(state.lowlinks[dependency]);
                    state.lowlinks.insert(node.clone(), lowlink);
                } else if state.on_stack.contains(dependency) {
                    let lowlink = state.lowlinks[&node].min(state.indices[dependency]);
                    state.lowlinks.insert(node.clone(), lowlink);
                }
            }
        }

        if state.lowlinks[&node] == state.indices[&node] {
            let mut component = BTreeSet::new();
            while let Some(member) = state.stack.pop() {
                state.on_stack.remove(&member);
                component.insert(member.clone());
                if member == node {
                    break;
                }
            }
            if component.len() > 1 || state.graph.get(&node).is_some_and(|dependencies| dependencies.contains(&node)) {
                state.components.push(component);
            }
        }
    }

    let mut state = Tarjan {
        graph,
        next: 0,
        indices: HashMap::new(),
        lowlinks: HashMap::new(),
        stack: Vec::new(),
        on_stack: BTreeSet::new(),
        components: Vec::new(),
    };
    for node in graph.keys() {
        if !state.indices.contains_key(node) {
            visit(node.clone(), &mut state);
        }
    }
    state.components.sort_by(|left, right| left.first().cmp(&right.first()));
    state.components
}

pub(crate) fn implementation_owner(
    module: &ModuleId,
    implementation: &phalcom_ast::ast::ImplDef,
    resolver: &dyn TypeResolver,
    declarations: &DeclarationTypeTable,
    store: &TypeStore,
) -> Option<DeclarationId> {
    match &implementation.target.expr {
        TypeAnnotationExpr::Tuple { .. } => {
            return Some(crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Tuple));
        }
        TypeAnnotationExpr::Record { .. } => {
            return Some(crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Record));
        }
        _ => {}
    }
    let reference = match &implementation.target.expr {
        TypeAnnotationExpr::Reference(reference) => reference,
        TypeAnnotationExpr::Application { origin, .. } => match &origin.expr {
            TypeAnnotationExpr::Reference(reference) => reference,
            _ => return None,
        },
        _ => return None,
    };
    let members = reference.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
    let declaration = resolver.resolve_type_name(module, &reference.root, &members)?;
    if let Some(alias_form) = resolver.resolve_alias_form(&declaration)
        && let Some(owner) = crate::impls::structural_behavior_owner(store, alias_form)
    {
        return Some(owner);
    }
    if declarations.semantic_form(&declaration).is_some_and(|form| {
        matches!(
            form,
            crate::declarations::DeclarationSemanticForm::Intrinsic(
                crate::declarations::IntrinsicTypeApplication::Tuple
                    | crate::declarations::IntrinsicTypeApplication::Record
                    | crate::declarations::IntrinsicTypeApplication::Pack
            )
        )
    }) {
        return Some(declaration);
    }
    Some(declaration)
}

fn build_source_associated_surfaces(
    shards: &BTreeMap<ModuleId, Arc<ModuleSemanticStructureShard>>,
    previous_shards: &BTreeMap<ModuleId, Arc<ModuleSemanticStructureShard>>,
    resolver: &dyn TypeResolver,
    declarations: &DeclarationTypeTable,
    store: &mut TypeStore,
    recomputed_modules: &BTreeSet<ModuleId>,
    previous_impl_owners: &BTreeMap<crate::identity::ImplId, DeclarationId>,
) -> (
    BTreeSet<DeclarationId>,
    Vec<(DeclarationId, Arc<crate::associated::AssociatedSurface>, Arc<[SemanticDiagnostic]>)>,
    BTreeMap<crate::identity::ImplId, DeclarationId>,
) {
    let mut owners_to_recompute = BTreeSet::new();
    owners_to_recompute.extend(
        previous_impl_owners
            .iter()
            .filter(|(impl_id, _)| recomputed_modules.contains(&impl_id.module))
            .map(|(_, owner)| owner.clone()),
    );
    for module in recomputed_modules {
        for (source_index, sources) in [shards, previous_shards].into_iter().enumerate() {
            let Some(shard) = sources.get(module) else { continue };
            for statement in &shard.source.program.statements {
                match statement {
                    Statement::Class(class_def) => {
                        owners_to_recompute.insert(DeclarationId::new(module.clone(), class_def.name.clone().into()));
                    }
                    Statement::Data(data_def) => {
                        if data_def.is_intrinsic_facade() {
                            continue;
                        }
                        owners_to_recompute.insert(DeclarationId::new(module.clone(), data_def.name.clone().into()));
                    }
                    Statement::Impl(implementation) if matches!(implementation.kind, phalcom_ast::ast::ImplKind::Inherent) => {
                        if source_index == 0
                            && let Some(owner) = implementation_owner(module, implementation, resolver, declarations, store)
                        {
                            owners_to_recompute.insert(owner);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let mut impl_shells = BTreeMap::<DeclarationId, (Vec<crate::associated::AssociatedMemberShell>, HashSet<phalcom_common::selector::SelectorBase>)>::new();
    let mut current_impl_owners = BTreeMap::new();
    for (module_id, shard) in shards {
        for (statement_index, statement) in shard.source.program.statements.iter().enumerate() {
            let Statement::Impl(implementation) = statement else { continue };
            if !matches!(implementation.kind, phalcom_ast::ast::ImplKind::Inherent) {
                continue;
            }
            let Some(owner) = implementation_owner(module_id, implementation, resolver, declarations, store) else {
                continue;
            };
            let impl_id = crate::identity::ImplId::new(module_id.clone(), crate::identity::ImplLocalId(statement_index as u32));
            current_impl_owners.insert(impl_id, owner.clone());
            let (shells, behavior_bases) = impl_shells.entry(owner.clone()).or_default();
            for member in &implementation.members {
                if let phalcom_ast::ast::ImplMember::TypeMember(definition) = member {
                    let selector = crate::types::type_member::TypeMemberSelector::from_syntax(&definition.selector);
                    let base = match &selector {
                        crate::types::type_member::TypeMemberSelector::Named(name) => phalcom_common::selector::SelectorBase::Named(name.to_string()),
                        crate::types::type_member::TypeMemberSelector::Subscript => phalcom_common::selector::SelectorBase::Subscript,
                    };
                    let kind = if declarations
                        .intrinsic_type_owner(&owner)
                        .and_then(|owner| crate::types::type_member::lookup_intrinsic_type_operation(owner, &selector))
                        .is_some()
                    {
                        crate::associated::AssociatedFamilyKind::IntrinsicTypeMember
                    } else {
                        crate::associated::AssociatedFamilyKind::TypeMember
                    };
                    shells.push(crate::associated::AssociatedMemberShell {
                        base,
                        kind,
                        member: crate::associated::AssociatedMemberId::TypeMember(crate::types::type_member::TypeMemberDeclarationId::new(
                            owner.clone(),
                            selector,
                        )),
                        module: module_id.clone(),
                        source_range: definition.range,
                    });
                    continue;
                }
                let Some(behavior) = member.behavior() else { continue };
                let class_member = match behavior {
                    phalcom_ast::ast::BehaviorMember::Method(method) => phalcom_ast::ast::ClassMember::Method(method.clone()),
                    phalcom_ast::ast::BehaviorMember::Getter(getter) => phalcom_ast::ast::ClassMember::Getter(getter.clone()),
                    phalcom_ast::ast::BehaviorMember::Setter(setter) => phalcom_ast::ast::ClassMember::Setter(setter.clone()),
                    phalcom_ast::ast::BehaviorMember::Index(index) => phalcom_ast::ast::ClassMember::Index(index.clone()),
                };
                if let Some(callable) = crate::checker::declaration_signature::callable_id_for_member(&owner, &class_member) {
                    behavior_bases.insert(callable.selector.base);
                }
            }
            if implementation.generic_parameters.is_empty()
                && implementation
                    .where_clause
                    .as_ref()
                    .is_none_or(|where_clause| where_clause.constraints.is_empty())
            {
                for definition in &implementation.associated_type_data {
                    if let Some(shell) = associated_static_domain_shell(&owner, definition, module_id) {
                        shells.push(shell);
                    }
                }
            }
        }
    }

    let mut surfaces = Vec::new();
    for owner in &owners_to_recompute {
        let Some(shard) = shards.get(&owner.module) else { continue };
        let module_id = &owner.module;
        let Some(statement) = shard.source.program.statements.iter().find(|statement| match statement {
            Statement::Class(class_def) => class_def.name.as_str() == owner.name.as_ref(),
            Statement::Data(data_def) => data_def.name.as_str() == owner.name.as_ref(),
            _ => false,
        }) else {
            continue;
        };
        let (surface, diagnostics) = match statement {
            Statement::Class(class_def) => {
                let mut shells = Vec::new();
                for member in &class_def.members {
                    let phalcom_ast::ast::ClassMember::TypeMember(definition) = member else {
                        continue;
                    };
                    let selector = crate::types::type_member::TypeMemberSelector::from_syntax(&definition.selector);
                    let base = match &selector {
                        crate::types::type_member::TypeMemberSelector::Named(name) => phalcom_common::selector::SelectorBase::Named(name.to_string()),
                        crate::types::type_member::TypeMemberSelector::Subscript => phalcom_common::selector::SelectorBase::Subscript,
                    };
                    let kind = if declarations
                        .intrinsic_type_owner(owner)
                        .and_then(|owner| crate::types::type_member::lookup_intrinsic_type_operation(owner, &selector))
                        .is_some()
                    {
                        crate::associated::AssociatedFamilyKind::IntrinsicTypeMember
                    } else {
                        crate::associated::AssociatedFamilyKind::TypeMember
                    };
                    shells.push(crate::associated::AssociatedMemberShell {
                        base,
                        kind,
                        member: crate::associated::AssociatedMemberId::TypeMember(crate::types::type_member::TypeMemberDeclarationId::new(
                            owner.clone(),
                            selector,
                        )),
                        module: module_id.clone(),
                        source_range: definition.range,
                    });
                }
                shells.extend(
                    class_def
                        .associated_type_data
                        .iter()
                        .filter_map(|definition| associated_static_domain_shell(owner, definition, module_id)),
                );
                let mut behavior_bases = class_def
                    .members
                    .iter()
                    .filter_map(|member| crate::checker::declaration_signature::callable_id_for_member(owner, member))
                    .map(|callable| callable.selector.base)
                    .collect::<HashSet<_>>();
                let (impl_static_shells, impl_behavior_bases) = impl_shells.get(owner).cloned().unwrap_or_default();
                shells.extend(impl_static_shells);
                behavior_bases.extend(impl_behavior_bases);
                let base = Arc::new(crate::associated::AssociatedSurface::new(owner.clone()));
                let (surface, diagnostics) =
                    crate::associated::extend_associated_surface_with_shells(owner, base, &shells, &behavior_bases, &HashSet::new(), module_id);
                (surface, diagnostics)
            }
            Statement::Data(data_def) => {
                let (static_shells, behavior_bases) = impl_shells.get(owner).cloned().unwrap_or_default();
                let (base, base_diagnostics) = if data_def.is_sum() {
                    let variants = data_def
                        .cases()
                        .unwrap_or_default()
                        .iter()
                        .map(|variant| crate::identity::VariantId::new(owner.clone(), phalcom_ast::selector::selector_from_variant(variant)))
                        .collect::<Vec<_>>();
                    crate::associated::build_associated_surface(
                        owner,
                        Some(&variants),
                        &behavior_bases,
                        &HashSet::new(),
                        module_id,
                        Some(SemanticSourceSpan::new(module_id.clone(), data_def.range)),
                    )
                } else {
                    (crate::associated::build_data_associated_surface(owner), Arc::from([]))
                };
                let (surface, diagnostics) =
                    crate::associated::extend_associated_surface_with_shells(owner, base, &static_shells, &behavior_bases, &HashSet::new(), module_id);
                let mut combined = base_diagnostics.to_vec();
                combined.extend(diagnostics.iter().cloned());
                (surface, Arc::from(combined.into_boxed_slice()))
            }
            _ => continue,
        };
        surfaces.push((owner.clone(), surface, diagnostics));
    }
    (owners_to_recompute, surfaces, current_impl_owners)
}

fn associated_surface_admits(
    surfaces: &crate::associated::AssociatedFamilyTable,
    owner: &DeclarationId,
    member: &crate::associated::AssociatedMemberId,
) -> bool {
    surfaces.surfaces.get(owner).is_some_and(|surface| {
        surface
            .families
            .values()
            .any(|family| family.members.iter().any(|candidate| candidate == member))
    })
}

enum TypeMemberDemandService {
    Current,
    Cycle,
    NoProgress,
}

/// The single publication path for a checked type-member owner.
#[allow(clippy::too_many_arguments)]
fn publish_type_member_owner(
    db: &mut SemanticDb,
    store: &mut TypeStore,
    resolver: &LinkedTypeResolver,
    surfaces: &crate::associated::AssociatedFamilyTable,
    members: &mut crate::parent_map::ParentMap<crate::types::type_member::TypeMemberDeclarationId, Arc<crate::db::product::TypeMemberDeclarationProduct>>,
    declarations: &DeclarationTypeTable,
    id: &crate::types::type_member::TypeMemberDeclarationId,
    module: &ModuleId,
    definition: &phalcom_ast::ast::TypeMemberDef,
    source: &crate::source::ParsedModuleUnit,
    diagnostics: &mut BTreeMap<ModuleId, Vec<SemanticDiagnostic>>,
) -> Result<bool, QueryOutcome<()>> {
    let (product, owner_diagnostics) =
        crate::checker::type_member_declaration::build_type_member_declaration(store, &id.owner, definition, declarations, resolver, module);
    diagnostics.entry(module.clone()).or_default().extend(owner_diagnostics);
    let Some(product) = product else { return Ok(false) };
    match query_type_member_declaration(db, Some(source), Arc::new(product)) {
        QueryOutcome::Ready(product) => {
            members.insert(id.clone(), product);
            resolver.update_associated_type_members(surfaces, members);
            Ok(true)
        }
        QueryOutcome::Cancelled => Err(QueryOutcome::Cancelled),
        QueryOutcome::BudgetExceeded(report) => Err(QueryOutcome::BudgetExceeded(report)),
        QueryOutcome::Blocked(reason) => Err(QueryOutcome::Blocked(reason)),
        QueryOutcome::Failed(error) => Err(QueryOutcome::Failed(error)),
    }
}

/// Services exact type-member readiness requests through the session's canonical
/// member owner query. This never manufactures or publishes a second product.
#[allow(clippy::too_many_arguments)]
fn service_type_member_demands(
    db: &mut SemanticDb,
    store: &mut TypeStore,
    resolver: &LinkedTypeResolver,
    demands: &BTreeSet<QueryKey>,
    signature_graph: &BTreeMap<TypeSignatureDependencyNode, BTreeSet<TypeSignatureDependencyNode>>,
    alias_sources: &BTreeMap<DeclarationId, (ModuleId, phalcom_ast::ast::TypeAliasDef)>,
    type_aliases: &TypeAliasTable,
    declarations: &DeclarationTypeTable,
    surfaces: &crate::associated::AssociatedFamilyTable,
    members: &mut crate::parent_map::ParentMap<crate::types::type_member::TypeMemberDeclarationId, Arc<crate::db::product::TypeMemberDeclarationProduct>>,
    static_case_signatures: &crate::ordered_map::OrderedMap<crate::identity::StaticCaseId, Arc<crate::static_case::StaticCaseSignatureProduct>>,
    cycle_members: &BTreeSet<crate::types::type_member::TypeMemberDeclarationId>,
    member_definitions: &BTreeMap<crate::types::type_member::TypeMemberDeclarationId, (ModuleId, &phalcom_ast::ast::TypeMemberDef)>,
    sources: &BTreeMap<ModuleId, Arc<crate::source::ParsedModuleUnit>>,
    diagnostics: &mut BTreeMap<ModuleId, Vec<SemanticDiagnostic>>,
) -> Result<TypeMemberDemandService, QueryOutcome<()>> {
    let mut pending = BTreeSet::new();
    for demand in demands {
        let QueryKey::TypeMemberDeclaration(id) = demand else {
            return Ok(TypeMemberDemandService::NoProgress);
        };
        pending.insert(id.clone());
    }

    loop {
        pending.retain(|id| !members.contains_key(id));
        if pending.is_empty() {
            resolver.update_associated_type_members(surfaces, members);
            return Ok(TypeMemberDemandService::Current);
        }

        // Reconcile a current DB product into the session resolver view before
        // attempting any owner computation.
        let mut made_progress = false;
        for id in pending.iter() {
            let key = QueryKey::TypeMemberDeclaration(id.clone());
            if db.validate_ready(&key)
                && let Some(product) = db.product(&key).and_then(|product| product.as_type_member_declaration()).cloned()
                && product.id == *id
            {
                members.insert(id.clone(), product);
                resolver.update_associated_type_members(surfaces, members);
                made_progress = true;
                break;
            }
        }
        if made_progress {
            continue;
        }

        // Expand exact member prerequisites and run one ready owner through the
        // same canonical builder/query publication used by the initial scheduler.
        let mut ready = None;
        let candidates = pending.iter().cloned().collect::<Vec<_>>();
        for id in candidates {
            if cycle_members.contains(&id) {
                continue;
            }
            let dependencies = signature_graph.get(&TypeSignatureDependencyNode::Member(id.clone()));
            let Some(dependencies) = dependencies else {
                ready = Some(id.clone());
                break;
            };
            let mut prerequisites_ready = true;
            for dependency in dependencies {
                let available = match dependency {
                    TypeSignatureDependencyNode::Alias(alias) => !alias_sources.contains_key(alias) || type_aliases.contains_key(alias),
                    TypeSignatureDependencyNode::Header(header) => declarations.generic_binder_header(header).is_some(),
                    TypeSignatureDependencyNode::Member(member) => members.contains_key(member),
                    TypeSignatureDependencyNode::StaticCaseSignature(case) => static_case_signatures.contains_key(case),
                };
                if !available {
                    prerequisites_ready = false;
                    if let TypeSignatureDependencyNode::Member(member) = dependency
                        && member_definitions.contains_key(member)
                        && !cycle_members.contains(member)
                    {
                        pending.insert(member.clone());
                    }
                }
            }
            if prerequisites_ready {
                ready = Some(id);
                break;
            }
        }
        let Some(id) = ready else {
            return Ok(if pending.iter().any(|id| cycle_members.contains(id)) {
                TypeMemberDemandService::Cycle
            } else {
                TypeMemberDemandService::NoProgress
            });
        };
        if !associated_surface_admits(surfaces, &id.owner, &crate::associated::AssociatedMemberId::TypeMember(id.clone())) {
            pending.remove(&id);
            continue;
        }
        let Some((module, definition)) = member_definitions.get(&id) else {
            return Ok(TypeMemberDemandService::NoProgress);
        };
        let Some(source) = sources.get(module) else {
            return Ok(TypeMemberDemandService::NoProgress);
        };
        if !publish_type_member_owner(
            db,
            store,
            resolver,
            surfaces,
            members,
            declarations,
            &id,
            module,
            definition,
            source,
            diagnostics,
        )? {
            return Ok(TypeMemberDemandService::NoProgress);
        }
        pending.remove(&id);
    }
}

fn associated_static_domain_shell(
    owner: &DeclarationId,
    definition: &phalcom_ast::ast::AlgebraicDef,
    module: &ModuleId,
) -> Option<crate::associated::AssociatedMemberShell> {
    if definition.phase != phalcom_ast::ast::AlgebraicPhase::TypeOnly || !definition.generic_parameters.is_empty() || definition.where_clause.is_some() {
        return None;
    }
    let id = crate::identity::AssociatedStaticDomainDeclarationId::new(owner.clone(), definition.name.clone());
    Some(crate::associated::AssociatedMemberShell {
        base: phalcom_common::selector::SelectorBase::Named(definition.name.clone()),
        kind: crate::associated::AssociatedFamilyKind::StaticDomain,
        member: crate::associated::AssociatedMemberId::StaticDomain(id),
        module: module.clone(),
        source_range: definition.range,
    })
}

fn class_declares_constructor_lifecycle(class_def: &phalcom_ast::ast::ClassDef) -> bool {
    class_def.members.iter().any(|member| {
        matches!(member, ClassMember::Method(method) if method.is_constructor || method.attributes.iter().any(|attribute| attribute.name == "constructor"))
    }) || class_def.attributes.iter().any(|attribute| matches!(attribute.name.as_str(), "construct" | "data"))
        || class_def
            .members
            .iter()
            .any(|member| matches!(member, ClassMember::Field(field) if !field.is_static && field.default.is_some()))
}

fn strip_constructor_field_prefix(name: &str) -> String {
    name.strip_prefix('_').unwrap_or(name).to_string()
}

fn constructor_field_input_fingerprint(fields: &[(&phalcom_ast::ast::FieldDef, FieldId)], source_text: &str) -> crate::db::ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    for (field, field_id) in fields {
        field_id.hash(&mut hasher);
        field.range.hash(&mut hasher);
        source_text.get(field.range.start..field.range.end).hash(&mut hasher);
    }
    crate::db::ProductFingerprint::new(hasher.finish())
}

fn initializer_signature_for_factory(factory: &CallableSemanticSignature, initializer: CallableId) -> CallableSemanticSignature {
    let parameters = factory
        .parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            let mut parameter = parameter.clone();
            parameter.id = crate::identity::CallableParameterId::new(initializer.clone(), index as u32);
            parameter.source = None;
            parameter
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    CallableSemanticSignature {
        callable: initializer.clone(),
        owner: factory.owner.clone(),
        side: DispatchSide::Instance,
        selector: initializer.selector.clone(),
        // Constructor lowering does not retain method-local generic binders on
        // the hidden initializer. Owner-level type terms remain canonical.
        generics: None,
        parameters,
        declared_return: factory.declared_return.clone(),
        return_validation: ReturnContractValidation::NotApplicable,
        inferred_return: None,
        source: None,
        implementation: factory.implementation,
        native_id: None,
        effects: factory.effects,
        raises: factory.raises,
        flow: factory.flow,
        lifecycle: factory.lifecycle,
    }
}

fn generated_constructor_signatures(
    owner: &DeclarationId,
    factory: CallableId,
    initializer: CallableId,
    parameter_fields: Vec<(String, String, FieldId, crate::declaration_type::DeclaredTypeFact)>,
    store: &mut TypeStore,
) -> (CallableSemanticSignature, CallableSemanticSignature, Vec<ConstructorParameterField>) {
    let self_type = store.self_type(crate::types::parameter::SelfTypeTerm {
        owner: owner.clone(),
        side: DispatchSide::Class,
        role: crate::types::parameter::SelfRole::InstanceType,
    });
    let declared_return = crate::declaration_type::DeclaredTypeFact::known(
        crate::types::parameter::TypeTerm::Canonical(self_type),
        crate::declaration_type::DeclaredTypeBasis::ConstructorSemantics,
    );
    let mut factory_parameters = Vec::with_capacity(parameter_fields.len());
    let mut parameter_bindings = Vec::with_capacity(parameter_fields.len());
    for (index, (local_name, label, field, declared_type)) in parameter_fields.into_iter().enumerate() {
        let parameter = crate::identity::CallableParameterId::new(factory.clone(), index as u32);
        factory_parameters.push(CallableParameterSemantic::new(parameter.clone(), local_name, declared_type).with_label(label));
        parameter_bindings.push(ConstructorParameterField { parameter, field });
    }
    let factory_signature = CallableSemanticSignature {
        callable: factory.clone(),
        owner: owner.clone(),
        side: DispatchSide::Class,
        selector: factory.selector.clone(),
        generics: None,
        parameters: factory_parameters.into_boxed_slice(),
        declared_return,
        return_validation: ReturnContractValidation::NotApplicable,
        inferred_return: None,
        source: None,
        implementation: phalcom_common::modifier::ImplementationKind::Phalcom,
        native_id: None,
        effects: phalcom_native_meta::EffectSpec::Unknown,
        raises: phalcom_native_meta::RaisesSpec::Unknown,
        flow: phalcom_native_meta::ReturnFlowSpec::Value,
        lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
    };
    let initializer_signature = initializer_signature_for_factory(&factory_signature, initializer);
    (factory_signature, initializer_signature, parameter_bindings)
}

fn compute_module_fingerprint(unit: &ParsedModuleUnit) -> u64 {
    let mut hasher = DefaultHasher::new();
    unit.id.hash(&mut hasher);
    unit.text.hash(&mut hasher);
    hasher.finish()
}

fn callable_type_annotations(member: &phalcom_ast::ast::ClassMember) -> Vec<&phalcom_ast::ast::TypeAnnotation> {
    crate::checker::declaration_signature::callable_type_annotations(member)
}

fn field_lifecycle_source_fingerprint(unit: &ParsedModuleUnit) -> u64 {
    let mut hasher = DefaultHasher::new();
    unit.id.hash(&mut hasher);
    for statement in &unit.program.statements {
        let Statement::Class(class_def) = statement else { continue };
        class_def.name.hash(&mut hasher);
        for member in &class_def.members {
            if let ClassMember::Field(field) = member {
                format!("{field:?}").hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

/// Builds checker-visible module bindings for one static initializer.
/// Runtime bindings retain their ordinary checker facts so access can be
/// rejected precisely. Constant bindings are unknown during dependency
/// discovery and acquire their type only from a published checked constant
/// product during the final check.
pub(crate) fn static_constant_binding_states(
    catalog: &crate::module_bindings::ModuleStaticBindingCatalog,
    runtime_states: &BTreeMap<phalcom_modules::linker::SymbolId, crate::checker::analysis::BindingState>,
    constants: Option<&crate::static_constant::StaticConstantTable>,
) -> BTreeMap<phalcom_modules::linker::SymbolId, crate::checker::analysis::BindingState> {
    let mut states = BTreeMap::new();
    for (visible_name, binding) in catalog.visible_bindings() {
        if binding.phase == crate::module_bindings::ModuleBindingPhase::Runtime {
            if let Some(state) = runtime_states.get(&binding.symbol) {
                states.insert(binding.symbol.clone(), state.clone());
            }
            continue;
        }

        let result_type = constants
            .and_then(|constants| constants.get(&crate::static_constant::StaticConstantId(binding.symbol.clone())))
            .and_then(|product| product.checked.as_ref().and_then(|c| c.result_type));
        let current = result_type.map_or_else(
            || crate::types::evidence::TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::NoTypeEvidence),
            |ty| crate::types::evidence::TypeKnowledge::established(ty, crate::types::evidence::EvidenceOrigin::Flow),
        );
        let seed = crate::checker::binding::BindingSeed {
            name: visible_name.to_owned(),
            parameter: None,
            range: binding.binding_range,
            contract: None,
            current: current.clone(),
            denotation: None,
            causal_invalidity: crate::checker::causal::CausalInvalidity::Clean,
            mutable: false,
            initialization: crate::checker::flow::InitializationState::DefinitelyInitialized,
        };
        let state = crate::checker::analysis::BindingState::from_seed(
            crate::identity::BindingId(0),
            seed,
            current,
            crate::checker::binding::BindingConsistency::Unconstrained,
        );
        states.insert(binding.symbol.clone(), state);
    }
    states
}

fn query_key_module_for_worklist(key: &QueryKey) -> Option<&ModuleId> {
    match key {
        QueryKey::ParsedModule(module)
        | QueryKey::UnlinkedInterface(module)
        | QueryKey::ResolvedImports(module)
        | QueryKey::LinkedInterface(module)
        | QueryKey::SemanticComponent(module)
        | QueryKey::SourceStructure(module)
        | QueryKey::AdvisoryModule(module)
        | QueryKey::ModuleDiagnostics(module)
        | QueryKey::ModuleMetadata(module)
        | QueryKey::LinkedName(module, _)
        | QueryKey::PublicExport(module, _) => Some(module),
        QueryKey::DeclarationShell(declaration)
        | QueryKey::DeclarationSurface(declaration)
        | QueryKey::TraitHeader(declaration)
        | QueryKey::TraitSurface(declaration)
        | QueryKey::HierarchyEdge(declaration)
        | QueryKey::DataDeclaration(declaration)
        | QueryKey::EnumDeclaration(declaration)
        | QueryKey::EnumRequirements(declaration)
        | QueryKey::ConstructorInitialization(declaration)
        | QueryKey::AssociatedSurface(declaration)
        | QueryKey::StaticDefinition(declaration) => Some(&declaration.module),
        QueryKey::StaticConstant(constant) => Some(&constant.0.module),
        QueryKey::ModuleStaticBindingCatalog(module) | QueryKey::ModuleStaticBindings(module) => Some(module),
        QueryKey::CheckedStaticBody(body) => Some(&body.0.module),
        QueryKey::AttributeArgument(argument) => Some(match &argument.owner {
            crate::attribute_analysis::AttributeOwnerId::Module(module) => module,
            crate::attribute_analysis::AttributeOwnerId::Declaration(declaration) => &declaration.module,
            crate::attribute_analysis::AttributeOwnerId::Callable(callable) => callable.module(),
            crate::attribute_analysis::AttributeOwnerId::Field(field) => &field.owner.module,
        }),
        QueryKey::TypeMemberDeclaration(member) => Some(&member.owner.module),
        QueryKey::AssociatedStaticDomainDeclaration(domain) => Some(&domain.owner.module),
        QueryKey::StaticDomainCaseCatalog(domain) => Some(match domain {
            crate::identity::StaticDomainDeclarationId::Nominal(declaration) => &declaration.module,
            crate::identity::StaticDomainDeclarationId::Associated(declaration) => &declaration.owner.module,
        }),
        QueryKey::StaticCaseSignature(case) => Some(match &case.domain {
            crate::identity::StaticDomainDeclarationId::Nominal(declaration) => &declaration.module,
            crate::identity::StaticDomainDeclarationId::Associated(declaration) => &declaration.owner.module,
        }),
        QueryKey::ResolvedImport(site) => Some(&site.importer),
        QueryKey::FieldSignature(field) => Some(&field.owner.module),
        QueryKey::CallableDefinition(callable)
        | QueryKey::CallableSignature(callable)
        | QueryKey::CallableBody(callable)
        | QueryKey::CallableEffects(callable)
        | QueryKey::CallableControl(callable)
        | QueryKey::CallableTermination(callable)
        | QueryKey::CallableContracts(callable)
        | QueryKey::VerificationConditions(callable)
        | QueryKey::SourceFormalAttachment(callable)
        | QueryKey::AdvisoryCallable(callable) => Some(callable.module()),
        QueryKey::ConformanceDispatch(impl_id) => Some(&impl_id.module),
        QueryKey::GenericConformance(key) => Some(&key.trait_ref.declaration.module),
        QueryKey::TraitDispatchBucket(bucket) => match &bucket.target_family {
            crate::trait_dispatch::TraitDispatchTargetFamily::Declaration(decl) => Some(&decl.module),
            crate::trait_dispatch::TraitDispatchTargetFamily::ExactEnumCase(variant) => Some(&variant.owner.module),
        },
    }
}

fn semantic_target_for_linked_symbol(symbol: &SymbolId, nominal_declarations: &HashSet<DeclarationId>) -> SemanticTargetId {
    let declaration = DeclarationId::new(symbol.module.clone(), symbol.name.clone());
    if nominal_declarations.contains(&declaration) {
        SemanticTargetId::Declaration(declaration)
    } else {
        SemanticTargetId::ModuleBinding(symbol.clone())
    }
}

struct SourceSemanticIndexInputs<'a> {
    sources: &'a BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    universe_source_provider: &'a phalcom_modules::UniverseSourceProvider,
    include_workspace_source_semantics: bool,
    include_canonical_source_semantics: bool,
    include_canonical_presentation_sources: bool,
    callable_analyses: &'a HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>,
    callable_signatures: &'a CallableSignatureTable,
    import_products: &'a BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>,
    require_canonical_import_products: bool,
    linked: &'a LinkedProgram,
    type_resolver: &'a dyn TypeResolver,
    nominal_declarations: &'a HashSet<DeclarationId>,
    previous: Option<&'a SourceSemanticIndex>,
    rebuild_modules: &'a BTreeSet<ModuleId>,
    retired_modules: &'a BTreeSet<ModuleId>,
    current_modules: &'a BTreeSet<ModuleId>,
    analysis_callables: &'a BTreeMap<ModuleId, BTreeSet<crate::identity::CallableId>>,
    import_sites_by_module: &'a BTreeMap<ModuleId, BTreeSet<phalcom_modules::identity::ImportSiteId>>,
    trait_surfaces: &'a TraitSurfaceTable,
}

fn build_source_semantic_index(inputs: SourceSemanticIndexInputs<'_>) -> (SourceSemanticIndex, BTreeMap<ModuleId, Arc<str>>) {
    let SourceSemanticIndexInputs {
        sources,
        universe_source_provider,
        include_workspace_source_semantics,
        include_canonical_source_semantics,
        include_canonical_presentation_sources,
        callable_analyses,
        callable_signatures,
        import_products,
        require_canonical_import_products,
        linked,
        type_resolver,
        nominal_declarations,
        previous,
        rebuild_modules,
        retired_modules,
        current_modules,
        analysis_callables,
        import_sites_by_module,
        trait_surfaces,
    } = inputs;
    // Canonical Universe modules are source-owned presentation inputs: index
    // their declarations for navigation without linking or deeply analyzing
    // their bodies as part of an ordinary workspace update. On an incremental
    // update, retain unchanged shards and materialize only the requested
    // rebuild set; cloning/scanning the complete source map here would defeat
    // the persistent source-index contract.
    let mut presentation_sources = BTreeMap::new();
    let provider = universe_source_provider;
    let canonical_modules = if include_canonical_source_semantics || include_canonical_presentation_sources {
        provider
            .nodes()
            .iter()
            .map(|node| {
                let path = phalcom_modules::ModulePath::from_components(
                    node.path
                        .iter()
                        .map(|component| phalcom_modules::ModuleComponent::from_identifier(component).expect("canonical Universe component"))
                        .collect::<Vec<_>>(),
                );
                ModuleId::universe(path)
            })
            .collect::<BTreeSet<_>>()
    } else {
        BTreeSet::new()
    };
    let mut index_sources = BTreeMap::new();
    if include_workspace_source_semantics {
        if previous.is_none() {
            index_sources.extend(sources.iter().map(|(module, source)| (module.clone(), source.clone())));
        } else {
            for module in rebuild_modules {
                if let Some(source) = sources.get(module) {
                    index_sources.insert(module.clone(), source.clone());
                }
            }
        }
    }
    for module in &canonical_modules {
        if sources.contains_key(module) || (previous.is_some() && !rebuild_modules.contains(module)) {
            continue;
        }
        let parsed = provider.load_parsed(module).expect("canonical Universe presentation source must load");
        if include_canonical_presentation_sources {
            presentation_sources.insert(module.clone(), parsed.text.clone());
        }
        if include_canonical_source_semantics {
            index_sources.insert(module.clone(), parsed);
        }
    }

    // Import products are keyed by the authored site. Retain only products
    // belonging to rebuilt modules in this publication context; unchanged
    // shards already carry their resolved source identities.
    let mut context = SourceIndexContext {
        require_canonical_import_products,
        import_products: rebuild_modules
            .iter()
            .flat_map(|module| import_sites_by_module.get(module).into_iter().flatten())
            .filter_map(|site| import_products.get(site).map(|product| (site.clone(), product.clone())))
            .collect(),
        ..SourceIndexContext::default()
    };
    for (module, source) in &index_sources {
        for statement in &source.program.statements {
            match statement {
                Statement::Trait(trait_def) => {
                    let declaration = DeclarationId::new(module.clone(), trait_def.name.clone().into());
                    let Some(surface) = trait_surfaces.get(&declaration) else { continue };
                    for member in &trait_def.members {
                        if let Some(behavior) = member.behavior() {
                            collect_associated_projection_targets_from_behavior(behavior, module, surface, &mut context.associated_projection_targets);
                        }
                        if let phalcom_ast::ast::TraitMember::Property(property) = member {
                            collect_associated_projection_targets(&property.annotation, module, surface, &mut context.associated_projection_targets);
                        }
                    }
                }
                Statement::Impl(impl_def) => {
                    let phalcom_ast::ast::ImplKind::Conformance { trait_ref, .. } = &impl_def.kind else {
                        continue;
                    };
                    let trait_ranges = resolve_type_reference_targets(module, &source.program, type_resolver);
                    let Some(trait_declaration) = trait_ranges
                        .iter()
                        .filter(|(range, _)| range.start >= trait_ref.range.start && range.end <= trait_ref.range.end)
                        .map(|(_, declaration)| declaration)
                        .next()
                    else {
                        continue;
                    };
                    let Some(surface) = trait_surfaces.get(trait_declaration) else { continue };
                    for member in &impl_def.members {
                        match member {
                            phalcom_ast::ast::ImplMember::AssociatedTypeBinding(binding) => {
                                if let Some(requirement) = surface.associated_type_by_name(&binding.name) {
                                    context
                                        .associated_type_targets
                                        .insert((module.clone(), binding.name_range), requirement.requirement.clone());
                                }
                                collect_associated_projection_targets(&binding.value, module, surface, &mut context.associated_projection_targets);
                            }
                            phalcom_ast::ast::ImplMember::Behavior(behavior) => {
                                collect_associated_projection_targets_from_behavior(behavior, module, surface, &mut context.associated_projection_targets);
                            }
                            phalcom_ast::ast::ImplMember::Delegation(_) | phalcom_ast::ast::ImplMember::TypeMember(_) => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
    for (module, source) in &index_sources {
        if previous.is_some() && !rebuild_modules.contains(module) {
            continue;
        }
        for (range, declaration) in resolve_type_reference_targets(module, &source.program, type_resolver) {
            context.type_reference_targets.insert((module.clone(), range), declaration);
        }
        for (range, member) in resolve_type_member_projection_targets(module, &source.program, type_resolver) {
            context.type_member_projection_targets.insert((module.clone(), range), member);
        }
        let type_parameters = resolve_type_parameter_source_targets(module, &source.program, type_resolver);
        for (range, name) in type_parameters.declarations {
            context.type_parameter_declarations.insert((module.clone(), range), name);
        }
        for (range, declaration_range) in type_parameters.references {
            context.type_parameter_reference_targets.insert((module.clone(), range), declaration_range);
        }
    }
    let mut imported_target_modules = BTreeSet::new();
    for product in context.import_products.values() {
        if let Ok(target) = &product.target {
            imported_target_modules.insert(target.clone());
        }
        imported_target_modules.extend(product.prefixes.iter().map(|prefix| prefix.module.clone()));
    }
    let context_modules = index_sources
        .keys()
        .cloned()
        .chain(imported_target_modules.iter().cloned())
        .collect::<BTreeSet<_>>();
    for module in context_modules {
        let Some(linked_module) = linked.modules.get(&module) else {
            continue;
        };
        context.modules.entry(module.path.to_string()).or_insert_with(|| module.clone());
        context.modules.entry(module.to_string()).or_insert_with(|| module.clone());
        for export in linked_module.interface.exports.values() {
            match &export.target {
                LinkedExportTarget::Binding(symbol) => {
                    context.targets.insert(
                        (module.clone(), export.public_name.to_string()),
                        semantic_target_for_linked_symbol(symbol, nominal_declarations),
                    );
                }
                LinkedExportTarget::Module(target) => {
                    context
                        .targets
                        .insert((module.clone(), export.public_name.to_string()), SemanticTargetId::Module(target.clone()));
                }
            }
        }
    }
    let scopes: BTreeMap<ModuleId, crate::source_index::SourceScopeIndex> = index_sources
        .iter()
        .filter(|(module, _)| previous.is_none_or(|_| rebuild_modules.contains(*module)))
        .map(|(module, source)| (module.clone(), build_source_scope_index(module.clone(), &source.program, &context)))
        .collect();
    if let Some(previous) = previous {
        for module in imported_target_modules {
            if rebuild_modules.contains(&module) {
                continue;
            }
            if let Some(previous_index) = previous.module(&module) {
                for callable in previous_index.structure.callable_sources.values() {
                    if let Some(owner) = callable.id.try_declaration_owner() {
                        context
                            .callable_targets
                            .insert((owner.clone(), callable.id.selector.clone()), callable.id.clone());
                    }
                }
            }
        }
    }
    for structure in scopes.values() {
        for callable in structure.callable_sources.values() {
            if let Some(owner) = callable.id.try_declaration_owner() {
                context
                    .callable_targets
                    .insert((owner.clone(), callable.id.selector.clone()), callable.id.clone());
            }
        }
    }
    for (module, source) in &index_sources {
        if previous.is_some() && !rebuild_modules.contains(module) {
            continue;
        }
        for statement in &source.program.statements {
            let phalcom_ast::ast::Statement::Class(class) = statement else { continue };
            context
                .targets
                .entry((module.clone(), class.name.clone()))
                .or_insert_with(|| SemanticTargetId::Declaration(DeclarationId::new(module.clone(), class.name.clone().into())));
        }
    }
    let mut index = previous.cloned().unwrap_or_else(SourceSemanticIndex::empty);
    if let Some(previous) = previous {
        index.set_stats(crate::source_index::SourceIndexUpdateStats::default());
        let rebuilt_retained = rebuild_modules
            .iter()
            .filter(|module| current_modules.contains(*module) && previous.module(module).is_some())
            .count();
        let retired_count = retired_modules.iter().filter(|module| previous.module(module).is_some()).count();
        let reused = previous.len().saturating_sub(rebuilt_retained).saturating_sub(retired_count);
        let mut stats = index.stats();
        stats.source_modules_reused = reused;
        index.set_stats(stats);
        for module in retired_modules {
            index.retire_module_shard(module);
        }
    }

    for (module, scope) in scopes {
        let Some(source) = index_sources.get(&module) else {
            continue;
        };
        let mut module_analyses = analysis_callables
            .get(&module)
            .into_iter()
            .flat_map(|callables| callables.iter())
            .filter_map(|callable| {
                callable_analyses.get(callable).or_else(|| {
                    let alternate_side = match callable.side {
                        crate::identity::DispatchSide::Instance => crate::identity::DispatchSide::Class,
                        crate::identity::DispatchSide::Class => crate::identity::DispatchSide::Instance,
                    };
                    let owner = callable.try_declaration_owner()?;
                    let alternate = crate::identity::CallableId::new(owner.clone(), callable.selector.clone(), alternate_side);
                    callable_analyses.get(&alternate)
                })
            })
            .map(Arc::as_ref)
            .collect::<Vec<_>>();
        module_analyses.sort_by_key(|analysis| analysis.callable.clone());
        for analysis in &module_analyses {
            if matches!(&analysis.callable.selector.base, phalcom_common::selector::SelectorBase::Named(name) if name == "<main>") {
                context.body_source_owners.insert(
                    analysis.callable.clone(),
                    crate::identity::SourceOwner::Module(analysis.callable.module().clone()),
                );
                continue;
            }
            let source_callable = callable_signatures.id_for_body(&analysis.callable).unwrap_or_else(|| analysis.callable.clone());
            if source_callable != analysis.callable {
                context
                    .body_source_owners
                    .insert(analysis.callable.clone(), crate::identity::SourceOwner::Callable(source_callable));
            }
        }
        let (shard, incidents) = crate::source_index::ModuleSourceIndex::from_scope_index_with_formal(scope, source, Some(&context), &module_analyses);
        index.replace_module_incidents(module.clone(), incidents);
        index.replace_module_shard(module, Arc::new(shard));
    }

    (index, presentation_sources)
}

/// Builds the advisory workspace from the same source/formal products that
/// will be published in the immutable snapshot. Missing source attachments
/// reduce advisory coverage only; they never prevent formal publication.
struct AdvisoryWorkspaceInputs<'a> {
    sources: &'a BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    source_index: &'a SourceSemanticIndex,
    callable_analyses: &'a HashMap<CallableId, Arc<crate::checker::CallableAnalysis>>,
    store: &'a TypeStore,
    declarations: &'a DeclarationTypeTable,
    callable_signatures: &'a CallableSignatureTable,
    dispatch: &'a SurfaceDispatchResolver,
    hierarchy: &'a MapTypeHierarchy,
    linked: &'a LinkedProgram,
    previous: Option<&'a AdvisoryWorkspace>,
    budget: QueryBudget,
    cancel: &'a CancellationToken,
}

fn build_advisory_workspace(inputs: AdvisoryWorkspaceInputs<'_>) -> AdvisoryWorkspace {
    let AdvisoryWorkspaceInputs {
        sources,
        source_index,
        callable_analyses,
        store,
        declarations,
        callable_signatures,
        dispatch,
        hierarchy,
        linked,
        previous,
        budget,
        cancel,
    } = inputs;
    let builtins = AdvisoryBuiltins::from_declarations(declarations);
    let resolve_module_member = |receiver: &crate::advisory::ValueShape, name: &str| {
        let crate::advisory::ValueShape::Module(module) = receiver else {
            return None;
        };
        if let Some(export) = linked.modules.get(module).and_then(|linked_module| linked_module.interface.exports.get(name)) {
            match &export.target {
                LinkedExportTarget::Module(target) => return Some(crate::advisory::ValueShape::Module(target.clone())),
                LinkedExportTarget::Binding(symbol) => {
                    let declaration = DeclarationId::new(symbol.module.clone(), symbol.name.clone());
                    if dispatch.surfaces().contains_key(&declaration) {
                        return Some(crate::advisory::ValueShape::ClassObject(declaration));
                    }
                }
            }
        }
        dispatch
            .surfaces()
            .keys()
            .find(|declaration| declaration.module == *module && declaration.name.as_ref() == name)
            .cloned()
            .map(crate::advisory::ValueShape::ClassObject)
    };
    let resolve_callable_for_shape = |receiver: &crate::advisory::ValueShape, name: &str, args: &[PackItem]| {
        let slots = args
            .iter()
            .map(|arg| match arg {
                PackItem::Positional { .. } | PackItem::Expand { .. } => Some(phalcom_common::selector::SelectorSlot::Positional),
                PackItem::Labeled {
                    label: PackLabel::Static { text, .. },
                    ..
                } => Some(phalcom_common::selector::SelectorSlot::Label(text.clone())),
                PackItem::Labeled {
                    label: PackLabel::Computed { .. },
                    ..
                } => None,
            })
            .collect::<Option<Vec<_>>>()?;
        let selector = Selector::method(name, slots).ok()?;
        let (owner, side) = match receiver {
            crate::advisory::ValueShape::ClassObject(owner) => (owner, DispatchSide::Class),
            crate::advisory::ValueShape::Instance(owner) => (owner, DispatchSide::Instance),
            _ => return None,
        };
        dispatch.resolve_callable_id(hierarchy, owner, side, &selector)
    };
    let resolve_formal_call_result = |callable: &CallableId, receiver: Option<&crate::advisory::ValueShape>| {
        let signature = callable_signatures.get(callable)?;
        let return_knowledge = signature.published_return_knowledge();
        let shape = receiver.map_or_else(
            || advisory_shape_from_formal(store, &return_knowledge),
            |receiver| advisory_shape_from_formal_for_receiver(store, &return_knowledge, receiver),
        );
        (!matches!(shape, crate::advisory::ValueShape::Unknown)).then(|| {
            AdvisoryFact::new(shape, AdvisoryConfidence::Interprocedural)
                .derive(AdvisoryConfidence::Interprocedural, AdvisoryOrigin::Callable(signature.callable.clone()))
        })
    };
    let advisory_transfer_target = |callable: &CallableId| {
        let is_constructor = callable_signatures.get(callable).is_some_and(|signature| signature.is_constructor());
        if is_constructor && callable.side == DispatchSide::Class {
            CallableId::new(callable.owner.clone(), callable.selector.clone(), DispatchSide::Instance)
        } else {
            callable.clone()
        }
    };
    let resolve_method_family = |receiver: &crate::advisory::ValueShape, spec: &phalcom_ast::ast::NormalizedSelectorSpec| {
        let pattern = match spec {
            phalcom_ast::ast::NormalizedSelectorSpec::Pattern(pattern) => pattern,
            phalcom_ast::ast::NormalizedSelectorSpec::Exact(_) => return None,
        };
        let (owner, side) = match receiver {
            crate::advisory::ValueShape::ClassObject(owner) => (owner, DispatchSide::Class),
            crate::advisory::ValueShape::Instance(owner) => (owner, DispatchSide::Instance),
            _ => return None,
        };
        let mut exact = Vec::new();
        let mut rest_candidates = Vec::new();
        for dispatch_owner in dispatch.dispatch_owners(hierarchy, owner, side) {
            if let Some(surface) = dispatch.get_surface(&dispatch_owner.declaration) {
                let members = surface.surface(dispatch_owner.side);
                let mut selectors = members.callable_signatures.keys().collect::<Vec<_>>();
                selectors.sort();
                for selector in selectors {
                    if !pattern.matches(selector) {
                        continue;
                    }
                    let Some(callable) = members.callables_by_selector.get(selector).cloned() else {
                        continue;
                    };
                    let signature = &members.callable_signatures[selector];
                    if signature.parameters.iter().any(|parameter| parameter.is_rest()) {
                        rest_candidates.push(callable);
                    } else {
                        exact.push((selector.clone(), callable));
                    }
                }
            }
        }
        exact.sort_by(|left, right| left.0.cmp(&right.0));
        exact.dedup();
        rest_candidates.sort();
        rest_candidates.dedup();
        Some(crate::advisory::CapturedMethodFamilyShape {
            source_behavior: owner.clone(),
            pattern: pattern.clone(),
            exact: exact.into_boxed_slice(),
            rest_candidates: rest_candidates.into_boxed_slice(),
        })
    };
    let mut formal_returns = BTreeMap::new();
    let mut ordered_analyses = callable_analyses.values().cloned().collect::<Vec<_>>();
    ordered_analyses.sort_by(|left, right| left.callable.cmp(&right.callable));
    for analysis in &ordered_analyses {
        let fact = callable_signatures
            .get_for_body(&analysis.callable)
            .map(|signature| {
                let return_knowledge = signature.published_return_knowledge();
                let shape = if signature.is_constructor() {
                    let receiver = crate::advisory::ValueShape::ClassObject(signature.owner.clone());
                    advisory_shape_from_formal_for_receiver(store, &return_knowledge, &receiver)
                } else {
                    advisory_shape_from_formal(store, &return_knowledge)
                };
                AdvisoryFact::new(shape, AdvisoryConfidence::Interprocedural)
                    .derive(AdvisoryConfidence::Interprocedural, AdvisoryOrigin::Callable(signature.callable.clone()))
            })
            .unwrap_or_else(AdvisoryFact::unknown);
        formal_returns.insert(analysis.callable.clone(), fact);
    }

    let mut parameter_facts = BTreeMap::new();
    let mut advisory_returns = formal_returns.clone();
    let mut modules = BTreeMap::new();
    let mut callables = BTreeMap::new();
    let mut advisory_budget = budget;

    let (workspace_partial, solver_status, solver_converged) = loop {
        if cancel.is_cancelled() {
            break (true, AdvisoryProductStatus::Cancelled, false);
        }
        if let Err(_report) = advisory_budget.charge_step() {
            break (true, AdvisoryProductStatus::BudgetExceeded, false);
        }
        let mut next_modules = BTreeMap::new();
        let mut next_callables = BTreeMap::new();
        let mut next_partial = false;
        let mut parameter_contributions = crate::advisory::AdvisoryParameterContributions::default();

        for (module, source) in sources {
            let Some(module_index) = source_index.module(module) else {
                next_partial = true;
                continue;
            };
            let scope_index = module_index.structure.as_ref();
            let mut fields = advisory_field_facts(AdvisoryFieldFactsInputs {
                source,
                scope_index,
                builtins: &builtins,
                callable_returns: &advisory_returns,
                resolve_callable_for_shape: Some(&resolve_callable_for_shape),
                resolve_formal_call_result: Some(&resolve_formal_call_result),
                advisory_transfer_target: Some(&advisory_transfer_target),
                resolve_module_member: Some(&resolve_module_member),
                resolve_method_family: Some(&resolve_method_family),
            });
            let mut expressions = BTreeMap::new();
            let mut bindings = BTreeMap::new();
            let mut parameters = BTreeMap::new();
            let mut targets = BTreeMap::new();
            let mut module_partial = false;

            for (site, target) in &scope_index.targets {
                targets.insert(site.clone(), advisory_target_resolution(site, target));
            }
            for attachment in module_index.attachments.values() {
                for (site, target) in &attachment.exact_targets {
                    targets.insert(site.clone(), advisory_target_resolution(site, target));
                }
            }

            let target_site_for_range = |range: SourceRange| {
                let candidates = module_index
                    .occurrences
                    .all()
                    .iter()
                    .filter(|occurrence| occurrence.range == range)
                    .map(|occurrence| occurrence.site.clone())
                    .collect::<Vec<_>>();
                (candidates.len() == 1).then(|| candidates[0].clone())
            };

            let mut member_bodies = BTreeMap::new();
            for statement in &source.program.statements {
                let Statement::Class(class) = statement else { continue };
                let declaration = DeclarationId::new(module.clone(), class.name.clone().into());
                for member in &class.members {
                    if let Some((callable, body, _range)) = advisory_callable_member(&declaration, member) {
                        member_bodies.insert(callable, body);
                    }
                }
            }

            for analysis in ordered_analyses
                .iter()
                .filter(|analysis| analysis.callable.module() == module)
                // The synthetic module entry is analyzed below with the entire
                // top-level statement list, not as a class member body.
                .filter(|analysis| analysis.callable.try_declaration_owner().is_some_and(|owner| owner.name.as_ref() != "<main>"))
            {
                let Some(body) = member_bodies.get(&analysis.callable).copied() else {
                    module_partial = true;
                    continue;
                };

                let attachment = module_index.attachments.get(&analysis.callable);
                if attachment.is_none() {
                    module_partial = true;
                }
                let mut sites_by_range = BTreeMap::<SourceRange, Vec<SourceSiteId>>::new();
                if let Some(attachment) = attachment {
                    for site in attachment.expression_sites.iter() {
                        sites_by_range.entry(site.range).or_default().push(site.id.clone());
                    }
                }
                let site_for_range = |range: SourceRange| {
                    let candidates = sites_by_range.get(&range)?;
                    (candidates.len() == 1).then(|| candidates[0].clone())
                };
                let resolved_callable_for_range = |range: SourceRange| {
                    let mut candidates = analysis
                        .expressions
                        .values()
                        .filter(|expression| expression.range == range)
                        .filter_map(|expression| expression.callable.clone());
                    let first = candidates.next()?;
                    candidates.next().is_none().then_some(first)
                };

                let mut seed_bindings = BTreeMap::new();
                for (parameter, binding) in callable_parameter_bindings(scope_index, analysis) {
                    let fact = parameter_facts
                        .get(parameter)
                        .cloned()
                        .unwrap_or_else(|| AdvisoryFact::unknown().derive(AdvisoryConfidence::Flow, AdvisoryOrigin::Binding(binding.declaration_site.clone())));
                    seed_bindings.insert(binding.declaration_site.clone(), fact);
                }
                let context = AdvisoryFlowContext {
                    scope_index,
                    fields: &fields,
                    callable_returns: &advisory_returns,
                    builtins: &builtins,
                    current_owner: analysis.callable.try_declaration_owner(),
                    dispatch_side: analysis.callable.side,
                    source_site_for_range: &site_for_range,
                    resolved_callable_for_range: &resolved_callable_for_range,
                    resolve_callable_for_shape: Some(&resolve_callable_for_shape),
                    resolve_formal_call_result: Some(&resolve_formal_call_result),
                    advisory_transfer_target: Some(&advisory_transfer_target),
                    resolve_module_member: Some(&resolve_module_member),
                    resolve_method_family: Some(&resolve_method_family),
                };
                let flow = analyze_statements(body, &context, seed_bindings);
                for (field, fact) in &flow.field_writes {
                    fields
                        .entry(field.clone())
                        .and_modify(|old| *old = old.join(fact))
                        .or_insert_with(|| fact.clone());
                }
                parameter_contributions.replace_source(
                    crate::advisory::AdvisoryContributionSource::Callable(analysis.callable.clone()),
                    flow.parameter_contributions.clone(),
                );
                let return_fact = flow.normal_return();
                let return_fact = if matches!(return_fact.shape, crate::advisory::ValueShape::Unknown) {
                    advisory_returns.get(&analysis.callable).cloned().unwrap_or(return_fact)
                } else {
                    return_fact
                };
                for (range, callable) in &flow.call_targets {
                    if let Some(site) = target_site_for_range(*range) {
                        let target = SemanticTargetId::Callable(callable.clone());
                        targets.entry(site.clone()).or_insert_with(|| advisory_target_resolution(&site, &target));
                    }
                }
                expressions.extend(flow.expressions);
                bindings.extend(flow.bindings);

                let mut summary_parameters = Vec::new();
                for (parameter, binding) in callable_parameter_bindings(scope_index, analysis) {
                    let fact = parameter_facts
                        .get(parameter)
                        .cloned()
                        .or_else(|| bindings.get(&binding.declaration_site).cloned())
                        .unwrap_or_else(AdvisoryFact::unknown);
                    parameters.insert(parameter.clone(), fact.clone());
                    summary_parameters.push((parameter.clone(), fact));
                }
                let summary = AdvisoryCallableSummary::new(
                    analysis.callable.clone(),
                    summary_parameters,
                    return_fact,
                    analysis.dependencies.to_vec(),
                    Default::default(),
                    advisory_status(analysis.status),
                );
                let summary = previous
                    .and_then(|old| old.callables.get(&analysis.callable))
                    .filter(|old| old.as_ref() == &summary)
                    .cloned()
                    .unwrap_or_else(|| Arc::new(summary));
                next_callables.insert(analysis.callable.clone(), summary);
            }

            let source_site_for_range = |range: SourceRange| {
                let candidates = module_index
                    .expression_sites
                    .iter()
                    .filter(|site| matches!(&site.id.owner, SourceOwner::Module(owner) if owner == &module_index.structure.module))
                    .filter(|site| site.range == range)
                    .map(|site| site.id.clone())
                    .collect::<Vec<_>>();
                (candidates.len() == 1).then(|| candidates.into_iter().next().expect("one source-site candidate"))
            };
            let resolved_callable_for_range = |range: SourceRange| {
                let candidates = module_index
                    .occurrences
                    .all()
                    .iter()
                    .filter(|occurrence| occurrence.range == range)
                    .filter_map(|occurrence| module_index.occurrences.target_for(&occurrence.site))
                    .filter_map(|target| match target {
                        SemanticTargetId::Callable(callable) => Some(callable.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                (candidates.len() == 1).then(|| candidates.into_iter().next().expect("one callable candidate"))
            };
            let top_level_context = AdvisoryFlowContext {
                scope_index,
                fields: &fields,
                callable_returns: &advisory_returns,
                builtins: &builtins,
                current_owner: None,
                dispatch_side: DispatchSide::Instance,
                source_site_for_range: &source_site_for_range,
                resolved_callable_for_range: &resolved_callable_for_range,
                resolve_callable_for_shape: Some(&resolve_callable_for_shape),
                resolve_formal_call_result: Some(&resolve_formal_call_result),
                advisory_transfer_target: Some(&advisory_transfer_target),
                resolve_module_member: Some(&resolve_module_member),
                resolve_method_family: Some(&resolve_method_family),
            };
            let top_level = analyze_statements(&source.program.statements, &top_level_context, BTreeMap::new());
            parameter_contributions.replace_source(
                crate::advisory::AdvisoryContributionSource::Module(module.clone()),
                top_level.parameter_contributions.clone(),
            );
            for (range, callable) in &top_level.call_targets {
                if let Some(site) = target_site_for_range(*range) {
                    let target = SemanticTargetId::Callable(callable.clone());
                    targets.entry(site.clone()).or_insert_with(|| advisory_target_resolution(&site, &target));
                }
            }
            expressions.extend(top_level.expressions);
            bindings.extend(top_level.bindings);

            let status = if module_partial {
                AdvisoryProductStatus::Partial
            } else {
                AdvisoryProductStatus::Complete
            };
            next_partial |= module_partial;
            let shard = AdvisoryModuleProduct::new(module.clone(), expressions, bindings, std::mem::take(&mut fields), parameters, targets, status);
            let shard = previous
                .and_then(|old| old.module(module))
                .filter(|old| old.fingerprint == shard.fingerprint)
                .cloned()
                .unwrap_or_else(|| Arc::new(shard));
            next_modules.insert(module.clone(), shard);
        }

        let next_parameter_facts = parameter_contributions
            .joined_iter()
            .map(|(slot, fact)| (slot.clone(), fact.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut solver_nodes = BTreeMap::new();
        for (callable, summary) in &next_callables {
            let mut contributions = crate::advisory::AdvisoryParameterContributions::default();
            let own_parameter_ids = summary.parameters.iter().map(|(parameter, _)| parameter.clone()).collect::<BTreeSet<_>>();
            let own_parameters = next_parameter_facts
                .iter()
                .filter(|(parameter, _)| own_parameter_ids.contains(*parameter))
                .map(|(parameter, fact)| (parameter.clone(), fact.clone()))
                .collect::<BTreeMap<_, _>>();
            contributions.replace_source(
                crate::advisory::AdvisoryContributionSource::Callable(callable.clone()),
                summary.parameters.iter().cloned().collect::<BTreeMap<_, _>>(),
            );
            contributions.replace_source(crate::advisory::AdvisoryContributionSource::Module(callable.module().clone()), own_parameters);
            solver_nodes.insert(
                callable.clone(),
                AdvisorySolverNode {
                    summary: summary.clone(),
                    parameters: contributions,
                },
            );
        }
        let solved = AdvisorySolver::new(AdvisorySolverBudget {
            max_steps: advisory_budget.max_steps.min(usize::MAX as u64) as usize,
        })
        .solve_with_cancel(solver_nodes, cancel);
        let mut next_returns = formal_returns.clone();
        for (callable, summary) in &solved.summaries {
            next_returns.insert(callable.clone(), summary.return_fact.clone());
        }
        let stable = next_parameter_facts == parameter_facts && next_returns == advisory_returns;
        modules = next_modules;
        callables = solved.summaries;
        parameter_facts = next_parameter_facts;
        advisory_returns = next_returns;
        if stable {
            break (next_partial || !solved.converged, solved.status, solved.converged);
        }
    };
    let workspace_partial = workspace_partial || !solver_converged;
    AdvisoryWorkspace::from_parts(
        modules,
        callables,
        if matches!(solver_status, AdvisoryProductStatus::Cancelled) {
            AdvisoryProductStatus::Cancelled
        } else if matches!(solver_status, AdvisoryProductStatus::BudgetExceeded) {
            AdvisoryProductStatus::BudgetExceeded
        } else if workspace_partial {
            AdvisoryProductStatus::Partial
        } else {
            AdvisoryProductStatus::Complete
        },
    )
}

struct AdvisoryFieldFactsInputs<'a> {
    source: &'a ParsedModuleUnit,
    scope_index: &'a crate::source_index::SourceScopeIndex,
    builtins: &'a AdvisoryBuiltins,
    callable_returns: &'a BTreeMap<CallableId, AdvisoryFact>,
    resolve_callable_for_shape: Option<CallableForShapeResolver<'a>>,
    resolve_formal_call_result: Option<FormalCallResultResolver<'a>>,
    advisory_transfer_target: Option<&'a dyn Fn(&CallableId) -> CallableId>,
    resolve_module_member: Option<ModuleMemberResolver<'a>>,
    resolve_method_family: Option<MethodFamilyResolver<'a>>,
}

fn advisory_field_facts(inputs: AdvisoryFieldFactsInputs<'_>) -> BTreeMap<FieldId, AdvisoryFact> {
    let AdvisoryFieldFactsInputs {
        source,
        scope_index,
        builtins,
        callable_returns,
        resolve_callable_for_shape,
        resolve_formal_call_result,
        advisory_transfer_target,
        resolve_module_member,
        resolve_method_family,
    } = inputs;
    let mut fields = BTreeMap::new();
    for statement in &source.program.statements {
        let Statement::Class(class) = statement else { continue };
        let owner = DeclarationId::new(scope_index.module.clone(), class.name.clone().into());
        for member in &class.members {
            let ClassMember::Field(field) = member else { continue };
            let field_id = FieldId::new(
                owner.clone(),
                field.name.clone(),
                if field.is_static { DispatchSide::Class } else { DispatchSide::Instance },
            );
            let fact = field.default.as_ref().map_or_else(AdvisoryFact::unknown, |expr| {
                let no_site = |_range: SourceRange| None;
                let no_callable = |_range: SourceRange| None;
                let context = crate::advisory::AdvisoryExpressionContext {
                    scope_index,
                    scope: scope_index.scope_at(expr.range().start),
                    bindings: &BTreeMap::new(),
                    fields: &fields,
                    callable_returns,
                    builtins,
                    current_owner: Some(&owner),
                    dispatch_side: field_id.side,
                    source_site_for_range: &no_site,
                    resolved_callable_for_range: &no_callable,
                    resolve_callable_for_shape,
                    resolve_formal_call_result,
                    advisory_transfer_target,
                    resolve_module_member,
                    resolve_method_family,
                    call_observer: None,
                    expression_observer: None,
                    field_observer: None,
                };
                analyze_expr(expr, &context)
            });
            fields
                .entry(field_id)
                .and_modify(|existing: &mut AdvisoryFact| *existing = existing.join(&fact))
                .or_insert(fact);
        }
    }
    fields
}

fn advisory_callable_member<'a>(declaration: &DeclarationId, member: &'a ClassMember) -> Option<(CallableId, &'a [Statement], SourceRange)> {
    match member {
        ClassMember::Method(method) => {
            let slots = method
                .params
                .iter()
                .filter(|parameter| parameter.rest_mode == phalcom_ast::ast::RestMode::None)
                .map(|parameter| {
                    parameter.label.as_ref().map_or(phalcom_common::selector::SelectorSlot::Positional, |label| {
                        if label == "_" {
                            phalcom_common::selector::SelectorSlot::Positional
                        } else {
                            phalcom_common::selector::SelectorSlot::Label(label.clone())
                        }
                    })
                })
                .collect::<Vec<_>>();
            let selector = Selector::method(&method.name, slots).ok()?;
            let side = if method.is_constructor {
                DispatchSide::Instance
            } else {
                crate::checker::declaration::member_side(member)
            };
            Some((CallableId::new(declaration.clone(), selector, side), method.body.statements()?, method.range))
        }
        ClassMember::Getter(getter) => Some((
            CallableId::new(
                declaration.clone(),
                Selector::getter(&getter.name).ok()?,
                crate::checker::declaration::member_side(member),
            ),
            getter.body.statements()?,
            getter.range,
        )),
        ClassMember::Setter(setter) => Some((
            CallableId::new(
                declaration.clone(),
                Selector::setter(&setter.name).ok()?,
                crate::checker::declaration::member_side(member),
            ),
            setter.body.statements()?,
            setter.range,
        )),
        ClassMember::Index(index) => {
            let slots = index
                .params
                .iter()
                .map(|parameter| {
                    parameter.label.as_ref().map_or(phalcom_common::selector::SelectorSlot::Positional, |label| {
                        phalcom_common::selector::SelectorSlot::Label(label.clone())
                    })
                })
                .collect::<Vec<_>>();
            let selector = match index.accessor {
                phalcom_ast::ast::IndexAccessor::Get => Selector::subscript_get(slots),
                phalcom_ast::ast::IndexAccessor::Set { .. } => Selector::subscript_set(slots),
            }
            .ok()?;
            Some((
                CallableId::new(declaration.clone(), selector, DispatchSide::Instance),
                index.body.statements()?,
                index.range,
            ))
        }
        ClassMember::Field(_) | ClassMember::Delegation(_) | ClassMember::TypeMember(_) => None,
    }
}

fn callable_parameter_bindings<'a>(
    scope_index: &'a crate::source_index::SourceScopeIndex,
    analysis: &'a crate::checker::CallableAnalysis,
) -> Vec<(&'a crate::identity::CallableParameterId, &'a crate::source_index::SourceBindingInfo)> {
    let mut bindings = analysis
        .bindings
        .values()
        .filter_map(|binding| {
            let parameter = binding.parameter.as_ref()?;
            let callable_source = scope_index.callable_sources.get(&parameter.callable)?;
            let site = callable_source.parameter_sites.get(parameter)?;
            let source_binding = scope_index.bindings.get(site)?;
            Some((parameter, source_binding))
        })
        .collect::<Vec<_>>();
    bindings.sort_by_key(|(parameter, _)| parameter.index);
    bindings
}

fn advisory_target_resolution(site: &SourceSiteId, target: &SemanticTargetId) -> AdvisoryTargetResolution {
    let origin = match target {
        SemanticTargetId::Binding(_) => AdvisoryOrigin::Binding(site.clone()),
        SemanticTargetId::Callable(_) => AdvisoryOrigin::CallSite(site.clone()),
        SemanticTargetId::Field(field) => AdvisoryOrigin::Field(field.clone()),
        SemanticTargetId::Declaration(_)
        | SemanticTargetId::ModuleBinding(_)
        | SemanticTargetId::Module(_)
        | SemanticTargetId::Variant(_)
        | SemanticTargetId::VariantFamily(_)
        | SemanticTargetId::VariantField(_)
        | SemanticTargetId::DataComponent(_)
        | SemanticTargetId::StaticDomain(_)
        | SemanticTargetId::StaticCase(_)
        | SemanticTargetId::AssociatedType(_)
        | SemanticTargetId::TypeMember(_) => AdvisoryOrigin::Constraint(site.clone()),
    };
    AdvisoryTargetResolution {
        target: target.clone(),
        confidence: AdvisoryConfidence::Exact,
        provenance: vec![origin],
    }
}

fn advisory_status(status: crate::checker::CallableAnalysisStatus) -> AdvisoryProductStatus {
    match status {
        crate::checker::CallableAnalysisStatus::Complete => AdvisoryProductStatus::Complete,
        crate::checker::CallableAnalysisStatus::Partial => AdvisoryProductStatus::Partial,
        crate::checker::CallableAnalysisStatus::Blocked => AdvisoryProductStatus::Blocked,
        crate::checker::CallableAnalysisStatus::Cancelled => AdvisoryProductStatus::Cancelled,
        crate::checker::CallableAnalysisStatus::BudgetExceeded => AdvisoryProductStatus::BudgetExceeded,
        crate::checker::CallableAnalysisStatus::InternalFailure(incident) => {
            AdvisoryProductStatus::InternalFailure(format!("analysis incident {}", incident.0).into_boxed_str())
        }
    }
}

/// Publishes body-derived return summaries into canonical callable signatures,
/// then refreshes dispatch as a derived lookup projection. The fixed-point pass
/// is required for calls such as `Probe.run -> Factory.of -> CellNum.new`.
fn member_selector(owner: &DeclarationId, member: &ClassMember) -> Option<Selector> {
    crate::checker::declaration_signature::callable_id_for_member(owner, member).map(|callable| callable.selector)
}

fn source_body_for_callable<'a>(
    callable: &CallableId,
    definition: Option<&crate::impls::EffectiveCallableDefinition>,
    unit: &'a ParsedModuleUnit,
) -> Option<(&'a [Statement], SourceRange)> {
    callable.try_declaration_owner()?;
    let owner = callable.declaration_owner();
    for (statement_index, statement) in unit.program.statements.iter().enumerate() {
        match statement {
            Statement::Class(class_def) if callable.owner == crate::identity::CallableOwnerId::Declaration(owner.clone()) => {
                let declaration = DeclarationId::new(unit.id.clone(), class_def.name.clone().into());
                for member in &class_def.members {
                    let Some(selector) = member_selector(&declaration, member) else {
                        continue;
                    };
                    let side = if matches!(member, ClassMember::Method(method) if method.is_constructor || method.attributes.iter().any(|attribute| attribute.name == "constructor"))
                    {
                        DispatchSide::Instance
                    } else {
                        crate::checker::declaration::member_side(member)
                    };
                    if CallableId::new(declaration.clone(), selector, side) != *callable {
                        continue;
                    }
                    return match member {
                        ClassMember::Method(method) => Some((method.body.statements()?, method.range)),
                        ClassMember::Getter(getter) => Some((getter.body.statements()?, getter.range)),
                        ClassMember::Setter(setter) => Some((setter.body.statements()?, setter.range)),
                        ClassMember::Index(index) => Some((index.body.statements()?, index.range)),
                        _ => None,
                    };
                }
            }
            Statement::Impl(impl_def) => {
                let expected_impl = crate::identity::ImplId::new(unit.id.clone(), crate::identity::ImplLocalId(statement_index as u32));
                for (source_member_index, impl_member) in impl_def.members.iter().enumerate() {
                    let Some(definition) = definition else {
                        continue;
                    };
                    if definition.origin != crate::impls::CallableDefinitionOrigin::InherentImpl(expected_impl.clone())
                        || definition.source_member_index != source_member_index
                    {
                        continue;
                    }
                    let Some(member) = impl_member.behavior() else {
                        continue;
                    };
                    let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(member);
                    let side = if syntax.attributes().iter().any(|attr| attr.name == "class")
                        || match member {
                            phalcom_ast::ast::BehaviorMember::Method(m) => m.is_static,
                            phalcom_ast::ast::BehaviorMember::Getter(g) => g.is_static,
                            phalcom_ast::ast::BehaviorMember::Setter(s) => s.is_static,
                            phalcom_ast::ast::BehaviorMember::Index(_) => false,
                        } {
                        DispatchSide::Class
                    } else {
                        DispatchSide::Instance
                    };
                    let matches_callable = match &impl_def.target.expr {
                        phalcom_ast::ast::TypeAnnotationExpr::Reference(sym) => {
                            let decl = DeclarationId::new(unit.id.clone(), sym.root.clone().into());
                            let owner_id = crate::identity::CallableOwnerId::Declaration(decl);
                            let cand_callable = crate::checker::declaration_signature::callable_id_for_syntax(&owner_id, syntax, side);
                            cand_callable.as_ref() == Some(callable)
                        }
                        phalcom_ast::ast::TypeAnnotationExpr::Application { origin, .. } => {
                            if let phalcom_ast::ast::TypeAnnotationExpr::Reference(sym) = &origin.expr {
                                let decl = DeclarationId::new(unit.id.clone(), sym.root.clone().into());
                                let owner_id = crate::identity::CallableOwnerId::Declaration(decl);
                                let cand_callable = crate::checker::declaration_signature::callable_id_for_syntax(&owner_id, syntax, side);
                                cand_callable.as_ref() == Some(callable)
                            } else {
                                false
                            }
                        }
                        phalcom_ast::ast::TypeAnnotationExpr::ExactEnumCase {
                            enum_target,
                            variant_name,
                            payload_shape,
                            ..
                        } => {
                            let enum_name_opt = match &enum_target.expr {
                                phalcom_ast::ast::TypeAnnotationExpr::Reference(sym) => Some(&sym.root),
                                phalcom_ast::ast::TypeAnnotationExpr::Application { origin, .. } => match &origin.expr {
                                    phalcom_ast::ast::TypeAnnotationExpr::Reference(sym) => Some(&sym.root),
                                    _ => None,
                                },
                                _ => None,
                            };
                            if let Some(enum_name) = enum_name_opt {
                                let decl = DeclarationId::new(unit.id.clone(), enum_name.clone().into());
                                let var_sel = phalcom_ast::selector::selector_from_exact_case_target(variant_name, payload_shape.as_ref());
                                let variant_id = crate::identity::VariantId::new(decl, var_sel);
                                let owner_id = crate::identity::CallableOwnerId::Variant(variant_id);
                                let cand_callable = crate::checker::declaration_signature::callable_id_for_syntax(&owner_id, syntax, side);
                                cand_callable.as_ref() == Some(callable)
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };
                    if matches_callable {
                        return match member {
                            phalcom_ast::ast::BehaviorMember::Method(method) => Some((method.body.statements()?, method.range)),
                            phalcom_ast::ast::BehaviorMember::Getter(getter) => Some((getter.body.statements()?, getter.range)),
                            phalcom_ast::ast::BehaviorMember::Setter(setter) => Some((setter.body.statements()?, setter.range)),
                            phalcom_ast::ast::BehaviorMember::Index(index) => Some((index.body.statements()?, index.range)),
                        };
                    }
                }
            }
            _ => {}
        }
    }
    None
}

struct DownstreamCallableBodyState<'a> {
    definitions: &'a crate::parent_map::ParentMap<CallableId, crate::impls::EffectiveCallableDefinition>,
    store: &'a mut TypeStore,
    hierarchy: &'a MapTypeHierarchy,
    dispatch: &'a mut SurfaceDispatchResolver,
    callable_signatures: &'a mut CallableSignatureTable,
    callable_analyses: &'a mut HashMap<CallableId, Arc<crate::checker::CallableAnalysis>>,
    callable_dispositions: &'a mut BTreeMap<CallableId, CallableRevisionDisposition>,
    budget: QueryBudget,
    cancel: &'a CancellationToken,
}

fn revalidate_downstream_callable_body(
    db: &mut SemanticDb,
    callable: &CallableId,
    unit: &ParsedModuleUnit,
    formal_inputs: &FormalQueryInputs<'_>,
    default_field_lifecycle: &crate::checker::field_lifecycle::FieldLifecycleTable,
    state: DownstreamCallableBodyState<'_>,
) -> Result<(), QueryOutcome<()>> {
    let DownstreamCallableBodyState {
        definitions,
        store,
        hierarchy,
        dispatch,
        callable_signatures,
        callable_analyses,
        callable_dispositions,
        budget,
        cancel,
    } = state;
    // Constructor entry flow must always start from source field defaults,
    // even when a later session pass rechecks the body after constructor
    // lifecycle has been published. Other callables consume that finalized
    // lifecycle as their field-read knowledge.
    let mut body_formal_inputs = *formal_inputs;
    if callable_signatures.get_for_body(callable).is_some_and(|signature| signature.is_constructor()) {
        body_formal_inputs.field_lifecycle = Some(default_field_lifecycle);
    }
    let definition = definitions.get(callable).cloned().map(Arc::new);
    if let Some(definition) = &definition {
        ready_or_error(query_callable_definition(db, definition.clone()))?;
    }
    let Some((body, body_range)) = source_body_for_callable(callable, definition.as_deref(), unit) else {
        return Ok(());
    };
    let body_context = definition.as_ref().and_then(|definition| definition.inherent_body_context.clone());
    let mut type_parameters = HashMap::new();
    let owner_signature = body_context
        .as_ref()
        .and_then(|context| context.generic_signature.as_ref())
        .or_else(|| formal_inputs.declarations.generic_signature(callable.declaration_owner()));
    if let Some(signature) = owner_signature {
        for &parameter in signature.parameters.iter() {
            let name = store.type_parameter(parameter).name.to_string();
            type_parameters.insert(name, type_level_binding_for_parameter(store, parameter));
        }
    }
    let scoped_resolver = crate::types::annotation::ScopedTypeResolver {
        parent: formal_inputs.base_resolver,
        type_parameters,
    };
    let key = QueryKey::CallableBody(callable.clone());
    refresh_cached_body_dependencies(db, &key, &body_formal_inputs, store)?;
    let previous_computation_revision = db.query_state(&key).and_then(|state| state.revision());
    let signature_id = crate::db::query::declaration_signature_id_for_body(db, callable, unit)
        .ok_or_else(|| QueryOutcome::Failed(format!("current callable body {callable:?} has no declaration signature identity")))?;
    let canonical_signature = callable_signatures
        .get(&signature_id)
        .cloned()
        .ok_or_else(|| QueryOutcome::Failed(format!("current callable body {callable:?} has no canonical signature {signature_id:?}")))?;
    let body_signature = body_context
        .as_ref()
        .map(|context| {
            context
                .signature_for_body(store, formal_inputs.declarations, canonical_signature.clone())
                .expect("validated covering substitution must materialize its accepted callable signature")
        })
        .unwrap_or(canonical_signature);
    let outcome = query_callable_body_with_formal_inputs(
        db,
        CallableBodyQuery {
            callable: callable.clone(),
            body,
            body_range,
            declared_signature: Some((&signature_id, &body_signature)),
            owner_generic_signature: body_context.as_ref().and_then(|context| context.generic_signature.as_ref()),
            receiver_value_override: body_context.as_ref().and_then(|context| context.runtime_receiver_override(callable.side)),
            instance_self_override: body_context.as_ref().map(|context| context.instance_self_override()),
            trait_surface: None,
            store,
            hierarchy,
            resolver: &scoped_resolver,
            declarations: formal_inputs.declarations,
            dispatch,
            module: callable.module().clone(),
            budget,
            cancel,
            formal_inputs: Some(&body_formal_inputs),
        },
    );
    match outcome {
        QueryOutcome::Ready(analysis) => {
            let recomputed = db
                .query_state(&key)
                .is_some_and(|state| state.revision() == Some(db.revision()) && previous_computation_revision != Some(db.revision()));
            callable_dispositions.insert(
                callable.clone(),
                if recomputed {
                    CallableRevisionDisposition::Recomputed
                } else {
                    CallableRevisionDisposition::Reused
                },
            );
            if let Some(sig) = callable_signatures.get_mut(callable) {
                if sig.return_validation != analysis.return_validation {
                    sig.return_validation = analysis.return_validation;
                    dispatch.update_callable_return_type(callable, sig.published_return_knowledge());
                }
            }
            callable_analyses.insert(callable.clone(), analysis);
            Ok(())
        }
        QueryOutcome::Cancelled => Err(QueryOutcome::Cancelled),
        QueryOutcome::BudgetExceeded(report) => Err(QueryOutcome::BudgetExceeded(report)),
        QueryOutcome::Blocked(reason) => Err(QueryOutcome::Blocked(reason)),
        QueryOutcome::Failed(error) => Err(QueryOutcome::Failed(error)),
    }
}

fn ready_or_error<T>(outcome: QueryOutcome<T>) -> Result<(), QueryOutcome<()>> {
    match outcome {
        QueryOutcome::Ready(_) => Ok(()),
        QueryOutcome::Cancelled => Err(QueryOutcome::Cancelled),
        QueryOutcome::BudgetExceeded(report) => Err(QueryOutcome::BudgetExceeded(report)),
        QueryOutcome::Blocked(reason) => Err(QueryOutcome::Blocked(reason)),
        QueryOutcome::Failed(error) => Err(QueryOutcome::Failed(error)),
    }
}

fn refresh_cached_body_dependencies(
    db: &mut SemanticDb,
    body_key: &QueryKey,
    formal_inputs: &FormalQueryInputs<'_>,
    store: &mut TypeStore,
) -> Result<(), QueryOutcome<()>> {
    ready_or_error(crate::db::query::ensure_cached_formal_semantic_dependencies_current(
        db,
        body_key,
        formal_inputs,
        store,
    ))
}

struct InferredCallableRefreshInputs<'a> {
    db: &'a mut SemanticDb,
    sources: &'a BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    store: &'a mut TypeStore,
    hierarchy: &'a MapTypeHierarchy,
    resolver: &'a LinkedTypeResolver,
    declarations: &'a DeclarationTypeTable,
    dispatch: &'a mut SurfaceDispatchResolver,
    callable_signatures: &'a mut CallableSignatureTable,
    constructor_initializations: &'a ConstructorInitializationTable,
    callable_analyses: &'a mut HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>,
    previous_callable_analyses: Option<&'a crate::checker::CallableAnalysisTable>,
    field_signatures: &'a FieldSignatureTable,
    field_lifecycle: &'a crate::checker::field_lifecycle::FieldLifecycleTable,
    enum_semantics: &'a EnumSemanticTable,
    data_semantics: &'a DataSemanticTable,
    associated_surfaces: &'a AssociatedFamilyTable,
    topology: &'a phalcom_modules::topology::ModuleTopology,
    callable_dispositions: &'a mut BTreeMap<CallableId, CallableRevisionDisposition>,
    semantic_work_modules: &'a BTreeSet<ModuleId>,
    previous_callable_signatures: Option<&'a CallableSignatureTable>,
    trait_default_callables: &'a BTreeSet<CallableId>,
    conformance_semantics: &'a crate::trait_dispatch::ConformanceSemanticView<'a>,
    withheld_result_callables: &'a BTreeSet<CallableId>,
    budget: QueryBudget,
    cancel: &'a CancellationToken,
}

fn publishable_inferred_return(
    status: crate::checker::analysis::CallableAnalysisStatus,
    summary: crate::types::evidence::TypeKnowledge,
) -> Option<crate::types::evidence::TypeKnowledge> {
    match status {
        crate::checker::analysis::CallableAnalysisStatus::Complete if summary.is_known() || summary.is_dynamic() => Some(summary),
        _ => None,
    }
}

/// Finds current direct-result SCCs that are not grounded by a declared result
/// and retracts any prior inferred-only signature products they contain.
///
/// The returned first set must be withheld from the generic result refresh for
/// this session pass. The second set contains roots that require one body
/// reanalysis after their stale signature products are retracted.
fn retract_stale_unseeded_callable_result_cycles(
    db: &mut SemanticDb,
    callable_analyses: &HashMap<CallableId, Arc<crate::checker::analysis::CallableAnalysis>>,
    callable_signatures: &mut crate::signature::CallableSignatureTable,
    dispatch: &mut SurfaceDispatchResolver,
) -> Result<(BTreeSet<CallableId>, BTreeSet<CallableId>), QueryOutcome<()>> {
    let components = callable_result_components(callable_analyses, callable_signatures);
    let unseeded = components
        .into_iter()
        .filter(|component| {
            component.iter().all(|callable| {
                callable_signatures
                    .get_for_body(callable)
                    .is_some_and(|signature| signature.declared_return.is_unknown())
            })
        })
        .collect::<Vec<_>>();
    let withheld = unseeded.iter().flatten().cloned().collect::<BTreeSet<_>>();
    let reanalysis_roots = unseeded
        .into_iter()
        .filter(|component| {
            component.iter().any(|callable| {
                callable_signatures
                    .get_for_body(callable)
                    .is_some_and(|signature| signature.inferred_return.is_some())
            })
        })
        .flatten()
        .collect::<BTreeSet<_>>();

    for callable in &reanalysis_roots {
        let Some(signature) = callable_signatures.get_mut(callable) else {
            continue;
        };
        // These are omitted-result signatures by construction. Explicit
        // declaration-owned results never enter this retraction path.
        signature.inferred_return = None;
        let signature = Arc::new(signature.clone());
        match crate::db::query::query_bootstrap_callable_signature(db, signature.clone()) {
            QueryOutcome::Ready(_) => {}
            QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
            QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
            QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
            QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
        }
        let _ = dispatch.update_callable_return_type(callable, signature.published_return_knowledge());
    }

    Ok((withheld, reanalysis_roots))
}

fn callable_result_components(
    callable_analyses: &HashMap<CallableId, Arc<crate::checker::analysis::CallableAnalysis>>,
    callable_signatures: &crate::signature::CallableSignatureTable,
) -> Vec<BTreeSet<CallableId>> {
    let nodes = callable_analyses
        .keys()
        .filter(|callable| callable_signatures.get_for_body(callable).is_some())
        .cloned()
        .collect::<BTreeSet<_>>();
    let graph = nodes
        .iter()
        .map(|callable| {
            let dependencies = callable_analyses
                .get(callable)
                .into_iter()
                .flat_map(|analysis| &analysis.exits.normal_returns)
                .filter_map(|fact| fact.direct_result_dependency.as_ref())
                .filter(|dependency| nodes.contains(*dependency))
                .cloned()
                .collect::<BTreeSet<_>>();
            (callable.clone(), dependencies)
        })
        .collect::<BTreeMap<_, _>>();
    callable_result_cycle_components(&graph)
}

fn refresh_unresolved_callable_result_cycle_diagnostics(
    callable_analyses: &HashMap<CallableId, Arc<crate::checker::analysis::CallableAnalysis>>,
    callable_signatures: &crate::signature::CallableSignatureTable,
    diagnostics_by_module: &mut BTreeMap<ModuleId, Vec<SemanticDiagnostic>>,
) {
    for diagnostics in diagnostics_by_module.values_mut() {
        diagnostics.retain(|diagnostic| diagnostic.code != DiagnosticCode::CallableResultCycle);
    }

    for component in callable_result_components(callable_analyses, callable_signatures) {
        let component_is_unseeded = component.iter().all(|callable| {
            callable_signatures
                .get_for_body(callable)
                .is_some_and(|signature| signature.declared_return.is_unknown() && signature.published_return_knowledge().is_unknown())
        });
        if !component_is_unseeded {
            continue;
        }
        let mut members = component.iter().collect::<Vec<_>>();
        members.sort_by_key(|callable| {
            let analysis = &callable_analyses[*callable];
            (callable.module(), analysis.body_range.start, analysis.body_range.end, *callable)
        });
        let Some(anchor) = members.first().copied() else { continue };
        let anchor_module = anchor.module();
        let anchor_range = callable_analyses[anchor].body_range;
        let mut diagnostic = SemanticDiagnostic::error_in(
            anchor_module.clone(),
            DiagnosticCode::CallableResultCycle,
            "omitted-result callable participates in a direct result cycle without an explicit result seed",
            anchor_range,
        );
        for member in members.iter().skip(1) {
            let module = member.module();
            let range = callable_analyses[*member].body_range;
            diagnostic = diagnostic.with_label_in(module.clone(), range, "participates in the unresolved result cycle");
        }
        diagnostic = diagnostic.with_note("declare a result type on one callable to provide an authoritative result seed");
        diagnostics_by_module.entry(anchor_module.clone()).or_default().push(diagnostic);
    }
}

fn callable_result_cycle_components(graph: &BTreeMap<CallableId, BTreeSet<CallableId>>) -> Vec<BTreeSet<CallableId>> {
    struct Tarjan<'a> {
        graph: &'a BTreeMap<CallableId, BTreeSet<CallableId>>,
        next: usize,
        indices: HashMap<CallableId, usize>,
        lowlinks: HashMap<CallableId, usize>,
        stack: Vec<CallableId>,
        on_stack: BTreeSet<CallableId>,
        components: Vec<BTreeSet<CallableId>>,
    }

    fn visit(node: CallableId, state: &mut Tarjan<'_>) {
        let index = state.next;
        state.next += 1;
        state.indices.insert(node.clone(), index);
        state.lowlinks.insert(node.clone(), index);
        state.stack.push(node.clone());
        state.on_stack.insert(node.clone());

        if let Some(dependencies) = state.graph.get(&node) {
            for dependency in dependencies {
                if !state.indices.contains_key(dependency) {
                    visit(dependency.clone(), state);
                    let lowlink = state.lowlinks[&node].min(state.lowlinks[dependency]);
                    state.lowlinks.insert(node.clone(), lowlink);
                } else if state.on_stack.contains(dependency) {
                    let lowlink = state.lowlinks[&node].min(state.indices[dependency]);
                    state.lowlinks.insert(node.clone(), lowlink);
                }
            }
        }

        if state.lowlinks[&node] == state.indices[&node] {
            let mut component = BTreeSet::new();
            while let Some(member) = state.stack.pop() {
                state.on_stack.remove(&member);
                component.insert(member.clone());
                if member == node {
                    break;
                }
            }
            if component.len() > 1 || state.graph.get(&node).is_some_and(|dependencies| dependencies.contains(&node)) {
                state.components.push(component);
            }
        }
    }

    let mut state = Tarjan {
        graph,
        next: 0,
        indices: HashMap::new(),
        lowlinks: HashMap::new(),
        stack: Vec::new(),
        on_stack: BTreeSet::new(),
        components: Vec::new(),
    };
    for node in graph.keys() {
        if !state.indices.contains_key(node) {
            visit(node.clone(), &mut state);
        }
    }
    state.components.sort_by(|left, right| left.first().cmp(&right.first()));
    state.components
}

fn same_published_return_contract(left: &crate::types::evidence::TypeKnowledge, right: &crate::types::evidence::TypeKnowledge) -> bool {
    match (left, right) {
        (crate::types::evidence::TypeKnowledge::Known(left), crate::types::evidence::TypeKnowledge::Known(right)) => {
            left.ty() == right.ty() && left.status() == right.status() && left.origin() == right.origin()
        }
        (crate::types::evidence::TypeKnowledge::Unknown(left), crate::types::evidence::TypeKnowledge::Unknown(right)) => left == right,
        (crate::types::evidence::TypeKnowledge::Dynamic(left), crate::types::evidence::TypeKnowledge::Dynamic(right)) => left == right,
        _ => false,
    }
}

fn refresh_inferred_callable_results(inputs: InferredCallableRefreshInputs<'_>) -> Result<(), QueryOutcome<()>> {
    let InferredCallableRefreshInputs {
        db,
        sources,
        store,
        hierarchy,
        resolver,
        declarations,
        dispatch,
        callable_signatures,
        constructor_initializations,
        callable_analyses,
        previous_callable_analyses,
        field_signatures,
        field_lifecycle,
        enum_semantics,
        data_semantics,
        associated_surfaces,
        topology,
        callable_dispositions,
        semantic_work_modules,
        previous_callable_signatures,
        trait_default_callables,
        conformance_semantics,
        withheld_result_callables,
        budget,
        cancel,
    } = inputs;
    let max_iterations = callable_analyses.len().saturating_add(1).max(1);
    let mut recomputed_in_prev_iteration = HashSet::new();

    for iteration in 0..max_iterations {
        if cancel.is_cancelled() {
            return Err(QueryOutcome::Cancelled);
        }

        let mut changed_callables = if iteration == 0 {
            callable_dispositions
                .iter()
                .filter_map(|(callable, disposition)| {
                    (*disposition == CallableRevisionDisposition::Recomputed
                        && callable_signatures
                            .get_for_body(callable)
                            .is_some_and(|signature| signature.declared_return.is_unknown()))
                    .then_some(callable.clone())
                })
                .collect()
        } else {
            HashSet::new()
        };

        for (callable, analysis) in callable_analyses.iter() {
            if withheld_result_callables.contains(callable) {
                // Unseeded direct-result SCC members were reanalyzed once
                // after inferred-only signatures were retracted. Their
                // results must not re-enter the generic publication loop.
                continue;
            }
            let Some(signature) = callable_signatures.get_for_body(callable) else {
                continue;
            };
            let signature_id = signature.callable.clone();
            if signature_id.try_declaration_owner().is_none() {
                // Conformance witnesses have no inherent dispatch surface;
                // their body proof must not update target-owned return data.
                continue;
            }

            let old_public = if iteration == 0 {
                previous_callable_signatures
                    .and_then(|previous| previous.get(&signature_id))
                    .map(CallableSemanticSignature::published_return_knowledge)
                    .unwrap_or_else(|| signature.published_return_knowledge())
            } else {
                signature.published_return_knowledge()
            };

            let Some(signature_mut) = callable_signatures.get_mut(&signature_id) else {
                continue;
            };

            if signature_mut.return_validation != analysis.return_validation {
                signature_mut.return_validation = analysis.return_validation;
            }

            if signature_mut.declared_return.is_unknown() {
                let summary = normal_return_summary(store, &analysis.exits.normal_returns);
                signature_mut.inferred_return = publishable_inferred_return(analysis.status, summary);
            }

            let new_public = signature_mut.published_return_knowledge();
            if trait_default_callables.contains(&signature_id) {
                let updated_signature = Arc::new(signature_mut.clone());
                match query_bootstrap_callable_signature(db, updated_signature) {
                    QueryOutcome::Ready(_) => {}
                    QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                    QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                    QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                    QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                }
            }
            let _ = dispatch.update_callable_return_type(&signature_id, new_public.clone());

            if !same_published_return_contract(&old_public, &new_public)
                || (recomputed_in_prev_iteration.contains(callable) && signature_mut.declared_return.is_unknown())
            {
                changed_callables.insert(callable.clone());
            }
        }

        if changed_callables.is_empty() {
            break;
        }

        let mut recomputed_in_this_iteration = HashSet::new();
        let refreshed_conformance_semantics = crate::trait_dispatch::ConformanceSemanticView {
            trait_dispatch: conformance_semantics.trait_dispatch,
            conformance_index: conformance_semantics.conformance_index,
            witness_plans: conformance_semantics.witness_plans,
            trait_surfaces: conformance_semantics.trait_surfaces,
            derived_conformance_rules: conformance_semantics.derived_conformance_rules,
            callable_signatures: Some(callable_signatures),
        };

        // Recheck only bodies that consume a newly published return contract.
        // This deliberately bypasses the source-surface query cache: that
        // cache owns the pre-inference unknown contract, while this pass is
        // producing the current revision's inferred contract. Unrelated
        // callable products remain pointer-stable for incremental reuse.
        for (module_id, parsed_unit) in sources {
            if !semantic_work_modules.contains(module_id) {
                continue;
            }
            for stmt in &parsed_unit.program.statements {
                let Statement::Class(class_def) = stmt else {
                    continue;
                };
                let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                let type_params_map = if let Some(sig) = declarations.generic_signature(&decl_id) {
                    let mut map = std::collections::HashMap::new();
                    for &param_id in sig.parameters.iter() {
                        let name = store.type_parameter(param_id).name.to_string();
                        let binding = type_level_binding_for_parameter(store, param_id);
                        map.insert(name, binding);
                    }
                    map
                } else {
                    std::collections::HashMap::new()
                };
                let scoped_resolver = crate::types::annotation::ScopedTypeResolver {
                    parent: resolver,
                    type_parameters: type_params_map,
                };

                for member in &class_def.members {
                    let side = match member {
                        ClassMember::Method(m) if m.is_constructor || m.attributes.iter().any(|a| a.name == "constructor") => {
                            crate::identity::DispatchSide::Instance
                        }
                        _ => crate::checker::declaration::member_side(member),
                    };
                    let (selector_opt, body_opt, range_opt) = match member {
                        ClassMember::Method(m) => {
                            let slots = m
                                .params
                                .iter()
                                .filter(|p| p.rest_mode == phalcom_ast::ast::RestMode::None)
                                .map(|p| {
                                    if let Some(ref label) = p.label {
                                        if label == "_" {
                                            phalcom_common::selector::SelectorSlot::Positional
                                        } else {
                                            phalcom_common::selector::SelectorSlot::Label(label.clone())
                                        }
                                    } else {
                                        phalcom_common::selector::SelectorSlot::Positional
                                    }
                                })
                                .collect::<Vec<_>>();
                            (Selector::method(&m.name, slots).ok(), m.body.statements(), Some(m.range))
                        }
                        ClassMember::Getter(g) => (Selector::getter(&g.name).ok(), g.body.statements(), Some(g.range)),
                        ClassMember::Setter(s) => (Selector::setter(&s.name).ok(), s.body.statements(), Some(s.range)),
                        _ => (None, None, None),
                    };

                    let (Some(selector), Some(body), Some(range)) = (selector_opt, body_opt, range_opt) else {
                        continue;
                    };
                    let callable = crate::identity::CallableId::new(decl_id.clone(), selector, side);
                    if withheld_result_callables.contains(&callable) {
                        continue;
                    }
                    let affected = callable_analyses
                        .get(&callable)
                        .is_some_and(|analysis| analysis.dependencies.iter().any(|dependency| changed_callables.contains(dependency)));
                    let inferred_return = callable_signatures
                        .get_for_body(&callable)
                        .is_some_and(|signature| signature.declared_return.is_unknown());
                    if !affected || !inferred_return {
                        continue;
                    }
                    let declared_signature = callable_signatures.get_for_body(&callable).map(|signature| (&signature.callable, signature));
                    let mut analysis = crate::checker::body::analyze_callable_body(
                        crate::checker::body::BodyAnalysisContext {
                            store,
                            hierarchy,
                            resolver: &scoped_resolver,
                            declarations,
                            dispatch,
                            trait_surface: None,
                            conformance_semantics: Some(&refreshed_conformance_semantics),
                            module: module_id.clone(),
                            caller_package: Some(topology.owning_package(module_id)),
                            access_topology: Some(topology),
                            module_values: None,
                            linked_module: None,
                            semantic_linked_module: None,
                        },
                        crate::checker::body::CallableBodyRequest {
                            callable: callable.clone(),
                            body,
                            body_range: range,
                            owner_generic_signature: None,
                            receiver_value_override: None,
                            instance_self_override: None,
                            declared_signature,
                            budget,
                            cancel,
                            field_signatures: Some(field_signatures),
                            field_lifecycle: Some(field_lifecycle),
                            constructor_initializations: Some(constructor_initializations),
                            callable_signatures: Some(&*callable_signatures),
                            enum_semantics: Some(enum_semantics),
                            data_semantics: Some(data_semantics),
                            associated_families: Some(associated_surfaces),
                            conformance_semantics: None,
                        },
                    );
                    analysis.dependency_fingerprint = crate::db::fingerprint::callable_body_product_fingerprint(&analysis);
                    let stable_previous = previous_callable_analyses
                        .and_then(|analyses| analyses.get(&callable))
                        .or_else(|| callable_analyses.get(&callable));
                    let replacement = match stable_previous {
                        Some(previous) if previous.dependency_fingerprint == analysis.dependency_fingerprint => previous.clone(),
                        _ => Arc::new(analysis),
                    };
                    let dependencies = crate::db::query::callable_body_dependency_edges(db, &callable, &replacement, Some(conformance_semantics))
                        .map_err(QueryOutcome::Failed)?;
                    callable_analyses.insert(callable.clone(), replacement.clone());
                    db.update_callable_body_product(&callable, replacement, dependencies);
                    callable_dispositions.insert(callable.clone(), CallableRevisionDisposition::Recomputed);
                    recomputed_in_this_iteration.insert(callable);
                }
            }
        }

        recomputed_in_prev_iteration = recomputed_in_this_iteration;
    }

    Ok(())
}

fn witness_specialization_terminal(error: crate::types::specialization::ReceiverSpecializationFailure, context: &str) -> crate::impls::ConformanceCompleteness {
    match error {
        crate::types::specialization::ReceiverSpecializationFailure::Blocked(reason) => crate::impls::ConformanceCompleteness::Blocked(reason),
        crate::types::specialization::ReceiverSpecializationFailure::Cancelled => crate::impls::ConformanceCompleteness::Cancelled,
        crate::types::specialization::ReceiverSpecializationFailure::BudgetExceeded(report) => crate::impls::ConformanceCompleteness::BudgetExceeded(report),
        other => crate::impls::ConformanceCompleteness::InternalFailure(format!("{context}: {other:?}").into_boxed_str()),
    }
}

fn publish_conformance_witness_plans(
    db: &mut SemanticDb,
    store: &mut TypeStore,
    hierarchy: &dyn crate::types::relation::TypeHierarchy,
    dispatch: &SurfaceDispatchResolver,
    declarations: &DeclarationTypeTable,
    conformance_index: &crate::impls::ConformanceIndex,
    callable_signatures: &crate::signature::CallableSignatureTable,
    conformance_witness_plans: &mut crate::impls::ConformanceWitnessPlans,
    conformance_witness_bodies: &BTreeSet<CallableId>,
    conformance_witness_visibilities: &BTreeMap<CallableId, crate::surface::MemberVisibility>,
    conformance_invalid_witnesses: &BTreeMap<crate::identity::ImplId, BTreeSet<crate::traits::TraitRequirementId>>,
    conformance_invalid_members: &BTreeSet<crate::identity::ImplId>,
    conformance_associated_type_plans: &BTreeMap<crate::identity::ImplId, crate::impls::ConformanceAssociatedTypePlan>,
    data_semantics: &DataSemanticTable,
    trait_surfaces: &TraitSurfaceTable,
    diags_by_module: &mut BTreeMap<ModuleId, Vec<SemanticDiagnostic>>,
    emit_incomplete_diagnostics: bool,
    diagnostic_work_modules: &BTreeSet<ModuleId>,
) -> Result<BTreeSet<crate::identity::ImplId>, QueryOutcome<()>> {
    let mut changed_plans = BTreeSet::new();
    for (impl_id, contribution) in conformance_index.iter() {
        if !contribution.is_lookup_eligible() {
            continue;
        }
        let Some(surface) = trait_surfaces.get(&contribution.trait_ref.declaration) else {
            continue;
        };
        let mut requirement_views = crate::impls::instantiate_conformance_requirements_with_signatures(
            store,
            Some(declarations),
            surface,
            &contribution.trait_ref,
            contribution.target_head,
            &HashMap::new(),
            callable_signatures,
        );
        if let Some(associated_type_plan) = conformance_associated_type_plans.get(impl_id) {
            crate::impls::normalize_source_requirement_views(
                store,
                &contribution.trait_ref,
                contribution.target_head,
                associated_type_plan,
                &mut requirement_views,
            );
        }
        let mut inherent_candidates = BTreeMap::new();
        let mut inherent_mismatches = BTreeMap::new();
        let mut deferred_inherent_candidates = BTreeMap::new();
        let mut data_candidates = BTreeMap::new();
        let mut terminal_candidates = BTreeMap::new();
        for (requirement, _member) in surface.iter() {
            let ambient_constraints = contribution
                .generic_signature
                .as_ref()
                .map(|signature| signature.constraints.as_ref())
                .unwrap_or(&[]);
            let effective = crate::impls::resolve_effective_inherent_witness(
                store,
                hierarchy,
                dispatch,
                callable_signatures,
                contribution.target_head,
                contribution.target.declaration(),
                &requirement.selector,
                requirement.side,
                ambient_constraints,
            );
            match effective {
                crate::impls::EffectiveInherentWitnessResolution::Candidate(candidate) => {
                    let Some(required) = requirement_views.get(requirement) else { continue };
                    let candidate_signature = match crate::impls::specialize_witness_signature(
                        store,
                        hierarchy,
                        contribution.target_head,
                        &candidate.owner,
                        candidate.signature.clone(),
                    ) {
                        Ok(signature) => signature,
                        Err(error) => {
                            let state = witness_specialization_terminal(error, "inherent witness specialization failed");
                            terminal_candidates
                                .entry(requirement.clone())
                                .and_modify(|current| crate::impls::replace_with_stronger_terminal_state(current, state.clone()))
                                .or_insert(state);
                            continue;
                        }
                    };
                    let compatibility =
                        crate::impls::check_witness_compatibility_with_visibility(store, hierarchy, required, &candidate_signature, candidate.visibility);
                    if matches!(compatibility, crate::impls::WitnessCompatibility::Compatible(_)) {
                        inherent_candidates.insert(requirement.clone(), candidate);
                    } else {
                        if let crate::impls::WitnessCompatibility::Incompatible(mismatch) = &compatibility {
                            inherent_mismatches.insert(requirement.clone(), mismatch.reason.clone());
                        }
                        if let Some(state) = crate::impls::witness_compatibility_terminal_state(&compatibility) {
                            terminal_candidates
                                .entry(requirement.clone())
                                .and_modify(|current| crate::impls::replace_with_stronger_terminal_state(current, state.clone()))
                                .or_insert(state);
                        }
                    }
                }
                crate::impls::EffectiveInherentWitnessResolution::Deferred { candidate, state } => {
                    let Some(required) = requirement_views.get(requirement) else { continue };
                    let candidate_signature = match crate::impls::specialize_witness_signature(
                        store,
                        hierarchy,
                        contribution.target_head,
                        &candidate.owner,
                        candidate.signature.clone(),
                    ) {
                        Ok(signature) => signature,
                        Err(error) => {
                            let state = witness_specialization_terminal(error, "deferred inherent witness specialization failed");
                            terminal_candidates
                                .entry(requirement.clone())
                                .and_modify(|current| crate::impls::replace_with_stronger_terminal_state(current, state.clone()))
                                .or_insert(state);
                            continue;
                        }
                    };
                    let compatibility =
                        crate::impls::check_witness_compatibility_with_visibility(store, hierarchy, required, &candidate_signature, candidate.visibility);
                    match compatibility {
                        crate::impls::WitnessCompatibility::Compatible(_) => {
                            deferred_inherent_candidates.insert(requirement.clone(), (candidate, state));
                        }
                        crate::impls::WitnessCompatibility::Incompatible(mismatch) => {
                            inherent_mismatches.insert(requirement.clone(), mismatch.reason);
                        }
                        other => {
                            if let Some(state) = crate::impls::witness_compatibility_terminal_state(&other) {
                                terminal_candidates
                                    .entry(requirement.clone())
                                    .and_modify(|current| crate::impls::replace_with_stronger_terminal_state(current, state.clone()))
                                    .or_insert(state);
                            }
                        }
                    }
                }
                crate::impls::EffectiveInherentWitnessResolution::Unknown(reason) => {
                    let state = crate::impls::ConformanceCompleteness::Unknown(reason);
                    terminal_candidates
                        .entry(requirement.clone())
                        .and_modify(|current| crate::impls::replace_with_stronger_terminal_state(current, state.clone()))
                        .or_insert(state);
                }
                crate::impls::EffectiveInherentWitnessResolution::Blocked(reason) => {
                    let state = crate::impls::ConformanceCompleteness::Blocked(reason);
                    terminal_candidates
                        .entry(requirement.clone())
                        .and_modify(|current| crate::impls::replace_with_stronger_terminal_state(current, state.clone()))
                        .or_insert(state);
                }
                crate::impls::EffectiveInherentWitnessResolution::Dynamic(obligation) => {
                    let state = crate::impls::ConformanceCompleteness::Dynamic(obligation);
                    terminal_candidates
                        .entry(requirement.clone())
                        .and_modify(|current| crate::impls::replace_with_stronger_terminal_state(current, state.clone()))
                        .or_insert(state);
                }
                crate::impls::EffectiveInherentWitnessResolution::Cancelled => {
                    terminal_candidates.insert(requirement.clone(), crate::impls::ConformanceCompleteness::Cancelled);
                }
                crate::impls::EffectiveInherentWitnessResolution::BudgetExceeded(report) => {
                    terminal_candidates
                        .entry(requirement.clone())
                        .and_modify(|current| {
                            crate::impls::replace_with_stronger_terminal_state(current, crate::impls::ConformanceCompleteness::BudgetExceeded(report.clone()))
                        })
                        .or_insert_with(|| crate::impls::ConformanceCompleteness::BudgetExceeded(report));
                }
                crate::impls::EffectiveInherentWitnessResolution::InternalFailure(message) => {
                    terminal_candidates
                        .entry(requirement.clone())
                        .and_modify(|current| {
                            crate::impls::replace_with_stronger_terminal_state(current, crate::impls::ConformanceCompleteness::InternalFailure(message.clone()))
                        })
                        .or_insert_with(|| crate::impls::ConformanceCompleteness::InternalFailure(message));
                }
                crate::impls::EffectiveInherentWitnessResolution::NotFound => {}
            }
            if requirement.side == DispatchSide::Instance
                && matches!(requirement.selector.kind, phalcom_common::selector::SelectorKind::Getter)
                && let Some(info) = data_semantics.get(contribution.target.declaration())
                && let Some(component) = info.find_component(&requirement.selector.encode())
                && let Some(component_type) = component.declared_type.canonical_type()
            {
                let specialized_component_type =
                    crate::impls::specialize_conformance_target_type(store, declarations, contribution.target_head, &HashMap::new(), component_type);
                if requirement_views.get(requirement).is_some_and(|required| {
                    let compatibility = crate::impls::check_data_component_compatibility(store, hierarchy, required, specialized_component_type);
                    if let Some(state) = crate::impls::witness_compatibility_terminal_state(&compatibility) {
                        terminal_candidates
                            .entry(requirement.clone())
                            .and_modify(|current| crate::impls::replace_with_stronger_terminal_state(current, state.clone()))
                            .or_insert(state);
                        false
                    } else {
                        matches!(compatibility, crate::impls::WitnessCompatibility::Compatible(_))
                    }
                }) {
                    data_candidates.insert(requirement.clone(), (component.id.clone(), specialized_component_type));
                }
            }
        }
        let invalid_requirements = conformance_invalid_witnesses.get(impl_id).cloned().unwrap_or_default();
        let plan = crate::impls::build_conformance_witness_plan(
            contribution,
            store,
            declarations,
            hierarchy,
            surface,
            callable_signatures,
            conformance_witness_bodies,
            conformance_witness_visibilities,
            &invalid_requirements,
            conformance_invalid_members.contains(impl_id),
            &inherent_candidates,
            &inherent_mismatches,
            &deferred_inherent_candidates,
            &data_candidates,
            &terminal_candidates,
            conformance_associated_type_plans
                .get(impl_id)
                .cloned()
                .unwrap_or_else(|| crate::impls::ConformanceAssociatedTypePlan::empty(impl_id.clone())),
            conformance_index,
            trait_surfaces,
            conformance_associated_type_plans,
        );
        let plan_key = QueryKey::ConformanceDispatch(impl_id.clone());
        let mut dependencies = Vec::new();
        let trait_surface_key = QueryKey::TraitSurface(contribution.trait_ref.declaration.clone());
        ready_or_error(crate::db::query::query_bootstrap_trait_surface(db, surface.clone()))?;
        let Some(surface_fingerprint) = db.ready_product_fingerprint(&trait_surface_key) else {
            return Err(QueryOutcome::Failed(format!(
                "trait surface {:?} was not published before conformance plan {impl_id:?}",
                contribution.trait_ref.declaration
            )));
        };
        dependencies.push(crate::db::DependencyEdge {
            dependent: plan_key.clone(),
            dependency: trait_surface_key,
            observed_fingerprint: surface_fingerprint,
        });
        let mut seen_signatures = BTreeSet::new();
        for (_, member) in surface.iter() {
            if !seen_signatures.insert(member.callable.clone()) {
                continue;
            }
            let Some(signature) = callable_signatures.get(&member.callable) else {
                // Bodyless declarations have no callable-body signature row;
                // their declared contract is already covered by the exact
                // TraitSurface dependency above.
                continue;
            };
            ready_or_error(query_bootstrap_callable_signature(db, Arc::new(signature.clone())))?;
            let signature_key = QueryKey::CallableSignature(member.callable.clone());
            let Some(observed_fingerprint) = db.ready_product_fingerprint(&signature_key) else {
                return Err(QueryOutcome::Failed(format!(
                    "canonical callable signature {:?} was not published before conformance plan {impl_id:?}",
                    member.callable
                )));
            };
            dependencies.push(crate::db::DependencyEdge {
                dependent: plan_key.clone(),
                dependency: signature_key,
                observed_fingerprint,
            });
        }
        let mut plan_identity_hasher = DefaultHasher::new();
        impl_id.hash(&mut plan_identity_hasher);
        let input_fingerprint = crate::db::InputFingerprint::new(plan_identity_hasher.finish());
        db.publish_ready(
            plan_key,
            db.revision(),
            input_fingerprint,
            plan.fingerprint,
            crate::db::state::QueryValue::from_bytes([]),
            dependencies,
        )
        .map_err(|error| QueryOutcome::Failed(format!("failed to publish conformance plan {impl_id:?}: {error:?}")))?;
        if emit_incomplete_diagnostics
            && diagnostic_work_modules.contains(&contribution.source.module)
            && let crate::impls::ConformanceCompleteness::Incomplete { failures } = &plan.completeness
        {
            let mut diagnostic = SemanticDiagnostic::error_in(
                contribution.source.module.clone(),
                DiagnosticCode::ImplConformanceIncomplete,
                format!(
                    "conformance for `{}` does not satisfy all requirements",
                    contribution.trait_ref.declaration.name
                ),
                contribution.source.range,
            );
            for failure in failures {
                match failure {
                    crate::impls::ConformanceFailure::Behavioral(failure) => diagnostic.labels.push(DiagnosticLabel::new(
                        SemanticSourceSpan::new(contribution.source.module.clone(), contribution.source.range),
                        format!("requirement `{}`: {}", failure.requirement.selector, failure.reason),
                    )),
                    crate::impls::ConformanceFailure::AssociatedType(failure) => diagnostic.labels.push(DiagnosticLabel::new(
                        failure.source.clone(),
                        format!("associated type `{}`: {}", failure.name, failure.reason),
                    )),
                }
            }
            diags_by_module.entry(contribution.source.module.clone()).or_default().push(diagnostic);
        }
        let replacement = match conformance_witness_plans.get(impl_id) {
            Some(previous) if previous.fingerprint == plan.fingerprint => previous.clone(),
            Some(_) => {
                changed_plans.insert(impl_id.clone());
                Arc::new(plan)
            }
            None => {
                changed_plans.insert(impl_id.clone());
                Arc::new(plan)
            }
        };
        conformance_witness_plans.insert(impl_id.clone(), replacement);
    }
    Ok(changed_plans)
}

#[cfg(test)]
mod tests {
    use super::semantic_diagnostics_from_module_diagnostics;
    use super::source_body_for_callable;
    use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
    use crate::diagnostic::DiagnosticCode;
    use crate::identity::{CallableId, DeclarationId, DispatchSide, ImplId, ImplLocalId};
    use crate::impls::{CallableDefinitionOrigin, EffectiveCallableDefinition};
    use crate::signature::{CallableSemanticSignature, ReturnContractValidation};
    use crate::types::id::TypeId;
    use crate::types::parameter::TypeTerm;
    use phalcom_common::range::SourceRange;
    use phalcom_common::selector::Selector;
    use phalcom_modules::diagnostic::{ModuleDiagnostic, ModuleDiagnosticKind};
    use phalcom_modules::identity::ModuleId;
    use phalcom_modules::source::ModuleKind;
    use std::sync::Arc;

    #[test]
    fn canonical_module_diagnostic_mapping_preserves_owner_range_and_code() {
        let module = ModuleId::universe_root();
        let cases = [
            (
                ModuleDiagnosticKind::UnresolvedImport { path: ".missing".into() },
                DiagnosticCode::ModuleImportUnresolved,
            ),
            (
                ModuleDiagnosticKind::ModulePathNotExposed {
                    path: "dep.private".into(),
                    project: "dep".into(),
                },
                DiagnosticCode::ModuleExposureRejected,
            ),
            (
                ModuleDiagnosticKind::NonExportedImport {
                    module: "dep".into(),
                    name: "Hidden".into(),
                },
                DiagnosticCode::ModuleExportMissing,
            ),
            (
                ModuleDiagnosticKind::RelativeImportBeyondRoot { dots: 2, depth: 1 },
                DiagnosticCode::ModuleRelativeInvalidRoot,
            ),
            (ModuleDiagnosticKind::BindingCollision { name: "name".into() }, DiagnosticCode::ModuleLinkFailed),
        ];
        let diagnostics = cases
            .iter()
            .enumerate()
            .map(|(index, (kind, _))| {
                ModuleDiagnostic::new(
                    module.clone(),
                    kind.clone(),
                    SourceRange::new(index, index + 1),
                    format!("module failure {index}"),
                )
            })
            .collect::<Vec<_>>();

        let semantic = semantic_diagnostics_from_module_diagnostics(&diagnostics);

        assert_eq!(semantic.len(), cases.len());
        for (index, (diagnostic, (_, expected_code))) in semantic.iter().zip(cases.iter()).enumerate() {
            assert_eq!(diagnostic.code, *expected_code);
            assert_eq!(diagnostic.primary.module, module);
            assert_eq!(diagnostic.primary_range, SourceRange::new(index, index + 1));
            assert_eq!(diagnostic.message, format!("module failure {index}"));
        }
    }

    #[test]
    fn source_body_selection_uses_impl_identity_and_member_index() {
        let module = ModuleId::universe_root();
        let source = "impl User { ping() -> Unit { 1 } }\nimpl User { ping() -> Unit { 2 } }\n";
        let parsed = phalcom_ast::parse(source, 0);
        assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
        let unit = crate::source::ParsedModuleUnit::new(module.clone(), ModuleKind::Module, None, Arc::from(source), Arc::new(parsed.program));
        let owner = DeclarationId::new(module.clone(), "User".into());
        let selector = Selector::method("ping", []).unwrap();
        let callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);
        let signature = CallableSemanticSignature {
            callable: callable.clone(),
            owner,
            side: DispatchSide::Instance,
            selector,
            generics: None,
            parameters: Box::new([]),
            declared_return: DeclaredTypeFact::known(TypeTerm::Canonical(TypeId(0)), DeclaredTypeBasis::SourceAnnotation),
            return_validation: ReturnContractValidation::NotApplicable,
            inferred_return: None,
            source: None,
            implementation: phalcom_common::modifier::ImplementationKind::Phalcom,
            native_id: None,
            effects: phalcom_native_meta::EffectSpec::Unknown,
            raises: phalcom_native_meta::RaisesSpec::Unknown,
            flow: phalcom_native_meta::ReturnFlowSpec::Value,
            lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
        };
        let definition = EffectiveCallableDefinition {
            callable: callable.clone(),
            origin: CallableDefinitionOrigin::InherentImpl(ImplId::new(module.clone(), ImplLocalId(1))),
            source_member_index: 0,
            signature,
            inherent_body_context: None,
            body_provenance: crate::impls::CallableDefinitionBodyProvenance::SourceBody,
        };
        let (_, selected_range) = source_body_for_callable(&callable, Some(&definition), &unit).expect("body selected");
        let expected_range = match &unit.program.statements[1] {
            phalcom_ast::ast::Statement::Impl(impl_def) => match impl_def.members[0].behavior().unwrap() {
                phalcom_ast::ast::BehaviorMember::Method(method) => method.range,
                _ => panic!("expected method"),
            },
            _ => panic!("expected second impl"),
        };
        assert_eq!(selected_range, expected_range, "selector scanning must not choose the first impl body");
    }
}

#[cfg(test)]
#[path = "session/publication_tests.rs"]
mod publication_tests;
