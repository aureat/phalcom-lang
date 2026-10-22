//! Error-aware state composition: failed stages skip subsequent callbacks.
from .state import State, StateStep
from .lens import Lens

/// A deferred state transition whose produced value can fail.
/// Failure retains the failing stage's successor state by default. Use
/// transactional() explicitly to restore the initial state value on failure.
data StateResult<S, T, E> private(_ transition: (S) -> StateStep<S, Result<T, E>>)

impl<S, T, E> StateResult<S, T, E> {
  @class new(_ transition: (S) -> StateStep<S, Result<T, E>>) -> StateResult<S, T, E> {
    StateResult<S, T, E>(transition)
  }
  @class pure(_ value: T) -> StateResult<S, T, E> {
    StateResult<S, T, E>.new(|state| {
      StateStep<S, Result<T, E>>(state: state, value: Result<T, E>::Ok(value))
    })
  }
  @class fail(_ error: E) -> StateResult<S, T, E> {
    StateResult<S, T, E>.new(|state| {
      StateStep<S, Result<T, E>>(state: state, value: Result<T, E>::Error(error))
    })
  }
  @class fromState(_ computation: State<S, T>) -> StateResult<S, T, E> {
    StateResult<S, T, E>.new(|initial| {
      let step = computation.run(initial)
      StateStep<S, Result<T, E>>(state: step.state, value: Result<T, E>::Ok(step.value))
    })
  }
  @class fromResult(_ result: Result<T, E>) -> StateResult<S, T, E> {
    StateResult<S, T, E>.new(|state| { StateStep<S, Result<T, E>>(state: state, value: result) })
  }
  run(_ initial: S) -> StateStep<S, Result<T, E>> { self.transition.call(initial) }
  eval(_ initial: S) -> Result<T, E> { self.run(initial).value }
  toState -> State<S, Result<T, E>> { State<S, Result<T, E>>.new(self.transition) }

  map<U>(_ f: (T) -> U) -> StateResult<S, U, E> {
    StateResult<S, U, E>.new(|initial| {
      let step = self.run(initial)
      StateStep<S, Result<U, E>>(state: step.state, value: step.value.map(f))
    })
  }
  mapErr<F>(_ f: (E) -> F) -> StateResult<S, T, F> {
    StateResult<S, T, F>.new(|initial| {
      let step = self.run(initial)
      StateStep<S, Result<T, F>>(state: step.state, value: step.value.mapErr(f))
    })
  }
  flatMap<U>(_ f: (T) -> StateResult<S, U, E>) -> StateResult<S, U, E> {
    StateResult<S, U, E>.new(|initial| {
      let step = self.run(initial)
      match step.value {
        Result::Ok(value) => f.call(value).run(step.state)
        Result::Error(error) => StateStep<S, Result<U, E>>(state: step.state, value: Result<U, E>::Error(error))
      }
    })
  }
  /// Recovery starts from the failed successor state; error types may change.
  orElse<F>(_ f: (E) -> StateResult<S, T, F>) -> StateResult<S, T, F> {
    StateResult<S, T, F>.new(|initial| {
      let step = self.run(initial)
      match step.value {
        Result::Ok(value) => StateStep<S, Result<T, F>>(state: step.state, value: Result<T, F>::Ok(value))
        Result::Error(error) => f.call(error).run(step.state)
      }
    })
  }
  /// Runs other only after success; preserves the first failure.
  then<U>(_ other: StateResult<S, U, E>) -> StateResult<S, U, E> {
    self.flatMap(|_| { other })
  }
  zipWith<U, R>(_ other: StateResult<S, U, E>, combine: (T, U) -> R) -> StateResult<S, R, E> {
    self.flatMap(|left| { other.map(|right| { combine.call(left, right) }) })
  }
  /// Rolls back value state on Result::Error. Does not catch exceptions or
  /// reverse contained mutations/external effects. Wrap the intended boundary.
  transactional() -> StateResult<S, T, E> {
    StateResult<S, T, E>.new(|initial| {
      let step = self.run(initial)
      match step.value {
        Result::Ok(_) => step
        Result::Error(_) => StateStep<S, Result<T, E>>(state: initial, value: step.value)
      }
    })
  }
  zoom<Q>(through lens: Lens<Q, S>) -> StateResult<Q, T, E> {
    StateResult<Q, T, E>.new(|whole| {
      let step = self.run(lens.get(whole))
      StateStep<Q, Result<T, E>>(state: lens.set(whole, to: step.state), value: step.value)
    })
  }
}

export StateResult
