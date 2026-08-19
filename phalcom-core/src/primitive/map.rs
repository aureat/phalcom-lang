//! Native primitives on `Map`.
//!
//! Realizes the [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)
//! floor for [ADR-0032](../../../docs/adr/accepted/0032-collections-representation-and-literals.md)'s
//! native `Map`: allocate, size, keyed get/put/has/remove, and indexed key/value
//! read (the last two back `Map#keys`/`Map#values`/`Map#each(_)`). These are
//! internal-only (`raw*`), wrapped by the `.ph`-defined public protocol
//! (`at(_)`/`at(_,put:)`/`size`/`includes(_)`/`remove(_)`/`keys`/`values`/
//! `each(_)`) in `core.ph`, except `new()`, which is a public primitive
//! directly (mirroring `List::new()`).
//!
//! ## The re-entrant key-hash crux
//!
//! Per the `heap::map` module doc, a key's bucket and equality are **Phalcom**
//! `hash`/`==`, sent via [`crate::vm::VM::send_dynamic`] — a re-entrant VM
//! call. `locate` (module-private) is the single place that performs this: it extracts every
//! candidate slot's key as an **owned** `Value` (`Value` is `Copy`) *before*
//! sending `==`, so no `&Heap` borrow is ever held across the send (the
//! `list_raw_at` arena discipline; `docs/forge/units/U-COLLTYPES/plan.md`
//! §Rubric).

use crate::error::{MapMutationError, PhResult, RuntimeError};
use crate::heap::ObjRef;
use crate::primitive::nil::wrap_some;
use crate::primitive::{expect_class, expect_map, is_mutable_collection_key, mutable_key_error, send_eq, send_hash};
use crate::value::Value;
use crate::vm::VM;

/// Converts a [`MapMutationError`] into the [`RuntimeError`] a `Map` raw
/// primitive should surface.
///
/// [`MapMutationError::Locked`] becomes the catchable
/// [`RuntimeError::ConcurrentMutation`] (the G0 reentrancy lock,
/// `docs/deferred/error-handling-followups.md` §1) — the real, reachable
/// case, hit when a key's `hash`/`==` calls back into a `Map::put_`/`remove_`
/// primitive on the same collection while [`locate`] holds the lock.
/// [`MapMutationError::OutOfRange`] becomes [`RuntimeError::Internal`] — a
/// defense-in-depth case that should be unreachable, since every caller
/// derives its `slot` from a fresh `locate` scan.
fn map_mutation_error(err: MapMutationError, collection: &'static str) -> RuntimeError {
    match err {
        MapMutationError::Locked => RuntimeError::ConcurrentMutation { collection },
        MapMutationError::OutOfRange => RuntimeError::Internal(format!("{collection} slot from locate() was out of range (internal invariant violation)")),
    }
}

/// Signature: `Map.class::new()` — allocates an empty map.
pub fn map_class_new(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::obj(vm.heap.alloc_map()))
}

/// Signature: `Map::size_` — the map's entry count.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Map`.
pub fn map_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_map(vm, receiver)?;
    Ok(Value::int(vm.heap.map(id).len() as i64))
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
pub(crate) fn locate_key(vm: &mut VM, id: ObjRef, key: Value) -> PhResult<(i64, Option<usize>)> {
    vm.heap.map_mut(id).enter_reentrant_send();
    let bucket_result = send_hash(vm, key);
    vm.heap.map_mut(id).exit_reentrant_send();
    let bucket = bucket_result?;

    let candidates: Vec<usize> = vm.heap.map(id).bucket(bucket).to_vec();
    for slot in candidates {
        let Some((candidate_key, _)) = vm.heap.map(id).entry_at(slot) else {
            continue;
        };
        let is_num_key = key.is_int() || key.is_float() || key.as_obj().is_some_and(|oid| vm.heap.as_large_int(oid).is_some());
        let is_num_cand = candidate_key.is_int() || candidate_key.is_float() || candidate_key.as_obj().is_some_and(|oid| vm.heap.as_large_int(oid).is_some());
        if is_num_key || is_num_cand {
            if crate::value::same_value_zero(candidate_key, key, &vm.heap) {
                return Ok((bucket, Some(slot)));
            }
        } else {
            vm.heap.map_mut(id).enter_reentrant_send();
            let eq_result = send_eq(vm, candidate_key, key);
            vm.heap.map_mut(id).exit_reentrant_send();
            if eq_result? {
                return Ok((bucket, Some(slot)));
            }
        }
    }
    Ok((bucket, None))
}

/// Signature: `Map::get_(_)` — `Some(value)` for a present key, including a
/// stored surface `None`, or immediate `None` if absent.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Map`, or
/// propagates a `hash`/`==` send failure (see the module-private `locate`).
pub fn map_raw_get(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_map(vm, receiver)?;
    let (_, slot) = locate_key(vm, id, args[0])?;
    match slot {
        Some(s) => Ok(wrap_some(vm, vm.heap.map(id).entry_at(s).expect("slot from locate() is live").1)?),
        None => Ok(vm.none_value()),
    }
}

