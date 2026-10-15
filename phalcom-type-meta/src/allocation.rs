//! Cumulative allocator-requested bytes for one staged artifact load.
use std::alloc::Layout;
use std::borrow::Borrow;
use std::ops::{Bound, RangeBounds};

/// A load owns a private copy; the caller commits remaining bytes only on success.
/// Allocator-internal bookkeeping is excluded. Temporary allocations are not refunded.
#[derive(Clone, Debug)]
pub struct AllocationBudget {
    remaining: usize,
    requested: usize,
}

/// A section-local ledger. Allocations are made against this private ledger and
/// are charged to the parent transaction only after the section succeeds.
/// Dropping an uncommitted child leaves the caller-visible parent unchanged;
/// the enclosing load still fails, so temporary allocations are never reused by
/// a later stage of the same transaction.
#[derive(Debug)]
pub struct ChildAllocationBudget {
    budget: AllocationBudget,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum AllocationError {
    #[error("artifact allocation budget exceeded")]
    Exhausted,
    #[error("artifact allocation layout overflow")]
    Overflow,
    #[error("artifact allocation failed")]
    Failed,
    /// A semantic decode/import path still allocates outside its load ledger.
    #[error("semantic artifact operation has an unaccounted allocation boundary")]
    UnaccountedBoundary,
}
impl AllocationBudget {
    pub fn new(remaining: usize) -> Self {
        Self { remaining, requested: 0 }
    }
    pub fn remaining(&self) -> usize {
        self.remaining
    }
    pub fn requested(&self) -> usize {
        self.requested
    }
    pub fn child(&self, limit: usize) -> ChildAllocationBudget {
        ChildAllocationBudget {
            budget: Self::new(self.remaining.min(limit)),
        }
    }
    pub fn commit_child(&mut self, child: ChildAllocationBudget) -> Result<(), AllocationError> {
        self.reserve_bytes(child.budget.requested)
    }
    pub fn reserve_bytes(&mut self, bytes: usize) -> Result<(), AllocationError> {
        let remaining = self.remaining.checked_sub(bytes).ok_or(AllocationError::Exhausted)?;
        let requested = self.requested.checked_add(bytes).ok_or(AllocationError::Overflow)?;
        self.remaining = remaining;
        self.requested = requested;
        Ok(())
    }
    pub fn reserve_layout(&mut self, layout: Layout) -> Result<(), AllocationError> {
        self.reserve_bytes(layout.size())
    }
    pub fn reserve_array<T>(&mut self, count: usize) -> Result<(), AllocationError> {
        self.reserve_layout(Layout::array::<T>(count).map_err(|_| AllocationError::Overflow)?)
    }
    pub fn vector<T>(&mut self, count: usize) -> Result<Vec<T>, AllocationError> {
        let before = (self.remaining, self.requested);
        self.reserve_array::<T>(count)?;
        let mut v = Vec::new();
        if v.try_reserve_exact(count).is_err() {
            self.restore(before);
            return Err(AllocationError::Failed);
        }
        Ok(v)
    }
    pub fn grow_vector_exact<T>(&mut self, values: &mut Vec<T>, additional: usize) -> Result<(), AllocationError> {
        if additional <= values.capacity().saturating_sub(values.len()) {
            return Ok(());
        }
        let target = values.len().checked_add(additional).ok_or(AllocationError::Overflow)?;
        // RawVec::try_reserve_exact uses grow_exact and requests this complete
        // replacement layout from the allocator. Reallocations therefore debit
        // the new requested layout, not merely the capacity delta.
        let before = (self.remaining, self.requested);
        self.reserve_array::<T>(target)?;
        if values.try_reserve_exact(additional).is_err() {
            self.restore(before);
            return Err(AllocationError::Failed);
        }
        Ok(())
    }

    pub fn string(&mut self, value: &str) -> Result<String, AllocationError> {
        let before = (self.remaining, self.requested);
        self.reserve_array::<u8>(value.len())?;
        let mut owned = String::new();
        if owned.try_reserve_exact(value.len()).is_err() {
            self.restore(before);
            return Err(AllocationError::Failed);
        }
        owned.push_str(value);
        Ok(owned)
    }

