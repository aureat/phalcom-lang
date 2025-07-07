use crate::error::PhResult;
use crate::value::Value;
use crate::vm::VM;

pub fn object_name(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::String(_receiver.name(vm)))
}
