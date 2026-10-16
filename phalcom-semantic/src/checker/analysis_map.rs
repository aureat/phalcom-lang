//! Source-mutable and artifact-frozen ordered maps for callable analysis.

use phalcom_type_meta::allocation::{AllocationBudget, AllocationError, BudgetedSortedMap, SortedMapEntriesError};
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::ops::Index;
use std::sync::Arc;

/// Immutable-map construction failure for a callable analysis owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisMapBuildError {
    /// The already-created entry buffer was not strictly ordered.
    Entries(SortedMapEntriesError),
    /// Publishing the shared immutable backing exceeded the allocation budget.
    Allocation(AllocationError),
}

/// A failed attempt to mutate an imported immutable analysis map.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrozenAnalysisMapMutation;

#[derive(Clone, Debug)]
enum AnalysisMapStorage<K, V> {
    Source(BTreeMap<K, V>),
    Frozen(Arc<BudgetedSortedMap<K, V>>),
}

/// Ordered analysis map with mutable source storage and immutable, charged
/// artifact storage behind one read interface.
///
/// Source checker products use the ordinary tree map while they are being
/// assembled. Imported canonical products use a shared [`BudgetedSortedMap`]
/// whose backing buffer was charged by the caller's [`AllocationBudget`].
/// Reads have the same ordering and lookup behavior for both representations.
#[derive(Clone, Debug)]
pub struct AnalysisMap<K, V> {
    storage: AnalysisMapStorage<K, V>,
}

impl<K: Ord, V: PartialEq> PartialEq for AnalysisMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<K: Ord, V: Eq> Eq for AnalysisMap<K, V> {}

impl<K: Ord, V> Default for AnalysisMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V> AnalysisMap<K, V> {
    /// Creates an empty source-side map.
    pub fn new() -> Self {
        Self {
            storage: AnalysisMapStorage::Source(BTreeMap::new()),
        }
    }

    /// Takes ownership of a source-side tree map.
    pub fn from_source(source: BTreeMap<K, V>) -> Self {
        Self {
            storage: AnalysisMapStorage::Source(source),
        }
    }

    /// Takes ownership of an already charged and strictly ordered entry
    /// buffer, validates its ordering, and charges the shared map owner.
    pub fn from_budgeted_entries(entries: Vec<(K, V)>, allocations: &mut AllocationBudget) -> Result<Self, AnalysisMapBuildError> {
        let frozen = BudgetedSortedMap::from_sorted_entries(entries).map_err(AnalysisMapBuildError::Entries)?;
        let frozen = allocations.arc(frozen).map_err(AnalysisMapBuildError::Allocation)?;
        Ok(Self {
            storage: AnalysisMapStorage::Frozen(frozen),
        })
    }

    /// Returns whether this map uses immutable imported backing.
    /// Returns whether this map uses immutable imported backing.
    pub fn is_frozen(&self) -> bool {
        matches!(self.storage, AnalysisMapStorage::Frozen(_))
    }

    /// Looks up one key without exposing the backing representation.
    pub fn get(&self, key: &K) -> Option<&V> {
        match &self.storage {
            AnalysisMapStorage::Source(source) => source.get(key),
            AnalysisMapStorage::Frozen(frozen) => frozen.get(key),
        }
    }

