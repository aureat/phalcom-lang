//! Pure binding-contract reconciliation.

use crate::identity::BindingId;
use crate::types::denotation::SemanticDenotation;
use crate::types::evidence::{ContractAssumptionEligibility, EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::outcome::{BlockReason, BudgetReport, DynamicBoundaryObligation, RelationFailure, RelationOutcome};
use crate::types::relation::{Assignability, RefutationReason, TypeHierarchy, check_knowledge_against_type};
use crate::types::store::TypeStore;
use phalcom_common::range::SourceRange;

/// Explicit declaration input. Current value knowledge never replaces its
/// persistent contract; both are retained in the resulting binding state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingSeed {
    pub name: String,
    pub range: SourceRange,
    pub contract: Option<BindingContract>,
    pub current: TypeKnowledge,
    pub denotation: Option<SemanticDenotation>,
    pub causal_invalidity: crate::checker::causal::CausalInvalidity,
    pub mutable: bool,
}

/// Result of inserting a declaration into its current lexical scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingDeclarationResult {
    Inserted(BindingId),
    Redeclared(BindingId),
}

/// Result of attempting a write against an existing binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingWriteResult {
    Applied,
    Immutable,
    Missing,
}

/// Source of a persistent binding contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingContractOrigin {
    SourceAnnotation,
    InferredInitializer,
    CallableParameter,
    ContextualBlockParameter,
    PatternBinding,
}

/// Persistent type constraint independent of current flow knowledge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingContract {
    pub ty: TypeId,
    pub origin: BindingContractOrigin,
    pub source: Option<SourceRange>,
}

/// Basis for an assumption that remains usable but unestablished.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssumptionBasis {
    MissingValueEvidence(UnknownReason),
    CallableParameterContract,
    ContextualParameterContract,
    DerivedEvidence(EvidenceOrigin),
}

/// Relation between current value knowledge and a persistent binding contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingConsistency {
    Unconstrained,
    Validated,
    Assumed {
        basis: AssumptionBasis,
    },
    Refuted {
        actual: TypeId,
        expected: TypeId,
        reason: RefutationReason,
    },
    DynamicBoundary {
        obligation: DynamicBoundaryObligation,
    },
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}

/// Result of pure contract reconciliation. No diagnostics or state mutation occur here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingReconciliation {
    pub current: TypeKnowledge,
    pub consistency: BindingConsistency,
}

fn assumption_basis(actual: &TypeKnowledge) -> AssumptionBasis {
    match actual.origin() {
        Some(EvidenceOrigin::CallableSignature) => AssumptionBasis::CallableParameterContract,
        Some(EvidenceOrigin::ContextualDerivation) => AssumptionBasis::ContextualParameterContract,
        Some(origin) => AssumptionBasis::DerivedEvidence(origin),
        None => AssumptionBasis::DerivedEvidence(EvidenceOrigin::ContextualDerivation),
    }
}

