//! Explicit state threading for parsers, counters, and deterministic workflows.

from .lens import Lens

/// The result of one transition, with the produced value and successor state.
/// Component bindings are immutable; contained objects may themselves be mutable.
data StateStep<S, T>(state: S, value: T)

/// A deferred transition from S to a value T and a successor S.
/// Creating or composing a State never executes its transition. Each run
/// executes it anew; results are not memoized. There is no internal state cell.
data State<S, T> private(_ transition: (S) -> StateStep<S, T>)

impl<S, T> State<S, T> {
  @class
  new(_ transition: (S) -> StateStep<S, T>) -> State<S, T> {
    State<S, T>(transition)
  }

  /// Produces a value without changing the supplied state.
  @class
  pure(_ value: T) -> State<S, T> {
    State<S, T>.new(|state| { StateStep<S, T>(state: state, value: value) })
  }

  /// Reads the supplied state as the produced value.
  @class
  get() -> State<S, T> where T == S {
    State<S, S>.new(|state| { StateStep<S, S>(state: state, value: state) })
  }

  /// Replaces the supplied state. Available on State<S, Unit>.
  @class
  put(_ replacement: S) -> State<S, T> where T == Unit {
    State<S, Unit>.new(|_| { StateStep<S, Unit>(state: replacement, value: ()) })
  }

  /// Updates the supplied state once per run, producing Unit.
  @class
  modify(_ f: (S) -> S) -> State<S, T> where T == Unit {
    State<S, Unit>.new(|state| { StateStep<S, Unit>(state: f.call(state), value: ()) })
  }

  /// Reads a projection of the state without changing it.
  @class
  inspect(_ f: (S) -> T) -> State<S, T> {
    State<S, T>.new(|state| { StateStep<S, T>(state: state, value: f.call(state)) })
  }

  /// Runs this transition with the supplied initial state.
  /// The returned `StateStep` contains the produced value and successor state.
  run(_ initial: S) -> StateStep<S, T> { self.transition.call(initial) }

  /// Executes the transition once and returns only its value.
  eval(_ initial: S) -> T { self.run(initial).value }

  /// Executes the transition once and returns only its successor state.
  exec(_ initial: S) -> S { self.run(initial).state }

  /// Transforms the produced value, preserving the successor state.
  map<U>(_ f: (T) -> U) -> State<S, U> {
    State<S, U>.new(|initial| {
      let step = self.run(initial)
      StateStep<S, U>(state: step.state, value: f.call(step.value))
    })
  }

  /// Runs this transition, then runs f(value) with its successor state.
  /// Each transition and f are invoked exactly once per completed run.
  flatMap<U>(_ f: (T) -> State<S, U>) -> State<S, U> {
    State<S, U>.new(|initial| {
      let step = self.run(initial)
      f.call(step.value).run(step.state)
    })
  }

  /// Runs both transitions left to right, then combines their values.
  /// The right transition receives the left transition's successor state.
  zipWith<U, R>(_ other: State<S, U>, combine: (T, U) -> R) -> State<S, R> {
    State<S, R>.new(|initial| {
      let left = self.run(initial)
      let right = other.run(left.state)
      StateStep<S, R>(state: right.state, value: combine.call(left.value, right.value))
    })
  }

  /// Runs this transition followed by other, discarding this value.
  then<U>(_ other: State<S, U>) -> State<S, U> {
    State<S, U>.new(|initial| { other.run(self.run(initial).state) })
  }

  /// Runs this transition against one part of a larger state.
  /// The lens reads once and replaces once; other components are preserved
  /// according to the supplied setter. No transition runs during composition.
  zoom<Q>(through lens: Lens<Q, S>) -> State<Q, T> {
    State<Q, T>.new(|whole| {
      let step = self.run(lens.get(whole))
      StateStep<Q, T>(state: lens.set(whole, to: step.state), value: step.value)
    })
  }

  /// Restores the original state value when a Result-valued transition fails.
  /// Success commits the successor state; the Result payload is preserved.
  /// This does not undo mutation of contained objects, external effects, or
  /// thrown exceptions. Use immutable state for transactional value updates.
  rollbackOnError<U, E>() -> State<S, Result<U, E>> where T == Result<U, E> {
    State<S, Result<U, E>>.new(|initial| {
      let step = self.run(initial)
      match step.value {
        Result::Ok(value) => StateStep<S, Result<U, E>>(state: step.state, value: Result::Ok(value))
        Result::Error(error) => StateStep<S, Result<U, E>>(state: initial, value: Result::Error(error))
      }
    })
  }
}

export State, StateStep
