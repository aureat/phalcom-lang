//! Normalization of rich native metadata specifications into canonical semantic types.

use super::evidence::{EvidenceAuthority, TypeEvidence, TypeKnowledge, UnknownReason};
use super::store::TypeStore;
use crate::identity::DeclarationId;
use phalcom_native_meta::types::TypeExprSpec;
use phalcom_native_meta::universe::UniverseKey;

/// Normalizes a native [`TypeExprSpec`] into canonical [`TypeKnowledge`].
pub fn normalize_native_type(
    store: &mut TypeStore,
    universe_resolver: &dyn Fn(UniverseKey) -> DeclarationId,
    spec: &TypeExprSpec,
) -> TypeKnowledge {
    match spec {
        TypeExprSpec::Never => TypeKnowledge::Known(TypeEvidence {
            ty: store.never(),
            authority: EvidenceAuthority::TrustedNative,
            provenance: Default::default(),
        }),
        TypeExprSpec::Universe(UniverseKey::Unit) => TypeKnowledge::Known(TypeEvidence {
            ty: store.unit(),
            authority: EvidenceAuthority::TrustedNative,
            provenance: Default::default(),
        }),
        TypeExprSpec::Universe(key) => {
            let decl = universe_resolver(*key);
            let ty = store.nominal(decl);
            TypeKnowledge::Known(TypeEvidence {
                ty,
                authority: EvidenceAuthority::TrustedNative,
                provenance: Default::default(),
            })
        }
        TypeExprSpec::Union(members) => {
            let mut tys = Vec::new();
            for m in *members {
                let k = normalize_native_type(store, universe_resolver, m);
                if let Some(ty) = k.ty() {
                    tys.push(ty);
                } else {
                    return TypeKnowledge::Unknown(UnknownReason::OpaqueNative);
                }
            }
            let union_ty = store.union(&tys);
            TypeKnowledge::Known(TypeEvidence {
                ty: union_ty,
                authority: EvidenceAuthority::TrustedNative,
                provenance: Default::default(),
            })
        }
        TypeExprSpec::Unknown => TypeKnowledge::Unknown(UnknownReason::OpaqueNative),
        _ => TypeKnowledge::Unknown(UnknownReason::OpaqueNative),
    }
}
