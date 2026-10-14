//! Cumulative allocator-requested bytes for one staged artifact load.
use std::alloc::Layout;

/// A load owns a private copy; the caller commits remaining bytes only on success.
/// Allocator-internal bookkeeping is excluded. Temporary allocations are not refunded.
#[derive(Clone, Debug)]
pub struct AllocationBudget {
    remaining: usize,
    requested: usize,
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
    pub fn reserve_layout(&mut self, layout: Layout) -> Result<(), AllocationError> {
        let remaining = self.remaining.checked_sub(layout.size()).ok_or(AllocationError::Exhausted)?;
        let requested = self.requested.checked_add(layout.size()).ok_or(AllocationError::Overflow)?;
        self.remaining = remaining;
        self.requested = requested;
        Ok(())
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
}
