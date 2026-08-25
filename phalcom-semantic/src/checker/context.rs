use crate::checker::analysis::{AnalysisStatus, ExpressionAnalysis, ExpressionAnalysisIndex, SemanticDependency};
use crate::checker::binding::{BindingContract, BindingContractOrigin, BindingDeclarationResult, BindingSeed, BindingWriteResult, reconcile_binding_contract};
use crate::checker::flow::FlowState;
use crate::declarations::{DeclarationTypeInfo, DeclarationTypeTable};
use crate::diagnostic::SemanticDiagnostic;
use crate::dispatch::{DispatchResult, SurfaceDispatchResolver};
use crate::identity::{BindingId, BodyId, CallableId, DeclarationId, DispatchSide, ExpressionId, LocalExpressionId, ModuleId};
use crate::types::annotation::TypeResolver;
use crate::types::denotation::{SemanticDenotation, ValueSemanticFact};
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge};
use crate::types::id::TypeId;
use crate::types::native::register_native_surfaces;
use crate::types::relation::{Assignability, TypeHierarchy, check_assignability};
use crate::types::store::{TypeData, TypeStore};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_native_surface::NATIVE_SURFACES;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
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
                record_query_dependency(&self.dependencies, SemanticDependency::LinkedInterface(current_module.clone()));
            }
            return None;
        };

        record_declaration_shell_dependency(&self.dependencies, &declaration);
        if &declaration.module != current_module && is_query_owned_module(current_module) && is_query_owned_module(&declaration.module) {
            record_query_dependency(&self.dependencies, SemanticDependency::LinkedInterface(current_module.clone()));
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

#[derive(Clone, Default)]
struct CallDependencyFrame {
    causal_invalidity: crate::checker::causal::CausalInvalidity,
    explanations: Vec<crate::identity::ExplanationId>,
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
    pub scopes: Vec<HashMap<String, LocalBindingInfo>>,
    pub flow: FlowState,
    pub body_id: BodyId,
    pub next_local_expr_id: u32,
    pub next_diagnostic_cause: u32,
    pub next_binding_id: u32,
    pub expressions: ExpressionAnalysisIndex,
    pub explanations: crate::explain::ExplanationArena,
    pub suppressed: std::collections::BTreeMap<ExpressionId, crate::identity::DiagnosticCauseId>,
    expression_owners: Vec<ExpressionId>,
    expression_owned_causes: BTreeMap<ExpressionId, crate::identity::DiagnosticCauseId>,
    resolved_callables: BTreeMap<ExpressionId, CallableId>,
    call_dependency_frames: Vec<CallDependencyFrame>,
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
            scopes: vec![HashMap::new()],
            flow: FlowState::new(),
            body_id: BodyId(0),
            next_local_expr_id: 0,
            next_diagnostic_cause: 0,
            next_binding_id: 0,
            expressions: ExpressionAnalysisIndex::new(),
            explanations: crate::explain::ExplanationArena::new(),
            suppressed: std::collections::BTreeMap::new(),
            expression_owners: Vec::new(),
            expression_owned_causes: BTreeMap::new(),
            resolved_callables: BTreeMap::new(),
            call_dependency_frames: Vec::new(),
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
            scopes: vec![HashMap::new()],
            flow: FlowState::new(),
            body_id: BodyId(0),
            next_local_expr_id: 0,
            next_diagnostic_cause: 0,
            next_binding_id: 0,
            expressions: ExpressionAnalysisIndex::new(),
            explanations: crate::explain::ExplanationArena::new(),
            suppressed: std::collections::BTreeMap::new(),
            expression_owners: Vec::new(),
            expression_owned_causes: BTreeMap::new(),
            resolved_callables: BTreeMap::new(),
            call_dependency_frames: Vec::new(),
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
            scopes: self.scopes.clone(),
            flow: self.flow.clone(),
            body_id: self.body_id,
            next_local_expr_id: self.next_local_expr_id,
            next_diagnostic_cause: self.next_diagnostic_cause,
            next_binding_id: self.next_binding_id,
            expressions: self.expressions.clone(),
            explanations: self.explanations.clone(),
            suppressed: self.suppressed.clone(),
            expression_owners: self.expression_owners.clone(),
            expression_owned_causes: self.expression_owned_causes.clone(),
            resolved_callables: self.resolved_callables.clone(),
            call_dependency_frames: self.call_dependency_frames.clone(),
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

    /// Allocates a monotonic diagnostic cause independent of expression IDs.
    pub fn alloc_diagnostic_cause(&mut self) -> crate::identity::DiagnosticCauseId {
        let cause = crate::identity::DiagnosticCauseId(self.next_diagnostic_cause);
        self.next_diagnostic_cause = self.next_diagnostic_cause.saturating_add(1);
        cause
    }

    pub fn push_expression_owner(&mut self, id: ExpressionId) {
        self.expression_owners.push(id);
    }

    pub fn current_expression_id(&self) -> Option<ExpressionId> {
        self.expression_owners.last().copied()
    }

    pub fn resolved_callable_for_current_expression(&self) -> Option<CallableId> {
        self.current_expression_id().and_then(|id| self.resolved_callables.get(&id).cloned())
    }

    pub(crate) fn begin_call_causal_capture(&mut self) {
        self.call_dependency_frames.push(CallDependencyFrame::default());
    }

    pub(crate) fn record_call_dependency(
        &mut self,
        causal_invalidity: crate::checker::causal::CausalInvalidity,
        explanation: Option<crate::identity::ExplanationId>,
    ) {
        if let Some(frame) = self.call_dependency_frames.last_mut() {
            frame.causal_invalidity = frame.causal_invalidity.join(causal_invalidity);
            if let Some(explanation) = explanation {
                if !frame.explanations.contains(&explanation) {
                    frame.explanations.push(explanation);
                }
            }
        }
    }

    pub(crate) fn end_call_causal_capture(&mut self) -> (crate::checker::causal::CausalInvalidity, Vec<crate::identity::ExplanationId>) {
        let frame = self.call_dependency_frames.pop().unwrap_or_default();
        (frame.causal_invalidity, frame.explanations)
    }

    pub(crate) fn explanation_for_expression(&self, id: ExpressionId) -> Option<crate::identity::ExplanationId> {
        self.expressions.get(&id).and_then(|analysis| analysis.explanation)
    }

    pub fn pop_expression_owner(&mut self, id: ExpressionId) -> Option<crate::identity::DiagnosticCauseId> {
        debug_assert_eq!(self.expression_owners.pop(), Some(id));
        self.expression_owned_causes.remove(&id)
    }

    /// Emits one diagnostic and records its owning expression without using
    /// source ranges as a proxy for semantic ownership.
    pub fn emit_diagnostic(&mut self, mut diagnostic: SemanticDiagnostic) -> Option<crate::identity::DiagnosticCauseId> {
        if diagnostic.severity != crate::diagnostic::DiagnosticSeverity::Error {
            self.diagnostics.push(diagnostic);
            return None;
        }
        let cause = diagnostic.root_cause.unwrap_or_else(|| self.alloc_diagnostic_cause());
        diagnostic.root_cause = Some(cause);
        if let Some(owner) = self.expression_owners.last().copied() {
            self.expression_owned_causes.entry(owner).or_insert(cause);
        }
        self.diagnostics.push(diagnostic);
        Some(cause)
    }

    /// Publishes diagnostics produced by a resolver while retaining every
    /// owning error cause in the bounded causal domain. Resolver APIs return a
    /// vector because one annotation may contain several invalid relations;
    /// callers must not append that vector directly to the context.
    pub fn publish_diagnostics(&mut self, diagnostics: impl IntoIterator<Item = SemanticDiagnostic>) -> crate::checker::causal::CausalInvalidity {
        diagnostics
            .into_iter()
            .filter_map(|diagnostic| self.emit_diagnostic(diagnostic))
            .map(crate::checker::causal::CausalInvalidity::One)
            .fold(crate::checker::causal::CausalInvalidity::Clean, crate::checker::causal::CausalInvalidity::join)
    }

    /// Resolves one source annotation and publishes all diagnostics under the
    /// current checker ownership frame.
    pub fn resolve_type_annotation(
        &mut self,
        resolver: &dyn TypeResolver,
        annotation: &phalcom_ast::ast::TypeAnnotation,
    ) -> (TypeKnowledge, crate::checker::causal::CausalInvalidity) {
        let mut diagnostics = Vec::new();
        let knowledge =
            crate::types::annotation::resolve_type_annotation(self.store, self.declarations, resolver, &self.current_module, annotation, &mut diagnostics);
        let causal_invalidity = self.publish_diagnostics(diagnostics);
        (knowledge, causal_invalidity)
    }

    pub fn enforce_assignability(
        &mut self,
        actual: &TypeKnowledge,
        expected: &TypeKnowledge,
        code: crate::diagnostic::DiagnosticCode,
        message: impl Into<String>,
        range: SourceRange,
    ) -> bool {
        match check_assignability(self.store, &self.hierarchy, actual, expected) {
            Assignability::Refuted { .. } => {
                self.emit_diagnostic(SemanticDiagnostic::error_in(self.current_module.clone(), code, message, range));
                false
            }
            _ => true,
        }
    }

    pub fn enforce_knowledge_against_type(
        &mut self,
        actual: &TypeKnowledge,
        expected: TypeId,
        code: crate::diagnostic::DiagnosticCode,
        message: impl Into<String>,
        range: SourceRange,
    ) -> bool {
        match crate::types::relation::check_knowledge_against_type(self.store, &self.hierarchy, actual, expected) {
            Assignability::Refuted { .. } => {
                self.emit_diagnostic(SemanticDiagnostic::error_in(self.current_module.clone(), code, message, range));
                false
            }
            _ => true,
        }
    }

    pub fn enforce_knowledge_against_type_owned(
        &mut self,
        actual: &TypeKnowledge,
        expected: TypeId,
        code: crate::diagnostic::DiagnosticCode,
        message: impl Into<String>,
        range: SourceRange,
        owner: ExpressionId,
    ) -> bool {
        match crate::types::relation::check_knowledge_against_type(self.store, &self.hierarchy, actual, expected) {
            Assignability::Refuted { .. } => {
                let cause = self.emit_diagnostic(SemanticDiagnostic::error_in(self.current_module.clone(), code, message, range));
                if let Some(cause) = cause {
                    self.expression_owned_causes.entry(owner).or_insert(cause);
                    if let Some(analysis) = self.expressions.get_mut(&owner) {
                        analysis.status = AnalysisStatus::Invalid(cause);
                        analysis.causal_invalidity = analysis.causal_invalidity.join(crate::checker::causal::CausalInvalidity::One(cause));
                    }
                }
                false
            }
            _ => true,
        }
    }

    pub fn record_expression(
        &mut self,
        id: ExpressionId,
        range: SourceRange,
        knowledge: TypeKnowledge,
        callable: Option<CallableId>,
        denotation: Option<SemanticDenotation>,
        status: AnalysisStatus,
    ) -> ExpressionAnalysis {
        let mut analysis = ExpressionAnalysis::ready(id, range, knowledge);
        analysis.callable = callable;
        analysis.denotation = denotation;
        analysis.status = status;
        self.expressions.insert(id, analysis.clone());
        analysis
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Inserts one explicit binding seed, retaining first declaration identity
    /// when same-scope redeclaration is attempted.
    pub fn declare_binding(&mut self, seed: BindingSeed) -> BindingDeclarationResult {
        let Some(scope) = self.scopes.last() else {
            return BindingDeclarationResult::Redeclared(BindingId(u32::MAX));
        };
        if let Some(existing) = scope.get(&seed.name).cloned() {
            let mut diagnostic = SemanticDiagnostic::error_in(
                self.current_module.clone(),
                crate::diagnostic::DiagnosticCode::BindingRedeclared,
                format!("binding `{}` is already declared in this scope", seed.name),
                seed.range,
            );
            if let Some(previous) = self.flow.get_binding(existing.id) {
                diagnostic = diagnostic.with_label(previous.range, "first declaration");
            }
            self.emit_diagnostic(diagnostic);
            return BindingDeclarationResult::Redeclared(existing.id);
        }

        let binding_id = self.alloc_binding();
        let reconciliation = reconcile_binding_contract(self.store, &self.hierarchy, seed.contract.as_ref(), &seed.current);
        let declared = seed.contract.as_ref().map(|contract| contract.ty);
        let denotation = seed.denotation;
        let name = seed.name.clone();
        let contract_explanation = seed.contract.as_ref().map(|contract| {
            let actual = reconciliation.current.clone();
            let status = actual.status().unwrap_or(crate::types::evidence::EvidenceStatus::Assumed);
            let origin = actual.origin().unwrap_or(crate::types::evidence::EvidenceOrigin::DeveloperAnnotation);
            let mut evidence = Vec::new();
            if let Some(source) = contract.source {
                evidence.push(crate::explain::EvidenceRef::SourceSpan(source));
            }
            evidence.push(crate::explain::EvidenceRef::TypeId(contract.ty));
            if let Some(actual_ty) = actual.ty() {
                evidence.push(crate::explain::EvidenceRef::TypeId(actual_ty));
            }
            self.record_derivation(
                crate::explain::ExplanationStep::BindingContract {
                    binding: binding_id,
                    actual,
                    contract: contract.ty,
                    consistency: reconciliation.consistency.clone(),
                },
                crate::explain::DerivationRule::BindingContract,
                status,
                origin,
                evidence,
                Vec::new(),
            )
        });
        self.flow.declare_seed(binding_id, seed, reconciliation.current, reconciliation.consistency);
        if let Some(explanation) = contract_explanation {
            self.flow.set_binding_explanation(binding_id, explanation);
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(
                name,
                LocalBindingInfo {
                    id: binding_id,
                    declared,
                    denotation,
                },
            );
        }
        BindingDeclarationResult::Inserted(binding_id)
    }

    pub fn bind_callable_parameter(&mut self, name: impl Into<String>, current: TypeKnowledge, range: SourceRange) -> BindingDeclarationResult {
        self.bind_callable_parameter_with_causal(name, current, range, crate::checker::causal::CausalInvalidity::Clean)
    }

    pub fn bind_callable_parameter_with_causal(
        &mut self,
        name: impl Into<String>,
        current: TypeKnowledge,
        range: SourceRange,
        causal_invalidity: crate::checker::causal::CausalInvalidity,
    ) -> BindingDeclarationResult {
        let contract = current.ty().map(|ty| BindingContract {
            ty,
            origin: BindingContractOrigin::CallableParameter,
            source: Some(range),
        });
        self.declare_binding(BindingSeed {
            name: name.into(),
            range,
            contract,
            current,
            denotation: None,
            causal_invalidity,
            mutable: false,
        })
    }

    pub fn bind_contextual_block_parameter(&mut self, name: impl Into<String>, ty: TypeId, range: SourceRange) -> BindingDeclarationResult {
        self.declare_binding(BindingSeed {
            name: name.into(),
            range,
            contract: Some(BindingContract {
                ty,
                origin: BindingContractOrigin::ContextualBlockParameter,
                source: Some(range),
            }),
            current: TypeKnowledge::assumed(ty, EvidenceOrigin::ContextualDerivation),
            denotation: None,
            causal_invalidity: crate::checker::causal::CausalInvalidity::Clean,
            mutable: false,
        })
    }

    pub fn bind_untyped_block_parameter(&mut self, name: impl Into<String>, range: SourceRange) -> BindingDeclarationResult {
        self.declare_binding(BindingSeed {
            name: name.into(),
            range,
            contract: None,
            current: TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::NoTypeEvidence),
            denotation: None,
            causal_invalidity: crate::checker::causal::CausalInvalidity::Clean,
            mutable: false,
        })
    }

    pub fn bind_pattern_binding(&mut self, name: impl Into<String>, fact: ValueSemanticFact, range: SourceRange) -> BindingDeclarationResult {
        let contract = fact.knowledge.ty().map(|ty| BindingContract {
            ty,
            origin: BindingContractOrigin::PatternBinding,
            source: Some(range),
        });
        self.declare_binding(BindingSeed {
            name: name.into(),
            range,
            contract,
            current: fact.knowledge,
            denotation: fact.denotation,
            causal_invalidity: crate::checker::causal::CausalInvalidity::Clean,
            mutable: true,
        })
    }

    pub fn lookup_binding_info(&self, name: &str) -> Option<&LocalBindingInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    pub fn lookup_local(&self, name: &str) -> Option<ValueSemanticFact> {
        let info = self.lookup_binding_info(name)?;
        let state = self.flow.get_binding(info.id)?;
        Some(ValueSemanticFact {
            knowledge: state.current.clone(),
            denotation: state.denotation,
        })
    }

    pub fn lookup_local_knowledge(&self, name: &str) -> Option<TypeKnowledge> {
        if let Some(info) = self.lookup_binding_info(name) {
            if let Some(k) = self.flow.get_current_type(info.id) {
                return Some(k.clone());
            }
        }
        self.lookup_local(name).map(|f| f.knowledge)
    }

    pub fn write_existing(
        &mut self,
        name: &str,
        fact: ValueSemanticFact,
        consistency: crate::checker::binding::BindingConsistency,
        causal_invalidity: crate::checker::causal::CausalInvalidity,
    ) -> BindingWriteResult {
        let Some(info) = self.lookup_binding_info(name).cloned() else {
            return BindingWriteResult::Missing;
        };
        let result = self.flow.write(info.id, fact.knowledge, fact.denotation, consistency, causal_invalidity);
        result
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
    pub(crate) fn record_consumed_callable_signature(&self, callable: &CallableId, signature: &crate::dispatch::CallableSignature) {
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
    pub fn declaration_generic_signature(&self, declaration: &DeclarationId) -> Option<crate::types::parameter::GenericSignature> {
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
                if let Some(expression) = self.current_expression_id() {
                    self.resolved_callables.insert(expression, resolved.callable.clone());
                }
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
        status: crate::types::evidence::EvidenceStatus,
        origin: crate::types::evidence::EvidenceOrigin,
        evidence: Vec<crate::explain::EvidenceRef>,
        parents: Vec<crate::identity::ExplanationId>,
    ) -> crate::identity::ExplanationId {
        self.explanations.alloc_full(step, rule, status, origin, evidence, parents)
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
        self.finalize_with_normal_returns(callable, body_range, status, Vec::new())
    }

    pub fn finalize_with_normal_returns(
        self,
        callable: CallableId,
        body_range: SourceRange,
        status: crate::checker::analysis::CallableAnalysisStatus,
        normal_return_values: Vec<crate::types::evidence::TypeKnowledge>,
    ) -> crate::checker::analysis::CallableAnalysis {
        let flow_graph = self
            .flow_graph
            .unwrap_or_else(|| std::sync::Arc::new(crate::checker::flow::graph::FlowGraph::default()));

        let mut known_bindings = std::collections::BTreeMap::new();
        for (b_id, state) in &self.flow.bindings {
            if let Some(ty) = state.current.ty() {
                known_bindings.insert(*b_id, ty);
            }
        }
        let entry_flow = crate::checker::analysis::FlowStateSummary {
            known_bindings,
            fact_count: self.flow.facts.len(),
        };

        let exits = crate::checker::analysis::BodyExitFacts {
            returns: if normal_return_values.is_empty() {
                Vec::new()
            } else {
                vec![entry_flow.clone()]
            },
            normal_return_values,
            throws: Vec::new(),
            unreachable: false,
        };

        crate::checker::analysis::CallableAnalysis {
            callable,
            body_range,
            expressions: self.expressions,
            bindings: self.flow.bindings,
            flow_graph,
            entry_flow,
            exits,
            diagnostics: std::sync::Arc::from(self.diagnostics.into_boxed_slice()),
            explanations: std::sync::Arc::new(self.explanations),
            dependencies: std::sync::Arc::from(self.dependencies.into_iter().collect::<Vec<_>>().into_boxed_slice()),
            semantic_dependencies: std::sync::Arc::from(self.semantic_dependencies.borrow().iter().cloned().collect::<Vec<_>>().into_boxed_slice()),
            dependency_fingerprint: crate::db::ProductFingerprint::new(0),
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CheckingContext;
    use crate::checker::causal::CausalInvalidity;
    use crate::declarations::bootstrap_universe_declarations;
    use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
    use crate::identity::{DeclarationId, ModuleId};
    use crate::types::SimpleTypeResolver;
    use crate::types::relation::MapTypeHierarchy;
    use crate::types::store::TypeStore;
    use phalcom_common::range::SourceRange;

    #[test]
    fn published_annotation_diagnostics_join_all_error_causes() {
        let module = ModuleId::core();
        let mut store = TypeStore::new();
        let declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(module.clone(), key.name().into()));
        let resolver = SimpleTypeResolver::new();
        let hierarchy = MapTypeHierarchy::new();
        let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

        let diagnostics = [
            SemanticDiagnostic::error_in(module.clone(), DiagnosticCode::AnnotationUnresolved, "first", SourceRange { start: 0, end: 1 }),
            SemanticDiagnostic::error_in(module, DiagnosticCode::AnnotationUnsupported, "second", SourceRange { start: 2, end: 3 }),
        ];
        let causal = ctx.publish_diagnostics(diagnostics);

        assert_eq!(causal, CausalInvalidity::Multiple);
        assert_eq!(ctx.diagnostics.len(), 2);
        assert!(ctx.diagnostics.iter().all(|diagnostic| diagnostic.root_cause.is_some()));
        assert_ne!(ctx.diagnostics[0].root_cause, ctx.diagnostics[1].root_cause);
    }
}
