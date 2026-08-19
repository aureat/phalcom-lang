//! Native primitives on `Number`.

use crate::error::{PhResult, RuntimeError};
use crate::value::{Value, normalize_bigint};
use crate::vm::VM;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::FromPrimitive;
use num_traits::cast::ToPrimitive;
use num_traits::identities::Zero;
use num_traits::sign::Signed;

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

fn check_limit_bigint(n: &BigInt, vm: &mut VM) -> PhResult<()> {
    let limit = vm.numeric_policy.max_integer_bits.unwrap_or(8_388_608);
    if integer_bits(n) > limit {
        return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("Integer bit length exceeds configured limit".to_string())));
    }
    Ok(())
}

fn floor_div_i64(a: i64, b: i64) -> i64 {
    let res = a / b;
    let rem = a % b;
    if (rem > 0 && b < 0) || (rem < 0 && b > 0) { res - 1 } else { res }
}

fn floor_mod_i64(a: i64, b: i64) -> i64 {
    let rem = a % b;
    if (rem > 0 && b < 0) || (rem < 0 && b > 0) { rem + b } else { rem }
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
    returns = Nothing,
    types = "(Object) -> Nothing",
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
    returns = Number,
    types = "(Number) -> Number",
    effects = pure
)]
pub fn number_add(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let pair = promote_pair(receiver, &args[0], vm)?;
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
    returns = Number,
    types = "(Number) -> Number",
    effects = pure
)]
pub fn number_sub(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let pair = promote_pair(receiver, &args[0], vm)?;
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
pub fn number_mul(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let pair = promote_pair(receiver, &args[0], vm)?;
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
pub fn number_div(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let pair = promote_pair(receiver, &args[0], vm)?;
    let (a, b) = match pair {
        PromotedPair::Int(a, b) => (a as f64, b as f64),
        PromotedPair::Big(a, b) => (a.to_f64().unwrap_or(f64::NAN), b.to_f64().unwrap_or(f64::NAN)),
        PromotedPair::Float(a, b) => (a, b),
    };
    Ok(Value::float(a / b))
}

/// Signature: `Number::~/(_)` — floor division.
pub fn number_floor_div(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let pair = promote_pair(receiver, &args[0], vm)?;
    match pair {
        PromotedPair::Int(a, b) => {
            if b == 0 {
                return Err(vm.raise_numeric_error(RuntimeError::DivideByZero));
            }
            Ok(Value::int(floor_div_i64(a, b)))
        }
        PromotedPair::Big(a, b) => {
            if b.is_zero() {
                return Err(vm.raise_numeric_error(RuntimeError::DivideByZero));
            }
            let res = a.div_floor(&b);
            check_limit_bigint(&res, vm)?;
            Ok(normalize_bigint(res, &mut vm.heap))
        }
        PromotedPair::Float(a, b) => {
            if b == 0.0 {
                return Err(vm.raise_numeric_error(RuntimeError::DivideByZero));
            }
            if !a.is_finite() || !b.is_finite() {
                return Err(vm.raise_numeric_error(RuntimeError::NonFiniteNumber("non-finite operand".to_string())));
            }
            let q = a / b;
            if !q.is_finite() {
                return Err(vm.raise_numeric_error(RuntimeError::NonFiniteNumber("non-finite quotient".to_string())));
            }
            if let Some(big) = BigInt::from_f64(q.floor()) {
                check_limit_bigint(&big, vm)?;
                Ok(normalize_bigint(big, &mut vm.heap))
            } else {
                Err(vm.raise_numeric_error(RuntimeError::NonFiniteNumber("non-finite quotient".to_string())))
            }
        }
    }
}

/// Signature: `Number::%(_)`
pub fn number_mod(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let pair = promote_pair(receiver, &args[0], vm)?;
    match pair {
        PromotedPair::Int(a, b) => {
            if b == 0 {
                return Err(vm.raise_numeric_error(RuntimeError::DivideByZero));
            }
            Ok(Value::int(floor_mod_i64(a, b)))
        }
        PromotedPair::Big(a, b) => {
            if b.is_zero() {
                return Err(vm.raise_numeric_error(RuntimeError::DivideByZero));
            }
            let res = a.mod_floor(&b);
            check_limit_bigint(&res, vm)?;
            Ok(normalize_bigint(res, &mut vm.heap))
        }
        PromotedPair::Float(a, b) => Ok(Value::float(a % b)),
    }
}

/// Signature: `Number::**(_)`
pub fn number_pow(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let base_val = receiver;
    let exp_val = &args[0];

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
pub fn number_lt(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let pair = promote_pair(receiver, &args[0], vm)?;
    match pair {
        PromotedPair::Int(a, b) => Ok(Value::bool(a < b)),
        PromotedPair::Big(a, b) => Ok(Value::bool(a < b)),
        PromotedPair::Float(a, b) => Ok(Value::bool(a < b)),
    }
}

/// Signature: `Number::<=(_)`
pub fn number_le(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let pair = promote_pair(receiver, &args[0], vm)?;
    match pair {
        PromotedPair::Int(a, b) => Ok(Value::bool(a <= b)),
        PromotedPair::Big(a, b) => Ok(Value::bool(a <= b)),
        PromotedPair::Float(a, b) => Ok(Value::bool(a <= b)),
    }
}

/// Signature: `Number::>(_)`
pub fn number_gt(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let pair = promote_pair(receiver, &args[0], vm)?;
    match pair {
        PromotedPair::Int(a, b) => Ok(Value::bool(a > b)),
        PromotedPair::Big(a, b) => Ok(Value::bool(a > b)),
        PromotedPair::Float(a, b) => Ok(Value::bool(a > b)),
    }
}

/// Signature: `Number::>=(_)`
pub fn number_ge(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let pair = promote_pair(receiver, &args[0], vm)?;
    match pair {
        PromotedPair::Int(a, b) => Ok(Value::bool(a >= b)),
        PromotedPair::Big(a, b) => Ok(Value::bool(a >= b)),
        PromotedPair::Float(a, b) => Ok(Value::bool(a >= b)),
    }
}

/// Signature: `Number::negated()`
pub fn number_negated(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
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
