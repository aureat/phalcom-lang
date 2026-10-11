from ..algebra.kind import PointKind
from ..algebra.unit import ExactPointUnitFor, FloatPointUnitFor
from ..exact.rational import Rational
from ..errors import UnitDefinitionError
from ..quantity.point import Point

/// Exact affine coordinate unit.
///
/// The transformation is `reference = magnitude × scale + offset`.
/// Affine units intentionally do not conform to `UnitLike`.
data AffineUnit<P> private(
  _ kind: P,
  _ symbol: String,
  _ scale: Rational,
  _ offset: Rational
)
where P |= PointKind

impl<P> AffineUnit<P>
  where P |= PointKind
{
  @class
  define(
    _ kind: P,
    _ symbol: String,
    scale: Rational,
    offset: Rational
  ) -> AffineUnit<P> {
    if scale.isZero {
      UnitDefinitionError.new("affine unit scale cannot be zero").raise()
    }
    AffineUnit(kind, symbol, scale, offset)
  }

  call(_ magnitude: Int) -> Point<P, Rational> {
    self.exact(Rational.whole(magnitude))
  }

  exact(_ magnitude: Rational) -> Point<P, Rational> {
    Point.reference(self.kind, magnitude * self.scale + self.offset)
  }

  approx(_ magnitude: Float) -> Point<P, Float> {
    Point.reference(
      self.kind,
      magnitude * self.scale.toFloat + self.offset.toFloat
    )
  }

  symbol -> String { self.symbol }
}

impl<P> ExactPointUnitFor<P> for AffineUnit<P>
  where P |= PointKind
{
  fromReferencePoint(_ magnitude: Rational) -> Rational {
    (magnitude - self.offset) / self.scale
  }
}

impl<P> FloatPointUnitFor<P> for AffineUnit<P>
  where P |= PointKind
{
  fromReferencePointFloat(_ magnitude: Float) -> Float {
    (magnitude - self.offset.toFloat) / self.scale.toFloat
  }
}

export AffineUnit
