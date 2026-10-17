from universe.numbers import Rational, Zero, One
from .laws import Semiring

data Reachability(_ value: Bool)
impl Semiring for Reachability {
  @class zero -> Reachability { Reachability(false) }
  @class one -> Reachability { Reachability(true) }
  alternative(_ other: Reachability) -> Reachability { Reachability(self.value or other.value) }
  sequential(_ other: Reachability) -> Reachability { Reachability(self.value and other.value) }
}

/// Arbitrary-precision nonnegative path multiplicity.
data Count private(_ value: Int)
impl Count {
  @class new(_ value: Int) -> Count {
    if value < 0 { ArgumentError.new("Count must be nonnegative").raise() }
    Count(value)
  }
  value -> Int { self.value }
}
impl Semiring for Count {
  @class zero -> Count { Count(0) }
  @class one -> Count { Count(1) }
  alternative(_ other: Count) -> Count { Count(self.value + other.value) }
  sequential(_ other: Count) -> Count { Count(self.value * other.value) }
}

/// Exact nonnegative weight. Values may exceed one: weights form a semiring,
/// while normalized probabilities do not form a closed additive semiring.
data Weight private(_ value: Rational)
impl Weight {
  @class new(_ value: Rational) -> Weight {
    if value < Rational.zero { ArgumentError.new("Weight must be nonnegative").raise() }
    Weight(value)
  }
  value -> Rational { self.value }
}
impl Semiring for Weight {
  @class zero -> Weight { Weight(Rational.zero) }
  @class one -> Weight { Weight(Rational.one) }
  alternative(_ other: Weight) -> Weight { Weight(self.value + other.value) }
  sequential(_ other: Weight) -> Weight { Weight(self.value * other.value) }
}

/// Min-plus with exact integer costs and an explicit positive infinity.
/// Negative finite costs are permitted; bounded walks do not solve negative cycles.
data Tropical {
  Infinity
  Cost(_ value: Int)
}
impl Semiring for Tropical {
  @class zero -> Tropical { Tropical::Infinity }
  @class one -> Tropical { Tropical::Cost(0) }
  alternative(_ other: Tropical) -> Tropical {
    match self {
      Tropical::Infinity => other
      Tropical::Cost(left) => {
        match other {
          Tropical::Infinity => self
          Tropical::Cost(right) => if left <= right { self } else { other }
        }
      }
    }
  }
  sequential(_ other: Tropical) -> Tropical {
    match self {
      Tropical::Infinity => Tropical::Infinity
      Tropical::Cost(left) => {
        match other {
          Tropical::Infinity => Tropical::Infinity
          Tropical::Cost(right) => Tropical::Cost(left + right)
        }
      }
    }
  }
}
export Reachability, Count, Weight, Tropical
