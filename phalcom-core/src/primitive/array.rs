//! Native primitives on `Array`.
//!
//! Implements the minimal primitive floor for `Array` (rawLength, rawAt, rawSet)
//! under COLL006.C1.P2.

use crate::error::{PhResult, RuntimeError};
use crate::heap::ObjRef;
use crate::primitive::expect_array;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Array::rawLength` — the array's fixed element count.
#[phalcom_native_macros::primitive(Array, "rawLength", visibility = internal)]
pub fn array_raw_length(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_array(vm, receiver)?;
    Ok(Value::int(vm.heap.array(id).len() as i64))
}

/// Signature: `Array::rawAt(_)` — raw indexed read.
/// Returns None on out of range.
#[phalcom_native_macros::primitive(Array, "rawAt(_)", visibility = internal)]
pub fn array_raw_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_array(vm, receiver)?;
    let len = vm.heap.array(id).len();
    match super::index::normalize_element_index(&vm.heap, &args[0], len)? {
        super::index::NormalizedIndex::Valid(idx) => match vm.heap.array(id).get(idx) {
            Some(value) => Ok(value),
            None => Ok(vm.none_value()),
        },
        super::index::NormalizedIndex::OutOfRange => Ok(vm.none_value()),
    }
}

/// Signature: `Array::rawSet(_,_)` — raw indexed write.
#[phalcom_native_macros::primitive(Array, "rawSet(_,_)", visibility = internal)]
pub fn array_raw_set(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_array(vm, receiver)?;
    let len = vm.heap.array(id).len();
    let index = match super::index::normalize_element_index(&vm.heap, &args[0], len)? {
        super::index::NormalizedIndex::Valid(idx) => idx,
        super::index::NormalizedIndex::OutOfRange => {
            return Err(RuntimeError::Type {
                expected: "an in-range index",
                found: "an out-of-range Number",
            }
            .into());
        }
    };
    match vm.heap.array_mut(id).set(index, args[1]) {
        Ok(()) => Ok(Value::unit()),
        Err(crate::heap::ArrayMutationError::Frozen) => Err(RuntimeError::FrozenMutation { collection: "Array" }.into()),
        Err(crate::heap::ArrayMutationError::OutOfBounds) => Err(RuntimeError::Type {
            expected: "an in-range index",
            found: "an out-of-range Number",
        }
        .into()),
    }
}

/// Creates a zero-copy Slice from normalized range bounds.
#[phalcom_native_macros::primitive(Array, "rawSlice(_,_)" , types = "(Int, Int) -> Unknown", visibility = internal)]
pub fn array_raw_slice(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    expect_array(vm, receiver)?;
    super::slice::make_slice(vm, receiver, args)
}

/// Signature: `Array::toString` — renders the array.
#[phalcom_native_macros::primitive(Array, "toString")]
pub fn array_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_array(vm, receiver)?;
    let elements = vm.heap.array(id).elements().to_vec();
    let mut parts: Vec<String> = Vec::with_capacity(elements.len());
    for elem in elements {
        parts.push(elem.to_display_string(vm)?);
    }
    let rendered = format!("Array([{}])", parts.join(", "));
    Ok(vm.alloc_string_value(rendered))
}
