use crate::error::{PhResult, RuntimeError};
use crate::expect_value;
use crate::value::Value;
use crate::vm::VM;

/// `Symbol::toString`
pub fn symbol_tostring(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let symbol = expect_value!(receiver, Symbol);
    let string = vm.interner.lookup(*symbol);

    Ok(Value::string_from_str(string))
}

/// `Symbol.class::from(_)`
pub fn symbol_class_from(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let string = expect_value!(&args[0], String);
    let symbol = vm.interner.intern(string.borrow().as_str());

    Ok(Value::Symbol(symbol))
}

/// `Symbol.class::new(_)`
pub fn symbol_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if let Some(arg) = args.first() {
        match arg {
            Value::String(s) => {
                let sym = vm.get_or_intern(&s.borrow().value());
                Ok(Value::Symbol(sym))
            }
            Value::Symbol(sym) => Ok(Value::Symbol(*sym)),
            _ => {
                let string_repr = arg.to_string(vm);
                let sym = vm.get_or_intern(&string_repr.borrow().value());
                Ok(Value::Symbol(sym))
            }
        }
    } else {
        Err(RuntimeError::Arity {
            signature: "Symbol.new(_)",
            found: args.len(),
            expected: 1,
        }
        .into())
    }
}
