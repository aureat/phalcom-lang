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
use std::cell::RefCell;
use std::collections::HashSet;
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
    Anonymous(crate::types::row::AnonymousRecordRemainder),
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
                        presence: field.presence,
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
                        presence: field.presence,
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
    StableRowLackCollector::new(store).collect(ty)
}

struct StableRowLackCollector<'a> {
    store: &'a TypeStore,
    visited_types: HashSet<TypeId>,
    visited_rows: HashSet<crate::types::id::RecordRowId>,
    visited_domains: HashSet<crate::types::id::StaticDomainId>,
    visited_lambdas: HashSet<crate::types::id::TypeLambdaId>,
    visited_scoped_types: HashSet<crate::types::id::ScopedTypeId>,
    emitted: HashSet<(TypeParameterId, Box<str>)>,
    lacks: Vec<StableRecordRowLack>,
}

impl<'a> StableRowLackCollector<'a> {
    fn new(store: &'a TypeStore) -> Self {
        Self {
            store,
            visited_types: HashSet::new(),
            visited_rows: HashSet::new(),
            visited_domains: HashSet::new(),
            visited_lambdas: HashSet::new(),
            visited_scoped_types: HashSet::new(),
            emitted: HashSet::new(),
            lacks: Vec::new(),
        }
    }

    fn collect(mut self, ty: TypeId) -> Vec<StableRecordRowLack> {
        self.visit_type(ty);
        self.lacks
    }

