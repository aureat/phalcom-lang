//! Native primitives for the kernel `Module` class.

use crate::error::{PhResult, RuntimeError};
use crate::primitive::object::object_does_not_understand;
use crate::value::Value;
use crate::vm::VM;

/// `Module.class::new()`
///
/// # Errors
///
/// Always returns [`RuntimeError::NotAllowed`] — linked program materialization
/// creates `Module` values; surface code cannot construct them directly.
#[phalcom_native_macros::primitive(Module, "new()", side = class)]
pub fn module_class_new(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::NotAllowed("Module instances cannot be created directly".to_string()).into())
}

/// `Module#doesNotUnderstand(_:)`
///
/// Export dispatch happens before class hierarchy lookup.
/// If an unknown send reaches here, it falls through directly to the `Object`
/// default `MessageNotUnderstood` raise.
#[phalcom_native_macros::primitive(Module, "doesNotUnderstand(_)")]
pub fn module_does_not_understand(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    object_does_not_understand(vm, receiver, args)
}
