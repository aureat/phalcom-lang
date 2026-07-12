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
///
/// Renders each argument via [`Value::to_display_string`], which sends
/// `toString` to any heap object with no bespoke native renderer, so this
/// stays in agreement with an explicit `.toString` send for user classes,
/// instances and metaclasses (U-ERR-FIX PRINT-TOSTRING).
///
/// # Errors
///
/// Propagates any error a `toString` send raises while rendering an argument.
pub fn system_class_print(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    for arg in args {
        let text = arg.to_display_string(vm)?;
        print!("{text}");
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
