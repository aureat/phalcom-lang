//! Native primitives on `String`.

use crate::error::PhResult;
use crate::primitive::expect_string;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `String::+(_)` — concatenates two strings.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`](crate::error::RuntimeError::Type) if either
/// operand is not a string.
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
