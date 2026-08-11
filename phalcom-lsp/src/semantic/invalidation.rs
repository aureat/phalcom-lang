//! Bounded semantic invalidation primitives.

use std::collections::BTreeSet;

use super::ids::ModuleId;

/// A deterministic set of modules awaiting recomputation.
#[derive(Clone, Debug, Default)]
pub struct InvalidationQueue {
    pending: BTreeSet<ModuleId>,
}

impl InvalidationQueue {
    /// Adds one changed or dependent module.
    pub fn push(&mut self, module: ModuleId) {
        self.pending.insert(module);
    }

    /// Drains queued modules in module-id order.
    pub fn drain(&mut self) -> impl Iterator<Item = ModuleId> + '_ {
        std::mem::take(&mut self.pending).into_iter()
    }

    /// Returns whether no module is pending.
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}
