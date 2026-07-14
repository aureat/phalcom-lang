//! Native primitives on `Range`.
//!
//! Realizes the [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)
//! floor for [ADR-0032](../../../docs/adr/accepted/0032-collections-representation-and-literals.md)'s
//! native `Range`: allocate from three bound fields, plus the three raw field
//! reads (`start_`/`end_`/`inclusive_`). That is the **whole** floor —
//! `size`/`at(_)`/`includes(_)`/`first`/`last`/`each(_)`/`toList`/`==`/`hash`
//! are all `.ph` over these + `Number` arithmetic
//! (`docs/spec/v0.2/core/tuple-and-range.md` §2). `new(_,_,_)` is a public
//! static primitive directly (mirroring `List::new()`), the construction path
//! until the `a..b`/`a...b` literal sigils activate (ADR-0032 §3.3,
//! reserved-inactive; RG-1's bound convention is honored now).

use crate::error::PhResult;
use crate::heap::ObjRef;
use crate::primitive::expect_range;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Range.class::new(start, end, inclusive)` — builds a range from
/// its three bound fields, with no validation beyond what `Number`/`Bool`
/// arithmetic naturally enforces downstream in `.ph` (a non-`Number`
/// `start`/`end` simply fails the first `.ph`-side arithmetic send, e.g.
/// `size`'s `end - start`, rather than being rejected here — mirrors how
/// `List.new()` does not validate its (absent) arguments).
pub fn range_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let start = args[0];
    let end = args[1];
    let inclusive = matches!(args[2], Value::Bool(true));
    Ok(Value::Obj(vm.heap.alloc_range(start, end, inclusive)))
}

/// Signature: `Range::start_` — the start bound (`Range#first`'s source).
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Range`.
pub fn range_raw_start(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_range(vm, receiver)?;
    Ok(vm.heap.range(id).start())
}

/// Signature: `Range::end_` — the end bound (inclusive/exclusive per
/// `inclusive_`).
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Range`.
pub fn range_raw_end(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_range(vm, receiver)?;
    Ok(vm.heap.range(id).end())
}

/// Signature: `Range::inclusive_` — `true` for `a..b` (inclusive of the end
/// bound), `false` for `a...b` (exclusive) — RG-1's bound convention.
///
/// # Errors
///
/// Returns [`crate::error::RuntimeError::Type`] if the receiver is not a `Range`.
pub fn range_raw_inclusive(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_range(vm, receiver)?;
    Ok(Value::Bool(vm.heap.range(id).inclusive()))
}
