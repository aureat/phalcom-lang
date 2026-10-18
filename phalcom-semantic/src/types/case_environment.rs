//! GADT case equality environment computation and normalization.

use crate::identity::DeclarationId;
use crate::ordered_map::OrderedMap;
use crate::types::id::{KindId, TypeId, TypeParameterId};
use crate::types::parameter::{GenericConstraint, TypeTerm};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use crate::types::store::{TypeData, TypeStore};
use crate::types::substitution::TypeSubstitution;
use std::collections::BTreeMap;

/// Per-variant GADT type equality environment.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct CaseTypeEnvironment {
    pub bindings: OrderedMap<TypeParameterId, TypeId>,
    pub equalities: Box<[GenericConstraint]>,
}

impl CaseTypeEnvironment {
    pub fn new(bindings: impl Into<OrderedMap<TypeParameterId, TypeId>>, equalities: Box<[GenericConstraint]>) -> Self {
        Self { bindings: bindings.into(), equalities }
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty() && self.equalities.is_empty()
    }

    pub(crate) fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        let mut bindings = allocations.vector(self.bindings.len())?;
        for (parameter, ty) in self.bindings.iter() {
            bindings.push((*parameter, *ty));
        }
        let bindings = OrderedMap::from_frozen_entries_with_allocation_budget(bindings, allocations).map_err(|error| match error {
            crate::ordered_map::FrozenOrderedMapError::Allocation(error) => error,
            crate::ordered_map::FrozenOrderedMapError::DuplicateKey => AllocationError::Failed,
        })?;
        let mut equalities = allocations.vector(self.equalities.len())?;
        for equality in self.equalities.iter() {
            equalities.push(crate::types::parameter::clone_generic_constraint_with_allocation_budget(equality, allocations)?);
        }
        Ok(Self {
            bindings,
            equalities: allocations.boxed_slice(equalities)?,
        })
    }

    pub fn to_substitution(&self) -> TypeSubstitution {
        let mut subst = TypeSubstitution::new();
        for (&param, &ty) in &self.bindings {
            subst.bind(param, ty);
        }
        subst
    }
}

/// Errors derived during GADT case equality environment computation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaseEnvironmentError {
    ResultWrongOwner { expected: DeclarationId, got: DeclarationId },
    ResultUnsaturated { expected_arity: usize, got_arity: usize },
    ResultNotProper,
    CyclicEquality { parameter: TypeParameterId, rhs: TypeId },
}

/// Derives a [`CaseTypeEnvironment`] from an enum declaration's type parameters and an explicit/default result type.
pub fn derive_case_environment(
    store: &mut TypeStore,
    owner: &DeclarationId,
    type_parameters: &[TypeParameterId],
    result_type: Option<TypeId>,
) -> Result<CaseTypeEnvironment, CaseEnvironmentError> {
    let Some(result_ty) = result_type else {
        return Ok(CaseTypeEnvironment::default());
    };

    if store.kind_of(result_ty) != KindId::TYPE {
        return Err(CaseEnvironmentError::ResultNotProper);
    }

    let Some((origin, args)) = store.applied_nominal_parts(result_ty) else {
        return Err(CaseEnvironmentError::ResultWrongOwner {
            expected: owner.clone(),
            got: DeclarationId::new(owner.module.clone(), "<non-nominal>".into()),
        });
    };

    if origin != *owner {
        return Err(CaseEnvironmentError::ResultWrongOwner {
            expected: owner.clone(),
            got: origin,
        });
    }

    if args.len() != type_parameters.len() {
        return Err(CaseEnvironmentError::ResultUnsaturated {
            expected_arity: type_parameters.len(),
            got_arity: args.len(),
        });
    }

    let mut bindings: BTreeMap<TypeParameterId, TypeId> = BTreeMap::new();

    for (param_idx, &param_id) in type_parameters.iter().enumerate() {
        let arg_ty = args[param_idx];

        // Apply existing bindings to arg_ty
        let mut current_subst = TypeSubstitution::new();
        for (&p, &t) in &bindings {
            current_subst.bind(p, t);
        }
        let normalized_arg = current_subst.apply(store, arg_ty);

        // Self-equality: P_i == P_i contributes no additional constraint
        if let TypeData::Parameter(other_param) = store.get(normalized_arg) {
            if *other_param == param_id {
                continue;
            }
        }

        // Occurs check: param_id must not occur in normalized_arg
        if store.contains_type_parameter(normalized_arg, param_id) {
            return Err(CaseEnvironmentError::CyclicEquality {
                parameter: param_id,
                rhs: normalized_arg,
            });
        }

        // Update all existing bindings with param_id -> normalized_arg
        let mut new_subst = TypeSubstitution::new();
        new_subst.bind(param_id, normalized_arg);

        let mut updated_bindings = BTreeMap::new();
        for (&p, &t) in &bindings {
            let updated_t = new_subst.apply(store, t);
            if store.contains_type_parameter(updated_t, p) {
                return Err(CaseEnvironmentError::CyclicEquality { parameter: p, rhs: updated_t });
            }
            updated_bindings.insert(p, updated_t);
        }

        updated_bindings.insert(param_id, normalized_arg);
        bindings = updated_bindings;
    }

    let equalities: Vec<GenericConstraint> = bindings
        .iter()
        .map(|(&param, &ty)| {
            let param_form = store.parameter_form(param);
            GenericConstraint::Equivalent {
                left: TypeTerm::Canonical(param_form),
                right: TypeTerm::Canonical(ty),
            }
        })
        .collect();

    Ok(CaseTypeEnvironment {
        bindings: bindings.into(),
        equalities: equalities.into_boxed_slice(),
    })
}
