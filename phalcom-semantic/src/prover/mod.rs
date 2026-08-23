//! Static prover engine and verification condition solver (Spec 05).

pub mod deterministic;
pub mod ir;
pub mod vc;

pub use deterministic::{simplify_proof_term, solve_vc_deterministic};
pub use ir::{ProofBinaryOp, ProofOpaqueReason, ProofTerm, ProofUnaryOp};
pub use vc::{Counterexample, ProofEvidence, ProofObligationKind, VcStatus, VcUnknownReason, VerificationCondition};
