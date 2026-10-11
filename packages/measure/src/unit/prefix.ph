from ..algebra.kind import QuantityKind
from ..exact.rational import Rational
from ..errors import UnitDefinitionError
from .linear import LinearUnit

/// Exact scale prefix such as kilo, milli, or micro.
///
/// A Prefix is immutable and statically constructible. Calling it with a
/// LinearUnit preserves the unit's semantic kind and composes the exact scale.
data Prefix(
  _ symbol: String,
  _ scale: Rational
)

impl Prefix {
  @class
  define(_ symbol: String, scale: Rational) -> Prefix {
    if scale <= Rational.zero {
      UnitDefinitionError.new("prefix scale must be positive").raise()
    }
    Prefix(symbol, scale)
  }

  symbol -> String { self.symbol }
  scale -> Rational { self.scale }

  call<K>(_ unit: LinearUnit<K>) -> LinearUnit<K>
    where K |= QuantityKind
  {
    unit.scaled(
      by: self.scale,
      symbol: "\(self.symbol)\(unit.symbol)"
    )
  }
}

export Prefix
