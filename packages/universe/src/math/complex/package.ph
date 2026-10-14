@!documentation("Principal-branch Complex<Float> mathematics. Signed zeros select branch-cut sides; poles and nonfinite arguments return IEEE components. Arguments require explicit complex construction.")
from .kernel import ComplexMathKernel as K

let sqrt = &K.sqrt(_)
let exp = &K.exp(_)
let log = &K.log(_)
let log1p = &K.log1p(_)
let sin = &K.sin(_)
let cos = &K.cos(_)
let tan = &K.tan(_)
let asin = &K.asin(_)
let acos = &K.acos(_)
let atan = &K.atan(_)
let sinh = &K.sinh(_)
let cosh = &K.cosh(_)
let tanh = &K.tanh(_)
let asinh = &K.asinh(_)
let acosh = &K.acosh(_)
let atanh = &K.atanh(_)
let powInt = &K.powInt(_, _)
let pow = &K.pow(_, _)

export sqrt, exp, log, log1p, sin, cos, tan, asin, acos, atan, sinh, cosh, tanh, asinh, acosh, atanh, powInt, pow
