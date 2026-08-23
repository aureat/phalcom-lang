//! Solver-local type inference session and term calculus (Spec 04.5).
//!
//! Law: InferVarId != TypeId. Inference variables are session-local reasoning entities,
//! never interned into canonical TypeStore or published in snapshots.

use crate::identity::{CallableId, ExplanationId, ExpressionId, InferVarId};
use crate::types::id::{KindId, TypeId, TypeParameterId};
use crate::types::outcome::{BlockReason, BudgetReport};
use crate::types::relation::{TypeHierarchy, is_subtype};
use crate::types::store::{CallableParameterType, CallableType, TupleTypeElement, TypeData, TypeStore};
use std::collections::HashMap;

/// A local inference term representing a canonical type, a solver variable, or a compound inference form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceTerm {
    Canonical(TypeId),
    Var(InferVarId),
    Applied {
        origin: Box<InferenceTerm>,
        arguments: Box<[InferenceTerm]>,
    },
    Union(Box<[InferenceTerm]>),
    Tuple(Box<[InferenceTupleElement]>),
    Callable(InferenceCallable),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceTupleElement {
    pub label: Option<Box<str>>,
    pub term: InferenceTerm,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceCallableParameter {
    pub label: Option<Box<str>>,
    pub term: InferenceTerm,
    pub rest: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceCallable {
    pub parameters: Box<[InferenceCallableParameter]>,
    pub return_type: Box<InferenceTerm>,
}

/// State of an inference variable in a session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferVarState {
    Unsolved,
    Solved(TypeId),
    Failed(InferenceFailureReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceFailureReason {
    OccursCheck,
    KindMismatch { expected: KindId, actual: KindId },
    ConflictingBounds { lower: TypeId, upper: TypeId },
}

/// Description of an inference variable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceVariable {
    pub id: InferVarId,
    pub kind: KindId,
    pub state: InferVarState,
}

/// Kind of constraint relation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceRelation {
    Equivalent(InferenceTerm, InferenceTerm),
    Subtype(InferenceTerm, InferenceTerm),
}

/// The origin of an inference constraint for causality and explanation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConstraintOrigin {
    Argument {
        call: ExpressionId,
        argument: ExpressionId,
        parameter_index: u16,
    },
    ExpectedResult {
        expression: ExpressionId,
    },
    BlockParameter {
        block: ExpressionId,
        parameter_index: u16,
    },
    BlockResult {
        block: ExpressionId,
    },
    CollectionElement {
        literal: ExpressionId,
    },
    GenericWhere {
        callable: CallableId,
        constraint_index: u16,
    },
    Explicit,
}

