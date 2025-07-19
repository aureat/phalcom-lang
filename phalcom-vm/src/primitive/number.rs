use crate::error::{PhResult, RuntimeError};
use crate::value::Value;
use crate::vm::VM;
use crate::expect_value;

pub const NUM_0: Value = Value::Number(0.0);
pub const NUM_1: Value = Value::Number(1.0);

/// Signature: `Number::name`
pub fn number_name(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let _n = expect_value!(receiver, Number);
    Ok(Value::string_from_str("Number"))
}

/// Signature: `Number::+(_)`
pub fn number_add(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(_receiver, Number);
    let other = expect_value!(&args[0], Number);
    Ok(Value::Number(this + other))
}

/// Signature: `Number::-(_)`
pub fn number_div(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(_receiver, Number);
    let other = expect_value!(&args[0], Number);

    if other == 0.0 {
        return Err(RuntimeError::ZeroDivision.into());
    }

    Ok(Value::Number(this / other))
}
