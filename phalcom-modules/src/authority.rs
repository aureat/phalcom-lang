//! Source-mutable and artifact-frozen ordered collections for module authority.

use phalcom_type_meta::allocation::{AllocationBudget, AllocationError, BudgetedSortedMap, BudgetedSortedSet, SortedMapEntriesError, SortedSetEntriesError};
use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Index, IndexMut};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ModuleAuthorityBuildError {
    #[error("sorted map entries error: {0}")]
    Entries(#[from] SortedMapEntriesError),
    #[error("sorted set entries error: {0}")]
    SetEntries(#[from] SortedSetEntriesError),
    #[error("allocation error: {0}")]
    Allocation(#[from] AllocationError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("attempted to mutate frozen module authority")]
pub struct FrozenModuleAuthorityMutation;

/// Ordered map backed either by a mutable `BTreeMap` (for source compilation)
/// or an immutable, budgeted `BudgetedSortedMap` in an `Arc` (for artifact loading).
#[derive(Clone)]
pub enum ModuleAuthorityMap<K, V> {
    Source(BTreeMap<K, V>),
    Frozen(Arc<BudgetedSortedMap<K, V>>),
}

impl<K: Ord, V> Default for ModuleAuthorityMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Debug + Ord, V: Debug> Debug for ModuleAuthorityMap<K, V> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.debug_map().entries(self.iter()).finish()
    }
}

impl<K: Ord, V: PartialEq> PartialEq for ModuleAuthorityMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<K: Ord, V: Eq> Eq for ModuleAuthorityMap<K, V> {}

impl<K: Ord + Hash, V: Hash> Hash for ModuleAuthorityMap<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for (key, value) in self.iter() {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl<K: Ord, V> ModuleAuthorityMap<K, V> {
    pub fn new() -> Self {
        Self::Source(BTreeMap::new())
    }

    pub fn from_source(map: BTreeMap<K, V>) -> Self {
        Self::Source(map)
    }

    /// Takes ownership of a strictly ordered, pre-budgeted entry buffer and charges the `Arc` owner.
    pub fn from_budgeted_sorted_entries(entries: Vec<(K, V)>, allocations: &mut AllocationBudget) -> Result<Self, ModuleAuthorityBuildError> {
        let frozen = BudgetedSortedMap::from_sorted_entries(entries)?;
        let frozen = allocations.arc(frozen)?;
        Ok(Self::Frozen(frozen))
    }

    /// Sorts and freezes a budgeted entry buffer.
    pub fn from_budgeted_entries(mut entries: Vec<(K, V)>, allocations: &mut AllocationBudget) -> Result<Self, ModuleAuthorityBuildError> {
        entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        Self::from_budgeted_sorted_entries(entries, allocations)
    }

    pub fn is_frozen(&self) -> bool {
        matches!(self, Self::Frozen(_))
    }

    pub fn get<Q: Ord + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        match self {
            Self::Source(source) => source.get(key),
            Self::Frozen(frozen) => frozen.get(key),
        }
    }

    pub fn contains_key<Q: Ord + ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        self.get(key).is_some()
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Source(source) => source.len(),
            Self::Frozen(frozen) => frozen.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> ModuleAuthorityMapIter<'_, K, V> {
        match self {
            Self::Source(source) => ModuleAuthorityMapIter::Source(source.iter()),
            Self::Frozen(frozen) => ModuleAuthorityMapIter::Frozen(frozen.as_entries().iter()),
        }
    }

    pub fn keys(&self) -> impl ExactSizeIterator<Item = &K> {
        self.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl ExactSizeIterator<Item = &V> {
        self.iter().map(|(_, v)| v)
    }

    pub fn get_mut<Q: Ord + ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        match self {
            Self::Source(source) => source.get_mut(key),
            Self::Frozen(_) => panic!("attempted to mutate frozen module authority"),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self {
            Self::Source(source) => source.insert(key, value),
            Self::Frozen(_) => panic!("attempted to mutate frozen module authority"),
        }
    }

    pub fn remove<Q: Ord + ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        match self {
            Self::Source(source) => source.remove(key),
            Self::Frozen(_) => panic!("attempted to mutate frozen module authority"),
        }
    }

    pub fn entry(&mut self, key: K) -> std::collections::btree_map::Entry<'_, K, V> {
        match self {
            Self::Source(source) => source.entry(key),
            Self::Frozen(_) => panic!("attempted to mutate frozen module authority"),
        }
    }

    pub fn retain(&mut self, predicate: impl FnMut(&K, &mut V) -> bool) {
        match self {
            Self::Source(source) => source.retain(predicate),
            Self::Frozen(_) => panic!("attempted to mutate frozen module authority"),
        }
    }

    pub fn clear(&mut self) {
        match self {
            Self::Source(source) => source.clear(),
            Self::Frozen(_) => panic!("attempted to mutate frozen module authority"),
        }
    }

    pub fn as_source(&self) -> Option<&BTreeMap<K, V>> {
        match self {
            Self::Source(source) => Some(source),
            Self::Frozen(_) => None,
        }
    }

    pub fn as_source_mut(&mut self) -> Option<&mut BTreeMap<K, V>> {
        match self {
            Self::Source(source) => Some(source),
            Self::Frozen(_) => None,
        }
    }

    pub fn as_frozen(&self) -> Option<&Arc<BudgetedSortedMap<K, V>>> {
        match self {
            Self::Source(_) => None,
            Self::Frozen(frozen) => Some(frozen),
        }
    }
}

