use crate::error::PhResult;
use crate::error::RuntimeError;
use crate::expect_value;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Class::superclass`
pub fn class_superclass(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    println!("{:?}", _receiver);
    let class = expect_value!(_receiver, Class);
    let superclass = class.borrow().superclass.clone();
    match superclass {
        Some(cls) => Ok(Value::Class(cls)),
        None => Ok(Value::Nil),
    }
}

/// Signature: `Class::superclass=(_)`
pub fn class_set_superclass(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::InvalidSetSuper.into())
}
