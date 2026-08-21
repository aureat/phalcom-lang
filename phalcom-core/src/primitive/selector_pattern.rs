//! Native bridge for structural selector-pattern values.

use crate::error::{PhResult, RuntimeError};
use crate::heap::Object;
use crate::value::Value;
use crate::vm::VM;
use phalcom_common::selector::Selector;

/// `SelectorPattern::matches(_)` accepts an exact selector symbol and applies
/// the already-shared structural matcher. Other candidate values are false.
#[phalcom_native_macros::primitive(
    SelectorPattern,
    "matches(_)",
    params = [Object],
    returns = Bool,
    types = "(Object) -> Bool",
    effects = pure
)]
pub fn selector_pattern_matches(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let Some(receiver_id) = receiver.as_obj() else {
        return Err(RuntimeError::Type { expected: "SelectorPattern", found: receiver.type_name() }.into());
    };
    let Object::SelectorPattern(pattern) = vm.heap.get(receiver_id) else {
        return Err(RuntimeError::Type { expected: "SelectorPattern", found: receiver.type_name() }.into());
    };
    let Some(selector) = args.first().and_then(|value| value.symbol_value()) else {
        return Ok(Value::bool(false));
    };
    let selector = Selector::decode(vm.resolve_symbol(selector));
    Ok(Value::bool(pattern.pattern.matches(&selector)))
}
