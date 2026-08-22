//! Native primitives on `Symbol`.

use crate::error::{PhResult, RuntimeError};
use crate::expect_value;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Symbol::toString` — the symbol's display form, `#{interned
/// text}` (U-CORE-4, BD-CORE4-2 Option A).
#[phalcom_native_macros::primitive(
    Symbol,
    "toString",
    params = [],
    returns = String,
    types = "() -> String",
    effects = pure
)]
pub fn symbol_tostring(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let symbol = expect_value!(receiver, Symbol);
    let text = symbol.to_string(vm);
    Ok(vm.alloc_string_value(text))
}

/// Signature: `Symbol::hash` — a digest of the interned id.
#[phalcom_native_macros::primitive(
    Symbol,
    "hash",
    params = [],
    returns = Int,
    types = "() -> Int",
    effects = pure
)]
pub fn symbol_hash(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let symbol = expect_value!(receiver, Symbol);
    Ok(crate::primitive::hash_code(u64::from(symbol.0)))
}

/// Signature: `Symbol::isSelector` — true if text represents an exact selector syntax.
#[phalcom_native_macros::primitive(
    Symbol,
    "isSelector",
    params = [],
    returns = Bool,
    types = "() -> Bool",
    effects = pure
)]
pub fn symbol_is_selector(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let symbol = expect_value!(receiver, Symbol);
    let text = vm.resolve_symbol(symbol);
    Ok(Value::bool(phalcom_common::selector::is_exact_selector_syntax(text)))
}

/// Signature: `Symbol::isSelectorPattern` — true if text represents a selector pattern syntax.
#[phalcom_native_macros::primitive(
    Symbol,
    "isSelectorPattern",
    params = [],
    returns = Bool,
    types = "() -> Bool",
    effects = pure
)]
pub fn symbol_is_selector_pattern(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let symbol = expect_value!(receiver, Symbol);
    let text = vm.resolve_symbol(symbol);
    Ok(Value::bool(phalcom_common::selector::is_selector_pattern_syntax(text)))
}

/// Signature: `Symbol.class::new(_)` — interns its argument into a symbol.
#[phalcom_native_macros::primitive(
    Symbol,
    "new(_)",
    params = [Object],
    returns = Symbol,
    types = "(Object) -> Symbol",
    side = class
)]
pub fn symbol_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let Some(arg) = args.first() else {
        return Err(RuntimeError::Arity {
            signature: "Symbol.new(_)",
            found: args.len(),
            expected: 1,
        }
        .into());
    };
    if let Some(sym) = arg.symbol_value() {
        Ok(Value::symbol(sym))
    } else if let Some(id) = arg.as_obj() {
        if vm.heap.as_string(id).is_some() {
            let text = vm.heap.string(id).value();
            Ok(Value::symbol(vm.get_or_intern(&text)))
        } else {
            let text = arg.to_string(vm);
            Ok(Value::symbol(vm.get_or_intern(&text)))
        }
    } else {
        let text = arg.to_string(vm);
        Ok(Value::symbol(vm.get_or_intern(&text)))
    }
}
