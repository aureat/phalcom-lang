//! Compiler-owned incremental workspace session (Spec 04.5 / Wave 5 / Tasks 16-18).

use crate::checker::context::CheckingContext;
use crate::checker::declaration::{check_class_bodies, register_class_surface};
use crate::checker::statement::check_statement;
use crate::db::budget::{CancellationToken, QueryBudget};
use crate::db::key::QueryKey;
use crate::db::query::query_callable_body;
use crate::db::state::QueryOutcome;
use crate::db::{InputFingerprint, ProductFingerprint, SemanticDb, SemanticProduct};
use crate::declarations::{DeclarationTypeInfo, DeclarationTypeTable, GenericSupertypeTemplate, bootstrap_universe_declarations};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::SurfaceDispatchResolver;
use crate::identity::{DeclarationId, ModuleId, WorkspaceId};
use crate::resolver::LinkedTypeResolver;
use crate::signature::CallableSignatureTable;
use crate::snapshot::SemanticSnapshot;
use crate::source::ParsedModuleUnit;
use crate::types::annotation::{TypeResolver, resolve_generic_signature, resolve_kind_syntax};
use crate::types::id::KindId;
use crate::types::native::register_native_surfaces;
use crate::types::parameter::TypeParameterOwner;
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::TypeStore;
use crate::workspace::SemanticWorkspaceInput;
use phalcom_ast::ast::{ClassMember, Statement};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::declaration::{DeclarationBlueprint, DeclarationKind, DeclarationRealizationError, DeclarationShellTable};
use phalcom_modules::graph::{SemanticEdge, SemanticEdgeKind, SemanticNodeId};
use phalcom_modules::interface::InterfaceBuilder;
use phalcom_modules::linker::LinkedProgram;
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
}

/// The result of an incremental semantic workspace update.
#[derive(Clone, Debug)]
pub struct SemanticWorkspaceUpdate {
    pub snapshot: Arc<SemanticSnapshot>,
    pub invalidated: Arc<[QueryKey]>,
    pub recomputed: Arc<[QueryKey]>,
    pub stats: SemanticUpdateStats,
}

/// Compiler-owned stateful semantic workspace session.
///
/// Owns the canonical `SemanticDb`, interner `TypeStore`, dependency index,
/// and published immutable snapshots across source revisions.
#[derive(Debug)]
pub struct SemanticWorkspaceSession {
    workspace: WorkspaceId,
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

        let base_declarations = bootstrap_universe_declarations(&mut store, &|key| {
            DeclarationId::new(ModuleId::core(), key.name().into())
        });

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

        let mut base_callable_signatures = CallableSignatureTable::new();
        for (_, signature) in native_report.callable_signatures {
            base_callable_signatures.insert(signature);
        }

