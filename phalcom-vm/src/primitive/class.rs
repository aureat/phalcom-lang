use crate::error::PhResult;
use crate::error::RuntimeError;
use crate::expect_value;
use crate::value::Value;
use crate::vm::VM;

pub fn class_superclass(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    const SIGNATURE: &str = "Class.superclass";
    let class = expect_value!(_receiver, Class);
    Ok(class.borrow().superclass_val())
}

pub fn class_set_superclass(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    const SIGNATURE: &str = "Class.superclass=(_)";

    Err(RuntimeError::InvalidSetSuper.into())
}
