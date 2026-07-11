//! Native primitives on `Symbol`.

use crate::error::{PhResult, RuntimeError};
use crate::expect_value;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Symbol::toString` — the symbol's interned text as a string.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a symbol.
pub fn symbol_tostring(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let symbol = expect_value!(receiver, Symbol);
    let text = vm.resolve_symbol(*symbol).to_string();
    Ok(vm.alloc_string_value(text))
}

/// Signature: `Symbol.class::new(_)` — interns its argument into a symbol.
///
/// A string is interned by content, a symbol is returned unchanged, and any
/// other value is first rendered via [`Value::to_string`] and then interned.
///
/// # Errors
///
/// Returns [`RuntimeError::Arity`] if called with no argument.
pub fn symbol_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let Some(arg) = args.first() else {
        return Err(RuntimeError::Arity {
            signature: "Symbol.new(_)",
            found: args.len(),
            expected: 1,
        }
        .into());
    };
    match arg {
        Value::Symbol(sym) => Ok(Value::Symbol(*sym)),
        Value::Obj(id) if vm.heap.as_string(*id).is_some() => {
            let text = vm.heap.string(*id).value();
            Ok(Value::Symbol(vm.get_or_intern(&text)))
        }
        other => {
            let text = other.to_string(vm);
            Ok(Value::Symbol(vm.get_or_intern(&text)))
        }
    }
}
