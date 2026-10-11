from ..algebra.kind import QuantityKind
from ..algebra.unit import FloatUnitFor
from ..quantity.quantity import Quantity
from ..errors import UnitDefinitionError

/// Approximate linear unit for an intrinsically inexact scale.
///
/// Approximate units deliberately do not conform to `UnitLike`; exact
/// structural unit composition stays Rational-backed.
data ApproximateUnit<K> private(
  _ kind: K,
  _ symbol: String,
  _ scale: Float
)
where K |= QuantityKind

impl<K> ApproximateUnit<K>
  where K |= QuantityKind
{
  @class
  define(_ kind: K, _ symbol: String, scale: Float) -> ApproximateUnit<K> {
    if not scale.isFinite or scale <= 0.0 {
      UnitDefinitionError.new("approximate unit scale must be finite and positive").raise()
    }
    ApproximateUnit(kind, symbol, scale)
  }

  call(_ magnitude: Float) -> Quantity<K, Float> {
    Quantity.reference(self.kind, magnitude * self.scale)
  }

  symbol -> String { self.symbol }

}

impl<K> FloatUnitFor<K> for ApproximateUnit<K>
  where K |= QuantityKind
{
}

export ApproximateUnit
