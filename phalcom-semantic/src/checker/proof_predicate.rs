//! Compiler-owned checked products for static proof predicates (LANG007.C4.P6 T9, LANG007.C4.P7 T0).

use crate::generic_evidence::GenericConformanceKey;
use crate::identity::{DeclarationId, ExpressionId};
use crate::types::id::TypeId;
use phalcom_common::range::SourceRange;
use std::collections::BTreeMap;

/// Canonical proof predicate operation kind.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProofPredicateKind {
    Equivalent,
    Subtype,
    Conforms,
}

/// Canonical semantic goal for a proof predicate proposition (LANG007.C4.P7 T0).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ProofPredicateGoal {
    Equivalent { left: TypeId, right: TypeId },
    Subtype { lower: TypeId, upper: TypeId },
    Conforms { key: GenericConformanceKey },
}

/// Checked product identifying one proof predicate expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofPredicateResolution {
    pub expression: ExpressionId,
    pub range: SourceRange,
    pub goal: ProofPredicateGoal,
    pub result_type: TypeId,
}

impl ProofPredicateResolution {
    pub fn kind(&self) -> ProofPredicateKind {
        match &self.goal {
            ProofPredicateGoal::Equivalent { .. } => ProofPredicateKind::Equivalent,
            ProofPredicateGoal::Subtype { .. } => ProofPredicateKind::Subtype,
            ProofPredicateGoal::Conforms { .. } => ProofPredicateKind::Conforms,
        }
    }
}

/// Index of proof predicate resolutions by expression ID.
pub type ProofPredicateResolutionIndex = BTreeMap<ExpressionId, ProofPredicateResolution>;

/// Returns the canonical declaration identity for a proof predicate kind.
pub fn proof_predicate_declaration(kind: ProofPredicateKind) -> DeclarationId {
    let name = match kind {
        ProofPredicateKind::Equivalent => "Equivalent",
        ProofPredicateKind::Subtype => "Subtype",
        ProofPredicateKind::Conforms => "Conforms",
    };
    DeclarationId::new(crate::core_surface::universe_module(&["core", "proof"]), name.into())
}

/// Checks whether a declaration corresponds to a canonical compiler-owned proof predicate.
pub fn is_proof_predicate_declaration(declaration: &DeclarationId) -> Option<ProofPredicateKind> {
    if declaration.module != crate::core_surface::universe_module(&["core", "proof"]) {
        return None;
    }
    match declaration.name.as_ref() {
        "Equivalent" => Some(ProofPredicateKind::Equivalent),
        "Subtype" => Some(ProofPredicateKind::Subtype),
        "Conforms" => Some(ProofPredicateKind::Conforms),
        _ => None,
    }
}
