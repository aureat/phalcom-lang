//! Domain errors raised by `measure`.

/// Base class for measurement-domain failures.
class MeasureError is Error {}

/// Raised when a structural dimension cannot satisfy a requested semantic kind
/// or unit.
class DimensionMismatchError is MeasureError {}

/// Raised when a declared `ProductKind` or `QuotientKind` relation disagrees
/// with the structural dimensions of its operands and output.
class InvalidSemanticLawError is MeasureError {}

/// Raised when constructing a unit with an invalid or non-invertible scale.
class UnitDefinitionError is MeasureError {}

export MeasureError, DimensionMismatchError, InvalidSemanticLawError, UnitDefinitionError
