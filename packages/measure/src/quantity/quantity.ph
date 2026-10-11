from ..algebra.kind import QuantityKind, ProductKind, QuotientKind
from ..algebra.scalar import QuantityScalar
from ..algebra.unit import ExactUnitFor, FloatUnitFor
from ..exact.rational import Rational
from ..errors import InvalidSemanticLawError

/// Unit-normalized semantic quantity.
///
/// `referenceMagnitude` is already expressed in the reference scale of `K`.
/// Source units and prefixes are eliminated at construction.
data Quantity<K, N> private(
  _ kind: K,
  _ referenceMagnitude: N
)
where K |= QuantityKind

impl<K, N> Quantity<K, N>
  where K |= QuantityKind
{
  @class
  reference(_ kind: K, _ magnitude: N) -> Quantity<K, N> {
    Quantity(kind, magnitude)
  }

  kind -> K { self.kind }
  magnitude -> N { self.referenceMagnitude }

  +(_ other: Quantity<K, N>) -> Quantity<K, N>
    where N |= QuantityScalar
  {
    Quantity.reference(
      self.kind,
      self.referenceMagnitude.addScalar(other.referenceMagnitude)
    )
  }

  -(_ other: Quantity<K, N>) -> Quantity<K, N>
    where N |= QuantityScalar
  {
    Quantity.reference(
      self.kind,
      self.referenceMagnitude.subtractScalar(other.referenceMagnitude)
    )
  }

  - -> Quantity<K, N>
    where N |= QuantityScalar
  {
    Quantity.reference(self.kind, self.referenceMagnitude.negateScalar)
  }

  /// Scales a quantity without changing semantic kind.
  ///
  /// Named rather than `*` because `*(_)` is already the semantic quantity
  /// product selector and Telos does not overload by RHS parameter type.
  scaled(by scalar: N) -> Quantity<K, N>
    where N |= QuantityScalar
  {
    Quantity.reference(
      self.kind,
      self.referenceMagnitude.multiplyScalar(scalar)
    )
  }

  divided(by scalar: N) -> Quantity<K, N>
    where N |= QuantityScalar
  {
    Quantity.reference(
      self.kind,
      self.referenceMagnitude.divideScalar(scalar)
    )
  }

  /// Dimensionless numeric ratio of two quantities with the same semantic kind.
  ratio(to other: Quantity<K, N>) -> N
    where N |= QuantityScalar
  {
    self.referenceMagnitude.divideScalar(other.referenceMagnitude)
  }

  compare(_ other: Quantity<K, N>) -> Ordering
    where N |= QuantityScalar
  {
    self.referenceMagnitude.compareScalar(other.referenceMagnitude)
  }

  <(_ other: Quantity<K, N>) -> Bool
    where N |= QuantityScalar
  { self.compare(other) === Ordering::Less }

  <=(_ other: Quantity<K, N>) -> Bool
    where N |= QuantityScalar
  {
    let order = self.compare(other)
    order === Ordering::Less or order === Ordering::Equal
  }

  >(_ other: Quantity<K, N>) -> Bool
    where N |= QuantityScalar
  { self.compare(other) === Ordering::Greater }

  >=(_ other: Quantity<K, N>) -> Bool
    where N |= QuantityScalar
  {
    let order = self.compare(other)
    order === Ordering::Greater or order === Ordering::Equal
  }

  *<R>(_ other: Quantity<R, N>)
    -> Quantity<<K as ProductKind<R>>::Output, N>
    where
      R |= QuantityKind,
      K |= ProductKind<R>,
      <K as ProductKind<R>>::Output |= QuantityKind,
      N |= QuantityScalar
  {
    let outputKind = self.kind.productKind(other.kind)
    let expected = self.kind.dimension * other.kind.dimension
    if not outputKind.dimension.equivalent(expected) {
      InvalidSemanticLawError.new("ProductKind result dimension does not match operand product").raise()
    }
    Quantity.reference(
      outputKind,
      self.referenceMagnitude.multiplyScalar(other.referenceMagnitude)
    )
  }

  /<R>(_ other: Quantity<R, N>)
    -> Quantity<<K as QuotientKind<R>>::Output, N>
    where
      R |= QuantityKind,
      K |= QuotientKind<R>,
      <K as QuotientKind<R>>::Output |= QuantityKind,
      N |= QuantityScalar
  {
    let outputKind = self.kind.quotientKind(other.kind)
    let expected = self.kind.dimension / other.kind.dimension
    if not outputKind.dimension.equivalent(expected) {
      InvalidSemanticLawError.new("QuotientKind result dimension does not match operand quotient").raise()
    }
    Quantity.reference(
      outputKind,
      self.referenceMagnitude.divideScalar(other.referenceMagnitude)
    )
  }

  /// Expresses an exact Rational quantity in a compatible exact unit.
  in<U>(_ unit: U) -> Rational
    where N == Rational, U |= ExactUnitFor<K>
  {
    unit.fromReference(self.referenceMagnitude)
  }

  /// Expresses a Float quantity in a compatible exact or approximate unit.
  inApprox<U>(_ unit: U) -> Float
    where N == Float, U |= FloatUnitFor<K>
  {
    unit.fromReferenceFloat(self.referenceMagnitude)
  }

  toString -> String {
    "\(self.referenceMagnitude) \(self.kind.name)"
  }
}

export Quantity