    /// Creates an exactly sized owned string slice with a fallible byte request.
    pub fn boxed_str(&mut self, value: &str) -> Result<Box<str>, AllocationError> {
        use std::mem::MaybeUninit;

        let before = (self.remaining, self.requested);
        let layout = Layout::array::<MaybeUninit<u8>>(value.len()).map_err(|_| AllocationError::Overflow)?;
        self.reserve_layout(layout)?;
        let mut bytes = match Box::<[u8]>::try_new_uninit_slice(value.len()) {
            Ok(bytes) => bytes,
            Err(_) => {
                self.restore(before);
                return Err(AllocationError::Failed);
            }
        };
        for (destination, source) in bytes.iter_mut().zip(value.as_bytes()) {
            destination.write(*source);
        }
        // SAFETY: every element is initialized exactly once from the valid UTF-8
        // source slice, and the box retains the same element count and layout.
        let bytes = unsafe { bytes.assume_init() };
        let bytes = Vec::from(bytes);
        // SAFETY: these bytes were copied directly from `&str`, so they remain
        // valid UTF-8; `Vec::from(Box<[u8]>)` preserves the exact byte length.
        let owned = unsafe { String::from_utf8_unchecked(bytes) };
        Ok(owned.into_boxed_str())
    }

    pub fn boxed<T>(&mut self, value: T) -> Result<Box<T>, AllocationError> {
        let before = (self.remaining, self.requested);
        self.reserve_layout(Layout::new::<T>())?;
        match Box::try_new(value) {
            Ok(value) => Ok(value),
            Err(_) => {
                self.restore(before);
                Err(AllocationError::Failed)
            }
        }
    }

    pub fn arc<T>(&mut self, value: T) -> Result<std::sync::Arc<T>, AllocationError> {
        // Arc allocates one block containing two reference counts and T. This
        // is the same checked layout used by the standard library Arc owner.
        let layout = Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(Layout::new::<T>())
            .map_err(|_| AllocationError::Overflow)?
            .0
            .pad_to_align();
        let before = (self.remaining, self.requested);
        self.reserve_layout(layout)?;
        match std::sync::Arc::try_new(value) {
            Ok(value) => Ok(value),
            Err(_) => {
                self.restore(before);
                Err(AllocationError::Failed)
            }
        }
    }

    /// Starts a sorted map whose pair buffer is charged before allocation.
    /// Additional pushes charge each replacement buffer before it is requested.
    pub fn sorted_map_builder<K, V>(&mut self, expected: usize) -> Result<BudgetedSortedMapBuilder<'_, K, V>, AllocationError> {
        let entries = self.vector(expected)?;
        Ok(BudgetedSortedMapBuilder { budget: self, entries })
    }

    /// Starts a sorted set whose element buffer is charged before allocation.
    pub fn sorted_set_builder<K>(&mut self, expected: usize) -> Result<BudgetedSortedSetBuilder<'_, K>, AllocationError> {
        let entries = self.vector(expected)?;
        Ok(BudgetedSortedSetBuilder { budget: self, entries })
    }

    fn restore(&mut self, state: (usize, usize)) {
        (self.remaining, self.requested) = state;
    }
}

/// Error returned when a frozen ordered map or set contains duplicate keys.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("duplicate key in budgeted sorted collection")]
pub struct DuplicateKey;

/// Error returned when an already allocated ordered-map buffer is not strictly ordered.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SortedMapEntriesError {
    #[error("duplicate key in sorted map entries")]
    DuplicateKey,
    #[error("sorted map entries are out of order")]
    OutOfOrder,
}

/// Immutable ordered map backed by one explicitly charged contiguous buffer.
#[derive(Debug, Eq, PartialEq)]
pub struct BudgetedSortedMap<K, V> {
    entries: Vec<(K, V)>,
}

impl<K: Ord, V> BudgetedSortedMap<K, V> {
    /// Takes ownership of a pre-budgeted, strictly ordered entry buffer.
    pub fn from_sorted_entries(entries: Vec<(K, V)>) -> Result<Self, SortedMapEntriesError> {
        for pair in entries.windows(2) {
            match pair[0].0.cmp(&pair[1].0) {
                std::cmp::Ordering::Less => {}
                std::cmp::Ordering::Equal => return Err(SortedMapEntriesError::DuplicateKey),
                std::cmp::Ordering::Greater => return Err(SortedMapEntriesError::OutOfOrder),
            }
        }
        Ok(Self { entries })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get<Q: Ord + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        let index = self.entries.binary_search_by(|(candidate, _)| candidate.borrow().cmp(key)).ok()?;
        Some(&self.entries[index].1)
    }

