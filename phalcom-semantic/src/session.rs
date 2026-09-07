//! Compiler-owned incremental workspace session (Spec 04.5 / Wave 5 / Tasks 16-18).

use crate::advisory::{
    AdvisoryBuiltins, AdvisoryCallableSummary, AdvisoryConfidence, AdvisoryFact, AdvisoryFlowContext, AdvisoryModuleProduct, AdvisoryOrigin,
    AdvisoryProductStatus, AdvisorySolver, AdvisorySolverBudget, AdvisorySolverNode, AdvisoryTargetResolution, AdvisoryWorkspace, CallableForShapeResolver,
    FormalCallResultResolver, MethodFamilyResolver, ModuleMemberResolver, advisory_shape_from_formal, advisory_shape_from_formal_for_receiver, analyze_expr,
    analyze_statements,
};
use crate::associated::{AssociatedFamilyTable, build_associated_surface};
use crate::checker::analysis::normal_return_summary;
use crate::checker::context::CheckingContext;
use crate::checker::declaration::check_class_field_initializers;
use crate::checker::statement::check_statement;
use crate::db::SemanticDb;
use crate::db::budget::{CancellationToken, QueryBudget};
use crate::db::key::QueryKey;
use crate::db::product::EnumRequirementsProduct;
use crate::db::query::{
    CallableBodyQuery, DeclarationSurfaceQuery, FormalQueryInputs, bootstrap_advisory_callable, query_advisory_callable, query_advisory_module,
    query_associated_surface, query_bootstrap_callable_signature, query_bootstrap_declaration_surface, query_bootstrap_hierarchy_edge,
    query_callable_body_with_formal_inputs, query_callable_signature_with_inputs, query_declaration_shell, query_declaration_surface, query_enum_declaration,
    query_enum_requirements, query_field_signature_with_inputs, query_hierarchy_edge, query_linked_interface, query_source_formal_attachment,
    query_source_structure, query_unlinked_interface,
};
use crate::db::state::QueryOutcome;
use crate::declarations::{
    DeclarationTypeInfo, DeclarationTypeTable, GenericSupertypeTemplate, NominalDeclarationHeader, TypeDeclarationShell, bootstrap_universe_declarations,
};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::SurfaceDispatchResolver;
use crate::enum_requirements::{EnumRequirementTable, check_enum_requirements};
use crate::enum_semantics::{EnumSemanticTable, VariantInfo};
use crate::identity::{CallableId, DeclarationId, DispatchSide, FieldId, ModuleId, SemanticTargetId, SourceOwner, SourceSiteId, WorkspaceId};
use crate::presentation::FormalSemanticProjection;
use crate::resolver::LinkedTypeResolver;
use crate::semantic_shard::ModuleSemanticStructureShard;
use crate::signature::{CallableSemanticSignature, CallableSignatureTable, FieldSignatureTable};
use crate::snapshot::SemanticSnapshot;
use crate::source::ParsedModuleUnit;
use crate::source_index::{SourceIndexContext, SourceSemanticIndex, build_source_scope_index, resolve_type_reference_targets};
use crate::type_alias::{TypeAliasInfo, TypeAliasTable};
use crate::types::annotation::{
    GenericBinderSite, TypeFormationOutcome, TypeFormationSite, TypeResolver, lower_scoped_type_alias_form, resolve_generic_signature, resolve_kind_syntax,
    type_level_binding_for_parameter,
};
use crate::types::id::{KindId, TypeId};
use crate::types::native::register_native_surfaces;
use crate::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner};
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::TypeStore;
use crate::workspace::SemanticWorkspaceInput;
use phalcom_ast::ast::{ClassMember, PackItem, PackLabel, Statement, TypeAnnotation, TypeAnnotationExpr};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::declaration::{DeclarationBlueprint, DeclarationKind, DeclarationRealizationError, DeclarationShellTable};
use phalcom_modules::graph::{SemanticEdge, SemanticEdgeKind, SemanticNodeId};
use phalcom_modules::interface::{InterfaceBuilder, LinkedExportTarget};
use phalcom_modules::linker::{LinkedProgram, SymbolId};
use phalcom_modules::{WorkspaceModuleSession, WorkspaceModuleSessionError, WorkspaceModuleUpdate, WorkspaceSourceBatchMutation, WorkspaceSourceMutation};
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

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
    pub reverse_candidates_considered: usize,
    pub semantic_dependents_recomputed: usize,
    pub semantic_dependents_reused: usize,
}

