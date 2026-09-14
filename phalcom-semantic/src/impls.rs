//! Canonical inherent implementation fragment analysis, target resolution, and contribution publication.

use crate::checker::context::{CheckerControl, CheckingContext};
use crate::checker::declaration_signature::{CallableSyntaxRef, semantic_signature_for_syntax_with_resolver};
use crate::db::ProductFingerprint;
use crate::declaration_type::DeclaredTypeState;
use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic, SemanticSourceSpan};
use crate::identity::{CallableId, CallableOwnerId, DeclarationId, DispatchSide, ImplId};
use crate::signature::CallableSemanticSignature;
use crate::surface::MemberVisibility;
use crate::traits::{InstantiatedTraitRequirement, TraitHeaderTable, TraitRef, TraitRefFormationError, TraitSurface};
use crate::types::annotation::{
    ScopedTypeResolver, TypeFormationSite, TypeLevelBinding, TypeResolver, resolve_type_annotation, type_level_binding_for_parameter,
};
use crate::types::environment::TypeEnvironment;
use crate::types::evidence::{TypeKnowledge, UnknownReason};
use crate::types::id::{KindId, TypeId, TypeParameterId};
use crate::types::outcome::{BlockReason, BudgetReport, CancellationToken, DynamicBoundaryObligation, QueryBudget, RelationOutcome};
use crate::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner, TypeTerm};
use crate::types::store::{TypeData, TypeStore};
use crate::types::substitution::TypeSubstitution;
use phalcom_ast::ast::{BehaviorMember, ImplDef, TypeAnnotationExpr};
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hash::{Hash, Hasher};

/// Computes the conditional inherent members selected for one canonical
/// receiver form. This is the shared semantic query used by non-checker
/// consumers; callers may enumerate or render the result, but must not run a
/// second applicability solver.
pub fn receiver_effective_conditional_members(
    store: &mut TypeStore,
    hierarchy: &dyn crate::types::relation::TypeHierarchy,
    dispatch: &crate::dispatch::SurfaceDispatchResolver,
    receiver_type: TypeId,
    lookup_owner: &DeclarationId,
    side: DispatchSide,
    ambient_constraints: &[crate::types::parameter::GenericConstraint],
) -> Vec<(DeclarationId, ConditionalInherentMember)> {
    receiver_effective_conditional_members_with_outcomes(store, hierarchy, dispatch, receiver_type, lookup_owner, side, ambient_constraints)
        .into_iter()
        .filter_map(|(owner, member, applicability)| matches!(applicability, ImplApplicabilityResult::Applicable(_)).then_some((owner, member)))
        .collect()
}

/// Conditional members together with the proof state that made them visible
/// to the receiver query. `NotApplicable` candidates are omitted; terminal
/// outcomes remain available to proof-producing callers instead of becoming
/// indistinguishable from a missing member.
fn receiver_effective_conditional_members_with_outcomes(
    store: &mut TypeStore,
    hierarchy: &dyn crate::types::relation::TypeHierarchy,
    dispatch: &crate::dispatch::SurfaceDispatchResolver,
    receiver_type: TypeId,
    lookup_owner: &DeclarationId,
    side: DispatchSide,
    ambient_constraints: &[crate::types::parameter::GenericConstraint],
) -> Vec<(DeclarationId, ConditionalInherentMember, ImplApplicabilityResult)> {
    let mut selected = BTreeSet::new();
    let mut result = Vec::new();
    let ordinary_selectors = dispatch
        .dispatch_owners(hierarchy, lookup_owner, side)
        .into_iter()
        .filter_map(|owner| dispatch.surface(&owner.declaration).map(|surface| surface.surface(owner.side)))
        .flat_map(|surface| surface.callable_signatures.keys().cloned())
        .collect::<BTreeSet<_>>();

    let exact_variant = match store.get(receiver_type) {
        TypeData::ExactCase { variant, .. } => Some(store.variant_identity(*variant).clone()),
        _ => None,
    };
    if let Some(variant) = exact_variant
        && let Some(set) = dispatch.get_conditional_members(&InherentImplTarget::ExactEnumCase(variant.clone()))
    {
        for member in set.members.iter().filter(|member| member.callable.side == side) {
            let selector = member.callable.selector.clone();
            if selected.contains(&selector) {
                continue;
            }
            let applicability = check_impl_domain_applicability(store, hierarchy, &member.domain, receiver_type, receiver_type, ambient_constraints);
            if !matches!(applicability, ImplApplicabilityResult::NotApplicable) {
                if matches!(applicability, ImplApplicabilityResult::Applicable(_)) {
                    selected.insert(selector);
                }
                result.push((variant.owner.clone(), member.clone(), applicability));
            }
        }
    }

    let control = crate::checker::context::CheckerControl::default();
    for owner in dispatch.dispatch_owners(hierarchy, lookup_owner, side) {
        let Ok(receiver_spec) = crate::types::specialization::specialize_receiver_to_owner(store, hierarchy, receiver_type, &owner.declaration, &control)
        else {
            continue;
        };
        let owner_view = receiver_spec.path.last().map(|step| step.specialized_form).unwrap_or(receiver_type);
        let Some(set) = dispatch.get_conditional_members(&InherentImplTarget::Declaration(owner.declaration.clone())) else {
            continue;
        };
        for member in set.members.iter().filter(|member| member.callable.side == owner.side) {
            let selector = member.callable.selector.clone();
            if ordinary_selectors.contains(&selector) || selected.contains(&selector) {
                continue;
            }
            let applicability = check_impl_domain_applicability(store, hierarchy, &member.domain, receiver_type, owner_view, ambient_constraints);
            if !matches!(applicability, ImplApplicabilityResult::NotApplicable) {
                if matches!(applicability, ImplApplicabilityResult::Applicable(_)) {
                    selected.insert(selector);
                }
                result.push((owner.declaration.clone(), member.clone(), applicability));
            }
        }
    }

    result
}

/// One canonical inherent witness candidate for a source conformance plan.
/// The candidate may come from a direct/inherited dispatch surface or from a
/// proven C2 conditional member; explicit conformance members and trait
/// defaults are intentionally outside this query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveInherentWitness {
    pub callable: CallableId,
    pub signature: CallableSemanticSignature,
    pub visibility: MemberVisibility,
    pub owner: DeclarationId,
    pub conditional_impl: Option<ImplId>,
    pub applicability: Option<InherentImplSpecialization>,
    pub conditional_domain: Option<Arc<InherentImplDomain>>,
}

/// Proof-aware result of resolving one inherent witness candidate. A terminal
/// applicability result is distinct from `NotFound`, because conformance
/// completeness must not turn an unresolved C2 query into a false absence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectiveInherentWitnessResolution {
    Candidate(EffectiveInherentWitness),
    Deferred {
        candidate: EffectiveInherentWitness,
        state: ConformanceCompleteness,
    },
    NotFound,
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}

/// Resolves one effective inherent member without scanning source syntax.
/// Ordinary dispatch wins over conditional members, and the conditional query
/// supplies exact-case, inherited, and constrained C2 applicability.
pub fn resolve_effective_inherent_witness(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    dispatch: &crate::dispatch::SurfaceDispatchResolver,
    callable_signatures: &crate::signature::CallableSignatureTable,
    receiver_type: TypeId,
    lookup_owner: &DeclarationId,
    selector: &phalcom_common::selector::Selector,
    side: DispatchSide,
    ambient_constraints: &[GenericConstraint],
) -> EffectiveInherentWitnessResolution {
    if let Some(callable) = dispatch.resolve_callable_id(hierarchy, lookup_owner, side, selector)
        && let Some(signature) = callable_signatures.get(&callable)
        && let Some(owner) = callable.try_declaration_owner().cloned()
    {
        let visibility = dispatch
            .surface(&owner)
            .and_then(|surface| surface.surface(side).callable_visibility.get(selector))
            .copied()
            .unwrap_or_default();
        return EffectiveInherentWitnessResolution::Candidate(EffectiveInherentWitness {
            callable,
            signature: signature.clone(),
            visibility,
            owner,
            conditional_impl: None,
            applicability: None,
            conditional_domain: None,
        });
    }

    let mut deferred = None;
    for (owner, member, applicability) in
        receiver_effective_conditional_members_with_outcomes(store, hierarchy, dispatch, receiver_type, lookup_owner, side, ambient_constraints).into_iter()
    {
        if member.callable.selector != *selector || member.callable.side != side {
            continue;
        }
        match applicability {
            ImplApplicabilityResult::Applicable(applicability) => {
                let signature = callable_signatures
                    .get(&member.callable)
                    .cloned()
                    .unwrap_or_else(|| member.signature_template.clone());
                return EffectiveInherentWitnessResolution::Candidate(EffectiveInherentWitness {
                    callable: member.callable,
                    signature,
                    visibility: member.visibility,
                    owner,
                    conditional_impl: Some(member.impl_id),
                    applicability: Some(applicability),
                    conditional_domain: Some(member.domain.clone()),
                });
            }
            ImplApplicabilityResult::NotApplicable => {}
            outcome @ (ImplApplicabilityResult::Unknown(_)
            | ImplApplicabilityResult::Blocked(_)
            | ImplApplicabilityResult::Dynamic(_)
            | ImplApplicabilityResult::Cancelled
            | ImplApplicabilityResult::BudgetExceeded(_)
            | ImplApplicabilityResult::InternalFailure(_)) => {
                let signature = callable_signatures
                    .get(&member.callable)
                    .cloned()
                    .unwrap_or_else(|| member.signature_template.clone());
                let candidate = EffectiveInherentWitness {
                    callable: member.callable,
                    signature,
                    visibility: member.visibility,
                    owner,
                    conditional_impl: Some(member.impl_id),
                    applicability: None,
                    conditional_domain: Some(member.domain),
                };
                let state = applicability_to_completeness(outcome);
                if let Some((_, current)) = deferred.as_ref() {
                    if completeness_terminal_rank(&state) > completeness_terminal_rank(current) {
                        deferred = Some((candidate, state));
                    }
                } else {
                    deferred = Some((candidate, state));
                }
            }
        }
    }
    match deferred {
        Some((candidate, state)) => EffectiveInherentWitnessResolution::Deferred { candidate, state },
        None => EffectiveInherentWitnessResolution::NotFound,
    }
}

fn applicability_to_completeness(outcome: ImplApplicabilityResult) -> ConformanceCompleteness {
    match outcome {
        ImplApplicabilityResult::Unknown(reason) => ConformanceCompleteness::Unknown(reason),
        ImplApplicabilityResult::Blocked(reason) => ConformanceCompleteness::Blocked(reason),
        ImplApplicabilityResult::Dynamic(obligation) => ConformanceCompleteness::Dynamic(obligation),
        ImplApplicabilityResult::Cancelled => ConformanceCompleteness::Cancelled,
        ImplApplicabilityResult::BudgetExceeded(report) => ConformanceCompleteness::BudgetExceeded(report),
        ImplApplicabilityResult::InternalFailure(message) => ConformanceCompleteness::InternalFailure(message.into_boxed_str()),
        ImplApplicabilityResult::Applicable(_) | ImplApplicabilityResult::NotApplicable => {
            ConformanceCompleteness::InternalFailure("non-terminal applicability was converted to terminal state".into())
        }
    }
}

/// Specializes a declaration-owned inherent witness through the exact
/// conformance target before comparing it with an instantiated requirement.
/// The callable identity remains declaration-owned; only its type view is
/// materialized in the conformance environment.
pub(crate) fn specialize_witness_signature(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    receiver: TypeId,
    owner: &DeclarationId,
    mut signature: CallableSemanticSignature,
) -> Result<CallableSemanticSignature, crate::types::specialization::ReceiverSpecializationFailure> {
    let specialization = crate::types::specialization::specialize_receiver_to_owner(store, hierarchy, receiver, owner, &CheckerControl::default())?;
    let environment = specialization.environment;
    let materialize_fact = |store: &mut TypeStore, fact: &mut crate::declaration_type::DeclaredTypeFact| {
        if let DeclaredTypeState::Known(TypeTerm::Canonical(ty)) = fact.state {
            fact.state = DeclaredTypeState::Known(TypeTerm::Canonical(
                crate::types::environment::TypeView::new(ty, environment.clone()).materialize(store),
            ));
        }
    };
    for parameter in &mut signature.parameters {
        materialize_fact(store, &mut parameter.declared_type);
    }
    materialize_fact(store, &mut signature.declared_return);
    if let Some(generics) = &mut signature.generics {
        for constraint in &mut generics.constraints {
            let materialize_term = |store: &mut TypeStore, term: &mut TypeTerm| {
                if let TypeTerm::Canonical(ty) = term {
                    *ty = crate::types::environment::TypeView::new(*ty, environment.clone()).materialize(store);
                }
            };
            match constraint {
                crate::types::parameter::GenericConstraint::Subtype { lower, upper }
                | crate::types::parameter::GenericConstraint::Equivalent { left: lower, right: upper } => {
                    materialize_term(store, lower);
                    materialize_term(store, upper);
                }
            }
        }
    }
    Ok(signature)
}

/// Explicit bijection mapping between impl type parameters and canonical target declaration parameters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoveringImplSubstitution {
    /// Mapping: impl-owned TypeParameterId -> declaration-owned TypeParameterId
    pub impl_to_decl: HashMap<TypeParameterId, TypeParameterId>,
    /// Inverse: declaration-owned TypeParameterId -> impl-owned TypeParameterId
    pub decl_to_impl: HashMap<TypeParameterId, TypeParameterId>,
}

impl CoveringImplSubstitution {
    pub fn new(impl_to_decl: HashMap<TypeParameterId, TypeParameterId>, decl_to_impl: HashMap<TypeParameterId, TypeParameterId>) -> Self {
        Self { impl_to_decl, decl_to_impl }
    }

    /// Converts the `impl_to_decl` map into a `TypeSubstitution` replacing impl parameter types with declaration parameter types.
    pub fn to_type_substitution(&self, store: &mut TypeStore) -> TypeSubstitution {
        let mut subst = TypeSubstitution::new();
        for (&impl_param, &decl_param) in &self.impl_to_decl {
            let decl_ty = store.parameter_form(decl_param);
            subst.bind(impl_param, decl_ty);
        }
        subst
    }
}

use crate::types::parameter::GenericConstraint;
use crate::types::relation::{TypeHierarchy, check_subtype_bounded};

/// First-class domain representing the specialized or constrained applicability scope of an inherent `impl` block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InherentImplDomain {
    pub impl_id: ImplId,
    pub target: InherentImplTarget,
    pub head_type: TypeId,
    pub generic_signature: Option<GenericSignature>,
    pub constraints: Box<[GenericConstraint]>,
}

/// Applicability regime of an inherent impl block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InherentImplApplicability {
    /// Unconditional non-generic target (e.g. `impl User`).
    Unconditional,
    /// Covering generic target (e.g. `impl<A, B> Pair<A, B>` or `impl<A, B> Pair<B, A>`).
    Covering(CoveringImplSubstitution),
    /// Receiver-specialized or constraint-conditioned target (e.g. `impl Point<Int>`, `impl<T> Pair<T, T>`, `impl<T> Point<T> where T <: Number`).
    Conditional(Arc<InherentImplDomain>),
}

/// Target of an inherent implementation block (nominal declaration or exact enum case).
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum InherentImplTarget {
    Declaration(DeclarationId),
    ExactEnumCase(VariantId),
}

impl InherentImplTarget {
    pub fn declaration(&self) -> &DeclarationId {
        match self {
            InherentImplTarget::Declaration(id) => id,
            InherentImplTarget::ExactEnumCase(variant_id) => &variant_id.owner,
        }
    }

    pub fn to_callable_owner(&self) -> CallableOwnerId {
        match self {
            InherentImplTarget::Declaration(id) => CallableOwnerId::Declaration(id.clone()),
            InherentImplTarget::ExactEnumCase(variant_id) => CallableOwnerId::Variant(variant_id.clone()),
        }
    }
}

impl From<InherentImplTarget> for CallableOwnerId {
    fn from(target: InherentImplTarget) -> Self {
        match target {
            InherentImplTarget::Declaration(id) => CallableOwnerId::Declaration(id),
            InherentImplTarget::ExactEnumCase(variant_id) => CallableOwnerId::Variant(variant_id),
        }
    }
}

/// Result of resolving the target of an inherent impl block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedInherentImplTarget {
    pub id: ImplId,
    pub target: InherentImplTarget,
    pub target_type: TypeId,
    pub generic_signature: Option<GenericSignature>,
    pub applicability: InherentImplApplicability,
    pub source: SemanticSourceSpan,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}

/// Canonical target identity shared by explicit conformance products.
///
/// This is deliberately separate from the inherent-implementation target
/// product. A conformance target identifies the declaration or exact enum case
/// whose head participates in ownership, lookup, and coherence; it does not
/// contribute ordinary behavior to that target.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ConformanceTarget {
    Declaration(DeclarationId),
    ExactEnumCase(crate::identity::VariantId),
}

impl ConformanceTarget {
    pub fn declaration(&self) -> &DeclarationId {
        match self {
            Self::Declaration(declaration) => declaration,
            Self::ExactEnumCase(variant) => &variant.owner,
        }
    }
}

