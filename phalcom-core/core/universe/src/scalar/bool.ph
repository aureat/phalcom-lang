//! Boolean logic and truth values.
//!
//! `Bool` is the abstract base class for boolean logic values `true` and `false`.

/// Abstract base class representing boolean truth values.
///
/// `Bool` provides core logical operations (`and`, `or`, `not`) and string conversion.
native class Bool is Object {
  @class
  native new() -> Bool

  @class
  native new(_ value: Dynamic) -> Bool

  native and(_ other: Bool) -> Bool
  native or(_ other: Bool) -> Bool
  native not -> Bool
  native ifTrue(_ then: Dynamic) -> Dynamic
  native ifFalse(_ otherwise: Dynamic) -> Dynamic
  native ifTrue(_ then: Dynamic, ifFalse: Dynamic) -> Dynamic
  native hash -> Int

  /// Returns the string representation (`"true"` or `"false"`).
  toString {
    if self {
      "true"
    } else {
      "false"
    }
  }
}

/// Concrete singleton class for the boolean value `true`.
native class True is Bool {}

/// Concrete singleton class for the boolean value `false`.
native class False is Bool {}
