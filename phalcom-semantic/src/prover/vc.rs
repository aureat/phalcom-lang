//! Verification condition model and status.

use super::ir::ProofTerm;
use crate::identity::BindingId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProofObligationKind {
    PreconditionHold,
    PostconditionHold,
    InvariantHold,
    AssertHold,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProofEvidence {
    Tautology,
    DirectSimplification,
    LinearArithmetic,
    ConstEvaluation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Counterexample {
    pub assignments: Vec<(BindingId, ProofTerm)>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum VcUnknownReason {
    ContainsOpaqueTerm,
    NonLinearArithmetic,
    IncompleteSolver,
    BudgetExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VcStatus {
    Proven(ProofEvidence),
    Disproven(Counterexample),
    Unknown(VcUnknownReason),
}

impl VcStatus {
    pub fn is_proven(&self) -> bool {
        matches!(self, Self::Proven(_))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationCondition {
    pub id: u32,
    pub obligation: ProofObligationKind,
    pub antecedent: ProofTerm,
    pub consequent: ProofTerm,
    pub status: VcStatus,
}
