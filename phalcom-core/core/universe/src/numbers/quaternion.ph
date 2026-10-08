from .capabilities import Addable, Subtractable, Multipliable, Negatable
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
