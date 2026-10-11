from ..algebra.kind import QuantityKind
from ..algebra.scalar import QuantityScalar
from ..algebra.unit import ExactStructuralUnit
from ..dimension.dimension import Dimension
from ..exact.rational import Rational
from ..errors import DimensionMismatchError
from .quantity import Quantity

/// Structurally dimensioned quantity without a nominal semantic kind.
///
/// `DimQuantity` is the runtime structural layer analogous to units-of-measure
/// algebra: dimensions compose automatically, but the library does not guess a
/// semantic meaning from the result. `refined(to:)` explicitly attaches a
/// `QuantityKind` after checking dimensions.
data DimQuantity<N> private(
  _ dimension: Dimension,
  _ referenceMagnitude: N
)

impl<N> DimQuantity<N> {
  @class
  reference(_ dimension: Dimension, _ magnitude: N) -> DimQuantity<N> {
    DimQuantity(dimension, magnitude)
  }

  dimension -> Dimension { self.dimension }
  magnitude -> N { self.referenceMagnitude }

  *(_ other: DimQuantity<N>) -> DimQuantity<N>
    where N |= QuantityScalar
  {
    DimQuantity.reference(
      self.dimension * other.dimension,
      self.referenceMagnitude.multiplyScalar(other.referenceMagnitude)
    )
  }

  /(_ other: DimQuantity<N>) -> DimQuantity<N>
    where N |= QuantityScalar
  {
    DimQuantity.reference(
      self.dimension / other.dimension,
      self.referenceMagnitude.divideScalar(other.referenceMagnitude)
    )
  }

  scaled(by scalar: N) -> DimQuantity<N>
    where N |= QuantityScalar
  {
    DimQuantity.reference(
      self.dimension,
      self.referenceMagnitude.multiplyScalar(scalar)
    )
  }

  /// Explicitly refines structural dimensional meaning into a nominal kind.
  refined<K>(to kind: K) -> Quantity<K, N>
    where K |= QuantityKind
  {
    if not self.dimension.equivalent(kind.dimension) {
      DimensionMismatchError.new("DimQuantity dimension does not match requested kind").raise()
    }
    Quantity.reference(kind, self.referenceMagnitude)
  }

  /// Expresses an exact structural quantity in a compatible structural unit.
  in<U>(_ unit: U) -> Rational
    where N == Rational, U |= ExactStructuralUnit
  {
    if not self.dimension.equivalent(unit.dimension) {
      DimensionMismatchError.new("structural unit has incompatible dimension").raise()
    }
    self.referenceMagnitude / unit.scale
  }

  /// Expresses a Float structural quantity in an exact structural unit.
  inApprox<U>(_ unit: U) -> Float
    where N == Float, U |= ExactStructuralUnit
  {
    if not self.dimension.equivalent(unit.dimension) {
      DimensionMismatchError.new("structural unit has incompatible dimension").raise()
    }
    self.referenceMagnitude / unit.scale.toFloat
  }

  toString -> String {
    "\(self.referenceMagnitude) [\(self.dimension)]"
  }
}

export DimQuantity
