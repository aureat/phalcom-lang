use crate::error::PhResult;
use crate::value::Value;
use crate::vm::VM;

pub fn system_class_print(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    for arg in args {
        print!("{}", arg);
    }
    println!();
    Ok(Value::Nil)
}
