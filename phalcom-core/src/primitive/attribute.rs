//! Native primitives backing the M-ATTR-ROOT attribute-retention mechanism
//! (`attribute-classes.md`): `Object#__attach`/`__attributes`/
//! `__freezeAttributes`. Registered once on `object_class` instance-side
//! (`universe/primitives.rs`) — since every class object, method object, and
//! module object sits under `Object` in the metaclass tower (ADR-0002), one
//! registration site covers all three receiver kinds.

use crate::error::{PhResult, RuntimeError};
use crate::heap::Object;
use crate::value::Value;
use crate::vm::VM;

// TODO: Can these methods be inlined wherever possible? Why call them and load frames?
// TODO: Change mentions non-heap value, a different heap object to the specific type name of the found value

/// Signature: `Object#__attach(_)` — appends `args[0]` (an `Attribute`
/// instance) to the receiver's attribute-retention store.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a class, method, or
/// module object, or `attr.frozen` ([`RuntimeError::NotAllowed`]) if the
/// receiver's store was already frozen (`__freezeAttributes`).
pub fn attribute_attach(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let attr = args[0];
    let Value::Obj(id) = *receiver else {
        return Err(RuntimeError::Type {
            expected: "Class, Method, or Module",
            found: "a non-heap value",
        }
        .into());
    };
    let attached = match vm.heap.get_mut(id) {
        Object::Class(c) => c.attach_attribute(attr),
        Object::Method(m) => m.attach_attribute(attr),
        Object::Module(m) => m.attach_attribute(attr),
        _ => {
            return Err(RuntimeError::Type {
                expected: "Class, Method, or Module",
                found: "a different heap object",
            }
            .into());
        }
    };
    if !attached {
        return Err(RuntimeError::NotAllowed("attr.frozen: attribute store is frozen".to_string()).into());
    }
    Ok(vm.none_value())
}

/// Signature: `Object#__attributes` — reads the receiver's attribute-
/// retention store into a fresh `List` (empty if nothing was ever attached).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a class, method, or
/// module object.
pub fn attribute_attributes(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let Value::Obj(id) = *receiver else {
        return Err(RuntimeError::Type {
            expected: "Class, Method, or Module",
            found: "a non-heap value",
        }
        .into());
    };
    // TODO: is cloning here necessary? anything cheaper?
    let attrs: Vec<Value> = match vm.heap.get(id) {
        Object::Class(c) => c.attributes.clone(),
        Object::Method(m) => m.attributes.clone(),
        Object::Module(m) => m.attributes.clone(),
        _ => {
            return Err(RuntimeError::Type {
                expected: "Class, Method, or Module",
                found: "a different heap object",
            }
            .into());
        }
    };
    Ok(Value::Obj(vm.heap.alloc(Object::List(crate::heap::ListObject::new(attrs)))))
}

/// Signature: `Object#__freezeAttributes` — marks the receiver's attribute
/// store frozen; further `__attach` calls raise `attr.frozen`.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a class, method, or
/// module object.
pub fn attribute_freeze(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let Value::Obj(id) = *receiver else {
        return Err(RuntimeError::Type {
            expected: "Class, Method, or Module",
            found: "a non-heap value",
        }
        .into());
    };
    match vm.heap.get_mut(id) {
        Object::Class(c) => c.attributes_frozen = true,
        Object::Method(m) => m.attributes_frozen = true,
        Object::Module(m) => m.attributes_frozen = true,
        _ => {
            return Err(RuntimeError::Type {
                expected: "Class, Method, or Module",
                found: "a different heap object",
            }
            .into());
        }
    }
    Ok(vm.none_value())
}
