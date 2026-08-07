//! Native primitives on `Bytes`.
//!
//! Realizes the [PDR-0011](../../../docs/decisions/0011-admit-bytes-native-octet-buffer.md)
//! ruling-3 floor for the kernel octet buffer — ten primitives from that record
//! plus [PDR-0013](../../../docs/decisions/0013-path-is-bytes-backed-filesystem-surface.md)
//! ruling 4's `utf8Lossy_` — under the container bulk-op posture: bulk
//! operations with no user code in their loop are native; every block-taking
//! selector stays `.ph` so `Fiber#yield` works mid-iteration
//! (`docs/spec/v0.2/core/bytes.md` §3.1). Return conventions mirror
//! [`crate::primitive::list`] exactly: fallible reads return the bare value or
//! the kernel `None`, bad writes are [`RuntimeError::Type`] errors, and
//! unit-returning operations return `None`.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{BytesObject, ObjRef};
use crate::primitive::expect_bytes;
use crate::primitive::list::expect_index;
use crate::value::Value;
use crate::vm::VM;

/// Extracts an octet — an integer `Number` in `0..=255` — from `value`.
///
/// The `bytes.md` §2 element contract, worded representation-independently:
/// under PDR-0012's tower these become small `Int`s with no change here.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a `Number`, or is
/// fractional, non-finite, negative, or above 255 (`bytes.md` law 2: no path
/// by which a non-octet enters a buffer).
fn expect_octet(value: &Value) -> PhResult<u8> {
    if let Value::Int(n) = value {
        if (0..=255).contains(n) {
            return Ok(*n as u8);
        }
    } else if let Value::Float(n) = value {
        if n.is_finite() && n.fract() == 0.0 && (0.0..=255.0).contains(n) {
            return Ok(*n as u8);
        }
    }
    Err(RuntimeError::Type {
        expected: "an integer Number in 0..=255",
        found: value.type_name(),
    }
    .into())
}

/// Signature: `Bytes.class::new(_)` — allocates a zero-filled buffer.
///
/// The allocate floor primitive (PDR-0011 ruling 3). Zero-filled birth is
/// `bytes.md` law 4: no constructor exposes uninitialized memory.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the length is not a non-negative
/// integer `Number`.
pub fn bytes_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let len = expect_index(&args[0])?;
    Ok(Value::Obj(vm.heap.alloc_bytes(BytesObject::new_zeroed(len))))
}

/// Signature: `Bytes.class::fromString_(_)` — a `String`'s UTF-8 bytes.
///
/// One native copy (PDR-0011 ruling 3's bulk-op posture); the `.ph` per-byte
/// alternative over ADR-0049's `byteAt_(_)` is exactly the dispatch cost the
/// arm exists to remove.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the argument is not a `String`.
pub fn bytes_class_from_string(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let string = crate::primitive::expect_string(vm, &args[0])?;
    let octets = string.into_bytes();
    Ok(Value::Obj(vm.heap.alloc_bytes(BytesObject::from_vec(octets))))
}

/// Signature: `Bytes::size_` — the octet count.
///
/// Constant for the buffer's lifetime (`bytes.md` law 3); `.ph`'s `size`
/// getter wraps this.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Bytes`.
pub fn bytes_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_bytes(vm, receiver)?;
    Ok(Value::Int(vm.heap.bytes(id).len() as i64))
}

/// Signature: `Bytes::at_(_)` — raw octet read.
///
/// Total: an out-of-range index surfaces the kernel `None` singleton, never
/// a panic and never the raw `nil` sentinel — `list_raw_at`'s exact
/// convention (`bytes.md` §3). Unlike `List`, the bare-or-`None` union is
/// unambiguous: an octet is never `None`.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Bytes` or the
/// index is not a non-negative integer `Number`.
pub fn bytes_raw_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_bytes(vm, receiver)?;
    let index = expect_index(&args[0])?;
    match vm.heap.bytes(id).get(index) {
        Some(octet) => Ok(Value::Int(octet as i64)),
        None => Ok(vm.none_value()),
    }
}

/// Signature: `Bytes::set_(_,_)` — raw octet write.
///
/// `list_raw_set`'s exact convention: a bad write is a hard type error, and
/// success returns the kernel `None` (`.ph`'s `set` lifts the error into a
/// raise and returns the receiver).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Bytes`, the
/// index is malformed or out of range, or the value is not an octet.
pub fn bytes_raw_set(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_bytes(vm, receiver)?;
    let index = expect_index(&args[0])?;
    let octet = expect_octet(&args[1])?;
    if !vm.heap.bytes_mut(id).set(index, octet) {
        return Err(RuntimeError::Type {
            expected: "an in-range index",
            found: "an out-of-range Number",
        }
        .into());
    }
    Ok(vm.none_value())
}

/// Signature: `Bytes::fill_(_)` — overwrite every octet (one memset).
///
/// `fill_(0)` **is** `zeroize` (`bytes.md` §7): complete because the backing
/// `Box<[u8]>` never reallocates, so no octet that ever held a secret lies
/// outside the buffer being filled.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Bytes` or the
/// value is not an octet.
pub fn bytes_raw_fill(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_bytes(vm, receiver)?;
    let octet = expect_octet(&args[0])?;
    vm.heap.bytes_mut(id).as_mut_slice().fill(octet);
    Ok(vm.none_value())
}

