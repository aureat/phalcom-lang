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
    let field_count = vm.heap.class(class_id).field_count;
    let instance = InstanceObject::new(class_id, field_count);
    Ok(Value::Obj(vm.heap.alloc(Object::Instance(instance))))
}

/// Signature: `Object::==(_)` — the base equality send (U5, control-flow.md
/// §1: `==`/`!=` are ordinary sends like every other operator). Delegates to
/// [`Value::value_eq`](crate::value::Value::value_eq) (content equality for
/// strings, identity for instances/classes/methods, by-value for
/// immediates), so it reproduces exactly today's `==` semantics — only the
/// *dispatch mechanism* changes. Any subclass (e.g. a user `==(other)`
/// override, per `person2.ph`) shadows this via ordinary method lookup.
pub fn object_eq(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::Bool(receiver.value_eq(&args[0], &vm.heap)))
}

/// Signature: `Object::!=(_)` — the base inequality send; the logical
/// negation of [`object_eq`].
pub fn object_neq(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::Bool(!receiver.value_eq(&args[0], &vm.heap)))
}
