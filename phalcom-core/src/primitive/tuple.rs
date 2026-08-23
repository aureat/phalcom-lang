//! Native primitives on `Tuple`.
//!
//! Realizes the [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)
//! floor for [ADR-0032](../../../docs/adr/accepted/0032-collections-representation-and-literals.md)'s
//! native `Tuple`: freeze a `List`'s elements into a fixed slice, length, and
//! indexed get. These are internal-only (`size_`/`at_`), wrapped by the
//! `.ph`-defined public protocol (`size`/`at(_)`/`each(_)`/`==`/`hash`) in
//! `core.ph`. Literal construction uses `BuildTuple`; `_$fromList` exists only
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

/// Internal `Tuple.class::_$fromList(_)` conversion bridge.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `args[0]` is not a `List`.
#[phalcom_native_macros::primitive(Tuple, "_$fromList(_)" , side = class, visibility = internal)]
pub fn tuple_from_list_internal(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let list_id: ObjRef = expect_list(vm, &args[0])?;
    let elements = vm.heap.list(list_id).elements().to_vec();
    finish_tuple(vm, elements, Vec::new()).map_err(|error| crate::product::runtime_error(vm, "Tuple label", error).into())
}

/// Signature: `Tuple::size_` — the tuple's fixed arity.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Tuple`.
#[phalcom_native_macros::primitive(Tuple, "_$size", visibility = internal)]
pub fn tuple_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_tuple(vm, receiver)?;
    Ok(Value::int(vm.heap.tuple(id).len() as i64))
}

#[phalcom_native_macros::primitive(Tuple, "_$at(_)", visibility = internal)]
pub fn tuple_raw_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_tuple(vm, receiver)?;
    let len = vm.heap.tuple(id).len();
    match super::index::normalize_element_index(&vm.heap, &args[0], len)? {
        super::index::NormalizedIndex::Valid(idx) => match vm.heap.tuple(id).get(idx) {
            Some(value) => Ok(value),
            None => Ok(vm.none_value()),
        },
        super::index::NormalizedIndex::OutOfRange => Ok(vm.none_value()),
    }
}

#[phalcom_native_macros::primitive(Tuple, "_$positionalSize", visibility = internal)]
pub fn tuple_raw_positional_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_tuple(vm, receiver)?;
    Ok(Value::int(vm.heap.tuple(id).positional_len() as i64))
}

#[phalcom_native_macros::primitive(Tuple, "_$labelAt(_)", visibility = internal)]
pub fn tuple_raw_label_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_tuple(vm, receiver)?;
    let len = vm.heap.tuple(id).labels().len();
    match super::index::normalize_element_index(&vm.heap, &args[0], len)? {
        super::index::NormalizedIndex::Valid(idx) => Ok(vm
            .heap
            .tuple(id)
            .labels()
            .get(idx)
            .copied()
            .map(Value::symbol)
            .unwrap_or_else(|| vm.none_value())),
        super::index::NormalizedIndex::OutOfRange => Ok(vm.none_value()),
    }
}

#[phalcom_native_macros::primitive(Tuple, "_$positionals", visibility = internal)]
pub fn tuple_raw_positionals(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_tuple(vm, receiver)?;
    let values = vm.heap.tuple(id).positionals().to_vec();
    finish_tuple(vm, values, Vec::new()).map_err(|error| crate::product::runtime_error(vm, "Tuple label", error).into())
}

#[phalcom_native_macros::primitive(Tuple, "_$labeled", visibility = internal)]
pub fn tuple_raw_labeled(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_tuple(vm, receiver)?;
    let entries = vm.heap.tuple(id).labeled_entries().collect();
    finish_tuple(vm, Vec::new(), entries).map_err(|error| crate::product::runtime_error(vm, "Tuple label", error).into())
}

/// Signature: `Tuple::slice_(_,_)` — rebuild a canonical half-open total-order slice.
///
/// `Range#sliceBounds_` owns bound interpretation. This primitive receives only
/// canonical coordinates and reconstructs through `finish_tuple` so a zero-length
/// result is the canonical `Unit` value. Values selected from the labeled suffix retain
/// their labels and encounter order.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] when the receiver is not a `Tuple`, either bound is
/// malformed, or the bounds do not satisfy `0 <= start <= end <= tuple size`.
#[phalcom_native_macros::primitive(Tuple, "_$slice(_,_)" , visibility = internal)]
pub fn tuple_raw_slice(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_tuple(vm, receiver)?;
    let start = expect_index(&args[0])?;
    let end = expect_index(&args[1])?;
    let tuple = vm.heap.tuple(id);
    if start > end || end > tuple.len() {
        return Err(RuntimeError::Type {
            expected: "slice bounds satisfying 0 <= start <= end <= tuple size",
            found: "invalid slice bounds",
        }
        .into());
    }

    let positional_len = tuple.positional_len();
    let values = tuple.values();
    let labels = tuple.labels();
    let mut positionals = Vec::with_capacity(end.saturating_sub(start).min(positional_len.saturating_sub(start)));
    let mut labeled = Vec::with_capacity(end.saturating_sub(positional_len).min(labels.len()));
    for index in start..end {
        if index < positional_len {
            positionals.push(values[index]);
        } else {
            labeled.push((labels[index - positional_len], values[index]));
        }
    }
    finish_tuple(vm, positionals, labeled).map_err(|error| crate::product::runtime_error(vm, "Tuple label", error).into())
}
