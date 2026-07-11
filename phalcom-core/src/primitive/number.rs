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
/// Follows IEEE-754 `f64` division: `1 / 0` is `inf`, `-1 / 0` is `-inf`,
/// `0 / 0` is `NaN` (control-flow.md/arithmetic goldens pin this — Phalcom's
/// flat `Number` never special-cases the divisor,
/// [ADR-0005](../../../docs/adr/0005-number-as-flat-f64.md)).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if either operand is not a number.
pub fn number_div(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(_receiver, Number);
    let other = expect_value!(&args[0], Number);
    Ok(Value::Number(this / other))
}

/// Signature: `Number::-(_)` — numeric subtraction.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if either operand is not a number.
pub fn number_sub(_vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(receiver, Number);
    let other = expect_value!(&args[0], Number);
    Ok(Value::Number(this - other))
}

/// Signature: `Number::*(_)` — numeric multiplication.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if either operand is not a number.
pub fn number_mul(_vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(receiver, Number);
    let other = expect_value!(&args[0], Number);
    Ok(Value::Number(this * other))
}

/// Signature: `Number::%(_)` — floating-point remainder (Rust `%`/IEEE-754
/// `fmod` semantics, sign follows the dividend).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if either operand is not a number.
pub fn number_mod(_vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(receiver, Number);
    let other = expect_value!(&args[0], Number);
    Ok(Value::Number(this % other))
}

/// Signature: `Number::<(_)` — less-than comparison.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if either operand is not a number.
pub fn number_lt(_vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(receiver, Number);
    let other = expect_value!(&args[0], Number);
    Ok(Value::Bool(this < other))
}

/// Signature: `Number::<=(_)` — less-than-or-equal comparison.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if either operand is not a number.
pub fn number_le(_vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(receiver, Number);
    let other = expect_value!(&args[0], Number);
    Ok(Value::Bool(this <= other))
}

/// Signature: `Number::>(_)` — greater-than comparison.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if either operand is not a number.
pub fn number_gt(_vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(receiver, Number);
    let other = expect_value!(&args[0], Number);
    Ok(Value::Bool(this > other))
}

/// Signature: `Number::>=(_)` — greater-than-or-equal comparison.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if either operand is not a number.
pub fn number_ge(_vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(receiver, Number);
    let other = expect_value!(&args[0], Number);
    Ok(Value::Bool(this >= other))
}

/// Signature: `Number::negated()` — unary numeric negation (surface `-x`,
/// control-flow.md §1, [ADR-0012](../../../docs/adr/0012-selector-encoding-and-dispatch.md)).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a number.
pub fn number_negated(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(receiver, Number);
    Ok(Value::Number(-this))
}
