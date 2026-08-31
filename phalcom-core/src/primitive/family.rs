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

#[phalcom_native_macros::primitive(Family, "receiver")]
pub fn family_receiver(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(vm.heap.family(family_id(vm, receiver)?).receiver)
}

#[phalcom_native_macros::primitive(Family, "selector")]
pub fn family_selector(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let spec = vm.heap.family(family_id(vm, receiver)?).spec;
    match spec {
        FamilySpec::Exact(selector) => Ok(Value::symbol(selector)),
        FamilySpec::Pattern(_) => Ok(vm.none_value()),
    }
}

#[phalcom_native_macros::primitive(Family, "pattern")]
pub fn family_pattern(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let spec = vm.heap.family(family_id(vm, receiver)?).spec;
    match spec {
        FamilySpec::Exact(_) => Ok(vm.none_value()),
        FamilySpec::Pattern(pattern) => Ok(Value::obj(pattern)),
    }
}

#[phalcom_native_macros::primitive(Family, "isExact")]
pub fn family_is_exact(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
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
