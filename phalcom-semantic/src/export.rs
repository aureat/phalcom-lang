//! Stable export descriptors for compiled modules and compiler seams.

use crate::identity::DeclarationId;
use crate::types::id::{InferVarId, KindId, TypeId};
use crate::types::kind::KindData;
use crate::types::parameter::TypeParameterOwner;
use crate::types::store::{TypeData, TypeStore};

/// Stable exported kind representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompiledKindRef {
    Type,
    Arrow {
        parameters: Box<[CompiledKindRef]>,
        result: Box<CompiledKindRef>,
    },
}

/// Stable owner for a type parameter in exported metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompiledTypeParameterOwner {
    Declaration(DeclarationId),
    Callable(crate::identity::CallableId),
}

/// Element in an exported tuple type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledTupleElement {
    pub label: Option<Box<str>>,
    pub ty: CompiledTypeRef,
}

/// Field in an exported record type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledRecordField {
    pub name: Box<str>,
    pub ty: CompiledTypeRef,
}

/// Parameter in an exported callable type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledCallableParam {
    pub name: Option<Box<str>>,
    pub ty: CompiledTypeRef,
}

/// Exported callable type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledCallableType {
    pub positional: Box<[CompiledCallableParam]>,
    pub return_type: Box<CompiledTypeRef>,
}

/// Stable, store-independent type form reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompiledTypeRef {
    Never,
    Unit,
    Nominal(DeclarationId),
    Applied {
        origin: Box<CompiledTypeRef>,
        arguments: Box<[CompiledTypeRef]>,
    },
    Union(Box<[CompiledTypeRef]>),
    Tuple(Box<[CompiledTupleElement]>),
    Record(Box<[CompiledRecordField]>),
    Callable(CompiledCallableType),
    Parameter {
        owner: CompiledTypeParameterOwner,
        index: u16,
    },
}

/// Error encountered when converting an internal type form into a stable export descriptor.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SemanticExportError {
    #[error("cannot export inference variable {0:?}")]
    InferenceVariable(InferVarId),
    #[error("cannot export non-exportable type form: {form:?}")]
    NonExportableTypeForm { form: TypeId },
}

/// Converts a canonical [`KindId`] into a stable [`CompiledKindRef`].
pub fn export_kind(store: &TypeStore, kind: KindId) -> CompiledKindRef {
    match store.get_kind(kind) {
        KindData::Type => CompiledKindRef::Type,
        KindData::Arrow { parameters, result } => {
            let p_kinds: Vec<CompiledKindRef> =
                parameters.iter().map(|&p| export_kind(store, p)).collect();
            let r_kind = export_kind(store, *result);
            CompiledKindRef::Arrow {
                parameters: p_kinds.into_boxed_slice(),
                result: Box::new(r_kind),
            }
        }
    }
}

/// Converts an interned [`TypeId`] into a stable [`CompiledTypeRef`].
pub fn export_type_form(
    store: &TypeStore,
    form: TypeId,
) -> Result<CompiledTypeRef, SemanticExportError> {
    match store.get(form) {
        TypeData::Never => Ok(CompiledTypeRef::Never),
        TypeData::Unit => Ok(CompiledTypeRef::Unit),
        TypeData::Nominal { declaration } => Ok(CompiledTypeRef::Nominal(declaration.clone())),
        TypeData::Applied { origin, arguments } => {
            let orig = export_type_form(store, *origin)?;
            let args = arguments
                .iter()
                .map(|&a| export_type_form(store, a))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CompiledTypeRef::Applied {
                origin: Box::new(orig),
                arguments: args.into_boxed_slice(),
            })
        }
        TypeData::Union(members) => {
            let mems = members
                .iter()
                .map(|&m| export_type_form(store, m))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CompiledTypeRef::Union(mems.into_boxed_slice()))
        }
        TypeData::Tuple(elems) => {
            let tuple_elems = elems
                .iter()
                .map(|e| {
                    export_type_form(store, e.ty).map(|t| CompiledTupleElement {
                        label: e.label.clone(),
                        ty: t,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CompiledTypeRef::Tuple(tuple_elems.into_boxed_slice()))
        }
        TypeData::Record(fields) => {
            let rec_fields = fields
                .iter()
                .map(|f| {
                    export_type_form(store, f.ty).map(|t| CompiledRecordField {
                        name: f.name.clone(),
                        ty: t,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CompiledTypeRef::Record(rec_fields.into_boxed_slice()))
        }
        TypeData::Callable(c) => {
            let pos = c
                .parameters
                .iter()
                .map(|p| {
                    export_type_form(store, p.ty).map(|t| CompiledCallableParam {
                        name: p.label.clone(),
                        ty: t,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let ret = export_type_form(store, c.return_type)?;
            Ok(CompiledTypeRef::Callable(CompiledCallableType {
                positional: pos.into_boxed_slice(),
                return_type: Box::new(ret),
            }))
        }
        TypeData::Parameter(p_id) => {
            let p_data = store.type_parameter(*p_id);
            let owner = match &p_data.owner {
                TypeParameterOwner::Declaration(decl) => {
                    CompiledTypeParameterOwner::Declaration(decl.clone())
                }
                TypeParameterOwner::Callable(c_id) => {
                    CompiledTypeParameterOwner::Callable(c_id.clone())
                }
            };
            Ok(CompiledTypeRef::Parameter {
                owner,
                index: p_data.index,
            })
        }
        TypeData::Infer(var) => Err(SemanticExportError::InferenceVariable(*var)),
        TypeData::ClassObject { .. } => {
            Err(SemanticExportError::NonExportableTypeForm { form })
        }
    }
}
