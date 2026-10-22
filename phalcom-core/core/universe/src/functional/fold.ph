//! Composable one-pass folds with fresh per-run state and typed projections.
from .chain import Chain
from universe.collections.iterator_laws import Iterable
from universe.numbers.capabilities import Addable, Zero

/// A fold with private accumulator S, input A, and output B.
/// initialize runs once per run; step once per input; finish once at completion.
/// Callback exceptions propagate. Fresh state depends on the initializer:
/// returning a shared mutable object does not create an independent copy.
data Fold<S, A, B> private(_ initialize: () -> S, _ step: (S, A) -> S, _ finish: (S) -> B)

impl<S, A, B> Fold<S, A, B> {
  @class new(initialize: () -> S, step: (S, A) -> S, finish: (S) -> B) -> Fold<S, A, B> {
    Fold<S, A, B>(initialize, step, finish)
  }

  run<I>(_ source: I) -> B where I |= Iterable, <I as Iterable>::Item == A {
    let mut state = self.initialize.call()
    let iterator = source.iterator
    while let Option::Some(value) = iterator.next {
      state = self.step.call(state, value)
    }
    self.finish.call(state)
  }

  map<U>(_ f: (B) -> U) -> Fold<S, A, U> {
    Fold<S, A, U>.new(initialize: self.initialize, step: self.step, finish: |state| { f.call(self.finish.call(state)) })
  }

  contramap<I>(_ f: (I) -> A) -> Fold<S, I, B> {
    Fold<S, I, B>.new(
      initialize: self.initialize,
      step: |state, value| { self.step.call(state, f.call(value)) },
      finish: self.finish
    )
  }

  /// Transforms the input and output types while retaining accumulator S.
  dimap<I, U>(before: (I) -> A, after: (B) -> U) -> Fold<S, I, U> {
    self.contramap(before).map(after)
  }

  /// Filters inputs before stepping. Still consumes the whole source.
  filter(_ predicate: (A) -> Bool) -> Fold<S, A, B> {
    Fold<S, A, B>.new(
      initialize: self.initialize,
      step: |state, value| { if predicate.call(value) { self.step.call(state, value) } else { state } },
      finish: self.finish
    )
  }

  /// Runs two folds in a SINGLE traversal. Initialization, each paired step,
  /// and finishing occur left to right. No input sequence is materialized.
  zipWith<Q, U, R>(_ other: Fold<Q, A, U>, combine: (B, U) -> R) -> Fold<Tuple<***(S, Q)>, A, R> {
    Fold<Tuple<***(S, Q)>, A, R>.new(
      initialize: || {
        let left = self.initialize.call()
        let right = other.initialize.call()
        (left, right)
      },
      step: |state, value| {
        let left = self.step.call(state[0], value)
        let right = other.step.call(state[1], value)
        (left, right)
      },
      finish: |state| {
        let left = self.finish.call(state[0])
        let right = other.finish.call(state[1])
        combine.call(left, right)
      }
    )
  }
}

class FoldFactory {
  @class count<A>() -> Fold<Int, A, Int> {
    Fold<Int, A, Int>.new(initialize: || { 0 }, step: |count, _| { count + 1 }, finish: |count| { count })
  }
  @class collect<A>() -> Fold<Chain<A>, A, Chain<A>> {
    Fold<Chain<A>, A, Chain<A>>.new(
      initialize: || { Chain<A>::Empty },
      step: |values, value| { values.prepend(value) },
      finish: |values| { values.reverse }
    )
  }
  @class sum<A>() -> Fold<A, A, A> where A |= Addable, A |= Zero {
    Fold<A, A, A>.new(initialize: || { A.zero }, step: |total, value| { total + value }, finish: |total| { total })
  }
  /// Skips predicate calls after the first true value; traversal still completes.
  @class any<A>(_ predicate: (A) -> Bool) -> Fold<Bool, A, Bool> {
    Fold<Bool, A, Bool>.new(
      initialize: || { false },
      step: |found, value| { found or predicate.call(value) },
      finish: |found| { found }
    )
  }
  /// Skips predicate calls after the first false value; traversal still completes.
  @class all<A>(_ predicate: (A) -> Bool) -> Fold<Bool, A, Bool> {
    Fold<Bool, A, Bool>.new(
      initialize: || { true },
      step: |valid, value| { valid and predicate.call(value) },
      finish: |valid| { valid }
    )
  }
  @class first<A>() -> Fold<Option<A>, A, Option<A>> {
    Fold<Option<A>, A, Option<A>>.new(
      initialize: || { Option<A>::None },
      step: |current, value| { match current { Option::Some(_) => current; Option::None => Option<A>::Some(value) } },
      finish: |current| { current }
    )
  }
  @class last<A>() -> Fold<Option<A>, A, Option<A>> {
    Fold<Option<A>, A, Option<A>>.new(
      initialize: || { Option<A>::None },
      step: |_, value| { Option<A>::Some(value) },
      finish: |current| { current }
    )
  }
}

export Fold, FoldFactory
