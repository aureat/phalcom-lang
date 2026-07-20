//! Native primitives on `Set`.
//!
//! Realizes the [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)
//! floor for [ADR-0032](../../../docs/adr/accepted/0032-collections-representation-and-literals.md)'s
//! native `Set` — a keys-only [`crate::heap::MapObject`] (DEC-CT-B,
//! `docs/forge/units/U-COLLTYPES/plan.md` §8): allocate, size, membership
//! add/has/remove, and indexed read (backs `Set#each(_)`). These are
//! internal-only (`raw*`), wrapped by the `.ph`-defined public protocol
//! (`add(_)`/`includes(_)`/`size`/`remove(_)`/`each(_)`) in `core.ph`, except
//! `new()`, which is a public primitive directly (mirroring `List::new()`).
//!
//! The re-entrant key-hash discipline is identical to [`crate::primitive::map`]'s
//! [`crate::primitive::map`] `locate` — see that module's doc for the
//! borrow-model proof; `locate` here (module-private) is its `Object::Set`-typed twin (same
//! algorithm, different heap accessor, since `Object::Map` and `Object::Set`
//! are distinct heap variants over the same [`crate::heap::MapObject`] backing
//! struct and so need distinct `vm.heap.set`/`set_mut` calls).

use crate::error::{MapMutationError, PhResult, RuntimeError};
use crate::heap::ObjRef;
use crate::primitive::{expect_set, is_mutable_collection_key, mutable_key_error, send_eq, send_hash};
use crate::value::Value;
use crate::vm::VM;

/// Converts a [`MapMutationError`] into the [`RuntimeError`] a `Set` raw
/// primitive should surface — the `Object::Set` twin of
/// `crate::primitive::map`'s identically-shaped private helper.
/// [`MapMutationError::Locked`] becomes the catchable
/// [`RuntimeError::ConcurrentMutation`] (the G0 reentrancy lock,
/// `docs/deferred/error-handling-followups.md` §1); [`MapMutationError::OutOfRange`]
/// becomes [`RuntimeError::Internal`] (defense-in-depth, should be
/// unreachable).
fn set_mutation_error(err: MapMutationError, collection: &'static str) -> RuntimeError {
    match err {
        MapMutationError::Locked => RuntimeError::ConcurrentMutation { collection },
        MapMutationError::OutOfRange => {
            RuntimeError::Internal(format!("{collection} slot from locate() was out of range (internal invariant violation)"))
        }
    }
}

/// Signature: `Set.class::new()` — allocates an empty set.
pub fn set_class_new(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::Obj(vm.heap.alloc_set()))
}

/// Signature: `Set::size_` — the set's element count.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Set`.
pub fn set_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_set(vm, receiver)?;
    Ok(Value::Number(vm.heap.set(id).len() as f64))
}

/// Locates `key` in the set at `id` — the `Object::Set` twin of
/// [`crate::primitive::map`]'s `locate`; see that module's doc for the
/// borrow-model proof (no `&Heap` borrow held across the `hash`/`==` sends)
/// and the G0 reentrancy lock it applies around each `hash`/`==` send
/// (`docs/deferred/error-handling-followups.md` §1).
///
/// # Errors
///
/// Propagates any [`crate::error::RuntimeError`] raised by the `hash`/`==` sends.
fn locate(vm: &mut VM, id: ObjRef, key: Value) -> PhResult<(i64, Option<usize>)> {
    vm.heap.set_mut(id).enter_reentrant_send();
    let bucket_result = send_hash(vm, key);
    vm.heap.set_mut(id).exit_reentrant_send();
    let bucket = bucket_result?;

    let candidates: Vec<usize> = vm.heap.set(id).bucket(bucket).to_vec();
    for slot in candidates {
        let Some(candidate_key) = vm.heap.set(id).key_at(slot) else {
            continue;
        };
        vm.heap.set_mut(id).enter_reentrant_send();
        let eq_result = send_eq(vm, candidate_key, key);
        vm.heap.set_mut(id).exit_reentrant_send();
        if eq_result? {
            return Ok((bucket, Some(slot)));
        }
    }
    Ok((bucket, None))
}

/// Signature: `Set::add_(_)` — inserts `key` if absent (idempotent),
/// returning the receiver (the `.ph` `add(_)` wrapper's chainable return).
///
/// Rejects a mutable-collection key (DEC-CT-C) — see
/// [`crate::primitive::map::map_raw_put`]'s identical rationale.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Set`;
/// returns a raised catchable `Error` if `key` is a mutable collection;
/// propagates a `hash`/`==` send failure.
pub fn set_raw_add(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_set(vm, receiver)?;
    let key = args[0];
    if is_mutable_collection_key(vm, &key) {
        return Err(mutable_key_error(vm, "Set").into());
    }
    let (bucket, slot) = locate(vm, id, key)?;
    if slot.is_none() {
        // Set entries carry no meaningful value slot (DEC-CT-B) — Nil, never
        // surfaced by `Set`'s `.ph` protocol.
        vm.heap
            .set_mut(id)
            .insert_new(bucket, key, Value::Nil)
            .map_err(|err| set_mutation_error(err, "Set"))?;
    }
    Ok(*receiver)
}

/// Signature: `Set::has_(_)` — membership test.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Set`, or
/// propagates a `hash`/`==` send failure.
pub fn set_raw_has(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_set(vm, receiver)?;
    let (_, slot) = locate(vm, id, args[0])?;
    Ok(Value::Bool(slot.is_some()))
}

/// Signature: `Set::remove_(_)` — deletes `key` if present; idempotent (a
/// missing key is a no-op), returning the receiver.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Set`, or
/// propagates a `hash`/`==` send failure.
pub fn set_raw_remove(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_set(vm, receiver)?;
    let (_, slot) = locate(vm, id, args[0])?;
    if let Some(s) = slot {
        vm.heap.set_mut(id).remove_at(s).map_err(|err| set_mutation_error(err, "Set"))?;
    }
    Ok(*receiver)
}

/// Signature: `Set::at_(_)` — the element at insertion-order slot `i`, or
/// the `None` singleton if `i` is out of range (total, mirrors
/// `list_raw_at`). Backs `Set#each(_)`.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Set`, or
/// if `i` is not a non-negative integer.
pub fn set_raw_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    use crate::error::RuntimeError;
    let id: ObjRef = expect_set(vm, receiver)?;
    let n = crate::expect_value!(&args[0], Number);
    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 {
        return Err(RuntimeError::Type {
            expected: "a non-negative integer index",
            found: args[0].type_name(),
        }
        .into());
    }
    let index = n as usize;
    match vm.heap.set(id).key_at(index) {
        Some(k) => Ok(k),
        None => Ok(vm.none_value()),
    }
}
