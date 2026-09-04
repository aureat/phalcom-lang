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
use crate::types::store::{CallableParameterType, CallableType, TupleTypeElement, TypeData, TypeStore};
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
}

/// Result of merging proof environments from two overlapping semantic spaces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProofMerge {
    Compatible(BranchProofEnvironment),
    Contradictory,
}

pub(crate) type LocalCaseProof = Option<(BTreeMap<TypeParameterId, LocalType>, Box<[LocalConstraint]>)>;

/// Refines a canonical scrutinee view against one freshly opened constructor
/// result. Flexible declaration parameters may be mapped to local terms, but
/// rigid leaves are opaque and are never rewritten.
#[allow(dead_code)]
pub(crate) fn solve_local_case_proof(
    store: &mut TypeStore,
    proof: &BranchProofEnvironment,
    expected_ty: TypeId,
    case: &CaseInstantiation,
) -> LocalCaseProof {
    solve_local_case_proof_against_local(store, proof, &LocalType::Canonical(expected_ty), case)
}

/// Refines an arbitrary local subject term against one freshly opened constructor
/// result. Preserves parent rigids while solving constructor-local equalities.
pub(crate) fn solve_local_case_proof_against_local(
    store: &mut TypeStore,
    proof: &BranchProofEnvironment,
    expected: &LocalType,
    case: &CaseInstantiation,
) -> LocalCaseProof {
    if !case.is_local() {
        return Some((BTreeMap::new(), Box::new([])));
    }

    let specialized_expected = apply_branch_proof_to_local(store, proof, expected);
    let (expected_term, exact_case_observation) = unpack_expected_local_term(store, specialized_expected, case);

    let mut bindings = BTreeMap::new();
    if !unify_local_types(store, &case.result_type, &expected_term, &mut bindings, exact_case_observation) {
        return None;
    }

    let mut equalities = case.constraints.to_vec();
    equalities.push(LocalConstraint::Equivalent {
        left: case.result_type.clone(),
        right: expected_term,
    });
    Some((bindings, equalities.into_boxed_slice()))
}

