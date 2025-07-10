use crate::error::PhResult;
use crate::error::RuntimeError;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Object::name`
pub fn object_name_(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::String(receiver.name(vm)))
}

/// Signature: `Object::class`
pub fn object_class_(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::Class(receiver.class(vm)))
}

/// Signature: `Object::class=(_)`
pub fn object_set_class(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::InvalidSetClass.into())
}

/// Signature: `Object::toString`
pub fn object_tostring(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class = receiver.class(vm);
    let name = receiver.name(vm);
    let class_borrow = class.borrow();
    let string = format!("<{} {}>", class_borrow.name.borrow().as_str(), name.borrow().as_str());
    drop(class_borrow);
    Ok(Value::string_from(string))
}
