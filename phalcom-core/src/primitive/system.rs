use crate::error::{PhResult, RuntimeError};
use crate::nil::NIL;
use crate::value::Value;
use crate::vm::VM;
use tracing::{debug, debug_span};

/// `System.class::print(_)`
pub fn system_class_print(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    for arg in args {
        print!("{}", arg.to_string(_vm).borrow().as_str());
    }
    println!();
    Ok(NIL)
}

/// `System.class::new()`
pub fn system_class_new(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::NotAllowed("System instances cannot be created".to_string()).into())
}