/// Returns whether the source module is one of the two canonical owners that
/// may declare an explicit conformance.
pub fn conformance_is_authorized(source_module: &crate::identity::ModuleId, trait_ref: &TraitRef, target: &ConformanceTarget) -> bool {
    source_module == &trait_ref.declaration.module || source_module == &target.declaration().module
}

/// Resolved source conformance header produced before workspace publication.
///
/// The product contains only declaration provenance and exact head identity.
/// It intentionally contains no witness, default-selection, or completeness
/// claim; those belong to C4.P2.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedConformanceHead {
    pub impl_id: ImplId,
    pub source_module: crate::identity::ModuleId,
    pub trait_ref: TraitRef,
    pub target: ConformanceTarget,
    pub target_head: TypeId,
    pub generic_signature: Option<GenericSignature>,
    pub source: SemanticSourceSpan,
    pub eligible: bool,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}

/// Static source contribution for one explicit conformance declaration.
///
/// This product is deliberately body-independent. `authorized` controls
/// eligibility for lookup; retaining an unauthorized contribution allows the
/// diagnostic and source layers to explain why it was excluded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConformanceContribution {
    pub impl_id: ImplId,
    pub source_module: crate::identity::ModuleId,
    pub trait_ref: TraitRef,
    pub target: ConformanceTarget,
    pub target_head: TypeId,
    pub generic_signature: Option<GenericSignature>,
    pub authorized: bool,
    pub eligible: bool,
    pub source: SemanticSourceSpan,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}

impl ConformanceContribution {
    pub fn from_resolved(head: ResolvedConformanceHead, authorized: bool) -> Self {
        Self {
            impl_id: head.impl_id,
            source_module: head.source_module,
            trait_ref: head.trait_ref,
            target: head.target,
            target_head: head.target_head,
            generic_signature: head.generic_signature,
            authorized,
            eligible: head.eligible,
            source: head.source,
            diagnostics: head.diagnostics,
        }
    }

    pub fn is_lookup_eligible(&self) -> bool {
        self.authorized && self.eligible
    }
}

/// Exact application of one source conformance head to a target/TraitRef pair.
///
/// A head match says only that the declaration domain applies. It is not
/// conformance evidence and does not claim that any trait requirement is
/// satisfied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConformanceHeadMatch {
    pub impl_id: ImplId,
    pub exact_target: TypeId,
    pub exact_trait_ref: TraitRef,
    pub impl_bindings: HashMap<TypeParameterId, TypeId>,
}

/// Source-level selection of one trait requirement. These templates retain
/// source identities and are specialized only when an exact head is queried.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequirementSelectionTemplate {
    ConformanceCallable {
        callable: CallableId,
    },
    InherentCallable {
        callable: CallableId,
        conditional_impl: Option<ImplId>,
        applicability: Option<InherentImplSpecialization>,
    },
    /// A conditional inherent candidate whose applicability is unresolved in
    /// the generic source plan. The exact evidence query decides the branch
    /// from the retained C2 domain and may use the independently valid
    /// fallback only when that domain is proven inapplicable.
    ConditionalInherent {
        candidate: EffectiveInherentWitness,
        pending: ConformanceCompleteness,
        fallback: Option<Box<RequirementSelectionTemplate>>,
    },
    DataComponent {
        component: crate::identity::DataComponentId,
        specialized_type: TypeId,
    },
    TraitDefault {
        callable: CallableId,
    },
}

/// Proof returned when a candidate satisfies one instantiated requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WitnessCompatibilityProof {
    pub requirement: crate::traits::TraitRequirementId,
    pub candidate: CallableId,
}

/// Refutation returned when a candidate cannot satisfy one requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WitnessMismatch {
    pub reason: Box<str>,
}

/// Bounded semantic result for witness compatibility. Terminal analysis
/// states remain visible to conformance completeness instead of being
/// silently treated as a source mismatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WitnessCompatibility {
    Compatible(WitnessCompatibilityProof),
    Incompatible(WitnessMismatch),
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}

pub(crate) fn witness_compatibility_terminal_state(result: &WitnessCompatibility) -> Option<ConformanceCompleteness> {
    match result {
        WitnessCompatibility::Unknown(reason) => Some(ConformanceCompleteness::Unknown(reason.clone())),
        WitnessCompatibility::Blocked(reason) => Some(ConformanceCompleteness::Blocked(reason.clone())),
        WitnessCompatibility::Dynamic(obligation) => Some(ConformanceCompleteness::Dynamic(obligation.clone())),
        WitnessCompatibility::Cancelled => Some(ConformanceCompleteness::Cancelled),
        WitnessCompatibility::BudgetExceeded(report) => Some(ConformanceCompleteness::BudgetExceeded(report.clone())),
        WitnessCompatibility::InternalFailure(message) => Some(ConformanceCompleteness::InternalFailure(message.clone())),
        WitnessCompatibility::Compatible(_) | WitnessCompatibility::Incompatible(_) => None,
    }
}

fn witness_fact_type(fact: &crate::declaration_type::DeclaredTypeFact) -> Result<TypeId, WitnessCompatibility> {
    match &fact.state {
        DeclaredTypeState::Known(TypeTerm::Canonical(ty)) => Ok(*ty),
        DeclaredTypeState::Known(TypeTerm::SelfType(_) | TypeTerm::Infer(_)) => Err(WitnessCompatibility::Blocked(BlockReason::RecursiveFixpoint)),
        DeclaredTypeState::Unknown(reason) => Err(WitnessCompatibility::Unknown(reason.clone())),
        DeclaredTypeState::Dynamic(reason) => Err(WitnessCompatibility::Dynamic(DynamicBoundaryObligation {
            reason: format!("dynamic declaration fact: {reason:?}"),
        })),
    }
}

fn witness_relation_result(outcome: RelationOutcome) -> Result<(), WitnessCompatibility> {
    match outcome {
        RelationOutcome::Proven { .. } => Ok(()),
        RelationOutcome::Refuted(failure) => Err(WitnessCompatibility::Incompatible(WitnessMismatch {
            reason: format!("type relation refuted: {failure:?}").into_boxed_str(),
        })),
        RelationOutcome::DynamicBoundary(obligation) => Err(WitnessCompatibility::Dynamic(obligation)),
        RelationOutcome::Blocked(reason) => Err(WitnessCompatibility::Blocked(reason)),
        RelationOutcome::Cancelled => Err(WitnessCompatibility::Cancelled),
        RelationOutcome::BudgetExceeded(report) => Err(WitnessCompatibility::BudgetExceeded(report)),
        RelationOutcome::InternalFailure(message) => Err(WitnessCompatibility::InternalFailure(message.into_boxed_str())),
    }
}

fn witness_visibility_covers(required: MemberVisibility, candidate: MemberVisibility) -> bool {
    match (required, candidate) {
        (MemberVisibility::Public, MemberVisibility::Public)
        | (MemberVisibility::Protected, MemberVisibility::Public | MemberVisibility::Protected)
        | (MemberVisibility::Private, MemberVisibility::Public | MemberVisibility::Protected | MemberVisibility::Private)
        | (MemberVisibility::Internal, MemberVisibility::Internal) => true,
        _ => false,
    }
}

fn witness_generic_term(store: &mut TypeStore, substitution: &TypeSubstitution, term: &TypeTerm) -> Option<TypeTerm> {
    match term {
        TypeTerm::Canonical(ty) => Some(TypeTerm::Canonical(substitution.apply(store, *ty))),
        TypeTerm::SelfType(term) => Some(TypeTerm::SelfType(term.clone())),
        TypeTerm::Infer(_) => None,
    }
}

fn generic_constraint_exact(left: &crate::types::parameter::GenericConstraint, right: &crate::types::parameter::GenericConstraint) -> bool {
    match (left, right) {
        (
            crate::types::parameter::GenericConstraint::Subtype {
                lower: left_lower,
                upper: left_upper,
            },
            crate::types::parameter::GenericConstraint::Subtype {
                lower: right_lower,
                upper: right_upper,
            },
        ) => left_lower == right_lower && left_upper == right_upper,
        (
            crate::types::parameter::GenericConstraint::Equivalent {
                left: left_left,
                right: left_right,
            },
            crate::types::parameter::GenericConstraint::Equivalent {
                left: right_left,
                right: right_right,
            },
        ) => (left_left == right_left && left_right == right_right) || (left_left == right_right && left_right == right_left),
        _ => false,
    }
}

fn constraint_subtype_relation(store: &mut TypeStore, hierarchy: &dyn TypeHierarchy, lower: TypeId, upper: TypeId) -> Result<bool, WitnessCompatibility> {
    match check_subtype_bounded(store, hierarchy, lower, upper, &mut QueryBudget::default(), &CancellationToken::default()) {
        RelationOutcome::Proven { .. } => Ok(true),
        RelationOutcome::Refuted(_) => Ok(false),
        outcome => Err(witness_relation_result(outcome).expect_err("non-terminal relation outcome must be proven or refuted")),
    }
}