fn unpack_expected_local_term(
    store: &TypeStore,
    expected: LocalType,
    case: &CaseInstantiation,
) -> (LocalType, bool) {
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
pub(crate) fn apply_branch_proof_to_local(
    store: &mut TypeStore,
    proof: &BranchProofEnvironment,
    local: &LocalType,
) -> LocalType {
    if proof.bindings.is_empty() && proof.local_bindings.is_empty() {
        return local.clone();
    }
    match local {
        LocalType::Canonical(ty) => {
            let specialized = apply_branch_proof(store, proof, *ty);
            if proof.local_bindings.is_empty() {
                LocalType::Canonical(specialized)
            } else {
                let replacements: HashMap<_, _> =
                    proof.local_bindings.iter().map(|(k, v)| (*k, v.clone())).collect();
                LocalType::from_canonical(store, specialized, &replacements)
            }
        }
        LocalType::Rigid(id) => LocalType::Rigid(*id),
        LocalType::Applied { origin, arguments } => LocalType::Applied {
            origin: Box::new(apply_branch_proof_to_local(store, proof, origin)),
            arguments: arguments
                .iter()
                .map(|arg| apply_branch_proof_to_local(store, proof, arg))
                .collect(),
        },
        LocalType::ExactCase { variant, enum_type } => LocalType::ExactCase {
            variant: variant.clone(),
            enum_type: Box::new(apply_branch_proof_to_local(store, proof, enum_type)),
        },
        LocalType::Union(members) => LocalType::Union(
            members
                .iter()
                .map(|m| apply_branch_proof_to_local(store, proof, m))
                .collect(),
        ),
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
                    name: f.name.clone(),
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
    left: &LocalType,
    right: &LocalType,
    bindings: &mut BTreeMap<TypeParameterId, LocalType>,
    exact_case_observation: bool,
) -> bool {
    if left == right {
        return true;
    }
    match (left, right) {
        (LocalType::Rigid(_), LocalType::Canonical(ty)) => {
            if matches!(store.get(*ty), TypeData::Parameter(_)) {
                bind_local_parameter(store, *ty, left, bindings)
            } else {
                exact_case_observation
            }
        }
        (LocalType::Canonical(ty), LocalType::Rigid(_)) => {
            if matches!(store.get(*ty), TypeData::Parameter(_)) {
                bind_local_parameter(store, *ty, right, bindings)
            } else {
                exact_case_observation
            }
        }
        (LocalType::Canonical(left), _) => bind_local_parameter(store, *left, right, bindings),
        (_, LocalType::Canonical(right)) => bind_local_parameter(store, *right, left, bindings),
        (
            LocalType::Applied {
                origin: left_origin,
                arguments: left_arguments,
            },
            LocalType::Applied {
                origin: right_origin,
                arguments: right_arguments,
            },
        ) => {
            left_arguments.len() == right_arguments.len()
                && unify_local_types(store, left_origin, right_origin, bindings, exact_case_observation)
                && left_arguments
                    .iter()
                    .zip(right_arguments.iter())
                    .all(|(left, right)| unify_local_types(store, left, right, bindings, exact_case_observation))
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
        ) => left_variant == right_variant && unify_local_types(store, left_enum, right_enum, bindings, exact_case_observation),
        (LocalType::Rigid(left), LocalType::Rigid(right)) => left == right,
        (LocalType::Union(left), LocalType::Union(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(left, right)| unify_local_types(store, left, right, bindings, exact_case_observation))
        }
        (LocalType::Tuple(left), LocalType::Tuple(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(left, right)| left.label == right.label && unify_local_types(store, &left.ty, &right.ty, bindings, exact_case_observation))
        }
        (LocalType::Record(left), LocalType::Record(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(left, right)| left.name == right.name && unify_local_types(store, &left.ty, &right.ty, bindings, exact_case_observation))
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
        ) => {
            left_parameters.len() == right_parameters.len()
                && left_parameters
                    .iter()
                    .zip(right_parameters.iter())
                    .all(|(left, right)| left.label == right.label && left.rest == right.rest && unify_local_types(store, &left.ty, &right.ty, bindings, exact_case_observation))
                && unify_local_types(store, left_return, right_return, bindings, exact_case_observation)
        }
        _ => false,
    }
}

fn bind_local_parameter(store: &TypeStore, ty: TypeId, replacement: &LocalType, bindings: &mut BTreeMap<TypeParameterId, LocalType>) -> bool {
    let TypeData::Parameter(parameter) = store.get(ty) else {
        return matches!(replacement, LocalType::Canonical(other) if *other == ty);
    };
    if let Some(existing) = bindings.get(parameter) {
        return existing == replacement;
    }
    if matches!(replacement, LocalType::Canonical(other) if *other == ty) {
        return true;
    }
    bindings.insert(*parameter, replacement.clone());
    true
}

/// Merges branch equalities without allowing a later constraint to overwrite
/// an incompatible earlier binding.
pub(crate) fn merge_branch_proofs(store: &mut TypeStore, left: &BranchProofEnvironment, right: &BranchProofEnvironment) -> ProofMerge {
    let mut substitution = TypeSubstitution::new();
    let mut parameters = BTreeSet::new();
    for (&parameter, &ty) in left.bindings.iter().chain(right.bindings.iter()) {
        parameters.insert(parameter);
        let parameter_ty = store.parameter_form(parameter);
        if !unify_equality(store, &mut substitution, parameter_ty, ty) {
            return ProofMerge::Contradictory;
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
        if !unify_equality(store, &mut substitution, *lhs, *rhs) {
            return ProofMerge::Contradictory;
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
                return ProofMerge::Contradictory;
            }
        } else {
            local_bindings.insert(parameter, right_type.clone());
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

/// Checks exact-case compatibility using the same equality relation as GADT
/// branch reachability. Invalid recovery IDs are only compatible by identity.
pub(crate) fn exact_cases_compatible(store: &mut TypeStore, left: TypeId, right: TypeId) -> bool {
    if left == right {
        return true;
    }
    if left.index() >= store.len() || right.index() >= store.len() {
        return false;
    }
    let mut substitution = TypeSubstitution::new();
    unify_equality(store, &mut substitution, left, right)
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
            if !unify_equality(store, &mut substitution, enum_parameter_ty, case_ty) {
                return GadtProofResult::Refuted;
            }
            equalities.push(TypeConstraint::Equal(enum_parameter_ty, case_ty));
            continue;
        };

        // The case result contributes an equality, and the scrutinee argument may
        // itself be a type parameter belonging to the enclosing callable. Solving
        // the two structurally is what introduces that branch-local refinement.
        if !unify_equality(store, &mut substitution, case_ty, scrutinee_arg_ty) {
            return GadtProofResult::Refuted;
        }
        if !unify_equality(store, &mut substitution, enum_parameter_ty, case_ty) {
            return GadtProofResult::Refuted;
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
        bindings,
        equalities: equalities.into_boxed_slice(),
        local_bindings: BTreeMap::new(),
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
                    .map(|&argument| normalize(store, substitution, argument, states))
                    .collect::<Vec<_>>();
                store.apply_type_form(origin, &arguments).unwrap_or(ty)
            }
            TypeData::ExactCase { variant, enum_type } => {
                let enum_type = normalize(store, substitution, enum_type, states);
                let variant = store.variant_identity(variant).clone();
                store.exact_case_type(&variant, enum_type).unwrap_or(ty)
            }
            TypeData::Union(members) => {
                let members = members.iter().map(|&member| normalize(store, substitution, member, states)).collect::<Vec<_>>();
                store.union(&members)
            }
            TypeData::Tuple(elements) => {
                let elements = elements
                    .iter()
                    .map(|element| TupleTypeElement {
                        label: element.label.clone(),
                        ty: normalize(store, substitution, element.ty, states),
                    })
                    .collect::<Vec<_>>();
                store.tuple(elements.into_boxed_slice())
            }
            TypeData::Record(row_id) => {
                let (fields, tail) = {
                    let row = store.record_row(row_id);
                    (row.fields.to_vec(), row.tail)
                };
                let fields = fields
                    .into_iter()
                    .map(|field| crate::types::row::RecordRowField {
                        name: field.name,
                        ty: normalize(store, substitution, field.ty, states),
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
            TypeData::Never | TypeData::Unit | TypeData::ClassObject { .. } | TypeData::Nominal { .. } | TypeData::Lambda(_) | TypeData::SelfType(_) => ty,
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
            for &argument in arguments.iter() {
                collect_type_parameters(store, argument, output);
            }
        }
        TypeData::ExactCase { enum_type, .. } => collect_type_parameters(store, *enum_type, output),
        TypeData::Union(members) => {
            for &member in members.iter() {
                collect_type_parameters(store, member, output);
            }
        }
        TypeData::Tuple(elements) => {
            for element in elements.iter() {
                collect_type_parameters(store, element.ty, output);
            }
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
        TypeData::Never
        | TypeData::Unit
        | TypeData::ClassObject { .. }
        | TypeData::Nominal { .. }
        | TypeData::Family(_)
        | TypeData::Lambda(_)
        | TypeData::SelfType(_) => {}
    }
}

/// Unifies two canonical type terms under an occurs-checked parameter substitution.
///
/// Only equality can establish a GADT case. Nominal subtyping deliberately does
/// not participate here: a superclass relationship does not prove type equality.
fn unify_equality(store: &mut TypeStore, substitution: &mut TypeSubstitution, left: TypeId, right: TypeId) -> bool {
    let left = apply_substitution_to_fixpoint(store, substitution, left);
    let right = apply_substitution_to_fixpoint(store, substitution, right);
    if left == right {
        return true;
    }

    match (store.get(left).clone(), store.get(right).clone()) {
        (TypeData::Parameter(parameter), _) => bind_parameter(store, substitution, parameter, right),
        (_, TypeData::Parameter(parameter)) => bind_parameter(store, substitution, parameter, left),
        (
            TypeData::Applied {
                origin: left_origin,
                arguments: left_arguments,
            },
            TypeData::Applied {
                origin: right_origin,
                arguments: right_arguments,
            },
        ) => {
            left_arguments.len() == right_arguments.len()
                && unify_equality(store, substitution, left_origin, right_origin)
                && left_arguments
                    .iter()
                    .zip(right_arguments.iter())
                    .all(|(&left, &right)| unify_equality(store, substitution, left, right))
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
        ) => left_variant == right_variant && unify_equality(store, substitution, left_enum, right_enum),
        (TypeData::Union(left_members), TypeData::Union(right_members)) => {
            left_members.len() == right_members.len()
                && left_members
                    .iter()
                    .zip(right_members.iter())
                    .all(|(&left, &right)| unify_equality(store, substitution, left, right))
        }
        (TypeData::Tuple(left_elements), TypeData::Tuple(right_elements)) => {
            left_elements.len() == right_elements.len()
                && left_elements
                    .iter()
                    .zip(right_elements.iter())
                    .all(|(left, right)| left.label == right.label && unify_equality(store, substitution, left.ty, right.ty))
        }
        (TypeData::Record(left_row_id), TypeData::Record(right_row_id)) => unify_record_rows(store, substitution, left_row_id, right_row_id),
        (TypeData::Callable(left_callable), TypeData::Callable(right_callable)) => {
            left_callable.parameters.len() == right_callable.parameters.len()
                && left_callable
                    .parameters
                    .iter()
                    .zip(right_callable.parameters.iter())
                    .all(|(left, right)| left.label == right.label && left.rest == right.rest && unify_equality(store, substitution, left.ty, right.ty))
                && unify_equality(store, substitution, left_callable.return_type, right_callable.return_type)
        }
        // The remaining canonical forms contain no ordinary proper-type
        // parameter positions that this v1 GADT solver is permitted to rewrite.
        // Since unequal TypeIds reached this arm, equality is refuted.
        _ => false,
    }
}

fn unify_record_rows(
    store: &mut TypeStore,
    substitution: &mut TypeSubstitution,
    left_row_id: crate::types::id::RecordRowId,
    right_row_id: crate::types::id::RecordRowId,
) -> bool {
    let left = store.record_row(left_row_id).clone();
    let right = store.record_row(right_row_id).clone();

    for left_field in left.fields.iter() {
        if let Some(right_field) = right.fields.iter().find(|field| field.name == left_field.name) {
            if !unify_equality(store, substitution, left_field.ty, right_field.ty) {
                return false;
            }
        } else if left.tail == RecordRowTail::Closed && right.tail == RecordRowTail::Closed {
            return false;
        }
    }
    for right_field in right.fields.iter() {
        if !left.fields.iter().any(|field| field.name == right_field.name) && left.tail == RecordRowTail::Closed && right.tail == RecordRowTail::Closed {
            return false;
        }
    }

    // Open row parameters can absorb unmatched fields. Their substitution is
    // owned by the row solver, not the proper-type substitution used here; the
    // equality proof therefore records compatibility without fabricating a
    // proper type for a row parameter.
    true
}

fn bind_parameter(store: &mut TypeStore, substitution: &mut TypeSubstitution, parameter: TypeParameterId, ty: TypeId) -> bool {
    if let Some(existing) = substitution.get(parameter) {
        return unify_equality(store, substitution, existing, ty);
    }

    let ty = apply_substitution_to_fixpoint(store, substitution, ty);
    let parameter_ty = store.parameter_form(parameter);
    if ty == parameter_ty {
        return true;
    }
    if store.contains_type_parameter(ty, parameter) {
        return false;
    }
    substitution.bind(parameter, ty);
    true
}
