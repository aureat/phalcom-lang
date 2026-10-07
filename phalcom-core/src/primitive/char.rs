//! Native primitives for immediate Unicode-scalar `Char` values.

use crate::error::{PhResult, RuntimeError};
use crate::primitive::option::wrap_some;
use crate::value::Value;
use crate::vm::VM;
use num_traits::ToPrimitive;

fn expect_char(receiver: &Value) -> PhResult<char> {
    receiver.as_char().ok_or_else(|| {
        RuntimeError::Type {
            expected: "Char",
            found: receiver.type_name(),
        }
        .into()
    })
}

/// Returns the Unicode scalar value as an `Int`.
#[phalcom_native_macros::primitive(Char, "codePoint", params = [], returns = Int, types = "() -> Int", effects = pure)]
pub fn char_code_point(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::int(expect_char(receiver)? as u32 as i64))
}

/// Returns the scalar's UTF-8 width in bytes.
#[phalcom_native_macros::primitive(Char, "utf8Length", params = [], returns = Int, types = "() -> Int", effects = pure)]
pub fn char_utf8_length(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::int(expect_char(receiver)?.len_utf8() as i64))
}

/// Returns a one-scalar UTF-8 `String`.
#[phalcom_native_macros::primitive(Char, "toString", params = [], returns = String, types = "() -> String", effects = pure)]
pub fn char_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let mut encoded = [0u8; 4];
    let text = expect_char(receiver)?.encode_utf8(&mut encoded);
    Ok(vm.alloc_string_value(text))
}

/// Returns whether this scalar is ASCII.
#[phalcom_native_macros::primitive(Char, "isAscii", params = [], returns = Bool, types = "() -> Bool", effects = pure)]
pub fn char_is_ascii(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(expect_char(receiver)?.is_ascii()))
}

/// Returns whether this scalar is an ASCII decimal digit.
#[phalcom_native_macros::primitive(Char, "isAsciiDigit", params = [], returns = Bool, types = "() -> Bool", effects = pure)]
pub fn char_is_ascii_digit(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(expect_char(receiver)?.is_ascii_digit()))
}

/// Returns whether this scalar is an ASCII alphabetic letter.
#[phalcom_native_macros::primitive(Char, "isAsciiAlphabetic", params = [], returns = Bool, types = "() -> Bool", effects = pure)]
pub fn char_is_ascii_alphabetic(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(expect_char(receiver)?.is_ascii_alphabetic()))
}

/// Returns whether this scalar is an ASCII lowercase letter.
#[phalcom_native_macros::primitive(Char, "isAsciiLowercase", params = [], returns = Bool, types = "() -> Bool", effects = pure)]
pub fn char_is_ascii_lowercase(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(expect_char(receiver)?.is_ascii_lowercase()))
}

/// Returns whether this scalar is an ASCII uppercase letter.
#[phalcom_native_macros::primitive(Char, "isAsciiUppercase", params = [], returns = Bool, types = "() -> Bool", effects = pure)]
pub fn char_is_ascii_uppercase(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(expect_char(receiver)?.is_ascii_uppercase()))
}

/// Returns whether this scalar is an ASCII letter or decimal digit.
#[phalcom_native_macros::primitive(Char, "isAsciiAlphanumeric", params = [], returns = Bool, types = "() -> Bool", effects = pure)]
pub fn char_is_ascii_alphanumeric(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(expect_char(receiver)?.is_ascii_alphanumeric()))
}

/// Returns whether this scalar is an ASCII hexadecimal digit.
#[phalcom_native_macros::primitive(Char, "isAsciiHexDigit", params = [], returns = Bool, types = "() -> Bool", effects = pure)]
pub fn char_is_ascii_hex_digit(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(expect_char(receiver)?.is_ascii_hexdigit()))
}

/// Returns whether this scalar has the Unicode `White_Space` property.
#[phalcom_native_macros::primitive(Char, "isWhitespace", params = [], returns = Bool, types = "() -> Bool", effects = pure)]
pub fn char_is_whitespace(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(expect_char(receiver)?.is_whitespace()))
}

/// Returns whether this scalar has Unicode General_Category `Cc`.
#[phalcom_native_macros::primitive(Char, "isControl", params = [], returns = Bool, types = "() -> Bool", effects = pure)]
pub fn char_is_control(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(expect_char(receiver)?.is_control()))
}

/// Returns a content hash consistent with scalar equality.
#[phalcom_native_macros::primitive(Char, "hash", params = [], returns = Int, types = "() -> Int", effects = pure)]
pub fn char_hash(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::int(expect_char(receiver)? as u32 as i64))
}

/// Creates a Char for a valid Unicode scalar, or returns None otherwise.
#[phalcom_native_macros::primitive(
    Char,
    "fromCodePoint(_)",
    params = [Int],
    returns = Option,
    types = "(Int) -> Option<Char>",
    side = class,
    effects = pure
)]
pub fn char_from_code_point(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let integer = crate::primitive::int::expect_int_big(&args[0], vm)?;
    match integer.to_u32().and_then(char::from_u32) {
        Some(value) => Ok(wrap_some(vm, Value::char(value))?),
        None => Ok(vm.none_value()),
    }
}