/// Signature: `Bytes::slice_(_,_)` — copy `[start, end)` into a fresh buffer.
///
/// Always a copy, never a view: under ADR-0050's non-moving collector a view
/// would retain its whole parent — Erlang's binary leak (PDR-0011 ruling 5).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Bytes`, either
/// bound is malformed, or the range does not satisfy
/// `start <= end <= size`.
pub fn bytes_raw_slice(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_bytes(vm, receiver)?;
    let start = expect_index(&args[0])?;
    let end = expect_index(&args[1])?;
    let source = vm.heap.bytes(id);
    if start > end || end > source.len() {
        return Err(RuntimeError::Type {
            expected: "a range with start <= end <= size",
            found: "an out-of-range slice bound",
        }
        .into());
    }
    let copy = source.as_slice()[start..end].to_vec();
    Ok(Value::Obj(vm.heap.alloc_bytes(BytesObject::from_vec(copy))))
}

/// Signature: `Bytes::copyInto_(_,_)` — copy the whole receiver into another
/// `Bytes` at an offset (one memmove).
///
/// The primitive that lets `.ph` `concat` and stream buffering run with zero
/// per-byte loops (PDR-0011 ruling 3). An overlapping self-copy behaves as
/// memmove (`bytes.md` law 6) — with a fixed-length buffer the only in-bounds
/// self-copy is offset 0, handled by `copy_within`.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver or first argument is not a
/// `Bytes`, the offset is malformed, or `offset + size` overflows the
/// destination.
pub fn bytes_raw_copy_into(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let src_id: ObjRef = expect_bytes(vm, receiver)?;
    let dst_id: ObjRef = expect_bytes(vm, &args[0])?;
    let offset = expect_index(&args[1])?;
    let src_len = vm.heap.bytes(src_id).len();
    let dst_len = vm.heap.bytes(dst_id).len();
    if offset.checked_add(src_len).is_none_or(|end| end > dst_len) {
        return Err(RuntimeError::Type {
            expected: "an offset with offset + size <= destination size",
            found: "an out-of-range offset",
        }
        .into());
    }
    if src_id == dst_id {
        // Only offset 0 can be in bounds for a whole-buffer self-copy;
        // `copy_within` gives memmove semantics regardless.
        vm.heap.bytes_mut(dst_id).as_mut_slice().copy_within(0..src_len, offset);
    } else {
        let (src, dst) = vm.heap.bytes_pair_mut(src_id, dst_id).expect("both handles were just validated as live Bytes");
        dst.as_mut_slice()[offset..offset + src_len].copy_from_slice(src.as_slice());
    }
    Ok(vm.none_value())
}

/// Signature: `Bytes::utf8_` — strict, fallible UTF-8 decode.
///
/// The decoded `String`, or the kernel `None` on any invalid sequence —
/// never a raise, never a truncation (`bytes.md` law 7). No other primitive
/// builds a `String` from arbitrary octets, which is the ADR-0019
/// inexpressibility ground it was admitted on.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Bytes`.
pub fn bytes_raw_utf8(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_bytes(vm, receiver)?;
    match std::str::from_utf8(vm.heap.bytes(id).as_slice()) {
        Ok(text) => {
            let owned = text.to_string();
            Ok(Value::Obj(vm.heap.alloc_string(owned)))
        }
        Err(_) => Ok(vm.none_value()),
    }
}

/// Signature: `Bytes::utf8Lossy_` — total lossy UTF-8 decode.
///
/// Invalid sequences become U+FFFD (Rust `from_utf8_lossy`). Admitted by
/// PDR-0013 ruling 4 for `Path#toString`: display must be total, and a lossy
/// decode is a no-user-code bulk operation `.ph` cannot express. Display
/// only — never round-trip the result into data (`filesystem.md` §2 law 2).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Bytes`.
pub fn bytes_raw_utf8_lossy(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_bytes(vm, receiver)?;
    let owned = String::from_utf8_lossy(vm.heap.bytes(id).as_slice()).into_owned();
    Ok(Value::Obj(vm.heap.alloc_string(owned)))
}

/// Signature: `Bytes::equalsConstantTime_(_)` — content equality in time
/// constant with respect to contents.
///
/// The one selector whose *timing* is part of its contract (`bytes.md` §8):
/// a `.ph` loop — and `.ph` `==` — short-circuits, which is correct there
/// and disqualifying here. Length mismatch returns `false` without
/// inspecting contents; **lengths are not concealed** (Go
/// `crypto/subtle.ConstantTimeCompare`'s shape; Node's throw-on-mismatch
/// leaks length through the exception path and was rejected, PDR-0011
/// ruling 6). The XOR-accumulate has no early exit, and
/// [`std::hint::black_box`] keeps the optimizer from rewriting it into a
/// short-circuiting `memcmp`.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a `Bytes`, or the
/// argument is not a `Bytes` — a non-`Bytes` argument raises rather than
/// comparing `false`, because silently answering `false` would hide a type
/// bug in the security-critical path (`bytes.md` §8.3).
pub fn bytes_raw_equals_constant_time(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let a_id: ObjRef = expect_bytes(vm, receiver)?;
    let b_id: ObjRef = expect_bytes(vm, &args[0])?;
    let a = vm.heap.bytes(a_id).as_slice();
    let b = vm.heap.bytes(b_id).as_slice();
    if a.len() != b.len() {
        return Ok(Value::Bool(false));
    }
    let mut acc: u8 = 0;
    for i in 0..a.len() {
        acc |= a[i] ^ b[i];
    }
    Ok(Value::Bool(std::hint::black_box(acc) == 0))
}
