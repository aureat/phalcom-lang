use crate::value::Value;
use crate::vm::VM;
use std::rc::Rc;

pub fn native_string_add(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> Result<Value, String> {
    if (args.len() != 1) {
        return Err("String.+ requires exactly one argument".to_string());
    }

    if !_receiver.is_string() || !args[0].is_string() {
        return Err("String.+ requires both receiver and argument to be strings".to_string());
    }

    let receiver_str = _receiver.as_string()?;
    let other_str = args[0].as_string()?;

    // Concatenate the two strings.
    let result = receiver_str.as_str().to_owned() + other_str.as_str();

    Ok(Value::String(Rc::new(result)))
}
