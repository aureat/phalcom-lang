//! Solver-local type inference session and term calculus (Spec 04.5).
//!
//! Law: InferVarId != TypeId. Inference variables are session-local reasoning entities,
//! never interned into canonical TypeStore or published in snapshots.

use super::context::CheckerControl;
use crate::identity::{CallableId, ExplanationId, ExpressionId, InferVarId};
use crate::types::evidence::{DynamicReason, EvidenceStatus, TypeKnowledge, UnknownReason};
use crate::types::id::{KindId, TypeId, TypeParameterId, VariantTypeId};
use crate::types::outcome::{BlockReason, BudgetReport};
use crate::types::relation::{TypeHierarchy, is_subtype};
use crate::types::store::{CallableParameterType, CallableType, TupleTypeElement, TypeData, TypeStore};
use std::collections::{HashMap, HashSet};

/// A local inference term representing a canonical type, a solver variable, or a compound inference form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceTerm {
    Canonical(TypeId),
    Var(InferVarId),
    Applied {
        origin: Box<InferenceTerm>,
        arguments: Box<[InferenceTerm]>,
    },
    ExactCase {
        variant: VariantTypeId,
        enum_type: Box<InferenceTerm>,
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

use phalcom_ast::ast::RestMode;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceCallableParameter {
    pub label: Option<Box<str>>,
    pub term: InferenceTerm,
    pub rest: RestMode,
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
    OccursCheck { var: InferVarId },
    KindMismatch { var: InferVarId, expected: KindId, actual: KindId },
    ConflictingBounds { var: InferVarId, lower: TypeId, upper: TypeId },
    MissingVariableMetadata { var: InferVarId },
    StructuralMismatch { left: Box<InferenceTerm>, right: Box<InferenceTerm> },
    UnresolvedSelf,
}

/// Description of an inference variable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceVariable {
    pub id: InferVarId,
    pub kind: KindId,
    pub state: InferVarState,
    pub support: Option<InferenceSupport>,
    /// Complete proof state for value premises that can influence this variable.
    pub proof: Option<InferenceProofState>,
}

/// Value evidence supporting an inferred generic variable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InferenceSupport {
    Established,
    Assumed,
}

impl InferenceSupport {
    fn join(self, other: Self) -> Self {
        if matches!(self, Self::Assumed) || matches!(other, Self::Assumed) {
            Self::Assumed
        } else {
            Self::Established
        }
    }
}

/// Proof state for required value premises participating in generic inference.
///
/// This domain is intentionally separate from solver support: a solved
/// substitution can still depend on an unavailable or dynamic source value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceProofState {
    Established,
    Assumed,
    Unknown(UnknownReason),
    Dynamic(DynamicReason),
}

impl InferenceProofState {
    /// Converts complete expression knowledge into solver-local proof state.
    pub fn from_knowledge(knowledge: &TypeKnowledge) -> Self {
        match knowledge {
            TypeKnowledge::Known(evidence) => match evidence.status() {
                EvidenceStatus::Established => Self::Established,
                EvidenceStatus::Assumed => Self::Assumed,
            },
            TypeKnowledge::Unknown(reason) => Self::Unknown(reason.clone()),
            TypeKnowledge::Dynamic(reason) => Self::Dynamic(reason.clone()),
        }
    }

    /// Meets required premises without discarding the weakest epistemic state.
    pub fn meet(self, other: Self) -> Self {
        use InferenceProofState::{Assumed, Dynamic, Established, Unknown};

        match (self, other) {
            (Unknown(left), Unknown(right)) => Unknown(crate::types::evidence::join_unknown_reason(left, right)),
            (Unknown(reason), _) | (_, Unknown(reason)) => Unknown(reason),
            (Dynamic(left), Dynamic(right)) => Dynamic(crate::types::evidence::join_dynamic_reason(left, right)),
            (Dynamic(reason), _) | (_, Dynamic(reason)) => Dynamic(reason),
            (Assumed, _) | (_, Assumed) => Assumed,
            (Established, Established) => Established,
        }
    }
}

