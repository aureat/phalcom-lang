/// Represents the outcome of an operation that may succeed or fail.
///
/// A `Result<T, E>` is either `Ok(value)`, containing a successful value of
/// type `T`, or `Err(error)`, containing a failure value of type `E`.
///
/// Operations on a result generally act on one variant while propagating the
/// other. Success transformations preserve `Err` values, while error
/// transformations preserve `Ok` values. Sequencing operations short-circuit
/// when they encounter the variant they do not handle.
///
/// @typeparam T The type of value carried by `Ok`.
/// @typeparam E The type of error carried by `Err`.
@native
enum Result<T, E> {

    /// A successful result containing a value.
    ///
    /// @param value The successful value.
    @variant
    Ok(_ value: T)

    /// A failed result containing an error.
    ///
    /// @param error The failure value.
    @variant
    Error(_ error: E)

    /// Eliminates this result by invoking the callback corresponding to its
    /// current variant.
    ///
    /// If this result is `Ok(value)`, invokes `ok` with the successful value.
    /// If this result is `Error(error)`, invokes `err` with the error.
    ///
    /// Exactly one callback is invoked. Both callbacks produce the same result
    /// type, making `match` the general-purpose eliminator for `Result`.
    ///
    /// @typeparam R The type produced by either callback.
    /// @param ok A callback invoked with the successful value when this result
    /// is `Ok`.
    /// @param err A callback invoked with the error when this result is `Error`.
    /// @returns The value returned by the invoked callback.
    match<R>(
        ok: (value: T) -> R,
        err: (error: E) -> R
    ) -> R {
        match self {
            Ok(value) => ok.call(value)
            Error(error) => err.call(error)
        }
    }

    /// Tests whether this result represents success.
    ///
    /// @returns `true` when this result is `Ok`; otherwise `false`.
    isOk -> Bool {
        match(
            ok: |v| { true },
            err: |e| { false }
        )
    }

    /// Tests whether this result represents failure.
    ///
    /// @returns `true` when this result is `Error`; otherwise `false`.
    isErr -> Bool {
        match(
            ok: |v| { false },
            err: |e| { true }
        )
    }

    /// Transforms the successful value of this result.
    ///
    /// If this result is `Ok(value)`, invokes `f` with `value` and returns a
    /// new `Ok` containing the transformed value. If this result is `Error`, the
    /// callback is not invoked and the error is propagated unchanged.
    ///
    /// @typeparam U The type of successful value produced by the transformation.
    /// @param f A callback that transforms the successful value.
    /// @returns A result containing the transformed success value, or the
    /// original error when this result is `Error`.
    map<U>(_ f: (value: T) -> U) -> Result<U, E> {
        self.match(
            ok: |v| { Result::Ok(f.call(v)) },
            err: |e| { Result::Error(e) }
        )
    }

    /// Transforms the error of this result.
    ///
    /// If this result is `Error(error)`, invokes `f` with `error` and returns a
    /// new `Error` containing the transformed error. If this result is `Ok`, the
    /// callback is not invoked and the successful value is propagated
    /// unchanged.
    ///
    /// @typeparam F The error type produced by the transformation.
    /// @param f A callback that transforms the error value.
    /// @returns A result containing the original successful value, or the
    /// transformed error when this result is `Error`.
    mapErr<F>(_ f: (error: E) -> F) -> Result<T, F> {
        self.match(
            ok: |v| { Result::Ok(v) },
            err: |e| { Result::Error(f.call(e)) }
        )
    }

    /// Transforms both possible values of this result.
    ///
    /// If this result is `Ok(value)`, invokes `ok` and wraps its result in
    /// `Ok`. If this result is `Error(error)`, invokes `err` and wraps its result
    /// in `Error`.
    ///
    /// Exactly one callback is invoked.
    ///
    /// @typeparam U The resulting successful value type.
    /// @typeparam F The resulting error type.
    /// @param ok A callback that transforms a successful value.
    /// @param err A callback that transforms an error value.
    /// @returns A result containing whichever transformed value corresponds to
    /// this result's current variant.
    mapBoth<U, F>(
        ok: (T) -> U,
        err: (E) -> F
    ) -> Result<U, F> {
        match self {
            Ok(value) => Result::Ok(ok.call(value))
            Error(error) => Result::Error(err.call(error))
        }
    }

    /// Chains another result-producing operation after a successful result.
    ///
    /// If this result is `Ok(value)`, invokes `f` with `value` and returns the
    /// result produced by `f` directly. If this result is `Error`, `f` is not
    /// invoked and the error is propagated unchanged.
    ///
    /// Unlike `map`, this method does not wrap the callback result in another
    /// `Ok`, allowing result-producing operations to be chained without
    /// creating nested results.
    ///
    /// @typeparam U The successful value type of the result produced by `f`.
    /// @param f A callback that receives the successful value and produces
    /// another result.
    /// @returns The result produced by `f` when this result is `Ok`; otherwise
    /// the original error.
    andThen<U>(
        _ f: (value: T) -> Result<U, E>
    ) -> Result<U, E> {
        self.match(
            ok: |v| { f.call(v) },
            err: |e| { Result::Error(e) }
        )
    }

