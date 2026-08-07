//! Native primitives on `Float`.

use crate::error::{PhResult, RuntimeError};
use crate::value::{Value, normalize_bigint};
use crate::vm::VM;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{Signed, ToPrimitive, Zero};

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
    match val {
        Value::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                Err(vm.raise_numeric_error(RuntimeError::NonFiniteNumber("non-finite float".to_string())))
            } else {
                Ok(*f)
            }
        }
        _ => Err(RuntimeError::Type {
            expected: "Float",
            found: val.type_name(),
        }
        .into()),
    }
}

/// Signature: `Float::abs()`
pub fn float_abs(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    match receiver {
        Value::Float(f) => {
            if f.is_nan() {
                Ok(Value::Float(f64::NAN))
            } else {
                Ok(Value::Float(f.abs()))
            }
        }
        _ => Err(RuntimeError::Type {
            expected: "Float",
            found: receiver.type_name(),
        }
        .into()),
    }
}

/// Signature: `Float::sign()`
pub fn float_sign(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    match receiver {
        Value::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                Err(RuntimeError::NonFiniteNumber("non-finite sign".to_string()).into())
            } else if *f == 0.0 {
                Ok(Value::Int(0))
            } else if f.is_sign_negative() {
                Ok(Value::Int(-1))
            } else {
                Ok(Value::Int(1))
            }
        }
        _ => Err(RuntimeError::Type {
            expected: "Float",
            found: receiver.type_name(),
        }
        .into()),
    }
}

/// Signature: `Float::floor()`
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
pub fn float_is_integer(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    match receiver {
        Value::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                Ok(Value::Bool(false))
            } else {
                Ok(Value::Bool(f.fract() == 0.0))
            }
        }
        _ => Err(RuntimeError::Type {
            expected: "Float",
            found: receiver.type_name(),
        }
        .into()),
    }
}

/// Signature: `Float::isNaN`
pub fn float_is_nan(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    match receiver {
        Value::Float(f) => Ok(Value::Bool(f.is_nan())),
        _ => Err(RuntimeError::Type {
            expected: "Float",
            found: receiver.type_name(),
        }
        .into()),
    }
}

/// Signature: `Float::isFinite`
pub fn float_is_finite(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    match receiver {
        Value::Float(f) => Ok(Value::Bool(f.is_finite())),
        _ => Err(RuntimeError::Type {
            expected: "Float",
            found: receiver.type_name(),
        }
        .into()),
    }
}

/// Signature: `Float::isInfinite`
pub fn float_is_infinite(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    match receiver {
        Value::Float(f) => Ok(Value::Bool(f.is_infinite())),
        _ => Err(RuntimeError::Type {
            expected: "Float",
            found: receiver.type_name(),
        }
        .into()),
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
        return Ok(Value::Float(0.0));
    };
    match arg {
        Value::Float(n) => Ok(Value::Float(*n)),
        Value::Int(n) => {
            // Check overflow boundary
            let f = *n as f64;
            if f.is_infinite() {
                return Err(vm.raise_numeric_error(RuntimeError::TypeConversion {
                    expected: "Float",
                    found: "Int (overflow)",
                }));
            }
            Ok(Value::Float(f))
        }
        Value::Obj(id) => {
            if let Some(b) = vm.heap.as_large_int(*id) {
                if let Some(f) = b.to_f64() {
                    if f.is_infinite() {
                        Err(vm.raise_numeric_error(RuntimeError::TypeConversion {
                            expected: "Float",
                            found: "Int (overflow)",
                        }))
                    } else {
                        Ok(Value::Float(f))
                    }
                } else {
                    Err(vm.raise_numeric_error(RuntimeError::TypeConversion {
                        expected: "Float",
                        found: "Int (overflow)",
                    }))
                }
            } else if let Some(s) = vm.heap.as_string(*id) {
                let text = s.value();
                if let Err(offset) = scan_float_text(&text) {
                    return Err(RuntimeError::ArgumentError(format!("invalid float text at byte {}", offset)).into());
                }

                // Once validated, clean underscores and parse
                let cleaned: String = text.chars().filter(|&c| c != '_').collect();
                if cleaned == "NaN" {
                    Ok(Value::Float(f64::NAN))
                } else if cleaned == "Infinity" {
                    Ok(Value::Float(f64::INFINITY))
                } else if cleaned == "-Infinity" {
                    Ok(Value::Float(f64::NEG_INFINITY))
                } else if let Ok(f) = cleaned.parse::<f64>() {
                    Ok(Value::Float(f))
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
        }
        _ => Err(RuntimeError::TypeConversion {
            expected: "Float",
            found: arg.type_name(),
        }
        .into()),
    }
}
