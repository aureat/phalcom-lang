//! Source-mutable and artifact-frozen ordered map backing.

use phalcom_type_meta::allocation::{AllocationBudget, AllocationError, BudgetedSortedMap, SortedMapEntriesError};
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::ops::Index;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct OrderedMap<K, V> {
    source: BTreeMap<K, V>,
    frozen: Option<Arc<BudgetedSortedMap<K, V>>>,
}

impl<K, V> Default for OrderedMap<K, V> {
    fn default() -> Self {
        Self { source: BTreeMap::new(), frozen: None }
    }
}

impl<K: Ord, V> OrderedMap<K, V> {
    pub fn from_source(source: BTreeMap<K, V>) -> Self {
        Self { source, frozen: None }
    }

    pub fn from_frozen_entries_with_allocation_budget(
        mut entries: Vec<(K, V)>,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, FrozenOrderedMapError> {
        if entries.is_empty() {
            return Ok(Self::default());
        }
        entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        let entries = BudgetedSortedMap::from_sorted_entries(entries).map_err(|error| match error {
            SortedMapEntriesError::DuplicateKey => FrozenOrderedMapError::DuplicateKey,
            SortedMapEntriesError::OutOfOrder => unreachable!("entries were sorted before frozen construction"),
        })?;
        let frozen = allocations.arc(entries).map_err(FrozenOrderedMapError::Allocation)?;
        Ok(Self { source: BTreeMap::new(), frozen: Some(frozen) })
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.source.get(key).or_else(|| self.frozen.as_ref().and_then(|entries| entries.get(key)))
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        assert!(
            !self.frozen.as_ref().is_some_and(|entries| entries.contains_key(&key)),
            "cannot overwrite a frozen ordered-map entry"
        );
        self.source.insert(key, value)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.source.remove(key)
    }

    /// Mutates only source/local entries; frozen canonical authority is shared.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where K: Borrow<Q>, Q: Ord + ?Sized {
        self.source.get_mut(key)
    }

    pub fn retain(&mut self, predicate: impl FnMut(&K, &mut V) -> bool) {
        self.source.retain(predicate);
    }

    pub fn iter(&self) -> OrderedMapIter<'_, K, V> {
        match &self.frozen {
            Some(entries) => OrderedMapIter::Merged {
                frozen: entries.as_entries().iter().peekable(),
                local: self.source.iter().peekable(),
            },
            None => OrderedMapIter::Source(self.source.iter()),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(key, _)| key)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_, value)| value)
    }

    pub fn len(&self) -> usize {
        self.source.len() + self.frozen.as_ref().map_or(0, |entries| entries.len())
    }

    pub fn is_empty(&self) -> bool {
        self.source.is_empty() && self.frozen.as_ref().is_none_or(|entries| entries.is_empty())
    }
}

impl<K: Ord, V> From<BTreeMap<K, V>> for OrderedMap<K, V> {
    fn from(source: BTreeMap<K, V>) -> Self {
        Self::from_source(source)
    }
}

pub enum OrderedMapIter<'a, K, V> {
    Source(std::collections::btree_map::Iter<'a, K, V>),
    Merged {
        frozen: std::iter::Peekable<std::slice::Iter<'a, (K, V)>>,
        local: std::iter::Peekable<std::collections::btree_map::Iter<'a, K, V>>,
    },
}

impl<'a, K: Ord, V> Iterator for OrderedMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Merged { frozen, local } => {
                if local.peek().is_some_and(|(key, _)| frozen.peek().is_none_or(|(frozen_key, _)| *key < frozen_key)) {
                    local.next()
                } else {
                    frozen.next().map(|(key, value)| (key, value))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Source(iter) => iter.size_hint(),
            Self::Merged { frozen, local } => {
                let count = frozen.len() + local.len();
                (count, Some(count))
            }
        }
    }
}

impl<K: Ord, V> ExactSizeIterator for OrderedMapIter<'_, K, V> {}

impl<K: Hash + Ord, V: Hash> Hash for OrderedMap<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for (key, value) in self.iter() {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl<K: Ord + PartialEq, V: PartialEq> PartialEq for OrderedMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(left, right)| left == right)
    }
}

impl<K: Ord + Eq, V: Eq> Eq for OrderedMap<K, V> {}

impl<'a, K: Ord, V> IntoIterator for &'a OrderedMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = OrderedMapIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K, V, Q> Index<&Q> for OrderedMap<K, V>
where
    K: Ord + Borrow<Q>,
    Q: Ord + ?Sized,
{
    type Output = V;

    fn index(&self, key: &Q) -> &Self::Output {
        self.get(key).expect("ordered-map entry is present")
    }
}

#[derive(Debug)]
pub enum FrozenOrderedMapError {
    Allocation(AllocationError),
    DuplicateKey,
}
