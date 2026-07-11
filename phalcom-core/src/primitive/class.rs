//! Native primitives on `Class`.

use crate::error::PhResult;
use crate::error::RuntimeError;
use crate::heap::Object;
use crate::instance::InstanceObject;
use crate::primitive::expect_class;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Class::superclass` — returns the receiver's superclass, or
/// `None` for the root class (which has no superclass).
///
/// The absent-superclass case yields the `None` singleton, not the raw `nil`
/// sentinel: the result flows directly to user code (Invariant 4,
/// [ADR-0007](../../../docs/adr/0007-option-some-none.md)).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a class.
pub fn class_superclass(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = expect_class(vm, receiver)?;
    match vm.heap.class(class_id).superclass {
        Some(superclass) => Ok(Value::Obj(superclass)),
        None => Ok(vm.none_value()),
    }
}

/// Signature: `Class::superclass=(_)` — always an error; the tower is fixed here.
///
/// # Errors
///
/// Always returns [`RuntimeError::InvalidSetSuper`].
pub fn class_set_superclass(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::InvalidSetSuper.into())
}

/// Signature: `Class::+(_)` — concatenates the two classes' names into a string.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if either operand is not a class.
pub fn class_add(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_class(vm, receiver)?;
    let other = expect_class(vm, &args[0])?;
    let name = format!("{}{}", vm.heap.class(this).name, vm.heap.class(other).name);
    Ok(vm.alloc_string_value(name))
}

/// Signature: `Class::new` — allocates a bare instance of the receiver class.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a class.
pub fn class_new(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = expect_class(vm, receiver)?;
    let instance = InstanceObject::new(class_id);
    Ok(Value::Obj(vm.heap.alloc(Object::Instance(instance))))
}
