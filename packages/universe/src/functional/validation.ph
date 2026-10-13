//! Accumulating independent errors in forms, configuration, and input decoding.
from .algebra import Semigroup

/// Either a validated value or an error payload.
/// zipWith accumulates BOTH errors through an explicit combining operation.
/// E can be a string, an immutable collection, or a domain-specific report.
data Validation<T, E> {
  @variant Valid(_ value: T)
  @variant Invalid(_ error: E)
}

impl<T, E> Validation<T, E> {
  @class
  fromResult(_ result: Result<T, E>) -> Validation<T, E> {
    match result {
      Result::Ok(value) => Validation<T, E>::Valid(value)
      Result::Error(error) => Validation<T, E>::Invalid(error)
    }
  }

  toResult -> Result<T, E> {
    match self {
      Validation::Valid(value) => Result::Ok(value)
      Validation::Invalid(error) => Result::Error(error)
    }
  }

  isValid -> Bool {
    match self {
      Validation::Valid(_) => true
      Validation::Invalid(_) => false
    }
  }

  isInvalid -> Bool { not self.isValid }

  /// Invokes exactly the callback selected by the current variant.
  fold<R>(valid: (T) -> R, invalid: (E) -> R) -> R {
    match self {
      Validation::Valid(value) => valid.call(value)
      Validation::Invalid(error) => invalid.call(error)
    }
  }

  map<U>(_ f: (T) -> U) -> Validation<U, E> {
    match self {
      Validation::Valid(value) => Validation<U, E>::Valid(f.call(value))
      Validation::Invalid(error) => Validation<U, E>::Invalid(error)
    }
  }

  mapErr<F>(_ f: (E) -> F) -> Validation<T, F> {
    match self {
      Validation::Valid(value) => Validation<T, F>::Valid(value)
      Validation::Invalid(error) => Validation<T, F>::Invalid(f.call(error))
    }
  }

  /// Checks a valid payload; an existing error propagates without callbacks.
  /// The error producer is invoked only when the predicate rejects the value.
  ensure(_ predicate: (T) -> Bool, error: (T) -> E) -> Validation<T, E> {
    match self {
      Validation::Invalid(_) => self
      Validation::Valid(value) => {
        if predicate.call(value) { self } else { Validation<T, E>::Invalid(error.call(value)) }
      }
    }
  }

  /// Combines independent validations that have already been evaluated.
  /// Both Valid: combine runs once. Both Invalid: errors runs once, with
  /// this error first. Mixed variants: the sole error is propagated and
  /// neither callback runs. Callback exceptions propagate normally.
  /// Use an associative errors operation for grouping-independent accumulation.
  zipWith<U, R>(
    _ other: Validation<U, E>,
    combine: (T, U) -> R,
    errors: (E, E) -> E
  ) -> Validation<R, E> {
    match self {
      Validation::Valid(left) => {
        match other {
          Validation::Valid(right) => Validation<R, E>::Valid(combine.call(left, right))
          Validation::Invalid(error) => Validation<R, E>::Invalid(error)
        }
      }
      Validation::Invalid(left) => {
        match other {
          Validation::Valid(_) => Validation<R, E>::Invalid(left)
          Validation::Invalid(right) => Validation<R, E>::Invalid(errors.call(left, right))
        }
      }
    }
  }
}

impl<T, E> Validation<T, E> where E |= Semigroup {
  /// Uses nominal Semigroup conformance for error accumulation.
  combineWith<U, R>(_ other: Validation<U, E>, using f: (T, U) -> R) -> Validation<R, E> {
    self.zipWith(other, combine: f, errors: |left, right| { left.combine(right) })
  }
}

export Validation
