//! Source-mutable and artifact-frozen ordered sets for callable analysis.

use phalcom_type_meta::allocation::{AllocationBudget, AllocationError, BudgetedSortedSet, SortedSetEntriesError};
use std::collections::BTreeSet;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

/// Immutable-set construction failure for a callable analysis owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisSetBuildError {
    /// The already-created entry buffer was not strictly ordered.
    Entries(SortedSetEntriesError),
    /// Publishing the shared immutable backing exceeded the allocation budget.
    Allocation(AllocationError),
}

/// A failed attempt to mutate an imported immutable analysis set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrozenAnalysisSetMutation;

#[derive(Clone, Debug)]
enum AnalysisSetStorage<T> {
    Source(BTreeSet<T>),
    Frozen(Arc<BudgetedSortedSet<T>>),
}

/// Ordered set with mutable source storage and immutable charged artifact
/// storage behind one read interface.
#[derive(Clone)]
pub struct AnalysisSet<T> {
    storage: AnalysisSetStorage<T>,
}

impl<T: Debug + Ord> Debug for AnalysisSet<T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.debug_set().entries(self.iter()).finish()
    }
}

impl<T: Ord> PartialEq for AnalysisSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<T: Ord> Eq for AnalysisSet<T> {}

impl<T: Ord> Default for AnalysisSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> AnalysisSet<T> {
    /// Creates an empty source-side set.
    pub fn new() -> Self {
        Self {
            storage: AnalysisSetStorage::Source(BTreeSet::new()),
        }
    }

    /// Takes ownership of a source-side tree set.
    pub fn from_source(source: BTreeSet<T>) -> Self {
        Self {
            storage: AnalysisSetStorage::Source(source),
        }
    }

    /// Takes ownership of an already charged and strictly ordered entry
    /// buffer, validates its ordering, and charges the shared frozen-set
    /// owner.
    pub fn from_budgeted_entries(entries: Vec<T>, allocations: &mut AllocationBudget) -> Result<Self, AnalysisSetBuildError> {
        let frozen = BudgetedSortedSet::from_sorted_entries(entries).map_err(AnalysisSetBuildError::Entries)?;
        let frozen = allocations.arc(frozen).map_err(AnalysisSetBuildError::Allocation)?;
        Ok(Self {
            storage: AnalysisSetStorage::Frozen(frozen),
        })
    }

    /// Returns whether this set uses immutable imported backing.
    pub fn is_frozen(&self) -> bool {
        matches!(self.storage, AnalysisSetStorage::Frozen(_))
    }

    /// Returns whether the set contains `value`.
    pub fn contains(&self, value: &T) -> bool {
        match &self.storage {
            AnalysisSetStorage::Source(source) => source.contains(value),
            AnalysisSetStorage::Frozen(frozen) => frozen.contains(value),
        }
    }

    /// Iterates values in sorted order.
    pub fn iter(&self) -> AnalysisSetIter<'_, T> {
        match &self.storage {
            AnalysisSetStorage::Source(source) => AnalysisSetIter::Source(source.iter()),
            AnalysisSetStorage::Frozen(frozen) => AnalysisSetIter::Frozen(frozen.as_entries().iter()),
        }
    }

    /// Returns the number of values.
    pub fn len(&self) -> usize {
        match &self.storage {
            AnalysisSetStorage::Source(source) => source.len(),
            AnalysisSetStorage::Frozen(frozen) => frozen.len(),
        }
    }

    /// Returns whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Inserts into source-side storage. Imported sets reject mutation.
    pub fn insert_source(&mut self, value: T) -> Result<bool, FrozenAnalysisSetMutation> {
        match &mut self.storage {
            AnalysisSetStorage::Source(source) => Ok(source.insert(value)),
            AnalysisSetStorage::Frozen(_) => Err(FrozenAnalysisSetMutation),
        }
    }

    /// Removes from source-side storage. Imported sets reject mutation.
    pub fn remove_source(&mut self, value: &T) -> Result<bool, FrozenAnalysisSetMutation> {
        match &mut self.storage {
            AnalysisSetStorage::Source(source) => Ok(source.remove(value)),
            AnalysisSetStorage::Frozen(_) => Err(FrozenAnalysisSetMutation),
        }
    }
}

