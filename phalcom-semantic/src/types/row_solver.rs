//! Query-local record-row constraint solver.
//!
//! Solver terms are normalized structural rows. They contain no canonical row
//! IDs, so exploration never depends on speculative rows interned by an
//! earlier query.

use super::id::{RecordRowId, TypeId, TypeParameterId};
use super::outcome::{BudgetReport, CancellationToken, QueryBudget};
use super::row::{RecordRowField, RecordRowFormationError, RecordRowTail};
use super::store::TypeStore;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RecordRowVarId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RecordRowTermTail {
    Closed,
    Parameter(TypeParameterId),
    Var(RecordRowVarId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordRowTerm {
    pub fields: Box<[RecordRowField]>,
    pub tail: RecordRowTermTail,
}

impl RecordRowTerm {
    pub fn new(fields: Vec<RecordRowField>, tail: RecordRowTermTail) -> Result<Self, Box<str>> {
        let mut fields = fields;
        fields.sort_by(|left, right| left.name.cmp(&right.name));
        for pair in fields.windows(2) {
            if pair[0].name == pair[1].name {
                return Err(pair[0].name.clone());
            }
        }
        Ok(Self {
            fields: fields.into_boxed_slice(),
            tail,
        })
    }

    pub fn closed(fields: Vec<RecordRowField>) -> Result<Self, Box<str>> {
        Self::new(fields, RecordRowTermTail::Closed)
    }

    pub fn from_canonical(store: &TypeStore, row: RecordRowId) -> Self {
        let data = store.record_row(row);
        Self {
            fields: data.fields.clone(),
            tail: match data.tail {
                RecordRowTail::Closed => RecordRowTermTail::Closed,
                RecordRowTail::Parameter(parameter) => RecordRowTermTail::Parameter(parameter),
            },
        }
    }

    fn extension(fields: Vec<RecordRowField>, tail: RecordRowTermTail) -> Self {
        let mut fields = fields;
        fields.sort_by(|left, right| left.name.cmp(&right.name));
        Self {
            fields: fields.into_boxed_slice(),
            tail,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordRowLacks {
    pub row: RecordRowVarId,
    pub field: Box<str>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecordRowSolution {
    pub substitutions: HashMap<RecordRowVarId, RecordRowTerm>,
}

impl RecordRowSolution {
    pub fn term_for(&self, variable: RecordRowVarId) -> Option<&RecordRowTerm> {
        self.substitutions.get(&variable)
    }

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
        let tail = match term.tail {
            RecordRowTermTail::Closed => RecordRowTail::Closed,
            RecordRowTermTail::Parameter(parameter) => RecordRowTail::Parameter(parameter),
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
pub enum RecordRowFailure {
    OccursCheckFailed { var: RecordRowVarId, term: RecordRowTerm },
    LacksViolation { field: Box<str>, row: RecordRowTerm },
    IncompatibleFields { field: Box<str>, expected: TypeId, actual: TypeId },
    MissingField { field: Box<str> },
    ExtraField { field: Box<str> },
    DuplicateField(Box<str>),
    RigidTailMismatch { expected: RecordRowTermTail, actual: RecordRowTermTail },
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
pub enum RecordRowSolveResult {
    Solved(RecordRowSolution),
    Underconstrained(RecordRowUnderconstrained),
    Rejected(RecordRowFailure),
    Blocked(RecordRowBlockedReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(IncidentId),
}

enum ControlFailure {
    Cancelled,
    Budget(BudgetReport),
    Rejected(RecordRowFailure),
}

pub struct RecordRowSolver {
    next_var: u32,
    allocated: Vec<RecordRowVarId>,
    substitutions: HashMap<RecordRowVarId, RecordRowTerm>,
    lacks: Vec<RecordRowLacks>,
}

impl Default for RecordRowSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordRowSolver {
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

    pub fn add_lacks(&mut self, row: RecordRowVarId, field: Box<str>) -> Result<(), RecordRowFailure> {
        let row_term = self.normalize_term(&RecordRowTerm::extension(Vec::new(), RecordRowTermTail::Var(row)));
        if self.term_contains_field(&row_term, &field) {
            return Err(RecordRowFailure::LacksViolation { field, row: row_term });
        }
        self.lacks.push(RecordRowLacks { row, field: field.clone() });
        self.propagate_lack_alias(row, field);
        Ok(())
    }

    pub fn occurs(&self, var: RecordRowVarId, term: &RecordRowTerm) -> bool {
        let term = self.normalize_term(term);
        matches!(term.tail, RecordRowTermTail::Var(other) if other == var)
    }

    pub fn normalize_term(&self, term: &RecordRowTerm) -> RecordRowTerm {
        self.normalize_term_with_seen(term, &mut HashSet::new())
    }

    fn normalize_term_with_seen(&self, term: &RecordRowTerm, seen: &mut HashSet<RecordRowVarId>) -> RecordRowTerm {
        let mut fields = term.fields.to_vec();
        let tail = match term.tail {
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

    pub fn unify(&mut self, left: &RecordRowTerm, right: &RecordRowTerm, store: &TypeStore) -> Result<(), RecordRowFailure> {
        let mut budget = QueryBudget::default();
        let cancellation = CancellationToken::new();
        self.unify_with_control(left, right, store, &mut budget, &cancellation)
            .map_err(|failure| match failure {
                ControlFailure::Rejected(error) => error,
                ControlFailure::Cancelled | ControlFailure::Budget(_) => RecordRowFailure::KindMismatch,
            })
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
        for (left, right) in equations {
            match self.unify_with_control(left, right, store, budget, cancellation) {
                Err(ControlFailure::Cancelled) => return RecordRowSolveResult::Cancelled,
                Err(ControlFailure::Budget(report)) => return RecordRowSolveResult::BudgetExceeded(report),
                Err(ControlFailure::Rejected(error)) => return RecordRowSolveResult::Rejected(error),
                Ok(()) => {}
            }
        }

        let unresolved = self
            .allocated
            .iter()
            .copied()
            .filter(|variable| {
                matches!(
                    self.normalize_term(&RecordRowTerm::extension(Vec::new(), RecordRowTermTail::Var(*variable))).tail,
                    RecordRowTermTail::Var(current) if current == *variable
                )
            })
            .collect::<Vec<_>>();
        if !unresolved.is_empty() {
            return RecordRowSolveResult::Underconstrained(RecordRowUnderconstrained {
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
        RecordRowSolveResult::Solved(RecordRowSolution { substitutions })
    }

    fn unify_with_control(
        &mut self,
        left: &RecordRowTerm,
        right: &RecordRowTerm,
        _store: &TypeStore,
        budget: &mut QueryBudget,
        cancellation: &CancellationToken,
    ) -> Result<(), ControlFailure> {
        cancellation.check().map_err(|_| ControlFailure::Cancelled)?;
        budget.charge_step().map_err(ControlFailure::Budget)?;

        let left = self.normalize_term(left);
        let right = self.normalize_term(right);
        self.validate_fields(&left)?;
        self.validate_fields(&right)?;

        let mut only_left = Vec::new();
        let mut only_right = Vec::new();
        for field in left.fields.iter() {
            if let Some(other) = right.fields.iter().find(|candidate| candidate.name == field.name) {
                if field.ty != other.ty {
                    return Err(ControlFailure::Rejected(RecordRowFailure::IncompatibleFields {
                        field: field.name.clone(),
                        expected: field.ty,
                        actual: other.ty,
                    }));
                }
            } else {
                only_left.push(field.clone());
            }
        }
        for field in right.fields.iter() {
            if !left.fields.iter().any(|candidate| candidate.name == field.name) {
                only_right.push(field.clone());
            }
        }

        if only_left.is_empty() && only_right.is_empty() {
            return self.unify_tails(left.tail, right.tail, budget, cancellation);
        }
        if only_left.is_empty() {
            return self.bind_tail(left.tail, RecordRowTerm::extension(only_right, right.tail), budget, cancellation);
        }
        if only_right.is_empty() {
            return self.bind_tail(right.tail, RecordRowTerm::extension(only_left, left.tail), budget, cancellation);
        }

        let fresh = self.fresh_var();
        self.bind_tail(
            left.tail,
            RecordRowTerm::extension(only_right, RecordRowTermTail::Var(fresh)),
            budget,
            cancellation,
        )?;
        self.bind_tail(
            right.tail,
            RecordRowTerm::extension(only_left, RecordRowTermTail::Var(fresh)),
            budget,
            cancellation,
        )
    }

    fn unify_tails(
        &mut self,
        left: RecordRowTermTail,
        right: RecordRowTermTail,
        budget: &mut QueryBudget,
        cancellation: &CancellationToken,
    ) -> Result<(), ControlFailure> {
        if left == right {
            return Ok(());
        }
        match (left, right) {
            (RecordRowTermTail::Var(variable), tail) | (tail, RecordRowTermTail::Var(variable)) => self.bind_tail(
                RecordRowTermTail::Var(variable),
                RecordRowTerm::extension(Vec::new(), tail),
                budget,
                cancellation,
            ),
            (expected, actual) => Err(ControlFailure::Rejected(RecordRowFailure::RigidTailMismatch { expected, actual })),
        }
    }

    fn bind_tail(
        &mut self,
        target: RecordRowTermTail,
        term: RecordRowTerm,
        budget: &mut QueryBudget,
        cancellation: &CancellationToken,
    ) -> Result<(), ControlFailure> {
        cancellation.check().map_err(|_| ControlFailure::Cancelled)?;
        budget.charge_step().map_err(ControlFailure::Budget)?;
        let term = self.normalize_term(&term);
        match target {
            RecordRowTermTail::Closed => {
                if term.fields.is_empty() && matches!(term.tail, RecordRowTermTail::Closed) {
                    Ok(())
                } else {
                    let field = term.fields.first().map(|field| field.name.clone()).unwrap_or_else(|| "<open-tail>".into());
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
            RecordRowTermTail::Var(variable) => {
                let variable_term = RecordRowTerm::extension(Vec::new(), RecordRowTermTail::Var(variable));
                if self.occurs(variable, &term) {
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

    fn validate_fields(&self, term: &RecordRowTerm) -> Result<(), ControlFailure> {
        for pair in term.fields.windows(2) {
            if pair[0].name == pair[1].name {
                return Err(ControlFailure::Rejected(RecordRowFailure::DuplicateField(pair[0].name.clone())));
            }
        }
        Ok(())
    }

    fn term_contains_field(&self, term: &RecordRowTerm, field: &str) -> bool {
        term.fields.iter().any(|candidate| candidate.name.as_ref() == field)
    }

    fn propagate_lack_alias(&mut self, row: RecordRowVarId, field: Box<str>) {
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
