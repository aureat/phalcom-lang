use crate::error::PhResult;
use crate::error::RuntimeError;
use crate::value::Value;
use crate::vm::VM;

pub fn object_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    const SIGNATURE: &str = "Object.name";
    Ok(Value::String(receiver.name_string_ref(vm)))
}

pub fn object_class(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    const SIGNATURE: &str = "Object.class";
    Ok(Value::Class(receiver.class(vm)))
}

pub fn object_set_class(_vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    const SIGNATURE: &str = "Object.class=(_)";
    Err(RuntimeError::InvalidSetClass.into())
}
