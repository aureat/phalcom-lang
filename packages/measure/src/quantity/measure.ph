from ..algebra.kind import PointKind, QuantityKind
from ..exact.rational import Rational
from ..unit.affine import AffineUnit
from ..unit.linear import LinearUnit
from .point import Point
from .quantity import Quantity

/// Unit-retaining exact linear measurement.
data Measure<K>(
  _ magnitude: Rational,
  _ unit: LinearUnit<K>
)
where K |= QuantityKind

impl<K> Measure<K>
  where K |= QuantityKind
{
  magnitude -> Rational { self.magnitude }
  unit -> LinearUnit<K> { self.unit }
  quantity -> Quantity<K, Rational> { self.unit.exact(self.magnitude) }
  toString -> String { "\(self.magnitude) \(self.unit.symbol)" }
}

/// Unit-retaining exact affine coordinate measurement.
data PointMeasure<P>(
  _ magnitude: Rational,
  _ unit: AffineUnit<P>
)
where P |= PointKind

impl<P> PointMeasure<P>
  where P |= PointKind
{
  magnitude -> Rational { self.magnitude }
  unit -> AffineUnit<P> { self.unit }
  point -> Point<P, Rational> { self.unit.exact(self.magnitude) }
  toString -> String { "\(self.magnitude) \(self.unit.symbol)" }
}

export Measure, PointMeasure
