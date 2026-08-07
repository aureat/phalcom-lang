//! Native primitives on `Tuple`.
//!
//! Realizes the [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)
//! floor for [ADR-0032](../../../docs/adr/accepted/0032-collections-representation-and-literals.md)'s
//! native `Tuple`: freeze a `List`'s elements into a fixed slice, length, and
//! indexed get. These are internal-only (`size_`/`at_`), wrapped by the
//! `.ph`-defined public protocol (`size`/`at(_)`/`each(_)`/`==`/`hash`) in
//! `core.ph`. Literal construction uses `BuildTuple`; `__fromList` exists only
//! as an internal conversion bridge for native-backed library values.
//!
//! **No mutation primitive exists** — `Tuple`'s immutability is a
//! representation guarantee ([`crate::heap::TupleObject`]'s `Box<[Value]>`),
//! not merely an absent selector.

use crate::error::{PhResult, RuntimeError};
use crate::heap::ObjRef;
use crate::primitive::{expect_list, expect_tuple};
use crate::product::finish_tuple;
use crate::value::Value;
use crate::vm::VM;

/// Extracts a non-negative integer index from `value` (mirrors
/// `primitive::list`'s identically-named helper).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a non-negative integer
/// `Number`.
pub(crate) fn expect_index(value: &Value) -> PhResult<usize> {
    match value {
        Value::Int(n) => {
            if *n < 0 {
                Err(RuntimeError::Type {
                    expected: "a non-negative integer index",
                    found: "int",
                }
                .into())
            } else {
                Ok(*n as usize)
            }
        }
        Value::Float(n) => {
            if !n.is_finite() || *n < 0.0 || n.fract() != 0.0 {
                Err(RuntimeError::Type {
                    expected: "a non-negative integer index",
                    found: "float",
                }
                .into())
            } else {
                Ok(*n as usize)
            }
        }
        other => Err(RuntimeError::Type {
            expected: "a non-negative integer index",
            found: other.type_name(),
        }
        .into()),
    }
}

/// Internal `Tuple.class::__fromList(_)` conversion bridge.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `args[0]` is not a `List`.
pub fn tuple_from_list_internal(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let list_id: ObjRef = expect_list(vm, &args[0])?;
    let elements = vm.heap.list(list_id).elements().to_vec();
    finish_tuple(vm, elements, Vec::new()).map_err(|error| RuntimeError::Internal(format!("tuple conversion failed: {error:?}")).into())
}

/// Signature: `Tuple::size_` — the tuple's fixed arity.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Tuple`.
pub fn tuple_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_tuple(vm, receiver)?;
    Ok(Value::Int(vm.heap.tuple(id).len() as i64))
}

/// Signature: `Tuple::at_(_)` — raw indexed read, total (mirrors
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

pub fn tuple_raw_positional_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_tuple(vm, receiver)?;
    Ok(Value::Int(vm.heap.tuple(id).positional_len() as i64))
}

pub fn tuple_raw_label_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_tuple(vm, receiver)?;
    let index = expect_index(&args[0])?;
    Ok(vm
        .heap
        .tuple(id)
        .labels()
        .get(index)
        .copied()
        .map(Value::Symbol)
        .unwrap_or_else(|| vm.none_value()))
}

pub fn tuple_raw_positionals(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_tuple(vm, receiver)?;
    let values = vm.heap.tuple(id).positionals().to_vec();
    finish_tuple(vm, values, Vec::new()).map_err(|error| RuntimeError::Internal(format!("tuple projection failed: {error:?}")).into())
}

pub fn tuple_raw_labeled(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_tuple(vm, receiver)?;
    let entries = vm.heap.tuple(id).labeled_entries().collect();
    finish_tuple(vm, Vec::new(), entries).map_err(|error| RuntimeError::Internal(format!("tuple projection failed: {error:?}")).into())
}
