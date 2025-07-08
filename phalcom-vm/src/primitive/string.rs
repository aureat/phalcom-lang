use crate::error::PhResult;
use crate::error::RuntimeError;
use crate::value::Value;
use crate::vm::VM;
use crate::{ensure_arity, expect_value};
use std::hash::Hash;
use std::rc::Rc;

pub fn string_add(_vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    const SIGNATURE: &str = "String.+(_)";
    ensure_arity!(SIGNATURE, args, 1);

    let first = expect_value!(receiver, String);
    let second = expect_value!(&args[0], String);

    let result = first.borrow().as_str().to_owned() + second.borrow().as_str();

    Ok(Value::string_from(result))
}

pub fn str_repeat(_vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    const SIGNATURE: &str = "String.repeat(_)";

    ensure_arity!(SIGNATURE, args, 1);

    let s = expect_value!(receiver, String);
    let n = expect_value!(&args[0], Number) as usize;

    let mut out = String::with_capacity(s.len() * n);
    for _ in 0..n {
        out.push_str(s.as_str());
    }

    Ok(Value::String(Rc::new(out)))
}