/// Reconciles current formal knowledge against a persistent binding contract.
pub fn reconcile_binding_contract(
    store: &TypeStore,
    hierarchy: &dyn TypeHierarchy,
    contract: Option<&BindingContract>,
    actual: &TypeKnowledge,
) -> BindingReconciliation {
    let Some(contract) = contract else {
        return BindingReconciliation {
            current: actual.clone(),
            consistency: BindingConsistency::Unconstrained,
        };
    };

    if let TypeKnowledge::Unknown(reason) = actual {
        if matches!(contract.origin, BindingContractOrigin::SourceAnnotation)
            && reason.contract_assumption_eligibility() == ContractAssumptionEligibility::MaySupplyAssumption
        {
            return BindingReconciliation {
                current: TypeKnowledge::assumed(contract.ty, EvidenceOrigin::DeveloperAnnotation),
                consistency: BindingConsistency::Assumed {
                    basis: AssumptionBasis::MissingValueEvidence(reason.clone()),
                },
            };
        }
        return BindingReconciliation {
            current: actual.clone(),
            consistency: BindingConsistency::Blocked(BlockReason::UnknownType(reason.clone())),
        };
    }

    if matches!(actual, TypeKnowledge::Dynamic(_)) {
        return BindingReconciliation {
            current: actual.clone(),
            consistency: BindingConsistency::DynamicBoundary {
                obligation: DynamicBoundaryObligation {
                    reason: "binding contract crosses dynamic boundary".into(),
                },
            },
        };
    }

    let relation = check_knowledge_against_type(store, hierarchy, actual, contract.ty);
    let consistency = match relation {
        Assignability::Assignable => {
            if actual.status() == Some(EvidenceStatus::Established) {
                BindingConsistency::Validated
            } else {
                BindingConsistency::Assumed {
                    basis: assumption_basis(actual),
                }
            }
        }
        Assignability::Refuted { actual, expected, reason } => BindingConsistency::Refuted { actual, expected, reason },
        Assignability::DynamicBoundary(obligation) => BindingConsistency::DynamicBoundary { obligation },
        Assignability::Blocked(reason) => BindingConsistency::Blocked(reason),
        Assignability::Cancelled => BindingConsistency::Cancelled,
        Assignability::BudgetExceeded(report) => BindingConsistency::BudgetExceeded(report),
        Assignability::InternalFailure(message) => BindingConsistency::InternalFailure(message.into_boxed_str()),
        Assignability::Uncertain => BindingConsistency::Blocked(BlockReason::RecursiveFixpoint),
    };

    BindingReconciliation {
        current: actual.clone(),
        consistency,
    }
}

/// Projects a fully structured relation result into binding consistency without
/// relabeling operational terminal outcomes as ordinary blockage.
pub fn reconcile_binding_relation(contract: Option<&BindingContract>, actual: &TypeKnowledge, relation: RelationOutcome) -> BindingReconciliation {
    let Some(contract) = contract else {
        return BindingReconciliation {
            current: actual.clone(),
            consistency: BindingConsistency::Unconstrained,
        };
    };

    if let TypeKnowledge::Unknown(reason) = actual {
        if matches!(contract.origin, BindingContractOrigin::SourceAnnotation)
            && reason.contract_assumption_eligibility() == ContractAssumptionEligibility::MaySupplyAssumption
        {
            return BindingReconciliation {
                current: TypeKnowledge::assumed(contract.ty, EvidenceOrigin::DeveloperAnnotation),
                consistency: BindingConsistency::Assumed {
                    basis: AssumptionBasis::MissingValueEvidence(reason.clone()),
                },
            };
        }
    }

    let consistency = match relation {
        RelationOutcome::Proven { .. } => {
            if actual.status() == Some(EvidenceStatus::Established) {
                BindingConsistency::Validated
            } else {
                BindingConsistency::Assumed {
                    basis: assumption_basis(actual),
                }
            }
        }
        RelationOutcome::Refuted(failure) => {
            let reason = match failure {
                RelationFailure::IncompatibleNominal { .. } => RefutationReason::IncompatibleNominal,
                RelationFailure::UnionMemberMismatch { .. } => RefutationReason::UnionMemberMismatch,
                RelationFailure::TypeMismatch { .. } | RelationFailure::CycleDetected { .. } | RelationFailure::DepthExceeded | RelationFailure::Custom(_) => {
                    RefutationReason::TypeMismatch
                }
            };
            BindingConsistency::Refuted {
                actual: actual.ty().unwrap_or(contract.ty),
                expected: contract.ty,
                reason,
            }
        }
        RelationOutcome::DynamicBoundary(obligation) => BindingConsistency::DynamicBoundary { obligation },
        RelationOutcome::Blocked(reason) => BindingConsistency::Blocked(reason),
        RelationOutcome::Cancelled => BindingConsistency::Cancelled,
        RelationOutcome::BudgetExceeded(report) => BindingConsistency::BudgetExceeded(report),
        RelationOutcome::InternalFailure(message) => BindingConsistency::InternalFailure(message.into_boxed_str()),
    };

    BindingReconciliation {
        current: actual.clone(),
        consistency,
    }
}