pub enum ModuleAuthorityMapIter<'a, K, V> {
    Source(std::collections::btree_map::Iter<'a, K, V>),
    Frozen(std::slice::Iter<'a, (K, V)>),
}

impl<'a, K, V> Iterator for ModuleAuthorityMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next().map(|(k, v)| (k, v)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<K, V> ExactSizeIterator for ModuleAuthorityMapIter<'_, K, V> {
    fn len(&self) -> usize {
        match self {
            Self::Source(iter) => iter.len(),
            Self::Frozen(iter) => iter.len(),
        }
    }
}

impl<K, V> DoubleEndedIterator for ModuleAuthorityMapIter<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next_back(),
            Self::Frozen(iter) => iter.next_back().map(|(k, v)| (k, v)),
        }
    }
}

impl<'a, K: Ord, V> IntoIterator for &'a ModuleAuthorityMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = ModuleAuthorityMapIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub enum ModuleAuthorityMapIntoIter<K, V> {
    Source(std::collections::btree_map::IntoIter<K, V>),
    Frozen(std::vec::IntoIter<(K, V)>),
}

impl<K, V> Iterator for ModuleAuthorityMapIntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Source(iter) => iter.size_hint(),
            Self::Frozen(iter) => iter.size_hint(),
        }
    }
}

impl<K, V> ExactSizeIterator for ModuleAuthorityMapIntoIter<K, V> {
    fn len(&self) -> usize {
        match self {
            Self::Source(iter) => iter.len(),
            Self::Frozen(iter) => iter.len(),
        }
    }
}

impl<K: Clone + Ord, V: Clone> IntoIterator for ModuleAuthorityMap<K, V> {
    type Item = (K, V);
    type IntoIter = ModuleAuthorityMapIntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Source(source) => ModuleAuthorityMapIntoIter::Source(source.into_iter()),
            Self::Frozen(frozen) => match Arc::try_unwrap(frozen) {
                Ok(map) => ModuleAuthorityMapIntoIter::Frozen(map.into_entries().into_iter()),
                Err(arc) => ModuleAuthorityMapIntoIter::Frozen(arc.as_entries().to_vec().into_iter()),
            },
        }
    }
}

impl<K: Ord + Borrow<Q>, Q: Ord + ?Sized, V> Index<&Q> for ModuleAuthorityMap<K, V> {
    type Output = V;

    fn index(&self, key: &Q) -> &Self::Output {
        self.get(key).expect("key present in module authority map")
    }
}

impl<K: Ord, V> IndexMut<&K> for ModuleAuthorityMap<K, V> {
    fn index_mut(&mut self, key: &K) -> &mut Self::Output {
        self.get_mut(key).expect("key present in module authority map")
    }
}

impl<K: Ord, V> From<BTreeMap<K, V>> for ModuleAuthorityMap<K, V> {
    fn from(source: BTreeMap<K, V>) -> Self {
        Self::Source(source)
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for ModuleAuthorityMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self::Source(iter.into_iter().collect())
    }
}

