// `Error` (U-CORE-6, ADR-0008) is bootstrapped natively with `_message` at
// fixed slot 0 (`message`/`raise` are native primitives reading/using it
// directly — `primitive/error.rs`), but ships with no user-visible way to
// *set* that slot: bare `Error.new()` (the generic 0-arg allocator) leaves it
// unset, and — per ADR-0011/U-INH §3.5 "fields stack, never alias" — a `.ph`
// *subclass* that independently assigns a same-named `_message` field gets
// its own **fresh** slot, not this one (`subclass_field_offset_stability`).
// So the only way to reach the native slot 0 from `.ph` is an `@constructor`
// declared on `Error` **itself**: this reopen (recognised at compile time as
// reopening the already-registered native class, not a fresh `extends`) adds
// exactly that, giving `Error.new(msg)` a working 1-arg constructor and every
// subclass a `super.new(msg)` to route through (error-handling.md §1,
// U-ERR: `throw ArgumentError("age must be >= 0")`-style user `Error`
// subclasses need this to carry a real `message`).
@native
class Error is Object {
  @native message -> String
  @native raise() -> Dynamic
  _message // : Option<String>
  _kind // : Option<Symbol>
  _cause // : Option<Error>
  _displaced // : Option<?>

  // Preserved bare 0-arg form (many pre-U-ERR call sites — `Error.new()` in
  // Fiber/Future fixtures — rely on it; declaring *any* `new`-named
  // `@constructor` drops the generic inherited bare allocator, U7, so this must
  // be declared explicitly alongside the 1-arg form below, not left implicit).
  @constructor
  new() {
    _message = None
    _kind = None
    _cause = None
    _displaced = None
  }

  @constructor
  new(_ msg) {
    _message = msg
    _kind = None
    _cause = None
    _displaced = None
  }

  initialize(with msg, of kind, from cause, displaced) {
    _message = msg
    _kind = kind
    _cause = cause
    _displaced = displaced
  }

  kind { _kind }

  kind=(put value) { _kind = value }

  cause { _cause }

  cause=(put value) { _cause = value }

  displaced { _displaced }

  displaced=(put value) { _displaced = value }
}

@native
class MessageNotUnderstood is Error {}

@native
class CannotYieldAcrossNativeFrame is Error {}

@native
class UseAfterCloseError is Error {}

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