    /// Recovers from a failed result using another result-producing operation.
    ///
    /// If this result is `Error(error)`, invokes `f` with `error` and returns the
    /// result produced by `f` directly. If this result is `Ok`, `f` is not
    /// invoked and the successful value is propagated unchanged.
    ///
    /// `orElse` is the error-side counterpart of `andThen`.
    ///
    /// @typeparam F The error type of the result produced by `f`.
    /// @param f A callback that receives the current error and produces an
    /// alternative result.
    /// @returns The original successful value when this result is `Ok`;
    /// otherwise the result produced by `f`.
    orElse<F>(
        _ f: (error: E) -> Result<T, F>
    ) -> Result<T, F> {
        self.match(
            ok: |value| {
                Result::Ok(value)
            },
            err: |error| {
                f.call(error)
            }
        )
    }

    /// Tests whether this result is `Ok` and its successful value satisfies a
    /// predicate.
    ///
    /// The predicate is evaluated only for an `Ok` value. An `Error` result
    /// always produces `false`.
    ///
    /// @param predicate A predicate evaluated against the successful value.
    /// @returns `true` when this result is `Ok` and `predicate` returns
    /// `true`; otherwise `false`.
    isOkAnd(_ predicate: (value: T) -> Bool) -> Bool {
        self.match(
            ok: |value| { predicate.call(value) },
            err: |error| { false }
        )
    }

    /// Tests whether this result is `Error` and its error satisfies a predicate.
    ///
    /// The predicate is evaluated only for an `Error` value. An `Ok` result
    /// always produces `false`.
    ///
    /// @param predicate A predicate evaluated against the error value.
    /// @returns `true` when this result is `Error` and `predicate` returns
    /// `true`; otherwise `false`.
    isErrAnd(_ predicate: (error: E) -> Bool) -> Bool {
        self.match(
            ok: |value| { false },
            err: |error| { predicate.call(error) }
        )
    }

    /// Performs an action on the successful value without changing the result.
    ///
    /// If this result is `Ok(value)`, invokes `f` with `value`. If this result
    /// is `Error`, the callback is not invoked.
    ///
    /// This method is intended for observation, logging, instrumentation, and
    /// other side effects within a result chain.
    ///
    /// @param f A callback invoked with the successful value when this result
    /// is `Ok`.
    /// @returns An equivalent result containing the original success or error
    /// value.
    inspect(
        _ f: (value: T) -> Unit
    ) -> Result<T, E> {
        self.match(
            ok: |value| {
                f.call(value)
                Result::Ok(value)
            },
            err: |error| {
                Result::Error(error)
            }
        )
    }

    /// Performs an action on the error without changing the result.
    ///
    /// If this result is `Error(error)`, invokes `f` with `error`. If this result
    /// is `Ok`, the callback is not invoked.
    ///
    /// This method is intended for observation, logging, instrumentation, and
    /// other side effects within a result chain.
    ///
    /// @param f A callback invoked with the error when this result is `Error`.
    /// @returns An equivalent result containing the original success or error
    /// value.
    inspectErr(
        _ f: (error: E) -> Unit
    ) -> Result<T, E> {
        self.match(
            ok: |value| {
                Result::Ok(value)
            },
            err: |error| {
                f.call(error)
                Result::Error(error)
            }
        )
    }

    /// Extracts the successful value from this result.
    ///
    /// If this result is `Ok(value)`, returns `value`. If this result is
    /// `Error(error)`, raises an `UnwrapError` containing the error.
    ///
    /// Use this operation only when failure represents an unrecoverable
    /// condition or has already been ruled out. Prefer `match`, `map`,
    /// `andThen`, or the defaulting operations when failure is expected.
    ///
    /// @returns The successful value carried by `Ok`.
    unwrap -> T {
        self.match(
            ok: |value| {
                value
            },
            err: |error| {
                UnwrapError(
                    "called Result.unwrap on an Error value",
                    error
                ).raise()
            }
        )
    }

    /// Extracts the error from this result.
    ///
    /// If this result is `Error(error)`, returns `error`. If this result is
    /// `Ok(value)`, raises an `UnwrapError` containing the successful value.
    ///
    /// @returns The error carried by `Error`.
    unwrapErr -> E {
        self.match(
            ok: |value| {
                UnwrapError(
                    "called Result.unwrapErr on an Ok value",
                    value
                ).raise()
            },
            err: |error| {
                error
            }
        )
    }

    /// Extracts the successful value or returns a fallback value.
    ///
    /// If this result is `Ok(value)`, returns `value`. If this result is
    /// `Error`, returns `default`.
    ///
    /// `default` is an already-evaluated value. Use `unwrapOrElse` when
    /// computing the fallback should be deferred until an error actually
    /// occurs.
    ///
    /// @param default The fallback value to use when this result is `Error`.
    /// @returns The successful value when this result is `Ok`; otherwise
    /// `default`.
    unwrapOr(_ default: T) -> T {
        self.match(
            ok: |v| { v },
            err: |e| { default }
        )
    }