/// Ordered set backed either by a mutable `BTreeSet` (for source compilation)
/// or an immutable, budgeted `BudgetedSortedSet` in an `Arc` (for artifact loading).
#[derive(Clone)]
pub enum ModuleAuthoritySet<K> {
    Source(BTreeSet<K>),
    Frozen(Arc<BudgetedSortedSet<K>>),
}

impl<K: Ord> Default for ModuleAuthoritySet<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Debug + Ord> Debug for ModuleAuthoritySet<K> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.debug_set().entries(self.iter()).finish()
    }
}

impl<K: Ord> PartialEq for ModuleAuthoritySet<K> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<K: Ord> Eq for ModuleAuthoritySet<K> {}

impl<K: Ord + Hash> Hash for ModuleAuthoritySet<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for item in self.iter() {
            item.hash(state);
        }
    }
}

impl<K: Ord> ModuleAuthoritySet<K> {
    pub fn new() -> Self {
        Self::Source(BTreeSet::new())
    }

    pub fn from_source(set: BTreeSet<K>) -> Self {
        Self::Source(set)
    }

    /// Takes ownership of a strictly ordered, pre-budgeted entry buffer and charges the `Arc` owner.
    pub fn from_budgeted_sorted_entries(entries: Vec<K>, allocations: &mut AllocationBudget) -> Result<Self, ModuleAuthorityBuildError> {
        let frozen = BudgetedSortedSet::from_sorted_entries(entries)?;
        let frozen = allocations.arc(frozen)?;
        Ok(Self::Frozen(frozen))
    }

    /// Sorts and freezes a budgeted entry buffer.
    pub fn from_budgeted_entries(mut entries: Vec<K>, allocations: &mut AllocationBudget) -> Result<Self, ModuleAuthorityBuildError> {
        entries.sort_unstable();
        Self::from_budgeted_sorted_entries(entries, allocations)
    }

    pub fn is_frozen(&self) -> bool {
        matches!(self, Self::Frozen(_))
    }

    pub fn contains<Q: Ord + ?Sized>(&self, value: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        match self {
            Self::Source(source) => source.contains(value),
            Self::Frozen(frozen) => frozen.contains(value),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Source(source) => source.len(),
            Self::Frozen(frozen) => frozen.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> ModuleAuthoritySetIter<'_, K> {
        match self {
            Self::Source(source) => ModuleAuthoritySetIter::Source(source.iter()),
            Self::Frozen(frozen) => ModuleAuthoritySetIter::Frozen(frozen.as_entries().iter()),
        }
    }

    pub fn insert(&mut self, value: K) -> bool {
        match self {
            Self::Source(source) => source.insert(value),
            Self::Frozen(_) => panic!("attempted to mutate frozen module authority"),
        }
    }

    pub fn remove<Q: Ord + ?Sized>(&mut self, value: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        match self {
            Self::Source(source) => source.remove(value),
            Self::Frozen(_) => panic!("attempted to mutate frozen module authority"),
        }
    }

    pub fn retain(&mut self, predicate: impl FnMut(&K) -> bool) {
        match self {
            Self::Source(source) => source.retain(predicate),
            Self::Frozen(_) => panic!("attempted to mutate frozen module authority"),
        }
    }

    pub fn clear(&mut self) {
        match self {
            Self::Source(source) => source.clear(),
            Self::Frozen(_) => panic!("attempted to mutate frozen module authority"),
        }
    }

    pub fn as_source(&self) -> Option<&BTreeSet<K>> {
        match self {
            Self::Source(source) => Some(source),
            Self::Frozen(_) => None,
        }
    }

    pub fn as_source_mut(&mut self) -> Option<&mut BTreeSet<K>> {
        match self {
            Self::Source(source) => Some(source),
            Self::Frozen(_) => None,
        }
    }

    pub fn as_frozen(&self) -> Option<&Arc<BudgetedSortedSet<K>>> {
        match self {
            Self::Source(_) => None,
            Self::Frozen(frozen) => Some(frozen),
        }
    }
}

pub enum ModuleAuthoritySetIter<'a, K> {
    Source(std::collections::btree_set::Iter<'a, K>),
    Frozen(std::slice::Iter<'a, K>),
}

impl<'a, K> Iterator for ModuleAuthoritySetIter<'a, K> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<K> ExactSizeIterator for ModuleAuthoritySetIter<'_, K> {
    fn len(&self) -> usize {
        match self {
            Self::Source(iter) => iter.len(),
            Self::Frozen(iter) => iter.len(),
        }
    }
}

impl<'a, K: Ord> IntoIterator for &'a ModuleAuthoritySet<K> {
    type Item = &'a K;
    type IntoIter = ModuleAuthoritySetIter<'a, K>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub enum ModuleAuthoritySetIntoIter<K> {
    Source(std::collections::btree_set::IntoIter<K>),
    Frozen(std::vec::IntoIter<K>),
}

impl<K> Iterator for ModuleAuthoritySetIntoIter<K> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Source(iter) => iter.size_hint(),
            Self::Frozen(iter) => iter.size_hint(),
        }
    }
}

