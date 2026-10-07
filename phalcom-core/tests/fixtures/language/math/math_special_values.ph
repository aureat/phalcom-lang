// area: math
// status: PASS

import universe.math as math

let positiveInfinity = 1.0 / 0.0
let nan = 0.0 / 0.0
let subnormal = 5e-324

System.print(math.sqrt(-1.0).isNaN)
System.print(math.sqrt(-0.0))
System.print(math.sqrt(positiveInfinity))
System.print(math.ln(-1.0).isNaN)
System.print(math.ln(0.0))
System.print(math.ln(-0.0))
System.print(math.ln(1.0))
System.print(math.ln(positiveInfinity))
System.print(math.ln1p(-1.0))
System.print(math.ln1p(-1.5).isNaN)
System.print(math.log(-1.0, base: 2.0).isNaN)
System.print(math.log(0.0, base: 2.0))
System.print(math.log2(positiveInfinity))
System.print(math.log10(0.0))
System.print(math.asin(1.0))
System.print(math.asin(1.01).isNaN)
System.print(math.acos(-1.0))
System.print(math.atan(positiveInfinity))
System.print(math.atan2(0.0, 0.0))
System.print(math.atan2(0.0, -0.0))
System.print(math.atan2(-0.0, 0.0))
System.print(math.atan2(-0.0, -0.0))
System.print(math.acosh(1.0))
System.print(math.acosh(0.5).isNaN)
System.print(math.atanh(1.0))
System.print(math.atanh(1.1).isNaN)
System.print(math.sqrt(subnormal).isFinite)
System.print(nan.isNaN)
