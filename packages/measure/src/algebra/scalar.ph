//! Package-local scalar capabilities used by normalized quantity arithmetic.

from ..exact.rational import Rational

trait QuantityScalar {
  addScalar(_ other: Self) -> Self
  subtractScalar(_ other: Self) -> Self
  multiplyScalar(_ other: Self) -> Self
  divideScalar(_ other: Self) -> Self
  negateScalar -> Self
  compareScalar(_ other: Self) -> Ordering
}

impl QuantityScalar for Rational {
  addScalar(_ other: Rational) -> Rational { self + other }
  subtractScalar(_ other: Rational) -> Rational { self - other }
  multiplyScalar(_ other: Rational) -> Rational { self * other }
  divideScalar(_ other: Rational) -> Rational { self / other }
  negateScalar -> Rational { -self }
  compareScalar(_ other: Rational) -> Ordering { self.compare(other) }
}

impl QuantityScalar for Float {
  addScalar(_ other: Float) -> Float { self + other }
  subtractScalar(_ other: Float) -> Float { self - other }
  multiplyScalar(_ other: Float) -> Float { self * other }
  divideScalar(_ other: Float) -> Float { self / other }
  negateScalar -> Float { -self }
  compareScalar(_ other: Float) -> Ordering { self.compare(other) }
}

export QuantityScalar
