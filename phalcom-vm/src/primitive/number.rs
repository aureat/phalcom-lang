use crate::value::Value;
use crate::vm::VM;

pub const NUM_0: Value = Value::Number(0.0);
pub const NUM_1: Value = Value::Number(1.0);

/// The native implementation for Number.+(other).
pub fn native_number_add(vm: &mut VM, _receiver: &Value, args: &[Value]) -> Result<Value, String> {
    // In a real VM, you'd get the receiver from the stack too.
    // For now, we assume it was a number.
    let receiver_val = _receiver.as_number()?;
    let other_val = args[0].as_number()?;
    Ok(Value::Number(receiver_val + other_val))
}
