//! Base error hierarchy and runtime exception definitions.
//!
//! All catchable exceptions in Phalcom inherit from `Error`. This module defines the
//! base `Error` class carrying messages, causes, kinds, and displaced payloads.

/// Base class for all exception and error types in Phalcom.
///
/// `Error` instances carry a message string, an optional diagnostic error kind, an optional
/// causal error chain, and an optional displaced payload.
native class Error is Object {
  native message -> String
  native raise() -> Dynamic

  mut _message
  mut _kind
  mut _cause
  mut _displaced

  /// Creates a blank `Error` with no message.
  @constructor
  new() {
    _message = None
    _kind = None
    _cause = None
    _displaced = None
  }

  /// Creates an `Error` initialized with a message string.
  ///
  /// @param msg The error description.
  @constructor
  new(_ msg) {
    _message = msg
    _kind = None
    _cause = None
    _displaced = None
  }

  /// Initializes all metadata fields of this error.
  initialize(with msg, of kind, from cause, displaced) {
    _message = msg
    _kind = kind
    _cause = cause
    _displaced = displaced
  }

  /// The symbolic error classification kind, if set.
  kind { _kind }

  /// Sets the symbolic error classification kind.
  kind=(_ value) { _kind = value }

  /// The underlying cause of this error, if chained.
  cause { _cause }

  /// Sets the underlying causal error.
  cause=(_ value) { _cause = value }

  /// The displaced object associated with this error, if any.
  displaced { _displaced }

  /// Sets the displaced object for this error.
  displaced=(_ value) { _displaced = value }
}

/// Raised when a receiver does not understand an invoked selector.
native class MessageNotUnderstood is Error {}

/// Raised when a fiber attempts to yield across an unmanaged native boundary.
native class CannotYieldAcrossNativeFrame is Error {}

/// Raised when accessing a closed resource or stream handle.
native class UseAfterCloseError is Error {}

/// Raised when unwrapping an `Option` or `Result` variant fails.
class UnwrapError is Error {
  /// Returns the diagnostic description of this unwrap failure.
  toString -> String {
    "called `Result::unwrap()` on an `Error` value"
  }
}

/// Raised when an asynchronous operation or future times out.
class TimeoutError is Error {}