/// Signature: `Map::put_(_,_)` — inserts or overwrites the entry for `key`,
/// returning `Some(previous_value)` for an existing key and `None` for a new
/// key.
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
    let (bucket, slot) = locate_key(vm, id, key)?;
    match slot {
        Some(s) => {
            let previous = vm.heap.map(id).entry_at(s).expect("slot from locate() is live").1;
            vm.heap
                .map_mut(id)
                .set_value_at(s, value)
                .ok_or_else(|| RuntimeError::Internal("Map slot from locate() was out of range (internal invariant violation)".to_string()))?;
            Ok(wrap_some(vm, previous)?)
        }
        None => {
            vm.heap
                .map_mut(id)
                .insert_new(bucket, key, value)
                .map_err(|err| map_mutation_error(err, "Map"))?;
            Ok(vm.none_value())
        }
    }
}

/// Inserts a B.3 literal association. Existing equal keys fail immediately.
pub(crate) fn map_literal_insert_unique(vm: &mut VM, id: ObjRef, key: Value, value: Value) -> PhResult<()> {
    if is_mutable_collection_key(vm, &key) {
        return Err(mutable_key_error(vm, "Map").into());
    }
    let (bucket, slot) = locate_key(vm, id, key)?;
    if slot.is_some() {
        return Err(duplicate_key_error(vm).into());
    }
    vm.heap
        .map_mut(id)
        .insert_new(bucket, key, value)
        .map_err(|err| map_mutation_error(err, "Map"))?;
    Ok(())
}

/// Builds B.3's ordinary catchable `DuplicateKeyError`. The class is defined
/// in `core.ph`, so resolving it from the loaded core module avoids widening
/// the Rust-installed class floor for one library-level error subtype.
fn duplicate_key_error(vm: &mut VM) -> RuntimeError {
    let rendered = "duplicate Map literal key".to_string();
    let core = vm.core_module().expect("core module is loaded before Map literals execute");
    let class_name = vm.interner.intern("DuplicateKeyError");
    let class_value = vm.heap.module(core).get(class_name).expect("DuplicateKeyError is defined by core.ph");
    let class_id = expect_class(vm, &class_value).expect("DuplicateKeyError global is a class");
    let field_count = vm.heap.class(class_id).field_count;
    let mut instance = crate::heap::InstanceObject::new(class_id, field_count);
    instance.slots[0] = vm.alloc_string_value(rendered.clone());
    let error = Value::obj(vm.heap.alloc(crate::heap::Object::Instance(instance)));
    RuntimeError::Raise {
        error,
        rendered,
        traceback: None,
        help: None,
    }
}

/// Signature: `Map::has_(_)` — whether `key` is present.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Map`, or
/// propagates a `hash`/`==` send failure.
pub fn map_raw_has(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_map(vm, receiver)?;
    let (_, slot) = locate_key(vm, id, args[0])?;
    Ok(Value::bool(slot.is_some()))
}

/// Signature: `Map::remove_(_)` — deletes the entry for `key` if present;
/// returning `Some(removed_value)` when present and `None` when absent.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Map`, or
/// propagates a `hash`/`==` send failure.
pub fn map_raw_remove(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_map(vm, receiver)?;
    let (_, slot) = locate_key(vm, id, args[0])?;
    if let Some(s) = slot {
        let (_, removed) = vm.heap.map_mut(id).remove_at(s).map_err(|err| map_mutation_error(err, "Map"))?;
        Ok(wrap_some(vm, removed)?)
    } else {
        Ok(vm.none_value())
    }
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
    if let Some(n) = value.as_int() {
        if n < 0 {
            Err(RuntimeError::Type {
                expected: "a non-negative integer index",
                found: "int",
            }
            .into())
        } else {
            Ok(n as usize)
        }
    } else if let Some(n) = value.as_float() {
        if !n.is_finite() || n < 0.0 || n.fract() != 0.0 {
            Err(RuntimeError::Type {
                expected: "a non-negative integer index",
                found: "float",
            }
            .into())
        } else {
            Ok(n as usize)
        }
    } else {
        Err(RuntimeError::Type {
            expected: "a non-negative integer index",
            found: value.type_name(),
        }
        .into())
    }
}

/// Signature: `Map::keyAt_(_)` — the key at insertion-order slot `i`, or the
/// immediate `None` if `i` is out of range. Backs `Map#keys`/`Map#each(_)`.
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

/// Signature: `Map::valueAt_(_)` — the value at insertion-order slot `i`, or
/// immediate `None` if `i` is out of range. Backs `Map#values`/`Map#each(_)`.
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
