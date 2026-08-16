//! Native primitives backing the M-ATTR-ROOT attribute-retention mechanism
//! (`attribute-classes.md`): `Object#__attach`/`__attributes`/
//! `__freezeAttributes`. Registered once on `object_class` instance-side
//! (`universe/primitives.rs`) — covering class and method receiver kinds.

use crate::error::{PhResult, RuntimeError};
use crate::heap::Object;
use crate::value::Value;
use crate::vm::VM;

// TODO: Can these methods be inlined wherever possible? Why call them and load frames?
// TODO: Change mentions non-heap value, a different heap object to the specific type name of the found value

/// Signature: `Object#__attach(_)` — appends `args[0]` (an `Attribute`
#[phalcom_native_macros::primitive(
    Object,
    "_$attach(_)",
    params = [Object],
    returns = Option,
    types = "(Object) -> Option",
    visibility = public
)]
pub fn attribute_attach(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let attr = args[0];
    let Value::Obj(id) = *receiver else {
        return Err(RuntimeError::Type {
            expected: "Class or Method",
            found: "a non-heap value",
        }
        .into());
    };
    let attached = match vm.heap.get_mut(id) {
        Object::Class(c) => c.attach_attribute(attr),
        Object::Method(m) => m.attach_attribute(attr),
        _ => {
            return Err(RuntimeError::Type {
                expected: "Class or Method",
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
#[phalcom_native_macros::primitive(
    Object,
    "_$attributes",
    params = [],
    returns = List,
    types = "() -> List",
    visibility = public
)]
pub fn attribute_attributes(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let Value::Obj(id) = *receiver else {
        return Err(RuntimeError::Type {
            expected: "Class or Method",
            found: "a non-heap value",
        }
        .into());
    };
    // TODO: is cloning here necessary? anything cheaper?
    let attrs: Vec<Value> = match vm.heap.get(id) {
        Object::Class(c) => c.attributes.clone(),
        Object::Method(m) => m.attributes.clone(),
        _ => {
            return Err(RuntimeError::Type {
                expected: "Class or Method",
                found: "a different heap object",
            }
            .into());
        }
    };
    Ok(Value::Obj(vm.heap.alloc(Object::List(crate::heap::ListObject::new(attrs)))))
}

/// Signature: `Object#__freezeAttributes` — marks the receiver's attribute
#[phalcom_native_macros::primitive(
    Object,
    "_$freezeAttributes()",
    params = [],
    returns = Option,
    types = "() -> Option",
    visibility = public
)]
pub fn attribute_freeze(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let Value::Obj(id) = *receiver else {
        return Err(RuntimeError::Type {
            expected: "Class or Method",
            found: "a non-heap value",
        }
        .into());
    };
    match vm.heap.get_mut(id) {
        Object::Class(c) => c.attributes_frozen = true,
        Object::Method(m) => m.attributes_frozen = true,
        _ => {
            return Err(RuntimeError::Type {
                expected: "Class or Method",
                found: "a different heap object",
            }
            .into());
        }
    }
    Ok(vm.none_value())
}