fn generic_constraint_implied(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    required_constraints: &[crate::types::parameter::GenericConstraint],
    candidate_constraint: &crate::types::parameter::GenericConstraint,
) -> Result<bool, WitnessCompatibility> {
    if required_constraints
        .iter()
        .any(|required| generic_constraint_exact(required, candidate_constraint))
    {
        return Ok(true);
    }
    let crate::types::parameter::GenericConstraint::Subtype {
        lower: candidate_lower,
        upper: candidate_upper,
    } = candidate_constraint
    else {
        return Ok(false);
    };
    let (TypeTerm::Canonical(candidate_lower), TypeTerm::Canonical(candidate_upper)) = (candidate_lower, candidate_upper) else {
        return Ok(false);
    };
    for required in required_constraints {
        match required {
            crate::types::parameter::GenericConstraint::Subtype {
                lower: required_lower,
                upper: required_upper,
            } => {
                let (TypeTerm::Canonical(required_lower), TypeTerm::Canonical(required_upper)) = (required_lower, required_upper) else {
                    continue;
                };
                if required_lower == candidate_lower && constraint_subtype_relation(store, hierarchy, *required_upper, *candidate_upper)? {
                    return Ok(true);
                }
            }
            crate::types::parameter::GenericConstraint::Equivalent {
                left: required_left,
                right: required_right,
            } => {
                let (TypeTerm::Canonical(required_left), TypeTerm::Canonical(required_right)) = (required_left, required_right) else {
                    continue;
                };
                let implied_upper = if required_left == candidate_lower {
                    Some(*required_right)
                } else if required_right == candidate_lower {
                    Some(*required_left)
                } else {
                    None
                };
                if let Some(implied_upper) = implied_upper
                    && constraint_subtype_relation(store, hierarchy, implied_upper, *candidate_upper)?
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn witness_generic_contract(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    required: &Option<GenericSignature>,
    candidate: &Option<GenericSignature>,
) -> Result<(), WitnessCompatibility> {
    match (required, candidate) {
        (None, None) => Ok(()),
        (Some(_), None) | (None, Some(_)) => Err(WitnessCompatibility::Incompatible(WitnessMismatch {
            reason: "witness generic arity does not match the requirement".into(),
        })),
        (Some(required), Some(candidate)) => {
            if required.parameter_count() != candidate.parameter_count() {
                return Err(WitnessCompatibility::Incompatible(WitnessMismatch {
                    reason: "witness generic kind contract does not match the requirement".into(),
                }));
            }
            for (&required_parameter, &candidate_parameter) in required.parameters.iter().zip(candidate.parameters.iter()) {
                let required_kind = required
                    .parameter_kinds
                    .iter()
                    .nth(required_parameter.index() as usize)
                    .copied()
                    .unwrap_or_else(|| store.type_parameter(required_parameter).kind);
                let candidate_kind = candidate
                    .parameter_kinds
                    .iter()
                    .nth(candidate_parameter.index() as usize)
                    .copied()
                    .unwrap_or_else(|| store.type_parameter(candidate_parameter).kind);
                if required_kind != candidate_kind {
                    return Err(WitnessCompatibility::Incompatible(WitnessMismatch {
                        reason: "witness generic kind contract does not match the requirement".into(),
                    }));
                }
            }
            if candidate.constraints.is_empty() {
                return Ok(());
            }
            if required.constraints.is_empty() || candidate.constraints.len() > required.constraints.len() {
                return Err(WitnessCompatibility::Incompatible(WitnessMismatch {
                    reason: "witness generic contract strengthens caller constraints".into(),
                }));
            }
            let mut alpha = TypeSubstitution::new();
            for (&required_parameter, &candidate_parameter) in required.parameters.iter().zip(candidate.parameters.iter()) {
                alpha.bind(candidate_parameter, store.parameter_form(required_parameter));
            }
            let candidate_constraints = candidate
                .constraints
                .iter()
                .map(|constraint| match constraint {
                    crate::types::parameter::GenericConstraint::Subtype { lower, upper } => Ok(crate::types::parameter::GenericConstraint::Subtype {
                        lower: witness_generic_term(store, &alpha, lower).ok_or(WitnessCompatibility::Blocked(BlockReason::RecursiveFixpoint))?,
                        upper: witness_generic_term(store, &alpha, upper).ok_or(WitnessCompatibility::Blocked(BlockReason::RecursiveFixpoint))?,
                    }),
                    crate::types::parameter::GenericConstraint::Equivalent { left, right } => Ok(crate::types::parameter::GenericConstraint::Equivalent {
                        left: witness_generic_term(store, &alpha, left).ok_or(WitnessCompatibility::Blocked(BlockReason::RecursiveFixpoint))?,
                        right: witness_generic_term(store, &alpha, right).ok_or(WitnessCompatibility::Blocked(BlockReason::RecursiveFixpoint))?,
                    }),
                })
                .collect::<Result<Vec<_>, WitnessCompatibility>>()?;
            for candidate_constraint in &candidate_constraints {
                if !generic_constraint_implied(store, hierarchy, &required.constraints, candidate_constraint)? {
                    return Err(WitnessCompatibility::Incompatible(WitnessMismatch {
                        reason: "witness generic contract strengthens caller constraints".into(),
                    }));
                }
            }
            Ok(())
        }
    }
}

fn callable_generic_alpha_substitution(store: &mut TypeStore, required: &Option<GenericSignature>, candidate: &Option<GenericSignature>) -> TypeSubstitution {
    let mut substitution = TypeSubstitution::new();
    if let (Some(required), Some(candidate)) = (required, candidate) {
        for (&required_parameter, &candidate_parameter) in required.parameters.iter().zip(candidate.parameters.iter()) {
            substitution.bind(candidate_parameter, store.parameter_form(required_parameter));
        }
    }
    substitution
}

/// Checks the callable contract using the canonical bounded type relation.
/// The caller remains responsible for selecting the source kind; this helper
/// only answers whether one already-selected candidate is safe.
pub fn check_witness_compatibility(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    required: &crate::traits::InstantiatedTraitRequirement,
    candidate: &CallableSemanticSignature,
) -> WitnessCompatibility {
    check_witness_compatibility_with_visibility(store, hierarchy, required, candidate, MemberVisibility::Public)
}

/// Visibility-aware witness compatibility entry point used by source-plan
/// candidate selection.
pub fn check_witness_compatibility_with_visibility(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    required: &crate::traits::InstantiatedTraitRequirement,
    candidate: &CallableSemanticSignature,
    candidate_visibility: MemberVisibility,
) -> WitnessCompatibility {
    if required.signature.selector != candidate.selector
        || required.signature.side != candidate.side
        || required.signature.parameters.len() != candidate.parameters.len()
    {
        return WitnessCompatibility::Incompatible(WitnessMismatch {
            reason: "selector, dispatch side, or parameter count differs".into(),
        });
    }
    if !witness_visibility_covers(required.visibility, candidate_visibility) {
        return WitnessCompatibility::Incompatible(WitnessMismatch {
            reason: "witness visibility does not cover the trait requirement".into(),
        });
    }
    if let Err(result) = witness_generic_contract(store, hierarchy, &required.signature.generics, &candidate.generics) {
        return result;
    }
    let alpha = callable_generic_alpha_substitution(store, &required.signature.generics, &candidate.generics);
    let mut budget = QueryBudget::default();
    let cancel = CancellationToken::default();
    for (required_parameter, candidate_parameter) in required.signature.parameters.iter().zip(candidate.parameters.iter()) {
        if required_parameter.rest != candidate_parameter.rest {
            return WitnessCompatibility::Incompatible(WitnessMismatch {
                reason: "witness parameter rest role differs".into(),
            });
        }
        let required_type = match witness_fact_type(&required_parameter.declared_type) {
            Ok(ty) => ty,
            Err(result) => return result,
        };
        let candidate_type = match witness_fact_type(&candidate_parameter.declared_type) {
            Ok(ty) => ty,
            Err(result) => return result,
        };
        let candidate_type = alpha.apply(store, candidate_type);
        if let Err(result) = witness_relation_result(check_subtype_bounded(store, hierarchy, required_type, candidate_type, &mut budget, &cancel)) {
            return result;
        }
    }
    let candidate_return = match witness_fact_type(&candidate.declared_return) {
        Ok(ty) => ty,
        Err(result) => return result,
    };
    let candidate_return = alpha.apply(store, candidate_return);
    let required_return = match witness_fact_type(&required.signature.declared_return) {
        Ok(ty) => ty,
        Err(result) => return result,
    };
    if let Err(result) = witness_relation_result(check_subtype_bounded(store, hierarchy, candidate_return, required_return, &mut budget, &cancel)) {
        return result;
    }
    WitnessCompatibility::Compatible(WitnessCompatibilityProof {
        requirement: required.requirement.clone(),
        candidate: candidate.callable.clone(),
    })
}

/// Checks a data component as a getter witness without manufacturing a
/// callable identity or publishing a synthetic getter signature.
pub fn check_data_component_compatibility(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    required: &crate::traits::InstantiatedTraitRequirement,
    component_type: TypeId,
) -> WitnessCompatibility {
    if !matches!(required.signature.selector.kind, phalcom_common::selector::SelectorKind::Getter) || !required.signature.parameters.is_empty() {
        return WitnessCompatibility::Incompatible(WitnessMismatch {
            reason: "data components can satisfy getter requirements only".into(),
        });
    }
    let required_return = match witness_fact_type(&required.signature.declared_return) {
        Ok(ty) => ty,
        Err(result) => return result,
    };
    match witness_relation_result(check_subtype_bounded(
        store,
        hierarchy,
        component_type,
        required_return,
        &mut QueryBudget::default(),
        &CancellationToken::default(),
    )) {
        Ok(()) => WitnessCompatibility::Compatible(WitnessCompatibilityProof {
            requirement: required.requirement.clone(),
            candidate: required.callable.clone(),
        }),
        Err(result) => result,
    }
}

/// Compatibility predicate retained for narrow callers that only need the
/// positive branch. New proof-producing consumers should use
/// [`check_witness_compatibility`].
pub fn witness_callable_is_compatible(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    required: &CallableSemanticSignature,
    candidate: &CallableSemanticSignature,
) -> bool {
    let requirement = crate::traits::InstantiatedTraitRequirement {
        requirement: crate::traits::TraitRequirementId::new(required.owner.clone(), required.selector.clone(), required.side),
        callable: required.callable.clone(),
        signature: required.clone(),
        visibility: MemberVisibility::Public,
        default_callable: None,
    };
    matches!(
        check_witness_compatibility(store, hierarchy, &requirement, candidate),
        WitnessCompatibility::Compatible(_)
    )
}

/// A requirement which prevented a source conformance from becoming complete.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequirementFailure {
    pub requirement: crate::traits::TraitRequirementId,
    pub reason: Box<str>,
}

/// Proof state of a source conformance witness plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConformanceCompleteness {
    Complete,
    Incomplete { failures: Box<[RequirementFailure]> },
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}

impl ConformanceCompleteness {
    /// Maps source-plan proof state to exact-resolution state without
    /// collapsing analysis uncertainty into an ordinary incomplete proof.
    fn into_resolution(self, impl_id: ImplId) -> Option<ConformanceResolution> {
        match self {
            Self::Complete => None,
            Self::Incomplete { .. } => Some(ConformanceResolution::Incomplete(impl_id)),
            Self::Unknown(reason) => Some(ConformanceResolution::Unknown(reason)),
            Self::Blocked(reason) => Some(ConformanceResolution::Blocked(reason)),
            Self::Dynamic(obligation) => Some(ConformanceResolution::Dynamic(obligation)),
            Self::Cancelled => Some(ConformanceResolution::Cancelled),
            Self::BudgetExceeded(report) => Some(ConformanceResolution::BudgetExceeded(report)),
            Self::InternalFailure(message) => Some(ConformanceResolution::InternalFailure(message)),
        }
    }
}

fn completeness_terminal_rank(state: &ConformanceCompleteness) -> u8 {
    match state {
        // Match the repository's query propagation order: an internal
        // failure is strongest, followed by budget/cancellation/blocking;
        // semantic dynamic and unknown boundaries are least decisive.
        ConformanceCompleteness::InternalFailure(_) => 6,
        ConformanceCompleteness::BudgetExceeded(_) => 5,
        ConformanceCompleteness::Cancelled => 4,
        ConformanceCompleteness::Blocked(_) => 3,
        ConformanceCompleteness::Dynamic(_) => 2,
        ConformanceCompleteness::Unknown(_) => 1,
        ConformanceCompleteness::Complete | ConformanceCompleteness::Incomplete { .. } => 0,
    }
}

pub(crate) fn retain_stronger_terminal_state(slot: &mut Option<ConformanceCompleteness>, candidate: ConformanceCompleteness) {
    if slot
        .as_ref()
        .is_none_or(|current| completeness_terminal_rank(&candidate) > completeness_terminal_rank(current))
    {
        *slot = Some(candidate);
    }
}

pub(crate) fn replace_with_stronger_terminal_state(slot: &mut ConformanceCompleteness, candidate: ConformanceCompleteness) {
    if completeness_terminal_rank(&candidate) > completeness_terminal_rank(slot) {
        *slot = candidate;
    }
}

/// One source conformance's generic witness/default selection proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConformanceWitnessPlan {
    pub impl_id: ImplId,
    pub source_module: crate::identity::ModuleId,
    pub target_template: TypeId,
    pub trait_ref_template: TraitRef,
    pub generic_signature: Option<GenericSignature>,
    /// Source-level trait requirements after binding the trait arguments and
    /// symbolic `Self` to the conformance head template.
    pub requirement_views: BTreeMap<crate::traits::TraitRequirementId, InstantiatedTraitRequirement>,
    pub requirements: BTreeMap<crate::traits::TraitRequirementId, RequirementSelectionTemplate>,
    pub fingerprint: ProductFingerprint,
    pub invalid_explicit_members: bool,
    pub completeness: ConformanceCompleteness,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}

/// Exact target/TraitRef proof instantiated from a source witness plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConformanceEvidence {
    pub source_impl: ImplId,
    pub exact_target: TypeId,
    pub exact_trait_ref: TraitRef,
    pub impl_environment: TypeEnvironment,
    pub requirement_views: BTreeMap<crate::traits::TraitRequirementId, InstantiatedTraitRequirement>,
    pub requirements: BTreeMap<crate::traits::TraitRequirementId, RequirementSelectionTemplate>,
    pub fingerprint: ProductFingerprint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConformanceResolution {
    Proven(Arc<ConformanceEvidence>),
    NotDeclared,
    InvalidSource(ImplId),
    Incomplete(ImplId),
    CoherenceConflict(Box<[ImplId]>),
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}

/// Builds the source witness/default plan. The caller supplies canonical
/// signatures already published under the conformance owner; target surfaces
/// are intentionally not mutated.
pub fn build_conformance_witness_plan(
    contribution: &ConformanceContribution,
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    hierarchy: &dyn TypeHierarchy,
    trait_surface: &crate::traits::TraitSurface,
    callable_signatures: &crate::signature::CallableSignatureTable,
    bodyful_callables: &BTreeSet<CallableId>,
    witness_visibilities: &BTreeMap<CallableId, MemberVisibility>,
    invalid_explicit_requirements: &BTreeSet<crate::traits::TraitRequirementId>,
    invalid_explicit_members: bool,
    inherent_candidates: &BTreeMap<crate::traits::TraitRequirementId, EffectiveInherentWitness>,
    inherent_mismatches: &BTreeMap<crate::traits::TraitRequirementId, Box<str>>,
    deferred_inherent_candidates: &BTreeMap<crate::traits::TraitRequirementId, (EffectiveInherentWitness, ConformanceCompleteness)>,
    data_candidates: &BTreeMap<crate::traits::TraitRequirementId, (crate::identity::DataComponentId, TypeId)>,
    terminal_candidates: &BTreeMap<crate::traits::TraitRequirementId, ConformanceCompleteness>,
) -> ConformanceWitnessPlan {
    let requirement_views = instantiate_conformance_requirements(
        store,
        Some(declarations),
        trait_surface,
        &contribution.trait_ref,
        contribution.target_head,
        &HashMap::new(),
    );
    let mut requirements = BTreeMap::new();
    let mut failures = Vec::new();
    let mut terminal_state = None;
    for (requirement, member) in trait_surface.iter() {
        if let Some(reason) = inherent_mismatches.get(requirement) {
            failures.push(RequirementFailure {
                requirement: requirement.clone(),
                reason: format!("inherent selector conflicts with requirement: {reason}").into_boxed_str(),
            });
            continue;
        }
        if invalid_explicit_requirements.contains(requirement) {
            failures.push(RequirementFailure {
                requirement: requirement.clone(),
                reason: "explicit witness declaration is invalid".into(),
            });
            continue;
        }
        let explicit = callable_signatures.iter().find_map(|(callable, _)| {
            (callable.conformance_owner() == Some(&contribution.impl_id) && callable.selector == requirement.selector && callable.side == requirement.side)
                .then_some(callable.clone())
        });
        if let Some(callable) = explicit {
            if bodyful_callables.contains(&callable) {
                let Some(candidate) = callable_signatures.get(&callable) else {
                    failures.push(RequirementFailure {
                        requirement: requirement.clone(),
                        reason: "explicit witness signature is unavailable".into(),
                    });
                    continue;
                };
                let Some(required_view) = requirement_views.get(requirement) else {
                    failures.push(RequirementFailure {
                        requirement: requirement.clone(),
                        reason: "instantiated trait requirement view is unavailable".into(),
                    });
                    continue;
                };
                match check_witness_compatibility_with_visibility(
                    store,
                    hierarchy,
                    required_view,
                    candidate,
                    witness_visibilities.get(&callable).copied().unwrap_or_default(),
                ) {
                    WitnessCompatibility::Compatible(_) => {
                        requirements.insert(requirement.clone(), RequirementSelectionTemplate::ConformanceCallable { callable });
                    }
                    WitnessCompatibility::Incompatible(mismatch) => failures.push(RequirementFailure {
                        requirement: requirement.clone(),
                        reason: mismatch.reason,
                    }),
                    WitnessCompatibility::Unknown(reason) => retain_stronger_terminal_state(&mut terminal_state, ConformanceCompleteness::Unknown(reason)),
                    WitnessCompatibility::Blocked(reason) => retain_stronger_terminal_state(&mut terminal_state, ConformanceCompleteness::Blocked(reason)),
                    WitnessCompatibility::Dynamic(obligation) => {
                        retain_stronger_terminal_state(&mut terminal_state, ConformanceCompleteness::Dynamic(obligation))
                    }
                    WitnessCompatibility::Cancelled => retain_stronger_terminal_state(&mut terminal_state, ConformanceCompleteness::Cancelled),
                    WitnessCompatibility::BudgetExceeded(report) => {
                        retain_stronger_terminal_state(&mut terminal_state, ConformanceCompleteness::BudgetExceeded(report))
                    }
                    WitnessCompatibility::InternalFailure(message) => {
                        retain_stronger_terminal_state(&mut terminal_state, ConformanceCompleteness::InternalFailure(message))
                    }
                }
            } else {
                failures.push(RequirementFailure {
                    requirement: requirement.clone(),
                    reason: "explicit witness has no body".into(),
                });
            }
        } else if let Some((candidate, pending)) = deferred_inherent_candidates.get(requirement) {
            let fallback = data_candidates
                .get(requirement)
                .map(|(component, specialized_type)| {
                    Box::new(RequirementSelectionTemplate::DataComponent {
                        component: component.clone(),
                        specialized_type: *specialized_type,
                    })
                })
                .or_else(|| {
                    member.default_present.then(|| {
                        Box::new(RequirementSelectionTemplate::TraitDefault {
                            callable: requirement.source_callable(),
                        })
                    })
                });
            requirements.insert(
                requirement.clone(),
                RequirementSelectionTemplate::ConditionalInherent {
                    candidate: candidate.clone(),
                    pending: pending.clone(),
                    fallback,
                },
            );
            retain_stronger_terminal_state(&mut terminal_state, pending.clone());
        } else if let Some(callable) = inherent_candidates.get(requirement) {
            requirements.insert(
                requirement.clone(),
                RequirementSelectionTemplate::InherentCallable {
                    callable: callable.callable.clone(),
                    conditional_impl: callable.conditional_impl.clone(),
                    applicability: callable.applicability.clone(),
                },
            );
        } else if let Some((component, specialized_type)) = data_candidates.get(requirement) {
            requirements.insert(
                requirement.clone(),
                RequirementSelectionTemplate::DataComponent {
                    component: component.clone(),
                    specialized_type: *specialized_type,
                },
            );
        } else if member.default_present {
            requirements.insert(
                requirement.clone(),
                RequirementSelectionTemplate::TraitDefault {
                    callable: requirement.source_callable(),
                },
            );
        } else if let Some(terminal) = terminal_candidates.get(requirement) {
            terminal_state = Some(terminal.clone());
        } else {
            failures.push(RequirementFailure {
                requirement: requirement.clone(),
                reason: "no explicit witness or trait default".into(),
            });
        }
    }
    let completeness = if !failures.is_empty() || invalid_explicit_members {
        ConformanceCompleteness::Incomplete {
            failures: failures.into_boxed_slice(),
        }
    } else if let Some(terminal_state) = terminal_state {
        terminal_state
    } else {
        ConformanceCompleteness::Complete
    };
    let mut plan = ConformanceWitnessPlan {
        impl_id: contribution.impl_id.clone(),
        source_module: contribution.source_module.clone(),
        target_template: contribution.target_head,
        trait_ref_template: contribution.trait_ref.clone(),
        generic_signature: contribution.generic_signature.clone(),
        requirement_views,
        requirements,
        fingerprint: ProductFingerprint::default(),
        invalid_explicit_members,
        completeness,
        diagnostics: contribution.diagnostics.clone(),
    };
    plan.fingerprint = conformance_witness_plan_fingerprint(&plan);
    plan
}

/// Computes the semantic identity of a source witness plan. Diagnostics and
/// source ranges are deliberately excluded; requirement views and selections
/// already carry the canonical contract and source identities they depend on.
pub fn conformance_witness_plan_fingerprint(plan: &ConformanceWitnessPlan) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    plan.impl_id.hash(&mut hasher);
    plan.source_module.hash(&mut hasher);
    plan.target_template.hash(&mut hasher);
    plan.trait_ref_template.hash(&mut hasher);
    format!("{:?}", plan.generic_signature).hash(&mut hasher);
    format!("{:?}", plan.requirement_views).hash(&mut hasher);
    format!("{:?}", plan.requirements).hash(&mut hasher);
    plan.invalid_explicit_members.hash(&mut hasher);
    format!("{:?}", plan.completeness).hash(&mut hasher);
    ProductFingerprint::new(hasher.finish())
}

fn conformance_evidence_fingerprint(
    source_plan: ProductFingerprint,
    target: TypeId,
    trait_ref: &TraitRef,
    environment: &TypeEnvironment,
    requirement_views: &BTreeMap<crate::traits::TraitRequirementId, InstantiatedTraitRequirement>,
    requirements: &BTreeMap<crate::traits::TraitRequirementId, RequirementSelectionTemplate>,
) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    source_plan.hash(&mut hasher);
    target.hash(&mut hasher);
    trait_ref.hash(&mut hasher);
    let mut bindings = environment.bindings.iter().collect::<Vec<_>>();
    bindings.sort_by_key(|(parameter, _)| **parameter);
    bindings.hash(&mut hasher);
    let mut row_bindings = environment.row_bindings.iter().collect::<Vec<_>>();
    row_bindings.sort_by_key(|(parameter, _)| **parameter);
    row_bindings.hash(&mut hasher);
    environment.self_binding.hash(&mut hasher);
    format!("{:?}", requirement_views).hash(&mut hasher);
    format!("{:?}", requirements).hash(&mut hasher);
    ProductFingerprint::new(hasher.finish())
}

