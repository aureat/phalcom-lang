//! Native primitives on `Bool`.

use crate::boolean::{FALSE, TRUE};
use crate::error::PhResult;
use crate::primitive::expect_class;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Bool.class::new(_)` — coerces its argument to a boolean.
///
/// `false`, `nil` and `0` become `false`; every other value becomes `true`.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`](crate::error::RuntimeError::Type) if the
/// receiver is not a class.
pub fn bool_class_new(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let receiver_id = expect_class(vm, receiver)?;
    // NOTE(U1/DEFERRED): these two prints are pre-existing debug noise carried
    // over verbatim to keep behavior identical; they are not exercised by any
    // golden. Flagged for removal in `docs/forge/DEFERRED.md`.
    println!("{}", Value::Obj(receiver_id).to_string(vm));
    let arg = &args[0];
    println!("{}", arg.to_string(vm));
    match arg {
        Value::Bool(b) => Ok(if *b { TRUE } else { FALSE }),
        Value::Nil => Ok(FALSE),
        Value::Number(n) => Ok(if *n != 0.0 { TRUE } else { FALSE }),
        _ => Ok(TRUE),
    }
}
