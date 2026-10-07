//! Domain-aware generic instantiation and checked materialization.

use super::id::{RecordRowId, TypeId, TypeParameterId};
use super::row::{RecordRowField, RecordRowFormationError, RecordRowTail};
use super::store::{CallableParameterType, CallableType, PackFormationError, TupleTypeElement, TypeData, TypeStore};
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
    #[error("pack formation failed while materializing a generic type: {0}")]
    Pack(#[from] PackFormationError),
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
                .map(|argument| match argument {
                    super::store::StaticArgument::Term(argument) => Ok(super::store::StaticArgument::Term(materialize_type_inner(
                        store,
                        *argument,
                        instantiation,
                        row_mode,
                        visiting_rows,
                    )?)),
                    super::store::StaticArgument::RecordRow(row) => Ok(super::store::StaticArgument::RecordRow(materialize_row_id(
                        store,
                        *row,
                        instantiation,
                        row_mode,
                        visiting_rows,
                    )?)),
                    super::store::StaticArgument::DynamicType => Ok(super::store::StaticArgument::DynamicType),
                })
                .collect::<Result<Vec<_>, TypeMaterializationError>>()?;
            store
                .apply_static_form(origin, &arguments)
                .map_err(|_| TypeMaterializationError::TypeApplication)
        }
        TypeData::ExactCase { variant, enum_type } => {
            let enum_type = materialize_type_inner(store, enum_type, instantiation, row_mode, visiting_rows)?;
            let variant = store.variant_identity(variant).clone();
            store
                .exact_case_type(&variant, enum_type)
                .map_err(|_| TypeMaterializationError::TypeApplication)
        }
        TypeData::AssociatedProjection(projection) => {
            let subject = materialize_type_inner(store, projection.subject, instantiation, row_mode, visiting_rows)?;
            let arguments = projection
                .trait_ref
                .arguments
                .iter()
                .map(|&argument| materialize_type_inner(store, argument, instantiation, row_mode, visiting_rows))
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice();
            Ok(store.associated_projection(
                subject,
                crate::traits::TraitRef::new(projection.trait_ref.declaration, arguments),
                projection.requirement,
            ))
        }
        TypeData::TypeMemberProjection(projection) => {
            let kind = store.kind_of(ty);
            let subject = match &projection.subject {
                super::type_member::ProjectionSubject::Type(sub_ty) => {
                    super::type_member::ProjectionSubject::Type(materialize_type_inner(store, *sub_ty, instantiation, row_mode, visiting_rows)?)
                }
                super::type_member::ProjectionSubject::RecordRow(row_id) => {
                    super::type_member::ProjectionSubject::RecordRow(materialize_row_id(store, *row_id, instantiation, row_mode, visiting_rows)?)
                }
            };
            let arguments = projection
                .arguments
                .iter()
                .map(|arg| match arg {
                    super::type_member::TypeMemberArgument::Type(arg_ty) => {
                        let mat = materialize_type_inner(store, *arg_ty, instantiation, row_mode, visiting_rows)?;
                        Ok(super::type_member::TypeMemberArgument::Type(mat))
                    }
                    super::type_member::TypeMemberArgument::Const(c) => Ok(super::type_member::TypeMemberArgument::Const(c.clone())),
                    super::type_member::TypeMemberArgument::RecordRow(row_id) => Ok(super::type_member::TypeMemberArgument::RecordRow(materialize_row_id(
                        store,
                        *row_id,
                        instantiation,
                        row_mode,
                        visiting_rows,
                    )?)),
                })
                .collect::<Result<Vec<_>, TypeMaterializationError>>()?
                .into_boxed_slice();
            Ok(store.type_member_projection(
                super::type_member::TypeLevelProjection {
                    subject,
                    selector: projection.selector.clone(),
                    arguments,
                    source_declaration: projection.source_declaration.clone(),
                },
                kind,
            ))
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
                .elements
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
        TypeData::TypePack(elements) => {
            let elements = elements
                .iter()
                .map(|&elem| materialize_type_inner(store, elem, instantiation, row_mode, visiting_rows))
                .collect::<Result<Vec<_>, TypeMaterializationError>>()?;
            store
                .type_pack(elements.into_boxed_slice())
                .map_err(|_| TypeMaterializationError::TypeApplication)
        }
        TypeData::LabeledTypePack(elements) => {
            let elements = elements
                .iter()
                .map(|elem| {
                    Ok(crate::types::store::LabeledTypePackElement {
                        label: elem.label.clone(),
                        element: materialize_type_inner(store, elem.element, instantiation, row_mode, visiting_rows)?,
                    })
                })
                .collect::<Result<Vec<_>, TypeMaterializationError>>()?;
            store
                .labeled_type_pack(elements.into_boxed_slice())
                .map_err(|_| TypeMaterializationError::TypeApplication)
        }
        TypeData::Pack(pack) => {
            let elements = pack
                .elements
                .iter()
                .map(|element| {
                    Ok(super::store::PackTypeElement {
                        label: element.label.clone(),
                        ty: materialize_type_inner(store, element.ty, instantiation, row_mode, visiting_rows)?,
                    })
                })
                .collect::<Result<Vec<_>, TypeMaterializationError>>()?;
            let pos_rem = match pack.positional_remainder {
                super::store::AnonymousTupleRemainder::Typed(t) => {
                    super::store::AnonymousTupleRemainder::Typed(materialize_type_inner(store, t, instantiation, row_mode, visiting_rows)?)
                }
                rem => rem,
            };
            let lab_rem = match pack.labeled_remainder {
                super::store::AnonymousTupleRemainder::Typed(t) => {
                    super::store::AnonymousTupleRemainder::Typed(materialize_type_inner(store, t, instantiation, row_mode, visiting_rows)?)
                }
                rem => rem,
            };
            Ok(store.open_pack(elements.into_boxed_slice(), pos_rem, lab_rem)?)
        }
        TypeData::PackFromPacks { positionals, labeled } => {
            let pos = materialize_type_inner(store, positionals, instantiation, row_mode, visiting_rows)?;
            let lab = materialize_type_inner(store, labeled, instantiation, row_mode, visiting_rows)?;
            Ok(store.pack_from_packs(pos, lab)?)
        }
        TypeData::TupleFromPacks { positionals, labeled } => {
            let pos = materialize_type_inner(store, positionals, instantiation, row_mode, visiting_rows)?;
            let lab = materialize_type_inner(store, labeled, instantiation, row_mode, visiting_rows)?;
            Ok(store.tuple_from_packs(pos, lab)?)
        }
        TypeData::CallableFromPacks {
            positionals,
            labeled,
            return_type,
        } => {
            let pos = materialize_type_inner(store, positionals, instantiation, row_mode, visiting_rows)?;
            let lab = materialize_type_inner(store, labeled, instantiation, row_mode, visiting_rows)?;
            let ret = materialize_type_inner(store, return_type, instantiation, row_mode, visiting_rows)?;
            Ok(store.callable_from_packs(pos, lab, ret)?)
        }
        TypeData::CompleteTypePack { positionals, labeled } => {
            let positionals = materialize_type_inner(store, positionals, instantiation, row_mode, visiting_rows)?;
            let labeled = materialize_type_inner(store, labeled, instantiation, row_mode, visiting_rows)?;
            Ok(store.complete_type_pack(positionals, labeled)?)
        }
        TypeData::TypePackConcat(segments) => {
            let segments = segments
                .iter()
                .map(|&segment| materialize_type_inner(store, segment, instantiation, row_mode, visiting_rows))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(store.type_pack_concat(&segments)?)
        }
        TypeData::LabeledTypePackConcat(segments) => {
            let segments = segments
                .iter()
                .map(|&segment| materialize_type_inner(store, segment, instantiation, row_mode, visiting_rows))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(store.labeled_type_pack_concat(&segments)?)
        }
        TypeData::StaticData(term) => {
            let arguments = term
                .arguments
                .iter()
                .map(|argument| match argument {
                    super::store::StaticArgument::Term(argument) => Ok(super::store::StaticArgument::Term(materialize_type_inner(
                        store,
                        *argument,
                        instantiation,
                        row_mode,
                        visiting_rows,
                    )?)),
                    super::store::StaticArgument::RecordRow(row) => Ok(super::store::StaticArgument::RecordRow(materialize_row_id(
                        store,
                        *row,
                        instantiation,
                        row_mode,
                        visiting_rows,
                    )?)),
                    super::store::StaticArgument::DynamicType => Ok(super::store::StaticArgument::DynamicType),
                })
                .collect::<Result<Vec<_>, TypeMaterializationError>>()?
                .into_boxed_slice();
            store
                .static_data(term.domain, term.constructor, arguments)
                .map_err(|_| TypeMaterializationError::TypeApplication)
        }
        TypeData::ConstInt(_) => Ok(ty),
        TypeData::Lambda(_)
        | TypeData::SelfType(_)
        | TypeData::Never
        | TypeData::Unit
        | TypeData::Nominal { .. }
        | TypeData::StaticDomainForm { .. }
        | TypeData::ClassObject { .. } => Ok(ty),
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
                key: field.key.clone(),
                ty: materialize_type_inner(store, field.ty, instantiation, row_mode, visiting_rows)?,
                presence: field.presence,
            })
        })
        .collect::<Result<Vec<_>, TypeMaterializationError>>()?;
    materialize_row_tail(store, fields, row.tail, instantiation, row_mode, visiting_rows)
}

fn materialize_row_id(
    store: &mut TypeStore,
    row_id: RecordRowId,
    instantiation: &GenericInstantiation,
    row_mode: RowMaterializationMode,
    visiting_rows: &mut HashSet<TypeParameterId>,
) -> Result<RecordRowId, TypeMaterializationError> {
    let (fields, tail) = materialize_row(store, row_id, instantiation, row_mode, visiting_rows)?;
    Ok(store.record_row_checked(fields, tail)?)
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
