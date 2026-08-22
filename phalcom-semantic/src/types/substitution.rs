//! Type parameter substitution for generic declarations and applied member views.

use super::id::{TypeId, TypeParameterId};
use super::store::{CallableParameterType, CallableType, RecordTypeField, TupleTypeElement, TypeData, TypeStore};
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
            TypeData::Record(fields) => {
                let subst_fields: Vec<RecordTypeField> = fields
                    .iter()
                    .map(|field| RecordTypeField {
                        name: field.name.clone(),
                        ty: self.apply(store, field.ty),
                    })
                    .collect();
                store.record(subst_fields.into_boxed_slice())
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
            TypeData::Never | TypeData::Unit | TypeData::Nominal { .. } | TypeData::ClassObject { .. } | TypeData::Infer(_) => ty,
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
