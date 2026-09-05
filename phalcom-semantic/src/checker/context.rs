use crate::checker::analysis::{AnalysisStatus, ExpressionAnalysis, ExpressionAnalysisIndex, SemanticDependency};
use crate::checker::binding::{BindingContract, BindingContractOrigin, BindingDeclarationResult, BindingSeed, BindingWriteResult, reconcile_binding_relation};
use crate::checker::flow::FlowState;
use crate::checker::incident::{
    BindingContractSummary, InternalFailurePolicy, InternalSemanticIncident, InternalSemanticIncidentDetails, InternalSemanticIncidentKind,
};
use crate::core_surface::CoreDeclarationIds;
use crate::db::budget::{BudgetReport, CancellationToken, QueryBudget};
use crate::declarations::{DeclarationTypeInfo, DeclarationTypeTable};
use crate::diagnostic::SemanticDiagnostic;
use crate::dispatch::{CallableParameter, CallableSignature, DispatchResult, ResolvedDispatchResult, SurfaceDispatchResolver};
use crate::identity::{
    AnalysisIncidentId, BindingId, BodyId, CallableId, DeclarationId, DiagnosticCauseId, DispatchSide, ExpressionId, LocalExpressionId, ModuleId,
};
use crate::signature::FieldSignatureTable;
use crate::surface::DeclarationSurface;
use crate::types::annotation::{TypeFormResolution, TypeFormationSite, TypeLevelBinding, TypeResolver};
use crate::types::denotation::{SemanticDenotation, ValueSemanticFact};
use crate::types::evidence::{ContractAssumptionEligibility, EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::native::register_native_surfaces;
use crate::types::outcome::{DynamicBoundaryObligation, RelationOutcome};
use crate::types::relation::{TypeHierarchy, check_assignability_bounded, check_knowledge_against_type_bounded, is_subtype};
use crate::types::rigid::{LocalConstraint, LocalType, RigidArena};
use crate::types::specialization::SpecializationControl;
use crate::types::specialization::{ReceiverSpecialization, ReceiverSpecializationFailure, specialize_receiver_to_owner};
use crate::types::store::{TypeData, TypeStore};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_native_surface::NATIVE_SURFACES;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
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

impl SpecializationControl for CheckerControl {
    fn charge_step(&self) -> Result<(), crate::types::outcome::BudgetReport> {
        Self::charge_step(self)
    }

    fn is_cancelled(&self) -> bool {
        Self::is_cancelled(self)
    }
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

    /// Charges one shared fixed-point iteration against query policy.
    pub fn charge_scc_iteration(&self) -> Result<(), BudgetReport> {
        self.budget.borrow_mut().charge_scc_iteration()
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

/// Every module identity participates in dependency tracking. Bootstrap
/// declarations are filtered separately by [`is_bootstrap_declaration`]; a
/// legacy-looking module path must not silently become a dependency-free
/// sentinel.
fn is_query_owned_module(module: &ModuleId) -> bool {
    let _ = module;
    true
}

/// Returns whether declaration belongs to immutable canonical Universe input.
///
/// Native surfaces and Universe source classes are installed as bootstrap
/// products, not recomputed source-query products. They remain valid semantic
/// inputs across revisions, so source-owned body queries must not capture
/// revision-local dependencies on their declaration surfaces or signatures.
fn is_bootstrap_declaration(declaration: &DeclarationId) -> bool {
    if !matches!(declaration.module.project, phalcom_modules::ProjectIdentity::Universe) {
        return false;
    }
    let Some(key) = phalcom_native_meta::UniverseKey::from_name(declaration.name.as_ref()) else {
        return false;
    };
    let components = declaration.module.path.components();
    components.len() == key.source_path().len() && components.iter().zip(key.source_path()).all(|(actual, expected)| actual.as_str() == *expected)
}

/// Built-in type-test callables are represented by the standalone bootstrap
/// surface, not by source-owned `CallableSignature` query products. They can
/// still be consumed during body checking, but must not create a dependency
/// on a query product that cannot exist for their source-less module.
fn is_builtin_type_test_callable(callable: &CallableId) -> bool {
    callable.side == DispatchSide::Instance
        && callable.declaration_owner() == &crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Object)
        && matches!(&callable.selector.base, phalcom_common::selector::SelectorBase::Named(name) if matches!(name.as_str(), "is" | "is!"))
        && matches!(callable.selector.kind, phalcom_common::selector::SelectorKind::Method)
        && callable.selector.slots.len() == 1
}

fn record_query_dependency(dependencies: &SharedSemanticDependencies, dependency: SemanticDependency) {
    dependencies.borrow_mut().insert(dependency);
}

fn record_declaration_surface_dependency(dependencies: &SharedSemanticDependencies, declaration: &DeclarationId) {
    if is_query_owned_module(&declaration.module) && !is_bootstrap_declaration(declaration) {
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

    pub(crate) fn inner(&self) -> &'a dyn TypeResolver {
        self.inner
    }
}

impl TypeResolver for TrackingTypeResolver<'_> {
    fn resolve_type_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId> {
        let declaration = self.inner.resolve_type_name(current_module, root, members);

        if is_query_owned_module(current_module) {
            record_query_dependency(&self.dependencies, SemanticDependency::LinkedName(current_module.clone(), root.to_string()));
        }

        let Some(declaration) = declaration else {
            if members.len() == 1 && is_query_owned_module(current_module) {
                if let Some(target_module) = self.inner.resolve_module_alias(current_module, root) {
                    if is_query_owned_module(&target_module) {
                        record_query_dependency(
                            &self.dependencies,
                            SemanticDependency::PublicExport(target_module, members[0].clone()),
                        );
                    }
                }
            }
            return None;
        };

        record_declaration_shell_dependency(&self.dependencies, &declaration);
        if &declaration.module != current_module && is_query_owned_module(current_module) && is_query_owned_module(&declaration.module) {
            let public_name = if members.is_empty() {
                root
            } else {
                members.last().unwrap().as_str()
            };
            record_query_dependency(
                &self.dependencies,
                SemanticDependency::PublicExport(declaration.module.clone(), public_name.to_string()),
            );
        }
        Some(declaration)
    }

    fn resolve_type_level_binding(&self, name: &str) -> Option<TypeLevelBinding> {
        self.inner.resolve_type_level_binding(name)
    }

    fn resolve_alias_form(&self, declaration: &DeclarationId) -> Option<TypeId> {
        self.inner.resolve_alias_form(declaration)
    }

    fn resolve_module_alias(&self, current_module: &ModuleId, alias: &str) -> Option<ModuleId> {
        self.inner.resolve_module_alias(current_module, alias)
    }
}

/// Hierarchy wrapper that records each mutable direct edge consumed by body checking.
#[derive(Clone)]
pub struct TrackingTypeHierarchy<'a> {
    inner: &'a dyn TypeHierarchy,
    declarations: &'a DeclarationTypeTable,
    dependencies: SharedSemanticDependencies,
}

impl<'a> TrackingTypeHierarchy<'a> {
    fn new(inner: &'a dyn TypeHierarchy, declarations: &'a DeclarationTypeTable, dependencies: SharedSemanticDependencies) -> Self {
        Self {
            inner,
            declarations,
            dependencies,
        }
    }

    pub(crate) fn inner(&self) -> &'a dyn TypeHierarchy {
        self.inner
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
        self.inner
            .supertype_template(declaration)
            .or_else(|| self.declarations.supertype_template(declaration))
    }
}

/// Metadata for a scoped local variable binding.
#[derive(Clone, Debug)]
pub struct LocalBindingInfo {
    pub id: BindingId,
    pub denotation: Option<SemanticDenotation>,
}

pub(crate) type CallCapture = (
    crate::checker::causal::CausalInvalidity,
    Vec<crate::identity::ExplanationId>,
    Option<AnalysisStatus>,
    Vec<(TypeId, LocalType)>,
);

#[derive(Clone, Default)]
struct CallDependencyFrame {
    causal_invalidity: crate::checker::causal::CausalInvalidity,
    explanations: Vec<crate::identity::ExplanationId>,
    status: Option<AnalysisStatus>,
    local_types: Vec<(TypeId, LocalType)>,
}

