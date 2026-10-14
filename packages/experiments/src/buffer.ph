from universe.functional.chain import Chain

/// Persistent storage with a type-level extent. Construction checks the extent
/// once; consumers cannot mutate the spine or manufacture an unchecked buffer.
data Sized<T, N: Int> private(_ values: Chain<T>)
impl<T, N: Int> Sized<T, N> {
  @class
  from(_ values: Chain<T>) -> Sized<T, N> {
    if N < 0 or values.size != N {
      ArgumentError.new("Sized: values must have the declared nonnegative extent").raise()
    }
    Sized<T, N>(values)
  }
  @class
  tabulate(_ f: (Int) -> T) -> Sized<T, N> {
    if N < 0 { ArgumentError.new("Sized: extent must be nonnegative").raise() }
    let mut values = Chain<T>::Empty
    let mut index = 0
    while index < N {
      values = values.prepend(f.call(index))
      index = index + 1
    }
    Sized<T, N>(values.reverse)
  }
  values -> Chain<T> { self.values }
  size -> Int { N }
  at(_ index: Int) -> T {
    match self.values.at(index) {
      Option::Some(value) => value
      Option::None => { ArgumentError.new("Sized.at: index outside extent").raise() }
    }
  }
  map<U>(_ f: (T) -> U) -> Sized<U, N> { Sized<U, N>.from(self.values.map(f)) }
  zipWith<U, R>(_ other: Sized<U, N>, combine f: (T, U) -> R) -> Sized<R, N> {
    let mut left = self.values
    let mut right = other.values
    let mut result = Chain<R>::Empty
    while not left.isEmpty {
      result = result.prepend(f.call(left.first.unwrap, right.first.unwrap))
      left = left.tail
      right = right.tail
    }
    Sized<R, N>.from(result.reverse)
  }
}
export Sized
