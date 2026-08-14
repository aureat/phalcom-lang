//! Native protocol helpers for immutable `MethodFamily` snapshots.
//!
//! Task 7 only creates and retains snapshots. Invocation and binding are added
//! in the following captured-call task so this module keeps the type check in
//! one place for those protocols.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{BoundMethodFamilyObject, ObjRef, Object};
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
pub fn method_family_bind(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let family = expect_method_family(vm, receiver)?;
    let bound_receiver = args.first().copied().ok_or_else(|| RuntimeError::Arity {
        signature: "bind",
        expected: 1,
        found: 0,
    })?;
    Ok(Value::Obj(vm.heap.alloc(Object::BoundMethodFamily(BoundMethodFamilyObject {
        family,
        receiver: bound_receiver,
    }))))
}
