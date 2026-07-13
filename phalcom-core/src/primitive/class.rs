//! Native primitives on `Class`.

use crate::error::PhResult;
use crate::error::RuntimeError;
use crate::heap::Object;
use crate::heap::InstanceObject;
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
/// A class's `superclass` is sealed at class creation (U13, DEC-U13a=A;
/// [ADR-0026](../../../docs/adr/0026-class-hierarchy-mutability.md);
/// [ADR-0041](../../../docs/adr/0041-hierarchy-stability-policy.md)): a
/// runtime reparent is rejected outright, never performed, so `ClassId`-keyed
/// dispatch and the fixed instance slot layout
/// ([ADR-0011](../../../docs/adr/0011-static-instance-slot-layout.md)) stay
/// provably stable. Method *reopening* — adding or replacing methods on an
/// existing class — is a separate axis and is unaffected by this seal.
///
/// # Errors
///
/// Always returns [`RuntimeError::InvalidSetSuper`].
pub fn class_set_superclass(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::InvalidSetSuper.into())
}

/// Signature: `Behavior::name` — the receiver class's OWN display name.
///
/// Shadows [`object_name`](super::object::object_name) for class and metaclass
/// receivers ([ADR-0023](../../../docs/adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)):
/// `Object#name` returns the *class-of-receiver*'s name, which for a class `C`
/// is the metaclass name `"C class"`, whereas `Behavior#name` returns the
/// class's own stored name `"C"`. Underivable — no `.ph` primitive exposes a
/// class's own [`name`](crate::heap::ClassObject::name) field, and there is no
/// `.ph` string slicing to recover `"C"` from `"C class"`. A non-class receiver
/// has no `Behavior` in its chain, so it still resolves `name` to `object_name`.
/// Side-effect-free (R-INV-1.6).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a class.
pub fn behavior_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = expect_class(vm, receiver)?;
    let name = vm.heap.class(class_id).name.clone();
    Ok(vm.alloc_string_value(name))
}

/// Signature: `Behavior::methods` — a fresh [`List`](crate::heap::ListObject)
/// of the selectors defined DIRECTLY on the receiver class, as
/// [`Symbol`](crate::interner::Symbol)s.
///
/// Enumerates the receiver's own method dictionary
/// ([`ClassObject::methods`](crate::heap::ClassObject::methods)) — the
/// non-inherited, own-dictionary keys — and returns them, one interned selector
/// symbol per binding (SD-2;
/// [ADR-0023](../../../docs/adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)).
/// Underivable — no `.ph` accessor reaches the method map. Side-effect-free
/// (R-INV-1.6): builds a new `List` and reads nothing but the map. Inherited /
/// `allMethods` walking is U-STD, derivable over this and `superclass`.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a class.
pub fn behavior_methods(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = expect_class(vm, receiver)?;
    let selectors: Vec<Value> = vm.heap.class(class_id).methods.keys().map(|selector| Value::Symbol(*selector)).collect();
    Ok(Value::Obj(vm.heap.alloc_list(selectors)))
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
    let field_count = vm.heap.class(class_id).field_count;
    let instance = InstanceObject::new(class_id, field_count);
    Ok(Value::Obj(vm.heap.alloc(Object::Instance(instance))))
}
