//! Contribution-indexed advisory parameter facts.

use std::collections::BTreeMap;

use crate::identity::{CallableId, ModuleId};

use super::AdvisoryFact;

/// Canonical callable parameter slot. Names are presentation metadata, not
/// contribution identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AdvisoryParameterSlot {
    pub callable: CallableId,
    pub index: u32,
}

impl AdvisoryParameterSlot {
    pub fn new(callable: CallableId, index: u32) -> Self {
        Self { callable, index }
    }
}

/// Canonical source of one parameter contribution.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AdvisoryContributionSource {
    Callable(CallableId),
    Module(ModuleId),
}

/// One changed joined parameter fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryParameterFactDelta {
    pub slot: AdvisoryParameterSlot,
    pub before: Option<AdvisoryFact>,
    pub after: Option<AdvisoryFact>,
}

/// Source-indexed contributions plus cached joined slots.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AdvisoryParameterContributions {
    by_source: BTreeMap<AdvisoryContributionSource, BTreeMap<AdvisoryParameterSlot, AdvisoryFact>>,
    joined: BTreeMap<AdvisoryParameterSlot, AdvisoryFact>,
}

impl AdvisoryParameterContributions {
    /// Replaces one caller/module source and recomputes only touched slots.
    pub fn replace_source(
        &mut self,
        source: AdvisoryContributionSource,
        contributions: BTreeMap<AdvisoryParameterSlot, AdvisoryFact>,
    ) -> Vec<AdvisoryParameterFactDelta> {
        let previous = self.by_source.insert(source.clone(), contributions.clone()).unwrap_or_default();
        let mut touched = previous.keys().cloned().collect::<Vec<_>>();
        touched.extend(contributions.keys().cloned());
        touched.sort();
        touched.dedup();
        self.recompute_touched(touched)
    }

    /// Removes one source and recomputes only slots it contributed.
    pub fn remove_source(&mut self, source: &AdvisoryContributionSource) -> Vec<AdvisoryParameterFactDelta> {
        let Some(previous) = self.by_source.remove(source) else { return Vec::new() };
        self.recompute_touched(previous.into_keys().collect())
    }

    /// Returns the current joined fact for one slot.
    pub fn get(&self, slot: &AdvisoryParameterSlot) -> Option<&AdvisoryFact> {
        self.joined.get(slot)
    }

    /// Iterates joined facts in canonical slot order.
    pub fn joined_iter(&self) -> impl Iterator<Item = (&AdvisoryParameterSlot, &AdvisoryFact)> {
        self.joined.iter()
    }

    fn recompute_touched(&mut self, touched: Vec<AdvisoryParameterSlot>) -> Vec<AdvisoryParameterFactDelta> {
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
