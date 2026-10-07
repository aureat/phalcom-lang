@!documentation("IEEE binary64 mathematical functions (radians, no implicit Int conversion) and exact Int algorithms.")

expose .integer

from universe.math.integer import IntegerMath

/// Native binary64 kernels; the public namespace exports selected callable Families.
native class FloatMathKernel is Object {
  @class native sqrt(_ value: Float) -> Float
  @class native cbrt(_ value: Float) -> Float
  @class native exp(_ value: Float) -> Float
  @class native exp2(_ value: Float) -> Float
  @class native expm1(_ value: Float) -> Float
  @class native ln(_ value: Float) -> Float
  @class native ln1p(_ value: Float) -> Float
  @class native log2(_ value: Float) -> Float
  @class native log10(_ value: Float) -> Float
  @class native sin(_ value: Float) -> Float
  @class native cos(_ value: Float) -> Float
  @class native tan(_ value: Float) -> Float
  @class native asin(_ value: Float) -> Float
  @class native acos(_ value: Float) -> Float
  @class native atan(_ value: Float) -> Float
  @class native atan2(_ y: Float, _ x: Float) -> Float
  @class native sinh(_ value: Float) -> Float
  @class native cosh(_ value: Float) -> Float
  @class native tanh(_ value: Float) -> Float
  @class native asinh(_ value: Float) -> Float
  @class native acosh(_ value: Float) -> Float
  @class native atanh(_ value: Float) -> Float
  @class native hypot(_ x: Float, _ y: Float) -> Float
}

const pi: Float = 3.141592653589793
const tau: Float = 6.283185307179586
const e: Float = 2.718281828459045
const ln2: Float = 0.6931471805599453
const ln10: Float = 2.302585092994046
const sqrt2: Float = 1.4142135623730951

let sqrt = &FloatMathKernel.sqrt(_)
let cbrt = &FloatMathKernel.cbrt(_)
let exp = &FloatMathKernel.exp(_)
let exp2 = &FloatMathKernel.exp2(_)
let expm1 = &FloatMathKernel.expm1(_)
let ln = &FloatMathKernel.ln(_)
let ln1p = &FloatMathKernel.ln1p(_)
let log2 = &FloatMathKernel.log2(_)
let log10 = &FloatMathKernel.log10(_)
let sin = &FloatMathKernel.sin(_)
let cos = &FloatMathKernel.cos(_)
let tan = &FloatMathKernel.tan(_)
let asin = &FloatMathKernel.asin(_)
let acos = &FloatMathKernel.acos(_)
let atan = &FloatMathKernel.atan(_)
let atan2 = &FloatMathKernel.atan2(_, _)
let sinh = &FloatMathKernel.sinh(_)
let cosh = &FloatMathKernel.cosh(_)
let tanh = &FloatMathKernel.tanh(_)
let asinh = &FloatMathKernel.asinh(_)
let acosh = &FloatMathKernel.acosh(_)
let atanh = &FloatMathKernel.atanh(_)
let hypot = &FloatMathKernel.hypot(_, _)

let gcd = &IntegerMath.gcd(_, _)
let lcm = &IntegerMath.lcm(_, _)
let isqrt = &IntegerMath.isqrt(_)

export (
  pi, tau, e, ln2, ln10, sqrt2,
  sqrt, cbrt, exp, exp2, expm1, ln, ln1p, log2, log10,
  sin, cos, tan, asin, acos, atan, atan2,
  sinh, cosh, tanh, asinh, acosh, atanh, hypot,
  gcd, lcm, isqrt
)
