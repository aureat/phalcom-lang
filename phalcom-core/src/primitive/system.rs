//! Native primitives on `System`.

use crate::error::{PhResult, RuntimeError};
use crate::value::Value;
use crate::vm::VM;

/// Signature: `System.class::print(_)` — prints its arguments, then a newline.
///
/// Returns the `None` singleton (surface absence value): `print` is a
/// statement-like send whose result is user-reachable (e.g. `print(print(1))`),
/// so it must never yield the raw `nil` sentinel (Invariant 4,
/// [ADR-0007](../../../docs/adr/0007-option-some-none.md)).
pub fn system_class_print(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    for arg in args {
        print!("{}", arg.to_string(vm));
    }
    println!();
    Ok(vm.none_value())
}

/// Signature: `System.class::new()` — always an error; `System` is not instantiable.
///
/// # Errors
///
/// Always returns [`RuntimeError::NotAllowed`].
pub fn system_class_new(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::NotAllowed("System instances cannot be created".to_string()).into())
}