/// Structured constraint tracked in an inference session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceConstraint {
    pub relation: InferenceRelation,
    pub origin: ConstraintOrigin,
    pub explanation: Option<ExplanationId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceSolution {
    pub substitutions: HashMap<InferVarId, TypeId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnderconstrainedInference {
    pub unsolved_vars: Vec<InferVarId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceConflict {
    pub var: InferVarId,
    pub reason: InferenceFailureReason,
}

/// Outcome of solving an inference session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceOutcome {
    Solved(InferenceSolution),
    Underconstrained(UnderconstrainedInference),
    Conflicting(InferenceConflict),
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
}

impl InferenceOutcome {
    pub fn is_solved(&self) -> bool {
        matches!(self, Self::Solved(_))
    }
}

use crate::types::parameter::{GenericSignature, TypeTerm};

/// Solver-local inference session.
#[derive(Clone, Debug, Default)]
pub struct InferenceSession {
    variables: Vec<InferenceVariable>,
    constraints: Vec<InferenceConstraint>,
    substitutions: HashMap<InferVarId, TypeId>,
    lower_bounds: HashMap<InferVarId, Vec<TypeId>>,
    upper_bounds: HashMap<InferVarId, Vec<TypeId>>,
    var_aliases: HashMap<InferVarId, InferVarId>,
    var_terms: HashMap<InferVarId, InferenceTerm>,
    next_var_index: u32,
}

impl InferenceSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocates a fresh inference variable with the given kind.
    pub fn fresh_variable(&mut self, kind: KindId) -> InferVarId {
        let var = InferVarId::from_index(self.next_var_index as usize);
        self.next_var_index += 1;
        self.variables.push(InferenceVariable {
            id: var,
            kind,
            state: InferVarState::Unsolved,
        });
        var
    }

    /// Finds the canonical representative variable for `v`.
    pub fn find_var(&self, mut v: InferVarId) -> InferVarId {
        while let Some(&next) = self.var_aliases.get(&v) {
            if next == v {
                break;
            }
            v = next;
        }
        v
    }

    /// Adds a constraint to the session.
    pub fn add_constraint(&mut self, relation: InferenceRelation, origin: ConstraintOrigin, explanation: Option<ExplanationId>) {
        self.constraints.push(InferenceConstraint { relation, origin, explanation });
    }

    /// Instantiates fresh inference variables for each generic parameter in `generic_sig`.
    pub fn instantiate_generic_signature(
        &mut self,
        generic_sig: &GenericSignature,
    ) -> HashMap<TypeParameterId, InferenceTerm> {
        let mut map = HashMap::new();
        for &param in &generic_sig.parameters {
            let var = self.fresh_variable(KindId::TYPE);
            map.insert(param, InferenceTerm::Var(var));
        }
        map
    }

    /// Converts a canonical `TypeId` to an `InferenceTerm`, replacing generic parameters with their instantiated inference terms.
    pub fn type_id_to_inference(
        &self,
        ty: TypeId,
        subst: &HashMap<TypeParameterId, InferenceTerm>,
        store: &TypeStore,
    ) -> InferenceTerm {
        match store.get(ty) {
            TypeData::Parameter(p) => {
                if let Some(t) = subst.get(p) {
                    t.clone()
                } else {
                    InferenceTerm::Canonical(ty)
                }
            }
            TypeData::Applied { origin, arguments } => {
                let orig_term = self.type_id_to_inference(*origin, subst, store);
                let arg_terms: Vec<InferenceTerm> = arguments
                    .iter()
                    .map(|&a| self.type_id_to_inference(a, subst, store))
                    .collect();
                InferenceTerm::Applied {
                    origin: Box::new(orig_term),
                    arguments: arg_terms.into_boxed_slice(),
                }
            }
            TypeData::Union(members) => {
                let member_terms: Vec<InferenceTerm> = members
                    .iter()
                    .map(|&m| self.type_id_to_inference(m, subst, store))
                    .collect();
                InferenceTerm::Union(member_terms.into_boxed_slice())
            }
            TypeData::Tuple(elems) => {
                let elem_terms: Vec<InferenceTupleElement> = elems
                    .iter()
                    .map(|e| InferenceTupleElement {
                        label: e.label.clone(),
                        term: self.type_id_to_inference(e.ty, subst, store),
                    })
                    .collect();
                InferenceTerm::Tuple(elem_terms.into_boxed_slice())
            }
            TypeData::Callable(c) => {
                let param_terms: Vec<InferenceCallableParameter> = c
                    .parameters
                    .iter()
                    .map(|p| InferenceCallableParameter {
                        label: p.label.clone(),
                        term: self.type_id_to_inference(p.ty, subst, store),
                        rest: p.rest,
                    })
                    .collect();
                let ret_term = self.type_id_to_inference(c.return_type, subst, store);
                InferenceTerm::Callable(InferenceCallable {
                    parameters: param_terms.into_boxed_slice(),
                    return_type: Box::new(ret_term),
                })
            }
            _ => InferenceTerm::Canonical(ty),
        }
    }

    /// Converts a `TypeTerm` to an `InferenceTerm`.
    pub fn type_term_to_inference(
        &self,
        term: &TypeTerm,
        subst: &HashMap<TypeParameterId, InferenceTerm>,
        store: &TypeStore,
    ) -> InferenceTerm {
        match term {
            TypeTerm::Canonical(ty) => self.type_id_to_inference(*ty, subst, store),
            TypeTerm::SelfType(_) => InferenceTerm::Canonical(store.unit()),
            TypeTerm::Infer(v) => InferenceTerm::Var(*v),
        }
    }

    /// Solves all accumulated constraints.
    pub fn solve(&mut self, store: &mut TypeStore, hierarchy: &dyn TypeHierarchy) -> InferenceOutcome {
        // Multi-pass iterative constraint solving
        let max_passes = 16;
        for _ in 0..max_passes {
            let mut changed = false;
            let constraints = self.constraints.clone();
            for constraint in &constraints {
                match &constraint.relation {
                    InferenceRelation::Equivalent(left, right) => {
                        if !self.unify_terms(left, right, store) {
                            return InferenceOutcome::Conflicting(InferenceConflict {
                                var: InferVarId::from_index(0),
                                reason: InferenceFailureReason::ConflictingBounds {
                                    lower: store.never(),
                                    upper: store.never(),
                                },
                            });
                        }
                    }
                    InferenceRelation::Subtype(sub, sup) => {
                        if !self.subtype_terms(sub, sup, store, hierarchy) {
                            return InferenceOutcome::Conflicting(InferenceConflict {
                                var: InferVarId::from_index(0),
                                reason: InferenceFailureReason::ConflictingBounds {
                                    lower: store.never(),
                                    upper: store.never(),
                                },
                            });
                        }
                    }
                }
            }

            // Try to resolve remaining var_terms
            let var_terms = self.var_terms.clone();
            for (var, term) in var_terms {
                let rep = self.find_var(var);
                if !self.substitutions.contains_key(&rep) {
                    if let Ok(ty) = self.materialize(&term, store) {
                        if self.bind(rep, ty, store) {
                            changed = true;
                        }
                    }
                }
            }

            // Try to resolve from lower/upper bounds
            let vars_to_check: Vec<InferVarId> = self.variables.iter().map(|v| self.find_var(v.id)).collect();
            for rep in vars_to_check {
                if !self.substitutions.contains_key(&rep) {
                    if let Some(lowers) = self.lower_bounds.get(&rep).cloned() {
                        if !lowers.is_empty() {
                            let candidate = store.union(&lowers);
                            if let Some(uppers) = self.upper_bounds.get(&rep) {
                                let mut ok = true;
                                for &upper in uppers {
                                    if !is_subtype(store, hierarchy, candidate, upper) {
                                        ok = false;
                                        break;
                                    }
                                }
                                if !ok {
                                    return InferenceOutcome::Conflicting(InferenceConflict {
                                        var: rep,
                                        reason: InferenceFailureReason::ConflictingBounds {
                                            lower: candidate,
                                            upper: uppers[0],
                                        },
                                    });
                                }
                            }
                            if self.bind(rep, candidate, store) {
                                changed = true;
                            }
                        }
                    } else if let Some(uppers) = self.upper_bounds.get(&rep).cloned() {
                        if uppers.len() == 1 {
                            if self.bind(rep, uppers[0], store) {
                                changed = true;
                            }
                        }
                    }
                }
            }

            if !changed {
                break;
            }
        }

        // Propagate solutions across alias classes
        for var in &self.variables {
            let rep = self.find_var(var.id);
            if let Some(&ty) = self.substitutions.get(&rep) {
                self.substitutions.insert(var.id, ty);
            }
        }

        // Check for unsolved variables
        let mut unsolved = Vec::new();
        for var in &self.variables {
            if !self.substitutions.contains_key(&var.id) {
                unsolved.push(var.id);
            }
        }

        if !unsolved.is_empty() {
            InferenceOutcome::Underconstrained(UnderconstrainedInference { unsolved_vars: unsolved })
        } else {
            InferenceOutcome::Solved(InferenceSolution {
                substitutions: self.substitutions.clone(),
            })
        }
    }

    /// Binds an inference variable to a canonical type, with occurs check.
    pub fn bind(&mut self, var: InferVarId, ty: TypeId, store: &TypeStore) -> bool {
        let rep = self.find_var(var);
        if self.occurs_in_type(rep, ty, store) {
            return false;
        }
        self.substitutions.insert(rep, ty);
        if let Some(v) = self.variables.iter_mut().find(|v| v.id == rep) {
            v.state = InferVarState::Solved(ty);
        }
        true
    }

    /// Checks if `var` occurs in `ty`.
    #[allow(clippy::only_used_in_recursion)]
    pub fn occurs_in_type(&self, var: InferVarId, ty: TypeId, store: &TypeStore) -> bool {
        match store.get(ty) {
            TypeData::Applied { origin, arguments } => {
                self.occurs_in_type(var, *origin, store) || arguments.iter().any(|&a| self.occurs_in_type(var, a, store))
            }
            TypeData::Union(members) => members.iter().any(|&m| self.occurs_in_type(var, m, store)),
            TypeData::Tuple(elems) => elems.iter().any(|e| self.occurs_in_type(var, e.ty, store)),
            TypeData::Record(row_id) => store.record_row(*row_id).fields.iter().any(|f| self.occurs_in_type(var, f.ty, store)),
            TypeData::Callable(c) => c.parameters.iter().any(|p| self.occurs_in_type(var, p.ty, store)) || self.occurs_in_type(var, c.return_type, store),
            _ => false,
        }
    }

    /// Checks if `var` occurs in an `InferenceTerm`.
    pub fn occurs_in_term(&self, var: InferVarId, term: &InferenceTerm) -> bool {
        let rep = self.find_var(var);
        match term {
            InferenceTerm::Canonical(_) => false,
            InferenceTerm::Var(v) => self.find_var(*v) == rep,
            InferenceTerm::Applied { origin, arguments } => self.occurs_in_term(rep, origin) || arguments.iter().any(|a| self.occurs_in_term(rep, a)),
            InferenceTerm::Union(members) => members.iter().any(|m| self.occurs_in_term(rep, m)),
            InferenceTerm::Tuple(elems) => elems.iter().any(|e| self.occurs_in_term(rep, &e.term)),
            InferenceTerm::Callable(c) => c.parameters.iter().any(|p| self.occurs_in_term(rep, &p.term)) || self.occurs_in_term(rep, &c.return_type),
        }
    }

    fn unify_terms(&mut self, left: &InferenceTerm, right: &InferenceTerm, store: &mut TypeStore) -> bool {
        match (left, right) {
            (InferenceTerm::Var(v1), InferenceTerm::Var(v2)) => {
                let rep1 = self.find_var(*v1);
                let rep2 = self.find_var(*v2);
                if rep1 == rep2 {
                    return true;
                }
                if let Some(ty1) = self.substitutions.get(&rep1).copied() {
                    let canon = InferenceTerm::Canonical(ty1);
                    return self.unify_terms(&canon, right, store);
                }
                if let Some(ty2) = self.substitutions.get(&rep2).copied() {
                    let canon = InferenceTerm::Canonical(ty2);
                    return self.unify_terms(left, &canon, store);
                }
                self.var_aliases.insert(rep1, rep2);
                // Merge bounds
                if let Some(lowers) = self.lower_bounds.remove(&rep1) {
                    self.lower_bounds.entry(rep2).or_default().extend(lowers);
                }
                if let Some(uppers) = self.upper_bounds.remove(&rep1) {
                    self.upper_bounds.entry(rep2).or_default().extend(uppers);
                }
                true
            }
            (InferenceTerm::Var(v), term) | (term, InferenceTerm::Var(v)) => {
                let rep = self.find_var(*v);
                if let Some(ty) = self.substitutions.get(&rep).copied() {
                    let canon = InferenceTerm::Canonical(ty);
                    self.unify_terms(&canon, term, store)
                } else if self.occurs_in_term(rep, term) {
                    false
                } else if let Ok(ty) = self.materialize(term, store) {
                    self.bind(rep, ty, store)
                } else {
                    self.var_terms.insert(rep, term.clone());
                    true
                }
            }
            (InferenceTerm::Canonical(t1), InferenceTerm::Canonical(t2)) => *t1 == *t2,
            (InferenceTerm::Applied { origin: o1, arguments: a1 }, InferenceTerm::Applied { origin: o2, arguments: a2 }) => {
                if a1.len() != a2.len() || !self.unify_terms(o1, o2, store) {
                    return false;
                }
                for (arg1, arg2) in a1.iter().zip(a2.iter()) {
                    if !self.unify_terms(arg1, arg2, store) {
                        return false;
                    }
                }
                true
            }
            (InferenceTerm::Canonical(ty), InferenceTerm::Applied { origin, arguments })
            | (InferenceTerm::Applied { origin, arguments }, InferenceTerm::Canonical(ty)) => {
                if let TypeData::Applied {
                    origin: orig_ty,
                    arguments: args_ty,
                } = store.get(*ty).clone()
                {
                    if args_ty.len() != arguments.len() {
                        return false;
                    }
                    let orig_term = InferenceTerm::Canonical(orig_ty);
                    if !self.unify_terms(origin, &orig_term, store) {
                        return false;
                    }
                    for (arg_term, &arg_ty) in arguments.iter().zip(args_ty.iter()) {
                        let canon = InferenceTerm::Canonical(arg_ty);
                        if !self.unify_terms(arg_term, &canon, store) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn subtype_terms(&mut self, sub: &InferenceTerm, sup: &InferenceTerm, store: &mut TypeStore, hier: &dyn TypeHierarchy) -> bool {
        match (sub, sup) {
            (InferenceTerm::Var(v), term) => {
                let rep = self.find_var(*v);
                if let Some(ty) = self.substitutions.get(&rep).copied() {
                    let canon = InferenceTerm::Canonical(ty);
                    self.subtype_terms(&canon, term, store, hier)
                } else if let Ok(ty) = self.materialize(term, store) {
                    self.upper_bounds.entry(rep).or_default().push(ty);
                    true
                } else {
                    self.unify_terms(sub, sup, store)
                }
            }
            (term, InferenceTerm::Var(v)) => {
                let rep = self.find_var(*v);
                if let Some(ty) = self.substitutions.get(&rep).copied() {
                    let canon = InferenceTerm::Canonical(ty);
                    self.subtype_terms(term, &canon, store, hier)
                } else if let Ok(ty) = self.materialize(term, store) {
                    self.lower_bounds.entry(rep).or_default().push(ty);
                    true
                } else {
                    self.unify_terms(sub, sup, store)
                }
            }
            (InferenceTerm::Canonical(t1), InferenceTerm::Canonical(t2)) => is_subtype(store, hier, *t1, *t2),
            _ => self.unify_terms(sub, sup, store),
        }
    }

    /// Materializes an `InferenceTerm` into a canonical `TypeId`, substituting solved variables.
    pub fn materialize(&self, term: &InferenceTerm, store: &mut TypeStore) -> Result<TypeId, UnderconstrainedInference> {
        match term {
            InferenceTerm::Canonical(ty) => Ok(*ty),
            InferenceTerm::Var(v) => {
                let rep = self.find_var(*v);
                if let Some(&ty) = self.substitutions.get(&rep) {
                    Ok(ty)
                } else {
                    Err(UnderconstrainedInference { unsolved_vars: vec![*v] })
                }
            }
            InferenceTerm::Applied { origin, arguments } => {
                let orig_ty = self.materialize(origin, store)?;
                let mut arg_tys = Vec::with_capacity(arguments.len());
                for arg in arguments.iter() {
                    arg_tys.push(self.materialize(arg, store)?);
                }
                store
                    .apply_type_form(orig_ty, &arg_tys)
                    .map_err(|_| UnderconstrainedInference { unsolved_vars: Vec::new() })
            }
            InferenceTerm::Union(members) => {
                let mut member_tys = Vec::with_capacity(members.len());
                for m in members.iter() {
                    member_tys.push(self.materialize(m, store)?);
                }
                Ok(store.union(&member_tys))
            }
            InferenceTerm::Tuple(elems) => {
                let mut tuple_elems = Vec::with_capacity(elems.len());
                for e in elems.iter() {
                    tuple_elems.push(TupleTypeElement {
                        label: e.label.clone(),
                        ty: self.materialize(&e.term, store)?,
                    });
                }
                Ok(store.tuple(tuple_elems.into_boxed_slice()))
            }
            InferenceTerm::Callable(c) => {
                let mut params = Vec::with_capacity(c.parameters.len());
                for p in c.parameters.iter() {
                    params.push(CallableParameterType {
                        label: p.label.clone(),
                        ty: self.materialize(&p.term, store)?,
                        rest: p.rest,
                    });
                }
                let ret = self.materialize(&c.return_type, store)?;
                Ok(store.callable(CallableType {
                    parameters: params.into_boxed_slice(),
                    return_type: ret,
                }))
            }
        }
    }
}

