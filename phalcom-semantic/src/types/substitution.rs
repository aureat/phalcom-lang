//! Type parameter substitution for generic declarations and applied member views.

use super::id::{TypeId, TypeParameterId};
use super::row::{RecordRowData, RecordRowField};
use super::store::{CallableParameterType, CallableType, TupleTypeElement, TypeData, TypeStore};
use crate::declarations::DeclarationTypeTable;
use std::collections::HashMap;

/// A map of type parameter replacements (`T -> Type`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TypeSubstitution {
    bindings: HashMap<TypeParameterId, TypeId>,
}

impl TypeSubstitution {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&mut self, param: TypeParameterId, ty: TypeId) {
        self.bindings.insert(param, ty);
    }

    pub fn get(&self, param: TypeParameterId) -> Option<TypeId> {
        self.bindings.get(&param).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Recursively applies this substitution to `ty`.
    pub fn apply(&self, store: &mut TypeStore, ty: TypeId) -> TypeId {
        if self.is_empty() {
            return ty;
        }

        match store.get(ty).clone() {
            TypeData::Parameter(param_id) => {
                if let Some(&replacement) = self.bindings.get(&param_id) {
                    replacement
                } else {
                    ty
                }
            }
            TypeData::Applied { origin, arguments } => {
                let subst_origin = self.apply(store, origin);
                let subst_args: Vec<TypeId> = arguments.iter().map(|&arg| self.apply(store, arg)).collect();
                store.apply_type_form(subst_origin, &subst_args).unwrap_or(ty)
            }
            TypeData::ExactCase { variant, enum_type } => {
                let subst_enum = self.apply(store, enum_type);
                let variant_id = store.variant_identity(variant).clone();
                store.exact_case_type(&variant_id, subst_enum).unwrap_or(ty)
            }
            TypeData::Union(members) => {
                let subst_members: Vec<TypeId> = members.iter().map(|&m| self.apply(store, m)).collect();
                store.union(&subst_members)
            }
            TypeData::Tuple(elements) => {
                let subst_elements: Vec<TupleTypeElement> = elements
                    .iter()
                    .map(|elem| TupleTypeElement {
                        label: elem.label.clone(),
                        ty: self.apply(store, elem.ty),
                    })
                    .collect();
                store.tuple(subst_elements.into_boxed_slice())
            }
            TypeData::Record(row_id) => {
                let (fields, tail) = {
                    let row = store.record_row(row_id);
                    (row.fields.to_vec(), row.tail)
                };
                let subst_fields: Vec<RecordRowField> = fields
                    .into_iter()
                    .map(|field| RecordRowField {
                        name: field.name,
                        ty: self.apply(store, field.ty),
                    })
                    .collect();
                let row_data = RecordRowData {
                    fields: subst_fields.into_boxed_slice(),
                    tail,
                };
                let new_row_id = store.intern_record_row(row_data);
                store.record_type(new_row_id)
            }
            TypeData::Callable(callable) => {
                let params: Vec<CallableParameterType> = callable
                    .parameters
                    .iter()
                    .map(|p| CallableParameterType {
                        label: p.label.clone(),
                        ty: self.apply(store, p.ty),
                        rest: p.rest,
                    })
                    .collect();
                let return_type = self.apply(store, callable.return_type);
                store.callable(CallableType {
                    parameters: params.into_boxed_slice(),
                    return_type,
                })
            }
            TypeData::SelfType(_) => ty,
            TypeData::Lambda(_) => ty,
            TypeData::Never | TypeData::Unit | TypeData::Nominal { .. } | TypeData::ClassObject { .. } => ty,
        }
    }
}