impl<T: Ord> From<BTreeSet<T>> for AnalysisSet<T> {
    fn from(source: BTreeSet<T>) -> Self {
        Self::from_source(source)
    }
}

impl<T: Ord> FromIterator<T> for AnalysisSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_source(iter.into_iter().collect())
    }
}

/// Allocation-free iterator over either backing representation.
pub enum AnalysisSetIter<'a, T> {
    /// Iterates a source-side tree set.
    Source(std::collections::btree_set::Iter<'a, T>),
    /// Iterates an imported frozen entry slice.
    Frozen(std::slice::Iter<'a, T>),
}

impl<'a, T> Iterator for AnalysisSetIter<'a, T> {
    type Item = &'a T;

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

impl<T> ExactSizeIterator for AnalysisSetIter<'_, T> {
    fn len(&self) -> usize {
        match self {
            Self::Source(iter) => iter.len(),
            Self::Frozen(iter) => iter.len(),
        }
    }
}

impl<T> DoubleEndedIterator for AnalysisSetIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next_back(),
            Self::Frozen(iter) => iter.next_back(),
        }
    }
}

impl<'a, T: Ord> IntoIterator for &'a AnalysisSet<T> {
    type Item = &'a T;
    type IntoIter = AnalysisSetIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{AnalysisSet, AnalysisSetBuildError, FrozenAnalysisSetMutation};
    use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
    use std::collections::BTreeSet;

    fn frozen_entries(budget: &mut AllocationBudget) -> AnalysisSet<u32> {
        let mut entries = budget.vector(2).expect("set entry buffer is budgeted");
        entries.extend([1, 3]);
        AnalysisSet::from_budgeted_entries(entries, budget).expect("ordered entries freeze")
    }

    #[test]
    fn source_and_frozen_sets_share_read_order_and_membership() {
        let source = AnalysisSet::from_source(BTreeSet::from([1, 3]));
        let mut budget = AllocationBudget::new(1024);
        let frozen = frozen_entries(&mut budget);
        for set in [&source, &frozen] {
            assert_eq!(set.iter().copied().collect::<Vec<_>>(), [1, 3]);
            assert!(set.contains(&3));
            assert!(!set.contains(&2));
            assert_eq!(set.len(), 2);
            assert!(!set.is_empty());
        }
        assert_eq!(source, frozen, "storage representation is not part of set equality");
        assert_eq!(format!("{source:?}"), format!("{frozen:?}"), "debug output exposes only semantic entries");
    }

    #[test]
    fn frozen_set_rejects_source_mutation_and_source_clones_are_isolated() {
        let mut budget = AllocationBudget::new(1024);
        let mut frozen = frozen_entries(&mut budget);
        assert_eq!(frozen.insert_source(4), Err(FrozenAnalysisSetMutation));
        assert_eq!(frozen.remove_source(&1), Err(FrozenAnalysisSetMutation));

        let mut source = AnalysisSet::from_source(BTreeSet::from([1]));
        let mut clone = source.clone();
        source.insert_source(2).unwrap();
        clone.insert_source(3).unwrap();
        assert_eq!(source.iter().copied().collect::<Vec<_>>(), [1, 2]);
        assert_eq!(clone.iter().copied().collect::<Vec<_>>(), [1, 3]);
    }

    #[test]
    fn frozen_set_owner_charge_requires_exact_budget() {
        fn build(budget: &mut AllocationBudget) -> Result<AnalysisSet<u32>, AnalysisSetBuildError> {
            let mut entries = budget.vector(2).map_err(AnalysisSetBuildError::Allocation)?;
            entries.extend([1, 3]);
            AnalysisSet::from_budgeted_entries(entries, budget)
        }

        let mut probe = AllocationBudget::new(1024);
        let _set = build(&mut probe).unwrap();
        let exact = probe.requested();
        let mut fits = AllocationBudget::new(exact);
        assert_eq!(build(&mut fits).unwrap().iter().copied().collect::<Vec<_>>(), [1, 3]);
        assert_eq!(fits.requested(), exact);
        let mut short = AllocationBudget::new(exact - 1);
        assert!(matches!(build(&mut short), Err(AnalysisSetBuildError::Allocation(AllocationError::Exhausted))));
    }
}
