//! Native primitives on `Number`.

use crate::error::{PhResult, RuntimeError};
use crate::primitive::float::float_to_rational;
use crate::value::{Value, normalize_bigint};
use crate::vm::VM;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::cast::ToPrimitive;
use num_traits::identities::Zero;
use num_traits::sign::Signed;
use phalcom_ast::selector::Selector;
use phalcom_semantic::core_surface::CoreDeclarationIds;
use phalcom_semantic::identity::VariantId;

pub const NUM_0: Value = Value::int(0);
pub const NUM_1: Value = Value::int(1);

fn integer_bits(n: &BigInt) -> usize {
    if n.is_zero() { 0 } else { n.bits() as usize }
}

fn extract_int(val: &Value, vm: &VM) -> Option<BigInt> {
    if let Some(n) = val.as_int() {
        Some(BigInt::from(n))
    } else if let Some(id) = val.as_obj() {
        vm.heap.as_large_int(id).cloned()
    } else {
        None
    }
}

enum EitherIntOrFloat {
    Int(BigInt),
    Float(f64),
}

fn expect_number_big_or_float(val: &Value, vm: &VM) -> PhResult<EitherIntOrFloat> {
    if let Some(big) = extract_int(val, vm) {
        Ok(EitherIntOrFloat::Int(big))
    } else if let Some(f) = val.as_float() {
        Ok(EitherIntOrFloat::Float(f))
    } else {
        Err(RuntimeError::Type {
            expected: "number",
            found: val.type_name(),
        }
        .into())
    }
}

enum PromotedPair {
    Int(i64, i64),
    Big(BigInt, BigInt),
    Float(f64, f64),
}

fn compare_integer_to_float(integer: &BigInt, float: f64) -> Option<std::cmp::Ordering> {
    if float.is_nan() {
        return None;
    }
    if float.is_infinite() {
        return Some(if float.is_sign_positive() {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        });
    }

    let (numerator, denominator) = float_to_rational(float);
    Some((integer * denominator).cmp(&numerator))
}

/// Compare numeric values without converting an integer through `f64`.
///
/// Mixed comparisons treat every finite float as its exact dyadic value. The
/// `None` result represents the unordered result required for NaN operands.
fn exact_compare(a: &Value, b: &Value, vm: &VM) -> PhResult<Option<std::cmp::Ordering>> {
    let a = expect_number_big_or_float(a, vm)?;
    let b = expect_number_big_or_float(b, vm)?;
    Ok(match (a, b) {
        (EitherIntOrFloat::Int(a), EitherIntOrFloat::Int(b)) => Some(a.cmp(&b)),
        (EitherIntOrFloat::Float(a), EitherIntOrFloat::Float(b)) => a.partial_cmp(&b),
        (EitherIntOrFloat::Int(a), EitherIntOrFloat::Float(b)) => compare_integer_to_float(&a, b),
        (EitherIntOrFloat::Float(a), EitherIntOrFloat::Int(b)) => compare_integer_to_float(&b, a).map(std::cmp::Ordering::reverse),
    })
}

fn is_number(value: &Value, vm: &VM) -> bool {
    extract_int(value, vm).is_some() || value.as_float().is_some()
}

fn promote_pair(a: &Value, b: &Value, vm: &VM) -> PhResult<PromotedPair> {
    let a_parsed = expect_number_big_or_float(a, vm)?;
    let b_parsed = expect_number_big_or_float(b, vm)?;
    match (a_parsed, b_parsed) {
        (EitherIntOrFloat::Float(af), EitherIntOrFloat::Float(bf)) => Ok(PromotedPair::Float(af, bf)),
        (EitherIntOrFloat::Float(af), EitherIntOrFloat::Int(bi)) => Ok(PromotedPair::Float(af, bi.to_f64().unwrap_or(f64::NAN))),
        (EitherIntOrFloat::Int(ai), EitherIntOrFloat::Float(bf)) => Ok(PromotedPair::Float(ai.to_f64().unwrap_or(f64::NAN), bf)),
        (EitherIntOrFloat::Int(ai), EitherIntOrFloat::Int(bi)) => {
            if let (Some(a_i), Some(b_i)) = (a.as_int(), b.as_int()) {
                Ok(PromotedPair::Int(a_i, b_i))
            } else {
                Ok(PromotedPair::Big(ai, bi))
            }
        }
    }
}

