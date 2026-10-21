use crate::access::{AccessContext, is_member_accessible};
use crate::checker::analysis::{AnalysisStatus, ExpressionAnalysis, ExpressionAnalysisIndex, SemanticDependency};
use crate::checker::binding::{BindingContract, BindingContractOrigin, BindingDeclarationResult, BindingSeed, BindingWriteResult, reconcile_binding_relation};
use crate::checker::flow::FlowState;
use crate::checker::incident::{
    BindingContractSummary, InternalFailurePolicy, InternalSemanticIncident, InternalSemanticIncidentDetails, InternalSemanticIncidentKind,
};
use crate::core_surface::CoreDeclarationIds;
use crate::db::budget::{BudgetReport, CancellationToken, QueryBudget};
use crate::db::key::QueryKey;
use crate::declarations::{DeclarationTypeInfo, DeclarationTypeTable};
use crate::diagnostic::SemanticDiagnostic;
use crate::dispatch::{CallableParameter, CallableSignature, DispatchResult, ResolvedDispatch, ResolvedDispatchResult, SurfaceDispatchResolver};
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
use phalcom_common::modifier::Visibility;
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_native_surface::NATIVE_SURFACES;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

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
type SharedSemanticQueryDemands = Rc<RefCell<BTreeSet<QueryKey>>>;

/// Shared control for one callable/query semantic analysis.
///
/// Relation checks and statement transfer consume the same budget and
/// cancellation token. This prevents nested relation calls from silently
/// resetting query limits or escaping cancellation.
#[derive(Clone)]
pub struct CheckerControl {
    pub(crate) budget: Rc<RefCell<QueryBudget>>,
    pub(crate) cancellation: CancellationToken,
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

/// Returns whether declaration is the aggregate Universe-root bootstrap input.
///
/// Canonical built-in declarations owned by concrete Universe source modules
/// remain ordinary semantic identities and must retain their fine-grained
/// dependency edges. Only the aggregate root is bootstrap-owned here.
fn is_bootstrap_declaration(declaration: &DeclarationId) -> bool {
    declaration.module == ModuleId::universe_root() || crate::checker::proof_predicate::is_proof_predicate_declaration(declaration).is_some()
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
    if is_query_owned_module(&declaration.module) && !is_bootstrap_declaration(declaration) {
        record_query_dependency(dependencies, SemanticDependency::DeclarationShell(declaration.clone()));
    }
}

fn record_hierarchy_dependency(dependencies: &SharedSemanticDependencies, declaration: &DeclarationId) {
    if is_query_owned_module(&declaration.module) && !is_bootstrap_declaration(declaration) {
        record_query_dependency(dependencies, SemanticDependency::HierarchyEdge(declaration.clone()));
    }
}

/// Type resolver wrapper that records declaration and linked-interface reads during body checking.
#[derive(Clone)]
pub struct TrackingTypeResolver<'a> {
    inner: &'a dyn TypeResolver,
    dependencies: SharedSemanticDependencies,
    query_demands: SharedSemanticQueryDemands,
    deferred_static_definitions: Rc<RefCell<BTreeSet<DeclarationId>>>,
}

impl<'a> TrackingTypeResolver<'a> {
    fn new(inner: &'a dyn TypeResolver, dependencies: SharedSemanticDependencies) -> Self {
        Self {
            inner,
            dependencies,
            query_demands: Rc::new(RefCell::new(BTreeSet::new())),
            deferred_static_definitions: Rc::new(RefCell::new(BTreeSet::new())),
        }
    }

    fn with_inner<'b>(&'b self, inner: &'b dyn TypeResolver) -> TrackingTypeResolver<'b> {
        TrackingTypeResolver {
            inner,
            dependencies: self.dependencies.clone(),
            query_demands: self.query_demands.clone(),
            deferred_static_definitions: self.deferred_static_definitions.clone(),
        }
    }

    /// Marks known alias owners whose canonical forms are intentionally absent
    /// until their computed Record keys have been projected. A lookup that
    /// selects one of these owners suspends on its exact StaticDefinition key
    /// instead of publishing the ordinary missing-form diagnostic.
    pub(crate) fn track_deferred_static_definition(&self, declaration: DeclarationId) {
        self.deferred_static_definitions.borrow_mut().insert(declaration);
    }

    pub(crate) fn semantic_query_demands_snapshot(&self) -> BTreeSet<QueryKey> {
        self.query_demands.borrow().clone()
    }

    pub(crate) fn inner(&self) -> &'a dyn TypeResolver {
        self.inner
    }

    /// Resolves an application target through the ordinary linked-name
    /// resolver while recording the static-definition product as its
    /// declaration dependency. Value-domain static definitions do not publish
    /// a nominal declaration shell or C3 alias product.
    pub(crate) fn resolve_static_definition_name(
        &self,
        current_module: &ModuleId,
        root: &str,
        members: &[String],
    ) -> Option<(DeclarationId, Arc<crate::static_definition::StaticDefinitionProduct>)> {
        if is_query_owned_module(current_module) {
            record_query_dependency(&self.dependencies, SemanticDependency::LinkedName(current_module.clone(), root.to_string()));
        }

        let qualified_target = if members.len() == 1 {
            self.inner.resolve_module_alias(current_module, root)
        } else {
            None
        };
        if let Some(target) = &qualified_target {
            if is_query_owned_module(target) {
                record_query_dependency(&self.dependencies, SemanticDependency::PublicExport(target.clone(), members[0].clone()));
            }
        }

        let declaration = self.inner.resolve_type_name(current_module, root, members)?;
        let product = self.inner.resolve_static_definition(&declaration)?;
        if is_query_owned_module(&declaration.module) && !is_bootstrap_declaration(&declaration) {
            record_query_dependency(&self.dependencies, SemanticDependency::StaticDefinition(declaration.clone()));
        }
        Some((declaration, product))
    }
}

/// Resolves one type name while exposing the exact semantic facts consumed by
/// the resolver. Formal query products use this boundary to own their reverse
/// invalidation edges instead of falling back to an aggregate interface edge.
pub(crate) fn resolve_type_name_with_dependencies(
    resolver: &dyn TypeResolver,
    current_module: &ModuleId,
    root: &str,
    members: &[String],
) -> (Option<DeclarationId>, BTreeSet<SemanticDependency>) {
    let dependencies = Rc::new(RefCell::new(BTreeSet::new()));
    let tracking = TrackingTypeResolver::new(resolver, dependencies.clone());
    let resolved = tracking.resolve_type_name(current_module, root, members);
    (resolved, dependencies.borrow().clone())
}

/// Resolves the declaration-level parent of a source class using the same
/// canonical name-resolution path used by formal hierarchy queries.
///
/// The returned dependencies are recorded by workspace queries. Immutable
/// Universe baseline construction consumes the same declaration result but
/// does not need workspace invalidation edges.
pub(crate) fn resolve_source_class_superclass(
    class_decl: &DeclarationId,
    class_def: &phalcom_ast::ast::ClassDef,
    resolver: &dyn TypeResolver,
) -> (Option<DeclarationId>, BTreeSet<SemanticDependency>) {
    let super_decl = if let Some(super_ref) = class_def.superclass_ref() {
        let members = super_ref.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
        let (resolved, dependencies) = resolve_type_name_with_dependencies(resolver, &class_decl.module, &super_ref.root, &members);
        let resolved = resolved.or_else(|| match (super_ref.root.as_str(), members.is_empty()) {
            ("Some", true) => Some(crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Some)),
            ("None", true) => Some(crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::None)),
            _ => None,
        });
        return (resolved, dependencies);
    } else {
        let object = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Object);
        (class_decl != &object).then_some(object)
    };
    (super_decl, BTreeSet::new())
}

impl TypeResolver for TrackingTypeResolver<'_> {
    fn resolve_type_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId> {
        if is_query_owned_module(current_module) {
            record_query_dependency(&self.dependencies, SemanticDependency::LinkedName(current_module.clone(), root.to_string()));
        }

        let qualified_target = if members.len() == 1 {
            self.inner.resolve_module_alias(current_module, root)
        } else {
            None
        };

        if let Some(target) = &qualified_target {
            if is_query_owned_module(target) {
                record_query_dependency(&self.dependencies, SemanticDependency::PublicExport(target.clone(), members[0].clone()));
            }
        }

        let declaration = self.inner.resolve_type_name(current_module, root, members);

        if let Some(declaration) = &declaration {
            record_declaration_shell_dependency(&self.dependencies, declaration);
        }

        declaration
    }

    fn projected_record_key(&self, syntax: &phalcom_ast::ast::RecordKeySyntax) -> Option<crate::static_key::ProjectedRecordKeyFact> {
        self.inner.projected_record_key(syntax)
    }

    fn resolve_trait_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId> {
        if is_query_owned_module(current_module) {
            record_query_dependency(&self.dependencies, SemanticDependency::LinkedName(current_module.clone(), root.to_string()));
        }

        let qualified_target = if members.len() == 1 {
            self.inner.resolve_module_alias(current_module, root)
        } else {
            None
        };

        if let Some(target) = &qualified_target {
            if is_query_owned_module(target) {
                record_query_dependency(&self.dependencies, SemanticDependency::PublicExport(target.clone(), members[0].clone()));
            }
        }

        let declaration = self.inner.resolve_type_name(current_module, root, members);
        if let Some(declaration) = &declaration {
            record_query_dependency(&self.dependencies, SemanticDependency::TraitHeader(declaration.clone()));
        }
        declaration
    }

    fn resolve_type_level_binding(&self, name: &str) -> Option<TypeLevelBinding> {
        self.inner.resolve_type_level_binding(name)
    }

    fn resolve_alias_form(&self, declaration: &DeclarationId) -> Option<TypeId> {
        let form = self.inner.resolve_alias_form(declaration);
        if form.is_none() && self.deferred_static_definitions.borrow().contains(declaration) {
            self.query_demands.borrow_mut().insert(QueryKey::StaticDefinition(declaration.clone()));
        }
        form
    }

    fn resolve_static_definition(&self, declaration: &DeclarationId) -> Option<Arc<crate::static_definition::StaticDefinitionProduct>> {
        let product = self.inner.resolve_static_definition(declaration);
        if product.is_some() {
            record_query_dependency(&self.dependencies, SemanticDependency::StaticDefinition(declaration.clone()));
        } else if self.deferred_static_definitions.borrow().contains(declaration) {
            self.query_demands.borrow_mut().insert(QueryKey::StaticDefinition(declaration.clone()));
        }
        product
    }

    fn resolve_module_alias(&self, current_module: &ModuleId, alias: &str) -> Option<ModuleId> {
        self.inner.resolve_module_alias(current_module, alias)
    }

    fn resolve_import_binding_id(&self, current_module: &ModuleId, name: &str) -> Option<phalcom_modules::identity::SourceImportBindingId> {
        self.inner.resolve_import_binding_id(current_module, name)
    }

    fn resolve_associated_type_requirement(&self, trait_declaration: &DeclarationId, name: &str) -> Option<crate::traits::AssociatedTypeRequirementId> {
        self.inner.resolve_associated_type_requirement(trait_declaration, name)
    }

    fn resolve_associated_type_member(
        &self,
        owner: &DeclarationId,
        selector: &crate::types::type_member::TypeMemberSelector,
    ) -> Option<std::sync::Arc<crate::db::product::TypeMemberDeclarationProduct>> {
        let id = self.inner.resolve_associated_type_member_id(owner, selector)?;
        let Some(product) = self.inner.resolve_associated_type_member(owner, selector) else {
            // Selection is authoritative even when its checked product has not
            // reached the current resolver view. Suspend on that exact owner;
            // never turn this absence into a source diagnostic or dependency.
            self.query_demands.borrow_mut().insert(QueryKey::TypeMemberDeclaration(id));
            return None;
        };
        if product.id == id {
            record_query_dependency(&self.dependencies, SemanticDependency::AssociatedSurface(owner.clone()));
            record_query_dependency(&self.dependencies, SemanticDependency::TypeMemberDeclaration(product.id.clone()));
            Some(product)
        } else {
            // The resolver must never answer a stable surface identity with a
            // different checked product. Treat disagreement as unavailable so
            // the owner boundary can refresh and retry canonically.
            self.query_demands.borrow_mut().insert(QueryKey::TypeMemberDeclaration(id));
            None
        }
    }

    fn resolve_associated_type_member_id(
        &self,
        owner: &DeclarationId,
        selector: &crate::types::type_member::TypeMemberSelector,
    ) -> Option<crate::types::type_member::TypeMemberDeclarationId> {
        self.inner.resolve_associated_type_member_id(owner, selector)
    }

    fn resolve_static_case_catalog(
        &self,
        domain: &crate::identity::StaticDomainDeclarationId,
    ) -> Option<std::sync::Arc<crate::static_case::StaticDomainCaseCatalogProduct>> {
        let product = self.inner.resolve_static_case_catalog(domain);
        if product.is_some() {
            record_query_dependency(&self.dependencies, SemanticDependency::StaticDomainCaseCatalog(domain.clone()));
        }
        product
    }

    fn resolve_static_case_signature(&self, case: &crate::identity::StaticCaseId) -> Option<std::sync::Arc<crate::static_case::StaticCaseSignatureProduct>> {
        let product = self.inner.resolve_static_case_signature(case);
        if product.is_some() {
            record_query_dependency(&self.dependencies, SemanticDependency::StaticCaseSignature(case.clone()));
        }
        product
    }

    fn resolve_associated_static_domain(&self, owner: &DeclarationId, name: &str) -> Option<crate::identity::AssociatedStaticDomainDeclarationId> {
        let selected = self.inner.resolve_associated_static_domain(owner, name);
        record_query_dependency(&self.dependencies, SemanticDependency::AssociatedSurface(owner.clone()));
        selected
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

/// Scoped evaluation phase fact distinguishing runtime execution from static evaluation (COLL006.C1.P1).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EvaluationPhase {
    #[default]
    Runtime,
    Static,
}

