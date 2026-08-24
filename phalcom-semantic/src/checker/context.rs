use crate::checker::analysis::{
    AnalysisStatus, BindingAnalysisIndex, ExpressionAnalysis, ExpressionAnalysisIndex, SemanticDependency,
};
use crate::checker::flow::FlowState;
use crate::declarations::{DeclarationTypeInfo, DeclarationTypeTable};
use crate::diagnostic::SemanticDiagnostic;
use crate::dispatch::{DispatchResult, SurfaceDispatchResolver};
use crate::identity::{
    BindingId, BodyId, CallableId, DeclarationId, DispatchSide, ExpressionId, LocalExpressionId, ModuleId,
};
use crate::types::annotation::TypeResolver;
use crate::types::denotation::{SemanticDenotation, ValueSemanticFact};
use crate::types::evidence::TypeKnowledge;
use crate::types::id::TypeId;
use crate::types::native::register_native_surfaces;
use crate::types::relation::TypeHierarchy;
use crate::types::store::{TypeData, TypeStore};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_native_surface::NATIVE_SURFACES;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;

/// Storage abstraction for dispatch resolver avoiding per-callable cloning.
///
/// Borrowed workspace dispatch remains read-only until a checker operation needs
/// to register a local/nested declaration. At that point [`Self::make_mut`]
/// lazily detaches by cloning once, preserving the no-clone fast path for normal
/// callable analysis.
pub enum DispatchAccess<'a> {
    /// Context owns its dispatch table and may mutate it directly.
    Owned(SurfaceDispatchResolver),
    /// Context borrows the immutable workspace dispatch table.
    Borrowed(&'a SurfaceDispatchResolver),
}

impl<'a> DispatchAccess<'a> {
    /// Returns the current dispatch resolver for read-only queries.
    pub fn get(&self) -> &SurfaceDispatchResolver {
        match self {
            Self::Owned(d) => d,
            Self::Borrowed(d) => d,
        }
    }

    /// Returns a mutable resolver, lazily detaching a borrowed resolver on first mutation.
    pub fn make_mut(&mut self) -> &mut SurfaceDispatchResolver {
        let detached = match self {
            Self::Borrowed(dispatch) => Some((**dispatch).clone()),
            Self::Owned(_) => None,
        };
        if let Some(detached) = detached {
            *self = Self::Owned(detached);
        }
        match self {
            Self::Owned(dispatch) => dispatch,
            Self::Borrowed(_) => unreachable!("borrowed dispatch was detached above"),
        }
    }
}

impl<'a> std::ops::Deref for DispatchAccess<'a> {
    type Target = SurfaceDispatchResolver;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

type SharedSemanticDependencies = Rc<RefCell<BTreeSet<SemanticDependency>>>;

/// The compatibility `core` surface is bootstrapped as an immutable session seed
/// and currently has no corresponding staged DB products. All other module
/// identities are query-owned and must participate in dependency tracking.
fn is_query_owned_module(module: &ModuleId) -> bool {
    let components = module.path.components();
    !(matches!(
        module.project,
        phalcom_modules::ProjectIdentity::Builtin(phalcom_modules::BuiltinProject::Universe)
    ) && components.len() == 1
        && components[0].as_str() == "core")
}

fn record_query_dependency(dependencies: &SharedSemanticDependencies, dependency: SemanticDependency) {
    dependencies.borrow_mut().insert(dependency);
}

fn record_declaration_surface_dependency(dependencies: &SharedSemanticDependencies, declaration: &DeclarationId) {
    if is_query_owned_module(&declaration.module) {
        record_query_dependency(dependencies, SemanticDependency::DeclarationSurface(declaration.clone()));
    }
}

fn record_declaration_shell_dependency(dependencies: &SharedSemanticDependencies, declaration: &DeclarationId) {
    if is_query_owned_module(&declaration.module) {
        record_query_dependency(dependencies, SemanticDependency::DeclarationShell(declaration.clone()));
    }
}

fn record_hierarchy_dependency(dependencies: &SharedSemanticDependencies, declaration: &DeclarationId) {
    if is_query_owned_module(&declaration.module) {
        record_query_dependency(dependencies, SemanticDependency::HierarchyEdge(declaration.clone()));
    }
}

/// Type resolver wrapper that records declaration and linked-interface reads during body checking.
#[derive(Clone)]
pub struct TrackingTypeResolver<'a> {
    inner: &'a dyn TypeResolver,
    dependencies: SharedSemanticDependencies,
}