fn cooperative_pair(a: &Value, b: &Value, vm: &VM) -> PhResult<Option<PromotedPair>> {
    if (extract_int(a, vm).is_none() && a.as_float().is_none()) || (extract_int(b, vm).is_none() && b.as_float().is_none()) {
        return Ok(None);
    }
    promote_pair(a, b, vm).map(Some)
}

fn check_limit_bigint(n: &BigInt, vm: &mut VM) -> PhResult<()> {
    let limit = vm.numeric_policy.max_integer_bits.unwrap_or(8_388_608);
    if integer_bits(n) > limit {
        return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("Integer bit length exceeds configured limit".to_string())));
    }
    Ok(())
}

fn checked_int_to_float(value: &BigInt, operation: &'static str, vm: &mut VM) -> PhResult<f64> {
    let Some(float) = value.to_f64() else {
        return Err(vm.raise_numeric_error(RuntimeError::NumericOverflow {
            operation,
            target_type: "Float",
        }));
    };
    if !float.is_finite() {
        return Err(vm.raise_numeric_error(RuntimeError::NumericOverflow {
            operation,
            target_type: "Float",
        }));
    }
    Ok(float)
}

fn floor_div_rational(an: BigInt, ad_power: usize, bn: BigInt, bd_power: usize) -> BigInt {
    (an << bd_power).div_floor(&(bn << ad_power))
}

fn exact_integer_division(a: BigInt, b: BigInt, vm: &mut VM) -> PhResult<(Value, Value)> {
    if b.is_zero() {
        return Err(vm.raise_numeric_error(RuntimeError::DivideByZero));
    }

    // Operate on BigInt before division so i64::MIN / -1 cannot reach a
    // fixed-width host operation that would panic or wrap.
    let (quotient, remainder) = a.div_mod_floor(&b);
    check_limit_bigint(&quotient, vm)?;
    check_limit_bigint(&remainder, vm)?;
    Ok((normalize_bigint(quotient, &mut vm.heap), normalize_bigint(remainder, &mut vm.heap)))
}

fn dyadic_to_float(numerator: &BigInt, denominator_power: usize) -> f64 {
    if numerator.is_zero() {
        return 0.0;
    }

    let negative = numerator.is_negative();
    let magnitude = numerator.abs();
    let exponent = magnitude.bits() as i32 - 1 - denominator_power as i32;
    let magnitude_bits = if exponent >= -1022 {
        let scale = denominator_power as i32 + exponent - 52;
        let (mut significand, remainder, halfway) = if scale > 0 {
            let scale = scale as usize;
            let quotient = &magnitude >> scale;
            let remainder = &magnitude - (&quotient << scale);
            (quotient, remainder, BigInt::from(1u8) << (scale - 1))
        } else {
            (&magnitude << (-scale as usize), BigInt::zero(), BigInt::zero())
        };
        if !halfway.is_zero() && (remainder > halfway || (remainder == halfway && significand.bit(0))) {
            significand += 1;
        }
        let mut exponent = exponent;
        if significand.bits() > 53 {
            significand >>= 1;
            exponent += 1;
        }
        let encoded = significand.to_u64().expect("normal dyadic significand fits in u64");
        ((exponent + 1023) as u64) << 52 | (encoded - (1u64 << 52))
    } else {
        let scale = denominator_power as i32 - 1074;
        let (mut significand, remainder, halfway) = if scale > 0 {
            let scale = scale as usize;
            let quotient = &magnitude >> scale;
            let remainder = &magnitude - (&quotient << scale);
            (quotient, remainder, BigInt::from(1u8) << (scale - 1))
        } else {
            (&magnitude << (-scale as usize), BigInt::zero(), BigInt::zero())
        };
        if !halfway.is_zero() && (remainder > halfway || (remainder == halfway && significand.bit(0))) {
            significand += 1;
        }
        significand.to_u64().expect("subnormal dyadic significand fits in u64")
    };
    let bits = if negative { magnitude_bits | (1u64 << 63) } else { magnitude_bits };
    f64::from_bits(bits)
}

