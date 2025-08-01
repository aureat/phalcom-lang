use crate::error::{PhResult, RuntimeError};
use crate::expect_value;
use crate::value::Value;
use crate::vm::VM;

pub const NUM_0: Value = Value::Number(0.0);
pub const NUM_1: Value = Value::Number(1.0);

/// `Number.class::new(_)`
pub fn number_class_new(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if let Some(arg) = args.first() {
        match arg {
            Value::Number(n) => Ok(Value::Number(*n)),
            Value::String(s) => {
                let parsed = s.borrow().value().parse::<f64>().map_err(|_| RuntimeError::TypeConversion {
                    expected: "number",
                    found: "value", // TODO: Type should be based on arg.type_name()
                });
                match parsed {
                    Ok(n) => Ok(Value::Number(n)),
                    Err(e) => Err(e.into()),
                }
            }
            Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
            _ => Err(RuntimeError::TypeConversion {
                expected: "number",
                found: arg.type_name(),
            }
            .into()),
        }
    } else {
        Ok(Value::Number(0.0))
    }
}

/// `Number::name`
pub fn number_name(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let _n = expect_value!(receiver, Number);
    Ok(Value::string_from_str("Number"))
}

/// `Number::+(_)`
pub fn number_add(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(_receiver, Number);
    let other = expect_value!(&args[0], Number);
    Ok(Value::Number(this + other))
}

/// `Number::-(_)`
pub fn number_div(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_value!(_receiver, Number);
    let other = expect_value!(&args[0], Number);

    if other == 0.0 {
        return Err(RuntimeError::ZeroDivision.into());
    }

    Ok(Value::Number(this / other))
}
