//! Native primitives for bounded `Buffer<T>` storage.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{BufferMutationError, BufferObject, ObjRef};
use crate::primitive::expect_buffer;
use crate::value::Value;
use crate::vm::VM;

fn expect_capacity(value: &Value) -> PhResult<usize> {
    let Some(capacity) = value.as_int() else {
        return Err(RuntimeError::Type {
            expected: "a non-negative Int capacity",
            found: value.type_name(),
        }
        .into());
    };
    usize::try_from(capacity).map_err(|_| RuntimeError::ResourceLimit.into())
}

fn normalized_index(vm: &VM, value: &Value, len: usize) -> PhResult<usize> {
    match super::index::normalize_element_index(&vm.heap, value, len)? {
        super::index::NormalizedIndex::Valid(index) => Ok(index),
        super::index::NormalizedIndex::OutOfRange => Err(RuntimeError::Type {
            expected: "an in-range index",
            found: "an out-of-range Number",
        }
        .into()),
    }
}

/// Allocates an empty Buffer with fixed logical capacity.
#[phalcom_native_macros::primitive(Buffer, "new(_)", side = class, types = "(Int) -> Unknown")]
pub fn buffer_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let capacity = expect_capacity(&args[0])?;
    let buffer = BufferObject::try_new(capacity).map_err(|_| RuntimeError::ResourceLimit)?;
    Ok(Value::obj(vm.heap.alloc_buffer(buffer)))
}

/// Returns the initialized element count.
#[phalcom_native_macros::primitive(Buffer, "rawLength", visibility = internal, types = "() -> Int")]
pub fn buffer_raw_length(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_buffer(vm, receiver)?;
    Ok(Value::int(vm.heap.buffer(id).len() as i64))
}

/// Returns logical capacity rather than the allocator's `Vec::capacity()`.
#[phalcom_native_macros::primitive(Buffer, "rawCapacity", visibility = internal, types = "() -> Int")]
pub fn buffer_raw_capacity(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_buffer(vm, receiver)?;
    Ok(Value::int(vm.heap.buffer(id).capacity() as i64))
}

/// Reads one initialized element; out-of-range indices fail.
#[phalcom_native_macros::primitive(Buffer, "rawAt(_)", visibility = internal, types = "(Int) -> Unknown")]
pub fn buffer_raw_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_buffer(vm, receiver)?;
    let index = normalized_index(vm, &args[0], vm.heap.buffer(id).len())?;
    vm.heap
        .buffer(id)
        .get(index)
        .ok_or_else(|| RuntimeError::Internal("normalized Buffer index is outside initialized storage".into()).into())
}

/// Replaces one initialized element without changing size or capacity.
#[phalcom_native_macros::primitive(Buffer, "rawSet(_,_)", visibility = internal, types = "(Int, Unknown) -> ()", effects = [mutation])]
pub fn buffer_raw_set(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_buffer(vm, receiver)?;
    let index = normalized_index(vm, &args[0], vm.heap.buffer(id).len())?;
    match vm.heap.buffer_mut(id).set(index, args[1]) {
        Ok(()) => Ok(Value::unit()),
        Err(BufferMutationError::OutOfBounds) => Err(RuntimeError::Internal("validated Buffer index left initialized storage".into()).into()),
        Err(BufferMutationError::CapacityExceeded) => Err(RuntimeError::Internal("Buffer set cannot exceed logical capacity".into()).into()),
    }
}

/// Attempts one append. Returns false at logical capacity without mutation.
#[phalcom_native_macros::primitive(Buffer, "rawPush(_)", visibility = internal, types = "(Unknown) -> Bool", effects = [mutation])]
pub fn buffer_raw_push(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_buffer(vm, receiver)?;
    let pushed = vm.heap.buffer_mut(id).push(args[0]).is_ok();
    Ok(Value::bool(pushed))
}

/// Creates a zero-copy Slice over the Buffer's initialized prefix.
#[phalcom_native_macros::primitive(Buffer, "rawSlice(_,_)" , types = "(Int, Int) -> Unknown", visibility = internal)]
pub fn buffer_raw_slice(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    expect_buffer(vm, receiver)?;
    super::slice::make_slice(vm, receiver, args)
}