/// Instantiates a complete source plan for one exact P1 head match without
/// repeating witness/default selection.
pub fn resolve_conformance_evidence(
    index: &ConformanceIndex,
    plans: &BTreeMap<ImplId, Arc<ConformanceWitnessPlan>>,
    trait_surface: &TraitSurface,
    declarations: &DeclarationTypeTable,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    target: TypeId,
    trait_ref: &TraitRef,
) -> ConformanceResolution {
    let matches = index.query_exact(store, target, trait_ref);
    match matches.len() {
        0 => return ConformanceResolution::NotDeclared,
        1 => {}
        _ => {
            return ConformanceResolution::CoherenceConflict(matches.into_iter().map(|item| item.impl_id).collect::<Vec<_>>().into_boxed_slice());
        }
    }
    let head = &matches[0];
    let Some(plan) = plans.get(&head.impl_id) else {
        return ConformanceResolution::InvalidSource(head.impl_id.clone());
    };
    let has_deferred_selection = plan
        .requirements
        .values()
        .any(|selection| matches!(selection, RequirementSelectionTemplate::ConditionalInherent { .. }));
    if let Some(resolution) = plan.completeness.clone().into_resolution(head.impl_id.clone())
        && !has_deferred_selection
    {
        return resolution;
    }
    let environment = conformance_environment(store, Some(declarations), trait_surface, &head.exact_trait_ref, target, &head.impl_bindings);
    let requirement_views = trait_surface.instantiate(store, &environment);
    let mut requirements = BTreeMap::new();
    for (requirement, selection) in &plan.requirements {
        let selection = match selection {
            RequirementSelectionTemplate::ConditionalInherent {
                candidate,
                pending: _pending,
                fallback,
            } => {
                let owner_specialization =
                    match crate::types::specialization::specialize_receiver_to_owner(store, hierarchy, target, &candidate.owner, &CheckerControl::default()) {
                        Ok(specialization) => specialization,
                        Err(error) => {
                            let state = match error {
                                crate::types::specialization::ReceiverSpecializationFailure::Blocked(reason) => ConformanceCompleteness::Blocked(reason),
                                crate::types::specialization::ReceiverSpecializationFailure::Cancelled => ConformanceCompleteness::Cancelled,
                                crate::types::specialization::ReceiverSpecializationFailure::BudgetExceeded(report) => {
                                    ConformanceCompleteness::BudgetExceeded(report)
                                }
                                other => ConformanceCompleteness::InternalFailure(
                                    format!("deferred inherent witness specialization failed: {other:?}").into_boxed_str(),
                                ),
                            };
                            return state
                                .into_resolution(head.impl_id.clone())
                                .expect("terminal specialization state must produce a resolution");
                        }
                    };
                let owner_view = owner_specialization.path.last().map(|step| step.specialized_form).unwrap_or(target);
                let Some(domain) = candidate.conditional_domain.as_ref() else {
                    return ConformanceResolution::InternalFailure("deferred inherent witness has no retained applicability domain".into());
                };
                match check_impl_domain_applicability(store, hierarchy, domain, target, owner_view, &[]) {
                    ImplApplicabilityResult::Applicable(applicability) => RequirementSelectionTemplate::InherentCallable {
                        callable: candidate.callable.clone(),
                        conditional_impl: Some(candidate.conditional_impl.clone().expect("conditional candidate has impl provenance")),
                        applicability: Some(applicability),
                    },
                    ImplApplicabilityResult::NotApplicable => {
                        let Some(fallback) = fallback else {
                            return ConformanceResolution::Incomplete(head.impl_id.clone());
                        };
                        specialize_requirement_selection(store, fallback, &environment)
                    }
                    other => {
                        return applicability_to_completeness(other)
                            .into_resolution(head.impl_id.clone())
                            .expect("terminal applicability state must produce a resolution");
                    }
                }
            }
            _ => specialize_requirement_selection(store, selection, &environment),
        };
        requirements.insert(requirement.clone(), selection);
    }
    let fingerprint = conformance_evidence_fingerprint(plan.fingerprint, target, &head.exact_trait_ref, &environment, &requirement_views, &requirements);
    ConformanceResolution::Proven(Arc::new(ConformanceEvidence {
        source_impl: head.impl_id.clone(),
        exact_target: target,
        exact_trait_ref: head.exact_trait_ref.clone(),
        impl_environment: environment,
        requirement_views,
        requirements,
        fingerprint,
    }))
}

pub(crate) fn instantiate_conformance_requirements(
    store: &mut TypeStore,
    declarations: Option<&DeclarationTypeTable>,
    trait_surface: &TraitSurface,
    trait_ref: &TraitRef,
    target: TypeId,
    impl_bindings: &HashMap<TypeParameterId, TypeId>,
) -> BTreeMap<crate::traits::TraitRequirementId, InstantiatedTraitRequirement> {
    let environment = conformance_environment(store, declarations, trait_surface, trait_ref, target, impl_bindings);
    trait_surface.instantiate(store, &environment)
}

/// Specializes a target-owned type (such as a data component declaration
/// type) through the source or exact conformance target environment.
pub(crate) fn specialize_conformance_target_type(
    store: &mut TypeStore,
    declarations: &DeclarationTypeTable,
    target: TypeId,
    impl_bindings: &HashMap<TypeParameterId, TypeId>,
    ty: TypeId,
) -> TypeId {
    let mut environment = TypeEnvironment::new();
    for (&parameter, &binding) in impl_bindings {
        environment.bind_param(parameter, binding);
    }
    bind_target_declaration_parameters(store, declarations, target, &mut environment);
    crate::types::environment::TypeView::new(ty, environment).materialize(store)
}

fn conformance_environment(
    store: &mut TypeStore,
    declarations: Option<&DeclarationTypeTable>,
    trait_surface: &TraitSurface,
    trait_ref: &TraitRef,
    target: TypeId,
    impl_bindings: &HashMap<TypeParameterId, TypeId>,
) -> TypeEnvironment {
    let mut environment = TypeEnvironment::new();
    for (&parameter, &binding) in impl_bindings {
        environment.bind_param(parameter, binding);
    }
    if let Some(declarations) = declarations {
        bind_target_declaration_parameters(store, declarations, target, &mut environment);
    }
    if let Some(signature) = &trait_surface.generic_signature {
        for (&parameter, &argument) in signature.parameters.iter().zip(trait_ref.arguments.iter()) {
            let argument = if impl_bindings.is_empty() {
                argument
            } else {
                let substitution = environment.to_substitution();
                substitution.apply(store, argument)
            };
            environment.bind_param(parameter, argument);
        }
    }
    environment.bind_self(target);
    environment
}

fn bind_target_declaration_parameters(store: &mut TypeStore, declarations: &DeclarationTypeTable, mut target: TypeId, environment: &mut TypeEnvironment) {
    while let TypeData::ExactCase { enum_type, .. } = store.get(target) {
        target = *enum_type;
    }
    let TypeData::Applied { origin, arguments } = store.get(target).clone() else {
        return;
    };
    let TypeData::Nominal { declaration } = store.get(origin).clone() else {
        return;
    };
    let Some(signature) = declarations.generic_signature(&declaration) else {
        return;
    };
    let substitution = environment.to_substitution();
    for (&parameter, &argument) in signature.parameters.iter().zip(arguments.iter()) {
        environment.bind_param(parameter, substitution.apply(store, argument));
    }
}

fn specialize_requirement_selection(
    store: &mut TypeStore,
    selection: &RequirementSelectionTemplate,
    environment: &TypeEnvironment,
) -> RequirementSelectionTemplate {
    match selection {
        RequirementSelectionTemplate::DataComponent { component, specialized_type } => RequirementSelectionTemplate::DataComponent {
            component: component.clone(),
            specialized_type: crate::types::environment::TypeView::new(*specialized_type, environment.clone()).materialize(store),
        },
        RequirementSelectionTemplate::ConformanceCallable { callable } => RequirementSelectionTemplate::ConformanceCallable { callable: callable.clone() },
        RequirementSelectionTemplate::InherentCallable {
            callable,
            conditional_impl,
            applicability,
        } => RequirementSelectionTemplate::InherentCallable {
            callable: callable.clone(),
            conditional_impl: conditional_impl.clone(),
            applicability: applicability
                .as_ref()
                .map(|applicability| specialize_inherent_applicability(store, applicability, environment)),
        },
        RequirementSelectionTemplate::TraitDefault { callable } => RequirementSelectionTemplate::TraitDefault { callable: callable.clone() },
        RequirementSelectionTemplate::ConditionalInherent { candidate, pending, fallback } => RequirementSelectionTemplate::ConditionalInherent {
            candidate: candidate.clone(),
            pending: pending.clone(),
            fallback: fallback
                .as_ref()
                .map(|fallback| Box::new(specialize_requirement_selection(store, fallback, environment))),
        },
    }
}

fn specialize_inherent_applicability(
    store: &mut TypeStore,
    applicability: &InherentImplSpecialization,
    environment: &TypeEnvironment,
) -> InherentImplSpecialization {
    let mut materialize = |ty| crate::types::environment::TypeView::new(ty, environment.clone()).materialize(store);
    let bindings = applicability.bindings.iter().map(|(&parameter, &ty)| (parameter, materialize(ty))).collect();
    let mut specialized_environment = TypeEnvironment::new();
    for (&parameter, &ty) in &applicability.environment.bindings {
        specialized_environment.bind_param(parameter, materialize(ty));
    }
    for (&parameter, &row) in &applicability.environment.row_bindings {
        specialized_environment.bind_row(parameter, row);
    }
    if let Some(self_binding) = applicability.environment.self_binding {
        specialized_environment.bind_self(materialize(self_binding));
    }
    InherentImplSpecialization {
        impl_id: applicability.impl_id.clone(),
        receiver: materialize(applicability.receiver),
        owner_view: materialize(applicability.owner_view),
        bindings,
        environment: specialized_environment,
    }
}

/// Workspace-level source index for explicit conformance heads.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConformanceIndex {
    contributions: BTreeMap<ImplId, ConformanceContribution>,
}

impl ConformanceIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, contribution: ConformanceContribution) {
        self.contributions.insert(contribution.impl_id.clone(), contribution);
    }

    pub fn remove_source(&mut self, module: &crate::identity::ModuleId) {
        self.contributions.retain(|impl_id, _| &impl_id.module != module);
    }

    pub fn get(&self, impl_id: &ImplId) -> Option<&ConformanceContribution> {
        self.contributions.get(impl_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ImplId, &ConformanceContribution)> {
        self.contributions.iter()
    }

    /// Returns every eligible source head matching the exact target and trait
    /// reference. Coherence is a separate query and must reject multiple
    /// matches rather than choosing by insertion or source order.
    pub fn query_exact(&self, store: &mut TypeStore, target: TypeId, trait_ref: &TraitRef) -> Vec<ConformanceHeadMatch> {
        let mut matches = Vec::new();
        for contribution in self.contributions.values().filter(|contribution| contribution.is_lookup_eligible()) {
            if contribution.trait_ref.declaration != trait_ref.declaration || contribution.trait_ref.arguments.len() != trait_ref.arguments.len() {
                continue;
            }
            let mut bindings = HashMap::new();
            if !match_impl_domain_head(store, contribution.target_head, target, &contribution.impl_id, &mut bindings) {
                continue;
            }
            if contribution
                .trait_ref
                .arguments
                .iter()
                .zip(trait_ref.arguments.iter())
                .all(|(&head, &actual)| match_impl_domain_head(store, head, actual, &contribution.impl_id, &mut bindings))
            {
                let substitution = bindings.iter().fold(TypeSubstitution::new(), |mut substitution, (&parameter, &ty)| {
                    substitution.bind(parameter, ty);
                    substitution
                });
                let exact_trait_ref = TraitRef::new(
                    trait_ref.declaration.clone(),
                    contribution
                        .trait_ref
                        .arguments
                        .iter()
                        .map(|&argument| substitution.apply(store, argument))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                );
                matches.push(ConformanceHeadMatch {
                    impl_id: contribution.impl_id.clone(),
                    exact_target: target,
                    exact_trait_ref,
                    impl_bindings: bindings,
                });
            }
        }
        matches
    }

    /// Returns pairs of eligible source heads whose joint target and trait
    /// patterns overlap. No precedence or specialization rule is applied.
    pub fn overlap_conflicts(&self, store: &TypeStore) -> Vec<(ImplId, ImplId)> {
        let contributions = self
            .contributions
            .values()
            .filter(|contribution| contribution.is_lookup_eligible())
            .collect::<Vec<_>>();
        let mut conflicts = Vec::new();
        for (index, first) in contributions.iter().enumerate() {
            for second in contributions.iter().skip(index + 1) {
                if first.trait_ref.declaration != second.trait_ref.declaration || first.trait_ref.arguments.len() != second.trait_ref.arguments.len() {
                    continue;
                }
                let mut bindings = HashMap::new();
                if !unify_conformance_head(store, first.target_head, second.target_head, &mut bindings) {
                    continue;
                }
                if first
                    .trait_ref
                    .arguments
                    .iter()
                    .zip(second.trait_ref.arguments.iter())
                    .all(|(&left, &right)| unify_conformance_head(store, left, right, &mut bindings))
                {
                    conflicts.push((first.impl_id.clone(), second.impl_id.clone()));
                }
            }
        }
        conflicts
    }
}

fn impl_parameter(store: &TypeStore, ty: TypeId) -> Option<TypeParameterId> {
    let TypeData::Parameter(parameter) = store.get(ty) else { return None };
    matches!(store.type_parameter(*parameter).owner, TypeParameterOwner::Impl(_)).then_some(*parameter)
}

fn occurs_in_conformance_binding(store: &TypeStore, parameter: TypeParameterId, ty: TypeId, bindings: &HashMap<TypeParameterId, TypeId>) -> bool {
    match store.get(ty) {
        TypeData::Parameter(candidate) => {
            if *candidate == parameter {
                true
            } else if let Some(&replacement) = bindings.get(candidate) {
                occurs_in_conformance_binding(store, parameter, replacement, bindings)
            } else {
                false
            }
        }
        TypeData::Applied { origin, arguments } => {
            occurs_in_conformance_binding(store, parameter, *origin, bindings)
                || arguments
                    .iter()
                    .any(|&argument| occurs_in_conformance_binding(store, parameter, argument, bindings))
        }
        TypeData::ExactCase { enum_type, .. } => occurs_in_conformance_binding(store, parameter, *enum_type, bindings),
        TypeData::Union(members) => members.iter().any(|&member| occurs_in_conformance_binding(store, parameter, member, bindings)),
        TypeData::Tuple(elements) => elements
            .iter()
            .any(|element| occurs_in_conformance_binding(store, parameter, element.ty, bindings)),
        TypeData::Record(row) => store
            .record_row(*row)
            .fields
            .iter()
            .any(|field| occurs_in_conformance_binding(store, parameter, field.ty, bindings)),
        TypeData::Callable(callable) => {
            callable
                .parameters
                .iter()
                .any(|callable_parameter| occurs_in_conformance_binding(store, parameter, callable_parameter.ty, bindings))
                || occurs_in_conformance_binding(store, parameter, callable.return_type, bindings)
        }
        _ => false,
    }
}

fn bind_conformance_parameter(store: &TypeStore, parameter: TypeParameterId, ty: TypeId, bindings: &mut HashMap<TypeParameterId, TypeId>) -> bool {
    if let Some(&existing) = bindings.get(&parameter) {
        return unify_conformance_head(store, existing, ty, bindings);
    }
    if occurs_in_conformance_binding(store, parameter, ty, bindings) {
        return false;
    }
    bindings.insert(parameter, ty);
    true
}

fn unify_conformance_head(store: &TypeStore, left: TypeId, right: TypeId, bindings: &mut HashMap<TypeParameterId, TypeId>) -> bool {
    if left == right {
        return true;
    }
    if let Some(parameter) = impl_parameter(store, left) {
        return bind_conformance_parameter(store, parameter, right, bindings);
    }
    if let Some(parameter) = impl_parameter(store, right) {
        return bind_conformance_parameter(store, parameter, left, bindings);
    }

    match (store.get(left), store.get(right)) {
        (TypeData::Nominal { declaration: left }, TypeData::Nominal { declaration: right }) => left == right,
        (
            TypeData::Applied {
                origin: left_origin,
                arguments: left_args,
            },
            TypeData::Applied {
                origin: right_origin,
                arguments: right_args,
            },
        ) => {
            left_args.len() == right_args.len()
                && unify_conformance_head(store, *left_origin, *right_origin, bindings)
                && left_args
                    .iter()
                    .zip(right_args.iter())
                    .all(|(&left, &right)| unify_conformance_head(store, left, right, bindings))
        }
        (
            TypeData::ExactCase {
                variant: left_variant,
                enum_type: left_enum,
            },
            TypeData::ExactCase {
                variant: right_variant,
                enum_type: right_enum,
            },
        ) => left_variant == right_variant && unify_conformance_head(store, *left_enum, *right_enum, bindings),
        (TypeData::Tuple(left), TypeData::Tuple(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(left, right)| left.label == right.label && unify_conformance_head(store, left.ty, right.ty, bindings))
        }
        (TypeData::Record(left), TypeData::Record(right)) => {
            let left = store.record_row(*left);
            let right = store.record_row(*right);
            left.fields.len() == right.fields.len()
                && left
                    .fields
                    .iter()
                    .zip(right.fields.iter())
                    .all(|(left, right)| left.name == right.name && unify_conformance_head(store, left.ty, right.ty, bindings))
        }
        _ => false,
    }
}

impl ResolvedInherentImplTarget {
    pub fn declaration(&self) -> &DeclarationId {
        self.target.declaration()
    }
}