    fn visit_type(&mut self, ty: TypeId) {
        if !self.visited_types.insert(ty) {
            return;
        }
        match self.store.get(ty).clone() {
            TypeData::Record(row) => self.visit_row(row),
            TypeData::Applied { origin, arguments } => {
                self.visit_type(origin);
                for argument in arguments.iter().copied() {
                    self.visit_static_argument(argument);
                }
            }
            TypeData::StaticDomainForm { domain } => self.visit_static_domain(domain),
            TypeData::StaticData(term) => {
                self.visit_static_domain(term.domain);
                for argument in term.arguments.iter().copied() {
                    self.visit_static_argument(argument);
                }
            }
            TypeData::ExactCase { enum_type, .. } => self.visit_type(enum_type),
            TypeData::Union(members) => {
                for member in members.iter().copied() {
                    self.visit_type(member);
                }
            }
            TypeData::Tuple(elements) => {
                for element in elements.elements.iter() {
                    self.visit_type(element.ty);
                }
            }
            TypeData::Pack(pack) => {
                for element in pack.elements.iter() {
                    self.visit_type(element.ty);
                }
            }
            TypeData::Callable(callable) => {
                for parameter in callable.parameters.iter() {
                    self.visit_type(parameter.ty);
                }
                self.visit_type(callable.return_type);
            }
            TypeData::Family(family_id) => {
                for member in self.store.get_family(family_id).members.iter() {
                    self.visit_type(member.ty);
                }
            }
            TypeData::AssociatedProjection(projection) => {
                self.visit_type(projection.subject);
                for argument in projection.trait_ref.arguments.iter().copied() {
                    self.visit_type(argument);
                }
            }
            TypeData::TypeMemberProjection(projection) => {
                use crate::types::type_member::{ProjectionSubject, TypeMemberArgument};
                match projection.subject {
                    ProjectionSubject::Type(subject) => self.visit_type(subject),
                    ProjectionSubject::RecordRow(row) => self.visit_row(row),
                }
                for argument in projection.arguments.iter() {
                    match argument {
                        TypeMemberArgument::Type(ty) => self.visit_type(*ty),
                        TypeMemberArgument::RecordRow(row) => self.visit_row(*row),
                        TypeMemberArgument::Const(_) => {}
                    }
                }
            }
            TypeData::TypePack(elements) | TypeData::TypePackConcat(elements) | TypeData::LabeledTypePackConcat(elements) => {
                for element in elements.iter().copied() {
                    self.visit_type(element);
                }
            }
            TypeData::LabeledTypePack(elements) => {
                for element in elements.iter() {
                    self.visit_type(element.element);
                }
            }
            TypeData::TupleFromPacks { positionals, labeled }
            | TypeData::PackFromPacks { positionals, labeled }
            | TypeData::CompleteTypePack { positionals, labeled } => {
                self.visit_type(positionals);
                self.visit_type(labeled);
            }
            TypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                self.visit_type(positionals);
                self.visit_type(labeled);
                self.visit_type(return_type);
            }
            TypeData::Lambda(lambda) => self.visit_lambda(lambda),
            TypeData::Never
            | TypeData::Unit
            | TypeData::ClassObject { .. }
            | TypeData::Nominal { .. }
            | TypeData::Parameter(_)
            | TypeData::SelfType(_)
            | TypeData::ConstInt(_) => {}
        }
    }

    fn visit_static_argument(&mut self, argument: crate::types::store::StaticArgument) {
        match argument {
            crate::types::store::StaticArgument::Term(ty) => self.visit_type(ty),
            crate::types::store::StaticArgument::RecordRow(row) => self.visit_row(row),
            crate::types::store::StaticArgument::DynamicType => {}
        }
    }

    fn visit_static_domain(&mut self, domain: crate::types::id::StaticDomainId) {
        use crate::types::store::StaticDomainData;
        if !self.visited_domains.insert(domain) {
            return;
        }
        let Some(StaticDomainData::Associated { owner_arguments, .. }) = self.store.static_domain(domain).cloned() else {
            return;
        };
        for argument in owner_arguments.iter().copied() {
            self.visit_static_argument(argument);
        }
    }

    fn visit_row(&mut self, row_id: crate::types::id::RecordRowId) {
        use crate::types::row::RecordRowTail;
        if !self.visited_rows.insert(row_id) {
            return;
        }
        let row = self.store.record_row(row_id).clone();
        if let RecordRowTail::Parameter(parameter) = row.tail {
            for field in row.fields.iter() {
                let key = (parameter, field.name.clone());
                if self.emitted.insert(key) {
                    self.lacks.push(StableRecordRowLack {
                        parameter,
                        field: field.name.clone(),
                    });
                }
            }
        }
        for field in row.fields.iter() {
            self.visit_type(field.ty);
        }
        if let RecordRowTail::Projection(projection) = row.tail {
            self.visit_type(projection);
        }
    }

    fn visit_lambda(&mut self, lambda: crate::types::id::TypeLambdaId) {
        if !self.visited_lambdas.insert(lambda) {
            return;
        }
        let body = self.store.arena().get_lambda(lambda).body;
        self.visit_scoped_type(body);
    }

    fn visit_scoped_type(&mut self, scoped: crate::types::id::ScopedTypeId) {
        use crate::types::type_lambda::{ScopedPackRemainder, ScopedRecordRowTail, ScopedRecordTail, ScopedTypeData};
        if !self.visited_scoped_types.insert(scoped) {
            return;
        }
        match self.store.arena().get_scoped(scoped).clone() {
            ScopedTypeData::Bound { .. } => {}
            ScopedTypeData::Free(ty) => self.visit_type(ty),
            ScopedTypeData::Applied { origin, arguments } => {
                self.visit_scoped_type(origin);
                for argument in arguments.iter().copied() {
                    self.visit_scoped_type(argument);
                }
            }
            ScopedTypeData::Union(members) | ScopedTypeData::TypePack(members) | ScopedTypeData::LabeledTypePackConcat(members) => {
                for member in members.iter().copied() {
                    self.visit_scoped_type(member);
                }
            }
            ScopedTypeData::Tuple(elements) => {
                for element in elements.iter() {
                    self.visit_scoped_type(element.ty);
                }
            }
            ScopedTypeData::Record(fields) => {
                for field in fields.iter() {
                    self.visit_scoped_type(field.ty);
                }
            }
            ScopedTypeData::OpenRecord(record) => {
                if let ScopedRecordTail::FreeParameter(parameter) = record.tail {
                    for field in record.fields.iter() {
                        let key = (parameter, field.name.clone());
                        if self.emitted.insert(key) {
                            self.lacks.push(StableRecordRowLack {
                                parameter,
                                field: field.name.clone(),
                            });
                        }
                    }
                }
                for field in record.fields.iter() {
                    self.visit_scoped_type(field.ty);
                }
            }
            ScopedTypeData::RecordRow(row) => {
                if let ScopedRecordRowTail::FreeParameter(parameter) = row.tail {
                    for field in row.fields.iter() {
                        let key = (parameter, field.name.clone());
                        if self.emitted.insert(key) {
                            self.lacks.push(StableRecordRowLack {
                                parameter,
                                field: field.name.clone(),
                            });
                        }
                    }
                }
                if let ScopedRecordRowTail::Projection(projection) = row.tail {
                    self.visit_type(projection);
                }
                for field in row.fields.iter() {
                    self.visit_scoped_type(field.ty);
                }
            }
            ScopedTypeData::Callable(callable) => {
                for parameter in callable.parameters.iter() {
                    self.visit_scoped_type(parameter.ty);
                }
                self.visit_scoped_type(callable.return_type);
            }
            ScopedTypeData::Pack(pack) => {
                for element in pack.elements.iter() {
                    self.visit_scoped_type(element.ty);
                }
                if let ScopedPackRemainder::Typed(ty) = pack.positional_remainder {
                    self.visit_scoped_type(ty);
                }
                if let ScopedPackRemainder::Typed(ty) = pack.labeled_remainder {
                    self.visit_scoped_type(ty);
                }
            }
            ScopedTypeData::LabeledTypePack(elements) => {
                for element in elements.iter() {
                    self.visit_scoped_type(element.element);
                }
            }
            ScopedTypeData::CompleteTypePack { positionals, labeled }
            | ScopedTypeData::TupleFromPacks { positionals, labeled }
            | ScopedTypeData::PackFromPacks { positionals, labeled } => {
                self.visit_scoped_type(positionals);
                self.visit_scoped_type(labeled);
            }
            ScopedTypeData::TypePackConcat(segments) => {
                for segment in segments.iter().copied() {
                    self.visit_scoped_type(segment);
                }
            }
            ScopedTypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => {
                self.visit_scoped_type(positionals);
                self.visit_scoped_type(labeled);
                self.visit_scoped_type(return_type);
            }
            ScopedTypeData::PackProjection { subject, .. } => self.visit_scoped_type(subject),
            ScopedTypeData::Lambda(lambda) => self.visit_lambda(lambda),
        }
    }
}