    /// Returns whether the key is present.
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Iterates key/value pairs in key order.
    pub fn iter(&self) -> AnalysisMapIter<'_, K, V> {
        match &self.storage {
            AnalysisMapStorage::Source(source) => AnalysisMapIter::Source(source.iter()),
            AnalysisMapStorage::Frozen(frozen) => AnalysisMapIter::Frozen(frozen.as_entries().iter()),
        }
    }

    /// Iterates keys in key order.
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &K> {
        self.iter().map(|(key, _)| key)
    }

    /// Iterates values in key order.
    pub fn values(&self) -> impl ExactSizeIterator<Item = &V> {
        self.iter().map(|(_, value)| value)
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        match &self.storage {
            AnalysisMapStorage::Source(source) => source.len(),
            AnalysisMapStorage::Frozen(frozen) => frozen.len(),
        }
    }

    /// Returns whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns mutable access only while this is a source-side map.
    pub fn get_source_mut(&mut self, key: &K) -> Result<Option<&mut V>, FrozenAnalysisMapMutation> {
        match &mut self.storage {
            AnalysisMapStorage::Source(source) => Ok(source.get_mut(key)),
            AnalysisMapStorage::Frozen(_) => Err(FrozenAnalysisMapMutation),
        }
    }

    /// Inserts into source-side storage. Imported maps reject mutation.
    pub fn insert_source(&mut self, key: K, value: V) -> Result<Option<V>, FrozenAnalysisMapMutation> {
        match &mut self.storage {
            AnalysisMapStorage::Source(source) => Ok(source.insert(key, value)),
            AnalysisMapStorage::Frozen(_) => Err(FrozenAnalysisMapMutation),
        }
    }

    /// Removes from source-side storage. Imported maps reject mutation.
    pub fn remove_source(&mut self, key: &K) -> Result<Option<V>, FrozenAnalysisMapMutation> {
        match &mut self.storage {
            AnalysisMapStorage::Source(source) => Ok(source.remove(key)),
            AnalysisMapStorage::Frozen(_) => Err(FrozenAnalysisMapMutation),
        }
    }

    /// Retains source-side entries. Imported maps reject mutation.
    pub fn retain_source(&mut self, predicate: impl FnMut(&K, &mut V) -> bool) -> Result<(), FrozenAnalysisMapMutation> {
        match &mut self.storage {
            AnalysisMapStorage::Source(source) => {
                source.retain(predicate);
                Ok(())
            }
            AnalysisMapStorage::Frozen(_) => Err(FrozenAnalysisMapMutation),
        }
    }

    /// Extends source-side storage. Imported maps reject mutation.
    pub fn extend_source(&mut self, entries: impl IntoIterator<Item = (K, V)>) -> Result<(), FrozenAnalysisMapMutation> {
        match &mut self.storage {
            AnalysisMapStorage::Source(source) => {
                source.extend(entries);
                Ok(())
            }
            AnalysisMapStorage::Frozen(_) => Err(FrozenAnalysisMapMutation),
        }
    }
}

impl<K: Ord, V> From<BTreeMap<K, V>> for AnalysisMap<K, V> {
    fn from(source: BTreeMap<K, V>) -> Self {
        Self::from_source(source)
    }
}

impl<K: Ord, V> From<HashMap<K, V>> for AnalysisMap<K, V> {
    fn from(source: HashMap<K, V>) -> Self {
        Self::from_source(source.into_iter().collect())
    }
}

impl<K: Ord, V, const N: usize> From<[(K, V); N]> for AnalysisMap<K, V> {
    fn from(entries: [(K, V); N]) -> Self {
        Self::from_source(entries.into_iter().collect())
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for AnalysisMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self::from_source(iter.into_iter().collect())
    }
}

impl<K: Ord, V> Index<&K> for AnalysisMap<K, V> {
    type Output = V;

    fn index(&self, key: &K) -> &Self::Output {
        self.get(key).expect("analysis map key is present")
    }
}

impl<K: Ord + Hash, V: Hash> Hash for AnalysisMap<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for (key, value) in self.iter() {
            key.hash(state);
            value.hash(state);
        }
    }
}

/// Allocation-free iterator over either backing representation.
///
/// Both variants yield entries in key order and borrow their owning map.
pub enum AnalysisMapIter<'a, K, V> {
    /// Iterates a source-side tree map.
    Source(std::collections::btree_map::Iter<'a, K, V>),
    /// Iterates an imported frozen entry slice.
    Frozen(std::slice::Iter<'a, (K, V)>),
}

impl<'a, K, V> Iterator for AnalysisMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next().map(|(key, value)| (key, value)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<K, V> ExactSizeIterator for AnalysisMapIter<'_, K, V> {
    fn len(&self) -> usize {
        match self {
            Self::Source(iter) => iter.len(),
            Self::Frozen(iter) => iter.len(),
        }
    }
}

impl<K, V> DoubleEndedIterator for AnalysisMapIter<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next_back(),
            Self::Frozen(iter) => iter.next_back().map(|(key, value)| (key, value)),
        }
    }
}

