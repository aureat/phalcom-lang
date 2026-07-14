//! A native, heap-backed insertion-ordered hash map.
//!
//! Realizes [ADR-0032 §1](../../../docs/adr/accepted/0032-collections-representation-and-literals.md)
//! (native heap-arm representation) and
//! [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)
//! (the raw-primitive floor amendment): `Map` is a dedicated
//! [`crate::heap::Object::Map`] heap variant, mirroring
//! [`crate::heap::ListObject`] — **not** an [`crate::heap::InstanceObject`].
//! There is no `Rc`/`RefCell`: mutation goes through `&mut Heap` like every
//! other heap object ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)).
//!
//! [`Object::Set`](crate::heap::Object::Set) reuses this same backing struct
//! (DEC-CT-B, `docs/forge/units/U-COLLTYPES/plan.md` §8) — a set is a
//! keys-only ordered hash map, so [`MapObject`] stores a `Value` "value" slot
//! per entry that a `Set` binding simply never reads (always [`Value::Nil`]
//! there). This halves the re-entrant-hash review surface: one borrow-model
//! proof covers both classes.
//!
//! ## The key-hashing crux
//!
//! Per [map-and-set.md §2](../../../docs/spec/v0.2/core/map-and-set.md),
//! `Map`/`Set` key by **Phalcom** `hash`+`==`, not Rust's `Value: Hash`/
//! [`Value::value_eq`](crate::value::Value::value_eq) (which give *identity*
//! for `Value::Obj` — wrong for a value-hashable `Tuple` key). So the raw
//! primitives ([`crate::primitive::map`]) must **re-enter the VM** to send
//! `hash`/`==` on keys, extracting the [`crate::heap::ObjRef`] first and
//! re-resolving it after every send (the `list_raw_at` arena-borrow
//! discipline) — never holding a `&Heap`/`&mut Heap` borrow of this struct
//! across a re-entrant send.
//!
//! This struct itself is pure data and never re-sends `hash`: every entry
//! caches the **already-computed** bucket it was inserted under
//! ([`MapObject::insert_new`]'s `bucket` argument), so
//! [`MapObject::remove_at`] can maintain the index purely from that cached
//! value — no rehashing, no re-entrant send, ever needed for internal
//! bookkeeping.

use crate::value::Value;
use std::collections::HashMap;

/// One stored `(key, value)` entry plus its cached Phalcom-`hash` bucket.
type Entry = (Value, Value, i64);

/// A native, insertion-ordered hash-map backing store.
///
/// `entries` is the source of truth for iteration order (map-and-set.md §2:
/// "stable within a run") and for the value at a given slot; `index` maps a
/// key's Phalcom `hash` (truncated to `i64` — see `primitive::map::locate`) to
/// the `entries` slots whose key hashes there, so a lookup is O(1) expected
/// plus a short `==`-disambiguation scan of same-bucket candidates (hash
/// collisions are correctness, not a bug — the census docs this).
#[derive(Debug, Clone, Default)]
pub struct MapObject {
    /// The map's entries, in insertion order, each tagged with the bucket
    /// its key hashed to at insertion time. A `Set` binding always leaves the
    /// `.1` (value) slot as [`Value::Nil`] — never surfaced, since `Set`'s
    /// `.ph` protocol never reads it.
    entries: Vec<Entry>,
    /// Bucket index: a key's Phalcom `hash` (truncated `i64`) to the
    /// `entries` slots whose key hashed there.
    index: HashMap<i64, Vec<usize>>,
}

impl MapObject {
    /// Builds an empty map/set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the entry count.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the map/set has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the candidate entry slots whose key hashed to `bucket`.
    ///
    /// The caller (a raw primitive) still must send `==` to each candidate's
    /// key to find the true match — this only narrows the scan (hash
    /// collisions are expected, not an error).
    pub fn bucket(&self, bucket: i64) -> &[usize] {
        self.index.get(&bucket).map_or(&[], |v| v.as_slice())
    }

    /// Returns the `(key, value)` pair at `slot`, or `None` if out of range.
    pub fn entry_at(&self, slot: usize) -> Option<(Value, Value)> {
        self.entries.get(slot).map(|&(k, v, _)| (k, v))
    }

    /// Returns the key at `slot`, in insertion order, or `None` if out of
    /// range — the `Set`-facing counterpart of [`Self::entry_at`].
    pub fn key_at(&self, slot: usize) -> Option<Value> {
        self.entries.get(slot).map(|&(k, _, _)| k)
    }

    /// Overwrites the value at an existing `slot`.
    ///
    /// # Panics
    ///
    /// Panics if `slot` is out of range — callers only reach this after a
    /// [`Self::bucket`] scan confirms `slot` is a live entry.
    pub fn set_value_at(&mut self, slot: usize, value: Value) {
        self.entries[slot].1 = value;
    }

    /// Appends a fresh `(key, value)` entry and indexes it under `bucket`.
    ///
    /// Does **not** check for an existing key at `bucket` — the caller (the
    /// `put_`/`add_` primitive) must have already confirmed no live entry
    /// matches `key` (via [`Self::bucket`] + a Phalcom `==` scan) before
    /// calling this, or the map gains a duplicate key.
    pub fn insert_new(&mut self, bucket: i64, key: Value, value: Value) {
        let slot = self.entries.len();
        self.entries.push((key, value, bucket));
        self.index.entry(bucket).or_default().push(slot);
    }

    /// Removes the entry at `slot` (swap-remove), maintaining the index from
    /// each entry's own cached bucket — no re-hashing.
    ///
    /// # Panics
    ///
    /// Panics if `slot` is out of range.
    pub fn remove_at(&mut self, slot: usize) -> (Value, Value) {
        let last = self.entries.len() - 1;
        let (key, value, bucket) = self.entries.swap_remove(slot);

        if let Some(candidates) = self.index.get_mut(&bucket) {
            candidates.retain(|&s| s != slot);
            if candidates.is_empty() {
                self.index.remove(&bucket);
            }
        }

        // `swap_remove` moved the former last entry into `slot` (unless
        // `slot` *was* the last entry) — retarget its index row from `last`
        // to `slot` using its own cached bucket, no rehash needed.
        if slot != last {
            let moved_bucket = self.entries[slot].2;
            if let Some(candidates) = self.index.get_mut(&moved_bucket) {
                for s in candidates.iter_mut() {
                    if *s == last {
                        *s = slot;
                    }
                }
            }
        }

        (key, value)
    }

    /// Borrows every entry (key, value, cached bucket), in insertion order.
    pub fn entries(&self) -> impl Iterator<Item = (Value, Value)> + '_ {
        self.entries.iter().map(|&(k, v, _)| (k, v))
    }
}
