//! Canonical algebraic declaration semantic products.

use crate::data_semantics::DataInfo;
use crate::enum_semantics::{EnumInfo, VariantInfo};
use crate::identity::{DeclarationId, SemanticSourceSpan, VariantId};
use crate::types::parameter::GenericSignature;
use phalcom_ast::ast::{AlgebraicPhase, ImplementationKind};
use std::sync::Arc;

pub type AlgebraicCaseId = VariantId;
pub type AlgebraicCaseInfo = VariantInfo;

/// The specific representation model of an algebraic declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlgebraicRepresentation {
    Product(Arc<DataInfo>),
    Sum(Arc<AlgebraicSumInfo>),
}

/// Structural semantic metadata for an algebraic sum declaration and its cases.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlgebraicSumInfo {
    pub enum_info: Arc<EnumInfo>,
    pub variants: Arc<[VariantInfo]>,
}

/// Complete structural semantic metadata for one declared algebraic root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlgebraicDeclarationInfo {
    pub owner: DeclarationId,
    pub phase: AlgebraicPhase,
    pub implementation: ImplementationKind,
    pub generic_signature: Option<GenericSignature>,
    pub representation: AlgebraicRepresentation,
    pub source: Option<SemanticSourceSpan>,
}

impl AlgebraicDeclarationInfo {
    pub fn is_product(&self) -> bool {
        matches!(self.representation, AlgebraicRepresentation::Product(_))
    }

    pub fn is_sum(&self) -> bool {
        matches!(self.representation, AlgebraicRepresentation::Sum(_))
    }

    pub fn product_info(&self) -> Option<&Arc<DataInfo>> {
        match &self.representation {
            AlgebraicRepresentation::Product(p) => Some(p),
            AlgebraicRepresentation::Sum(_) => None,
        }
    }

    pub fn sum_info(&self) -> Option<&Arc<AlgebraicSumInfo>> {
        match &self.representation {
            AlgebraicRepresentation::Sum(s) => Some(s),
            AlgebraicRepresentation::Product(_) => None,
        }
    }
}
