//! Nominal semantic-kind relations layered above structural dimensions.

from ..dimension.dimension import Dimension

/// Nominal semantic identity of a normalized quantity.
trait QuantityKind {
  name -> String
  dimension -> Dimension
}

trait ProductKind<Rhs> {
  type Output
  productKind(_ rhs: Rhs) -> Self::Output
}

trait QuotientKind<Rhs> {
  type Output
  quotientKind(_ rhs: Rhs) -> Self::Output
}

trait PointKind {
  type Difference
  name -> String
  dimension -> Dimension
  differenceKind -> Self::Difference
}

export QuantityKind, ProductKind, QuotientKind, PointKind
