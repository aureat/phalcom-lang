/// Represents the presence or absence of a value.
///
/// `Option<T>` contains either `Some(value)`, which carries a value of type `T`,
/// or `None`, which represents absence.
///
/// `Option` is the standard way to represent values that may be absent. Phalcom
/// does not expose a general-purpose `nil` value for this purpose.
///
/// Most operations preserve absence: transformations are applied to `Some`
/// values while `None` propagates through the operation.
///
/// @typeparam T The type of value carried by `Some`.
@native
enum Option<T> {

  /// An `Option` containing a value.
  ///
  /// @param value The wrapped value.
  @variant
  Some(_ value: T)

  /// An `Option` containing no value.
  @variant
  None

  /// Eliminates this option by invoking the callback corresponding to its
  /// current variant.
  ///
  /// When this option is `Some(value)`, `some` is invoked with the wrapped
  /// value. When this option is `None`, the zero-argument `none` callback is
  /// invoked.
  ///
  /// Exactly one callback is invoked.
  ///
  /// @typeparam R The common return type of both callbacks.
  /// @param some A one-argument callback invoked with the wrapped value when
  /// this option is `Some`.
  /// @param none A zero-argument callback invoked when this option is `None`.
  /// @returns The value returned by the invoked callback.
  @native
  match<R>(some: (value: T) -> R, none: () -> R) -> R

  /// Performs an action when this option is `None`.
  ///
  /// If this option is `None`, invokes the zero-argument callback `f`. If this
  /// option is `Some`, the callback is not invoked.
  ///
  /// The option itself is returned unchanged, making this operation suitable
  /// for observing absence inside a chain without extracting or transforming
  /// the contained value.
  ///
  /// @param f A zero-argument callback invoked only when this option is `None`.
  /// @returns This option unchanged.
  ifNone(_ f: () -> Unit) -> Option<T> {
    match(
      some: |v| self,
      none: || {
        f.call()
        self
      }
    )
  }

  /// Returns this option when it contains a value, or computes an alternative
  /// option when it is `None`.
  ///
  /// The fallback is evaluated lazily: `f` is invoked only when this option is
  /// `None`.
  ///
  /// This operation is the method-level target of the `??` operator.
  ///
  /// @param f A zero-argument callback that produces the fallback option.
  /// @returns This option when it is `Some`; otherwise, the option returned by
  /// `f`.
  orElse(_ f: () -> Option<T>) -> Option<T> {
    match(
      some: |v| self,
      none: || f.call()
    )
  }

  /// Tests whether this option contains a value.
  ///
  /// @returns `true` when this option is `Some`; otherwise `false`.
  isSome -> Bool {
    match(
      some: |v| true,
      none: || false
    )
  }

  /// Tests whether this option represents absence.
  ///
  /// @returns `true` when this option is `None`; otherwise `false`.
  isNone -> Bool {
    match(
      some: |v| false,
      none: || true
    )
  }

  /// Transforms the value contained by this option.
  ///
  /// If this option is `Some(value)`, invokes `f` with `value` and wraps the
  /// returned value in `Some`. If this option is `None`, the callback is not
  /// invoked and absence is propagated unchanged.
  ///
  /// @typeparam U The transformed payload type.
  /// @param f A one-argument callback that transforms the wrapped value.
  /// @returns `Some` containing the transformed value when this option is
  /// `Some`; otherwise `None`.
  map<U>(_ f: (value: T) -> U) -> Option<U> {
    match(
      some: |v| Option::Some(f.call(v)),
      none: || self
    )
  }

  /// Transforms the value contained by this option using an operation that
  /// itself returns an `Option`.
  ///
  /// Unlike `map`, the option returned by `f` is used directly rather than
  /// being wrapped in another `Some`. This allows option-producing operations
  /// to be chained without creating nested options.
  ///
  /// If this option is `None`, `f` is not invoked and absence is propagated.
  ///
  /// @typeparam U The payload type of the returned option.
  /// @param f A one-argument callback that receives the wrapped value and
  /// returns an option.
  /// @returns The option returned by `f` when this option is `Some`; otherwise
  /// `None`.
  flatMap<U>(_ f: (value: T) -> Option<U>) -> Option<U> {
    match(
      some: |v| f.call(v),
      none: || self
    )
  }

