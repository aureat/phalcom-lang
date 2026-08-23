//! Proof IR terms for static verification condition generation.

use crate::identity::BindingId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProofUnaryOp {
    Not,
    Neg,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProofBinaryOp {
    And,
    Or,
    Implies,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ProofOpaqueReason {
    DynamicValue,
    UninterpretedCall,
    UnsupportedSyntax,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProofTerm {
    BoolConst(bool),
    IntConst(i64),
    Var(BindingId),
    OldVar(BindingId),
    Result,
    Unary(ProofUnaryOp, Box<ProofTerm>),
    Binary(ProofBinaryOp, Box<ProofTerm>, Box<ProofTerm>),
    FieldAccess(Box<ProofTerm>, Box<str>),
    Ite(Box<ProofTerm>, Box<ProofTerm>, Box<ProofTerm>),
    Opaque(ProofOpaqueReason),
}

impl ProofTerm {
    pub const TRUE: Self = Self::BoolConst(true);
    pub const FALSE: Self = Self::BoolConst(false);

    pub fn not(self) -> Self {
        Self::Unary(ProofUnaryOp::Not, Box::new(self))
    }

    pub fn and(self, other: Self) -> Self {
        Self::Binary(ProofBinaryOp::And, Box::new(self), Box::new(other))
    }

    pub fn or(self, other: Self) -> Self {
        Self::Binary(ProofBinaryOp::Or, Box::new(self), Box::new(other))
    }

    pub fn implies(self, other: Self) -> Self {
        Self::Binary(ProofBinaryOp::Implies, Box::new(self), Box::new(other))
    }

    pub fn eq(self, other: Self) -> Self {
        Self::Binary(ProofBinaryOp::Eq, Box::new(self), Box::new(other))
    }
}
