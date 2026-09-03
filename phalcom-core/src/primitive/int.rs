//! Native primitives on `Int`.

use crate::error::{PhResult, RuntimeError};
use crate::value::{Value, normalize_bigint};
use crate::vm::VM;
use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive, Zero};

pub(crate) fn expect_int_big(val: &Value, vm: &mut VM) -> PhResult<BigInt> {
    if let Some(n) = val.as_int() {
        Ok(BigInt::from(n))
    } else if let Some(id) = val.as_obj() {
        if let Some(b) = vm.heap.as_large_int(id) {
            Ok(b.clone())
        } else {
            Err(RuntimeError::Type {
                expected: "Int",
                found: val.type_name(),
            }
            .into())
        }
    } else {
        Err(RuntimeError::Type {
            expected: "Int",
            found: val.type_name(),
        }
        .into())
    }
}

fn integer_bits(n: &BigInt) -> usize {
    if n.is_zero() { 0 } else { n.bits() as usize }
}

fn try_int_big(val: &Value, vm: &VM) -> Option<BigInt> {
    val.as_int()
        .map(BigInt::from)
        .or_else(|| val.as_obj().and_then(|id| vm.heap.as_large_int(id).cloned()))
}

/// Signature: `Int::&(_)` — bitwise AND.
#[phalcom_native_macros::primitive(
    Int,
    "&(_)",
    params = [Int],
    returns = Int,
    types = "(Int) -> Int",
    effects = pure
)]
pub fn int_and(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let a = expect_int_big(receiver, vm)?;
    let Some(b) = try_int_big(&args[0], vm) else {
        return Ok(vm.require_semantic_roots()?.unsupported);
    };
    let res = a & b;
    Ok(normalize_bigint(res, &mut vm.heap))
}

/// Signature: `Int::|(_)` — bitwise OR.
#[phalcom_native_macros::primitive(
    Int,
    "|(_)",
    params = [Int],
    returns = Int,
    types = "(Int) -> Int",
    effects = pure
)]
pub fn int_or(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let a = expect_int_big(receiver, vm)?;
    let Some(b) = try_int_big(&args[0], vm) else {
        return Ok(vm.require_semantic_roots()?.unsupported);
    };
    let res = a | b;
    Ok(normalize_bigint(res, &mut vm.heap))
}

/// Signature: `Int::^(_)` — bitwise XOR.
#[phalcom_native_macros::primitive(
    Int,
    "^(_)",
    params = [Int],
    returns = Int,
    types = "(Int) -> Int",
    effects = pure
)]
pub fn int_xor(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let a = expect_int_big(receiver, vm)?;
    let Some(b) = try_int_big(&args[0], vm) else {
        return Ok(vm.require_semantic_roots()?.unsupported);
    };
    let res = a ^ b;
    Ok(normalize_bigint(res, &mut vm.heap))
}

/// Signature: `Int::~` — bitwise NOT.
#[phalcom_native_macros::primitive(
    Int,
    "~",
    params = [],
    returns = Int,
    types = "() -> Int",
    effects = pure
)]
pub fn int_not(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let a = expect_int_big(receiver, vm)?;
    let res = !a;
    Ok(normalize_bigint(res, &mut vm.heap))
}

/// Signature: `Int::<<(_)` — bitwise left shift.
#[phalcom_native_macros::primitive(
    Int,
    "<<(_)",
    params = [Int],
    returns = Int,
    types = "(Int) -> Int",
    effects = pure
)]
pub fn int_shl(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let a = expect_int_big(receiver, vm)?;
    let b_val = &args[0];
    let Some(b) = try_int_big(b_val, vm) else {
        return Ok(vm.require_semantic_roots()?.unsupported);
    };
    if b.is_negative() {
        return Err(vm.raise_numeric_error(RuntimeError::InvalidShift("shift count must be non-negative".to_string())));
    }
    let limit = vm.numeric_policy.max_integer_bits.unwrap_or(8_388_608);
    let b_usize = match b.to_usize() {
        Some(u) if u <= limit => u,
        _ => return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("Left shift exceeds configured bit limit".to_string()))),
    };
    if integer_bits(&a) + b_usize > limit {
        return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("Left shift exceeds configured bit limit".to_string())));
    }
    let res = a << b_usize;
    Ok(normalize_bigint(res, &mut vm.heap))
}

/// Signature: `Int::>>(_)` — bitwise right shift.
#[phalcom_native_macros::primitive(
    Int,
    ">>(_)",
    params = [Int],
    returns = Int,
    types = "(Int) -> Int",
    effects = pure
)]
pub fn int_shr(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let a = expect_int_big(receiver, vm)?;
    let b_val = &args[0];
    let Some(b) = try_int_big(b_val, vm) else {
        return Ok(vm.require_semantic_roots()?.unsupported);
    };
    if b.is_negative() {
        return Err(vm.raise_numeric_error(RuntimeError::InvalidShift("shift count must be non-negative".to_string())));
    }
    let b_usize = match b.to_usize() {
        Some(u) => u,
        None => {
            // Shift is larger than address space; since it's nonnegative:
            // if positive, results in 0; if negative, results in -1.
            let res = if a.is_negative() { -1 } else { 0 };
            return Ok(Value::int(res));
        }
    };
    let res = a >> b_usize;
    Ok(normalize_bigint(res, &mut vm.heap))
}

