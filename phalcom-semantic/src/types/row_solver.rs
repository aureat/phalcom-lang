//! Query-local record-row constraint solver.
//!
//! Solver terms are normalized structural rows. They contain no canonical row
//! IDs, so exploration never depends on speculative rows interned by an
//! earlier query.

use super::id::{RecordRowId, TypeId, TypeParameterId};
use super::outcome::{BudgetReport, CancellationToken, QueryBudget};
use super::row::{RecordRowField, RecordRowFormationError, RecordRowTail};
use super::store::TypeStore;
use crate::field_key::FieldKey;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RecordRowVarId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RecordRowTermTail<V = TypeId> {
    Closed,
    Parameter(TypeParameterId),
    Projection(TypeId),
    Var(RecordRowVarId),
    Anonymous(crate::types::row::AnonymousRecordRemainder<V>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordRowTerm<V = TypeId> {
    pub fields: Box<[RecordRowField<V>]>,
    pub tail: RecordRowTermTail<V>,
}

impl<V> RecordRowTerm<V> {
    pub fn new(fields: Vec<RecordRowField<V>>, tail: RecordRowTermTail<V>) -> Result<Self, FieldKey> {
        let mut fields = fields;
        fields.sort_by(|left, right| left.key.canonical_cmp(&right.key));
        for pair in fields.windows(2) {
            if pair[0].key == pair[1].key {
                return Err(pair[0].key.clone());
            }
        }
        Ok(Self {
            fields: fields.into_boxed_slice(),
            tail,
        })
    }

    pub fn closed(fields: Vec<RecordRowField<V>>) -> Result<Self, FieldKey> {
        Self::new(fields, RecordRowTermTail::Closed)
    }

    fn extension(fields: Vec<RecordRowField<V>>, tail: RecordRowTermTail<V>) -> Self {
        let mut fields = fields;
        fields.sort_by(|left, right| left.key.canonical_cmp(&right.key));
        Self {
            fields: fields.into_boxed_slice(),
            tail,
        }
    }
}

impl RecordRowTerm<TypeId> {
    pub fn from_canonical(store: &TypeStore, row: RecordRowId) -> Self {
        let data = store.record_row(row);
        Self {
            fields: data.fields.clone(),
            tail: match data.tail {
                RecordRowTail::Closed => RecordRowTermTail::Closed,
                RecordRowTail::Parameter(parameter) => RecordRowTermTail::Parameter(parameter),
                RecordRowTail::Projection(proj) => RecordRowTermTail::Projection(proj),
                RecordRowTail::Anonymous(rem) => RecordRowTermTail::Anonymous(rem),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordRowLacks {
    pub row: RecordRowVarId,
    pub field: FieldKey,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecordRowSolution<V = TypeId> {
    pub substitutions: HashMap<RecordRowVarId, RecordRowTerm<V>>,
}

impl<V> RecordRowSolution<V> {
    pub fn term_for(&self, variable: RecordRowVarId) -> Option<&RecordRowTerm<V>> {
        self.substitutions.get(&variable)
    }
}

impl RecordRowSolution<TypeId> {
    pub fn zonk_variable_to_canonical(&self, variable: RecordRowVarId, store: &mut TypeStore) -> Result<RecordRowId, RecordRowZonkError> {
        let term = self.term_for(variable).ok_or(RecordRowZonkError::Unsolved(variable))?;
        self.zonk_term_to_canonical(term, store, &mut HashSet::new())
    }

    fn zonk_term_to_canonical(
        &self,
        term: &RecordRowTerm,
        store: &mut TypeStore,
        visiting: &mut HashSet<RecordRowVarId>,
    ) -> Result<RecordRowId, RecordRowZonkError> {
        let mut fields = term.fields.to_vec();
        let tail = match term.tail.clone() {
            RecordRowTermTail::Closed => RecordRowTail::Closed,
            RecordRowTermTail::Parameter(parameter) => RecordRowTail::Parameter(parameter),
            RecordRowTermTail::Projection(proj) => RecordRowTail::Projection(proj),
            RecordRowTermTail::Anonymous(rem) => RecordRowTail::Anonymous(rem),
            RecordRowTermTail::Var(variable) => {
                if !visiting.insert(variable) {
                    return Err(RecordRowZonkError::Recursive(variable));
                }
                let next = self.term_for(variable).ok_or(RecordRowZonkError::Unsolved(variable))?;
                let row = self.zonk_term_to_canonical(next, store, visiting)?;
                visiting.remove(&variable);
                let data = store.record_row(row).clone();
                fields.extend(data.fields.into_vec());
                data.tail
            }
        };
        store.record_row_checked(fields, tail).map_err(RecordRowZonkError::Formation)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordRowFailure<V = TypeId> {
    OccursCheckFailed { var: RecordRowVarId, term: RecordRowTerm<V> },
    LacksViolation { field: FieldKey, row: RecordRowTerm<V> },
    IncompatibleFields { field: FieldKey, expected: TypeId, actual: TypeId },
    MissingField { field: FieldKey },
    ExtraField { field: FieldKey },
    DuplicateField(FieldKey),
    RigidTailMismatch { expected: RecordRowTermTail<V>, actual: RecordRowTermTail<V> },
    KindMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordRowBlockedReason {
    UnboundVariable(RecordRowVarId),
    AmbiguousExtension,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordRowUnderconstrained {
    pub variables: Box<[RecordRowVarId]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IncidentId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordRowZonkError {
    Unsolved(RecordRowVarId),
    Recursive(RecordRowVarId),
    Formation(RecordRowFormationError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordRowSolveResult<V = TypeId> {
    Solved(RecordRowSolution<V>),
    Underconstrained(RecordRowUnderconstrained),
    Rejected(RecordRowFailure<V>),
    Blocked(RecordRowBlockedReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(IncidentId),
}

enum ControlFailure<V = TypeId> {
    Cancelled,
    Budget(BudgetReport),
    Rejected(RecordRowFailure<V>),
}

/// Shape-solving result. Equal-label payload pairs must be equated by the
/// owning inference coordinator; row solving itself only owns labels and tails.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordRowShapeSolveResult<V> {
    Solved {
        solution: RecordRowSolution<V>,
        field_equations: Vec<(FieldKey, V, V)>,
    },
    Underconstrained(RecordRowUnderconstrained),
    Rejected(RecordRowFailure<V>),
    Blocked(RecordRowBlockedReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(IncidentId),
}

#[derive(Clone, Debug)]
pub struct RecordRowSolver<V = TypeId> {
    next_var: u32,
    allocated: Vec<RecordRowVarId>,
    substitutions: HashMap<RecordRowVarId, RecordRowTerm<V>>,
    lacks: Vec<RecordRowLacks>,
}

impl<V: Clone + Eq> Default for RecordRowSolver<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Clone + Eq> RecordRowSolver<V> {
    pub fn new() -> Self {
        Self {
            next_var: 0,
            allocated: Vec::new(),
            substitutions: HashMap::new(),
            lacks: Vec::new(),
        }
    }

    pub fn fresh_var(&mut self) -> RecordRowVarId {
        let id = RecordRowVarId(self.next_var);
        self.next_var += 1;
        self.allocated.push(id);
        id
    }

    pub fn add_lacks(&mut self, row: RecordRowVarId, field: FieldKey) -> Result<(), RecordRowFailure<V>> {
        let row_term = self.normalize_term(&RecordRowTerm::extension(Vec::new(), RecordRowTermTail::Var(row)));
        if self.term_contains_field(&row_term, &field) {
            return Err(RecordRowFailure::LacksViolation { field, row: row_term });
        }
        if self.lacks.iter().any(|lack| lack.row == row && lack.field == field) {
            return Ok(());
        }
        self.lacks.push(RecordRowLacks { row, field: field.clone() });
        self.propagate_lack_alias(row, field);
        Ok(())
    }

    pub fn occurs(&self, var: RecordRowVarId, term: &RecordRowTerm<V>) -> bool {
        let term = self.normalize_term(term);
        matches!(term.tail, RecordRowTermTail::Var(other) if other == var)
    }

    pub fn normalize_term(&self, term: &RecordRowTerm<V>) -> RecordRowTerm<V> {
        self.normalize_term_with_seen(term, &mut HashSet::new())
    }

    fn normalize_term_with_seen(&self, term: &RecordRowTerm<V>, seen: &mut HashSet<RecordRowVarId>) -> RecordRowTerm<V> {
        let mut fields = term.fields.to_vec();
        let tail = match term.tail.clone() {
            RecordRowTermTail::Var(variable) => {
                if !seen.insert(variable) {
                    return RecordRowTerm::extension(fields, RecordRowTermTail::Var(variable));
                }
                if let Some(substitution) = self.substitutions.get(&variable) {
                    let normalized = self.normalize_term_with_seen(substitution, seen);
                    fields.extend(normalized.fields.into_vec());
                    seen.remove(&variable);
                    normalized.tail
                } else {
                    seen.remove(&variable);
                    RecordRowTermTail::Var(variable)
                }
            }
            tail => tail,
        };
        RecordRowTerm::extension(fields, tail)
    }

    pub fn solve_shapes_for_roots(
        &mut self,
        equations: &[(RecordRowTerm<V>, RecordRowTerm<V>)],
        roots: &[RecordRowVarId],
        budget: &mut QueryBudget,
        cancellation: &CancellationToken,
    ) -> RecordRowShapeSolveResult<V> {
        self.solve_shapes_for_roots_with_payload_occurs(equations, roots, budget, cancellation, |_, _| false)
    }

    /// Solves row labels and tails while delegating cross-domain occurs checks
    /// to the owner of row payloads. Ordinary row callers use the unchecked
    /// wrapper above because `TypeId` cannot contain query-local row variables.
    pub fn solve_shapes_for_roots_with_payload_occurs<F>(
        &mut self,
        equations: &[(RecordRowTerm<V>, RecordRowTerm<V>)],
        roots: &[RecordRowVarId],
        budget: &mut QueryBudget,
        cancellation: &CancellationToken,
        mut payload_occurs: F,
    ) -> RecordRowShapeSolveResult<V>
    where
        F: FnMut(RecordRowVarId, &V) -> bool,
    {
        let mut field_equations = Vec::new();
        for (left, right) in equations {
            match self.unify_shape_with_control(left, right, budget, cancellation, &mut payload_occurs) {
                Err(ControlFailure::Cancelled) => return RecordRowShapeSolveResult::Cancelled,
                Err(ControlFailure::Budget(report)) => return RecordRowShapeSolveResult::BudgetExceeded(report),
                Err(ControlFailure::Rejected(error)) => return RecordRowShapeSolveResult::Rejected(error),
                Ok(mut equalities) => field_equations.append(&mut equalities),
            }
        }

        let unresolved = roots
            .iter()
            .copied()
            .filter(|variable| !self.substitutions.contains_key(variable))
            .collect::<Vec<_>>();
        if !unresolved.is_empty() {
            return RecordRowShapeSolveResult::Underconstrained(RecordRowUnderconstrained {
                variables: unresolved.into_boxed_slice(),
            });
        }

        let substitutions = self
            .allocated
            .iter()
            .filter_map(|variable| {
                let normalized = self.normalize_term(&RecordRowTerm::extension(Vec::new(), RecordRowTermTail::Var(*variable)));
                (!matches!(normalized.tail, RecordRowTermTail::Var(current) if current == *variable)).then_some((*variable, normalized))
            })
            .collect();
        RecordRowShapeSolveResult::Solved {
            solution: RecordRowSolution { substitutions },
            field_equations,
        }
    }

    fn unify_shape_with_control(
        &mut self,
        left: &RecordRowTerm<V>,
        right: &RecordRowTerm<V>,
        budget: &mut QueryBudget,
        cancellation: &CancellationToken,
        payload_occurs: &mut impl FnMut(RecordRowVarId, &V) -> bool,
    ) -> Result<Vec<(FieldKey, V, V)>, ControlFailure<V>> {
        cancellation.check().map_err(|_| ControlFailure::Cancelled)?;
        budget.charge_step().map_err(ControlFailure::Budget)?;

        let left = self.normalize_term(left);
        let right = self.normalize_term(right);
        self.validate_fields(&left)?;
        self.validate_fields(&right)?;

        let mut field_equations = Vec::new();
        let mut only_left = Vec::new();
        let mut only_right = Vec::new();
        let (mut left_index, mut right_index) = (0, 0);
        while left_index < left.fields.len() && right_index < right.fields.len() {
            let left_field = &left.fields[left_index];
            let right_field = &right.fields[right_index];
            match left_field.key.canonical_cmp(&right_field.key) {
                std::cmp::Ordering::Less => {
                    only_left.push(left_field.clone());
                    left_index += 1;
                }
                std::cmp::Ordering::Greater => {
                    only_right.push(right_field.clone());
                    right_index += 1;
                }
                std::cmp::Ordering::Equal => {
                    field_equations.push((left_field.key.clone(), left_field.ty.clone(), right_field.ty.clone()));
                    left_index += 1;
                    right_index += 1;
                }
            }
        }
        only_left.extend(left.fields[left_index..].iter().cloned());
        only_right.extend(right.fields[right_index..].iter().cloned());

        // The public term constructors and normalization path keep fixed
        // prefixes canonical. Equal comparisons here therefore correspond to
        // exact FieldKey identity, including the Symbol/String distinction.
        debug_assert!(field_equations.windows(2).all(|pair| pair[0].0 < pair[1].0));

        if only_left.is_empty() && only_right.is_empty() {
            self.unify_tails(left.tail, right.tail, budget, cancellation, payload_occurs)?;
        } else if only_left.is_empty() {
            self.bind_tail(
                left.tail,
                RecordRowTerm::extension(only_right, right.tail),
                budget,
                cancellation,
                payload_occurs,
            )?;
        } else if only_right.is_empty() {
            self.bind_tail(right.tail, RecordRowTerm::extension(only_left, left.tail), budget, cancellation, payload_occurs)?;
        } else {
            let fresh = self.fresh_var();
            self.bind_tail(
                left.tail,
                RecordRowTerm::extension(only_right, RecordRowTermTail::Var(fresh)),
                budget,
                cancellation,
                payload_occurs,
            )?;
            self.bind_tail(
                right.tail,
                RecordRowTerm::extension(only_left, RecordRowTermTail::Var(fresh)),
                budget,
                cancellation,
                payload_occurs,
            )?;
        }
        Ok(field_equations)
    }

    fn term_contains_field(&self, term: &RecordRowTerm<V>, field: &FieldKey) -> bool {
        term.fields.iter().any(|candidate| candidate.key == *field)
    }

    fn propagate_lack_alias(&mut self, row: RecordRowVarId, field: FieldKey) {
        let normalized = self.normalize_term(&RecordRowTerm::extension(Vec::new(), RecordRowTermTail::Var(row)));
        if let RecordRowTermTail::Var(next) = normalized.tail {
            if next != row && !self.lacks.iter().any(|lack| lack.row == next && lack.field == field) {
                self.lacks.push(RecordRowLacks { row: next, field });
            }
        }
    }

    fn propagate_lacks(&mut self, from: RecordRowVarId, to: RecordRowVarId) {
        let pending = self
            .lacks
            .iter()
            .filter(|lack| lack.row == from)
            .map(|lack| lack.field.clone())
            .collect::<Vec<_>>();
        for field in pending {
            if !self.lacks.iter().any(|lack| lack.row == to && lack.field == field) {
                self.lacks.push(RecordRowLacks { row: to, field });
            }
        }
    }
}

impl RecordRowSolver<TypeId> {
    pub fn unify(&mut self, left: &RecordRowTerm, right: &RecordRowTerm, store: &TypeStore) -> Result<(), RecordRowFailure> {
        let mut budget = QueryBudget::default();
        let cancellation = CancellationToken::new();
        let mut trial = self.clone();
        let mut payload_occurs = |_: RecordRowVarId, _: &TypeId| false;
        match trial.unify_shape_with_control(left, right, &mut budget, &cancellation, &mut payload_occurs) {
            Ok(field_equations) => {
                if let Some((field, expected, actual)) = field_equations.into_iter().find(|(_, expected, actual)| expected != actual) {
                    return Err(RecordRowFailure::IncompatibleFields { field, expected, actual });
                }
                *self = trial;
                let _ = store;
                Ok(())
            }
            Err(ControlFailure::Rejected(error)) => Err(error),
            Err(ControlFailure::Cancelled | ControlFailure::Budget(_)) => Err(RecordRowFailure::KindMismatch),
        }
    }

    pub fn solve(
        &mut self,
        left: &RecordRowTerm,
        right: &RecordRowTerm,
        store: &TypeStore,
        budget: &mut QueryBudget,
        cancellation: &CancellationToken,
    ) -> RecordRowSolveResult {
        self.solve_many(std::slice::from_ref(&(left.clone(), right.clone())), store, budget, cancellation)
    }

    pub fn solve_many(
        &mut self,
        equations: &[(RecordRowTerm, RecordRowTerm)],
        store: &TypeStore,
        budget: &mut QueryBudget,
        cancellation: &CancellationToken,
    ) -> RecordRowSolveResult {
        let mut trial = self.clone();
        let roots = trial.allocated.clone();
        match trial.solve_shapes_for_roots(equations, &roots, budget, cancellation) {
            RecordRowShapeSolveResult::Solved { field_equations, solution } => {
                if let Some((field, expected, actual)) = field_equations.into_iter().find(|(_, expected, actual)| expected != actual) {
                    return RecordRowSolveResult::Rejected(RecordRowFailure::IncompatibleFields { field, expected, actual });
                }
                *self = trial;
                let _ = store;
                RecordRowSolveResult::Solved(solution)
            }
            RecordRowShapeSolveResult::Underconstrained(result) => {
                *self = trial;
                RecordRowSolveResult::Underconstrained(result)
            }
            RecordRowShapeSolveResult::Rejected(error) => RecordRowSolveResult::Rejected(error),
            RecordRowShapeSolveResult::Blocked(reason) => RecordRowSolveResult::Blocked(reason),
            RecordRowShapeSolveResult::Cancelled => RecordRowSolveResult::Cancelled,
            RecordRowShapeSolveResult::BudgetExceeded(report) => RecordRowSolveResult::BudgetExceeded(report),
            RecordRowShapeSolveResult::InternalFailure(id) => RecordRowSolveResult::InternalFailure(id),
        }
    }
}

impl<V: Clone + Eq> RecordRowSolver<V> {
    fn unify_tails(
        &mut self,
        left: RecordRowTermTail<V>,
        right: RecordRowTermTail<V>,
        budget: &mut QueryBudget,
        cancellation: &CancellationToken,
        payload_occurs: &mut impl FnMut(RecordRowVarId, &V) -> bool,
    ) -> Result<(), ControlFailure<V>> {
        if left == right {
            return Ok(());
        }
        match (left, right) {
            (RecordRowTermTail::Var(variable), tail) | (tail, RecordRowTermTail::Var(variable)) => self.bind_tail(
                RecordRowTermTail::Var(variable),
                RecordRowTerm::extension(Vec::new(), tail),
                budget,
                cancellation,
                payload_occurs,
            ),
            (expected, actual) => Err(ControlFailure::Rejected(RecordRowFailure::RigidTailMismatch { expected, actual })),
        }
    }

    fn bind_tail(
        &mut self,
        target: RecordRowTermTail<V>,
        term: RecordRowTerm<V>,
        budget: &mut QueryBudget,
        cancellation: &CancellationToken,
        payload_occurs: &mut impl FnMut(RecordRowVarId, &V) -> bool,
    ) -> Result<(), ControlFailure<V>> {
        cancellation.check().map_err(|_| ControlFailure::Cancelled)?;
        budget.charge_step().map_err(ControlFailure::Budget)?;
        let term = self.normalize_term(&term);
        match target {
            RecordRowTermTail::Closed => {
                if term.fields.is_empty() && matches!(term.tail, RecordRowTermTail::Closed) {
                    Ok(())
                } else {
                    let field = term.fields.first().map(|field| field.key.clone()).unwrap_or_else(|| FieldKey::symbol("<open-tail>"));
                    Err(ControlFailure::Rejected(RecordRowFailure::ExtraField { field }))
                }
            }
            RecordRowTermTail::Parameter(parameter) => {
                if term.fields.is_empty() && term.tail == RecordRowTermTail::Parameter(parameter) {
                    Ok(())
                } else {
                    Err(ControlFailure::Rejected(RecordRowFailure::RigidTailMismatch {
                        expected: RecordRowTermTail::Parameter(parameter),
                        actual: term.tail,
                    }))
                }
            }
            RecordRowTermTail::Projection(proj) => {
                if term.fields.is_empty() && term.tail == RecordRowTermTail::Projection(proj) {
                    Ok(())
                } else {
                    Err(ControlFailure::Rejected(RecordRowFailure::RigidTailMismatch {
                        expected: RecordRowTermTail::Projection(proj),
                        actual: term.tail,
                    }))
                }
            }
            RecordRowTermTail::Anonymous(rem) => {
                if term.fields.is_empty() && term.tail == RecordRowTermTail::Anonymous(rem.clone()) {
                    Ok(())
                } else {
                    Err(ControlFailure::Rejected(RecordRowFailure::RigidTailMismatch {
                        expected: RecordRowTermTail::Anonymous(rem),
                        actual: term.tail,
                    }))
                }
            }
            RecordRowTermTail::Var(variable) => {
                let variable_term = RecordRowTerm::extension(Vec::new(), RecordRowTermTail::Var(variable));
                if self.occurs(variable, &term) {
                    return Err(ControlFailure::Rejected(RecordRowFailure::OccursCheckFailed { var: variable, term }));
                }
                if term.fields.iter().any(|field| payload_occurs(variable, &field.ty)) {
                    return Err(ControlFailure::Rejected(RecordRowFailure::OccursCheckFailed { var: variable, term }));
                }
                for lack in self.lacks.iter().filter(|lack| lack.row == variable) {
                    if self.term_contains_field(&term, &lack.field) {
                        return Err(ControlFailure::Rejected(RecordRowFailure::LacksViolation {
                            field: lack.field.clone(),
                            row: term.clone(),
                        }));
                    }
                }
                if term == variable_term {
                    return Ok(());
                }
                if let RecordRowTermTail::Var(next) = term.tail {
                    self.propagate_lacks(variable, next);
                }
                self.substitutions.insert(variable, term);
                Ok(())
            }
        }
    }

    fn validate_fields(&self, term: &RecordRowTerm<V>) -> Result<(), ControlFailure<V>> {
        for pair in term.fields.windows(2) {
            if pair[0].key == pair[1].key {
                return Err(ControlFailure::Rejected(RecordRowFailure::DuplicateField(pair[0].key.clone())));
            }
        }
        Ok(())
    }
}
