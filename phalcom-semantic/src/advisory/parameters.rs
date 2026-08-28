//! Contribution-indexed advisory parameter facts.

use std::collections::BTreeMap;

use crate::identity::{CallableId, CallableParameterId, ModuleId};

use super::AdvisoryFact;

/// Compatibility name for the canonical callable parameter identity.
///
/// Advisory analysis does not own a parallel parameter namespace: parameter
/// contributions are keyed by the declaration-owned [`CallableParameterId`].
pub type AdvisoryParameterSlot = CallableParameterId;

/// Canonical source of one parameter contribution.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AdvisoryContributionSource {
    Callable(CallableId),
    Module(ModuleId),
}

/// One changed joined parameter fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryParameterFactDelta {
    pub slot: CallableParameterId,
    pub before: Option<AdvisoryFact>,
    pub after: Option<AdvisoryFact>,
}

/// Source-indexed contributions plus cached joined canonical parameters.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AdvisoryParameterContributions {
    by_source: BTreeMap<AdvisoryContributionSource, BTreeMap<CallableParameterId, AdvisoryFact>>,
    joined: BTreeMap<CallableParameterId, AdvisoryFact>,
}

impl AdvisoryParameterContributions {
    /// Replaces one caller/module source and recomputes only touched parameters.
    pub fn replace_source(
        &mut self,
        source: AdvisoryContributionSource,
        contributions: BTreeMap<CallableParameterId, AdvisoryFact>,
    ) -> Vec<AdvisoryParameterFactDelta> {
        let previous = self.by_source.insert(source.clone(), contributions.clone()).unwrap_or_default();
        let mut touched = previous.keys().cloned().collect::<Vec<_>>();
        touched.extend(contributions.keys().cloned());
        touched.sort();
        touched.dedup();
        self.recompute_touched(touched)
    }

    /// Removes one source and recomputes only parameters it contributed.
    pub fn remove_source(&mut self, source: &AdvisoryContributionSource) -> Vec<AdvisoryParameterFactDelta> {
        let Some(previous) = self.by_source.remove(source) else { return Vec::new() };
        self.recompute_touched(previous.into_keys().collect())
    }

    /// Returns the current joined fact for one canonical parameter.
    pub fn get(&self, parameter: &CallableParameterId) -> Option<&AdvisoryFact> {
        self.joined.get(parameter)
    }

    /// Iterates joined facts in canonical parameter order.
    pub fn joined_iter(&self) -> impl Iterator<Item = (&CallableParameterId, &AdvisoryFact)> {
        self.joined.iter()
    }

    fn recompute_touched(&mut self, touched: Vec<CallableParameterId>) -> Vec<AdvisoryParameterFactDelta> {
        let mut deltas = Vec::new();
        for slot in touched {
            let before = self.joined.get(&slot).cloned();
            let after = self
                .by_source
                .values()
                .filter_map(|source| source.get(&slot))
                .cloned()
                .reduce(|left, right| left.join(&right));
            match &after {
                Some(value) => {
                    self.joined.insert(slot.clone(), value.clone());
                }
                None => {
                    self.joined.remove(&slot);
                }
            }
            if before != after {
                deltas.push(AdvisoryParameterFactDelta { slot, before, after });
            }
        }
        deltas
    }
}