impl<K> ExactSizeIterator for ModuleAuthoritySetIntoIter<K> {
    fn len(&self) -> usize {
        match self {
            Self::Source(iter) => iter.len(),
            Self::Frozen(iter) => iter.len(),
        }
    }
}

impl<K: Clone + Ord> IntoIterator for ModuleAuthoritySet<K> {
    type Item = K;
    type IntoIter = ModuleAuthoritySetIntoIter<K>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Source(source) => ModuleAuthoritySetIntoIter::Source(source.into_iter()),
            Self::Frozen(frozen) => match Arc::try_unwrap(frozen) {
                Ok(set) => ModuleAuthoritySetIntoIter::Frozen(set.into_entries().into_iter()),
                Err(arc) => ModuleAuthoritySetIntoIter::Frozen(arc.as_entries().to_vec().into_iter()),
            },
        }
    }
}

impl<K: Ord> From<BTreeSet<K>> for ModuleAuthoritySet<K> {
    fn from(source: BTreeSet<K>) -> Self {
        Self::Source(source)
    }
}

impl<K: Ord> FromIterator<K> for ModuleAuthoritySet<K> {
    fn from_iter<T: IntoIterator<Item = K>>(iter: T) -> Self {
        Self::Source(iter.into_iter().collect())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Common Read Adapters for Facade / View Consumers
// ─────────────────────────────────────────────────────────────────────────────

pub enum ModuleAuthorityMapRef<'a, K, V> {
    Source(&'a BTreeMap<K, V>),
    Frozen(&'a BudgetedSortedMap<K, V>),
}

impl<'a, K, V> Clone for ModuleAuthorityMapRef<'a, K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, K, V> Copy for ModuleAuthorityMapRef<'a, K, V> {}

impl<'a, K: Ord, V> ModuleAuthorityMapRef<'a, K, V> {
    pub fn get<Q: Ord + ?Sized>(&self, key: &Q) -> Option<&'a V>
    where
        K: Borrow<Q>,
    {
        match self {
            Self::Source(map) => map.get(key),
            Self::Frozen(map) => map.get(key),
        }
    }

    pub fn contains_key<Q: Ord + ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        self.get(key).is_some()
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Source(map) => map.len(),
            Self::Frozen(map) => map.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn keys(&self) -> ModuleAuthorityMapRefKeys<'a, K, V> {
        match self {
            Self::Source(map) => ModuleAuthorityMapRefKeys::Source(map.keys()),
            Self::Frozen(map) => ModuleAuthorityMapRefKeys::Frozen(map.as_entries().iter()),
        }
    }

    pub fn values(&self) -> ModuleAuthorityMapRefValues<'a, K, V> {
        match self {
            Self::Source(map) => ModuleAuthorityMapRefValues::Source(map.values()),
            Self::Frozen(map) => ModuleAuthorityMapRefValues::Frozen(map.as_entries().iter()),
        }
    }

    pub fn iter(&self) -> ModuleAuthorityMapRefIter<'a, K, V> {
        match self {
            Self::Source(map) => ModuleAuthorityMapRefIter::Source(map.iter()),
            Self::Frozen(map) => ModuleAuthorityMapRefIter::Frozen(map.as_entries().iter()),
        }
    }
}

