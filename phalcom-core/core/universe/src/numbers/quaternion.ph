from .capabilities import Addable, Subtractable, Multipliable, Negatable, Zero, One
from .algebra import Basis, I, J, K, BasisRealizable, RealEmbeddable
from .complex import Complex

/// Immutable Hamilton quaternion with four components of one scalar type.
data Quaternion<T>(_ real: T, _ i: T, _ j: T, _ k: T)

impl<T> Quaternion<T> {
  @class
  new<C>(_ real: C, _ i: C, _ j: C, _ k: C) -> Quaternion<C> {
    Quaternion<C>(real, i, j, k)
  }

  complexPart -> Complex<T> { Complex<T>(self.real, self.i) }
}

impl<T> Zero for Quaternion<T> where T |= Zero {
  @class zero -> Quaternion<T> {
    Quaternion<T>(T.zero, T.zero, T.zero, T.zero)
  }
}

impl<T> One for Quaternion<T> where T |= Zero, T |= One {
  @class one -> Quaternion<T> {
    Quaternion<T>(T.one, T.zero, T.zero, T.zero)
  }
}

impl<T> RealEmbeddable for Quaternion<T> where T |= Zero {
  type Scalar = T

  @class fromReal(_ value: T) -> Quaternion<T> {
    Quaternion<T>(value, T.zero, T.zero, T.zero)
  }
}

impl<T> BasisRealizable<Basis<I>> for Quaternion<T> where T |= Zero, T |= One {
  type Scalar = T

  @class along(_ coefficient: T, _ basis: Basis<I>) -> Quaternion<T> {
    Quaternion<T>(T.zero, coefficient, T.zero, T.zero)
  }
}

impl<T> BasisRealizable<Basis<J>> for Quaternion<T> where T |= Zero, T |= One {
  type Scalar = T

  @class along(_ coefficient: T, _ basis: Basis<J>) -> Quaternion<T> {
    Quaternion<T>(T.zero, T.zero, coefficient, T.zero)
  }
}

impl<T> BasisRealizable<Basis<K>> for Quaternion<T> where T |= Zero, T |= One {
  type Scalar = T

  @class along(_ coefficient: T, _ basis: Basis<K>) -> Quaternion<T> {
    Quaternion<T>(T.zero, T.zero, T.zero, coefficient)
  }
}

impl<T> Quaternion<T> where T |= Addable {
  +(_ other: Quaternion<T>) -> Quaternion<T> {
    Quaternion<T>(self.real + other.real,
                  self.i + other.i,
                  self.j + other.j,
                  self.k + other.k)
  }
}

impl<T> Quaternion<T> where T |= Subtractable {
  -(_ other: Quaternion<T>) -> Quaternion<T> {
    Quaternion<T>(self.real - other.real,
                  self.i - other.i,
                  self.j - other.j,
                  self.k - other.k)
  }
}

impl<T> Quaternion<T> where T |= Negatable {
  - -> Quaternion<T> { Quaternion<T>(-self.real, -self.i, -self.j, -self.k) }
  conjugate -> Quaternion<T> { Quaternion<T>(self.real, -self.i, -self.j, -self.k) }
}

impl<T> Quaternion<T> where T |= Addable, T |= Multipliable {
  normSquared -> T {
    self.real * self.real + self.i * self.i + self.j * self.j + self.k * self.k
  }
}

impl<T> Quaternion<T> where T |= Addable, T |= Subtractable, T |= Multipliable {
  *(_ other: Quaternion<T>) -> Quaternion<T> {
    Quaternion<T>(
      self.real * other.real - self.i * other.i - self.j * other.j - self.k * other.k,
      self.real * other.i + self.i * other.real + self.j * other.k - self.k * other.j,
      self.real * other.j - self.i * other.k + self.j * other.real + self.k * other.i,
      self.real * other.k + self.i * other.j - self.j * other.i + self.k * other.real)
  }
}

export Quaternion