fn pow_bigint(base: &BigInt, exp: &BigInt, limit: usize, vm: &mut VM) -> PhResult<BigInt> {
    let mut res = BigInt::from(1);
    let mut temp_base = base.clone();
    let mut temp_exp = exp.clone();
    while !temp_exp.is_zero() {
        if temp_exp.bit(0) {
            res = &res * &temp_base;
            if integer_bits(&res) > limit {
                return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("Exponentiation exceeds bit limit".to_string())));
            }
        }
        temp_base = &temp_base * &temp_base;
        if !temp_exp.is_zero() && integer_bits(&temp_base) > limit {
            return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("Exponentiation exceeds bit limit".to_string())));
        }
        temp_exp >>= 1;
    }
    Ok(res)
}

/// Signature: `Number.class::new(_)` — raises `#abstractClass`.
#[phalcom_native_macros::primitive(
    Number,
    "new(_)",
    params = [Object],
    returns = Never,
    types = "(Object) -> Never",
    side = class,
    flow = never
)]
pub fn number_class_new(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let error_cls = vm.universe.classes.error_class;
    let field_count = vm.heap.class(error_cls).field_count;
    let mut inst = crate::heap::InstanceObject::new(error_cls, field_count);
    let msg = "cannot instantiate abstract class Number".to_string();
    inst.slots[0] = vm.alloc_string_value(msg.clone());
    let kind_sym = vm.get_or_intern("abstractClass");
    inst.slots[1] = Value::symbol(kind_sym);
    let err_obj = vm.heap.alloc(crate::heap::Object::Instance(inst));
    Err(RuntimeError::Raise {
        error: Value::obj(err_obj),
        rendered: msg,
        traceback: None,
        help: None,
    }
    .into())
}

#[phalcom_native_macros::primitive(Number, "new()", side = class)]
pub fn number_class_new_default(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    number_class_new(vm, receiver, &[])
}

/// Signature: `Float.class::new(_)` — coerces/constructs a `Float`.
#[phalcom_native_macros::primitive(
    Float,
    "new(_)",
    params = [Object],
    returns = Float,
    types = "(Object) -> Float",
    side = class
)]
pub fn float_class_new(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let Some(arg) = args.first() else {
        return Ok(Value::float(0.0));
    };
    if let Some(n) = arg.as_float() {
        Ok(Value::float(n))
    } else if let Some(n) = arg.as_int() {
        Ok(Value::float(n as f64))
    } else if let Some(id) = arg.as_obj() {
        if let Some(b) = vm.heap.as_large_int(id) {
            Ok(Value::float(b.to_f64().unwrap_or(f64::NAN)))
        } else if let Some(s) = vm.heap.as_string(id) {
            let text = s.value();
            if let Ok(f) = text.parse::<f64>() {
                Ok(Value::float(f))
            } else {
                Err(RuntimeError::TypeConversion {
                    expected: "Float",
                    found: "String",
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
    } else {
        Err(RuntimeError::TypeConversion {
            expected: "Float",
            found: arg.type_name(),
        }
        .into())
    }
}

/// Signature: `Number::hash` — digest of the mathematical value.
#[phalcom_native_macros::primitive(
    Number,
    "hash",
    params = [],
    returns = Int,
    types = "() -> Int",
    effects = pure
)]
pub fn number_hash(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if let Some(n) = receiver.as_int() {
        Ok(crate::primitive::hash_code(n as u64))
    } else if let Some(val) = receiver.as_float() {
        let bits = if val == 0.0 {
            0
        } else if val.is_finite() && val.fract() == 0.0 && val.abs() < 9_007_199_254_740_992.0 {
            (val as i64) as u64
        } else {
            val.to_bits()
        };
        Ok(crate::primitive::hash_code(bits))
    } else if let Some(id) = receiver.as_obj() {
        if let Some(b) = vm.heap.as_large_int(id) {
            let mut state = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(b, &mut state);
            use std::hash::Hasher;
            Ok(crate::primitive::hash_code(state.finish()))
        } else {
            Err(RuntimeError::Type {
                expected: "Number",
                found: receiver.type_name(),
            }
            .into())
        }
    } else {
        Err(RuntimeError::Type {
            expected: "Number",
            found: receiver.type_name(),
        }
        .into())
    }
}

/// Signature: `Number::toString`
#[phalcom_native_macros::primitive(
    Number,
    "toString",
    params = [],
    returns = String,
    types = "() -> String",
    effects = pure
)]
pub fn number_to_string(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let text = receiver.to_string(vm);
    Ok(vm.alloc_string_value(text))
}

