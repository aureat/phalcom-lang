//! Native protocol for dynamic selector Families.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{FamilySpec, Object};
use crate::method::{ArgumentView, CallOutcome};
use crate::value::Value;
use crate::vm::VM;

fn family_id(vm: &VM, receiver: &Value) -> Result<crate::heap::ObjRef, RuntimeError> {
    let Value::Obj(id) = receiver else {
        return Err(RuntimeError::Type {
            expected: "Family",
            found: receiver.type_name(),
        });
    };
    if matches!(vm.heap.get(*id), Object::Family(_)) {
        Ok(*id)
    } else {
        Err(RuntimeError::Type {
            expected: "Family",
            found: receiver.type_name(),
        })
    }
}

pub fn family_receiver(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(vm.heap.family(family_id(vm, receiver)?).receiver)
}

pub fn family_selector(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let spec = vm.heap.family(family_id(vm, receiver)?).spec;
    match spec {
        FamilySpec::Exact(selector) => Ok(Value::Symbol(selector)),
        FamilySpec::Pattern(_) => Ok(vm.none_value()),
    }
}

pub fn family_pattern(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let spec = vm.heap.family(family_id(vm, receiver)?).spec;
    match spec {
        FamilySpec::Exact(_) => Ok(vm.none_value()),
        FamilySpec::Pattern(pattern) => Ok(Value::Obj(pattern)),
    }
}

pub fn family_is_exact(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let family = vm.heap.family(family_id(vm, receiver)?);
    Ok(Value::Bool(matches!(family.spec, FamilySpec::Exact(_))))
}

pub fn family_get(vm: &mut VM, _receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    vm.activate_family_with_kind(args, crate::vm::FamilyInvocationKind::Getter, phalcom_common::range::SourceRange::default())
}

pub fn family_set(vm: &mut VM, _receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    vm.activate_family_with_kind(args, crate::vm::FamilyInvocationKind::Setter, phalcom_common::range::SourceRange::default())
}
