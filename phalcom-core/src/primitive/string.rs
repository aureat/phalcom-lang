//! Native primitives on `String`.

use crate::error::{PhResult, RuntimeError};
use crate::primitive::expect_string;
use crate::value::Value;
use crate::vm::VM;
use num_traits::ToPrimitive;

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

/// Signature: `String::rawByteCount` — the byte length of the underlying UTF-8 buffer.
#[phalcom_native_macros::primitive(
    String,
    "rawByteCount",
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

/// Signature: `String::rawByteAt(_)` — read a single raw byte from the buffer.
#[phalcom_native_macros::primitive(
    String,
    "rawByteAt(_)",
    params = [Int],
    returns = "Option<Int>",
    types = "(Int) -> Option<Int>",
    effects = pure,
    visibility = internal
)]
pub fn string_raw_byte_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let s = expect_string(vm, receiver)?;
    let int_val = crate::primitive::int::expect_int_big(&args[0], vm)?;
    let Some(idx) = int_val.to_usize() else {
        return Ok(Value::none());
    };

    if idx < s.len() {
        Ok(Value::int(s.as_bytes()[idx] as i64).wrap_some().unwrap())
    } else {
        Ok(Value::none())
    }
}

/// Signature: `String::rawSlice(_,_)` — extract a substring by byte range `[start, end)`.
#[phalcom_native_macros::primitive(
    String,
    "rawSlice(_,_)",
    params = [Int, Int],
    returns = String,
    types = "(Int, Int) -> String",
    effects = pure,
    visibility = internal
)]
pub fn string_raw_slice(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let s = expect_string(vm, receiver)?;
    let start_big = crate::primitive::int::expect_int_big(&args[0], vm)?;
    let end_big = crate::primitive::int::expect_int_big(&args[1], vm)?;

    let Some(start) = start_big.to_usize() else {
        return Err(RuntimeError::Type {
            expected: "valid slice range",
            found: "out of bounds",
        }
        .into());
    };

    let Some(end) = end_big.to_usize() else {
        return Err(RuntimeError::Type {
            expected: "valid slice range",
            found: "out of bounds",
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

/// Signature: `String::isCharBoundary(_)` — returns whether `byteOffset` is a valid UTF-8 boundary.
#[phalcom_native_macros::primitive(
    String,
    "isCharBoundary(_)",
    params = [Int],
    returns = Bool,
    types = "(Int) -> Bool",
    effects = pure
)]
pub fn string_is_char_boundary(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let s = expect_string(vm, receiver)?;
    let int_val = crate::primitive::int::expect_int_big(&args[0], vm)?;
    let Some(offset) = int_val.to_usize() else {
        return Ok(Value::bool(false));
    };
    if offset > s.len() {
        return Ok(Value::bool(false));
    }
    Ok(Value::bool(s.is_char_boundary(offset)))
}

/// Signature: `String::decodeCharAt(_)` — decodes the Unicode scalar at byte offset `byteOffset`.
#[phalcom_native_macros::primitive(
    String,
    "decodeCharAt(_)",
    params = [Int],
    returns = "Option<Char>",
    types = "(Int) -> Option<Char>",
    effects = pure
)]
pub fn string_decode_char_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let s = expect_string(vm, receiver)?;
    let int_val = crate::primitive::int::expect_int_big(&args[0], vm)?;
    let Some(offset) = int_val.to_usize() else {
        return Ok(Value::none());
    };
    if offset >= s.len() || !s.is_char_boundary(offset) {
        return Ok(Value::none());
    }
    let ch = s[offset..].chars().next().expect("valid non-EOF char boundary");
    Ok(Value::char(ch).wrap_some().unwrap())
}

/// Signature: `String::lowercase()` — converts the string to lowercase using default Unicode case mapping.
#[phalcom_native_macros::primitive(
    String,
    "lowercase()",
    params = [],
    returns = String,
    types = "() -> String",
    effects = pure
)]
pub fn string_lowercase(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let s = expect_string(vm, receiver)?;
    Ok(vm.alloc_string_value(s.to_lowercase()))
}

