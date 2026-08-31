//! GADT branch proof engine and equality refinement (Part 05.1).

use crate::enum_semantics::VariantInfo;
use crate::identity::DeclarationId;
use crate::match_semantics::BranchProofEnvironment;
use crate::types::constraint::TypeConstraint;
use crate::types::id::TypeId;
use crate::types::relation::{TypeHierarchy, is_subtype};
use crate::types::store::TypeStore;

/// Result of evaluating GADT specialization and reachability for a variant case.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GadtProofResult {
    /// Case is reachable with the given branch proof environment and specialized exact case.
    Reachable {
        proof: BranchProofEnvironment,
        exact_case: TypeId,
    },
    /// Case is contradictory / impossible under the scrutinee type.
    Refuted,
}

/// Solves GADT equality constraints between a variant's case environment and the scrutinee type.
pub fn solve_gadt_branch_proof(
    store: &mut TypeStore,
    hier: &dyn TypeHierarchy,
    owner_decl: &DeclarationId,
    variant_info: &VariantInfo,
    scrutinee_ty: TypeId,
) -> GadtProofResult {
    // If the variant has no GADT case constraints, it is always reachable
    if variant_info.case_environment.is_empty() {
        let exact_case = store.exact_case_type(&variant_info.id, scrutinee_ty)
            .unwrap_or(variant_info.exact_case_template);
        return GadtProofResult::Reachable {
            proof: BranchProofEnvironment::default(),
            exact_case,
        };
    }

    // Inspect scrutinee applied nominal type arguments
    let scrutinee_args = match store.applied_nominal_parts(scrutinee_ty) {
        Some((decl, args)) if decl == *owner_decl => args.to_vec(),
        _ => Vec::new(),
    };

    let mut bindings = variant_info.case_environment.bindings.clone();
    let mut equalities = Vec::new();

    // Check consistency between case bindings and scrutinee arguments
    for (&param_id, &case_ty) in &variant_info.case_environment.bindings {
        let param_index = param_id.index() as usize;
        if let Some(&scrutinee_arg_ty) = scrutinee_args.get(param_index) {
            // Check if case_ty is compatible with scrutinee_arg_ty
            if case_ty != scrutinee_arg_ty {
                // If neither is a type parameter and types are distinct without subtyping:
                let sub_left = is_subtype(store, hier, case_ty, scrutinee_arg_ty);
                let sub_right = is_subtype(store, hier, scrutinee_arg_ty, case_ty);
                if !sub_left && !sub_right {
                    return GadtProofResult::Refuted;
                }
            }
            equalities.push(TypeConstraint::Equal(case_ty, scrutinee_arg_ty));
            bindings.insert(param_id, scrutinee_arg_ty);
        } else {
            equalities.push(TypeConstraint::Equal(store.parameter_form(param_id), case_ty));
        }
    }

    let exact_case = store.exact_case_type(&variant_info.id, scrutinee_ty)
        .unwrap_or(variant_info.exact_case_template);

    GadtProofResult::Reachable {
        proof: BranchProofEnvironment {
            bindings,
            equalities: equalities.into_boxed_slice(),
        },
        exact_case,
    }
}
