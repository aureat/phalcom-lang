use phalcom_semantic::contracts::{ConditionKind, ContractCondition, ContractSpec};
use phalcom_semantic::identity::{BindingId, BodyId, ExpressionId, LocalExpressionId};
use phalcom_semantic::prover::deterministic::{simplify_proof_term, solve_vc_deterministic};
use phalcom_semantic::prover::ir::{ProofBinaryOp, ProofOpaqueReason, ProofTerm, ProofUnaryOp};
use phalcom_semantic::prover::vc::{ProofEvidence, ProofObligationKind, VcStatus, VcUnknownReason, VerificationCondition};

#[test]
fn test_contracts_spec_data_model() {
    let eid = ExpressionId::new(BodyId(1), LocalExpressionId(0));
    let mut spec = ContractSpec::default();
    assert!(spec.is_empty());

    spec.preconditions.push(ContractCondition {
        expression: eid,
        kind: ConditionKind::Requires,
        label: Some("non_negative".into()),
    });
    assert!(!spec.is_empty());
}

#[test]
fn test_proof_term_simplification_identities() {
    // !!P -> P
    let p = ProofTerm::Var(BindingId(1));
    let not_not_p = ProofTerm::Unary(ProofUnaryOp::Not, Box::new(ProofTerm::Unary(ProofUnaryOp::Not, Box::new(p.clone()))));
    assert_eq!(simplify_proof_term(&not_not_p), p);

    // true && P -> P
    let true_and_p = ProofTerm::TRUE.and(p.clone());
    assert_eq!(simplify_proof_term(&true_and_p), p);

    // false && P -> false
    let false_and_p = ProofTerm::FALSE.and(p.clone());
    assert_eq!(simplify_proof_term(&false_and_p), ProofTerm::FALSE);

    // true || P -> true
    let true_or_p = ProofTerm::TRUE.or(p.clone());
    assert_eq!(simplify_proof_term(&true_or_p), ProofTerm::TRUE);

    // P ==> P -> true
    let p_implies_p = p.clone().implies(p.clone());
    assert_eq!(simplify_proof_term(&p_implies_p), ProofTerm::TRUE);

    // 5 > 3 -> true
    let five_gt_three = ProofTerm::Binary(ProofBinaryOp::Gt, Box::new(ProofTerm::IntConst(5)), Box::new(ProofTerm::IntConst(3)));
    assert_eq!(simplify_proof_term(&five_gt_three), ProofTerm::TRUE);
}

#[test]
fn test_solve_vc_tautology_and_identities() {
    let p = ProofTerm::Var(BindingId(1));
    let mut vc = VerificationCondition {
        id: 1,
        obligation: ProofObligationKind::PostconditionHold,
        antecedent: p.clone(),
        consequent: p.clone(),
        status: VcStatus::Unknown(VcUnknownReason::IncompleteSolver),
    };

    solve_vc_deterministic(&mut vc);
    assert_eq!(vc.status, VcStatus::Proven(ProofEvidence::DirectSimplification));
    assert!(vc.status.is_proven());
}

#[test]
fn test_solve_vc_disproven() {
    let mut vc = VerificationCondition {
        id: 2,
        obligation: ProofObligationKind::PreconditionHold,
        antecedent: ProofTerm::TRUE,
        consequent: ProofTerm::FALSE,
        status: VcStatus::Unknown(VcUnknownReason::IncompleteSolver),
    };

    solve_vc_deterministic(&mut vc);
    assert!(matches!(vc.status, VcStatus::Disproven(_)));
    assert!(!vc.status.is_proven());
}

#[test]
fn test_solve_vc_opaque_unknown() {
    let opaque_term = ProofTerm::Opaque(ProofOpaqueReason::DynamicValue);
    let mut vc = VerificationCondition {
        id: 3,
        obligation: ProofObligationKind::InvariantHold,
        antecedent: ProofTerm::TRUE,
        consequent: opaque_term,
        status: VcStatus::Unknown(VcUnknownReason::IncompleteSolver),
    };

    solve_vc_deterministic(&mut vc);
    assert_eq!(vc.status, VcStatus::Unknown(VcUnknownReason::ContainsOpaqueTerm));
    assert!(!vc.status.is_proven());
}
