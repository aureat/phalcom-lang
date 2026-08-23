//! Type environment, specialization views, and lazy member projection.

use super::id::{TypeId, TypeParameterId};
use super::row::{RecordRowData, RecordRowField};
use super::store::{CallableParameterType, CallableType, TupleTypeElement, TypeData, TypeStore};
use super::substitution::TypeSubstitution;
use crate::identity::{CallableId, DeclarationId};
use std::collections::HashMap;

/// An environment mapping type parameters and `Self` bindings for lazy specialization.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TypeEnvironment {
    pub bindings: HashMap<TypeParameterId, TypeId>,
    pub self_binding: Option<TypeId>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_param(&mut self, param: TypeParameterId, ty: TypeId) {
        self.bindings.insert(param, ty);
    }

    pub fn bind_self(&mut self, ty: TypeId) {
        self.self_binding = Some(ty);
    }

    pub fn get_param(&self, param: TypeParameterId) -> Option<TypeId> {
        self.bindings.get(&param).copied()
    }

    pub fn get_self(&self) -> Option<TypeId> {
        self.self_binding
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty() && self.self_binding.is_none()
    }

    pub fn to_substitution(&self) -> TypeSubstitution {
        let mut subst = TypeSubstitution::new();
        for (&p, &t) in &self.bindings {
            subst.bind(p, t);
        }
        subst
    }
}

/// A lazy view of a `TypeId` under a specialization `TypeEnvironment`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeView {
    pub root: TypeId,
    pub environment: TypeEnvironment,
}

impl TypeView {
    pub fn new(root: TypeId, environment: TypeEnvironment) -> Self {
        Self { root, environment }
    }

    pub fn identity(root: TypeId) -> Self {
        Self {
            root,
            environment: TypeEnvironment::new(),
        }
    }

    /// Materializes this lazy view into a canonical `TypeId` in `store`.
    pub fn materialize(&self, store: &mut TypeStore) -> TypeId {
        if self.environment.is_empty() {
            return self.root;
        }
        materialize_view(store, self.root, &self.environment)
    }
}

fn materialize_view(store: &mut TypeStore, ty: TypeId, env: &TypeEnvironment) -> TypeId {
    match store.get(ty).clone() {
        TypeData::Parameter(param) => {
            if let Some(replacement) = env.get_param(param) {
                replacement
            } else {
                ty
            }
        }
        TypeData::SelfType(term) => {
            if let Some(self_ty) = env.get_self() {
                self_ty
            } else {
                ty
            }
        }
        TypeData::Applied { origin, arguments } => {
            let subst_origin = materialize_view(store, origin, env);
            let subst_args: Vec<TypeId> = arguments.iter().map(|&a| materialize_view(store, a, env)).collect();
            store.apply_type_form(subst_origin, &subst_args).unwrap_or(ty)
        }
        TypeData::Union(members) => {
            let subst_members: Vec<TypeId> = members.iter().map(|&m| materialize_view(store, m, env)).collect();
            store.union(&subst_members)
        }
        TypeData::Tuple(elems) => {
            let subst_elems: Vec<TupleTypeElement> = elems
                .iter()
                .map(|e| TupleTypeElement {
                    label: e.label.clone(),
                    ty: materialize_view(store, e.ty, env),
                })
                .collect();
            store.tuple(subst_elems.into_boxed_slice())
        }
        TypeData::Record(row_id) => {
            let (fields, tail) = {
                let row = store.record_row(row_id);
                (row.fields.to_vec(), row.tail)
            };
            let subst_fields: Vec<RecordRowField> = fields
                .into_iter()
                .map(|f| RecordRowField {
                    name: f.name,
                    ty: materialize_view(store, f.ty, env),
                })
                .collect();
            let row_data = RecordRowData {
                fields: subst_fields.into_boxed_slice(),
                tail,
            };
            let new_row_id = store.intern_record_row(row_data);
            store.record_type(new_row_id)
        }
        TypeData::Callable(call) => {
            let params: Vec<CallableParameterType> = call
                .parameters
                .iter()
                .map(|p| CallableParameterType {
                    label: p.label.clone(),
                    ty: materialize_view(store, p.ty, env),
                    rest: p.rest,
                })
                .collect();
            let return_type = materialize_view(store, call.return_type, env);
            store.callable(CallableType {
                parameters: params.into_boxed_slice(),
                return_type,
            })
        }
        TypeData::Never | TypeData::Unit | TypeData::Nominal { .. } | TypeData::ClassObject { .. } | TypeData::Lambda(_) => ty,
    }
}

/// A specialized view over a callable member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpecializedCallableView {
    pub callable: CallableId,
    pub environment: TypeEnvironment,
}

/// A specialized member view (method or field).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpecializedMemberView {
    pub callable: Option<CallableId>,
    pub environment: TypeEnvironment,
}
