//! Cumulative allocator-requested bytes for one staged artifact load.
use std::alloc::Layout;

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
        self.reserve_array::<T>(count)?;
        let mut v = Vec::new();
        v.try_reserve_exact(count).map_err(|_| AllocationError::Failed)?;
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
        self.reserve_array::<T>(target)?;
        values.try_reserve_exact(additional).map_err(|_| AllocationError::Failed)
    }

    pub fn string(&mut self, value: &str) -> Result<String, AllocationError> {
        self.reserve_array::<u8>(value.len())?;
        let mut owned = String::new();
        owned.try_reserve_exact(value.len()).map_err(|_| AllocationError::Failed)?;
        owned.push_str(value);
        Ok(owned)
    }

    pub fn boxed<T>(&mut self, value: T) -> Result<Box<T>, AllocationError> {
        self.reserve_layout(Layout::new::<T>())?;
        Box::try_new(value).map_err(|_| AllocationError::Failed)
    }

    pub fn arc<T>(&mut self, value: T) -> Result<std::sync::Arc<T>, AllocationError> {
        // Arc allocates one block containing two reference counts and T. This
        // is the same checked layout used by the standard library Arc owner.
        let layout = Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(Layout::new::<T>())
            .map_err(|_| AllocationError::Overflow)?
            .0
            .pad_to_align();
        self.reserve_layout(layout)?;
        std::sync::Arc::try_new(value).map_err(|_| AllocationError::Failed)
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
