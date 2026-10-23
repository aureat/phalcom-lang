//! Shared typed environments without global configuration or hidden storage.

/// A deferred computation that reads an environment R and produces T.
/// Composition preserves the environment. Each run invokes the computation
/// anew; construction never invokes it and there is no memoization.
data Reader<R, T> private(_ read: (R) -> T)

impl<R, T> Reader<R, T> {
  @class
  new(_ read: (R) -> T) -> Reader<R, T> { Reader<R, T>(read) }

  /// Produces a constant value for any environment.
  @class
  pure(_ value: T) -> Reader<R, T> { Reader<R, T>.new(|_| { value }) }

  /// Produces the environment itself. Available on Reader<R, R>.
  @class
  ask() -> Reader<R, T> where T == R { Reader<R, R>.new(|environment| { environment }) }

  /// Runs this computation with the supplied environment.
  run(_ environment: R) -> T { self.read.call(environment) }

  /// When run, transforms the produced value and retains the same environment.
  map<U>(_ f: (T) -> U) -> Reader<R, U> {
    Reader<R, U>.new(|environment| { f.call(self.run(environment)) })
  }

  /// Runs this computation, then runs f(value) in the SAME environment.
  /// The first computation, f, and the selected computation run once each.
  flatMap<U>(_ f: (T) -> Reader<R, U>) -> Reader<R, U> {
    Reader<R, U>.new(|environment| { f.call(self.run(environment)).run(environment) })
  }

  /// Runs both computations left to right in the same environment.
  zipWith<U, V>(_ other: Reader<R, U>, combine: (T, U) -> V) -> Reader<R, V> {
    Reader<R, V>.new(|environment| {
      let left = self.run(environment)
      let right = other.run(environment)
      combine.call(left, right)
    })
  }

  /// Changes the environment seen by this computation for one run.
  /// The adapter runs once; the caller's environment binding is unchanged.
  local(_ f: (R) -> R) -> Reader<R, T> {
    Reader<R, T>.new(|environment| { self.run(f.call(environment)) })
  }

  /// Adapts a different environment type into the environment this needs.
  contramap<Q>(_ f: (Q) -> R) -> Reader<Q, T> {
    Reader<Q, T>.new(|environment| { self.run(f.call(environment)) })
  }
}

export Reader
