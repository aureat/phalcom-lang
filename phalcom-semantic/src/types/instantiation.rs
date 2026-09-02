//! Domain-aware generic instantiation and checked materialization.

use super::id::{RecordRowId, TypeId, TypeParameterId};
use super::row::{RecordRowField, RecordRowFormationError, RecordRowTail};
use super::store::{CallableParameterType, CallableType, TupleTypeElement, TypeData, TypeStore};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GenericInstantiation {
    type_bindings: HashMap<TypeParameterId, TypeId>,
    row_bindings: HashMap<TypeParameterId, RecordRowId>,
}

impl GenericInstantiation {
    pub fn bind_type(&mut self, parameter: TypeParameterId, ty: TypeId) {
        self.type_bindings.insert(parameter, ty);
    }

    pub fn bind_row(&mut self, parameter: TypeParameterId, row: RecordRowId) {
        self.row_bindings.insert(parameter, row);
    }

    pub fn type_binding(&self, parameter: TypeParameterId) -> Option<TypeId> {
        self.type_bindings.get(&parameter).copied()
    }

    pub fn row_binding(&self, parameter: TypeParameterId) -> Option<RecordRowId> {
        self.row_bindings.get(&parameter).copied()
    }

    pub fn type_bindings(&self) -> &HashMap<TypeParameterId, TypeId> {
        &self.type_bindings
    }

