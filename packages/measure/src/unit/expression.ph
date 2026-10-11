from ..algebra.unit import ExactStructuralUnit
from ..dimension.dimension import Dimension
from ..exact.rational import Rational
from ..errors import UnitDefinitionError
from ..quantity.dimensional import DimQuantity

/// Exact structural unit expression.
///
/// Physical equivalence is determined only by `dimension` and `scale`.
/// `symbol` is presentation metadata.
data UnitExpr private(
  _ dimension: Dimension,
  _ scale: Rational,
  _ symbol: String
)

/// Values that participate in exact multiplicative unit algebra.
trait UnitLike {
  expression -> UnitExpr
}

impl UnitExpr {
  @class
  define(_ dimension: Dimension, scale: Rational, symbol: String) -> UnitExpr {
    if scale <= Rational.zero {
      UnitDefinitionError.new("linear unit scale must be positive").raise()
    }
    UnitExpr(dimension, scale, symbol)
  }

  dimension -> Dimension { self.dimension }
  scale -> Rational { self.scale }
  symbol -> String { self.symbol }
  expression -> UnitExpr { self }

  /// Constructs an exact structural quantity without assigning a nominal kind.
  call(_ magnitude: Int) -> DimQuantity<Rational> {
    self.exact(Rational.whole(magnitude))
  }

  exact(_ magnitude: Rational) -> DimQuantity<Rational> {
    DimQuantity.reference(self.dimension, magnitude * self.scale)
  }

  approx(_ magnitude: Float) -> DimQuantity<Float> {
    DimQuantity.reference(self.dimension, magnitude * self.scale.toFloat)
  }

  *<U>(_ other: U) -> UnitExpr
    where U |= UnitLike
  {
    let rhs = other.expression
    UnitExpr(
      self.dimension * rhs.dimension,
      self.scale * rhs.scale,
      "\(self.symbol)·\(rhs.symbol)"
    )
  }

  /<U>(_ other: U) -> UnitExpr
    where U |= UnitLike
  {
    let rhs = other.expression
    UnitExpr(
      self.dimension / rhs.dimension,
      self.scale / rhs.scale,
      "\(self.symbol)/\(rhs.symbol)"
    )
  }

  **(_ exponent: Int) -> UnitExpr {
    UnitExpr(
      self.dimension ** exponent,
      self.scale ** exponent,
      "\(self.symbol)^\(exponent)"
    )
  }

  scaled(by factor: Rational, symbol: String) -> UnitExpr {
    if factor <= Rational.zero {
      UnitDefinitionError.new("linear unit scale factor must be positive").raise()
    }
    UnitExpr(self.dimension, self.scale * factor, symbol)
  }

  equivalent(_ other: UnitExpr) -> Bool {
    self.dimension.equivalent(other.dimension) and self.scale == other.scale
  }

  toString -> String { self.symbol }
}

impl UnitLike for UnitExpr {
  expression -> UnitExpr { self }
}

impl ExactStructuralUnit for UnitExpr {
  dimension -> Dimension { self.dimension }
  scale -> Rational { self.scale }
}

export UnitExpr, UnitLike
