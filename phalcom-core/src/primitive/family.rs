//! Native protocol for dynamic selector Families.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{FamilySpec, Object};
use crate::method::{ArgumentView, CallOutcome};
use crate::value::Value;
use crate::vm::VM;

fn family_id(vm: &VM, receiver: &Value) -> Result<crate::heap::ObjRef, RuntimeError> {
    if let Some(id) = receiver.as_obj() {
        if matches!(vm.heap.get(id), Object::Family(_) | Object::AssociatedFamily(_)) {
            return Ok(id);
        }
    }
    Err(RuntimeError::Type {
        expected: "Family",
        found: receiver.type_name(),
    })
}

fn associated_family<'a>(vm: &'a VM, receiver: &Value) -> Option<&'a crate::heap::AssociatedFamilyObject> {
    receiver.as_obj().and_then(|id| match vm.heap.get(id) {
        Object::AssociatedFamily(family) => Some(family.as_ref()),
        _ => None,
    })
}

#[phalcom_native_macros::primitive(Family, "receiver")]
pub fn family_receiver(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if let Some(family) = associated_family(vm, receiver) {
        return Ok(family.bound_owner.unwrap_or_else(|| vm.none_value()));
    }
    Ok(vm.heap.family(family_id(vm, receiver)?).receiver)
}

#[phalcom_native_macros::primitive(Family, "selector")]
pub fn family_selector(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if associated_family(vm, receiver).is_some() {
        return Ok(vm.none_value());
    }
    let spec = vm.heap.family(family_id(vm, receiver)?).spec;
    match spec {
        FamilySpec::Exact(selector) => Ok(Value::symbol(selector)),
        FamilySpec::Pattern(_) => Ok(vm.none_value()),
    }
}

#[phalcom_native_macros::primitive(Family, "pattern")]
pub fn family_pattern(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if associated_family(vm, receiver).is_some() {
        return Ok(vm.none_value());
    }
    let spec = vm.heap.family(family_id(vm, receiver)?).spec;
    match spec {
        FamilySpec::Exact(_) => Ok(vm.none_value()),
        FamilySpec::Pattern(pattern) => Ok(Value::obj(pattern)),
    }
}

#[phalcom_native_macros::primitive(Family, "isExact")]
pub fn family_is_exact(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if associated_family(vm, receiver).is_some() {
        return Ok(Value::bool(false));
    }
    let family = vm.heap.family(family_id(vm, receiver)?);
    Ok(Value::bool(matches!(family.spec, FamilySpec::Exact(_))))
}

#[phalcom_native_macros::primitive(Family, "get()", abi = shape)]
pub fn family_get(vm: &mut VM, _receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    vm.activate_family_with_kind(args, crate::vm::FamilyInvocationKind::Getter, phalcom_common::range::SourceRange::default())
}

#[phalcom_native_macros::primitive(Family, "set(_)" , abi = shape)]
pub fn family_set(vm: &mut VM, _receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    vm.activate_family_with_kind(args, crate::vm::FamilyInvocationKind::Setter, phalcom_common::range::SourceRange::default())
}

#[phalcom_native_macros::primitive(Family, "value", abi = shape)]
pub fn family_value(vm: &mut VM, _receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    vm.activate_family_with_kind(args, crate::vm::FamilyInvocationKind::Getter, phalcom_common::range::SourceRange::default())
}

#[phalcom_native_macros::primitive(Family, "value=(_)", abi = shape)]
pub fn family_value_set(vm: &mut VM, _receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    vm.activate_family_with_kind(args, crate::vm::FamilyInvocationKind::Setter, phalcom_common::range::SourceRange::default())
}

#[phalcom_native_macros::primitive(Family, "get(_)", abi = shape)]
pub fn family_get_shape(vm: &mut VM, _receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    let shape = args.positional(vm, 0).ok_or(RuntimeError::Arity {
        signature: "get(_)",
        expected: 1,
        found: args.positional_count(),
    })?;
    let shape_id = crate::primitive::expect_tuple(vm, &shape)?;
    let tuple = vm.heap.tuple(shape_id);
    let positionals = tuple.positionals().to_vec();
    let labeled_values = tuple.labeled_values().to_vec();
    let labels = tuple.labels().to_vec().into_boxed_slice();
    let receiver_idx = args.receiver_index();
    vm.stack.truncate(receiver_idx + 1);
    vm.stack.extend(positionals.iter().copied());
    vm.stack.extend(labeled_values);
    let view = ArgumentView::shaped_with_labels(
        receiver_idx,
        positionals.len(),
        labels,
        vm.get_or_intern("get(_)"),
        args.caller_authority().0,
        args.caller_authority().1,
    );
    vm.activate_family_with_kind(
        view,
        crate::vm::FamilyInvocationKind::SubscriptGet,
        phalcom_common::range::SourceRange::default(),
    )
}

#[phalcom_native_macros::primitive(Family, "set(_,_)", abi = shape)]
pub fn family_set_shape(vm: &mut VM, _receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    let shape = args.positional(vm, 0).ok_or(RuntimeError::Arity {
        signature: "set(_,_)",
        expected: 2,
        found: args.positional_count(),
    })?;
    let value = args.positional(vm, 1).ok_or(RuntimeError::Arity {
        signature: "set(_,_)",
        expected: 2,
        found: args.positional_count(),
    })?;
    let shape_id = crate::primitive::expect_tuple(vm, &shape)?;
    let tuple = vm.heap.tuple(shape_id);
    let positionals = tuple.positionals().to_vec();
    let labeled_values = tuple.labeled_values().to_vec();
    let labels = tuple.labels().to_vec().into_boxed_slice();
    let receiver_idx = args.receiver_index();
    vm.stack.truncate(receiver_idx + 1);
    vm.stack.extend(positionals.iter().copied());
    vm.stack.extend(labeled_values);
    vm.stack.push(value);
    let view = ArgumentView::shaped_with_labels(
        receiver_idx,
        positionals.len() + 1,
        labels,
        vm.get_or_intern("set(_,_)"),
        args.caller_authority().0,
        args.caller_authority().1,
    );
    vm.activate_family_with_kind(
        view,
        crate::vm::FamilyInvocationKind::SubscriptSet,
        phalcom_common::range::SourceRange::default(),
    )
}
