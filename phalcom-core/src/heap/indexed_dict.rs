//! Distinct ordered key/value storage for IndexedDict.

use crate::error::MapMutationError;
use crate::value::Value;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
struct Entry {
    key: Value,
    value: Value,
    cached_hash: i64,
}

#[derive(Clone, Debug, Default)]
pub struct IndexedDictObject {
    entries: Vec<Entry>,
    index: HashMap<i64, Vec<usize>>,
    reentrant_depth: u32,
}

impl IndexedDictObject {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn bucket(&self, hash: i64) -> &[usize] {
        self.index.get(&hash).map_or(&[], Vec::as_slice)
    }

    pub fn entry_at(&self, slot: usize) -> Option<(Value, Value)> {
        self.entries.get(slot).map(|entry| (entry.key, entry.value))
    }

    pub fn key_at(&self, slot: usize) -> Option<Value> {
        self.entries.get(slot).map(|entry| entry.key)
    }

    pub fn value_at(&self, slot: usize) -> Option<Value> {
        self.entries.get(slot).map(|entry| entry.value)
    }

    pub fn entries(&self) -> impl Iterator<Item = (Value, Value)> + '_ {
        self.entries.iter().map(|entry| (entry.key, entry.value))
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

    pub fn insert_new(&mut self, hash: i64, key: Value, value: Value) -> Result<(), MapMutationError> {
        if self.is_locked() {
            return Err(MapMutationError::Locked);
        }
        let slot = self.entries.len();
        self.entries.push(Entry { key, value, cached_hash: hash });
        self.index.entry(hash).or_default().push(slot);
        Ok(())
    }

    /// Existing-value replacement is non-structural, matching Map's callback
    /// linearization rule.
    pub fn set_value_at(&mut self, slot: usize, value: Value) -> Option<()> {
        let entry = self.entries.get_mut(slot)?;
        entry.value = value;
        Some(())
    }

    pub fn insert_at(&mut self, slot: usize, hash: i64, key: Value, value: Value) -> Result<(), MapMutationError> {
        if self.is_locked() {
            return Err(MapMutationError::Locked);
        }
        if slot > self.entries.len() {
            return Err(MapMutationError::OutOfRange);
        }
        let mut entries = self.entries.clone();
        entries.insert(slot, Entry { key, value, cached_hash: hash });
        let index = build_index(&entries);
        self.entries = entries;
        self.index = index;
        Ok(())
    }

    pub fn move_at(&mut self, from: usize, to: usize) -> Result<(), MapMutationError> {
        if self.is_locked() {
            return Err(MapMutationError::Locked);
        }
        if from >= self.entries.len() || to >= self.entries.len() {
            return Err(MapMutationError::OutOfRange);
        }
        if from == to {
            return Ok(());
        }
        let mut entries = self.entries.clone();
        let entry = entries.remove(from);
        entries.insert(to, entry);
        let index = build_index(&entries);
        self.entries = entries;
        self.index = index;
        Ok(())
    }

    pub fn remove_at(&mut self, slot: usize) -> Result<(Value, Value), MapMutationError> {
        if self.is_locked() {
            return Err(MapMutationError::Locked);
        }
        if slot >= self.entries.len() {
            return Err(MapMutationError::OutOfRange);
        }
        let mut entries = self.entries.clone();
        let removed = entries.remove(slot);
        let index = build_index(&entries);
        self.entries = entries;
        self.index = index;
        Ok((removed.key, removed.value))
    }

    pub fn clear(&mut self) -> Result<(), MapMutationError> {
        if self.is_locked() {
            return Err(MapMutationError::Locked);
        }
        self.entries.clear();
        self.index.clear();
        Ok(())
    }
}

fn build_index(entries: &[Entry]) -> HashMap<i64, Vec<usize>> {
    let mut index: HashMap<i64, Vec<usize>> = HashMap::new();
    for (slot, entry) in entries.iter().enumerate() {
        index.entry(entry.cached_hash).or_default().push(slot);
    }
    index
}