/// Resolves an explicit `impl TraitRef for Target` header under one
/// impl-owned generic environment.
pub fn resolve_conformance_head(
    ctx: &mut CheckingContext<'_>,
    trait_headers: &TraitHeaderTable,
    impl_id: &ImplId,
    impl_def: &ImplDef,
) -> Result<ResolvedConformanceHead, Vec<SemanticDiagnostic>> {
    let phalcom_ast::ast::ImplKind::Conformance { trait_ref: trait_syntax, .. } = &impl_def.kind else {
        return Err(vec![SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::AnnotationUnresolved,
            "expected an explicit conformance implementation",
            impl_def.range,
        )]);
    };

    let mut diagnostics = Vec::new();
    let mut impl_type_parameter_ids = Vec::new();
    let mut impl_type_parameter_map = HashMap::new();
    for (index, parameter) in impl_def.generic_parameters.iter().enumerate() {
        let data = TypeParameterData::new(TypeParameterOwner::Impl(impl_id.clone()), index as u32, parameter.name.clone(), KindId::TYPE);
        let id = ctx.store.intern_type_parameter(data);
        impl_type_parameter_ids.push(id);
        impl_type_parameter_map.insert(parameter.name.clone(), type_level_binding_for_parameter(ctx.store, id));
    }

    let parent_resolver = ctx.resolver.clone();
    let impl_resolver = ScopedTypeResolver {
        parent: &parent_resolver,
        type_parameters: impl_type_parameter_map,
    };
    let formation_site = TypeFormationSite::module(ctx.current_module.clone());

    let mut constraints = Vec::new();
    if let Some(where_clause) = &impl_def.where_clause {
        diagnostics.push(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::ImplWhereClauseUnsupported,
            "conformance where-clauses are deferred until generic trait constraints are implemented",
            where_clause.range,
        ));
        for constraint in &where_clause.constraints {
            match constraint {
                phalcom_ast::ast::GenericConstraintSyntax::Subtype { lower, upper, .. } => {
                    let lower = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, lower, &mut diagnostics);
                    let upper = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, upper, &mut diagnostics);
                    if let (TypeKnowledge::Known(lower), TypeKnowledge::Known(upper)) = (lower, upper) {
                        constraints.push(GenericConstraint::Subtype {
                            lower: TypeTerm::Canonical(lower.ty()),
                            upper: TypeTerm::Canonical(upper.ty()),
                        });
                    }
                }
                phalcom_ast::ast::GenericConstraintSyntax::Equivalent { left, right, .. } => {
                    let left = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, left, &mut diagnostics);
                    let right = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, right, &mut diagnostics);
                    if let (TypeKnowledge::Known(left), TypeKnowledge::Known(right)) = (left, right) {
                        constraints.push(GenericConstraint::Equivalent {
                            left: TypeTerm::Canonical(left.ty()),
                            right: TypeTerm::Canonical(right.ty()),
                        });
                    }
                }
                phalcom_ast::ast::GenericConstraintSyntax::Invalid { message, range } => {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        message.clone(),
                        *range,
                    ));
                }
            }
        }
    }

    let generic_signature = if impl_type_parameter_ids.is_empty() && constraints.is_empty() {
        None
    } else {
        Some(GenericSignature::with_constraints(
            TypeParameterOwner::Impl(impl_id.clone()),
            impl_type_parameter_ids.clone().into_boxed_slice(),
            constraints.into_boxed_slice(),
        ))
    };

    let trait_declaration = trait_syntax.origin_symbol_ref().and_then(|reference| {
        let members = reference.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
        ctx.resolver.resolve_type_name(&ctx.current_module, &reference.root, &members).or_else(|| {
            // Trait declarations intentionally do not enter the nominal
            // declaration type table. Preserve canonical local trait
            // identity through the C3 header table when the ordinary
            // value-type resolver therefore has no result.
            if members.is_empty() {
                let local = DeclarationId::new(ctx.current_module.clone(), reference.root.clone().into());
                trait_headers.contains(&local).then_some(local)
            } else {
                None
            }
        })
    });
    let Some(trait_declaration) = trait_declaration else {
        diagnostics.push(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::AnnotationUnresolved,
            "conformance trait reference could not be resolved",
            trait_syntax.range,
        ));
        return Err(diagnostics);
    };
    let Some(trait_header) = trait_headers.get(&trait_declaration) else {
        diagnostics.push(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::AnnotationUnresolved,
            format!("declaration `{}` is not a trait", trait_declaration.name),
            trait_syntax.range,
        ));
        return Err(diagnostics);
    };

    let trait_arguments = match &trait_syntax.expr {
        TypeAnnotationExpr::Reference(_) => Vec::new(),
        TypeAnnotationExpr::Application { arguments, .. } => {
            let mut resolved = Vec::with_capacity(arguments.len());
            for argument in arguments {
                let TypeKnowledge::Known(evidence) =
                    resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, argument, &mut diagnostics)
                else {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        "conformance trait argument could not be resolved",
                        argument.range,
                    ));
                    continue;
                };
                resolved.push(evidence.ty());
            }
            resolved
        }
        _ => {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                "conformance trait reference must be a trait name or application",
                trait_syntax.range,
            ));
            return Err(diagnostics);
        }
    };
    let expected_arguments = trait_header.generic_signature.as_ref().map_or(0, |signature| signature.parameters.len());
    if trait_arguments.len() != expected_arguments {
        diagnostics.push(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::AnnotationUnresolved,
            format!(
                "trait `{}` expects {expected_arguments} type arguments, got {}",
                trait_declaration.name,
                trait_arguments.len()
            ),
            trait_syntax.range,
        ));
        return Err(diagnostics);
    }
    if let Some(signature) = &trait_header.generic_signature {
        for (&argument, &parameter) in trait_arguments.iter().zip(signature.parameters.iter()) {
            let expected_kind = ctx.store.type_parameter(parameter).kind;
            let actual_kind = ctx.store.kind_of(argument);
            if expected_kind != actual_kind {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::KindExpectedType,
                    format!("trait argument has kind {actual_kind:?}, expected {expected_kind:?}"),
                    trait_syntax.range,
                ));
                return Err(diagnostics);
            }
        }
    }
    let trait_ref = match TraitRef::form(ctx.store, trait_headers, &ctx.hierarchy, trait_declaration, &trait_arguments) {
        Ok(trait_ref) => trait_ref,
        Err(error) => {
            let message = format!("invalid conformance trait reference: {error}");
            let range = trait_syntax.range;
            let code = match error {
                TraitRefFormationError::Kind { .. } => DiagnosticCode::KindExpectedType,
                TraitRefFormationError::NotTrait(_)
                | TraitRefFormationError::Arity { .. }
                | TraitRefFormationError::InvalidArgument { .. }
                | TraitRefFormationError::ConstraintUnsatisfied { .. }
                | TraitRefFormationError::ConstraintNotCanonical { .. } => DiagnosticCode::AnnotationUnresolved,
            };
            diagnostics.push(SemanticDiagnostic::error_in(ctx.current_module.clone(), code, message, range));
            return Err(diagnostics);
        }
    };

    let (target_head, target) = if let TypeAnnotationExpr::ExactEnumCase {
        enum_target,
        variant_name,
        payload_shape,
        ..
    } = &impl_def.target.expr
    {
        let TypeKnowledge::Known(enum_evidence) =
            resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, enum_target, &mut diagnostics)
        else {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplTargetNotNominal,
                "conformance exact-case enum target could not be resolved",
                enum_target.range,
            ));
            return Err(diagnostics);
        };
        let enum_declaration = match ctx.store.get(enum_evidence.ty()).clone() {
            TypeData::Nominal { declaration } => declaration,
            TypeData::Applied { origin, .. } => match ctx.store.get(origin).clone() {
                TypeData::Nominal { declaration } => declaration,
                _ => {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ImplTargetNotNominal,
                        "conformance exact-case enum target must be nominal",
                        enum_target.range,
                    ));
                    return Err(diagnostics);
                }
            },
            _ => {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplTargetNotNominal,
                    "conformance exact-case enum target must be nominal",
                    enum_target.range,
                ));
                return Err(diagnostics);
            }
        };
        let variant = crate::identity::VariantId::new(
            enum_declaration,
            phalcom_ast::selector::selector_from_exact_case_target(variant_name, payload_shape.as_ref()),
        );
        let Some(variant_info) = ctx.variant_info(&variant).cloned() else {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplTargetNotNominal,
                "conformance exact enum case is not a declared variant",
                impl_def.target.range,
            ));
            return Err(diagnostics);
        };
        (variant_info.exact_case_template, ConformanceTarget::ExactEnumCase(variant))
    } else {
        let TypeKnowledge::Known(target_evidence) =
            resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, &impl_def.target, &mut diagnostics)
        else {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplTargetNotNominal,
                "conformance target must be a nominal declaration or exact enum case",
                impl_def.target.range,
            ));
            return Err(diagnostics);
        };
        let target_head = target_evidence.ty();
        let target = match ctx.store.get(target_head).clone() {
            TypeData::Nominal { declaration } => {
                if trait_headers.contains(&declaration) || ctx.resolver.resolve_alias_form(&declaration).is_some() {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ImplTargetNotNominal,
                        "conformance target must not be a trait or type alias",
                        impl_def.target.range,
                    ));
                    return Err(diagnostics);
                }
                if ctx.declaration_generic_signature(&declaration).is_some() {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ImplSpecializedTargetUnsupported,
                        format!(
                            "generic declaration `{}` requires explicit type arguments in a conformance target",
                            declaration.name
                        ),
                        impl_def.target.range,
                    ));
                    return Err(diagnostics);
                }
                ConformanceTarget::Declaration(declaration)
            }
            TypeData::Applied { origin, .. } => {
                let TypeData::Nominal { declaration } = ctx.store.get(origin).clone() else {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ImplTargetNotNominal,
                        "conformance target application must have a nominal declaration origin",
                        impl_def.target.range,
                    ));
                    return Err(diagnostics);
                };
                if trait_headers.contains(&declaration) || ctx.resolver.resolve_alias_form(&declaration).is_some() {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ImplTargetNotNominal,
                        "conformance target must not be a trait or type alias",
                        impl_def.target.range,
                    ));
                    return Err(diagnostics);
                }
                ConformanceTarget::Declaration(declaration)
            }
            _ => {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplTargetNotNominal,
                    "conformance target must be a nominal declaration or exact enum case",
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }
        };
        (target_head, target)
    };

    for parameter in impl_type_parameter_ids {
        if !type_contains_impl_param(ctx.store, target_head, impl_id)
            && !trait_ref
                .arguments
                .iter()
                .any(|&argument| type_contains_impl_param(ctx.store, argument, impl_id))
        {
            let name = ctx.store.type_parameter(parameter).name.clone();
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplUnusedTypeParameter,
                format!("type parameter `{name}` is unused in conformance head"),
                impl_def.range,
            ));
        }
    }

    Ok(ResolvedConformanceHead {
        impl_id: impl_id.clone(),
        source_module: ctx.current_module.clone(),
        trait_ref,
        target,
        target_head,
        generic_signature,
        source: SemanticSourceSpan::new(ctx.current_module.clone(), impl_def.range),
        eligible: diagnostics.is_empty(),
        diagnostics: diagnostics.into_boxed_slice(),
    })
}

/// A single behavior-only member contributed by an inherent impl block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InherentMemberContribution {
    pub callable: CallableId,
    pub signature: CallableSemanticSignature,
    pub visibility: MemberVisibility,
    pub source_member: usize,
    pub is_requirement: bool,
}

/// Canonical contribution product for an inherent impl block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InherentImplContribution {
    pub id: ImplId,
    pub target: InherentImplTarget,
    pub generic_signature: Option<GenericSignature>,
    pub covering: Option<CoveringImplSubstitution>,
    /// Whether applicability depends on the receiver beyond the target's
    /// canonical declaration/exact-case identity. Exact-case members retain a
    /// domain for semantic lookup even when their implementation is
    /// unconditional for that hidden case class.
    pub is_conditionally_applicable: bool,
    pub domain: Option<Arc<InherentImplDomain>>,
    pub members: Box<[InherentMemberContribution]>,
    pub source: SemanticSourceSpan,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}

impl InherentImplContribution {
    pub fn target_declaration(&self) -> &DeclarationId {
        self.target.declaration()
    }
}

use crate::associated::AssociatedSurface;
use crate::data_semantics::DataInfo;
use crate::identity::{DataComponentId, FieldId, VariantId};
use crate::surface::DeclarationSurface;
use phalcom_common::selector::Selector;
use std::collections::HashSet;
use std::sync::Arc;

/// A conditional member contributed by a specialized or constrained inherent impl block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConditionalInherentMember {
    pub impl_id: ImplId,
    pub domain: Arc<InherentImplDomain>,
    /// Exact-case identity is represented through this index for semantic
    /// lookup, but only receiver-dependent domains need conditional runtime
    /// lowering. Unconditional/covering exact-case members may be installed
    /// on their hidden variant class.
    pub is_conditionally_applicable: bool,
    pub callable: CallableId,
    pub signature_template: CallableSemanticSignature,
    pub visibility: MemberVisibility,
    pub source_member: usize,
    pub is_requirement: bool,
}

/// Target-indexed set of conditional inherent members.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConditionalInherentMemberSet {
    pub target: Option<InherentImplTarget>,
    pub members: Vec<ConditionalInherentMember>,
    pub by_selector: HashMap<(DispatchSide, Selector), Vec<usize>>,
}

impl ConditionalInherentMemberSet {
    pub fn new(target: Option<InherentImplTarget>) -> Self {
        Self {
            target,
            members: Vec::new(),
            by_selector: HashMap::new(),
        }
    }

    pub fn add_member(&mut self, member: ConditionalInherentMember) {
        let side = member.signature_template.side;
        let selector = member.signature_template.callable.selector.clone();
        let index = self.members.len();
        self.members.push(member);
        self.by_selector.entry((side, selector)).or_default().push(index);
    }

    pub fn get_members(&self, side: DispatchSide, selector: &Selector) -> Option<&[usize]> {
        self.by_selector.get(&(side, selector.clone())).map(|v| v.as_slice())
    }

    pub fn get_member(&self, side: DispatchSide, selector: &Selector) -> Option<&ConditionalInherentMember> {
        let indices = self.get_members(side, selector)?;
        indices.first().map(|&idx| &self.members[idx])
    }
}

/// Proof evidence capturing the specialized instantiation of an inherent impl domain for a receiver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InherentImplSpecialization {
    pub impl_id: ImplId,
    pub receiver: TypeId,
    pub owner_view: TypeId,
    pub bindings: HashMap<TypeParameterId, TypeId>,
    pub environment: TypeEnvironment,
}

/// Outcome of checking applicability of an inherent impl domain to a receiver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImplApplicabilityResult {
    Applicable(InherentImplSpecialization),
    NotApplicable,
    /// The domain may apply, but the available type information is not enough
    /// to prove or refute it.
    Unknown(UnknownReason),
    /// The applicability query could not complete a sound judgment.
    Blocked(BlockReason),
    /// Applicability crosses an explicit dynamic boundary.
    Dynamic(DynamicBoundaryObligation),
    /// The query was cancelled before a judgment was available.
    Cancelled,
    /// The query exhausted one of its bounded resources.
    BudgetExceeded(BudgetReport),
    /// The semantic relation engine reported an internal failure.
    InternalFailure(String),
}

/// Target-indexed set of inherent impl fragments within a module.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InherentImplSet {
    pub target: DeclarationId,
    pub impls: Box<[ImplId]>,
}

/// Origin of a callable definition accepted into the effective surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallableDefinitionOrigin {
    /// Defined in the primary class/enum declaration.
    PrimaryDeclaration,
    /// Defined in an inherent `impl` block fragment.
    InherentImpl(ImplId),
    /// Defined inside an explicit conformance. This callable is semantic
    /// witness provenance only and must not be projected into an inherent
    /// target surface or runtime dispatch table by P2.
    ConformanceWitness(ImplId),
}

/// Canonical definition provenance for an accepted callable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveCallableDefinition {
    pub callable: CallableId,
    pub origin: CallableDefinitionOrigin,
    pub source_member_index: usize,
    pub signature: CallableSemanticSignature,
}

/// Product of merging a primary declared surface with inherent impl contributions.
#[derive(Clone, Debug)]
pub struct EffectiveSurfaceProduct {
    pub owner: DeclarationId,
    pub surface: Arc<DeclarationSurface>,
    pub conditional_members: Arc<ConditionalInherentMemberSet>,
    pub definitions: BTreeMap<CallableId, EffectiveCallableDefinition>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}

impl EffectiveSurfaceProduct {
    pub fn new(
        owner: DeclarationId,
        surface: Arc<DeclarationSurface>,
        conditional_members: Arc<ConditionalInherentMemberSet>,
        definitions: BTreeMap<CallableId, EffectiveCallableDefinition>,
        diagnostics: Arc<[SemanticDiagnostic]>,
    ) -> Self {
        Self {
            owner,
            surface,
            conditional_members,
            definitions,
            diagnostics,
        }
    }
}

/// Internal origin of a reserved member entry during conflict checking.
#[derive(Clone, Debug, Eq, PartialEq)]
enum EffectiveMemberOrigin {
    PrimaryCallable(CallableId),
    InherentCallable { callable: CallableId, impl_id: ImplId },
    DataComponent(DataComponentId),
    VariantConstructor(VariantId),
}

