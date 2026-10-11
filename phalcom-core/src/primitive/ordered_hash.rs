//! Shared Telos key lookup for mutable ordered hash collections.
//!
//! Map and Set use distinct heap variants and accessors but currently share
//! `MapObject` storage. Keep key hashing, collision equality, numeric
//! SameValueZero handling, and re-entrant callback locking in this one place.
//! No heap borrow may remain live while a key's Telos `hash` or `==` runs.

use crate::error::PhResult;
use crate::heap::ObjRef;
use crate::value::Value;
use crate::vm::VM;

use super::{send_eq, send_hash};

/// The heap arm whose ordered hash backing is being searched.
#[derive(Clone, Copy)]
pub(super) enum KeyedCollectionKind {
    Map,
    Set,
}

impl KeyedCollectionKind {
    fn enter_reentrant_send(self, vm: &mut VM, id: ObjRef) {
        match self {
            Self::Map => vm.heap.map_mut(id).enter_reentrant_send(),
            Self::Set => vm.heap.set_mut(id).enter_reentrant_send(),
        }
    }

    fn exit_reentrant_send(self, vm: &mut VM, id: ObjRef) {
        match self {
            Self::Map => vm.heap.map_mut(id).exit_reentrant_send(),
            Self::Set => vm.heap.set_mut(id).exit_reentrant_send(),
        }
    }

    fn candidate_key(self, vm: &VM, id: ObjRef, slot: usize) -> Option<Value> {
        match self {
            Self::Map => vm.heap.map(id).entry_at(slot).map(|(key, _)| key),
            Self::Set => vm.heap.set(id).key_at(slot),
        }
    }

    fn candidate_slots(self, vm: &VM, id: ObjRef, bucket: i64) -> Vec<usize> {
        match self {
            Self::Map => vm.heap.map(id).bucket(bucket).to_vec(),
            Self::Set => vm.heap.set(id).bucket(bucket).to_vec(),
        }
    }
}

/// Sends a key callback while the target collection rejects re-entrant
/// structural mutation. The collection borrow ends before `send` executes,
/// and the lock is released before propagating either success or failure.
fn with_reentrant_send<T>(vm: &mut VM, id: ObjRef, kind: KeyedCollectionKind, send: impl FnOnce(&mut VM) -> PhResult<T>) -> PhResult<T> {
    kind.enter_reentrant_send(vm, id);
    let result = send(vm);
    kind.exit_reentrant_send(vm, id);
    result
}

fn is_numeric_key(vm: &VM, value: Value) -> bool {
    value.is_int() || value.is_float() || value.as_obj().is_some_and(|oid| vm.heap.as_large_int(oid).is_some())
}

/// Locates `key`, returning its Telos hash bucket and matching ordered slot.
///
/// Hash and equality are Telos sends. Candidate values and slot lists are
/// copied out before any callback, so no heap borrow crosses a re-entrant VM
/// call. Hash collisions are disambiguated with Telos `==`; numeric values
/// retain the runtime's SameValueZero fast path.
pub(super) fn locate_key(vm: &mut VM, id: ObjRef, key: Value, kind: KeyedCollectionKind) -> PhResult<(i64, Option<usize>)> {
    let bucket = with_reentrant_send(vm, id, kind, |vm| send_hash(vm, key))?;
    let candidates = kind.candidate_slots(vm, id, bucket);
    let numeric_key = is_numeric_key(vm, key);

    for slot in candidates {
        let Some(candidate_key) = kind.candidate_key(vm, id, slot) else {
            continue;
        };

        if numeric_key || is_numeric_key(vm, candidate_key) {
            if crate::value::same_value_zero(candidate_key, key, &vm.heap) {
                return Ok((bucket, Some(slot)));
            }
        } else if with_reentrant_send(vm, id, kind, |vm| send_eq(vm, candidate_key, key))? {
            return Ok((bucket, Some(slot)));
        }
    }

    Ok((bucket, None))
}