/// Signature: `Number::+(_)`
#[phalcom_native_macros::primitive(
    Number,
    "+(_)",
    params = [Number],
    returns = Self,
    types = "(Number) -> Self",
    flow = receiver,
    effects = pure
)]
pub fn number_add(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let Some(pair) = cooperative_pair(receiver, &args[0], vm)? else {
        return Ok(vm.require_semantic_roots()?.unsupported);
    };
    match pair {
        PromotedPair::Int(a, b) => {
            if let Some(res) = a.checked_add(b) {
                Ok(Value::int(res))
            } else {
                let res = BigInt::from(a) + BigInt::from(b);
                check_limit_bigint(&res, vm)?;
                Ok(normalize_bigint(res, &mut vm.heap))
            }
        }
        PromotedPair::Big(a, b) => {
            let res = a + b;
            check_limit_bigint(&res, vm)?;
            Ok(normalize_bigint(res, &mut vm.heap))
        }
        PromotedPair::Float(a, b) => Ok(Value::float(a + b)),
    }
}

/// Signature: `Number::-(_)`
#[phalcom_native_macros::primitive(
    Number,
    "-(_)",
    params = [Number],
    returns = Self,
    types = "(Number) -> Self",
    flow = receiver,
    effects = pure
)]
pub fn number_sub(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let Some(pair) = cooperative_pair(receiver, &args[0], vm)? else {
        return Ok(vm.require_semantic_roots()?.unsupported);
    };
    match pair {
        PromotedPair::Int(a, b) => {
            if let Some(res) = a.checked_sub(b) {
                Ok(Value::int(res))
            } else {
                let res = BigInt::from(a) - BigInt::from(b);
                check_limit_bigint(&res, vm)?;
                Ok(normalize_bigint(res, &mut vm.heap))
            }
        }
        PromotedPair::Big(a, b) => {
            let res = a - b;
            check_limit_bigint(&res, vm)?;
            Ok(normalize_bigint(res, &mut vm.heap))
        }
        PromotedPair::Float(a, b) => Ok(Value::float(a - b)),
    }
}

/// Signature: `Number::*(_)`
#[phalcom_native_macros::primitive(
    Number,
    "*(_)",
    params = [Number],
    returns = Self,
    types = "(Number) -> Self",
    flow = receiver,
    effects = pure
)]
pub fn number_mul(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let Some(pair) = cooperative_pair(receiver, &args[0], vm)? else {
        return Ok(vm.require_semantic_roots()?.unsupported);
    };
    match pair {
        PromotedPair::Int(a, b) => {
            if let Some(res) = a.checked_mul(b) {
                Ok(Value::int(res))
            } else {
                let res = BigInt::from(a) * BigInt::from(b);
                check_limit_bigint(&res, vm)?;
                Ok(normalize_bigint(res, &mut vm.heap))
            }
        }
        PromotedPair::Big(a, b) => {
            let res = a * b;
            check_limit_bigint(&res, vm)?;
            Ok(normalize_bigint(res, &mut vm.heap))
        }
        PromotedPair::Float(a, b) => Ok(Value::float(a * b)),
    }
}

/// Signature: `Number::/(_)` — true division.
#[phalcom_native_macros::primitive(
    Number,
    "/(_)",
    params = [Number],
    returns = Float,
    types = "(Number) -> Float",
    effects = pure
)]
pub fn number_div(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let Some(pair) = cooperative_pair(receiver, &args[0], vm)? else {
        return Ok(vm.require_semantic_roots()?.unsupported);
    };
    let (a, b) = match pair {
        PromotedPair::Int(a, b) => (a as f64, b as f64),
        PromotedPair::Big(a, b) => (a.to_f64().unwrap_or(f64::NAN), b.to_f64().unwrap_or(f64::NAN)),
        PromotedPair::Float(a, b) => (a, b),
    };
    Ok(Value::float(a / b))
}

