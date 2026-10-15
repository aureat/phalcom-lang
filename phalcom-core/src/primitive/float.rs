//! Native primitives on `Float`.

use crate::error::{PhResult, RuntimeError};
use crate::value::{Value, normalize_bigint};
use crate::vm::VM;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{ToPrimitive, Zero};
use std::num::FpCategory;

// IEEE-754 adjacent representable values, compatible with the workspace's
// Rust 1.85 MSRV (f64::next_up/down require Rust 1.86).
fn next_up_f64(value: f64) -> f64 {
    if value.is_nan() || value == f64::INFINITY {
        return value;
    }
    if value == 0.0 {
        return f64::from_bits(1);
    }
    let bits = value.to_bits();
    f64::from_bits(if value.is_sign_positive() { bits + 1 } else { bits - 1 })
}

fn next_down_f64(value: f64) -> f64 {
    if value.is_nan() || value == f64::NEG_INFINITY {
        return value;
    }
    if value == 0.0 {
        return f64::from_bits((1u64 << 63) | 1);
    }
    let bits = value.to_bits();
    f64::from_bits(if value.is_sign_positive() { bits - 1 } else { bits + 1 })
}

fn expect_float_value(value: &Value) -> Result<f64, RuntimeError> {
    value.as_float().ok_or(RuntimeError::Type {
        expected: "Float",
        found: value.type_name(),
    })
}

/// Signature: `Float::nextUp` — the adjacent representable value toward +infinity.
#[phalcom_native_macros::primitive(Float, "nextUp")]
pub fn float_next_up(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::float(next_up_f64(expect_float_value(receiver)?)))
}

/// Signature: `Float::nextDown` — the adjacent representable value toward -infinity.
#[phalcom_native_macros::primitive(Float, "nextDown")]
pub fn float_next_down(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::float(next_down_f64(expect_float_value(receiver)?)))
}

/// Signature: `Float::ulp` — the spacing at the magnitude of the receiver.
#[phalcom_native_macros::primitive(Float, "ulp")]
pub fn float_ulp(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let value = expect_float_value(receiver)?;
    let spacing = if value.is_nan() || value.is_infinite() {
        value.abs()
    } else {
        let magnitude = value.abs();
        if magnitude == 0.0 {
            f64::from_bits(1)
        } else {
            let above = next_up_f64(magnitude);
            if above.is_infinite() {
                magnitude - next_down_f64(magnitude)
            } else {
                above - magnitude
            }
        }
    };
    Ok(Value::float(spacing))
}

/// Signature: `Float::totalCompare(_)` — IEEE total ordering, including NaNs and signed zero.
#[phalcom_native_macros::primitive(Float, "totalCompare(_)" , params = [Float], returns = Ordering, types = "(Float) -> Ordering", effects = pure)]
pub fn float_total_compare(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let left = expect_float_value(receiver)?;
    let right = expect_float_value(&args[0])?;
    let result = match left.total_cmp(&right) {
        std::cmp::Ordering::Less => "less",
        std::cmp::Ordering::Equal => "equal",
        std::cmp::Ordering::Greater => "greater",
    };
    super::number::ordering_value(vm, result)
}

#[phalcom_native_macros::primitive(Float, "isNormal")]
pub fn float_is_normal(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(expect_float_value(receiver)?.is_normal()))
}

#[phalcom_native_macros::primitive(Float, "isSubnormal")]
pub fn float_is_subnormal(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(expect_float_value(receiver)?.classify() == FpCategory::Subnormal))
}

#[phalcom_native_macros::primitive(Float, "isPositiveZero")]
pub fn float_is_positive_zero(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let value = expect_float_value(receiver)?;
    Ok(Value::bool(value == 0.0 && !value.is_sign_negative()))
}

#[phalcom_native_macros::primitive(Float, "isNegativeZero")]
pub fn float_is_negative_zero(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let value = expect_float_value(receiver)?;
    Ok(Value::bool(value == 0.0 && value.is_sign_negative()))
}

