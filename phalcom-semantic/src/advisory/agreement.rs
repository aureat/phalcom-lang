//! Pure comparison between ready formal knowledge and advisory shape.
//!
//! This module is diagnostic and test support only. It never participates in
//! checker acceptance and never turns an advisory observation into formal
//! evidence.

use crate::checker::{AnalysisStatus, ExpressionAnalysis};
use crate::types::evidence::TypeKnowledge;
use crate::types::store::TypeStore;

use super::formal::advisory_shape_from_formal;
use super::{AdvisoryFact, ValueShape};

/// Relationship between a ready formal projection and an advisory shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdvisoryAgreement {
    /// The advisory shape agrees with the supported formal projection.
    Compatible,
    /// The advisory shape is a strict refinement of that projection.
    MoreSpecific,
    /// No sound comparison is available for these representations.
    Incomparable,
    /// Formal analysis is unknown, dynamic, invalid, suppressed, or not ready.
    Unknown,
}

/// Compares one expression's advisory observation with its formal product.
///
/// Only a `Ready` expression with `TypeKnowledge::Known` is comparable. All
/// other statuses deliberately return `Unknown`; disagreement must not alter
/// the formal product or emit a hard diagnostic.
pub fn compare_expression(store: &TypeStore, formal: &ExpressionAnalysis, advisory: &AdvisoryFact) -> AdvisoryAgreement {
    if !matches!(formal.status, AnalysisStatus::Ready) {
        return AdvisoryAgreement::Unknown;
    }
    compare_known(store, &formal.knowledge, &advisory.shape)
}

/// Compares a formal knowledge value with an advisory shape when no expression
/// status wrapper is available.
pub fn compare_known(store: &TypeStore, formal: &TypeKnowledge, advisory: &ValueShape) -> AdvisoryAgreement {
    let TypeKnowledge::Known(_) = formal else {
        return AdvisoryAgreement::Unknown;
    };
    let projected = advisory_shape_from_formal(store, formal);
    compare_shapes(&projected, advisory)
}

fn compare_shapes(formal: &ValueShape, advisory: &ValueShape) -> AdvisoryAgreement {
    if matches!(advisory, ValueShape::Unknown) || matches!(formal, ValueShape::Unknown) {
        return AdvisoryAgreement::Unknown;
    }
    if formal == advisory {
        return AdvisoryAgreement::Compatible;
    }
    if is_shape_refinement(formal, advisory) {
        AdvisoryAgreement::MoreSpecific
    } else {
        AdvisoryAgreement::Incomparable
    }
}

fn is_shape_refinement(formal: &ValueShape, advisory: &ValueShape) -> bool {
    match (formal, advisory) {
        (ValueShape::Union(formal), advisory) => formal.iter().any(|candidate| candidate == advisory),
        (ValueShape::List(formal), ValueShape::ExactList(advisory)) => advisory.iter().all(|item| is_shape_refinement(formal, item) || formal.as_ref() == item),
        (ValueShape::Tuple(formal), ValueShape::Tuple(advisory)) if formal.len() == advisory.len() => formal
            .iter()
            .zip(advisory.iter())
            .all(|(formal, advisory)| formal == advisory || is_shape_refinement(formal, advisory)),
        _ => false,
    }
}
