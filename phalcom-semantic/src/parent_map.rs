//! Immutable canonical maps with local-only mutable workspace contributions.

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::sync::Arc;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError, BudgetedSortedMap, SortedMapEntriesError};

/// A deterministic map whose canonical parent is shared without copying entries.
/// Mutations affect only local entries; canonical identities cannot be shadowed.
#[derive(Clone, Debug)]
pub struct ParentMap<K, V> {
    parent: Option<Arc<Self>>,
    frozen: Option<Arc<BudgetedSortedMap<K, V>>>,
    local: BTreeMap<K, V>,
}

impl<K, V> Default for ParentMap<K, V> {
    fn default() -> Self {
        Self {
            parent: None,
            frozen: None,
            local: BTreeMap::new(),
        }
    }
}

impl<K: Ord, V> ParentMap<K, V> {
    /// Attaches an immutable parent in constant time.
    pub fn with_parent(parent: Arc<Self>) -> Self {
        Self {
            parent: Some(parent),
            frozen: None,
            local: BTreeMap::new(),
        }
    }

    /// Publishes immutable sorted authority from an already budgeted entry buffer.
    pub fn from_frozen_entries_with_allocation_budget(
        mut entries: Vec<(K, V)>,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, FrozenParentMapError> {
        entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        let frozen = BudgetedSortedMap::from_sorted_entries(entries).map_err(|error| match error {
            SortedMapEntriesError::DuplicateKey => FrozenParentMapError::DuplicateKey,
            SortedMapEntriesError::OutOfOrder => unreachable!("entries were sorted before frozen construction"),
        })?;
        let frozen = allocations.arc(frozen).map_err(FrozenParentMapError::Allocation)?;
        Ok(Self {
            frozen: Some(frozen),
            ..Self::default()
        })
    }

    /// Returns the local contribution, then consults the canonical parent.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.local
            .get(key)
            .or_else(|| self.frozen.as_ref().and_then(|frozen| frozen.get(key)))
            .or_else(|| self.parent.as_ref().and_then(|parent| parent.get(key)))
    }

    /// Returns a mutable local contribution; parent entries are never copied.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.local.get_mut(key)
    }

    /// Tests membership across the immutable parent and local contributions.
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Inserts a local entry, rejecting collisions with canonical identities.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        assert!(
            !self.parent.as_ref().is_some_and(|parent| parent.contains_key(&key))
                && !self.frozen.as_ref().is_some_and(|frozen| frozen.contains_key(&key)),
            "cannot overwrite a canonical map entry"
        );
        self.local.insert(key, value)
    }

    /// Removes only a local entry.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.local.remove(key)
    }

    /// Filters only local contributions, preserving the canonical parent.
    pub fn retain(&mut self, mut predicate: impl FnMut(&K, &mut V) -> bool) {
        self.local.retain(|key, value| predicate(key, value));
    }

    /// Iterates parent entries followed by local entries in deterministic order.
    pub fn iter(&self) -> Box<dyn Iterator<Item = (&K, &V)> + '_> {
        let local = self.frozen.iter().flat_map(|frozen| frozen.iter()).chain(self.local.iter());
        match &self.parent {
            Some(parent) => Box::new(parent.iter().chain(local)),
            None => Box::new(local),
        }
    }

    /// Iterates only contributions owned by this workspace.
    pub fn iter_local(&self) -> impl Iterator<Item = (&K, &V)> {
        self.frozen.iter().flat_map(|frozen| frozen.iter()).chain(self.local.iter())
    }

    /// Iterates only this table's immutable frozen contribution.
    pub fn iter_frozen(&self) -> impl Iterator<Item = (&K, &V)> {
        self.frozen.iter().flat_map(|frozen| frozen.iter())
    }

    /// Iterates only this table's mutable source-built contribution.
    pub fn iter_source_local(&self) -> impl Iterator<Item = (&K, &V)> {
        self.local.iter()
    }

    pub fn frozen_len(&self) -> usize {
        self.frozen.as_ref().map_or(0, |entries| entries.len())
    }

    /// Mutably iterates only contributions owned by this workspace.
    pub fn iter_local_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.local.iter_mut()
    }

    /// Iterates only the immutable canonical parent, without local entries.
    pub fn iter_parent(&self) -> Box<dyn Iterator<Item = (&K, &V)> + '_> {
        match &self.parent {
            Some(parent) => parent.iter(),
            None => Box::new(std::iter::empty()),
        }
    }

    /// Iterates all visible keys.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(key, _)| key)
    }

    /// Iterates all visible values.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_, value)| value)
    }

    /// Counts only local contributions.
    pub fn local_len(&self) -> usize {
        self.local.len() + self.frozen.as_ref().map_or(0, |frozen| frozen.len())
    }

    /// Counts only immutable parent entries.
    pub fn parent_len(&self) -> usize {
        self.parent.as_ref().map_or(0, |parent| parent.len())
    }

    /// Counts parent and local contributions without scanning their entries.
    pub fn len(&self) -> usize {
        self.local_len() + self.parent_len()
    }

    /// Tests whether both parent and local contributions are empty.
    pub fn is_empty(&self) -> bool {
        self.local_len() == 0 && self.parent.as_ref().is_none_or(|parent| parent.is_empty())
    }
}

