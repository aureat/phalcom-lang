@!documentation("Positive-real special functions, structured mathematical families (Gamma, Beta, Normal, Combinatorics), and error functions.")

from .kernel import SpecialKernel, NumericalRangeError, NumericalEvaluationError

// Combinatorial functions
let logFactorial = &SpecialKernel.logFactorial(_)
let logChoose = &SpecialKernel.logChoose(_, _)

// Error functions
let erf = &SpecialKernel.erf(_)
let erfc = &SpecialKernel.erfc(_)
let logErfc = &SpecialKernel.logErfc(_)

// Structured mathematical families
data Gamma()
data Beta()
data Normal()
data Combinatorics()

impl Gamma {
  @class
  log(_ x: Float) -> Float {
    SpecialKernel.logGamma(x)
  }

  @class
  p(_ a: Float, _ x: Float) -> Float {
    SpecialKernel.regularizedGammaP(a, x)
  }

  @class
  q(_ a: Float, _ x: Float) -> Float {
    SpecialKernel.regularizedGammaQ(a, x)
  }

  @class
  logP(_ a: Float, _ x: Float) -> Float {
    SpecialKernel.logRegularizedGammaP(a, x)
  }

  @class
  logQ(_ a: Float, _ x: Float) -> Float {
    SpecialKernel.logRegularizedGammaQ(a, x)
  }

  @class
  inverseP(_ a: Float, _ p: Float) -> Float {
    SpecialKernel.inverseRegularizedGammaP(a, p)
  }
}

impl Beta {
  @class
  log(_ a: Float, _ b: Float) -> Float {
    SpecialKernel.logBeta(a, b)
  }

  @class
  regularized(_ x: Float, _ a: Float, _ b: Float) -> Float {
    SpecialKernel.regularizedBeta(a, b, x)
  }

  @class
  complement(_ x: Float, _ a: Float, _ b: Float) -> Float {
    SpecialKernel.regularizedBetaComplement(a, b, x)
  }

  @class
  logRegularized(_ x: Float, _ a: Float, _ b: Float) -> Float {
    SpecialKernel.logRegularizedBeta(a, b, x)
  }

  @class
  logComplement(_ x: Float, _ a: Float, _ b: Float) -> Float {
    SpecialKernel.logRegularizedBetaComplement(a, b, x)
  }

  @class
  inverse(_ p: Float, _ a: Float, _ b: Float) -> Float {
    SpecialKernel.inverseRegularizedBeta(a, b, p)
  }
}

impl Normal {
  @class
  quantile(_ p: Float) -> Float {
    SpecialKernel.inverseNormal(p)
  }

  @class
  cdf(_ x: Float) -> Float {
    Normal.survival(-x)
  }

  @class
  logCdf(_ x: Float) -> Float {
    Normal.logSurvival(-x)
  }

  @class
  survival(_ x: Float) -> Float {
    if x.isNaN { ArgumentError.new("Normal.survival requires a non-NaN Float").raise() }
    0.5 * SpecialKernel.erfc(x * 0.7071067811865475)
  }

  @class
  logSurvival(_ x: Float) -> Float {
    if x.isNaN { ArgumentError.new("Normal.logSurvival requires a non-NaN Float").raise() }
    SpecialKernel.logErfc(x * 0.7071067811865475) - 0.6931471805599453
  }
}

impl Combinatorics {
  @class
  logFactorial(_ n: Int) -> Float {
    SpecialKernel.logFactorial(n)
  }

  @class
  logChoose(_ n: Int, _ k: Int) -> Float {
    SpecialKernel.logChoose(n, k)
  }
}

export (
  logFactorial,
  logChoose,

  erf,
  erfc,
  logErfc,

  Gamma,
  Beta,
  Normal,
  Combinatorics,

  NumericalRangeError,
  NumericalEvaluationError
)
