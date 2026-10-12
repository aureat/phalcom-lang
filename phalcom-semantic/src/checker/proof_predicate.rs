//! Compiler-owned checked products for static proof predicates (LANG007.C4.P6 T9).

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

/// Checked product identifying one proof predicate expression.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofPredicateResolution {
    pub expression: ExpressionId,
    pub range: SourceRange,
    pub kind: ProofPredicateKind,
    pub left: TypeId,
    pub right: TypeId,
    pub result_type: TypeId,
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
