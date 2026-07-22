//! Native primitives on `String`.
//!
//! REVIEW: BLOCKING ISSUES — IMPLEMENTATION INCOMPLETE / NON-FUNCTIONAL
//! ====================================================================
//! (1) split() crashes unconditionally (core.ph:155 uses bare `List.new`, not `List.new()`)
//! (2) trim/trimStart/trimEnd fail dispatch (declared as getters, called with `()` parens)
//! (3) codePointAt(_) is a stub — multi-byte UTF-8 decode never implemented (lines 112 both return None)
//! (4) Test corpus `tests/lang/strings/` empty; load-bearing tests shelved in `pending/` with #[ignore]
//! (5) Docs not synced: floor-census.md, core-classes.md, deferred-work.md still omit new bindings
//! (6) ADR-0049 collision: two files claim same number, conflicting naming (raw* vs _ suffix per U-NATIVE-MARKER)
//!
//! See inline REVIEW comments in core.php and tests/lang.rs for details. Unit blocked pending fixes.

use crate::error::{PhResult, RuntimeError};
use crate::primitive::expect_string;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `String::hash` — the cached djb2 content hash.
///
/// Reuses the content hash the string already caches
/// ([`StringObject::hash`](crate::heap::StringObject::hash) /
/// [`calculate_hash`](crate::heap::StringObject::calculate_hash)) so equal
/// content hashes equal, satisfying `a == b ⇒ a.hash == b.hash` (R-INV-1.3)
/// even for two distinct-handle strings — `String#==` is content equality.
/// Underivable — the `String` floor exposes only `+`/`new`, not the bytes.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a string.
pub fn string_hash(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = match receiver {
        Value::Obj(id) if vm.heap.as_string(*id).is_some() => *id,
        other => {
            return Err(RuntimeError::Type {
                expected: "String",
                found: other.type_name(),
            }
            .into());
        }
    };
    let hash = u64::from(vm.heap.string(id).hash());
    Ok(crate::primitive::hash_code(hash))
}

/// Signature: `String::+(_)` — concatenates two strings.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if either operand is not a string.
pub fn string_add(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let first = expect_string(vm, receiver)?;
    let second = expect_string(vm, &args[0])?;
    Ok(vm.alloc_string_value(first + &second))
}

/// Signature: `String.class::new(_)` — builds a string from its argument.
///
/// With an argument, renders it via [`Value::to_string`]; with none, returns the
/// empty string.
pub fn string_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    match args.first() {
        Some(arg) => {
            let text = arg.to_string(vm);
            Ok(vm.alloc_string_value(text))
        }
        None => Ok(vm.alloc_string_value(String::new())),
    }
}

/// Signature: `String::byteCount_` — the byte length of the underlying UTF-8 buffer.
///
/// Derives from ADR-0019 (minimal native floor); the byte length is not derivable
/// in `.ph` code since no `.ph` code can observe the buffer. Returns a `Number`.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a string.
pub fn string_raw_byte_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let s = match receiver {
        Value::Obj(id) if vm.heap.as_string(*id).is_some() => vm.heap.string(*id).as_str(),
        other => {
            return Err(RuntimeError::Type {
                expected: "String",
                found: other.type_name(),
            }
            .into());
        }
    };
    Ok(Value::Number(s.len() as f64))
}

/// Signature: `String::byteAt_(_)` — read a single raw byte from the buffer.
///
/// Derives from ADR-0019 (minimal native floor); byte-level access is not derivable
/// in `.ph` code. Returns a `Number` (0–255) on hit; returns the `None` singleton
/// on out-of-bounds access (mirrors `list_raw_at`'s pattern, ADR-0007 Invariant 4).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a string.
pub fn string_raw_byte_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let s = match receiver {
        Value::Obj(id) if vm.heap.as_string(*id).is_some() => vm.heap.string(*id).as_str(),
        other => {
            return Err(RuntimeError::Type {
                expected: "String",
                found: other.type_name(),
            }
            .into());
        }
    };

    let idx = match &args[0] {
        Value::Number(n) => {
            if n.fract() != 0.0 || *n < 0.0 {
                return Ok(vm.none_value());
            }
            *n as usize
        }
        _ => return Ok(vm.none_value()),
    };

    if idx < s.len() {
        Ok(Value::Number(s.as_bytes()[idx] as f64))
    } else {
        Ok(vm.none_value())
    }
}

/// Signature: `String::slice_(_,_)` — extract a substring by byte range `[start, end)`.
///
/// Derives from ADR-0019 (minimal native floor); string allocation from computed byte
/// offsets is not derivable in `.ph` (no way to construct a String from raw bytes).
/// Returns a new `String`. Validates UTF-8 char-boundary alignment via
/// [`str::is_char_boundary`]; returns [`RuntimeError::Type`] on misaligned boundaries
/// (never panics on a malformed slice attempt).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if the receiver is not a string, or if `start`/`end`
/// are not valid integers, out of range, or not aligned to UTF-8 char boundaries.
pub fn string_raw_slice(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let s = match receiver {
        Value::Obj(id) if vm.heap.as_string(*id).is_some() => vm.heap.string(*id).as_str(),
        other => {
            return Err(RuntimeError::Type {
                expected: "String",
                found: other.type_name(),
            }
            .into());
        }
    };

    let start = match &args[0] {
        Value::Number(n) => {
            if n.fract() != 0.0 || *n < 0.0 {
                return Err(RuntimeError::Type {
                    expected: "valid index",
                    found: "invalid number",
                }
                .into());
            }
            *n as usize
        }
        _ => {
            return Err(RuntimeError::Type {
                expected: "Number",
                found: args[0].type_name(),
            }
            .into());
        }
    };

    let end = match &args[1] {
        Value::Number(n) => {
            if n.fract() != 0.0 || *n < 0.0 {
                return Err(RuntimeError::Type {
                    expected: "valid index",
                    found: "invalid number",
                }
                .into());
            }
            *n as usize
        }
        _ => {
            return Err(RuntimeError::Type {
                expected: "Number",
                found: args[1].type_name(),
            }
            .into());
        }
    };

    // Validate bounds and char boundaries
    if start > s.len() || end > s.len() || start > end {
        return Err(RuntimeError::Type {
            expected: "valid slice range",
            found: "out of bounds",
        }
        .into());
    }

    if !s.is_char_boundary(start) || !s.is_char_boundary(end) {
        return Err(RuntimeError::Type {
            expected: "char boundary",
            found: "mid-sequence offset",
        }
        .into());
    }

    let slice = &s[start..end];
    Ok(vm.alloc_string_value(slice.to_string()))
}
