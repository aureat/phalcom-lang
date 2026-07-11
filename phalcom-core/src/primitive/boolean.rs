//! Native primitives on `Bool`.
//!
//! Beyond `Bool.class::new(_)`, this module registers the sacred-selector
//! *fallbacks* — the real message-send implementations of `and(_)`, `or(_)`,
//! `not()`, `ifTrue(_)`, `ifFalse(_)` and `ifTrue(_)ifFalse(_)` (control-flow.md
//! §2–3). They are what every `Bool`-receiver sacred send resolves to
//! whether or not the compiler's inliner ([ADR-0017](../../../docs/adr/0017-sacred-selector-inliner.md))
//! took the fast path for a given call site: the inliner's guarded jump
//! opcodes are an optimization over calling these, never a divergent
//! reimplementation of their semantics.

use crate::boolean::{FALSE, TRUE};
use crate::error::{PhResult, RuntimeError};
use crate::nil::NIL;
use crate::primitive::block::block_call;
use crate::primitive::expect_class;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Bool.class::new(_)` — coerces its argument to a boolean.
///
/// `false`, `nil` and `0` become `false`; every other value becomes `true`.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the
/// receiver is not a class.
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
        Value::Number(n) => Ok(if *n != 0.0 { TRUE } else { FALSE }),
        _ => Ok(TRUE),
    }
}

/// Extracts the `bool` payload of a `Bool` receiver.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a `Bool`.
fn expect_bool(value: &Value) -> PhResult<bool> {
    match value {
        Value::Bool(b) => Ok(*b),
        other => Err(RuntimeError::Type { expected: "Bool", found: other.type_name() }.into()),
    }
}

/// Signature: `Bool::and(_)` — sacred, lazy logical conjunction
/// (control-flow.md §2). Short-circuits to `false` without calling `args[0]`
/// when the receiver is `false`; otherwise calls the block and returns its
/// result *as-is* (not coerced to `Bool`), matching Smalltalk `and:`
/// semantics. Ordinary, overridable method — this is the fallback the
/// inliner's `GuardBool` deopt path sends to.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Bool`, or any
/// error raised calling the block.
pub fn bool_and(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if !expect_bool(receiver)? {
        return Ok(FALSE);
    }
    block_call(vm, &args[0], &[])
}

/// Signature: `Bool::or(_)` — sacred, lazy logical disjunction
/// (control-flow.md §2). Short-circuits to `true` without calling `args[0]`
/// when the receiver is `true`; otherwise calls the block and returns its
/// result as-is. See [`bool_and`] for the shared design notes.
///
/// # Errors
///
/// See [`bool_and`].
pub fn bool_or(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if expect_bool(receiver)? {
        return Ok(TRUE);
    }
    block_call(vm, &args[0], &[])
}

/// Signature: `Bool::not()` — sacred logical negation.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Bool`.
pub fn bool_not(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(if expect_bool(receiver)? { FALSE } else { TRUE })
}

/// Signature: `Bool::ifTrue(_)` — sacred one-armed conditional.
///
/// Calls and returns `args[0]`'s block result when the receiver is `true`;
/// otherwise returns the private `nil` sentinel (the "no branch taken"
/// placeholder — U6 reroutes every such surface `nil` to `None`, U5 leaves
/// it as-is per its guardrails).
///
/// # Errors
///
/// See [`bool_and`].
pub fn bool_if_true(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if expect_bool(receiver)? {
        block_call(vm, &args[0], &[])
    } else {
        Ok(NIL)
    }
}

/// Signature: `Bool::ifFalse(_)` — sacred one-armed conditional, mirror of
/// [`bool_if_true`] for the `false` branch.
///
/// # Errors
///
/// See [`bool_and`].
pub fn bool_if_false(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if expect_bool(receiver)? {
        Ok(NIL)
    } else {
        block_call(vm, &args[0], &[])
    }
}

/// Signature: `Bool::ifTrue(_)ifFalse(_)` — sacred paired conditional
/// (control-flow.md §1: `if (c) {A} else {B}` desugars here, *not* to the
/// spec's illustrative `ifTrue{}.ifNone{}` `Option` chain — see U5-plan.md
/// §4.1, keeping U5 independent of U6's `Option`). Calls and returns
/// `args[0]` (the `if` arm) when the receiver is `true`, `args[1]` (the
/// `else` arm) otherwise.
///
/// # Errors
///
/// See [`bool_and`].
pub fn bool_if_true_if_false(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let branch = if expect_bool(receiver)? { &args[0] } else { &args[1] };
    block_call(vm, branch, &[])
}