#[phalcom_native_macros::primitive(Float, "epsilon", params = [], returns = Float, types = "() -> Float", side = class, effects = pure)]
pub fn float_epsilon(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::float(f64::EPSILON))
}

#[phalcom_native_macros::primitive(Float, "minPositiveSubnormal", params = [], returns = Float, types = "() -> Float", side = class, effects = pure)]
pub fn float_min_positive_subnormal(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::float(f64::from_bits(1)))
}

#[phalcom_native_macros::primitive(Float, "minNormal", params = [], returns = Float, types = "() -> Float", side = class, effects = pure)]
pub fn float_min_normal(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::float(f64::MIN_POSITIVE))
}

#[phalcom_native_macros::primitive(Float, "maxFinite", params = [], returns = Float, types = "() -> Float", side = class, effects = pure)]
pub fn float_max_finite(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::float(f64::MAX))
}

/// Convert a finite f64 exactly to a rational (Numerator, Denominator)
pub fn float_to_rational(f: f64) -> (BigInt, BigInt) {
    let bits = f.to_bits();
    let sign = (bits >> 63) != 0;
    let mantissa = bits & 0x000F_FFFF_FFFF_FFFF;
    let exponent = ((bits >> 52) & 0x7FF) as i16;

    let (m, e) = if exponent == 0 {
        // Subnormal
        (mantissa, -1022 - 52)
    } else {
        // Normal
        (mantissa | 0x0010_0000_0000_0000, exponent - 1023 - 52)
    };

    let mut n = BigInt::from(m);
    if sign {
        n = -n;
    }

    if e >= 0 {
        let d = BigInt::from(1);
        (n << (e as usize), d)
    } else {
        let d = BigInt::from(1) << ((-e) as usize);
        (n, d)
    }
}

/// Helper to check if a Float receiver is finite
fn expect_finite_float(val: &Value, vm: &mut VM) -> PhResult<f64> {
    if let Some(f) = val.as_float() {
        if f.is_nan() || f.is_infinite() {
            Err(vm.raise_numeric_error(RuntimeError::NonFiniteNumber("non-finite float".to_string())))
        } else {
            Ok(f)
        }
    } else {
        Err(RuntimeError::Type {
            expected: "Float",
            found: val.type_name(),
        }
        .into())
    }
}

/// Signature: `Float::abs()`
#[phalcom_native_macros::primitive(Float, "abs")]
pub fn float_abs(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if let Some(f) = receiver.as_float() {
        if f.is_nan() { Ok(Value::float(f64::NAN)) } else { Ok(Value::float(f.abs())) }
    } else {
        Err(RuntimeError::Type {
            expected: "Float",
            found: receiver.type_name(),
        }
        .into())
    }
}

/// Signature: `Float::sign()`
#[phalcom_native_macros::primitive(Float, "sign")]
pub fn float_sign(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if let Some(f) = receiver.as_float() {
        if f.is_nan() || f.is_infinite() {
            Err(RuntimeError::NonFiniteNumber("non-finite sign".to_string()).into())
        } else if f == 0.0 {
            Ok(Value::int(0))
        } else if f.is_sign_negative() {
            Ok(Value::int(-1))
        } else {
            Ok(Value::int(1))
        }
    } else {
        Err(RuntimeError::Type {
            expected: "Float",
            found: receiver.type_name(),
        }
        .into())
    }
}

/// Signature: `Float::floor()`
#[phalcom_native_macros::primitive(Float, "floor")]
pub fn float_floor(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let f = expect_finite_float(receiver, vm)?;
    let (n, d) = float_to_rational(f);
    let q = n.div_floor(&d);
    let limit = vm.numeric_policy.max_integer_bits.unwrap_or(8_388_608);
    if q.bits() as usize > limit {
        return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("Float floor exceeds bit limit".to_string())));
    }
    Ok(normalize_bigint(q, &mut vm.heap))
}