/// Coordinates ordinary type inference and query-local row inference without
/// allowing either solver to represent values from the other domain.
pub struct GenericApplicationSession {
    pub types: super::inference::InferenceSession,
    pub rows: RecordRowSolver,
    pub parameter_bindings: std::collections::HashMap<TypeParameterId, GenericInferenceBinding>,
    row_equations: Vec<(RecordRowTerm, RecordRowTerm)>,
    row_proofs: std::collections::HashMap<RecordRowVarId, super::inference::InferenceProofState>,
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
                    presence: field.presence,
                })
                .map_err(|_| CombinedInferenceFailure::UnderconstrainedType(parameter))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let tail = match term.tail {
        RecordRowTermTail::Closed => RecordRowTail::Closed,
        RecordRowTermTail::Parameter(parameter) => RecordRowTail::Parameter(parameter),
        RecordRowTermTail::Projection(projection) => RecordRowTail::Projection(projection),
        RecordRowTermTail::Anonymous(remainder) => RecordRowTail::Anonymous(remainder),
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
    store.record_row_checked(fields, tail).map_err(|error| CombinedInferenceFailure::RowZonk {
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
        InferenceTerm::TypeMemberProjection { subject, arguments, .. } => {
            use super::inference::{InferenceProjectionSubject, InferenceTypeMemberArgument};
            let subject_has_rows = match subject {
                InferenceProjectionSubject::Type(term) => term_has_row_variables(term),
                InferenceProjectionSubject::RecordRow(row) => record_has_row_variables(row),
            };
            subject_has_rows
                || arguments.iter().any(|argument| match argument {
                    InferenceTypeMemberArgument::Type(term) => term_has_row_variables(term),
                    InferenceTypeMemberArgument::Const(_) => false,
                    InferenceTypeMemberArgument::RecordRow(row) => record_has_row_variables(row),
                })
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

fn row_variables_in_term(term: &InferenceTerm, variables: &mut HashSet<RecordRowVarId>) {
    match term {
        InferenceTerm::Record(record) => record_row_variables(record, variables),
        InferenceTerm::Applied { origin, arguments } => {
            row_variables_in_term(origin, variables);
            for argument in arguments.iter() {
                static_argument_row_variables(argument, variables);
            }
        }
        InferenceTerm::StaticData { arguments, .. } => {
            for argument in arguments.iter() {
                static_argument_row_variables(argument, variables);
            }
        }
        InferenceTerm::ExactCase { enum_type, .. } => row_variables_in_term(enum_type, variables),
        InferenceTerm::Union(members) => members.iter().for_each(|member| row_variables_in_term(member, variables)),
        InferenceTerm::Tuple(elements) => elements.iter().for_each(|element| row_variables_in_term(&element.term, variables)),
        InferenceTerm::Callable(callable) => {
            callable
                .parameters
                .iter()
                .for_each(|parameter| row_variables_in_term(&parameter.term, variables));
            row_variables_in_term(&callable.return_type, variables);
        }
        InferenceTerm::Family(members) => members.iter().for_each(|member| row_variables_in_term(&member.term, variables)),
        InferenceTerm::AssociatedProjection { subject, trait_ref, .. } => {
            row_variables_in_term(subject, variables);
            trait_ref.arguments.iter().for_each(|argument| row_variables_in_term(argument, variables));
        }
        InferenceTerm::TypeMemberProjection { subject, arguments, .. } => {
            use super::inference::{InferenceProjectionSubject, InferenceTypeMemberArgument};
            match subject {
                InferenceProjectionSubject::Type(term) => row_variables_in_term(term, variables),
                InferenceProjectionSubject::RecordRow(row) => record_row_variables(row, variables),
            }
            for argument in arguments.iter() {
                match argument {
                    InferenceTypeMemberArgument::Type(term) => row_variables_in_term(term, variables),
                    InferenceTypeMemberArgument::Const(_) => {}
                    InferenceTypeMemberArgument::RecordRow(row) => record_row_variables(row, variables),
                }
            }
        }
        InferenceTerm::TypePack(elements) => elements.iter().for_each(|element| row_variables_in_term(element, variables)),
        InferenceTerm::LabeledTypePack(elements) => elements.iter().for_each(|element| row_variables_in_term(&element.term, variables)),
        InferenceTerm::CompleteTypePack { positionals, labeled } | InferenceTerm::TupleFromPacks { positionals, labeled } => {
            row_variables_in_term(positionals, variables);
            row_variables_in_term(labeled, variables);
        }
        InferenceTerm::TypePackConcat(segments) | InferenceTerm::LabeledTypePackConcat(segments) => {
            segments.iter().for_each(|segment| row_variables_in_term(segment, variables));
        }
        InferenceTerm::CallableFromPacks {
            positionals,
            labeled,
            return_type,
        } => {
            row_variables_in_term(positionals, variables);
            row_variables_in_term(labeled, variables);
            row_variables_in_term(return_type, variables);
        }
        InferenceTerm::PackProjection { subject, .. } => row_variables_in_term(subject, variables),
        InferenceTerm::Canonical(_) | InferenceTerm::Var(_) | InferenceTerm::Rigid(_) => {}
    }
}

fn static_argument_row_variables(argument: &super::inference::InferenceStaticArgument, variables: &mut HashSet<RecordRowVarId>) {
    match argument {
        super::inference::InferenceStaticArgument::Term(term) => row_variables_in_term(term, variables),
        super::inference::InferenceStaticArgument::RecordRow(row) => record_row_variables(row, variables),
        super::inference::InferenceStaticArgument::DynamicType => {}
    }
}

fn record_row_variables(row: &InferenceRecord, variables: &mut HashSet<RecordRowVarId>) {
    if let InferenceRecordTail::Var(variable) = row.tail {
        variables.insert(variable);
    }
    row.fields.iter().for_each(|field| row_variables_in_term(&field.term, variables));
}

fn static_argument_has_row_variables(argument: &super::inference::InferenceStaticArgument) -> bool {
    match argument {
        super::inference::InferenceStaticArgument::Term(term) => term_has_row_variables(term),
        super::inference::InferenceStaticArgument::RecordRow(row) => record_has_row_variables(row),
        super::inference::InferenceStaticArgument::DynamicType => false,
    }
}

fn record_has_row_variables(row: &InferenceRecord) -> bool {
    matches!(row.tail, InferenceRecordTail::Var(_)) || row.fields.iter().any(|field| term_has_row_variables(&field.term))
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
            row_proofs: std::collections::HashMap::new(),
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
            row_proofs: std::collections::HashMap::new(),
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
        terms.extend(self.parameter_bindings.iter().filter_map(|(&parameter, binding)| match binding {
            GenericInferenceBinding::RecordRow(variable) => Some((parameter, *variable)),
            GenericInferenceBinding::Type(_) | GenericInferenceBinding::FixedType(_) | GenericInferenceBinding::UntrackedType => None,
        }));
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
                    Err(CombinedInferenceFailure::RowRejected(lift_canonical_row_failure(
                        RecordRowFailure::MissingField { field: required.name.clone() },
                    )))
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
            RecordRowTail::Anonymous(rem) => RecordRowTermTail::Anonymous(rem),
        };
        match formal.tail {
            InferenceRecordTail::Closed => {}
            InferenceRecordTail::Parameter(parameter) => {
                if !extra.is_empty() || actual_tail != RecordRowTermTail::Parameter(parameter) {
                    return Err(CombinedInferenceFailure::RowRejected(lift_canonical_row_failure(
                        RecordRowFailure::RigidTailMismatch {
                            expected: RecordRowTermTail::Parameter(parameter),
                            actual: actual_tail,
                        },
                    )));
                }
            }
            InferenceRecordTail::Projection(proj) => {
                if !extra.is_empty() || actual_tail != RecordRowTermTail::Projection(proj) {
                    return Err(CombinedInferenceFailure::RowRejected(lift_canonical_row_failure(
                        RecordRowFailure::RigidTailMismatch {
                            expected: RecordRowTermTail::Projection(proj),
                            actual: actual_tail,
                        },
                    )));
                }
            }
            InferenceRecordTail::Anonymous(rem) => {
                if !extra.is_empty() || actual_tail != RecordRowTermTail::Anonymous(rem) {
                    return Err(CombinedInferenceFailure::RowRejected(lift_canonical_row_failure(
                        RecordRowFailure::RigidTailMismatch {
                            expected: RecordRowTermTail::Anonymous(rem),
                            actual: actual_tail,
                        },
                    )));
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
                            RecordRowTermTail::Anonymous(rem) => InferenceRecordTail::Anonymous(rem),
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

    /// Binds a declaration-owned RecordFields parameter from residual fields
    /// collected by a shape-aware data-constructor adapter. Fixed source fields
    /// are represented as row-lacks constraints, so an expansion cannot
    /// silently overwrite them.
    pub fn constrain_record_row_expansion(
        &mut self,
        parameter: TypeParameterId,
        fields: Vec<InferenceRecordField>,
        fixed_fields: Vec<Box<str>>,
    ) -> Result<(), CombinedInferenceFailure<InferenceTerm>> {
        let Some(GenericInferenceBinding::RecordRow(variable)) = self.parameter_bindings.get(&parameter).copied() else {
            return Err(CombinedInferenceFailure::UnderconstrainedRow(parameter));
        };
        let Some(session) = &self.query_session else {
            return Err(CombinedInferenceFailure::Blocked(BlockReason::RecursiveFixpoint));
        };

        for field in fixed_fields {
            session
                .borrow_mut()
                .add_record_row_lacks(variable, field)
                .map_err(CombinedInferenceFailure::RowRejected)?;
        }

        let left = InferenceRecord {
            fields: Box::new([]),
            tail: InferenceRecordTail::Var(variable),
        };
        let right = InferenceRecord {
            fields: fields.into_boxed_slice(),
            tail: InferenceRecordTail::Closed,
        };
        session.borrow_mut().enqueue_record_row_equation(&left, &right);
        Ok(())
    }

    /// Records the value evidence that established one contribution to an
    /// inferred row. This state belongs to the application query, never to the
    /// canonical RecordRow identity.
    pub fn record_record_row_expansion_premise(&mut self, parameter: TypeParameterId, knowledge: &crate::types::evidence::TypeKnowledge) {
        let Some(GenericInferenceBinding::RecordRow(variable)) = self.parameter_bindings.get(&parameter).copied() else {
            return;
        };
        let proof = super::inference::InferenceProofState::from_knowledge(knowledge);
        self.row_proofs
            .entry(variable)
            .and_modify(|current| *current = current.clone().meet(proof.clone()))
            .or_insert(proof);
    }

    /// Preserves independent value evidence when a known source record
    /// determines the residual portion of a formal row.
    pub fn record_known_record_argument_premise(&mut self, formal: &InferenceRecord, knowledge: &crate::types::evidence::TypeKnowledge) {
        let InferenceRecordTail::Var(variable) = formal.tail else {
            return;
        };
        let proof = super::inference::InferenceProofState::from_knowledge(knowledge);
        self.row_proofs
            .entry(variable)
            .and_modify(|current| *current = current.clone().meet(proof.clone()))
            .or_insert(proof);
    }

    /// Context may select an otherwise unconstrained row in the published
    /// result, but that selection is only assumed evidence.
    pub fn record_row_context_selection(&mut self, term: &InferenceTerm) {
        let mut variables = HashSet::new();
        row_variables_in_term(term, &mut variables);
        for variable in variables {
            self.row_proofs.entry(variable).or_insert(super::inference::InferenceProofState::Assumed);
        }
    }

    /// Marks only return-relevant row variables that were unsolved before an
    /// expected-result relation and became solved by that relation. Existing
    /// value proof is retained, and rows already solved from constructor shape
    /// are not weakened by a compatible context.
    pub fn record_expected_result_row_selection(
        &mut self,
        term: &InferenceTerm,
        previously_unsolved: &[RecordRowVarId],
        solution: &RecordRowSolution<InferenceTerm>,
    ) {
        let mut returned = HashSet::new();
        row_variables_in_term(term, &mut returned);
        for variable in previously_unsolved {
            if returned.contains(variable) && solution.term_for(*variable).is_some() {
                self.row_proofs.entry(*variable).or_insert(super::inference::InferenceProofState::Assumed);
            }
        }
    }

    /// Meets ordinary type-variable proofs with only the row proofs reachable
    /// from this result. Solved closed empty rows have no value premise and
    /// are exact; other unproven rows remain underconstrained.
    pub fn proof_state_for_term(
        &self,
        type_session: &super::inference::InferenceSession,
        term: &InferenceTerm,
        solution: Option<&RecordRowSolution<InferenceTerm>>,
    ) -> super::inference::InferenceProofState {
        fn visit_row(
            application: &GenericApplicationSession,
            type_session: &super::inference::InferenceSession,
            variable: RecordRowVarId,
            solution: Option<&RecordRowSolution<InferenceTerm>>,
            visiting: &mut HashSet<RecordRowVarId>,
        ) -> super::inference::InferenceProofState {
            use super::inference::InferenceProofState;
            if !visiting.insert(variable) {
                return InferenceProofState::Unknown(crate::types::evidence::UnknownReason::InferenceBlocked);
            }
            let Some(solution) = solution else {
                visiting.remove(&variable);
                return application
                    .row_proofs
                    .get(&variable)
                    .cloned()
                    .unwrap_or(InferenceProofState::Unknown(crate::types::evidence::UnknownReason::InferenceBlocked));
            };
            let Some(row) = solution.term_for(variable) else {
                visiting.remove(&variable);
                return application
                    .row_proofs
                    .get(&variable)
                    .cloned()
                    .unwrap_or(InferenceProofState::Unknown(crate::types::evidence::UnknownReason::InferenceBlocked));
            };
            let mut proof = application.row_proofs.get(&variable).cloned().unwrap_or_else(|| {
                if row.fields.is_empty() && row.tail == RecordRowTermTail::Closed {
                    InferenceProofState::Established
                } else {
                    InferenceProofState::Unknown(crate::types::evidence::UnknownReason::NoTypeEvidence)
                }
            });
            let mut nested = HashSet::new();
            for field in row.fields.iter() {
                proof = proof.meet(type_session.proof_state_for_term(&field.ty));
                row_variables_in_term(&field.ty, &mut nested);
            }
            if let RecordRowTermTail::Var(tail) = row.tail {
                nested.insert(tail);
            }
            for dependency in nested {
                proof = proof.meet(visit_row(application, type_session, dependency, Some(solution), visiting));
            }
            visiting.remove(&variable);
            proof
        }

        let mut variables = HashSet::new();
        row_variables_in_term(term, &mut variables);
        let mut proof = type_session.proof_state_for_term(term);
        let mut visiting = HashSet::new();
        for variable in variables {
            proof = proof.meet(visit_row(self, type_session, variable, solution, &mut visiting));
        }
        proof
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
        let Some(frame) = self.query_frame else {
            return RecordRowSolveResult::InternalFailure(crate::types::row_solver::IncidentId(0));
        };
        session.borrow().row_solution_for_roots(frame, &roots, budget, cancellation)
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

#[cfg(test)]
mod stable_row_lacks_tests {
    use super::{GenericApplicationSession, StableRecordRowLack, collect_stable_record_row_lacks};
    use crate::identity::{AssociatedStaticDomainDeclarationId, DeclarationId, ModuleId};
    use crate::types::id::{KindId, TypeParameterId};
    use crate::types::outcome::{CancellationToken, QueryBudget};
    use crate::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner};
    use crate::types::row::{RecordRowField, RecordRowTail};
    use crate::types::row_solver::{RecordRowFailure, RecordRowSolveResult};
    use crate::types::store::{RecordTypeField, StaticArgument, TypeData, TypeStore};

    fn owner(name: &str) -> DeclarationId {
        DeclarationId::new(ModuleId::universe_root(), name.into())
    }

    fn row_parameter(store: &mut TypeStore) -> (TypeParameterOwner, TypeParameterId) {
        let parameter_owner = TypeParameterOwner::Declaration(owner("RowOwner"));
        let parameter = store.intern_type_parameter(TypeParameterData::new(parameter_owner.clone(), 0, "R", KindId::RECORD_ROW));
        (parameter_owner, parameter)
    }

    fn prefixed_open_row(store: &mut TypeStore, parameter: TypeParameterId) -> crate::types::id::RecordRowId {
        let int = store.nominal_type(owner("Int"));
        store
            .record_row_checked(vec![RecordRowField::required("known".into(), int)], RecordRowTail::Parameter(parameter))
            .expect("valid prefixed row")
    }

    fn assert_nested_lack_rejects_duplicate_extension(store: &mut TypeStore, parameter: TypeParameterId, nested: crate::types::id::TypeId) {
        assert_eq!(
            collect_stable_record_row_lacks(store, nested),
            vec![StableRecordRowLack {
                parameter,
                field: "known".into(),
            }]
        );

        let parameter_owner = TypeParameterOwner::Declaration(owner("RowOwner"));
        let signature = GenericSignature::new(parameter_owner, Box::new([parameter]));
        let mut session = GenericApplicationSession::new(&signature, store);
        session.constrain_signature_type_lacks(nested, store).expect("nested stable lack should bind");

        let formal = store
            .record_row_type_checked(Vec::new(), RecordRowTail::Parameter(parameter))
            .expect("formal open row");
        let int = store.nominal_type(owner("Int"));
        let actual = store
            .record_row_type_checked(vec![RecordTypeField::required("known".into(), int)], RecordRowTail::Closed)
            .expect("actual row with duplicate field");
        let crate::checker::InferenceTerm::Record(formal_record) = session.type_term(formal, store) else {
            panic!("expected formal row inference term")
        };
        session
            .constrain_known_record_argument(actual, &formal_record, store)
            .expect("known record constraint should be enqueued");
        let mut budget = QueryBudget::default();
        let cancellation = CancellationToken::new();
        assert!(matches!(
            session.solve_rows(store, &mut budget, &cancellation),
            RecordRowSolveResult::Rejected(RecordRowFailure::LacksViolation { field, .. }) if field.as_ref() == "known"
        ));
    }

    #[test]
    fn applied_record_row_argument_preserves_nested_prefix_lacks() {
        let mut store = TypeStore::new();
        let (_owner, parameter) = row_parameter(&mut store);
        let row = prefixed_open_row(&mut store, parameter);
        let origin = store.nominal_type(owner("Applied"));
        let applied = store.intern_with_kind(
            TypeData::Applied {
                origin,
                arguments: vec![StaticArgument::RecordRow(row)].into_boxed_slice(),
            },
            KindId::TYPE,
        );
        assert_nested_lack_rejects_duplicate_extension(&mut store, parameter, applied);
    }

    #[test]
    fn static_data_record_row_argument_preserves_nested_prefix_lacks() {
        let mut store = TypeStore::new();
        let (_owner, parameter) = row_parameter(&mut store);
        let row = prefixed_open_row(&mut store, parameter);
        let declaration = owner("Shape");
        let domain = store.nominal_static_domain(declaration.clone());
        let constructor = store.intern_static_constructor(domain, declaration, "Row", vec![KindId::RECORD_ROW].into_boxed_slice());
        let static_data = store
            .static_data(domain, constructor, vec![StaticArgument::RecordRow(row)].into_boxed_slice())
            .expect("valid row static argument");
        assert_nested_lack_rejects_duplicate_extension(&mut store, parameter, static_data);
    }

    #[test]
    fn associated_static_domain_owner_row_argument_preserves_nested_prefix_lacks() {
        let mut store = TypeStore::new();
        let (_owner, parameter) = row_parameter(&mut store);
        let row = prefixed_open_row(&mut store, parameter);
        let owner = owner("Container");
        let declaration = AssociatedStaticDomainDeclarationId::new(owner.clone(), "State");
        let domain = store.associated_static_domain(declaration, vec![StaticArgument::RecordRow(row)].into_boxed_slice());
        let constructor = store.intern_static_constructor(domain, owner, "Ready", Box::new([]));
        let static_data = store.static_data(domain, constructor, Box::new([])).expect("valid associated static case");
        assert_nested_lack_rejects_duplicate_extension(&mut store, parameter, static_data);
    }
}

#[cfg(test)]
mod row_proof_matrix_tests {
    use super::{GenericApplicationSession, GenericInferenceBinding, InferenceRecord, InferenceRecordTail};
    use crate::checker::inference::InferenceProofState;
    use crate::identity::{DeclarationId, ModuleId};
    use crate::types::evidence::{DynamicReason, EvidenceOrigin, TypeKnowledge, UnknownReason};
    use crate::types::id::{KindId, TypeParameterId};
    use crate::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner};
    use crate::types::row::RecordRowField;
    use crate::types::row_solver::{RecordRowSolution, RecordRowTerm, RecordRowTermTail, RecordRowVarId};
    use crate::types::store::TypeStore;
    use std::collections::HashMap;

    fn owner() -> DeclarationId {
        DeclarationId::new(ModuleId::universe_root(), "RowProofMatrix".into())
    }

    fn new_application(count: usize) -> (TypeStore, GenericApplicationSession, Vec<TypeParameterId>, Vec<RecordRowVarId>) {
        let mut store = TypeStore::new();
        let owner = TypeParameterOwner::Declaration(owner());
        let parameters = (0..count)
            .map(|index| store.intern_type_parameter(TypeParameterData::new(owner.clone(), index as u32, format!("R{index}"), KindId::RECORD_ROW)))
            .collect::<Vec<_>>();
        let signature = GenericSignature::new(owner, parameters.clone().into_boxed_slice());
        let application = GenericApplicationSession::new(&signature, &store);
        let variables = parameters
            .iter()
            .map(|parameter| match application.parameter_bindings[parameter] {
                GenericInferenceBinding::RecordRow(variable) => variable,
                _ => unreachable!(),
            })
            .collect();
        (store, application, parameters, variables)
    }

    fn result_row(variable: RecordRowVarId) -> crate::checker::InferenceTerm {
        crate::checker::InferenceTerm::Record(InferenceRecord {
            fields: Box::new([]),
            tail: InferenceRecordTail::Var(variable),
        })
    }

    fn closed_solution(
        variable: RecordRowVarId,
        fields: Vec<RecordRowField<crate::checker::InferenceTerm>>,
    ) -> RecordRowSolution<crate::checker::InferenceTerm> {
        RecordRowSolution {
            substitutions: HashMap::from([(
                variable,
                RecordRowTerm {
                    fields: fields.into_boxed_slice(),
                    tail: RecordRowTermTail::Closed,
                },
            )]),
        }
    }

    #[test]
    fn source_premise_row_proof_matrix_preserves_strength_and_meets_contributions() {
        let (mut store, mut application, parameters, variables) = new_application(1);
        let int = store.nominal_type(owner());
        let field = || RecordRowField::required("value".into(), crate::checker::InferenceTerm::Canonical(int));
        let type_session = crate::checker::InferenceSession::new();

        application.record_record_row_expansion_premise(parameters[0], &TypeKnowledge::established(int, EvidenceOrigin::Syntax));
        assert_eq!(
            application.proof_state_for_term(&type_session, &result_row(variables[0]), Some(&closed_solution(variables[0], vec![field()]))),
            InferenceProofState::Established,
            "one established contribution remains established"
        );

        let (mut store, mut application, parameters, variables) = new_application(1);
        let int = store.nominal_type(owner());
        let field = || RecordRowField::required("value".into(), crate::checker::InferenceTerm::Canonical(int));
        let type_session = crate::checker::InferenceSession::new();
        application.record_record_row_expansion_premise(parameters[0], &TypeKnowledge::established(int, EvidenceOrigin::Syntax));
        application.record_record_row_expansion_premise(parameters[0], &TypeKnowledge::assumed(int, EvidenceOrigin::CallableSignature));
        assert_eq!(
            application.proof_state_for_term(&type_session, &result_row(variables[0]), Some(&closed_solution(variables[0], vec![field()]))),
            InferenceProofState::Assumed,
            "mixed contributions meet to the weaker known evidence"
        );

        let (mut store, mut application, parameters, variables) = new_application(1);
        let int = store.nominal_type(owner());
        let type_session = crate::checker::InferenceSession::new();
        application.record_record_row_expansion_premise(parameters[0], &TypeKnowledge::established(int, EvidenceOrigin::Syntax));
        application.record_record_row_expansion_premise(parameters[0], &TypeKnowledge::established(int, EvidenceOrigin::Syntax));
        assert_eq!(
            application.proof_state_for_term(
                &type_session,
                &result_row(variables[0]),
                Some(&closed_solution(
                    variables[0],
                    vec![
                        RecordRowField::required("left".into(), crate::checker::InferenceTerm::Canonical(int)),
                        RecordRowField::required("right".into(), crate::checker::InferenceTerm::Canonical(int)),
                    ],
                )),
            ),
            InferenceProofState::Established,
            "multiple established contributions remain established"
        );
    }

    #[test]
    fn unknown_and_dynamic_row_premises_remain_the_weakest_state() {
        let (mut store, mut application, parameters, variables) = new_application(1);
        let int = store.nominal_type(owner());
        let type_session = crate::checker::InferenceSession::new();
        application.record_record_row_expansion_premise(parameters[0], &TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence));
        assert_eq!(
            application.proof_state_for_term(
                &type_session,
                &result_row(variables[0]),
                Some(&closed_solution(
                    variables[0],
                    vec![RecordRowField::required("value".into(), crate::checker::InferenceTerm::Canonical(int))],
                )),
            ),
            InferenceProofState::Unknown(UnknownReason::NoTypeEvidence)
        );

        let (mut store, mut application, parameters, variables) = new_application(1);
        let int = store.nominal_type(owner());
        let type_session = crate::checker::InferenceSession::new();
        application.record_record_row_expansion_premise(parameters[0], &TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape));
        assert_eq!(
            application.proof_state_for_term(
                &type_session,
                &result_row(variables[0]),
                Some(&closed_solution(
                    variables[0],
                    vec![RecordRowField::required("value".into(), crate::checker::InferenceTerm::Canonical(int))],
                )),
            ),
            InferenceProofState::Dynamic(DynamicReason::ExplicitEscape)
        );
    }

    #[test]
    fn solved_empty_row_is_exact_but_unproven_restricted_row_is_not() {
        let (_store, application, _parameters, variables) = new_application(1);
        let type_session = crate::checker::InferenceSession::new();
        let empty = closed_solution(variables[0], Vec::new());
        assert_eq!(
            application.proof_state_for_term(&type_session, &result_row(variables[0]), Some(&empty)),
            InferenceProofState::Established
        );

        let (mut store, mut application, parameters, variables) = new_application(1);
        let int = store.nominal_type(owner());
        application.record_record_row_expansion_premise(parameters[0], &TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence));
        application
            .rows
            .add_lacks(variables[0], "fixed".into())
            .expect("lacks constraint should be accepted");
        let constrained = closed_solution(
            variables[0],
            vec![RecordRowField::required("other".into(), crate::checker::InferenceTerm::Canonical(int))],
        );
        assert_eq!(
            application.proof_state_for_term(&type_session, &result_row(variables[0]), Some(&constrained)),
            InferenceProofState::Unknown(UnknownReason::NoTypeEvidence),
            "shape restrictions do not establish value evidence"
        );
    }

    #[test]
    fn nested_dependencies_propagate_but_irrelevant_weak_rows_do_not() {
        let (mut store, mut application, parameters, variables) = new_application(3);
        let int = store.nominal_type(owner());
        application.record_record_row_expansion_premise(parameters[0], &TypeKnowledge::established(int, EvidenceOrigin::Syntax));
        application.record_record_row_expansion_premise(parameters[1], &TypeKnowledge::assumed(int, EvidenceOrigin::CallableSignature));
        application.record_record_row_expansion_premise(parameters[2], &TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape));
        let type_session = crate::checker::InferenceSession::new();
        let nested = RecordRowSolution {
            substitutions: HashMap::from([
                (
                    variables[0],
                    RecordRowTerm {
                        fields: vec![RecordRowField::required(
                            "inner".into(),
                            crate::checker::InferenceTerm::Record(InferenceRecord {
                                fields: Box::new([]),
                                tail: InferenceRecordTail::Var(variables[1]),
                            }),
                        )]
                        .into_boxed_slice(),
                        tail: RecordRowTermTail::Closed,
                    },
                ),
                (
                    variables[1],
                    RecordRowTerm {
                        fields: vec![RecordRowField::required("value".into(), crate::checker::InferenceTerm::Canonical(int))].into_boxed_slice(),
                        tail: RecordRowTermTail::Closed,
                    },
                ),
                (
                    variables[2],
                    RecordRowTerm {
                        fields: vec![RecordRowField::required("unrelated".into(), crate::checker::InferenceTerm::Canonical(int))].into_boxed_slice(),
                        tail: RecordRowTermTail::Closed,
                    },
                ),
            ]),
        };
        assert_eq!(
            application.proof_state_for_term(&type_session, &result_row(variables[0]), Some(&nested)),
            InferenceProofState::Assumed,
            "nested row evidence participates in the result meet"
        );
        assert_eq!(
            application.proof_state_for_term(&type_session, &result_row(variables[1]), Some(&nested)),
            InferenceProofState::Assumed,
            "a Dynamic row outside the returned term must not weaken it"
        );
    }
}
