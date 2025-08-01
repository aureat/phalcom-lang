use crate::error::{PhResult, RuntimeError};
use crate::value::Value;
use crate::vm::VM;

/// `Method.class::new(_)`
pub fn method_class_new(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::NotAllowed("Method instances cannot be created directly".to_string()).into())
}