pub enum ModuleAuthorityMapRefIter<'a, K, V> {
    Source(std::collections::btree_map::Iter<'a, K, V>),
    Frozen(std::slice::Iter<'a, (K, V)>),
}

impl<'a, K, V> Iterator for ModuleAuthorityMapRefIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next().map(|(k, v)| (k, v)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<K, V> ExactSizeIterator for ModuleAuthorityMapRefIter<'_, K, V> {
    fn len(&self) -> usize {
        match self {
            Self::Source(iter) => iter.len(),
            Self::Frozen(iter) => iter.len(),
        }
    }
}

pub enum ModuleAuthorityMapRefKeys<'a, K, V> {
    Source(std::collections::btree_map::Keys<'a, K, V>),
    Frozen(std::slice::Iter<'a, (K, V)>),
}

impl<'a, K, V> Iterator for ModuleAuthorityMapRefKeys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next().map(|(k, _)| k),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<K, V> ExactSizeIterator for ModuleAuthorityMapRefKeys<'_, K, V> {
    fn len(&self) -> usize {
        match self {
            Self::Source(iter) => iter.len(),
            Self::Frozen(iter) => iter.len(),
        }
    }
}

pub enum ModuleAuthorityMapRefValues<'a, K, V> {
    Source(std::collections::btree_map::Values<'a, K, V>),
    Frozen(std::slice::Iter<'a, (K, V)>),
}

impl<'a, K, V> Iterator for ModuleAuthorityMapRefValues<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next().map(|(_, v)| v),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<K, V> ExactSizeIterator for ModuleAuthorityMapRefValues<'_, K, V> {
    fn len(&self) -> usize {
        match self {
            Self::Source(iter) => iter.len(),
            Self::Frozen(iter) => iter.len(),
        }
    }
}

impl<'a, K: Ord, V> IntoIterator for ModuleAuthorityMapRef<'a, K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = ModuleAuthorityMapRefIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K: Ord, V> From<&'a BTreeMap<K, V>> for ModuleAuthorityMapRef<'a, K, V> {
    fn from(map: &'a BTreeMap<K, V>) -> Self {
        Self::Source(map)
    }
}

impl<'a, K: Ord, V> From<&'a ModuleAuthorityMap<K, V>> for ModuleAuthorityMapRef<'a, K, V> {
    fn from(map: &'a ModuleAuthorityMap<K, V>) -> Self {
        match map {
            ModuleAuthorityMap::Source(source) => Self::Source(source),
            ModuleAuthorityMap::Frozen(frozen) => Self::Frozen(frozen),
        }
    }
}

impl<'a, K: Ord, V> From<&'a Arc<BTreeMap<K, V>>> for ModuleAuthorityMapRef<'a, K, V> {
    fn from(map: &'a Arc<BTreeMap<K, V>>) -> Self {
        Self::Source(map.as_ref())
    }
}

impl<'a, K: Ord, V> From<&'a Arc<BudgetedSortedMap<K, V>>> for ModuleAuthorityMapRef<'a, K, V> {
    fn from(map: &'a Arc<BudgetedSortedMap<K, V>>) -> Self {
        Self::Frozen(map)
    }
}