/// Declared callable return contract. This is checking context, not value
/// evidence; return expressions are judged against its type and provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableReturnContract {
    pub ty: TypeId,
    pub basis: crate::declaration_type::DeclaredTypeBasis,
    pub origin: crate::types::evidence::EvidenceOrigin,
    pub is_dynamic: bool,
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

pub struct FlowProbeResult<T> {
    pub value: T,
    pub flow: FlowState,
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
    stable_record_row_lacks: HashSet<(crate::types::id::TypeParameterId, Box<str>)>,
    pub scopes: Vec<HashMap<String, LocalBindingInfo>>,
    pub flow: FlowState,
    pub(crate) normal_return_exits: Vec<crate::checker::analysis::NormalReturnFact>,
    pub(crate) throw_exit_flows: Vec<crate::checker::analysis::FlowStateSummary>,
    /// Binding products remain queryable after a branch-local scope closes.
    pub binding_history: BTreeMap<BindingId, crate::checker::analysis::BindingState>,
    pub(crate) loop_frames: Vec<LoopFlowFrame>,
    pub body_id: BodyId,
    pub next_local_expr_id: u32,
    pub next_diagnostic_cause: u32,
    pub next_analysis_incident: u32,
    pub next_binding_id: u32,
    pub expressions: ExpressionAnalysisIndex,
    pub associated_resolutions: crate::checker::associated::AssociatedResolutionIndex,
    pub family_applications: crate::checker::associated::FamilyApplicationResolutionIndex,
    pub match_resolutions: crate::match_semantics::MatchResolutionIndex,
    pub explanations: crate::explain::ExplanationArena,
    expression_owners: Vec<ExpressionId>,
    expression_owned_causes: BTreeMap<ExpressionId, crate::identity::DiagnosticCauseId>,
    resolved_callables: BTreeMap<ExpressionId, CallableId>,
    call_dependency_frames: Vec<CallDependencyFrame>,
    pub flow_graph: Option<std::sync::Arc<crate::checker::flow::graph::FlowGraph>>,
    pub dependencies: BTreeSet<CallableId>,
    semantic_dependencies: SharedSemanticDependencies,
    field_signatures: Option<&'a FieldSignatureTable>,
    field_lifecycle: Option<&'a crate::checker::field_lifecycle::FieldLifecycleTable>,
    pub enum_table: Option<&'a crate::enum_semantics::EnumSemanticTable>,
    pub associated_table: Option<&'a crate::associated::AssociatedFamilyTable>,
    pub dispatch: DispatchAccess<'a>,
    pub core_ids: CoreDeclarationIds,
    pub(crate) rigids: RigidArena,
    /// Active branch-local constraints used only while checking one branch.
    pub(crate) active_local_constraints: Vec<LocalConstraint>,
    /// Query-local type views for live bindings. Durable flow state remains
    /// canonical and cannot contain rigid variables.
    local_binding_types: BTreeMap<BindingId, LocalType>,
    /// Local types read by the currently analyzed closure from an enclosing
    /// scope. Closure environments are not existential packages, so these
    /// values must be rejected before the closure is published.
    closure_capture_frames: Vec<(usize, Vec<LocalType>)>,
    pub diagnostics: Vec<SemanticDiagnostic>,
    pub analysis_incidents: BTreeMap<AnalysisIncidentId, InternalSemanticIncident>,
    pub terminal_status: Option<AnalysisStatus>,
    inference_contexts: BTreeMap<crate::checker::inference::InferenceContextId, Rc<RefCell<crate::checker::inference::InferenceSession>>>,
    next_inference_context_id: u32,
    inference_frames: Vec<(crate::checker::inference::InferenceContextId, crate::checker::inference::InferenceFrameId)>,
    symbolic_inference_results: BTreeMap<ExpressionId, crate::checker::typed_expr::SymbolicInferenceResult>,
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
                .unwrap_or_else(|| crate::core_surface::universe_declaration(record.owner()));
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
            hierarchy: TrackingTypeHierarchy::new(hierarchy, declarations, semantic_dependencies.clone()),
            resolver: TrackingTypeResolver::new(resolver, semantic_dependencies.clone()),
            declarations,
            current_module,
            current_class: None,
            current_side: DispatchSide::Instance,
            current_callable: None,
            internal_failure_policy: InternalFailurePolicy::Contain,
            control: CheckerControl::default(),
            expected_return: None,
            stable_record_row_lacks: HashSet::new(),
            scopes: vec![HashMap::new()],
            flow: FlowState::new(),
            normal_return_exits: Vec::new(),
            throw_exit_flows: Vec::new(),
            binding_history: BTreeMap::new(),
            loop_frames: Vec::new(),
            body_id: BodyId(0),
            next_local_expr_id: 0,
            next_diagnostic_cause: 0,
            next_analysis_incident: 0,
            next_binding_id: 0,
            expressions: ExpressionAnalysisIndex::new(),
            associated_resolutions: crate::checker::associated::AssociatedResolutionIndex::new(),
            family_applications: crate::checker::associated::FamilyApplicationResolutionIndex::new(),
            match_resolutions: crate::match_semantics::MatchResolutionIndex::new(),
            explanations: crate::explain::ExplanationArena::new(),
            expression_owners: Vec::new(),
            expression_owned_causes: BTreeMap::new(),
            resolved_callables: BTreeMap::new(),
            call_dependency_frames: Vec::new(),
            flow_graph: None,
            dependencies: BTreeSet::new(),
            semantic_dependencies,
            field_signatures: None,
            field_lifecycle: None,
            enum_table: None,
            associated_table: None,
            dispatch: DispatchAccess::Owned(dispatch),
            core_ids: CoreDeclarationIds::default(),
            rigids: RigidArena::new(),
            active_local_constraints: Vec::new(),
            local_binding_types: BTreeMap::new(),
            closure_capture_frames: Vec::new(),

