/// Reports a valid argument or result outside the supported numeric range.
class NumericalRangeError is Error {}
/// An iterative numerical algorithm did not converge.
class NumericalEvaluationError is Error {}

export NumericalRangeError, NumericalEvaluationError
