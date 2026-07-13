//! Native primitives on `Map`.
//!
//! Realizes the [ADR-0039](../../../docs/adr/0039-amend-floor-admit-collection-container-primitives.md)
//! floor for [ADR-0032](../../../docs/adr/0032-collections-representation-and-literals.md)'s
//! native `Map`: allocate, size, keyed get/put/has/remove, and indexed key/value
//! read (the last two back `Map#keys`/`Map#values`/`Map#each(_)`). These are
//! internal-only (`raw*`), wrapped by the `.ph`-defined public protocol
//! (`at(_)`/`at(_,put:)`/`size`/`includes(_)`/`remove(_)`/`keys`/`values`/
//! `each(_)`) in `core.ph`, except `new()`, which is a public primitive
//! directly (mirroring `List::new()`).
//!
//! ## The re-entrant key-hash crux
//!
//! Per [`crate::heap::map`]'s module doc, a key's bucket and equality are **Phalcom**
//! `hash`/`==`, sent via [`crate::vm::VM::send_dynamic`] — a re-entrant VM
//! call. `locate` (module-private) is the single place that performs this: it extracts every
//! candidate slot's key as an **owned** `Value` (`Value` is `Copy`) *before*
//! sending `==`, so no `&Heap` borrow is ever held across the send (the
//! `list_raw_at` arena discipline; `docs/forge/units/U-COLLTYPES/plan.md`
//! §Rubric).

use crate::error::PhResult;
use crate::heap::ObjRef;
use crate::primitive::{expect_map, is_mutable_collection_key, mutable_key_error, send_eq, send_hash};
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Map.class::new()` — allocates an empty map.
pub fn map_class_new(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::Obj(vm.heap.alloc_map()))
}

/// Signature: `Map::rawSize` — the map's entry count.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Map`.
pub fn map_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_map(vm, receiver)?;
    Ok(Value::Number(vm.heap.map(id).len() as f64))
}

/// Locates `key` in the map at `id`: computes its bucket (sends `hash`) and,
/// if a same-bucket entry's key sends `==` true against it, that entry's slot.
///
/// Never holds a `&Heap` borrow across the `hash`/`==` sends — see the module
/// doc's borrow-discipline note.
///
/// # Errors
///
/// Propagates any [`crate::error::RuntimeError`] raised by the `hash`/`==` sends
/// (e.g. a key whose `hash` tries to `Fiber.yield` under this native frame,
/// which correctly raises `CannotYieldAcrossNativeFrame` — ADR-0030 §4).
fn locate(vm: &mut VM, id: ObjRef, key: Value) -> PhResult<(i64, Option<usize>)> {
    let bucket = send_hash(vm, key)?;
    let candidates: Vec<usize> = vm.heap.map(id).bucket(bucket).to_vec();
    for slot in candidates {
        let Some((candidate_key, _)) = vm.heap.map(id).entry_at(slot) else {
            continue;
        };
        if send_eq(vm, candidate_key, key)? {
            return Ok((bucket, Some(slot)));
        }
    }
    Ok((bucket, None))
}

/// Signature: `Map::rawGet(_)` — value for `key`, or the `None` singleton if
/// absent (total, mirroring `list_raw_at`'s `Some`/`None`-free raw shape:
/// hit returns the raw value, miss returns `vm.none_value()`).
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Map`, or
/// propagates a `hash`/`==` send failure (see the module-private `locate`).
pub fn map_raw_get(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_map(vm, receiver)?;
    let (_, slot) = locate(vm, id, args[0])?;
    match slot {
        Some(s) => Ok(vm.heap.map(id).entry_at(s).expect("slot from locate() is live").1),
        None => Ok(vm.none_value()),
    }
}

/// Signature: `Map::rawPut(_,_)` — inserts or overwrites the entry for `key`,
/// returning the receiver (the `.ph` `at(_, put:)` wrapper's chainable
/// return).
///
/// Rejects a mutable-collection key (DEC-CT-C, `List`/`Map`/`Set`): its
/// identity `hash` is inconsistent with structural `==`
/// (collection-protocol.md law 4), which would silently corrupt the bucket
/// index if admitted.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Map`;
/// returns a raised catchable `Error` ([`crate::error::RuntimeError::Raise`]) if
/// `key` is a mutable collection; propagates a `hash`/`==` send failure.
pub fn map_raw_put(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_map(vm, receiver)?;
    let key = args[0];
    let value = args[1];
    if is_mutable_collection_key(vm, &key) {
        return Err(mutable_key_error(vm, "Map").into());
    }
    let (bucket, slot) = locate(vm, id, key)?;
    match slot {
        Some(s) => vm.heap.map_mut(id).set_value_at(s, value),
        None => vm.heap.map_mut(id).insert_new(bucket, key, value),
    }
    Ok(*receiver)
}

/// Signature: `Map::rawHas(_)` — whether `key` is present.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Map`, or
/// propagates a `hash`/`==` send failure.
pub fn map_raw_has(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_map(vm, receiver)?;
    let (_, slot) = locate(vm, id, args[0])?;
    Ok(Value::Bool(slot.is_some()))
}

/// Signature: `Map::rawRemove(_)` — deletes the entry for `key` if present;
/// idempotent (a missing key is a no-op), returning the receiver.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Map`, or
/// propagates a `hash`/`==` send failure.
pub fn map_raw_remove(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_map(vm, receiver)?;
    let (_, slot) = locate(vm, id, args[0])?;
    if let Some(s) = slot {
        vm.heap.map_mut(id).remove_at(s);
    }
    Ok(*receiver)
}

/// Extracts a non-negative integer index from `value` (mirrors
/// `primitive::list`'s identically-named helper).
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if `value` is not a non-negative
/// integer `Number`.
fn expect_index(value: &Value) -> PhResult<usize> {
    use crate::error::RuntimeError;
    let n = crate::expect_value!(value, Number);
    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 {
        return Err(RuntimeError::Type {
            expected: "a non-negative integer index",
            found: value.type_name(),
        }
        .into());
    }
    Ok(n as usize)
}

/// Signature: `Map::rawKeyAt(_)` — the key at insertion-order slot `i`, or the
/// `None` singleton if `i` is out of range. Backs `Map#keys`/`Map#each(_)`.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Map`, or
/// if `i` is not a non-negative integer.
pub fn map_raw_key_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_map(vm, receiver)?;
    let index = expect_index(&args[0])?;
    match vm.heap.map(id).entry_at(index) {
        Some((k, _)) => Ok(k),
        None => Ok(vm.none_value()),
    }
}

/// Signature: `Map::rawValueAt(_)` — the value at insertion-order slot `i`, or
/// the `None` singleton if `i` is out of range. Backs `Map#values`/`Map#each(_)`.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Map`, or
/// if `i` is not a non-negative integer.
pub fn map_raw_value_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_map(vm, receiver)?;
    let index = expect_index(&args[0])?;
    match vm.heap.map(id).entry_at(index) {
        Some((_, v)) => Ok(v),
        None => Ok(vm.none_value()),
    }
}
