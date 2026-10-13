from universe.random import Generator, Distribution
from universe.functional.chain import Chain
from ..buffer import Sized

data SamplingError(_ attempts: Int)

/// Deferred sampling specialized to an explicit generator type. The caller owns
/// the mutable generator; every run advances that same object. Composition never
/// draws, copies, resets, or implicitly forks it. No density capability is derived.
data Sample<G, T> private(_ draw: (G) -> T) where G |= Generator
impl<G, T> Sample<G, T> where G |= Generator {
  type Output = T
  type RandomSource = G
  @class new(_ draw: (G) -> T) -> Sample<G, T> { Sample<G, T>(draw) }
  @class pure(_ value: T) -> Sample<G, T> { Sample<G, T>.new(|_| { value }) }
  @class from<D>(_ distribution: D) -> Sample<G, T>
    where D |= Distribution, <D as Distribution>::Output == T
  { Sample<G, T>.new(|generator| { distribution.sample(using: generator) }) }
  @class uniform -> Sample<G, T> where T == Float {
    Sample<G, T>.new(|generator| { generator.nextFloat })
  }
  @class integer(from lower: Int, below upper: Int) -> Sample<G, T> where T == Int {
    if lower >= upper { ArgumentError.new("Sample.integer: bounds must be ordered").raise() }
    Sample<G, T>.new(|generator| { generator.nextInt(from: lower, below: upper) })
  }
  run(using generator: G) -> T { self.draw.call(generator) }
  map<U>(_ f: (T) -> U) -> Sample<G, U> {
    Sample<G, U>.new(|generator| { f.call(self.run(using: generator)) })
  }
  flatMap<U>(_ f: (T) -> Sample<G, U>) -> Sample<G, U> {
    Sample<G, U>.new(|generator| { f.call(self.run(using: generator)).run(using: generator) })
  }
  zipWith<U, R>(_ other: Sample<G, U>, combine f: (T, U) -> R) -> Sample<G, R> {
    Sample<G, R>.new(|generator| {
      let left = self.run(using: generator)
      let right = other.run(using: generator)
      f.call(left, right)
    })
  }
  paired<U>(_ other: Sample<G, U>) -> Sample<G, (T, U)> {
    self.zipWith(other, combine: |left, right| { (left, right) })
  }
  then<U>(_ other: Sample<G, U>) -> Sample<G, U> {
    self.flatMap(|_| { other })
  }
  replicate(_ count: Int) -> Sample<G, Chain<T>> {
    if count < 0 { ArgumentError.new("Sample.replicate: count must be nonnegative").raise() }
    Sample<G, Chain<T>>.new(|generator| {
      let mut values = Chain<T>::Empty
      let mut index = 0
      while index < count {
        values = values.prepend(self.run(using: generator))
        index = index + 1
      }
      values.reverse
    })
  }
  sized<N: Int>() -> Sample<G, Sized<T, N>> {
    if N < 0 { ArgumentError.new("Sample.sized: extent must be nonnegative").raise() }
    self.replicate(N).map(|values| { Sized<T, N>.from(values) })
  }
  /// Bounded rejection: failure consumes the attempted draws and does not
  /// rewind the generator. A zero budget fails without drawing.
  filter(_ predicate: (T) -> Bool, attempts budget: Int) -> Sample<G, Result<T, SamplingError>> {
    if budget < 0 { ArgumentError.new("Sample.filter: budget must be nonnegative").raise() }
    Sample<G, Result<T, SamplingError>>.new(|generator| {
      let mut attempted = 0
      while attempted < budget {
        let value = self.run(using: generator)
        attempted = attempted + 1
        if predicate.call(value) { return Result::Ok(value) }
      }
      Result::Error(SamplingError(attempted))
    })
  }
  /// Traversal preserves experiment and generator-consumption order.
  @class sequence(_ programs: Chain<Sample<G, T>>) -> Sample<G, Chain<T>> {
    Sample<G, Chain<T>>.new(|generator| { programs.map(|program| { program.run(using: generator) }) })
  }
}
export Sample, SamplingError