/// Signature: `Number::~/(_)` — floor division.
#[phalcom_native_macros::primitive(
    Number,
    "~/(_)",
    params = [Number],
    returns = Self,
    types = "(Number) -> Self",
    flow = receiver,
    effects = pure
)]
pub fn number_floor_div(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let left = expect_number_big_or_float(receiver, vm)?;
    let right = expect_number_big_or_float(&args[0], vm)?;
    match (left, right) {
        (EitherIntOrFloat::Int(a), EitherIntOrFloat::Int(b)) => {
            let (quotient, _) = exact_integer_division(a, b, vm)?;
            Ok(quotient)
        }
        (left, right) => {
            let (an, ad_power) = match left {
                EitherIntOrFloat::Int(value) => (value, 0),
                EitherIntOrFloat::Float(value) if value.is_finite() => {
                    let (numerator, denominator) = float_to_rational(value);
                    (numerator, denominator.bits() as usize - 1)
                }
                EitherIntOrFloat::Float(_) => {
                    return Err(vm.raise_numeric_error(RuntimeError::NonFiniteNumber("non-finite operand".to_string())));
                }
            };
            let (bn, bd_power) = match right {
                EitherIntOrFloat::Int(value) => (value, 0),
                EitherIntOrFloat::Float(value) if value.is_finite() => {
                    let (numerator, denominator) = float_to_rational(value);
                    (numerator, denominator.bits() as usize - 1)
                }
                EitherIntOrFloat::Float(_) => {
                    return Err(vm.raise_numeric_error(RuntimeError::NonFiniteNumber("non-finite operand".to_string())));
                }
            };
            if bn.is_zero() {
                return Err(vm.raise_numeric_error(RuntimeError::DivideByZero));
            }
            let result = floor_div_rational(an, ad_power, bn, bd_power);
            check_limit_bigint(&result, vm)?;
            Ok(normalize_bigint(result, &mut vm.heap))
        }
    }
}

/// Signature: `Number::%(_)`
#[phalcom_native_macros::primitive(
    Number,
    "%(_)",
    params = [Number],
    returns = Self,
    types = "(Number) -> Self",
    flow = receiver,
    effects = pure
)]
pub fn number_mod(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let left = expect_number_big_or_float(receiver, vm)?;
    let right = expect_number_big_or_float(&args[0], vm)?;
    match (left, right) {
        (EitherIntOrFloat::Int(a), EitherIntOrFloat::Int(b)) => {
            let (_, remainder) = exact_integer_division(a, b, vm)?;
            Ok(remainder)
        }
        (left, right) => {
            let a = match left {
                EitherIntOrFloat::Int(value) => checked_int_to_float(&value, "remainder", vm)?,
                EitherIntOrFloat::Float(value) => value,
            };
            let b = match right {
                EitherIntOrFloat::Int(value) => checked_int_to_float(&value, "remainder", vm)?,
                EitherIntOrFloat::Float(value) => value,
            };
            if !a.is_finite() || !b.is_finite() || b == 0.0 {
                return Ok(Value::float(f64::NAN));
            }
            if a == 0.0 {
                return Ok(Value::float(0.0f64.copysign(b)));
            }
            let (an, ad) = float_to_rational(a);
            let (bn, bd) = float_to_rational(b);
            let ad_power = ad.bits() as usize - 1;
            let bd_power = bd.bits() as usize - 1;
            let quotient = floor_div_rational(an.clone(), ad_power, bn.clone(), bd_power);
            let common_power = ad_power.max(bd_power);
            let numerator = (an << (common_power - ad_power)) - quotient * (bn << (common_power - bd_power));
            let result = dyadic_to_float(&numerator, common_power);
            Ok(Value::float(if result == 0.0 { 0.0f64.copysign(b) } else { result }))
        }
    }
}