/// Signature: `Float::ceil()`
#[phalcom_native_macros::primitive(Float, "ceil")]
pub fn float_ceil(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let f = expect_finite_float(receiver, vm)?;
    let (n, d) = float_to_rational(f);
    let (q, r) = n.div_mod_floor(&d);
    let res = if r.is_zero() { q } else { q + 1 };
    let limit = vm.numeric_policy.max_integer_bits.unwrap_or(8_388_608);
    if res.bits() as usize > limit {
        return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("Float ceil exceeds bit limit".to_string())));
    }
    Ok(normalize_bigint(res, &mut vm.heap))
}

/// Signature: `Float::truncated()`
#[phalcom_native_macros::primitive(Float, "truncated")]
pub fn float_truncated(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let f = expect_finite_float(receiver, vm)?;
    let (n, d) = float_to_rational(f);
    let q = n / d;
    let limit = vm.numeric_policy.max_integer_bits.unwrap_or(8_388_608);
    if q.bits() as usize > limit {
        return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("Float truncate exceeds bit limit".to_string())));
    }
    Ok(normalize_bigint(q, &mut vm.heap))
}

/// Signature: `Float::rounded()`
#[phalcom_native_macros::primitive(Float, "rounded")]
pub fn float_rounded(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let f = expect_finite_float(receiver, vm)?;
    let (n, d) = float_to_rational(f);
    let (q, r) = n.div_mod_floor(&d);
    let half = &d / 2;
    let res = if r < half {
        q
    } else if r > half {
        q + 1
    } else {
        // Halfway tie-to-even
        if &q % 2 == BigInt::zero() { q } else { q + 1 }
    };
    let limit = vm.numeric_policy.max_integer_bits.unwrap_or(8_388_608);
    if res.bits() as usize > limit {
        return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("Float round exceeds bit limit".to_string())));
    }
    Ok(normalize_bigint(res, &mut vm.heap))
}

/// Signature: `Float::toIntExact()`
#[phalcom_native_macros::primitive(Float, "toIntExact")]
pub fn float_to_int_exact(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let f = expect_finite_float(receiver, vm)?;
    let (n, d) = float_to_rational(f);
    let (q, r) = n.div_mod_floor(&d);
    if !r.is_zero() {
        return Err(vm.raise_numeric_error(RuntimeError::TypeConversion {
            expected: "Int",
            found: "Float (fractional)",
        }));
    }
    let limit = vm.numeric_policy.max_integer_bits.unwrap_or(8_388_608);
    if q.bits() as usize > limit {
        return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("toIntExact exceeds bit limit".to_string())));
    }
    Ok(normalize_bigint(q, &mut vm.heap))
}

/// Signature: `Float::isInteger`
#[phalcom_native_macros::primitive(Float, "isInteger")]
pub fn float_is_integer(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if let Some(f) = receiver.as_float() {
        if f.is_nan() || f.is_infinite() {
            Ok(Value::bool(false))
        } else {
            Ok(Value::bool(f.fract() == 0.0))
        }
    } else {
        Err(RuntimeError::Type {
            expected: "Float",
            found: receiver.type_name(),
        }
        .into())
    }
}

/// Signature: `Float::isNaN`
#[phalcom_native_macros::primitive(Float, "isNaN")]
pub fn float_is_nan(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if let Some(f) = receiver.as_float() {
        Ok(Value::bool(f.is_nan()))
    } else {
        Err(RuntimeError::Type {
            expected: "Float",
            found: receiver.type_name(),
        }
        .into())
    }
}

/// Signature: `Float::isFinite`
#[phalcom_native_macros::primitive(Float, "isFinite")]
pub fn float_is_finite(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if let Some(f) = receiver.as_float() {
        Ok(Value::bool(f.is_finite()))
    } else {
        Err(RuntimeError::Type {
            expected: "Float",
            found: receiver.type_name(),
        }
        .into())
    }
}

