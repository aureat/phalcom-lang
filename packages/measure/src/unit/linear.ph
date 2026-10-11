from ..algebra.kind import QuantityKind
from ..algebra.unit import ExactStructuralUnit, ExactUnitFor, FloatUnitFor
from ..dimension.dimension import Dimension
from ..exact.rational import Rational
from ..errors import UnitDefinitionError
from ..quantity.quantity import Quantity
from .expression import UnitExpr, UnitLike

/// Named exact linear unit for semantic kind `K`.
///
/// Calling a unit with an `Int` creates an exact Rational quantity. Rational
/// source magnitudes use `exact(_)`; Float input is deliberately explicit via
/// `approx(_)` because Telos does not overload `call(_)` by argument type.
data LinearUnit<K> private(
  _ kind: K,
  _ expression: UnitExpr
)
where K |= QuantityKind

impl<K> LinearUnit<K>
  where K |= QuantityKind
{
  @class
  define(_ kind: K, _ symbol: String, _ expression: UnitExpr) -> LinearUnit<K> {
    if not expression.dimension.equivalent(kind.dimension) {
      UnitDefinitionError.new("unit \(symbol) does not match quantity kind \(kind.name)").raise()
    }
    LinearUnit(
      kind,
      UnitExpr.define(expression.dimension, scale: expression.scale, symbol: symbol)
    )
  }

  @class
  base(_ kind: K, _ symbol: String) -> LinearUnit<K> {
    LinearUnit.define(
      kind,
      symbol,
      UnitExpr.define(kind.dimension, scale: Rational.one, symbol: symbol)
    )
  }

  kind -> K { self.kind }
  expression -> UnitExpr { self.expression }
  symbol -> String { self.expression.symbol }

  call(_ magnitude: Int) -> Quantity<K, Rational> {
    self.exact(Rational.whole(magnitude))
  }

  exact(_ magnitude: Rational) -> Quantity<K, Rational> {
    Quantity.reference(self.kind, magnitude * self.expression.scale)
  }

  approx(_ magnitude: Float) -> Quantity<K, Float> {
    Quantity.reference(self.kind, magnitude * self.expression.scale.toFloat)
  }

  scaled(by factor: Rational, symbol: String) -> LinearUnit<K> {
    LinearUnit.define(
      self.kind,
      symbol,
      self.expression.scaled(by: factor, symbol: symbol)
    )
  }

  *<U>(_ other: U) -> UnitExpr
    where U |= UnitLike
  { self.expression * other }

  /<U>(_ other: U) -> UnitExpr
    where U |= UnitLike
  { self.expression / other }

  **(_ exponent: Int) -> UnitExpr { self.expression ** exponent }
}

impl<K> UnitLike for LinearUnit<K>
  where K |= QuantityKind
{
  expression -> UnitExpr { self.expression }
}

impl<K> ExactStructuralUnit for LinearUnit<K>
  where K |= QuantityKind
{
  dimension -> Dimension { self.expression.dimension }
  scale -> Rational { self.expression.scale }
}

impl<K> ExactUnitFor<K> for LinearUnit<K>
  where K |= QuantityKind
{
  fromReference(_ magnitude: Rational) -> Rational {
    magnitude / self.expression.scale
  }
}

impl<K> FloatUnitFor<K> for LinearUnit<K>
  where K |= QuantityKind
{
  fromReferenceFloat(_ magnitude: Float) -> Float {
    magnitude / self.expression.scale.toFloat
  }
}

export LinearUnit
