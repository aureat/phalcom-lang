//! Native bridge for structural selector-pattern values.

use crate::error::{PhResult, RuntimeError};
use crate::heap::Object;
use crate::heap::selector_pattern::SelectorPatternObject;
use crate::value::Value;
use crate::vm::VM;
use phalcom_common::selector::{
    Selector, SelectorBase, SelectorPattern, is_exact_selector_syntax,
};

fn extract_symbol_or_string(vm: &VM, arg: &Value) -> PhResult<String> {
    if let Some(sym) = arg.symbol_value() {
        Ok(vm.resolve_symbol(sym).to_string())
    } else if let Some(id) = arg.as_obj() {
        if let Some(s) = vm.heap.as_string(id) {
            Ok(s.value())
        } else {
            Err(RuntimeError::Type {
                expected: "Symbol or String",
                found: arg.type_name(),
            }
            .into())
        }
    } else {
        Err(RuntimeError::Type {
            expected: "Symbol or String",
            found: arg.type_name(),
        }
        .into())
    }
}

fn construct_selector_pattern(vm: &mut VM, args: &[Value]) -> PhResult<Value> {
    let Some(arg) = args.first() else {
        return Err(RuntimeError::Arity {
            signature: "SelectorPattern.call(_)",
            found: args.len(),
            expected: 1,
        }
        .into());
    };

    if let Some(id) = arg.as_obj() {
        if let Some(_pat) = vm.heap.as_selector_pattern(id) {
            return Ok(Value::obj(id));
        }
    }

    let text = extract_symbol_or_string(vm, arg)?;
    if is_exact_selector_syntax(&text) && !text.contains("...") {
        return Err(RuntimeError::Message(
            "SelectorPattern requires a selector pattern. Received exact selector.".into(),
        )
        .into());
    }

    let pattern = SelectorPattern::try_decode_pattern(&text)
        .map_err(|_| RuntimeError::Message("Invalid selector pattern syntax".into()))?;

    let pattern_obj = SelectorPatternObject::compile(pattern, &mut vm.interner);
    let obj = vm.heap.alloc(Object::SelectorPattern(Box::new(pattern_obj)));
    Ok(Value::obj(obj))
}

/// Signature: `SelectorPattern.class::call(_)`
#[phalcom_native_macros::primitive(
    SelectorPattern,
    "call(_)",
    params = [Object],
    returns = SelectorPattern,
    types = "(Object) -> SelectorPattern",
    side = class
)]
pub fn selector_pattern_class_call(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    construct_selector_pattern(vm, args)
}

/// Signature: `SelectorPattern.class::from(_)`
#[phalcom_native_macros::primitive(
    SelectorPattern,
    "from(_)",
    params = [Object],
    returns = SelectorPattern,
    types = "(Object) -> SelectorPattern",
    side = class
)]
pub fn selector_pattern_class_from(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    construct_selector_pattern(vm, args)
}

/// Signature: `SelectorPattern.class::new(_)`
#[phalcom_native_macros::primitive(
    SelectorPattern,
    "new(_)",
    params = [Object],
    returns = SelectorPattern,
    types = "(Object) -> SelectorPattern",
    side = class
)]
pub fn selector_pattern_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    construct_selector_pattern(vm, args)
}

fn expect_selector_pattern(vm: &VM, receiver: &Value) -> PhResult<SelectorPatternObject> {
    let Some(id) = receiver.as_obj() else {
        return Err(RuntimeError::Type {
            expected: "SelectorPattern",
            found: receiver.type_name(),
        }
        .into());
    };
    let Some(pat) = vm.heap.as_selector_pattern(id) else {
        return Err(RuntimeError::Type {
            expected: "SelectorPattern",
            found: receiver.type_name(),
        }
        .into());
    };
    Ok(pat.clone())
}

/// Signature: `SelectorPattern::matches(_)`
#[phalcom_native_macros::primitive(
    SelectorPattern,
    "matches(_)",
    params = [Object],
    returns = Bool,
    types = "(Object) -> Bool",
    effects = pure
)]
pub fn selector_pattern_matches(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let pattern = expect_selector_pattern(vm, receiver)?;
    let Some(arg) = args.first() else {
        return Ok(Value::bool(false));
    };
    if let Some(sym) = arg.symbol_value() {
        let selector = Selector::decode(vm.resolve_symbol(sym));
        return Ok(Value::bool(pattern.pattern.matches(&selector)));
    }
    if let Some(id) = arg.as_obj() {
        if let Some(sel) = vm.heap.as_selector(id) {
            return Ok(Value::bool(pattern.pattern.matches(&sel.selector)));
        }
    }
    Ok(Value::bool(false))
}

/// Signature: `SelectorPattern::toString`
#[phalcom_native_macros::primitive(
    SelectorPattern,
    "toString",
    params = [],
    returns = String,
    types = "() -> String",
    effects = pure
)]
pub fn selector_pattern_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let pattern = expect_selector_pattern(vm, receiver)?;
    Ok(vm.alloc_string_value(format!("SelectorPattern(#{})", pattern.pattern.encode())))
}

/// Signature: `SelectorPattern::==(_)`
#[phalcom_native_macros::primitive(
    SelectorPattern,
    "==(_)",
    params = [Object],
    returns = Bool,
    types = "(Object) -> Bool",
    effects = pure
)]
pub fn selector_pattern_equals(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let pattern = expect_selector_pattern(vm, receiver)?;
    let Some(arg) = args.first() else {
        return Ok(Value::bool(false));
    };
    if let Some(id) = arg.as_obj() {
        if let Some(other) = vm.heap.as_selector_pattern(id) {
            return Ok(Value::bool(pattern.pattern == other.pattern));
        }
    }
    Ok(Value::bool(false))
}

/// Signature: `SelectorPattern::hash`
#[phalcom_native_macros::primitive(
    SelectorPattern,
    "hash",
    params = [],
    returns = Int,
    types = "() -> Int",
    effects = pure
)]
pub fn selector_pattern_hash(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let pattern = expect_selector_pattern(vm, receiver)?;
    let encoded = pattern.pattern.encode();
    let hash = encoded
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(u64::from(b)));
    Ok(crate::primitive::hash_code(hash))
}

/// Signature: `SelectorPattern::base`
#[phalcom_native_macros::primitive(
    SelectorPattern,
    "base",
    params = [],
    returns = Symbol,
    types = "() -> Symbol",
    effects = pure
)]
pub fn selector_pattern_base(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let pattern = expect_selector_pattern(vm, receiver)?;
    let base_str = match &pattern.pattern.base {
        SelectorBase::Named(name) => name.as_str(),
        SelectorBase::Subscript => "[]",
    };
    Ok(Value::symbol(vm.get_or_intern(base_str)))
}
