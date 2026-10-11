//! Heap storage for the mutable exact-multiplicity hash collection.

use crate::error::MapMutationError;
use crate::value::Value;
use num_bigint::BigInt;
use num_traits::Zero;
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct Entry {
    key: Value,
    count: BigInt,
    cached_hash: i64,
}

/// Ordered distinct members with exact occurrence counts and cached hash
/// buckets. All Telos hash/equality sends are owned by the primitive layer.
#[derive(Clone, Debug, Default)]
pub struct MutableBagObject {
    entries: Vec<Entry>,
    index: HashMap<i64, Vec<usize>>,
    total_count: BigInt,
    reentrant_depth: u32,
}

impl MutableBagObject {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn total_count(&self) -> &BigInt {
        &self.total_count
    }

    pub fn bucket(&self, bucket: i64) -> &[usize] {
        self.index.get(&bucket).map_or(&[], Vec::as_slice)
    }

    pub fn entry_at(&self, slot: usize) -> Option<(Value, BigInt)> {
        self.entries.get(slot).map(|entry| (entry.key, entry.count.clone()))
    }

    pub fn key_at(&self, slot: usize) -> Option<Value> {
        self.entries.get(slot).map(|entry| entry.key)
    }

    pub fn count_at(&self, slot: usize) -> Option<BigInt> {
        self.entries.get(slot).map(|entry| entry.count.clone())
    }

    pub fn enter_reentrant_send(&mut self) {
        self.reentrant_depth = self.reentrant_depth.saturating_add(1);
    }

    pub fn exit_reentrant_send(&mut self) {
        self.reentrant_depth = self.reentrant_depth.saturating_sub(1);
    }

    pub fn is_locked(&self) -> bool {
        self.reentrant_depth != 0
    }

    /// Append a new distinct member and publish its positive count and total.
    pub fn insert_new(&mut self, bucket: i64, key: Value, count: BigInt, total: BigInt) -> Result<(), MapMutationError> {
        if self.is_locked() {
            return Err(MapMutationError::Locked);
        }
        debug_assert!(count > BigInt::zero());
        let slot = self.entries.len();
        self.entries.push(Entry {
            key,
            count,
            cached_hash: bucket,
        });
        self.index.entry(bucket).or_default().push(slot);
        self.total_count = total;
        Ok(())
    }

    /// Update an existing positive count and total without changing its slot.
    pub fn set_count(&mut self, slot: usize, count: BigInt, total: BigInt) -> Result<(), MapMutationError> {
        if self.is_locked() {
            return Err(MapMutationError::Locked);
        }
        let Some(entry) = self.entries.get_mut(slot) else {
            return Err(MapMutationError::OutOfRange);
        };
        debug_assert!(count > BigInt::zero());
        entry.count = count;
        self.total_count = total;
        Ok(())
    }

    /// Remove the entry at `slot` and repair indices using cached hashes only.
    pub fn remove_at(&mut self, slot: usize, total: BigInt) -> Result<(Value, BigInt), MapMutationError> {
        if self.is_locked() {
            return Err(MapMutationError::Locked);
        }
        if slot >= self.entries.len() {
            return Err(MapMutationError::OutOfRange);
        }
        let removed = self.entries.remove(slot);
        if let Some(slots) = self.index.get_mut(&removed.cached_hash) {
            slots.retain(|candidate| *candidate != slot);
            if slots.is_empty() {
                self.index.remove(&removed.cached_hash);
            }
        }
        for new_slot in slot..self.entries.len() {
            let hash = self.entries[new_slot].cached_hash;
            if let Some(slots) = self.index.get_mut(&hash) {
                if let Some(old_slot) = slots.iter_mut().find(|candidate| **candidate == new_slot + 1) {
                    *old_slot = new_slot;
                }
            }
        }
        self.total_count = total;
        Ok((removed.key, removed.count))
    }

    pub fn clear(&mut self) -> Result<(), MapMutationError> {
        if self.is_locked() {
            return Err(MapMutationError::Locked);
        }
        self.entries.clear();
        self.index.clear();
        self.total_count = BigInt::zero();
        Ok(())
    }

    pub fn entries(&self) -> impl Iterator<Item = (Value, BigInt)> + '_ {
        self.entries.iter().map(|entry| (entry.key, entry.count.clone()))
    }
}