/// Signature: `Float::isInfinite`
#[phalcom_native_macros::primitive(Float, "isInfinite")]
pub fn float_is_infinite(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if let Some(f) = receiver.as_float() {
        Ok(Value::bool(f.is_infinite()))
    } else {
        Err(RuntimeError::Type {
            expected: "Float",
            found: receiver.type_name(),
        }
        .into())
    }
}

/// Grammar validation for INT-TEXT
pub fn scan_int_text(s: &str) -> Result<(), usize> {
    if s.is_empty() {
        return Err(0);
    }
    let bytes = s.as_bytes();
    let mut i = 0;

    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    if i >= bytes.len() {
        return Err(i);
    }

    if bytes[i] == b'0' {
        i += 1;
        while i < bytes.len() {
            if bytes[i] == b'0' {
                i += 1;
            } else if bytes[i] == b'_' {
                if i + 1 < bytes.len() && bytes[i + 1] == b'0' {
                    i += 2;
                } else {
                    return Err(i);
                }
            } else {
                break;
            }
        }
    } else if bytes[i] >= b'1' && bytes[i] <= b'9' {
        i += 1;
        while i < bytes.len() {
            if bytes[i] >= b'0' && bytes[i] <= b'9' {
                i += 1;
            } else if bytes[i] == b'_' {
                if i + 1 < bytes.len() && bytes[i + 1] >= b'0' && bytes[i + 1] <= b'9' {
                    i += 2;
                } else {
                    return Err(i);
                }
            } else {
                break;
            }
        }
    } else {
        return Err(i);
    }

    if i < bytes.len() {
        return Err(i);
    }
    Ok(())
}

/// Grammar validation for FLOAT-TEXT
pub fn scan_float_text(s: &str) -> Result<(), usize> {
    if s.is_empty() {
        return Err(0);
    }
    if s == "NaN" || s == "Infinity" || s == "-Infinity" {
        return Ok(());
    }

    let bytes = s.as_bytes();
    let mut i = 0;

    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }
    if i >= bytes.len() {
        return Err(i);
    }

    if bytes[i] >= b'0' && bytes[i] <= b'9' {
        i = parse_dec_digits(bytes, i)?;
        if i < bytes.len() && bytes[i] == b'.' {
            i += 1;
            if i < bytes.len() && bytes[i] >= b'0' && bytes[i] <= b'9' {
                i = parse_dec_digits(bytes, i)?;
            } else {
                return Err(i);
            }
        }
    } else if bytes[i] == b'.' {
        i += 1;
        i = parse_dec_digits(bytes, i)?;
    } else {
        return Err(i);
    }

    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
            i += 1;
        }
        i = parse_dec_digits(bytes, i)?;
    }

    if i < bytes.len() {
        return Err(i);
    }
    Ok(())
}

fn parse_dec_digits(bytes: &[u8], mut i: usize) -> Result<usize, usize> {
    if i >= bytes.len() || !(bytes[i] >= b'0' && bytes[i] <= b'9') {
        return Err(i);
    }
    i += 1;
    while i < bytes.len() {
        if bytes[i] >= b'0' && bytes[i] <= b'9' {
            i += 1;
        } else if bytes[i] == b'_' {
            if i + 1 < bytes.len() && bytes[i + 1] >= b'0' && bytes[i + 1] <= b'9' {
                i += 2;
            } else {
                return Err(i);
            }
        } else {
            break;
        }
    }
    Ok(i)
}

