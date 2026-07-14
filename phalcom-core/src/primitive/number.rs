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

/// Signature: `Number::hash` — a digest of the mathematical value.
///
/// Digests the *value*, class-agnostically (forward-compat §4;
/// [ADR-0023](../../../docs/adr/accepted/0023-amend-floor-admit-hash-and-kernel-reflection.md)):
/// an integral number hashes to its integer, so a future `Integer 2` and
/// `Float 2.0` agree (open-Q2); `-0.0` is normalized to `0.0`; non-integral /
/// infinite values hash by their canonical IEEE-754 bits. `a == b ⇒
/// a.hash == b.hash` (R-INV-1.3) and the digest is stable within a run
/// (R-INV-1.4). Underivable — `.ph` has no bit access to an `f64`.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a number.
pub fn number_hash(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let n = expect_value!(receiver, Number);
    let bits = if n == 0.0 {
        // Unify +0.0 and -0.0 (which are `==` but have distinct bit patterns).
        0
    } else if n.is_finite() && n.fract() == 0.0 && n.abs() < 9_007_199_254_740_992.0 {
        // Integral and in the safe-integer range → hash as that integer.
        (n as i64) as u64
    } else {
        // Non-integral or non-finite → canonical bits.
        n.to_bits()
    };
    Ok(crate::primitive::hash_code(bits))
}

/// Signature: `Number::toString` — the numeric value as a decimal string
/// (U-CORE-4).
///
/// Delegates to the shared native renderer ([`Value::to_string`]) so
/// `n.toString` is byte-identical to `System.print(n)` (R-INV-4.1). Reads the
/// receiver's `f64` representation, which is unreachable from `.ph` — hence a
/// floor primitive ([ADR-0019](../../../docs/adr/accepted/0019-freeze-vm-blessed-primitive-floor.md)
/// amendment; cf. [`number_hash`], decisions.md Q1). Bound on `Number` (the
/// abstract numeric root), not a concrete f64 path, so a future `Integer`/
/// `Float` split (forward-compat §4) can refine this per-subclass without
/// breaking dispatch identity.
///
/// Always succeeds: [`Value::to_string`] is total over every `Value` variant.
pub fn number_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let text = receiver.to_string(vm); // immutable borrow ends before the alloc below
    Ok(vm.alloc_string_value(text))
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
/// [ADR-0005](../../../docs/adr/retired/0005-number-as-flat-f64.md)).
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
/// control-flow.md §1, [ADR-0012](../../../docs/adr/accepted/0012-selector-encoding-and-dispatch.md)).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a number.
pub fn number_negated(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(receiver, Number);
    Ok(Value::Number(-this))
}
