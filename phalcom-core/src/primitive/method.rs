//! Native primitives on `Method` — the reflective callable-tower surface.
//!
//! `Method` is reified as an [`Object::Method`]
//! heap object and re-parented under `Function` ([ADR-0006](../../docs/adr/0006-function-as-abstract-callable-root.md)),
//! so it inherits the shared call protocol (`arity`/`name`/`callWith`/`call`)
//! but does not answer raw `call` while unbound (`primitive::block::resolve_callable`).
//! This module adds the reflection surface that closes the gap: reifying
//! ([`crate::primitive::object::object_method_for`]), applying a reified
//! method to an explicit receiver ([`method_invoke_on`]), closing one over a
//! receiver ([`method_bind`]), and reading its selector/holder
//! ([`method_selector`]/[`method_holder`]) — U-CORE-3,
//! [ADR-0028](../../docs/adr/0028-amend-floor-admit-method-reflection.md).

use crate::error::{PhResult, RuntimeError};
use crate::heap::{BoundMethodObject, Object};
use crate::primitive::{expect_list, expect_method};
use crate::value::Value;
use crate::vm::VM;

/// `Method.class::new(_)`
pub fn method_class_new(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::NotAllowed("Method instances cannot be created directly".to_string()).into())
}

/// Signature: `Method::invokeOn(_,_)` — applies the reified method (`self`,
/// the receiver of this send) to the explicit receiver `args[0]` and the
/// argument [`List`](crate::list::ListObject) `args[1]`, unpacked as
/// positional arguments (functions.md §3, U-CORE-3).
///
/// Runs the **exact** reified method against `args[0]` via
/// [`VM::invoke_method_object`] — no re-dispatch by selector, unlike
/// `perform`. The caller is responsible for receiver compatibility (extracting
/// `Number#+` and applying it to a `String` misbehaves inside `number_add`;
/// this is deliberate and documented, not a defect).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `self` is not a `Method` or `args[1]` is
/// not a `List`, [`RuntimeError::Arity`] on an argument-count mismatch
/// (R-INV-3.4), or any error raised while running the method body.
pub fn method_invoke_on(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let method_id = expect_method(vm, receiver)?;
    let target = args[0];
    let list_id = expect_list(vm, &args[1])?;
    let elements: Vec<Value> = vm.heap.list(list_id).elements().to_vec();
    vm.invoke_method_object(method_id, target, &elements)
}

/// Signature: `Method::bind(_)` — closes the reified method (`self`) over
/// `args[0]` as its receiver, returning an
/// [`Object::BoundMethod`] whose surface
/// class is `Block` (ADR-0006) and which responds to `call` (functions.md
/// §3, U-CORE-3). `bound.call(a)` ≡ `method.invokeOn(recv, [a])` (R-INV-3.3):
/// both funnel through [`VM::invoke_method_object`] via
/// [`crate::primitive::block::block_call`]'s `BoundMethod` intercept.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `self` is not a `Method`.
pub fn method_bind(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let method_id = expect_method(vm, receiver)?;
    let bound = BoundMethodObject { method: method_id, receiver: args[0] };
    Ok(Value::Obj(vm.heap.alloc(Object::BoundMethod(bound))))
}

/// Signature: `Method::selector` — the interned selector
/// [`Symbol`](crate::interner::Symbol) exactly as it was resolved
/// (`MethodObject.signature.selector`; functions.md §3, U-CORE-3).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `self` is not a `Method`.
pub fn method_selector(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let method_id = expect_method(vm, receiver)?;
    Ok(Value::Symbol(vm.heap.method(method_id).signature.selector))
}

/// Signature: `Method::holder` — the defining `Class` (or metaclass, for a
/// class-side method); the shared `None` singleton
/// ([ADR-0007](../../docs/adr/0007-option-some-none.md)) if the method is
/// unbound (`MethodObject.holder`; functions.md §3, U-CORE-3).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `self` is not a `Method`.
pub fn method_holder(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let method_id = expect_method(vm, receiver)?;
    match vm.heap.method(method_id).holder {
        Some(class_id) => Ok(Value::Obj(class_id)),
        None => Ok(vm.none_value()),
    }
}