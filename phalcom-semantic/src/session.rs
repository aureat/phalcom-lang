//! Compiler-owned incremental workspace session (Spec 04.5 / Wave 5 / Tasks 16-18).

use crate::advisory::{
    AdvisoryBuiltins, AdvisoryCallableSummary, AdvisoryConfidence, AdvisoryFact, AdvisoryFlowContext, AdvisoryModuleProduct, AdvisoryOrigin,
    AdvisoryProductStatus, AdvisorySolver, AdvisorySolverBudget, AdvisorySolverNode, AdvisoryTargetResolution, AdvisoryWorkspace, advisory_fact_from_formal,
    advisory_shape_from_formal, advisory_shape_from_formal_for_receiver, analyze_expr, analyze_statements,
};
use crate::checker::analysis::normal_return_summary;
use crate::checker::context::CheckingContext;
use crate::checker::declaration::check_class_field_initializers;
use crate::checker::statement::check_statement;
use crate::core_surface::render_canonical_core_source;
use crate::db::SemanticDb;
use crate::db::budget::{CancellationToken, QueryBudget};
use crate::db::key::QueryKey;
use crate::db::query::{
    FormalQueryInputs, bootstrap_advisory_callable, query_advisory_callable, query_advisory_module, query_callable_body_with_formal_inputs,
    query_callable_signature, query_declaration_shell, query_declaration_surface, query_hierarchy_edge, query_linked_interface, query_source_formal_attachment,
    query_source_structure, query_unlinked_interface,
};
use crate::db::state::QueryOutcome;
use crate::declarations::{DeclarationTypeInfo, DeclarationTypeTable, GenericSupertypeTemplate, bootstrap_universe_declarations};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::SurfaceDispatchResolver;
use crate::identity::{CallableId, DeclarationId, DispatchSide, FieldId, ModuleId, SemanticTargetId, SourceOwner, SourceSiteId, WorkspaceId};
use crate::resolver::LinkedTypeResolver;
use crate::signature::{CallableSignatureTable, FieldSignatureTable};
use crate::snapshot::SemanticSnapshot;
use crate::source::ParsedModuleUnit;
use crate::source_index::{SourceIndexContext, SourceSemanticIndex, build_source_scope_index, resolve_type_reference_targets};
use crate::types::annotation::{TypeResolver, resolve_generic_signature, resolve_kind_syntax};
use crate::types::id::KindId;
use crate::types::native::register_native_surfaces;
use crate::types::parameter::TypeParameterOwner;
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::TypeStore;
use crate::workspace::SemanticWorkspaceInput;
use phalcom_ast::ast::{ClassMember, DependencyDecl, ImportDecl, PackItem, PackLabel, Statement};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::declaration::{DeclarationBlueprint, DeclarationKind, DeclarationRealizationError, DeclarationShellTable};
use phalcom_modules::graph::{SemanticEdge, SemanticEdgeKind, SemanticNodeId};
use phalcom_modules::interface::{InterfaceBuilder, LinkedExportTarget};
use phalcom_modules::linker::LinkedProgram;
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CallableRevisionDisposition {
    Reused,
    Recomputed,
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
    pub effects: SemanticPublicationEffects,
}

