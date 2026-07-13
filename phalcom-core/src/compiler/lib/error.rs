use phalcom_ast::error::SyntaxError;
use phalcom_common::range::SourceRange;
use thiserror::Error;

/// An error raised while lowering the AST to bytecode.
#[derive(Error, Debug, Clone)]
pub enum CompilerError {
    /// A catch-all for otherwise-unclassified compilation failures.
    #[error("Unknown error during compilation.")]
    Unknown,

    /// A reference to a variable the compiler cannot resolve.
    #[error("Undefined variable '{0}'.")]
    UndefinedVariable(String),

    /// An assignment whose left-hand side is not an assignable target.
    #[error("Invalid assignment target.")]
    InvalidAssignmentTarget,

    /// A reassignment of a `let`-bound name (local, upvalue or global).
    ///
    /// `let` bindings are immutable per
    /// [ADR-0014](../../../docs/adr/0014-let-var-bindings.md); only `var`
    /// bindings may be reassigned. The offending name is carried for the
    /// diagnostic.
    #[error("Cannot reassign immutable `let` binding '{0}'; declare it with `var` to allow mutation.")]
    AssignToImmutable(String),

    /// A `let` binding written without an initializer.
    ///
    /// `let x` with no `= expr` is rejected at compile time
    /// ([ADR-0014](../../../docs/adr/0014-let-var-bindings.md)); an
    /// uninitialized binding must use `var x`, which reads the surface `None`
    /// value ([ADR-0007](../../../docs/adr/0007-option-type.md)). The offending
    /// name is carried for the diagnostic.
    #[error("`let` binding '{0}' requires an initializer; use `var {0}` for an uninitialized binding.")]
    LetWithoutInitializer(String),

    /// A destructuring `let`/`var` pattern written without an initializer.
    ///
    /// A tuple or list [`Pattern`] has nothing to unpack from an absent
    /// value, so `let (a, b)` / `var [a, b]` with no `= expr` is rejected
    /// regardless of `let`/`var` (U14, open-questions.md Q7,
    /// [ADR-0046](../../../docs/adr/0046-destructuring-bindings.md)) — unlike
    /// a bare-name `var x`, which is allowed and reads `None`
    /// ([`CompilerError::LetWithoutInitializer`]'s counterpart for the
    /// non-destructuring case). The [`SourceRange`] is the pattern's span.
    #[error("a destructuring `let`/`var` pattern requires an initializer to unpack.")]
    DestructuringWithoutInitializer(SourceRange),

    /// A branch condition that is a syntactically detectable `Option` literal.
    ///
    /// `Option` has no truth value: `if (None)`, `if (Some.new(x))` and the
    /// like are compile errors, and any non-`Bool` condition is a hard runtime
    /// type error (no coercion). Reach through `.isSome`/`.isNone` or use
    /// `ifSome`/`ifNone` instead
    /// ([ADR-0007](../../../docs/adr/0007-option-type.md),
    /// values-and-absence §3.5; BD-U6-1 enforcement = typed branch +
    /// literal-only compile check).
    #[error("An `Option` value has no truth value; use `.isSome`/`.isNone` or `ifSome`/`ifNone` instead of a boolean condition.")]
    OptionTruthiness,

    /// A syntax error surfaced from the front-end parser.
    #[error(transparent)]
    Parse(#[from] SyntaxError),

    /// A free-form compiler diagnostic.
    #[error("{0}")]
    Message(String),

    /// A field read whose name is in no assignment set in the class (ADR-0011).
    #[error("Read-before-write: field '{0}' is used before being assigned anywhere in this class.")]
    ReadBeforeWrite(String),

    /// An explicit value returned from a construct initializer.
    #[error("Cannot return a value from an initializer.")]
    ReturnValueFromInitializer,

    /// A `super.sel(…)` send written where there is no enclosing class body to
    /// anchor the walk (top level, or a free function).
    ///
    /// `super` starts method lookup at the *superclass of the defining class*
    /// (method-lookup.md §1.14, U-INH §3.4); with no defining class there is no
    /// superclass to start from.
    #[error("`super` cannot be used outside a method: there is no defining class to start the lookup above.")]
    SuperOutsideMethod,

    /// A bare `super` that is not the receiver of a message send.
    ///
    /// `super` is only meaningful as `super.sel(…)` — it names the current
    /// receiver but redirects the lookup start, so it has no value on its own
    /// (U-INH §3.4). `super` no longer silently evaluates to `nil`.
    #[error("`super` may only be used as the receiver of a message send, e.g. `super.method(...)`.")]
    BareSuper,

    /// A `break` written outside any enclosing loop.
    ///
    /// `break`/`continue` are resolved lexically against the compiler's
    /// loop-context stack (ADR-0035 §3, iteration.md §3, U-ITER specification
    /// §4); with the stack empty there is no loop to leave, so this is a
    /// compile error (C-ITER-7). The [`SourceRange`] is the offending keyword's
    /// span.
    #[error("`break` outside of a loop: `break` may only appear inside a `for` loop body.")]
    BreakOutsideLoop(SourceRange),

    /// A `continue` written outside any enclosing loop.
    ///
    /// The `continue` counterpart of [`CompilerError::BreakOutsideLoop`]
    /// (ADR-0035 §3, C-ITER-7). The [`SourceRange`] is the offending keyword's
    /// span.
    #[error("`continue` outside of a loop: `continue` may only appear inside a `for` loop body.")]
    ContinueOutsideLoop(SourceRange),

    /// A `throw` of a syntactically-detectable non-`Error` literal.
    ///
    /// `throw "oops"`, `throw 42`, `throw true` are compile errors
    /// ([error-handling.md §1](../../../docs/spec/v0.2/error-handling.md)):
    /// only `Error` and its subclasses respond to `raise()`, and a literal's
    /// non-`Error`-ness is provable without flow typing. A `throw someVariable`
    /// cannot be statically classified and defers to the runtime
    /// `doesNotUnderstand` miss on `raise()` instead
    /// ([ADR-0031](../../../docs/adr/0031-error-handling-surface-syntax.md) §1).
    /// The [`SourceRange`] is the offending literal's own span.
    #[error("`throw` of a non-`Error` literal is a compile error; only `Error` subclasses are throwable.")]
    ThrowNonError(SourceRange),

    /// An `import` statement written anywhere other than a compilation
    /// unit's own top level.
    ///
    /// `import` resolves, loads and binds another `Module` (U15, DEC-U15);
    /// like `class`, it is a program-shape construct, not an ordinary
    /// statement — placing it inside a method/block/constructor/class body
    /// is rejected here rather than silently compiling a per-call reload.
    #[error("`import` is only allowed at a compilation unit's own top level, not inside a method, block, or class body.")]
    ImportNotAtTopLevel,
}