/// Builds a substitution from declaration generic signature and applied type arguments.
pub fn substitution_for_applied(declarations: &DeclarationTypeTable, store: &TypeStore, applied: TypeId) -> Option<TypeSubstitution> {
    let (origin, arguments) = match store.get(applied) {
        TypeData::Applied { origin, arguments } => (*origin, arguments),
        _ => return None,
    };

    let mut base = origin;
    while let TypeData::Applied { origin: next, .. } = store.get(base) {
        base = *next;
    }

    if let TypeData::Nominal { declaration } = store.get(base) {
        if let Some(info) = declarations.get(declaration) {
            if let Some(sig) = &info.generic_signature {
                let mut subst = TypeSubstitution::new();
                for (&param_id, &arg_ty) in sig.parameters.iter().zip(arguments.iter()) {
                    subst.bind(param_id, arg_ty);
                }
                return Some(subst);
            }
        }
    }

    None
}

/// Specializes `Self` type terms within `ty` based on the concrete `receiver` type.
pub fn specialize_self_type(store: &mut TypeStore, declarations: &DeclarationTypeTable, receiver: TypeId, ty: TypeId) -> TypeId {
    match store.get(ty).clone() {
        TypeData::SelfType(term) => match term.role {
            crate::types::parameter::SelfRole::ReceiverValue => receiver,
            crate::types::parameter::SelfRole::InstanceType => match store.get(receiver).clone() {
                TypeData::ClassObject { declaration } => declarations.form(&declaration).unwrap_or(receiver),
                TypeData::Nominal { .. } | TypeData::Applied { .. } => receiver,
                _ => declarations.form(&term.owner).unwrap_or(receiver),
            },
        },
        TypeData::Applied { origin, arguments } => {
            let subst_origin = specialize_self_type(store, declarations, receiver, origin);
            let subst_args: Vec<TypeId> = arguments.iter().map(|&arg| specialize_self_type(store, declarations, receiver, arg)).collect();
            store.apply_type_form(subst_origin, &subst_args).unwrap_or(ty)
        }
        TypeData::ExactCase { variant, enum_type } => {
            let subst_enum = specialize_self_type(store, declarations, receiver, enum_type);
            let variant_id = store.variant_identity(variant).clone();
            store.exact_case_type(&variant_id, subst_enum).unwrap_or(ty)
        }
        TypeData::Union(members) => {
            let subst_members: Vec<TypeId> = members.iter().map(|&m| specialize_self_type(store, declarations, receiver, m)).collect();
            store.union(&subst_members)
        }
        TypeData::Tuple(elements) => {
            let subst_elements: Vec<TupleTypeElement> = elements
                .iter()
                .map(|elem| TupleTypeElement {
                    label: elem.label.clone(),
                    ty: specialize_self_type(store, declarations, receiver, elem.ty),
                })
                .collect();
            store.tuple(subst_elements.into_boxed_slice())
        }
        TypeData::Record(row_id) => {
            let (fields, tail) = {
                let row = store.record_row(row_id);
                (row.fields.to_vec(), row.tail)
            };
            let subst_fields: Vec<RecordRowField> = fields
                .into_iter()
                .map(|field| RecordRowField {
                    name: field.name,
                    ty: specialize_self_type(store, declarations, receiver, field.ty),
                })
                .collect();
            let row_data = RecordRowData {
                fields: subst_fields.into_boxed_slice(),
                tail,
            };
            let new_row_id = store.intern_record_row(row_data);
            store.record_type(new_row_id)
        }
        TypeData::Callable(callable) => {
            let params: Vec<CallableParameterType> = callable
                .parameters
                .iter()
                .map(|p| CallableParameterType {
                    label: p.label.clone(),
                    ty: specialize_self_type(store, declarations, receiver, p.ty),
                    rest: p.rest,
                })
                .collect();
            let return_type = specialize_self_type(store, declarations, receiver, callable.return_type);
            store.callable(CallableType {
                parameters: params.into_boxed_slice(),
                return_type,
            })
        }
        TypeData::Parameter(_) | TypeData::Lambda(_) | TypeData::Never | TypeData::Unit | TypeData::Nominal { .. } | TypeData::ClassObject { .. } => ty,
    }
}
