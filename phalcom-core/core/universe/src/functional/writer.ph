//! Values with typed accumulated logs; no global output sink.
from .algebra import Semigroup, Monoid

/// A value and a log. Combining logs requires only Semigroup; pure needs Monoid.
data Writer<W, T>(value: T, log: W)

impl<W, T> Writer<W, T> {
  type Value = T
  type Log = W

  map<U>(_ f: (T) -> U) -> Writer<W, U> {
    Writer<W, U>(value: f.call(self.value), log: self.log)
  }

  mapLog<V>(_ f: (W) -> V) -> Writer<V, T> {
    Writer<V, T>(value: self.value, log: f.call(self.log))
  }

  /// Observes the complete log without discarding it.
  listen -> Writer<W, Tuple<***(T, W)>> {
    Writer<W, Tuple<***(T, W)>>(value: (self.value, self.log), log: self.log)
  }

  /// Replaces the output log after the computation, preserving the value.
  censor(_ f: (W) -> W) -> Writer<W, T> { self.mapLog(f) }
}

impl<W, T> Writer<W, T> where W |= Semigroup {
  /// Logs combine left to right; f runs exactly once.
  flatMap<U>(_ f: (T) -> Writer<W, U>) -> Writer<W, U> {
    let next = f.call(self.value)
    Writer<W, U>(value: next.value, log: self.log.combine(next.log))
  }
  zipWith<U, R>(_ other: Writer<W, U>, combine: (T, U) -> R) -> Writer<W, R> {
    let value = combine.call(self.value, other.value)
    Writer<W, R>(value: value, log: self.log.combine(other.log))
  }
}

impl<W, T> Writer<W, T> where W |= Semigroup, W |= Monoid {
  @class pure(_ value: T) -> Writer<W, T> { Writer<W, T>(value: value, log: W.empty) }
  @class tell(_ log: W) -> Writer<W, T> where T == Unit {
    Writer<W, Unit>(value: (), log: log)
  }
}

export Writer