fn semantic_diagnostics_from_module_diagnostics(diagnostics: &[phalcom_modules::diagnostic::ModuleDiagnostic]) -> Vec<SemanticDiagnostic> {
    diagnostics
        .iter()
        .map(|diag| {
            let code = match &diag.kind {
                phalcom_modules::diagnostic::ModuleDiagnosticKind::RuntimeCycle { .. } => DiagnosticCode::ModuleRuntimeCycle,
                phalcom_modules::diagnostic::ModuleDiagnosticKind::UnresolvedImport { .. }
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::RelativeImportBeyondRoot { .. }
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::RelativeImportWithoutPackage
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::ModuleNotFound(_)
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::PackageNotFound(_)
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::ModulePathNotExposed { .. }
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::ImportOutsideSourceRoot(_)
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::UnknownImportRoot(_) => DiagnosticCode::ModuleImportUnresolved,
                phalcom_modules::diagnostic::ModuleDiagnosticKind::UnknownImportName { .. }
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::NonExportedImport { .. }
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::UnknownExport { .. }
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::DuplicateExport { .. }
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::DuplicateDeclaration { .. }
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::ExposeOutsidePackage
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::InvalidExposeTarget(_)
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::ImportOutsidePreamble
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::InvalidModuleMetadata { .. }
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::ModuleAttributeOutsideHeader
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::InvalidModuleName(_)
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::ParseError(_)
                | phalcom_modules::diagnostic::ModuleDiagnosticKind::InterfaceError(_) => DiagnosticCode::ModuleInterfaceFailed,
                _ => DiagnosticCode::ModuleLinkFailed,
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

fn ready_kind_for_predeclaration(store: &mut TypeStore, syntax: Option<&phalcom_ast::ast::KindSyntax>) -> Option<KindId> {
    match syntax.map_or(crate::types::annotation::KindResolution::Ready(KindId::TYPE), |kind| {
        resolve_kind_syntax(store, kind)
    }) {
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

#[derive(Clone, Debug, Default)]
struct SemanticModuleDelta {
    changed_modules: BTreeSet<ModuleId>,
    removed_modules: BTreeSet<ModuleId>,
    identity_changes: BTreeSet<ModuleId>,
    module_stats: phalcom_modules::session::WorkspaceModuleStats,
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
    db: SemanticDb,
    store: TypeStore,
    base_declarations: DeclarationTypeTable,
    base_hierarchy: MapTypeHierarchy,
    base_dispatch: SurfaceDispatchResolver,
    base_callable_signatures: CallableSignatureTable,
    base_enum_semantics: EnumSemanticTable,
    base_enum_products: Vec<Arc<crate::db::product::EnumDeclarationProduct>>,
    base_associated_surfaces: AssociatedFamilyTable,
    base_associated_surface_products: Vec<Arc<crate::associated::AssociatedSurface>>,
    base_enum_requirements: EnumRequirementTable,
    base_enum_requirement_products: Vec<(DeclarationId, Arc<crate::db::product::EnumRequirementsProduct>)>,
    sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    source_fingerprints: BTreeMap<ModuleId, u64>,
    field_lifecycle_fingerprints: BTreeMap<ModuleId, u64>,
    default_field_lifecycle: crate::checker::field_lifecycle::FieldLifecycleTable,
    semantic_structure_shards: BTreeMap<ModuleId, Arc<ModuleSemanticStructureShard>>,
    generic_header_dependencies: BTreeMap<DeclarationId, BTreeSet<DeclarationId>>,
    alias_sources: BTreeMap<DeclarationId, (ModuleId, phalcom_ast::ast::TypeAliasDef)>,
    alias_dependencies: BTreeMap<DeclarationId, BTreeSet<DeclarationId>>,
    alias_forms: Arc<BTreeMap<DeclarationId, TypeId>>,
    module_diagnostics: BTreeMap<ModuleId, Vec<phalcom_modules::diagnostic::ModuleDiagnostic>>,
    semantic_diagnostic_contributions: BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>>,
    last_snapshot: Option<Arc<SemanticSnapshot>>,
    last_known_good: Option<Arc<SemanticSnapshot>>,
}

impl Default for SemanticWorkspaceSession {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticWorkspaceSession {
    /// Creates a new workspace session with universe declarations and native surfaces bootstrapped.
    pub fn new() -> Self {
        Self::with_workspace(WorkspaceId::from_raw(1))
    }

    /// Creates a new workspace session for a specific workspace ID.
    pub fn with_workspace(workspace: WorkspaceId) -> Self {
        let db = SemanticDb::with_workspace(workspace);
        let mut store = TypeStore::new();

        let mut base_declarations = bootstrap_universe_declarations(&mut store, &crate::core_surface::universe_declaration);

        // `Some` and `None` are runtime support classes for the canonical
        // Option enum rather than exported class bindings. They still need
        // declaration-backed forms so superclass checks can report the
        // canonical sealed-inheritance diagnostic without making `None` a
        // value binding in expression resolution.
        for key in [phalcom_native_meta::UniverseKey::Some, phalcom_native_meta::UniverseKey::None] {
            let declaration = crate::core_surface::universe_declaration(key);
            let form = store.nominal_type(declaration.clone());
            let class_object_type = store.class_object_type(declaration.clone());
            base_declarations.insert(DeclarationTypeInfo {
                declaration,
                form,
                class_object_type,
                kind: KindId::TYPE,
                generic_signature: None,
                supertype_template: None,
            });
        }

        let mut base_hierarchy = MapTypeHierarchy::new();
        for relation in phalcom_native_meta::UNIVERSE_CLASS_RELATIONS {
            if let Some(superclass) = relation.superclass {
                base_hierarchy.insert(
                    crate::core_surface::universe_declaration(relation.class),
                    crate::core_surface::universe_declaration(superclass),
                );
            }
        }

        let mut base_dispatch = SurfaceDispatchResolver::new();
        let known_declarations: HashSet<DeclarationId> = base_declarations.iter().map(|(decl_id, _)| decl_id.clone()).collect();
        let dummy_linked = Arc::new(LinkedProgram {
            universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
            modules: BTreeMap::new(),
            graphs: phalcom_modules::graph::ModuleGraphs::default(),
            entry: ModuleId::universe_root(),
            initialization_order: vec![ModuleId::universe_root()],
        });
        let resolver = LinkedTypeResolver::new(dummy_linked, known_declarations, ModuleId::universe_root());
        let native_report = register_native_surfaces(&mut store, &base_declarations, &resolver, &ModuleId::universe_root(), &mut base_dispatch)
            .expect("canonical native surface must import during semantic bootstrap");
        crate::checker::context::ensure_core_object_type_tests(&mut store, &base_declarations, &mut base_dispatch);

        let mut base_callable_signatures = CallableSignatureTable::new();
        for (_, signature) in native_report.callable_signatures {
            base_callable_signatures.insert(signature);
        }
        let core_class_new = crate::checker::declaration_signature::canonical_core_class_new_signature(&mut store);
        if base_callable_signatures.get(&core_class_new.callable).is_none() {
            base_callable_signatures.insert(core_class_new);
        }

        let mut base_enum_semantics = EnumSemanticTable::new();
        let mut base_enum_products = Vec::new();
        let mut base_associated_surfaces = AssociatedFamilyTable::new();
        let mut base_associated_surface_products = Vec::new();
        let mut base_enum_requirements = EnumRequirementTable::new();
        let mut base_enum_requirement_products = Vec::new();

        let provider = phalcom_modules::UniverseSourceProvider::new();

        // Canonical source enums are not native-meta classes. Add their
        // declaration forms before constructing the linked resolver so
        // source users can resolve `Result` and `Ordering` through the same
        // core identity path as native declarations. `Option` already has a
        // native declaration form and keeps that canonical generic identity.
        for node in provider.nodes() {
            let path = phalcom_modules::ModulePath::from_components(
                node.path
                    .iter()
                    .map(|p| phalcom_modules::ModuleComponent::from_identifier(p).expect("valid component"))
                    .collect::<Vec<_>>(),
            );
            let module_id = phalcom_modules::ModuleId::universe(path);
            let parsed = provider
                .load_parsed(&module_id)
                .unwrap_or_else(|e| panic!("failed to load universe module {module_id}: {e}"));
            for stmt in &parsed.program.statements {
                let phalcom_ast::ast::Statement::Enum(enum_def) = stmt else {
                    continue;
                };
                let decl_id = DeclarationId::new(module_id.clone(), enum_def.name.clone().into());
                if base_declarations.get(&decl_id).is_some() {
                    continue;
                }

                let mut parameter_ids = Vec::new();
                let mut parameter_kinds = Vec::new();
                for (index, parameter) in enum_def.generic_parameters.iter().enumerate() {
                    let Some(kind) = ready_kind_for_predeclaration(&mut store, parameter.kind.as_ref()) else {
                        parameter_ids.clear();
                        break;
                    };
                    let parameter_id = store.intern_type_parameter(TypeParameterData::new(
                        TypeParameterOwner::Declaration(decl_id.clone()),
                        index as u32,
                        parameter.name.clone(),
                        kind,
                    ));
                    parameter_ids.push(parameter_id);
                    parameter_kinds.push(kind);
                }

                if parameter_ids.len() != enum_def.generic_parameters.len() {
                    continue;
                }

                let (form, kind, generic_signature) = if parameter_ids.is_empty() {
                    (store.nominal_type(decl_id.clone()), KindId::TYPE, None)
                } else {
                    let kind = store.arrow_kind(parameter_kinds.into_boxed_slice(), KindId::TYPE);
                    let form = store.nominal_form(decl_id.clone(), kind);
                    let signature =
                        crate::types::parameter::GenericSignature::new(TypeParameterOwner::Declaration(decl_id.clone()), parameter_ids.into_boxed_slice());
                    (form, kind, Some(signature))
                };

                base_declarations.insert(DeclarationTypeInfo {
                    declaration: decl_id.clone(),
                    form,
                    class_object_type: store.class_object_type(decl_id),
                    kind,
                    generic_signature,
                    supertype_template: None,
                });
            }
        }

        for node in provider.nodes() {
            let path = phalcom_modules::ModulePath::from_components(
                node.path
                    .iter()
                    .map(|p| phalcom_modules::ModuleComponent::from_identifier(p).expect("valid component"))
                    .collect::<Vec<_>>(),
            );
            let module_id = phalcom_modules::ModuleId::universe(path);
            let parsed = provider
                .load_parsed(&module_id)
                .unwrap_or_else(|e| panic!("failed to load universe module {module_id}: {e}"));
            for stmt in &parsed.program.statements {
                if let phalcom_ast::ast::Statement::Enum(enum_def) = stmt {
                    let decl_id = DeclarationId::new(module_id.clone(), enum_def.name.clone().into());
                    let Some(enum_product) =
                        crate::checker::enum_declaration::build_enum_semantics(&decl_id, enum_def, &mut store, &base_declarations, &resolver, &module_id)
                    else {
                        continue;
                    };
                    base_enum_semantics.insert_enum(enum_product.info.clone());
                    for v in enum_product.variants.iter() {
                        base_enum_semantics.insert_variant(Arc::new(v.clone()));
                    }
                    base_enum_products.push(Arc::new(enum_product));

                    let mut behavior_ctx = crate::checker::CheckingContext::new_with_dispatch_ref(
                        &mut store,
                        &base_hierarchy,
                        &resolver,
                        &base_declarations,
                        &base_dispatch,
                        module_id.clone(),
                    );
                    behavior_ctx.attach_enum_semantics(&base_enum_semantics);
                    let behavior_product = crate::checker::enum_behavior::build_enum_behavior(&mut behavior_ctx, &decl_id, enum_def);

                    let mut surface = base_dispatch.surface(&decl_id).cloned().unwrap_or_default();
                    for default_sig in behavior_product.root_defaults.iter() {
                        base_callable_signatures.insert(default_sig.clone());
                        let projection = crate::checker::declaration_signature::project_semantic_signature(default_sig);
                        surface.add_callable(default_sig.side, projection);
                    }
                    base_dispatch.register_surface(decl_id.clone(), surface);
                    if let Some(ty) = base_declarations.form(&decl_id) {
                        base_dispatch.register_type(ty, decl_id.clone());
                    }

                    for case_sigs in behavior_product.case_implementations.values() {
                        for case_sig in case_sigs.iter() {
                            base_callable_signatures.insert(case_sig.clone());
                        }
                    }

                    let behavior_bases: std::collections::HashSet<phalcom_common::selector::SelectorBase> = enum_def
                        .members
                        .iter()
                        .filter_map(|m| match m {
                            phalcom_ast::ast::EnumMember::Behavior(b) => {
                                let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(b);
                                Some(syntax.selector_base())
                            }
                            _ => None,
                        })
                        .collect();

                    let (assoc_surface, _assoc_diags) = build_associated_surface(
                        &decl_id,
                        Some(
                            &enum_def
                                .members
                                .iter()
                                .filter_map(|m| match m {
                                    phalcom_ast::ast::EnumMember::Variant(v) => {
                                        let sel = phalcom_ast::selector::selector_from_variant(v);
                                        Some(crate::identity::VariantId::new(decl_id.clone(), sel))
                                    }
                                    _ => None,
                                })
                                .collect::<Vec<_>>(),
                        ),
                        &behavior_bases,
                        &std::collections::HashSet::new(),
                        &module_id,
                        Some(crate::diagnostic::SemanticSourceSpan::new(module_id.clone(), enum_def.range)),
                    );
                    base_associated_surfaces.insert(decl_id.clone(), assoc_surface.clone());
                    base_associated_surface_products.push(assoc_surface);

                    let variants_info: Vec<VariantInfo> = enum_def
                        .members
                        .iter()
                        .filter_map(|m| match m {
                            phalcom_ast::ast::EnumMember::Variant(v) => {
                                let sel = phalcom_ast::selector::selector_from_variant(v);
                                let vid = crate::identity::VariantId::new(decl_id.clone(), sel);
                                base_enum_semantics.variant_info(&vid).cloned()
                            }
                            _ => None,
                        })
                        .collect();

                    let mut case_methods_map: HashMap<crate::identity::VariantId, Vec<crate::signature::CallableSemanticSignature>> = HashMap::new();
                    for (v_id, sigs) in &behavior_product.case_implementations {
                        case_methods_map.insert(v_id.clone(), sigs.to_vec());
                    }

                    let (case_statuses, req_diags) = check_enum_requirements(
                        &decl_id,
                        base_enum_semantics.enum_info(&decl_id).unwrap(),
                        &variants_info,
                        &behavior_product.root_requirements,
                        &case_methods_map,
                        &mut store,
                        &base_hierarchy,
                        &module_id,
                    );
                    base_enum_requirements.insert(decl_id.clone(), Arc::from(behavior_product.root_requirements.clone()), case_statuses.clone());
                    let req_product = Arc::new(EnumRequirementsProduct {
                        requirements: Arc::from(behavior_product.root_requirements),
                        case_statuses,
                        diagnostics: req_diags,
                    });
                    base_enum_requirement_products.push((decl_id, req_product));
                }
            }
        }

        Self {
            workspace,
            module_session: WorkspaceModuleSession::new(),
            db,
            store,
            base_declarations,
            base_hierarchy,
            base_dispatch,
            base_callable_signatures,
            base_enum_semantics,
            base_enum_products,
            base_associated_surfaces,
            base_associated_surface_products,
            base_enum_requirements,
            base_enum_requirement_products,
            sources: BTreeMap::new(),
            source_fingerprints: BTreeMap::new(),
            field_lifecycle_fingerprints: BTreeMap::new(),
            default_field_lifecycle: crate::checker::field_lifecycle::FieldLifecycleTable::default(),
            semantic_structure_shards: BTreeMap::new(),
            generic_header_dependencies: BTreeMap::new(),
            alias_sources: BTreeMap::new(),
            alias_dependencies: BTreeMap::new(),
            alias_forms: Arc::new(BTreeMap::new()),
            module_diagnostics: BTreeMap::new(),
            semantic_diagnostic_contributions: BTreeMap::new(),
            last_snapshot: None,
            last_known_good: None,
        }
    }

    pub fn workspace(&self) -> WorkspaceId {
        self.workspace
    }

    /// Returns persistent project/source/module ownership used by compiler updates.
    pub fn module_session(&self) -> &WorkspaceModuleSession {
        &self.module_session
    }

    /// Mutably borrows persistent module ownership for a worker-side batch.
    pub fn module_session_mut(&mut self) -> &mut WorkspaceModuleSession {
        &mut self.module_session
    }

    /// Applies one module lifecycle mutation and publishes its semantic snapshot.
    pub fn apply_module_mutation(&mut self, mutation: WorkspaceSourceMutation) -> Result<SemanticWorkspaceUpdate, WorkspaceModuleSessionError> {
        let update = self.module_session.apply(mutation)?;
        Ok(self.update_module_workspace(update))
    }

    /// Applies one heterogeneous module/source batch and publishes one
    /// canonical semantic snapshot for the resulting workspace generation.
    pub fn apply_module_mutations<I>(&mut self, mutations: I) -> Result<SemanticWorkspacePublication, WorkspaceModuleSessionError>
    where
        I: IntoIterator<Item = WorkspaceSourceBatchMutation>,
    {
        let update = self.module_session.apply_batch(mutations)?;
        Ok(self.update_module_workspace(update))
    }

    /// Publishes semantic products for an already-linked module workspace update.
    pub fn update_module_workspace(&mut self, update: WorkspaceModuleUpdate) -> SemanticWorkspaceUpdate {
        let generation = self.module_session.generation();
        let delta = SemanticModuleDelta {
            changed_modules: update.changed_modules.clone(),
            removed_modules: update.removed_modules.clone(),
            identity_changes: update.identity_changes.clone(),
            module_stats: update.stats.clone(),
        };
        self.update_with_delta(
            SemanticWorkspaceInput {
                linked: update.linked,
                sources: update.sources,
                interfaces: update.interfaces,
                import_products: update.import_products,
                import_sites_by_module: update.sites_by_importer,
                require_canonical_import_products: true,
                diagnostics: update.diagnostics,
                blocked_modules: update.blocked_modules,
                generation,
                topology: Some(update.topology),
                reverse_imports: Some(update.reverse_importers),
            },
            Some(delta),
        )
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
        &mut self.store
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
    pub fn update(&mut self, input: SemanticWorkspaceInput) -> SemanticWorkspaceUpdate {
        self.update_with_delta(input, None)
    }

    fn update_with_delta(&mut self, input: SemanticWorkspaceInput, module_delta: Option<SemanticModuleDelta>) -> SemanticWorkspaceUpdate {
        let generation = input.generation;
        let sources = input.sources.clone();
        let linked = input.linked.clone();
        let module_stats = module_delta.as_ref().map(|delta| delta.module_stats.clone());
        self.update_with_budget_and_cancel_and_delta(input, QueryBudget::default(), &CancellationToken::new(), module_delta)
            .unwrap_or_else(|_| {
                let snapshot = self.last_known_good.clone().unwrap_or_else(|| {
                    Arc::new(SemanticSnapshot::new_with_callable_analyses(
                        self.workspace,
                        self.db.revision(),
                        generation,
                        Arc::new(self.store.clone()),
                        Arc::new(sources),
                        Arc::new(self.base_dispatch.surfaces().clone()),
                        Arc::new(self.base_dispatch.clone()),
                        Arc::new(self.base_callable_signatures.clone()),
                        Arc::new(self.base_declarations.clone()),
                        Arc::new(self.base_hierarchy.clone()),
                        Arc::new(BTreeMap::new()),
                        Arc::new(linked.graphs.semantics.clone()),
                        Arc::new(HashMap::new()),
                    ))
                });
                SemanticWorkspaceUpdate {
                    snapshot,
                    invalidated: Arc::from(Vec::new()),
                    recomputed: Arc::from(Vec::new()),
                    stats: SemanticUpdateStats::default(),
                    module_stats,
                    effects: SemanticPublicationEffects::default(),
                }
            })
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

        self.db.begin_revision();
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
                .filter(|(module, linked)| previous.module_products.linked.get(*module) != Some(&linked.interface))
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

        let previous_structure_shards = self.semantic_structure_shards.clone();
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
        let contribution_delta = contribution_delta(&semantic_structure_shards, &previous_structure_shards);
        self.semantic_structure_shards = semantic_structure_shards;
        let current_declarations = self
            .semantic_structure_shards
            .values()
            .flat_map(|shard| shard.declaration_header_fingerprints.keys().cloned())
            .collect::<BTreeSet<_>>();
        let removed_callable_bodies = previous_snapshot
            .as_ref()
            .into_iter()
            .flat_map(|snapshot| snapshot.callable_analyses.keys())
            .filter(|callable| callable.declaration_owner().name.as_ref() != "<main>" && !current_declarations.contains(callable.declaration_owner()))
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut removed_query_roots = BTreeSet::new();
        for declaration in &contribution_delta.declarations_removed {
            removed_query_roots.insert(QueryKey::DeclarationShell(declaration.clone()));
            removed_query_roots.insert(QueryKey::DeclarationSurface(declaration.clone()));
            removed_query_roots.insert(QueryKey::HierarchyEdge(declaration.clone()));
            removed_query_roots.insert(QueryKey::LinkedName(declaration.module.clone(), declaration.name.to_string()));
            removed_query_roots.insert(QueryKey::PublicExport(declaration.module.clone(), declaration.name.to_string()));
        }
        for callable in &contribution_delta.callable_signatures_removed {
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
            self.db.retire_query(&QueryKey::HierarchyEdge(declaration.clone()));
        }
        for callable in &contribution_delta.callable_signatures_removed {
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

        self.sources = input.sources.clone();
        self.source_fingerprints = new_fingerprints;

        // Generic declaration headers form a second exact dependency graph:
        // a provider header edit must reach only headers whose bounds,
        // constraints, or superclass template mention that provider.
        let mut generic_header_work = contribution_delta.declarations.clone();
        let mut changed_header_dependencies = contribution_delta.declarations.clone();
        // A structural edit can rebind names used by a superclass template
        // without changing the template text itself.  Rebuild the exact
        // declaration headers owned by those changed shards so retained
        // nominal metadata cannot preserve the previous resolved target.
        for module in contribution_delta.structural_modules.clone() {
            if let Some(shard) = self.semantic_structure_shards.get(&module) {
                generic_header_work.extend(shard.declaration_header_fingerprints.keys().cloned());
            }
        }
        changed_header_dependencies.extend(generic_header_work.iter().cloned());
        loop {
            let additions = self
                .generic_header_dependencies
                .iter()
                .filter(|(consumer, dependencies)| {
                    !generic_header_work.contains(*consumer) && dependencies.iter().any(|dependency| changed_header_dependencies.contains(dependency))
                })
                .map(|(consumer, _)| consumer.clone())
                .collect::<Vec<_>>();
            if additions.is_empty() {
                break;
            }
            changed_header_dependencies.extend(additions.iter().cloned());
            generic_header_work.extend(additions);
        }

        // Convert source contribution deltas into typed query roots. The
        // dependency index then propagates those roots through exact reverse
        // edges; a changed module no longer seeds every cached query owned by
        // that module.
        let mut hierarchy_edge_work = BTreeSet::new();
        let mut declaration_surface_work = BTreeSet::new();
        let mut callable_signature_work = BTreeSet::new();
        let mut callable_body_work = contribution_delta.callable_bodies.clone();
        callable_body_work.extend(retired_body_dependents);
        let mut field_signature_work = BTreeSet::new();
        let mut declaration_shell_work = BTreeSet::new();
        let mut semantic_work_modules = if previous_snapshot.is_none() {
            current_modules.clone()
        } else {
            let mut roots = BTreeSet::new();
            for declaration in &generic_header_work {
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
            if let Some(previous) = previous_snapshot.as_ref() {
                for (module, linked) in &input.linked.modules {
                    if previous.module_products.linked.get(module) != Some(&linked.interface) {
                        roots.insert(QueryKey::LinkedInterface(module.clone()));
                    }
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
            let closure = self.db.index().reverse_closure(roots);
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
                .map(|callable| callable.declaration_owner().clone()),
        );
        declaration_surface_work.extend(contribution_delta.field_signatures.iter().map(|field| field.owner.clone()));
        let changed_declarations = hierarchy_edge_work
            .iter()
            .chain(declaration_shell_work.iter())
            .chain(declaration_surface_work.iter())
            .collect::<BTreeSet<_>>();
        for (importer_id, linked_mod) in &input.linked.modules {
            if !current_modules.contains(importer_id) || semantic_work_modules.contains(importer_id) {
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
        let formal_work_modules = structural_work_modules
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
                .map(|callable| callable.declaration_owner().clone())
                .filter(|declaration| current_modules.contains(&declaration.module)),
        );
        formal_declarations.extend(
            field_signature_work
                .iter()
                .map(|field| field.owner.clone())
                .filter(|declaration| current_modules.contains(&declaration.module)),
        );
        let formal_declaration_modules = formal_declarations
            .iter()
            .map(|declaration| declaration.module.clone())
            .collect::<BTreeSet<_>>();
        let mut diagnostic_work_modules = semantic_work_modules.clone();
        diagnostic_work_modules.extend(hierarchy_work_modules.iter().cloned());
        diagnostic_work_modules.extend(formal_work_modules.iter().cloned());
        diagnostic_work_modules.extend(field_lifecycle_changed_modules.iter().cloned());
        diagnostic_work_modules.extend(modules_with_changed_module_diagnostics.iter().cloned());
        diagnostic_work_modules.extend(removed_modules.iter().cloned());
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
                    declarations.insert(info.clone());
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
            initial_blueprints.extend(shard.declarations.iter().cloned());
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
                    let kind = if !class_def.generic_parameters.is_empty() {
                        let mut param_kinds = Vec::with_capacity(class_def.generic_parameters.len());
                        for parameter in &class_def.generic_parameters {
                            let Some(kind) = ready_kind_for_predeclaration(&mut self.store, parameter.kind.as_ref()) else {
                                param_kinds.clear();
                                break;
                            };
                            param_kinds.push(kind);
                        }
                        if param_kinds.len() != class_def.generic_parameters.len() {
                            continue;
                        }
                        self.store.arrow_kind(param_kinds.into_boxed_slice(), KindId::TYPE)
                    } else {
                        KindId::TYPE
                    };

                    let form = if kind == KindId::TYPE {
                        self.store.nominal_type(decl_id.clone())
                    } else {
                        self.store.nominal_form(decl_id.clone(), kind)
                    };
                    let class_obj_type = self.store.class_object_type(decl_id.clone());
                    declarations.insert(DeclarationTypeInfo {
                        declaration: decl_id,
                        form,
                        class_object_type: class_obj_type,
                        kind,
                        generic_signature: None,
                        supertype_template: None,
                    });
                } else if let Statement::Enum(enum_def) = stmt {
                    let decl_id = DeclarationId::new(module_id.clone(), enum_def.name.clone().into());
                    let retain_previous_header = previous_snapshot
                        .as_ref()
                        .is_some_and(|previous| !generic_header_work.contains(&decl_id) && previous.declarations.get(&decl_id).is_some());
                    if retain_previous_header {
                        continue;
                    }
                    let kind = if !enum_def.generic_parameters.is_empty() {
                        let mut param_kinds = Vec::with_capacity(enum_def.generic_parameters.len());
                        for parameter in &enum_def.generic_parameters {
                            let Some(kind) = ready_kind_for_predeclaration(&mut self.store, parameter.kind.as_ref()) else {
                                param_kinds.clear();
                                break;
                            };
                            param_kinds.push(kind);
                        }
                        if param_kinds.len() != enum_def.generic_parameters.len() {
                            continue;
                        }
                        self.store.arrow_kind(param_kinds.into_boxed_slice(), KindId::TYPE)
                    } else {
                        KindId::TYPE
                    };

                    let form = if kind == KindId::TYPE {
                        self.store.nominal_type(decl_id.clone())
                    } else {
                        self.store.nominal_form(decl_id.clone(), kind)
                    };
                    let class_obj_type = self.store.class_object_type(decl_id.clone());
                    declarations.insert(DeclarationTypeInfo {
                        declaration: decl_id,
                        form,
                        class_object_type: class_obj_type,
                        kind,
                        generic_signature: None,
                        supertype_template: None,
                    });
                }
            }
        }
        shell_table.predeclare(initial_blueprints);

        // 3. Construct LinkedTypeResolver
        let mut known_declarations: HashSet<DeclarationId> = declarations.iter().map(|(decl_id, _)| decl_id.clone()).collect();
        known_declarations.extend(alias_declarations.iter().cloned());
        let resolver = LinkedTypeResolver::with_retained_alias_forms(
            input.linked.clone(),
            known_declarations.clone(),
            ModuleId::universe_root(),
            self.alias_forms.clone(),
        );

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
        let mut blocked_declarations = BTreeSet::new();
        let mut type_aliases = previous_snapshot
            .as_ref()
            .map_or_else(TypeAliasTable::new, |snapshot| (*snapshot.type_aliases).clone());
        let mut alias_dependencies = self.alias_dependencies.clone();
        let mut alias_recomputed_declarations = declaration_shell_work
            .iter()
            .filter(|declaration| type_aliases.contains_key(declaration))
            .cloned()
            .collect::<BTreeSet<_>>();
        for module in &structural_recomputed_modules {
            alias_recomputed_declarations.extend(type_aliases.declarations_for_module(module).cloned());
            if let Some(shard) = self.semantic_structure_shards.get(module) {
                alias_recomputed_declarations.extend(shard.aliases.iter().cloned());
            }
        }
        // Alias source edits are not identity edits, so the source shard delta
        // alone cannot seed retained alias consumers. Walk the compact retained
        // dependency graph in reverse to add exactly those consumers whose
        // lowered forms depend on a changed alias.
        loop {
            let additions = alias_dependencies
                .iter()
                .filter(|(consumer, dependencies)| {
                    alias_sources.contains_key(*consumer)
                        && !alias_recomputed_declarations.contains(*consumer)
                        && dependencies.iter().any(|dependency| alias_recomputed_declarations.contains(dependency))
                })
                .map(|(consumer, _)| consumer.clone())
                .collect::<Vec<_>>();
            if additions.is_empty() {
                break;
            }
            alias_recomputed_declarations.extend(additions);
        }
        let mut alias_form_updates = BTreeMap::new();
        for declaration in &alias_recomputed_declarations {
            resolver.remove_alias_form(declaration.clone());
            alias_form_updates.insert(declaration.clone(), None);
        }
        for module in &structural_recomputed_modules {
            type_aliases.remove_module(module);
        }
        for declaration in &alias_recomputed_declarations {
            type_aliases.remove(declaration);
            alias_dependencies.remove(declaration);
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
            let mut dependencies = BTreeSet::new();
            collect_alias_dependencies(&alias.body, module_id, &resolver, &alias_declarations, &mut dependencies);
            alias_dependencies.insert(declaration.clone(), dependencies);
        }
        stats.alias_dependency_nodes_considered = alias_recomputed_declarations.len();
        let alias_cycle_seeds = if previous_snapshot.is_none() {
            alias_dependencies.keys().cloned().collect()
        } else {
            alias_recomputed_declarations.clone()
        };
        let alias_cycles = find_alias_cycles_from_seeds(&alias_dependencies, &alias_cycle_seeds);
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

        // Lower aliases after their alias dependencies. This matters for
        // imported aliases and preserves transparent substitution across
        // modules; cycles were removed above and never reach publication.
        let mut pending_aliases = alias_recomputed_declarations.clone();
        for declaration in &alias_cycles {
            pending_aliases.remove(declaration);
        }
        while let Some(declaration) = pending_aliases
            .iter()
            .find(|declaration| {
                alias_dependencies
                    .get(*declaration)
                    .is_none_or(|dependencies| dependencies.iter().all(|dependency| !pending_aliases.contains(dependency)))
            })
            .cloned()
        {
            let Some((module_id, alias)) = alias_sources.get(&declaration) else {
                pending_aliases.remove(&declaration);
                continue;
            };
            let signature = if alias.generic_parameters.is_empty() {
                None
            } else {
                let outcome = resolve_generic_signature(
                    &mut self.store,
                    &declarations,
                    &resolver,
                    &TypeFormationSite::module(module_id.clone()),
                    TypeParameterOwner::Declaration(declaration.clone()),
                    GenericBinderSite::TypeAlias,
                    &alias.generic_parameters,
                    alias.where_clause.as_ref(),
                    diags_by_module.entry(module_id.clone()).or_default(),
                );
                retain_generic_signature(outcome, module_id, alias.range, diags_by_module.entry(module_id.clone()).or_default())
            };
            if !alias.generic_parameters.is_empty() && signature.is_none() {
                pending_aliases.remove(&declaration);
                continue;
            }
            let site = TypeFormationSite::module(module_id.clone());
            let mut diagnostics = Vec::new();
            let form = match signature.as_ref() {
                Some(signature) => lower_scoped_type_alias_form(&mut self.store, &declarations, &resolver, &site, signature, &alias.body, &mut diagnostics),
                None => crate::types::annotation::resolve_type_form(&mut self.store, &declarations, &resolver, &site, &alias.body, &mut diagnostics),
            };
            diags_by_module.entry(module_id.clone()).or_default().extend(diagnostics);
            let TypeFormationOutcome::Ready(form) = form else {
                pending_aliases.remove(&declaration);
                continue;
            };
            let dependencies = alias_dependencies.get(&declaration).cloned().unwrap_or_default();
            let info = TypeAliasInfo {
                declaration: declaration.clone(),
                kind: self.store.kind_of(form),
                kind_shape: self.store.format_kind(self.store.kind_of(form)).into_boxed_str(),
                generic_signature: signature,
                structural_form: self.store.format_type(form).into_boxed_str(),
                form,
                dependencies: dependencies.into_iter().collect::<Vec<_>>().into_boxed_slice(),
                source: crate::diagnostic::SemanticSourceSpan::new(module_id.clone(), alias.range),
            };
            resolver.insert_alias_form(declaration.clone(), form);
            alias_form_updates.insert(declaration.clone(), Some(form));
            type_aliases.insert(info);
            pending_aliases.remove(&declaration);
        }

        // Generic signatures and supertype templates. Retain the dependency
        // contribution for unchanged headers; only structurally changed or
        // exact reverse-dependent headers are rebuilt below.
        let mut next_generic_header_dependencies = self.generic_header_dependencies.clone();
        for module in &structural_recomputed_modules {
            if let Some(previous_shard) = previous_structure_shards.get(module) {
                for declaration in previous_shard.declaration_header_fingerprints.keys() {
                    let still_declared = self
                        .semantic_structure_shards
                        .get(module)
                        .is_some_and(|shard| shard.declaration_header_fingerprints.contains_key(declaration));
                    if !still_declared || generic_header_work.contains(declaration) {
                        next_generic_header_dependencies.remove(declaration);
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
                                phalcom_ast::ast::GenericConstraintSyntax::Invalid { .. } => {}
                            }
                        }
                    }
                    header_dependencies.remove(&decl_id);
                    next_generic_header_dependencies.insert(decl_id.clone(), header_dependencies);
                    let formation_site = TypeFormationSite::member(module_id.clone(), decl_id.clone(), DispatchSide::Instance);
                    let generic_signature = if !class_def.generic_parameters.is_empty() {
                        let outcome = resolve_generic_signature(
                            &mut self.store,
                            &declarations,
                            &resolver,
                            &formation_site,
                            TypeParameterOwner::Declaration(decl_id.clone()),
                            GenericBinderSite::NominalDeclaration,
                            &class_def.generic_parameters,
                            class_def.where_clause.as_ref(),
                            diags_by_module.entry(module_id.clone()).or_default(),
                        );
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
                        continue;
                    }
                    let header = NominalDeclarationHeader::from_signature(&mut self.store, decl_id.clone(), generic_signature);

                    let supertype_template = if let Some(super_ann) = &class_def.superclass {
                        let type_params_map = if let Some(ref sig) = header.generic_signature {
                            let mut map = std::collections::HashMap::new();
                            for &param_id in sig.parameters.iter() {
                                let name = self.store.type_parameter(param_id).name.to_string();
                                let binding = type_level_binding_for_parameter(&mut self.store, param_id);
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
                            &mut self.store,
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
                                if self.store.kind_of(ty) != KindId::TYPE && !is_canonical_erased_iterable_supertype(parsed_unit, &self.store, ty) {
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
                                Some(ty)
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

                    let type_info = header.into_type_info(supertype_template);
                    if let Some(template) = type_info.supertype_template.clone() {
                        hierarchy.insert_template(template);
                    }
                    declarations.insert(type_info);
                } else if let Statement::Enum(enum_def) = stmt {
                    let decl_id = DeclarationId::new(module_id.clone(), enum_def.name.clone().into());
                    if previous_snapshot.is_some() && !generic_header_work.contains(&decl_id) {
                        continue;
                    }
                    let mut header_dependencies = BTreeSet::new();
                    if let Some(where_clause) = &enum_def.where_clause {
                        for constraint in &where_clause.constraints {
                            match constraint {
                                phalcom_ast::ast::GenericConstraintSyntax::Subtype { lower, upper, .. }
                                | phalcom_ast::ast::GenericConstraintSyntax::Equivalent { left: lower, right: upper, .. } => {
                                    collect_type_annotation_declarations(lower, &module_id, &resolver, &mut header_dependencies);
                                    collect_type_annotation_declarations(upper, &module_id, &resolver, &mut header_dependencies);
                                }
                                phalcom_ast::ast::GenericConstraintSyntax::Invalid { .. } => {}
                            }
                        }
                    }
                    header_dependencies.remove(&decl_id);
                    next_generic_header_dependencies.insert(decl_id.clone(), header_dependencies);
                    let formation_site = TypeFormationSite::member(module_id.clone(), decl_id.clone(), DispatchSide::Instance);
                    let generic_signature = if !enum_def.generic_parameters.is_empty() {
                        let outcome = resolve_generic_signature(
                            &mut self.store,
                            &declarations,
                            &resolver,
                            &formation_site,
                            TypeParameterOwner::Declaration(decl_id.clone()),
                            GenericBinderSite::NominalDeclaration,
                            &enum_def.generic_parameters,
                            enum_def.where_clause.as_ref(),
                            diags_by_module.entry(module_id.clone()).or_default(),
                        );
                        Some(retain_generic_signature(
                            outcome,
                            &module_id,
                            enum_def.range,
                            diags_by_module.entry(module_id.clone()).or_default(),
                        ))
                        .flatten()
                    } else {
                        None
                    };

                    if !enum_def.generic_parameters.is_empty() && generic_signature.is_none() {
                        continue;
                    }
                    let header = NominalDeclarationHeader::from_signature(&mut self.store, decl_id.clone(), generic_signature);

                    declarations.insert(header.into_type_info(None));
                }
            }
        }

        // Publish declaration type metadata as explicit DB products before any
        // formal surface, signature, or body query can consume it.
        let mut published_shells = BTreeSet::new();
        for (declaration, info) in self.base_declarations.iter() {
            published_shells.insert(declaration.clone());
            match query_declaration_shell(&mut self.db, Arc::new(TypeDeclarationShell::Nominal(info.clone()))) {
                QueryOutcome::Ready(_) => {}
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            }
        }
        for (declaration, surface) in self.base_dispatch.surfaces() {
            match query_bootstrap_declaration_surface(&mut self.db, declaration.clone(), Arc::new(surface.clone())) {
                QueryOutcome::Ready(_) => {}
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            }
        }
        for (_, signature) in self.base_callable_signatures.iter() {
            match query_bootstrap_callable_signature(&mut self.db, Arc::new(signature.clone())) {
                QueryOutcome::Ready(_) => {}
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            }
        }
        for relation in phalcom_native_meta::UNIVERSE_CLASS_RELATIONS {
            let class_decl = crate::core_surface::universe_declaration(relation.class);
            let super_decl = relation.superclass.map(crate::core_surface::universe_declaration);
            match query_bootstrap_hierarchy_edge(&mut self.db, class_decl, super_decl) {
                QueryOutcome::Ready(_) => {}
                QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
            }
        }

        // Publish alias shells in dependency order so each edge records the
        // dependency's actual structural product fingerprint.
        let alias_shell_work = if previous_snapshot.is_none() {
            alias_recomputed_declarations.clone()
        } else {
            alias_recomputed_declarations.clone()
        };
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
                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
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
                        linked_interface.clone(),
                        &resolver,
                        input.linked.as_ref(),
                        &declarations,
                        &input.import_products,
                    ) {
                        QueryOutcome::Ready(edge) => edge,
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    };

                    if let Some(super_decl) = &edge.super_decl {
                        hierarchy.insert(class_decl.clone(), super_decl.clone());
                    } else if let Some(super_ref) = class_def.superclass_ref() {
                        diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                            module_id.clone(),
                            DiagnosticCode::AnnotationUnresolved,
                            format!("unresolved superclass `{}`", super_ref.root),
                            super_ref.range,
                        ));
                    }
                } else if let Statement::Enum(enum_def) = stmt {
                    let enum_decl = DeclarationId::new(module_id.clone(), enum_def.name.clone().into());
                    if blocked_declarations.contains(&enum_decl) || (previous_snapshot.is_some() && !hierarchy_query_work.contains(&enum_decl)) {
                        continue;
                    }
                    let edge = match query_hierarchy_edge(
                        &mut self.db,
                        enum_decl.clone(),
                        parsed_unit.clone(),
                        linked_interface.clone(),
                        &resolver,
                        input.linked.as_ref(),
                        &declarations,
                        &input.import_products,
                    ) {
                        QueryOutcome::Ready(edge) => edge,
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    };
                    if let Some(super_decl) = &edge.super_decl {
                        hierarchy.insert(enum_decl, super_decl.clone());
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
        let mut field_signatures = previous_snapshot
            .as_ref()
            .map_or_else(FieldSignatureTable::new, |snapshot| (*snapshot.field_signatures).clone());
        let previous_callable_formal_states = callable_signature_work
            .iter()
            .filter_map(|callable| callable_signatures.get(callable).cloned().map(|signature| (callable.clone(), signature)))
            .collect::<BTreeMap<_, _>>();
        for declaration in &declaration_surface_work {
            dispatch.remove_surface(declaration);
        }
        for declaration in &contribution_delta.declarations_removed {
            dispatch.remove_surface(declaration);
        }
        for callable in &callable_signature_work {
            callable_signatures.remove(callable);
        }
        for callable in &contribution_delta.callable_signatures_removed {
            callable_signatures.remove(callable);
        }
        for field in &field_signature_work {
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
                    match query_field_signature_with_inputs(
                        &mut self.db,
                        field_id,
                        parsed_unit.clone(),
                        &mut self.store,
                        &hierarchy,
                        &resolver,
                        &declarations,
                        Some(input.linked.as_ref()),
                        Some(&type_aliases),
                        Some(&input.import_products),
                    ) {
                        QueryOutcome::Ready(signature) => field_signatures.insert((*signature).clone()),
                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
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
                    if previous_snapshot.is_some() && !callable_signature_work.contains(&callable_id) {
                        continue;
                    }
                    match query_callable_signature_with_inputs(
                        &mut self.db,
                        callable_id.clone(),
                        parsed_unit.clone(),
                        &mut self.store,
                        &hierarchy,
                        &resolver,
                        &declarations,
                        Some(input.linked.as_ref()),
                        Some(&type_aliases),
                        Some(&input.import_products),
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

                let surface = match query_declaration_surface(
                    &mut self.db,
                    DeclarationSurfaceQuery {
                        decl_id: decl_id.clone(),
                        unit: parsed_unit.clone(),
                        linked_interface: linked_interface.clone(),
                        store: &mut self.store,
                        hierarchy: &hierarchy,
                        resolver: &resolver,
                        declarations: &declarations,
                        type_aliases: Some(&type_aliases),
                        linked: Some(input.linked.as_ref()),
                        import_products: Some(&input.import_products),
                    },
                ) {
                    QueryOutcome::Ready(surface) => surface,
                    QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                    QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                    QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
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
        // 6b. Compile and publish enum declarations, associated surfaces, and closed-enum requirements.
        let mut enum_semantics = previous_snapshot
            .as_ref()
            .map_or_else(|| self.base_enum_semantics.clone(), |snapshot| (*snapshot.enum_semantics).clone());
        let mut enum_requirements_table = previous_snapshot
            .as_ref()
            .map_or_else(|| self.base_enum_requirements.clone(), |snapshot| (*snapshot.enum_requirements).clone());
        let mut associated_surfaces_table = previous_snapshot
            .as_ref()
            .map_or_else(|| self.base_associated_surfaces.clone(), |snapshot| (*snapshot.associated_surfaces).clone());
        for module in structural_work_modules.iter().chain(removed_modules.iter()) {
            enum_semantics.remove_module(module);
            enum_requirements_table.remove_module(module);
            associated_surfaces_table.remove_module(module);
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
                let Statement::Enum(enum_def) = stmt else {
                    continue;
                };
                let decl_id = DeclarationId::new(module_id.clone(), enum_def.name.clone().into());
                let Some(enum_product) =
                    crate::checker::enum_declaration::build_enum_semantics(&decl_id, enum_def, &mut self.store, &declarations, &resolver, module_id)
                else {
                    diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                        module_id.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        format!("enum `{}` has no published declaration type", enum_def.name),
                        enum_def.range,
                    ));
                    continue;
                };

                diags_by_module
                    .entry(module_id.clone())
                    .or_default()
                    .extend(enum_product.diagnostics.iter().cloned());

                enum_semantics.insert_enum(enum_product.info.clone());
                for v in enum_product.variants.iter() {
                    enum_semantics.insert_variant(Arc::new(v.clone()));
                }

                let arc_enum_product = Arc::new(enum_product);
                let _ = query_enum_declaration(&mut self.db, arc_enum_product);

                let mut behavior_ctx =
                    crate::checker::CheckingContext::new_with_dispatch_ref(&mut self.store, &hierarchy, &resolver, &declarations, &dispatch, module_id.clone());
                behavior_ctx.attach_enum_semantics(&enum_semantics);
                let behavior_product = crate::checker::enum_behavior::build_enum_behavior(&mut behavior_ctx, &decl_id, enum_def);
                diags_by_module
                    .entry(module_id.clone())
                    .or_default()
                    .extend(behavior_product.diagnostics.iter().cloned());

                // Publish root defaults to callable_signatures and dispatch surface
                let mut surface = dispatch.surface(&decl_id).cloned().unwrap_or_default();
                for default_sig in behavior_product.root_defaults.iter() {
                    callable_signatures.insert(default_sig.clone());
                    let projection = crate::checker::declaration_signature::project_semantic_signature(default_sig);
                    surface.add_callable(default_sig.side, projection);
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
                        store: &mut self.store,
                        hierarchy: &hierarchy,
                        resolver: &resolver,
                        declarations: &declarations,
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

                let behavior_bases: std::collections::HashSet<phalcom_common::selector::SelectorBase> = enum_def
                    .members
                    .iter()
                    .filter_map(|m| match m {
                        phalcom_ast::ast::EnumMember::Behavior(b) => {
                            let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(b);
                            Some(syntax.selector_base())
                        }
                        _ => None,
                    })
                    .collect();

                let (assoc_surface, assoc_diags) = build_associated_surface(
                    &decl_id,
                    Some(
                        &enum_def
                            .members
                            .iter()
                            .filter_map(|m| match m {
                                phalcom_ast::ast::EnumMember::Variant(v) => {
                                    let sel = phalcom_ast::selector::selector_from_variant(v);
                                    Some(crate::identity::VariantId::new(decl_id.clone(), sel))
                                }
                                _ => None,
                            })
                            .collect::<Vec<_>>(),
                    ),
                    &behavior_bases,
                    &std::collections::HashSet::new(),
                    module_id,
                    Some(crate::diagnostic::SemanticSourceSpan::new(module_id.clone(), enum_def.range)),
                );
                diags_by_module.entry(module_id.clone()).or_default().extend(assoc_diags.iter().cloned());
                associated_surfaces_table.insert(decl_id.clone(), assoc_surface.clone());
                let _ = query_associated_surface(&mut self.db, assoc_surface);

                let variants_info: Vec<VariantInfo> = enum_def
                    .members
                    .iter()
                    .filter_map(|m| match m {
                        phalcom_ast::ast::EnumMember::Variant(v) => {
                            let sel = phalcom_ast::selector::selector_from_variant(v);
                            let vid = crate::identity::VariantId::new(decl_id.clone(), sel);
                            enum_semantics.variant_info(&vid).cloned()
                        }
                        _ => None,
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
                    &mut self.store,
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

        for module_id in &structural_work_modules {
            let Some(shard) = self.semantic_structure_shards.get(module_id) else {
                continue;
            };
            for statement in &shard.source.program.statements {
                let declaration = match statement {
                    Statement::Class(class_def) => DeclarationId::new(module_id.clone(), class_def.name.clone().into()),
                    Statement::Enum(enum_def) => DeclarationId::new(module_id.clone(), enum_def.name.clone().into()),
                    _ => continue,
                };
                if !associated_surfaces_table.surfaces.contains_key(&declaration) {
                    let assoc_surface = Arc::new(crate::associated::AssociatedSurface::new(declaration.clone()));
                    associated_surfaces_table.insert(declaration, assoc_surface.clone());
                    let _ = query_associated_surface(&mut self.db, assoc_surface);
                }
            }
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
                let mut ctx = CheckingContext::new_with_dispatch_ref(&mut self.store, &hierarchy, &resolver, &declarations, &dispatch, module_id.clone());
                ctx.attach_field_signatures(&field_signatures);
                ctx.attach_enum_semantics(&enum_semantics);
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
                })
                .map(|(callable, analysis)| (callable.clone(), analysis.clone()))
                .collect()
        });
        for constructors_only in [true, false] {
            for (module_id, shard) in &self.semantic_structure_shards {
                if !semantic_work_modules.contains(module_id) {
                    continue;
                }
                let parsed_unit = &shard.source;
                for stmt in &parsed_unit.program.statements {
                    if let Statement::Class(class_def) = stmt {
                        let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                        if blocked_declarations.contains(&decl_id) {
                            continue;
                        }
                        let type_params_map = if let Some(sig) = declarations.generic_signature(&decl_id) {
                            let mut map = std::collections::HashMap::new();
                            for &param_id in sig.parameters.iter() {
                                let name = self.store.type_parameter(param_id).name.to_string();
                                let binding = type_level_binding_for_parameter(&mut self.store, param_id);
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
                                    import_products: &input.import_products,
                                    hierarchy: &hierarchy,
                                    base_resolver: &resolver,
                                    declarations: &declarations,
                                    type_aliases: &type_aliases,
                                    field_signatures: Some(&field_signatures),
                                    field_lifecycle: Some(if is_constructor { &default_field_lifecycle } else { &field_lifecycle }),
                                    enum_semantics: Some(&enum_semantics),
                                    associated_families: Some(&associated_surfaces_table),
                                };

                                if previous_snapshot.is_some() && !callable_body_work.contains(&callable_id) {
                                    if callable_analyses.contains_key(&callable_id) {
                                        if refresh_cached_body_dependencies(&mut self.db, &query_key, &formal_inputs, &mut self.store).is_ok() {
                                            if self.db.validate_ready(&query_key) {
                                                callable_dispositions.entry(callable_id.clone()).or_insert(CallableRevisionDisposition::Reused);
                                                continue;
                                            }
                                        }
                                    }
                                }

                                if let Err(outcome) = refresh_cached_body_dependencies(&mut self.db, &query_key, &formal_inputs, &mut self.store) {
                                    return Err(outcome);
                                }
                                let previous_computation_revision = self.db.query_state(&query_key).and_then(|state| state.revision());
                                let outcome = query_callable_body_with_formal_inputs(
                                    &mut self.db,
                                    CallableBodyQuery {
                                        callable: callable_id.clone(),
                                        body,
                                        body_range: range,
                                        store: &mut self.store,
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
                                        if !analysis.diagnostics.is_empty() {
                                            diags_by_module
                                                .entry(module_id.clone())
                                                .or_default()
                                                .extend(analysis.diagnostics.iter().cloned());
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
                                                        analysis.callable.declaration_owner() == &decl_id && analysis.callable.side == DispatchSide::Instance
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
                                    QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                                    QueryOutcome::Failed(err) => return Err(QueryOutcome::Failed(err)),
                                }
                            }
                        }
                    } else if let Statement::Enum(enum_def) = stmt {
                        if constructors_only {
                            continue;
                        }
                        let decl_id = DeclarationId::new(module_id.clone(), enum_def.name.clone().into());
                        let type_params_map = if let Some(sig) = declarations.generic_signature(&decl_id) {
                            let mut map = std::collections::HashMap::new();
                            for &param_id in sig.parameters.iter() {
                                let name = self.store.type_parameter(param_id).name.to_string();
                                let binding = type_level_binding_for_parameter(&mut self.store, param_id);
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

                        // Check root bodyful behavior
                        for member in &enum_def.members {
                            if let phalcom_ast::ast::EnumMember::Behavior(b) = member {
                                let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(b);
                                let is_class_side = syntax.attributes().iter().any(|a| a.name == "class")
                                    || match b {
                                        phalcom_ast::ast::EnumBehaviorMember::Method(m) => m.is_static,
                                        phalcom_ast::ast::EnumBehaviorMember::Getter(g) => g.is_static,
                                        phalcom_ast::ast::EnumBehaviorMember::Setter(s) => s.is_static,
                                        phalcom_ast::ast::EnumBehaviorMember::Index(_) => false,
                                    };
                                let side = if is_class_side {
                                    crate::identity::DispatchSide::Class
                                } else {
                                    crate::identity::DispatchSide::Instance
                                };

                                let (selector_opt, body_opt, range_opt) = match b {
                                    phalcom_ast::ast::EnumBehaviorMember::Method(m) => {
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
                                    phalcom_ast::ast::EnumBehaviorMember::Getter(g) => (Selector::getter(&g.name).ok(), g.body.statements(), Some(g.range)),
                                    phalcom_ast::ast::EnumBehaviorMember::Setter(s) => (Selector::setter(&s.name).ok(), s.body.statements(), Some(s.range)),
                                    phalcom_ast::ast::EnumBehaviorMember::Index(i) => {
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
                                        (sel, Some(i.body.as_slice()), Some(i.range))
                                    }
                                };

                                if let (Some(selector), Some(body), Some(range)) = (selector_opt, body_opt, range_opt) {
                                    let callable_id = crate::identity::CallableId::new(decl_id.clone(), selector, side);
                                    let query_key = QueryKey::CallableBody(callable_id.clone());

                                    let formal_inputs = FormalQueryInputs {
                                        sources: &retained_sources,
                                        source_resolution_input,
                                        linked_component_product,
                                        linked: &input.linked,
                                        import_products: &input.import_products,
                                        hierarchy: &hierarchy,
                                        base_resolver: &resolver,
                                        declarations: &declarations,
                                        type_aliases: &type_aliases,
                                        field_signatures: Some(&field_signatures),
                                        field_lifecycle: Some(&field_lifecycle),
                                        enum_semantics: Some(&enum_semantics),
                                        associated_families: Some(&associated_surfaces_table),
                                    };

                                    if previous_snapshot.is_some() && !callable_body_work.contains(&callable_id) {
                                        if callable_analyses.contains_key(&callable_id) {
                                            if refresh_cached_body_dependencies(&mut self.db, &query_key, &formal_inputs, &mut self.store).is_ok() {
                                                if self.db.validate_ready(&query_key) {
                                                    callable_dispositions.entry(callable_id.clone()).or_insert(CallableRevisionDisposition::Reused);
                                                    continue;
                                                }
                                            }
                                        }
                                    }

                                    if let Err(outcome) = refresh_cached_body_dependencies(&mut self.db, &query_key, &formal_inputs, &mut self.store) {
                                        return Err(outcome);
                                    }
                                    let previous_computation_revision = self.db.query_state(&query_key).and_then(|state| state.revision());
                                    let outcome = query_callable_body_with_formal_inputs(
                                        &mut self.db,
                                        CallableBodyQuery {
                                            callable: callable_id.clone(),
                                            body,
                                            body_range: range,
                                            store: &mut self.store,
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
                                            if !analysis.diagnostics.is_empty() {
                                                diags_by_module
                                                    .entry(module_id.clone())
                                                    .or_default()
                                                    .extend(analysis.diagnostics.iter().cloned());
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
                                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                                        QueryOutcome::Failed(err) => {
                                            return Err(QueryOutcome::Failed(err));
                                        }
                                    }
                                }
                            }
                        }

                        // Check case-local bodyful behavior
                        for member in &enum_def.members {
                            if let phalcom_ast::ast::EnumMember::Variant(v) = member {
                                let sel = phalcom_ast::selector::selector_from_variant(v);
                                let variant_id = crate::identity::VariantId::new(decl_id.clone(), sel);
                                if let Some(ref variant_body) = v.body {
                                    for case_member in &variant_body.members {
                                        let (selector_opt, body_opt, range_opt) = match case_member {
                                            phalcom_ast::ast::EnumBehaviorMember::Method(m) => {
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
                                            phalcom_ast::ast::EnumBehaviorMember::Getter(g) => {
                                                (Selector::getter(&g.name).ok(), g.body.statements(), Some(g.range))
                                            }
                                            phalcom_ast::ast::EnumBehaviorMember::Setter(s) => {
                                                (Selector::setter(&s.name).ok(), s.body.statements(), Some(s.range))
                                            }
                                            phalcom_ast::ast::EnumBehaviorMember::Index(i) => {
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
                                                (sel, Some(i.body.as_slice()), Some(i.range))
                                            }
                                        };

                                        if let (Some(selector), Some(body), Some(range)) = (selector_opt, body_opt, range_opt) {
                                            let callable_id = crate::identity::CallableId::case_method(variant_id.clone(), selector);
                                            let query_key = QueryKey::CallableBody(callable_id.clone());

                                            let formal_inputs = FormalQueryInputs {
                                                sources: &retained_sources,
                                                source_resolution_input,
                                                linked_component_product,
                                                linked: &input.linked,
                                                import_products: &input.import_products,
                                                hierarchy: &hierarchy,
                                                base_resolver: &resolver,
                                                declarations: &declarations,
                                                type_aliases: &type_aliases,
                                                field_signatures: Some(&field_signatures),
                                                field_lifecycle: Some(&field_lifecycle),
                                                enum_semantics: Some(&enum_semantics),
                                                associated_families: Some(&associated_surfaces_table),
                                            };

                                            if previous_snapshot.is_some() && !callable_body_work.contains(&callable_id) {
                                                if callable_analyses.contains_key(&callable_id) {
                                                    if refresh_cached_body_dependencies(&mut self.db, &query_key, &formal_inputs, &mut self.store).is_ok() {
                                                        if self.db.validate_ready(&query_key) {
                                                            callable_dispositions.entry(callable_id.clone()).or_insert(CallableRevisionDisposition::Reused);
                                                            continue;
                                                        }
                                                    }
                                                }
                                            }

                                            if let Err(outcome) = refresh_cached_body_dependencies(&mut self.db, &query_key, &formal_inputs, &mut self.store) {
                                                return Err(outcome);
                                            }
                                            let previous_computation_revision = self.db.query_state(&query_key).and_then(|state| state.revision());
                                            let outcome = query_callable_body_with_formal_inputs(
                                                &mut self.db,
                                                CallableBodyQuery {
                                                    callable: callable_id.clone(),
                                                    body,
                                                    body_range: range,
                                                    store: &mut self.store,
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
                                                        state.revision() == Some(self.db.revision())
                                                            && previous_computation_revision != Some(self.db.revision())
                                                    }) {
                                                        callable_dispositions.insert(callable_id.clone(), CallableRevisionDisposition::Recomputed);
                                                    } else {
                                                        callable_dispositions.entry(callable_id.clone()).or_insert(CallableRevisionDisposition::Reused);
                                                    }
                                                    if !analysis.diagnostics.is_empty() {
                                                        diags_by_module
                                                            .entry(module_id.clone())
                                                            .or_default()
                                                            .extend(analysis.diagnostics.iter().cloned());
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
                                                QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                                                QueryOutcome::Failed(err) => return Err(QueryOutcome::Failed(err)),
                                            }
                                        }
                                    }
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
                    if current_modules.contains(&callable.module())
                        && callable_dispositions.get(&callable) != Some(&CallableRevisionDisposition::Recomputed) =>
                {
                    Some(callable)
                }
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        semantic_work_modules.extend(downstream_body_work.iter().map(|callable| callable.module().clone()));
        for callable in downstream_body_work {
            let Some(unit) = retained_sources.get(&callable.module()).cloned() else {
                continue;
            };
            let is_constructor = callable_signatures.get_for_body(&callable).is_some_and(|signature| signature.is_constructor());
            let formal_inputs = FormalQueryInputs {
                sources: &retained_sources,
                source_resolution_input,
                linked_component_product,
                linked: &input.linked,
                import_products: &input.import_products,
                hierarchy: &hierarchy,
                base_resolver: &resolver,
                declarations: &declarations,
                type_aliases: &type_aliases,
                field_signatures: Some(&field_signatures),
                field_lifecycle: Some(if is_constructor { &default_field_lifecycle } else { &field_lifecycle }),
                enum_semantics: Some(&enum_semantics),
                associated_families: Some(&associated_surfaces_table),
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
                if let Err(outcome) = refresh_cached_body_dependencies(&mut self.db, &QueryKey::CallableBody(callable.clone()), &formal_inputs, &mut self.store)
                {
                    return Err(outcome);
                }
                if self.db.validate_ready(&QueryKey::CallableBody(callable.clone())) {
                    continue;
                }
            }
            if let Err(outcome) = revalidate_downstream_callable_body(
                &mut self.db,
                &callable,
                &unit,
                &formal_inputs,
                &mut self.store,
                &hierarchy,
                &mut dispatch,
                &mut callable_signatures,
                &mut callable_analyses,
                &mut callable_dispositions,
                &mut diags_by_module,
                budget,
                cancel,
            ) {
                return Err(outcome);
            }
        }

        // 7. Refine Callable Return Interfaces and Reach Fixed Point
        //
        // This is a specialized session-level loop: body inference and
        // interface publication iterate until return contracts stabilize.
        // During each iteration, we only run inference and update dispatch; we
        // do not run full checking or re-emit diagnostics.
        if let Err(outcome) = refresh_inferred_callable_results(InferredCallableRefreshInputs {
            db: &mut self.db,
            sources: &retained_sources,
            store: &mut self.store,
            hierarchy: &hierarchy,
            resolver: &resolver,
            declarations: &declarations,
            dispatch: &mut dispatch,
            callable_signatures: &mut callable_signatures,
            callable_analyses: &mut callable_analyses,
            previous_callable_analyses: previous_snapshot.as_ref().map(|snapshot| snapshot.callable_analyses.as_ref()),
            field_signatures: &field_signatures,
            field_lifecycle: &field_lifecycle,
            enum_semantics: &enum_semantics,
            associated_surfaces: &associated_surfaces_table,
            callable_dispositions: &mut callable_dispositions,
            diagnostics: &mut diags_by_module,
            semantic_work_modules: &semantic_work_modules,
            previous_callable_signatures: previous_snapshot.as_ref().map(|snapshot| snapshot.callable_signatures.as_ref()),
            budget,
            cancel,
        }) {
            return Err(outcome);
        }

        for (module_id, shard) in &self.semantic_structure_shards {
            if !semantic_work_modules.contains(module_id) {
                continue;
            }
            let parsed_unit = &shard.source;
            let mut ctx = CheckingContext::new_with_dispatch_ref(&mut self.store, &hierarchy, &resolver, &declarations, &dispatch, module_id.clone());
            ctx.attach_field_signatures(&field_signatures);
            ctx.attach_field_lifecycle(&field_lifecycle);
            ctx.attach_enum_semantics(&enum_semantics);
            ctx.attach_associated_families(&associated_surfaces_table);

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

            if !ctx.diagnostics.is_empty() {
                diags_by_module.entry(module_id.clone()).or_default().extend(ctx.diagnostics.clone());
            }

            let body_range = parsed_unit.program.preamble.range;
            let module_decl = DeclarationId::new(module_id.clone(), "<main>".into());
            let callable_id = CallableId::new(module_decl, Selector::getter("<main>").unwrap(), DispatchSide::Instance);
            let analysis = ctx.finalize(callable_id.clone(), body_range, crate::checker::CallableAnalysisStatus::Complete);
            callable_analyses.insert(callable_id, Arc::new(analysis));
        }

        // 8. Freeze and Publish Immutable Snapshot
        let mut diagnostics_map: BTreeMap<ModuleId, Arc<[SemanticDiagnostic]>> = BTreeMap::new();
        for (module_id, diags) in diags_by_module {
            if !diags.is_empty() {
                diagnostics_map.insert(module_id, Arc::from(diags.into_boxed_slice()));
            }
        }
        let mut semantic_diagnostic_contributions = BTreeMap::new();
        for (module_id, diagnostics) in &diagnostics_map {
            let mut semantic_diagnostics = diagnostics.to_vec();
            if let Some(module_diagnostics) = current_module_diagnostics.get(module_id) {
                for module_diagnostic in semantic_diagnostics_from_module_diagnostics(module_diagnostics) {
                    if let Some(index) = semantic_diagnostics.iter().position(|diagnostic| diagnostic == &module_diagnostic) {
                        semantic_diagnostics.remove(index);
                    }
                }
            }
            if !semantic_diagnostics.is_empty() {
                semantic_diagnostic_contributions.insert(module_id.clone(), Arc::from(semantic_diagnostics.into_boxed_slice()));
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

        let topology = input.topology.unwrap_or_else(|| {
            Arc::new(phalcom_modules::topology::ModuleTopology::from_parts(
                phalcom_modules::stabilization::ResolverGeneration(input.generation),
                &input.linked.universe,
                &unlinked_map,
                &sources_loc_map,
            ))
        });
        let reverse_imports = input.reverse_imports.unwrap_or_else(|| {
            let mut rev: BTreeMap<ModuleId, BTreeSet<ModuleId>> = BTreeMap::new();
            for ((importer, _), target) in &resolved_imports_map {
                rev.entry(target.clone()).or_default().insert(importer.clone());
            }
            Arc::new(rev)
        });

        let module_products = Arc::new(crate::snapshot::ModuleQueryProducts::new(
            input.linked.universe.clone(),
            Arc::new(unlinked_map),
            Arc::new(linked_map),
            Arc::new(resolved_imports_map.clone()),
            Arc::new(sources_loc_map),
            topology,
            reverse_imports,
        ));

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
            source_index_analysis_callables.insert(module.clone(), callables);
        }
        let (mut source_index, presentation_sources) = build_source_semantic_index(
            &input.sources,
            &callable_analyses,
            &input.import_products,
            input.require_canonical_import_products,
            input.linked.as_ref(),
            &resolver,
            &known_declarations,
            previous_snapshot.as_deref().map(|snapshot| snapshot.source_index.as_ref()),
            &source_index_rebuild_modules,
            &removed_modules,
            &current_modules,
            &source_index_analysis_callables,
            &input.import_sites_by_module,
        );
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
        for declaration in &blocked_declarations {
            declarations.remove(declaration);
        }
        callable_analyses
            .retain(|callable, _| callable.declaration_owner().name.as_ref() == "<main>" || current_declarations.contains(callable.declaration_owner()));
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

        let mut snapshot_obj = SemanticSnapshot::new_with_callable_analyses(
            self.workspace,
            self.db.revision(),
            input.generation,
            Arc::new(self.store.clone()),
            Arc::new(retained_sources.clone()),
            Arc::new(dispatch.surfaces().clone()),
            Arc::new(dispatch),
            Arc::new(callable_signatures),
            Arc::new(declarations),
            Arc::new(hierarchy),
            Arc::new(diagnostics_map),
            Arc::new(semantic_graph),
            Arc::new(callable_analyses),
        );

        snapshot_obj = snapshot_obj.with_field_signatures(Arc::new(field_signatures));
        snapshot_obj = snapshot_obj.with_presentation_sources(Arc::new(presentation_sources));
        snapshot_obj = snapshot_obj.with_source_index(Arc::new(source_index));
        snapshot_obj = snapshot_obj.with_formal_projection(Arc::new(formal_projection));
        snapshot_obj = snapshot_obj.with_enum_semantics(Arc::new(enum_semantics));
        snapshot_obj = snapshot_obj.with_enum_requirements(Arc::new(enum_requirements_table));
        snapshot_obj = snapshot_obj.with_associated_surfaces(Arc::new(associated_surfaces_table));
        snapshot_obj = snapshot_obj.with_type_aliases(Arc::new(type_aliases));
        snapshot_obj = snapshot_obj.with_semantic_structure_shards(Arc::new(self.semantic_structure_shards.clone()));
        snapshot_obj.advisory = Arc::new(advisory);
        snapshot_obj.module_products = module_products;
        if !input.blocked_modules.is_empty() {
            snapshot_obj.status = crate::snapshot::SnapshotStatus::Partial {
                blocked_modules: input.blocked_modules.len() as u32,
            };
        }
        let snapshot = Arc::new(snapshot_obj);

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
                QueryKey::CallableSignature(callable) if snapshot.sources.contains_key(&callable.module()) => {
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
                QueryKey::CallableBody(callable) if snapshot.sources.contains_key(&callable.module()) => {
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
        for module in diagnostic_work_modules {
            let before = previous_diagnostics.and_then(|diagnostics| diagnostics.get(&module));
            let after = snapshot.diagnostics.get(&module);
            if before != after {
                diagnostics_changed.insert(module);
            }
        }

        let module_graph_changed = previous_snapshot.as_deref().is_none_or(|previous| {
            previous.semantic_graph != snapshot.semantic_graph
                || previous.module_products.resolved_imports != snapshot.module_products.resolved_imports
                || !previous.module_products.linked.keys().eq(snapshot.module_products.linked.keys())
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
            advisory_changed: changed_modules.clone(),
            declaration_index_changed,
            module_graph_changed,
        };

        // These counters describe compiler-owned product work, not protocol
        // requests. A source mutation refreshes the affected source shard and
        // its advisory/callable products; graph/declaration work is tracked
        // separately so callers can distinguish a body edit from a project
        // lifecycle event.
        stats.source_indexes_recomputed = changed_modules.len();
        stats.advisory_sources_recomputed = changed_modules.len();
        stats.modules_relinked = if module_graph_changed { changed_modules.len() } else { 0 };
        stats.project_graph_rebuilt = effects.module_graph_changed;
        if previous_snapshot.is_some() {
            for callable in snapshot.callable_analyses.keys() {
                if callable.declaration_owner().name.as_ref() != "<main>" {
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
        stats.advisory_callables_recomputed = stats.callables_recomputed;
        let recomputed_keys = callable_dispositions
            .iter()
            .filter(|(_, disposition)| **disposition == CallableRevisionDisposition::Recomputed)
            .map(|(callable, _)| QueryKey::CallableBody(callable.clone()))
            .collect::<Vec<_>>();

        self.last_snapshot = Some(snapshot.clone());
        self.last_known_good = Some(snapshot.clone());
        self.field_lifecycle_fingerprints = next_field_lifecycle_fingerprints;
        self.generic_header_dependencies = next_generic_header_dependencies;
        self.alias_sources = alias_sources;
        self.alias_dependencies = alias_dependencies;
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

fn is_canonical_erased_iterable_supertype(unit: &ParsedModuleUnit, store: &TypeStore, ty: TypeId) -> bool {
    if unit.id.project != phalcom_modules::ProjectIdentity::Universe {
        return false;
    }
    let crate::types::store::TypeData::Nominal { declaration } = store.get(ty) else {
        return false;
    };
    if declaration != &crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Iterable) {
        return false;
    }
    phalcom_modules::UniverseSourceProvider::new()
        .load_parsed(&unit.id)
        .is_ok_and(|canonical| canonical.text == unit.text)
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
                collect_alias_dependencies(argument, module, resolver, aliases, dependencies);
            }
        }
        TypeAnnotationExpr::Union { members, .. } => {
            for member in members {
                collect_alias_dependencies(member, module, resolver, aliases, dependencies);
            }
        }
        TypeAnnotationExpr::Tuple { elements, .. } => {
            for element in elements {
                collect_alias_dependencies(&element.ty, module, resolver, aliases, dependencies);
            }
        }
        TypeAnnotationExpr::Callable { parameters, result, .. } => {
            for parameter in parameters {
                collect_alias_dependencies(&parameter.ty, module, resolver, aliases, dependencies);
            }
            collect_alias_dependencies(result, module, resolver, aliases, dependencies);
        }
        TypeAnnotationExpr::Record { fields, .. } => {
            for field in fields {
                collect_alias_dependencies(&field.ty, module, resolver, aliases, dependencies);
            }
        }
        TypeAnnotationExpr::TypeLambda { body, .. } => collect_alias_dependencies(body, module, resolver, aliases, dependencies),
        TypeAnnotationExpr::Unit { .. }
        | TypeAnnotationExpr::Dynamic { .. }
        | TypeAnnotationExpr::Never { .. }
        | TypeAnnotationExpr::SelfType { .. }
        | TypeAnnotationExpr::Invalid { .. } => {}
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
                collect_type_annotation_declarations(argument, module, resolver, dependencies);
            }
        }
        TypeAnnotationExpr::Union { members, .. } => {
            for member in members {
                collect_type_annotation_declarations(member, module, resolver, dependencies);
            }
        }
        TypeAnnotationExpr::Tuple { elements, .. } => {
            for element in elements {
                collect_type_annotation_declarations(&element.ty, module, resolver, dependencies);
            }
        }
        TypeAnnotationExpr::Callable { parameters, result, .. } => {
            for parameter in parameters {
                collect_type_annotation_declarations(&parameter.ty, module, resolver, dependencies);
            }
            collect_type_annotation_declarations(result, module, resolver, dependencies);
        }
        TypeAnnotationExpr::Record { fields, .. } => {
            for field in fields {
                collect_type_annotation_declarations(&field.ty, module, resolver, dependencies);
            }
        }
        TypeAnnotationExpr::TypeLambda { body, .. } => collect_type_annotation_declarations(body, module, resolver, dependencies),
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

fn compute_module_fingerprint(unit: &ParsedModuleUnit) -> u64 {
    let mut hasher = DefaultHasher::new();
    unit.id.hash(&mut hasher);
    unit.text.hash(&mut hasher);
    hasher.finish()
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
        | QueryKey::HierarchyEdge(declaration)
        | QueryKey::EnumDeclaration(declaration)
        | QueryKey::EnumRequirements(declaration)
        | QueryKey::AssociatedSurface(declaration) => Some(&declaration.module),
        QueryKey::ResolvedImport(site) => Some(&site.importer),
        QueryKey::FieldSignature(field) => Some(&field.owner.module),
        QueryKey::CallableSignature(callable)
        | QueryKey::CallableBody(callable)
        | QueryKey::CallableEffects(callable)
        | QueryKey::CallableControl(callable)
        | QueryKey::CallableTermination(callable)
        | QueryKey::CallableContracts(callable)
        | QueryKey::VerificationConditions(callable)
        | QueryKey::SourceFormalAttachment(callable)
        | QueryKey::AdvisoryCallable(callable) => Some(callable.module()),
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

fn build_source_semantic_index(
    sources: &BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    callable_analyses: &HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>,
    import_products: &BTreeMap<phalcom_modules::identity::ImportSiteId, Arc<phalcom_modules::resolver::ImportResolutionProduct>>,
    require_canonical_import_products: bool,
    linked: &LinkedProgram,
    type_resolver: &dyn TypeResolver,
    nominal_declarations: &HashSet<DeclarationId>,
    previous: Option<&SourceSemanticIndex>,
    rebuild_modules: &BTreeSet<ModuleId>,
    retired_modules: &BTreeSet<ModuleId>,
    current_modules: &BTreeSet<ModuleId>,
    analysis_callables: &BTreeMap<ModuleId, BTreeSet<crate::identity::CallableId>>,
    import_sites_by_module: &BTreeMap<ModuleId, BTreeSet<phalcom_modules::identity::ImportSiteId>>,
) -> (SourceSemanticIndex, BTreeMap<ModuleId, Arc<str>>) {
    // Canonical Universe modules are source-owned presentation inputs: index
    // their declarations for navigation without linking or deeply analyzing
    // their bodies as part of an ordinary workspace update. On an incremental
    // update, retain unchanged shards and materialize only the requested
    // rebuild set; cloning/scanning the complete source map here would defeat
    // the persistent source-index contract.
    let mut presentation_sources = BTreeMap::new();
    let provider = phalcom_modules::UniverseSourceProvider::new();
    let presentation_modules = provider
        .nodes()
        .into_iter()
        .map(|node| {
            let path = phalcom_modules::ModulePath::from_components(
                node.path
                    .iter()
                    .map(|component| phalcom_modules::ModuleComponent::from_identifier(component).expect("canonical Universe component"))
                    .collect::<Vec<_>>(),
            );
            ModuleId::universe(path)
        })
        .collect::<BTreeSet<_>>();
    let mut index_sources = BTreeMap::new();
    if previous.is_none() {
        index_sources.extend(sources.iter().map(|(module, source)| (module.clone(), source.clone())));
    } else {
        for module in rebuild_modules {
            if let Some(source) = sources.get(module) {
                index_sources.insert(module.clone(), source.clone());
            }
        }
    }
    for module in &presentation_modules {
        if sources.contains_key(module) || (previous.is_some() && !rebuild_modules.contains(module)) {
            continue;
        }
        let parsed = provider.load_parsed(module).expect("canonical Universe presentation source must load");
        presentation_sources.insert(module.clone(), parsed.text.clone());
        index_sources.insert(module.clone(), parsed);
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
        if previous.is_some() && !rebuild_modules.contains(module) {
            continue;
        }
        for (range, declaration) in resolve_type_reference_targets(module, &source.program, type_resolver) {
            context.type_reference_targets.insert((module.clone(), range), declaration);
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
                    context
                        .callable_targets
                        .insert((callable.id.declaration_owner().clone(), callable.id.selector.clone()), callable.id.clone());
                }
            }
        }
    }
    for structure in scopes.values() {
        for callable in structure.callable_sources.values() {
            context
                .callable_targets
                .insert((callable.id.declaration_owner().clone(), callable.id.selector.clone()), callable.id.clone());
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
                    let alternate = crate::identity::CallableId::new(callable.declaration_owner().clone(), callable.selector.clone(), alternate_side);
                    callable_analyses.get(&alternate)
                })
            })
            .map(Arc::as_ref)
            .filter(|analysis| !matches!(&analysis.callable.selector.base, phalcom_common::selector::SelectorBase::Named(name) if name == "<main>"))
            .collect::<Vec<_>>();
        module_analyses.sort_by_key(|analysis| analysis.callable.clone());
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
                .filter(|analysis| analysis.callable.declaration_owner().name.as_ref() != "<main>")
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
                    current_owner: Some(analysis.callable.declaration_owner()),
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
                index.body.as_slice(),
                index.range,
            ))
        }
        ClassMember::Field(_) | ClassMember::Variant(_) => None,
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
        | SemanticTargetId::VariantField(_) => AdvisoryOrigin::Constraint(site.clone()),
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
fn member_selector(member: &ClassMember) -> Option<Selector> {
    match member {
        ClassMember::Method(method) => {
            let slots = method
                .params
                .iter()
                .filter(|parameter| parameter.rest_mode == phalcom_ast::ast::RestMode::None)
                .map(|parameter| match &parameter.label {
                    Some(label) if label != "_" => phalcom_common::selector::SelectorSlot::Label(label.clone()),
                    _ => phalcom_common::selector::SelectorSlot::Positional,
                })
                .collect::<Vec<_>>();
            Selector::method(&method.name, slots).ok()
        }
        ClassMember::Getter(getter) => Selector::getter(&getter.name).ok(),
        ClassMember::Setter(setter) => Selector::setter(&setter.name).ok(),
        _ => None,
    }
}

fn enum_behavior_selector(behavior: &phalcom_ast::ast::EnumBehaviorMember) -> Option<Selector> {
    match behavior {
        phalcom_ast::ast::EnumBehaviorMember::Method(method) => {
            let slots = method
                .params
                .iter()
                .filter(|parameter| parameter.rest_mode == phalcom_ast::ast::RestMode::None)
                .map(|parameter| match &parameter.label {
                    Some(label) if label != "_" => phalcom_common::selector::SelectorSlot::Label(label.clone()),
                    _ => phalcom_common::selector::SelectorSlot::Positional,
                })
                .collect::<Vec<_>>();
            Selector::method(&method.name, slots).ok()
        }
        phalcom_ast::ast::EnumBehaviorMember::Getter(getter) => Selector::getter(&getter.name).ok(),
        phalcom_ast::ast::EnumBehaviorMember::Setter(setter) => Selector::setter(&setter.name).ok(),
        phalcom_ast::ast::EnumBehaviorMember::Index(index) => {
            let slots = index
                .params
                .iter()
                .map(|parameter| match &parameter.label {
                    Some(label) if label != "_" => phalcom_common::selector::SelectorSlot::Label(label.clone()),
                    _ => phalcom_common::selector::SelectorSlot::Positional,
                })
                .collect::<Vec<_>>();
            match index.accessor {
                phalcom_ast::ast::IndexAccessor::Get => Selector::subscript_get(slots).ok(),
                phalcom_ast::ast::IndexAccessor::Set { .. } => Selector::subscript_set(slots).ok(),
            }
        }
    }
}

fn source_body_for_callable<'a>(callable: &CallableId, unit: &'a ParsedModuleUnit) -> Option<(&'a [Statement], SourceRange)> {
    let owner = callable.declaration_owner();
    for statement in &unit.program.statements {
        match statement {
            Statement::Class(class_def) if callable.owner == crate::identity::CallableOwnerId::Declaration(owner.clone()) => {
                let declaration = DeclarationId::new(unit.id.clone(), class_def.name.clone().into());
                for member in &class_def.members {
                    let Some(selector) = member_selector(member) else {
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
                        _ => None,
                    };
                }
            }
            Statement::Enum(enum_def) => {
                let declaration = DeclarationId::new(unit.id.clone(), enum_def.name.clone().into());
                if callable.owner == crate::identity::CallableOwnerId::Declaration(owner.clone()) {
                    for member in &enum_def.members {
                        let phalcom_ast::ast::EnumMember::Behavior(behavior) = member else {
                            continue;
                        };
                        let Some(selector) = enum_behavior_selector(behavior) else {
                            continue;
                        };
                        let side = if behavior.attributes().iter().any(|attribute| attribute.name == "class")
                            || match behavior {
                                phalcom_ast::ast::EnumBehaviorMember::Method(method) => method.is_static,
                                phalcom_ast::ast::EnumBehaviorMember::Getter(getter) => getter.is_static,
                                phalcom_ast::ast::EnumBehaviorMember::Setter(setter) => setter.is_static,
                                phalcom_ast::ast::EnumBehaviorMember::Index(_) => false,
                            } {
                            DispatchSide::Class
                        } else {
                            DispatchSide::Instance
                        };
                        if CallableId::new(declaration.clone(), selector, side) != *callable {
                            continue;
                        }
                        return match behavior {
                            phalcom_ast::ast::EnumBehaviorMember::Method(method) => Some((method.body.statements()?, method.range)),
                            phalcom_ast::ast::EnumBehaviorMember::Getter(getter) => Some((getter.body.statements()?, getter.range)),
                            phalcom_ast::ast::EnumBehaviorMember::Setter(setter) => Some((setter.body.statements()?, setter.range)),
                            phalcom_ast::ast::EnumBehaviorMember::Index(index) => Some((index.body.as_slice(), index.range)),
                        };
                    }
                }
                let crate::identity::CallableOwnerId::Variant(variant_id) = &callable.owner else {
                    continue;
                };
                for member in &enum_def.members {
                    let phalcom_ast::ast::EnumMember::Variant(variant) = member else {
                        continue;
                    };
                    let candidate_variant = crate::identity::VariantId::new(declaration.clone(), phalcom_ast::selector::selector_from_variant(variant));
                    if &candidate_variant != variant_id {
                        continue;
                    }
                    let Some(variant_body) = &variant.body else {
                        continue;
                    };
                    for case_member in &variant_body.members {
                        let Some(selector) = enum_behavior_selector(case_member) else {
                            continue;
                        };
                        if CallableId::case_method(candidate_variant.clone(), selector) != *callable {
                            continue;
                        }
                        return match case_member {
                            phalcom_ast::ast::EnumBehaviorMember::Method(method) => Some((method.body.statements()?, method.range)),
                            phalcom_ast::ast::EnumBehaviorMember::Getter(getter) => Some((getter.body.statements()?, getter.range)),
                            phalcom_ast::ast::EnumBehaviorMember::Setter(setter) => Some((setter.body.statements()?, setter.range)),
                            phalcom_ast::ast::EnumBehaviorMember::Index(index) => Some((index.body.as_slice(), index.range)),
                        };
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn revalidate_downstream_callable_body(
    db: &mut SemanticDb,
    callable: &CallableId,
    unit: &ParsedModuleUnit,
    formal_inputs: &FormalQueryInputs<'_>,
    store: &mut TypeStore,
    hierarchy: &MapTypeHierarchy,
    dispatch: &mut SurfaceDispatchResolver,
    callable_signatures: &mut CallableSignatureTable,
    callable_analyses: &mut HashMap<CallableId, Arc<crate::checker::CallableAnalysis>>,
    callable_dispositions: &mut BTreeMap<CallableId, CallableRevisionDisposition>,
    diagnostics: &mut BTreeMap<ModuleId, Vec<SemanticDiagnostic>>,
    budget: QueryBudget,
    cancel: &CancellationToken,
) -> Result<(), QueryOutcome<()>> {
    let Some((body, body_range)) = source_body_for_callable(callable, unit) else {
        return Ok(());
    };
    let mut type_parameters = HashMap::new();
    if let Some(signature) = formal_inputs.declarations.generic_signature(callable.declaration_owner()) {
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
    refresh_cached_body_dependencies(db, &key, formal_inputs, store)?;
    let previous_computation_revision = db.query_state(&key).and_then(|state| state.revision());
    let outcome = query_callable_body_with_formal_inputs(
        db,
        CallableBodyQuery {
            callable: callable.clone(),
            body,
            body_range,
            store,
            hierarchy,
            resolver: &scoped_resolver,
            declarations: formal_inputs.declarations,
            dispatch,
            module: callable.module().clone(),
            budget,
            cancel,
            formal_inputs: Some(formal_inputs),
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
            if !analysis.diagnostics.is_empty() {
                diagnostics
                    .entry(callable.module().clone())
                    .or_default()
                    .extend(analysis.diagnostics.iter().cloned());
            }
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
    callable_analyses: &'a mut HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>,
    previous_callable_analyses: Option<&'a HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>>,
    field_signatures: &'a FieldSignatureTable,
    field_lifecycle: &'a crate::checker::field_lifecycle::FieldLifecycleTable,
    enum_semantics: &'a EnumSemanticTable,
    associated_surfaces: &'a AssociatedFamilyTable,
    callable_dispositions: &'a mut BTreeMap<CallableId, CallableRevisionDisposition>,
    diagnostics: &'a mut BTreeMap<ModuleId, Vec<SemanticDiagnostic>>,
    semantic_work_modules: &'a BTreeSet<ModuleId>,
    previous_callable_signatures: Option<&'a CallableSignatureTable>,
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
        callable_analyses,
        previous_callable_analyses,
        field_signatures,
        field_lifecycle,
        enum_semantics,
        associated_surfaces,
        callable_dispositions,
        diagnostics,
        semantic_work_modules,
        previous_callable_signatures,
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
            let Some(signature) = callable_signatures.get_for_body(callable) else {
                continue;
            };
            let signature_id = signature.callable.clone();

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
                            module: module_id.clone(),
                        },
                        crate::checker::body::CallableBodyRequest {
                            callable: callable.clone(),
                            body,
                            body_range: range,
                            declared_signature,
                            budget,
                            cancel,
                            field_signatures: Some(field_signatures),
                            field_lifecycle: Some(field_lifecycle),
                            enum_semantics: Some(enum_semantics),
                            associated_families: Some(associated_surfaces),
                        },
                    );
                    analysis.dependency_fingerprint = crate::db::fingerprint::callable_body_product_fingerprint(&analysis);
                    if !analysis.diagnostics.is_empty() {
                        let module_diagnostics = diagnostics.entry(module_id.clone()).or_default();
                        for diagnostic in analysis.diagnostics.iter() {
                            if !module_diagnostics.contains(diagnostic) {
                                module_diagnostics.push(diagnostic.clone());
                            }
                        }
                    }
                    let stable_previous = previous_callable_analyses
                        .and_then(|analyses| analyses.get(&callable))
                        .or_else(|| callable_analyses.get(&callable));
                    let replacement = match stable_previous {
                        Some(previous) if previous.dependency_fingerprint == analysis.dependency_fingerprint => previous.clone(),
                        _ => Arc::new(analysis),
                    };
                    callable_analyses.insert(callable.clone(), replacement.clone());
                    db.update_callable_body_product(&callable, replacement);
                    callable_dispositions.insert(callable.clone(), CallableRevisionDisposition::Recomputed);
                    recomputed_in_this_iteration.insert(callable);
                }
            }
        }

        recomputed_in_prev_iteration = recomputed_in_this_iteration;
    }

    Ok(())
}