/// Signature: `Number::**(_)`
#[phalcom_native_macros::primitive(
    Number,
    "**(_)",
    params = [Number],
    returns = Self,
    types = "(Number) -> Self",
    flow = receiver,
    effects = pure
)]
pub fn number_pow(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let base_val = receiver;
    let exp_val = &args[0];

    let Some(_pair) = cooperative_pair(base_val, exp_val, vm)? else {
        return Ok(vm.require_semantic_roots()?.unsupported);
    };

    // Identity check 1: 1 ** y = 1 (or 1.0)
    if base_val.as_int() == Some(1) {
        return Ok(if exp_val.is_float() { Value::float(1.0) } else { Value::int(1) });
    }
    // Identity check 2: x ** 0 = 1 (or 1.0)
    if exp_val.as_int() == Some(0) {
        return Ok(if base_val.is_float() { Value::float(1.0) } else { Value::int(1) });
    }

    let base_parsed = expect_number_big_or_float(base_val, vm)?;
    let exp_parsed = expect_number_big_or_float(exp_val, vm)?;

    match (base_parsed, exp_parsed) {
        (EitherIntOrFloat::Int(base), EitherIntOrFloat::Int(exp)) => {
            if exp.is_negative() {
                // Negative exponent promotes to Float
                let base_f = base.to_f64().unwrap_or(f64::NAN);
                let exp_f = exp.to_f64().unwrap_or(f64::NAN);
                if base_f == 0.0 {
                    return Err(vm.raise_numeric_error(RuntimeError::DivideByZero));
                }
                Ok(Value::float(base_f.powf(exp_f)))
            } else {
                if base.is_zero() {
                    return Ok(Value::int(0));
                }
                // Preflight estimated bits
                let limit = vm.numeric_policy.max_integer_bits.unwrap_or(8_388_608);
                let exp_usize = match exp.to_usize() {
                    Some(u) if u <= limit => u,
                    _ => return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("Power exponent exceeds limit".to_string()))),
                };
                let est_bits = integer_bits(&base) * exp_usize;
                if est_bits > limit {
                    return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("Power result exceeds bit limit".to_string())));
                }
                let res = pow_bigint(&base, &exp, limit, vm)?;
                Ok(normalize_bigint(res, &mut vm.heap))
            }
        }
        (base, exp) => {
            let base_f = match base {
                EitherIntOrFloat::Int(b) => b.to_f64().unwrap_or(f64::NAN),
                EitherIntOrFloat::Float(f) => f,
            };
            let exp_f = match exp {
                EitherIntOrFloat::Int(b) => b.to_f64().unwrap_or(f64::NAN),
                EitherIntOrFloat::Float(f) => f,
            };
            if base_f == 0.0 && exp_f < 0.0 {
                return Err(vm.raise_numeric_error(RuntimeError::DivideByZero));
            }
            Ok(Value::float(base_f.powf(exp_f)))
        }
    }
}

/// Signature: `Number::<(_)`
#[phalcom_native_macros::primitive(
    Number,
    "<(_)",
    params = [Number],
    returns = Bool,
    types = "(Number) -> Bool",
    effects = pure
)]
pub fn number_lt(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(matches!(exact_compare(receiver, &args[0], vm)?, Some(std::cmp::Ordering::Less))))
}

/// Signature: `Number::<=(_)`
#[phalcom_native_macros::primitive(
    Number,
    "<=(_)",
    params = [Number],
    returns = Bool,
    types = "(Number) -> Bool",
    effects = pure
)]
pub fn number_le(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(matches!(
        exact_compare(receiver, &args[0], vm)?,
        Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
    )))
}

/// Signature: `Number::>(_)`
#[phalcom_native_macros::primitive(
    Number,
    ">(_)",
    params = [Number],
    returns = Bool,
    types = "(Number) -> Bool",
    effects = pure
)]
pub fn number_gt(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(matches!(exact_compare(receiver, &args[0], vm)?, Some(std::cmp::Ordering::Greater))))
}

/// Signature: `Number::>=(_)`
#[phalcom_native_macros::primitive(
    Number,
    ">=(_)",
    params = [Number],
    returns = Bool,
    types = "(Number) -> Bool",
    effects = pure
)]
pub fn number_ge(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::bool(matches!(
        exact_compare(receiver, &args[0], vm)?,
        Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
    )))
}

#[cfg(test)]
mod tests {
    use super::{compare_integer_to_float, dyadic_to_float};
    use num_bigint::BigInt;

    #[test]
    fn dyadic_rounding_handles_ties_and_subnormal_boundary() {
        let tie_even = (BigInt::from(1u64) << 53) + 1;
        assert_eq!(dyadic_to_float(&tie_even, 53).to_bits(), 1.0f64.to_bits());

        let tie_odd = (BigInt::from(1u64) << 53) + 3;
        assert_eq!(dyadic_to_float(&tie_odd, 53).to_bits(), 1.0000000000000004f64.to_bits());

        let smallest_normal = BigInt::from(1u8);
        assert_eq!(dyadic_to_float(&smallest_normal, 1022).to_bits(), (2f64).powi(-1022).to_bits());
        assert_eq!(dyadic_to_float(&smallest_normal, 1074).to_bits(), 1);
    }

