@!documentation("Float-only IEEE binary64 functions, stable Float reductions, and exact arbitrary-precision Int algorithms. Advanced special functions are in the optional numerics package. Angles use radians except degrees/radians conversions. NaN, infinity, signed-zero, and domain results follow the Rust f64 backend; Int is never implicitly converted to Float.")

expose .complex

from .float_kernel import FloatMathKernel
from .float_kernel import FloatAngleHelpers
from .numerical_errors import NumericalRangeError, NumericalEvaluationError
from .integer import IntegerMath
from .reduction import FloatReduction

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
let log = &FloatMathKernel.log(_, base)
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
let degrees = &FloatAngleHelpers.degrees(_)
let radians = &FloatAngleHelpers.radians(_)

let logSumExp = &FloatReduction.logSumExp(_)
let sum = &FloatReduction.sum(_)
let mean = &FloatReduction.mean(_)
let logAddExp = &FloatReduction.logAddExp(_, _)
let log1mexp = &FloatReduction.log1mexp(_)

let factorial = &IntegerMath.factorial(_)
let choose = &IntegerMath.choose(_, _)
let fallingFactorial = &IntegerMath.fallingFactorial(_, _)
let risingFactorial = &IntegerMath.risingFactorial(_, _)

let gcd = &IntegerMath.gcd(_, _)
let lcm = &IntegerMath.lcm(_, _)
let isqrt = &IntegerMath.isqrt(_)

export (
  pi, tau, e, ln2, ln10, sqrt2,
  sqrt, cbrt, exp, exp2, expm1, ln, ln1p, log, log2, log10,
  sin, cos, tan, asin, acos, atan, atan2,
  degrees, radians, sinh, cosh, tanh, asinh, acosh, atanh, hypot,
  gcd, lcm, isqrt, sum, mean, logSumExp, logAddExp, log1mexp,
  factorial, choose, fallingFactorial, risingFactorial,
  NumericalRangeError, NumericalEvaluationError
)