/// Checks and merges a primary declaration surface with inherent impl contributions.
pub fn build_effective_surface(
    owner: &DeclarationId,
    primary_surface: &DeclarationSurface,
    primary_signatures: &HashMap<CallableId, CallableSemanticSignature>,
    inherent_contributions: &[InherentImplContribution],
    data_info: Option<&DataInfo>,
    associated_surface: Option<&AssociatedSurface>,
) -> EffectiveSurfaceProduct {
    let mut diagnostics = Vec::new();
    let mut effective_surface = DeclarationSurface::new(Some(owner.clone()));
    let mut conditional_members = ConditionalInherentMemberSet::new(Some(InherentImplTarget::Declaration(owner.clone())));
    let mut definitions = BTreeMap::new();

    // Reserved namespace tracking per dispatch side
    // (Side, Selector) -> EffectiveMemberOrigin
    let mut reserved_selectors: HashMap<(DispatchSide, Selector), EffectiveMemberOrigin> = HashMap::new();

    // (Side, Field Name) -> (FieldId, MemberVisibility)
    let mut reserved_fields: HashMap<(DispatchSide, String), (FieldId, MemberVisibility)> = HashMap::new();

    // 1. Reserve data components on instance side if applicable
    if let Some(info) = data_info {
        for component in &info.components {
            if let Ok(selector) = Selector::getter(component.local_name.as_ref()) {
                let origin = EffectiveMemberOrigin::DataComponent(component.id.clone());
                reserved_selectors.insert((DispatchSide::Instance, selector), origin);
            }
        }
    }

    // 2. Reserve enum variant constructors on class side if applicable
    if let Some(assoc) = associated_surface {
        for family in assoc.families.values() {
            for member in family.members.iter() {
                if let crate::associated::AssociatedMemberId::Variant(v_id) = member {
                    let origin = EffectiveMemberOrigin::VariantConstructor(v_id.clone());
                    reserved_selectors.insert((DispatchSide::Class, v_id.selector.clone()), origin);
                }
            }
        }
    }

    // 3. Register primary fields
    for (name, field_id) in &primary_surface.instance.fields_by_name {
        let ty = primary_surface
            .instance
            .fields
            .get(name)
            .cloned()
            .unwrap_or(TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape));
        let visibility = primary_surface.instance.field_visibility.get(name).copied().unwrap_or(MemberVisibility::Public);
        reserved_fields.insert((DispatchSide::Instance, name.clone()), (field_id.clone(), visibility));
        effective_surface
            .instance
            .add_field_with_visibility(Some(owner), DispatchSide::Instance, name, ty, visibility);
    }
    for (name, field_id) in &primary_surface.class.fields_by_name {
        let ty = primary_surface
            .class
            .fields
            .get(name)
            .cloned()
            .unwrap_or(TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape));
        let visibility = primary_surface.class.field_visibility.get(name).copied().unwrap_or(MemberVisibility::Public);
        reserved_fields.insert((DispatchSide::Class, name.clone()), (field_id.clone(), visibility));
        effective_surface
            .class
            .add_field_with_visibility(Some(owner), DispatchSide::Class, name, ty, visibility);
    }

    // 4. Register primary callables
    for (side, surface) in [
        (DispatchSide::Instance, &primary_surface.instance),
        (DispatchSide::Class, &primary_surface.class),
    ] {
        for (selector, sig) in &surface.callable_signatures {
            let callable_id = CallableId::new(owner.clone(), selector.clone(), side);
            let visibility = surface.callable_visibility.get(selector).copied().unwrap_or(MemberVisibility::Public);

            // Check if selector already reserved (e.g. data component collision with primary getter)
            if let Some(existing) = reserved_selectors.get(&(side, selector.clone())) {
                match existing {
                    EffectiveMemberOrigin::DataComponent(comp_id) => {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            owner.module.clone(),
                            DiagnosticCode::ImplMemberConflict,
                            format!("declaration member `{}` conflicts with data component index {}", selector, comp_id.index),
                            phalcom_common::range::SourceRange::default(),
                        ));
                        continue;
                    }
                    EffectiveMemberOrigin::VariantConstructor(v_id) => {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            owner.module.clone(),
                            DiagnosticCode::EnumFamilyCategoryConflict,
                            format!("class callable `{}` conflicts with variant constructor `{}`", selector, v_id.selector),
                            phalcom_common::range::SourceRange::default(),
                        ));
                        continue;
                    }
                    _ => {}
                }
            }

            reserved_selectors.insert((side, selector.clone()), EffectiveMemberOrigin::PrimaryCallable(callable_id.clone()));
            effective_surface
                .surface_mut(side)
                .add_callable_with_visibility(Some(owner), side, sig.clone(), visibility);

            if let Some(sem_sig) = primary_signatures.get(&callable_id) {
                definitions.insert(
                    callable_id.clone(),
                    EffectiveCallableDefinition {
                        callable: callable_id,
                        origin: CallableDefinitionOrigin::PrimaryDeclaration,
                        source_member_index: 0,
                        signature: sem_sig.clone(),
                    },
                );
            }
        }
    }

    // 5. Merge inherent impl contributions in deterministic order
    for contribution in inherent_contributions {
        if matches!(contribution.target, InherentImplTarget::ExactEnumCase(_)) {
            continue;
        }
        if contribution.target.declaration() != owner {
            continue;
        }
        diagnostics.extend(contribution.diagnostics.iter().cloned());

        for member in contribution.members.iter() {
            let side = member.signature.side;
            let selector = &member.signature.callable.selector;
            let callable_id = &member.signature.callable;

            // Conflict check
            if let Some(existing) = reserved_selectors.get(&(side, selector.clone())) {
                let err_msg = match existing {
                    EffectiveMemberOrigin::PrimaryCallable(_) => {
                        format!(
                            "impl member `{}` on `{}` conflicts with a primary declaration member of the same selector",
                            selector, owner.name
                        )
                    }
                    EffectiveMemberOrigin::InherentCallable { impl_id, .. } => {
                        format!(
                            "impl member `{}` on `{}` conflicts with another inherent impl member from impl {:?}",
                            selector, owner.name, impl_id
                        )
                    }
                    EffectiveMemberOrigin::DataComponent(comp_id) => {
                        format!(
                            "impl getter `{}` on `{}` conflicts with data component index {}",
                            selector, owner.name, comp_id.index
                        )
                    }
                    EffectiveMemberOrigin::VariantConstructor(v_id) => {
                        format!(
                            "class-side impl member `{}` on `{}` conflicts with enum variant constructor `{}`",
                            selector, owner.name, v_id.selector
                        )
                    }
                };

                let span_range = member.signature.source.as_ref().map(|s| s.range).unwrap_or_default();
                diagnostics.push(SemanticDiagnostic::error_in(
                    owner.module.clone(),
                    DiagnosticCode::ImplMemberConflict,
                    err_msg,
                    span_range,
                ));
                continue;
            }

            reserved_selectors.insert(
                (side, selector.clone()),
                EffectiveMemberOrigin::InherentCallable {
                    callable: callable_id.clone(),
                    impl_id: contribution.id.clone(),
                },
            );

            if let Some(domain) = &contribution.domain {
                conditional_members.add_member(ConditionalInherentMember {
                    impl_id: contribution.id.clone(),
                    domain: domain.clone(),
                    is_conditionally_applicable: contribution.is_conditionally_applicable,
                    callable: callable_id.clone(),
                    signature_template: member.signature.clone(),
                    visibility: member.visibility,
                    source_member: member.source_member,
                    is_requirement: member.is_requirement,
                });
            } else {
                let projection = crate::checker::declaration_signature::project_semantic_signature(&member.signature);
                effective_surface
                    .surface_mut(side)
                    .add_callable_with_visibility(Some(owner), side, projection, member.visibility);
            }

            if !member.is_requirement {
                definitions.insert(
                    callable_id.clone(),
                    EffectiveCallableDefinition {
                        callable: callable_id.clone(),
                        origin: CallableDefinitionOrigin::InherentImpl(contribution.id.clone()),
                        source_member_index: member.source_member,
                        signature: member.signature.clone(),
                    },
                );
            }
        }
    }

    EffectiveSurfaceProduct {
        owner: owner.clone(),
        surface: Arc::new(effective_surface),
        conditional_members: Arc::new(conditional_members),
        definitions,
        diagnostics: Arc::from(diagnostics.into_boxed_slice()),
    }
}

/// Calculates visibility for a behavior member.
pub fn behavior_member_visibility(member: &BehaviorMember) -> MemberVisibility {
    let (name, attributes) = match member {
        BehaviorMember::Method(item) => (Some(item.name.as_str()), item.attributes.as_slice()),
        BehaviorMember::Getter(item) => (Some(item.name.as_str()), item.attributes.as_slice()),
        BehaviorMember::Setter(item) => (Some(item.name.as_str()), item.attributes.as_slice()),
        BehaviorMember::Index(item) => (None, item.attributes.as_slice()),
    };
    if name.is_some_and(|name| name.starts_with("_$")) {
        MemberVisibility::Internal
    } else if attributes.iter().any(|attribute| attribute.name == "private") {
        MemberVisibility::Private
    } else if attributes.iter().any(|attribute| attribute.name == "protected") {
        MemberVisibility::Protected
    } else {
        MemberVisibility::Public
    }
}

/// Recursively collects all type parameters owned by `impl_id` in a `TypeId`.
pub fn collect_impl_params_in_type(store: &TypeStore, ty: TypeId, impl_id: &ImplId, out: &mut HashSet<TypeParameterId>) {
    match store.get(ty) {
        TypeData::Parameter(p) => {
            let data = store.type_parameter(*p);
            if matches!(&data.owner, TypeParameterOwner::Impl(id) if id == impl_id) {
                out.insert(*p);
            }
        }
        TypeData::Applied { origin, arguments } => {
            collect_impl_params_in_type(store, *origin, impl_id, out);
            for &arg in arguments.iter() {
                collect_impl_params_in_type(store, arg, impl_id, out);
            }
        }
        TypeData::ExactCase { enum_type, .. } => {
            collect_impl_params_in_type(store, *enum_type, impl_id, out);
        }
        TypeData::Union(members) => {
            for &m in members.iter() {
                collect_impl_params_in_type(store, m, impl_id, out);
            }
        }
        TypeData::Tuple(elements) => {
            for e in elements.iter() {
                collect_impl_params_in_type(store, e.ty, impl_id, out);
            }
        }
        TypeData::Record(row) => {
            let row_data = store.record_row(*row);
            for f in row_data.fields.iter() {
                collect_impl_params_in_type(store, f.ty, impl_id, out);
            }
        }
        TypeData::Callable(c) => {
            for p in c.parameters.iter() {
                collect_impl_params_in_type(store, p.ty, impl_id, out);
            }
            collect_impl_params_in_type(store, c.return_type, impl_id, out);
        }
        _ => {}
    }
}

