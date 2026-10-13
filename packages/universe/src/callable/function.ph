//! Abstract function and callable representation.
//!
//! `Function` is the root class for invokable routines, methods, and closures.

/// Root abstract class for callable values in Phalcom.
///
/// Inherited by `Closure` and `Method`. Provides dynamic invocation, arity inspection,
/// and error-capturing helpers.
native primitive class Function is Object {
  native arity -> Int
  native name -> Symbol
  native callWith(_ value: Dynamic) -> Dynamic
  native primitive call(***args: Dynamic) -> Dynamic

  /// Executes this zero-argument callable, capturing any raised `Error` into a `Result`.
  ///
  /// Returns `Result::Ok(value)` upon successful completion, or `Result::Error(e)`
  /// if an error was raised.
  ///
  /// @returns Execution outcome as a `Result`.
  attempt() {
    return || { Result::Ok(self.call()) }.on(Error) |e| { Result::Error(e) }
  }
}
