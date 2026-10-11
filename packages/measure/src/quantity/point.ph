from ..algebra.kind import PointKind, QuantityKind
from ..algebra.scalar import QuantityScalar
from ..algebra.unit import ExactPointUnitFor, FloatPointUnitFor
from ..exact.rational import Rational
from .quantity import Quantity

/// Normalized affine point.
///
/// Point subtraction is reserved for `Point - Point -> Difference`. Because
/// Telos does not overload `-(_)` by RHS type, translation by a difference is
/// expressed with `advanced(by:)` and `retreated(by:)`.
data Point<P, N> private(
  _ kind: P,
  _ referenceMagnitude: N
)
where P |= PointKind

impl<P, N> Point<P, N>
  where
    P |= PointKind,
    <P as PointKind>::Difference |= QuantityKind
{
  @class
  reference(_ kind: P, _ magnitude: N) -> Point<P, N> {
    Point(kind, magnitude)
  }

  kind -> P { self.kind }
  magnitude -> N { self.referenceMagnitude }

  -(_ other: Point<P, N>)
    -> Quantity<<P as PointKind>::Difference, N>
    where N |= QuantityScalar
  {
    Quantity.reference(
      self.kind.differenceKind,
      self.referenceMagnitude.subtractScalar(other.referenceMagnitude)
    )
  }

  advanced(
    by difference: Quantity<<P as PointKind>::Difference, N>
  ) -> Point<P, N>
    where N |= QuantityScalar
  {
    Point.reference(
      self.kind,
      self.referenceMagnitude.addScalar(difference.magnitude)
    )
  }

  retreated(
    by difference: Quantity<<P as PointKind>::Difference, N>
  ) -> Point<P, N>
    where N |= QuantityScalar
  {
    Point.reference(
      self.kind,
      self.referenceMagnitude.subtractScalar(difference.magnitude)
    )
  }

  compare(_ other: Point<P, N>) -> Ordering
    where N |= QuantityScalar
  {
    self.referenceMagnitude.compareScalar(other.referenceMagnitude)
  }

  <(_ other: Point<P, N>) -> Bool
    where N |= QuantityScalar
  { self.compare(other) === Ordering::Less }

  <=(_ other: Point<P, N>) -> Bool
    where N |= QuantityScalar
  {
    let order = self.compare(other)
    order === Ordering::Less or order === Ordering::Equal
  }

  >(_ other: Point<P, N>) -> Bool
    where N |= QuantityScalar
  { self.compare(other) === Ordering::Greater }

  >=(_ other: Point<P, N>) -> Bool
    where N |= QuantityScalar
  {
    let order = self.compare(other)
    order === Ordering::Greater or order === Ordering::Equal
  }

  in<U>(_ unit: U) -> Rational
    where N == Rational, U |= ExactPointUnitFor<P>
  {
    unit.fromReferencePoint(self.referenceMagnitude)
  }

  inApprox<U>(_ unit: U) -> Float
    where N == Float, U |= FloatPointUnitFor<P>
  {
    unit.fromReferencePointFloat(self.referenceMagnitude)
  }

  toString -> String { "\(self.referenceMagnitude) \(self.kind.name)" }
}

export Point
