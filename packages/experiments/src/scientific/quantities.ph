// The established measurement package owns dimensions, units, affine points,
// exact conversion, and semantic quantity-kind multiplication. Re-export its
// contracts instead of introducing a second dimension algebra.
from measure.quantity.quantity import Quantity
from measure.quantity.point import Point as QuantityPoint
from measure.algebra.kind import QuantityKind, ProductKind, QuotientKind, PointKind
from measure.algebra.scalar import QuantityScalar
from measure.exact.rational import Rational
from .vector import Vector

/// Vector addition remains kind-safe. A Quantity<K, S> cannot be added to a
/// Quantity<L, S> through this adapter unless K and L are the same type.
class QuantityVectors {
  @class
  add<K, S, N: Int, Space>(_ left: Vector<Quantity<K, S>, N, Space>, _ right: Vector<Quantity<K, S>, N, Space>) -> Vector<Quantity<K, S>, N, Space>
    where K |= QuantityKind, S |= QuantityScalar
  {
    Vector<Quantity<K, S>, N, Space>.from(left.coordinates.zipWith(right.coordinates, combine: |a, b| { a + b }))
  }
  @class
  scaled<K, S, N: Int, Space>(_ vector: Vector<Quantity<K, S>, N, Space>, by factor: S) -> Vector<Quantity<K, S>, N, Space>
    where K |= QuantityKind, S |= QuantityScalar
  {
    vector.map(|value| { value.scaled(by: factor) })
  }
  /// Dimensional multiplication follows measure.ProductKind, including its
  /// runtime semantic-law check. Coordinate-space identity is preserved.
  @class
  multiplied<K, L, S, N: Int, Space>(_ vector: Vector<Quantity<K, S>, N, Space>, by factor: Quantity<L, S>) -> Vector<Quantity<<K as ProductKind<L>>::Output, S>, N, Space>
    where K |= QuantityKind, L |= QuantityKind, K |= ProductKind<L>,
      <K as ProductKind<L>>::Output |= QuantityKind, S |= QuantityScalar
  {
    vector.map(|value| { value * factor })
  }
}
export Quantity, QuantityPoint, QuantityKind, ProductKind, QuotientKind, PointKind, QuantityScalar, Rational, QuantityVectors