impl<'a> TrackingTypeResolver<'a> {
    fn new(inner: &'a dyn TypeResolver, dependencies: SharedSemanticDependencies) -> Self {
        Self { inner, dependencies }
    }
}

impl TypeResolver for TrackingTypeResolver<'_> {
    fn resolve_type_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId> {
        let declaration = self.inner.resolve_type_name(current_module, root, members);
        let Some(declaration) = declaration else {
            if is_query_owned_module(current_module) {
                record_query_dependency(
                    &self.dependencies,
                    SemanticDependency::LinkedInterface(current_module.clone()),
                );
            }
            return None;
        };

        record_declaration_shell_dependency(&self.dependencies, &declaration);
        if &declaration.module != current_module
            && is_query_owned_module(current_module)
            && is_query_owned_module(&declaration.module)
        {
            record_query_dependency(
                &self.dependencies,
                SemanticDependency::LinkedInterface(current_module.clone()),
            );
        }
        Some(declaration)
    }

    fn resolve_type_parameter(&self, name: &str) -> Option<TypeId> {
        self.inner.resolve_type_parameter(name)
    }

    fn current_declaration(&self) -> Option<DeclarationId> {
        self.inner.current_declaration()
    }
}

/// Hierarchy wrapper that records each mutable direct edge consumed by body checking.
#[derive(Clone)]
pub struct TrackingTypeHierarchy<'a> {
    inner: &'a dyn TypeHierarchy,
    dependencies: SharedSemanticDependencies,
}

impl<'a> TrackingTypeHierarchy<'a> {
    fn new(inner: &'a dyn TypeHierarchy, dependencies: SharedSemanticDependencies) -> Self {
        Self { inner, dependencies }
    }
}

impl TypeHierarchy for TrackingTypeHierarchy<'_> {
    fn superclass(&self, declaration: &DeclarationId) -> Option<&DeclarationId> {
        record_hierarchy_dependency(&self.dependencies, declaration);
        self.inner.superclass(declaration)
    }

    fn is_subclass(&self, sub: &DeclarationId, sup: &DeclarationId) -> bool {
        if sub == sup {
            return true;
        }

        let mut current = sub;
        let mut visited = BTreeSet::new();
        while visited.insert(current.clone()) {
            record_hierarchy_dependency(&self.dependencies, current);
            let Some(parent) = self.inner.superclass(current) else {
                return false;
            };
            if parent == sup {
                return true;
            }
            current = parent;
        }
        false
    }

    fn supertype_template(&self, declaration: &DeclarationId) -> Option<&crate::declarations::GenericSupertypeTemplate> {
        record_hierarchy_dependency(&self.dependencies, declaration);
        self.inner.supertype_template(declaration)
    }
}

/// Metadata for a scoped local variable binding.
#[derive(Clone, Debug)]
pub struct LocalBindingInfo {
    pub id: BindingId,
    pub declared: Option<TypeId>,
    pub denotation: Option<SemanticDenotation>,
}

/// Environment of local bindings in current lexical block/scope.
#[derive(Clone, Debug, Default)]
pub struct LocalEnv {
    bindings: HashMap<String, ValueSemanticFact>,
}

impl LocalEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, fact: ValueSemanticFact) {
        self.bindings.insert(name.into(), fact);
    }

    pub fn get(&self, name: &str) -> Option<&ValueSemanticFact> {
        self.bindings.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut ValueSemanticFact> {
        self.bindings.get_mut(name)
    }
}

