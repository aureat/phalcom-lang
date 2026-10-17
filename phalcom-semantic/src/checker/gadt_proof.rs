//! GADT branch proof engine and equality refinement (Part 05.1).

use crate::enum_semantics::VariantInfo;
use crate::identity::DeclarationId;
use crate::match_semantics::BranchProofEnvironment;
use crate::types::CaseInstantiation;
use crate::types::constraint::TypeConstraint;
use crate::types::id::{TypeId, TypeParameterId};
use crate::types::relation::TypeHierarchy;
use crate::types::rigid::{LocalConstraint, LocalType};
use crate::types::row::RecordRowTail;
use crate::types::store::{
    AnonymousTupleRemainder, CallableParameterType, CallableType, PackTypeElement, StaticArgument, TupleTypeElement, TypeData, TypeStore,
};
use crate::types::substitution::TypeSubstitution;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Result of evaluating GADT specialization and reachability for a variant case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GadtProofResult {
    /// Case is reachable with the given branch proof environment and specialized exact case.
    Reachable {
        /// Equalities and parameter substitutions established by observing the case.
        proof: BranchProofEnvironment,
        /// Exact case type specialized by the solved branch equalities.
        exact_case: TypeId,
    },
    /// Case is contradictory / impossible under the scrutinee type.
    Refuted,
    /// The supported equality machinery cannot establish reachability or contradiction.
    Blocked(crate::types::outcome::BlockReason),
}

