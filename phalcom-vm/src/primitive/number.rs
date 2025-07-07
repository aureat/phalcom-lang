use crate::error::{PhResult, RuntimeError};
use crate::value::Value;
use crate::vm::VM;
use crate::{ensure_arity, expect_value};

pub const NUM_0: Value = Value::Number(0.0);
pub const NUM_1: Value = Value::Number(1.0);

pub fn number_add(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    const SIGNATURE: &str = "Number.+(_)";
    ensure_arity!(SIGNATURE, args, 1);

    let this = expect_value!(_receiver, Number);
    let other = expect_value!(&args[0], Number);
    Ok(Value::Number(this + other))
}

pub fn number_div(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    const SIGNATURE: &str = "Number./(_)";
    ensure_arity!(SIGNATURE, args, 1);

    let this = expect_value!(_receiver, Number);
    let other = expect_value!(&args[0], Number);

    if other == 0.0 {
        return Err(RuntimeError::ZeroDivision.into());
    }

    Ok(Value::Number(this / other))
}
