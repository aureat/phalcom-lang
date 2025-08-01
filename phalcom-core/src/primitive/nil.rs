use crate::error::PhResult;
use crate::nil::NIL;
use crate::value::Value;
use crate::vm::VM;

/// `Nil.class::new()`
pub fn nil_class_new(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(NIL)
}
