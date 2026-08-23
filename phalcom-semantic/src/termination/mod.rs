//! Termination and divergence reasoning (Spec 05).

pub mod analysis;
pub mod cfg;
pub mod ranking;

pub use analysis::analyze_callable_termination;
pub use cfg::check_cfg_acyclicity;
pub use ranking::RankingMeasure;

use crate::identity::BindingId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminationKnowledge {
    Proven(TerminationEvidence),
    Refuted(TerminationCounterevidence),
    Blocked(TerminationBlockedReason),
    Opaque(TerminationBlockedReason),
}

impl TerminationKnowledge {
    pub fn is_proven(&self) -> bool {
        matches!(self, Self::Proven(_))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminationEvidence {
    AcyclicCfg,
    StructuralDecreasing(BindingId),
    IntegerDecreasing(BindingId),
    CollectionDecreasing(BindingId),
    TrustedNative,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TerminationCounterevidence {
    UnboundedSelfCall,
    NonProgressLoop,
    InfiniteCoinductive,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TerminationBlockedReason {
    UnsupportedRecursionPattern,
    DynamicCallee,
    OpaqueNative,
    BudgetExhausted,
    UnprovenLoop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminationRequirement {
    ExplicitTotalAnnotation,
    ProofContext,
    None,
}
