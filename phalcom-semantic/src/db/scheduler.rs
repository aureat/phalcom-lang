//! Deterministic query execution scheduler.

use crate::db::key::QueryKey;
use std::collections::BTreeSet;

/// Deterministic worklist scheduler ordering queries by canonical priority.
#[derive(Clone, Debug, Default)]
pub struct QueryScheduler {
    queue: BTreeSet<QueryKey>,
}

impl QueryScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue(&mut self, key: QueryKey) {
        self.queue.insert(key);
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn pop_next(&mut self) -> Option<QueryKey> {
        let next = self.queue.iter().next().cloned()?;
        self.queue.remove(&next);
        Some(next)
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }
}