    pub fn row_bindings(&self) -> &HashMap<TypeParameterId, RecordRowId> {
        &self.row_bindings
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RowMaterializationMode {
    PreserveUnboundStableTail,
    RequireSolvedTail,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TypeMaterializationError {
    #[error("record row parameter has no solved row binding")]
    UnresolvedRowParameter(TypeParameterId),
    #[error("recursive record row substitution")]
    RecursiveRowSubstitution(TypeParameterId),
    #[error("record row formation failed: {0}")]
    RecordRow(#[from] RecordRowFormationError),
    #[error("type application failed while materializing a generic type")]
    TypeApplication,
}

pub fn materialize_type(
    store: &mut TypeStore,
    ty: TypeId,
    instantiation: &GenericInstantiation,
    row_mode: RowMaterializationMode,
) -> Result<TypeId, TypeMaterializationError> {
    materialize_type_inner(store, ty, instantiation, row_mode, &mut HashSet::new())
}

fn materialize_type_inner(
    store: &mut TypeStore,
    ty: TypeId,
    instantiation: &GenericInstantiation,
    row_mode: RowMaterializationMode,
    visiting_rows: &mut HashSet<TypeParameterId>,
) -> Result<TypeId, TypeMaterializationError> {
    match store.get(ty).clone() {
        TypeData::Parameter(parameter) => Ok(instantiation.type_binding(parameter).unwrap_or(ty)),
        TypeData::Applied { origin, arguments } => {
            let origin = materialize_type_inner(store, origin, instantiation, row_mode, visiting_rows)?;
            let arguments = arguments
                .iter()
                .map(|&argument| materialize_type_inner(store, argument, instantiation, row_mode, visiting_rows))
                .collect::<Result<Vec<_>, _>>()?;
            store.apply_type_form(origin, &arguments).map_err(|_| TypeMaterializationError::TypeApplication)
        }
        TypeData::ExactCase { variant, enum_type } => {
            let enum_type = materialize_type_inner(store, enum_type, instantiation, row_mode, visiting_rows)?;
            let variant = store.variant_identity(variant).clone();
            store
                .exact_case_type(&variant, enum_type)
                .map_err(|_| TypeMaterializationError::TypeApplication)
        }
        TypeData::Union(members) => {
            let members = members
                .iter()
                .map(|&member| materialize_type_inner(store, member, instantiation, row_mode, visiting_rows))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(store.union(&members))
        }
        TypeData::Tuple(elements) => {
            let elements = elements
                .iter()
                .map(|element| {
                    Ok(TupleTypeElement {
                        label: element.label.clone(),
                        ty: materialize_type_inner(store, element.ty, instantiation, row_mode, visiting_rows)?,
                    })
                })
                .collect::<Result<Vec<_>, TypeMaterializationError>>()?;
            Ok(store.tuple(elements.into_boxed_slice()))
        }
        TypeData::Record(row_id) => {
            let (fields, tail) = materialize_row(store, row_id, instantiation, row_mode, visiting_rows)?;
            Ok(store.record_row_type_checked(fields, tail)?)
        }
        TypeData::Callable(callable) => {
            let parameters = callable
                .parameters
                .iter()
                .map(|parameter| {
                    Ok(CallableParameterType {
                        label: parameter.label.clone(),
                        ty: materialize_type_inner(store, parameter.ty, instantiation, row_mode, visiting_rows)?,
                        rest: parameter.rest,
                    })
                })
                .collect::<Result<Vec<_>, TypeMaterializationError>>()?;
            let return_type = materialize_type_inner(store, callable.return_type, instantiation, row_mode, visiting_rows)?;
            Ok(store.callable(CallableType {
                parameters: parameters.into_boxed_slice(),
                return_type,
            }))
        }
        TypeData::Family(family_id) => {
            let family = store.get_family(family_id).clone();
            let members = family
                .members
                .iter()
                .map(|member| {
                    Ok(super::family::FamilyMemberType {
                        operation: member.operation.clone(),
                        member_kind: member.member_kind,
                        ty: materialize_type_inner(store, member.ty, instantiation, row_mode, visiting_rows)?,
                    })
                })
                .collect::<Result<Vec<_>, TypeMaterializationError>>()?;
            store.family_type(members).map_err(|_| TypeMaterializationError::TypeApplication)
        }
        TypeData::Lambda(_) | TypeData::SelfType(_) | TypeData::Never | TypeData::Unit | TypeData::Nominal { .. } | TypeData::ClassObject { .. } => Ok(ty),
    }
}

fn materialize_row(
    store: &mut TypeStore,
    row_id: RecordRowId,
    instantiation: &GenericInstantiation,
    row_mode: RowMaterializationMode,
    visiting_rows: &mut HashSet<TypeParameterId>,
) -> Result<(Vec<RecordRowField>, RecordRowTail), TypeMaterializationError> {
    let row = store.record_row(row_id).clone();
    let fields = row
        .fields
        .iter()
        .map(|field| {
            Ok(RecordRowField {
                name: field.name.clone(),
                ty: materialize_type_inner(store, field.ty, instantiation, row_mode, visiting_rows)?,
            })
        })
        .collect::<Result<Vec<_>, TypeMaterializationError>>()?;
    materialize_row_tail(store, fields, row.tail, instantiation, row_mode, visiting_rows)
}

fn materialize_row_tail(
    store: &mut TypeStore,
    mut fields: Vec<RecordRowField>,
    tail: RecordRowTail,
    instantiation: &GenericInstantiation,
    row_mode: RowMaterializationMode,
    visiting_rows: &mut HashSet<TypeParameterId>,
) -> Result<(Vec<RecordRowField>, RecordRowTail), TypeMaterializationError> {
    let RecordRowTail::Parameter(parameter) = tail else {
        return Ok((fields, tail));
    };
    let Some(bound_row) = instantiation.row_binding(parameter) else {
        return match row_mode {
            RowMaterializationMode::PreserveUnboundStableTail => Ok((fields, tail)),
            RowMaterializationMode::RequireSolvedTail => Err(TypeMaterializationError::UnresolvedRowParameter(parameter)),
        };
    };
    if !visiting_rows.insert(parameter) {
        return Err(TypeMaterializationError::RecursiveRowSubstitution(parameter));
    }
    let (bound_fields, bound_tail) = materialize_row(store, bound_row, instantiation, row_mode, visiting_rows)?;
    visiting_rows.remove(&parameter);
    fields.extend(bound_fields);
    materialize_row_tail(store, fields, bound_tail, instantiation, row_mode, visiting_rows)
}
