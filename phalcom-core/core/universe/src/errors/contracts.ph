// Design-by-Contract failure classes (U-ANNOT-CONTRACTS,
// `docs/spec/v0.2/experimental/annotations-contracts.md`). Each is raised by
// the woven `@requires`/`@ensures`/`@invariant` check emitted by
// `phalcom-core/src/compiler/attributes.rs`'s `build_check_stmt` — zero Rust,
// zero new primitive, ordinary `Error` subclasses that inherit `message`/
// `raise` from the `_message`-slot machinery above.
class PreconditionError is Error {}

class PostconditionError is Error {}

class InvariantError is Error {}

// Boundary-guard exception (error-handling.md §1, U-STRING). Raised by library
// code to indicate invalid argument values or arities. Zero fields — the
// inherited `Error` `@constructor new(msg)` gives `ArgumentError.new(msg)` a working
// constructor for `throw ArgumentError.new("msg")` sites (U-INH inherited-ctor
// resolution).