/// Matches a domain head type structurally against a receiver form and binds impl-owned parameters.
pub fn match_impl_domain_head(
    store: &TypeStore,
    domain_head: TypeId,
    receiver_form: TypeId,
    impl_id: &ImplId,
    bindings: &mut HashMap<TypeParameterId, TypeId>,
) -> bool {
    if domain_head == receiver_form {
        if let TypeData::Parameter(p) = store.get(domain_head) {
            if store.type_parameter(*p).owner == TypeParameterOwner::Impl(impl_id.clone()) {
                if let Some(&existing) = bindings.get(p) {
                    return existing == receiver_form;
                } else {
                    bindings.insert(*p, receiver_form);
                }
            }
        }
        return true;
    }

    match (store.get(domain_head), store.get(receiver_form)) {
        (TypeData::Parameter(p), _) => {
            if store.type_parameter(*p).owner == TypeParameterOwner::Impl(impl_id.clone()) {
                if let Some(&existing) = bindings.get(p) {
                    existing == receiver_form
                } else {
                    bindings.insert(*p, receiver_form);
                    true
                }
            } else {
                domain_head == receiver_form
            }
        }
        (TypeData::Nominal { declaration: d1 }, TypeData::Nominal { declaration: d2 }) => d1 == d2,
        (TypeData::Applied { origin: o1, arguments: a1 }, TypeData::Applied { origin: o2, arguments: a2 }) => {
            if a1.len() != a2.len() {
                return false;
            }
            if !match_impl_domain_head(store, *o1, *o2, impl_id, bindings) {
                return false;
            }
            for (&arg1, &arg2) in a1.iter().zip(a2.iter()) {
                if !match_impl_domain_head(store, arg1, arg2, impl_id, bindings) {
                    return false;
                }
            }
            true
        }
        (TypeData::ExactCase { variant: v1, enum_type: e1 }, TypeData::ExactCase { variant: v2, enum_type: e2 }) => {
            if v1 != v2 {
                return false;
            }
            match_impl_domain_head(store, *e1, *e2, impl_id, bindings)
        }
        (TypeData::Tuple(elems1), TypeData::Tuple(elems2)) => {
            if elems1.len() != elems2.len() {
                return false;
            }
            for (e1, e2) in elems1.iter().zip(elems2.iter()) {
                if e1.label != e2.label {
                    return false;
                }
                if !match_impl_domain_head(store, e1.ty, e2.ty, impl_id, bindings) {
                    return false;
                }
            }
            true
        }
        (TypeData::Record(row1), TypeData::Record(row2)) => {
            let r1 = store.record_row(*row1);
            let r2 = store.record_row(*row2);
            if r1.fields.len() != r2.fields.len() {
                return false;
            }
            for (f1, f2) in r1.fields.iter().zip(r2.fields.iter()) {
                if f1.name != f2.name {
                    return false;
                }
                if !match_impl_domain_head(store, f1.ty, f2.ty, impl_id, bindings) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

/// Outcome of checking all constraints belonging to one inherent impl domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImplConstraintResult {
    /// Every domain constraint was proven.
    Satisfied,
    /// At least one domain constraint was refuted.
    NotApplicable,
    /// The available type information is insufficient for a judgment.
    Unknown(UnknownReason),
    /// The semantic relation query could not complete soundly.
    Blocked(BlockReason),
    /// The constraint crosses an explicit dynamic boundary.
    Dynamic(DynamicBoundaryObligation),
    /// The query was cancelled.
    Cancelled,
    /// The query exhausted a bounded resource.
    BudgetExceeded(BudgetReport),
    /// The semantic relation engine reported an internal failure.
    InternalFailure(String),
}

fn contains_unresolved_applicability_type(store: &TypeStore, ty: TypeId) -> bool {
    match store.get(ty) {
        TypeData::Parameter(_) | TypeData::SelfType(_) | TypeData::Lambda(_) => true,
        TypeData::Applied { origin, arguments } => {
            contains_unresolved_applicability_type(store, *origin) || arguments.iter().any(|&argument| contains_unresolved_applicability_type(store, argument))
        }
        TypeData::ExactCase { enum_type, .. } => contains_unresolved_applicability_type(store, *enum_type),
        TypeData::Union(members) => members.iter().any(|&member| contains_unresolved_applicability_type(store, member)),
        TypeData::Tuple(elements) => elements.iter().any(|element| contains_unresolved_applicability_type(store, element.ty)),
        TypeData::Record(row) => store
            .record_row(*row)
            .fields
            .iter()
            .any(|field| contains_unresolved_applicability_type(store, field.ty)),
        TypeData::Callable(callable) => {
            callable
                .parameters
                .iter()
                .any(|parameter| contains_unresolved_applicability_type(store, parameter.ty))
                || contains_unresolved_applicability_type(store, callable.return_type)
        }
        _ => false,
    }
}

fn relation_to_impl_constraint_result(outcome: RelationOutcome) -> ImplConstraintResult {
    match outcome {
        RelationOutcome::Proven { .. } => ImplConstraintResult::Satisfied,
        RelationOutcome::Refuted(_) => ImplConstraintResult::NotApplicable,
        RelationOutcome::DynamicBoundary(obligation) => ImplConstraintResult::Dynamic(obligation),
        RelationOutcome::Blocked(reason) => ImplConstraintResult::Blocked(reason),
        RelationOutcome::Cancelled => ImplConstraintResult::Cancelled,
        RelationOutcome::BudgetExceeded(report) => ImplConstraintResult::BudgetExceeded(report),
        RelationOutcome::InternalFailure(message) => ImplConstraintResult::InternalFailure(message),
    }
}

/// Checks whether an inherent impl domain's substituted generic constraints are satisfied.
pub fn check_impl_domain_constraints(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    domain: &InherentImplDomain,
    bindings: &HashMap<TypeParameterId, TypeId>,
    ambient_constraints: &[GenericConstraint],
) -> ImplConstraintResult {
    let mut subst = TypeSubstitution::new();
    for (&p, &t) in bindings {
        subst.bind(p, t);
    }

    for constraint in domain.constraints.iter() {
        match constraint {
            GenericConstraint::Subtype { lower, upper } => {
                let lower_ty = match lower {
                    TypeTerm::Canonical(ty) => subst.apply(store, *ty),
                    _ => return ImplConstraintResult::Blocked(BlockReason::InvalidAnnotation(DiagnosticCode::AnnotationUnresolved)),
                };
                let upper_ty = match upper {
                    TypeTerm::Canonical(ty) => subst.apply(store, *ty),
                    _ => return ImplConstraintResult::Blocked(BlockReason::InvalidAnnotation(DiagnosticCode::AnnotationUnresolved)),
                };

                if lower_ty == upper_ty {
                    continue;
                }
                let mut budget = QueryBudget::default();
                let cancellation = CancellationToken::new();
                match relation_to_impl_constraint_result(crate::types::relation::check_subtype_bounded(
                    store,
                    hierarchy,
                    lower_ty,
                    upper_ty,
                    &mut budget,
                    &cancellation,
                )) {
                    ImplConstraintResult::Satisfied => continue,
                    terminal @ (ImplConstraintResult::Dynamic(_)
                    | ImplConstraintResult::Blocked(_)
                    | ImplConstraintResult::Cancelled
                    | ImplConstraintResult::BudgetExceeded(_)
                    | ImplConstraintResult::InternalFailure(_)) => return terminal,
                    ImplConstraintResult::NotApplicable | ImplConstraintResult::Unknown(_) => {}
                }
                let mut proven = false;
                if let TypeData::Parameter(_p) = store.get(lower_ty) {
                    for amb in ambient_constraints {
                        match amb {
                            GenericConstraint::Subtype {
                                lower: amb_lower,
                                upper: amb_upper,
                            } => {
                                if let (TypeTerm::Canonical(al), TypeTerm::Canonical(au)) = (amb_lower, amb_upper) {
                                    if *al == lower_ty && crate::types::relation::is_subtype(store, hierarchy, *au, upper_ty) {
                                        proven = true;
                                        break;
                                    }
                                }
                            }
                            GenericConstraint::Equivalent {
                                left: amb_left,
                                right: amb_right,
                            } => {
                                if let (TypeTerm::Canonical(al), TypeTerm::Canonical(ar)) = (amb_left, amb_right) {
                                    if *al == lower_ty && crate::types::relation::is_subtype(store, hierarchy, *ar, upper_ty) {
                                        proven = true;
                                        break;
                                    }
                                    if *ar == lower_ty && crate::types::relation::is_subtype(store, hierarchy, *al, upper_ty) {
                                        proven = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                if !proven {
                    return if contains_unresolved_applicability_type(store, lower_ty) || contains_unresolved_applicability_type(store, upper_ty) {
                        ImplConstraintResult::Unknown(UnknownReason::UnderconstrainedTypeVariable)
                    } else {
                        ImplConstraintResult::NotApplicable
                    };
                }
            }
            GenericConstraint::Equivalent { left, right } => {
                let left_ty = match left {
                    TypeTerm::Canonical(ty) => subst.apply(store, *ty),
                    _ => return ImplConstraintResult::Blocked(BlockReason::InvalidAnnotation(DiagnosticCode::AnnotationUnresolved)),
                };
                let right_ty = match right {
                    TypeTerm::Canonical(ty) => subst.apply(store, *ty),
                    _ => return ImplConstraintResult::Blocked(BlockReason::InvalidAnnotation(DiagnosticCode::AnnotationUnresolved)),
                };
                if left_ty == right_ty {
                    continue;
                }
                let mut proven = false;
                for amb in ambient_constraints {
                    if let GenericConstraint::Equivalent { left: al, right: ar } = amb {
                        if let (TypeTerm::Canonical(l), TypeTerm::Canonical(r)) = (al, ar) {
                            if (*l == left_ty && *r == right_ty) || (*l == right_ty && *r == left_ty) {
                                proven = true;
                                break;
                            }
                        }
                    }
                }
                if !proven {
                    return if contains_unresolved_applicability_type(store, left_ty) || contains_unresolved_applicability_type(store, right_ty) {
                        ImplConstraintResult::Unknown(UnknownReason::UnderconstrainedTypeVariable)
                    } else {
                        ImplConstraintResult::NotApplicable
                    };
                }
            }
        }
    }
    ImplConstraintResult::Satisfied
}

/// Checks applicability of an inherent impl domain to a receiver and owner view.
pub fn check_impl_domain_applicability(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    domain: &InherentImplDomain,
    receiver: TypeId,
    owner_view: TypeId,
    ambient_constraints: &[GenericConstraint],
) -> ImplApplicabilityResult {
    let mut bindings = HashMap::new();
    if !match_impl_domain_head(store, domain.head_type, owner_view, &domain.impl_id, &mut bindings) {
        return ImplApplicabilityResult::NotApplicable;
    }

    match check_impl_domain_constraints(store, hierarchy, domain, &bindings, ambient_constraints) {
        ImplConstraintResult::Satisfied => {}
        ImplConstraintResult::NotApplicable => return ImplApplicabilityResult::NotApplicable,
        ImplConstraintResult::Unknown(reason) => return ImplApplicabilityResult::Unknown(reason),
        ImplConstraintResult::Blocked(reason) => return ImplApplicabilityResult::Blocked(reason),
        ImplConstraintResult::Dynamic(obligation) => return ImplApplicabilityResult::Dynamic(obligation),
        ImplConstraintResult::Cancelled => return ImplApplicabilityResult::Cancelled,
        ImplConstraintResult::BudgetExceeded(report) => return ImplApplicabilityResult::BudgetExceeded(report),
        ImplConstraintResult::InternalFailure(message) => return ImplApplicabilityResult::InternalFailure(message),
    }

    let mut env = TypeEnvironment::new();
    for (&p, &t) in &bindings {
        env.bind_param(p, t);
    }
    env.bind_self(owner_view);

    ImplApplicabilityResult::Applicable(InherentImplSpecialization {
        impl_id: domain.impl_id.clone(),
        receiver,
        owner_view,
        bindings,
        environment: env,
    })
}

/// Resolves the target of an `impl` statement.
pub fn resolve_inherent_impl_target(
    ctx: &mut CheckingContext<'_>,
    impl_id: &ImplId,
    impl_def: &ImplDef,
) -> Result<ResolvedInherentImplTarget, Vec<SemanticDiagnostic>> {
    let mut diagnostics = Vec::new();
    let source = SemanticSourceSpan::new(ctx.current_module.clone(), impl_def.range);

    // 1. Build impl-owned generic parameters
    let mut impl_type_parameter_ids = Vec::new();
    let mut impl_type_parameter_map: HashMap<String, TypeLevelBinding> = HashMap::new();

    for (index, param_syntax) in impl_def.generic_parameters.iter().enumerate() {
        let param_data = TypeParameterData::new(TypeParameterOwner::Impl(impl_id.clone()), index as u32, param_syntax.name.clone(), KindId::TYPE);
        let param_id = ctx.store.intern_type_parameter(param_data);
        impl_type_parameter_ids.push(param_id);
        let binding = type_level_binding_for_parameter(ctx.store, param_id);
        impl_type_parameter_map.insert(param_syntax.name.clone(), binding);
    }

    // 2. Resolve where clause constraints
    let parent_resolver = ctx.resolver.clone();
    let impl_resolver = ScopedTypeResolver {
        parent: &parent_resolver,
        type_parameters: impl_type_parameter_map.clone(),
    };
    let formation_site = TypeFormationSite::module(ctx.current_module.clone());

    let mut constraints = Vec::new();
    if let Some(ref where_clause) = impl_def.where_clause {
        for c in &where_clause.constraints {
            match c {
                phalcom_ast::ast::GenericConstraintSyntax::Subtype { lower, upper, range: _ } => {
                    let l_k = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, lower, &mut diagnostics);
                    let u_k = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, upper, &mut diagnostics);
                    if let (TypeKnowledge::Known(l_ev), TypeKnowledge::Known(u_ev)) = (l_k, u_k) {
                        constraints.push(GenericConstraint::Subtype {
                            lower: TypeTerm::Canonical(l_ev.ty()),
                            upper: TypeTerm::Canonical(u_ev.ty()),
                        });
                    }
                }
                phalcom_ast::ast::GenericConstraintSyntax::Equivalent { left, right, range: _ } => {
                    let l_k = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, left, &mut diagnostics);
                    let r_k = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, right, &mut diagnostics);
                    if let (TypeKnowledge::Known(l_ev), TypeKnowledge::Known(r_ev)) = (l_k, r_k) {
                        constraints.push(GenericConstraint::Equivalent {
                            left: TypeTerm::Canonical(l_ev.ty()),
                            right: TypeTerm::Canonical(r_ev.ty()),
                        });
                    }
                }
                phalcom_ast::ast::GenericConstraintSyntax::Invalid { message, range } => {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        message.clone(),
                        *range,
                    ));
                }
            }
        }
    }

    let impl_generic_signature = if impl_type_parameter_ids.is_empty() && constraints.is_empty() {
        None
    } else {
        Some(GenericSignature::with_constraints(
            TypeParameterOwner::Impl(impl_id.clone()),
            impl_type_parameter_ids.clone().into_boxed_slice(),
            constraints.clone().into_boxed_slice(),
        ))
    };

    // 3. Resolve target type annotation
    // Check for exact enum case target syntax
    if let TypeAnnotationExpr::ExactEnumCase {
        enum_target,
        variant_name,
        variant_name_range,
        generic_arguments,
        payload_shape,
        range,
    } = &impl_def.target.expr
    {
        let knowledge = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, enum_target, &mut diagnostics);
        let TypeKnowledge::Known(evidence) = knowledge else {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplTargetNotNominal,
                "inherent impl target enum must be a nominal type declaration",
                enum_target.range,
            ));
            return Err(diagnostics);
        };
        let enum_ty = evidence.ty();
        let (enum_decl, enum_args) = match ctx.store.get(enum_ty).clone() {
            TypeData::Nominal { declaration } => (declaration, Vec::new()),
            TypeData::Applied { origin, arguments } => {
                let orig_decl = match ctx.store.get(origin) {
                    TypeData::Nominal { declaration } => declaration.clone(),
                    _ => {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::ImplTargetNotNominal,
                            "inherent impl target enum must be a nominal type declaration",
                            enum_target.range,
                        ));
                        return Err(diagnostics);
                    }
                };
                (orig_decl, arguments.into_vec())
            }
            _ => {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplTargetNotNominal,
                    "inherent impl target enum must be a nominal type declaration",
                    enum_target.range,
                ));
                return Err(diagnostics);
            }
        };

        // Same-module check
        if enum_decl.module != impl_id.module {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplForeignTarget,
                format!(
                    "cannot define inherent impl for foreign declaration `{}` in module `{}`",
                    enum_decl.name, enum_decl.module
                ),
                enum_target.range,
            ));
            return Err(diagnostics);
        }

        // Type alias check
        if ctx.resolver.resolve_alias_form(&enum_decl).is_some() {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplTargetTypeAlias,
                format!("cannot define inherent impl for type alias `{}`", enum_decl.name),
                enum_target.range,
            ));
            return Err(diagnostics);
        }

        // Must be an enum declaration
        let Some(enum_info) = ctx.enum_info(&enum_decl) else {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplTargetNotNominal,
                format!("declaration `{}` is not an enum declaration", enum_decl.name),
                enum_target.range,
            ));
            return Err(diagnostics);
        };
        let enum_info = enum_info.clone();

        // Derive selector and VariantId
        let selector = phalcom_ast::selector::selector_from_exact_case_target(variant_name, payload_shape.as_ref());
        let variant_id = VariantId::new(enum_decl.clone(), selector.clone());

        // Check if variant exists
        let variant_info = ctx.variant_info(&variant_id).cloned();
        if variant_info.is_none() && !enum_info.variants.contains(&variant_id) {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::EnumRequirementMissing,
                format!("enum `{}` has no variant matching `{}`", enum_decl.name, selector.encode()),
                *variant_name_range,
            ));
            return Err(diagnostics);
        }

        let enum_decl_sig = ctx.declaration_generic_signature(&enum_decl);
        let empty_params: [TypeParameterId; 0] = [];
        let decl_params = enum_decl_sig.as_ref().map_or(&empty_params[..], |s| &s.parameters);
        let variant_constructor_sig = variant_info
            .as_ref()
            .and_then(|v| v.constructor.as_ref().and_then(|c| c.generic_signature.as_ref()));
        let variant_params = variant_constructor_sig.map_or(&empty_params[..], |s| &s.parameters);

        if enum_args.len() != decl_params.len() {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplSpecializedTargetUnsupported,
                format!(
                    "generic arity mismatch for enum `{}` in exact-case target: expected {} arguments, got {}",
                    enum_decl.name,
                    decl_params.len(),
                    enum_args.len()
                ),
                enum_target.range,
            ));
            return Err(diagnostics);
        }

        if generic_arguments.len() != variant_params.len() {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplSpecializedTargetUnsupported,
                format!(
                    "generic arity mismatch for variant `{}` in exact-case target: expected {} arguments, got {}",
                    variant_id.selector.encode(),
                    variant_params.len(),
                    generic_arguments.len()
                ),
                *range,
            ));
            return Err(diagnostics);
        }

        // Collect used impl params in target head
        let mut used_impl_params = HashSet::new();
        for &arg_ty in &enum_args {
            collect_impl_params_in_type(ctx.store, arg_ty, impl_id, &mut used_impl_params);
        }

        let mut var_arg_types = Vec::new();
        for gen_arg_syntax in generic_arguments.iter() {
            let gen_arg_res = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, gen_arg_syntax, &mut diagnostics);
            let TypeKnowledge::Known(arg_ev) = gen_arg_res else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplSpecializedTargetUnsupported,
                    "invalid generic argument in exact case impl target",
                    gen_arg_syntax.range,
                ));
                return Err(diagnostics);
            };
            var_arg_types.push(arg_ev.ty());
            collect_impl_params_in_type(ctx.store, arg_ev.ty(), impl_id, &mut used_impl_params);
        }

        // Check for unused impl parameters
        for &impl_param in &impl_type_parameter_ids {
            if !used_impl_params.contains(&impl_param) {
                let p_name = &ctx.store.type_parameter(impl_param).name;
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplUnusedTypeParameter,
                    format!("type parameter `{}` is not used in inherent impl target", p_name),
                    impl_def.range,
                ));
                return Err(diagnostics);
            }
        }

        let total_target_params = decl_params.len() + variant_params.len();
        let target_type = variant_info.as_ref().map_or(enum_ty, |v| v.exact_case_template);

        // Check if covering bijection or conditional
        let is_covering_bijection = constraints.is_empty()
            && total_target_params == impl_type_parameter_ids.len()
            && enum_args.iter().all(|&a| matches!(ctx.store.get(a), TypeData::Parameter(p) if matches!(&ctx.store.type_parameter(*p).owner, TypeParameterOwner::Impl(id) if id == impl_id)))
            && var_arg_types.iter().all(|&a| matches!(ctx.store.get(a), TypeData::Parameter(p) if matches!(&ctx.store.type_parameter(*p).owner, TypeParameterOwner::Impl(id) if id == impl_id)))
            && used_impl_params.len() == total_target_params;

        let applicability = if total_target_params == 0 && constraints.is_empty() {
            InherentImplApplicability::Unconditional
        } else if is_covering_bijection {
            let mut impl_to_decl = HashMap::new();
            let mut decl_to_impl = HashMap::new();
            for (i, &arg_ty) in enum_args.iter().enumerate() {
                if let TypeData::Parameter(p_id) = ctx.store.get(arg_ty) {
                    impl_to_decl.insert(*p_id, decl_params[i]);
                    decl_to_impl.insert(decl_params[i], *p_id);
                }
            }
            for (i, &arg_ty) in var_arg_types.iter().enumerate() {
                if let TypeData::Parameter(p_id) = ctx.store.get(arg_ty) {
                    impl_to_decl.insert(*p_id, variant_params[i]);
                    decl_to_impl.insert(variant_params[i], *p_id);
                }
            }
            InherentImplApplicability::Covering(CoveringImplSubstitution::new(impl_to_decl, decl_to_impl))
        } else {
            let domain = Arc::new(InherentImplDomain {
                impl_id: impl_id.clone(),
                target: InherentImplTarget::ExactEnumCase(variant_id.clone()),
                head_type: target_type,
                generic_signature: impl_generic_signature.clone(),
                constraints: constraints.into_boxed_slice(),
            });
            InherentImplApplicability::Conditional(domain)
        };

        return Ok(ResolvedInherentImplTarget {
            id: impl_id.clone(),
            target: InherentImplTarget::ExactEnumCase(variant_id),
            target_type,
            generic_signature: impl_generic_signature,
            applicability,
            source,
            diagnostics: diagnostics.into_boxed_slice(),
        });
    }

    if let TypeAnnotationExpr::Reference(sym) = &impl_def.target.expr {
        if sym.members.is_empty() {
            let sym_decl = DeclarationId::new(ctx.current_module.clone(), sym.root.clone().into());
            if ctx.resolver.resolve_alias_form(&sym_decl).is_some() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplTargetTypeAlias,
                    format!("cannot define inherent impl for type alias `{}`", sym.root),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }
        }
    }

    let knowledge = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, &impl_def.target, &mut diagnostics);

    let TypeKnowledge::Known(evidence) = knowledge else {
        diagnostics.push(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::ImplTargetNotNominal,
            "inherent impl target must be a nominal type declaration (class, data, or enum)",
            impl_def.target.range,
        ));
        return Err(diagnostics);
    };

    let target_type = evidence.ty();

    // 4. Validate nominal declaration target and compute covering or conditional applicability
    let (target_decl, applicability) = match ctx.store.get(target_type).clone() {
        TypeData::Nominal { declaration } => {
            // Check same module rule
            if declaration.module != impl_id.module {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplForeignTarget,
                    format!(
                        "cannot define inherent impl for foreign declaration `{}` in module `{}`",
                        declaration.name, declaration.module
                    ),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }

            // Check type alias
            if ctx.resolver.resolve_alias_form(&declaration).is_some() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplTargetTypeAlias,
                    format!("cannot define inherent impl for type alias `{}`", declaration.name),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }

            let decl_sig = ctx.declaration_generic_signature(&declaration);
            let decl_param_count = decl_sig.map_or(0, |s| s.parameters.len());

            if decl_param_count == 0 {
                if !impl_type_parameter_ids.is_empty() {
                    for &impl_param in &impl_type_parameter_ids {
                        let p_name = &ctx.store.type_parameter(impl_param).name;
                        diagnostics.push(SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::ImplUnusedTypeParameter,
                            format!("type parameter `{}` is not used in inherent impl target", p_name),
                            impl_def.range,
                        ));
                    }
                    return Err(diagnostics);
                }
                if constraints.is_empty() {
                    (declaration, InherentImplApplicability::Unconditional)
                } else {
                    let domain = Arc::new(InherentImplDomain {
                        impl_id: impl_id.clone(),
                        target: InherentImplTarget::Declaration(declaration.clone()),
                        head_type: target_type,
                        generic_signature: impl_generic_signature.clone(),
                        constraints: constraints.into_boxed_slice(),
                    });
                    (declaration, InherentImplApplicability::Conditional(domain))
                }
            } else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplSpecializedTargetUnsupported,
                    format!("unapplied generic declaration `{}` in inherent impl target is invalid", declaration.name),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }
        }
        TypeData::Applied { origin, arguments } => {
            let orig_decl = match ctx.store.get(origin) {
                TypeData::Nominal { declaration } => declaration.clone(),
                _ => {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ImplTargetNotNominal,
                        "inherent impl target must be a nominal type declaration",
                        impl_def.target.range,
                    ));
                    return Err(diagnostics);
                }
            };

            // Check same module rule
            if orig_decl.module != impl_id.module {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplForeignTarget,
                    format!(
                        "cannot define inherent impl for foreign declaration `{}` in module `{}`",
                        orig_decl.name, orig_decl.module
                    ),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }

            // Check type alias
            if ctx.resolver.resolve_alias_form(&orig_decl).is_some() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplTargetTypeAlias,
                    format!("cannot define inherent impl for type alias `{}`", orig_decl.name),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }

            let decl_sig = ctx.declaration_generic_signature(&orig_decl);
            let Some(decl_sig) = decl_sig else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplSpecializedTargetUnsupported,
                    format!("declaration `{}` is not generic but received type arguments", orig_decl.name),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            };

            let decl_params = decl_sig.parameters.clone();

            if arguments.len() != decl_params.len() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplSpecializedTargetUnsupported,
                    format!(
                        "generic arity mismatch for inherent impl on `{}`: target has {} parameters, got {}",
                        orig_decl.name,
                        decl_params.len(),
                        arguments.len()
                    ),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }

            // Collect used impl params across all arguments
            let mut used_impl_params = HashSet::new();
            for &arg_ty in arguments.iter() {
                collect_impl_params_in_type(ctx.store, arg_ty, impl_id, &mut used_impl_params);
            }

            // Check for unused impl parameters
            for &impl_param in &impl_type_parameter_ids {
                if !used_impl_params.contains(&impl_param) {
                    let p_name = &ctx.store.type_parameter(impl_param).name;
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ImplUnusedTypeParameter,
                        format!("type parameter `{}` is not used in inherent impl target", p_name),
                        impl_def.range,
                    ));
                    return Err(diagnostics);
                }
            }

            // Check if covering bijection
            let mut is_covering_bijection = constraints.is_empty() && arguments.len() == impl_type_parameter_ids.len();
            let mut impl_to_decl = HashMap::new();
            let mut decl_to_impl = HashMap::new();
            let mut seen_impl_params = HashSet::new();

            if is_covering_bijection {
                for (i, &arg_ty) in arguments.iter().enumerate() {
                    let decl_param = decl_params[i];
                    if let TypeData::Parameter(p_id) = ctx.store.get(arg_ty) {
                        let p_data = ctx.store.type_parameter(*p_id);
                        if let TypeParameterOwner::Impl(ref owner_impl) = p_data.owner {
                            if owner_impl == impl_id && seen_impl_params.insert(*p_id) {
                                impl_to_decl.insert(*p_id, decl_param);
                                decl_to_impl.insert(decl_param, *p_id);
                                continue;
                            }
                        }
                    }
                    is_covering_bijection = false;
                    break;
                }
            }

            if is_covering_bijection {
                let covering = CoveringImplSubstitution::new(impl_to_decl, decl_to_impl);
                (orig_decl, InherentImplApplicability::Covering(covering))
            } else {
                let domain = Arc::new(InherentImplDomain {
                    impl_id: impl_id.clone(),
                    target: InherentImplTarget::Declaration(orig_decl.clone()),
                    head_type: target_type,
                    generic_signature: impl_generic_signature.clone(),
                    constraints: constraints.into_boxed_slice(),
                });
                (orig_decl, InherentImplApplicability::Conditional(domain))
            }
        }
        _ => {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplTargetNotNominal,
                "inherent impl target must be a nominal type declaration (class, data, or enum)",
                impl_def.target.range,
            ));
            return Err(diagnostics);
        }
    };

    Ok(ResolvedInherentImplTarget {
        id: impl_id.clone(),
        target: InherentImplTarget::Declaration(target_decl),
        target_type,
        generic_signature: impl_generic_signature,
        applicability,
        source,
        diagnostics: diagnostics.into_boxed_slice(),
    })
}

