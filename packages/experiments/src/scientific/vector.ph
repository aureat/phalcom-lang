from universe.numbers import Addable, Subtractable, Multipliable, Zero
from ..buffer import Sized

/// N is a static extent; Space is a nominal coordinate-space tag. Equal extents
/// alone do not make world, camera, and screen vectors interchangeable.
data Vector<S, N: Int, Space> private(_ coordinates: Sized<S, N>)
impl<S, N: Int, Space> Vector<S, N, Space> {
  type Scalar = S
  type CoordinateSpace = Space
  @class from(_ coordinates: Sized<S, N>) -> Vector<S, N, Space> { Vector<S, N, Space>(coordinates) }
  @class tabulate(_ f: (Int) -> S) -> Vector<S, N, Space> { Vector<S, N, Space>(Sized<S, N>.tabulate(f)) }
  @class zero -> Vector<S, N, Space> where S |= Zero { Vector<S, N, Space>.tabulate(|_| { S.zero }) }
  coordinates -> Sized<S, N> { self.coordinates }
  at(_ index: Int) -> S { self.coordinates.at(index) }
  +(_ other: Vector<S, N, Space>) -> Vector<S, N, Space> where S |= Addable {
    Vector<S, N, Space>.from(self.coordinates.zipWith(other.coordinates, combine: |left, right| { left + right }))
  }
  -(_ other: Vector<S, N, Space>) -> Vector<S, N, Space> where S |= Subtractable {
    Vector<S, N, Space>.from(self.coordinates.zipWith(other.coordinates, combine: |left, right| { left - right }))
  }
  scaled(by factor: S) -> Vector<S, N, Space> where S |= Multipliable {
    Vector<S, N, Space>.from(self.coordinates.map(|value| { value * factor }))
  }
  /// Componentwise transform preserves the coordinate space and extent.
  map<T>(_ f: (S) -> T) -> Vector<T, N, Space> { Vector<T, N, Space>.from(self.coordinates.map(f)) }
  /// Coordinate dot product assumes the standard Euclidean metric in this basis.
  /// It is not a metric inferred from the space tag or from physical dimensions.
  dotEuclidean(_ other: Vector<S, N, Space>) -> S
    where S |= Addable, S |= Multipliable, S |= Zero
  {
    self.coordinates.zipWith(other.coordinates, combine: |left, right| { left * right }).values.fold(
      initial: S.zero, using: |sum, value| { sum + value })
  }
}

/// Affine positions are distinct from displacement vectors. There is no point
/// addition or point scaling: subtract positions, then translate by a vector.
data Point<S, N: Int, Space> private(_ coordinates: Sized<S, N>)
impl<S, N: Int, Space> Point<S, N, Space> {
  @class from(_ coordinates: Sized<S, N>) -> Point<S, N, Space> { Point<S, N, Space>(coordinates) }
  coordinates -> Sized<S, N> { self.coordinates }
  -(_ other: Point<S, N, Space>) -> Vector<S, N, Space> where S |= Subtractable {
    Vector<S, N, Space>.from(self.coordinates.zipWith(other.coordinates, combine: |left, right| { left - right }))
  }
  translated(by displacement: Vector<S, N, Space>) -> Point<S, N, Space> where S |= Addable {
    Point<S, N, Space>.from(self.coordinates.zipWith(displacement.coordinates, combine: |left, right| { left + right }))
  }
}
export Vector, Point
