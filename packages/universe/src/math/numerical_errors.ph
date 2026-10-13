/// Reports a valid argument or result outside the supported numeric range.
class NumericalRangeError is Error {}
/// Reports failure of a bounded numerical evaluation to converge.
class NumericalEvaluationError is Error {}

export NumericalRangeError, NumericalEvaluationError
