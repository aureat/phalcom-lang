//! Binary64 mathematical kernels declared in `universe.math.float_kernel`.

use crate::error::{PhResult, RuntimeError};
use crate::value::Value;
use crate::vm::VM;

fn float_argument(value: &Value) -> PhResult<f64> {
    value.as_float().ok_or_else(|| {
        RuntimeError::Type {
            expected: "Float",
            found: value.type_name(),
        }
        .into()
    })
}

fn unary(args: &[Value], operation: fn(f64) -> f64) -> PhResult<Value> {
    Ok(Value::float(operation(float_argument(&args[0])?)))
}

/// Binary64 square root.
#[phalcom_native_macros::primitive(FloatMathKernel, "sqrt(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_sqrt(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::sqrt) }

/// Binary64 cube root.
#[phalcom_native_macros::primitive(FloatMathKernel, "cbrt(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_cbrt(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::cbrt) }

/// Binary64 exponential.
#[phalcom_native_macros::primitive(FloatMathKernel, "exp(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_exp(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::exp) }

/// Binary64 base-two exponential.
#[phalcom_native_macros::primitive(FloatMathKernel, "exp2(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_exp2(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::exp2) }

/// Binary64 exp(x)-1 near zero.
#[phalcom_native_macros::primitive(FloatMathKernel, "expm1(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_expm1(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::exp_m1) }

/// Binary64 natural logarithm.
#[phalcom_native_macros::primitive(FloatMathKernel, "ln(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_ln(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::ln) }

/// Binary64 ln(1+x) near zero.
#[phalcom_native_macros::primitive(FloatMathKernel, "ln1p(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_ln1p(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::ln_1p) }

/// Binary64 logarithm with an explicit base.
#[phalcom_native_macros::primitive(
    FloatMathKernel, "log(_,base)", params = [Float, base: Float], returns = Float,
    types = "(Float, base: Float) -> Float", side = class, effects = pure
)]
pub fn math_log(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::float(float_argument(&args[0])?.log(float_argument(&args[1])?)))
}

/// Binary64 base-two logarithm.
#[phalcom_native_macros::primitive(FloatMathKernel, "log2(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_log2(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::log2) }

/// Binary64 base-ten logarithm.
#[phalcom_native_macros::primitive(FloatMathKernel, "log10(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_log10(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::log10) }

/// Binary64 sine in radians.
#[phalcom_native_macros::primitive(FloatMathKernel, "sin(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_sin(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::sin) }

/// Binary64 cosine in radians.
#[phalcom_native_macros::primitive(FloatMathKernel, "cos(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_cos(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::cos) }

/// Binary64 tangent in radians.
#[phalcom_native_macros::primitive(FloatMathKernel, "tan(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_tan(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::tan) }

/// Binary64 inverse sine in radians.
#[phalcom_native_macros::primitive(FloatMathKernel, "asin(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_asin(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::asin) }

/// Binary64 inverse cosine in radians.
#[phalcom_native_macros::primitive(FloatMathKernel, "acos(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_acos(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::acos) }

/// Binary64 inverse tangent in radians.
#[phalcom_native_macros::primitive(FloatMathKernel, "atan(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_atan(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::atan) }

/// Binary64 hyperbolic sine.
#[phalcom_native_macros::primitive(FloatMathKernel, "sinh(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_sinh(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::sinh) }

/// Binary64 hyperbolic cosine.
#[phalcom_native_macros::primitive(FloatMathKernel, "cosh(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_cosh(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::cosh) }

/// Binary64 hyperbolic tangent.
#[phalcom_native_macros::primitive(FloatMathKernel, "tanh(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_tanh(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::tanh) }

/// Binary64 inverse hyperbolic sine.
#[phalcom_native_macros::primitive(FloatMathKernel, "asinh(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_asinh(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::asinh) }

/// Binary64 inverse hyperbolic cosine.
#[phalcom_native_macros::primitive(FloatMathKernel, "acosh(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_acosh(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::acosh) }

/// Binary64 inverse hyperbolic tangent.
#[phalcom_native_macros::primitive(FloatMathKernel, "atanh(_)", params = [Float], returns = Float, types = "(Float) -> Float", side = class, effects = pure)]
pub fn math_atanh(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> { unary(args, f64::atanh) }

/// Binary64 two-argument hypotenuse with scaled evaluation.
#[phalcom_native_macros::primitive(
    FloatMathKernel, "hypot(_,_)", params = [Float, Float], returns = Float,
    types = "(Float, Float) -> Float", side = class, effects = pure
)]
pub fn math_hypot(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::float(float_argument(&args[0])?.hypot(float_argument(&args[1])?)))
}

/// IEEE-compatible two-argument arctangent, with `y` preceding `x`.
#[phalcom_native_macros::primitive(
    FloatMathKernel, "atan2(_,_)", params = [Float, Float], returns = Float,
    types = "(Float, Float) -> Float", side = class, effects = pure
)]
pub fn math_atan2(_vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    Ok(Value::float(float_argument(&args[0])?.atan2(float_argument(&args[1])?)))
}
