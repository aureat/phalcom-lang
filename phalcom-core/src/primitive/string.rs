//! Native primitives on `String`.
//!
//! REVIEW: BLOCKING ISSUES — IMPLEMENTATION INCOMPLETE / NON-FUNCTIONAL
//! ====================================================================
//! (1) split() crashes unconditionally (core.ph:155 uses bare `List.new`, not `List.new()`)
//! (2) trim/trimStart/trimEnd fail dispatch (declared as getters, called with `()` parens)
//! (3) codePointAt(_) is a stub — multi-byte UTF-8 decode never implemented (lines 112 both return None)
//! (4) Test corpus `tests/fixtures/language/strings/` empty; load-bearing tests shelved in `pending/` with `#[ignore]`
//! (5) Docs not synced: floor-census.md, core-classes.md, deferred-work.md still omit new bindings
//! (6) ADR-0049 collision: two files claim same number, conflicting naming (raw* vs _ suffix per U-NATIVE-MARKER)
//!
//! See inline REVIEW comments in core.php and the language corpus runner for details. Unit blocked pending fixes.

use crate::error::{PhResult, RuntimeError};
use crate::primitive::expect_string;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `String::hash` — the cached djb2 content hash.
#[phalcom_native_macros::primitive(
    String,
    "hash",
    params = [],
    returns = Int,
    types = "() -> Int",
    effects = pure
)]
pub fn string_hash(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = if let Some(id) = receiver.as_obj() {
        if vm.heap.as_string(id).is_some() {
            id
        } else {
            return Err(RuntimeError::Type {
                expected: "String",
                found: receiver.type_name(),
            }
            .into());
        }
    } else {
        return Err(RuntimeError::Type {
            expected: "String",
            found: receiver.type_name(),
        }
        .into());
    };
    let hash = u64::from(vm.heap.string(id).hash());
    Ok(crate::primitive::hash_code(hash))
}

/// Signature: `String::+(_)` — concatenates two strings.
#[phalcom_native_macros::primitive(
    String,
    "+(_)",
    params = [String],
    returns = String,
    types = "(String) -> String",
    effects = pure
)]
pub fn string_add(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let first = expect_string(vm, receiver)?;
    let second = expect_string(vm, &args[0])?;
    Ok(vm.alloc_string_value(first + &second))
}

/// Signature: `String.class::new(_)` — builds a string from its argument.
#[phalcom_native_macros::primitive(
    String,
    "new(_)",
    params = [Object],
    returns = String,
    types = "(Object) -> String",
    side = class
)]
pub fn string_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    match args.first() {
        Some(arg) => {
            let text = arg.to_string(vm);
            Ok(vm.alloc_string_value(text))
        }
        None => Ok(vm.alloc_string_value(String::new())),
    }
}

#[phalcom_native_macros::primitive(String, "new()", side = class)]
pub fn string_class_new_default(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    string_class_new(vm, receiver, &[])
}

/// Signature: `String::_$byteCount` — the byte length of the underlying UTF-8 buffer.
#[phalcom_native_macros::primitive(
    String,
    "_$byteCount",
    params = [],
    returns = Int,
    types = "() -> Int",
    effects = pure,
    visibility = internal
)]
pub fn string_raw_byte_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let s = if let Some(id) = receiver.as_obj() {
        if vm.heap.as_string(id).is_some() {
            vm.heap.string(id).as_str()
        } else {
            return Err(RuntimeError::Type {
                expected: "String",
                found: receiver.type_name(),
            }
            .into());
        }
    } else {
        return Err(RuntimeError::Type {
            expected: "String",
            found: receiver.type_name(),
        }
        .into());
    };
    Ok(Value::int(s.len() as i64))
}

/// Signature: `String::_$byteAt(_)` — read a single raw byte from the buffer.
#[phalcom_native_macros::primitive(
    String,
    "_$byteAt(_)",
    params = [Int],
    returns = "Option<Int>",
    types = "(Int) -> Option<Int>",
    effects = pure,
    visibility = internal
)]
pub fn string_raw_byte_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let s = if let Some(id) = receiver.as_obj() {
        if vm.heap.as_string(id).is_some() {
            vm.heap.string(id).as_str()
        } else {
            return Err(RuntimeError::Type {
                expected: "String",
                found: receiver.type_name(),
            }
            .into());
        }
    } else {
        return Err(RuntimeError::Type {
            expected: "String",
            found: receiver.type_name(),
        }
        .into());
    };

    let idx = if let Some(n) = args[0].as_int() {
        if n < 0 {
            return Ok(vm.none_value());
        }
        n as usize
    } else if let Some(n) = args[0].as_float() {
        if n.fract() != 0.0 || n < 0.0 {
            return Ok(vm.none_value());
        }
        n as usize
    } else {
        return Ok(vm.none_value());
    };

    if idx < s.len() {
        Ok(Value::int(s.as_bytes()[idx] as i64))
    } else {
        Ok(vm.none_value())
    }
}

/// Signature: `String::_$slice(_,_)` — extract a substring by byte range `[start, end)`.
#[phalcom_native_macros::primitive(
    String,
    "_$slice(_,_)",
    params = [Int, Int],
    returns = String,
    types = "(Int, Int) -> String",
    effects = pure,
    visibility = internal
)]
pub fn string_raw_slice(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let s = if let Some(id) = receiver.as_obj() {
        if vm.heap.as_string(id).is_some() {
            vm.heap.string(id).as_str()
        } else {
            return Err(RuntimeError::Type {
                expected: "String",
                found: receiver.type_name(),
            }
            .into());
        }
    } else {
        return Err(RuntimeError::Type {
            expected: "String",
            found: receiver.type_name(),
        }
        .into());
    };

    let start = if let Some(n) = args[0].as_int() {
        if n < 0 {
            return Err(RuntimeError::Type {
                expected: "valid index",
                found: "invalid number",
            }
            .into());
        }
        n as usize
    } else if let Some(n) = args[0].as_float() {
        if n.fract() != 0.0 || n < 0.0 {
            return Err(RuntimeError::Type {
                expected: "valid index",
                found: "invalid number",
            }
            .into());
        }
        n as usize
    } else {
        return Err(RuntimeError::Type {
            expected: "number",
            found: args[0].type_name(),
        }
        .into());
    };

    let end = if let Some(n) = args[1].as_int() {
        if n < 0 {
            return Err(RuntimeError::Type {
                expected: "valid index",
                found: "invalid number",
            }
            .into());
        }
        n as usize
    } else if let Some(n) = args[1].as_float() {
        if n.fract() != 0.0 || n < 0.0 {
            return Err(RuntimeError::Type {
                expected: "valid index",
                found: "invalid number",
            }
            .into());
        }
        n as usize
    } else {
        return Err(RuntimeError::Type {
            expected: "number",
            found: args[1].type_name(),
        }
        .into());
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
