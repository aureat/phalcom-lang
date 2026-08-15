//! Native primitives on `Bool`.
//!
//! Beyond `Bool.class::new(_)`, this module registers the sacred-selector
//! *fallbacks* — the real message-send implementations of `and(_)`, `or(_)`,
//! `not()`, `ifTrue(_)`, `ifFalse(_)` and `ifTrue(_)ifFalse(_)` (control-flow.md
//! §2–3). They are what every `Bool`-receiver sacred send resolves to
//! whether or not the compiler's inliner ([ADR-0018](../../../docs/adr/accepted/0018-sacred-selector-inliner-and-override-guard.md))
//! took the fast path for a given call site: the inliner's guarded jump
//! opcodes are an optimization over calling these, never a divergent
//! reimplementation of their semantics.

use crate::error::{PhResult, RuntimeError};
use crate::primitive::block::block_call;
use crate::primitive::expect_class;
use crate::primitive::nil::wrap_some;
use crate::value::Value;
use crate::value::{FALSE, TRUE};
use crate::vm::VM;

/// Signature: `Bool.class::new(_)` — coerces its argument to a boolean.
#[phalcom_native_macros::primitive(
    Bool,
    "new(_)",
    params = [Object],
    returns = Bool,
    types = "(Object) -> Bool",
    side = class
)]
pub fn bool_class_new(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let receiver_id = expect_class(vm, receiver)?;
    // NOTE(U1/DEFERRED): these two prints are pre-existing debug noise carried
    // over verbatim to keep behavior identical; they are not exercised by any
    // golden. Flagged for removal in `docs/forge/DEFERRED.md`.
    println!("{}", Value::Obj(receiver_id).to_string(vm));
    let arg = &args[0];
    println!("{}", arg.to_string(vm));
    match arg {
        Value::Bool(b) => Ok(if *b { TRUE } else { FALSE }),
        Value::Nil => Ok(FALSE),
        Value::Int(n) => Ok(if *n != 0 { TRUE } else { FALSE }),
        Value::Float(n) => Ok(if *n != 0.0 { TRUE } else { FALSE }),
        _ => Ok(TRUE),
    }
}

/// Signature: `Bool::hash` — `1` for `true`, `0` for `false`.
#[phalcom_native_macros::primitive(
    Bool,
    "hash",
    params = [],
    returns = Int,
    types = "() -> Int",
    effects = pure
)]
pub fn bool_hash(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let bit = u64::from(matches!(receiver, Value::Bool(true)));
    Ok(crate::primitive::hash_code(bit))
}

/// Extracts the `bool` payload of a `Bool` receiver.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a `Bool`.
fn expect_bool(value: &Value) -> PhResult<bool> {
    match value {
        Value::Bool(b) => Ok(*b),
        other => Err(RuntimeError::Type {
            expected: "Bool",
            found: other.type_name(),
        }
        .into()),
    }
}

/// Signature: `Bool::and(_)` — sacred, lazy logical conjunction
#[phalcom_native_macros::primitive(
    Bool,
    "and(_)",
    params = [Object],
    returns = Object,
    types = "(Object) -> Object",
    intrinsic = BoolAnd
)]
pub fn bool_and(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if !expect_bool(receiver)? {
        return Ok(FALSE);
    }
    block_call(vm, &args[0], &[])
}

/// Signature: `Bool::or(_)` — sacred, lazy logical disjunction
#[phalcom_native_macros::primitive(
    Bool,
    "or(_)",
    params = [Object],
    returns = Object,
    types = "(Object) -> Object",
    intrinsic = BoolOr
)]
pub fn bool_or(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if expect_bool(receiver)? {
        return Ok(TRUE);
    }
    block_call(vm, &args[0], &[])
}

/// Signature: `Bool::not()` — sacred logical negation.
#[phalcom_native_macros::primitive(
    Bool,
    "not()",
    params = [],
    returns = Bool,
    types = "() -> Bool",
    intrinsic = BoolNot,
    effects = pure
)]
pub fn bool_not(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(if expect_bool(receiver)? { FALSE } else { TRUE })
}

/// Signature: `Bool::ifTrue(_)` — sacred one-armed conditional.
#[phalcom_native_macros::primitive(
    Bool,
    "ifTrue(_)",
    params = [Object],
    returns = Option,
    types = "(Object) -> Option"
)]
pub fn bool_if_true(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if expect_bool(receiver)? {
        let result = block_call(vm, &args[0], &[])?;
        Ok(wrap_some(vm, result)?)
    } else {
        Ok(vm.none_value())
    }
}

/// Signature: `Bool::ifFalse(_)` — sacred one-armed conditional, mirror of
#[phalcom_native_macros::primitive(
    Bool,
    "ifFalse(_)",
    params = [Object],
    returns = Option,
    types = "(Object) -> Option"
)]
pub fn bool_if_false(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if expect_bool(receiver)? {
        Ok(vm.none_value())
    } else {
        let result = block_call(vm, &args[0], &[])?;
        Ok(wrap_some(vm, result)?)
    }
}

/// Signature: `Bool::ifTrue(_)ifFalse(_)` — sacred paired conditional
#[phalcom_native_macros::primitive(
    Bool,
    "ifTrue(_,ifFalse)",
    params = [Object, ifFalse: Object],
    returns = Object,
    types = "(Object, ifFalse: Object) -> Object"
)]
pub fn bool_if_true_if_false(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let branch = if expect_bool(receiver)? { &args[0] } else { &args[1] };
    block_call(vm, branch, &[])
}
