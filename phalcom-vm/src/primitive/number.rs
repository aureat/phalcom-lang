use crate::error::{PhResult, RuntimeError};
use crate::value::Value;
use crate::vm::VM;
use crate::expect_value;

pub const NUM_0: Value = Value::Number(0.0);
pub const NUM_1: Value = Value::Number(1.0);

/// Signature: `Number::+(_)`
pub fn number_add(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(_receiver, Number);
    let other = expect_value!(&args[0], Number);
    Ok(Value::Number(this + other))
}

/// Signature: `Number::-(_)`
pub fn number_div(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(_receiver, Number);
    let other = expect_value!(&args[0], Number);

    if other == 0.0 {
        return Err(RuntimeError::ZeroDivision.into());
    }

    Ok(Value::Number(this / other))
}
