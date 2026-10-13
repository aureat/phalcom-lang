//! Deferred environment access combined with fail-fast recoverable errors.
from .reader import Reader

data ReaderResult<R, T, E> private(_ read: (R) -> Result<T, E>)
impl<R, T, E> ReaderResult<R, T, E> {
  type Environment = R
  type Value = T
  type ErrorType = E

  @class new(_ read: (R) -> Result<T, E>) -> ReaderResult<R, T, E> { ReaderResult<R, T, E>(read) }
  @class pure(_ value: T) -> ReaderResult<R, T, E> {
    ReaderResult<R, T, E>.new(|_| { Result<T, E>::Ok(value) })
  }
  @class fail(_ error: E) -> ReaderResult<R, T, E> {
    ReaderResult<R, T, E>.new(|_| { Result<T, E>::Error(error) })
  }
  @class fromReader(_ computation: Reader<R, T>) -> ReaderResult<R, T, E> {
    ReaderResult<R, T, E>.new(|environment| { Result<T, E>::Ok(computation.run(environment)) })
  }
  run(_ environment: R) -> Result<T, E> { self.read.call(environment) }
  toReader -> Reader<R, Result<T, E>> { Reader<R, Result<T, E>>.new(self.read) }
  map<U>(_ f: (T) -> U) -> ReaderResult<R, U, E> {
    ReaderResult<R, U, E>.new(|environment| { self.run(environment).map(f) })
  }
  mapErr<F>(_ f: (E) -> F) -> ReaderResult<R, T, F> {
    ReaderResult<R, T, F>.new(|environment| { self.run(environment).mapErr(f) })
  }
  flatMap<U>(_ f: (T) -> ReaderResult<R, U, E>) -> ReaderResult<R, U, E> {
    ReaderResult<R, U, E>.new(|environment| {
      match self.run(environment) {
        Result::Ok(value) => f.call(value).run(environment)
        Result::Error(error) => Result<U, E>::Error(error)
      }
    })
  }
  orElse<F>(_ f: (E) -> ReaderResult<R, T, F>) -> ReaderResult<R, T, F> {
    ReaderResult<R, T, F>.new(|environment| {
      match self.run(environment) {
        Result::Ok(value) => Result<T, F>::Ok(value)
        Result::Error(error) => f.call(error).run(environment)
      }
    })
  }
  zipWith<U, V>(_ other: ReaderResult<R, U, E>, combine: (T, U) -> V) -> ReaderResult<R, V, E> {
    self.flatMap(|left| { other.map(|right| { combine.call(left, right) }) })
  }
  local(_ f: (R) -> R) -> ReaderResult<R, T, E> {
    ReaderResult<R, T, E>.new(|environment| { self.run(f.call(environment)) })
  }
  contramap<Q>(_ f: (Q) -> R) -> ReaderResult<Q, T, E> {
    ReaderResult<Q, T, E>.new(|environment| { self.run(f.call(environment)) })
  }
}

export ReaderResult
