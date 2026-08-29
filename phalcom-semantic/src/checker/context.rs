use crate::checker::analysis::{AnalysisStatus, ExpressionAnalysis, ExpressionAnalysisIndex, SemanticDependency};
use crate::checker::binding::{BindingContract, BindingContractOrigin, BindingDeclarationResult, BindingSeed, BindingWriteResult, reconcile_binding_relation};
use crate::checker::flow::FlowState;
use crate::checker::incident::{
    BindingContractSummary, InternalFailurePolicy, InternalSemanticIncident, InternalSemanticIncidentDetails, InternalSemanticIncidentKind,
};
use crate::db::budget::{BudgetReport, CancellationToken, QueryBudget};
use crate::declarations::{DeclarationTypeInfo, DeclarationTypeTable};
use crate::diagnostic::SemanticDiagnostic;
use crate::dispatch::{CallableParameter, CallableSignature, DispatchResult, ResolvedDispatchResult, SurfaceDispatchResolver};
use crate::identity::{
    AnalysisIncidentId, BindingId, BodyId, CallableId, DeclarationId, DiagnosticCauseId, DispatchSide, ExpressionId, LocalExpressionId, ModuleId,
};
use crate::signature::FieldSignatureTable;
use crate::surface::DeclarationSurface;
use crate::types::annotation::TypeResolver;
use crate::types::denotation::{SemanticDenotation, ValueSemanticFact};
use crate::types::evidence::{ContractAssumptionEligibility, EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::native::register_native_surfaces;
use crate::types::outcome::{DynamicBoundaryObligation, RelationOutcome};
use crate::types::relation::{TypeHierarchy, check_assignability_bounded, check_knowledge_against_type_bounded};
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

/// Shared control for one callable/query semantic analysis.
///
/// Relation checks and statement transfer consume the same budget and
/// cancellation token. This prevents nested relation calls from silently
/// resetting query limits or escaping cancellation.
#[derive(Clone)]
pub struct CheckerControl {
    budget: Rc<RefCell<QueryBudget>>,
    cancellation: CancellationToken,
}

impl Default for CheckerControl {
    fn default() -> Self {
        Self::new(QueryBudget::default(), &CancellationToken::new())
    }
}

impl CheckerControl {
    pub fn new(budget: QueryBudget, cancellation: &CancellationToken) -> Self {
        Self {
            budget: Rc::new(RefCell::new(budget)),
            cancellation: cancellation.clone(),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    pub fn charge_step(&self) -> Result<(), BudgetReport> {
        self.budget.borrow_mut().charge_step()
    }

    pub fn relation<R>(&self, f: impl FnOnce(&mut QueryBudget, &CancellationToken) -> R) -> R {
        let mut budget = self.budget.borrow_mut();
        f(&mut budget, &self.cancellation)
    }
}

#[derive(Clone, Default)]
pub(crate) struct LoopFlowFrame {
    pub continues: Vec<FlowState>,
    pub breaks: Vec<FlowState>,
}

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
    pub denotation: Option<SemanticDenotation>,
}

#[derive(Clone, Default)]
struct CallDependencyFrame {
    causal_invalidity: crate::checker::causal::CausalInvalidity,
    explanations: Vec<crate::identity::ExplanationId>,
    status: Option<AnalysisStatus>,
}

/// Declared callable return contract. This is checking context, not value
/// evidence; return expressions are judged against its type and provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableReturnContract {
    pub ty: TypeId,
    pub origin: crate::types::evidence::EvidenceOrigin,
    pub source: Option<SourceRange>,
}

/// Structured result of applying one bounded relation at a checker boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationApplication {
    pub outcome: RelationOutcome<()>,
    pub cause: Option<DiagnosticCauseId>,
    pub status: Option<AnalysisStatus>,
    pub explanation: Option<crate::identity::ExplanationId>,
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
    pub current_callable: Option<CallableId>,
    pub internal_failure_policy: InternalFailurePolicy,
    pub control: CheckerControl,
    pub expected_return: Option<CallableReturnContract>,
    pub scopes: Vec<HashMap<String, LocalBindingInfo>>,
    pub flow: FlowState,
    throw_exit_flows: Vec<crate::checker::analysis::FlowStateSummary>,
    /// Binding products remain queryable after a branch-local scope closes.
    pub binding_history: BTreeMap<BindingId, crate::checker::analysis::BindingState>,
    pub(crate) loop_frames: Vec<LoopFlowFrame>,
    pub body_id: BodyId,
    pub next_local_expr_id: u32,
    pub next_diagnostic_cause: u32,
    pub next_analysis_incident: u32,
    pub next_binding_id: u32,
    pub expressions: ExpressionAnalysisIndex,
    pub explanations: crate::explain::ExplanationArena,
    expression_owners: Vec<ExpressionId>,
    expression_owned_causes: BTreeMap<ExpressionId, crate::identity::DiagnosticCauseId>,
    resolved_callables: BTreeMap<ExpressionId, CallableId>,
    call_dependency_frames: Vec<CallDependencyFrame>,
    pub flow_graph: Option<std::sync::Arc<crate::checker::flow::graph::FlowGraph>>,
    pub dependencies: BTreeSet<CallableId>,
    semantic_dependencies: SharedSemanticDependencies,
    field_signatures: Option<&'a FieldSignatureTable>,
    pub dispatch: DispatchAccess<'a>,
    pub diagnostics: Vec<SemanticDiagnostic>,
    pub analysis_incidents: BTreeMap<AnalysisIncidentId, InternalSemanticIncident>,
    pub terminal_status: Option<AnalysisStatus>,
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
        ensure_core_object_type_tests(store, declarations, &mut dispatch);

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
        let mut dispatch = dispatch;
        ensure_core_object_type_tests(store, declarations, &mut dispatch);
        let semantic_dependencies = Rc::new(RefCell::new(BTreeSet::new()));
        Self {
            store,
            hierarchy: TrackingTypeHierarchy::new(hierarchy, semantic_dependencies.clone()),
            resolver: TrackingTypeResolver::new(resolver, semantic_dependencies.clone()),
            declarations,
            current_module,
            current_class: None,
            current_side: DispatchSide::Instance,
            current_callable: None,
            internal_failure_policy: InternalFailurePolicy::Contain,
            control: CheckerControl::default(),
            expected_return: None,
            scopes: vec![HashMap::new()],
            flow: FlowState::new(),
            throw_exit_flows: Vec::new(),
            binding_history: BTreeMap::new(),
            loop_frames: Vec::new(),
            body_id: BodyId(0),
            next_local_expr_id: 0,
            next_diagnostic_cause: 0,
            next_analysis_incident: 0,
            next_binding_id: 0,
            expressions: ExpressionAnalysisIndex::new(),
            explanations: crate::explain::ExplanationArena::new(),
            expression_owners: Vec::new(),
            expression_owned_causes: BTreeMap::new(),
            resolved_callables: BTreeMap::new(),
            call_dependency_frames: Vec::new(),
            flow_graph: None,
            dependencies: BTreeSet::new(),
            semantic_dependencies,
            field_signatures: None,
            dispatch: DispatchAccess::Owned(dispatch),
            diagnostics: Vec::new(),
            analysis_incidents: BTreeMap::new(),
            terminal_status: None,
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
        Self::new_with_dispatch_ref_and_control(store, hierarchy, resolver, declarations, dispatch, current_module, CheckerControl::default())
    }

    pub fn new_with_dispatch_ref_and_control(
        store: &'a mut TypeStore,
        hierarchy: &'a dyn TypeHierarchy,
        resolver: &'a dyn TypeResolver,
        declarations: &'a DeclarationTypeTable,
        dispatch: &'a SurfaceDispatchResolver,
        current_module: ModuleId,
        control: CheckerControl,
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
            current_callable: None,
            internal_failure_policy: InternalFailurePolicy::Contain,
            control,
            expected_return: None,
            scopes: vec![HashMap::new()],
            flow: FlowState::new(),
            throw_exit_flows: Vec::new(),
            binding_history: BTreeMap::new(),
            loop_frames: Vec::new(),
            body_id: BodyId(0),
            next_local_expr_id: 0,
            next_diagnostic_cause: 0,
            next_analysis_incident: 0,
            next_binding_id: 0,
            expressions: ExpressionAnalysisIndex::new(),
            explanations: crate::explain::ExplanationArena::new(),
            expression_owners: Vec::new(),
            expression_owned_causes: BTreeMap::new(),
            resolved_callables: BTreeMap::new(),
            call_dependency_frames: Vec::new(),
            flow_graph: None,
            dependencies: BTreeSet::new(),
            semantic_dependencies,
            field_signatures: None,
            dispatch: DispatchAccess::Borrowed(dispatch),
            diagnostics: Vec::new(),
            analysis_incidents: BTreeMap::new(),
            terminal_status: None,
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
            current_callable: self.current_callable.clone(),
            internal_failure_policy: self.internal_failure_policy,
            control: self.control.clone(),
            expected_return: self.expected_return.clone(),
            scopes: self.scopes.clone(),
            flow: self.flow.clone(),
            throw_exit_flows: self.throw_exit_flows.clone(),
            binding_history: self.binding_history.clone(),
            loop_frames: self.loop_frames.clone(),
            body_id: self.body_id,
            next_local_expr_id: self.next_local_expr_id,
            next_diagnostic_cause: self.next_diagnostic_cause,
            next_analysis_incident: self.next_analysis_incident,
            next_binding_id: self.next_binding_id,
            expressions: self.expressions.clone(),
            explanations: self.explanations.clone(),
            expression_owners: self.expression_owners.clone(),
            expression_owned_causes: self.expression_owned_causes.clone(),
            resolved_callables: self.resolved_callables.clone(),
            call_dependency_frames: self.call_dependency_frames.clone(),
            flow_graph: self.flow_graph.clone(),
            dependencies: self.dependencies.clone(),
            semantic_dependencies: self.semantic_dependencies.clone(),
            field_signatures: self.field_signatures,
            dispatch: DispatchAccess::Borrowed(self.dispatch.get()),
            diagnostics: Vec::new(),
            analysis_incidents: self.analysis_incidents.clone(),
            terminal_status: self.terminal_status.clone(),
        }
    }

    pub fn alloc_binding(&mut self) -> BindingId {
        let id = self.next_binding_id;
        self.next_binding_id += 1;
        BindingId(id)
    }

    pub fn charge_step(&self) -> Result<(), BudgetReport> {
        self.control.charge_step()
    }

    pub fn is_cancelled(&self) -> bool {
        self.control.is_cancelled()
    }

    fn current_flow_summary(&self) -> crate::checker::analysis::FlowStateSummary {
        let bindings = self
            .flow
            .bindings
            .iter()
            .map(|(binding, state)| {
                (
                    *binding,
                    crate::checker::analysis::FlowBindingSummary {
                        knowledge: state.current.clone(),
                        contract: state.contract.clone(),
                        consistency: state.consistency.clone(),
                        mutable: state.mutable,
                    },
                )
            })
            .collect();
        crate::checker::analysis::FlowStateSummary {
            bindings,
            fields: self
                .flow
                .fields
                .iter()
                .map(|(field, state)| {
                    (
                        field.clone(),
                        crate::checker::analysis::FlowFieldSummary {
                            contract: state.contract.clone(),
                            current: state.current.clone(),
                            initialization: state.initialization,
                        },
                    )
                })
                .collect(),
            fact_count: self.flow.facts.len(),
        }
    }

    pub(crate) fn record_throw_exit(&mut self) {
        self.throw_exit_flows.push(self.current_flow_summary());
    }

    pub(crate) fn solve_inference(&mut self, session: &mut crate::checker::inference::InferenceSession) -> crate::checker::inference::InferenceOutcome {
        session.solve_with_control(self.store, &self.hierarchy, &self.control)
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

    pub(crate) fn owning_cause_for_current_expression(&self) -> Option<DiagnosticCauseId> {
        self.current_expression_id().and_then(|id| self.expression_owned_causes.get(&id).copied())
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

    pub(crate) fn record_call_status(&mut self, status: AnalysisStatus) {
        if matches!(status, AnalysisStatus::Ready) {
            return;
        }
        if let Some(frame) = self.call_dependency_frames.last_mut() {
            frame.status = Some(status);
        } else {
            self.record_terminal_status(status);
        }
    }

    pub(crate) fn call_status_is_recorded(&self) -> bool {
        self.call_dependency_frames.last().is_some_and(|frame| frame.status.is_some())
    }

    pub(crate) fn record_terminal_status(&mut self, status: AnalysisStatus) {
        if matches!(status, AnalysisStatus::Ready) {
            return;
        }
        if matches!(status, AnalysisStatus::InternalFailure(_)) || self.terminal_status.is_none() {
            self.terminal_status = Some(status);
        }
    }

    pub(crate) fn apply_flow_predicate(&mut self, predicate: &crate::checker::flow::FlowPredicate) -> Option<crate::identity::ExplanationId> {
        let prior_parent = predicate
            .binding()
            .and_then(|binding| self.flow.get_binding(binding))
            .and_then(|state| state.explanation);
        let hierarchy = &self.hierarchy;
        let applied = crate::checker::flow::transfer::apply_predicate(&mut self.flow, predicate, self.store, hierarchy)?;
        let predicate_kind = match predicate {
            crate::checker::flow::FlowPredicate::IsInstance { .. } => crate::explain::PredicateKind::IsInstance,
            crate::checker::flow::FlowPredicate::IsNotInstance { .. } => crate::explain::PredicateKind::IsNotInstance,
            crate::checker::flow::FlowPredicate::IsNil { .. } => crate::explain::PredicateKind::IsNil,
            crate::checker::flow::FlowPredicate::NotNil { .. } => crate::explain::PredicateKind::NotNil,
            crate::checker::flow::FlowPredicate::Equal { .. } | crate::checker::flow::FlowPredicate::EqualLiteral { .. } => {
                crate::explain::PredicateKind::EqualLiteral
            }
            crate::checker::flow::FlowPredicate::NotEqual { .. } | crate::checker::flow::FlowPredicate::NotEqualLiteral { .. } => {
                crate::explain::PredicateKind::NotEqualLiteral
            }
            crate::checker::flow::FlowPredicate::OrderedPredicate { .. } => crate::explain::PredicateKind::Ordered,
            crate::checker::flow::FlowPredicate::Truthy { .. } => crate::explain::PredicateKind::Truthy,
            crate::checker::flow::FlowPredicate::Falsy { .. } => crate::explain::PredicateKind::Falsy,
        };
        let explanation = self.record_derivation(
            crate::explain::ExplanationStep::FlowRefinement {
                binding: applied.binding,
                predicate: predicate_kind,
                prior: applied.prior,
                refined: applied.refined.clone(),
            },
            crate::explain::DerivationRule::FlowRefinement { predicate_kind },
            applied.refined.status().unwrap_or(crate::types::evidence::EvidenceStatus::Established),
            crate::types::evidence::EvidenceOrigin::Flow,
            Vec::new(),
            prior_parent.into_iter().collect(),
        );
        self.flow.facts.insert(predicate.clone(), explanation);
        self.flow.set_binding_explanation(applied.binding, explanation);
        Some(explanation)
    }

    pub fn join_flow_states(&mut self, states: &[FlowState]) -> Result<FlowState, crate::checker::flow::state::FlowInvariantFailure> {
        FlowState::join_with_hierarchy(states, self.store, &self.hierarchy)
    }

    pub fn publish_flow_join_failure(&mut self, failure: crate::checker::flow::state::FlowInvariantFailure, range: SourceRange) -> AnalysisStatus {
        let details = match failure {
            crate::checker::flow::state::FlowInvariantFailure::DivergentBindingContract { binding, left, right } => {
                InternalSemanticIncidentDetails::DivergentBindingContract {
                    binding,
                    left: BindingContractSummary::from(left.as_ref()),
                    right: BindingContractSummary::from(right.as_ref()),
                }
            }
            crate::checker::flow::state::FlowInvariantFailure::DivergentMutability { binding, left, right } => {
                InternalSemanticIncidentDetails::DivergentMutability { binding, left, right }
            }
            crate::checker::flow::state::FlowInvariantFailure::DivergentFieldContract { field, left, right } => {
                InternalSemanticIncidentDetails::DivergentFieldContract {
                    field,
                    left: *left,
                    right: *right,
                }
            }
        };
        let incident = self.record_internal_incident(InternalSemanticIncidentKind::FlowInvariantViolation, details, Some(range));
        let status = AnalysisStatus::InternalFailure(incident);
        self.poison_flow(incident);
        status
    }

    pub fn poison_flow(&mut self, incident: AnalysisIncidentId) {
        self.flow = FlowState::poisoned(incident);
    }

    pub(crate) fn end_call_causal_capture(
        &mut self,
    ) -> (
        crate::checker::causal::CausalInvalidity,
        Vec<crate::identity::ExplanationId>,
        Option<AnalysisStatus>,
    ) {
        let frame = self.call_dependency_frames.pop().unwrap_or_default();
        (frame.causal_invalidity, frame.explanations, frame.status)
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

    pub(crate) fn attach_explanation_to_cause(&mut self, cause: crate::identity::DiagnosticCauseId, explanation: crate::identity::ExplanationId) {
        let Some(callable) = self.current_callable.clone() else {
            return;
        };
        let reference = crate::diagnostic::ExplanationRef::new(callable, explanation);
        if let Some(diagnostic) = self.diagnostics.iter_mut().rev().find(|diagnostic| diagnostic.root_cause == Some(cause)) {
            if !diagnostic.explanations.contains(&reference) {
                diagnostic.explanations.push(reference);
            }
        }
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

    pub(crate) fn record_type_relation_with_parents(
        &mut self,
        actual: &TypeKnowledge,
        expected: TypeId,
        outcome: &RelationOutcome<()>,
        range: SourceRange,
        parents: Vec<crate::identity::ExplanationId>,
    ) -> crate::identity::ExplanationId {
        let status = actual.status().unwrap_or(crate::types::evidence::EvidenceStatus::Assumed);
        let origin = actual.origin().unwrap_or(crate::types::evidence::EvidenceOrigin::Flow);
        self.record_derivation(
            crate::explain::ExplanationStep::TypeRelation {
                actual: actual.clone(),
                expected,
                outcome: outcome.clone(),
            },
            crate::explain::DerivationRule::TypeRelation,
            status,
            origin,
            vec![crate::explain::EvidenceRef::TypeId(expected), crate::explain::EvidenceRef::SourceSpan(range)],
            parents,
        )
    }

    fn record_type_relation(
        &mut self,
        actual: &TypeKnowledge,
        expected: TypeId,
        outcome: &RelationOutcome<()>,
        range: SourceRange,
    ) -> crate::identity::ExplanationId {
        let parents = self
            .current_expression_id()
            .and_then(|expression| self.explanation_for_expression(expression))
            .into_iter()
            .collect();
        self.record_type_relation_with_parents(actual, expected, outcome, range, parents)
    }

    pub fn apply_relation_outcome(
        &mut self,
        outcome: RelationOutcome<()>,
        code: crate::diagnostic::DiagnosticCode,
        message: impl Into<String>,
        range: SourceRange,
        owner: Option<ExpressionId>,
        explanation: Option<crate::identity::ExplanationId>,
    ) -> RelationApplication {
        let message = message.into();
        let cause = if matches!(&outcome, RelationOutcome::Refuted(_)) {
            let mut diagnostic = SemanticDiagnostic::error_in(self.current_module.clone(), code, message, range);
            if let (Some(callable), Some(explanation)) = (self.current_callable.clone(), explanation) {
                diagnostic = diagnostic.with_explanation(crate::diagnostic::ExplanationRef::new(callable, explanation));
            }
            self.emit_diagnostic(diagnostic)
        } else {
            None
        };

        let status = match (&outcome, cause) {
            (RelationOutcome::Proven { .. }, _) => None,
            (RelationOutcome::Refuted(_), Some(cause)) => Some(AnalysisStatus::Invalid(cause)),
            (RelationOutcome::DynamicBoundary(_), _) => Some(AnalysisStatus::DynamicBoundary(crate::types::evidence::DynamicReason::RuntimeReflection)),
            (RelationOutcome::Blocked(reason), _) => Some(AnalysisStatus::Blocked(reason.clone())),
            (RelationOutcome::Cancelled, _) => Some(AnalysisStatus::Cancelled),
            (RelationOutcome::BudgetExceeded(report), _) => Some(AnalysisStatus::BudgetExceeded(report.clone())),
            (RelationOutcome::InternalFailure(message), _) => Some(AnalysisStatus::InternalFailure(self.record_internal_incident(
                InternalSemanticIncidentKind::RelationInvariantViolation,
                InternalSemanticIncidentDetails::Message {
                    message: message.clone().into_boxed_str(),
                },
                Some(range),
            ))),
            (RelationOutcome::Refuted(_), None) => None,
        };
        if let Some(status) = status.clone() {
            self.record_call_status(status);
        }
        if let Some(cause) = cause {
            self.record_call_dependency(crate::checker::causal::CausalInvalidity::One(cause), None);
        }

        if let Some(owner) = owner {
            if let Some(status) = status.clone() {
                if let Some(analysis) = self.expressions.get_mut(&owner) {
                    analysis.status = status;
                    if let Some(cause) = cause {
                        analysis.causal_invalidity = analysis.causal_invalidity.join(crate::checker::causal::CausalInvalidity::One(cause));
                    }
                }
            }
        }

        if let Some(explanation) = explanation {
            self.record_call_dependency(crate::checker::causal::CausalInvalidity::Clean, Some(explanation));
        }
        RelationApplication {
            outcome,
            cause,
            status,
            explanation,
        }
    }

    pub fn publish_analysis_incident(&mut self, message: impl Into<String>) -> AnalysisIncidentId {
        self.record_internal_incident(
            InternalSemanticIncidentKind::RelationInvariantViolation,
            InternalSemanticIncidentDetails::Message {
                message: message.into().into_boxed_str(),
            },
            None,
        )
    }

    pub fn record_internal_incident(
        &mut self,
        kind: InternalSemanticIncidentKind,
        details: InternalSemanticIncidentDetails,
        range: Option<SourceRange>,
    ) -> AnalysisIncidentId {
        let incident = crate::identity::InternalSemanticIncidentId(self.next_analysis_incident);
        self.next_analysis_incident += 1;
        let record = InternalSemanticIncident {
            id: incident,
            kind,
            module: self.current_module.clone(),
            callable: self.current_callable.clone(),
            expression: self.current_expression_id(),
            range,
            details,
        };
        self.analysis_incidents.insert(incident, record);
        self.record_terminal_status(AnalysisStatus::InternalFailure(incident));
        if matches!(self.internal_failure_policy, InternalFailurePolicy::FailFast) {
            let incident = self.analysis_incidents.get(&incident).expect("incident stored before fail-fast policy");
            panic!("INTERNAL SEMANTIC INVARIANT FAILURE\n{incident:#?}");
        }
        incident
    }

    pub fn set_internal_failure_policy(&mut self, policy: InternalFailurePolicy) {
        self.internal_failure_policy = policy;
    }

    pub fn apply_assignability(
        &mut self,
        actual: &TypeKnowledge,
        expected: &TypeKnowledge,
        code: crate::diagnostic::DiagnosticCode,
        message: impl Into<String>,
        range: SourceRange,
    ) -> RelationApplication {
        let outcome = self
            .control
            .relation(|budget, cancellation| check_assignability_bounded(self.store, &self.hierarchy, actual, expected, budget, cancellation));
        let explanation = expected.ty().map(|expected_ty| self.record_type_relation(actual, expected_ty, &outcome, range));
        self.apply_relation_outcome(outcome, code, message, range, None, explanation)
    }

    pub fn apply_knowledge_against_type(
        &mut self,
        actual: &TypeKnowledge,
        expected: TypeId,
        code: crate::diagnostic::DiagnosticCode,
        message: impl Into<String>,
        range: SourceRange,
    ) -> RelationApplication {
        let outcome = self
            .control
            .relation(|budget, cancellation| check_knowledge_against_type_bounded(self.store, &self.hierarchy, actual, expected, budget, cancellation));
        let explanation = Some(self.record_type_relation(actual, expected, &outcome, range));
        self.apply_relation_outcome(outcome, code, message, range, None, explanation)
    }

    /// Evaluates one contract relation while sharing this body's budget and
    /// cancellation state. The caller owns diagnostic policy and reconciliation.
    pub fn check_knowledge_against_type(&mut self, actual: &TypeKnowledge, expected: TypeId) -> RelationOutcome {
        self.control
            .relation(|budget, cancellation| check_knowledge_against_type_bounded(self.store, &self.hierarchy, actual, expected, budget, cancellation))
    }

    pub fn apply_knowledge_against_type_owned(
        &mut self,
        actual: &TypeKnowledge,
        expected: TypeId,
        code: crate::diagnostic::DiagnosticCode,
        message: impl Into<String>,
        range: SourceRange,
        owner: ExpressionId,
    ) -> RelationApplication {
        let outcome = self
            .control
            .relation(|budget, cancellation| check_knowledge_against_type_bounded(self.store, &self.hierarchy, actual, expected, budget, cancellation));
        let explanation = Some(self.record_type_relation(actual, expected, &outcome, range));
        let application = self.apply_relation_outcome(outcome, code, message, range, Some(owner), explanation);
        if let Some(cause) = application.cause {
            self.expression_owned_causes.entry(owner).or_insert(cause);
        }
        application
    }

    /// Publishes the complete expression product after analysis has settled.
    /// Production expression analysis uses this single publication path.
    pub(crate) fn publish_expression_analysis(
        &mut self,
        id: ExpressionId,
        range: SourceRange,
        typed: &crate::checker::typed_expr::TypedExpression,
        explanation: Option<crate::identity::ExplanationId>,
    ) -> ExpressionAnalysis {
        typed.debug_assert_coherent();

        let mut analysis = ExpressionAnalysis::ready(id, range, typed.knowledge.clone());
        analysis.callable = typed.callable.clone();
        analysis.denotation = typed.denotation;
        analysis.status = typed.status.clone();
        analysis.causal_invalidity = typed.causal_invalidity;
        analysis.explanation = explanation;

        self.expressions.insert(id, analysis.clone());
        analysis
    }

    pub(crate) fn sync_expression_outcome(&mut self, typed: &crate::checker::typed_expr::TypedExpression) {
        typed.debug_assert_coherent();

        let Some(id) = typed.expression_id else {
            return;
        };
        let Some(analysis) = self.expressions.get_mut(&id) else {
            return;
        };

        analysis.knowledge = typed.knowledge.clone();
        analysis.callable = typed.callable.clone();
        analysis.denotation = typed.denotation;
        analysis.status = typed.status.clone();
        analysis.causal_invalidity = typed.causal_invalidity;
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
        let relation = match seed.contract.as_ref() {
            None => RelationOutcome::proven(()),
            Some(contract) => match &seed.current {
                TypeKnowledge::Unknown(reason)
                    if matches!(contract.origin, BindingContractOrigin::SourceAnnotation)
                        && reason.contract_assumption_eligibility() == ContractAssumptionEligibility::MaySupplyAssumption =>
                {
                    RelationOutcome::proven(())
                }
                TypeKnowledge::Unknown(reason) => RelationOutcome::Blocked(crate::types::outcome::BlockReason::UnknownType(reason.clone())),
                TypeKnowledge::Dynamic(_) => RelationOutcome::DynamicBoundary(DynamicBoundaryObligation {
                    reason: "binding contract crosses dynamic boundary".into(),
                }),
                TypeKnowledge::Known(_) => self.check_knowledge_against_type(&seed.current, contract.ty),
            },
        };
        let reconciliation = reconcile_binding_relation(seed.contract.as_ref(), &seed.current, relation);
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
        if let Some(state) = self.flow.get_binding(binding_id).cloned() {
            self.binding_history.insert(binding_id, state);
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, LocalBindingInfo { id: binding_id, denotation });
        }
        BindingDeclarationResult::Inserted(binding_id)
    }

    pub fn bind_callable_parameter(&mut self, name: impl Into<String>, current: TypeKnowledge, range: SourceRange) -> BindingDeclarationResult {
        self.bind_callable_parameter_with_identity(name, current, range, crate::checker::causal::CausalInvalidity::Clean, None)
    }

    pub fn bind_canonical_callable_parameter(
        &mut self,
        parameter: &crate::signature::CallableParameterSemantic,
        fallback_range: SourceRange,
    ) -> BindingDeclarationResult {
        let range = parameter.source.as_ref().map_or(fallback_range, |source| source.range);
        self.bind_callable_parameter_with_identity(
            parameter.local_name.to_string(),
            parameter.declared_type.to_knowledge(),
            range,
            crate::checker::causal::CausalInvalidity::Clean,
            Some(parameter.id.clone()),
        )
    }

    pub fn bind_callable_parameter_with_causal(
        &mut self,
        name: impl Into<String>,
        current: TypeKnowledge,
        range: SourceRange,
        causal_invalidity: crate::checker::causal::CausalInvalidity,
    ) -> BindingDeclarationResult {
        self.bind_callable_parameter_with_identity(name, current, range, causal_invalidity, None)
    }

    fn bind_callable_parameter_with_identity(
        &mut self,
        name: impl Into<String>,
        current: TypeKnowledge,
        range: SourceRange,
        causal_invalidity: crate::checker::causal::CausalInvalidity,
        parameter: Option<crate::identity::CallableParameterId>,
    ) -> BindingDeclarationResult {
        let current = current
            .ty()
            .map(|ty| TypeKnowledge::assumed(ty, crate::types::evidence::EvidenceOrigin::CallableSignature))
            .unwrap_or(current);
        let contract = current.ty().map(|ty| BindingContract {
            ty,
            origin: BindingContractOrigin::CallableParameter,
            source: Some(range),
        });
        self.declare_binding(BindingSeed {
            name: name.into(),
            parameter,
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
            parameter: None,
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
            parameter: None,
            name: name.into(),
            range,
            contract: None,
            current: TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::NoTypeEvidence),
            denotation: None,
            causal_invalidity: crate::checker::causal::CausalInvalidity::Clean,
            mutable: false,
        })
    }

    pub fn bind_pattern_binding_with_causal(
        &mut self,
        name: impl Into<String>,
        fact: ValueSemanticFact,
        range: SourceRange,
        causal_invalidity: crate::checker::causal::CausalInvalidity,
    ) -> BindingDeclarationResult {
        let contract = fact.knowledge.ty().map(|ty| BindingContract {
            ty,
            origin: BindingContractOrigin::PatternBinding,
            source: Some(range),
        });
        self.declare_binding(BindingSeed {
            parameter: None,
            name: name.into(),
            range,
            contract,
            current: fact.knowledge,
            denotation: fact.denotation,
            causal_invalidity,
            mutable: true,
        })
    }

    pub fn bind_pattern_binding(&mut self, name: impl Into<String>, fact: ValueSemanticFact, range: SourceRange) -> BindingDeclarationResult {
        self.bind_pattern_binding_with_causal(name, fact, range, crate::checker::causal::CausalInvalidity::Clean)
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
        if let Some(state) = self.flow.get_binding(info.id).cloned() {
            self.binding_history.insert(info.id, state);
        }
        result
    }

    pub(crate) fn push_loop_frame(&mut self) {
        self.loop_frames.push(LoopFlowFrame::default());
    }

    pub(crate) fn pop_loop_frame(&mut self) -> LoopFlowFrame {
        self.loop_frames.pop().unwrap_or_default()
    }

    pub(crate) fn record_continue(&mut self) {
        let state = self.flow.clone();
        if let Some(frame) = self.loop_frames.last_mut() {
            frame.continues.push(state);
        }
    }

    pub(crate) fn record_break(&mut self) {
        let state = self.flow.clone();
        if let Some(frame) = self.loop_frames.last_mut() {
            frame.breaks.push(state);
        }
    }

    /// Records an explicitly consumed semantic dependency.
    pub(crate) fn record_semantic_dependency(&self, dependency: SemanticDependency) {
        record_query_dependency(&self.semantic_dependencies, dependency);
    }

    pub(crate) fn semantic_dependencies_snapshot(&self) -> BTreeSet<SemanticDependency> {
        self.semantic_dependencies.borrow().clone()
    }

    /// Records a dispatch lookup's structural and callable-type dependencies.
    ///
    /// `DeclarationSurface` owns selector/visibility/hierarchy projection only;
    /// every query-owned callable's type contract is represented by its
    /// canonical `CallableSignature` product, including partial declarations.
    pub(crate) fn record_consumed_callable_signature(&self, callable: &CallableId, _signature: &crate::dispatch::CallableSignature) {
        if !is_query_owned_module(&callable.owner.module) {
            return;
        }
        record_declaration_surface_dependency(&self.semantic_dependencies, &callable.owner);
        self.record_semantic_dependency(SemanticDependency::CallableSignature(callable.clone()));
    }

    /// Attaches compiler-owned canonical field declaration knowledge.
    pub fn attach_field_signatures(&mut self, field_signatures: &'a FieldSignatureTable) {
        self.field_signatures = Some(field_signatures);
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

    fn dispatch_owner_for_lookup(&self, receiver: TypeId, lookup: crate::dispatch::DispatchLookup) -> Option<(DeclarationId, DispatchSide)> {
        match lookup {
            crate::dispatch::DispatchLookup::Super { defining_class, side } => {
                self.hierarchy.superclass(&defining_class).cloned().map(|super_decl| (super_decl, side))
            }
            crate::dispatch::DispatchLookup::Normal => match self.store.get(receiver) {
                TypeData::ClassObject { declaration } => Some((declaration.clone(), DispatchSide::Class)),
                TypeData::Nominal { declaration } => Some((declaration.clone(), DispatchSide::Instance)),
                TypeData::Applied { origin, .. } => {
                    let mut curr_origin = *origin;
                    while let TypeData::Applied { origin: inner_origin, .. } = self.store.get(curr_origin) {
                        curr_origin = *inner_origin;
                    }
                    if let TypeData::Nominal { declaration } = self.store.get(curr_origin) {
                        Some((declaration.clone(), DispatchSide::Instance))
                    } else {
                        None
                    }
                }
                _ => None,
            },
        }
    }

    fn specialize_dispatch_signature(&mut self, receiver: TypeId, mut signature: crate::dispatch::CallableSignature) -> crate::dispatch::CallableSignature {
        if let Some(subst) = self.substitution_for_applied_receiver(receiver) {
            for parameter in &mut signature.parameters {
                parameter.ty = parameter.ty.map_type(|ty| subst.apply(self.store, ty));
            }
            signature.return_type = signature.return_type.map_type(|ty| subst.apply(self.store, ty));
        }
        for parameter in &mut signature.parameters {
            parameter.ty = parameter.ty.map_type(|ty| self.specialize_self_type(receiver, ty));
        }
        signature.return_type = signature.return_type.map_type(|ty| self.specialize_self_type(receiver, ty));
        signature
    }

    pub(crate) fn resolve_dispatch_target(&mut self, receiver: TypeId, selector: &Selector, lookup: crate::dispatch::DispatchLookup) -> ResolvedDispatchResult {
        let Some((decl, side)) = self.dispatch_owner_for_lookup(receiver, lookup) else {
            return ResolvedDispatchResult::Missing { visited_owners: Box::new([]) };
        };

        let result = self.dispatch.get().resolve_dispatch_with_trace(&self.hierarchy, &decl, side, selector);
        match result {
            ResolvedDispatchResult::Found(mut resolved) => {
                for owner in resolved.visited_owners.iter() {
                    record_declaration_surface_dependency(&self.semantic_dependencies, owner);
                }
                self.dependencies.insert(resolved.callable.clone());
                self.record_consumed_callable_signature(&resolved.callable, &resolved.signature);
                resolved.specialization = Some(crate::dispatch::DispatchSignatureSpecialization {
                    receiver,
                    unspecialized_return: resolved.signature.return_type.clone(),
                });
                resolved.signature = self.specialize_dispatch_signature(receiver, resolved.signature);
                if let Some(expression) = self.current_expression_id() {
                    self.resolved_callables.insert(expression, resolved.callable.clone());
                }
                ResolvedDispatchResult::Found(resolved)
            }
            ResolvedDispatchResult::Ambiguous(mut ambiguous) => {
                for resolved in &mut ambiguous {
                    for owner in resolved.visited_owners.iter() {
                        record_declaration_surface_dependency(&self.semantic_dependencies, owner);
                    }
                    self.dependencies.insert(resolved.callable.clone());
                    self.record_consumed_callable_signature(&resolved.callable, &resolved.signature);
                    resolved.specialization = Some(crate::dispatch::DispatchSignatureSpecialization {
                        receiver,
                        unspecialized_return: resolved.signature.return_type.clone(),
                    });
                    resolved.signature = self.specialize_dispatch_signature(receiver, resolved.signature.clone());
                }
                ResolvedDispatchResult::Ambiguous(ambiguous)
            }
            ResolvedDispatchResult::Missing { visited_owners } => {
                for owner in visited_owners.iter() {
                    record_declaration_surface_dependency(&self.semantic_dependencies, owner);
                }
                ResolvedDispatchResult::Missing { visited_owners }
            }
            ResolvedDispatchResult::Dynamic => ResolvedDispatchResult::Dynamic,
        }
    }

    pub fn resolve_dispatch(&mut self, receiver: TypeId, selector: &Selector, lookup: crate::dispatch::DispatchLookup) -> DispatchResult {
        match self.resolve_dispatch_target(receiver, selector, lookup) {
            ResolvedDispatchResult::Found(resolved) => DispatchResult::Found(Box::new(resolved.signature)),
            ResolvedDispatchResult::Ambiguous(ambiguous) => DispatchResult::Ambiguous(ambiguous.into_iter().map(|resolved| resolved.signature).collect()),
            ResolvedDispatchResult::Missing { .. } => DispatchResult::Missing,
            ResolvedDispatchResult::Dynamic => DispatchResult::Dynamic,
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
        let field = crate::identity::FieldId::new(decl.clone(), name, side);
        let signature = self.field_signatures?.get(&field)?;
        if is_query_owned_module(&field.owner.module) {
            self.record_semantic_dependency(SemanticDependency::FieldSignature(field));
        }
        Some(signature.declared_type.to_knowledge())
    }

    pub(crate) fn resolve_field_contract(&self, owner: &DeclarationId, side: DispatchSide, name: &str) -> Option<(crate::identity::FieldId, TypeKnowledge)> {
        let field = crate::identity::FieldId::new(owner.clone(), name, side);
        let signature = self.field_signatures?.get(&field)?;
        if is_query_owned_module(&field.owner.module) {
            self.record_semantic_dependency(SemanticDependency::FieldSignature(field.clone()));
        }
        Some((field, signature.declared_type.to_knowledge()))
    }

    pub(crate) fn resolve_current_field(&self, owner: &DeclarationId, side: DispatchSide, name: &str) -> Option<(crate::identity::FieldId, TypeKnowledge)> {
        let (field, contract) = self.resolve_field_contract(owner, side, name)?;
        let current = self.flow.get_field_current(&field).cloned().unwrap_or(contract);
        Some((field, current))
    }

    pub(crate) fn write_current_field(&mut self, field: crate::identity::FieldId, contract: TypeKnowledge, current: TypeKnowledge) {
        if self.flow.get_field(&field).is_none() {
            self.flow.seed_field(crate::checker::flow::FieldState {
                field: field.clone(),
                contract,
                current: TypeKnowledge::Unknown(UnknownReason::MissingInitializer),
                initialization: crate::checker::flow::FieldInitialization::Uninitialized,
                version: 0,
            });
        }
        self.flow
            .write_field(&field, current, crate::checker::flow::FieldInitialization::DefinitelyInitialized);
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

    /// Legacy compatibility query. Suppression is now carried by the owning
    /// expression's [`AnalysisStatus`] and is never reconstructed from a
    /// context-side side table.
    pub fn suppression_cause(&self, _id: ExpressionId) -> Option<crate::checker::causal::SuppressionCause> {
        None
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
        mut self,
        callable: CallableId,
        body_range: SourceRange,
        status: crate::checker::analysis::CallableAnalysisStatus,
        normal_return_values: Vec<crate::types::evidence::TypeKnowledge>,
    ) -> crate::checker::analysis::CallableAnalysis {
        let entry_flow = self.current_flow_summary();
        let flow_graph = self
            .flow_graph
            .unwrap_or_else(|| std::sync::Arc::new(crate::checker::flow::graph::FlowGraph::default()));

        let return_summary = crate::checker::analysis::normal_return_summary(self.store, &normal_return_values);
        let return_explanation = Some(self.explanations.alloc(
            crate::explain::ExplanationStep::CallableReturnSummary {
                callable: callable.clone(),
                returns: normal_return_values.clone().into_boxed_slice(),
                result: return_summary.clone(),
            },
            return_summary.status().unwrap_or(crate::types::evidence::EvidenceStatus::Assumed),
            return_summary.origin().unwrap_or(crate::types::evidence::EvidenceOrigin::Flow),
            Vec::new(),
        ));
        let exits = crate::checker::analysis::BodyExitFacts {
            returns: if normal_return_values.is_empty() {
                Vec::new()
            } else {
                vec![entry_flow.clone()]
            },
            normal_return_values,
            throws: self.throw_exit_flows,
            unreachable: false,
        };

        crate::checker::analysis::CallableAnalysis {
            callable,
            body_range,
            expressions: self.expressions,
            bindings: {
                let mut bindings = self.binding_history;
                bindings.extend(self.flow.bindings);
                bindings
            },
            flow_graph,
            entry_flow,
            exits,
            diagnostics: std::sync::Arc::from(self.diagnostics.into_boxed_slice()),
            internal_incidents: std::sync::Arc::from(self.analysis_incidents.into_values().collect::<Vec<_>>().into_boxed_slice()),
            explanations: std::sync::Arc::new(self.explanations),
            return_explanation,
            dependencies: std::sync::Arc::from(self.dependencies.into_iter().collect::<Vec<_>>().into_boxed_slice()),
            semantic_dependencies: std::sync::Arc::from(self.semantic_dependencies.borrow().iter().cloned().collect::<Vec<_>>().into_boxed_slice()),
            dependency_fingerprint: crate::db::ProductFingerprint::new(0),
            status,
        }
    }
}

/// Keeps core type-test calls available to standalone semantic checking.
/// Workspace sessions normally publish these declarations from the embedded
/// core source; direct checker fixtures intentionally do not load that module.
pub(crate) fn ensure_core_object_type_tests(store: &mut TypeStore, declarations: &DeclarationTypeTable, dispatch: &mut SurfaceDispatchResolver) {
    let class = DeclarationId::new(ModuleId::core(), "Class".into());
    let mut class_surface = dispatch
        .get_surface(&class)
        .cloned()
        .unwrap_or_else(|| DeclarationSurface::new(Some(class.clone())));
    let canonical_new = crate::checker::declaration_signature::canonical_core_class_new_signature(store);
    if class_surface.instance.get_callable(&canonical_new.selector).is_none() {
        class_surface.add_callable(
            DispatchSide::Instance,
            crate::checker::declaration_signature::project_semantic_signature(&canonical_new),
        );
    }
    dispatch.register_type(declarations.form(&class).unwrap_or_else(|| store.nominal_type(class.clone())), class.clone());
    dispatch.register_surface(class, class_surface);

    let object = DeclarationId::new(ModuleId::core(), "Object".into());
    if let Some(bool_ty) = declarations.form(&DeclarationId::new(ModuleId::core(), "Bool".into())) {
        let mut surface = dispatch
            .get_surface(&object)
            .cloned()
            .unwrap_or_else(|| DeclarationSurface::new(Some(object.clone())));
        for method in ["is", "is!"] {
            let Ok(selector) = Selector::method(method, [phalcom_common::selector::SelectorSlot::Positional]) else {
                continue;
            };
            if surface.instance.get_callable(&selector).is_some() {
                continue;
            }
            let parameter = CallableParameter::new("class", TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence));
            let signature = CallableSignature::new(
                selector,
                vec![parameter],
                TypeKnowledge::established(bool_ty, EvidenceOrigin::DeclarationSemantics),
            );
            surface.add_callable(DispatchSide::Instance, signature);
        }
        dispatch.register_type(declarations.form(&object).unwrap_or_else(|| store.nominal_type(object.clone())), object.clone());
        dispatch.register_surface(object, surface);
    }
}

#[cfg(test)]
mod tests {
    use super::CheckingContext;
    use crate::checker::binding::{BindingContract, BindingContractOrigin};
    use crate::checker::causal::CausalInvalidity;
    use crate::checker::flow::state::FlowInvariantFailure;
    use crate::checker::incident::{InternalFailurePolicy, InternalSemanticIncidentDetails, InternalSemanticIncidentKind};
    use crate::declarations::bootstrap_universe_declarations;
    use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
    use crate::dispatch::{CallableSignature, ResolvedDispatchResult};
    use crate::identity::{CallableId, DeclarationId, DispatchSide, ModuleId};
    use crate::types::SimpleTypeResolver;
    use crate::types::id::TypeId;
    use crate::types::relation::MapTypeHierarchy;
    use crate::types::store::TypeStore;
    use phalcom_common::range::SourceRange;
    use std::panic::{AssertUnwindSafe, catch_unwind};

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

    #[test]
    fn flow_invariant_is_recorded_before_callable_containment() {
        let module = ModuleId::core();
        let mut store = TypeStore::new();
        let declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(module.clone(), key.name().into()));
        let resolver = SimpleTypeResolver::new();
        let hierarchy = MapTypeHierarchy::new();
        let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module);
        let binding = crate::identity::BindingId(7);
        let left = BindingContract {
            ty: TypeId(1),
            origin: BindingContractOrigin::SourceAnnotation,
            source: Some(SourceRange { start: 1, end: 2 }),
        };
        let right = BindingContract {
            ty: TypeId(2),
            origin: BindingContractOrigin::SourceAnnotation,
            source: Some(SourceRange { start: 3, end: 4 }),
        };

        let status = ctx.publish_flow_join_failure(
            FlowInvariantFailure::DivergentBindingContract {
                binding,
                left: Some(left),
                right: Some(right),
            },
            SourceRange { start: 1, end: 4 },
        );

        let incident = ctx.analysis_incidents.values().next().expect("incident recorded");
        assert!(matches!(status, crate::checker::analysis::AnalysisStatus::InternalFailure(_)));
        assert!(ctx.flow.is_poisoned());
        assert!(matches!(incident.kind, InternalSemanticIncidentKind::FlowInvariantViolation));
        assert!(matches!(incident.details, InternalSemanticIncidentDetails::DivergentBindingContract { binding: id, .. } if id == binding));
        assert!(matches!(
            ctx.terminal_status,
            Some(crate::checker::analysis::AnalysisStatus::InternalFailure(_))
        ));
    }

    #[test]
    fn fail_fast_policy_panics_only_after_recording_incident() {
        let module = ModuleId::core();
        let mut store = TypeStore::new();
        let declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(module.clone(), key.name().into()));
        let resolver = SimpleTypeResolver::new();
        let hierarchy = MapTypeHierarchy::new();
        let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module);
        ctx.set_internal_failure_policy(InternalFailurePolicy::FailFast);

        let result = catch_unwind(AssertUnwindSafe(|| {
            ctx.record_internal_incident(
                InternalSemanticIncidentKind::RelationInvariantViolation,
                InternalSemanticIncidentDetails::Message { message: "test".into() },
                None,
            )
        }));

        assert!(result.is_err());
        assert_eq!(ctx.analysis_incidents.len(), 1);
        assert!(matches!(
            ctx.terminal_status,
            Some(crate::checker::analysis::AnalysisStatus::InternalFailure(_))
        ));
    }

    #[test]
    fn dispatch_target_preserves_callable_identity() {
        let module = ModuleId::core();
        let mut store = TypeStore::new();
        let declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(module.clone(), key.name().into()));
        let resolver = SimpleTypeResolver::new();
        let hierarchy = MapTypeHierarchy::new();
        let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

        let owner = DeclarationId::new(module, "Owner".into());
        let selector = phalcom_common::selector::Selector::getter("value").unwrap();
        let callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);
        let int_decl = DeclarationId::new(ctx.current_module.clone(), "Int".into());
        let int = ctx.nominal_type_of(&int_decl);
        let signature = CallableSignature::new(
            selector,
            Vec::new(),
            crate::types::evidence::TypeKnowledge::established(int, crate::types::evidence::EvidenceOrigin::CallableSignature),
        );
        let mut surface = crate::surface::DeclarationSurface::new(Some(owner.clone()));
        surface.add_callable(DispatchSide::Instance, signature);
        ctx.register_surface(owner.clone(), surface);

        let receiver = ctx.nominal_type_of(&owner);
        let selector = phalcom_common::selector::Selector::getter("value").unwrap();
        let result = ctx.resolve_dispatch_target(receiver, &selector, crate::dispatch::DispatchLookup::Normal);
        let ResolvedDispatchResult::Found(resolved) = result else {
            panic!("expected resolved target");
        };
        assert_eq!(resolved.callable, callable);
        assert_eq!(resolved.signature.return_type.ty(), Some(int));
    }
}
