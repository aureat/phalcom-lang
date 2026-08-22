//! Type constraints and local constraint solving for Phase 2 inference.

use super::id::{InferVarId, TypeId};
use super::relation::{TypeHierarchy, is_subtype};
use super::store::{CallableParameterType, CallableType, RecordTypeField, TupleTypeElement, TypeData, TypeStore};
use phalcom_common::selector::Selector;
use std::collections::HashMap;

/// A static type constraint generated during expression analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeConstraint {
    /// Two types must be identical (unification).
    Equal(TypeId, TypeId),
    /// Left type must be a subtype of right type.
    Subtype(TypeId, TypeId),
    /// Receiver type must support a member/selector.
    HasMember(TypeId, Selector),
}

/// A set of accumulated type constraints.
#[derive(Clone, Debug, Default)]
pub struct ConstraintSet {
    pub constraints: Vec<TypeConstraint>,
}

impl ConstraintSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, constraint: TypeConstraint) {
        self.constraints.push(constraint);
    }

    pub fn extend(&mut self, other: ConstraintSet) {
        self.constraints.extend(other.constraints);
    }

    pub fn is_empty(&self) -> bool {
        self.constraints.is_empty()
    }

    pub fn len(&self) -> usize {
        self.constraints.len()
    }
}

/// Local constraint solver and substitution map for type inference.
#[derive(Clone, Debug, Default)]
pub struct LocalConstraintSolver {
    pub substitutions: HashMap<InferVarId, TypeId>,
    next_var_index: u32,
}