impl<K: Ord + Eq, V: Eq> PartialEq for ParentMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().all(|(key, value)| other.get(key) == Some(value))
    }
}

impl<K: Ord + Eq, V: Eq> Eq for ParentMap<K, V> {}

#[derive(Debug)]
pub enum FrozenParentMapError {
    Allocation(AllocationError),
    DuplicateKey,
}

impl<K: Ord, V> std::ops::Index<&K> for ParentMap<K, V> {
    type Output = V;

    fn index(&self, key: &K) -> &V {
        self.get(key).expect("map key is present")
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for ParentMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self {
            parent: None,
            frozen: None,
            local: iter.into_iter().collect(),
        }
    }
}

impl<K: Ord + Hash + Eq + Clone, V: Clone> From<&HashMap<K, V>> for ParentMap<K, V> {
    fn from(map: &HashMap<K, V>) -> Self {
        map.iter().map(|(key, value)| (key.clone(), value.clone())).collect()
    }
}

impl<'a, K: Ord, V> IntoIterator for &'a ParentMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_entries_survive_local_mutation_and_overlay_cloning() {
        let parent = Arc::new([(1, "canonical")].into_iter().collect::<ParentMap<_, _>>());
        let mut overlay = ParentMap::with_parent(parent.clone());
        assert!(Arc::ptr_eq(overlay.parent.as_ref().unwrap(), &parent));
        assert_eq!(overlay.local_len(), 0);
        assert_eq!(overlay.parent_len(), 1);
        assert_eq!(overlay.iter_parent().collect::<Vec<_>>(), vec![(&1, &"canonical")]);
        assert_eq!(overlay.get(&1), Some(&"canonical"));
        assert_eq!(overlay.get_mut(&1), None);
        assert_eq!(overlay.remove(&1), None);
        overlay.insert(2, "local");
        let published = overlay.clone();
        overlay.retain(|_, _| false);
        assert_eq!(overlay.get(&1), Some(&"canonical"));
        assert_eq!(published.iter().collect::<Vec<_>>(), vec![(&1, &"canonical"), (&2, &"local")]);
        assert_eq!(parent.len(), 1);
        assert_eq!(overlay.len(), 1);
        assert_eq!(overlay.parent_len(), 1);
    }

    #[test]
    #[should_panic(expected = "cannot overwrite a canonical map entry")]
    fn canonical_identity_cannot_be_shadowed() {
        let parent = Arc::new([(1, "canonical")].into_iter().collect::<ParentMap<_, _>>());
        ParentMap::with_parent(parent).insert(1, "local");
    }
}