impl<'a, K: Ord, V> IntoIterator for &'a AnalysisMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = AnalysisMapIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{AnalysisMap, AnalysisMapBuildError, FrozenAnalysisMapMutation};
    use phalcom_type_meta::allocation::AllocationBudget;
    use std::collections::BTreeMap;

    fn frozen_entries(budget: &mut AllocationBudget) -> AnalysisMap<u32, &'static str> {
        let mut entries = budget.vector(2).expect("entry buffer is budgeted");
        entries.push((1, "one"));
        entries.push((3, "three"));
        AnalysisMap::from_budgeted_entries(entries, budget).expect("ordered entries freeze")
    }

    #[test]
    fn source_and_frozen_maps_share_read_order_and_lookup_behavior() {
        let source = AnalysisMap::from_source(BTreeMap::from([(1, "one"), (3, "three")]));
        let mut budget = AllocationBudget::new(1024);
        let frozen = frozen_entries(&mut budget);

        for map in [&source, &frozen] {
            assert_eq!(map.get(&1), Some(&"one"));
            assert!(map.contains_key(&3));
            assert_eq!(map.iter().map(|(key, value)| (*key, *value)).collect::<Vec<_>>(), [(1, "one"), (3, "three")]);
            assert_eq!(map.keys().copied().collect::<Vec<_>>(), [1, 3]);
            assert_eq!(map.values().copied().collect::<Vec<_>>(), ["one", "three"]);
            assert_eq!(map.len(), 2);
            assert!(!map.is_empty());
        }
        assert_eq!(source, frozen, "storage representation is not part of map equality");
    }

    #[test]
    fn frozen_map_rejects_every_source_mutation_api() {
        let mut budget = AllocationBudget::new(1024);
        let mut frozen = frozen_entries(&mut budget);

        assert_eq!(frozen.insert_source(4, "four"), Err(FrozenAnalysisMapMutation));
        assert_eq!(frozen.get_source_mut(&1), Err(FrozenAnalysisMapMutation));
        assert_eq!(frozen.remove_source(&1), Err(FrozenAnalysisMapMutation));
        assert_eq!(frozen.retain_source(|_, _| true), Err(FrozenAnalysisMapMutation));
        assert_eq!(frozen.len(), 2);
    }

    #[test]
    fn source_map_clone_mutation_is_isolated() {
        let mut original = AnalysisMap::from_source(BTreeMap::from([(1, "one")]));
        let mut clone = original.clone();

        original.insert_source(2, "two").unwrap();
        clone.insert_source(3, "three").unwrap();

        assert_eq!(original.keys().copied().collect::<Vec<_>>(), [1, 2]);
        assert_eq!(clone.keys().copied().collect::<Vec<_>>(), [1, 3]);
    }

    #[test]
    fn frozen_map_clone_shares_its_charged_backing() {
        let mut budget = AllocationBudget::new(1024);
        let frozen = frozen_entries(&mut budget);
        let cloned = frozen.clone();

        match (&frozen.storage, &cloned.storage) {
            (super::AnalysisMapStorage::Frozen(left), super::AnalysisMapStorage::Frozen(right)) => assert!(std::sync::Arc::ptr_eq(left, right)),
            _ => panic!("expected frozen backing"),
        }
    }

    #[test]
    fn frozen_owner_charge_is_transactional_at_exact_fit() {
        let mut probe = AllocationBudget::new(4096);
        let entries = {
            let mut entries = probe.vector(2).unwrap();
            entries.push((1, "one"));
            entries.push((3, "three"));
            entries
        };
        let before_owner = probe.requested();
        let _frozen = AnalysisMap::from_budgeted_entries(entries, &mut probe).unwrap();
        let exact = probe.requested();
        assert!(exact > before_owner);

        let mut short = AllocationBudget::new(exact - 1);
        let entries = {
            let mut entries = short.vector(2).unwrap();
            entries.push((1, "one"));
            entries.push((3, "three"));
            entries
        };
        let parent_requested = short.requested();
        assert_eq!(
            AnalysisMap::from_budgeted_entries(entries, &mut short),
            Err(AnalysisMapBuildError::Allocation(phalcom_type_meta::allocation::AllocationError::Exhausted))
        );
        assert_eq!(short.requested(), parent_requested);
    }
}
