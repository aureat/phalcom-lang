from .kernel import SpecialKernel, NumericalRangeError, NumericalEvaluationError

let logFactorial = &SpecialKernel.logFactorial(_)
let logChoose = &SpecialKernel.logChoose(_, _)

let logGamma = &SpecialKernel.logGamma(_)
let logBeta = &SpecialKernel.logBeta(_, _)
let erfc = &SpecialKernel.erfc(_)
let logErfc = &SpecialKernel.logErfc(_)
let regularizedGammaP = &SpecialKernel.regularizedGammaP(_, _)
let regularizedGammaQ = &SpecialKernel.regularizedGammaQ(_, _)
let logRegularizedGammaP = &SpecialKernel.logRegularizedGammaP(_, _)
let logRegularizedGammaQ = &SpecialKernel.logRegularizedGammaQ(_, _)
let regularizedBeta = &SpecialKernel.regularizedBeta(_, _, _)
let regularizedBetaComplement = &SpecialKernel.regularizedBetaComplement(_, _, _)
let logRegularizedBeta = &SpecialKernel.logRegularizedBeta(_, _, _)
let logRegularizedBetaComplement = &SpecialKernel.logRegularizedBetaComplement(_, _, _)
let inverseRegularizedBeta = &SpecialKernel.inverseRegularizedBeta(_, _, _)
let inverseRegularizedGammaP = &SpecialKernel.inverseRegularizedGammaP(_, _)
let inverseNormal = &SpecialKernel.inverseNormal(_)

export (
  logFactorial, logChoose,
  logGamma,
  logBeta,
  erfc,
  logErfc,
  regularizedGammaP,
  regularizedGammaQ,
  logRegularizedGammaP,
  logRegularizedGammaQ,
  regularizedBeta,
  regularizedBetaComplement,
  logRegularizedBeta,
  logRegularizedBetaComplement,
  inverseRegularizedBeta,
  inverseRegularizedGammaP,
  inverseNormal,
  NumericalRangeError, NumericalEvaluationError
)
