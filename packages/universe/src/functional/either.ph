//! A neutral choice between two payload types; neither side denotes an error.
data Either<L, R> {
  Left(_ value: L)
  Right(_ value: R)
}
impl<L, R> Either<L, R> {
  /// Invokes only the callback selected by the current variant.
  fold<T>(left: (L) -> T, right: (R) -> T) -> T {
    match self {
      Either::Left(value) => left.call(value)
      Either::Right(value) => right.call(value)
    }
  }
  /// Returns true when this value is `Left`.
  isLeft -> Bool {
    match self {
      Either::Left(_) => true
      Either::Right(_) => false
    }
  }
  /// Returns true when this value is `Right`.
  isRight -> Bool { not self.isLeft }
  /// Returns the left payload, or `None` for a `Right` value.
  left -> Option<L> {
    match self {
      Either::Left(value) => Option::Some(value)
      Either::Right(_) => Option::None
    }
  }
  /// Returns the right payload, or `None` for a `Left` value.
  right -> Option<R> {
    match self {
      Either::Left(_) => Option::None
      Either::Right(value) => Option::Some(value)
    }
  }
  /// Transforms a `Left` payload and leaves a `Right` payload unchanged.
  mapLeft<A>(_ f: (L) -> A) -> Either<A, R> {
    match self {
      Either::Left(value) => Either<A, R>::Left(f.call(value))
      Either::Right(value) => Either<A, R>::Right(value)
    }
  }
  /// Transforms a `Right` payload and leaves a `Left` payload unchanged.
  mapRight<B>(_ f: (R) -> B) -> Either<L, B> {
    match self {
      Either::Left(value) => Either<L, B>::Left(value)
      Either::Right(value) => Either<L, B>::Right(f.call(value))
    }
  }
  /// Applies `left` to a `Left` payload or `right` to a `Right` payload.
  bimap<A, B>(left: (L) -> A, right: (R) -> B) -> Either<A, B> {
    match self {
      Either::Left(value) => Either<A, B>::Left(left.call(value))
      Either::Right(value) => Either<A, B>::Right(right.call(value))
    }
  }
  /// Reverses the variant while preserving its payload.
  swap -> Either<R, L> {
    match self {
      Either::Left(value) => Either<R, L>::Right(value)
      Either::Right(value) => Either<R, L>::Left(value)
    }
  }
  /// Extracts either payload only when the two payload types are equivalent.
  merge -> L where L == R {
    match self { Either::Left(value) => value; Either::Right(value) => value }
  }
}

export Either