/// The active context during semantic type checking.
pub struct CheckingContext<'a> {
    pub store: &'a mut TypeStore,
    pub hierarchy: TrackingTypeHierarchy<'a>,
    pub resolver: TrackingTypeResolver<'a>,
    pub declarations: &'a DeclarationTypeTable,
    pub current_module: ModuleId,
    pub caller_package: Option<ModuleId>,
    pub access_topology: Option<&'a phalcom_modules::topology::ModuleTopology>,
    pub phase: EvaluationPhase,
    /// Checked module value facts available to this body. These facts retain
    /// canonical source `SymbolId`s even when reached through an import alias.
    module_value_bindings: Option<&'a crate::parent_map::ParentMap<ModuleId, Arc<crate::module_bindings::ModuleStaticBindingIndex>>>,
    /// Linked public value exports keyed by the exporting module and public
    /// spelling. This is deliberately separate from ordinary object dispatch.
    module_value_exports: Option<&'a BTreeMap<(ModuleId, String), phalcom_modules::linker::SymbolId>>,
    pub current_class: Option<DeclarationId>,
    pub current_side: DispatchSide,
    pub current_callable: Option<CallableId>,
    pub(crate) imported_source_bindings: BTreeMap<BindingId, phalcom_modules::identity::SourceImportBindingId>,
    pub(crate) observed_import_phases: BTreeMap<phalcom_modules::identity::SourceImportBindingId, phalcom_modules::graph::DependencyPhase>,
    /// Owner whose canonical instance constructor body is currently checked.
    /// Set only from a declaration signature carrying constructor semantics.
    pub(crate) current_constructor_owner: Option<DeclarationId>,
    /// Stable identity of the enclosing checked static-definition body.
    /// Static application ordinals are local to this owner, not to their target.
    pub(crate) current_static_body_owner: Option<crate::static_definition::StaticBodyId>,
    pub internal_failure_policy: InternalFailurePolicy,
    pub control: CheckerControl,
    pub expected_return: Option<CallableReturnContract>,
    /// Expected kind for a return from a kinded static definition. Ordinary
    /// callable and value-domain returns use `expected_return` above.
    pub expected_kinded_return: Option<crate::types::id::KindId>,
    stable_record_row_lacks: HashSet<(crate::types::id::TypeParameterId, crate::field_key::FieldKey)>,
    pub scopes: Vec<HashMap<String, LocalBindingInfo>>,
    pub flow: FlowState,
    pub(crate) normal_return_exits: Vec<crate::checker::analysis::NormalReturnFact>,
    pub(crate) throw_exit_flows: Vec<crate::checker::analysis::FlowStateSummary>,
    /// Binding products remain queryable after a branch-local scope closes.
    pub binding_history: BTreeMap<BindingId, crate::checker::analysis::BindingState>,
    /// Query-local declaration phase retained by stable lexical binding identity.
    binding_phases: BTreeMap<BindingId, phalcom_ast::ast::BindingPhase>,
    pub(crate) loop_frames: Vec<LoopFlowFrame>,
    pub body_id: BodyId,
    pub next_local_expr_id: u32,
    pub next_diagnostic_cause: u32,
    pub next_analysis_incident: u32,
    pub next_binding_id: u32,
    pub expressions: ExpressionAnalysisIndex,
    pub(crate) checked_static_applications: Vec<crate::static_definition::CheckedStaticApplication>,
    pub(crate) checked_static_constructions: BTreeMap<ExpressionId, crate::static_definition::CheckedStaticConstruction>,
    pub for_iteration_lanes: crate::checker::analysis::ForIterationLaneResolutionIndex,
    pub static_iteration_lanes: crate::checker::static_iteration::StaticIterationLaneResolutionIndex,
    pub positional_expansions: crate::checker::analysis::PositionalExpansionIterationResolutionIndex,
    pub known_invalid_positional_expansions: crate::checker::analysis::KnownInvalidPositionalExpansionIndex,
    pub sequence_literals: crate::checker::analysis::SequenceLiteralResolutionIndex,
    pub(crate) sequence_publication: super::sequence_publication::SequencePublication,
    pub(crate) deferred_applications: BTreeMap<super::sequence_publication::PublicationOwner, super::sequence_publication::DeferredApplicationRequirements>,
    pub(crate) call_publication_owners: BTreeMap<ExpressionId, Vec<(super::sequence_publication::PublicationOwner, bool)>>,
    pub associated_resolutions: crate::checker::associated::AssociatedResolutionIndex,
    pub callable_reference_resolutions: crate::checker::associated::CallableReferenceResolutionIndex,
    pub family_applications: crate::checker::associated::FamilyApplicationResolutionIndex,
    pub match_resolutions: crate::match_semantics::MatchResolutionIndex,
    pub pattern_site_resolutions: crate::match_semantics::PatternSiteResolutionIndex,
    pub proof_predicates: crate::checker::proof_predicate::ProofPredicateResolutionIndex,
    pub explanations: crate::explain::ExplanationArena,
    expression_owners: Vec<ExpressionId>,
    expression_owned_causes: BTreeMap<ExpressionId, crate::identity::DiagnosticCauseId>,
    resolved_callables: BTreeMap<ExpressionId, CallableId>,
    resolved_conditional_dispatches: BTreeMap<ExpressionId, crate::dispatch::ConditionalDispatchSelection>,
    resolved_conditional_capabilities: BTreeMap<ExpressionId, crate::dispatch::ConditionalSelectedCallCapability>,
    resolved_trait_dispatches: BTreeMap<ExpressionId, crate::trait_dispatch::TraitDispatchSite>,
    resolved_trait_dispatch_candidates: BTreeMap<ExpressionId, Box<[crate::trait_dispatch::TraitEvidencedMemberCandidate]>>,
    call_dependency_frames: Vec<CallDependencyFrame>,
    pub flow_graph: Option<std::sync::Arc<crate::checker::flow::graph::FlowGraph>>,
    pub dependencies: BTreeSet<CallableId>,
    semantic_dependencies: SharedSemanticDependencies,
    /// Import provenance follows the selected lexical identity, so shadowing
    /// does not observe a module product merely because an import was seeded.
    module_binding_dependencies: BTreeMap<BindingId, Vec<SemanticDependency>>,
    /// Storage origins for module values copied into this callable's scope.
    /// Other binding handles are body-local and are materialized as such when
    /// the callable analysis is finalized.
    seeded_module_value_origins: BTreeMap<BindingId, crate::checker::analysis::CallableBindingOrigin>,
    field_signatures: Option<&'a FieldSignatureTable>,
    field_lifecycle: Option<&'a crate::checker::field_lifecycle::FieldLifecycleTable>,
    pub(crate) constructor_initializations: Option<&'a crate::signature::ConstructorInitializationTable>,
    pub(crate) callable_signatures: Option<&'a crate::signature::CallableSignatureTable>,
    pub(crate) constructor_parent_owner: Option<DeclarationId>,
    pub(crate) constructor_super_required: bool,
    pub(crate) constructor_super_prologue_allowed: bool,
    pub(crate) constructor_super_call_seen: bool,
    pub(crate) constructor_super_initializer: Option<crate::checker::analysis::ConstructorSuperCallResolution>,
    pub data_table: Option<&'a crate::data_semantics::DataSemanticTable>,
    pub enum_table: Option<&'a crate::enum_semantics::EnumSemanticTable>,
    pub associated_table: Option<&'a crate::associated::AssociatedFamilyTable>,
    /// Trait contract visible while checking a trait default body. This is
    /// deliberately separate from ordinary declaration dispatch surfaces.
    pub trait_surface: Option<&'a crate::traits::TraitSurface>,
    pub conformance_semantics: Option<&'a crate::trait_dispatch::ConformanceSemanticView<'a>>,
    pub dispatch: DispatchAccess<'a>,
    pub generic_evidence: crate::generic_evidence::GenericEvidenceEnvironment,
    pub ambient_constraints: Vec<crate::types::parameter::GenericConstraint>,
    pub receiver_value_override: Option<TypeId>,
    pub instance_self_override: Option<TypeId>,
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
    inference_contexts: BTreeMap<crate::checker::inference::InferenceContextId, Rc<RefCell<crate::checker::inference::InferenceCoordinator>>>,
    next_inference_context_id: u32,
    inference_frames: Vec<(crate::checker::inference::InferenceContextId, crate::checker::inference::InferenceFrameId)>,
    symbolic_inference_results: BTreeMap<ExpressionId, crate::checker::typed_expr::SymbolicInferenceResult>,
    call_specializations: BTreeMap<ExpressionId, crate::checker::analysis::CallSpecialization>,
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
            caller_package: None,
            access_topology: None,
            phase: EvaluationPhase::Runtime,
            module_binding_dependencies: BTreeMap::new(),
            seeded_module_value_origins: BTreeMap::new(),
            module_value_bindings: None,
            module_value_exports: None,
            current_class: None,
            current_side: DispatchSide::Instance,
            current_callable: None,
            imported_source_bindings: BTreeMap::new(),
            observed_import_phases: BTreeMap::new(),
            current_constructor_owner: None,
            current_static_body_owner: None,
            internal_failure_policy: InternalFailurePolicy::Contain,
            control: CheckerControl::default(),
            expected_return: None,
            expected_kinded_return: None,
            stable_record_row_lacks: HashSet::new(),
            scopes: vec![HashMap::new()],
            flow: FlowState::new(),
            normal_return_exits: Vec::new(),
            throw_exit_flows: Vec::new(),
            binding_history: BTreeMap::new(),
            binding_phases: BTreeMap::new(),
            loop_frames: Vec::new(),
            body_id: BodyId(0),
            next_local_expr_id: 0,
            next_diagnostic_cause: 0,
            next_analysis_incident: 0,
            next_binding_id: 0,
            expressions: ExpressionAnalysisIndex::new(),
            checked_static_applications: Vec::new(),
            checked_static_constructions: BTreeMap::new(),
            for_iteration_lanes: Default::default(),
            static_iteration_lanes: Default::default(),
            positional_expansions: Default::default(),
            known_invalid_positional_expansions: Default::default(),
            sequence_literals: Default::default(),
            sequence_publication: Default::default(),
            call_publication_owners: BTreeMap::new(),
            deferred_applications: BTreeMap::new(),
            associated_resolutions: crate::checker::associated::AssociatedResolutionIndex::new(),
            callable_reference_resolutions: crate::checker::associated::CallableReferenceResolutionIndex::new(),
            family_applications: crate::checker::associated::FamilyApplicationResolutionIndex::new(),
            match_resolutions: crate::match_semantics::MatchResolutionIndex::new(),
            pattern_site_resolutions: crate::match_semantics::PatternSiteResolutionIndex::new(),
            proof_predicates: BTreeMap::new(),
            explanations: crate::explain::ExplanationArena::new(),
            expression_owners: Vec::new(),
            expression_owned_causes: BTreeMap::new(),
            resolved_callables: BTreeMap::new(),
            resolved_conditional_dispatches: BTreeMap::new(),
            resolved_conditional_capabilities: BTreeMap::new(),
            resolved_trait_dispatches: BTreeMap::new(),
            resolved_trait_dispatch_candidates: BTreeMap::new(),
            call_dependency_frames: Vec::new(),
            flow_graph: None,
            dependencies: BTreeSet::new(),
            semantic_dependencies,
            field_signatures: None,
            field_lifecycle: None,
            constructor_initializations: None,
            callable_signatures: None,
            constructor_parent_owner: None,
            constructor_super_required: false,
            constructor_super_prologue_allowed: false,
            constructor_super_call_seen: false,
            constructor_super_initializer: None,
            data_table: None,
            enum_table: None,
            associated_table: None,
            trait_surface: None,
            conformance_semantics: None,
            dispatch: DispatchAccess::Owned(dispatch),
            generic_evidence: crate::generic_evidence::GenericEvidenceEnvironment::empty(),
            ambient_constraints: Vec::new(),
            receiver_value_override: None,
            instance_self_override: None,
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
            call_specializations: BTreeMap::new(),
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
            caller_package: None,
            access_topology: None,
            phase: EvaluationPhase::Runtime,
            module_binding_dependencies: BTreeMap::new(),
            seeded_module_value_origins: BTreeMap::new(),
            module_value_bindings: None,
            module_value_exports: None,
            current_class: None,
            current_side: DispatchSide::Instance,
            current_callable: None,
            imported_source_bindings: BTreeMap::new(),
            observed_import_phases: BTreeMap::new(),
            current_constructor_owner: None,
            current_static_body_owner: None,
            internal_failure_policy: InternalFailurePolicy::Contain,
            control,
            expected_return: None,
            expected_kinded_return: None,
            stable_record_row_lacks: HashSet::new(),
            scopes: vec![HashMap::new()],
            flow: FlowState::new(),
            normal_return_exits: Vec::new(),
            throw_exit_flows: Vec::new(),
            binding_history: BTreeMap::new(),
            binding_phases: BTreeMap::new(),
            loop_frames: Vec::new(),
            body_id: BodyId(0),
            next_local_expr_id: 0,
            next_diagnostic_cause: 0,
            next_analysis_incident: 0,
            next_binding_id: 0,
            expressions: ExpressionAnalysisIndex::new(),
            checked_static_applications: Vec::new(),
            checked_static_constructions: BTreeMap::new(),
            for_iteration_lanes: Default::default(),
            static_iteration_lanes: Default::default(),
            positional_expansions: Default::default(),
            known_invalid_positional_expansions: Default::default(),
            sequence_literals: Default::default(),
            sequence_publication: Default::default(),
            call_publication_owners: BTreeMap::new(),
            deferred_applications: BTreeMap::new(),
            associated_resolutions: crate::checker::associated::AssociatedResolutionIndex::new(),
            callable_reference_resolutions: crate::checker::associated::CallableReferenceResolutionIndex::new(),
            family_applications: crate::checker::associated::FamilyApplicationResolutionIndex::new(),
            match_resolutions: crate::match_semantics::MatchResolutionIndex::new(),
            pattern_site_resolutions: crate::match_semantics::PatternSiteResolutionIndex::new(),
            proof_predicates: BTreeMap::new(),
            explanations: crate::explain::ExplanationArena::new(),
            expression_owners: Vec::new(),
            expression_owned_causes: BTreeMap::new(),
            resolved_callables: BTreeMap::new(),
            resolved_conditional_dispatches: BTreeMap::new(),
            resolved_conditional_capabilities: BTreeMap::new(),
            resolved_trait_dispatches: BTreeMap::new(),
            resolved_trait_dispatch_candidates: BTreeMap::new(),
            call_dependency_frames: Vec::new(),
            flow_graph: None,
            dependencies: BTreeSet::new(),
            semantic_dependencies,
            field_signatures: None,
            field_lifecycle: None,
            constructor_initializations: None,
            callable_signatures: None,
            constructor_parent_owner: None,
            constructor_super_required: false,
            constructor_super_prologue_allowed: false,
            constructor_super_call_seen: false,
            constructor_super_initializer: None,
            data_table: None,
            enum_table: None,
            associated_table: None,
            trait_surface: None,
            conformance_semantics: None,
            dispatch: DispatchAccess::Borrowed(dispatch),
            generic_evidence: crate::generic_evidence::GenericEvidenceEnvironment::empty(),
            ambient_constraints: Vec::new(),
            receiver_value_override: None,
            instance_self_override: None,
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
            call_specializations: BTreeMap::new(),
        }
    }

    pub fn with_resolver<'b>(&'b mut self, resolver: &'b dyn TypeResolver) -> CheckingContext<'b> {
        CheckingContext {
            store: self.store,
            hierarchy: TrackingTypeHierarchy::new(self.hierarchy.inner, self.declarations, self.semantic_dependencies.clone()),
            resolver: self.resolver.with_inner(resolver),
            declarations: self.declarations,
            current_module: self.current_module.clone(),
            caller_package: self.caller_package.clone(),
            access_topology: self.access_topology,
            phase: self.phase,
            module_binding_dependencies: self.module_binding_dependencies.clone(),
            seeded_module_value_origins: self.seeded_module_value_origins.clone(),
            module_value_bindings: self.module_value_bindings,
            module_value_exports: self.module_value_exports,
            current_class: self.current_class.clone(),
            current_side: self.current_side,
            current_callable: self.current_callable.clone(),
            imported_source_bindings: self.imported_source_bindings.clone(),
            observed_import_phases: self.observed_import_phases.clone(),
            current_constructor_owner: self.current_constructor_owner.clone(),
            current_static_body_owner: self.current_static_body_owner.clone(),
            internal_failure_policy: self.internal_failure_policy,
            control: self.control.clone(),
            expected_return: self.expected_return.clone(),
            expected_kinded_return: self.expected_kinded_return,
            stable_record_row_lacks: self.stable_record_row_lacks.clone(),
            scopes: self.scopes.clone(),
            flow: self.flow.clone(),
            normal_return_exits: self.normal_return_exits.clone(),
            throw_exit_flows: self.throw_exit_flows.clone(),
            binding_history: self.binding_history.clone(),
            binding_phases: self.binding_phases.clone(),
            loop_frames: self.loop_frames.clone(),
            body_id: self.body_id,
            next_local_expr_id: self.next_local_expr_id,
            next_diagnostic_cause: self.next_diagnostic_cause,
            next_analysis_incident: self.next_analysis_incident,
            next_binding_id: self.next_binding_id,
            expressions: self.expressions.clone(),
            checked_static_applications: self.checked_static_applications.clone(),
            checked_static_constructions: self.checked_static_constructions.clone(),
            for_iteration_lanes: self.for_iteration_lanes.clone(),
            static_iteration_lanes: self.static_iteration_lanes.clone(),
            positional_expansions: self.positional_expansions.clone(),
            known_invalid_positional_expansions: self.known_invalid_positional_expansions.clone(),
            sequence_literals: self.sequence_literals.clone(),
            sequence_publication: self.sequence_publication.clone(),
            call_publication_owners: self.call_publication_owners.clone(),
            deferred_applications: self.deferred_applications.clone(),
            associated_resolutions: self.associated_resolutions.clone(),
            callable_reference_resolutions: self.callable_reference_resolutions.clone(),
            family_applications: self.family_applications.clone(),
            match_resolutions: self.match_resolutions.clone(),
            pattern_site_resolutions: self.pattern_site_resolutions.clone(),
            proof_predicates: self.proof_predicates.clone(),
            explanations: self.explanations.clone(),
            expression_owners: self.expression_owners.clone(),
            expression_owned_causes: self.expression_owned_causes.clone(),
            resolved_callables: self.resolved_callables.clone(),
            resolved_conditional_dispatches: self.resolved_conditional_dispatches.clone(),
            resolved_conditional_capabilities: self.resolved_conditional_capabilities.clone(),
            resolved_trait_dispatches: self.resolved_trait_dispatches.clone(),
            resolved_trait_dispatch_candidates: self.resolved_trait_dispatch_candidates.clone(),
            call_dependency_frames: self.call_dependency_frames.clone(),
            flow_graph: self.flow_graph.clone(),
            dependencies: self.dependencies.clone(),
            semantic_dependencies: self.semantic_dependencies.clone(),
            field_signatures: self.field_signatures,
            field_lifecycle: self.field_lifecycle,
            constructor_initializations: self.constructor_initializations,
            callable_signatures: self.callable_signatures,
            constructor_parent_owner: self.constructor_parent_owner.clone(),
            constructor_super_required: self.constructor_super_required,
            constructor_super_prologue_allowed: self.constructor_super_prologue_allowed,
            constructor_super_call_seen: self.constructor_super_call_seen,
            constructor_super_initializer: self.constructor_super_initializer.clone(),
            data_table: self.data_table,
            enum_table: self.enum_table,
            associated_table: self.associated_table,
            trait_surface: self.trait_surface,
            conformance_semantics: self.conformance_semantics,
            dispatch: DispatchAccess::Borrowed(self.dispatch.get()),
            generic_evidence: self.generic_evidence.clone(),
            ambient_constraints: self.ambient_constraints.clone(),
            receiver_value_override: self.receiver_value_override,
            instance_self_override: self.instance_self_override,
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
            call_specializations: self.call_specializations.clone(),
        }
    }

    /// Supplies the canonical package identity for the current caller module.
    pub fn set_caller_package(&mut self, package: ModuleId) {
        self.caller_package = Some(package);
    }

    pub fn set_access_topology(&mut self, topology: &'a phalcom_modules::topology::ModuleTopology) {
        self.access_topology = Some(topology);
    }

    /// Attaches canonical checked value facts for module imports and qualified
    /// module exports. The linker owns public-name resolution; the binding
    /// index owns the exported value's type and denotation.
    pub(crate) fn attach_module_value_exports(
        &mut self,
        bindings: &'a crate::parent_map::ParentMap<ModuleId, Arc<crate::module_bindings::ModuleStaticBindingIndex>>,
        exports: &'a BTreeMap<(ModuleId, String), phalcom_modules::linker::SymbolId>,
    ) {
        self.module_value_bindings = Some(bindings);
        self.module_value_exports = Some(exports);
    }

    /// Seeds the current module's already checked values plus selective
    /// imports into a callable body's outer scope.
    pub(crate) fn seed_linked_module_import_values(
        &mut self,
        linked_module: &phalcom_modules::linker::LinkedModule,
        bindings: &crate::parent_map::ParentMap<ModuleId, Arc<crate::module_bindings::ModuleStaticBindingIndex>>,
    ) {
        self.seed_current_module_values(bindings);
        self.seed_linked_import_values(linked_module, bindings);
    }

    /// Seeds the current module's already checked top-level values into a
    /// callable body's outer scope.
    pub(crate) fn seed_current_module_values(
        &mut self,
        bindings: &crate::parent_map::ParentMap<ModuleId, Arc<crate::module_bindings::ModuleStaticBindingIndex>>,
    ) {
        if let Some(index) = bindings.get(&self.current_module).cloned() {
            for (name, fact) in index.visible_bindings() {
                if fact.symbol.module != self.current_module {
                    continue;
                }
                self.declare_module_value_binding(name.to_owned(), &fact.state, fact.phase, fact.symbol.clone(), None);
            }
        }
    }

    /// Seeds only selective imports from the canonical linked target.
    ///
    /// Top-level module rechecking must use this form: its local declarations
    /// are about to be checked from source and seeding their already checked
    /// facts would redeclare every local binding in the same lexical scope.
    pub(crate) fn seed_linked_import_values(
        &mut self,
        linked_module: &phalcom_modules::linker::LinkedModule,
        bindings: &crate::parent_map::ParentMap<ModuleId, Arc<crate::module_bindings::ModuleStaticBindingIndex>>,
    ) {
        for (local_name, import_id) in &linked_module.bindings.imports {
            let Some(phalcom_modules::linker::LinkedReadSpec::Binding(symbol)) = linked_module.linked_reads.get(import_id.0 as usize) else {
                continue;
            };
            let Some(fact) = bindings.get(&symbol.module).and_then(|index| index.get(symbol)) else {
                continue;
            };
            self.declare_module_value_binding(local_name.to_string(), &fact.state, fact.phase, symbol.clone(), Some(*import_id));
        }
    }

    pub(crate) fn record_import_phase(&mut self, id: &phalcom_modules::identity::SourceImportBindingId, phase: phalcom_modules::graph::DependencyPhase) {
        self.observed_import_phases
            .entry(id.clone())
            .and_modify(|p| *p = p.join(phase))
            .or_insert(phase);
    }

    pub(crate) fn seed_symbolic_module_import_values(
        &mut self,
        semantic_module: &phalcom_modules::linker::SemanticLinkedModule,
        linked_module: &phalcom_modules::linker::LinkedModule,
        bindings: &crate::parent_map::ParentMap<ModuleId, Arc<crate::module_bindings::ModuleStaticBindingIndex>>,
    ) {
        for import_binding in semantic_module.imports.bindings.values() {
            let phalcom_modules::linker::SymbolicImportTarget::Binding(ref symbol) = import_binding.target else {
                continue;
            };
            let Some(fact) = bindings.get(&symbol.module).and_then(|index| index.get(symbol)) else {
                continue;
            };
            let local_name = import_binding.local_name.to_string();
            let import_id = linked_module
                .bindings
                .imports
                .get(local_name.as_str())
                .copied()
                .expect("symbolic imported binding must have a linked import slot");
            assert!(
                matches!(
                    linked_module.linked_reads.get(import_id.0 as usize),
                    Some(phalcom_modules::linker::LinkedReadSpec::Binding(linked_symbol)) if linked_symbol == symbol
                ),
                "symbolic imported binding slot must resolve to the same canonical symbol"
            );
            self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::LinkedName(
                self.current_module.clone(),
                local_name.clone(),
            ));
            self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::ModuleStaticBindings(symbol.module.clone()));
            if let BindingDeclarationResult::Inserted(binding_id) =
                self.declare_module_value_binding(local_name, &fact.state, fact.phase, symbol.clone(), Some(import_id))
            {
                self.imported_source_bindings.insert(binding_id, import_binding.id.clone());
            }
        }
    }

    /// Seeds the current module's checked top-level values and its symbolic
    /// imports into a callable body's outer scope.
    pub(crate) fn seed_symbolic_module_values(
        &mut self,
        semantic_module: &phalcom_modules::linker::SemanticLinkedModule,
        linked_module: &phalcom_modules::linker::LinkedModule,
        bindings: &crate::parent_map::ParentMap<ModuleId, Arc<crate::module_bindings::ModuleStaticBindingIndex>>,
    ) {
        self.seed_current_module_values(bindings);
        self.seed_symbolic_module_import_values(semantic_module, linked_module, bindings);
    }

    /// Resolves `module_alias.public_name` as an exported *value*, without
    /// treating the module as an object with a synthetic dispatch surface.
    pub(crate) fn resolve_module_export_value(&self, module_alias: &str, public_name: &str) -> Option<ValueSemanticFact> {
        self.record_semantic_dependency(SemanticDependency::LinkedName(self.current_module.clone(), module_alias.to_owned()));
        let module = self.resolver.resolve_module_alias(&self.current_module, module_alias)?;
        self.record_semantic_dependency(SemanticDependency::PublicExport(module.clone(), public_name.to_owned()));
        self.record_semantic_dependency(SemanticDependency::ModuleStaticBindingCatalog(module.clone()));
        let exports = self.module_value_exports?;
        let symbol = exports.get(&(module.clone(), public_name.to_owned()))?;
        let bindings = self.module_value_bindings?;
        let fact = bindings.get(&symbol.module)?.get(symbol)?;
        self.record_semantic_dependency(SemanticDependency::PublicExport(module, public_name.to_owned()));
        self.record_semantic_dependency(SemanticDependency::ModuleStaticBindings(symbol.module.clone()));
        Some(ValueSemanticFact {
            knowledge: fact.state.current.clone(),
            denotation: fact.state.denotation.clone(),
        })
    }

    /// Projects the current checker scope into the shared access relation.
    pub fn access_context(&self) -> AccessContext {
        AccessContext::with_provenance(
            self.current_class.clone(),
            self.current_callable.clone(),
            Some(self.current_module.clone()),
            self.caller_package.clone(),
            Some(self.current_module.project),
        )
    }

    pub(crate) fn declaration_member_is_accessible(&self, owner: &DeclarationId, visibility: Visibility) -> bool {
        let declaration_package = self.access_topology.map(|topology| topology.owning_package(&owner.module));
        is_member_accessible(visibility, owner, declaration_package.as_ref(), &self.access_context(), &self.hierarchy)
    }

    pub(crate) fn data_constructor_is_accessible(&self, data_info: &crate::data_semantics::DataInfo) -> bool {
        self.declaration_member_is_accessible(&data_info.owner, data_info.constructor.visibility)
    }

    fn dispatch_is_accessible(&self, resolved: &ResolvedDispatch) -> bool {
        let Some(owner) = resolved
            .trait_dispatch
            .as_ref()
            .map(|selection| selection.requirement.owner.clone())
            .or_else(|| resolved.callable.try_declaration_owner().cloned())
        else {
            return true;
        };
        self.declaration_member_is_accessible(&owner, resolved.visibility)
    }

    fn authorize_dispatch(&self, mut resolved: ResolvedDispatch) -> ResolvedDispatch {
        resolved.access_denied = !self.dispatch_is_accessible(&resolved);
        resolved
    }

    /// Checks a candidate without committing solver, flow, rigid or publication
    /// state. Dependency reads and budget/cancellation remain shared. A unique
    /// winner must subsequently be checked through the live application path.
    pub(crate) fn run_candidate_probe<T>(&mut self, run: impl FnOnce(&mut CheckingContext<'_>) -> T) -> T {
        let resolver = self.resolver.inner;
        let mut probe = self.with_resolver(resolver);
        probe.inference_contexts = probe
            .inference_contexts
            .iter()
            .map(|(id, session)| (*id, Rc::new(RefCell::new(session.borrow().fork()))))
            .collect();
        let deferred = probe.resolver.deferred_static_definitions.borrow().clone();
        probe.resolver.deferred_static_definitions = Rc::new(RefCell::new(deferred));
        let result = run(&mut probe);
        let dependencies = probe.dependencies.clone();
        drop(probe);
        self.dependencies.extend(dependencies);
        result
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

        probe.caller_package = self.caller_package.clone();
        probe.access_topology = self.access_topology;
        probe.module_binding_dependencies = self.module_binding_dependencies.clone();
        probe.seeded_module_value_origins = self.seeded_module_value_origins.clone();
        probe.module_value_bindings = self.module_value_bindings;
        probe.module_value_exports = self.module_value_exports;
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
        probe.trait_surface = self.trait_surface;
        probe.binding_history = self.binding_history.clone();
        probe.binding_phases = self.binding_phases.clone();
        probe.next_binding_id = self.next_binding_id;
        probe.next_local_expr_id = self.next_local_expr_id;
        probe.next_diagnostic_cause = self.next_diagnostic_cause;
        probe.next_analysis_incident = self.next_analysis_incident;
        probe.inference_contexts = self.inference_contexts.clone();
        probe.next_inference_context_id = self.next_inference_context_id;
        probe.inference_frames = self.inference_frames.clone();
        probe.sequence_publication = self.sequence_publication.probe();
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
                        initialization: state.initialization,
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

    pub(crate) fn solve_inference(&mut self, session: &mut crate::checker::inference::InferenceCoordinator) -> crate::checker::inference::InferenceOutcome {
        session.solve_with_control(self.store, &self.hierarchy, &self.control)
    }

    pub(crate) fn propagate_inference(
        &mut self,
        session: &mut crate::checker::inference::InferenceCoordinator,
    ) -> Result<bool, Box<crate::checker::inference::InferenceOutcome>> {
        session.propagate_with_control(self.store, &self.hierarchy, &self.control)
    }

    pub(crate) fn solve_inference_in_frame(
        &mut self,
        session: &mut crate::checker::inference::InferenceCoordinator,
        frame: crate::checker::inference::InferenceFrameId,
    ) -> crate::checker::inference::InferenceOutcome {
        let empty_surfaces = crate::traits::TraitSurfaceTable::new();
        const MAX_ASSOCIATED_INFERENCE_ITERATIONS: usize = 16;
        for _ in 0..MAX_ASSOCIATED_INFERENCE_ITERATIONS {
            match session.propagate_with_control(self.store, &self.hierarchy, &self.control) {
                Ok(_) => {
                    let mut progress = false;
                    let norm_progress = {
                        let mut budget = self.control.budget.borrow_mut();
                        let mut norm_context = crate::types::ProjectionNormalizationContext::new(
                            crate::types::ProjectionNormalizationMode::Generic {
                                evidence: &self.generic_evidence,
                                conformance_index: self.conformance_semantics.map(|cs| cs.conformance_index),
                                witness_plans: self.conformance_semantics.map(|cs| cs.witness_plans),
                                trait_surfaces: self.conformance_semantics.map(|cs| cs.trait_surfaces).unwrap_or(&empty_surfaces),
                                derived_rules: self.conformance_semantics.map(|cs| cs.derived_conformance_rules),
                                declarations: self.declarations,
                                hierarchy: self.hierarchy.inner(),
                            },
                            &mut budget,
                            &self.control.cancellation,
                        );
                        session.extract_and_normalize_projections(self.store, |store, ty| crate::types::normalize_type(store, ty, &mut norm_context).type_id())
                    };
                    match norm_progress {
                        Ok(changed) => {
                            if std::env::var_os("PHALCOM_INFERENCE_TRACE").is_some() {
                                eprintln!(
                                    "inference frame {:?}: projection rewrite changed={changed} residual={:?}",
                                    frame,
                                    session.residual_obligations()
                                );
                            }
                            // Rewrites, including reflexive equalities with no
                            // new binding, require a canonical residual retry.
                            progress |= changed;
                        }
                        Err(failure) => {
                            return session.failure_outcome(failure, None, None);
                        }
                    }
                    if !progress {
                        break;
                    }
                }
                Err(outcome) => return *outcome,
            }
        }
        session.finish_frame(frame)
    }

    /// Creates one query-local inference graph and its root application frame.
    pub(crate) fn create_inference_context(&mut self) -> (crate::checker::inference::InferenceContextId, crate::checker::inference::InferenceFrameId) {
        let mut context = crate::checker::inference::InferenceContextId(self.next_inference_context_id);
        while self.inference_contexts.contains_key(&context) {
            self.next_inference_context_id = self.next_inference_context_id.saturating_add(1);
            context = crate::checker::inference::InferenceContextId(self.next_inference_context_id);
        }
        self.next_inference_context_id = self.next_inference_context_id.saturating_add(1);
        let mut session = crate::checker::inference::InferenceCoordinator::new();
        let frame = session.root_frame();
        self.inference_contexts.insert(context, Rc::new(RefCell::new(session)));
        self.sequence_publication
            .register(super::sequence_publication::PublicationOwner(context, frame), None);
        (context, frame)
    }

    /// Returns a cloneable handle to an active query-local inference graph.
    pub(crate) fn inference_session(
        &self,
        context: crate::checker::inference::InferenceContextId,
    ) -> Option<Rc<RefCell<crate::checker::inference::InferenceCoordinator>>> {
        self.inference_contexts.get(&context).cloned()
    }

    /// Begins a child application frame in an existing query-local graph.
    pub(crate) fn begin_inference_frame(
        &mut self,
        context: crate::checker::inference::InferenceContextId,
        parent: crate::checker::inference::InferenceFrameId,
    ) -> Option<crate::checker::inference::InferenceFrameId> {
        let session = self.inference_contexts.get(&context)?.clone();
        let session_state = session.borrow();
        if session_state.frame_is_closed(parent) || (parent.0 != 0 && session_state.frame_parent(parent).is_none()) {
            return None;
        }
        drop(session_state);
        let frame = session.borrow_mut().begin_frame(Some(parent));
        self.sequence_publication.register(
            super::sequence_publication::PublicationOwner(context, frame),
            Some(super::sequence_publication::PublicationOwner(context, parent)),
        );
        Some(frame)
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
        self.sequence_publication.dispose(context, &mut self.sequence_literals);
        self.deferred_applications.retain(|owner, _| owner.0 != context);
        self.inference_contexts.remove(&context);
    }

    pub(crate) fn sequence_dependencies(&self) -> super::sequence_publication::PublicationDependencies {
        self.current_expression_id()
            .and_then(|expression| self.sequence_publication.expression_dependencies.get(&expression))
            .cloned()
            .unwrap_or_default()
    }

    pub(crate) fn stage_sequence(
        &mut self,
        resolution: super::analysis::SequenceLiteralResolution,
        mut dependencies: super::sequence_publication::PublicationDependencies,
    ) {
        dependencies.extend(self.sequence_dependencies());
        self.sequence_publication.stage(resolution, dependencies, &mut self.sequence_literals);
    }

    pub(crate) fn sequence_local_resolution(&self, expression: ExpressionId) -> Option<&super::analysis::SequenceLiteralResolution> {
        self.sequence_literals
            .get(&expression)
            .or_else(|| self.sequence_publication.local_resolution(expression))
    }

    pub(crate) fn reject_sequence_expression(&mut self, expression: ExpressionId) {
        self.sequence_publication.discard_expression(expression);
        self.sequence_literals
            .remove_source(&expression)
            .expect("checking context sequence products are source mutable");
    }

    pub(crate) fn reject_all_sequence_products(&mut self) {
        let expressions = self.sequence_literals.keys().copied().collect::<Vec<_>>();
        for expression in expressions {
            self.reject_sequence_expression(expression);
        }
        let contexts = self.inference_contexts.keys().copied().collect::<Vec<_>>();
        for context in contexts {
            self.finish_inference_context(context);
        }
    }

    pub(crate) fn accept_sequence_owner(&mut self, owner: super::sequence_publication::PublicationOwner) -> Result<(), AnalysisStatus> {
        let requirements = self
            .deferred_applications
            .values()
            .filter(|requirements| self.sequence_publication.requires_owner(owner) && self.sequence_publication.depends_on(requirements.owner, owner))
            .cloned()
            .collect::<Vec<_>>();
        for requirements in &requirements {
            if let Err(status) = requirements.validate(self) {
                self.reject_sequence_owner(owner);
                if matches!(status, AnalysisStatus::Blocked(_)) {
                    self.emit_diagnostic(SemanticDiagnostic::error_in(
                        self.current_module.clone(),
                        crate::diagnostic::DiagnosticCode::GenericInferenceUnderconstrained,
                        "required generic application is not ready for construction publication",
                        requirements.range,
                    ));
                }
                if let Some(expression) = self
                    .expressions
                    .get_source_mut(&requirements.expression)
                    .expect("checking context expression products are source mutable")
                {
                    expression.status = status.clone();
                }
                self.record_terminal_status(status.clone());
                return Err(status);
            }
        }
        // Readiness validation completes for every required child before any
        // terminal owner receipt can flush a product into the committed map.
        for requirements in requirements {
            self.sequence_publication.locally_ready(requirements.owner, &mut self.sequence_literals);
        }
        self.sequence_publication.locally_ready(owner, &mut self.sequence_literals);
        Ok(())
    }

    pub(crate) fn reject_sequence_owner(&mut self, owner: super::sequence_publication::PublicationOwner) {
        self.sequence_publication.reject(owner, &mut self.sequence_literals);
        self.deferred_applications.remove(&owner);
    }

    pub(crate) fn finish_call_sequence_publication(&mut self, status: &mut AnalysisStatus, causal: super::causal::CausalInvalidity) {
        let Some(expression) = self.current_expression_id() else {
            return;
        };
        let owners = self.call_publication_owners.remove(&expression).unwrap_or_default();
        for (owner, owns_context) in owners {
            let readiness = self.deferred_applications.get(&owner);
            if status.is_ready() && matches!(causal, super::causal::CausalInvalidity::Clean) && readiness.is_some() {
                if readiness.is_some_and(|requirements| !requirements.deferred) {
                    if let Err(failure) = self.accept_sequence_owner(owner) {
                        *status = failure;
                    }
                }
            } else {
                self.reject_sequence_owner(owner);
            }
            if owns_context {
                self.finish_inference_context(owner.0);
            }
        }
    }

    pub(crate) fn record_sequence_dependencies(&mut self, dependencies: &super::sequence_publication::PublicationDependencies) {
        if let Some(expression) = self.current_expression_id() {
            self.sequence_publication
                .expression_dependencies
                .entry(expression)
                .or_default()
                .extend(dependencies.iter().copied());
        }
    }

    /// Durable artifacts cannot be marked ready while executable facts still
    /// await an owner decision. Existing source errors remain the primary cause.
    pub(crate) fn ensure_sequence_publication_settled(&mut self, range: SourceRange) -> bool {
        if !self.sequence_publication.has_pending() {
            return true;
        }
        if !self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == crate::diagnostic::DiagnosticSeverity::Error)
        {
            self.publish_analysis_incident("sequence construction reached artifact extraction with an unsettled inference owner");
            self.emit_diagnostic(SemanticDiagnostic::error_in(
                self.current_module.clone(),
                crate::diagnostic::DiagnosticCode::AnalysisInternalFailure,
                "sequence construction inference has not completed",
                range,
            ));
        }
        let contexts = self.inference_contexts.keys().copied().collect::<Vec<_>>();
        for context in contexts {
            self.finish_inference_context(context);
        }
        false
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

    pub(crate) fn record_checked_static_application(
        &mut self,
        source_range: SourceRange,
        target: DeclarationId,
        signature: crate::static_definition::StaticDefinitionSignature,
        arguments: Box<[crate::static_definition::CheckedStaticApplicationArgument]>,
    ) -> Option<crate::static_definition::StaticApplicationId> {
        let expression = self.current_expression_id()?;
        let body = self.current_static_body_owner.clone()?;
        let ordinal = u32::try_from(self.checked_static_applications.len()).ok()?;
        let id = crate::static_definition::StaticApplicationId { body, ordinal };
        self.checked_static_applications.push(crate::static_definition::CheckedStaticApplication {
            id: id.clone(),
            expression,
            source_range,
            target,
            signature,
            arguments,
        });
        Some(id)
    }

    pub(crate) fn record_checked_static_construction(&mut self, construction: crate::static_definition::CheckedStaticConstruction) {
        self.checked_static_constructions.insert(construction.expression, construction);
    }

    pub(crate) fn expression_id_for_static_range(&self, range: phalcom_common::range::SourceRange) -> Option<ExpressionId> {
        let mut matches = self.expressions.iter().filter_map(|(id, analysis)| (analysis.range == range).then_some(*id));
        let expression = matches.next()?;
        matches.next().is_none().then_some(expression)
    }

    pub(crate) fn trait_dispatch_candidates_for_current_expression(&self) -> Option<&[crate::trait_dispatch::TraitEvidencedMemberCandidate]> {
        self.current_expression_id()
            .and_then(|id| self.resolved_trait_dispatch_candidates.get(&id))
            .map(Box::as_ref)
    }

    pub fn resolved_callable_for_current_expression(&self) -> Option<CallableId> {
        self.current_expression_id().and_then(|id| self.resolved_callables.get(&id).cloned())
    }

    pub(crate) fn record_explicit_callable_selection(&mut self, callable: &CallableId) {
        self.dependencies.insert(callable.clone());
        if let Some(expression) = self.current_expression_id() {
            self.resolved_callables.insert(expression, callable.clone());
        }
    }

    pub(crate) fn seed_stable_record_row_lacks(&mut self, facts: impl IntoIterator<Item = crate::checker::row_inference::StableRecordRowLack>) {
        for fact in facts {
            self.stable_record_row_lacks.insert((fact.parameter, fact.field));
        }
    }

    pub(crate) fn stable_record_row_lacks(&self, parameter: crate::types::id::TypeParameterId, field: &crate::field_key::FieldKey) -> bool {
        self.stable_record_row_lacks
            .iter()
            .any(|(candidate, key)| *candidate == parameter && key == field)
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

    pub(crate) fn current_call_explanations(&self) -> Vec<crate::identity::ExplanationId> {
        self.call_dependency_frames.last().map(|frame| frame.explanations.clone()).unwrap_or_default()
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
        let owner = self.expression_owners.pop();
        debug_assert_eq!(owner, Some(id));
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
        let mut roots = Vec::new();
        crate::phase_evidence::collect_type_annotation_roots(annotation, &mut roots);
        for root in roots {
            if let Some(import_id) = resolver.resolve_import_binding_id(&self.current_module, &root) {
                self.record_import_phase(&import_id, phalcom_modules::graph::DependencyPhase::InterfaceOnly);
            }
        }
        let mut diagnostics = Vec::new();
        let mut site = if let Some(owner) = self.current_class.clone() {
            TypeFormationSite::member(self.current_module.clone(), owner, self.current_side)
        } else {
            TypeFormationSite::module(self.current_module.clone())
        };
        if let Some(self_type) = self.instance_self_override {
            site = site.with_self_type_override(self_type);
        }
        let knowledge = crate::types::annotation::resolve_type_annotation(self.store, self.declarations, resolver, &site, annotation, &mut diagnostics);
        let causal_invalidity = self.publish_diagnostics(diagnostics);
        (knowledge, causal_invalidity)
    }

    /// Elaborates one source annotation, supporting inference holes in authorized
    /// initializer contexts (COLL006.C1.P1).
    pub fn elaborate_type_annotation(
        &mut self,
        resolver: &dyn TypeResolver,
        annotation: &phalcom_ast::ast::TypeAnnotation,
    ) -> (
        Option<crate::checker::annotation_elaboration::AnnotationElaboration>,
        crate::checker::causal::CausalInvalidity,
    ) {
        match crate::checker::annotation_elaboration::elaborate_annotation(self, resolver, annotation) {
            Ok(elaboration) => (Some(elaboration), crate::checker::causal::CausalInvalidity::Clean),
            Err(diagnostics) => {
                let causal = self.publish_diagnostics(diagnostics);
                (None, causal)
            }
        }
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
                if let Some(analysis) = self
                    .expressions
                    .get_source_mut(&owner)
                    .expect("checking context expression products are source mutable")
                {
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

    fn lexical_types_equivalent(&self, actual: TypeId, expected: TypeId) -> bool {
        self.generic_evidence
            .are_types_equivalent(self.store, actual, expected)
            .is_proven()
    }

    pub fn apply_assignability(
        &mut self,
        actual: &TypeKnowledge,
        expected: &TypeKnowledge,
        code: crate::diagnostic::DiagnosticCode,
        message: impl Into<String>,
        range: SourceRange,
    ) -> RelationApplication {
        let outcome = match (actual.ty(), expected.ty()) {
            (Some(actual_ty), Some(expected_ty)) if self.lexical_types_equivalent(actual_ty, expected_ty) => RelationOutcome::proven(()),
            _ => self
                .control
                .relation(|budget, cancellation| check_assignability_bounded(self.store, &self.hierarchy, actual, expected, budget, cancellation)),
        };
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
        let outcome = if actual.ty().is_some_and(|actual_ty| self.lexical_types_equivalent(actual_ty, expected)) {
            RelationOutcome::proven(())
        } else {
            self.control
                .relation(|budget, cancellation| check_knowledge_against_type_bounded(self.store, &self.hierarchy, actual, expected, budget, cancellation))
        };
        let explanation = Some(self.record_type_relation(actual, expected, &outcome, range));
        self.apply_relation_outcome(outcome, code, message, range, None, explanation)
    }

    /// Evaluates one contract relation while sharing this body's budget and
    /// cancellation state. The caller owns diagnostic policy and reconciliation.
    pub fn check_knowledge_against_type(&mut self, actual: &TypeKnowledge, expected: TypeId) -> RelationOutcome {
        if actual.ty().is_some_and(|actual_ty| self.lexical_types_equivalent(actual_ty, expected)) {
            RelationOutcome::proven(())
        } else {
            self.control
                .relation(|budget, cancellation| check_knowledge_against_type_bounded(self.store, &self.hierarchy, actual, expected, budget, cancellation))
        }
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
        let outcome = if actual.ty().is_some_and(|actual_ty| self.lexical_types_equivalent(actual_ty, expected)) {
            RelationOutcome::proven(())
        } else {
            self.control
                .relation(|budget, cancellation| check_knowledge_against_type_bounded(self.store, &self.hierarchy, actual, expected, budget, cancellation))
        };
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
        // Consume the per-analysis publication so a later re-analysis cannot
        // accidentally inherit a specialization from an older solution.
        analysis.call_specialization = self.call_specializations.remove(&id);
        analysis.conditional_dispatch = self.resolved_conditional_dispatches.get(&id).cloned();
        analysis.conditional_capability = self.resolved_conditional_capabilities.get(&id).cloned();
        analysis.trait_dispatch = self.resolved_trait_dispatches.get(&id).cloned();
        analysis.trait_dispatch_candidates = self.resolved_trait_dispatch_candidates.get(&id).cloned();
        analysis.contextual_realization = typed.contextual_realization.clone();
        analysis.denotation = typed.denotation.clone();
        analysis.status = typed.status.clone();
        analysis.causal_invalidity = typed.causal_invalidity;
        analysis.explanation = explanation;
        analysis.binding_read_safety = typed.binding_read_safety;
        analysis.binding_read = typed.binding_read;
        analysis.binding_write = typed.binding_write;

        self.expressions
            .insert_source(id, analysis.clone())
            .expect("checking context expression products are source mutable");
        analysis
    }

    pub(crate) fn sync_expression_outcome(&mut self, typed: &crate::checker::typed_expr::TypedExpression) {
        typed.debug_assert_coherent();

        let Some(id) = typed.expression_id else {
            return;
        };
        let Some(analysis) = self
            .expressions
            .get_source_mut(&id)
            .expect("checking context expression products are source mutable")
        else {
            return;
        };

        analysis.knowledge = typed.knowledge.clone();
        analysis.callable = typed.callable.clone();
        if let Some(specialization) = self.call_specializations.remove(&id) {
            analysis.call_specialization = Some(specialization);
        }
        analysis.conditional_dispatch = self.resolved_conditional_dispatches.get(&id).cloned();
        analysis.conditional_capability = self.resolved_conditional_capabilities.get(&id).cloned();
        analysis.trait_dispatch = self.resolved_trait_dispatches.get(&id).cloned();
        analysis.trait_dispatch_candidates = self.resolved_trait_dispatch_candidates.get(&id).cloned();
        analysis.contextual_realization = typed.contextual_realization.clone();
        analysis.denotation = typed.denotation.clone();
        analysis.status = typed.status.clone();
        analysis.causal_invalidity = typed.causal_invalidity;
        analysis.binding_read_safety = typed.binding_read_safety;
        analysis.binding_read = typed.binding_read;
        analysis.binding_write = typed.binding_write;
    }

    pub(crate) fn take_call_specialization(&mut self, id: ExpressionId) -> Option<crate::checker::analysis::CallSpecialization> {
        self.call_specializations.remove(&id)
    }

    pub(crate) fn publish_call_specialization(&mut self, id: ExpressionId, specialization: crate::checker::analysis::CallSpecialization) {
        self.call_specializations.insert(id, specialization);
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

    pub(crate) fn record_binding_phase(&mut self, binding: BindingId, phase: phalcom_ast::ast::BindingPhase) {
        if phase == phalcom_ast::ast::BindingPhase::Static {
            self.binding_phases.insert(binding, phase);
        }
    }

    pub(crate) fn is_static_binding(&self, binding: BindingId) -> bool {
        self.binding_phases.get(&binding) == Some(&phalcom_ast::ast::BindingPhase::Static)
    }

    /// Imports an already checked binding fact into the current lexical scope
    /// with a fresh body-local identity, without rerunning contract inference.
    pub(crate) fn declare_checked_binding(
        &mut self,
        name: String,
        source: &crate::checker::analysis::BindingState,
        phase: crate::module_bindings::ModuleBindingPhase,
    ) -> BindingDeclarationResult {
        let Some(scope) = self.scopes.last() else {
            return BindingDeclarationResult::Redeclared(BindingId(u32::MAX));
        };
        if let Some(existing) = scope.get(&name).cloned() {
            let mut diagnostic = SemanticDiagnostic::error_in(
                self.current_module.clone(),
                crate::diagnostic::DiagnosticCode::BindingRedeclared,
                format!("binding `{name}` is already declared in this scope"),
                source.range,
            );
            if let Some(previous) = self.flow.get_binding(existing.id) {
                diagnostic = diagnostic.with_label(previous.range, "first declaration");
            }
            self.emit_diagnostic(diagnostic);
            return BindingDeclarationResult::Redeclared(existing.id);
        }

        let binding_id = self.alloc_binding();
        if phase == crate::module_bindings::ModuleBindingPhase::StaticConstant {
            self.record_binding_phase(binding_id, phalcom_ast::ast::BindingPhase::Static);
        }
        let seed = BindingSeed {
            name: name.clone(),
            parameter: None,
            range: source.range,
            contract: source.contract.clone(),
            current: source.current.clone(),
            denotation: source.denotation.clone(),
            causal_invalidity: source.causal_invalidity,
            mutable: source.mutable,
            initialization: source.initialization,
        };
        self.flow.declare_seed(binding_id, seed, source.current.clone(), source.consistency.clone());
        if let Some(state) = self.flow.get_binding(binding_id).cloned() {
            self.binding_history.insert(binding_id, state);
        }
        self.scopes.last_mut().expect("scope checked above").insert(
            name,
            LocalBindingInfo {
                id: binding_id,
                denotation: source.denotation.clone(),
            },
        );
        BindingDeclarationResult::Inserted(binding_id)
    }

    /// Imports a checked module value under a fresh local binding handle and
    /// retains its canonical storage owner. The source binding's range may be
    /// owned by a different module; callers attach the local import site via
    /// the module source index rather than treating it as a body-local token.
    pub(crate) fn declare_module_value_binding(
        &mut self,
        name: String,
        source: &crate::checker::analysis::BindingState,
        phase: crate::module_bindings::ModuleBindingPhase,
        symbol: phalcom_modules::linker::SymbolId,
        import: Option<phalcom_modules::linker::ImportBindingId>,
    ) -> BindingDeclarationResult {
        let result = self.declare_checked_binding(name.clone(), source, phase);
        if let BindingDeclarationResult::Inserted(binding) = &result {
            self.seeded_module_value_origins.insert(
                *binding,
                crate::checker::analysis::CallableBindingOrigin::ModuleValue {
                    symbol: symbol.clone(),
                    import,
                },
            );
            let mut dependencies = Vec::with_capacity(2);
            if import.is_some() {
                dependencies.push(crate::checker::analysis::SemanticDependency::LinkedName(self.current_module.clone(), name));
            }
            dependencies.push(crate::checker::analysis::SemanticDependency::ModuleStaticBindings(symbol.module.clone()));
            self.module_binding_dependencies.insert(*binding, dependencies);
        }
        result
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
        let body_type_fact = parameter.contract.derive_body_type(self.store);
        self.bind_callable_parameter_with_identity(
            parameter.local_name.to_string(),
            body_type_fact.to_knowledge(),
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
            initialization: crate::checker::flow::InitializationState::DefinitelyInitialized,
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
            initialization: crate::checker::flow::InitializationState::DefinitelyInitialized,
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
            initialization: crate::checker::flow::InitializationState::DefinitelyInitialized,
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
            mutable: false,
            initialization: crate::checker::flow::InitializationState::DefinitelyInitialized,
        })
    }

    pub fn bind_pattern_binding(&mut self, name: impl Into<String>, fact: ValueSemanticFact, range: SourceRange) -> BindingDeclarationResult {
        self.bind_pattern_binding_with_causal(name, fact, range, crate::checker::causal::CausalInvalidity::Clean)
    }

    pub fn attach_causal_invalidity_to_binding(&mut self, binding: BindingId, causal_invalidity: crate::checker::causal::CausalInvalidity) {
        if causal_invalidity == crate::checker::causal::CausalInvalidity::Clean {
            return;
        }
        if let Some(state) = self.flow.bindings.get_mut(&binding) {
            state.causal_invalidity = state.causal_invalidity.join(causal_invalidity);
        }
    }

    pub fn lookup_binding_info(&self, name: &str) -> Option<&LocalBindingInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                if let Some(dependencies) = self.module_binding_dependencies.get(&info.id) {
                    for dependency in dependencies {
                        self.record_semantic_dependency(dependency.clone());
                    }
                }
                return Some(info);
            }
        }
        None
    }

    pub(crate) fn binding_is_in_current_scope(&self, binding: BindingId) -> bool {
        self.scopes.last().is_some_and(|scope| scope.values().any(|info| info.id == binding))
    }

    pub(crate) fn is_enclosing_closure_binding(&self, binding: BindingId) -> bool {
        let Some(boundary) = self.closure_capture_frames.last().map(|(boundary, _)| *boundary) else {
            return false;
        };
        self.scopes.iter().take(boundary).any(|scope| scope.values().any(|info| info.id == binding))
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
        let outward = expected.map(|ty| self.store.format_type(ty)).unwrap_or_else(|| "inferred result".to_string());
        self.emit_diagnostic(SemanticDiagnostic::error_in(
            self.current_module.clone(),
            crate::diagnostic::DiagnosticCode::ExistentialEscape,
            format!("branch-local type {rigids} escapes into {outward}"),
            range,
        ));
        false
    }

    pub fn lookup_local(&self, name: &str) -> Option<ValueSemanticFact> {
        let Some(info) = self.lookup_binding_info(name) else {
            if self.module_value_bindings.is_some() {
                self.record_semantic_dependency(SemanticDependency::ModuleStaticBindingCatalog(self.current_module.clone()));
            }
            return None;
        };
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

    /// Records the context-independent exact proof root when the checker
    /// actually consults the global exact proof path or receives no proof.
    /// Lexical assumptions and derived results stay attached to their owning
    /// evidence and are not represented by a bare proposition key.
    pub(crate) fn record_exact_conformance_query_dependency(
        &self,
        key: &crate::generic_evidence::GenericConformanceKey,
        proof: &crate::generic_evidence::GenericConformanceProof,
    ) {
        use crate::generic_evidence::{GenericConformanceEvidence, GenericConformanceProof};
        let context_sensitive = matches!(
            proof,
            GenericConformanceProof::Proven(GenericConformanceEvidence::Assumed(_) | GenericConformanceEvidence::Derived(_))
        );
        if !context_sensitive && matches!(key.subject, crate::types::parameter::TypeTerm::Canonical(_)) {
            self.record_semantic_dependency(SemanticDependency::GenericConformance(key.clone()));
        }
    }

    pub(crate) fn semantic_dependencies_snapshot(&self) -> BTreeSet<SemanticDependency> {
        self.semantic_dependencies.borrow().clone()
    }

    /// Returns exact owner queries selected while their products were not yet
    /// current. Callers must discard this attempt's diagnostics and products.
    pub(crate) fn semantic_query_demands_snapshot(&self) -> BTreeSet<QueryKey> {
        self.resolver.semantic_query_demands_snapshot()
    }

    /// Records a dispatch lookup's structural and callable-type dependencies.
    ///
    /// `DeclarationSurface` owns selector/visibility/hierarchy projection only;
    /// every query-owned callable's type contract is represented by its
    /// canonical `CallableSignature` product, including partial declarations.
    pub(crate) fn record_consumed_callable_signature(&self, callable: &CallableId, _signature: &crate::dispatch::CallableSignature) {
        // Generated constructor signatures are canonical snapshot members but
        // have no source CallableSignature query of their own. Their owner
        // ConstructorInitialization product is the dependency authority.
        if self
            .constructor_initializations
            .and_then(|products| products.get(callable))
            .is_some_and(|product| product.origin != crate::signature::ConstructorOrigin::Source)
        {
            return;
        }
        let Some(owner) = callable.try_declaration_owner() else {
            self.record_semantic_dependency(SemanticDependency::CallableSignature(callable.clone()));
            return;
        };
        if !is_query_owned_module(callable.module()) || is_bootstrap_declaration(owner) || is_builtin_type_test_callable(callable) {
            return;
        }
        record_declaration_surface_dependency(&self.semantic_dependencies, owner);
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

    pub fn attach_constructor_initializations(&mut self, products: &'a crate::signature::ConstructorInitializationTable) {
        self.constructor_initializations = Some(products);
    }

    pub fn attach_callable_signatures(&mut self, signatures: &'a crate::signature::CallableSignatureTable) {
        self.callable_signatures = Some(signatures);
    }

    /// Attaches compiler-owned data semantics table.
    pub fn attach_data_semantics(&mut self, data_table: &'a crate::data_semantics::DataSemanticTable) {
        self.data_table = Some(data_table);
    }

    pub fn attach_conformance_semantics(&mut self, view: &'a crate::trait_dispatch::ConformanceSemanticView<'a>) {
        self.conformance_semantics = Some(view);
    }

    /// Reads data metadata while recording the data-declaration dependency.
    pub fn data_info(&self, owner: &DeclarationId) -> Option<&crate::data_semantics::DataInfo> {
        let info = self.data_table.and_then(|t| t.get(owner).map(|arc| &**arc));
        if info.is_some() {
            self.record_data_declaration_dependency(owner);
        }
        info
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
        let info = self.enum_table.and_then(|t| t.enums.get(owner).map(|arc| &**arc));
        if info.is_some() {
            self.record_enum_declaration_dependency(owner);
        }
        info
    }

    /// Reads variant metadata while recording the enum-declaration dependency.
    pub fn variant_info(&self, variant: &crate::identity::VariantId) -> Option<&crate::enum_semantics::VariantInfo> {
        let info = self.enum_table.and_then(|t| t.variants.get(variant).map(|arc| &**arc));
        if info.is_some() {
            self.record_enum_declaration_dependency(&variant.owner);
        }
        info
    }

    /// Reads associated family surface while recording the associated-surface dependency.
    pub fn associated_surface(&self, owner: &DeclarationId) -> Option<&crate::associated::AssociatedSurface> {
        let info = self.associated_table.and_then(|t| t.surfaces.get(owner).map(|arc| &**arc));
        if info.is_some() {
            self.record_associated_surface_dependency(owner);
        }
        info
    }

    /// Returns the dispatch resolver currently visible to this context.
    pub fn dispatch_ref(&self) -> &SurfaceDispatchResolver {
        self.dispatch.get()
    }

    /// Reads declaration metadata while recording the declaration-shell dependency.
    pub fn declaration_info(&self, declaration: &DeclarationId) -> Option<&DeclarationTypeInfo> {
        let info = self.declarations.get(declaration)?;
        record_declaration_shell_dependency(&self.semantic_dependencies, declaration);
        Some(info)
    }

    /// Returns whether a declaration's canonical result kind is a TypeOnly
    /// static domain. Generic algebraic declarations carry the domain in the
    /// final result of their arrow kind.
    pub fn declaration_is_type_only(&self, declaration: &DeclarationId) -> bool {
        let Some(info) = self.declaration_info(declaration) else {
            return false;
        };
        let mut kind = info.kind;
        loop {
            match self.store.get_kind(kind) {
                crate::types::kind::KindData::StaticDomain(_) => return true,
                crate::types::kind::KindData::Arrow { result, .. } => kind = *result,
                _ => return false,
            }
        }
    }

    /// Reads a declaration generic signature while recording the declaration-shell dependency.
    pub fn declaration_generic_signature(&self, declaration: &DeclarationId) -> Option<crate::types::parameter::GenericSignature> {
        if !self.declarations.contains(declaration) {
            return None;
        }
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
                if self.trait_surface.is_some() {
                    return None;
                }
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

    fn conditional_selected_call_capability(
        &mut self,
        selection: crate::dispatch::ConditionalDispatchSelection,
        domain: &crate::impls::InherentImplDomain,
        applicability: crate::impls::InherentImplSpecialization,
    ) -> Option<crate::dispatch::ConditionalSelectedCallCapability> {
        if applicability.impl_id != selection.impl_id {
            return None;
        }
        let provenance = applicability.conformance_provenance.as_ref()?;
        let required = domain
            .constraints
            .iter()
            .filter(|constraint| matches!(constraint, crate::types::parameter::GenericConstraint::Conforms { .. }))
            .count();
        if provenance.len() != required {
            return None;
        }
        for binding in provenance.iter() {
            self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                binding.callee_conformance.trait_ref.declaration.clone(),
            ));
            match &binding.source {
                crate::checker::analysis::GenericCallConformanceSource::Exact { evidence } => {
                    self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::ConformanceDispatch(evidence.source_impl.clone()));
                }
                crate::checker::analysis::GenericCallConformanceSource::Forwarded { caller_conformance } => {
                    self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                        caller_conformance.trait_ref.declaration.clone(),
                    ));
                }
                crate::checker::analysis::GenericCallConformanceSource::Derived { evidence } => {
                    self.record_derived_conformance_dependencies(evidence);
                }
                crate::checker::analysis::GenericCallConformanceSource::CompilerDerived { evidence } => {
                    self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                        evidence.key.trait_ref.declaration.clone(),
                    ));
                }
            }
        }
        Some(crate::dispatch::ConditionalSelectedCallCapability { selection, applicability })
    }

    fn record_derived_conformance_dependencies(&mut self, evidence: &crate::generic_evidence::DerivedConformanceEvidence) {
        match &evidence.origin {
            crate::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source } => {
                self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::ConformanceDispatch(
                    exact_source.source_impl.clone(),
                ));
                self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                    exact_source.exact_trait_ref.declaration.clone(),
                ));
            }
            crate::generic_evidence::DerivedConformanceOrigin::GuardedRule { rule, exact_trait_ref, .. } => {
                self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::ConformanceDispatch(rule.impl_id.clone()));
                self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(exact_trait_ref.declaration.clone()));
            }
        }
        for nested in evidence.nested.iter() {
            if let crate::generic_evidence::NestedGenericEvidence::Conformance(nested) = nested {
                match nested {
                    crate::generic_evidence::GenericConformanceEvidence::Exact(evidence) => {
                        self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::ConformanceDispatch(evidence.source_impl.clone()));
                        self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                            evidence.exact_trait_ref.declaration.clone(),
                        ));
                    }
                    crate::generic_evidence::GenericConformanceEvidence::Derived(evidence) => self.record_derived_conformance_dependencies(evidence),
                    crate::generic_evidence::GenericConformanceEvidence::Assumed(evidence) => {
                        self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                            evidence.key.trait_ref.declaration.clone(),
                        ));
                    }
                    crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(evidence) => {
                        self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                            evidence.key.trait_ref.declaration.clone(),
                        ));
                    }
                }
            }
        }
    }

    /// Records the exact semantic inputs consumed by an algebraic
    /// realization capability selection.
    pub(crate) fn record_contextual_realization_dependencies(&mut self, selection: &crate::checker::analysis::RealizationCapabilitySelection) {
        match selection {
            crate::checker::analysis::RealizationCapabilitySelection::Evidenced(selection) => {
                self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitHeader(
                    selection.exact_trait_ref.declaration.clone(),
                ));
                self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                    selection.exact_trait_ref.declaration.clone(),
                ));
                match &selection.evidence {
                    crate::generic_evidence::GenericConformanceEvidence::Exact(evidence) => {
                        self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::ConformanceDispatch(evidence.source_impl.clone()));
                    }
                    crate::generic_evidence::GenericConformanceEvidence::Derived(evidence) => self.record_derived_conformance_dependencies(evidence),
                    crate::generic_evidence::GenericConformanceEvidence::Assumed(evidence) => {
                        self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                            evidence.key.trait_ref.declaration.clone(),
                        ));
                    }
                    crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(evidence) => {
                        self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                            evidence.key.trait_ref.declaration.clone(),
                        ));
                    }
                }
                if let Some(callable) = &selection.callable {
                    self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::CallableSignature(callable.clone()));
                }
            }
            crate::checker::analysis::RealizationCapabilitySelection::Assumed(selection) => {
                self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitHeader(
                    selection.conformance.trait_ref.declaration.clone(),
                ));
                self.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                    selection.conformance.trait_ref.declaration.clone(),
                ));
            }
        }
    }

    pub(crate) fn specialize_dispatch_signature(
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
            parameter.contract = match &parameter.contract {
                crate::dispatch::CallableParameterContract::Fixed(ty) => {
                    crate::dispatch::CallableParameterContract::Fixed(ty.map_type(|ty| specialize_type(self, ty)))
                }
                crate::dispatch::CallableParameterContract::Rest { mode, contract } => crate::dispatch::CallableParameterContract::Rest {
                    mode: *mode,
                    contract: match contract {
                        crate::dispatch::RestValueContractKnowledge::Element(ty) => {
                            crate::dispatch::RestValueContractKnowledge::Element(ty.map_type(|ty| specialize_type(self, ty)))
                        }
                        crate::dispatch::RestValueContractKnowledge::Lane(lane_id) => {
                            crate::dispatch::RestValueContractKnowledge::Lane(specialize_type(self, *lane_id))
                        }
                        crate::dispatch::RestValueContractKnowledge::Unconstrained => crate::dispatch::RestValueContractKnowledge::Unconstrained,
                        crate::dispatch::RestValueContractKnowledge::Invalid(reason) => crate::dispatch::RestValueContractKnowledge::Invalid(reason.clone()),
                    },
                },
            };
        }
        signature.return_type = signature.return_type.map_type(|ty| specialize_type(self, ty));
        if let Some(mut generics) = signature.generics.take() {
            generics.constraints = generics
                .constraints
                .iter()
                .map(|constraint| {
                    let constraint = crate::types::substitution::specialize_self_generic_constraint(self.store, self.declarations, receiver, constraint);
                    crate::types::parameter::map_generic_constraint(
                        &constraint,
                        self,
                        |ctx, ty| crate::types::environment::TypeView::new(ty, environment.clone()).materialize(ctx.store),
                        |ctx, self_term| {
                            let ty = ctx.store.self_type(self_term.clone());
                            crate::types::parameter::TypeTerm::Canonical(
                                crate::types::environment::TypeView::new(ty, environment.clone()).materialize(ctx.store),
                            )
                        },
                    )
                })
                .collect();
            let constraint_mentions_parameter = |term: &crate::types::parameter::TypeTerm, parameter| match term {
                crate::types::parameter::TypeTerm::Canonical(ty) => self.store.contains_type_parameter(*ty, parameter),
                crate::types::parameter::TypeTerm::SelfType(_) | crate::types::parameter::TypeTerm::Infer(_) => false,
            };
            let parameter_contract_mentions = |contract: &crate::dispatch::CallableParameterContract, p| match contract {
                crate::dispatch::CallableParameterContract::Fixed(ty) => ty.ty().is_some_and(|ty| self.store.contains_type_parameter(ty, p)),
                crate::dispatch::CallableParameterContract::Rest { contract, .. } => match contract {
                    crate::dispatch::RestValueContractKnowledge::Element(ty) => ty.ty().is_some_and(|ty| self.store.contains_type_parameter(ty, p)),
                    crate::dispatch::RestValueContractKnowledge::Lane(lane_id) => self.store.contains_type_parameter(*lane_id, p),
                    crate::dispatch::RestValueContractKnowledge::Unconstrained => false,
                    crate::dispatch::RestValueContractKnowledge::Invalid(_) => false,
                },
            };
            let remains_in_signature = generics.parameters.iter().copied().any(|parameter| {
                signature.parameters.iter().any(|p| parameter_contract_mentions(&p.contract, parameter))
                    || signature.return_type.ty().is_some_and(|ty| self.store.contains_type_parameter(ty, parameter))
                    || generics.constraints.iter().any(|constraint| match constraint {
                        crate::types::parameter::GenericConstraint::Subtype { lower, upper }
                        | crate::types::parameter::GenericConstraint::Equivalent { left: lower, right: upper } => {
                            constraint_mentions_parameter(lower, parameter) || constraint_mentions_parameter(upper, parameter)
                        }
                        crate::types::parameter::GenericConstraint::Conforms { subject, trait_ref } => {
                            constraint_mentions_parameter(subject, parameter)
                                || trait_ref.arguments.iter().any(|&arg| self.store.contains_type_parameter(arg, parameter))
                        }
                    })
            });
            signature.generics = remains_in_signature.then_some(generics);
        }
        Ok((signature, specialization))
    }

    fn resolve_exact_case_conditional(
        &mut self,
        specialization_receiver: TypeId,
        side: DispatchSide,
        selector: &Selector,
        visited_owners: Box<[DeclarationId]>,
    ) -> Option<ResolvedDispatch> {
        let variant = match self.store.get(specialization_receiver) {
            TypeData::ExactCase { variant, .. } => self.store.variant_identity(*variant).clone(),
            _ => return None,
        };
        let cond_set = self
            .dispatch
            .get()
            .get_conditional_members(&crate::impls::InherentImplTarget::ExactEnumCase(variant.clone()))
            .cloned()?;
        let member = cond_set.get_member(side, selector)?;
        let proof_context = crate::impls::ConditionalImplProofContext {
            lexical_evidence: Some(&self.generic_evidence),
            conformance_index: self.conformance_semantics.map(|view| view.conformance_index),
            witness_plans: self.conformance_semantics.map(|view| view.witness_plans),
            trait_surfaces: self.conformance_semantics.map(|view| view.trait_surfaces),
            derived_rules: self.conformance_semantics.map(|view| view.derived_conformance_rules),
            declarations: Some(self.declarations),
        };
        let app = crate::impls::check_impl_domain_applicability_with_proofs(
            self.store,
            &self.hierarchy,
            &member.domain,
            specialization_receiver,
            specialization_receiver,
            &self.ambient_constraints,
            &proof_context,
        );
        let crate::impls::ImplApplicabilityResult::Applicable(spec) = app else {
            return None;
        };

        let selection = crate::dispatch::ConditionalDispatchSelection {
            impl_id: member.impl_id.clone(),
            callable: member.callable.clone(),
            declaring_owner: variant.owner.clone(),
            side,
        };
        let capability = self.conditional_selected_call_capability(selection.clone(), &member.domain, spec)?;

        let mut signature = crate::checker::declaration_signature::project_semantic_signature(&member.signature_template);
        let unspecialized_return = signature.return_type.clone();
        let environment = capability.applicability.environment.clone();
        let specialize_type = |ctx: &mut Self, ty| {
            let self_specialized = ctx.specialize_self_type(specialization_receiver, ty);
            crate::types::environment::TypeView::new(self_specialized, environment.clone()).materialize(ctx.store)
        };
        for parameter in &mut signature.parameters {
            parameter.ty = parameter.ty.map_type(|ty| specialize_type(self, ty));
        }
        signature.return_type = signature.return_type.map_type(|ty| specialize_type(self, ty));
        self.dependencies.insert(member.callable.clone());
        self.record_consumed_callable_signature(&member.callable, &signature);
        if let Some(expression) = self.current_expression_id() {
            self.resolved_callables.insert(expression, member.callable.clone());
            self.resolved_conditional_dispatches.insert(expression, selection.clone());
            self.resolved_conditional_capabilities.insert(expression, capability.clone());
        }
        Some(ResolvedDispatch {
            callable: member.callable.clone(),
            signature,
            visibility: member.visibility,
            access_denied: false,
            specialization: Some(crate::dispatch::DispatchSignatureSpecialization {
                receiver: specialization_receiver,
                declaring_owner: variant.owner.clone(),
                environment: capability.applicability.environment.clone(),
                path: Box::new([]),
                unspecialized_return,
            }),
            conditional: Some(selection),
            conditional_capability: Some(capability),
            trait_dispatch: None,
            abstract_contract: false,
            visited_owners,
        })
    }

    pub(crate) fn resolve_dispatch_target(&mut self, receiver: TypeId, selector: &Selector, lookup: crate::dispatch::DispatchLookup) -> ResolvedDispatchResult {
        self.resolve_dispatch_target_with_specialization(receiver, None, DispatchSide::Instance, selector, lookup)
    }

    fn resolve_trait_contract_target(
        &mut self,
        receiver: TypeId,
        selector: &Selector,
        lookup: &crate::dispatch::DispatchLookup,
    ) -> Option<ResolvedDispatchResult> {
        if !matches!(lookup, crate::dispatch::DispatchLookup::Normal) {
            return None;
        }
        let TypeData::SelfType(self_term) = self.store.get(receiver) else {
            return None;
        };
        let surface = self.trait_surface?;
        if self_term.owner != surface.declaration
            || self_term.side != DispatchSide::Instance
            || self_term.role != crate::types::parameter::SelfRole::InstanceType
        {
            return None;
        }
        let Some(member) = surface.get_by_selector(selector, DispatchSide::Instance) else {
            return Some(ResolvedDispatchResult::Missing {
                visited_owners: Box::new([surface.declaration.clone()]),
            });
        };
        let signature = crate::checker::declaration_signature::project_semantic_signature(&member.signature);
        self.dependencies.insert(member.callable.clone());
        self.record_semantic_dependency(SemanticDependency::TraitSurface(surface.declaration.clone()));
        if let Some(expression) = self.current_expression_id() {
            self.resolved_callables.insert(expression, member.callable.clone());
            self.resolved_trait_dispatches.insert(
                expression,
                crate::trait_dispatch::TraitDispatchSite::AbstractRequirement {
                    requirement: member.requirement.clone(),
                },
            );
        }
        Some(ResolvedDispatchResult::Found(Box::new(self.authorize_dispatch(ResolvedDispatch {
            callable: member.callable.clone(),
            signature,
            visibility: member.visibility,
            access_denied: false,
            specialization: None,
            conditional: None,
            conditional_capability: None,
            trait_dispatch: None,
            abstract_contract: true,
            visited_owners: Box::new([surface.declaration.clone()]),
        }))))
    }

    fn resolve_assumed_trait_target(&mut self, receiver: TypeId, selector: &Selector, side: DispatchSide) -> Option<ResolvedDispatchResult> {
        let mut assumed_candidates = Vec::new();
        for key in self.generic_evidence.conformances().keys() {
            let subject_matches = match &key.subject {
                crate::types::parameter::TypeTerm::Canonical(ty) => *ty == receiver,
                crate::types::parameter::TypeTerm::SelfType(st) => {
                    matches!(self.store.get(receiver), crate::types::store::TypeData::SelfType(term) if term == st)
                }
                crate::types::parameter::TypeTerm::Infer(_) => false,
            };
            if subject_matches {
                if let Some(view) = self.conformance_semantics {
                    if let Some(surface) = view.trait_surfaces.get(&key.trait_ref.declaration) {
                        if let Some(member) = surface.get_by_selector(selector, side) {
                            assumed_candidates.push((key.clone(), member.clone(), surface.clone()));
                        }
                    }
                } else if let Some(surface) = self.trait_surface {
                    if surface.declaration == key.trait_ref.declaration {
                        if let Some(member) = surface.get_by_selector(selector, side) {
                            assumed_candidates.push((key.clone(), member.clone(), std::sync::Arc::new((*surface).clone())));
                        }
                    }
                }
            }
        }

        if assumed_candidates.is_empty() {
            return None;
        }

        if assumed_candidates.len() == 1 {
            let (conformance_key, mut member, surface) = assumed_candidates.remove(0);
            if member.default_present {
                if let Some(canonical_signature) = self
                    .conformance_semantics
                    .and_then(|view| view.callable_signatures)
                    .and_then(|table| table.get(&member.callable))
                    .filter(|signature| signature.inferred_return.is_some())
                    .cloned()
                {
                    member.signature = canonical_signature;
                    self.record_semantic_dependency(SemanticDependency::CallableSignature(member.callable.clone()));
                }
            }
            let mut signature = crate::checker::declaration_signature::project_semantic_signature(&member.signature);
            let mut type_env = crate::types::environment::TypeEnvironment::default();
            // The proven subject may be a rigid caller parameter. Trait
            // `Self` denotes that subject for either requirement side.
            type_env.bind_self(receiver).expect("dispatch type environment is source mutable");
            if let Some(generics) = &surface.generic_signature {
                for (idx, &param) in generics.parameters.iter().enumerate() {
                    if let Some(&arg) = conformance_key.trait_ref.arguments.get(idx) {
                        type_env.bind_param(param, arg).expect("dispatch type environment is source mutable");
                    }
                }
            }
            let env_clone = type_env.clone();
            let specialize_type = |ctx: &mut Self, ty: TypeId| {
                let self_spec = ctx.specialize_self_type(receiver, ty);
                crate::types::environment::TypeView::new(self_spec, env_clone.clone()).materialize(ctx.store)
            };
            for parameter in &mut signature.parameters {
                parameter.ty = parameter.ty.map_type(|ty| specialize_type(self, ty));
            }
            signature.return_type = signature.return_type.map_type(|ty| specialize_type(self, ty));

            let callable = member.callable.clone();
            self.dependencies.insert(callable.clone());
            self.record_semantic_dependency(SemanticDependency::TraitSurface(surface.declaration.clone()));
            if let Some(expression) = self.current_expression_id() {
                self.resolved_callables.insert(expression, callable.clone());
                self.resolved_trait_dispatches.insert(
                    expression,
                    crate::trait_dispatch::TraitDispatchSite::Assumed(crate::trait_dispatch::AssumedTraitDispatchSelection {
                        conformance: conformance_key,
                        requirement: member.requirement.clone(),
                        signature: member.signature.clone(),
                    }),
                );
            }
            Some(ResolvedDispatchResult::Found(Box::new(self.authorize_dispatch(ResolvedDispatch {
                callable,
                signature,
                visibility: assumed_candidates.first().map(|(_, member, _)| member.visibility).unwrap_or_default(),
                access_denied: false,
                specialization: None,
                conditional: None,
                conditional_capability: None,
                trait_dispatch: None,
                abstract_contract: false,
                visited_owners: vec![].into_boxed_slice(),
            }))))
        } else {
            let ambiguous = assumed_candidates
                .into_iter()
                .map(|(_, member, _)| ResolvedDispatch {
                    callable: member.callable.clone(),
                    signature: crate::checker::declaration_signature::project_semantic_signature(&member.signature),
                    visibility: member.visibility,
                    access_denied: false,
                    specialization: None,
                    conditional: None,
                    conditional_capability: None,
                    trait_dispatch: None,
                    abstract_contract: false,
                    visited_owners: vec![].into_boxed_slice(),
                })
                .collect();
            Some(ResolvedDispatchResult::Ambiguous(ambiguous))
        }
    }

    fn apply_inferred_trait_default_signature(&mut self, selection: &mut crate::trait_dispatch::TraitDispatchSelection, callable: &CallableId) {
        if !matches!(&selection.selection, crate::impls::RequirementSelectionTemplate::TraitDefault { .. }) {
            return;
        }
        let Some(view) = self.conformance_semantics else {
            return;
        };
        let Some(canonical_signature) = view.callable_signatures.and_then(|table| table.get(callable)) else {
            return;
        };
        let Some(projected) = crate::impls::project_inferred_trait_default_signature(
            self.store,
            selection,
            canonical_signature,
            view.trait_surfaces,
            view.conformance_index,
            view.witness_plans,
            self.declarations,
            self.hierarchy.inner(),
        ) else {
            return;
        };
        selection.signature = projected;
        self.record_semantic_dependency(SemanticDependency::CallableSignature(callable.clone()));
    }

    fn resolve_evidenced_trait_target(&mut self, receiver: TypeId, selector: &Selector, side: DispatchSide) -> Option<ResolvedDispatchResult> {
        let view = self.conformance_semantics?;
        let resolution = crate::trait_dispatch::resolve_trait_evidenced_candidates(
            view.trait_dispatch,
            view.conformance_index,
            view.witness_plans,
            view.trait_surfaces,
            view.derived_conformance_rules,
            Some(&self.generic_evidence),
            self.declarations,
            self.store,
            &self.hierarchy,
            receiver,
            selector,
            side,
        );
        match resolution {
            crate::trait_dispatch::TraitDispatchResolution::Found(mut selection) => {
                let callable = selection.callable.clone().unwrap_or_else(|| selection.requirement.source_callable());
                self.apply_inferred_trait_default_signature(&mut selection, &callable);
                let signature = crate::checker::declaration_signature::project_semantic_signature(&selection.signature);
                self.dependencies.insert(callable.clone());
                self.record_semantic_dependency(SemanticDependency::ConformanceDispatch(selection.source_impl.clone()));
                self.record_semantic_dependency(SemanticDependency::TraitSurface(selection.requirement.owner.clone()));
                if let crate::generic_evidence::GenericConformanceEvidence::Derived(evidence) = &selection.evidence {
                    self.record_derived_conformance_dependencies(evidence);
                }
                if let Some(expression) = self.current_expression_id() {
                    self.resolved_callables.insert(expression, callable.clone());
                    self.resolved_trait_dispatches
                        .insert(expression, crate::trait_dispatch::TraitDispatchSite::Evidenced(selection.clone()));
                }
                let visibility = view
                    .trait_surfaces
                    .get(&selection.requirement.owner)
                    .and_then(|surface| surface.get(&selection.requirement))
                    .map(|member| member.visibility)
                    .unwrap_or(crate::surface::MemberVisibility::Public);
                Some(ResolvedDispatchResult::Found(Box::new(self.authorize_dispatch(ResolvedDispatch {
                    callable,
                    signature,
                    visibility,
                    access_denied: false,
                    specialization: None,
                    conditional: None,
                    conditional_capability: None,
                    trait_dispatch: Some(selection),
                    abstract_contract: false,
                    visited_owners: vec![].into_boxed_slice(),
                }))))
            }
            crate::trait_dispatch::TraitDispatchResolution::Ambiguous(candidates) => {
                if let Some(expression) = self.current_expression_id() {
                    self.resolved_trait_dispatch_candidates.insert(expression, candidates.clone());
                }
                let ambiguous = candidates
                    .into_iter()
                    .map(|candidate| {
                        let callable =
                            crate::trait_dispatch::selected_callable(&candidate.selection).unwrap_or_else(|| candidate.requirement.source_callable());
                        let visibility = view
                            .trait_surfaces
                            .get(&candidate.requirement.owner)
                            .and_then(|surface| surface.get(&candidate.requirement))
                            .map(|member| member.visibility)
                            .unwrap_or(crate::surface::MemberVisibility::Public);
                        ResolvedDispatch {
                            callable,
                            signature: crate::checker::declaration_signature::project_semantic_signature(&candidate.signature),
                            visibility,
                            access_denied: false,
                            specialization: None,
                            conditional: None,
                            conditional_capability: None,
                            trait_dispatch: None,
                            abstract_contract: false,
                            visited_owners: vec![].into_boxed_slice(),
                        }
                    })
                    .collect();
                Some(ResolvedDispatchResult::Ambiguous(ambiguous))
            }
            crate::trait_dispatch::TraitDispatchResolution::Missing => None,
            crate::trait_dispatch::TraitDispatchResolution::Incomplete(impl_id) => Some(ResolvedDispatchResult::TraitTerminal(
                crate::trait_dispatch::TraitDispatchTerminal::Incomplete(impl_id),
            )),
            crate::trait_dispatch::TraitDispatchResolution::CoherenceConflict(impl_ids) => Some(ResolvedDispatchResult::TraitTerminal(
                crate::trait_dispatch::TraitDispatchTerminal::CoherenceConflict(impl_ids),
            )),
            crate::trait_dispatch::TraitDispatchResolution::Unknown(reason) => Some(ResolvedDispatchResult::TraitTerminal(
                crate::trait_dispatch::TraitDispatchTerminal::Unknown(reason),
            )),
            crate::trait_dispatch::TraitDispatchResolution::Blocked(reason) => Some(ResolvedDispatchResult::TraitTerminal(
                crate::trait_dispatch::TraitDispatchTerminal::Blocked(reason),
            )),
            crate::trait_dispatch::TraitDispatchResolution::Dynamic(obligation) => Some(ResolvedDispatchResult::TraitTerminal(
                crate::trait_dispatch::TraitDispatchTerminal::Dynamic(obligation),
            )),
            crate::trait_dispatch::TraitDispatchResolution::Cancelled => {
                Some(ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Cancelled))
            }
            crate::trait_dispatch::TraitDispatchResolution::BudgetExceeded(report) => Some(ResolvedDispatchResult::TraitTerminal(
                crate::trait_dispatch::TraitDispatchTerminal::BudgetExceeded(report),
            )),
            crate::trait_dispatch::TraitDispatchResolution::InternalFailure(message) => Some(ResolvedDispatchResult::TraitTerminal(
                crate::trait_dispatch::TraitDispatchTerminal::InternalFailure(message),
            )),
        }
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
        fallback_side: DispatchSide,
        selector: &Selector,
        lookup: crate::dispatch::DispatchLookup,
    ) -> ResolvedDispatchResult {
        if let Some(result) = self.resolve_trait_contract_target(dispatch_receiver, selector, &lookup) {
            return result;
        }
        let Some((decl, side)) = self.dispatch_owner_for_lookup(dispatch_receiver, lookup.clone()) else {
            if let Some(result) = self.resolve_assumed_trait_target(dispatch_receiver, selector, fallback_side) {
                return result;
            }
            if let Some(result) = self.resolve_evidenced_trait_target(dispatch_receiver, selector, fallback_side) {
                return result;
            }
            return ResolvedDispatchResult::Missing { visited_owners: Box::new([]) };
        };

        // Keep the type form that identifies a conformance subject separate
        // from the class-object receiver used for ordinary class-side lookup.
        // A raw nominal form is not itself a runtime dispatch receiver, but
        // it is the exact target needed to query a class-side witness.
        let conformance_receiver = specialization_receiver.unwrap_or(dispatch_receiver);
        let result = self.dispatch.get().resolve_dispatch_with_trace(&self.hierarchy, &decl, side, selector);
        // A raw class type form carries declaration parameters only as a
        // template. Keep class-object `Self` formation for that case; applied
        // forms carry the actual receiver arguments used for specialization.
        let specialization_receiver = specialization_receiver
            .filter(|receiver| matches!(self.store.get(*receiver), TypeData::Applied { .. } | TypeData::ExactCase { .. }))
            .unwrap_or(dispatch_receiver);
        let visited_owners = match &result {
            ResolvedDispatchResult::Found(resolved) => resolved.visited_owners.clone(),
            ResolvedDispatchResult::Ambiguous(resolved) => resolved.first().map(|resolved| resolved.visited_owners.clone()).unwrap_or_else(|| Box::new([])),
            ResolvedDispatchResult::Missing { visited_owners } => visited_owners.clone(),
            ResolvedDispatchResult::TraitTerminal(_) | ResolvedDispatchResult::Dynamic => Box::new([]),
        };
        if let Some(resolved) = self.resolve_exact_case_conditional(specialization_receiver, side, selector, visited_owners) {
            return ResolvedDispatchResult::Found(Box::new(self.authorize_dispatch(resolved)));
        }
        match result {
            ResolvedDispatchResult::Found(mut resolved) => {
                resolved = Box::new(self.authorize_dispatch(*resolved));
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
                    resolved.access_denied = !self.dispatch_is_accessible(resolved);
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

                for owner in visited_owners.iter() {
                    let cond_set = self
                        .dispatch
                        .get()
                        .get_conditional_members(&crate::impls::InherentImplTarget::Declaration(owner.clone()))
                        .cloned();
                    if let Some(cond_set) = cond_set {
                        if let Some(member) = cond_set.get_member(side, selector) {
                            if let Ok(receiver_spec) =
                                specialize_receiver_to_owner(&mut *self.store, &self.hierarchy, specialization_receiver, owner, &self.control)
                            {
                                let owner_type = receiver_spec.path.last().map(|s| s.specialized_form).unwrap_or(specialization_receiver);
                                let proof_context = crate::impls::ConditionalImplProofContext {
                                    lexical_evidence: Some(&self.generic_evidence),
                                    conformance_index: self.conformance_semantics.map(|view| view.conformance_index),
                                    witness_plans: self.conformance_semantics.map(|view| view.witness_plans),
                                    trait_surfaces: self.conformance_semantics.map(|view| view.trait_surfaces),
                                    derived_rules: self.conformance_semantics.map(|view| view.derived_conformance_rules),
                                    declarations: Some(self.declarations),
                                };
                                let app = crate::impls::check_impl_domain_applicability_with_proofs(
                                    self.store,
                                    &self.hierarchy,
                                    &member.domain,
                                    specialization_receiver,
                                    owner_type,
                                    &self.ambient_constraints,
                                    &proof_context,
                                );
                                if let crate::impls::ImplApplicabilityResult::Applicable(spec) = app {
                                    let selection = crate::dispatch::ConditionalDispatchSelection {
                                        impl_id: member.impl_id.clone(),
                                        callable: member.callable.clone(),
                                        declaring_owner: owner.clone(),
                                        side,
                                    };
                                    if let Some(capability) = self.conditional_selected_call_capability(selection.clone(), &member.domain, spec) {
                                        let mut signature = crate::checker::declaration_signature::project_semantic_signature(&member.signature_template);
                                        let unspecialized_return = signature.return_type.clone();
                                        let mut environment = receiver_spec.environment;
                                        for (&p, &t) in &capability.applicability.bindings {
                                            environment.bind_param(p, t).expect("conditional dispatch environment is source mutable");
                                        }
                                        let env = environment.clone();
                                        let specialize_type = |ctx: &mut Self, ty| {
                                            let self_specialized = ctx.specialize_self_type(specialization_receiver, ty);
                                            crate::types::environment::TypeView::new(self_specialized, env.clone()).materialize(ctx.store)
                                        };
                                        for parameter in &mut signature.parameters {
                                            parameter.ty = parameter.ty.map_type(|ty| specialize_type(self, ty));
                                        }
                                        signature.return_type = signature.return_type.map_type(|ty| specialize_type(self, ty));
                                        self.dependencies.insert(member.callable.clone());
                                        self.record_consumed_callable_signature(&member.callable, &signature);
                                        if let Some(expression) = self.current_expression_id() {
                                            self.resolved_callables.insert(expression, member.callable.clone());
                                            self.resolved_conditional_dispatches.insert(expression, selection.clone());
                                            self.resolved_conditional_capabilities.insert(expression, capability.clone());
                                        }
                                        return ResolvedDispatchResult::Found(Box::new(self.authorize_dispatch(ResolvedDispatch {
                                            callable: member.callable.clone(),
                                            signature,
                                            visibility: member.visibility,
                                            access_denied: false,
                                            specialization: Some(crate::dispatch::DispatchSignatureSpecialization {
                                                receiver: specialization_receiver,
                                                declaring_owner: owner.clone(),
                                                environment,
                                                path: receiver_spec.path,
                                                unspecialized_return,
                                            }),
                                            conditional: Some(selection),
                                            conditional_capability: Some(capability),
                                            trait_dispatch: None,
                                            abstract_contract: false,
                                            visited_owners,
                                        })));
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(result) = self.resolve_assumed_trait_target(conformance_receiver, selector, side) {
                    return result;
                }

                if let Some(view) = self.conformance_semantics {
                    let trait_resolution = crate::trait_dispatch::resolve_trait_evidenced_candidates(
                        view.trait_dispatch,
                        view.conformance_index,
                        view.witness_plans,
                        view.trait_surfaces,
                        view.derived_conformance_rules,
                        Some(&self.generic_evidence),
                        self.declarations,
                        self.store,
                        &self.hierarchy,
                        conformance_receiver,
                        selector,
                        side,
                    );
                    match trait_resolution {
                        crate::trait_dispatch::TraitDispatchResolution::Found(mut selection) => {
                            let callable = selection.callable.clone().unwrap_or_else(|| selection.requirement.source_callable());
                            self.apply_inferred_trait_default_signature(&mut selection, &callable);
                            let signature = crate::checker::declaration_signature::project_semantic_signature(&selection.signature);
                            self.dependencies.insert(callable.clone());
                            self.record_semantic_dependency(SemanticDependency::ConformanceDispatch(selection.source_impl.clone()));
                            self.record_semantic_dependency(SemanticDependency::TraitSurface(selection.requirement.owner.clone()));
                            if let crate::generic_evidence::GenericConformanceEvidence::Derived(evidence) = &selection.evidence {
                                self.record_derived_conformance_dependencies(evidence);
                            }
                            if let Some(expression) = self.current_expression_id() {
                                self.resolved_callables.insert(expression, callable.clone());
                                self.resolved_trait_dispatches
                                    .insert(expression, crate::trait_dispatch::TraitDispatchSite::Evidenced(selection.clone()));
                            }
                            return ResolvedDispatchResult::Found(Box::new(
                                self.authorize_dispatch(ResolvedDispatch {
                                    callable,
                                    signature,
                                    visibility: view
                                        .trait_surfaces
                                        .get(&selection.requirement.owner)
                                        .and_then(|surface| surface.get(&selection.requirement))
                                        .map(|member| member.visibility)
                                        .unwrap_or(crate::surface::MemberVisibility::Public),
                                    access_denied: false,
                                    specialization: None,
                                    conditional: None,
                                    conditional_capability: None,
                                    trait_dispatch: Some(selection),
                                    abstract_contract: false,
                                    visited_owners: vec![].into_boxed_slice(),
                                }),
                            ));
                        }
                        crate::trait_dispatch::TraitDispatchResolution::Ambiguous(candidates) => {
                            if let Some(expression) = self.current_expression_id() {
                                self.resolved_trait_dispatch_candidates.insert(expression, candidates.clone());
                            }
                            let ambiguous = candidates
                                .into_iter()
                                .map(|candidate| ResolvedDispatch {
                                    callable: crate::trait_dispatch::selected_callable(&candidate.selection)
                                        .unwrap_or_else(|| candidate.requirement.source_callable()),
                                    signature: crate::checker::declaration_signature::project_semantic_signature(&candidate.signature),
                                    visibility: view
                                        .trait_surfaces
                                        .get(&candidate.requirement.owner)
                                        .and_then(|surface| surface.get(&candidate.requirement))
                                        .map(|member| member.visibility)
                                        .unwrap_or(crate::surface::MemberVisibility::Public),
                                    access_denied: false,
                                    specialization: None,
                                    conditional: None,
                                    conditional_capability: None,
                                    trait_dispatch: None,
                                    abstract_contract: false,
                                    visited_owners: vec![].into_boxed_slice(),
                                })
                                .collect();
                            return ResolvedDispatchResult::Ambiguous(ambiguous);
                        }
                        crate::trait_dispatch::TraitDispatchResolution::Missing => {}
                        crate::trait_dispatch::TraitDispatchResolution::Incomplete(impl_id) => {
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Incomplete(impl_id));
                        }
                        crate::trait_dispatch::TraitDispatchResolution::CoherenceConflict(impl_ids) => {
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::CoherenceConflict(impl_ids));
                        }
                        crate::trait_dispatch::TraitDispatchResolution::Unknown(reason) => {
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Unknown(reason));
                        }
                        crate::trait_dispatch::TraitDispatchResolution::Blocked(reason) => {
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Blocked(reason));
                        }
                        crate::trait_dispatch::TraitDispatchResolution::Dynamic(obligation) => {
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Dynamic(obligation));
                        }
                        crate::trait_dispatch::TraitDispatchResolution::Cancelled => {
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Cancelled);
                        }
                        crate::trait_dispatch::TraitDispatchResolution::BudgetExceeded(report) => {
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::BudgetExceeded(report));
                        }
                        crate::trait_dispatch::TraitDispatchResolution::InternalFailure(message) => {
                            return ResolvedDispatchResult::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::InternalFailure(message));
                        }
                    }
                }
                ResolvedDispatchResult::Missing { visited_owners }
            }
            ResolvedDispatchResult::TraitTerminal(terminal) => ResolvedDispatchResult::TraitTerminal(terminal),
            ResolvedDispatchResult::Dynamic => ResolvedDispatchResult::Dynamic,
        }
    }

    pub fn resolve_dispatch(&mut self, receiver: TypeId, selector: &Selector, lookup: crate::dispatch::DispatchLookup) -> DispatchResult {
        match self.resolve_dispatch_target(receiver, selector, lookup) {
            ResolvedDispatchResult::Found(resolved) => DispatchResult::Found(Box::new(resolved.signature)),
            ResolvedDispatchResult::Ambiguous(ambiguous) => DispatchResult::Ambiguous(ambiguous.into_iter().map(|resolved| resolved.signature).collect()),
            ResolvedDispatchResult::Missing { .. } => DispatchResult::Missing,
            ResolvedDispatchResult::TraitTerminal(_) | ResolvedDispatchResult::Dynamic => DispatchResult::Dynamic,
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

    /// Resolves the canonical field signature for declaration-owned semantic
    /// elaboration while recording the same dependency as field reads.
    pub(crate) fn resolve_field_signature(&self, owner: &DeclarationId, side: DispatchSide, name: &str) -> Option<&crate::signature::FieldSemanticSignature> {
        let field = crate::identity::FieldId::new(owner.clone(), name, side);
        let signature = self.field_signatures?.get(&field)?;
        if is_query_owned_module(&field.owner.module) {
            self.record_semantic_dependency(SemanticDependency::FieldSignature(field));
        }
        Some(signature)
    }

    /// Resolves a canonical source callable signature while
    /// recording the signature query as a dependency of the current owner.
    pub(crate) fn resolve_callable_signature(&self, callable: &CallableId) -> Option<&crate::signature::CallableSemanticSignature> {
        let signature = self.callable_signatures.and_then(|table| table.get(callable)).or_else(|| {
            self.conformance_semantics
                .and_then(|view| view.callable_signatures)
                .and_then(|table| table.get(callable))
        })?;
        if is_query_owned_module(callable.module()) {
            self.record_semantic_dependency(SemanticDependency::CallableSignature(callable.clone()));
        }
        Some(signature)
    }

    /// Resolves the nearest declaration-owned field signature in the class
    /// hierarchy, preserving the declaring field's canonical `FieldId`.
    pub(crate) fn resolve_field_signature_in_hierarchy(
        &self,
        owner: &DeclarationId,
        side: DispatchSide,
        name: &str,
    ) -> Option<&crate::signature::FieldSemanticSignature> {
        let mut current = owner.clone();
        let mut visited = BTreeSet::new();
        while visited.insert(current.clone()) {
            if let Some(signature) = self.resolve_field_signature(&current, side, name) {
                return Some(signature);
            }
            let Some(parent) = self.hierarchy.superclass(&current) else {
                break;
            };
            current = parent.clone();
        }
        None
    }

    pub(crate) fn resolve_field_contract_in_hierarchy(
        &self,
        owner: &DeclarationId,
        side: DispatchSide,
        name: &str,
    ) -> Option<(crate::identity::FieldId, TypeKnowledge)> {
        let signature = self.resolve_field_signature_in_hierarchy(owner, side, name)?;
        Some((signature.field.clone(), signature.declared_type.to_knowledge()))
    }

    pub(crate) fn resolve_field_read(
        &self,
        owner: &DeclarationId,
        side: DispatchSide,
        name: &str,
    ) -> Option<(crate::identity::FieldId, TypeKnowledge, crate::checker::causal::CausalInvalidity)> {
        let (field, contract) = self.resolve_field_contract_in_hierarchy(owner, side, name)?;
        let checking_same_receiver_hierarchy = self.current_side == side
            && self
                .current_class
                .as_ref()
                .is_some_and(|current_class| self.hierarchy.is_subclass(current_class, &field.owner));
        if checking_same_receiver_hierarchy {
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
        self.resolver.resolve_type_name(&self.current_module, name, &[]).or_else(|| {
            let current = self.current_class.as_ref()?;
            (current.module == self.current_module && current.name.as_ref() == name && self.declarations.contains(current)).then(|| current.clone())
        })
    }

    pub fn resolve_type_name_with_path(&self, root: &str, members: &[String]) -> Option<DeclarationId> {
        self.resolver.resolve_type_name(&self.current_module, root, members)
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
        self.explanations
            .alloc_full(step, rule, status, origin, evidence, parents)
            .expect("checking context explanation arena is source mutable")
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
        let mut status = status;
        if !self.ensure_sequence_publication_settled(body_range) && matches!(status, super::analysis::CallableAnalysisStatus::Complete) {
            status = match self.terminal_status {
                Some(AnalysisStatus::InternalFailure(incident)) => super::analysis::CallableAnalysisStatus::InternalFailure(incident),
                _ => super::analysis::CallableAnalysisStatus::Blocked,
            };
        }
        let return_validation = self.validate_return_contract(status, &normal_returns);
        let frozen_proofs = match crate::types::proof::freeze_callable_proofs(&self.rigids, self.store, &self.match_resolutions, &self.pattern_site_resolutions)
        {
            Ok(proofs) => proofs,
            Err(error) => {
                let incident = self.publish_analysis_incident(format!("callable proof publication failed: {error:?}"));
                status = super::analysis::CallableAnalysisStatus::InternalFailure(incident);
                crate::types::proof::FrozenCallableProofs {
                    graph: Default::default(),
                    matches: Default::default(),
                    sites: Default::default(),
                }
            }
        };
        let semantic_dependencies = self.semantic_dependencies_snapshot();
        let entry_flow = self.current_flow_summary();
        let flow_graph = self
            .flow_graph
            .unwrap_or_else(|| std::sync::Arc::new(crate::checker::flow::graph::FlowGraph::default()));

        let return_summary = crate::checker::analysis::normal_return_summary(self.store, &normal_returns);
        let return_explanation = Some(
            self.explanations
                .alloc(
                    crate::explain::ExplanationStep::CallableReturnSummary {
                        callable: callable.clone(),
                        returns: normal_returns.iter().map(|exit| exit.knowledge.clone()).collect::<Vec<_>>().into_boxed_slice(),
                        result: return_summary.clone(),
                    },
                    return_summary.status().unwrap_or(crate::types::evidence::EvidenceStatus::Assumed),
                    return_summary.origin().unwrap_or(crate::types::evidence::EvidenceOrigin::Flow),
                    Vec::new(),
                )
                .expect("checking context explanation arena is source mutable"),
        );
        let exits = crate::checker::analysis::BodyExitFacts {
            normal_returns,
            throws: self.throw_exit_flows,
            unreachable: false,
        };
        let mut bindings: crate::checker::analysis::BindingAnalysisIndex = self.binding_history.into();
        bindings
            .extend_source(self.flow.bindings)
            .expect("checking context binding products are source mutable");
        let binding_origins = bindings
            .keys()
            .map(|binding| {
                (
                    *binding,
                    self.seeded_module_value_origins
                        .get(binding)
                        .cloned()
                        .unwrap_or(crate::checker::analysis::CallableBindingOrigin::BodyLocal),
                )
            })
            .collect::<crate::checker::analysis::CallableBindingOriginIndex>();

        crate::checker::analysis::CallableAnalysis {
            callable,
            body_range,
            body_source_owner: None,
            expressions: self.expressions,
            for_iteration_lanes: std::sync::Arc::new(self.for_iteration_lanes),
            positional_expansions: std::sync::Arc::new(self.positional_expansions),
            known_invalid_positional_expansions: std::sync::Arc::new(self.known_invalid_positional_expansions),
            sequence_literals: std::sync::Arc::new(self.sequence_literals),
            bindings,
            binding_origins,
            associated_resolutions: std::sync::Arc::new(self.associated_resolutions),
            callable_reference_resolutions: std::sync::Arc::new(self.callable_reference_resolutions),
            family_applications: std::sync::Arc::new(self.family_applications),
            match_resolutions: std::sync::Arc::new(frozen_proofs.matches),
            pattern_site_resolutions: std::sync::Arc::new(frozen_proofs.sites),
            local_proof_graph: std::sync::Arc::new(frozen_proofs.graph),
            flow_graph,
            entry_flow,
            exits,
            return_validation,
            diagnostics: std::sync::Arc::from(self.diagnostics.into_boxed_slice()),
            internal_incidents: std::sync::Arc::from(self.analysis_incidents.into_values().collect::<Vec<_>>().into_boxed_slice()),
            explanations: std::sync::Arc::new(self.explanations),
            return_explanation,
            dependencies: std::sync::Arc::from(self.dependencies.into_iter().collect::<Vec<_>>().into_boxed_slice()),
            semantic_dependencies: std::sync::Arc::from(semantic_dependencies.into_iter().collect::<Vec<_>>().into_boxed_slice()),
            constructor_super_initializer: self.constructor_super_initializer,
            dependency_fingerprint: crate::db::ProductFingerprint::new(0),
            status,
            observed_import_phases: std::sync::Arc::new(crate::checker::analysis_map::AnalysisMap::from_source(self.observed_import_phases)),
        }
    }

    pub fn record_associated_resolution(&mut self, expr_id: ExpressionId, resolution: crate::checker::associated::AssociatedResolution) {
        self.associated_resolutions
            .insert_source(expr_id, resolution)
            .expect("checking context associated resolutions are source mutable");
    }

    pub fn record_callable_reference_resolution(&mut self, expr_id: ExpressionId, resolution: crate::checker::associated::CallableReferenceResolution) {
        self.callable_reference_resolutions
            .insert_source(expr_id, resolution)
            .expect("checking context callable references are source mutable");
    }

    pub fn record_family_application(&mut self, expr_id: ExpressionId, resolution: crate::checker::associated::FamilyApplicationResolution) {
        self.family_applications
            .insert_source(expr_id, resolution)
            .expect("checking context family applications are source mutable");
    }

    pub fn record_match_resolution(&mut self, expr_id: ExpressionId, resolution: crate::match_semantics::MatchResolution) {
        self.match_resolutions
            .insert_source(expr_id, resolution)
            .expect("checking context match resolutions are source mutable");
    }

    pub fn record_pattern_site_resolution(&mut self, resolution: crate::match_semantics::PatternSiteResolution) {
        let key = crate::match_semantics::PatternSiteKey::new(resolution.kind, resolution.range);
        self.pattern_site_resolutions
            .insert_source(key, resolution)
            .expect("checking context pattern resolutions are source mutable");
    }

    pub fn record_proof_predicate(&mut self, resolution: crate::checker::proof_predicate::ProofPredicateResolution) {
        self.proof_predicates.insert(resolution.expression, resolution);
    }

    pub fn record_data_declaration_dependency(&self, decl: &DeclarationId) {
        self.semantic_dependencies
            .borrow_mut()
            .insert(crate::checker::analysis::SemanticDependency::DataDeclaration(decl.clone()));
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

fn local_type_is_soundly_widenable(ctx: &mut CheckingContext<'_>, local_type: &LocalType, expected: TypeId, constraints: &[LocalConstraint]) -> bool {
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
        LocalType::Union(members) => members.iter().all(|member| local_type_is_soundly_widenable(ctx, member, expected, constraints)),
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
            LocalType::Applied {
                origin: left_origin,
                arguments: left_arguments,
            },
            LocalType::Applied {
                origin: right_origin,
                arguments: right_arguments,
            },
        ) if left_arguments.len() == right_arguments.len() => {
            collect_equivalent_rigid_bindings(left_origin, right_origin, bindings);
            for (left, right) in left_arguments.iter().zip(right_arguments.iter()) {
                collect_equivalent_rigid_bindings(left, right, bindings);
            }
        }
        (
            LocalType::ExactCase {
                variant: left_variant,
                enum_type: left_enum,
            },
            LocalType::ExactCase {
                variant: right_variant,
                enum_type: right_enum,
            },
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
            for (left, right) in left.iter().zip(right.iter()).filter(|(left, right)| left.key == right.key) {
                collect_equivalent_rigid_bindings(&left.ty, &right.ty, bindings);
            }
        }
        (
            LocalType::Callable {
                parameters: left_parameters,
                return_type: left_return,
            },
            LocalType::Callable {
                parameters: right_parameters,
                return_type: right_return,
            },
        ) if left_parameters.len() == right_parameters.len() => {
            for (left, right) in left_parameters
                .iter()
                .zip(right_parameters.iter())
                .filter(|(left, right)| left.label == right.label && left.rest == right.rest)
            {
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
                    key: field.key.clone(),
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
    use super::{CheckingContext, TrackingTypeResolver};
    use crate::checker::binding::{BindingContract, BindingContractOrigin};
    use crate::checker::causal::CausalInvalidity;
    use crate::checker::flow::state::FlowInvariantFailure;
    use crate::checker::incident::{InternalSemanticIncidentDetails, InternalSemanticIncidentKind};
    use crate::declarations::bootstrap_universe_declarations;
    use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
    use crate::dispatch::{CallableSignature, ResolvedDispatchResult};
    use crate::identity::{CallableId, DeclarationId, DispatchSide, ModuleId};
    use crate::types::SimpleTypeResolver;
    use crate::types::annotation::TypeResolver;
    use crate::types::id::TypeId;
    use crate::types::relation::MapTypeHierarchy;
    use crate::types::store::TypeStore;
    use crate::types::type_member::{TypeMemberDeclarationId, TypeMemberSelector};
    use phalcom_common::range::SourceRange;
    use std::cell::RefCell;
    use std::collections::BTreeSet;
    use std::rc::Rc;

    struct IdentityOnlyTypeResolver(TypeMemberDeclarationId);

    impl TypeResolver for IdentityOnlyTypeResolver {
        fn resolve_type_name(&self, _current_module: &ModuleId, _root: &str, _members: &[String]) -> Option<DeclarationId> {
            None
        }

        fn resolve_associated_type_member_id(&self, _owner: &DeclarationId, _selector: &TypeMemberSelector) -> Option<TypeMemberDeclarationId> {
            Some(self.0.clone())
        }

        fn resolve_associated_type_member(
            &self,
            _owner: &DeclarationId,
            _selector: &TypeMemberSelector,
        ) -> Option<std::sync::Arc<crate::db::product::TypeMemberDeclarationProduct>> {
            panic!("an unready product must not be fetched from the stale resolver view")
        }
    }

    #[test]
    fn tracking_resolver_demands_the_exact_unready_type_member_owner() {
        let module = ModuleId::universe_root();
        let owner = DeclarationId::new(module, "Owner".into());
        let id = TypeMemberDeclarationId::new(owner.clone(), TypeMemberSelector::named("Item"));
        let key = crate::db::key::QueryKey::TypeMemberDeclaration(id.clone());
        let inner = IdentityOnlyTypeResolver(id);
        let dependencies = Rc::new(RefCell::new(BTreeSet::new()));
        let demands = Rc::new(RefCell::new(BTreeSet::new()));
        let resolver = TrackingTypeResolver {
            inner: &inner,
            dependencies,
            query_demands: demands.clone(),
            deferred_static_definitions: Rc::new(RefCell::new(BTreeSet::new())),
        };

        let product = resolver.resolve_associated_type_member(&owner, &TypeMemberSelector::named("Item"));

        assert!(product.is_none());
        assert_eq!(*demands.borrow(), BTreeSet::from([key]));
    }

    #[test]
    fn tracking_resolver_demands_only_a_known_deferred_static_definition() {
        let module = ModuleId::universe_root();
        let declaration = DeclarationId::new(module, "DeferredAlias".into());
        let inner = crate::types::SimpleTypeResolver::new();
        let dependencies = Rc::new(RefCell::new(BTreeSet::new()));
        let demands = Rc::new(RefCell::new(BTreeSet::new()));
        let resolver = TrackingTypeResolver {
            inner: &inner,
            dependencies,
            query_demands: demands.clone(),
            deferred_static_definitions: Rc::new(RefCell::new(BTreeSet::new())),
        };

        assert!(resolver.resolve_alias_form(&declaration).is_none());
        assert!(demands.borrow().is_empty(), "ordinary absent aliases are not scheduler demands");

        resolver.track_deferred_static_definition(declaration.clone());
        assert!(resolver.resolve_alias_form(&declaration).is_none());
        assert_eq!(*demands.borrow(), BTreeSet::from([crate::db::QueryKey::StaticDefinition(declaration)]));
    }

    #[test]
    fn expression_owner_stack_unwinds_in_all_build_profiles() {
        let module = ModuleId::universe_root();
        let mut store = TypeStore::new();
        let declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(module.clone(), key.name().into()));
        let resolver = SimpleTypeResolver::new();
        let hierarchy = MapTypeHierarchy::new();
        let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module);
        let outer = ctx.alloc_expression_id();
        let inner = ctx.alloc_expression_id();
        ctx.push_expression_owner(outer);
        ctx.push_expression_owner(inner);
        ctx.pop_expression_owner(inner);
        assert_eq!(ctx.current_expression_id(), Some(outer));
        ctx.pop_expression_owner(outer);
        assert_eq!(ctx.current_expression_id(), None);
    }

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
        declarations.replace_authoritative(crate::declarations::DeclarationTypeInfo {
            declaration: owner.clone(),
            form: owner_form,
            class_object_type: owner_class_object,
            kind: crate::types::id::KindId::TYPE,
            binding: crate::declarations::DeclarationTypeBinding::ordinary(),
            generic_binder_header: None,
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
