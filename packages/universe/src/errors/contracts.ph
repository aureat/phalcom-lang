//! Contract verification and specification failure exception types.
//!
//! Defines the exception hierarchy for Design-by-Contract assertions,
//! including preconditions, postconditions, and class invariants.

/// Raised when a method precondition (`@requires`) is violated by the caller.
class PreconditionError is Error {}

/// Raised when a method postcondition (`@ensures`) fails upon returning.
class PostconditionError is Error {}

/// Raised when a class state invariant (`@invariant`) fails verification.
class InvariantError is Error {}