/// Signature: `String::uppercase()` — converts the string to uppercase using default Unicode case mapping.
#[phalcom_native_macros::primitive(
    String,
    "uppercase()",
    params = [],
    returns = String,
    types = "() -> String",
    effects = pure
)]
pub fn string_uppercase(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let s = expect_string(vm, receiver)?;
    Ok(vm.alloc_string_value(s.to_uppercase()))
}

/// Signature: `String::rawEquals(_)` — tests string content equality.
#[phalcom_native_macros::primitive(
    String,
    "rawEquals(_)",
    params = [Object],
    returns = Bool,
    types = "(Object) -> Bool",
    effects = pure,
    visibility = internal
)]
pub fn string_raw_equals(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(receiver.value_eq(&args[0], &vm.heap)))
}

/// Signature: `String::rawFindSlice(_,_,_)` — searches for substring within byte bounds.
#[phalcom_native_macros::primitive(
    String,
    "rawFindSlice(_,_,_)",
    params = [String, Int, Int],
    returns = "Option<Int>",
    types = "(String, Int, Int) -> Option<Int>",
    effects = pure,
    visibility = internal
)]
pub fn string_raw_find_slice(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let s = expect_string(vm, receiver)?;
    let needle = expect_string(vm, &args[0])?;
    let start_big = crate::primitive::int::expect_int_big(&args[1], vm)?;
    let end_big = crate::primitive::int::expect_int_big(&args[2], vm)?;
    let Some(start) = start_big.to_usize() else {
        return Ok(Value::none());
    };
    let Some(end) = end_big.to_usize() else {
        return Ok(Value::none());
    };
    if start > end || end > s.len() {
        return Ok(Value::none());
    }
    if !s.is_char_boundary(start) || !s.is_char_boundary(end) {
        return Ok(Value::none());
    }
    let slice = &s[start..end];
    match slice.find(needle.as_str()) {
        Some(offset) => Ok(Value::int(offset as i64).wrap_some().unwrap()),
        None => Ok(Value::none()),
    }
}

/// Signature: `String.class::rawJoin(_)` — joins a list of strings into one without intermediate allocations.
#[phalcom_native_macros::primitive(String, "rawJoin(_)", side = class, visibility = internal)]
pub fn string_class_raw_join(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let list_ref = match args[0].as_obj() {
        Some(r) => r,
        None => return Err(RuntimeError::Type { expected: "List", found: args[0].type_name() }.into()),
    };
    let elements = vm.heap.list(list_ref).elements().to_vec();
    let mut total_len = 0;
    for elem in &elements {
        if let Some(s_ref) = elem.as_obj() {
            total_len += vm.heap.string(s_ref).as_str().len();
        }
    }
    let mut res = String::with_capacity(total_len);
    for elem in &elements {
        if let Some(s_ref) = elem.as_obj() {
            res.push_str(vm.heap.string(s_ref).as_str());
        }
    }
    Ok(vm.alloc_string_value(res))
}

/// Signature: `String::rawRFindSlice(_,_,_)` — searches backwards for substring within byte bounds.
#[phalcom_native_macros::primitive(
    String,
    "rawRFindSlice(_,_,_)",
    params = [String, Int, Int],
    returns = "Option<Int>",
    types = "(String, Int, Int) -> Option<Int>",
    effects = pure,
    visibility = internal
)]
pub fn string_raw_rfind_slice(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let s = expect_string(vm, receiver)?;
    let needle = expect_string(vm, &args[0])?;
    let start_big = crate::primitive::int::expect_int_big(&args[1], vm)?;
    let end_big = crate::primitive::int::expect_int_big(&args[2], vm)?;
    let Some(start) = start_big.to_usize() else {
        return Ok(Value::none());
    };
    let Some(end) = end_big.to_usize() else {
        return Ok(Value::none());
    };
    if start > end || end > s.len() {
        return Ok(Value::none());
    }
    if !s.is_char_boundary(start) || !s.is_char_boundary(end) {
        return Ok(Value::none());
    }
    let slice = &s[start..end];
    match slice.rfind(needle.as_str()) {
        Some(offset) => Ok(Value::int(offset as i64).wrap_some().unwrap()),
        None => Ok(Value::none()),
    }
}

