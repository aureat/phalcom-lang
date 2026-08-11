//! Native primitives on `List`.
//!
//! Realizes the [ADR-0019](../../../docs/adr/accepted/0019-freeze-vm-blessed-primitive-floor.md)
//! floor for [ADR-0020](../../../docs/adr/accepted/0020-kernel-list-native-array-protocol.md)'s
//! kernel `List`: allocate, length, indexed get, indexed set, and push
//! (amortized growth is `Vec::push`'s own doubling strategy, so there is no
//! separate "grow" primitive — see the U-LIST return contract). These are
//! internal-only (`length_`/`at_`/`set_`/`push_`), wrapped by the
//! `.ph`-defined public protocol (`size`/`at(_:)`/`add(_:)`/`each(_:)`) in
//! `core.ph`, except `new()` and `toString`, which are public primitives
//! directly (mirroring `String`'s `+(_)`; see the return contract for why
//! `toString` is native rather than `.ph`-defined this unit).

use crate::error::{PhResult, RuntimeError};
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
pub(crate) fn expect_index(value: &Value) -> PhResult<usize> {
    match value {
        Value::Int(n) => {
            if *n < 0 {
                Err(RuntimeError::Type {
                    expected: "a non-negative integer index",
                    found: "int",
                }
                .into())
            } else {
                Ok(*n as usize)
            }
        }
        Value::Float(n) => {
            if !n.is_finite() || *n < 0.0 || n.fract() != 0.0 {
                Err(RuntimeError::Type {
                    expected: "a non-negative integer index",
                    found: "float",
                }
                .into())
            } else {
                Ok(*n as usize)
            }
        }
        other => Err(RuntimeError::Type {
            expected: "a non-negative integer index",
            found: other.type_name(),
        }
        .into()),
    }
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
    Ok(Value::Int(vm.heap.list(id).len() as i64))
}

/// Signature: `List::at_(_)` — raw indexed read.
///
/// The indexed-get floor primitive (ADR-0020 §3); `.ph`'s `at(_:)` wraps
/// this. An out-of-range index surfaces immediate `None`
/// directly — never a panic, never the raw `nil` sentinel (Invariant 4,
/// mirroring U6's absence boundary).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `List`, or if the
/// index is not a non-negative integer `Number`.
pub fn list_raw_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_list(vm, receiver)?;
    let len = vm.heap.list(id).len();
    match super::index::normalize_element_index(&vm.heap, &args[0], len)? {
        super::index::NormalizedIndex::Valid(idx) => match vm.heap.list(id).get(idx) {
            Some(value) => Ok(value),
            None => Ok(vm.none_value()),
        },
        super::index::NormalizedIndex::OutOfRange => Ok(vm.none_value()),
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
    let len = vm.heap.list(id).len();
    let index = match super::index::normalize_element_index(&vm.heap, &args[0], len)? {
        super::index::NormalizedIndex::Valid(idx) => idx,
        super::index::NormalizedIndex::OutOfRange => {
            return Err(RuntimeError::Type {
                expected: "an in-range index",
                found: "an out-of-range Number",
            }
            .into());
        }
    };
    vm.heap.list_mut(id).set(index, args[1]);
    Ok(Value::Unit)
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
    Ok(Value::Unit)
}

/// Signature: `List::replaceSlice_(_,_,_)` — replace a canonical half-open span.
///
/// The bounds have already been normalized by `Range#sliceBounds_` in `core.ph`.
/// Replacement is deliberately restricted to `List` for C.3: accepting an arbitrary
/// iterable would require a boundedness and re-entrant iteration policy. Snapshotting
/// before the mutable borrow also makes `list.replaceSlice_(..., list)` safe when the
/// replacement and destination are the same list.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] when either receiver is not a `List`, either bound
/// is not a non-negative integer, or the bounds do not satisfy `start <= end <= len`.
pub fn list_replace_slice(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let destination: ObjRef = expect_list(vm, receiver)?;
    let replacement: ObjRef = expect_list(vm, &args[2])?;
    let start = expect_index(&args[0])?;
    let end = expect_index(&args[1])?;
    let len = vm.heap.list(destination).len();
    if start > end || end > len {
        return Err(RuntimeError::Type {
            expected: "slice bounds satisfying 0 <= start <= end <= list size",
            found: "invalid slice bounds",
        }
        .into());
    }

    let replacements = vm.heap.list(replacement).elements().to_vec();
    vm.heap.list_mut(destination).replace_slice(start, end, replacements);
    Ok(Value::Unit)
}

/// Signature: `List::toString` — renders as `"[e1, e2, e3]"`.
///
/// A public native primitive rather than a `.ph` method wrapping `each(_:)`
/// (see the return contract): rendering an element correctly requires
/// sending each element `toString` ([`Value::to_display_string`]), so that
/// a user `toString` override on an element (or a nested container) is
/// honored recursively; building this via `.ph` `each` + string
/// interpolation would work too but pays an extra `.ph` call-frame per
/// element for no benefit over doing the send natively here.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `List`.
/// Propagates any error an element's `toString` send raises.
pub fn list_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_list(vm, receiver)?;
    let elements: Vec<Value> = vm.heap.list(id).elements().to_vec();
    let mut parts: Vec<String> = Vec::with_capacity(elements.len());
    for v in &elements {
        parts.push(v.to_display_string(vm)?);
    }
    Ok(vm.alloc_string_value(format!("[{}]", parts.join(", "))))
}
