//! Bounded formal-to-advisory projection.
//!
//! This helper reads canonical `TypeData` only. It does not synthesize
//! declarations, diagnostics, or checker knowledge, and its depth bound makes
//! pathological generic/union shapes fail closed to `Unknown`.

use crate::types::evidence::TypeKnowledge;
use crate::types::id::TypeId;
use crate::types::parameter::{SelfRole, SelfTypeTerm};
use crate::types::store::{TypeData, TypeStore};

use super::{AdvisoryFact, AdvisoryOrigin, ValueShape};

pub const DEFAULT_FORMAL_PROJECTION_DEPTH: usize = 8;

/// Projects one formal knowledge result into advisory runtime shape.
pub fn advisory_fact_from_formal(store: &TypeStore, knowledge: &TypeKnowledge, origin: AdvisoryOrigin) -> AdvisoryFact {
    AdvisoryFact::interprocedural(advisory_shape_from_formal(store, knowledge), origin)
}

/// Projects formal knowledge to a bounded advisory shape without constructing
/// advisory provenance.
pub fn advisory_shape_from_formal(store: &TypeStore, knowledge: &TypeKnowledge) -> ValueShape {
    match knowledge {
        TypeKnowledge::Known(evidence) => shape_from_type(store, evidence.ty(), DEFAULT_FORMAL_PROJECTION_DEPTH),
        TypeKnowledge::Unknown(_) | TypeKnowledge::Dynamic(_) => ValueShape::Unknown,
    }
}

/// Projects formal knowledge relative to a concrete runtime receiver shape.
/// This is the advisory counterpart of formal `Self` specialization: the
/// callable contract remains canonical while only its dependent result is
/// projected for editor/runtime-shape consumers.
pub fn advisory_shape_from_formal_for_receiver(store: &TypeStore, knowledge: &TypeKnowledge, receiver: &ValueShape) -> ValueShape {
    match knowledge {
        TypeKnowledge::Known(evidence) => shape_from_type_for_receiver(store, evidence.ty(), receiver, DEFAULT_FORMAL_PROJECTION_DEPTH),
        TypeKnowledge::Unknown(_) | TypeKnowledge::Dynamic(_) => ValueShape::Unknown,
    }
}

fn shape_from_type_for_receiver(store: &TypeStore, ty: TypeId, receiver: &ValueShape, depth: usize) -> ValueShape {
    if depth == 0 {
        return ValueShape::Unknown;
    }
    match store.get(ty) {
        TypeData::SelfType(term) => shape_from_self(term, receiver, depth - 1),
        TypeData::Never => ValueShape::Never,
        TypeData::Unit => ValueShape::Unit,
        TypeData::ClassObject { declaration } => ValueShape::ClassObject(declaration.clone()),
        TypeData::Nominal { declaration } => ValueShape::Instance(declaration.clone()),
        TypeData::Applied { origin, .. } => shape_from_type_for_receiver(store, *origin, receiver, depth - 1),
        TypeData::ExactCase { enum_type, .. } => shape_from_type_for_receiver(store, *enum_type, receiver, depth - 1),
        TypeData::Union(types) => ValueShape::bounded_union(types.iter().map(|ty| shape_from_type_for_receiver(store, *ty, receiver, depth - 1))),
        TypeData::Tuple(elements) => ValueShape::Tuple(
            elements
                .iter()
                .map(|element| shape_from_type_for_receiver(store, element.ty, receiver, depth - 1))
                .collect::<Vec<_>>()
                .into(),
        ),
        TypeData::Record(_) | TypeData::Callable(_) | TypeData::Family(_) | TypeData::Parameter(_) | TypeData::Lambda(_) => ValueShape::Unknown,
    }
}

fn shape_from_self(term: &SelfTypeTerm, receiver: &ValueShape, depth: usize) -> ValueShape {
    if depth == 0 {
        return ValueShape::Unknown;
    }
    match term.role {
        SelfRole::ReceiverValue => receiver.clone(),
        SelfRole::InstanceType => match receiver {
            ValueShape::ClassObject(declaration) | ValueShape::Instance(declaration) => ValueShape::Instance(declaration.clone()),
            ValueShape::Union(shapes) => ValueShape::bounded_union(shapes.iter().map(|shape| shape_from_self(term, shape, depth - 1))),
            _ => ValueShape::Unknown,
        },
    }
}

fn shape_from_type(store: &TypeStore, ty: TypeId, depth: usize) -> ValueShape {
    if depth == 0 {
        return ValueShape::Unknown;
    }
    match store.get(ty) {
        TypeData::Never => ValueShape::Never,
        TypeData::Unit => ValueShape::Unit,
        TypeData::ClassObject { declaration } => ValueShape::ClassObject(declaration.clone()),
        TypeData::Nominal { declaration } => ValueShape::Instance(declaration.clone()),
        TypeData::Applied { origin, .. } => shape_from_type(store, *origin, depth - 1),
        TypeData::ExactCase { enum_type, .. } => shape_from_type(store, *enum_type, depth - 1),
        TypeData::Union(types) => ValueShape::bounded_union(types.iter().map(|ty| shape_from_type(store, *ty, depth - 1))),
        TypeData::Tuple(elements) => ValueShape::Tuple(
            elements
                .iter()
                .map(|element| shape_from_type(store, element.ty, depth - 1))
                .collect::<Vec<_>>()
                .into(),
        ),
        TypeData::Record(_) | TypeData::Callable(_) | TypeData::Family(_) | TypeData::Parameter(_) | TypeData::Lambda(_) | TypeData::SelfType(_) => {
            ValueShape::Unknown
        }
    }
}
