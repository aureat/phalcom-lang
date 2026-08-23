//! Query-local record row constraint solver.
//!
//! Handles unification, lacks propagation, row extension, and row subtraction.
//! Row variables and terms are strictly query-local and never escape the solver.

use super::id::{RecordRowId, TypeId};
use super::row::{RecordRowData, RecordRowField};
use super::store::TypeStore;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RecordRowVarId(pub u32);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordRowTerm {
    Canonical(RecordRowId),
    Var(RecordRowVarId),
    Extend { fields: Vec<RecordRowField>, tail: Box<RecordRowTerm> },
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordRowFailure {
    OccursCheckFailed { var: RecordRowVarId, term: RecordRowTerm },
    LacksViolation { field: Box<str>, row: RecordRowTerm },
    IncompatibleFields { field: Box<str>, expected: TypeId, actual: TypeId },
    MissingField { field: Box<str> },
    ExtraField { field: Box<str> },
    KindMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordRowBlockedReason {
    UnboundVariable(RecordRowVarId),
    AmbiguousExtension,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RowBudgetReport {
    pub steps: usize,
    pub limit: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IncidentId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordRowSolveResult {
    Solved(RecordRowSolution),
    Rejected(RecordRowFailure),
    Blocked(RecordRowBlockedReason),
    Cancelled,
    BudgetExceeded(RowBudgetReport),
    InternalFailure(IncidentId),
}

pub struct RecordRowSolver {
    next_var: u32,
    substitutions: HashMap<RecordRowVarId, RecordRowTerm>,
    lacks: Vec<RecordRowLacks>,
    step_count: usize,
    step_limit: usize,
}

impl RecordRowSolver {
    pub fn new(step_limit: usize) -> Self {
        Self {
            next_var: 0,
            substitutions: HashMap::new(),
            lacks: Vec::new(),
            step_count: 0,
            step_limit,
        }
    }

    pub fn fresh_var(&mut self) -> RecordRowVarId {
        let id = RecordRowVarId(self.next_var);
        self.next_var += 1;
        id
    }

    pub fn add_lacks(&mut self, row: RecordRowVarId, field: Box<str>) {
        self.lacks.push(RecordRowLacks { row, field });
    }

    pub fn occurs(&self, var: RecordRowVarId, term: &RecordRowTerm) -> bool {
        match term {
            RecordRowTerm::Canonical(_) => false,
            RecordRowTerm::Var(v) => {
                if *v == var {
                    return true;
                }
                if let Some(subst) = self.substitutions.get(v) {
                    self.occurs(var, subst)
                } else {
                    false
                }
            }
            RecordRowTerm::Extend { tail, .. } => self.occurs(var, tail),
        }
    }

    pub fn normalize_term(&self, term: &RecordRowTerm) -> RecordRowTerm {
        match term {
            RecordRowTerm::Canonical(_) => term.clone(),
            RecordRowTerm::Var(v) => {
                if let Some(subst) = self.substitutions.get(v) {
                    self.normalize_term(subst)
                } else {
                    term.clone()
                }
            }
            RecordRowTerm::Extend { fields, tail } => {
                let norm_tail = self.normalize_term(tail);
                RecordRowTerm::Extend {
                    fields: fields.clone(),
                    tail: Box::new(norm_tail),
                }
            }
        }
    }

    pub fn unify(&mut self, left: &RecordRowTerm, right: &RecordRowTerm, store: &TypeStore) -> Result<(), RecordRowFailure> {
        self.step_count += 1;
        if self.step_count > self.step_limit {
            return Err(RecordRowFailure::KindMismatch);
        }

        let left_norm = self.normalize_term(left);
        let right_norm = self.normalize_term(right);

        match (&left_norm, &right_norm) {
            (RecordRowTerm::Var(v1), RecordRowTerm::Var(v2)) if v1 == v2 => Ok(()),
            (RecordRowTerm::Var(v), term) | (term, RecordRowTerm::Var(v)) => {
                if self.occurs(*v, term) {
                    return Err(RecordRowFailure::OccursCheckFailed { var: *v, term: term.clone() });
                }
                for lack in &self.lacks {
                    if lack.row == *v && self.term_contains_field(term, &lack.field, store) {
                        return Err(RecordRowFailure::LacksViolation {
                            field: lack.field.clone(),
                            row: term.clone(),
                        });
                    }
                }
                self.substitutions.insert(*v, term.clone());
                Ok(())
            }
            (RecordRowTerm::Canonical(id1), RecordRowTerm::Canonical(id2)) => {
                if id1 == id2 {
                    return Ok(());
                }
                let r1 = store.record_row(*id1);
                let r2 = store.record_row(*id2);
                if r1.fields.len() != r2.fields.len() || r1.tail != r2.tail {
                    return Err(RecordRowFailure::KindMismatch);
                }
                for (f1, f2) in r1.fields.iter().zip(r2.fields.iter()) {
                    if f1.name != f2.name {
                        return Err(RecordRowFailure::MissingField { field: f1.name.clone() });
                    }
                    if f1.ty != f2.ty {
                        return Err(RecordRowFailure::IncompatibleFields {
                            field: f1.name.clone(),
                            expected: f1.ty,
                            actual: f2.ty,
                        });
                    }
                }
                Ok(())
            }
            (RecordRowTerm::Canonical(id), RecordRowTerm::Extend { fields, tail }) | (RecordRowTerm::Extend { fields, tail }, RecordRowTerm::Canonical(id)) => {
                let row_data = store.record_row(*id);
                let mut remaining_fields = Vec::new();
                for f in row_data.fields.iter() {
                    if let Some(ext_f) = fields.iter().find(|ef| ef.name == f.name) {
                        if ext_f.ty != f.ty {
                            return Err(RecordRowFailure::IncompatibleFields {
                                field: f.name.clone(),
                                expected: ext_f.ty,
                                actual: f.ty,
                            });
                        }
                    } else {
                        remaining_fields.push(f.clone());
                    }
                }
                for ext_f in fields.iter() {
                    if !row_data.fields.iter().any(|f| f.name == ext_f.name) {
                        return Err(RecordRowFailure::ExtraField { field: ext_f.name.clone() });
                    }
                }
                let remainder_row_data = RecordRowData {
                    fields: remaining_fields.into_boxed_slice(),
                    tail: row_data.tail,
                };
                let rem_id = store.find_record_row(&remainder_row_data).ok_or(RecordRowFailure::KindMismatch)?;
                self.unify(tail, &RecordRowTerm::Canonical(rem_id), store)
            }
            (RecordRowTerm::Extend { fields: f1, tail: t1 }, RecordRowTerm::Extend { fields: f2, tail: t2 }) => {
                let mut common = Vec::new();
                let mut only_1 = Vec::new();
                let mut only_2 = Vec::new();
                for f in f1 {
                    if let Some(other) = f2.iter().find(|x| x.name == f.name) {
                        if f.ty != other.ty {
                            return Err(RecordRowFailure::IncompatibleFields {
                                field: f.name.clone(),
                                expected: f.ty,
                                actual: other.ty,
                            });
                        }
                        common.push(f.clone());
                    } else {
                        only_1.push(f.clone());
                    }
                }
                for f in f2 {
                    if !common.iter().any(|x| x.name == f.name) {
                        only_2.push(f.clone());
                    }
                }
                if only_1.is_empty() && only_2.is_empty() {
                    self.unify(t1, t2, store)
                } else if only_1.is_empty() {
                    let ext2 = RecordRowTerm::Extend {
                        fields: only_2,
                        tail: t2.clone(),
                    };
                    self.unify(t1, &ext2, store)
                } else if only_2.is_empty() {
                    let ext1 = RecordRowTerm::Extend {
                        fields: only_1,
                        tail: t1.clone(),
                    };
                    self.unify(&ext1, t2, store)
                } else {
                    let fresh = self.fresh_var();
                    let fresh_term = RecordRowTerm::Var(fresh);
                    let ext_for_1 = RecordRowTerm::Extend {
                        fields: only_2,
                        tail: Box::new(fresh_term.clone()),
                    };
                    let ext_for_2 = RecordRowTerm::Extend {
                        fields: only_1,
                        tail: Box::new(fresh_term),
                    };
                    self.unify(t1, &ext_for_1, store)?;
                    self.unify(t2, &ext_for_2, store)
                }
            }
        }
    }

    fn term_contains_field(&self, term: &RecordRowTerm, field: &str, store: &TypeStore) -> bool {
        match term {
            RecordRowTerm::Canonical(id) => {
                let row = store.record_row(*id);
                row.fields.iter().any(|f| f.name.as_ref() == field)
            }
            RecordRowTerm::Var(v) => {
                if let Some(subst) = self.substitutions.get(v) {
                    self.term_contains_field(subst, field, store)
                } else {
                    false
                }
            }
            RecordRowTerm::Extend { fields, tail } => fields.iter().any(|f| f.name.as_ref() == field) || self.term_contains_field(tail, field, store),
        }
    }

    pub fn solve(mut self, left: &RecordRowTerm, right: &RecordRowTerm, store: &TypeStore) -> RecordRowSolveResult {
        match self.unify(left, right, store) {
            Ok(()) => {
                if self.step_count > self.step_limit {
                    RecordRowSolveResult::BudgetExceeded(RowBudgetReport {
                        steps: self.step_count,
                        limit: self.step_limit,
                    })
                } else {
                    RecordRowSolveResult::Solved(RecordRowSolution {
                        substitutions: self.substitutions,
                    })
                }
            }
            Err(failure) => {
                if self.step_count > self.step_limit {
                    RecordRowSolveResult::BudgetExceeded(RowBudgetReport {
                        steps: self.step_count,
                        limit: self.step_limit,
                    })
                } else {
                    RecordRowSolveResult::Rejected(failure)
                }
            }
        }
    }
}
