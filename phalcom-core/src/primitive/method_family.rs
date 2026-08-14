//! Native protocol helpers for immutable `MethodFamily` snapshots.
//!
//! The snapshot is immutable after capture: reflection returns copied selector
//! metadata or reified Method handles, while binding preserves the captured
//! routing table and never consults the bound receiver for selection.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{BoundMethodFamilyObject, ObjRef, Object};
use crate::method::{ArgumentView, CallOutcome};
use crate::value::Value;
use crate::vm::VM;

pub(crate) fn expect_method_family(vm: &VM, receiver: &Value) -> PhResult<ObjRef> {
    match receiver {
        Value::Obj(id) if matches!(vm.heap.get(*id), Object::MethodFamily(_)) => Ok(*id),
        other => Err(RuntimeError::Type {
            expected: "MethodFamily",
            found: other.type_name(),
        }
        .into()),
    }
}

/// `MethodFamily#bind(_)` closes an immutable snapshot over a receiver. The
/// receiver is intentionally not inspected: selection belongs entirely to the
/// captured snapshot and happens only when the bound value is called.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] for a non-family receiver and
/// [`RuntimeError::Arity`] when the receiver argument is missing.
pub fn method_family_bind(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let family = expect_method_family(vm, receiver)?;
    let bound_receiver = args.first().copied().ok_or(RuntimeError::Arity {
        signature: "bind",
        expected: 1,
        found: 0,
    })?;
    Ok(Value::Obj(vm.heap.alloc(Object::BoundMethodFamily(BoundMethodFamilyObject {
        family,
        receiver: bound_receiver,
    }))))
}

/// Returns captured selector metadata as a fresh mutable list.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `receiver` is not a `MethodFamily`.
pub fn method_family_selectors(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let family = expect_method_family(vm, receiver)?;
    let selectors = {
        let family = vm.heap.method_family(family);
        let mut selectors = family.exact_methods.keys().copied().map(Value::Symbol).collect::<Vec<_>>();
        selectors.extend(
            family
                .rest_candidates
                .iter()
                .map(|method| Value::Symbol(vm.heap.method(*method).signature.selector)),
        );
        selectors
    };
    Ok(Value::Obj(vm.heap.alloc_list(selectors)))
}

/// Reports the number of captured exact and rest routes.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `receiver` is not a `MethodFamily`.
pub fn method_family_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let family = expect_method_family(vm, receiver)?;
    let size = {
        let family = vm.heap.method_family(family);
        family.exact_methods.len() + family.rest_candidates.len()
    };
    Ok(Value::Int(size as i64))
}

/// `MethodFamily#methodFor(_)` returns a captured Method for its canonical
/// selector, subject to the caller's current visibility authority.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] for a non-family receiver or non-symbol
/// selector, and [`RuntimeError::Arity`] when the selector is missing.
pub fn method_family_method_for(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let caller_authority = (vm.current_access_class(), vm.current_has_internal_privilege());
    method_family_method_for_as(vm, receiver, args, caller_authority)
}

/// Shape-aware gateway for `MethodFamily#methodFor(_)`. The native method
/// context is the MethodFamily implementation itself, so the original caller
/// authority must come from the incoming argument view rather than from the
/// current native frame.
///
/// # Errors
///
/// Propagates the same type and arity errors as [`method_family_method_for`].
pub fn method_family_method_for_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    let selector = args.positional(vm, 0).ok_or_else(|| RuntimeError::Arity {
        signature: "methodFor",
        expected: 1,
        found: args.positional_count(),
    })?;
    let value = method_family_method_for_as(vm, &receiver, &[selector], args.caller_authority())?;
    Ok(CallOutcome::Returned(value))
}

fn method_family_method_for_as(vm: &mut VM, receiver: &Value, args: &[Value], caller_authority: (Option<crate::heap::ClassId>, bool)) -> PhResult<Value> {
    let family = expect_method_family(vm, receiver)?;
    let selector = match args.first() {
        Some(Value::Symbol(selector)) => *selector,
        Some(other) => {
            return Err(RuntimeError::Type {
                expected: "Symbol",
                found: other.type_name(),
            }
            .into());
        }
        None => {
            return Err(RuntimeError::Arity {
                signature: "methodFor",
                expected: 1,
                found: 0,
            }
            .into());
        }
    };

    let method = {
        let family = vm.heap.method_family(family);
        family.exact_methods.get(&selector).copied().or_else(|| {
            family
                .rest_candidates
                .iter()
                .copied()
                .find(|method| vm.heap.method(*method).signature.selector == selector)
        })
    };
    match method {
        Some(method) if vm.authorize_method_access_as(method, caller_authority.0, caller_authority.1).is_ok() => Ok(Value::Obj(method)),
        _ => Ok(vm.none_value()),
    }
}
