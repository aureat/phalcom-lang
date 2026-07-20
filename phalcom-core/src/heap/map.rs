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

use crate::error::MapMutationError;
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
    /// Reentrant-send depth: nonzero while a `locate` call (in
    /// [`crate::primitive::map`]/[`crate::primitive::set`]) has a `hash`/`==`
    /// send in flight against this collection's own key. A counter, not a
    /// bool, so a read-only reentrant call (e.g. one key's `==` reading
    /// `m.at(_)` on the same map) nests correctly without the inner call's
    /// exit clearing the outer call's lock. See [`Self::enter_reentrant_send`].
    reentrant_depth: u32,
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

    /// Overwrites the value at an existing `slot`. Not a structural mutation
    /// (no slot creation/removal/reindex), so — unlike [`Self::insert_new`]/
    /// [`Self::remove_at`] — this is **not** guarded by
    /// [`Self::is_locked`]: overwriting an existing key's value in place
    /// remains legal from within that same key's `hash`/`==`
    /// (`docs/deferred/error-handling-followups.md` §1).
    ///
    /// Returns `None` if `slot` is out of range instead of panicking
    /// (defense-in-depth, same rationale as [`MapMutationError::OutOfRange`]) —
    /// callers only reach this after a [`Self::bucket`] scan confirms `slot`
    /// is a live entry, so `None` should be unreachable in practice.
    pub fn set_value_at(&mut self, slot: usize, value: Value) -> Option<()> {
        let entry = self.entries.get_mut(slot)?;
        entry.1 = value;
        Some(())
    }

    /// Appends a fresh `(key, value)` entry and indexes it under `bucket`.
    ///
    /// Does **not** check for an existing key at `bucket` — the caller (the
    /// `put_`/`add_` primitive) must have already confirmed no live entry
    /// matches `key` (via [`Self::bucket`] + a Phalcom `==` scan) before
    /// calling this, or the map gains a duplicate key.
    ///
    /// # Errors
    ///
    /// Returns [`MapMutationError::Locked`] if this collection is currently
    /// inside a reentrant `hash`/`==` send window
    /// ([`Self::is_locked`]) — a key's `hash`/`==` may not structurally
    /// mutate the collection it is being compared for.
    pub fn insert_new(&mut self, bucket: i64, key: Value, value: Value) -> Result<(), MapMutationError> {
        if self.is_locked() {
            return Err(MapMutationError::Locked);
        }
        let slot = self.entries.len();
        self.entries.push((key, value, bucket));
        self.index.entry(bucket).or_default().push(slot);
        Ok(())
    }

    /// Removes the entry at `slot` (swap-remove), maintaining the index from
    /// each entry's own cached bucket — no re-hashing.
    ///
    /// # Errors
    ///
    /// Returns [`MapMutationError::Locked`] if this collection is currently
    /// inside a reentrant `hash`/`==` send window ([`Self::is_locked`]).
    /// Returns [`MapMutationError::OutOfRange`] if `slot` is out of range
    /// (defense-in-depth — every caller derives `slot` from a fresh
    /// [`Self::bucket`] scan, so this should be unreachable in practice).
    pub fn remove_at(&mut self, slot: usize) -> Result<(Value, Value), MapMutationError> {
        if self.is_locked() {
            return Err(MapMutationError::Locked);
        }
        if slot >= self.entries.len() {
            return Err(MapMutationError::OutOfRange);
        }
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

        Ok((key, value))
    }

    /// Borrows every entry (key, value, cached bucket), in insertion order.
    pub fn entries(&self) -> impl Iterator<Item = (Value, Value)> + '_ {
        self.entries.iter().map(|&(k, v, _)| (k, v))
    }

    /// Returns `true` while a reentrant `hash`/`==` send is in flight against
    /// this collection's own key ([`Self::enter_reentrant_send`]/
    /// [`Self::exit_reentrant_send`]). [`Self::insert_new`]/[`Self::remove_at`]
    /// consult this to reject a structural mutation attempted from within
    /// that send.
    pub fn is_locked(&self) -> bool {
        self.reentrant_depth > 0
    }

    /// Marks the start of a reentrant `hash`/`==` send window
    /// (`crate::primitive::map`'s/`crate::primitive::set`'s module-private
    /// `locate`), incrementing the reentrancy depth counter. Must be paired with
    /// [`Self::exit_reentrant_send`] on **every** exit path of the send,
    /// including an `Err` return — callers use a guard pattern or an
    /// explicit clear-before-`?` to guarantee this (never leave the
    /// collection permanently locked after a failed send).
    pub fn enter_reentrant_send(&mut self) {
        self.reentrant_depth += 1;
    }

    /// Marks the end of a reentrant `hash`/`==` send window, decrementing the
    /// reentrancy depth counter. See [`Self::enter_reentrant_send`].
    pub fn exit_reentrant_send(&mut self) {
        debug_assert!(self.reentrant_depth > 0, "exit_reentrant_send without a matching enter");
        self.reentrant_depth = self.reentrant_depth.saturating_sub(1);
    }
}
