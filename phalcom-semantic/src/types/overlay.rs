//! Lazily materialized persistent maps used by local semantic-store overlays.

use im::HashMap;
use std::fmt;
use std::hash::Hash;

/// A persistent hash map whose empty state does not allocate its HAMT root.
///
/// Artifact-loaded stores start with empty local overlays over frozen canonical
/// prefixes. Keeping that state as `None` lets artifact loading publish those
/// stores without constructing opaque `im::HashMap` nodes. The persistent map
/// is created on the first local insertion, preserving copy-on-write forks.
#[derive(Clone)]
pub(crate) struct PersistentHashMapOverlay<K, V> {
    map: Option<HashMap<K, V>>,
}

impl<K, V> Default for PersistentHashMapOverlay<K, V> {
    fn default() -> Self {
        Self { map: None }
    }
}

impl<K, V> fmt::Debug for PersistentHashMapOverlay<K, V> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PersistentHashMapOverlay")
            .field("state", &if self.map.is_some() { "populated" } else { "empty" })
            .finish()
    }
}

impl<K: Eq + Hash + Clone, V: Clone> PersistentHashMapOverlay<K, V> {
    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        self.map.as_ref()?.get(key)
    }

    pub(crate) fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.map.get_or_insert_with(HashMap::new).insert(key, value)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&K, &V)> + '_ {
        self.map.iter().flat_map(|map| map.iter())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.map.as_ref().map_or(true, |m| m.is_empty())
    }

    #[cfg(test)]
    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        match (&self.map, &other.map) {
            (None, None) => true,
            (Some(left), Some(right)) => left.ptr_eq(right),
            _ => false,
        }
    }
}
