//! Native primitives for `Selector` objects.

use crate::error::{PhResult, RuntimeError};
use crate::heap::Object;
use crate::heap::selector::SelectorObject;
use crate::value::Value;
use crate::vm::VM;
use phalcom_common::selector::{
    Selector, SelectorBase, SelectorKind, SelectorSlot, is_selector_pattern_syntax,
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

fn construct_selector(vm: &mut VM, args: &[Value]) -> PhResult<Value> {
    let Some(arg) = args.first() else {
        return Err(RuntimeError::Arity {
            signature: "Selector.call(_)",
            found: args.len(),
            expected: 1,
        }
        .into());
    };

    if let Some(id) = arg.as_obj() {
        if let Some(_sel_obj) = vm.heap.as_selector(id) {
            return Ok(Value::obj(id));
        }
    }

    let text = extract_symbol_or_string(vm, arg)?;
    if is_selector_pattern_syntax(&text) || text.contains("...") {
        return Err(RuntimeError::Message(
            "Cannot construct Selector from selector pattern symbol. Use SelectorPattern instead.".into(),
        )
        .into());
    }

    let selector = Selector::try_decode_exact(&text)
        .map_err(|_| RuntimeError::Message("Invalid selector syntax".into()))?;

    let canonical = selector.encode();
    let sym = vm.get_or_intern(&canonical);

    let selector_obj = SelectorObject { selector, symbol: sym };
    let obj = vm.heap.alloc(Object::Selector(Box::new(selector_obj)));
    Ok(Value::obj(obj))
}

/// Signature: `Selector.class::call(_)`
#[phalcom_native_macros::primitive(
    Selector,
    "call(_)",
    params = [Object],
    returns = Selector,
    types = "(Object) -> Selector",
    side = class
)]
pub fn selector_class_call(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    construct_selector(vm, args)
}

/// Signature: `Selector.class::from(_)`
#[phalcom_native_macros::primitive(
    Selector,
    "from(_)",
    params = [Object],
    returns = Selector,
    types = "(Object) -> Selector",
    side = class
)]
pub fn selector_class_from(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    construct_selector(vm, args)
}

/// Signature: `Selector.class::new(_)`
#[phalcom_native_macros::primitive(
    Selector,
    "new(_)",
    params = [Object],
    returns = Selector,
    types = "(Object) -> Selector",
    side = class
)]
pub fn selector_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    construct_selector(vm, args)
}

fn expect_selector(vm: &VM, receiver: &Value) -> PhResult<SelectorObject> {
    let Some(id) = receiver.as_obj() else {
        return Err(RuntimeError::Type {
            expected: "Selector",
            found: receiver.type_name(),
        }
        .into());
    };
    let Some(selector) = vm.heap.as_selector(id) else {
        return Err(RuntimeError::Type {
            expected: "Selector",
            found: receiver.type_name(),
        }
        .into());
    };
    Ok(selector.clone())
}

/// Signature: `Selector::toString`
#[phalcom_native_macros::primitive(
    Selector,
    "toString",
    params = [],
    returns = String,
    types = "() -> String",
    effects = pure
)]
pub fn selector_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let selector = expect_selector(vm, receiver)?;
    Ok(vm.alloc_string_value(format!("Selector(#{})", selector.selector.encode())))
}

/// Signature: `Selector::==(_)`
#[phalcom_native_macros::primitive(
    Selector,
    "==(_)",
    params = [Object],
    returns = Bool,
    types = "(Object) -> Bool",
    effects = pure
)]
pub fn selector_equals(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let selector = expect_selector(vm, receiver)?;
    let Some(arg) = args.first() else {
        return Ok(Value::bool(false));
    };
    if let Some(sym) = arg.symbol_value() {
        return Ok(Value::bool(selector.symbol == sym || selector.selector.encode() == vm.resolve_symbol(sym)));
    }
    if let Some(id) = arg.as_obj() {
        if let Some(other) = vm.heap.as_selector(id) {
            return Ok(Value::bool(selector.selector == other.selector));
        }
    }
    Ok(Value::bool(false))
}

/// Signature: `Selector::hash`
#[phalcom_native_macros::primitive(
    Selector,
    "hash",
    params = [],
    returns = Int,
    types = "() -> Int",
    effects = pure
)]
pub fn selector_hash(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let selector = expect_selector(vm, receiver)?;
    Ok(crate::primitive::hash_code(u64::from(selector.symbol.0)))
}

/// Signature: `Selector::base`
#[phalcom_native_macros::primitive(
    Selector,
    "base",
    params = [],
    returns = Symbol,
    types = "() -> Symbol",
    effects = pure
)]
pub fn selector_base(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let selector = expect_selector(vm, receiver)?;
    let base_str = match &selector.selector.base {
        SelectorBase::Named(name) => name.as_str(),
        SelectorBase::Subscript => "[]",
    };
    Ok(Value::symbol(vm.get_or_intern(base_str)))
}

/// Signature: `Selector::kind`
#[phalcom_native_macros::primitive(
    Selector,
    "kind",
    params = [],
    returns = Symbol,
    types = "() -> Symbol",
    effects = pure
)]
pub fn selector_kind(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let selector = expect_selector(vm, receiver)?;
    let kind_str = match selector.selector.kind {
        SelectorKind::Getter => "getter",
        SelectorKind::Setter => "setter",
        SelectorKind::Method => "method",
        SelectorKind::SubscriptGet => "[_]",
        SelectorKind::SubscriptSet => "[_]=(put)",
    };
    Ok(Value::symbol(vm.get_or_intern(kind_str)))
}

/// Signature: `Selector::slots`
#[phalcom_native_macros::primitive(
    Selector,
    "slots",
    params = [],
    returns = List,
    types = "() -> List",
    effects = pure
)]
pub fn selector_slots(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let selector = expect_selector(vm, receiver)?;
    let slots: Vec<Value> = selector
        .selector
        .slots
        .iter()
        .map(|slot| match slot {
            SelectorSlot::Positional => Value::symbol(vm.get_or_intern("_")),
            SelectorSlot::Label(label) => Value::symbol(vm.get_or_intern(label)),
        })
        .collect();
    Ok(Value::obj(vm.heap.alloc_list(slots)))
}