/// Signature: `Float.class::new(_)`
pub fn float_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let Some(arg) = args.first() else {
        return Ok(Value::float(0.0));
    };
    if let Some(n) = arg.as_float() {
        Ok(Value::float(n))
    } else if let Some(n) = arg.as_int() {
        let f = n as f64;
        if f.is_infinite() {
            return Err(vm.raise_numeric_error(RuntimeError::TypeConversion {
                expected: "Float",
                found: "Int (overflow)",
            }));
        }
        Ok(Value::float(f))
    } else if let Some(id) = arg.as_obj() {
        if let Some(b) = vm.heap.as_large_int(id) {
            if let Some(f) = b.to_f64() {
                if f.is_infinite() {
                    Err(vm.raise_numeric_error(RuntimeError::TypeConversion {
                        expected: "Float",
                        found: "Int (overflow)",
                    }))
                } else {
                    Ok(Value::float(f))
                }
            } else {
                Err(vm.raise_numeric_error(RuntimeError::TypeConversion {
                    expected: "Float",
                    found: "Int (overflow)",
                }))
            }
        } else if let Some(s) = vm.heap.as_string(id) {
            let text = s.value();
            if let Err(offset) = scan_float_text(&text) {
                return Err(RuntimeError::ArgumentError(format!("invalid float text at byte {}", offset)).into());
            }

            let cleaned: String = text.chars().filter(|&c| c != '_').collect();
            if cleaned == "NaN" {
                Ok(Value::float(f64::NAN))
            } else if cleaned == "Infinity" {
                Ok(Value::float(f64::INFINITY))
            } else if cleaned == "-Infinity" {
                Ok(Value::float(f64::NEG_INFINITY))
            } else if let Ok(f) = cleaned.parse::<f64>() {
                Ok(Value::float(f))
            } else {
                Err(RuntimeError::ArgumentError("invalid float text".to_string()).into())
            }
        } else {
            Err(RuntimeError::TypeConversion {
                expected: "Float",
                found: arg.type_name(),
            }
            .into())
        }
    } else {
        Err(RuntimeError::TypeConversion {
            expected: "Float",
            found: arg.type_name(),
        }
        .into())
    }
}

#[phalcom_native_macros::primitive(Float, "new()", side = class)]
pub fn float_class_new_default(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    float_class_new(vm, receiver, &[])
}

#[cfg(test)]
mod adjacent_float_tests {
    use super::{next_down_f64, next_up_f64};

    #[test]
    fn adjacent_values_preserve_zeros_subnormals_and_infinity_boundaries() {
        let smallest = f64::from_bits(1);
        let negative_smallest = f64::from_bits((1u64 << 63) | 1);
        assert_eq!(next_up_f64(0.0).to_bits(), smallest.to_bits());
        assert_eq!(next_up_f64(-0.0).to_bits(), smallest.to_bits());
        assert_eq!(next_down_f64(0.0).to_bits(), negative_smallest.to_bits());
        assert_eq!(next_down_f64(-0.0).to_bits(), negative_smallest.to_bits());
        assert_eq!(next_down_f64(smallest).to_bits(), 0.0f64.to_bits());
        assert_eq!(next_up_f64(negative_smallest).to_bits(), (-0.0f64).to_bits());
        assert_eq!(next_up_f64(f64::MAX), f64::INFINITY);
        assert_eq!(next_down_f64(f64::MIN), f64::NEG_INFINITY);
        assert_eq!(next_down_f64(f64::INFINITY), f64::MAX);
        assert_eq!(next_up_f64(f64::NEG_INFINITY), f64::MIN);
    }

    #[test]
    fn adjacent_values_preserve_nan_payloads_and_terminal_infinities() {
        let nan = f64::from_bits(0x7ff8_0000_0000_0021);
        assert_eq!(next_up_f64(nan).to_bits(), nan.to_bits());
        assert_eq!(next_down_f64(nan).to_bits(), nan.to_bits());
        assert_eq!(next_up_f64(f64::INFINITY), f64::INFINITY);
        assert_eq!(next_down_f64(f64::NEG_INFINITY), f64::NEG_INFINITY);
    }
}
