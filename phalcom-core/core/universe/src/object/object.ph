//! Root object model and foundational class hierarchy for Phalcom.
//!
//! `Object` is the primordial superclass of all values and classes. It defines
//! reflection, identity, classification, and relational comparisons.

/// Primordial root class for all objects in Phalcom.
///
/// Provides basic object identity, string formatting, dynamic invocation, classification (`is`),
/// and relational comparison operators derived from bilateral comparison.
native class Object {
  native name -> Dynamic

  native class -> Dynamic

  native class=(_ value: Dynamic) -> Unit

  native toString -> String

  native hash -> Int

  native ==(_ other: Dynamic) -> Bool

  native !=(_ other: Dynamic) -> Bool

  native ===(_ other: Dynamic) -> Bool

  native matches(_ other: Dynamic) -> Bool

  native understands(_ selector: Dynamic) -> Bool

  native perform(_ selector: Dynamic, ***args: Dynamic) -> Dynamic

  native respondsTo(_ selector: Dynamic) -> Bool

  native doesNotUnderstand(_ message: Dynamic) -> Dynamic

  native methodFor(_ selector: Dynamic) -> Dynamic

  internal native invariantEnter() -> Dynamic

  internal native invariantExit() -> Dynamic

  internal native rawAttributes -> Dynamic

  internal native attachAttribute(_ attribute: Dynamic) -> Dynamic

  internal native freezeAttributes() -> Dynamic

  /// Tests whether this object is an instance of `cls` or one of its subclasses.
  ///
  /// @param cls The class to test against.
  /// @returns `true` if `cls` is found in the ancestor chain; otherwise `false`.
  is(_ cls) {
    let mut c = self.class
    while c != None {
      if c == cls {
        return true
      }
      c = c.superclass
    }
    false
  }

  /// Tests whether this object is an exact, direct instance of `cls` (ignoring subclasses).
  ///
  /// @param cls The class to test against.
  /// @returns `true` if direct class equals `cls`; otherwise `false`.
  is!(_ cls) { self.class == cls }

  /// Less-than comparison derived from `<=>`.
  <(_ other) { (self <=> other) === Ordering::Less }

  /// Less-than-or-equal comparison derived from `<=>`.
  <=(_ other) {
    let mut order = (self <=> other)
    (order === Ordering::Less) or (order === Ordering::Equal)
  }

  /// Greater-than comparison derived from `<=>`.
  >(_ other) { (self <=> other) === Ordering::Greater }

  /// Greater-than-or-equal comparison derived from `<=>`.
  >=(_ other) {
    let mut order = (self <=> other)
    (order === Ordering::Greater) or (order === Ordering::Equal)
  }
}
