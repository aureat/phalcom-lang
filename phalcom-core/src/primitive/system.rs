//! Native primitives on `System`.

use crate::error::{PhResult, RuntimeError};
use crate::nil::NIL;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `System.class::print(_)` — prints its arguments, then a newline.
pub fn system_class_print(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    for arg in args {
        print!("{}", arg.to_string(vm));
    }
    println!();
    Ok(NIL)
}

/// Signature: `System.class::new()` — always an error; `System` is not instantiable.
///
/// # Errors
///
/// Always returns [`RuntimeError::NotAllowed`].
pub fn system_class_new(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::NotAllowed("System instances cannot be created".to_string()).into())
}
