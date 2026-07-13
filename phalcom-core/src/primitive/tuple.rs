//! Native primitives on `Tuple`.
//!
//! Realizes the [ADR-0039](../../../docs/adr/0039-amend-floor-admit-collection-container-primitives.md)
//! floor for [ADR-0032](../../../docs/adr/0032-collections-representation-and-literals.md)'s
//! native `Tuple`: freeze a `List`'s elements into a fixed slice, length, and
//! indexed get. These are internal-only (`rawSize`/`rawAt`), wrapped by the
//! `.ph`-defined public protocol (`size`/`at(_)`/`each(_)`/`==`/`hash`) in
//! `core.ph`, except `fromList(_)`, which is a public static primitive
//! directly (mirroring `List::new()`) — the construction path until the
//! `(a, b)` literal's parser lowering (already shipped by U-COLL) resolves
//! against this class.
//!
//! **No mutation primitive exists** — `Tuple`'s immutability is a
//! representation guarantee ([`crate::heap::TupleObject`]'s `Box<[Value]>`),
//! not merely an absent selector.

use crate::error::{PhResult, RuntimeError};
use crate::heap::ObjRef;
use crate::primitive::{expect_list, expect_tuple};
use crate::value::Value;
use crate::vm::VM;

/// Extracts a non-negative integer index from `value` (mirrors
/// `primitive::list`'s identically-named helper).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a non-negative integer
/// `Number`.
fn expect_index(value: &Value) -> PhResult<usize> {
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

/// Signature: `Tuple.class::fromList(_)` — freezes a `List`'s current
/// elements into a fresh, fixed-length `Tuple`.
///
/// The `(a, b)` literal's parser lowering (U-COLL) targets exactly this send
/// (`Tuple.fromList(List.new().add(a).add(b))`). Copies the source list's
/// elements rather than aliasing its buffer, so a later mutation of the
/// source `List` cannot retroactively "mutate" the frozen `Tuple`.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `args[0]` is not a `List`.
pub fn tuple_class_from_list(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let list_id: ObjRef = expect_list(vm, &args[0])?;
    let elements: Box<[Value]> = vm.heap.list(list_id).elements().to_vec().into_boxed_slice();
    Ok(Value::Obj(vm.heap.alloc_tuple(elements)))
}

/// Signature: `Tuple::rawSize` — the tuple's fixed arity.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Tuple`.
pub fn tuple_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_tuple(vm, receiver)?;
    Ok(Value::Number(vm.heap.tuple(id).len() as f64))
}

/// Signature: `Tuple::rawAt(_)` — raw indexed read, total (mirrors
/// `list_raw_at`: hit returns the raw element, miss returns the `None`
/// singleton — never a panic, never the raw `nil` sentinel).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Tuple`, or if the
/// index is not a non-negative integer `Number`.
pub fn tuple_raw_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_tuple(vm, receiver)?;
    let index = expect_index(&args[0])?;
    match vm.heap.tuple(id).get(index) {
        Some(value) => Ok(value),
        None => Ok(vm.none_value()),
    }
}
