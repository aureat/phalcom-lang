//! Native primitives on `Method` — the reflective callable-tower surface.
//!
//! `Method` is reified as an [`Object::Method`] heap object under `Object`.
//! It exposes reflection and explicit receiver application; it is not a
//! `Function` descendant and does not answer raw `call` while unbound.
//! This module adds the reflection surface that closes the gap: reifying
//! ([`crate::primitive::object::object_method_for`]), applying a reified
//! method to an explicit receiver ([`method_invoke_on_shape`]), closing one over a
//! receiver ([`method_bind`]), and reading its selector/holder
//! ([`method_selector`]/[`method_holder`]) — U-CORE-3,
//! [ADR-0028](../../docs/adr/accepted/0028-amend-floor-admit-method-reflection.md).

use crate::error::{PhResult, RuntimeError};
use crate::heap::{BoundMethodObject, Object};
use crate::method::{ArgumentView, CallOutcome};
use crate::primitive::expect_method;
use crate::value::Value;
use crate::vm::VM;

/// `Method.class::new(_)`
pub fn method_class_new(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::NotAllowed("Method instances cannot be created directly".to_string()).into())
}

/// Shape-aware `Method#invokeOn(_,***)` gateway. The explicit receiver is
/// validated before the stack window is rewritten; the remaining values then
/// enter the exact reified method directly, without selector redispatch or a
/// packed `List` intermediary.
pub fn method_invoke_on_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    let method_id = expect_method(vm, &receiver)?;
    let target = args.positional(vm, 0).ok_or_else(|| RuntimeError::Arity {
        signature: "invokeOn",
        expected: 1,
        found: args.positional_count(),
    })?;
    let (caller, internal) = args.caller_authority();
    vm.authorize_method_access_as(method_id, caller, internal)?;
    let labels = args.labels().to_vec();
    let residual_positionals = args.positional_count().checked_sub(1).ok_or_else(|| RuntimeError::Arity {
        signature: "invokeOn",
        expected: 1,
        found: args.positional_count(),
    })?;
    let actual_selector = vm.validate_captured_method_shape(method_id, residual_positionals, &labels)?;

    let receiver_index = args.receiver_index();
    let residual = vm.stack[receiver_index + 2..].to_vec();
    vm.stack[receiver_index] = target;
    vm.stack.truncate(receiver_index + 1);
    vm.stack.extend_from_slice(&residual);
    let shaped = args.with_selector(actual_selector, residual_positionals, labels.into_boxed_slice());
    vm.activate_captured_method_as(target, method_id, shaped, phalcom_common::range::SourceRange::default())
}

/// Signature: `Method::bind(_)` — closes the reified method (`self`) over
/// `args[0]` as its receiver, returning an
/// [`Object::BoundMethod`] whose surface
/// class is `BoundMethod` and which responds to the shared Function gateway
/// (functions.md §3, U-CORE-3). Bound activation targets the stored method
/// directly; it does not redispatch by selector.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `self` is not a `Method`.
pub fn method_bind(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let method_id = expect_method(vm, receiver)?;
    let bound = BoundMethodObject {
        method: method_id,
        receiver: args[0],
    };
    Ok(Value::obj(vm.heap.alloc(Object::BoundMethod(bound))))
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
    Ok(Value::symbol(vm.heap.method(method_id).signature.selector))
}

/// Signature: `Method::holder` — the defining `Class` (or metaclass, for a
/// class-side method); immediate `None`
/// ([ADR-0007](../../docs/adr/accepted/0007-option-some-none.md)) if the method is
/// unbound (`MethodObject.holder`; functions.md §3, U-CORE-3).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `self` is not a `Method`.
pub fn method_holder(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let method_id = expect_method(vm, receiver)?;
    match vm.heap.method(method_id).holder {
        Some(class_id) => Ok(Value::obj(class_id)),
        None => Ok(vm.none_value()),
    }
}
