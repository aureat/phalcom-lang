//! Native primitives on `Object` — the tower root.

use crate::error::PhResult;
use crate::error::RuntimeError;
use crate::heap::Object;
use crate::instance::InstanceObject;
use crate::primitive::expect_class;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Object::name` — returns the receiver's class name as a string.
pub fn object_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = receiver.class(vm);
    let name = vm.heap.class(class_id).name.clone();
    Ok(vm.alloc_string_value(name))
}

/// Signature: `Object::class` — returns the receiver's class.
pub fn object_class(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::Obj(receiver.class(vm)))
}

/// Signature: `Object::class=(_)` — always an error; an object's class is fixed.
///
/// # Errors
///
/// Always returns [`RuntimeError::InvalidSetClass`].
pub fn object_set_class(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::InvalidSetClass.into())
}

/// Signature: `Object.class::new` — allocates a bare instance of the receiver class.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a class.
pub fn object_class_new(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = expect_class(vm, receiver)?;
    let instance = InstanceObject::new(class_id);
    Ok(Value::Obj(vm.heap.alloc(Object::Instance(instance))))
}
