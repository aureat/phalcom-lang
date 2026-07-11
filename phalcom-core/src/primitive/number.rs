//! Native primitives on `Number`.

use crate::error::{PhResult, RuntimeError};
use crate::expect_value;
use crate::value::Value;
use crate::vm::VM;

/// The number `0`, as a constant [`Value`].
pub const NUM_0: Value = Value::Number(0.0);
/// The number `1`, as a constant [`Value`].
pub const NUM_1: Value = Value::Number(1.0);

/// Signature: `Number.class::new(_)` — coerces its argument to a number.
///
/// Accepts a number (identity), a numeric string (parsed), or a boolean
/// (`1`/`0`). With no argument, returns `0`.
///
/// # Errors
///
/// Returns [`RuntimeError::TypeConversion`] if the argument is a non-numeric
/// string or an otherwise non-convertible value.
pub fn number_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let Some(arg) = args.first() else {
        return Ok(Value::Number(0.0));
    };
    match arg {
        Value::Number(n) => Ok(Value::Number(*n)),
        Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
        Value::Obj(id) if vm.heap.as_string(*id).is_some() => {
            let text = vm.heap.string(*id).value();
            text.parse::<f64>().map(Value::Number).map_err(|_| {
                RuntimeError::TypeConversion {
                    expected: "number",
                    found: "value", // TODO: base this on arg.type_name() once granular.
                }
                .into()
            })
        }
        _ => Err(RuntimeError::TypeConversion {
            expected: "number",
            found: arg.type_name(),
        }
        .into()),
    }
}

/// Signature: `Number::+(_)` — numeric addition.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if either operand is not a number.
pub fn number_add(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(_receiver, Number);
    let other = expect_value!(&args[0], Number);
    Ok(Value::Number(this + other))
}

/// Signature: `Number::/(_)` — numeric division.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if either operand is not a number, or
/// [`RuntimeError::ZeroDivision`] if the divisor is zero.
pub fn number_div(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(_receiver, Number);
    let other = expect_value!(&args[0], Number);

    if other == 0.0 {
        return Err(RuntimeError::ZeroDivision.into());
    }

    Ok(Value::Number(this / other))
}