/// Compatibility name for existing compiler tests and lower-level callers.
pub type SemanticWorkspaceUpdate = SemanticWorkspacePublication;

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
    sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    source_fingerprints: BTreeMap<ModuleId, u64>,
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

        let base_declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(ModuleId::core(), key.name().into()));

        let mut base_hierarchy = MapTypeHierarchy::new();
        for relation in phalcom_native_meta::UNIVERSE_CLASS_RELATIONS {
            if let Some(superclass) = relation.superclass {
                base_hierarchy.insert(
                    DeclarationId::new(ModuleId::core(), relation.class.name().into()),
                    DeclarationId::new(ModuleId::core(), superclass.name().into()),
                );
            }
        }

        let mut base_dispatch = SurfaceDispatchResolver::new();
        let known_declarations: HashSet<DeclarationId> = base_declarations.iter().map(|(decl_id, _)| decl_id.clone()).collect();
        let dummy_linked = Arc::new(LinkedProgram {
            universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
            modules: BTreeMap::new(),
            graphs: phalcom_modules::graph::ModuleGraphs::default(),
            entry: ModuleId::core(),
            initialization_order: vec![ModuleId::core()],
        });
        let resolver = LinkedTypeResolver::new(dummy_linked, known_declarations, ModuleId::core());
        let native_report = register_native_surfaces(&mut store, &base_declarations, &resolver, &ModuleId::core(), &mut base_dispatch)
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

        Self {
            workspace,
            module_session: WorkspaceModuleSession::new(),
            db,
            store,
            base_declarations,
            base_hierarchy,
            base_dispatch,
            base_callable_signatures,
            sources: BTreeMap::new(),
            source_fingerprints: BTreeMap::new(),
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
        self.update(SemanticWorkspaceInput {
            linked: update.linked,
            sources: update.sources,
            generation,
        })
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

    /// Performs an incremental semantic analysis update on this session.
    pub fn update(&mut self, input: SemanticWorkspaceInput) -> SemanticWorkspaceUpdate {
        let generation = input.generation;
        let sources = input.sources.clone();
        let linked = input.linked.clone();
        self.update_with_budget_and_cancel(input, QueryBudget::default(), &CancellationToken::new())
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
        if cancel.is_cancelled() {
            return Err(QueryOutcome::Cancelled);
        }

        self.db.begin_revision();
        let mut stats = SemanticUpdateStats::default();
        let mut invalidated_keys = BTreeSet::new();
        let mut callable_dispositions = BTreeMap::new();
        let previous_sources = self.sources.clone();
        let previous_snapshot = self.last_snapshot.clone();

        // 1. Refresh source-owned staged products without eager reverse invalidation.
        //
        // A source edit changes ParsedModule input identity. Query-local recomputation
        // preserves downstream cached products, and their dependency fingerprints decide
        // lazily whether semantic propagation stops or continues. UnlinkedInterface is
        // evaluated for every source so an unchanged unlinked semantic product can become
        // current and allow linked/formal/body products to remain reusable.
        let mut new_fingerprints = BTreeMap::new();
        let mut changed_modules = BTreeSet::new();
        for (module_id, unit) in &input.sources {
            let fp = compute_module_fingerprint(unit);
            new_fingerprints.insert(module_id.clone(), fp);

            let existed = self.source_fingerprints.contains_key(module_id);
            let changed = self.source_fingerprints.get(module_id).copied() != Some(fp);

            if changed {
                changed_modules.insert(module_id.clone());
                stats.modules_recomputed += 1;
                if existed {
                    invalidated_keys.insert(QueryKey::ParsedModule(module_id.clone()));
                }
            }

            match query_unlinked_interface(&mut self.db, module_id.clone(), unit.clone()) {
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
                if let Some(snapshot) = previous_snapshot.as_ref() {
                    seeds.extend(
                        snapshot
                            .callable_analyses
                            .keys()
                            .filter(|callable| callable.owner.module == *old_module_id)
                            .cloned()
                            .map(QueryKey::AdvisoryCallable),
                    );
                }
                let closure = self.db.invalidate(seeds);
                invalidated_keys.extend(closure);
            }
        }

        self.sources = input.sources.clone();
        self.source_fingerprints = new_fingerprints;

        // 2. Predeclare Every Source Declaration
        let mut declarations = self.base_declarations.clone();
        let mut hierarchy = self.base_hierarchy.clone();
        let mut shell_table = DeclarationShellTable::default();
        let mut initial_blueprints: Vec<DeclarationBlueprint> = declarations
            .iter()
            .map(|(decl_id, _)| DeclarationBlueprint {
                id: decl_id.clone(),
                kind: DeclarationKind::Class,
            })
            .collect();

        for (module_id, parsed_unit) in &input.sources {
            for stmt in &parsed_unit.program.statements {
                if let Statement::Class(class_def) = stmt {
                    let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                    initial_blueprints.push(DeclarationBlueprint {
                        id: decl_id.clone(),
                        kind: DeclarationKind::Class,
                    });
                    if declarations.get(&decl_id).is_none() {
                        let kind = if !class_def.generic_parameters.is_empty() {
                            let param_kinds: Vec<KindId> = class_def
                                .generic_parameters
                                .iter()
                                .map(|p| p.kind.as_ref().map_or(KindId::TYPE, |k| resolve_kind_syntax(&mut self.store, k)))
                                .collect();
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
        }
        shell_table.predeclare(initial_blueprints);

        // 3. Construct LinkedTypeResolver
        let known_declarations: HashSet<DeclarationId> = declarations.iter().map(|(decl_id, _)| decl_id.clone()).collect();
        let resolver = LinkedTypeResolver::new(input.linked.clone(), known_declarations, ModuleId::core());

        // 4. Enrich Semantic Graph
        let mut semantic_graph = input.linked.graphs.semantics.clone();
        for (module_id, parsed_unit) in &input.sources {
            for stmt in &parsed_unit.program.statements {
                if let Statement::Class(class_def) = stmt {
                    let from_node = SemanticNodeId::Declaration {
                        module: module_id.clone(),
                        name: class_def.name.clone().into(),
                    };

                    if let Some(super_ref) = class_def.superclass_ref() {
                        let members: Vec<String> = super_ref.members.iter().map(|m| m.name.clone()).collect();
                        if let Some(target_decl) = resolver.resolve_type_name(module_id, &super_ref.root, &members) {
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
                    }
                }
            }
        }

        // 5. Realize Declaration Shells
        let mut diags_by_module: BTreeMap<ModuleId, Vec<SemanticDiagnostic>> = BTreeMap::new();
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

        // Generic signatures and supertype templates
        for (module_id, parsed_unit) in &input.sources {
            for stmt in &parsed_unit.program.statements {
                if let Statement::Class(class_def) = stmt {
                    let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                    let generic_signature = if !class_def.generic_parameters.is_empty() {
                        Some(resolve_generic_signature(
                            &mut self.store,
                            &declarations,
                            &resolver,
                            module_id,
                            TypeParameterOwner::Declaration(decl_id.clone()),
                            &class_def.generic_parameters,
                            class_def.where_clause.as_ref(),
                            diags_by_module.entry(module_id.clone()).or_default(),
                        ))
                    } else {
                        None
                    };

                    let supertype_template = if let Some(super_ann) = &class_def.superclass {
                        let type_params_map = if let Some(ref sig) = generic_signature {
                            let mut map = std::collections::HashMap::new();
                            for &param_id in sig.parameters.iter() {
                                let name = self.store.type_parameter(param_id).name.to_string();
                                let param_form = self.store.parameter_form(param_id);
                                map.insert(name, param_form);
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
                        let form_res =
                            crate::types::annotation::resolve_type_form(&mut self.store, &declarations, &scoped_resolver, module_id, super_ann, &mut diags);
                        let super_ty = match form_res {
                            crate::types::annotation::TypeFormResolution::Known(ty) => Some(ty),
                            crate::types::annotation::TypeFormResolution::Dynamic | crate::types::annotation::TypeFormResolution::Unknown(_) => None,
                        };
                        diags_by_module.entry(module_id.clone()).or_default().extend(diags);
                        super_ty.map(|ty| GenericSupertypeTemplate {
                            declaration: decl_id.clone(),
                            supertype: ty,
                        })
                    } else {
                        None
                    };

                    if let Some(info) = declarations.get(&decl_id).cloned() {
                        declarations.insert(DeclarationTypeInfo {
                            declaration: info.declaration,
                            form: info.form,
                            class_object_type: info.class_object_type,
                            kind: info.kind,
                            generic_signature,
                            supertype_template,
                        });
                    }
                }
            }
        }

        // Publish declaration type metadata as explicit DB products before any
        // formal surface, signature, or body query can consume it.
        let mut published_shells = BTreeSet::new();
        for (module_id, parsed_unit) in &input.sources {
            for statement in &parsed_unit.program.statements {
                if let Statement::Class(class_def) = statement {
                    let declaration = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                    if published_shells.insert(declaration.clone()) {
                        let Some(info) = declarations.get(&declaration).cloned() else {
                            return Err(QueryOutcome::Failed(format!("missing declaration metadata for {declaration:?}")));
                        };
                        match query_declaration_shell(&mut self.db, Arc::new(info)) {
                            QueryOutcome::Ready(_) => {}
                            QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                            QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                            QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                            QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                        }
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
        for (module_id, parsed_unit) in &input.sources {
            let Some(linked_module) = input.linked.modules.get(module_id) else {
                return Err(QueryOutcome::Failed(format!(
                    "linked module prerequisite is missing for semantic source {module_id:?}"
                )));
            };
            let linked_interface = Arc::new(linked_module.interface.clone());

            for stmt in &parsed_unit.program.statements {
                if let Statement::Class(class_def) = stmt {
                    let class_decl = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                    let edge = match query_hierarchy_edge(&mut self.db, class_decl.clone(), parsed_unit.clone(), linked_interface.clone(), &resolver) {
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
                }
            }
        }

        // 6. Materialize compatibility dispatch/signature tables from DB-owned formal products.
        let mut dispatch = self.base_dispatch.clone();
        let mut callable_signatures = self.base_callable_signatures.clone();
        let mut field_signatures = FieldSignatureTable::new();

        for (module_id, parsed_unit) in &input.sources {
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
                {
                    let mut context = CheckingContext::new(&mut self.store, &hierarchy, &resolver, &declarations, module_id.clone());
                    for member in &class_def.members {
                        if let Some(signature) = crate::checker::declaration_signature::semantic_field_signature_for_member(&mut context, &decl_id, member) {
                            field_signatures.insert(signature);
                        }
                    }
                    if !context.diagnostics.is_empty() {
                        diags_by_module.entry(module_id.clone()).or_default().extend(context.diagnostics);
                    }
                }
                // Publish declaration-owned callable signatures first. Dispatch
                // surfaces are compatibility projections of these facts.
                for member in &class_def.members {
                    let Some(callable_id) = crate::checker::declaration_signature::callable_id_for_member(&decl_id, member) else {
                        continue;
                    };
                    match query_callable_signature(
                        &mut self.db,
                        callable_id,
                        parsed_unit.clone(),
                        &mut self.store,
                        &hierarchy,
                        &resolver,
                        &declarations,
                    ) {
                        QueryOutcome::Ready(signature) => callable_signatures.insert((*signature).clone()),
                        QueryOutcome::Blocked(reason) => return Err(QueryOutcome::Blocked(reason)),
                        QueryOutcome::Cancelled => return Err(QueryOutcome::Cancelled),
                        QueryOutcome::BudgetExceeded(report) => return Err(QueryOutcome::BudgetExceeded(report)),
                        QueryOutcome::Failed(error) => return Err(QueryOutcome::Failed(error)),
                    }
                }

                let surface = match query_declaration_surface(
                    &mut self.db,
                    decl_id.clone(),
                    parsed_unit.clone(),
                    linked_interface.clone(),
                    &mut self.store,
                    &hierarchy,
                    &resolver,
                    &declarations,
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
            }
        }

        // 7. Check field defaults, constructors, then ordinary callable bodies.
        // Constructor-first ordering makes lifecycle publication independent of
        // source member order.
        let mut default_field_lifecycle = crate::checker::field_lifecycle::FieldLifecycleTable::default();
        for (module_id, parsed_unit) in &input.sources {
            let mut ctx = CheckingContext::new_with_dispatch_ref(&mut self.store, &hierarchy, &resolver, &declarations, &dispatch, module_id.clone());
            for stmt in &parsed_unit.program.statements {
                if let Statement::Class(class_def) = stmt {
                    default_field_lifecycle.extend(crate::checker::field_lifecycle::default_field_seeds(&mut ctx, class_def));
                }
            }
        }
        let mut field_lifecycle = default_field_lifecycle.clone();
        let mut callable_analyses = HashMap::new();
        for constructors_only in [true, false] {
            for (module_id, parsed_unit) in &input.sources {
                for stmt in &parsed_unit.program.statements {
                    if let Statement::Class(class_def) = stmt {
                        let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                        let type_params_map = if let Some(sig) = declarations.generic_signature(&decl_id) {
                            let mut map = std::collections::HashMap::new();
                            for &param_id in sig.parameters.iter() {
                                let name = self.store.type_parameter(param_id).name.to_string();
                                let param_form = self.store.parameter_form(param_id);
                                map.insert(name, param_form);
                            }
                            map
                        } else {
                            std::collections::HashMap::new()
                        };
                        let scoped_resolver = crate::types::annotation::ScopedTypeResolver {
                            parent: &resolver,
                            type_parameters: type_params_map,
                        };

                        for member in &class_def.members {
                            let is_constructor =
                                matches!(member, ClassMember::Method(m) if m.is_constructor || m.attributes.iter().any(|a| a.name == "constructor"));
                            if is_constructor != constructors_only {
                                continue;
                            }
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
                                        .map(|p| {
                                            if let Some(ref l) = p.label {
                                                phalcom_common::selector::SelectorSlot::Label(l.clone())
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
                                    sources: &input.sources,
                                    linked: &input.linked,
                                    hierarchy: &hierarchy,
                                    base_resolver: &resolver,
                                    declarations: &declarations,
                                    field_lifecycle: Some(&field_lifecycle),
                                };
                                let outcome = query_callable_body_with_formal_inputs(
                                    &mut self.db,
                                    callable_id.clone(),
                                    body,
                                    range,
                                    &mut self.store,
                                    &hierarchy,
                                    &scoped_resolver,
                                    &declarations,
                                    &dispatch,
                                    module_id.clone(),
                                    budget,
                                    cancel,
                                    Some(&formal_inputs),
                                );

                                match outcome {
                                    QueryOutcome::Ready(analysis) => {
                                        if self.db.query_state(&query_key).is_some_and(|s| s.revision() == Some(self.db.revision())) {
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
                                        callable_analyses.insert(callable_id.clone(), analysis);
                                        if is_constructor {
                                            let finalized = crate::checker::field_lifecycle::finalize_instance_field_lifecycle(
                                                &default_field_lifecycle,
                                                callable_analyses
                                                    .values()
                                                    .filter(|analysis| analysis.callable.owner == decl_id && analysis.callable.side == DispatchSide::Instance)
                                                    .filter(|analysis| {
                                                        callable_signatures
                                                            .get_for_body(&analysis.callable)
                                                            .is_some_and(|signature| signature.is_constructor())
                                                    })
                                                    .map(AsRef::as_ref),
                                            );
                                            for (field, fact) in finalized.fields {
                                                if field.owner == decl_id {
                                                    field_lifecycle.fields.insert(field, fact);
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
                    }
                }
            }
        }

        for (module_id, parsed_unit) in &input.sources {
            let mut ctx = CheckingContext::new_with_dispatch_ref(&mut self.store, &hierarchy, &resolver, &declarations, &dispatch, module_id.clone());

            for stmt in &parsed_unit.program.statements {
                match stmt {
                    Statement::Class(class_def) => {
                        check_class_field_initializers(&mut ctx, class_def);
                    }
                    _ => {
                        check_statement(&mut ctx, stmt);
                    }
                }
            }

            if !ctx.diagnostics.is_empty() {
                diags_by_module.entry(module_id.clone()).or_default().extend(ctx.diagnostics);
            }
        }

        // Source return annotations are published before bodies are checked,
        // so an unannotated callable initially has an unknown return surface.
        // Publish body-derived normal-return summaries into the local dispatch
        // view, then recheck callers until that view reaches a fixed point.
        refresh_inferred_callable_results(
            &input.sources,
            &mut self.store,
            &hierarchy,
            &resolver,
            &declarations,
            &mut dispatch,
            &mut callable_signatures,
            &mut callable_analyses,
            previous_snapshot.as_ref().map(|snapshot| snapshot.callable_analyses.as_ref()),
            &field_lifecycle,
            &mut callable_dispositions,
            &mut diags_by_module,
            budget,
            cancel,
        )?;

        // 8. Freeze and Publish Immutable Snapshot
        let mut diagnostics_map = BTreeMap::new();
        for (module_id, diags) in diags_by_module {
            diagnostics_map.insert(module_id, Arc::from(diags.into_boxed_slice()));
        }

        let mut unlinked_map = BTreeMap::new();
        let mut linked_map = BTreeMap::new();
        let mut resolved_imports_map = self.module_session.resolved_imports().clone();
        let mut sources_loc_map = BTreeMap::new();

        for (mod_id, unit) in &input.sources {
            if let Ok(unlinked) = InterfaceBuilder::build(mod_id.clone(), unit.kind, &unit.program) {
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

        let module_products = Arc::new(crate::snapshot::ModuleQueryProducts::new(
            input.linked.universe.clone(),
            Arc::new(unlinked_map),
            Arc::new(linked_map),
            Arc::new(resolved_imports_map.clone()),
            Arc::new(sources_loc_map),
        ));

        let (mut source_index, presentation_sources) =
            build_source_semantic_index(&input.sources, &callable_analyses, &resolved_imports_map, input.linked.as_ref(), &resolver);
        if let Some(previous) = self.last_snapshot.as_deref() {
            for (module, current) in source_index.modules.clone() {
                let Some(previous_module) = previous.source_index.module_arc(&module) else {
                    continue;
                };
                if previous_module.fingerprints() == current.fingerprints() {
                    source_index.modules.insert(module, previous_module);
                }
            }
            source_index.rebuild_target_occurrences();
        }
        for (module, module_index) in &source_index.modules {
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
        let mut advisory = build_advisory_workspace(
            &input.sources,
            &source_index,
            &callable_analyses,
            &self.store,
            &declarations,
            &callable_signatures,
            &dispatch,
            &hierarchy,
            input.linked.as_ref(),
            self.last_snapshot.as_deref().map(|snapshot| snapshot.advisory.as_ref()),
            budget,
            cancel,
        );
        let mut advisory_query_failed = None;
        for summary in advisory.callables.values() {
            if let QueryOutcome::Failed(error) = bootstrap_advisory_callable(&mut self.db, summary.clone()) {
                advisory_query_failed = Some(error);
                break;
            }
        }
        if advisory_query_failed.is_none() {
            for summary in advisory.callables.values() {
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
                    .filter(|callable| callable.owner.module == *module)
                    .cloned()
                    .collect::<Vec<_>>();
                if let QueryOutcome::Failed(error) = query_advisory_module(&mut self.db, module_product.clone(), callables) {
                    advisory_query_failed = Some(error);
                    break;
                }
            }
        }
        if let Some(error) = advisory_query_failed {
            advisory = advisory.with_status(AdvisoryProductStatus::InternalFailure(error.into_boxed_str()));
        }
        let mut snapshot_obj = SemanticSnapshot::new_with_callable_analyses(
            self.workspace,
            self.db.revision(),
            input.generation,
            Arc::new(self.store.clone()),
            Arc::new(input.sources),
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
        snapshot_obj.advisory = Arc::new(advisory);
        snapshot_obj.module_products = module_products;
        let snapshot = Arc::new(snapshot_obj);

        let mut diagnostics_changed = BTreeSet::new();
        let previous_diagnostics = previous_snapshot.as_ref().map(|snapshot| snapshot.diagnostics.as_ref());
        let module_ids = previous_sources.keys().chain(snapshot.sources.keys()).cloned().collect::<BTreeSet<_>>();
        for module in module_ids {
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
        stats.advisory_callables_recomputed = snapshot
            .callable_analyses
            .values()
            .filter(|analysis| changed_modules.contains(&analysis.callable.owner.module))
            .count();
        stats.modules_relinked = module_graph_changed.then_some(changed_modules.len()).unwrap_or_default();
        stats.project_graph_rebuilt = effects.module_graph_changed;
        stats.callables_recomputed = callable_dispositions
            .values()
            .filter(|disposition| **disposition == CallableRevisionDisposition::Recomputed)
            .count();
        stats.callables_reused = callable_dispositions
            .values()
            .filter(|disposition| **disposition == CallableRevisionDisposition::Reused)
            .count();
        let recomputed_keys = callable_dispositions
            .iter()
            .filter_map(|(callable, disposition)| (*disposition == CallableRevisionDisposition::Recomputed).then(|| QueryKey::CallableBody(callable.clone())))
            .collect::<Vec<_>>();

        self.last_snapshot = Some(snapshot.clone());
        self.last_known_good = Some(snapshot.clone());

        Ok(SemanticWorkspaceUpdate {
            snapshot,
            invalidated: Arc::from(invalidated_keys.into_iter().collect::<Vec<_>>()),
            recomputed: Arc::from(recomputed_keys),
            stats,
            effects,
        })
    }
}

fn compute_module_fingerprint(unit: &ParsedModuleUnit) -> u64 {
    let mut hasher = DefaultHasher::new();
    unit.id.hash(&mut hasher);
    unit.text.hash(&mut hasher);
    hasher.finish()
}

fn build_source_semantic_index(
    sources: &BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    callable_analyses: &HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>,
    resolved_imports: &BTreeMap<(ModuleId, String), ModuleId>,
    linked: &LinkedProgram,
    type_resolver: &dyn TypeResolver,
) -> (SourceSemanticIndex, BTreeMap<ModuleId, Arc<str>>) {
    let mut context = SourceIndexContext {
        resolved_imports: resolved_imports.clone(),
        ..SourceIndexContext::default()
    };
    for (module, source) in sources {
        for (range, declaration) in resolve_type_reference_targets(module, &source.program, type_resolver) {
            context.type_reference_targets.insert((module.clone(), range), declaration);
        }
    }
    for (module, linked_module) in &linked.modules {
        if !sources.contains_key(module) {
            continue;
        }
        context.modules.entry(module.path.to_string()).or_insert_with(|| module.clone());
        context.modules.entry(module.to_string()).or_insert_with(|| module.clone());
        if let Some(source) = sources.get(module) {
            for dependency in &source.program.preamble.dependencies {
                let DependencyDecl::Import(ImportDecl::Module(module_import)) = dependency else {
                    continue;
                };
                let binding_name = module_import
                    .alias
                    .as_ref()
                    .map(|alias| alias.name.as_str())
                    .or_else(|| module_import.path.segments.last().map(|segment| segment.name.as_str()))
                    .or_else(|| match &module_import.path.root {
                        phalcom_ast::ast::ImportRoot::Absolute(segment) => Some(segment.name.as_str()),
                        phalcom_ast::ast::ImportRoot::Relative { .. } => None,
                    });
                let Some(binding_name) = binding_name else {
                    continue;
                };
                let Some(import_id) = linked_module.bindings.imports.get(binding_name) else {
                    continue;
                };
                let Some(phalcom_modules::linker::LinkedReadSpec::Module(target)) = linked_module.linked_reads.get(import_id.0 as usize) else {
                    continue;
                };
                context
                    .resolved_imports
                    .insert((module.clone(), module_import.path.to_string()), target.clone());
            }
        }
        for export in linked_module.interface.exports.values() {
            match &export.target {
                LinkedExportTarget::Binding(symbol) => {
                    context.targets.insert(
                        (module.clone(), export.public_name.to_string()),
                        SemanticTargetId::Declaration(DeclarationId::new(symbol.module.clone(), symbol.name.clone())),
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
    let scopes: BTreeMap<ModuleId, crate::source_index::SourceScopeIndex> = sources
        .iter()
        .map(|(module, source)| (module.clone(), build_source_scope_index(module.clone(), &source.program, &context)))
        .collect();
    for structure in scopes.values() {
        for callable in structure.callable_sources.values() {
            context
                .callable_targets
                .insert((callable.id.owner.clone(), callable.id.selector.clone()), callable.id.clone());
        }
    }
    for (module, source) in sources {
        for statement in &source.program.statements {
            let phalcom_ast::ast::Statement::Class(class) = statement else { continue };
            context
                .targets
                .entry((module.clone(), class.name.clone()))
                .or_insert_with(|| SemanticTargetId::Declaration(DeclarationId::new(module.clone(), class.name.clone().into())));
        }
    }
    let mut index = SourceSemanticIndex::from_scope_indices_with_programs_and_context(scopes, sources, Some(&context));
    for analysis in callable_analyses.values() {
        let module = &analysis.callable.owner.module;
        if index.module(module).is_some() {
            let _ = index.attach_formal_analysis(module, analysis);
        }
    }

    let mut presentation_sources = BTreeMap::new();
    let core = ModuleId::core();
    if !sources.contains_key(&core) {
        let text = render_canonical_core_source();
        let parsed = phalcom_ast::parse(&text, 0);
        assert!(
            parsed.errors.is_empty(),
            "compiler-owned canonical core presentation must parse: {:#?}",
            parsed.errors
        );
        let structure = build_source_scope_index(core.clone(), &parsed.program, &SourceIndexContext::default());
        let mut core_index = SourceSemanticIndex::from_scope_indices(BTreeMap::from([(core.clone(), structure)]));
        let shard = core_index.modules.remove(&core).expect("canonical core presentation shard");
        index.modules.insert(core.clone(), shard);
        index.rebuild_target_occurrences();
        presentation_sources.insert(core, text);
    }

    (index, presentation_sources)
}

/// Builds the advisory workspace from the same source/formal products that
/// will be published in the immutable snapshot. Missing source attachments
/// reduce advisory coverage only; they never prevent formal publication.
fn build_advisory_workspace(
    sources: &BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    source_index: &SourceSemanticIndex,
    callable_analyses: &HashMap<CallableId, Arc<crate::checker::CallableAnalysis>>,
    store: &TypeStore,
    declarations: &DeclarationTypeTable,
    callable_signatures: &CallableSignatureTable,
    dispatch: &SurfaceDispatchResolver,
    hierarchy: &MapTypeHierarchy,
    linked: &LinkedProgram,
    previous: Option<&AdvisoryWorkspace>,
    budget: QueryBudget,
    cancel: &CancellationToken,
) -> AdvisoryWorkspace {
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
                    if signature.parameters.iter().any(|parameter| parameter.rest) {
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
            let mut fields = advisory_field_facts(
                source,
                scope_index,
                &builtins,
                &advisory_returns,
                Some(&resolve_callable_for_shape),
                Some(&resolve_formal_call_result),
                Some(&advisory_transfer_target),
                Some(&resolve_module_member),
                Some(&resolve_method_family),
            );
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

            for analysis in ordered_analyses.iter().filter(|analysis| analysis.callable.owner.module == *module) {
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
                    current_owner: Some(&analysis.callable.owner),
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
            contributions.replace_source(
                crate::advisory::AdvisoryContributionSource::Module(callable.owner.module.clone()),
                own_parameters,
            );
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

fn advisory_field_facts(
    source: &ParsedModuleUnit,
    scope_index: &crate::source_index::SourceScopeIndex,
    builtins: &AdvisoryBuiltins,
    callable_returns: &BTreeMap<CallableId, AdvisoryFact>,
    resolve_callable_for_shape: Option<&dyn Fn(&crate::advisory::ValueShape, &str, &[PackItem]) -> Option<CallableId>>,
    resolve_formal_call_result: Option<&dyn Fn(&CallableId, Option<&crate::advisory::ValueShape>) -> Option<AdvisoryFact>>,
    advisory_transfer_target: Option<&dyn Fn(&CallableId) -> CallableId>,
    resolve_module_member: Option<&dyn Fn(&crate::advisory::ValueShape, &str) -> Option<crate::advisory::ValueShape>>,
    resolve_method_family: Option<
        &dyn Fn(&crate::advisory::ValueShape, &phalcom_ast::ast::NormalizedSelectorSpec) -> Option<crate::advisory::CapturedMethodFamilyShape>,
    >,
) -> BTreeMap<FieldId, AdvisoryFact> {
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
                .map(|parameter| {
                    parameter.label.as_ref().map_or(phalcom_common::selector::SelectorSlot::Positional, |label| {
                        phalcom_common::selector::SelectorSlot::Label(label.clone())
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
        SemanticTargetId::Declaration(_) | SemanticTargetId::Module(_) => AdvisoryOrigin::Constraint(site.clone()),
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
fn refresh_inferred_callable_results(
    sources: &BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    store: &mut TypeStore,
    hierarchy: &MapTypeHierarchy,
    resolver: &LinkedTypeResolver,
    declarations: &DeclarationTypeTable,
    dispatch: &mut SurfaceDispatchResolver,
    callable_signatures: &mut CallableSignatureTable,
    callable_analyses: &mut HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>,
    previous_callable_analyses: Option<&HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>>,
    field_lifecycle: &crate::checker::field_lifecycle::FieldLifecycleTable,
    callable_dispositions: &mut BTreeMap<CallableId, CallableRevisionDisposition>,
    diagnostics: &mut BTreeMap<ModuleId, Vec<SemanticDiagnostic>>,
    budget: QueryBudget,
    cancel: &CancellationToken,
) -> Result<(), QueryOutcome<()>> {
    let max_iterations = callable_analyses.len().saturating_add(1).max(1);

    for _ in 0..max_iterations {
        if cancel.is_cancelled() {
            return Err(QueryOutcome::Cancelled);
        }

        let candidates = callable_analyses
            .iter()
            .filter_map(|(callable, analysis)| {
                let signature = callable_signatures.get_for_body(callable)?;
                if !signature.published_return_knowledge().is_unknown() {
                    return None;
                }
                Some((callable.clone(), signature.callable.clone(), analysis.exits.normal_return_values.clone()))
            })
            .collect::<Vec<_>>();

        let mut changed_callables = HashSet::new();
        for (callable, signature_id, values) in candidates {
            let summary = normal_return_summary(store, &values);
            if !summary.is_known() {
                continue;
            }
            let Some(signature) = callable_signatures.get_mut(&signature_id) else {
                continue;
            };
            if signature.inferred_return.as_ref() == Some(&summary) {
                continue;
            }
            signature.inferred_return = Some(summary.clone());
            changed_callables.insert(callable.clone());

            // Dispatch is a derived lookup projection. Failure to update that
            // projection must never suppress canonical semantic publication.
            let _ = dispatch.update_callable_return_type(&signature_id, summary);
        }

        if changed_callables.is_empty() {
            break;
        }

        // Recheck only bodies that consume a newly published return contract.
        // This deliberately bypasses the source-surface query cache: that
        // cache owns the pre-inference unknown contract, while this pass is
        // producing the current revision's inferred contract. Unrelated
        // callable products remain pointer-stable for incremental reuse.
        for (module_id, parsed_unit) in sources {
            for stmt in &parsed_unit.program.statements {
                let Statement::Class(class_def) = stmt else {
                    continue;
                };
                let decl_id = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                let type_params_map = if let Some(sig) = declarations.generic_signature(&decl_id) {
                    let mut map = std::collections::HashMap::new();
                    for &param_id in sig.parameters.iter() {
                        let name = store.type_parameter(param_id).name.to_string();
                        let param_form = store.parameter_form(param_id);
                        map.insert(name, param_form);
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
                                .map(|p| {
                                    if let Some(ref label) = p.label {
                                        phalcom_common::selector::SelectorSlot::Label(label.clone())
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
                    if !affected {
                        continue;
                    }
                    let declared_signature = callable_signatures.get_for_body(&callable).map(|signature| (&signature.callable, signature));
                    let mut analysis = crate::checker::body::analyze_callable_body_with_fields(
                        callable.clone(),
                        body,
                        range,
                        store,
                        hierarchy,
                        &scoped_resolver,
                        declarations,
                        dispatch,
                        declared_signature,
                        module_id.clone(),
                        budget,
                        cancel,
                        Some(field_lifecycle),
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
                    callable_analyses.insert(callable.clone(), replacement);
                    callable_dispositions.insert(callable, CallableRevisionDisposition::Recomputed);
                }
            }
        }
    }

    Ok(())
}
