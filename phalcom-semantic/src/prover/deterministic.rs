//! Backend-free deterministic simplifier and evaluator for verification conditions.

use super::ir::{ProofBinaryOp, ProofTerm, ProofUnaryOp};
use super::vc::{ProofEvidence, VcStatus, VcUnknownReason, VerificationCondition};

/// Recursively simplifies a ProofTerm using deterministic algebraic identities.
pub fn simplify_proof_term(term: &ProofTerm) -> ProofTerm {
    match term {
        ProofTerm::Unary(ProofUnaryOp::Not, inner) => {
            let simplified_inner = simplify_proof_term(inner);
            match simplified_inner {
                ProofTerm::BoolConst(b) => ProofTerm::BoolConst(!b),
                ProofTerm::Unary(ProofUnaryOp::Not, double_inner) => *double_inner,
                other => ProofTerm::Unary(ProofUnaryOp::Not, Box::new(other)),
            }
        }
        ProofTerm::Unary(ProofUnaryOp::Neg, inner) => {
            let simplified_inner = simplify_proof_term(inner);
            match simplified_inner {
                ProofTerm::IntConst(n) => ProofTerm::IntConst(-n),
                other => ProofTerm::Unary(ProofUnaryOp::Neg, Box::new(other)),
            }
        }
        ProofTerm::Binary(op, left, right) => {
            let s_left = simplify_proof_term(left);
            let s_right = simplify_proof_term(right);

            match op {
                ProofBinaryOp::And => match (&s_left, &s_right) {
                    (ProofTerm::BoolConst(false), _) | (_, ProofTerm::BoolConst(false)) => ProofTerm::BoolConst(false),
                    (ProofTerm::BoolConst(true), other) | (other, ProofTerm::BoolConst(true)) => other.clone(),
                    (l, r) if l == r => l.clone(),
                    _ => ProofTerm::Binary(ProofBinaryOp::And, Box::new(s_left), Box::new(s_right)),
                },
                ProofBinaryOp::Or => match (&s_left, &s_right) {
                    (ProofTerm::BoolConst(true), _) | (_, ProofTerm::BoolConst(true)) => ProofTerm::BoolConst(true),
                    (ProofTerm::BoolConst(false), other) | (other, ProofTerm::BoolConst(false)) => other.clone(),
                    (l, r) if l == r => l.clone(),
                    _ => ProofTerm::Binary(ProofBinaryOp::Or, Box::new(s_left), Box::new(s_right)),
                },
                ProofBinaryOp::Implies => match (&s_left, &s_right) {
                    (ProofTerm::BoolConst(false), _) => ProofTerm::BoolConst(true),
                    (_, ProofTerm::BoolConst(true)) => ProofTerm::BoolConst(true),
                    (ProofTerm::BoolConst(true), other) => other.clone(),
                    (l, r) if l == r => ProofTerm::BoolConst(true),
                    _ => ProofTerm::Binary(ProofBinaryOp::Implies, Box::new(s_left), Box::new(s_right)),
                },
                ProofBinaryOp::Eq => match (&s_left, &s_right) {
                    (l, r) if l == r => ProofTerm::BoolConst(true),
                    (ProofTerm::BoolConst(a), ProofTerm::BoolConst(b)) => ProofTerm::BoolConst(a == b),
                    (ProofTerm::IntConst(a), ProofTerm::IntConst(b)) => ProofTerm::BoolConst(a == b),
                    _ => ProofTerm::Binary(ProofBinaryOp::Eq, Box::new(s_left), Box::new(s_right)),
                },
                ProofBinaryOp::Ne => match (&s_left, &s_right) {
                    (l, r) if l == r => ProofTerm::BoolConst(false),
                    (ProofTerm::BoolConst(a), ProofTerm::BoolConst(b)) => ProofTerm::BoolConst(a != b),
                    (ProofTerm::IntConst(a), ProofTerm::IntConst(b)) => ProofTerm::BoolConst(a != b),
                    _ => ProofTerm::Binary(ProofBinaryOp::Ne, Box::new(s_left), Box::new(s_right)),
                },
                ProofBinaryOp::Lt => match (&s_left, &s_right) {
                    (ProofTerm::IntConst(a), ProofTerm::IntConst(b)) => ProofTerm::BoolConst(a < b),
                    (l, r) if l == r => ProofTerm::BoolConst(false),
                    _ => ProofTerm::Binary(ProofBinaryOp::Lt, Box::new(s_left), Box::new(s_right)),
                },
                ProofBinaryOp::Le => match (&s_left, &s_right) {
                    (ProofTerm::IntConst(a), ProofTerm::IntConst(b)) => ProofTerm::BoolConst(a <= b),
                    (l, r) if l == r => ProofTerm::BoolConst(true),
                    _ => ProofTerm::Binary(ProofBinaryOp::Le, Box::new(s_left), Box::new(s_right)),
                },
                ProofBinaryOp::Gt => match (&s_left, &s_right) {
                    (ProofTerm::IntConst(a), ProofTerm::IntConst(b)) => ProofTerm::BoolConst(a > b),
                    (l, r) if l == r => ProofTerm::BoolConst(false),
                    _ => ProofTerm::Binary(ProofBinaryOp::Gt, Box::new(s_left), Box::new(s_right)),
                },
                ProofBinaryOp::Ge => match (&s_left, &s_right) {
                    (ProofTerm::IntConst(a), ProofTerm::IntConst(b)) => ProofTerm::BoolConst(a >= b),
                    (l, r) if l == r => ProofTerm::BoolConst(true),
                    _ => ProofTerm::Binary(ProofBinaryOp::Ge, Box::new(s_left), Box::new(s_right)),
                },
                ProofBinaryOp::Add => match (&s_left, &s_right) {
                    (ProofTerm::IntConst(a), ProofTerm::IntConst(b)) => ProofTerm::IntConst(a.wrapping_add(*b)),
                    (other, ProofTerm::IntConst(0)) | (ProofTerm::IntConst(0), other) => other.clone(),
                    _ => ProofTerm::Binary(ProofBinaryOp::Add, Box::new(s_left), Box::new(s_right)),
                },
                ProofBinaryOp::Sub => match (&s_left, &s_right) {
                    (ProofTerm::IntConst(a), ProofTerm::IntConst(b)) => ProofTerm::IntConst(a.wrapping_sub(*b)),
                    (other, ProofTerm::IntConst(0)) => other.clone(),
                    (l, r) if l == r => ProofTerm::IntConst(0),
                    _ => ProofTerm::Binary(ProofBinaryOp::Sub, Box::new(s_left), Box::new(s_right)),
                },
                ProofBinaryOp::Mul => match (&s_left, &s_right) {
                    (ProofTerm::IntConst(a), ProofTerm::IntConst(b)) => ProofTerm::IntConst(a.wrapping_mul(*b)),
                    (other, ProofTerm::IntConst(1)) | (ProofTerm::IntConst(1), other) => other.clone(),
                    (_, ProofTerm::IntConst(0)) | (ProofTerm::IntConst(0), _) => ProofTerm::IntConst(0),
                    _ => ProofTerm::Binary(ProofBinaryOp::Mul, Box::new(s_left), Box::new(s_right)),
                },
            }
        }
        ProofTerm::Ite(cond, then_b, else_b) => {
            let s_cond = simplify_proof_term(cond);
            match s_cond {
                ProofTerm::BoolConst(true) => simplify_proof_term(then_b),
                ProofTerm::BoolConst(false) => simplify_proof_term(else_b),
                other => ProofTerm::Ite(Box::new(other), Box::new(simplify_proof_term(then_b)), Box::new(simplify_proof_term(else_b))),
            }
        }
        term => term.clone(),
    }
}

/// Solves a verification condition using deterministic backend-free methods.
pub fn solve_vc_deterministic(vc: &mut VerificationCondition) {
    let full_implication = ProofTerm::Binary(ProofBinaryOp::Implies, Box::new(vc.antecedent.clone()), Box::new(vc.consequent.clone()));

    let simplified = simplify_proof_term(&full_implication);

    match simplified {
        ProofTerm::BoolConst(true) => {
            vc.status = VcStatus::Proven(ProofEvidence::DirectSimplification);
        }
        ProofTerm::BoolConst(false) => {
            vc.status = VcStatus::Disproven(super::vc::Counterexample { assignments: Vec::new() });
        }
        ProofTerm::Opaque(_) => {
            vc.status = VcStatus::Unknown(VcUnknownReason::ContainsOpaqueTerm);
        }
        _ => {
            vc.status = VcStatus::Unknown(VcUnknownReason::IncompleteSolver);
        }
    }
}