impl LocalConstraintSolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocates a fresh inference variable and interns it in the TypeStore.
    pub fn fresh_var(&mut self, store: &mut TypeStore) -> (InferVarId, TypeId) {
        let var = InferVarId::from_index(self.next_var_index as usize);
        self.next_var_index += 1;
        let ty = store.infer(var);
        (var, ty)
    }

    /// Records a substitution binding `var := ty`.
    pub fn bind(&mut self, var: InferVarId, ty: TypeId) {
        self.substitutions.insert(var, ty);
    }

    /// Resolves an inference variable through the substitution map.
    pub fn resolve(&self, ty: TypeId, store: &TypeStore) -> TypeId {
        match store.get(ty) {
            TypeData::Infer(var) => {
                if let Some(&target) = self.substitutions.get(var) {
                    if target != ty {
                        return self.resolve(target, store);
                    }
                }
                ty
            }
            _ => ty,
        }
    }

    /// Deeply substitutes all solved inference variables within a composite type.
    pub fn substitute_type(&self, ty: TypeId, store: &mut TypeStore) -> TypeId {
        let resolved = self.resolve(ty, store);
        match store.get(resolved).clone() {
            TypeData::Applied { origin, arguments } => {
                let subst_origin = self.substitute_type(origin, store);
                let subst_args: Vec<TypeId> = arguments.iter().map(|&arg| self.substitute_type(arg, store)).collect();
                store.applied(subst_origin, subst_args.into_boxed_slice())
            }
            TypeData::Union(members) => {
                let subst_members: Vec<TypeId> = members.iter().map(|&m| self.substitute_type(m, store)).collect();
                store.union(&subst_members)
            }
            TypeData::Tuple(elements) => {
                let subst_elems: Vec<TupleTypeElement> = elements
                    .iter()
                    .map(|elem| TupleTypeElement {
                        label: elem.label.clone(),
                        ty: self.substitute_type(elem.ty, store),
                    })
                    .collect();
                store.tuple(subst_elems.into_boxed_slice())
            }
            TypeData::Record(fields) => {
                let subst_fields: Vec<RecordTypeField> = fields
                    .iter()
                    .map(|f| RecordTypeField {
                        name: f.name.clone(),
                        ty: self.substitute_type(f.ty, store),
                    })
                    .collect();
                store.record(subst_fields.into_boxed_slice())
            }
            TypeData::Callable(callable) => {
                let subst_params: Vec<CallableParameterType> = callable
                    .parameters
                    .iter()
                    .map(|p| CallableParameterType {
                        label: p.label.clone(),
                        ty: self.substitute_type(p.ty, store),
                        rest: p.rest,
                    })
                    .collect();
                let subst_ret = self.substitute_type(callable.return_type, store);
                store.callable(CallableType {
                    parameters: subst_params.into_boxed_slice(),
                    return_type: subst_ret,
                })
            }
            _ => resolved,
        }
    }

    /// Solves the accumulated constraints, updating substitutions.
    pub fn solve(&mut self, set: &ConstraintSet, store: &mut TypeStore, hierarchy: &dyn TypeHierarchy) -> bool {
        for constraint in &set.constraints {
            match constraint {
                TypeConstraint::Equal(a, b) => {
                    let a_res = self.resolve(*a, store);
                    let b_res = self.resolve(*b, store);
                    if !self.unify(a_res, b_res, store) {
                        return false;
                    }
                }
                TypeConstraint::Subtype(sub, sup) => {
                    let sub_res = self.resolve(*sub, store);
                    let sup_res = self.resolve(*sup, store);
                    if let TypeData::Infer(var) = store.get(sub_res) {
                        self.bind(*var, sup_res);
                    } else if let TypeData::Infer(var) = store.get(sup_res) {
                        self.bind(*var, sub_res);
                    } else if !is_subtype(store, hierarchy, sub_res, sup_res) {
                        return false;
                    }
                }
                TypeConstraint::HasMember(_, _) => {
                    // Validated by dispatch/surface lookup
                }
            }
        }
        true
    }

    /// Unifies two types, recording bindings when inference variables are encountered.
    pub fn unify(&mut self, a: TypeId, b: TypeId, store: &mut TypeStore) -> bool {
        let a_res = self.resolve(a, store);
        let b_res = self.resolve(b, store);

        if a_res == b_res {
            return true;
        }

        match (store.get(a_res).clone(), store.get(b_res).clone()) {
            (TypeData::Infer(var), _) => {
                self.bind(var, b_res);
                true
            }
            (_, TypeData::Infer(var)) => {
                self.bind(var, a_res);
                true
            }
            (TypeData::Applied { origin: o1, arguments: args1 }, TypeData::Applied { origin: o2, arguments: args2 }) => {
                if o1 != o2 || args1.len() != args2.len() {
                    return false;
                }
                for (&arg1, &arg2) in args1.iter().zip(args2.iter()) {
                    if !self.unify(arg1, arg2, store) {
                        return false;
                    }
                }
                true
            }
            (TypeData::Tuple(elems1), TypeData::Tuple(elems2)) => {
                if elems1.len() != elems2.len() {
                    return false;
                }
                for (e1, e2) in elems1.iter().zip(elems2.iter()) {
                    if e1.label != e2.label || !self.unify(e1.ty, e2.ty, store) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::DeclarationId;
    use crate::types::relation::MapTypeHierarchy;
    use phalcom_modules::identity::ModuleId;

    fn test_decl(name: &str) -> DeclarationId {
        let module = ModuleId::core();
        DeclarationId::new(module, name.into())
    }

    #[test]
    fn solves_list_element_unification() {
        let mut store = TypeStore::new();
        let hier = MapTypeHierarchy::new();
        let mut solver = LocalConstraintSolver::new();

        let int_decl = test_decl("Int");
        let list_decl = test_decl("List");
        let t_int = store.nominal(int_decl);

        let (_var, t_infer) = solver.fresh_var(&mut store);
        let list_infer = store.list_of(list_decl.clone(), t_infer);
        let list_int = store.list_of(list_decl, t_int);

        let mut constraints = ConstraintSet::new();
        constraints.add(TypeConstraint::Equal(list_infer, list_int));

        assert!(solver.solve(&constraints, &mut store, &hier));
        let resolved = solver.substitute_type(list_infer, &mut store);
        assert_eq!(resolved, list_int);
    }
}