    #[test]
    fn mixed_comparison_keeps_integer_precision() {
        let integer = BigInt::from(9_007_199_254_740_993i64);
        let float = 9_007_199_254_740_992.0;

        assert_eq!(compare_integer_to_float(&integer, float), Some(std::cmp::Ordering::Greater));
        assert_eq!(
            compare_integer_to_float(&BigInt::from(9_007_199_254_740_992i64), float),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(compare_integer_to_float(&integer, f64::INFINITY), Some(std::cmp::Ordering::Less));
        assert_eq!(compare_integer_to_float(&integer, f64::NAN), None);
    }
}

pub(crate) fn ordering_value(vm: &mut VM, selector: &str) -> PhResult<Value> {
    let variant_name = match selector {
        "less" => "Less",
        "equal" => "Equal",
        "greater" => "Greater",
        "unordered" => "Unordered",
        _ => return Err(RuntimeError::Internal(format!("unknown Ordering variant `{selector}`")).into()),
    };
    let variant = VariantId::new(
        CoreDeclarationIds::default().ordering,
        Selector::getter(variant_name).expect("valid Ordering variant selector"),
    );
    let runtime_variant = vm
        .adt_registry
        .variant_by_semantic(&variant)
        .ok_or_else(|| RuntimeError::Internal(format!("Ordering variant `{variant_name}` unavailable")))?;
    if let Some(value) = vm.adt_registry.variant_descriptor(runtime_variant).and_then(|descriptor| descriptor.singleton) {
        return Ok(value);
    }
    vm.construct_variant_value(runtime_variant, Vec::new()).map_err(Into::into)
}

/// Signature: `Number::<=> ( _)` — returns the canonical Ordering singleton.
#[phalcom_native_macros::primitive(
    Number,
    "compare(_)",
    params = [Number],
    returns = Ordering,
    types = "(Number) -> Ordering",
    effects = pure
)]
pub fn number_compare(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if !is_number(receiver, vm) || !is_number(&args[0], vm) {
        return Ok(vm.require_semantic_roots()?.unsupported);
    }
    let Some(selector) = exact_compare(receiver, &args[0], vm)? else {
        return ordering_value(vm, "unordered");
    };
    ordering_value(
        vm,
        match selector {
            std::cmp::Ordering::Less => "less",
            std::cmp::Ordering::Equal => "equal",
            std::cmp::Ordering::Greater => "greater",
        },
    )
}

/// Signature: `Number::-` — arithmetic negation (unary `-x`).
///
/// Returns the additive inverse of the receiver. Promotes `i64::MIN` to a
/// `LargeInt` to avoid overflow.
#[phalcom_native_macros::primitive(
    Number,
    "-",
    params = [],
    returns = Self,
    types = "() -> Self",
    flow = receiver,
    effects = pure
)]
pub fn number_negate(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if let Some(n) = receiver.as_int() {
        if let Some(neg) = n.checked_neg() {
            Ok(Value::int(neg))
        } else {
            let big = -BigInt::from(n);
            check_limit_bigint(&big, vm)?;
            Ok(normalize_bigint(big, &mut vm.heap))
        }
    } else if let Some(n) = receiver.as_float() {
        Ok(Value::float(-n))
    } else if let Some(id) = receiver.as_obj() {
        if let Some(big) = vm.heap.as_large_int(id) {
            let neg = -big.clone();
            check_limit_bigint(&neg, vm)?;
            Ok(normalize_bigint(neg, &mut vm.heap))
        } else {
            Err(RuntimeError::Type {
                expected: "number",
                found: receiver.type_name(),
            }
            .into())
        }
    } else {
        Err(RuntimeError::Type {
            expected: "number",
            found: receiver.type_name(),
        }
        .into())
    }
}

/// Signature: `Number::+` — unary plus identity (unary `+x`).
///
/// Returns the receiver unchanged. Defined so user-defined classes and
/// numeric subclasses can override the `+` getter when needed.
#[phalcom_native_macros::primitive(
    Number,
    "+",
    params = [],
    returns = Self,
    types = "() -> Self",
    flow = receiver,
    effects = pure
)]
pub fn number_unary_plus(_vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(*receiver)
}

#[phalcom_native_macros::primitive(Float, "rawParse(_)", side = class, visibility = internal)]
pub fn float_class_raw_parse(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let s_ref = match args[0].as_obj() {
        Some(r) => r,
        None => return Ok(Value::none()),
    };
    let s = vm.heap.string(s_ref).as_str().trim();
    match s.parse::<f64>() {
        Ok(v) => Ok(Value::float(v).wrap_some().unwrap()),
        Err(_) => Ok(Value::none()),
    }
}
