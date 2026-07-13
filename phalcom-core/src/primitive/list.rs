//! Native primitives on `List`.
//!
//! Realizes the [ADR-0019](../../../docs/adr/0019-freeze-vm-blessed-primitive-floor.md)
//! floor for [ADR-0020](../../../docs/adr/0020-kernel-list-native-array-protocol.md)'s
//! kernel `List`: allocate, length, indexed get, indexed set, and push
//! (amortized growth is `Vec::push`'s own doubling strategy, so there is no
//! separate "grow" primitive — see the U-LIST return contract). These are
//! internal-only (`length_`/`at_`/`set_`/`push_`), wrapped by the
//! `.ph`-defined public protocol (`size`/`at(_:)`/`add(_:)`/`each(_:)`) in
//! `core.ph`, except `new()` and `toString`, which are public primitives
//! directly (mirroring `String`'s `+(_)`; see the return contract for why
//! `toString` is native rather than `.ph`-defined this unit).

use crate::error::{PhResult, RuntimeError};
use crate::expect_value;
use crate::heap::ObjRef;
use crate::primitive::expect_list;
use crate::value::Value;
use crate::vm::VM;

/// Extracts a non-negative integer index from `value`.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a `Number`, or is
/// negative, fractional, or non-finite — an index must be a whole,
/// non-negative, finite count (`U-LIST-plan.md` §3: a malformed index is a
/// hard type error, never a silent wrap or truncation).
fn expect_index(value: &Value) -> PhResult<usize> {
    let n = expect_value!(value, Number);
    if !n.is_finite() || n < 0.0 || n.fract() != 0.0 {
        return Err(RuntimeError::Type {
            expected: "a non-negative integer index",
            found: value.type_name(),
        }
        .into());
    }
    Ok(n as usize)
}

/// Signature: `List.class::new()` — allocates an empty list.
///
/// The allocate floor primitive (ADR-0020 §3).
pub fn list_class_new(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::Obj(vm.heap.alloc_list(Vec::new())))
}

/// Signature: `List::length_` — the list's element count.
///
/// The length floor primitive (ADR-0020 §3); `.ph`'s `size` getter wraps
/// this.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `List`.
pub fn list_raw_length(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_list(vm, receiver)?;
    Ok(Value::Number(vm.heap.list(id).len() as f64))
}

/// Signature: `List::at_(_)` — raw indexed read.
///
/// The indexed-get floor primitive (ADR-0020 §3); `.ph`'s `at(_:)` wraps
/// this. An out-of-range index surfaces the kernel `None` singleton
/// directly — never a panic, never the raw `nil` sentinel (Invariant 4,
/// mirroring U6's absence boundary).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `List`, or if the
/// index is not a non-negative integer `Number`.
pub fn list_raw_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_list(vm, receiver)?;
    let index = expect_index(&args[0])?;
    match vm.heap.list(id).get(index) {
        Some(value) => Ok(value),
        None => Ok(vm.none_value()),
    }
}

/// Signature: `List::set_(_,_)` — raw indexed write.
///
/// The indexed-set floor primitive (ADR-0020 §3). Implemented but **not**
/// surfaced at the `.ph` layer this unit — no `at(_:put:)` selector exists
/// yet (see the return contract's `DEFERRED.md` entry).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `List`, the index
/// is not a non-negative integer `Number`, or the index is out of range.
pub fn list_raw_set(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_list(vm, receiver)?;
    let index = expect_index(&args[0])?;
    let list = vm.heap.list_mut(id);
    if index >= list.len() {
        return Err(RuntimeError::Type {
            expected: "an in-range index",
            found: "an out-of-range Number",
        }
        .into());
    }
    list.set(index, args[1]);
    Ok(vm.none_value())
}

/// Signature: `List::push_(_)` — appends one element.
///
/// The push floor primitive (ADR-0020 §3), backed by `Vec::push`'s own
/// amortized growth (folding the "grow" primitive into this one — see the
/// return contract). `.ph`'s `add(_:)` wraps this and returns `self` for
/// chaining.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `List`.
pub fn list_raw_push(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_list(vm, receiver)?;
    vm.heap.list_mut(id).push(args[0]);
    Ok(vm.none_value())
}

/// Signature: `List::toString` — renders as `"[e1, e2, e3]"`.
///
/// A public native primitive rather than a `.ph` method wrapping `each(_:)`
/// (see the return contract): rendering an element correctly requires
/// [`Value::to_string`], which is not itself reachable as a `.ph`-callable
/// message (e.g. `Number` has no user-facing `toString` primitive yet), so
/// building this via `.ph` `each` + per-element `.toString` would render
/// every non-`String` element as its `Object` default (`"<ClassName>"`)
/// instead of its value.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `List`.
pub fn list_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_list(vm, receiver)?;
    let elements: Vec<Value> = vm.heap.list(id).elements().to_vec();
    let rendered: Vec<String> = elements.iter().map(|v| v.to_string(vm)).collect();
    Ok(vm.alloc_string_value(format!("[{}]", rendered.join(", "))))
}
