//! Minimal immutable Vector storage primitives.

use crate::error::{PhResult, RuntimeError};
use crate::value::Value;
use crate::vm::VM;

/// Returns the immutable Vector payload length.
#[phalcom_native_macros::primitive(Vector, "rawLength", params = [], returns = Int, types = "() -> Int", visibility = internal, effects = pure)]
pub fn vector_raw_length(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().filter(|id| vm.heap.as_vector(*id).is_some()).ok_or(RuntimeError::Type {
        expected: "Vector",
        found: receiver.type_name(),
    })?;
    Ok(Value::int(vm.heap.vector(id).len() as i64))
}

/// Reads one existing element. Out-of-range access fails rather than returning
/// an absence sentinel as a value of the generic element type.
#[phalcom_native_macros::primitive(Vector, "rawAt(_)", params = [Int], types = "(Int) -> Unknown", visibility = internal, effects = pure)]
pub fn vector_raw_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = receiver.as_obj().filter(|id| vm.heap.as_vector(*id).is_some()).ok_or(RuntimeError::Type {
        expected: "Vector",
        found: receiver.type_name(),
    })?;
    match super::index::normalize_element_index(&vm.heap, &args[0], vm.heap.vector(id).len())? {
        super::index::NormalizedIndex::Valid(index) => vm.heap.vector(id).get(index).ok_or_else(|| {
            RuntimeError::Internal("normalized Vector index is outside storage".into()).into()
        }),
        super::index::NormalizedIndex::OutOfRange => Err(RuntimeError::Type {
            expected: "an in-range index",
            found: "an out-of-range Number",
        }.into()),
    }
}