    /// Extracts the successful value or computes a fallback from the error.
    ///
    /// If this result is `Ok(value)`, returns `value` without invoking `f`.
    /// If this result is `Error(error)`, invokes `f` with `error` and returns the
    /// resulting value.
    ///
    /// The fallback is therefore evaluated lazily and may depend on the
    /// particular error.
    ///
    /// @param f A callback that computes a fallback value from an error.
    /// @returns The successful value when this result is `Ok`; otherwise the
    /// fallback value produced by `f`.
    unwrapOrElse(
        _ f: (error: E) -> T
    ) -> T {
        self.match(
            ok: |value| {
                value
            },
            err: |error| {
                f.call(error)
            }
        )
    }

    /// Extracts the successful value, using a caller-provided message when
    /// extraction fails.
    ///
    /// If this result is `Ok(value)`, returns `value`. If this result is
    /// `Error(error)`, raises an `UnwrapError` containing `message` and the
    /// underlying error.
    ///
    /// @param message The diagnostic message to associate with a failed
    /// extraction.
    /// @returns The successful value carried by `Ok`.
    expect(_ message: String) -> T {
        self.match(
            ok: |value| {
                value
            },
            err: |error| {
                UnwrapError.new(message, error).raise()
            }
        )
    }

    /// Extracts the error, using a caller-provided message when the result is
    /// unexpectedly successful.
    ///
    /// If this result is `Error(error)`, returns `error`. If this result is
    /// `Ok(value)`, raises an `UnwrapError` containing `message` and the
    /// successful value.
    ///
    /// @param message The diagnostic message to associate with a failed error
    /// extraction.
    /// @returns The error carried by `Error`.
    expectErr(_ message: String) -> E {
        self.match(
            ok: |value| {
                UnwrapError.new(message, value).raise()
            },
            err: |error| {
                error
            }
        )
    }

    /// Removes one level of nested `Result`.
    ///
    /// This operation is available when the successful value of this result is
    /// itself a `Result<U, E>`.
    ///
    /// `Ok(Ok(value))` becomes `Ok(value)`, `Ok(Error(error))` becomes
    /// `Error(error)`, and an outer `Error(error)` is propagated unchanged.
    ///
    /// @typeparam U The successful value type of the nested result.
    /// @returns The inner result when this result is `Ok`; otherwise an `Error`
    /// containing the outer error.
    flatten<U>() -> Result<U, E>
        where T == Result<U, E>
    {
        self.match(
            ok: |result| {
                result
            },
            err: |error| {
                Result::Error(error)
            }
        )
    }

    /// Exchanges the nesting order of `Result` and `Option`.
    ///
    /// This operation is available when the successful value of this result is
    /// an `Option<U>`.
    ///
    /// `Ok(Some(value))` becomes `Some(Ok(value))`, `Ok(None)` becomes `None`,
    /// and `Error(error)` becomes `Some(Error(error))`.
    ///
    /// @typeparam U The value type contained by the nested option.
    /// @returns The corresponding optional result.
    transpose<U>() -> Option<Result<U, E>> where T == Option<U>
    {
        self.match(
            ok: |option| {
                option.match(
                    some: |value| {
                        Option::Some(Result::Ok(value))
                    },
                    none: || {
                        Option::None
                    }
                )
            },
            err: |error| {
                Option::Some(Result::Error(error))
            }
        )
    }

    /// Converts the successful side of this result into an `Option`.
    ///
    /// `Ok(value)` becomes `Some(value)`. `Error` becomes `None`, discarding the
    /// error.
    ///
    /// @returns `Some` containing the successful value when this result is
    /// `Ok`; otherwise `None`.
    ok -> Option<T> {
        self.match(
            ok: |v| { Option::Some(v) },
            err: |e| { Option::None }
        )
    }

    /// Converts the error side of this result into an `Option`.
    ///
    /// `Error(error)` becomes `Some(error)`. `Ok` becomes `None`, discarding the
    /// successful value.
    ///
    /// @returns `Some` containing the error when this result is `Error`;
    /// otherwise `None`.
    err -> Option<E> {
        self.match(
            ok: |v| { Option::None },
            err: |e| { Option::Some(e) }
        )
    }

    /// Returns a string representation of this result.
    ///
    /// `Ok(value)` is represented as `Result::Ok(<value>)` and `Error(error)` as
    /// `Result::Error(<error>)`. The contained value or error is rendered using
    /// its own `toString` representation.
    ///
    /// @returns The string representation of this result.
    toString -> String {
        self.match(
            ok: |v| { "Result::Ok(" + v.toString + ")" },
            err: |e| { "Result::Error(" + e.toString + ")" }
        )
    }
}

/// Represents an invalid attempt to extract the opposite variant of a
/// `Result`.
///
/// An `UnwrapError` is raised when an operation requiring `Ok` receives an
/// `Error`, or when an operation requiring `Error` receives an `Ok`.
class UnwrapError is Error {

    /// Returns the diagnostic description of this unwrap failure.
    ///
    /// @returns A human-readable description of the invalid extraction.
    toString -> String {
        "called `Result::unwrap()` on an `Error` value"
    }
}
