use crate::checker::analysis::{AnalysisStatus, BindingAnalysisIndex, ExpressionAnalysis, ExpressionAnalysisIndex};
use crate::checker::flow::FlowState;
use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::SemanticDiagnostic;
use crate::dispatch::{DispatchResult, SurfaceDispatchResolver};
use crate::identity::{BindingId, BodyId, CallableId, DeclarationId, DispatchSide, ExpressionId, LocalExpressionId, ModuleId};
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
use std::collections::HashMap;

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
    pub hierarchy: &'a dyn TypeHierarchy,
    pub resolver: &'a dyn TypeResolver,
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
    pub dependencies: Vec<CallableId>,
    pub dispatch: SurfaceDispatchResolver,
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
        Self {
            store,
            hierarchy,
            resolver,
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
            dependencies: Vec::new(),
            dispatch,
            diagnostics: Vec::new(),
        }
    }

    pub fn with_resolver<'b>(&'b mut self, resolver: &'b dyn TypeResolver) -> CheckingContext<'b> {
        CheckingContext {
            store: self.store,
            hierarchy: self.hierarchy,
            resolver,
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
            dispatch: self.dispatch.clone(),
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

        let res = self.dispatch.resolve_dispatch_on_owner(self.hierarchy, &decl, side, selector);
        if let DispatchResult::Found(mut sig) = res {
            if let Some(subst) = crate::types::substitution::substitution_for_applied(self.declarations, self.store, receiver) {
                for param in &mut sig.parameters {
                    if let TypeKnowledge::Known(ref mut ev) = param.ty {
                        ev.ty = subst.apply(self.store, ev.ty);
                    }
                }
                if let TypeKnowledge::Known(ref mut ev) = sig.return_type {
                    ev.ty = subst.apply(self.store, ev.ty);
                }
            }
            DispatchResult::Found(sig)
        } else {
            res
        }
    }

    pub fn register_surface(&mut self, decl: DeclarationId, surface: crate::surface::DeclarationSurface) {
        self.dispatch.register_surface(decl, surface);
    }

    pub fn nominal_type_of(&mut self, decl: &DeclarationId) -> TypeId {
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
            dependencies: std::sync::Arc::from(self.dependencies.into_boxed_slice()),
            dependency_fingerprint: crate::db::ProductFingerprint::new(0),
            status,
        }
    }
}
