//! Reflection surface for classes and metaclass behaviors.
//!
//! `Behavior` provides introspection capabilities for class hierarchy, methods,
//! selector binding, and attached attributes.

/// Abstract root class for class behaviors and metaclasses.
///
/// Implemented by `Class` and `Metaclass`. Supplies reflection for methods,
/// superclass navigation, and decorator attribute inspection.
native primitive class Behavior is Object {
  native superclass -> Dynamic

  native superclass=(_ value: Dynamic) -> Dynamic

  native name -> String

  native methods -> Dynamic

  native >>(_ selector: Dynamic) -> Dynamic

  /// Returns all metadata attributes attached to this behavior.
  attributes { rawAttributes }

  /// Filters attributes by type or class.
  ///
  /// @param cls Target attribute class.
  /// @returns List of matching attached attributes.
  attributesOfType(_ cls) { rawAttributes.filter(|a| a is cls) }
}