/// Signature: `Int::bitAt(_)` — returns the bit at `index`.
#[phalcom_native_macros::primitive(
    Int,
    "bitAt(_)",
    params = [Int],
    returns = Int,
    types = "(Int) -> Int",
    effects = pure
)]
pub fn int_bit_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let a = expect_int_big(receiver, vm)?;
    let idx_val = &args[0];
    let idx = expect_int_big(idx_val, vm)?;
    if idx.is_negative() {
        return Err(vm.raise_numeric_error(RuntimeError::InvalidBitIndex("bit index must be non-negative".to_string())));
    }
    let idx_usize = match idx.to_usize() {
        Some(u) => u,
        None => {
            // Index is larger than address space; since it's nonnegative:
            // if positive, bit is 0; if negative, bit is 1.
            let res = if a.is_negative() { 1 } else { 0 };
            return Ok(Value::int(res));
        }
    };
    let res = if a.bit(idx_usize as u64) { 1 } else { 0 };
    Ok(Value::int(res))
}

/// Signature: `Int::bitCount` — returns the number of 1 bits in `|self|`.
#[phalcom_native_macros::primitive(
    Int,
    "bitCount",
    params = [],
    returns = Int,
    types = "() -> Int",
    effects = pure
)]
pub fn int_bit_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let a = expect_int_big(receiver, vm)?;
    let abs_a = a.abs();
    let digits = abs_a.magnitude().to_u32_digits();
    let count: u64 = digits.iter().map(|&d| d.count_ones() as u64).sum();
    Ok(Value::int(count as i64))
}

/// Signature: `Int::bitLength` — returns the bit length of `|self|`.
#[phalcom_native_macros::primitive(
    Int,
    "bitLength",
    params = [],
    returns = Int,
    types = "() -> Int",
    effects = pure
)]
pub fn int_bit_length(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let a = expect_int_big(receiver, vm)?;
    let abs_a = a.abs();
    let len = abs_a.magnitude().bits();
    Ok(Value::int(len as i64))
}

/// Signature: `Int::trailingZeros` — returns trailing zeros of `|self|`.
#[phalcom_native_macros::primitive(
    Int,
    "trailingZeros",
    params = [],
    returns = Int,
    types = "() -> Int",
    effects = pure
)]
pub fn int_trailing_zeros(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let a = expect_int_big(receiver, vm)?;
    if a.is_zero() {
        return Err(RuntimeError::UndefinedNumericOperation("trailingZeros of zero is undefined".to_string()).into());
    }
    let abs_a = a.abs();
    let digits = abs_a.magnitude().to_u32_digits();
    let mut count = 0;
    for (i, &d) in digits.iter().enumerate() {
        if d != 0 {
            count += (i as i64) * 32 + (d.trailing_zeros() as i64);
            break;
        }
    }
    Ok(Value::int(count))
}

/// Signature: `Int.class::new(_)` — coerces/constructs an `Int`.
#[phalcom_native_macros::primitive(
    Int,
    "new(_)",
    params = [Object],
    returns = Int,
    types = "(Object) -> Int",
    side = class
)]
pub fn int_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let Some(arg) = args.first() else {
        return Ok(Value::int(0));
    };
    if let Some(n) = arg.as_int() {
        Ok(Value::int(n))
    } else if let Some(id) = arg.as_obj() {
        if vm.heap.as_large_int(id).is_some() {
            Ok(*arg)
        } else if vm.heap.as_string(id).is_some() {
            let text = vm.heap.string(id).value();
            if let Err(offset) = crate::primitive::float::scan_int_text(&text) {
                return Err(RuntimeError::NumericText {
                    target_type: "Int",
                    byte_offset: offset,
                }
                .into());
            }
            let cleaned: String = text.chars().filter(|&c| c != '_').collect();
            if let Ok(i) = cleaned.parse::<i64>() {
                Ok(Value::int(i))
            } else if let Ok(b) = cleaned.parse::<BigInt>() {
                Ok(normalize_bigint(b, &mut vm.heap))
            } else {
                Err(RuntimeError::NumericConversion {
                    expected: "Int",
                    found: "String",
                    operation: "Int.new",
                }
                .into())
            }
        } else {
            Err(RuntimeError::NumericConversion {
                expected: "Int",
                found: arg.type_name(),
                operation: "Int.new",
            }
            .into())
        }
    } else {
        Err(RuntimeError::NumericConversion {
            expected: "Int",
            found: arg.type_name(),
            operation: "Int.new",
        }
        .into())
    }
}

#[phalcom_native_macros::primitive(Int, "new()", side = class)]
pub fn int_class_new_default(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    int_class_new(vm, receiver, &[])
}
