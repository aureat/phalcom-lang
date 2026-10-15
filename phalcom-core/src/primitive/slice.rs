//! Minimal native reads for owner-rooted `Slice<T>` views.

use crate::error::{PhResult, RuntimeError};
use crate::primitive::expect_slice;
use crate::value::Value;
use crate::vm::VM;

fn normalized_bound(value: &Value) -> PhResult<usize> {
    let Some(bound) = value.as_int() else {
        return Err(RuntimeError::Type {
            expected: "a normalized integer slice bound",
            found: value.type_name(),
        }
        .into());
    };
    let Ok(bound) = usize::try_from(bound) else {
        return Err(RuntimeError::Type {
            expected: "a non-negative host-sized slice bound",
            found: "an out-of-range Number",
        }
        .into());
    };
    Ok(bound)
}

/// Constructs a zero-copy Slice while preserving the heap's canonical owner
/// normalization and range validation.
pub(crate) fn make_slice(vm: &mut VM, owner: &Value, args: &[Value]) -> PhResult<Value> {
    let owner_id = owner.as_obj().ok_or_else(|| RuntimeError::Type {
        expected: "Array, Vector, Buffer, or Slice",
        found: owner.type_name(),
    })?;
    let start = normalized_bound(&args[0])?;
    let end = normalized_bound(&args[1])?;
    let len = end.checked_sub(start).ok_or(RuntimeError::Type {
        expected: "ordered normalized slice bounds",
        found: "a reversed range",
    })?;
    let slice_id = vm.heap.alloc_slice_from_range(owner_id, start, len).map_err(|error| match error {
        crate::heap::SliceError::UnsupportedOwner => RuntimeError::Type {
            expected: "Array, Vector, Buffer, or Slice",
            found: owner.type_name(),
        },
        crate::heap::SliceError::RangeOverflow | crate::heap::SliceError::OutOfBounds => RuntimeError::Type {
            expected: "in-range normalized slice bounds",
            found: "an out-of-range Number",
        },
    })?;
    Ok(Value::obj(slice_id))
}

/// Returns the visible length of a Slice.
#[phalcom_native_macros::primitive(Slice, "rawLength", types = "() -> Int", visibility = internal)]
pub fn slice_raw_length(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_slice(vm, receiver)?;
    Ok(Value::int(vm.heap.slice(id).len() as i64))
}

/// Reads a current backing value through a normalized Slice range.
#[phalcom_native_macros::primitive(Slice, "rawAt(_)", types = "(Int) -> Unknown", visibility = internal)]
pub fn slice_raw_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_slice(vm, receiver)?;
    match super::index::normalize_element_index(&vm.heap, &args[0], vm.heap.slice(id).len())? {
        super::index::NormalizedIndex::Valid(index) => vm
            .heap
            .slice_element(id, index)
            .ok_or_else(|| RuntimeError::Internal("normalized Slice index is outside backing storage".into()).into()),
        super::index::NormalizedIndex::OutOfRange => Err(RuntimeError::Type {
            expected: "an in-range index",
            found: "an out-of-range Number",
        }
        .into()),
    }
}

/// Creates a Slice using bounds already normalized by `Range.rawSliceBounds`.
#[phalcom_native_macros::primitive(Slice, "rawSlice(_,_)" , types = "(Int, Int) -> Unknown", visibility = internal)]
pub fn slice_raw_slice(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    expect_slice(vm, receiver)?;
    make_slice(vm, receiver, args)
}