/// A required generic value premise, retained even when no canonical type can
/// be extracted from its expression knowledge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredInferencePremise {
    pub term: InferenceTerm,
    pub origin: ConstraintOrigin,
    pub proof: InferenceProofState,
    pub explanation: Option<ExplanationId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InferenceSubtypeEdge {
    sub: InferVarId,
    sup: InferVarId,
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
    pub support: Option<InferenceSupport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceSolution {
    pub substitutions: HashMap<InferVarId, TypeId>,
    pub support: HashMap<InferVarId, InferenceSupport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnderconstrainedInference {
    pub unsolved_vars: Vec<InferVarId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceConflict {
    /// Deterministic bounded causal constraint set. The legacy single index is
    /// retained during v1 migration for callers that only need the failing edge.
    pub constraint_indices: Box<[u32]>,
    pub constraint_index: Option<u32>,
    pub origin: Option<ConstraintOrigin>,
    pub failure: InferenceFailureReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SolveEffect {
    Unchanged,
    Changed,
}

impl SolveEffect {
    fn is_changed(self) -> bool {
        matches!(self, Self::Changed)
    }
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
    InternalFailure(InferenceFailureReason),
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
    required_premises: Vec<RequiredInferencePremise>,
    subtype_edges: Vec<InferenceSubtypeEdge>,
    bound_origins: HashMap<(InferVarId, TypeId), (u32, ConstraintOrigin)>,
    variable_constraint_indices: HashMap<InferVarId, Vec<u32>>,
    next_var_index: u32,
}

impl InferenceSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocates a fresh inference variable with the given kind.
    pub fn fresh_variable(&mut self, kind: KindId) -> InferVarId {
        self.fresh_variable_with_support(kind, None)
    }

    /// Allocates an inference variable with explicit value support.
    pub fn fresh_variable_with_support(&mut self, kind: KindId, support: Option<InferenceSupport>) -> InferVarId {
        let var = InferVarId::from_index(self.next_var_index as usize);
        self.next_var_index += 1;
        self.variables.push(InferenceVariable {
            id: var,
            kind,
            state: InferVarState::Unsolved,
            support,
            proof: None,
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
        self.constraints.push(InferenceConstraint {
            relation,
            origin,
            explanation,
            support: None,
        });
    }

    /// Adds a constraint and records value support for any variables it reaches.
    pub fn add_constraint_with_support(
        &mut self,
        relation: InferenceRelation,
        origin: ConstraintOrigin,
        explanation: Option<ExplanationId>,
        support: InferenceSupport,
    ) {
        self.record_relation_support(&relation, support);
        self.constraints.push(InferenceConstraint {
            relation,
            origin,
            explanation,
            support: Some(support),
        });
    }

    /// Records one generic argument as a proof premise before any type-id
    /// filtering occurs.
    pub fn record_required_premise(&mut self, term: &InferenceTerm, origin: ConstraintOrigin, knowledge: &TypeKnowledge, explanation: Option<ExplanationId>) {
        let proof = InferenceProofState::from_knowledge(knowledge);
        self.record_term_proof_state(term, proof.clone());
        self.required_premises.push(RequiredInferencePremise {
            term: term.clone(),
            origin,
            proof,
            explanation,
        });
    }

    /// Returns proof state for all inference variables occurring in a term.
    pub fn proof_state_for_term(&self, term: &InferenceTerm) -> InferenceProofState {
        let mut variables = self.term_variables(term).into_iter();
        let Some(first) = variables.next() else {
            return InferenceProofState::Established;
        };
        let mut proof = self
            .variable_by_representative(first)
            .and_then(|variable| variable.proof.clone())
            .unwrap_or(InferenceProofState::Unknown(UnknownReason::UnderconstrainedTypeVariable));
        for variable in variables {
            let current = self
                .variable_by_representative(variable)
                .and_then(|candidate| candidate.proof.clone())
                .unwrap_or(InferenceProofState::Unknown(UnknownReason::UnderconstrainedTypeVariable));
            proof = proof.meet(current);
        }
        proof
    }

    pub fn projected_parameters_for_term(&self, term: &InferenceTerm, parameters: &HashMap<TypeParameterId, InferenceTerm>) -> Vec<TypeParameterId> {
        let variables = self.term_variables(term);
        let mut result: Vec<_> = parameters
            .iter()
            .filter_map(|(parameter, mapped)| match mapped {
                InferenceTerm::Var(variable) if variables.contains(&self.find_var(*variable)) => Some(*parameter),
                _ => None,
            })
            .collect();
        result.sort_by_key(|parameter| parameter.index());
        result
    }

    pub fn constraint_explanation_roots(&self, indices: &[u32]) -> Vec<ExplanationId> {
        let mut roots = indices
            .iter()
            .filter_map(|index| self.constraints.get(*index as usize).and_then(|constraint| constraint.explanation))
            .collect::<Vec<_>>();
        roots.sort_by_key(|id| id.0);
        roots.dedup();
        roots
    }

    pub fn constraint_origin(&self, index: u32) -> Option<&ConstraintOrigin> {
        self.constraints.get(index as usize).map(|constraint| &constraint.origin)
    }

    pub fn parameter_for_variable(&self, variable: InferVarId, parameters: &HashMap<TypeParameterId, InferenceTerm>) -> Option<TypeParameterId> {
        let representative = self.find_var(variable);
        let mut candidates = parameters
            .iter()
            .filter_map(|(parameter, term)| match term {
                InferenceTerm::Var(candidate) if self.find_var(*candidate) == representative => Some(*parameter),
                _ => None,
            })
            .collect::<Vec<_>>();
        candidates.sort_by_key(|parameter| parameter.index());
        candidates.into_iter().next()
    }

    pub fn all_constraint_explanation_roots(&self) -> Vec<ExplanationId> {
        let mut roots = self.constraints.iter().filter_map(|constraint| constraint.explanation).collect::<Vec<_>>();
        roots.sort_by_key(|id| id.0);
        roots.dedup();
        roots
    }

    pub fn projected_solution(
        &mut self,
        parameter: TypeParameterId,
        parameters: &HashMap<TypeParameterId, InferenceTerm>,
        store: &mut TypeStore,
    ) -> Option<TypeId> {
        let term = parameters.get(&parameter)?;
        self.materialize(term, store).ok()
    }

    fn variable_by_representative(&self, variable: InferVarId) -> Option<&InferenceVariable> {
        let representative = self.find_var(variable);
        self.variables.iter().find(|candidate| candidate.id == representative)
    }

    /// Instantiates fresh inference variables for each generic parameter in `generic_sig`.
    pub fn instantiate_generic_signature(&mut self, generic_sig: &GenericSignature, store: &TypeStore) -> HashMap<TypeParameterId, InferenceTerm> {
        let mut map = HashMap::new();
        for &param in &generic_sig.parameters {
            let var = self.fresh_variable(store.type_parameter(param).kind);
            map.insert(param, InferenceTerm::Var(var));
        }
        map
    }

    /// Converts a canonical `TypeId` to an `InferenceTerm`, replacing generic parameters with their instantiated inference terms.
    pub fn type_id_to_inference(&self, ty: TypeId, subst: &HashMap<TypeParameterId, InferenceTerm>, store: &TypeStore) -> InferenceTerm {
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
                let arg_terms: Vec<InferenceTerm> = arguments.iter().map(|&a| self.type_id_to_inference(a, subst, store)).collect();
                InferenceTerm::Applied {
                    origin: Box::new(orig_term),
                    arguments: arg_terms.into_boxed_slice(),
                }
            }
            TypeData::ExactCase { variant, enum_type } => {
                let enum_term = self.type_id_to_inference(*enum_type, subst, store);
                InferenceTerm::ExactCase {
                    variant: *variant,
                    enum_type: Box::new(enum_term),
                }
            }
            TypeData::Union(members) => {
                let member_terms: Vec<InferenceTerm> = members.iter().map(|&m| self.type_id_to_inference(m, subst, store)).collect();
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
    ) -> Result<InferenceTerm, InferenceFailureReason> {
        match term {
            TypeTerm::Canonical(ty) => Ok(self.type_id_to_inference(*ty, subst, store)),
            TypeTerm::SelfType(_) => Err(InferenceFailureReason::UnresolvedSelf),
            TypeTerm::Infer(v) => Ok(InferenceTerm::Var(*v)),
        }
    }

    /// Solves all accumulated constraints.
    pub fn solve(&mut self, store: &mut TypeStore, hierarchy: &dyn TypeHierarchy) -> InferenceOutcome {
        let control = CheckerControl::default();
        self.solve_with_control(store, hierarchy, &control)
    }

    /// Solves all accumulated constraints while consuming the caller's shared
    /// cancellation token and query budget.
    pub fn solve_with_control(&mut self, store: &mut TypeStore, hierarchy: &dyn TypeHierarchy, control: &CheckerControl) -> InferenceOutcome {
        // Multi-pass iterative constraint solving
        let max_passes = 16;
        let mut converged = false;
        for _ in 0..max_passes {
            if control.is_cancelled() {
                return InferenceOutcome::Cancelled;
            }
            let mut changed = false;
            let constraints = self.constraints.clone();
            for (constraint_index, constraint) in constraints.iter().enumerate() {
                if control.is_cancelled() {
                    return InferenceOutcome::Cancelled;
                }
                if let Err(report) = control.charge_step() {
                    return InferenceOutcome::BudgetExceeded(report);
                }
                let effect = match &constraint.relation {
                    InferenceRelation::Equivalent(left, right) => self.unify_terms(left, right, store),
                    InferenceRelation::Subtype(sub, sup) => self.subtype_terms(sub, sup, store, hierarchy),
                };
                match effect {
                    Ok(effect) => {
                        changed |= effect.is_changed();
                        self.record_bound_origin(&constraint.relation, constraint.origin.clone(), constraint_index as u32, store);
                        self.record_variable_constraint_indices(&constraint.relation, constraint_index as u32);
                    }
                    Err(failure) => {
                        let related = self.related_constraint_indices(&constraint.relation);
                        return self.failure_outcome_with_related(failure, Some(constraint_index as u32), Some(constraint.origin.clone()), &related);
                    }
                }
            }

            match self.propagate_subtype_edges(store, hierarchy, control) {
                Ok(effect) => changed |= effect.is_changed(),
                Err(outcome) => return outcome,
            }

            // Try to resolve remaining var_terms
            let var_terms = self.var_terms.clone();
            for (var, term) in var_terms {
                if control.is_cancelled() {
                    return InferenceOutcome::Cancelled;
                }
                if let Err(report) = control.charge_step() {
                    return InferenceOutcome::BudgetExceeded(report);
                }
                let rep = self.find_var(var);
                if !self.substitutions.contains_key(&rep) {
                    if let Ok(ty) = self.materialize(&term, store) {
                        match self.bind(rep, ty, store) {
                            Ok(effect) => changed |= effect.is_changed(),
                            Err(failure) => {
                                return self.failure_outcome(failure, None, None);
                            }
                        }
                    }
                }
            }

            // Try to resolve from lower/upper bounds
            let vars_to_check: Vec<InferVarId> = self.variables.iter().map(|v| self.find_var(v.id)).collect();
            for rep in vars_to_check {
                if control.is_cancelled() {
                    return InferenceOutcome::Cancelled;
                }
                if let Err(report) = control.charge_step() {
                    return InferenceOutcome::BudgetExceeded(report);
                }
                if !self.substitutions.contains_key(&rep) {
                    if let Some(lowers) = self.lower_bounds.get(&rep).cloned() {
                        if !lowers.is_empty() {
                            let candidate = store.union(&lowers);
                            if let Some(uppers) = self.upper_bounds.get(&rep) {
                                let mut ok = true;
                                let mut failed_upper = None;
                                for &upper in uppers {
                                    if !is_subtype(store, hierarchy, candidate, upper) {
                                        ok = false;
                                        failed_upper = Some(upper);
                                        break;
                                    }
                                }
                                if !ok {
                                    let failure = InferenceFailureReason::ConflictingBounds {
                                        var: rep,
                                        lower: candidate,
                                        upper: failed_upper.expect("failed upper recorded with non-empty failure"),
                                    };
                                    let upper = match &failure {
                                        InferenceFailureReason::ConflictingBounds { upper, .. } => *upper,
                                        _ => unreachable!("bound reconciliation creates ConflictingBounds"),
                                    };
                                    let (constraint_index, origin) = self
                                        .bound_origins
                                        .get(&(rep, upper))
                                        .map(|(index, origin)| (Some(*index), Some(origin.clone())))
                                        .unwrap_or((None, None));
                                    return self.failure_outcome(failure, constraint_index, origin);
                                }
                            }
                            match self.bind(rep, candidate, store) {
                                Ok(effect) => changed |= effect.is_changed(),
                                Err(failure) => {
                                    return self.failure_outcome(failure, None, None);
                                }
                            }
                        }
                    } else if let Some(uppers) = self.upper_bounds.get(&rep).cloned() {
                        if uppers.len() == 1 {
                            match self.bind(rep, uppers[0], store) {
                                Ok(effect) => changed |= effect.is_changed(),
                                Err(failure) => {
                                    return self.failure_outcome(failure, None, None);
                                }
                            }
                        }
                    }
                }
            }

            if !changed {
                converged = true;
                break;
            }
        }

        if !converged {
            return InferenceOutcome::Blocked(BlockReason::RecursiveFixpoint);
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
                support: self
                    .variables
                    .iter()
                    .filter_map(|variable| variable.support.map(|support| (variable.id, support)))
                    .collect(),
            })
        }
    }

    fn failure_outcome(&mut self, failure: InferenceFailureReason, constraint_index: Option<u32>, origin: Option<ConstraintOrigin>) -> InferenceOutcome {
        self.failure_outcome_with_related(failure, constraint_index, origin, &[])
    }

    fn failure_outcome_with_related(
        &mut self,
        failure: InferenceFailureReason,
        constraint_index: Option<u32>,
        origin: Option<ConstraintOrigin>,
        related_constraint_indices: &[u32],
    ) -> InferenceOutcome {
        self.mark_failure(&failure);
        if matches!(failure, InferenceFailureReason::MissingVariableMetadata { .. }) {
            InferenceOutcome::InternalFailure(failure)
        } else {
            let mut constraint_indices = related_constraint_indices.to_vec();
            if let Some(index) = constraint_index {
                constraint_indices.push(index);
            }
            if let InferenceFailureReason::ConflictingBounds { var, lower, upper } = &failure {
                let rep = self.find_var(*var);
                for bound in [*lower, *upper] {
                    if let Some((index, _)) = self.bound_origins.get(&(rep, bound)) {
                        constraint_indices.push(*index);
                    }
                }
            }
            constraint_indices.sort_unstable();
            constraint_indices.dedup();
            constraint_indices.truncate(4);
            InferenceOutcome::Conflicting(InferenceConflict {
                constraint_indices: constraint_indices.into_boxed_slice(),
                constraint_index,
                origin,
                failure,
            })
        }
    }

    fn record_variable_constraint_indices(&mut self, relation: &InferenceRelation, constraint_index: u32) {
        let terms = match relation {
            InferenceRelation::Equivalent(left, right) | InferenceRelation::Subtype(left, right) => [left, right],
        };
        for term in terms {
            for variable in self.term_variables(term) {
                let representative = self.find_var(variable);
                let indices = self.variable_constraint_indices.entry(representative).or_default();
                if !indices.contains(&constraint_index) {
                    indices.push(constraint_index);
                }
            }
        }
    }

    fn related_constraint_indices(&self, relation: &InferenceRelation) -> Vec<u32> {
        let terms = match relation {
            InferenceRelation::Equivalent(left, right) | InferenceRelation::Subtype(left, right) => [left, right],
        };
        let mut indices = Vec::new();
        for term in terms {
            for variable in self.term_variables(term) {
                let representative = self.find_var(variable);
                if let Some(recorded) = self.variable_constraint_indices.get(&representative) {
                    indices.extend(recorded.iter().copied());
                }
            }
        }
        indices.sort_unstable();
        indices.dedup();
        indices
    }

    fn record_bound_origin(&mut self, relation: &InferenceRelation, origin: ConstraintOrigin, constraint_index: u32, store: &mut TypeStore) {
        match relation {
            InferenceRelation::Subtype(InferenceTerm::Var(variable), term) => {
                let representative = self.find_var(*variable);
                if let Ok(ty) = self.materialize(term, store) {
                    self.bound_origins.entry((representative, ty)).or_insert((constraint_index, origin));
                }
            }
            InferenceRelation::Subtype(term, InferenceTerm::Var(variable)) => {
                let representative = self.find_var(*variable);
                if let Ok(ty) = self.materialize(term, store) {
                    self.bound_origins.entry((representative, ty)).or_insert((constraint_index, origin));
                }
            }
            _ => {}
        }
    }

    #[allow(clippy::result_large_err)]
    fn propagate_subtype_edges(
        &mut self,
        store: &mut TypeStore,
        hierarchy: &dyn TypeHierarchy,
        control: &CheckerControl,
    ) -> Result<SolveEffect, InferenceOutcome> {
        let edges = self.subtype_edges.clone();
        let mut changed = false;
        for edge in edges {
            if control.is_cancelled() {
                return Err(InferenceOutcome::Cancelled);
            }
            if let Err(report) = control.charge_step() {
                return Err(InferenceOutcome::BudgetExceeded(report));
            }
            let sub = self.find_var(edge.sub);
            let sup = self.find_var(edge.sup);
            if sub == sup {
                continue;
            }

            let sub_ty = self.substitutions.get(&sub).copied();
            let sup_ty = self.substitutions.get(&sup).copied();
            match (sub_ty, sup_ty) {
                (Some(sub_ty), Some(sup_ty)) => {
                    if !is_subtype(store, hierarchy, sub_ty, sup_ty) {
                        let failure = InferenceFailureReason::StructuralMismatch {
                            left: Box::new(InferenceTerm::Var(sub)),
                            right: Box::new(InferenceTerm::Var(sup)),
                        };
                        return Err(self.failure_outcome(failure, None, None));
                    }
                }
                (Some(sub_ty), None) => {
                    let added = self.add_lower_bound(sup, sub_ty);
                    changed |= added;
                    if added {
                        self.propagate_variable_evidence(sub, sup);
                    }
                }
                (None, Some(sup_ty)) => {
                    changed |= self.add_upper_bound(sub, sup_ty);
                }
                (None, None) => {
                    let lower_bounds = self.lower_bounds.get(&sub).cloned().unwrap_or_default();
                    for lower in lower_bounds {
                        let added = self.add_lower_bound(sup, lower);
                        changed |= added;
                        if added {
                            self.propagate_variable_evidence(sub, sup);
                        }
                    }
                    let upper_bounds = self.upper_bounds.get(&sup).cloned().unwrap_or_default();
                    for upper in upper_bounds {
                        changed |= self.add_upper_bound(sub, upper);
                    }
                }
            }
        }
        Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
    }

    fn add_lower_bound(&mut self, variable: InferVarId, ty: TypeId) -> bool {
        let representative = self.find_var(variable);
        let bounds = self.lower_bounds.entry(representative).or_default();
        if bounds.contains(&ty) {
            false
        } else {
            bounds.push(ty);
            true
        }
    }

    fn add_upper_bound(&mut self, variable: InferVarId, ty: TypeId) -> bool {
        let representative = self.find_var(variable);
        let bounds = self.upper_bounds.entry(representative).or_default();
        if bounds.contains(&ty) {
            false
        } else {
            bounds.push(ty);
            true
        }
    }

    fn propagate_variable_evidence(&mut self, from: InferVarId, to: InferVarId) {
        let from_rep = self.find_var(from);
        let support = self.variable_by_representative(from_rep).and_then(|variable| variable.support);
        let proof = self.variable_by_representative(from_rep).and_then(|variable| variable.proof.clone());
        if let Some(support) = support {
            self.record_variable_support(to, support);
        }
        if let Some(proof) = proof {
            self.record_variable_proof(to, proof);
        }
    }

    /// Returns whether an inference term contains a generic variable.
    pub fn term_has_variables(&self, term: &InferenceTerm) -> bool {
        !self.term_variables(term).is_empty()
    }

    /// Returns aggregate value support for all variables influencing a term.
    /// `None` means at least one influencing variable has no value support.
    pub fn term_support(&self, term: &InferenceTerm) -> Option<InferenceSupport> {
        let variables = self.term_variables(term);
        let mut support: Option<InferenceSupport> = None;
        for variable in variables {
            let current = self
                .variables
                .iter()
                .find(|candidate| self.find_var(candidate.id) == self.find_var(variable))
                .and_then(|candidate| candidate.support)?;
            support = Some(match support {
                Some(previous) => previous.join(current),
                None => current,
            });
        }
        support.or(Some(InferenceSupport::Established))
    }

    fn term_variables(&self, term: &InferenceTerm) -> HashSet<InferVarId> {
        let mut variables = HashSet::new();
        self.collect_term_variables(term, &mut variables);
        variables
    }

    fn collect_term_variables(&self, term: &InferenceTerm, variables: &mut HashSet<InferVarId>) {
        match term {
            InferenceTerm::Var(variable) => {
                variables.insert(self.find_var(*variable));
            }
            InferenceTerm::Applied { origin, arguments } => {
                self.collect_term_variables(origin, variables);
                for argument in arguments.iter() {
                    self.collect_term_variables(argument, variables);
                }
            }
            InferenceTerm::ExactCase { enum_type, .. } => {
                self.collect_term_variables(enum_type, variables);
            }
            InferenceTerm::Union(members) => {
                for member in members.iter() {
                    self.collect_term_variables(member, variables);
                }
            }
            InferenceTerm::Tuple(elements) => {
                for element in elements.iter() {
                    self.collect_term_variables(&element.term, variables);
                }
            }
            InferenceTerm::Callable(callable) => {
                for parameter in callable.parameters.iter() {
                    self.collect_term_variables(&parameter.term, variables);
                }
                self.collect_term_variables(&callable.return_type, variables);
            }
            InferenceTerm::Canonical(_) => {}
        }
    }

    fn record_relation_support(&mut self, relation: &InferenceRelation, support: InferenceSupport) {
        match relation {
            InferenceRelation::Equivalent(left, right) | InferenceRelation::Subtype(left, right) => {
                self.record_term_support(left, support);
                self.record_term_support(right, support);
            }
        }
    }

    fn record_term_proof_state(&mut self, term: &InferenceTerm, proof: InferenceProofState) {
        match term {
            InferenceTerm::Var(variable) => self.record_variable_proof(*variable, proof),
            InferenceTerm::Applied { origin, arguments } => {
                self.record_term_proof_state(origin, proof.clone());
                for argument in arguments.iter() {
                    self.record_term_proof_state(argument, proof.clone());
                }
            }
            InferenceTerm::ExactCase { enum_type, .. } => {
                self.record_term_proof_state(enum_type, proof);
            }
            InferenceTerm::Union(members) => {
                for member in members.iter() {
                    self.record_term_proof_state(member, proof.clone());
                }
            }
            InferenceTerm::Tuple(elements) => {
                for element in elements.iter() {
                    self.record_term_proof_state(&element.term, proof.clone());
                }
            }
            InferenceTerm::Callable(callable) => {
                for parameter in callable.parameters.iter() {
                    self.record_term_proof_state(&parameter.term, proof.clone());
                }
                self.record_term_proof_state(&callable.return_type, proof);
            }
            InferenceTerm::Canonical(_) => {}
        }
    }

    fn record_term_support(&mut self, term: &InferenceTerm, support: InferenceSupport) {
        match term {
            InferenceTerm::Var(variable) => self.record_variable_support(*variable, support),
            InferenceTerm::Applied { origin, arguments } => {
                self.record_term_support(origin, support);
                for argument in arguments.iter() {
                    self.record_term_support(argument, support);
                }
            }
            InferenceTerm::ExactCase { enum_type, .. } => {
                self.record_term_support(enum_type, support);
            }
            InferenceTerm::Union(members) => {
                for member in members.iter() {
                    self.record_term_support(member, support);
                }
            }
            InferenceTerm::Tuple(elements) => {
                for element in elements.iter() {
                    self.record_term_support(&element.term, support);
                }
            }
            InferenceTerm::Callable(callable) => {
                for parameter in callable.parameters.iter() {
                    self.record_term_support(&parameter.term, support);
                }
                self.record_term_support(&callable.return_type, support);
            }
            InferenceTerm::Canonical(_) => {}
        }
    }

    fn record_variable_support(&mut self, variable: InferVarId, support: InferenceSupport) {
        let representative = self.find_var(variable);
        if let Some(candidate) = self.variables.iter_mut().find(|candidate| candidate.id == representative) {
            candidate.support = Some(match candidate.support {
                Some(previous) => previous.join(support),
                None => support,
            });
        }
    }

    fn record_variable_proof(&mut self, variable: InferVarId, proof: InferenceProofState) {
        let representative = self.find_var(variable);
        if let Some(candidate) = self.variables.iter_mut().find(|candidate| candidate.id == representative) {
            candidate.proof = Some(match candidate.proof.take() {
                Some(previous) => previous.meet(proof),
                None => proof,
            });
        }
    }

    fn mark_failure(&mut self, failure: &InferenceFailureReason) {
        let var = match failure {
            InferenceFailureReason::OccursCheck { var }
            | InferenceFailureReason::KindMismatch { var, .. }
            | InferenceFailureReason::ConflictingBounds { var, .. }
            | InferenceFailureReason::MissingVariableMetadata { var } => Some(*var),
            InferenceFailureReason::StructuralMismatch { .. } | InferenceFailureReason::UnresolvedSelf => None,
        };
        if let Some(var) = var {
            let rep = self.find_var(var);
            if let Some(state) = self.variables.iter_mut().find(|candidate| candidate.id == rep) {
                state.state = InferVarState::Failed(failure.clone());
            }
        }
    }

    /// Binds an inference variable to a canonical type, with occurs and kind checks.
    pub fn bind(&mut self, var: InferVarId, ty: TypeId, store: &TypeStore) -> Result<SolveEffect, InferenceFailureReason> {
        let rep = self.find_var(var);
        if self.occurs_in_type(rep, ty, store) {
            return Err(InferenceFailureReason::OccursCheck { var: rep });
        }
        let expected_kind = self
            .variables
            .iter()
            .find(|candidate| candidate.id == rep)
            .map(|candidate| candidate.kind)
            .ok_or(InferenceFailureReason::MissingVariableMetadata { var: rep })?;
        let actual_kind = store.kind_of(ty);
        if expected_kind != actual_kind {
            return Err(InferenceFailureReason::KindMismatch {
                var: rep,
                expected: expected_kind,
                actual: actual_kind,
            });
        }
        if self.substitutions.get(&rep).copied() == Some(ty) {
            return Ok(SolveEffect::Unchanged);
        }
        self.substitutions.insert(rep, ty);
        if let Some(v) = self.variables.iter_mut().find(|v| v.id == rep) {
            v.state = InferVarState::Solved(ty);
        }
        Ok(SolveEffect::Changed)
    }

    /// Checks if `var` occurs in `ty`.
    #[allow(clippy::only_used_in_recursion)]
    pub fn occurs_in_type(&self, var: InferVarId, ty: TypeId, store: &TypeStore) -> bool {
        match store.get(ty) {
            TypeData::Applied { origin, arguments } => {
                self.occurs_in_type(var, *origin, store) || arguments.iter().any(|&a| self.occurs_in_type(var, a, store))
            }
            TypeData::ExactCase { enum_type, .. } => self.occurs_in_type(var, *enum_type, store),
            TypeData::Union(members) => members.iter().any(|&m| self.occurs_in_type(var, m, store)),
            TypeData::Tuple(elems) => elems.iter().any(|e| self.occurs_in_type(var, e.ty, store)),
            TypeData::Record(row_id) => store.record_row(*row_id).fields.iter().any(|f| self.occurs_in_type(var, f.ty, store)),
            TypeData::Callable(c) => c.parameters.iter().any(|p| self.occurs_in_type(var, p.ty, store)) || self.occurs_in_type(var, c.return_type, store),
            TypeData::Family(fid) => store.get_family(*fid).members.iter().any(|m| self.occurs_in_type(var, m.ty, store)),
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
            InferenceTerm::ExactCase { enum_type, .. } => self.occurs_in_term(rep, enum_type),
            InferenceTerm::Union(members) => members.iter().any(|m| self.occurs_in_term(rep, m)),
            InferenceTerm::Tuple(elems) => elems.iter().any(|e| self.occurs_in_term(rep, &e.term)),
            InferenceTerm::Callable(c) => c.parameters.iter().any(|p| self.occurs_in_term(rep, &p.term)) || self.occurs_in_term(rep, &c.return_type),
        }
    }

    fn unify_terms(&mut self, left: &InferenceTerm, right: &InferenceTerm, store: &mut TypeStore) -> Result<SolveEffect, InferenceFailureReason> {
        match (left, right) {
            (InferenceTerm::Var(v1), InferenceTerm::Var(v2)) => {
                let rep1 = self.find_var(*v1);
                let rep2 = self.find_var(*v2);
                if rep1 == rep2 {
                    return Ok(SolveEffect::Unchanged);
                }
                if let Some(ty1) = self.substitutions.get(&rep1).copied() {
                    let canon = InferenceTerm::Canonical(ty1);
                    return self.unify_terms(&canon, right, store);
                }
                if let Some(ty2) = self.substitutions.get(&rep2).copied() {
                    let canon = InferenceTerm::Canonical(ty2);
                    return self.unify_terms(left, &canon, store);
                }
                let kind1 = self
                    .variables
                    .iter()
                    .find(|candidate| candidate.id == rep1)
                    .map(|candidate| candidate.kind)
                    .ok_or(InferenceFailureReason::MissingVariableMetadata { var: rep1 })?;
                let kind2 = self
                    .variables
                    .iter()
                    .find(|candidate| candidate.id == rep2)
                    .map(|candidate| candidate.kind)
                    .ok_or(InferenceFailureReason::MissingVariableMetadata { var: rep2 })?;
                if kind1 != kind2 {
                    return Err(InferenceFailureReason::KindMismatch {
                        var: rep1,
                        expected: kind1,
                        actual: kind2,
                    });
                }
                let support = self
                    .variables
                    .iter()
                    .find(|candidate| candidate.id == rep1)
                    .and_then(|candidate| candidate.support)
                    .into_iter()
                    .chain(
                        self.variables
                            .iter()
                            .find(|candidate| candidate.id == rep2)
                            .and_then(|candidate| candidate.support),
                    )
                    .reduce(InferenceSupport::join);
                let proof = self
                    .variables
                    .iter()
                    .filter(|candidate| candidate.id == rep1 || candidate.id == rep2)
                    .filter_map(|candidate| candidate.proof.clone())
                    .reduce(InferenceProofState::meet);
                self.var_aliases.insert(rep1, rep2);
                if let Some(support) = support {
                    self.record_variable_support(rep2, support);
                }
                if let Some(proof) = proof {
                    if let Some(candidate) = self.variables.iter_mut().find(|candidate| candidate.id == rep2) {
                        candidate.proof = Some(proof);
                    }
                }
                // Merge bounds
                if let Some(lowers) = self.lower_bounds.remove(&rep1) {
                    self.lower_bounds.entry(rep2).or_default().extend(lowers);
                }
                if let Some(uppers) = self.upper_bounds.remove(&rep1) {
                    self.upper_bounds.entry(rep2).or_default().extend(uppers);
                }
                let origins = std::mem::take(&mut self.bound_origins);
                for ((variable, ty), origin) in origins {
                    let representative = if variable == rep1 { rep2 } else { variable };
                    self.bound_origins.entry((representative, ty)).or_insert(origin);
                }
                if let Some(indices) = self.variable_constraint_indices.remove(&rep1) {
                    let target = self.variable_constraint_indices.entry(rep2).or_default();
                    target.extend(indices);
                    target.sort_unstable();
                    target.dedup();
                }
                Ok(SolveEffect::Changed)
            }
            (InferenceTerm::Var(v), term) | (term, InferenceTerm::Var(v)) => {
                let rep = self.find_var(*v);
                if let Some(ty) = self.substitutions.get(&rep).copied() {
                    let canon = InferenceTerm::Canonical(ty);
                    self.unify_terms(&canon, term, store)
                } else if self.occurs_in_term(rep, term) {
                    Err(InferenceFailureReason::OccursCheck { var: rep })
                } else if let Ok(ty) = self.materialize(term, store) {
                    self.bind(rep, ty, store)
                } else {
                    self.var_terms.insert(rep, term.clone());
                    Ok(SolveEffect::Changed)
                }
            }
            (InferenceTerm::Canonical(t1), InferenceTerm::Canonical(t2)) => {
                if t1 == t2 {
                    Ok(SolveEffect::Unchanged)
                } else {
                    Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    })
                }
            }
            (InferenceTerm::ExactCase { variant: v1, enum_type: e1 }, InferenceTerm::ExactCase { variant: v2, enum_type: e2 }) => {
                if v1 != v2 {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                self.unify_terms(e1, e2, store)
            }
            (InferenceTerm::Canonical(ty), InferenceTerm::ExactCase { variant, enum_type })
            | (InferenceTerm::ExactCase { variant, enum_type }, InferenceTerm::Canonical(ty)) => {
                if let TypeData::ExactCase {
                    variant: canon_var,
                    enum_type: canon_enum,
                } = store.get(*ty).clone()
                {
                    if canon_var != *variant {
                        return Err(InferenceFailureReason::StructuralMismatch {
                            left: Box::new(left.clone()),
                            right: Box::new(right.clone()),
                        });
                    }
                    let canon_enum_term = InferenceTerm::Canonical(canon_enum);
                    self.unify_terms(enum_type, &canon_enum_term, store)
                } else {
                    Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    })
                }
            }
            (InferenceTerm::Applied { origin: o1, arguments: a1 }, InferenceTerm::Applied { origin: o2, arguments: a2 }) => {
                if a1.len() != a2.len() {
                    return Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    });
                }
                let mut changed = self.unify_terms(o1, o2, store)?.is_changed();
                for (arg1, arg2) in a1.iter().zip(a2.iter()) {
                    changed |= self.unify_terms(arg1, arg2, store)?.is_changed();
                }
                Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
            }
            (InferenceTerm::Canonical(ty), InferenceTerm::Applied { origin, arguments })
            | (InferenceTerm::Applied { origin, arguments }, InferenceTerm::Canonical(ty)) => {
                if let TypeData::Applied {
                    origin: orig_ty,
                    arguments: args_ty,
                } = store.get(*ty).clone()
                {
                    if args_ty.len() != arguments.len() {
                        return Err(InferenceFailureReason::StructuralMismatch {
                            left: Box::new(left.clone()),
                            right: Box::new(right.clone()),
                        });
                    }
                    let orig_term = InferenceTerm::Canonical(orig_ty);
                    let mut changed = self.unify_terms(origin, &orig_term, store)?.is_changed();
                    for (arg_term, &arg_ty) in arguments.iter().zip(args_ty.iter()) {
                        let canon = InferenceTerm::Canonical(arg_ty);
                        changed |= self.unify_terms(arg_term, &canon, store)?.is_changed();
                    }
                    Ok(if changed { SolveEffect::Changed } else { SolveEffect::Unchanged })
                } else {
                    Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(left.clone()),
                        right: Box::new(right.clone()),
                    })
                }
            }
            _ => Err(InferenceFailureReason::StructuralMismatch {
                left: Box::new(left.clone()),
                right: Box::new(right.clone()),
            }),
        }
    }

    fn subtype_terms(
        &mut self,
        sub: &InferenceTerm,
        sup: &InferenceTerm,
        store: &mut TypeStore,
        hier: &dyn TypeHierarchy,
    ) -> Result<SolveEffect, InferenceFailureReason> {
        match (sub, sup) {
            (InferenceTerm::ExactCase { enum_type, .. }, sup) => self.subtype_terms(enum_type, sup, store, hier),
            (InferenceTerm::Var(sub_var), InferenceTerm::Var(sup_var)) => {
                let sub_rep = self.find_var(*sub_var);
                let sup_rep = self.find_var(*sup_var);
                if sub_rep == sup_rep {
                    return Ok(SolveEffect::Unchanged);
                }
                if let Some(sub_ty) = self.substitutions.get(&sub_rep).copied() {
                    return self.subtype_terms(&InferenceTerm::Canonical(sub_ty), sup, store, hier);
                }
                if let Some(sup_ty) = self.substitutions.get(&sup_rep).copied() {
                    return self.subtype_terms(sub, &InferenceTerm::Canonical(sup_ty), store, hier);
                }
                if self
                    .subtype_edges
                    .iter()
                    .any(|edge| self.find_var(edge.sub) == sub_rep && self.find_var(edge.sup) == sup_rep)
                {
                    Ok(SolveEffect::Unchanged)
                } else {
                    self.subtype_edges.push(InferenceSubtypeEdge { sub: sub_rep, sup: sup_rep });
                    Ok(SolveEffect::Changed)
                }
            }
            (InferenceTerm::Var(v), term) => {
                let rep = self.find_var(*v);
                if let Some(ty) = self.substitutions.get(&rep).copied() {
                    let canon = InferenceTerm::Canonical(ty);
                    self.subtype_terms(&canon, term, store, hier)
                } else if let Ok(ty) = self.materialize(term, store) {
                    Ok(if self.add_upper_bound(rep, ty) {
                        SolveEffect::Changed
                    } else {
                        SolveEffect::Unchanged
                    })
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
                    Ok(if self.add_lower_bound(rep, ty) {
                        SolveEffect::Changed
                    } else {
                        SolveEffect::Unchanged
                    })
                } else {
                    self.unify_terms(sub, sup, store)
                }
            }
            (InferenceTerm::Canonical(t1), InferenceTerm::Canonical(t2)) => {
                if is_subtype(store, hier, *t1, *t2) {
                    Ok(SolveEffect::Unchanged)
                } else {
                    Err(InferenceFailureReason::StructuralMismatch {
                        left: Box::new(sub.clone()),
                        right: Box::new(sup.clone()),
                    })
                }
            }
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
            InferenceTerm::ExactCase { variant, enum_type } => {
                let enum_ty = self.materialize(enum_type, store)?;
                let variant_id = store.variant_identity(*variant).clone();
                store
                    .exact_case_type(&variant_id, enum_ty)
                    .map_err(|_| UnderconstrainedInference { unsolved_vars: Vec::new() })
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

#[cfg(test)]
mod tests {
    use super::{InferenceFailureReason, InferenceOutcome, InferenceSession};
    use crate::identity::InferVarId;

    #[test]
    fn missing_variable_metadata_is_an_internal_failure() {
        let mut session = InferenceSession::new();
        let outcome = session.failure_outcome(
            InferenceFailureReason::MissingVariableMetadata {
                var: InferVarId::from_index(99),
            },
            None,
            None,
        );

        assert!(matches!(
            outcome,
            InferenceOutcome::InternalFailure(InferenceFailureReason::MissingVariableMetadata { .. })
        ));
    }
}
