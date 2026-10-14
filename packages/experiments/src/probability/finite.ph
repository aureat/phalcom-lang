from universe.functional.chain import Chain
from universe.numbers import Rational, Zero, One

data Outcome<T>(_ value: T, _ weight: Rational)
data FiniteError {
  ZeroMass
  NegativeWeight(_ weight: Rational)
  ZeroEvent
}

/// Nonempty, normalized, positive rational masses on equivalence classes.
/// The spine and weights are immutable. Outcomes and equality callbacks must
/// remain stable; wrapping a mutable outcome does not freeze its contents.
/// equal must be a pure equivalence relation. Encounter order picks representatives.
data FiniteDistribution<T> private(
  _ outcomes: Chain<Outcome<T>>,
  _ equal: (T, T) -> Bool
)
impl<T> FiniteDistribution<T> {
  type Value = T
  @class
  private merge(_ entries: Chain<Outcome<T>>, adding entry: Outcome<T>, equal: (T, T) -> Bool) -> Chain<Outcome<T>> {
    let mut found = false
    let merged = entries.map(|existing| {
      if equal.call(existing.value, entry.value) {
        found = true
        return Outcome<T>(existing.value, existing.weight + entry.weight)
      }
      existing
    })
    if found { return merged }
    merged.append(Chain<Outcome<T>>::Empty.prepend(entry))
  }
  @class
  fromWeights(_ entries: Chain<Outcome<T>>, equal: (T, T) -> Bool) -> Result<FiniteDistribution<T>, FiniteError> {
    let mut merged = Chain<Outcome<T>>::Empty
    let mut total = Rational.zero
    let mut cursor = entries
    while not cursor.isEmpty {
      let entry = cursor.first.unwrap
      if entry.weight < Rational.zero { return Result::Error(FiniteError::NegativeWeight(entry.weight)) }
      if entry.weight > Rational.zero {
        total = total + entry.weight
        merged = FiniteDistribution<T>.merge(merged, adding: entry, equal: equal)
      }
      cursor = cursor.tail
    }
    if total == Rational.zero { return Result::Error(FiniteError::ZeroMass) }
    let normalized = merged.map(|entry| { Outcome<T>(entry.value, entry.weight / total) })
    Result::Ok(FiniteDistribution<T>(normalized, equal))
  }
  @class
  pure(_ value: T, equal: (T, T) -> Bool) -> FiniteDistribution<T> {
    FiniteDistribution<T>(Chain<Outcome<T>>::Empty.prepend(Outcome<T>(value, Rational.one)), equal)
  }
  @class
  uniform(_ values: Chain<T>, equal: (T, T) -> Bool) -> Result<FiniteDistribution<T>, FiniteError> {
    FiniteDistribution<T>.fromWeights(values.map(|value| { Outcome<T>(value, Rational.one) }), equal: equal)
  }
  outcomes -> Chain<Outcome<T>> { self.outcomes }
  support -> Chain<T> { self.outcomes.map(|entry| { entry.value }) }
  size -> Int { self.outcomes.size }
  mass(_ value: T) -> Rational {
    self.probability(|candidate| { self.equal.call(candidate, value) })
  }
  probability(_ event: (T) -> Bool) -> Rational {
    self.outcomes.fold(initial: Rational.zero, using: |sum, entry| {
      if event.call(entry.value) { sum + entry.weight } else { sum }
    })
  }
  expectation(_ observable: (T) -> Rational) -> Rational {
    self.outcomes.fold(initial: Rational.zero, using: |sum, entry| { sum + entry.weight * observable.call(entry.value) })
  }
  variance(_ observable: (T) -> Rational) -> Rational {
    // Evaluate each observable once; callbacks may be costly.
    let values = self.outcomes.map(|entry| { Outcome<Rational>(observable.call(entry.value), entry.weight) })
    let mean = values.fold(initial: Rational.zero, using: |sum, entry| { sum + entry.weight * entry.value })
    values.fold(initial: Rational.zero, using: |sum, entry| {
      let delta = entry.value - mean
      sum + entry.weight * delta * delta
    })
  }
  /// Arbitrary deterministic transformations retain exact mass, merging output
  /// equivalence classes with an explicitly chosen output equality.
  map<U>(_ f: (T) -> U, equal: (U, U) -> Bool) -> FiniteDistribution<U> {
    let entries = self.outcomes.map(|entry| { Outcome<U>(f.call(entry.value), entry.weight) })
    // Nonempty positive input cannot produce a zero or negative total.
    FiniteDistribution<U>.fromWeights(entries, equal: equal).unwrap
  }
  /// Dependent experiments multiply conditional masses, then sum alternatives.
  flatMap<U>(_ f: (T) -> FiniteDistribution<U>, equal: (U, U) -> Bool) -> FiniteDistribution<U> {
    let entries = self.outcomes.flatMap(|outer| {
      f.call(outer.value).outcomes.map(|inner| { Outcome<U>(inner.value, outer.weight * inner.weight) })
    })
    FiniteDistribution<U>.fromWeights(entries, equal: equal).unwrap
  }
  zipWith<U, R>(_ other: FiniteDistribution<U>, combine f: (T, U) -> R, equal: (R, R) -> Bool) -> FiniteDistribution<R> {
    self.flatMap(|left| { other.map(|right| { f.call(left, right) }, equal: equal) }, equal: equal)
  }
  condition(_ event: (T) -> Bool) -> Result<FiniteDistribution<T>, FiniteError> {
    let retained = self.outcomes.filter(|entry| { event.call(entry.value) })
    if retained.isEmpty { return Result::Error(FiniteError::ZeroEvent) }
    FiniteDistribution<T>.fromWeights(retained, equal: self.equal)
  }
  /// Exact Bayesian likelihood update. Likelihoods are nonnegative weights;
  /// impossible evidence is an error, never an invented posterior.
  reweight(_ likelihood: (T) -> Rational) -> Result<FiniteDistribution<T>, FiniteError> {
    let mut entries = Chain<Outcome<T>>::Empty
    let mut cursor = self.outcomes
    while not cursor.isEmpty {
      let entry = cursor.first.unwrap
      let evidence = likelihood.call(entry.value)
      if evidence < Rational.zero { return Result::Error(FiniteError::NegativeWeight(evidence)) }
      entries = entries.prepend(Outcome<T>(entry.value, entry.weight * evidence))
      cursor = cursor.tail
    }
    FiniteDistribution<T>.fromWeights(entries.reverse, equal: self.equal)
  }
}
export Outcome, FiniteError, FiniteDistribution