pub enum ModuleAuthoritySetRef<'a, K> {
    Source(&'a BTreeSet<K>),
    Frozen(&'a BudgetedSortedSet<K>),
}

impl<'a, K> Clone for ModuleAuthoritySetRef<'a, K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, K> Copy for ModuleAuthoritySetRef<'a, K> {}

impl<'a, K: Ord> ModuleAuthoritySetRef<'a, K> {
    pub fn contains<Q: Ord + ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        match self {
            Self::Source(set) => set.contains(key),
            Self::Frozen(set) => set.contains(key),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Source(set) => set.len(),
            Self::Frozen(set) => set.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> ModuleAuthoritySetRefIter<'a, K> {
        match self {
            Self::Source(set) => ModuleAuthoritySetRefIter::Source(set.iter()),
            Self::Frozen(set) => ModuleAuthoritySetRefIter::Frozen(set.as_entries().iter()),
        }
    }
}

pub enum ModuleAuthoritySetRefIter<'a, K> {
    Source(std::collections::btree_set::Iter<'a, K>),
    Frozen(std::slice::Iter<'a, K>),
}

impl<'a, K> Iterator for ModuleAuthoritySetRefIter<'a, K> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<K> ExactSizeIterator for ModuleAuthoritySetRefIter<'_, K> {
    fn len(&self) -> usize {
        match self {
            Self::Source(iter) => iter.len(),
            Self::Frozen(iter) => iter.len(),
        }
    }
}

impl<'a, K: Ord> IntoIterator for ModuleAuthoritySetRef<'a, K> {
    type Item = &'a K;
    type IntoIter = ModuleAuthoritySetRefIter<'a, K>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K: Ord> From<&'a BTreeSet<K>> for ModuleAuthoritySetRef<'a, K> {
    fn from(set: &'a BTreeSet<K>) -> Self {
        Self::Source(set)
    }
}

impl<'a, K: Ord> From<&'a ModuleAuthoritySet<K>> for ModuleAuthoritySetRef<'a, K> {
    fn from(set: &'a ModuleAuthoritySet<K>) -> Self {
        match set {
            ModuleAuthoritySet::Source(source) => Self::Source(source),
            ModuleAuthoritySet::Frozen(frozen) => Self::Frozen(frozen),
        }
    }
}

impl<'a, K: Ord> From<&'a Arc<BTreeSet<K>>> for ModuleAuthoritySetRef<'a, K> {
    fn from(set: &'a Arc<BTreeSet<K>>) -> Self {
        Self::Source(set.as_ref())
    }
}

impl<'a, K: Ord> From<&'a Arc<BudgetedSortedSet<K>>> for ModuleAuthoritySetRef<'a, K> {
    fn from(set: &'a Arc<BudgetedSortedSet<K>>) -> Self {
        Self::Frozen(set)
    }
}

/// Facade read adapter for reverse imports, accommodating both source maps and frozen authority maps.
#[derive(Clone, Copy)]
pub enum ReverseImportsRef<'a> {
    Source(&'a BTreeMap<crate::identity::ModuleId, BTreeSet<crate::identity::ModuleId>>),
    Authority(ModuleAuthorityMapRef<'a, crate::identity::ModuleId, ModuleAuthoritySet<crate::identity::ModuleId>>),
}

impl<'a> ReverseImportsRef<'a> {
    pub fn get(&self, module: &crate::identity::ModuleId) -> Option<ModuleAuthoritySetRef<'a, crate::identity::ModuleId>> {
        match self {
            Self::Source(map) => map.get(module).map(ModuleAuthoritySetRef::Source),
            Self::Authority(map) => map.get(module).map(Into::into),
        }
    }
}

impl<'a> From<&'a BTreeMap<crate::identity::ModuleId, BTreeSet<crate::identity::ModuleId>>> for ReverseImportsRef<'a> {
    fn from(map: &'a BTreeMap<crate::identity::ModuleId, BTreeSet<crate::identity::ModuleId>>) -> Self {
        Self::Source(map)
    }
}

impl<'a> From<&'a Arc<BTreeMap<crate::identity::ModuleId, BTreeSet<crate::identity::ModuleId>>>> for ReverseImportsRef<'a> {
    fn from(map: &'a Arc<BTreeMap<crate::identity::ModuleId, BTreeSet<crate::identity::ModuleId>>>) -> Self {
        Self::Source(map.as_ref())
    }
}

impl<'a> From<&'a ModuleAuthorityMap<crate::identity::ModuleId, ModuleAuthoritySet<crate::identity::ModuleId>>> for ReverseImportsRef<'a> {
    fn from(map: &'a ModuleAuthorityMap<crate::identity::ModuleId, ModuleAuthoritySet<crate::identity::ModuleId>>) -> Self {
        Self::Authority(map.into())
    }
}

impl<'a> From<ModuleAuthorityMapRef<'a, crate::identity::ModuleId, ModuleAuthoritySet<crate::identity::ModuleId>>> for ReverseImportsRef<'a> {
    fn from(map: ModuleAuthorityMapRef<'a, crate::identity::ModuleId, ModuleAuthoritySet<crate::identity::ModuleId>>) -> Self {
        Self::Authority(map)
    }
}
