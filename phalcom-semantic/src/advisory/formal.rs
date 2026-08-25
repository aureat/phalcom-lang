//! Bounded formal-to-advisory projection.
//!
//! This helper reads canonical `TypeData` only. It does not synthesize
//! declarations, diagnostics, or checker knowledge, and its depth bound makes
//! pathological generic/union shapes fail closed to `Unknown`.

use crate::types::evidence::TypeKnowledge;
use crate::types::id::TypeId;
use crate::types::store::{TypeData, TypeStore};

use super::{AdvisoryFact, AdvisoryOrigin, ValueShape};

pub const DEFAULT_FORMAL_PROJECTION_DEPTH: usize = 8;

/// Projects one formal knowledge result into advisory runtime shape.
pub fn advisory_fact_from_formal(store: &TypeStore, knowledge: &TypeKnowledge, origin: AdvisoryOrigin) -> AdvisoryFact {
    let shape = match knowledge {
        TypeKnowledge::Known(evidence) => shape_from_type(store, evidence.ty, DEFAULT_FORMAL_PROJECTION_DEPTH),
        TypeKnowledge::Unknown(_) | TypeKnowledge::Dynamic(_) => ValueShape::Unknown,
    };
    AdvisoryFact::interprocedural(shape, origin)
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
        TypeData::Union(types) => ValueShape::bounded_union(types.iter().map(|ty| shape_from_type(store, *ty, depth - 1))),
        TypeData::Tuple(elements) => ValueShape::Tuple(
            elements
                .iter()
                .map(|element| shape_from_type(store, element.ty, depth - 1))
                .collect::<Vec<_>>()
                .into(),
        ),
        TypeData::Record(_) | TypeData::Callable(_) | TypeData::Parameter(_) | TypeData::Lambda(_) | TypeData::SelfType(_) => ValueShape::Unknown,
    }
}
