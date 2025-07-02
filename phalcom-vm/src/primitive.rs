use crate::value::Value;
use crate::vm::VM;

/// The native implementation for Number.__add__(other).
pub fn native_number_add(vm: &mut VM, _receiver: &Value, args: &[Value]) -> Result<Value, String> {
    // In a real VM, you'd get the receiver from the stack too.
    // For now, we assume it was a number.
    let receiver_val = _receiver.as_number()?;
    let other_val = args[0].as_number()?;
    Ok(Value::Number(receiver_val + other_val))
}