/// Result of merging proof environments from two overlapping semantic spaces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProofMerge {
    Compatible(BranchProofEnvironment),
    Contradictory,
    Blocked(crate::types::outcome::BlockReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LocalCaseProof {
    Reachable {
        bindings: BTreeMap<TypeParameterId, LocalType>,
        equalities: Box<[LocalConstraint]>,
    },
    Refuted,
    Blocked(crate::types::outcome::BlockReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum LocalProofFailure {
    Refuted,
    Blocked(crate::types::outcome::BlockReason),
}

type LocalUnification = Result<(), LocalProofFailure>;

fn local_proof_blocked() -> LocalProofFailure {
    LocalProofFailure::Blocked(crate::types::outcome::BlockReason::UnknownType(
        crate::types::evidence::UnknownReason::UnderconstrainedTypeVariable,
    ))
}

pub(crate) fn local_proof_control(control: &crate::checker::context::CheckerControl) -> Result<(), crate::types::outcome::BlockReason> {
    if control.is_cancelled() {
        return Err(crate::types::outcome::BlockReason::UnknownType(
            crate::types::evidence::UnknownReason::InferenceCancelled,
        ));
    }
    control.charge_step().map_err(crate::types::outcome::BlockReason::BudgetExceeded)
}

/// Refines a canonical scrutinee view against one freshly opened constructor
/// result. Flexible declaration parameters may be mapped to local terms, but
/// rigid leaves are opaque and are never rewritten.
#[allow(dead_code)]
pub(crate) fn solve_local_case_proof(store: &mut TypeStore, proof: &BranchProofEnvironment, expected_ty: TypeId, case: &CaseInstantiation) -> LocalCaseProof {
    solve_local_case_proof_against_local(
        store,
        proof,
        &LocalType::Canonical(expected_ty),
        case,
        &crate::checker::context::CheckerControl::default(),
    )
}

/// Refines an arbitrary local subject term against one freshly opened constructor
/// result. Preserves parent rigids while solving constructor-local equalities.
pub(crate) fn solve_local_case_proof_against_local(
    store: &mut TypeStore,
    proof: &BranchProofEnvironment,
    expected: &LocalType,
    case: &CaseInstantiation,
    control: &crate::checker::context::CheckerControl,
) -> LocalCaseProof {
    if let Err(reason) = local_proof_control(control) {
        return LocalCaseProof::Blocked(reason);
    }
    if !case.is_local() && matches!(expected, LocalType::Canonical(_)) && proof.local_bindings.is_empty() {
        return LocalCaseProof::Reachable {
            bindings: BTreeMap::new(),
            equalities: Box::new([]),
        };
    }

    let specialized_expected = apply_branch_proof_to_local(store, proof, expected);
    let (expected_term, exact_case_observation) = unpack_expected_local_term(store, specialized_expected, case);

    let mut bindings = BTreeMap::new();
    if let Err(failure) = unify_local_types(
        store,
        case,
        &case.result_type,
        &expected_term,
        &mut bindings,
        exact_case_observation,
        control,
        0,
    ) {
        return match failure {
            LocalProofFailure::Refuted => LocalCaseProof::Refuted,
            LocalProofFailure::Blocked(reason) => LocalCaseProof::Blocked(reason),
        };
    }

    let mut equalities = case.constraints.to_vec();
    equalities.push(LocalConstraint::Equivalent {
        left: case.result_type.clone(),
        right: expected_term,
    });
    LocalCaseProof::Reachable {
        bindings,
        equalities: equalities.into_boxed_slice(),
    }
}

fn unpack_expected_local_term(store: &TypeStore, expected: LocalType, case: &CaseInstantiation) -> (LocalType, bool) {
    match expected {
        LocalType::Canonical(ty) => {
            let exact_case_observation = matches!(store.get(ty), TypeData::ExactCase { .. });
            let unpacked = match store.get(ty).clone() {
                TypeData::ExactCase { variant, enum_type } if store.variant_identity(variant) == &case.variant => {
                    LocalType::from_canonical(store, enum_type, &case.replacements())
                }
                _ => LocalType::from_canonical(store, ty, &case.replacements()),
            };
            (unpacked, exact_case_observation)
        }
        LocalType::ExactCase { variant, enum_type } => {
            if variant == case.variant {
                (*enum_type, true)
            } else {
                (LocalType::ExactCase { variant, enum_type }, true)
            }
        }
        other => (other, false),
    }
}

/// Applies branch proof substitutions to a LocalType without materializing rigids.
pub(crate) fn apply_branch_proof_to_local(store: &mut TypeStore, proof: &BranchProofEnvironment, local: &LocalType) -> LocalType {
    if proof.bindings.is_empty() && proof.local_bindings.is_empty() {
        return local.clone();
    }
    match local {
        LocalType::Canonical(ty) => {
            let specialized = apply_branch_proof(store, proof, *ty);
            if proof.local_bindings.is_empty() {
                LocalType::Canonical(specialized)
            } else {
                let replacements: HashMap<_, _> = proof.local_bindings.iter().map(|(k, v)| (*k, v.clone())).collect();
                LocalType::from_canonical(store, specialized, &replacements)
            }
        }
        LocalType::Rigid(id) => LocalType::Rigid(*id),
        LocalType::Applied { origin, arguments } => LocalType::Applied {
            origin: Box::new(apply_branch_proof_to_local(store, proof, origin)),
            arguments: arguments.iter().map(|arg| apply_branch_proof_to_local(store, proof, arg)).collect(),
        },
        LocalType::ExactCase { variant, enum_type } => LocalType::ExactCase {
            variant: variant.clone(),
            enum_type: Box::new(apply_branch_proof_to_local(store, proof, enum_type)),
        },
        LocalType::Union(members) => LocalType::Union(members.iter().map(|m| apply_branch_proof_to_local(store, proof, m)).collect()),
        LocalType::Tuple(elements) => LocalType::Tuple(
            elements
                .iter()
                .map(|el| crate::types::rigid::LocalTupleElement {
                    label: el.label.clone(),
                    ty: apply_branch_proof_to_local(store, proof, &el.ty),
                })
                .collect(),
        ),
        LocalType::Record(fields) => LocalType::Record(
            fields
                .iter()
                .map(|f| crate::types::rigid::LocalRecordField {
                    key: f.key.clone(),
                    ty: apply_branch_proof_to_local(store, proof, &f.ty),
                })
                .collect(),
        ),
        LocalType::Callable { parameters, return_type } => LocalType::Callable {
            parameters: parameters
                .iter()
                .map(|p| crate::types::rigid::LocalCallableParameter {
                    label: p.label.clone(),
                    ty: apply_branch_proof_to_local(store, proof, &p.ty),
                    rest: p.rest,
                })
                .collect(),
            return_type: Box::new(apply_branch_proof_to_local(store, proof, return_type)),
        },
    }
}

fn unify_local_types(
    store: &TypeStore,
    case: &CaseInstantiation,
    left: &LocalType,
    right: &LocalType,
    bindings: &mut BTreeMap<TypeParameterId, LocalType>,
    exact_case_observation: bool,
    control: &crate::checker::context::CheckerControl,
    depth: u16,
) -> LocalUnification {
    local_proof_control(control).map_err(LocalProofFailure::Blocked)?;
    let max_depth = control.budget.borrow().max_type_depth;
    if depth > max_depth {
        return Err(LocalProofFailure::Blocked(crate::types::outcome::BlockReason::BudgetExceeded(
            crate::types::outcome::BudgetReport::new(crate::types::outcome::BudgetKind::TypeDepth, u64::from(max_depth), u64::from(depth)),
        )));
    }
    // A retained canonical leaf containing a constructor binder could not be
    // localized by LocalType. It must not erase existential identity or refute
    // a case merely because this representation has no local counterpart.
    for term in [left, right] {
        if let LocalType::Canonical(ty) = term {
            if case.local_rigids.keys().any(|parameter| store.contains_type_parameter(*ty, *parameter)) && !matches!(store.get(*ty), TypeData::Parameter(_)) {
                return Err(LocalProofFailure::Blocked(crate::types::outcome::BlockReason::ReflectionBoundary));
            }
        }
    }
    if left == right {
        return Ok(());
    }
    let recurse = |left: &LocalType, right: &LocalType, bindings: &mut BTreeMap<TypeParameterId, LocalType>| {
        unify_local_types(store, case, left, right, bindings, exact_case_observation, control, depth.saturating_add(1))
    };
    match (left, right) {
        (LocalType::Rigid(rigid), other) | (other, LocalType::Rigid(rigid)) => {
            let rigid_term = LocalType::Rigid(*rigid);
            if let Some((&parameter, _)) = case.local_rigids.iter().find(|(_, value)| *value == rigid) {
                if let Some(existing) = bindings.get(&parameter).cloned() {
                    recurse(&existing, other, bindings)
                } else if other.free_rigids().contains(rigid) {
                    Err(LocalProofFailure::Refuted)
                } else {
                    bindings.insert(parameter, other.clone());
                    Ok(())
                }
            } else if let LocalType::Canonical(ty) = other {
                if matches!(store.get(*ty), TypeData::Parameter(_)) {
                    bind_local_parameter(store, *ty, &rigid_term, bindings)
                } else if exact_case_observation {
                    Ok(())
                } else {
                    Err(local_proof_blocked())
                }
            } else if exact_case_observation {
                Ok(())
            } else {
                // Parent rigids are opaque: the absence of a supported equality
                // proof is not evidence that their indices are contradictory.
                Err(local_proof_blocked())
            }
        }
        (LocalType::Canonical(ty), other) | (other, LocalType::Canonical(ty)) => {
            if matches!(store.get(*ty), TypeData::Parameter(_)) {
                return bind_local_parameter(store, *ty, other, bindings);
            }
            let expanded = LocalType::from_canonical(store, *ty, &HashMap::new());
            if !matches!(expanded, LocalType::Canonical(_)) {
                return recurse(&expanded, other, bindings);
            }
            match other {
                LocalType::Canonical(other_ty) if canonical_local_leaf_is_closed(store, *ty) && canonical_local_leaf_is_closed(store, *other_ty) => {
                    Err(LocalProofFailure::Refuted)
                }
                _ => Err(local_proof_blocked()),
            }
        }
        (
            LocalType::Applied {
                origin: left_origin,
                arguments: left_args,
            },
            LocalType::Applied {
                origin: right_origin,
                arguments: right_args,
            },
        ) => {
            if left_args.len() != right_args.len() {
                return Err(LocalProofFailure::Refuted);
            }
            recurse(left_origin, right_origin, bindings)?;
            for (left, right) in left_args.iter().zip(right_args.iter()) {
                recurse(left, right, bindings)?;
            }
            Ok(())
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
        ) => {
            if left_variant != right_variant {
                return Err(LocalProofFailure::Refuted);
            }
            recurse(left_enum, right_enum, bindings)
        }
        (LocalType::Union(left), LocalType::Union(right)) => {
            if left.len() != right.len() {
                return Err(local_proof_blocked());
            }
            for (left, right) in left.iter().zip(right.iter()) {
                // Local union members need not retain canonical ordering after
                // substitution. A failed pairing cannot refute set equality.
                if let Err(failure) = recurse(left, right, bindings) {
                    return Err(match failure {
                        LocalProofFailure::Refuted => local_proof_blocked(),
                        blocked => blocked,
                    });
                }
            }
            Ok(())
        }
        (LocalType::Tuple(left), LocalType::Tuple(right)) => {
            if left.len() != right.len() {
                return Err(LocalProofFailure::Refuted);
            }
            for (left, right) in left.iter().zip(right.iter()) {
                if left.label != right.label {
                    return Err(LocalProofFailure::Refuted);
                }
                recurse(&left.ty, &right.ty, bindings)?;
            }
            Ok(())
        }
        (LocalType::Record(left), LocalType::Record(right)) => {
            if left.len() != right.len() {
                return Err(LocalProofFailure::Refuted);
            }
            for left in left.iter() {
                let Some(right) = right.iter().find(|right| right.key == left.key) else {
                    return Err(LocalProofFailure::Refuted);
                };
                recurse(&left.ty, &right.ty, bindings)?;
            }
            Ok(())
        }
        (
            LocalType::Callable {
                parameters: left,
                return_type: left_return,
            },
            LocalType::Callable {
                parameters: right,
                return_type: right_return,
            },
        ) => {
            if left.len() != right.len() {
                return Err(LocalProofFailure::Refuted);
            }
            for (left, right) in left.iter().zip(right.iter()) {
                if left.label != right.label || left.rest != right.rest {
                    return Err(LocalProofFailure::Refuted);
                }
                recurse(&left.ty, &right.ty, bindings)?;
            }
            recurse(left_return, right_return, bindings)
        }
        _ => Err(local_proof_blocked()),
    }
}

fn canonical_local_leaf_is_closed(store: &TypeStore, ty: TypeId) -> bool {
    matches!(
        store.get(ty),
        TypeData::Never | TypeData::Unit | TypeData::Nominal { .. } | TypeData::ClassObject { .. } | TypeData::ConstInt(_)
    )
}

fn bind_local_parameter(store: &TypeStore, ty: TypeId, replacement: &LocalType, bindings: &mut BTreeMap<TypeParameterId, LocalType>) -> LocalUnification {
    let TypeData::Parameter(parameter) = store.get(ty) else {
        return Err(local_proof_blocked());
    };
    if let Some(existing) = bindings.get(parameter) {
        return if existing == replacement { Ok(()) } else { Err(local_proof_blocked()) };
    }
    if matches!(replacement, LocalType::Canonical(other) if *other == ty) {
        return Ok(());
    }
    if local_contains_parameter(store, replacement, *parameter) {
        return Err(LocalProofFailure::Refuted);
    }
    bindings.insert(*parameter, replacement.clone());
    Ok(())
}

fn local_contains_parameter(store: &TypeStore, term: &LocalType, parameter: TypeParameterId) -> bool {
    match term {
        LocalType::Canonical(ty) => store.contains_type_parameter(*ty, parameter),
        LocalType::Rigid(_) => false,
        LocalType::Applied { origin, arguments } => {
            local_contains_parameter(store, origin, parameter) || arguments.iter().any(|term| local_contains_parameter(store, term, parameter))
        }
        LocalType::ExactCase { enum_type, .. } => local_contains_parameter(store, enum_type, parameter),
        LocalType::Union(terms) => terms.iter().any(|term| local_contains_parameter(store, term, parameter)),
        LocalType::Tuple(elements) => elements.iter().any(|element| local_contains_parameter(store, &element.ty, parameter)),
        LocalType::Record(fields) => fields.iter().any(|field| local_contains_parameter(store, &field.ty, parameter)),
        LocalType::Callable { parameters, return_type } => {
            parameters.iter().any(|entry| local_contains_parameter(store, &entry.ty, parameter)) || local_contains_parameter(store, return_type, parameter)
        }
    }
}

/// Merges branch equalities without allowing a later constraint to overwrite
/// an incompatible earlier binding.
pub(crate) fn merge_branch_proofs(store: &mut TypeStore, left: &BranchProofEnvironment, right: &BranchProofEnvironment) -> ProofMerge {
    let mut substitution = TypeSubstitution::new();
    let mut parameters = BTreeSet::new();
    for (&parameter, &ty) in left.bindings.iter().chain(right.bindings.iter()) {
        parameters.insert(parameter);
        let parameter_ty = store.parameter_form(parameter);
        if let Err(failure) = unify_equality(store, &mut substitution, parameter_ty, ty) {
            return match failure {
                LocalProofFailure::Refuted => ProofMerge::Contradictory,
                LocalProofFailure::Blocked(reason) => ProofMerge::Blocked(reason),
            };
        }
    }

    let mut equalities = Vec::new();
    for equality in left.equalities.iter().chain(right.equalities.iter()) {
        let TypeConstraint::Equal(lhs, rhs) = equality else {
            if !equalities.contains(equality) {
                equalities.push(equality.clone());
            }
            continue;
        };
        if let Err(failure) = unify_equality(store, &mut substitution, *lhs, *rhs) {
            return match failure {
                LocalProofFailure::Refuted => ProofMerge::Contradictory,
                LocalProofFailure::Blocked(reason) => ProofMerge::Blocked(reason),
            };
        }
        if !equalities.contains(equality) {
            equalities.push(equality.clone());
        }
    }

    // Equality solving may discover parameters that were not explicitly in a
    // branch binding (for example through an exact case). Retain those facts
    // by collecting parameters from all equality terms as well.
    for equality in equalities.iter() {
        let TypeConstraint::Equal(lhs, rhs) = equality else { continue };
        let mut equality_parameters = Vec::new();
        collect_type_parameters(store, *lhs, &mut equality_parameters);
        collect_type_parameters(store, *rhs, &mut equality_parameters);
        parameters.extend(equality_parameters);
    }

    let bindings = parameters
        .into_iter()
        .filter_map(|parameter| {
            let parameter_ty = store.parameter_form(parameter);
            let resolved = apply_substitution_to_fixpoint(store, &substitution, parameter_ty);
            (resolved != parameter_ty).then_some((parameter, resolved))
        })
        .collect();

    let mut local_bindings = left.local_bindings.clone();
    for (&parameter, right_type) in &right.local_bindings {
        if let Some(left_type) = local_bindings.get(&parameter) {
            if left_type != right_type {
                // Unequal local syntax, especially opaque rigids, is not a
                // contradiction proof. Only distinct closed canonical leaves
                // establish a contradiction without additional local solving.
                if let (LocalType::Canonical(left), LocalType::Canonical(right)) = (left_type, right_type) {
                    if canonical_local_leaf_is_closed(store, *left) && canonical_local_leaf_is_closed(store, *right) {
                        return ProofMerge::Contradictory;
                    }
                }
                return ProofMerge::Blocked(crate::types::outcome::BlockReason::ReflectionBoundary);
            }
        } else {
            local_bindings
                .insert_source(parameter, right_type.clone())
                .expect("new GADT branch proofs use source-mutable maps");
        }
    }
    let mut local_equalities = left.local_equalities.to_vec();
    for equality in right.local_equalities.iter() {
        if !local_equalities.contains(equality) {
            local_equalities.push(equality.clone());
        }
    }

    ProofMerge::Compatible(BranchProofEnvironment {
        bindings,
        equalities: equalities.into_boxed_slice(),
        local_bindings,
        local_equalities: local_equalities.into_boxed_slice(),
    })
}

/// Solves GADT equality constraints between a variant's case environment and the scrutinee type.
///
/// GADT elimination is equality-producing, not subtype filtering. In particular, a
/// scrutinee such as `Expr<T>` remains compatible with a case returning `Expr<Int>`:
/// observing that case establishes `T = Int`. Conversely, a concrete `Expr<Bool>`
/// refutes the same case because `Bool = Int` cannot be solved.
pub fn solve_gadt_branch_proof(
    store: &mut TypeStore,
    _hier: &dyn TypeHierarchy,
    owner_decl: &DeclarationId,
    variant_info: &VariantInfo,
    scrutinee_ty: TypeId,
) -> GadtProofResult {
    if variant_info.case_environment.is_empty() {
        let exact_case = store
            .exact_case_type(&variant_info.id, scrutinee_ty)
            .unwrap_or(variant_info.exact_case_template);
        return GadtProofResult::Reachable {
            proof: BranchProofEnvironment::default(),
            exact_case,
        };
    }

    let scrutinee_args = match store.applied_nominal_parts(scrutinee_ty) {
        Some((decl, args)) if decl == *owner_decl => args,
        _ => Vec::new(),
    };

    let mut substitution = TypeSubstitution::new();
    let mut equalities = Vec::with_capacity(variant_info.case_environment.bindings.len());

    for (&enum_parameter, &case_ty) in &variant_info.case_environment.bindings {
        let parameter_index = store.type_parameter(enum_parameter).index as usize;
        let enum_parameter_ty = store.parameter_form(enum_parameter);
        let Some(&scrutinee_arg_ty) = scrutinee_args.get(parameter_index) else {
            // Unspecialized enum roots retain the declaration-owned case equality.
            if let Err(failure) = unify_equality(store, &mut substitution, enum_parameter_ty, case_ty) {
                return match failure {
                    LocalProofFailure::Refuted => GadtProofResult::Refuted,
                    LocalProofFailure::Blocked(reason) => GadtProofResult::Blocked(reason),
                };
            }
            equalities.push(TypeConstraint::Equal(enum_parameter_ty, case_ty));
            continue;
        };

        // The case result contributes an equality, and the scrutinee argument may
        // itself be a type parameter belonging to the enclosing callable. Solving
        // the two structurally is what introduces that branch-local refinement.
        if let Err(failure) = unify_equality(store, &mut substitution, case_ty, scrutinee_arg_ty) {
            return match failure {
                LocalProofFailure::Refuted => GadtProofResult::Refuted,
                LocalProofFailure::Blocked(reason) => GadtProofResult::Blocked(reason),
            };
        }
        if let Err(failure) = unify_equality(store, &mut substitution, enum_parameter_ty, case_ty) {
            return match failure {
                LocalProofFailure::Refuted => GadtProofResult::Refuted,
                LocalProofFailure::Blocked(reason) => GadtProofResult::Blocked(reason),
            };
        }
        equalities.push(TypeConstraint::Equal(case_ty, scrutinee_arg_ty));
    }

    let mut bindings = std::collections::BTreeMap::new();
    for (&enum_parameter, &case_ty) in &variant_info.case_environment.bindings {
        bindings.insert(enum_parameter, apply_substitution_to_fixpoint(store, &substitution, case_ty));
    }

    // Preserve substitutions for scrutinee-owned parameters as well. Branch
    // consumers need these equalities to refine callable-generic terms, while the
    // declaration-owned entries above retain the canonical GADT case environment.
    for parameter in parameters_referenced_by_scrutinee(store, &scrutinee_args) {
        let parameter_ty = store.parameter_form(parameter);
        let refined = apply_substitution_to_fixpoint(store, &substitution, parameter_ty);
        if refined != parameter_ty {
            bindings.insert(parameter, refined);
        }
    }

    let proof = BranchProofEnvironment {
        bindings: bindings.into(),
        equalities: equalities.into_boxed_slice(),
        local_bindings: Default::default(),
        local_equalities: Box::new([]),
    };
    let specialized_scrutinee = apply_branch_proof(store, &proof, scrutinee_ty);
    let exact_case = store
        .exact_case_type(&variant_info.id, specialized_scrutinee)
        .unwrap_or(variant_info.exact_case_template);

    GadtProofResult::Reachable { proof, exact_case }
}

/// Applies all substitutions established by a branch proof to a canonical type.
///
/// The substitution API deliberately performs one structural pass. Repeating to a
/// fixpoint also resolves short parameter chains such as `T -> U`, `U -> Int`.
pub(crate) fn apply_branch_proof(store: &mut TypeStore, proof: &BranchProofEnvironment, ty: TypeId) -> TypeId {
    if proof.bindings.is_empty() {
        return ty;
    }
    let mut substitution = TypeSubstitution::new();
    for (&parameter, &replacement) in &proof.bindings {
        substitution.bind(parameter, replacement);
    }
    apply_substitution_to_fixpoint(store, &substitution, ty)
}

fn apply_substitution_to_fixpoint(store: &mut TypeStore, substitution: &TypeSubstitution, ty: TypeId) -> TypeId {
    #[derive(Clone, Copy)]
    enum VisitState {
        Visiting,
        Resolved(TypeId),
    }

    fn normalize(store: &mut TypeStore, substitution: &TypeSubstitution, ty: TypeId, states: &mut BTreeMap<TypeId, VisitState>) -> TypeId {
        if let Some(state) = states.get(&ty) {
            return match state {
                VisitState::Resolved(resolved) => *resolved,
                // TypeSubstitution binding is occurs-checked. Reaching this
                // branch means malformed recovery data; retain current term
                // instead of looping or returning a partially iterated chain.
                VisitState::Visiting => ty,
            };
        }
        states.insert(ty, VisitState::Visiting);
        let normalized = match store.get(ty).clone() {
            TypeData::Parameter(parameter) => substitution
                .get(parameter)
                .filter(|replacement| *replacement != ty)
                .map(|replacement| normalize(store, substitution, replacement, states))
                .unwrap_or(ty),
            TypeData::Applied { origin, arguments } => {
                let origin = normalize(store, substitution, origin, states);
                let arguments = arguments
                    .iter()
                    .map(|argument| argument.as_type_id().map(|argument| normalize(store, substitution, argument, states)))
                    .collect::<Option<Vec<_>>>();
                let Some(arguments) = arguments else { return ty };
                store.apply_type_form(origin, &arguments).unwrap_or(ty)
            }
            TypeData::ExactCase { variant, enum_type } => {
                let enum_type = normalize(store, substitution, enum_type, states);
                let variant = store.variant_identity(variant).clone();
                store.exact_case_type(&variant, enum_type).unwrap_or(ty)
            }
            TypeData::AssociatedProjection(projection) => {
                let subject = normalize(store, substitution, projection.subject, states);
                let arguments = projection
                    .trait_ref
                    .arguments
                    .iter()
                    .map(|&argument| normalize(store, substitution, argument, states))
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                store.associated_projection(
                    subject,
                    crate::traits::TraitRef::new(projection.trait_ref.declaration, arguments),
                    projection.requirement,
                )
            }
            TypeData::TypeMemberProjection(projection) => {
                let kind = store.kind_of(ty);
                let subject = match &projection.subject {
                    crate::types::type_member::ProjectionSubject::Type(sub_ty) => {
                        crate::types::type_member::ProjectionSubject::Type(normalize(store, substitution, *sub_ty, states))
                    }
                    crate::types::type_member::ProjectionSubject::RecordRow(row_id) => crate::types::type_member::ProjectionSubject::RecordRow(*row_id),
                };
                let arguments = projection
                    .arguments
                    .iter()
                    .map(|arg| match arg {
                        crate::types::type_member::TypeMemberArgument::Type(arg_ty) => {
                            crate::types::type_member::TypeMemberArgument::Type(normalize(store, substitution, *arg_ty, states))
                        }
                        crate::types::type_member::TypeMemberArgument::Const(c) => crate::types::type_member::TypeMemberArgument::Const(c.clone()),
                        crate::types::type_member::TypeMemberArgument::RecordRow(row_id) => crate::types::type_member::TypeMemberArgument::RecordRow(*row_id),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                store.type_member_projection(
                    crate::types::type_member::TypeLevelProjection {
                        subject,
                        selector: projection.selector.clone(),
                        arguments,
                        source_declaration: projection.source_declaration.clone(),
                    },
                    kind,
                )
            }
            TypeData::Union(members) => {
                let members = members.iter().map(|&member| normalize(store, substitution, member, states)).collect::<Vec<_>>();
                store.union(&members)
            }
            TypeData::Tuple(tuple) => {
                let elements = tuple
                    .elements
                    .iter()
                    .map(|element| TupleTypeElement {
                        label: element.label.clone(),
                        ty: normalize(store, substitution, element.ty, states),
                    })
                    .collect::<Vec<_>>();
                store.open_tuple(elements.into_boxed_slice(), tuple.positional_remainder, tuple.labeled_remainder)
            }
            TypeData::Record(row_id) => {
                let (fields, tail) = {
                    let row = store.record_row(row_id);
                    (row.fields.to_vec(), row.tail)
                };
                let fields = fields
                    .into_iter()
                    .map(|field| crate::types::row::RecordRowField {
                        key: field.key,
                        ty: normalize(store, substitution, field.ty, states),
                        presence: field.presence,
                    })
                    .collect::<Vec<_>>();
                store
                    .record_row_type_checked(fields, tail)
                    .expect("GADT proof normalization must preserve canonical Record-row invariants")
            }
            TypeData::Callable(callable) => {
                let parameters = callable
                    .parameters
                    .iter()
                    .map(|parameter| CallableParameterType {
                        label: parameter.label.clone(),
                        ty: normalize(store, substitution, parameter.ty, states),
                        rest: parameter.rest,
                    })
                    .collect::<Vec<_>>();
                let return_type = normalize(store, substitution, callable.return_type, states);
                store.callable(CallableType {
                    parameters: parameters.into_boxed_slice(),
                    return_type,
                })
            }
            TypeData::Family(family_id) => {
                let family = store.get_family(family_id).clone();
                let members = family
                    .members
                    .iter()
                    .map(|member| crate::types::family::FamilyMemberType {
                        operation: member.operation.clone(),
                        member_kind: member.member_kind,
                        ty: normalize(store, substitution, member.ty, states),
                    })
                    .collect::<Vec<_>>();
                store.family_type(members).unwrap_or(ty)
            }
            TypeData::TypePack(elements) => {
                let elements = elements
                    .iter()
                    .map(|&element| normalize(store, substitution, element, states))
                    .collect::<Vec<_>>();
                store.type_pack(elements.into_boxed_slice()).unwrap_or(ty)
            }
            TypeData::LabeledTypePack(elements) => {
                let elements = elements
                    .iter()
                    .map(|element| crate::types::store::LabeledTypePackElement {
                        label: element.label.clone(),
                        element: normalize(store, substitution, element.element, states),
                    })
                    .collect::<Vec<_>>();
                store.labeled_type_pack(elements.into_boxed_slice()).unwrap_or(ty)
            }
            TypeData::Pack(pack) => {
                let elements = pack
                    .elements
                    .iter()
                    .map(|element| PackTypeElement {
                        label: element.label.clone(),
                        ty: normalize(store, substitution, element.ty, states),
                    })
                    .collect::<Vec<_>>();
                let pos_rem = match pack.positional_remainder {
                    AnonymousTupleRemainder::Typed(rem_ty) => AnonymousTupleRemainder::Typed(normalize(store, substitution, rem_ty, states)),
                    other => other,
                };
                let lab_rem = match pack.labeled_remainder {
                    AnonymousTupleRemainder::Typed(rem_ty) => AnonymousTupleRemainder::Typed(normalize(store, substitution, rem_ty, states)),
                    other => other,
                };
                store.open_pack(elements.into_boxed_slice(), pos_rem, lab_rem).unwrap_or(ty)
            }
            TypeData::PackFromPacks { positionals, labeled } => {
                let positionals = normalize(store, substitution, positionals, states);
                let labeled = normalize(store, substitution, labeled, states);
                store.pack_from_packs(positionals, labeled).unwrap_or(ty)
            }
            TypeData::TupleFromPacks { positionals, labeled } => {
                let positionals = normalize(store, substitution, positionals, states);
                let labeled = normalize(store, substitution, labeled, states);
                store.tuple_from_packs(positionals, labeled).unwrap_or(ty)
            }
            TypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                let positionals = normalize(store, substitution, positionals, states);
                let labeled = normalize(store, substitution, labeled, states);
                let return_type = normalize(store, substitution, return_type, states);
                store.callable_from_packs(positionals, labeled, return_type).unwrap_or(ty)
            }
            TypeData::CompleteTypePack { positionals, labeled } => {
                let positionals = normalize(store, substitution, positionals, states);
                let labeled = normalize(store, substitution, labeled, states);
                store.complete_type_pack(positionals, labeled).unwrap_or(ty)
            }
            TypeData::TypePackConcat(segments) => {
                let segments = segments
                    .iter()
                    .map(|&segment| normalize(store, substitution, segment, states))
                    .collect::<Vec<_>>();
                store.type_pack_concat(&segments).unwrap_or(ty)
            }
            TypeData::LabeledTypePackConcat(segments) => {
                let segments = segments
                    .iter()
                    .map(|&segment| normalize(store, substitution, segment, states))
                    .collect::<Vec<_>>();
                store.labeled_type_pack_concat(&segments).unwrap_or(ty)
            }
            TypeData::StaticData(term) => {
                let arguments = term
                    .arguments
                    .iter()
                    .map(|argument| match argument {
                        StaticArgument::Term(argument) => StaticArgument::Term(normalize(store, substitution, *argument, states)),
                        StaticArgument::RecordRow(row) => StaticArgument::RecordRow(*row),
                        StaticArgument::DynamicType => StaticArgument::DynamicType,
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                store.static_data(term.domain, term.constructor, arguments).unwrap_or(ty)
            }
            TypeData::ConstInt(_) => ty,
            TypeData::Never
            | TypeData::Unit
            | TypeData::ClassObject { .. }
            | TypeData::Nominal { .. }
            | TypeData::StaticDomainForm { .. }
            | TypeData::Lambda(_)
            | TypeData::SelfType(_) => ty,
        };
        states.insert(ty, VisitState::Resolved(normalized));
        normalized
    }

    normalize(store, substitution, ty, &mut BTreeMap::new())
}

fn parameters_referenced_by_scrutinee(store: &TypeStore, arguments: &[TypeId]) -> Vec<TypeParameterId> {
    let mut parameters = Vec::new();
    for &argument in arguments {
        collect_type_parameters(store, argument, &mut parameters);
    }
    parameters.sort_unstable();
    parameters.dedup();
    parameters
}

fn collect_type_parameters(store: &TypeStore, ty: TypeId, output: &mut Vec<TypeParameterId>) {
    match store.get(ty) {
        TypeData::Parameter(parameter) => output.push(*parameter),
        TypeData::Applied { origin, arguments } => {
            collect_type_parameters(store, *origin, output);
            for argument in arguments.iter().filter_map(|argument| argument.as_type_id()) {
                collect_type_parameters(store, argument, output);
            }
        }
        TypeData::ExactCase { enum_type, .. } => collect_type_parameters(store, *enum_type, output),
        TypeData::AssociatedProjection(projection) => {
            collect_type_parameters(store, projection.subject, output);
            for &argument in projection.trait_ref.arguments.iter() {
                collect_type_parameters(store, argument, output);
            }
        }
        TypeData::TypeMemberProjection(projection) => {
            if let crate::types::type_member::ProjectionSubject::Type(ty) = projection.subject {
                collect_type_parameters(store, ty, output);
            }
            for argument in projection.arguments.iter() {
                if let crate::types::type_member::TypeMemberArgument::Type(ty) = argument {
                    collect_type_parameters(store, *ty, output);
                }
            }
        }
        TypeData::Union(members) => {
            for &member in members.iter() {
                collect_type_parameters(store, member, output);
            }
        }
        TypeData::Tuple(tuple) => {
            for element in tuple.elements.iter() {
                collect_type_parameters(store, element.ty, output);
            }
        }
        TypeData::Pack(pack) => {
            for element in pack.elements.iter() {
                collect_type_parameters(store, element.ty, output);
            }
        }
        TypeData::PackFromPacks { positionals, labeled } => {
            collect_type_parameters(store, *positionals, output);
            collect_type_parameters(store, *labeled, output);
        }
        TypeData::Callable(callable) => {
            for parameter in callable.parameters.iter() {
                collect_type_parameters(store, parameter.ty, output);
            }
            collect_type_parameters(store, callable.return_type, output);
        }
        TypeData::Record(row) => {
            for field in store.record_row(*row).fields.iter() {
                collect_type_parameters(store, field.ty, output);
            }
        }
        TypeData::TypePack(elements) => {
            for &element in elements.iter() {
                collect_type_parameters(store, element, output);
            }
        }
        TypeData::LabeledTypePack(elements) => {
            for element in elements.iter() {
                collect_type_parameters(store, element.element, output);
            }
        }
        TypeData::TupleFromPacks { positionals, labeled } => {
            collect_type_parameters(store, *positionals, output);
            collect_type_parameters(store, *labeled, output);
        }
        TypeData::CallableFromPacks {
            positionals,
            labeled,
            return_type,
        } => {
            collect_type_parameters(store, *positionals, output);
            collect_type_parameters(store, *labeled, output);
            collect_type_parameters(store, *return_type, output);
        }
        TypeData::CompleteTypePack { positionals, labeled } => {
            collect_type_parameters(store, *positionals, output);
            collect_type_parameters(store, *labeled, output);
        }
        TypeData::TypePackConcat(segments) | TypeData::LabeledTypePackConcat(segments) => {
            for &segment in segments.iter() {
                collect_type_parameters(store, segment, output);
            }
        }
        TypeData::StaticData(term) => {
            for argument in term.arguments.iter() {
                if let StaticArgument::Term(argument) = argument {
                    collect_type_parameters(store, *argument, output);
                }
            }
        }
        TypeData::ConstInt(_) => {}
        TypeData::Never
        | TypeData::Unit
        | TypeData::ClassObject { .. }
        | TypeData::Nominal { .. }
        | TypeData::Family(_)
        | TypeData::Lambda(_)
        | TypeData::SelfType(_)
        | TypeData::StaticDomainForm { .. } => {}
    }
}

/// Unifies two canonical type terms under an occurs-checked parameter substitution.
///
/// Only equality can establish a GADT case. Nominal subtyping deliberately does
/// not participate here: a superclass relationship does not prove type equality.
fn unify_equality(store: &mut TypeStore, substitution: &mut TypeSubstitution, left: TypeId, right: TypeId) -> LocalUnification {
    let left = apply_substitution_to_fixpoint(store, substitution, left);
    let right = apply_substitution_to_fixpoint(store, substitution, right);
    if left == right {
        return Ok(());
    }
    match (store.get(left).clone(), store.get(right).clone()) {
        (TypeData::Parameter(parameter), _) => bind_parameter(store, substitution, parameter, right),
        (_, TypeData::Parameter(parameter)) => bind_parameter(store, substitution, parameter, left),
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
            if left_args.len() != right_args.len() {
                return Err(LocalProofFailure::Refuted);
            }
            unify_equality(store, substitution, left_origin, right_origin)?;
            for (left, right) in left_args.iter().zip(right_args.iter()) {
                match (left.as_type_id(), right.as_type_id()) {
                    (Some(left), Some(right)) => unify_equality(store, substitution, left, right)?,
                    _ if left == right => {}
                    _ => return Err(local_proof_blocked()),
                }
            }
            Ok(())
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
        ) => {
            if left_variant != right_variant {
                return Err(LocalProofFailure::Refuted);
            }
            unify_equality(store, substitution, left_enum, right_enum)
        }
        (TypeData::Union(left_members), TypeData::Union(right_members)) => {
            if left_members.len() != right_members.len() {
                return Err(local_proof_blocked());
            }
            for (left, right) in left_members.iter().zip(right_members.iter()) {
                // An unmatched positional pairing does not refute set equivalence.
                if unify_equality(store, substitution, *left, *right).is_err() {
                    return Err(local_proof_blocked());
                }
            }
            Ok(())
        }
        (TypeData::Tuple(left_tuple), TypeData::Tuple(right_tuple)) => {
            if left_tuple.positional_remainder != AnonymousTupleRemainder::None
                || right_tuple.positional_remainder != AnonymousTupleRemainder::None
                || left_tuple.labeled_remainder != AnonymousTupleRemainder::None
                || right_tuple.labeled_remainder != AnonymousTupleRemainder::None
            {
                return Err(local_proof_blocked());
            }
            if left_tuple.elements.len() != right_tuple.elements.len() {
                return Err(LocalProofFailure::Refuted);
            }
            for (left, right) in left_tuple.elements.iter().zip(right_tuple.elements.iter()) {
                if left.label != right.label {
                    return Err(LocalProofFailure::Refuted);
                }
                unify_equality(store, substitution, left.ty, right.ty)?;
            }
            Ok(())
        }
        (TypeData::Record(left_row), TypeData::Record(right_row)) => unify_record_rows(store, substitution, left_row, right_row),
        (TypeData::Callable(left_callable), TypeData::Callable(right_callable)) => {
            if left_callable.parameters.len() != right_callable.parameters.len() {
                return Err(LocalProofFailure::Refuted);
            }
            for (left, right) in left_callable.parameters.iter().zip(right_callable.parameters.iter()) {
                if left.label != right.label || left.rest != right.rest {
                    return Err(LocalProofFailure::Refuted);
                }
                unify_equality(store, substitution, left.ty, right.ty)?;
            }
            unify_equality(store, substitution, left_callable.return_type, right_callable.return_type)
        }
        _ if canonical_has_closed_head(store, left) && canonical_has_closed_head(store, right) => Err(LocalProofFailure::Refuted),
        // Projections, higher-kind/static forms, and unsupported representations
        // have no contradiction proof in this solver. They cannot omit a case.
        _ => Err(local_proof_blocked()),
    }
}

fn canonical_has_closed_head(store: &TypeStore, ty: TypeId) -> bool {
    match store.get(ty) {
        TypeData::Applied { origin, .. } => matches!(store.get(*origin), TypeData::Nominal { .. }),
        TypeData::Tuple(tuple) => tuple.positional_remainder == AnonymousTupleRemainder::None && tuple.labeled_remainder == AnonymousTupleRemainder::None,
        TypeData::Record(row) => store.record_row(*row).tail == RecordRowTail::Closed,
        TypeData::Callable(_) | TypeData::ExactCase { .. } => true,
        _ => canonical_local_leaf_is_closed(store, ty),
    }
}

fn unify_record_rows(
    store: &mut TypeStore,
    substitution: &mut TypeSubstitution,
    left_row_id: crate::types::id::RecordRowId,
    right_row_id: crate::types::id::RecordRowId,
) -> LocalUnification {
    let left = store.record_row(left_row_id).clone();
    let right = store.record_row(right_row_id).clone();
    if left.tail != RecordRowTail::Closed || right.tail != RecordRowTail::Closed {
        return Err(local_proof_blocked());
    }
    if left.fields.len() != right.fields.len() {
        return Err(LocalProofFailure::Refuted);
    }
    for left_field in left.fields.iter() {
        let Some(right_field) = right.fields.iter().find(|field| field.key == left_field.key) else {
            return Err(LocalProofFailure::Refuted);
        };
        unify_equality(store, substitution, left_field.ty, right_field.ty)?;
    }
    Ok(())
}

fn bind_parameter(store: &mut TypeStore, substitution: &mut TypeSubstitution, parameter: TypeParameterId, ty: TypeId) -> LocalUnification {
    if let Some(existing) = substitution.get(parameter) {
        return unify_equality(store, substitution, existing, ty);
    }
    let ty = apply_substitution_to_fixpoint(store, substitution, ty);
    let parameter_ty = store.parameter_form(parameter);
    if ty == parameter_ty {
        return Ok(());
    }
    if store.contains_type_parameter(ty, parameter) {
        return Err(LocalProofFailure::Refuted);
    }
    substitution.bind(parameter, ty);
    Ok(())
}

#[cfg(test)]
mod local_outcome_tests {
    use super::*;
    use crate::checker::context::CheckerControl;
    use crate::identity::{ModuleId, VariantId};
    use crate::types::id::KindId;
    use crate::types::outcome::{BlockReason, CancellationToken, QueryBudget};
    use crate::types::parameter::{TypeParameterData, TypeParameterOwner};
    use crate::types::rigid::{LocalTupleElement, RigidArena, RigidOrigin};
    use phalcom_common::selector::Selector;

    fn fixture() -> (TypeStore, CaseInstantiation, LocalType, TypeId) {
        let mut store = TypeStore::new();
        let owner = DeclarationId::new(ModuleId::universe_root(), "LocalOutcome".into());
        let origin = store.nominal_type(owner.clone());
        let parameter = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner.clone()), 0, "A", KindId::TYPE));
        let mut arena = RigidArena::new();
        let parent_scope = arena.fresh_scope(None);
        let parent = arena.fresh(parent_scope, KindId::TYPE, RigidOrigin::Synthetic);
        let scope = arena.fresh_scope(Some(parent_scope));
        let variant = VariantId::new(owner, Selector::getter("Case").unwrap());
        let rigid = arena.fresh(
            scope,
            KindId::TYPE,
            RigidOrigin::VariantParameter {
                variant: variant.clone(),
                parameter,
            },
        );
        let local = |argument| LocalType::Applied {
            origin: Box::new(LocalType::Canonical(origin)),
            arguments: Box::new([argument]),
        };
        let case = CaseInstantiation {
            variant,
            scope,
            local_rigids: BTreeMap::from([(parameter, rigid)]).into(),
            payload_types: Box::new([LocalType::Rigid(rigid)]),
            result_type: local(LocalType::Rigid(rigid)),
            constraints: Box::new([]),
        };
        (store, case, local(LocalType::Rigid(parent)), origin)
    }

    #[test]
    fn local_case_outcomes_preserve_nested_parent_rigids_and_refute_closed_mismatch() {
        let (mut store, case, expected, _) = fixture();
        let outcome = solve_local_case_proof_against_local(&mut store, &BranchProofEnvironment::default(), &expected, &case, &CheckerControl::default());
        let LocalCaseProof::Reachable { bindings, equalities } = outcome else {
            panic!("supported local shape must be reachable: {outcome:?}");
        };
        assert_eq!(bindings.values().next(), expected_argument(&expected));
        assert!(!equalities.is_empty());
        assert_eq!(case.payload_types[0].free_rigids().len(), 1);
        let other = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "Other".into()));
        let mismatch = LocalType::Applied {
            origin: Box::new(LocalType::Canonical(other)),
            arguments: Box::new([LocalType::Canonical(store.unit())]),
        };
        assert_eq!(
            solve_local_case_proof_against_local(&mut store, &BranchProofEnvironment::default(), &mismatch, &case, &CheckerControl::default()),
            LocalCaseProof::Refuted
        );
    }

    fn expected_argument(expected: &LocalType) -> Option<&LocalType> {
        match expected {
            LocalType::Applied { arguments, .. } => arguments.first(),
            _ => None,
        }
    }

    #[test]
    fn local_case_outcomes_refute_occurs_checks_and_block_unresolved_projection() {
        let (mut store, mut case, _, origin) = fixture();
        let rigid = *case.local_rigids.values().next().unwrap();
        let recursive = LocalType::Applied {
            origin: Box::new(LocalType::Canonical(origin)),
            arguments: Box::new([LocalType::Tuple(Box::new([LocalTupleElement {
                label: None,
                ty: LocalType::Rigid(rigid),
            }]))]),
        };
        assert_eq!(
            solve_local_case_proof_against_local(&mut store, &BranchProofEnvironment::default(), &recursive, &case, &CheckerControl::default()),
            LocalCaseProof::Refuted
        );
        let trait_owner = DeclarationId::new(ModuleId::universe_root(), "Projected".into());
        let projection = store.associated_projection(
            origin,
            crate::traits::TraitRef::new(trait_owner.clone(), Vec::new()),
            crate::traits::AssociatedTypeRequirementId::new(trait_owner, 0),
        );
        case.result_type = LocalType::Applied {
            origin: Box::new(LocalType::Canonical(origin)),
            arguments: Box::new([LocalType::Canonical(projection)]),
        };
        let expected = LocalType::Applied {
            origin: Box::new(LocalType::Canonical(origin)),
            arguments: Box::new([LocalType::Canonical(store.unit())]),
        };
        assert!(matches!(
            solve_local_case_proof_against_local(&mut store, &BranchProofEnvironment::default(), &expected, &case, &CheckerControl::default()),
            LocalCaseProof::Blocked(_)
        ));
    }

    #[test]
    fn canonical_projection_and_proof_merge_uncertainty_remain_blocked() {
        let (mut store, case, _, origin) = fixture();
        let owner = DeclarationId::new(ModuleId::universe_root(), "Projection".into());
        let projection = store.associated_projection(
            origin,
            crate::traits::TraitRef::new(owner.clone(), Vec::new()),
            crate::traits::AssociatedTypeRequirementId::new(owner, 0),
        );
        let concrete = store.unit();
        assert!(matches!(
            unify_equality(&mut store, &mut TypeSubstitution::new(), projection, concrete),
            Err(LocalProofFailure::Blocked(_))
        ));
        let parameter = *case.local_rigids.keys().next().unwrap();
        let left = BranchProofEnvironment {
            bindings: BTreeMap::from([(parameter, projection)]).into(),
            ..Default::default()
        };
        let right = BranchProofEnvironment {
            bindings: BTreeMap::from([(parameter, concrete)]).into(),
            ..Default::default()
        };
        assert!(matches!(merge_branch_proofs(&mut store, &left, &right), ProofMerge::Blocked(_)));
        let left = BranchProofEnvironment {
            local_bindings: BTreeMap::from([(parameter, LocalType::Canonical(projection))]).into(),
            ..Default::default()
        };
        let right = BranchProofEnvironment {
            local_bindings: BTreeMap::from([(parameter, LocalType::Canonical(concrete))]).into(),
            ..Default::default()
        };
        assert!(matches!(merge_branch_proofs(&mut store, &left, &right), ProofMerge::Blocked(_)));

        let left = LocalType::Union(Box::new([LocalType::Canonical(origin), LocalType::Canonical(concrete)]));
        let right = LocalType::Union(Box::new([LocalType::Canonical(concrete), LocalType::Canonical(origin)]));
        assert!(matches!(
            unify_local_types(&store, &case, &left, &right, &mut BTreeMap::new(), false, &CheckerControl::default(), 0),
            Err(LocalProofFailure::Blocked(_))
        ));
    }

    #[test]
    fn local_case_outcomes_keep_cancellation_budget_and_depth_failure_blocked() {
        let (mut store, case, expected, _) = fixture();
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        assert!(matches!(
            solve_local_case_proof_against_local(
                &mut store,
                &BranchProofEnvironment::default(),
                &expected,
                &case,
                &CheckerControl::new(QueryBudget::default(), &cancellation)
            ),
            LocalCaseProof::Blocked(BlockReason::UnknownType(crate::types::evidence::UnknownReason::InferenceCancelled))
        ));
        let cancellation = CancellationToken::new();
        assert!(matches!(
            solve_local_case_proof_against_local(
                &mut store,
                &BranchProofEnvironment::default(),
                &expected,
                &case,
                &CheckerControl::new(QueryBudget::new(0), &cancellation)
            ),
            LocalCaseProof::Blocked(BlockReason::BudgetExceeded(_))
        ));
        let budget = QueryBudget {
            max_type_depth: 0,
            ..QueryBudget::default()
        };
        assert!(matches!(
            solve_local_case_proof_against_local(
                &mut store,
                &BranchProofEnvironment::default(),
                &expected,
                &case,
                &CheckerControl::new(budget, &cancellation)
            ),
            LocalCaseProof::Blocked(BlockReason::BudgetExceeded(_))
        ));
    }
}