/// The active context during semantic type checking.
pub struct CheckingContext<'a> {
    pub store: &'a mut TypeStore,
    pub hierarchy: TrackingTypeHierarchy<'a>,
    pub resolver: TrackingTypeResolver<'a>,
    pub declarations: &'a DeclarationTypeTable,
    pub current_module: ModuleId,
    pub current_class: Option<DeclarationId>,
    pub current_side: DispatchSide,
    pub expected_return: Option<TypeKnowledge>,
    pub local_envs: Vec<LocalEnv>,
    pub scopes: Vec<HashMap<String, LocalBindingInfo>>,
    pub flow: FlowState,
    pub body_id: BodyId,
    pub next_local_expr_id: u32,
    pub next_binding_id: u32,
    pub expressions: ExpressionAnalysisIndex,
    pub bindings: BindingAnalysisIndex,
    pub explanations: crate::explain::ExplanationArena,
    pub suppressed: std::collections::BTreeMap<ExpressionId, crate::identity::DiagnosticCauseId>,
    pub flow_graph: Option<std::sync::Arc<crate::checker::flow::graph::FlowGraph>>,
    pub dependencies: BTreeSet<CallableId>,
    semantic_dependencies: SharedSemanticDependencies,
    pub dispatch: DispatchAccess<'a>,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

impl<'a> CheckingContext<'a> {
    pub fn new(
        store: &'a mut TypeStore,
        hierarchy: &'a dyn TypeHierarchy,
        resolver: &'a dyn TypeResolver,
        declarations: &'a DeclarationTypeTable,
        current_module: ModuleId,
    ) -> Self {
        let mut dispatch = SurfaceDispatchResolver::new();
        let has_complete_native_core = NATIVE_SURFACES.iter().all(|record| {
            let owner = resolver
                .resolve_type_name(&current_module, record.owner().name(), &[])
                .unwrap_or_else(|| DeclarationId::new(ModuleId::core(), record.owner().name().into()));
            declarations.form(&owner).is_some()
        });
        if has_complete_native_core {
            register_native_surfaces(store, declarations, resolver, &current_module, &mut dispatch)
                .expect("canonical native surface must import during checking");
        }

        Self::new_with_dispatch(store, hierarchy, resolver, declarations, dispatch, current_module)
    }

    pub fn new_with_dispatch(
        store: &'a mut TypeStore,
        hierarchy: &'a dyn TypeHierarchy,
        resolver: &'a dyn TypeResolver,
        declarations: &'a DeclarationTypeTable,
        dispatch: SurfaceDispatchResolver,
        current_module: ModuleId,
    ) -> Self {
        let semantic_dependencies = Rc::new(RefCell::new(BTreeSet::new()));
        Self {
            store,
            hierarchy: TrackingTypeHierarchy::new(hierarchy, semantic_dependencies.clone()),
            resolver: TrackingTypeResolver::new(resolver, semantic_dependencies.clone()),
            declarations,
            current_module,
            current_class: None,
            current_side: DispatchSide::Instance,
            expected_return: None,
            local_envs: vec![LocalEnv::new()],
            scopes: vec![HashMap::new()],
            flow: FlowState::new(),
            body_id: BodyId(0),
            next_local_expr_id: 0,
            next_binding_id: 0,
            expressions: ExpressionAnalysisIndex::new(),
            bindings: BindingAnalysisIndex::new(),
            explanations: crate::explain::ExplanationArena::new(),
            suppressed: std::collections::BTreeMap::new(),
            flow_graph: None,
            dependencies: BTreeSet::new(),
            semantic_dependencies,
            dispatch: DispatchAccess::Owned(dispatch),
            diagnostics: Vec::new(),
        }
    }

