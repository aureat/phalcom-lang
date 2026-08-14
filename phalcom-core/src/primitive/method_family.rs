//! Native protocol helpers for immutable `MethodFamily` snapshots.
//!
//! Task 7 only creates and retains snapshots. Invocation and binding are added
//! in the following captured-call task so this module keeps the type check in
//! one place for those protocols.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{MethodFamilyObject, ObjRef, Object};
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

pub(crate) fn method_family(vm: &VM, id: ObjRef) -> &MethodFamilyObject {
    vm.heap.method_family(id)
}