  /// Retains a contained value only when it satisfies a predicate.
  ///
  /// If this option is `Some(value)`, invokes `pred` with `value`. The original
  /// `Some` is retained when the predicate returns `true`; otherwise the result
  /// is `None`.
  ///
  /// If this option is already `None`, the predicate is not invoked and
  /// absence is propagated unchanged.
  ///
  /// @param pred A one-argument predicate evaluated for a contained value.
  /// @returns This option when it is `None` or when the predicate succeeds;
  /// otherwise `None`.
  filter(_ pred: (value: T) -> Bool) -> Option<T> {
    match(
      some: |v| {
        if (pred.call(v)) {
          self
        } else {
          Option::None
        }
      },
      none: || {
        self
      }
    )
  }

  /// Performs an action on the contained value without changing this option.
  ///
  /// If this option is `Some(value)`, invokes `f` with `value`. If this option
  /// is `None`, the callback is not invoked.
  ///
  /// The option itself is returned unchanged, making this operation suitable
  /// for observation, logging, or other side effects within an option chain.
  ///
  /// @param f A one-argument callback invoked with the wrapped value when this
  /// option is `Some`.
  /// @returns This option unchanged.
  ifSome(_ f: (value: T) -> Unit) -> Option<T> {
    match(
      some: |v| {
        f.call(v)
        self
      },
      none: || self
    )
  }

  /// Extracts the contained value or returns a fallback value.
  ///
  /// If this option is `Some(value)`, returns `value`. If this option is
  /// `None`, returns `default`.
  ///
  /// The fallback value is evaluated before this method is called. Use
  /// `orElse` when the fallback should be produced lazily as another option.
  ///
  /// @param default A value of the option's payload type `T` to return when
  /// this option is `None`.
  /// @returns The contained value when this option is `Some`; otherwise
  /// `default`. The result type is `T`.
  unwrapOr(_ default: T) -> T {
    match(
      some: |v| v,
      none: || default
    )
  }

  /// Returns a string representation of this option.
  ///
  /// A `Some(value)` is represented as `Some(<value>)`, using the contained
  /// value's own `toString` representation. `None` is represented as `None`.
  ///
  /// @returns The string representation of this option.
  toString -> String {
    match(
      some: |v| "Some(" + v.toString + ")",
      none: || "None"
    )
  }

  /// Converts this option into a `Result`, supplying an error for the absent
  /// case.
  ///
  /// `Some(value)` becomes `Ok(value)`. `None` becomes `Error(err)`.
  ///
  /// The error value is used only for the `None` case.
  ///
  /// @typeparam E The error type of the resulting `Result`.
  /// @param err The error value to use when this option is `None`.
  /// @returns `Ok` containing the wrapped value when this option is `Some`;
  /// otherwise `Error` containing `err`.
  okOr<E>(_ err: E) -> Result<T, E> {
    match(
      some: |v| Result::Ok(v),
      none: || Result::Error(err)
    )
  }

  /// Tests this option for value equality with another option.
  ///
  /// Two `Some` values are equal when their wrapped values are equal. Two
  /// `None` values are equal. A `Some` and a `None` are never equal.
  ///
  /// Values that are not options compare unequal.
  ///
  /// @param other The value to compare with this option.
  /// @returns `true` when `other` is an equivalent option; otherwise `false`.
  ==(_ other) -> Bool {
    other.is(Option).ifFalse {
      return false
    }

    match(
      some: |v| {
        other.match(
          some: |ov| v == ov,
          none: || false
        )
      },
      none: || other.isNone
    )
  }

  /// Computes the hash value of this option.
  ///
  /// `Some(value)` uses the wrapped value's hash. `None` uses the canonical
  /// hash value for absence.
  ///
  /// Equal options produce equal hash values.
  ///
  /// @returns The hash value of this option.
  hash -> Int {
    self.match(
      some: |v| v.hash,
      none: || 0
    )
  }
}