    pub fn new_with_dispatch_ref(
        store: &'a mut TypeStore,
        hierarchy: &'a dyn TypeHierarchy,
        resolver: &'a dyn TypeResolver,
        declarations: &'a DeclarationTypeTable,
        dispatch: &'a SurfaceDispatchResolver,
        current_module: ModuleId,
    ) -> Self {
        let semantic_dependencies = Rc::new(RefCell::new(BTreeSet::new()));
        Self {
            store,
            hierarchy: TrackingTypeHierarchy::new(hierarchy, semantic_dependencies.clone()),
            resolver: TrackingTypeResolver::new(resolver, semantic_dependencies.clone()),
            declarations,
            current_module,
            current_class: None,
            current_side: DispatchSide::Instance,
            expected_return: None,
            local_envs: vec![LocalEnv::new()],
            scopes: vec![HashMap::new()],
            flow: FlowState::new(),
            body_id: BodyId(0),
            next_local_expr_id: 0,
            next_binding_id: 0,
            expressions: ExpressionAnalysisIndex::new(),
            bindings: BindingAnalysisIndex::new(),
            explanations: crate::explain::ExplanationArena::new(),
            suppressed: std::collections::BTreeMap::new(),
            flow_graph: None,
            dependencies: BTreeSet::new(),
            semantic_dependencies,
            dispatch: DispatchAccess::Borrowed(dispatch),
            diagnostics: Vec::new(),
        }
    }

    pub fn with_resolver<'b>(&'b mut self, resolver: &'b dyn TypeResolver) -> CheckingContext<'b> {
        CheckingContext {
            store: self.store,
            hierarchy: TrackingTypeHierarchy::new(self.hierarchy.inner, self.semantic_dependencies.clone()),
            resolver: TrackingTypeResolver::new(resolver, self.semantic_dependencies.clone()),
            declarations: self.declarations,
            current_module: self.current_module.clone(),
            current_class: self.current_class.clone(),
            current_side: self.current_side,
            expected_return: self.expected_return.clone(),
            local_envs: self.local_envs.clone(),
            scopes: self.scopes.clone(),
            flow: self.flow.clone(),
            body_id: self.body_id,
            next_local_expr_id: self.next_local_expr_id,
            next_binding_id: self.next_binding_id,
            expressions: self.expressions.clone(),
            bindings: self.bindings.clone(),
            explanations: self.explanations.clone(),
            suppressed: self.suppressed.clone(),
            flow_graph: self.flow_graph.clone(),
            dependencies: self.dependencies.clone(),
            semantic_dependencies: self.semantic_dependencies.clone(),
            dispatch: DispatchAccess::Borrowed(self.dispatch.get()),
            diagnostics: Vec::new(),
        }
    }

    pub fn alloc_binding(&mut self) -> BindingId {
        let id = self.next_binding_id;
        self.next_binding_id += 1;
        BindingId(id)
    }

    pub fn alloc_expression_id(&mut self) -> ExpressionId {
        let local = LocalExpressionId(self.next_local_expr_id);
        self.next_local_expr_id += 1;
        ExpressionId::new(self.body_id, local)
    }

    pub fn record_expression(
        &mut self,
        id: ExpressionId,
        range: SourceRange,
        knowledge: TypeKnowledge,
        denotation: Option<SemanticDenotation>,
        status: AnalysisStatus,
    ) -> ExpressionAnalysis {
        let mut analysis = ExpressionAnalysis::ready(id, range, knowledge);
        analysis.denotation = denotation;
        analysis.status = status;
        self.expressions.insert(id, analysis.clone());
        analysis
    }

    pub fn push_scope(&mut self) {
        self.local_envs.push(LocalEnv::new());
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.local_envs.pop();
        self.scopes.pop();
    }