    pub fn contains_key<Q: Ord + ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        self.get(key).is_some()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&K, &V)> {
        self.entries.iter().map(|(key, value)| (key, value))
    }

    pub fn keys(&self) -> impl ExactSizeIterator<Item = &K> {
        self.entries.iter().map(|(key, _)| key)
    }

    pub fn values(&self) -> impl ExactSizeIterator<Item = &V> {
        self.entries.iter().map(|(_, value)| value)
    }

    /// Exposes the ordered backing slice for zero-allocation owner adapters.
    pub fn as_entries(&self) -> &[(K, V)] {
        &self.entries
    }

    /// Iterates over the requested key bounds without allocating scratch.
    pub fn range<Q, R>(&self, bounds: R) -> impl Iterator<Item = (&K, &V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        let start = match bounds.start_bound() {
            Bound::Included(key) => self.entries.partition_point(|(candidate, _)| candidate.borrow() < key),
            Bound::Excluded(key) => self.entries.partition_point(|(candidate, _)| candidate.borrow() <= key),
            Bound::Unbounded => 0,
        };
        let end = match bounds.end_bound() {
            Bound::Included(key) => self.entries.partition_point(|(candidate, _)| candidate.borrow() <= key),
            Bound::Excluded(key) => self.entries.partition_point(|(candidate, _)| candidate.borrow() < key),
            Bound::Unbounded => self.entries.len(),
        };
        let (start, end) = if start <= end { (start, end) } else { (0, 0) };
        self.entries[start..end].iter().map(|(key, value)| (key, value))
    }
}

/// Mutable, budget-aware builder for [`BudgetedSortedMap`].
pub struct BudgetedSortedMapBuilder<'a, K, V> {
    budget: &'a mut AllocationBudget,
    entries: Vec<(K, V)>,
}

impl<K: Ord, V> BudgetedSortedMapBuilder<'_, K, V> {
    pub fn push(&mut self, key: K, value: V) -> Result<(), AllocationError> {
        self.budget.grow_vector_exact(&mut self.entries, 1)?;
        self.entries.push((key, value));
        Ok(())
    }

    pub fn finish(mut self) -> Result<BudgetedSortedMap<K, V>, DuplicateKey> {
        self.entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        if self.entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(DuplicateKey);
        }
        Ok(BudgetedSortedMap { entries: self.entries })
    }
}

/// Immutable ordered set backed by one explicitly charged contiguous buffer.
#[derive(Debug, Eq, PartialEq)]
pub struct BudgetedSortedSet<K> {
    entries: Vec<K>,
}

impl<K: Ord> BudgetedSortedSet<K> {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains<Q: Ord + ?Sized>(&self, value: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        self.entries.binary_search_by(|candidate| candidate.borrow().cmp(value)).is_ok()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &K> {
        self.entries.iter()
    }

    pub fn range<Q, R>(&self, bounds: R) -> impl Iterator<Item = &K>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
        R: RangeBounds<Q>,
    {
        let start = match bounds.start_bound() {
            Bound::Included(value) => self.entries.partition_point(|candidate| candidate.borrow() < value),
            Bound::Excluded(value) => self.entries.partition_point(|candidate| candidate.borrow() <= value),
            Bound::Unbounded => 0,
        };
        let end = match bounds.end_bound() {
            Bound::Included(value) => self.entries.partition_point(|candidate| candidate.borrow() <= value),
            Bound::Excluded(value) => self.entries.partition_point(|candidate| candidate.borrow() < value),
            Bound::Unbounded => self.entries.len(),
        };
        let (start, end) = if start <= end { (start, end) } else { (0, 0) };
        self.entries[start..end].iter()
    }
}

/// Mutable, budget-aware builder for [`BudgetedSortedSet`].
pub struct BudgetedSortedSetBuilder<'a, K> {
    budget: &'a mut AllocationBudget,
    entries: Vec<K>,
}

impl<K: Ord> BudgetedSortedSetBuilder<'_, K> {
    pub fn push(&mut self, value: K) -> Result<(), AllocationError> {
        self.budget.grow_vector_exact(&mut self.entries, 1)?;
        self.entries.push(value);
        Ok(())
    }

    pub fn finish(mut self) -> Result<BudgetedSortedSet<K>, DuplicateKey> {
        self.entries.sort_unstable();
        if self.entries.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(DuplicateKey);
        }
        Ok(BudgetedSortedSet { entries: self.entries })
    }
}

impl ChildAllocationBudget {
    pub fn remaining(&self) -> usize {
        self.budget.remaining()
    }

    pub fn requested(&self) -> usize {
        self.budget.requested()
    }
}

impl std::ops::Deref for ChildAllocationBudget {
    type Target = AllocationBudget;

    fn deref(&self) -> &Self::Target {
        &self.budget
    }
}

impl std::ops::DerefMut for ChildAllocationBudget {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.budget
    }
}
