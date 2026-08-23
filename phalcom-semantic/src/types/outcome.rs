//! Explicit bounded relation outcomes, terminal states, and budgets.

use crate::diagnostic::DiagnosticCode;
use crate::identity::{DeclarationId, StableModuleKey};
use crate::types::evidence::UnknownReason;
use crate::types::id::TypeId;
use std::fmt;

/// Distinction of budget dimensions.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BudgetKind {
    Steps,
    RelationPairs,
    SccIterations,
    TypeDepth,
    DiagnosticNotes,
}

impl fmt::Display for BudgetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Steps => write!(f, "steps"),
            Self::RelationPairs => write!(f, "relation_pairs"),
            Self::SccIterations => write!(f, "scc_iterations"),
            Self::TypeDepth => write!(f, "type_depth"),
            Self::DiagnosticNotes => write!(f, "diagnostic_notes"),
        }
    }
}

/// A report indicating budget exhaustion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetReport {
    pub kind: BudgetKind,
    pub limit: u64,
    pub used: u64,
}

impl BudgetReport {
    pub const fn new(kind: BudgetKind, limit: u64, used: u64) -> Self {
        Self { kind, limit, used }
    }

    pub const fn kind(&self) -> BudgetKind {
        self.kind
    }

    pub const fn limit(&self) -> u64 {
        self.limit
    }

    pub const fn used(&self) -> u64 {
        self.used
    }
}

/// Budget configuration for queries and relation checking.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryBudget {
    pub max_steps: u64,
    pub max_relation_pairs: u32,
    pub max_scc_iterations: u32,
    pub max_type_depth: u16,
    pub max_diagnostic_notes: u16,

    pub steps_taken: u64,
    pub pairs_checked: u32,
}

/// Error returned when a cancellation token observes cancellation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CancellationError;

impl Default for QueryBudget {
    fn default() -> Self {
        Self {
            max_steps: 10_000,
            max_relation_pairs: 1_000,
            max_scc_iterations: 100,
            max_type_depth: 128,
            max_diagnostic_notes: 32,
            steps_taken: 0,
            pairs_checked: 0,
        }
    }
}

impl QueryBudget {
    pub const fn new(max_steps: u64) -> Self {
        Self {
            max_steps,
            max_relation_pairs: 1_000,
            max_scc_iterations: 100,
            max_type_depth: 128,
            max_diagnostic_notes: 32,
            steps_taken: 0,
            pairs_checked: 0,
        }
    }

    pub fn charge_step(&mut self) -> Result<(), BudgetReport> {
        self.steps_taken += 1;
        if self.steps_taken > self.max_steps {
            Err(BudgetReport::new(BudgetKind::Steps, self.max_steps, self.steps_taken))
        } else {
            Ok(())
        }
    }

    pub fn charge_pair(&mut self) -> Result<(), BudgetReport> {
        self.pairs_checked += 1;
        if self.pairs_checked > self.max_relation_pairs {
            Err(BudgetReport::new(
                BudgetKind::RelationPairs,
                self.max_relation_pairs as u64,
                self.pairs_checked as u64,
            ))
        } else {
            Ok(())
        }
    }
}

/// Cooperative cancellation token.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn check(&self) -> Result<(), CancellationError> {
        if self.is_cancelled() { Err(CancellationError) } else { Ok(()) }
    }
}

/// Reason why a relation or query is blocked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlockReason {
    UnknownType(UnknownReason),
    UnresolvedDependency(StableModuleKey),
    InvalidAnnotation(DiagnosticCode),
    RecursiveFixpoint,
    OpaqueNative(Box<str>),
    ReflectionBoundary,
    BudgetExceeded(BudgetReport),
    SuppressedDependency,
}

/// Evidence supporting a proven relation judgment.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RelationEvidence {
    pub notes: Vec<String>,
}

/// Cause refuting a relation judgment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationFailure {
    TypeMismatch { actual: TypeId, expected: TypeId },
    IncompatibleNominal { actual: DeclarationId, expected: DeclarationId },
    UnionMemberMismatch { actual: TypeId, expected: TypeId },
    CycleDetected { sub: TypeId, sup: TypeId },
    DepthExceeded,
    Custom(String),
}

/// Obligation attached to a dynamic boundary outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DynamicBoundaryObligation {
    pub reason: String,
}

/// Explicit bounded relation outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationOutcome<T = ()> {
    Proven { value: T, evidence: RelationEvidence },
    Refuted(RelationFailure),
    DynamicBoundary(DynamicBoundaryObligation),
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(String),
}

impl<T> RelationOutcome<T> {
    pub fn is_proven(&self) -> bool {
        matches!(self, Self::Proven { .. })
    }

    pub fn is_refuted(&self) -> bool {
        matches!(self, Self::Refuted(_))
    }

    pub fn is_dynamic_boundary(&self) -> bool {
        matches!(self, Self::DynamicBoundary(_))
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Blocked(_))
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }

    pub fn is_budget_exceeded(&self) -> bool {
        matches!(self, Self::BudgetExceeded(_))
    }

    pub fn is_internal_failure(&self) -> bool {
        matches!(self, Self::InternalFailure(_))
    }

    pub fn proven(value: T) -> Self {
        Self::Proven {
            value,
            evidence: RelationEvidence::default(),
        }
    }
}