            diagnostics: Vec::new(),
            analysis_incidents: BTreeMap::new(),
            terminal_status: None,
            inference_contexts: BTreeMap::new(),
            next_inference_context_id: 0,
            inference_frames: Vec::new(),
            symbolic_inference_results: BTreeMap::new(),
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
            hierarchy: TrackingTypeHierarchy::new(hierarchy, declarations, semantic_dependencies.clone()),
            resolver: TrackingTypeResolver::new(resolver, semantic_dependencies.clone()),
            declarations,
            current_module,
            current_class: None,
            current_side: DispatchSide::Instance,
            current_callable: None,
            internal_failure_policy: InternalFailurePolicy::Contain,
            control,
            expected_return: None,
            stable_record_row_lacks: HashSet::new(),
            scopes: vec![HashMap::new()],
            flow: FlowState::new(),
            normal_return_exits: Vec::new(),
            throw_exit_flows: Vec::new(),
            binding_history: BTreeMap::new(),
            loop_frames: Vec::new(),
            body_id: BodyId(0),
            next_local_expr_id: 0,
            next_diagnostic_cause: 0,
            next_analysis_incident: 0,
            next_binding_id: 0,
            expressions: ExpressionAnalysisIndex::new(),
            associated_resolutions: crate::checker::associated::AssociatedResolutionIndex::new(),
            family_applications: crate::checker::associated::FamilyApplicationResolutionIndex::new(),
            match_resolutions: crate::match_semantics::MatchResolutionIndex::new(),
            explanations: crate::explain::ExplanationArena::new(),
            expression_owners: Vec::new(),
            expression_owned_causes: BTreeMap::new(),
            resolved_callables: BTreeMap::new(),
            call_dependency_frames: Vec::new(),
            flow_graph: None,
            dependencies: BTreeSet::new(),
            semantic_dependencies,
            field_signatures: None,
            field_lifecycle: None,
            enum_table: None,
            associated_table: None,
            dispatch: DispatchAccess::Borrowed(dispatch),
            core_ids: CoreDeclarationIds::default(),
            rigids: RigidArena::new(),
            active_local_constraints: Vec::new(),
            local_binding_types: BTreeMap::new(),
            closure_capture_frames: Vec::new(),
            diagnostics: Vec::new(),
            analysis_incidents: BTreeMap::new(),
            terminal_status: None,
            inference_contexts: BTreeMap::new(),
            next_inference_context_id: 0,
            inference_frames: Vec::new(),
            symbolic_inference_results: BTreeMap::new(),
        }
    }

    pub fn with_resolver<'b>(&'b mut self, resolver: &'b dyn TypeResolver) -> CheckingContext<'b> {
        CheckingContext {
            store: self.store,
            hierarchy: TrackingTypeHierarchy::new(self.hierarchy.inner, self.declarations, self.semantic_dependencies.clone()),
            resolver: TrackingTypeResolver::new(resolver, self.semantic_dependencies.clone()),
            declarations: self.declarations,
            current_module: self.current_module.clone(),
            current_class: self.current_class.clone(),
            current_side: self.current_side,
            current_callable: self.current_callable.clone(),
            internal_failure_policy: self.internal_failure_policy,
            control: self.control.clone(),
            expected_return: self.expected_return.clone(),
            stable_record_row_lacks: self.stable_record_row_lacks.clone(),
            scopes: self.scopes.clone(),
            flow: self.flow.clone(),
            normal_return_exits: self.normal_return_exits.clone(),
            throw_exit_flows: self.throw_exit_flows.clone(),
            binding_history: self.binding_history.clone(),
            loop_frames: self.loop_frames.clone(),
            body_id: self.body_id,
            next_local_expr_id: self.next_local_expr_id,
            next_diagnostic_cause: self.next_diagnostic_cause,
            next_analysis_incident: self.next_analysis_incident,
            next_binding_id: self.next_binding_id,
            expressions: self.expressions.clone(),
            associated_resolutions: self.associated_resolutions.clone(),
            family_applications: self.family_applications.clone(),
            match_resolutions: self.match_resolutions.clone(),
            explanations: self.explanations.clone(),
            expression_owners: self.expression_owners.clone(),
            expression_owned_causes: self.expression_owned_causes.clone(),
            resolved_callables: self.resolved_callables.clone(),
            call_dependency_frames: self.call_dependency_frames.clone(),
            flow_graph: self.flow_graph.clone(),
            dependencies: self.dependencies.clone(),
            semantic_dependencies: self.semantic_dependencies.clone(),
            field_signatures: self.field_signatures,
            field_lifecycle: self.field_lifecycle,
            enum_table: self.enum_table,
            associated_table: self.associated_table,
            dispatch: DispatchAccess::Borrowed(self.dispatch.get()),
            core_ids: self.core_ids.clone(),
            rigids: self.rigids.clone(),
            active_local_constraints: self.active_local_constraints.clone(),
            local_binding_types: self.local_binding_types.clone(),
            closure_capture_frames: self.closure_capture_frames.clone(),

            diagnostics: Vec::new(),

            analysis_incidents: self.analysis_incidents.clone(),
            terminal_status: self.terminal_status.clone(),
            inference_contexts: self.inference_contexts.clone(),
            next_inference_context_id: self.next_inference_context_id,
            inference_frames: self.inference_frames.clone(),
            symbolic_inference_results: self.symbolic_inference_results.clone(),
        }
    }

    /// Runs a speculative flow probe in an isolated child context.
    /// Child-local products (diagnostics, derivations, callable exits, expressions,
    /// dependencies) are discarded while the canonical TypeStore is updated monotonically
    /// and CheckerControl budget/cancellation are shared.
    pub fn run_flow_probe<T>(&mut self, entry: FlowState, run: impl FnOnce(&mut CheckingContext<'_>) -> T) -> FlowProbeResult<T> {
        let mut probe = CheckingContext::new_with_dispatch_ref_and_control(
            self.store,
            self.hierarchy.inner(),
            self.resolver.inner(),
            self.declarations,
            self.dispatch.get(),
            self.current_module.clone(),
            self.control.clone(),
        );

        probe.current_class = self.current_class.clone();
        probe.current_side = self.current_side;
        probe.current_callable = self.current_callable.clone();
        probe.expected_return = self.expected_return.clone();
        probe.stable_record_row_lacks = self.stable_record_row_lacks.clone();
        probe.scopes = self.scopes.clone();
        probe.flow = entry;
        probe.body_id = self.body_id;
        probe.field_signatures = self.field_signatures;
        probe.field_lifecycle = self.field_lifecycle;
        probe.enum_table = self.enum_table;
        probe.associated_table = self.associated_table;
        probe.binding_history = self.binding_history.clone();
        probe.next_binding_id = self.next_binding_id;
        probe.next_local_expr_id = self.next_local_expr_id;
        probe.next_diagnostic_cause = self.next_diagnostic_cause;
        probe.next_analysis_incident = self.next_analysis_incident;
        probe.inference_contexts = self.inference_contexts.clone();
        probe.next_inference_context_id = self.next_inference_context_id;
        probe.inference_frames = self.inference_frames.clone();
        probe.symbolic_inference_results = self.symbolic_inference_results.clone();
        probe.rigids = self.rigids.clone();
        probe.active_local_constraints = self.active_local_constraints.clone();
        probe.local_binding_types = self.local_binding_types.clone();
        probe.closure_capture_frames = self.closure_capture_frames.clone();

        let value = run(&mut probe);
        let flow = probe.flow;
        self.rigids = probe.rigids;

        FlowProbeResult { value, flow }
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

    pub fn current_flow_summary(&self) -> crate::checker::analysis::FlowStateSummary {
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
                            validity: state.validity.clone(),
                            causal_invalidity: state.causal_invalidity,
                        },
                    )
                })
                .collect(),

            fact_count: self.flow.facts.len(),
        }
    }

    pub fn record_return_exit(&mut self, fact: crate::checker::analysis::NormalReturnFact) {
        self.normal_return_exits.push(fact);
        self.flow.mark_unreachable();
    }

    #[allow(dead_code)]
    pub(crate) fn record_throw_exit(&mut self) {
        self.record_throw_exit_and_terminate();
    }

    pub(crate) fn record_throw_exit_and_terminate(&mut self) {
        self.throw_exit_flows.push(self.current_flow_summary());
        self.flow.mark_unreachable();
    }

    pub(crate) fn record_break_and_terminate(&mut self) {
        self.record_break();
        self.flow.mark_unreachable();
    }

    pub(crate) fn record_continue_and_terminate(&mut self) {
        self.record_continue();
        self.flow.mark_unreachable();
    }

    pub fn normal_return_exits(&self) -> &[crate::checker::analysis::NormalReturnFact] {
        &self.normal_return_exits
    }

    pub(crate) fn take_normal_return_exits(&mut self) -> Vec<crate::checker::analysis::NormalReturnFact> {
        std::mem::take(&mut self.normal_return_exits)
    }

    pub(crate) fn solve_inference(&mut self, session: &mut crate::checker::inference::InferenceSession) -> crate::checker::inference::InferenceOutcome {
        session.solve_with_control(self.store, &self.hierarchy, &self.control)
    }

    pub(crate) fn propagate_inference(
        &mut self,
        session: &mut crate::checker::inference::InferenceSession,
    ) -> Result<bool, crate::checker::inference::InferenceOutcome> {
        session.propagate_with_control(self.store, &self.hierarchy, &self.control)
    }

    pub(crate) fn solve_inference_in_frame(
        &mut self,
        session: &mut crate::checker::inference::InferenceSession,
        frame: crate::checker::inference::InferenceFrameId,
    ) -> crate::checker::inference::InferenceOutcome {
        match session.propagate_with_control(self.store, &self.hierarchy, &self.control) {
            Ok(_) => session.finish_frame(frame),
            Err(outcome) => outcome,
        }
    }

    /// Creates one query-local inference graph and its root application frame.
    pub(crate) fn create_inference_context(&mut self) -> (crate::checker::inference::InferenceContextId, crate::checker::inference::InferenceFrameId) {
        let mut context = crate::checker::inference::InferenceContextId(self.next_inference_context_id);
        while self.inference_contexts.contains_key(&context) {
            self.next_inference_context_id = self.next_inference_context_id.saturating_add(1);
            context = crate::checker::inference::InferenceContextId(self.next_inference_context_id);
        }
        self.next_inference_context_id = self.next_inference_context_id.saturating_add(1);
        let mut session = crate::checker::inference::InferenceSession::new();
        let frame = session.root_frame();
        self.inference_contexts.insert(context, Rc::new(RefCell::new(session)));
        (context, frame)
    }

    /// Returns a cloneable handle to an active query-local inference graph.
    pub(crate) fn inference_session(
        &self,
        context: crate::checker::inference::InferenceContextId,
    ) -> Option<Rc<RefCell<crate::checker::inference::InferenceSession>>> {
        self.inference_contexts.get(&context).cloned()
    }

    /// Begins a child application frame in an existing query-local graph.
    pub(crate) fn begin_inference_frame(
        &self,
        context: crate::checker::inference::InferenceContextId,
        parent: crate::checker::inference::InferenceFrameId,
    ) -> Option<crate::checker::inference::InferenceFrameId> {
        let session = self.inference_contexts.get(&context)?.clone();
        let session_state = session.borrow();
        if session_state.frame_is_closed(parent) || (parent.0 != 0 && session_state.frame_parent(parent).is_none()) {
            return None;
        }
        drop(session_state);
        Some(session.borrow_mut().begin_frame(Some(parent)))
    }

    pub(crate) fn current_inference_frame(
        &self,
        context: crate::checker::inference::InferenceContextId,
    ) -> Option<crate::checker::inference::InferenceFrameId> {
        self.inference_frames
            .iter()
            .rev()
            .find_map(|(active_context, frame)| (*active_context == context).then_some(*frame))
    }

    pub(crate) fn push_inference_frame(&mut self, context: crate::checker::inference::InferenceContextId, frame: crate::checker::inference::InferenceFrameId) {
        self.inference_frames.push((context, frame));
    }

    pub(crate) fn pop_inference_frame(&mut self, context: crate::checker::inference::InferenceContextId, frame: crate::checker::inference::InferenceFrameId) {
        if let Some(index) = self
            .inference_frames
            .iter()
            .rposition(|(active_context, active_frame)| *active_context == context && *active_frame == frame)
        {
            self.inference_frames.remove(index);
        }
    }

    pub(crate) fn publish_symbolic_inference_result(&mut self, expression: ExpressionId, result: crate::checker::typed_expr::SymbolicInferenceResult) {
        self.symbolic_inference_results.insert(expression, result);
    }

    pub(crate) fn take_symbolic_inference_result(&mut self, expression: ExpressionId) -> Option<crate::checker::typed_expr::SymbolicInferenceResult> {
        self.symbolic_inference_results.remove(&expression)
    }

    /// Drops a completed query-local graph. No solver-local identity survives
    /// this boundary into snapshots or other durable semantic products.
    pub(crate) fn finish_inference_context(&mut self, context: crate::checker::inference::InferenceContextId) {
        self.inference_contexts.remove(&context);
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

    pub(crate) fn seed_stable_record_row_lacks(&mut self, facts: impl IntoIterator<Item = crate::checker::row_inference::StableRecordRowLack>) {
        for fact in facts {
            self.stable_record_row_lacks.insert((fact.parameter, fact.field));
        }
    }

    pub(crate) fn stable_record_row_lacks(&self, parameter: crate::types::id::TypeParameterId, field: &str) -> bool {
        self.stable_record_row_lacks
            .iter()
            .any(|(candidate, name)| *candidate == parameter && name.as_ref() == field)
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

    pub(crate) fn record_call_local_type(&mut self, ty: TypeId, local_type: LocalType) {
        if let Some(frame) = self.call_dependency_frames.last_mut() {
            if !frame.local_types.iter().any(|(existing, local)| *existing == ty && *local == local_type) {
                frame.local_types.push((ty, local_type));
            }
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

    pub(crate) fn apply_flow_predicate(&mut self, predicate: &crate::checker::flow::TrustedFlowPredicate) -> crate::checker::flow::PredicateTransfer {
        let prior_parent = predicate
            .predicate
            .binding()
            .and_then(|binding| self.flow.get_binding(binding))
            .and_then(|state| state.explanation);
        let hierarchy = &self.hierarchy;
        let outcome = crate::checker::flow::transfer::apply_predicate(&mut self.flow, predicate, self.store, hierarchy);
        match &outcome {
            crate::checker::flow::PredicateTransfer::Unchanged => {}
            crate::checker::flow::PredicateTransfer::Contradiction { .. } => {
                self.flow.mark_unreachable();
            }
            crate::checker::flow::PredicateTransfer::Refined(applied) => {
                let predicate_kind = match &predicate.predicate {
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
                        prior: applied.prior.clone(),
                        refined: applied.refined.clone(),
                    },
                    crate::explain::DerivationRule::FlowRefinement { predicate_kind },
                    applied.refined.status().unwrap_or(crate::types::evidence::EvidenceStatus::Established),
                    crate::types::evidence::EvidenceOrigin::Flow,
                    Vec::new(),
                    prior_parent.into_iter().collect(),
                );
                self.flow.facts.insert(predicate.predicate.clone(), explanation);
                self.flow.set_binding_explanation(applied.binding, explanation);
                if let Some(state) = self.flow.get_binding(applied.binding).cloned() {
                    self.binding_history.insert(applied.binding, state);
                }
            }
        }
        outcome
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

    pub(crate) fn end_call_causal_capture(&mut self) -> CallCapture {
        let frame = self.call_dependency_frames.pop().unwrap_or_default();
        (frame.causal_invalidity, frame.explanations, frame.status, frame.local_types)
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
        let site = if let Some(owner) = self.current_class.clone() {
            TypeFormationSite::member(self.current_module.clone(), owner, self.current_side)
        } else {
            TypeFormationSite::module(self.current_module.clone())
        };
        let knowledge = crate::types::annotation::resolve_type_annotation(self.store, self.declarations, resolver, &site, annotation, &mut diagnostics);
        let causal_invalidity = self.publish_diagnostics(diagnostics);
        (knowledge, causal_invalidity)
    }

    /// Resolves one expression type form without enforcing proper `Type` kind.
    ///
    /// Type-form expressions may denote constructors or lambdas. Their value
    /// descriptor type is assigned by the expression checker after this exact
    /// formation outcome is returned.
    pub fn resolve_type_form(
        &mut self,
        resolver: &dyn TypeResolver,
        site: &TypeFormationSite,
        annotation: &phalcom_ast::ast::TypeAnnotation,
    ) -> (TypeFormResolution, crate::checker::causal::CausalInvalidity) {
        let mut diagnostics = Vec::new();
        let resolution = crate::types::annotation::resolve_type_form(self.store, self.declarations, resolver, site, annotation, &mut diagnostics);
        let causal_invalidity = self.publish_diagnostics(diagnostics);
        (resolution, causal_invalidity)
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
        analysis.denotation = typed.denotation.clone();
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
        analysis.denotation = typed.denotation.clone();
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
        let denotation = seed.denotation.clone();
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

    pub(crate) fn binding_is_in_current_scope(&self, binding: BindingId) -> bool {
        self.scopes
            .last()
            .is_some_and(|scope| scope.values().any(|info| info.id == binding))
    }

    pub(crate) fn local_binding_type(&self, binding: BindingId) -> Option<&LocalType> {
        self.local_binding_types.get(&binding)
    }

    pub(crate) fn set_local_binding_type(&mut self, binding: BindingId, local_type: LocalType) {
        self.local_binding_types.insert(binding, local_type);
    }

    pub(crate) fn begin_closure_capture(&mut self) {
        self.closure_capture_frames.push((self.scopes.len(), Vec::new()));
    }

    pub(crate) fn record_local_capture(&mut self, binding: BindingId, local_type: LocalType) {
        let Some(boundary) = self.closure_capture_frames.last().map(|(boundary, _)| *boundary) else {
            return;
        };
        let is_outer_binding = self.scopes.iter().take(boundary).any(|scope| scope.values().any(|info| info.id == binding));
        if is_outer_binding {
            if let Some((_, captured)) = self.closure_capture_frames.last_mut() {
                if !captured.contains(&local_type) {
                    captured.push(local_type);
                }
            }
        }
    }

    pub(crate) fn end_closure_capture(&mut self) -> Vec<LocalType> {
        self.closure_capture_frames.pop().map(|(_, captured)| captured).unwrap_or_default()
    }

    /// Checks whether a query-local type may cross the current scope boundary.
    /// Canonical flow and metadata products never receive the local type; a
    /// successful check means the caller may publish its rigid-free contract
    /// view instead.
    pub(crate) fn check_local_type_escape(
        &mut self,
        local_type: Option<&LocalType>,
        expected: Option<TypeId>,
        additional_constraints: &[LocalConstraint],
        range: SourceRange,
    ) -> bool {
        let Some(local_type) = local_type else {
            return true;
        };
        if local_type.free_rigids().is_empty() {
            return true;
        }

        let mut constraints = self.active_local_constraints.clone();
        constraints.extend_from_slice(additional_constraints);
        if let Some(expected) = expected {
            if local_type_is_soundly_widenable(self, local_type, expected, &constraints) {
                return true;
            }
        }

        let rigids = local_type
            .free_rigids()
            .into_iter()
            .map(|rigid| format!("κ{}", rigid.0))
            .collect::<Vec<_>>()
            .join(", ");
        let outward = expected
            .map(|ty| self.store.format_type(ty))
            .unwrap_or_else(|| "inferred result".to_string());
        self.emit_diagnostic(SemanticDiagnostic::error_in(
            self.current_module.clone(),
            crate::diagnostic::DiagnosticCode::ExistentialEscape,
            format!("branch-local type {rigids} escapes into {outward}"),
            range,
        ));
        false
    }

    pub fn lookup_local(&self, name: &str) -> Option<ValueSemanticFact> {
        let info = self.lookup_binding_info(name)?;
        let state = self.flow.get_binding(info.id)?;
        Some(ValueSemanticFact {
            knowledge: state.current.clone(),
            denotation: state.denotation.clone(),
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
        if !is_query_owned_module(callable.module()) || is_bootstrap_declaration(callable.declaration_owner()) || is_builtin_type_test_callable(callable) {
            return;
        }
        record_declaration_surface_dependency(&self.semantic_dependencies, callable.declaration_owner());
        self.record_semantic_dependency(SemanticDependency::CallableSignature(callable.clone()));
    }

    /// Attaches compiler-owned canonical field declaration knowledge.
    pub fn attach_field_signatures(&mut self, field_signatures: &'a FieldSignatureTable) {
        self.field_signatures = Some(field_signatures);
    }

    /// Attaches compiler-owned instance field lifecycle table.
    pub fn attach_field_lifecycle(&mut self, field_lifecycle: &'a crate::checker::field_lifecycle::FieldLifecycleTable) {
        self.field_lifecycle = Some(field_lifecycle);
    }

    /// Attaches compiler-owned enum semantics table.
    pub fn attach_enum_semantics(&mut self, enum_table: &'a crate::enum_semantics::EnumSemanticTable) {
        self.enum_table = Some(enum_table);
    }

    /// Attaches compiler-owned associated family table.
    pub fn attach_associated_families(&mut self, associated_table: &'a crate::associated::AssociatedFamilyTable) {
        self.associated_table = Some(associated_table);
    }

    /// Reads enum metadata while recording the enum-declaration dependency.
    pub fn enum_info(&self, owner: &DeclarationId) -> Option<&crate::enum_semantics::EnumInfo> {
        self.record_enum_declaration_dependency(owner);
        self.enum_table.and_then(|t| t.enums.get(owner).map(|arc| &**arc))
    }

    /// Reads variant metadata while recording the enum-declaration dependency.
    pub fn variant_info(&self, variant: &crate::identity::VariantId) -> Option<&crate::enum_semantics::VariantInfo> {
        self.record_enum_declaration_dependency(&variant.owner);
        self.enum_table.and_then(|t| t.variants.get(variant).map(|arc| &**arc))
    }

    /// Reads associated family surface while recording the associated-surface dependency.
    pub fn associated_surface(&self, owner: &DeclarationId) -> Option<&crate::associated::AssociatedSurface> {
        self.record_associated_surface_dependency(owner);
        self.associated_table.and_then(|t| t.surfaces.get(owner).map(|arc| &**arc))
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
                TypeData::ExactCase { enum_type, .. } => current = *enum_type,
                TypeData::Nominal { declaration } | TypeData::ClassObject { declaration } => return Some(declaration.clone()),
                _ => return None,
            }
        }
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

    /// Materializes one declaration-owned type template under an actual
    /// receiver, including an applied class-side type form.
    pub(crate) fn specialize_type_to_receiver(&mut self, receiver: TypeId, owner: &DeclarationId, ty: TypeId) -> Option<TypeId> {
        let specialization = specialize_receiver_to_owner(self.store, &self.hierarchy, receiver, owner, &self.control).ok()?;
        Some(crate::types::environment::TypeView::new(ty, specialization.environment).materialize(self.store))
    }

    pub(crate) fn dispatch_owner_for_lookup(&self, receiver: TypeId, lookup: crate::dispatch::DispatchLookup) -> Option<(DeclarationId, DispatchSide)> {
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
                TypeData::ExactCase { enum_type, .. } => self.dispatch_owner_for_lookup(*enum_type, lookup),
                _ => None,
            },
        }
    }

    fn specialize_dispatch_signature(
        &mut self,
        receiver: TypeId,
        declaring_owner: &DeclarationId,
        mut signature: crate::dispatch::CallableSignature,
    ) -> Result<(crate::dispatch::CallableSignature, ReceiverSpecialization), ReceiverSpecializationFailure> {
        let specialization = specialize_receiver_to_owner(&mut *self.store, &self.hierarchy, receiver, declaring_owner, &self.control)?;
        let environment = specialization.environment.clone();
        let specialize_type = |ctx: &mut Self, ty| {
            let self_specialized = ctx.specialize_self_type(receiver, ty);
            crate::types::environment::TypeView::new(self_specialized, environment.clone()).materialize(ctx.store)
        };
        for parameter in &mut signature.parameters {
            parameter.ty = parameter.ty.map_type(|ty| specialize_type(self, ty));
        }
        signature.return_type = signature.return_type.map_type(|ty| specialize_type(self, ty));
        if let Some(mut generics) = signature.generics.take() {
            let specialize_term = |ctx: &mut Self, term: &crate::types::parameter::TypeTerm| match term {
                crate::types::parameter::TypeTerm::Canonical(ty) => crate::types::parameter::TypeTerm::Canonical(specialize_type(ctx, *ty)),
                crate::types::parameter::TypeTerm::SelfType(self_term) => {
                    let self_ty = ctx.store.self_type(self_term.clone());
                    crate::types::parameter::TypeTerm::Canonical(ctx.specialize_self_type(receiver, self_ty))
                }
                crate::types::parameter::TypeTerm::Infer(variable) => crate::types::parameter::TypeTerm::Infer(*variable),
            };
            generics.constraints = generics
                .constraints
                .iter()
                .map(|constraint| match constraint {
                    crate::types::parameter::GenericConstraint::Subtype { lower, upper } => crate::types::parameter::GenericConstraint::Subtype {
                        lower: specialize_term(self, lower),
                        upper: specialize_term(self, upper),
                    },
                    crate::types::parameter::GenericConstraint::Equivalent { left, right } => crate::types::parameter::GenericConstraint::Equivalent {
                        left: specialize_term(self, left),
                        right: specialize_term(self, right),
                    },
                })
                .collect();
            let constraint_mentions_parameter = |term: &crate::types::parameter::TypeTerm, parameter| match term {
                crate::types::parameter::TypeTerm::Canonical(ty) => self.store.contains_type_parameter(*ty, parameter),
                crate::types::parameter::TypeTerm::SelfType(_) | crate::types::parameter::TypeTerm::Infer(_) => false,
            };
            let remains_in_signature = generics.parameters.iter().copied().any(|parameter| {
                signature
                    .parameters
                    .iter()
                    .filter_map(|parameter| parameter.ty.ty())
                    .any(|ty| self.store.contains_type_parameter(ty, parameter))
                    || signature.return_type.ty().is_some_and(|ty| self.store.contains_type_parameter(ty, parameter))
                    || generics.constraints.iter().any(|constraint| match constraint {
                        crate::types::parameter::GenericConstraint::Subtype { lower, upper }
                        | crate::types::parameter::GenericConstraint::Equivalent { left: lower, right: upper } => {
                            constraint_mentions_parameter(lower, parameter) || constraint_mentions_parameter(upper, parameter)
                        }
                    })
            });
            signature.generics = remains_in_signature.then_some(generics);
        }
        Ok((signature, specialization))
    }

    pub(crate) fn resolve_dispatch_target(&mut self, receiver: TypeId, selector: &Selector, lookup: crate::dispatch::DispatchLookup) -> ResolvedDispatchResult {
        self.resolve_dispatch_target_with_specialization(receiver, None, selector, lookup)
    }

    /// Resolves dispatch against `dispatch_receiver` while optionally
    /// specializing the selected signature against a distinct proper type
    /// form.  Class-object values carry dispatch identity, whereas an applied
    /// class type form carries declaration arguments needed by class-side
    /// templates.  Keeping those inputs separate preserves both facts.
    pub(crate) fn resolve_dispatch_target_with_specialization(
        &mut self,
        dispatch_receiver: TypeId,
        specialization_receiver: Option<TypeId>,
        selector: &Selector,
        lookup: crate::dispatch::DispatchLookup,
    ) -> ResolvedDispatchResult {
        let Some((decl, side)) = self.dispatch_owner_for_lookup(dispatch_receiver, lookup.clone()) else {
            return ResolvedDispatchResult::Missing { visited_owners: Box::new([]) };
        };

        let result = self.dispatch.get().resolve_dispatch_with_trace(&self.hierarchy, &decl, side, selector);
        // A raw class type form carries declaration parameters only as a
        // template. Keep class-object `Self` formation for that case; applied
        // forms carry the actual receiver arguments used for specialization.
        let specialization_receiver = specialization_receiver
            .filter(|receiver| matches!(self.store.get(*receiver), TypeData::Applied { .. } | TypeData::ExactCase { .. }))
            .unwrap_or(dispatch_receiver);
        match result {
            ResolvedDispatchResult::Found(mut resolved) => {
                for owner in resolved.visited_owners.iter() {
                    record_declaration_surface_dependency(&self.semantic_dependencies, owner);
                }
                self.dependencies.insert(resolved.callable.clone());
                self.record_consumed_callable_signature(&resolved.callable, &resolved.signature);
                let unspecialized_return = resolved.signature.return_type.clone();
                let declaring_owner = resolved.callable.declaration_owner().clone();
                let Ok((signature, specialization)) = self.specialize_dispatch_signature(specialization_receiver, &declaring_owner, resolved.signature) else {
                    return ResolvedDispatchResult::Dynamic;
                };
                resolved.specialization = Some(crate::dispatch::DispatchSignatureSpecialization {
                    receiver: specialization_receiver,
                    declaring_owner,
                    environment: specialization.environment,
                    path: specialization.path,
                    unspecialized_return,
                });
                resolved.signature = signature;
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
                    let unspecialized_return = resolved.signature.return_type.clone();
                    let declaring_owner = resolved.callable.declaration_owner().clone();
                    let Ok((signature, specialization)) =
                        self.specialize_dispatch_signature(specialization_receiver, &declaring_owner, resolved.signature.clone())
                    else {
                        return ResolvedDispatchResult::Dynamic;
                    };
                    resolved.specialization = Some(crate::dispatch::DispatchSignatureSpecialization {
                        receiver: specialization_receiver,
                        declaring_owner,
                        environment: specialization.environment,
                        path: specialization.path,
                        unspecialized_return,
                    });
                    resolved.signature = signature;
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

    pub(crate) fn resolve_field_read(
        &self,
        owner: &DeclarationId,
        side: DispatchSide,
        name: &str,
    ) -> Option<(crate::identity::FieldId, TypeKnowledge, crate::checker::causal::CausalInvalidity)> {
        let (field, contract) = self.resolve_field_contract(owner, side, name)?;
        if self.current_class.as_ref() == Some(owner) && self.current_side == side {
            if let Some(state) = self.flow.get_field(&field) {
                return Some((field, state.current.clone(), state.causal_invalidity));
            }
        }
        if let Some(fact) = self.field_lifecycle.and_then(|t| t.fields.get(&field)) {
            return Some((field, fact.read_knowledge.clone(), fact.causal_invalidity));
        }
        Some((field, contract, crate::checker::causal::CausalInvalidity::Clean))
    }

    pub(crate) fn resolve_current_field(
        &self,
        owner: &DeclarationId,
        side: DispatchSide,
        name: &str,
    ) -> Option<(crate::identity::FieldId, TypeKnowledge, crate::checker::causal::CausalInvalidity)> {
        self.resolve_field_read(owner, side, name)
    }

    pub(crate) fn write_current_field(
        &mut self,
        field: crate::identity::FieldId,
        contract: TypeKnowledge,
        current: TypeKnowledge,
        validity: crate::checker::flow::FieldContractValidity,
        causal_invalidity: crate::checker::causal::CausalInvalidity,
    ) {
        if self.flow.get_field(&field).is_none() {
            self.flow.seed_field(crate::checker::flow::FieldState {
                field: field.clone(),
                contract,
                current: TypeKnowledge::Unknown(UnknownReason::MissingInitializer),
                initialization: crate::checker::flow::FieldInitialization::Uninitialized,
                validity: crate::checker::flow::FieldContractValidity::Unchecked,
                causal_invalidity: crate::checker::causal::CausalInvalidity::Clean,
                version: 0,
            });
        }
        self.flow.write_field(
            &field,
            current,
            crate::checker::flow::FieldInitialization::DefinitelyInitialized,
            validity,
            causal_invalidity,
        );
    }

    pub fn resolve_type_name(&self, name: &str) -> Option<DeclarationId> {
        self.resolver.resolve_type_name(&self.current_module, name, &[])
    }

    pub fn resolve_type_parameter(&self, name: &str) -> Option<TypeId> {
        self.resolver.resolve_type_parameter(name)
    }

    pub fn nominal_type_of(&mut self, decl: &DeclarationId) -> Option<TypeId> {
        record_declaration_shell_dependency(&self.semantic_dependencies, decl);
        self.declarations.form(decl)
    }

    /// Returns the proper instance type for a declaration in its generic body.
    ///
    /// A declaration form is intentionally constructor-kinded while its generic
    /// parameters are unsaturated. An instance expression such as `self` must
    /// carry that declaration's canonical parameter forms instead of exposing
    /// the constructor as a value type.
    pub fn instance_type_of(&mut self, decl: &DeclarationId) -> Option<TypeId> {
        record_declaration_shell_dependency(&self.semantic_dependencies, decl);
        let Some(info) = self.declarations.get(decl) else {
            return Some(self.store.nominal(decl.clone()));
        };
        let form = info.form;
        let Some(signature) = info.generic_signature.as_ref() else {
            return Some(form);
        };
        let parameters = signature.parameters.to_vec();
        let arguments = parameters
            .iter()
            .map(|&parameter| (self.store.type_parameter(parameter).kind == crate::types::id::KindId::TYPE).then(|| self.store.parameter_form(parameter)))
            .collect::<Option<Vec<_>>>()?;
        self.store.apply_type_form(form, &arguments).ok()
    }

    pub(crate) fn core_type(&mut self, decl: &DeclarationId) -> Option<TypeId> {
        self.declaration_info(decl).map(|info| info.form)
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
        mut self,
        callable: CallableId,
        body_range: SourceRange,
        status: crate::checker::analysis::CallableAnalysisStatus,
    ) -> crate::checker::analysis::CallableAnalysis {
        let normal_returns = self.take_normal_return_exits();
        self.finalize_with_normal_returns(callable, body_range, status, normal_returns)
    }

    pub fn finalize_with_normal_returns(
        mut self,
        callable: CallableId,
        body_range: SourceRange,
        status: crate::checker::analysis::CallableAnalysisStatus,
        normal_returns: Vec<crate::checker::analysis::NormalReturnFact>,
    ) -> crate::checker::analysis::CallableAnalysis {
        let return_validation = self.validate_return_contract(status, &normal_returns);
        let entry_flow = self.current_flow_summary();
        let flow_graph = self
            .flow_graph
            .unwrap_or_else(|| std::sync::Arc::new(crate::checker::flow::graph::FlowGraph::default()));

        let return_summary = crate::checker::analysis::normal_return_summary(self.store, &normal_returns);
        let return_explanation = Some(self.explanations.alloc(
            crate::explain::ExplanationStep::CallableReturnSummary {
                callable: callable.clone(),
                returns: normal_returns.iter().map(|exit| exit.knowledge.clone()).collect::<Vec<_>>().into_boxed_slice(),
                result: return_summary.clone(),
            },
            return_summary.status().unwrap_or(crate::types::evidence::EvidenceStatus::Assumed),
            return_summary.origin().unwrap_or(crate::types::evidence::EvidenceOrigin::Flow),
            Vec::new(),
        ));
        let exits = crate::checker::analysis::BodyExitFacts {
            normal_returns,
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
            associated_resolutions: std::sync::Arc::new(self.associated_resolutions),
            family_applications: std::sync::Arc::new(self.family_applications),
            match_resolutions: std::sync::Arc::new(self.match_resolutions),
            flow_graph,
            entry_flow,
            exits,
            return_validation,
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

    pub fn record_associated_resolution(&mut self, expr_id: ExpressionId, resolution: crate::checker::associated::AssociatedResolution) {
        self.associated_resolutions.insert(expr_id, resolution);
    }

    pub fn record_family_application(&mut self, expr_id: ExpressionId, resolution: crate::checker::associated::FamilyApplicationResolution) {
        self.family_applications.insert(expr_id, resolution);
    }

    pub fn record_match_resolution(&mut self, expr_id: ExpressionId, resolution: crate::match_semantics::MatchResolution) {
        self.match_resolutions.insert(expr_id, resolution);
    }

    pub fn record_enum_declaration_dependency(&self, decl: &DeclarationId) {
        self.semantic_dependencies
            .borrow_mut()
            .insert(crate::checker::analysis::SemanticDependency::EnumDeclaration(decl.clone()));
    }

    pub fn record_associated_surface_dependency(&self, decl: &DeclarationId) {
        self.semantic_dependencies
            .borrow_mut()
            .insert(crate::checker::analysis::SemanticDependency::AssociatedSurface(decl.clone()));
    }

    pub fn validate_return_contract(
        &mut self,
        status: crate::checker::analysis::CallableAnalysisStatus,
        normal_returns: &[crate::checker::analysis::NormalReturnFact],
    ) -> crate::signature::ReturnContractValidation {
        let Some(contract) = self.expected_return.clone() else {
            return crate::signature::ReturnContractValidation::NotApplicable;
        };
        if contract.basis != crate::declaration_type::DeclaredTypeBasis::SourceAnnotation {
            return crate::signature::ReturnContractValidation::NotApplicable;
        }
        if contract.is_dynamic {
            return crate::signature::ReturnContractValidation::DynamicBoundary;
        }
        if status != crate::checker::analysis::CallableAnalysisStatus::Complete {
            return crate::signature::ReturnContractValidation::Blocked;
        }
        if normal_returns.is_empty() {
            return crate::signature::ReturnContractValidation::Satisfied(crate::types::evidence::EvidenceStatus::Established);
        }

        let mut accumulated_status = crate::types::evidence::EvidenceStatus::Established;
        let mut has_dynamic = false;

        for exit in normal_returns {
            // 1. Raw relation check first when Known to detect genuine type refutation
            if let Some(_raw_ty) = exit.knowledge.ty() {
                let outcome = self.check_knowledge_against_type(&exit.knowledge, contract.ty);
                if matches!(outcome, RelationOutcome::Refuted(_)) {
                    return crate::signature::ReturnContractValidation::Refuted;
                }
            }

            // 2. Causal / status admissibility check
            if exit.causal_invalidity != crate::checker::causal::CausalInvalidity::Clean {
                return crate::signature::ReturnContractValidation::Blocked;
            }
            match &exit.status {
                AnalysisStatus::Ready => {}
                AnalysisStatus::DynamicBoundary(_) => {
                    has_dynamic = true;
                    continue;
                }
                AnalysisStatus::Blocked(_)
                | AnalysisStatus::Cancelled
                | AnalysisStatus::BudgetExceeded(_)
                | AnalysisStatus::InternalFailure(_)
                | AnalysisStatus::Invalid(_)
                | AnalysisStatus::Suppressed(_) => {
                    return crate::signature::ReturnContractValidation::Blocked;
                }
            }

            // 3. Admissible publication knowledge check
            let pub_k = exit.publication_knowledge();
            match &pub_k {
                TypeKnowledge::Dynamic(_) => {
                    has_dynamic = true;
                }
                TypeKnowledge::Unknown(_) => {
                    return crate::signature::ReturnContractValidation::Blocked;
                }
                TypeKnowledge::Known(evidence) => {
                    let outcome = self.check_knowledge_against_type(&pub_k, contract.ty);
                    match outcome {
                        RelationOutcome::Proven { .. } => {
                            accumulated_status = accumulated_status.meet(evidence.status());
                        }
                        RelationOutcome::Refuted(_) => {
                            return crate::signature::ReturnContractValidation::Refuted;
                        }
                        RelationOutcome::Blocked(_) | RelationOutcome::Cancelled | RelationOutcome::BudgetExceeded(_) | RelationOutcome::InternalFailure(_) => {
                            return crate::signature::ReturnContractValidation::Blocked;
                        }
                        RelationOutcome::DynamicBoundary(_) => {
                            has_dynamic = true;
                        }
                    }
                }
            }
        }

        if has_dynamic {
            crate::signature::ReturnContractValidation::DynamicBoundary
        } else {
            crate::signature::ReturnContractValidation::Satisfied(accumulated_status)
        }
    }
}

/// Keeps core type-test calls available to standalone semantic checking.
/// Workspace sessions normally publish these declarations from the embedded
/// core source; direct checker fixtures intentionally do not load that module.
pub(crate) fn ensure_core_object_type_tests(store: &mut TypeStore, declarations: &DeclarationTypeTable, dispatch: &mut SurfaceDispatchResolver) {
    let class = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Class);
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
    if let Some(class_form) = declarations.form(&class) {
        dispatch.register_type(class_form, class.clone());
    }
    dispatch.register_surface(class, class_surface);

    let object = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Object);
    let bool_decl = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Bool);
    if let Some(bool_ty) = declarations.form(&bool_decl) {
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
        if let Some(object_form) = declarations.form(&object) {
            dispatch.register_type(object_form, object.clone());
        }
        dispatch.register_surface(object, surface);
    }
}

fn local_type_is_soundly_widenable(
    ctx: &mut CheckingContext<'_>,
    local_type: &LocalType,
    expected: TypeId,
    constraints: &[LocalConstraint],
) -> bool {
    let expected_local = LocalType::from_canonical(ctx.store, expected, &HashMap::new());
    for constraint in constraints {
        let LocalConstraint::Equivalent { left, right } = constraint else {
            continue;
        };
        let mut rigid_bindings = BTreeMap::new();
        collect_equivalent_rigid_bindings(left, right, &mut rigid_bindings);
        if rigid_bindings.is_empty() {
            continue;
        }
        let rewritten = rewrite_local_rigids(local_type, &rigid_bindings);
        if rewritten.alpha_equivalent(&expected_local) {
            return true;
        }
    }

    if let Some(upper) = constraints.iter().find_map(|constraint| match constraint {
        LocalConstraint::Subtype {
            lower,
            upper: LocalType::Canonical(upper),
        } if lower == local_type => Some(*upper),
        _ => None,
    }) {
        return upper == expected || is_subtype(ctx.store, &ctx.hierarchy, upper, expected);
    }

    match local_type {
        LocalType::Canonical(ty) => is_subtype(ctx.store, &ctx.hierarchy, *ty, expected),
        LocalType::Applied { origin, .. } => match origin.as_ref() {
            LocalType::Canonical(origin) => is_subtype(ctx.store, &ctx.hierarchy, *origin, expected),
            _ => false,
        },
        LocalType::Union(members) => members
            .iter()
            .all(|member| local_type_is_soundly_widenable(ctx, member, expected, constraints)),
        LocalType::Tuple(elements) => {
            let LocalType::Tuple(expected_elements) = LocalType::from_canonical(ctx.store, expected, &HashMap::new()) else {
                return false;
            };
            elements.len() == expected_elements.len()
                && elements.iter().zip(expected_elements.iter()).all(|(left, right)| {
                    let LocalType::Canonical(expected) = &right.ty else {
                        return false;
                    };
                    local_type_is_soundly_widenable(ctx, &left.ty, *expected, constraints)
                })
        }
        _ => false,
    }
}

fn collect_equivalent_rigid_bindings(left: &LocalType, right: &LocalType, bindings: &mut BTreeMap<crate::types::id::RigidTypeVariableId, LocalType>) {
    match (left, right) {
        (LocalType::Rigid(rigid), other) if other.free_rigids().is_empty() => {
            if bindings.get(rigid).is_none_or(|existing| existing == other) {
                bindings.insert(*rigid, other.clone());
            }
        }
        (other, LocalType::Rigid(rigid)) if other.free_rigids().is_empty() => {
            if bindings.get(rigid).is_none_or(|existing| existing == other) {
                bindings.insert(*rigid, other.clone());
            }
        }
        (
            LocalType::Applied { origin: left_origin, arguments: left_arguments },
            LocalType::Applied { origin: right_origin, arguments: right_arguments },
        ) if left_arguments.len() == right_arguments.len() => {
            collect_equivalent_rigid_bindings(left_origin, right_origin, bindings);
            for (left, right) in left_arguments.iter().zip(right_arguments.iter()) {
                collect_equivalent_rigid_bindings(left, right, bindings);
            }
        }
        (
            LocalType::ExactCase { variant: left_variant, enum_type: left_enum },
            LocalType::ExactCase { variant: right_variant, enum_type: right_enum },
        ) if left_variant == right_variant => collect_equivalent_rigid_bindings(left_enum, right_enum, bindings),
        (LocalType::Union(left), LocalType::Union(right)) if left.len() == right.len() => {
            for (left, right) in left.iter().zip(right.iter()) {
                collect_equivalent_rigid_bindings(left, right, bindings);
            }
        }
        (LocalType::Tuple(left), LocalType::Tuple(right)) if left.len() == right.len() => {
            for (left, right) in left.iter().zip(right.iter()).filter(|(left, right)| left.label == right.label) {
                collect_equivalent_rigid_bindings(&left.ty, &right.ty, bindings);
            }
        }
        (LocalType::Record(left), LocalType::Record(right)) if left.len() == right.len() => {
            for (left, right) in left.iter().zip(right.iter()).filter(|(left, right)| left.name == right.name) {
                collect_equivalent_rigid_bindings(&left.ty, &right.ty, bindings);
            }
        }
        (
            LocalType::Callable { parameters: left_parameters, return_type: left_return },
            LocalType::Callable { parameters: right_parameters, return_type: right_return },
        ) if left_parameters.len() == right_parameters.len() => {
            for (left, right) in left_parameters.iter().zip(right_parameters.iter()).filter(|(left, right)| left.label == right.label && left.rest == right.rest) {
                collect_equivalent_rigid_bindings(&left.ty, &right.ty, bindings);
            }
            collect_equivalent_rigid_bindings(left_return, right_return, bindings);
        }
        _ => {}
    }
}

fn rewrite_local_rigids(local: &LocalType, bindings: &BTreeMap<crate::types::id::RigidTypeVariableId, LocalType>) -> LocalType {
    match local {
        LocalType::Rigid(rigid) => bindings.get(rigid).cloned().unwrap_or_else(|| local.clone()),
        LocalType::Canonical(_) => local.clone(),
        LocalType::Applied { origin, arguments } => LocalType::Applied {
            origin: Box::new(rewrite_local_rigids(origin, bindings)),
            arguments: arguments.iter().map(|argument| rewrite_local_rigids(argument, bindings)).collect(),
        },
        LocalType::ExactCase { variant, enum_type } => LocalType::ExactCase {
            variant: variant.clone(),
            enum_type: Box::new(rewrite_local_rigids(enum_type, bindings)),
        },
        LocalType::Union(members) => LocalType::Union(members.iter().map(|member| rewrite_local_rigids(member, bindings)).collect()),
        LocalType::Tuple(elements) => LocalType::Tuple(
            elements
                .iter()
                .map(|element| crate::types::rigid::LocalTupleElement {
                    label: element.label.clone(),
                    ty: rewrite_local_rigids(&element.ty, bindings),
                })
                .collect(),
        ),
        LocalType::Record(fields) => LocalType::Record(
            fields
                .iter()
                .map(|field| crate::types::rigid::LocalRecordField {
                    name: field.name.clone(),
                    ty: rewrite_local_rigids(&field.ty, bindings),
                })
                .collect(),
        ),
        LocalType::Callable { parameters, return_type } => LocalType::Callable {
            parameters: parameters
                .iter()
                .map(|parameter| crate::types::rigid::LocalCallableParameter {
                    label: parameter.label.clone(),
                    ty: rewrite_local_rigids(&parameter.ty, bindings),
                    rest: parameter.rest,
                })
                .collect(),
            return_type: Box::new(rewrite_local_rigids(return_type, bindings)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::CheckingContext;
    use crate::checker::binding::{BindingContract, BindingContractOrigin};
    use crate::checker::causal::CausalInvalidity;
    use crate::checker::flow::state::FlowInvariantFailure;
    use crate::checker::incident::{InternalSemanticIncidentDetails, InternalSemanticIncidentKind};
    use crate::declarations::bootstrap_universe_declarations;
    use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
    use crate::dispatch::{CallableSignature, ResolvedDispatchResult};
    use crate::identity::{CallableId, DeclarationId, DispatchSide, ModuleId};
    use crate::types::SimpleTypeResolver;
    use crate::types::id::TypeId;
    use crate::types::relation::MapTypeHierarchy;
    use crate::types::store::TypeStore;
    use phalcom_common::range::SourceRange;

    #[test]
    fn published_annotation_diagnostics_join_all_error_causes() {
        let module = ModuleId::universe_root();
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
        let module = ModuleId::universe_root();
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
    fn dispatch_target_preserves_callable_identity() {
        let module = ModuleId::universe_root();
        let mut store = TypeStore::new();
        let mut declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(module.clone(), key.name().into()));
        let owner = DeclarationId::new(module.clone(), "Owner".into());
        let owner_form = store.nominal_type(owner.clone());
        let owner_class_object = store.class_object_type(owner.clone());
        declarations.insert(crate::declarations::DeclarationTypeInfo {
            declaration: owner.clone(),
            form: owner_form,
            class_object_type: owner_class_object,
            kind: crate::types::id::KindId::TYPE,
            generic_signature: None,
            supertype_template: None,
        });
        let resolver = SimpleTypeResolver::new();
        let hierarchy = MapTypeHierarchy::new();
        let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

        let selector = phalcom_common::selector::Selector::getter("value").unwrap();
        let callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);
        let int_decl = DeclarationId::new(ctx.current_module.clone(), "Int".into());
        let int = ctx.nominal_type_of(&int_decl).expect("bootstrap Int form");
        let signature = CallableSignature::new(
            selector,
            Vec::new(),
            crate::types::evidence::TypeKnowledge::established(int, crate::types::evidence::EvidenceOrigin::CallableSignature),
        );
        let mut surface = crate::surface::DeclarationSurface::new(Some(owner.clone()));
        surface.add_callable(DispatchSide::Instance, signature);
        ctx.register_surface(owner.clone(), surface);

        let receiver = ctx.nominal_type_of(&owner).expect("owner declaration form");
        let selector = phalcom_common::selector::Selector::getter("value").unwrap();
        let result = ctx.resolve_dispatch_target(receiver, &selector, crate::dispatch::DispatchLookup::Normal);
        let ResolvedDispatchResult::Found(resolved) = result else {
            panic!("expected resolved target");
        };
        assert_eq!(resolved.callable, callable);
        assert_eq!(resolved.signature.return_type.ty(), Some(int));
    }
}
