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
use std::cell::RefCell;
use std::rc::Rc;

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
    Projection(TypeId),
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
    FixedType(TypeId),
    UntrackedType,
    RecordRow(RecordRowVarId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CombinedInferenceFailure<V = TypeId> {
    RowRejected(RecordRowFailure<V>),
    RowZonk { parameter: TypeParameterId, error: RecordRowZonkError },
    UnderconstrainedType(TypeParameterId),
    UnderconstrainedRow(TypeParameterId),
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
}

fn lift_canonical_row_failure(failure: RecordRowFailure<TypeId>) -> RecordRowFailure<InferenceTerm> {
    match failure {
        RecordRowFailure::OccursCheckFailed { var, term } => RecordRowFailure::OccursCheckFailed {
            var,
            term: RecordRowTerm {
                fields: term
                    .fields
                    .iter()
                    .map(|field| crate::types::row::RecordRowField {
                        name: field.name.clone(),
                        ty: InferenceTerm::Canonical(field.ty),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                tail: term.tail,
            },
        },
        RecordRowFailure::LacksViolation { field, row } => RecordRowFailure::LacksViolation {
            field,
            row: RecordRowTerm {
                fields: row
                    .fields
                    .iter()
                    .map(|field| crate::types::row::RecordRowField {
                        name: field.name.clone(),
                        ty: InferenceTerm::Canonical(field.ty),
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                tail: row.tail,
            },
        },
        RecordRowFailure::IncompatibleFields { field, expected, actual } => RecordRowFailure::IncompatibleFields { field, expected, actual },
        RecordRowFailure::MissingField { field } => RecordRowFailure::MissingField { field },
        RecordRowFailure::ExtraField { field } => RecordRowFailure::ExtraField { field },
        RecordRowFailure::DuplicateField(field) => RecordRowFailure::DuplicateField(field),
        RecordRowFailure::RigidTailMismatch { expected, actual } => RecordRowFailure::RigidTailMismatch { expected, actual },
        RecordRowFailure::KindMismatch => RecordRowFailure::KindMismatch,
    }
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
            for argument in arguments.iter().filter_map(|argument| argument.as_type_id()) {
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
        TypeData::AssociatedProjection(projection) => {
            collect_stable_record_row_lacks_inner(store, projection.subject, visited, emitted, lacks);
            for &argument in projection.trait_ref.arguments.iter() {
                collect_stable_record_row_lacks_inner(store, argument, visited, emitted, lacks);
            }
        }
        TypeData::TypeMemberProjection(projection) => {
            if let crate::types::type_member::ProjectionSubject::Type(ty) = projection.subject {
                collect_stable_record_row_lacks_inner(store, ty, visited, emitted, lacks);
            }
            for argument in projection.arguments.iter() {
                if let crate::types::type_member::TypeMemberArgument::Type(ty) = argument {
                    collect_stable_record_row_lacks_inner(store, *ty, visited, emitted, lacks);
                }
            }
        }
        TypeData::TypePack(elements) => {
            for &element in elements.iter() {
                collect_stable_record_row_lacks_inner(store, element, visited, emitted, lacks);
            }
        }
        TypeData::LabeledTypePack(elements) => {
            for element in elements.iter() {
                collect_stable_record_row_lacks_inner(store, element.element, visited, emitted, lacks);
            }
        }
        TypeData::TupleFromPacks { positionals, labeled } => {
            collect_stable_record_row_lacks_inner(store, *positionals, visited, emitted, lacks);
            collect_stable_record_row_lacks_inner(store, *labeled, visited, emitted, lacks);
        }
        TypeData::CallableFromPacks {
            positionals,
            labeled,
            return_type,
        } => {
            collect_stable_record_row_lacks_inner(store, *positionals, visited, emitted, lacks);
            collect_stable_record_row_lacks_inner(store, *labeled, visited, emitted, lacks);
            collect_stable_record_row_lacks_inner(store, *return_type, visited, emitted, lacks);
        }
        TypeData::CompleteTypePack { positionals, labeled } => {
            collect_stable_record_row_lacks_inner(store, *positionals, visited, emitted, lacks);
            collect_stable_record_row_lacks_inner(store, *labeled, visited, emitted, lacks);
        }
        TypeData::TypePackConcat(segments) | TypeData::LabeledTypePackConcat(segments) => {
            for &segment in segments.iter() {
                collect_stable_record_row_lacks_inner(store, segment, visited, emitted, lacks);
            }
        }
        TypeData::StaticData(term) => {
            for argument in term.arguments.iter() {
                if let crate::types::store::StaticArgument::Term(argument) = argument {
                    collect_stable_record_row_lacks_inner(store, *argument, visited, emitted, lacks);
                }
            }
        }
        TypeData::ConstInt(_) => {}
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
    query_session: Option<Rc<RefCell<super::inference::InferenceCoordinator>>>,
    query_frame: Option<super::inference::InferenceFrameId>,
}

fn materialize_query_row_term(
    session: &super::inference::InferenceSession,
    solution: &RecordRowSolution<InferenceTerm>,
    term: &RecordRowTerm<InferenceTerm>,
    parameter: TypeParameterId,
    store: &mut TypeStore,
    visiting: &mut HashSet<RecordRowVarId>,
) -> Result<crate::types::id::RecordRowId, CombinedInferenceFailure<InferenceTerm>> {
    let mut fields = term
        .fields
        .iter()
        .map(|field| {
            session
                .materialize(&field.ty, store)
                .map(|ty| crate::types::row::RecordRowField {
                    name: field.name.clone(),
                    ty,
                })
                .map_err(|_| CombinedInferenceFailure::UnderconstrainedType(parameter))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let tail = match term.tail {
        RecordRowTermTail::Closed => RecordRowTail::Closed,
        RecordRowTermTail::Parameter(parameter) => RecordRowTail::Parameter(parameter),
        RecordRowTermTail::Projection(projection) => RecordRowTail::Projection(projection),
        RecordRowTermTail::Var(variable) => {
            if !visiting.insert(variable) {
                return Err(CombinedInferenceFailure::RowZonk {
                    parameter,
                    error: RecordRowZonkError::Recursive(variable),
                });
            }
            let Some(next) = solution.term_for(variable) else {
                return Err(CombinedInferenceFailure::RowZonk {
                    parameter,
                    error: RecordRowZonkError::Unsolved(variable),
                });
            };
            let next_id = materialize_query_row_term(session, solution, next, parameter, store, visiting)?;
            visiting.remove(&variable);
            let next = store.record_row(next_id).clone();
            fields.extend(next.fields.into_vec());
            next.tail
        }
    };
    store
        .record_row_checked(fields, tail)
        .map_err(|error| CombinedInferenceFailure::RowZonk {
            parameter,
            error: RecordRowZonkError::Formation(error),
        })
}

pub fn term_has_row_variables(term: &InferenceTerm) -> bool {
    match term {
        InferenceTerm::Record(record) => {
            matches!(record.tail, InferenceRecordTail::Var(_)) || record.fields.iter().any(|field| term_has_row_variables(&field.term))
        }
        InferenceTerm::Applied { origin, arguments } => term_has_row_variables(origin) || arguments.iter().any(static_argument_has_row_variables),
        InferenceTerm::ExactCase { enum_type, .. } => term_has_row_variables(enum_type),
        InferenceTerm::Union(members) => members.iter().any(term_has_row_variables),
        InferenceTerm::Tuple(elements) => elements.iter().any(|element| term_has_row_variables(&element.term)),
        InferenceTerm::Callable(callable) => {
            callable.parameters.iter().any(|parameter| term_has_row_variables(&parameter.term)) || term_has_row_variables(&callable.return_type)
        }
        InferenceTerm::Family(members) => members.iter().any(|member| term_has_row_variables(&member.term)),
        InferenceTerm::AssociatedProjection { subject, trait_ref, .. } => {
            term_has_row_variables(subject) || trait_ref.arguments.iter().any(term_has_row_variables)
        }
        InferenceTerm::TypePack(elements) => elements.iter().any(term_has_row_variables),
        InferenceTerm::LabeledTypePack(elements) => elements.iter().any(|elem| term_has_row_variables(&elem.term)),
        InferenceTerm::CompleteTypePack { positionals, labeled } => term_has_row_variables(positionals) || term_has_row_variables(labeled),
        InferenceTerm::TypePackConcat(segments) | InferenceTerm::LabeledTypePackConcat(segments) => segments.iter().any(term_has_row_variables),
        InferenceTerm::TupleFromPacks { positionals, labeled } => term_has_row_variables(positionals) || term_has_row_variables(labeled),
        InferenceTerm::CallableFromPacks {
            positionals,
            labeled,
            return_type,
        } => term_has_row_variables(positionals) || term_has_row_variables(labeled) || term_has_row_variables(return_type),
        InferenceTerm::PackProjection { subject, .. } => term_has_row_variables(subject),
        InferenceTerm::StaticData { arguments, .. } => arguments.iter().any(static_argument_has_row_variables),
        InferenceTerm::Canonical(_) | InferenceTerm::Var(_) | InferenceTerm::Rigid(_) => false,
    }
}

fn static_argument_has_row_variables(argument: &super::inference::InferenceStaticArgument) -> bool {
    match argument {
        super::inference::InferenceStaticArgument::Term(term) => term_has_row_variables(term),
        super::inference::InferenceStaticArgument::RecordRow(row) => {
            matches!(row.tail, InferenceRecordTail::Var(_)) || row.fields.iter().any(|field| term_has_row_variables(&field.term))
        }
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
            query_session: None,
            query_frame: None,
        }
    }

    /// Creates a lightweight application view over the query-owned inference
    /// graph. All row identities allocated here remain valid until the root
    /// query context is dropped.
    pub(crate) fn new_for_query_context(
        generic_signatures: &[&GenericSignature],
        store: &TypeStore,
        session: Rc<RefCell<super::inference::InferenceCoordinator>>,
        frame: super::inference::InferenceFrameId,
        type_terms: &std::collections::HashMap<TypeParameterId, InferenceTerm>,
        fixed_generics: &[(TypeParameterId, TypeId)],
    ) -> Self {
        let mut parameter_bindings = std::collections::HashMap::new();
        let fixed_generics = fixed_generics.iter().copied().collect::<std::collections::HashMap<_, _>>();
        for generic_signature in generic_signatures {
            for &parameter in generic_signature.parameters.iter() {
                let kind = store.type_parameter(parameter).kind;
                let binding = if kind == KindId::RECORD_ROW {
                    let variable = session.borrow_mut().fresh_record_row_variable_in_frame(frame);
                    session.borrow_mut().bind_record_row_parameter(frame, parameter, variable);
                    GenericInferenceBinding::RecordRow(variable)
                } else if let Some(InferenceTerm::Var(variable)) = type_terms.get(&parameter) {
                    GenericInferenceBinding::Type(*variable)
                } else if let Some(&ty) = fixed_generics.get(&parameter) {
                    GenericInferenceBinding::FixedType(ty)
                } else {
                    // Every non-row declaration parameter is normally
                    // instantiated in `type_terms`; this fallback preserves
                    // the binding classification if a future domain adds a
                    // non-inference fixed form.
                    GenericInferenceBinding::UntrackedType
                };
                parameter_bindings.insert(parameter, binding);
            }
        }
        Self {
            // These legacy fields are retained for the standalone helper API;
            // query work is routed through `query_session` below.
            types: super::inference::InferenceSession::new(),
            rows: RecordRowSolver::new(),
            parameter_bindings,
            row_equations: Vec::new(),
            query_session: Some(session),
            query_frame: Some(frame),
        }
    }

    pub fn type_terms(&self) -> std::collections::HashMap<TypeParameterId, InferenceTerm> {
        self.parameter_bindings
            .iter()
            .filter_map(|(&parameter, binding)| match binding {
                GenericInferenceBinding::Type(variable) => Some((parameter, InferenceTerm::Var(*variable))),
                GenericInferenceBinding::FixedType(ty) => Some((parameter, InferenceTerm::Canonical(*ty))),
                GenericInferenceBinding::RecordRow(_) | GenericInferenceBinding::UntrackedType => None,
            })
            .collect()
    }

    pub fn row_terms(&self) -> std::collections::HashMap<TypeParameterId, RecordRowVarId> {
        let mut terms = self
            .query_session
            .as_ref()
            .zip(self.query_frame)
            .map(|(session, frame)| session.borrow().row_parameter_terms_for_frame(frame))
            .unwrap_or_default();
        terms.extend(self.parameter_bindings
            .iter()
            .filter_map(|(&parameter, binding)| match binding {
                GenericInferenceBinding::RecordRow(variable) => Some((parameter, *variable)),
                GenericInferenceBinding::Type(_) | GenericInferenceBinding::FixedType(_) | GenericInferenceBinding::UntrackedType => None,
            })
        );
        terms
    }

    pub fn type_term(&self, ty: TypeId, store: &TypeStore) -> InferenceTerm {
        let type_terms = self.type_terms();
        let row_terms = self.row_terms();
        if let Some(session) = &self.query_session {
            session.borrow().type_id_to_inference_with_rows(ty, &type_terms, &row_terms, store)
        } else {
            self.types.type_id_to_inference_with_rows(ty, &type_terms, &row_terms, store)
        }
    }

    pub fn constrain_signature_type_lacks(&mut self, ty: TypeId, store: &TypeStore) -> Result<(), CombinedInferenceFailure<InferenceTerm>> {
        for lack in collect_stable_record_row_lacks(store, ty) {
            let Some(GenericInferenceBinding::RecordRow(variable)) = self.parameter_bindings.get(&lack.parameter).copied() else {
                continue;
            };
            if let Some(session) = &self.query_session {
                session
                    .borrow_mut()
                    .add_record_row_lacks(variable, lack.field)
                    .map_err(CombinedInferenceFailure::RowRejected)?;
            } else {
                self.rows
                    .add_lacks(variable, lack.field)
                    .map_err(|failure| CombinedInferenceFailure::RowRejected(lift_canonical_row_failure(failure)))?;
            }
        }
        Ok(())
    }

    pub fn constrain_known_record_argument(
        &mut self,
        actual_ty: TypeId,
        formal: &InferenceRecord,
        store: &TypeStore,
    ) -> Result<Option<Vec<(InferenceTerm, InferenceTerm)>>, CombinedInferenceFailure<InferenceTerm>> {
        let TypeData::Record(actual_row_id) = store.get(actual_ty) else {
            return Ok(None);
        };
        let actual_row = store.record_row(*actual_row_id).clone();
        let mut ordinary = Vec::new();
        for required in formal.fields.iter() {
            let Some(actual_field) = actual_row.find_field(&required.name) else {
                return if actual_row.tail == RecordRowTail::Closed {
                    Err(CombinedInferenceFailure::RowRejected(lift_canonical_row_failure(RecordRowFailure::MissingField {
                        field: required.name.clone(),
                    })))
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
            RecordRowTail::Projection(proj) => RecordRowTermTail::Projection(proj),
        };
        match formal.tail {
            InferenceRecordTail::Closed => {}
            InferenceRecordTail::Parameter(parameter) => {
                if !extra.is_empty() || actual_tail != RecordRowTermTail::Parameter(parameter) {
                    return Err(CombinedInferenceFailure::RowRejected(lift_canonical_row_failure(RecordRowFailure::RigidTailMismatch {
                        expected: RecordRowTermTail::Parameter(parameter),
                        actual: actual_tail,
                    })));
                }
            }
            InferenceRecordTail::Projection(proj) => {
                if !extra.is_empty() || actual_tail != RecordRowTermTail::Projection(proj) {
                    return Err(CombinedInferenceFailure::RowRejected(lift_canonical_row_failure(RecordRowFailure::RigidTailMismatch {
                        expected: RecordRowTermTail::Projection(proj),
                        actual: actual_tail,
                    })));
                }
            }
            InferenceRecordTail::Var(variable) => {
                if let Some(session) = &self.query_session {
                    let left = InferenceRecord {
                        fields: Box::new([]),
                        tail: InferenceRecordTail::Var(variable),
                    };
                    let right = InferenceRecord {
                        fields: extra
                            .into_iter()
                            .map(|field| InferenceRecordField {
                                name: field.name,
                                term: InferenceTerm::Canonical(field.ty),
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        tail: match actual_tail {
                            RecordRowTermTail::Closed => InferenceRecordTail::Closed,
                            RecordRowTermTail::Parameter(parameter) => InferenceRecordTail::Parameter(parameter),
                            RecordRowTermTail::Projection(projection) => InferenceRecordTail::Projection(projection),
                            RecordRowTermTail::Var(variable) => InferenceRecordTail::Var(variable),
                        },
                    };
                    session.borrow_mut().enqueue_record_row_equation(&left, &right);
                } else {
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
        }
        Ok(Some(ordinary))
    }

    pub fn solve_rows(&mut self, store: &TypeStore, budget: &mut QueryBudget, cancellation: &CancellationToken) -> RecordRowSolveResult {
        self.rows.solve_many(&self.row_equations, store, budget, cancellation)
    }

    pub fn solve_rows_for_query(&self, budget: &mut QueryBudget, cancellation: &CancellationToken) -> RecordRowSolveResult<InferenceTerm> {
        let Some(session) = &self.query_session else {
            return RecordRowSolveResult::InternalFailure(crate::types::row_solver::IncidentId(0));
        };
        let roots = self
            .parameter_bindings
            .values()
            .filter_map(|binding| match binding {
                GenericInferenceBinding::RecordRow(variable) => Some(*variable),
                GenericInferenceBinding::Type(_) | GenericInferenceBinding::FixedType(_) | GenericInferenceBinding::UntrackedType => None,
            })
            .collect::<Vec<_>>();
        session.borrow().row_solution_for_roots(&roots, budget, cancellation)
    }

    pub fn has_pending_ancestor_row_dependency(&self, solution: &RecordRowSolution<InferenceTerm>, term: &InferenceTerm) -> bool {
        let (Some(session), Some(frame)) = (&self.query_session, self.query_frame) else {
            return false;
        };
        session.borrow().term_has_unresolved_ancestor_row(term, solution, frame)
    }

    pub fn build_instantiation(
        &mut self,
        type_solution: &super::inference::InferenceSolution,
        row_solution: &RecordRowSolution,
        store: &mut TypeStore,
    ) -> Result<GenericInstantiation, CombinedInferenceFailure> {
        self.build_instantiation_from_types(&self.types, type_solution, row_solution, store)
    }

    pub fn build_instantiation_from_active_terms(
        &self,
        session: &super::inference::InferenceSession,
        type_terms: &std::collections::HashMap<TypeParameterId, InferenceTerm>,
        row_solution: &RecordRowSolution,
        store: &mut TypeStore,
    ) -> Result<GenericInstantiation, CombinedInferenceFailure> {
        let mut result = GenericInstantiation::default();
        for (&parameter, binding) in &self.parameter_bindings {
            match binding {
                GenericInferenceBinding::Type(_) | GenericInferenceBinding::FixedType(_) | GenericInferenceBinding::UntrackedType => {
                    let Some(term) = type_terms.get(&parameter) else {
                        return Err(CombinedInferenceFailure::UnderconstrainedType(parameter));
                    };
                    let ty = session
                        .materialize(term, store)
                        .map_err(|_| CombinedInferenceFailure::UnderconstrainedType(parameter))?;
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

    pub fn build_instantiation_from_active_terms_in_query(
        &self,
        session: &super::inference::InferenceSession,
        type_terms: &std::collections::HashMap<TypeParameterId, InferenceTerm>,
        row_solution: &RecordRowSolution<InferenceTerm>,
        store: &mut TypeStore,
    ) -> Result<GenericInstantiation, CombinedInferenceFailure<InferenceTerm>> {
        let mut result = GenericInstantiation::default();
        for (&parameter, binding) in &self.parameter_bindings {
            match binding {
                GenericInferenceBinding::Type(_) | GenericInferenceBinding::FixedType(_) | GenericInferenceBinding::UntrackedType => {
                    let Some(term) = type_terms.get(&parameter) else {
                        return Err(CombinedInferenceFailure::UnderconstrainedType(parameter));
                    };
                    let ty = session
                        .materialize(term, store)
                        .map_err(|_| CombinedInferenceFailure::UnderconstrainedType(parameter))?;
                    result.bind_type(parameter, ty);
                }
                GenericInferenceBinding::RecordRow(variable) => {
                    let Some(term) = row_solution.term_for(*variable) else {
                        return Err(CombinedInferenceFailure::UnderconstrainedRow(parameter));
                    };
                    let row = materialize_query_row_term(session, row_solution, term, parameter, store, &mut HashSet::new())?;
                    result.bind_row(parameter, row);
                }
            }
        }
        Ok(result)
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
                GenericInferenceBinding::FixedType(ty) => result.bind_type(parameter, *ty),
                GenericInferenceBinding::UntrackedType => return Err(CombinedInferenceFailure::UnderconstrainedType(parameter)),
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
