use crate::error::{PhResult, RuntimeError};
use crate::expect_value;
use crate::value::Value;
use crate::vm::VM;

pub fn symbol_tostring(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let symbol = expect_value!(receiver, Symbol);
    let string = vm.interner.lookup(*symbol);

    Ok(Value::string_from_str(string))
}

pub fn symbol_class_from(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let string = expect_value!(&args[0], String);
    let symbol = vm.interner.intern(string.borrow().as_str());

    Ok(Value::Symbol(symbol))
}
