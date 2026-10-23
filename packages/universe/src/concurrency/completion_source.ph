from .fiber import Future
from universe.errors.error import Error
from universe.result import Result

class CompletionSource<T> {
  mut _future

  /// Creates a new completion source and its underlying pending future.
  @constructor
  new() {
    let f: Future<T> = Future.new()
    _future = f
  }

  /// Returns the underlying future managed by this completion source.
  future -> Future<T> {
    _future
  }

  /// Attempts to resolve the underlying future with `value`.
  ///
  /// @param value Fulfillment value.
  /// @returns `true` if this call settled the future; `false` if already settled.
  tryResolve(_ value: T) -> Bool {
    let res: Result<T, Error> = Result::Ok(value)
    _future.trySettle(res)
  }

  /// Attempts to reject the underlying future with `error`.
  ///
  /// @param error Rejection error.
  /// @returns `true` if this call settled the future; `false` if already settled.
  tryReject(_ error: Error) -> Bool {
    let res: Result<T, Error> = Result::Error(error)
    _future.trySettle(res)
  }
}

export CompletionSource
