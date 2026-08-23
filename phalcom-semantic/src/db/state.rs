//! Query evaluation states, stored values, and outcomes.

use crate::db::budget::BudgetReport;
use crate::db::key::ProductFingerprint;
use crate::identity::SemanticRevision;
use crate::types::outcome::BlockReason;
use std::sync::Arc;

/// Byte-addressable or type-erased value stored for a computed query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryValue {
    pub bytes: Arc<[u8]>,
}

impl QueryValue {
    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Self {
        Self {
            bytes: Arc::from(bytes.as_ref()),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// The recorded state of a query in the database.
#[derive(Clone, Debug, PartialEq)]
pub enum QueryState {
    Vacant,
    Computing {
        revision: SemanticRevision,
        stack_index: u32,
    },
    Ready {
        revision: SemanticRevision,
        fingerprint: ProductFingerprint,
        value: QueryValue,
    },
    BudgetExceeded {
        revision: SemanticRevision,
        report: BudgetReport,
    },
    Blocked {
        revision: SemanticRevision,
        reason: BlockReason,
    },
    Cancelled {
        revision: SemanticRevision,
    },
    Failed {
        revision: SemanticRevision,
        failure: String,
    },
}

impl QueryState {
    pub fn revision(&self) -> Option<SemanticRevision> {
        match self {
            Self::Vacant => None,
            Self::Computing { revision, .. }
            | Self::Ready { revision, .. }
            | Self::BudgetExceeded { revision, .. }
            | Self::Blocked { revision, .. }
            | Self::Cancelled { revision }
            | Self::Failed { revision, .. } => Some(*revision),
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    /// Returns the input/result fingerprint recorded for a ready product.
    ///
    /// Non-ready states deliberately have no reusable fingerprint. A caller
    /// must not treat a cancelled, blocked, or failed query as a cache hit.
    pub fn fingerprint(&self) -> Option<ProductFingerprint> {
        match self {
            Self::Ready { fingerprint, .. } => Some(*fingerprint),
            _ => None,
        }
    }

    pub fn as_ready_value(&self) -> Option<&QueryValue> {
        match self {
            Self::Ready { value, .. } => Some(value),
            _ => None,
        }
    }
}

/// Outcome of evaluating a query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryOutcome<T> {
    Ready(T),
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    Failed(String),
}

impl<T> QueryOutcome<T> {
    pub fn ready(val: T) -> Self {
        Self::Ready(val)
    }

    pub fn cancelled() -> Self {
        Self::Cancelled
    }

    pub fn budget_exceeded(report: BudgetReport) -> Self {
        Self::BudgetExceeded(report)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }

    pub fn is_budget_exceeded(&self) -> bool {
        matches!(self, Self::BudgetExceeded(_))
    }
}

/// Error returned when attempting to publish a query result against a stale revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublishError {
    stale: bool,
    expected_revision: SemanticRevision,
    actual_revision: SemanticRevision,
}

impl PublishError {
    pub const fn stale(expected_revision: SemanticRevision, actual_revision: SemanticRevision) -> Self {
        Self {
            stale: true,
            expected_revision,
            actual_revision,
        }
    }

    pub const fn is_stale(self) -> bool {
        self.stale
    }

    pub const fn expected_revision(self) -> SemanticRevision {
        self.expected_revision
    }

    pub const fn actual_revision(self) -> SemanticRevision {
        self.actual_revision
    }
}
