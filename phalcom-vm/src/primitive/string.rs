use crate::error::PhResult;
use crate::error::RuntimeError;
use crate::expect_value;
use crate::value::Value;
use crate::vm::VM;

/// `String::+(_)`
pub fn string_add(_vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let first = expect_value!(receiver, String);
    let second = expect_value!(&args[0], String);

    let result = first.borrow().as_str().to_owned() + second.borrow().as_str();

    Ok(Value::string_from(result))
}

/// `String::repeat(_)`
pub fn string_repeat(_vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let s = expect_value!(receiver, String);
    let n = expect_value!(&args[0], Number) as usize;

    let string_borrowed = s.borrow();
    let string = string_borrowed.as_str();

    let mut out = String::with_capacity(string.len() * n);
    for _ in 0..n {
        out.push_str(string);
    }

    drop(string_borrowed);

    Ok(Value::string_from(out))
}

/// `String::hash`
pub fn string_hash(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let s = expect_value!(receiver, String);
    let hash = s.borrow().hash();

    Ok(Value::Number(hash as f64))
}

/// `String.class::new(_)`
pub fn string_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if let Some(arg) = args.first() {
        let string = arg.to_string(vm);
        Ok(Value::String(string))
    } else {
        Ok(Value::string_from_str(""))
    }
}