/// Applies a type substitution to all types referenced in a `CallableSemanticSignature`.
pub fn apply_covering_to_signature(store: &mut TypeStore, subst: &TypeSubstitution, mut signature: CallableSemanticSignature) -> CallableSemanticSignature {
    if subst.is_empty() {
        return signature;
    }

    let mut new_params = Vec::with_capacity(signature.parameters.len());
    for mut param in signature.parameters.into_vec() {
        if let DeclaredTypeState::Known(TypeTerm::Canonical(ty)) = param.declared_type.state {
            let new_ty = subst.apply(store, ty);
            param.declared_type.state = DeclaredTypeState::Known(TypeTerm::Canonical(new_ty));
        }
        new_params.push(param);
    }
    signature.parameters = new_params.into_boxed_slice();

    if let DeclaredTypeState::Known(TypeTerm::Canonical(ret_ty)) = signature.declared_return.state {
        let new_ret_ty = subst.apply(store, ret_ty);
        signature.declared_return.state = DeclaredTypeState::Known(TypeTerm::Canonical(new_ret_ty));
    }

    signature
}

/// Checks if a type contains any parameters owned by `impl_id`.
pub fn type_contains_impl_param(store: &TypeStore, ty: TypeId, impl_id: &ImplId) -> bool {
    match store.get(ty) {
        TypeData::Parameter(p) => {
            let data = store.type_parameter(*p);
            matches!(&data.owner, TypeParameterOwner::Impl(id) if id == impl_id)
        }
        TypeData::Applied { origin, arguments } => {
            type_contains_impl_param(store, *origin, impl_id) || arguments.iter().any(|&arg| type_contains_impl_param(store, arg, impl_id))
        }
        TypeData::ExactCase { enum_type, .. } => type_contains_impl_param(store, *enum_type, impl_id),
        TypeData::Union(members) => members.iter().any(|&m| type_contains_impl_param(store, m, impl_id)),
        TypeData::Tuple(elements) => elements.iter().any(|e| type_contains_impl_param(store, e.ty, impl_id)),
        TypeData::Record(row) => {
            let row_data = store.record_row(*row);
            row_data.fields.iter().any(|f| type_contains_impl_param(store, f.ty, impl_id))
        }
        TypeData::Callable(c) => {
            c.parameters.iter().any(|p| type_contains_impl_param(store, p.ty, impl_id)) || type_contains_impl_param(store, c.return_type, impl_id)
        }
        _ => false,
    }
}

/// Builds the canonical `InherentImplContribution` from an `ImplDef`.
pub fn build_inherent_impl_contribution(ctx: &mut CheckingContext<'_>, impl_id: &ImplId, impl_def: &ImplDef) -> InherentImplContribution {
    let mut diagnostics = Vec::new();
    let source = SemanticSourceSpan::new(ctx.current_module.clone(), impl_def.range);

    let resolved_target = match resolve_inherent_impl_target(ctx, impl_id, impl_def) {
        Ok(target) => {
            diagnostics.extend(target.diagnostics.clone().into_vec());
            target
        }
        Err(diags) => {
            diagnostics.extend(diags);
            return InherentImplContribution {
                id: impl_id.clone(),
                target: InherentImplTarget::Declaration(DeclarationId::new(ctx.current_module.clone(), "_".into())),
                generic_signature: None,
                covering: None,
                is_conditionally_applicable: false,
                domain: None,
                members: Box::new([]),
                source,
                diagnostics: diagnostics.into_boxed_slice(),
            };
        }
    };

    let target_owner = resolved_target.target.to_callable_owner();
    let is_conditionally_applicable = matches!(&resolved_target.applicability, InherentImplApplicability::Conditional(_));
    let is_exact_case = matches!(resolved_target.target, InherentImplTarget::ExactEnumCase(_));
    let is_enum_root = match &resolved_target.target {
        InherentImplTarget::Declaration(decl_id) => ctx.enum_info(decl_id).is_some(),
        InherentImplTarget::ExactEnumCase(_) => false,
    };

    // Build the impl-scoped resolver
    let mut impl_type_parameter_map: HashMap<String, TypeLevelBinding> = HashMap::new();
    if let Some(ref sig) = resolved_target.generic_signature {
        for &param_id in sig.parameters.iter() {
            let name = ctx.store.type_parameter(param_id).name.to_string();
            let binding = type_level_binding_for_parameter(ctx.store, param_id);
            impl_type_parameter_map.insert(name, binding);
        }
    }

    let parent_resolver = ctx.resolver.clone();
    let impl_resolver = ScopedTypeResolver {
        parent: &parent_resolver,
        type_parameters: impl_type_parameter_map,
    };

    let covering_subst = match &resolved_target.applicability {
        InherentImplApplicability::Covering(cov) => Some(cov.to_type_substitution(ctx.store)),
        InherentImplApplicability::Unconditional | InherentImplApplicability::Conditional(_) => None,
    };

    let domain = match &resolved_target.applicability {
        InherentImplApplicability::Conditional(d) => Some(d.clone()),
        InherentImplApplicability::Unconditional | InherentImplApplicability::Covering(_) if is_exact_case => Some(Arc::new(InherentImplDomain {
            impl_id: impl_id.clone(),
            target: resolved_target.target.clone(),
            head_type: resolved_target.target_type,
            generic_signature: resolved_target.generic_signature.clone(),
            constraints: Box::new([]),
        })),
        _ => None,
    };

    let mut members = Vec::new();

    for (source_member_idx, member) in impl_def.members.iter().enumerate() {
        let syntax = CallableSyntaxRef::from(member);
        let visibility = behavior_member_visibility(member);

        // Reject constructors in impl
        let is_constructor = match member {
            BehaviorMember::Method(m) => m.is_constructor || m.attributes.iter().any(|a| a.name == "constructor"),
            _ => false,
        };
        if is_constructor {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplConstructorUnsupported,
                "constructor lifecycle methods cannot be declared in inherent impl blocks",
                syntax.range(),
            ));
            continue;
        }

        let is_class_side = syntax.attributes().iter().any(|attr| attr.name == "class")
            || match member {
                BehaviorMember::Method(m) => m.is_static,
                BehaviorMember::Getter(g) => g.is_static,
                BehaviorMember::Setter(s) => s.is_static,
                BehaviorMember::Index(_) => false,
            };

        if is_exact_case {
            if is_class_side {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::EnumCaseStaticBehaviorUnsupported,
                    "case-local behavior on exact enum cases cannot be class-side",
                    syntax.range(),
                ));
                continue;
            }
            if !syntax.has_body() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::EnumCaseDeclarationOnlyBehavior,
                    "case-local behavior on exact enum cases must have an executable body",
                    syntax.range(),
                ));
                continue;
            }
        }

        let is_requirement = if is_enum_root {
            if !syntax.has_body() {
                if is_class_side {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::EnumCaseStaticBehaviorUnsupported,
                        "class-side requirement is not supported on enums",
                        syntax.range(),
                    ));
                    continue;
                }
                true
            } else {
                false
            }
        } else if !is_exact_case {
            if !syntax.has_body() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplBodylessMemberUnsupported,
                    "bodyless member declarations in inherent impl blocks are not supported",
                    syntax.range(),
                ));
                continue;
            }
            false
        } else {
            false
        };

        let side = if is_class_side { DispatchSide::Class } else { DispatchSide::Instance };

        let Some(raw_sig) = semantic_signature_for_syntax_with_resolver(ctx, &target_owner, &impl_resolver, syntax, side) else {
            continue;
        };

        // Canonicalize signature into target declaration parameter space for covering impls
        let final_sig = if let Some(ref subst) = covering_subst {
            apply_covering_to_signature(ctx.store, subst, raw_sig)
        } else {
            raw_sig
        };

        // Invariant assertion: published unconditional surface signatures contain no free target-head TypeParameterOwner::Impl(current_impl)
        if domain.is_none() {
            for param in final_sig.parameters.iter() {
                if let DeclaredTypeState::Known(TypeTerm::Canonical(ty)) = param.declared_type.state {
                    debug_assert!(
                        !type_contains_impl_param(ctx.store, ty, impl_id),
                        "published signature parameter contains free impl parameter"
                    );
                }
            }
            if let DeclaredTypeState::Known(TypeTerm::Canonical(ret_ty)) = final_sig.declared_return.state {
                debug_assert!(
                    !type_contains_impl_param(ctx.store, ret_ty, impl_id),
                    "published signature return type contains free impl parameter"
                );
            }
        }

        members.push(InherentMemberContribution {
            callable: final_sig.callable.clone(),
            signature: final_sig,
            visibility,
            source_member: source_member_idx,
            is_requirement,
        });
    }

    InherentImplContribution {
        id: impl_id.clone(),
        target: resolved_target.target,
        generic_signature: resolved_target.generic_signature,
        covering: match resolved_target.applicability {
            InherentImplApplicability::Covering(cov) => Some(cov),
            _ => None,
        },
        is_conditionally_applicable,
        domain,
        members: members.into_boxed_slice(),
        source,
        diagnostics: diagnostics.into_boxed_slice(),
    }
}

#[cfg(test)]
mod completeness_tests {
    use super::{ConformanceCompleteness, ConformanceResolution, RequirementFailure};
    use crate::identity::{DeclarationId, ImplId, ImplLocalId, ModuleId};
    use crate::types::evidence::UnknownReason;
    use crate::types::outcome::{BlockReason, BudgetKind, BudgetReport, DynamicBoundaryObligation};
    use phalcom_modules::identity::{ModulePath, ResolvedProjectId};

    fn impl_id() -> ImplId {
        ImplId::new(ModuleId::resolved(ResolvedProjectId::from_raw(9), ModulePath::root()), ImplLocalId(4))
    }

    #[test]
    fn completeness_maps_each_non_proven_state_without_collapsing_it() {
        let id = impl_id();
        let failure = RequirementFailure {
            requirement: crate::traits::TraitRequirementId::new(
                DeclarationId::new(id.module.clone(), "Trait".into()),
                phalcom_common::selector::Selector::getter("value").expect("selector"),
                crate::identity::DispatchSide::Instance,
            ),
            reason: "missing witness".into(),
        };
        let cases: &[(ConformanceCompleteness, fn(ConformanceResolution) -> bool)] = &[
            (
                ConformanceCompleteness::Incomplete {
                    failures: vec![failure].into_boxed_slice(),
                },
                |r| matches!(r, ConformanceResolution::Incomplete(_)),
            ),
            (ConformanceCompleteness::Unknown(UnknownReason::NoTypeEvidence), |r| {
                matches!(r, ConformanceResolution::Unknown(_))
            }),
            (ConformanceCompleteness::Blocked(BlockReason::RecursiveFixpoint), |r| {
                matches!(r, ConformanceResolution::Blocked(_))
            }),
            (ConformanceCompleteness::Dynamic(DynamicBoundaryObligation { reason: "dynamic".into() }), |r| {
                matches!(r, ConformanceResolution::Dynamic(_))
            }),
            (ConformanceCompleteness::Cancelled, |r| matches!(r, ConformanceResolution::Cancelled)),
            (ConformanceCompleteness::BudgetExceeded(BudgetReport::new(BudgetKind::Steps, 1, 2)), |r| {
                matches!(r, ConformanceResolution::BudgetExceeded(_))
            }),
            (ConformanceCompleteness::InternalFailure("internal".into()), |r| {
                matches!(r, ConformanceResolution::InternalFailure(_))
            }),
        ];
        for (completeness, predicate) in cases {
            assert!(predicate(completeness.clone().into_resolution(id.clone()).expect("non-complete resolution")));
        }
        assert!(ConformanceCompleteness::Complete.into_resolution(id).is_none());
    }

    #[test]
    fn terminal_state_retains_the_stronger_query_outcome() {
        let mut state = Some(ConformanceCompleteness::Unknown(UnknownReason::NoTypeEvidence));
        super::retain_stronger_terminal_state(&mut state, ConformanceCompleteness::Blocked(BlockReason::RecursiveFixpoint));
        assert!(matches!(state, Some(ConformanceCompleteness::Blocked(_))));
    }
}
