//! Deterministic advisory callable summary products.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::db::ProductFingerprint;
use crate::identity::CallableId;

use super::{AdvisoryFact, AdvisoryParameterSlot};

/// Explicit outcome for advisory publication. It never becomes formal status.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum AdvisoryProductStatus {
    Complete,
    Partial,
    Unknown,
    Blocked,
    Cancelled,
    BudgetExceeded,
    InternalFailure(Box<str>),
}

/// Minimal effect placeholder until advisory effect aggregation is migrated.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AdvisorySummaryEffects {
    pub unknown: bool,
}

/// Immutable callable-level advisory summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryCallableSummary {
    pub callable: CallableId,
    pub parameters: Arc<[(AdvisoryParameterSlot, AdvisoryFact)]>,
    pub return_fact: AdvisoryFact,
    pub dependencies: Arc<[CallableId]>,
    pub effects: AdvisorySummaryEffects,
    pub status: AdvisoryProductStatus,
    pub fingerprint: ProductFingerprint,
}

impl AdvisoryCallableSummary {
    /// Constructs a summary and fingerprints semantic fields in canonical order.
    pub fn new(
        callable: CallableId,
        mut parameters: Vec<(AdvisoryParameterSlot, AdvisoryFact)>,
        return_fact: AdvisoryFact,
        mut dependencies: Vec<CallableId>,
        effects: AdvisorySummaryEffects,
        status: AdvisoryProductStatus,
    ) -> Self {
        parameters.sort_by(|left, right| left.0.cmp(&right.0));
        dependencies.sort();
        dependencies.dedup();
        let fingerprint = fingerprint(&callable, &parameters, &return_fact, &dependencies, &effects, &status);
        Self {
            callable,
            parameters: Arc::from(parameters.into_boxed_slice()),
            return_fact,
            dependencies: Arc::from(dependencies.into_boxed_slice()),
            effects,
            status,
            fingerprint,
        }
    }
}

fn fingerprint(
    callable: &CallableId,
    parameters: &[(AdvisoryParameterSlot, AdvisoryFact)],
    return_fact: &AdvisoryFact,
    dependencies: &[CallableId],
    effects: &AdvisorySummaryEffects,
    status: &AdvisoryProductStatus,
) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    callable.hash(&mut hasher);
    for (slot, fact) in parameters {
        slot.hash(&mut hasher);
        fact.fingerprint().raw().hash(&mut hasher);
    }
    return_fact.fingerprint().raw().hash(&mut hasher);
    dependencies.hash(&mut hasher);
    effects.unknown.hash(&mut hasher);
    status.hash(&mut hasher);
    ProductFingerprint::new(hasher.finish())
}
