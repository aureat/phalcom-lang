/// A valid argument exceeds this implementation's numerical range.
class NumericalRangeError is Error {}
/// An iterative numerical algorithm did not converge.
class NumericalEvaluationError is Error {}

export NumericalRangeError, NumericalEvaluationError