    pub fn bind_local_var(
        &mut self,
        name: impl Into<String>,
        declared: Option<TypeId>,
        initial: TypeKnowledge,
        mutable: bool,
        denotation: Option<SemanticDenotation>,
        range: SourceRange,
    ) -> BindingId {
        let name_str = name.into();
        let binding_id = self.alloc_binding();
        self.flow.declare(binding_id, name_str.clone(), range, declared, initial.clone(), mutable);
        if let Some(state) = self.flow.get_binding(binding_id) {
            self.bindings.insert(binding_id, state.clone());
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name_str.clone(),
                LocalBindingInfo {
                    id: binding_id,
                    declared,
                    denotation,
                },
            );
        }
        let fact = ValueSemanticFact {
            knowledge: initial,
            denotation,
        };
        if let Some(env) = self.local_envs.last_mut() {
            env.insert(name_str, fact);
        }
        binding_id
    }

    pub fn bind_local(&mut self, name: impl Into<String>, fact: ValueSemanticFact, range: SourceRange) {
        let declared = fact.knowledge.ty();
        self.bind_local_var(name, declared, fact.knowledge, true, fact.denotation, range);
    }

    pub fn lookup_binding_info(&self, name: &str) -> Option<&LocalBindingInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    pub fn lookup_local(&self, name: &str) -> Option<&ValueSemanticFact> {
        for env in self.local_envs.iter().rev() {
            if let Some(k) = env.get(name) {
                return Some(k);
            }
        }
        None
    }

    pub fn lookup_local_knowledge(&self, name: &str) -> Option<TypeKnowledge> {
        if let Some(info) = self.lookup_binding_info(name) {
            if let Some(k) = self.flow.get_current_type(info.id) {
                return Some(k.clone());
            }
        }
        self.lookup_local(name).map(|f| f.knowledge.clone())
    }

    pub fn assign_existing(&mut self, name: &str, fact: ValueSemanticFact) -> bool {
        let mut found = false;
        if let Some(info) = self.lookup_binding_info(name).cloned() {
            self.flow.assign(info.id, fact.knowledge.clone());
            if let Some(state) = self.flow.get_binding(info.id) {
                self.bindings.insert(info.id, state.clone());
            }
            found = true;
        }
        for env in self.local_envs.iter_mut().rev() {
            if let Some(slot) = env.get_mut(name) {
                *slot = fact;
                return true;
            }
        }
        found
    }

    /// Records an explicitly consumed semantic dependency.
    pub(crate) fn record_semantic_dependency(&self, dependency: SemanticDependency) {
        record_query_dependency(&self.semantic_dependencies, dependency);
    }

    pub(crate) fn semantic_dependencies_snapshot(&self) -> BTreeSet<SemanticDependency> {
        self.semantic_dependencies.borrow().clone()
    }

    /// Records the canonical dependency for a callable signature consumed from a declaration surface.
    ///
    /// Source callables with any unknown parameter or return type cannot yet be
    /// represented by the canonical `CallableSemanticSignature` product. Their
    /// declaration surface therefore remains the fail-closed dependency until
    /// inference establishes a complete canonical contract.
    pub(crate) fn record_consumed_callable_signature(
        &self,
        callable: &CallableId,
        signature: &crate::dispatch::CallableSignature,
    ) {
        if !is_query_owned_module(&callable.owner.module) {
            return;
        }
        record_declaration_surface_dependency(&self.semantic_dependencies, &callable.owner);
        if signature.has_complete_types() {
            self.record_semantic_dependency(SemanticDependency::CallableSignature(callable.clone()));
        }
    }

    /// Returns the dispatch resolver currently visible to this context.
    pub fn dispatch_ref(&self) -> &SurfaceDispatchResolver {
        self.dispatch.get()
    }

    /// Reads declaration metadata while recording the declaration-shell dependency.
    pub fn declaration_info(&self, declaration: &DeclarationId) -> Option<&DeclarationTypeInfo> {
        record_declaration_shell_dependency(&self.semantic_dependencies, declaration);
        self.declarations.get(declaration)
    }

    /// Reads a declaration generic signature while recording the declaration-shell dependency.
    pub fn declaration_generic_signature(
        &self,
        declaration: &DeclarationId,
    ) -> Option<crate::types::parameter::GenericSignature> {
        record_declaration_shell_dependency(&self.semantic_dependencies, declaration);
        self.declarations.generic_signature(declaration).cloned()
    }

    fn receiver_declaration(&self, receiver: TypeId) -> Option<DeclarationId> {
        let mut current = receiver;
        loop {
            match self.store.get(current) {
                TypeData::Applied { origin, .. } => current = *origin,
                TypeData::Nominal { declaration } | TypeData::ClassObject { declaration } => return Some(declaration.clone()),
                _ => return None,
            }
        }
    }

    fn substitution_for_applied_receiver(&self, receiver: TypeId) -> Option<crate::types::substitution::TypeSubstitution> {
        if let Some(origin) = self.receiver_declaration(receiver) {
            record_declaration_shell_dependency(&self.semantic_dependencies, &origin);
        }
        crate::types::substitution::substitution_for_applied(self.declarations, self.store, receiver)
    }

    fn specialize_self_type(&mut self, receiver: TypeId, ty: TypeId) -> TypeId {
        if let Some(origin) = self.receiver_declaration(receiver) {
            record_declaration_shell_dependency(&self.semantic_dependencies, &origin);
        }
        if let Some(owner) = self.current_class.as_ref() {
            record_declaration_shell_dependency(&self.semantic_dependencies, owner);
        }
        crate::types::substitution::specialize_self_type(self.store, self.declarations, receiver, ty)
    }

    pub fn resolve_dispatch(&mut self, receiver: TypeId, selector: &Selector, lookup: crate::dispatch::DispatchLookup) -> DispatchResult {
        let (decl, side) = match lookup {
            crate::dispatch::DispatchLookup::Super { defining_class, side } => {
                if let Some(super_decl) = self.hierarchy.superclass(&defining_class) {
                    (super_decl.clone(), side)
                } else {
                    return DispatchResult::Missing;
                }
            }
            crate::dispatch::DispatchLookup::Normal => match self.store.get(receiver) {
                TypeData::ClassObject { declaration } => (declaration.clone(), DispatchSide::Class),
                TypeData::Nominal { declaration } => (declaration.clone(), DispatchSide::Instance),
                TypeData::Applied { origin, .. } => {
                    let mut curr_origin = *origin;
                    while let TypeData::Applied { origin: inner_origin, .. } = self.store.get(curr_origin) {
                        curr_origin = *inner_origin;
                    }
                    if let TypeData::Nominal { declaration } = self.store.get(curr_origin) {
                        (declaration.clone(), DispatchSide::Instance)
                    } else {
                        return DispatchResult::Missing;
                    }
                }
                _ => return DispatchResult::Missing,
            },
        };

        let res = self.dispatch.get().resolve_dispatch_with_trace(&self.hierarchy, &decl, side, selector);
        match res {
            crate::dispatch::ResolvedDispatchResult::Found(resolved) => {
                for owner in resolved.visited_owners.iter() {
                    record_declaration_surface_dependency(&self.semantic_dependencies, owner);
                }
                self.dependencies.insert(resolved.callable.clone());
                self.record_consumed_callable_signature(&resolved.callable, &resolved.signature);
                let mut sig = resolved.signature;
                if let Some(subst) = self.substitution_for_applied_receiver(receiver) {
                    for param in &mut sig.parameters {
                        if let TypeKnowledge::Known(ref mut ev) = param.ty {
                            ev.ty = subst.apply(self.store, ev.ty);
                        }
                    }
                    if let TypeKnowledge::Known(ref mut ev) = sig.return_type {
                        ev.ty = subst.apply(self.store, ev.ty);
                    }
                }
                for param in &mut sig.parameters {
                    if let TypeKnowledge::Known(ref mut ev) = param.ty {
                        ev.ty = self.specialize_self_type(receiver, ev.ty);
                    }
                }
                if let TypeKnowledge::Known(ref mut ev) = sig.return_type {
                    ev.ty = self.specialize_self_type(receiver, ev.ty);
                }
                DispatchResult::Found(sig)
            }
            crate::dispatch::ResolvedDispatchResult::Ambiguous(amb) => {
                for rd in &amb {
                    for owner in rd.visited_owners.iter() {
                        record_declaration_surface_dependency(&self.semantic_dependencies, owner);
                    }
                    self.record_consumed_callable_signature(&rd.callable, &rd.signature);
                }
                DispatchResult::Ambiguous(amb.into_iter().map(|rd| rd.signature).collect())
            }
            crate::dispatch::ResolvedDispatchResult::Missing { visited_owners } => {
                for owner in visited_owners.iter() {
                    record_declaration_surface_dependency(&self.semantic_dependencies, owner);
                }
                DispatchResult::Missing
            }
            crate::dispatch::ResolvedDispatchResult::Dynamic => DispatchResult::Dynamic,
        }
    }

    pub fn register_surface(&mut self, decl: DeclarationId, surface: crate::surface::DeclarationSurface) {
        self.dispatch.make_mut().register_surface(decl, surface);
    }

    pub fn get_surface(&self, decl: &DeclarationId) -> Option<&crate::surface::DeclarationSurface> {
        record_declaration_surface_dependency(&self.semantic_dependencies, decl);
        self.dispatch.get().get_surface(decl)
    }

    pub fn get_field(&self, decl: &DeclarationId, side: DispatchSide, name: &str) -> Option<TypeKnowledge> {
        record_declaration_surface_dependency(&self.semantic_dependencies, decl);
        self.dispatch.get().get_surface(decl).and_then(|s| s.get_field(side, name)).cloned()
    }

    pub fn resolve_type_name(&self, name: &str) -> Option<DeclarationId> {
        self.resolver.resolve_type_name(&self.current_module, name, &[])
    }

    pub fn nominal_type_of(&mut self, decl: &DeclarationId) -> TypeId {
        record_declaration_shell_dependency(&self.semantic_dependencies, decl);
        if let Some(form) = self.declarations.form(decl) {
            form
        } else {
            self.store.nominal_type(decl.clone())
        }
    }

    pub fn record_derivation(
        &mut self,
        step: crate::explain::ExplanationStep,
        rule: crate::explain::DerivationRule,
        authority: crate::types::evidence::EvidenceAuthority,
        evidence: Vec<crate::explain::EvidenceRef>,
        parents: Vec<crate::identity::ExplanationId>,
    ) -> crate::identity::ExplanationId {
        self.explanations.alloc_full(step, rule, authority, evidence, parents)
    }

    pub fn suppression_cause(&self, id: ExpressionId) -> Option<crate::identity::DiagnosticCauseId> {
        self.suppressed.get(&id).copied()
    }

    pub fn mark_suppressed(&mut self, id: ExpressionId, cause: crate::identity::DiagnosticCauseId) {
        self.suppressed.insert(id, cause);
    }

    pub fn finalize(
        self,
        callable: CallableId,
        body_range: SourceRange,
        status: crate::checker::analysis::CallableAnalysisStatus,
    ) -> crate::checker::analysis::CallableAnalysis {
        let flow_graph = self
            .flow_graph
            .unwrap_or_else(|| std::sync::Arc::new(crate::checker::flow::graph::FlowGraph::default()));

        let mut known_bindings = std::collections::BTreeMap::new();
        for (b_id, state) in &self.bindings {
            if let Some(ty) = state.current.ty() {
                known_bindings.insert(*b_id, ty);
            }
        }
        let entry_flow = crate::checker::analysis::FlowStateSummary {
            known_bindings,
            fact_count: self.flow.facts.len(),
        };

        let exits = crate::checker::analysis::BodyExitFacts {
            returns: vec![entry_flow.clone()],
            throws: Vec::new(),
            unreachable: false,
        };

        crate::checker::analysis::CallableAnalysis {
            callable,
            body_range,
            expressions: self.expressions,
            bindings: self.bindings,
            flow_graph,
            entry_flow,
            exits,
            diagnostics: std::sync::Arc::from(self.diagnostics.into_boxed_slice()),
            explanations: std::sync::Arc::new(self.explanations),
            dependencies: std::sync::Arc::from(self.dependencies.into_iter().collect::<Vec<_>>().into_boxed_slice()),
            semantic_dependencies: std::sync::Arc::from(
                self.semantic_dependencies
                    .borrow()
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            dependency_fingerprint: crate::db::ProductFingerprint::new(0),
            status,
        }
    }
}
