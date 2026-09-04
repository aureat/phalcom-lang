//! Record-row inference domain kept separate from ordinary type inference.

use crate::identity::InferVarId;
use crate::types::id::{KindId, TypeId, TypeParameterId};
use crate::types::instantiation::GenericInstantiation;
use crate::types::outcome::{BlockReason, BudgetReport, CancellationToken, QueryBudget};
use crate::types::parameter::GenericSignature;
use crate::types::row::RecordRowTail;
use crate::types::row_solver::{
    RecordRowFailure, RecordRowSolution, RecordRowSolveResult, RecordRowSolver, RecordRowTerm, RecordRowTermTail, RecordRowVarId, RecordRowZonkError,
};
use crate::types::store::{TypeData, TypeStore};
use std::collections::HashSet;

use super::inference::InferenceTerm;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceRecordField {
    pub name: Box<str>,
    pub term: InferenceTerm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InferenceRecordTail {
    Closed,
    Parameter(TypeParameterId),
    Var(RecordRowVarId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferenceRecord {
    pub fields: Box<[InferenceRecordField]>,
    pub tail: InferenceRecordTail,
}

/// Stable semantic evidence that an open Record tail excludes one field.
/// This deliberately contains no query-local row-solver identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct StableRecordRowLack {
    pub parameter: TypeParameterId,
    pub field: Box<str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenericInferenceBinding {
    Type(InferVarId),
    RecordRow(RecordRowVarId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CombinedInferenceFailure {
    RowRejected(RecordRowFailure),
    RowZonk { parameter: TypeParameterId, error: RecordRowZonkError },
    UnderconstrainedType(TypeParameterId),
    UnderconstrainedRow(TypeParameterId),
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
}

pub(crate) fn collect_stable_record_row_lacks(store: &TypeStore, ty: TypeId) -> Vec<StableRecordRowLack> {
    let mut lacks = Vec::new();
    let mut visited = HashSet::new();
    let mut emitted = HashSet::new();
    collect_stable_record_row_lacks_inner(store, ty, &mut visited, &mut emitted, &mut lacks);
    lacks
}

fn collect_stable_record_row_lacks_inner(
    store: &TypeStore,
    ty: TypeId,
    visited: &mut HashSet<TypeId>,
    emitted: &mut HashSet<(TypeParameterId, Box<str>)>,
    lacks: &mut Vec<StableRecordRowLack>,
) {
    if !visited.insert(ty) {
        return;
    }

    match store.get(ty) {
        TypeData::Record(row_id) => {
            let row = store.record_row(*row_id);
            if let RecordRowTail::Parameter(parameter) = row.tail {
                for field in row.fields.iter() {
                    let key = (parameter, field.name.clone());
                    if emitted.insert(key) {
                        lacks.push(StableRecordRowLack {
                            parameter,
                            field: field.name.clone(),
                        });
                    }
                }
            }
            for field in row.fields.iter() {
                collect_stable_record_row_lacks_inner(store, field.ty, visited, emitted, lacks);
            }
        }
        TypeData::Applied { origin, arguments } => {
            collect_stable_record_row_lacks_inner(store, *origin, visited, emitted, lacks);
            for &argument in arguments.iter() {
                collect_stable_record_row_lacks_inner(store, argument, visited, emitted, lacks);
            }
        }
        TypeData::ExactCase { enum_type, .. } => collect_stable_record_row_lacks_inner(store, *enum_type, visited, emitted, lacks),
        TypeData::Union(members) => {
            for &member in members.iter() {
                collect_stable_record_row_lacks_inner(store, member, visited, emitted, lacks);
            }
        }
        TypeData::Tuple(elements) => {
            for element in elements.iter() {
                collect_stable_record_row_lacks_inner(store, element.ty, visited, emitted, lacks);
            }
        }
        TypeData::Callable(callable) => {
            for parameter in callable.parameters.iter() {
                collect_stable_record_row_lacks_inner(store, parameter.ty, visited, emitted, lacks);
            }
            collect_stable_record_row_lacks_inner(store, callable.return_type, visited, emitted, lacks);
        }
        TypeData::Family(family_id) => {
            for member in store.get_family(*family_id).members.iter() {
                collect_stable_record_row_lacks_inner(store, member.ty, visited, emitted, lacks);
            }
        }
        TypeData::Never
        | TypeData::Unit
        | TypeData::ClassObject { .. }
        | TypeData::Nominal { .. }
        | TypeData::Parameter(_)
        | TypeData::Lambda(_)
        | TypeData::SelfType(_) => {}
    }
}

/// Coordinates ordinary type inference and query-local row inference without
/// allowing either solver to represent values from the other domain.
pub struct GenericApplicationSession {
    pub types: super::inference::InferenceSession,
    pub rows: RecordRowSolver,
    pub parameter_bindings: std::collections::HashMap<TypeParameterId, GenericInferenceBinding>,
    row_equations: Vec<(RecordRowTerm, RecordRowTerm)>,
}

pub fn term_has_row_variables(term: &InferenceTerm) -> bool {
    match term {
        InferenceTerm::Record(record) => {
            matches!(record.tail, InferenceRecordTail::Var(_)) || record.fields.iter().any(|field| term_has_row_variables(&field.term))
        }
        InferenceTerm::Applied { origin, arguments } => term_has_row_variables(origin) || arguments.iter().any(term_has_row_variables),
        InferenceTerm::ExactCase { enum_type, .. } => term_has_row_variables(enum_type),
        InferenceTerm::Union(members) => members.iter().any(term_has_row_variables),
        InferenceTerm::Tuple(elements) => elements.iter().any(|element| term_has_row_variables(&element.term)),
        InferenceTerm::Callable(callable) => {
            callable.parameters.iter().any(|parameter| term_has_row_variables(&parameter.term)) || term_has_row_variables(&callable.return_type)
        }
        InferenceTerm::Family(members) => members.iter().any(|member| term_has_row_variables(&member.term)),
        InferenceTerm::Canonical(_) | InferenceTerm::Var(_) | InferenceTerm::Rigid(_) => false,
    }
}

impl GenericApplicationSession {
    pub fn new(generic_signature: &GenericSignature, store: &TypeStore) -> Self {
        Self::new_for_domains(&[generic_signature], store)
    }

    /// Creates one combined row/type application session for several
    /// owner-preserving generic signatures. The signatures remain canonical
    /// products; only their application variables share this query-local
    /// session.
    pub fn new_for_domains(generic_signatures: &[&GenericSignature], store: &TypeStore) -> Self {
        let mut types = super::inference::InferenceSession::new();
        let mut rows = RecordRowSolver::new();
        let mut parameter_bindings = std::collections::HashMap::new();
        for generic_signature in generic_signatures {
            for &parameter in generic_signature.parameters.iter() {
                let kind = store.type_parameter(parameter).kind;
                let binding = if kind == KindId::RECORD_ROW {
                    GenericInferenceBinding::RecordRow(rows.fresh_var())
                } else {
                    GenericInferenceBinding::Type(types.fresh_variable(kind))
                };
                parameter_bindings.insert(parameter, binding);
            }
        }
        Self {
            types,
            rows,
            parameter_bindings,
            row_equations: Vec::new(),
        }
    }

    pub fn type_terms(&self) -> std::collections::HashMap<TypeParameterId, InferenceTerm> {
        self.parameter_bindings
            .iter()
            .filter_map(|(&parameter, binding)| match binding {
                GenericInferenceBinding::Type(variable) => Some((parameter, InferenceTerm::Var(*variable))),
                GenericInferenceBinding::RecordRow(_) => None,
            })
            .collect()
    }

    pub fn row_terms(&self) -> std::collections::HashMap<TypeParameterId, RecordRowVarId> {
        self.parameter_bindings
            .iter()
            .filter_map(|(&parameter, binding)| match binding {
                GenericInferenceBinding::RecordRow(variable) => Some((parameter, *variable)),
                GenericInferenceBinding::Type(_) => None,
            })
            .collect()
    }

    pub fn type_term(&self, ty: TypeId, store: &TypeStore) -> InferenceTerm {
        self.types.type_id_to_inference_with_rows(ty, &self.type_terms(), &self.row_terms(), store)
    }

    pub fn constrain_signature_type_lacks(&mut self, ty: TypeId, store: &TypeStore) -> Result<(), CombinedInferenceFailure> {
        for lack in collect_stable_record_row_lacks(store, ty) {
            let Some(GenericInferenceBinding::RecordRow(variable)) = self.parameter_bindings.get(&lack.parameter).copied() else {
                continue;
            };
            self.rows.add_lacks(variable, lack.field).map_err(CombinedInferenceFailure::RowRejected)?;
        }
        Ok(())
    }

    pub fn constrain_known_record_argument(
        &mut self,
        actual_ty: TypeId,
        formal: &InferenceRecord,
        store: &TypeStore,
    ) -> Result<Option<Vec<(InferenceTerm, InferenceTerm)>>, CombinedInferenceFailure> {
        let TypeData::Record(actual_row_id) = store.get(actual_ty) else {
            return Ok(None);
        };
        let actual_row = store.record_row(*actual_row_id).clone();
        let mut ordinary = Vec::new();
        for required in formal.fields.iter() {
            let Some(actual_field) = actual_row.find_field(&required.name) else {
                return if actual_row.tail == RecordRowTail::Closed {
                    Err(CombinedInferenceFailure::RowRejected(RecordRowFailure::MissingField {
                        field: required.name.clone(),
                    }))
                } else {
                    Err(CombinedInferenceFailure::Blocked(BlockReason::RecursiveFixpoint))
                };
            };
            ordinary.push((InferenceTerm::Canonical(actual_field), required.term.clone()));
        }

        let extra = actual_row
            .fields
            .iter()
            .filter(|field| !formal.fields.iter().any(|required| required.name == field.name))
            .cloned()
            .collect::<Vec<_>>();
        let actual_tail = match actual_row.tail {
            RecordRowTail::Closed => RecordRowTermTail::Closed,
            RecordRowTail::Parameter(parameter) => RecordRowTermTail::Parameter(parameter),
        };
        match formal.tail {
            InferenceRecordTail::Closed => {}
            InferenceRecordTail::Parameter(parameter) => {
                if !extra.is_empty() || actual_tail != RecordRowTermTail::Parameter(parameter) {
                    return Err(CombinedInferenceFailure::RowRejected(RecordRowFailure::RigidTailMismatch {
                        expected: RecordRowTermTail::Parameter(parameter),
                        actual: actual_tail,
                    }));
                }
            }
            InferenceRecordTail::Var(variable) => {
                self.row_equations.push((
                    RecordRowTerm {
                        fields: Box::new([]),
                        tail: RecordRowTermTail::Var(variable),
                    },
                    RecordRowTerm {
                        fields: extra.into_boxed_slice(),
                        tail: actual_tail,
                    },
                ));
            }
        }
        Ok(Some(ordinary))
    }

    pub fn solve_rows(&mut self, store: &TypeStore, budget: &mut QueryBudget, cancellation: &CancellationToken) -> RecordRowSolveResult {
        self.rows.solve_many(&self.row_equations, store, budget, cancellation)
    }

    pub fn build_instantiation(
        &mut self,
        type_solution: &super::inference::InferenceSolution,
        row_solution: &RecordRowSolution,
        store: &mut TypeStore,
    ) -> Result<GenericInstantiation, CombinedInferenceFailure> {
        self.build_instantiation_from_types(&self.types, type_solution, row_solution, store)
    }

    pub fn build_instantiation_from_types(
        &self,
        types: &super::inference::InferenceSession,
        type_solution: &super::inference::InferenceSolution,
        row_solution: &RecordRowSolution,
        store: &mut TypeStore,
    ) -> Result<GenericInstantiation, CombinedInferenceFailure> {
        let mut result = GenericInstantiation::default();
        for (&parameter, binding) in &self.parameter_bindings {
            match binding {
                GenericInferenceBinding::Type(variable) => {
                    let Some(ty) = types.solved_type_for(type_solution, *variable) else {
                        return Err(CombinedInferenceFailure::UnderconstrainedType(parameter));
                    };
                    result.bind_type(parameter, ty);
                }
                GenericInferenceBinding::RecordRow(variable) => {
                    let Some(_) = row_solution.term_for(*variable) else {
                        return Err(CombinedInferenceFailure::UnderconstrainedRow(parameter));
                    };
                    let row = row_solution
                        .zonk_variable_to_canonical(*variable, store)
                        .map_err(|error| CombinedInferenceFailure::RowZonk { parameter, error })?;
                    result.bind_row(parameter, row);
                }
            }
        }
        Ok(result)
    }
}
