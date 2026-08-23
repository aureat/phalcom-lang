//! Intraprocedural effect inference consuming CallableAnalysis products.

use super::atom::{EffectAtom, EffectSet};
use super::summary::{EffectKnowledge, EffectOpaqueReason};
use crate::checker::analysis::{AnalysisStatus, CallableAnalysis};
use crate::types::evidence::DynamicReason;
use phalcom_native_meta::primitive::{EffectSpec, NativeEffect};

pub fn adapt_effect_atom(native: NativeEffect) -> EffectAtom {
    EffectAtom::from_native(native)
}

pub fn adapt_effect_spec(spec: EffectSpec) -> EffectKnowledge {
    match spec {
        EffectSpec::Pure => EffectKnowledge::Known(EffectSet::EMPTY),
        EffectSpec::Known(atoms) => {
            let mut set = EffectSet::EMPTY;
            for &a in atoms {
                set = set.insert(adapt_effect_atom(a));
            }
            EffectKnowledge::Known(set)
        }
        EffectSpec::Unknown => EffectKnowledge::Opaque(EffectOpaqueReason::MissingNativeMetadata),
    }
}

/// Infers the intraprocedural effect knowledge of a callable from its analysis product.
pub fn infer_intraprocedural_effects(analysis: &CallableAnalysis) -> EffectKnowledge {
    let current_effects = EffectSet::EMPTY;

    for expr in analysis.expressions.values() {
        match &expr.status {
            AnalysisStatus::Ready => {}
            AnalysisStatus::DynamicBoundary(reason) => {
                let opaque_reason = match reason {
                    DynamicReason::RuntimeReflection => EffectOpaqueReason::ReflectivePerform,
                    DynamicReason::DynamicRestPack => EffectOpaqueReason::DynamicDispatch,
                    DynamicReason::ExplicitEscape => EffectOpaqueReason::ForeignBoundary,
                };
                return EffectKnowledge::Opaque(opaque_reason);
            }
            AnalysisStatus::Blocked(_) => {
                return EffectKnowledge::Opaque(EffectOpaqueReason::UnknownDependency);
            }
            AnalysisStatus::Invalid(_) => {}
            AnalysisStatus::Cancelled | AnalysisStatus::BudgetExceeded(_) | AnalysisStatus::InternalFailure(_) => {
                return EffectKnowledge::Opaque(EffectOpaqueReason::UnsupportedConstruct);
            }
        }
    }

    EffectKnowledge::Known(current_effects)
}
