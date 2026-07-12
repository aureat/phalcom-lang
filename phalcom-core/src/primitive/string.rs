//! Native primitives on `String`.

use crate::error::{PhResult, RuntimeError};
use crate::primitive::expect_string;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `String::hash` — the cached djb2 content hash.
///
/// Reuses the content hash the string already caches
/// ([`StringObject::hash`](crate::string::StringObject::hash) /
/// [`calculate_hash`](crate::string::StringObject::calculate_hash)) so equal
/// content hashes equal, satisfying `a == b ⇒ a.hash == b.hash` (R-INV-1.3)
/// even for two distinct-handle strings — `String#==` is content equality.
/// Underivable — the `String` floor exposes only `+`/`new`, not the bytes.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a string.
pub fn string_hash(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = match receiver {
        Value::Obj(id) if vm.heap.as_string(*id).is_some() => *id,
        other => return Err(RuntimeError::Type { expected: "String", found: other.type_name() }.into()),
    };
    let hash = u64::from(vm.heap.string(id).hash());
    Ok(crate::primitive::hash_code(hash))
}

/// Signature: `String::+(_)` — concatenates two strings.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if either operand is not a string.
pub fn string_add(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let first = expect_string(vm, receiver)?;
    let second = expect_string(vm, &args[0])?;
    Ok(vm.alloc_string_value(first + &second))
}

/// Signature: `String.class::new(_)` — builds a string from its argument.
///
/// With an argument, renders it via [`Value::to_string`]; with none, returns the
/// empty string.
pub fn string_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    match args.first() {
        Some(arg) => {
            let text = arg.to_string(vm);
            Ok(vm.alloc_string_value(text))
        }
        None => Ok(vm.alloc_string_value(String::new())),
    }
}
