from universe.numbers import Addable, Multipliable, Zero, One
from ..buffer import Sized
from .vector import Vector, Point

/// Output rows of Input coefficients: an Output x Input matrix. Typed nested
/// storage avoids unchecked flat indexing and type-level extent multiplication.
data LinearMap<S, Input: Int, Output: Int, From, To> private(
  _ rows: Sized<Sized<S, Input>, Output>
)
impl<S, Input: Int, Output: Int, From, To> LinearMap<S, Input, Output, From, To> {
  type Source = From
  type Target = To
  @class fromRows(_ rows: Sized<Sized<S, Input>, Output>) -> LinearMap<S, Input, Output, From, To> {
    LinearMap<S, Input, Output, From, To>(rows)
  }
  @class tabulate(_ f: (Int, Int) -> S) -> LinearMap<S, Input, Output, From, To> {
    LinearMap<S, Input, Output, From, To>.fromRows(Sized<Sized<S, Input>, Output>.tabulate(
      |row| { Sized<S, Input>.tabulate(|column| { f.call(row, column) }) }))
  }
  @class identity -> LinearMap<S, Input, Output, From, To>
    where Input == Output, From == To, S |= Zero, S |= One
  {
    LinearMap<S, Input, Output, From, To>.tabulate(|row, column| { if row == column { S.one } else { S.zero } })
  }
  rows -> Sized<Sized<S, Input>, Output> { self.rows }
  coefficient(row: Int, column: Int) -> S { self.rows.at(row).at(column) }
  apply(_ vector: Vector<S, Input, From>) -> Vector<S, Output, To>
    where S |= Addable, S |= Multipliable, S |= Zero
  {
    Vector<S, Output, To>.from(self.rows.map(|row| {
      row.zipWith(vector.coordinates, combine: |coefficient, value| { coefficient * value }).values.fold(
        initial: S.zero, using: |sum, value| { sum + value })
    }))
  }
  /// Apply self first, then other. Both the intermediate extent and space match
  /// statically. Scalar multiplication order follows other * self.
  then<P: Int, Next>(_ other: LinearMap<S, Output, P, To, Next>) -> LinearMap<S, Input, P, From, Next>
    where S |= Addable, S |= Multipliable, S |= Zero
  {
    LinearMap<S, Input, P, From, Next>.tabulate(|row, column| {
      let mut sum = S.zero
      let mut middle = 0
      while middle < Output {
        sum = sum + other.coefficient(row: row, column: middle) * self.coefficient(row: middle, column: column)
        middle = middle + 1
      }
      sum
    })
  }
  +(_ other: LinearMap<S, Input, Output, From, To>) -> LinearMap<S, Input, Output, From, To>
    where S |= Addable
  {
    LinearMap<S, Input, Output, From, To>.fromRows(self.rows.zipWith(other.rows, combine: |left, right| {
      left.zipWith(right, combine: |a, b| { a + b })
    }))
  }
  scaled(by factor: S) -> LinearMap<S, Input, Output, From, To> where S |= Multipliable {
    LinearMap<S, Input, Output, From, To>.fromRows(self.rows.map(|row| { row.map(|value| { value * factor }) }))
  }
  /// Adjoint only for the standard Euclidean metrics on both coordinate bases.
  /// Transposition is not generally an inverse coordinate transformation.
  adjointEuclidean -> LinearMap<S, Output, Input, To, From> {
    LinearMap<S, Output, Input, To, From>.tabulate(|row, column| { self.coefficient(row: column, column: row) })
  }
}

data AffineMap<S, Input: Int, Output: Int, From, To>(
  linear: LinearMap<S, Input, Output, From, To>,
  translation: Vector<S, Output, To>
)
impl<S, Input: Int, Output: Int, From, To> AffineMap<S, Input, Output, From, To> {
  apply(_ point: Point<S, Input, From>) -> Point<S, Output, To>
    where S |= Addable, S |= Multipliable, S |= Zero
  {
    let displacement = self.linear.apply(Vector<S, Input, From>.from(point.coordinates)) + self.translation
    Point<S, Output, To>.from(displacement.coordinates)
  }
  then<P: Int, Next>(_ other: AffineMap<S, Output, P, To, Next>) -> AffineMap<S, Input, P, From, Next>
    where S |= Addable, S |= Multipliable, S |= Zero
  {
    AffineMap<S, Input, P, From, Next>(
      linear: self.linear.then(other.linear),
      translation: other.linear.apply(self.translation) + other.translation)
  }
}
export LinearMap, AffineMap
