use crate::error::RuntimeError;
use crate::boolean::{FALSE, TRUE};
use crate::error::PhResult;
use crate::expect_value;
use crate::value::Value;
use crate::vm::VM;

/// `Bool.class::new(_)`
pub fn bool_class_new(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let receiver = expect_value!(_receiver, Class);
    println!("{}", Value::Class(receiver.clone()));
    let arg = &args[0];
    println!("{arg}");
    match arg {
        Value::Bool(b) => Ok(if *b { TRUE } else { FALSE }),
        Value::Nil => Ok(FALSE),
        Value::Number(n) => Ok(if *n != 0.0 { TRUE } else { FALSE }),
        _ => Ok(TRUE),
    }
}
