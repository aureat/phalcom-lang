//! GADT branch proof engine and equality refinement (Part 05.1).

use crate::enum_semantics::VariantInfo;
use crate::identity::DeclarationId;
use crate::match_semantics::BranchProofEnvironment;
use crate::types::constraint::TypeConstraint;
use crate::types::id::{TypeId, TypeParameterId};
use crate::types::relation::TypeHierarchy;
use crate::types::store::{TypeData, TypeStore};
use crate::types::substitution::TypeSubstitution;

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
    let mut current = ty;
    // A valid occurs-checked substitution cannot contain a cycle. The bound keeps
    // this helper total even if an invalid environment reaches it through recovery.
    for _ in 0..64 {
        let next = substitution.apply(store, current);
        if next == current {
            return current;
        }
        current = next;
    }
    current
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