        Self {
            workspace,
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
        let mut recomputed_keys = Vec::new();

        // 1. Detect changed/removed/new source modules
        let mut new_fingerprints = BTreeMap::new();
        for (module_id, unit) in &input.sources {
            let fp = compute_module_fingerprint(unit);
            new_fingerprints.insert(module_id.clone(), fp);

            let changed = match self.source_fingerprints.get(module_id) {
                Some(&old_fp) => old_fp != fp,
                None => true,
            };

            if changed {
                stats.modules_recomputed += 1;
                let parsed_key = QueryKey::ParsedModule(module_id.clone());
                let unlinked_key = QueryKey::UnlinkedInterface(module_id.clone());
                let linked_key = QueryKey::LinkedInterface(module_id.clone());
                let diags_key = QueryKey::ModuleDiagnostics(module_id.clone());

                let closure = self.db.invalidate([parsed_key, unlinked_key, linked_key, diags_key]);
                invalidated_keys.extend(closure);

                // Publish parsed module and unlinked interface into DB
                let _ = self.db.publish_product_ready(
                    QueryKey::ParsedModule(module_id.clone()),
                    self.db.revision(),
                    InputFingerprint::new(fp),
                    ProductFingerprint::new(fp),
                    SemanticProduct::ParsedModule(unit.clone()),
                    Vec::new(),
                );

                if let Ok(unlinked) = InterfaceBuilder::build(module_id.clone(), unit.kind, &unit.program) {
                    let _ = self.db.publish_product_ready(
                        QueryKey::UnlinkedInterface(module_id.clone()),
                        self.db.revision(),
                        InputFingerprint::new(fp),
                        ProductFingerprint::new(fp),
                        SemanticProduct::UnlinkedInterface(Arc::new(unlinked)),
                        Vec::new(),
                    );
                }
            }
        }

        for old_module_id in self.sources.keys() {
            if !input.sources.contains_key(old_module_id) {
                let parsed_key = QueryKey::ParsedModule(old_module_id.clone());
                let unlinked_key = QueryKey::UnlinkedInterface(old_module_id.clone());
                let linked_key = QueryKey::LinkedInterface(old_module_id.clone());
                let diags_key = QueryKey::ModuleDiagnostics(old_module_id.clone());
                let closure = self.db.invalidate([parsed_key, unlinked_key, linked_key, diags_key]);
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
                        let form_res = crate::types::annotation::resolve_type_form(&mut self.store, &declarations, &scoped_resolver, module_id, super_ann, &mut diags);
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

        // Build hierarchy
        for (module_id, parsed_unit) in &input.sources {
            for stmt in &parsed_unit.program.statements {
                if let Statement::Class(class_def) = stmt {
                    let class_decl = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                    let (super_decl_opt, super_decl_for_insert) = if let Some(super_ref) = class_def.superclass_ref() {
                        let members: Vec<String> = super_ref.members.iter().map(|m| m.name.clone()).collect();
                        if let Some(super_decl) = resolver.resolve_type_name(module_id, &super_ref.root, &members) {
                            (Some(super_decl.clone()), Some(super_decl))
                        } else {
                            diags_by_module.entry(module_id.clone()).or_default().push(SemanticDiagnostic::error_in(
                                module_id.clone(),
                                DiagnosticCode::AnnotationUnresolved,
                                format!("unresolved superclass `{}`", super_ref.root),
                                super_ref.range,
                            ));
                            (None, None)
                        }
                    } else {
                        let obj_decl = DeclarationId::new(ModuleId::core(), "Object".into());
                        if class_decl != obj_decl {
                            (Some(obj_decl.clone()), Some(obj_decl))
                        } else {
                            (None, None)
                        }
                    };
                    if let Some(s) = super_decl_for_insert {
                        hierarchy.insert(class_decl.clone(), s);
                    }
                    let fp = crate::db::fingerprint::hierarchy_edge_product_fingerprint(&class_decl, &super_decl_opt);
                    let _ = self.db.publish_product_ready(
                        QueryKey::HierarchyEdge(class_decl.clone()),
                        self.db.revision(),
                        InputFingerprint::new(fp.raw()),
                        fp,
                        SemanticProduct::HierarchyEdge(Arc::new(crate::hierarchy_product::HierarchyEdgeProduct::new(class_decl, super_decl_opt))),
                        Vec::new(),
                    );
                }
            }
        }

        // Publish linked interfaces
        for (module_id, linked_mod) in &input.linked.modules {
            let fp = crate::db::fingerprint::linked_interface_product_fingerprint(&linked_mod.interface);
            let _ = self.db.publish_product_ready(
                QueryKey::LinkedInterface(module_id.clone()),
                self.db.revision(),
                InputFingerprint::new(fp.raw()),
                fp,
                SemanticProduct::LinkedInterface(Arc::new(linked_mod.interface.clone())),
                Vec::new(),
            );
        }

        // 6. Collect Declaration Surfaces
        let mut dispatch = self.base_dispatch.clone();
        let mut callable_signatures = self.base_callable_signatures.clone();

        for (module_id, parsed_unit) in &input.sources {
            let mut dummy_ctx = CheckingContext::new(&mut self.store, &hierarchy, &resolver, &declarations, module_id.clone());

            for stmt in &parsed_unit.program.statements {
                if let Statement::Class(class_def) = stmt {
                    register_class_surface(&mut dummy_ctx, class_def);
                }
            }

            for (decl_id, surface) in dummy_ctx.dispatch.surfaces() {
                dispatch.register_surface(decl_id.clone(), surface.clone());
                let surf_fp = crate::db::fingerprint::declaration_surface_product_fingerprint(surface);
                let _ = self.db.publish_product_ready(
                    QueryKey::DeclarationSurface(decl_id.clone()),
                    self.db.revision(),
                    InputFingerprint::new(surf_fp.raw()),
                    surf_fp,
                    SemanticProduct::DeclarationSurface(Arc::new(surface.clone())),
                    Vec::new(),
                );
                for (side, member_surface) in [
                    (crate::identity::DispatchSide::Instance, &surface.instance),
                    (crate::identity::DispatchSide::Class, &surface.class),
                ] {
                    for (sel, sig) in &member_surface.callable_signatures {
                        let callable_id = crate::identity::CallableId::new(decl_id.clone(), sel.clone(), side);
                        if let Some(return_type) = sig.return_type.ty() {
                            let parameters = sig
                                .parameters
                                .iter()
                                .enumerate()
                                .filter_map(|(index, parameter)| {
                                    let ty = parameter.ty.ty()?;
                                    let mut p = crate::signature::CallableParameterSemantic::new(index as u32, parameter.local_name.clone(), ty.into());
                                    if let Some(ref l) = parameter.external_label {
                                        p = p.with_label(l.clone());
                                    }
                                    if parameter.rest {
                                        p = p.with_rest(phalcom_ast::ast::RestMode::Positional);
                                    }
                                    Some(p)
                                })
                                .collect::<Vec<_>>()
                                .into_boxed_slice();
                            let sem_sig = crate::signature::CallableSemanticSignature {
                                callable: callable_id.clone(),
                                owner: decl_id.clone(),
                                side,
                                selector: sel.clone(),
                                generics: None,
                                parameters,
                                return_type: return_type.into(),
                                source: None,
                                implementation: phalcom_native_meta::ImplementationKind::Source,
                                native_id: None,
                                effects: phalcom_native_meta::EffectSpec::Unknown,
                                raises: phalcom_native_meta::RaisesSpec::Unknown,
                                flow: phalcom_native_meta::ReturnFlowSpec::Value,
                                lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
                            };
                            callable_signatures.insert(sem_sig.clone());
                            let sig_fp = crate::db::fingerprint::callable_signature_product_fingerprint(&sem_sig);
                            let _ = self.db.publish_product_ready(
                                QueryKey::CallableSignature(callable_id.clone()),
                                self.db.revision(),
                                InputFingerprint::new(sig_fp.raw()),
                                sig_fp,
                                SemanticProduct::CallableSignature(Arc::new(sem_sig)),
                                Vec::new(),
                            );
                        }
                    }
                }
                if let Some(ty) = declarations.form(decl_id) {
                    dispatch.register_type(ty, decl_id.clone());
                }
            }
        }

        // 7. Check Callable Bodies with DB caching and reuse
        let mut callable_analyses = HashMap::new();
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

                            let outcome = query_callable_body(
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
                            );

                            match outcome {
                                QueryOutcome::Ready(analysis) => {
                                    if self.db.query_state(&query_key).is_some_and(|s| s.revision() == Some(self.db.revision())) {
                                        recomputed_keys.push(query_key);
                                        stats.callables_recomputed += 1;
                                    } else {
                                        stats.callables_reused += 1;
                                    }
                                    callable_analyses.insert(callable_id, analysis);
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

            let mut ctx = CheckingContext::new_with_dispatch_ref(
                &mut self.store,
                &hierarchy,
                &resolver,
                &declarations,
                &dispatch,
                module_id.clone(),
            );

            for stmt in &parsed_unit.program.statements {
                match stmt {
                    Statement::Class(class_def) => {
                        check_class_bodies(&mut ctx, class_def);
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

        // 8. Freeze and Publish Immutable Snapshot
        let mut diagnostics_map = BTreeMap::new();
        for (module_id, diags) in diags_by_module {
            diagnostics_map.insert(module_id, Arc::from(diags.into_boxed_slice()));
        }

        let mut unlinked_map = BTreeMap::new();
        let mut linked_map = BTreeMap::new();
        let mut resolved_imports_map = BTreeMap::new();
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
            for (name, _) in &linked_mod.bindings.imports {
                for read_spec in &linked_mod.linked_reads {
                    match read_spec {
                        phalcom_modules::linker::LinkedReadSpec::Binding(sym) => {
                            if sym.name.as_ref() == name.as_ref() {
                                resolved_imports_map.insert((mod_id.clone(), name.to_string()), sym.module.clone());
                            }
                        }
                        phalcom_modules::linker::LinkedReadSpec::Module(target_mod) => {
                            resolved_imports_map.insert((mod_id.clone(), name.to_string()), target_mod.clone());
                        }
                    }
                }
            }
        }

        let module_products = Arc::new(crate::snapshot::ModuleQueryProducts::new(
            input.linked.universe.clone(),
            Arc::new(unlinked_map),
            Arc::new(linked_map),
            Arc::new(resolved_imports_map),
            Arc::new(sources_loc_map),
        ));

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
        snapshot_obj.module_products = module_products;
        let snapshot = Arc::new(snapshot_obj);

        self.last_snapshot = Some(snapshot.clone());
        self.last_known_good = Some(snapshot.clone());

        Ok(SemanticWorkspaceUpdate {
            snapshot,
            invalidated: Arc::from(invalidated_keys.into_iter().collect::<Vec<_>>()),
            recomputed: Arc::from(recomputed_keys),
            stats,
        })
    }
}

fn compute_module_fingerprint(unit: &ParsedModuleUnit) -> u64 {
    let mut hasher = DefaultHasher::new();
    unit.id.hash(&mut hasher);
    unit.text.hash(&mut hasher);
    hasher.finish()
}
