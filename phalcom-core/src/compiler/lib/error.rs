use phalcom_ast::error::SyntaxError;
use phalcom_common::range::SourceRange;
use thiserror::Error;

/// Defensive compiler-side classification for malformed rest declarations.
///
/// Source parsing catches these first, but attribute expansion and compiler
/// synthesis also create AST members and therefore need a typed backstop.
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestDeclarationErrorKind {
    #[error("more than one positional rest parameter")]
    DuplicatePositional,
    #[error("more than one labeled rest parameter")]
    DuplicateLabeled,
    #[error("more than one complete rest parameter")]
    DuplicateComplete,
    #[error("complete rest cannot coexist with positional or labeled rest")]
    CompleteConflict,
    #[error("labeled and complete rest parameters must be terminal")]
    TerminalRestNotLast,
}

/// An error raised while lowering the AST to bytecode.
#[derive(Error, Debug, Clone)]
pub enum CompilerError {
    /// A source-level selector or send cannot be represented by bytecode's
    /// one-byte arity operand.
    #[error("{subject} has {found} arguments; bytecode supports at most {limit}")]
    ArityLimit {
        subject: &'static str,
        found: usize,
        limit: u8,
        span: SourceRange,
    },

    /// A catch-all for otherwise-unclassified compilation failures.
    #[error("Unknown error during compilation.")]
    Unknown,

    /// A reference to a variable the compiler cannot resolve.
    #[error("Undefined variable '{0}'.")]
    UndefinedVariable(String),

    /// An assignment whose left-hand side is not an assignable target.
    #[error("Invalid assignment target.")]
    InvalidAssignmentTarget,

    /// The implementation selector/field namespaces are reserved to the
    /// bootstrap core and compiler-generated runtime hooks.
    #[error("internal.namespace_reserved: '{0}' is reserved to the core/runtime implementation.")]
    InternalNamespaceReserved(String, SourceRange),

    /// A reassignment of a `const`-bound name (local, upvalue or global), or
    /// a captured write to a `const` through a closure (L-3).
    ///
    /// `const` bindings are immutable per
    /// [ADR-0064](../../../docs/adr/accepted/0064-let-const-bindings-and-field-mutability.md);
    /// only `let` bindings may be reassigned. The offending name is carried
    /// for the diagnostic.
    #[error("Cannot reassign immutable `const` binding '{0}'; declare it with `let` to allow mutation.")]
    AssignToImmutable(String),

    /// A `const` binding written without an initializer.
    ///
    /// `const x` with no `= expr` is rejected at compile time
    /// ([ADR-0064](../../../docs/adr/accepted/0064-let-const-bindings-and-field-mutability.md));
    /// an uninitialized binding must use `let x`, which reads the surface
    /// `None` value ([ADR-0007](../../../docs/adr/accepted/0007-option-type.md)).
    /// The offending name is carried for the diagnostic
    /// (`binding.const_requires_initializer`).
    #[error("`const` binding '{0}' requires an initializer; use `let {0}` for an uninitialized binding.")]
    ConstWithoutInitializer(String),

    /// A destructuring `let`/`const` pattern written without an initializer.
    ///
    /// A tuple or list [`Pattern`](phalcom_ast::ast::Pattern) has nothing to unpack from an absent
    /// value, so `let (a, b)` / `const [a, b]` with no `= expr` is rejected
    /// regardless of `let`/`const` (U14, open-questions.md Q7,
    /// [ADR-0046](../../../docs/adr/accepted/0046-destructuring-bindings.md)) — unlike
    /// a bare-name `let x`, which is allowed and reads `None`
    /// ([`CompilerError::ConstWithoutInitializer`]'s counterpart for the
    /// non-destructuring case). The [`SourceRange`] is the pattern's span.
    #[error("a destructuring `let`/`const` pattern requires an initializer to unpack.")]
    DestructuringWithoutInitializer(SourceRange),

    /// A field declared with the `let` keyword (`let _x`).
    ///
    /// Fields have exactly two spellings (L-2, ADR-0064 §3): bare `_x`
    /// (mutable) or `const _x` (immutable) — there is no third, keyworded
    /// mutable form. The offending field name is carried for the diagnostic
    /// (`field.no_mutable_keyword`).
    #[error("mutable fields take no keyword; write `{0}` instead of `let {0}`.")]
    FieldNoMutableKeyword(String),

    /// A write to a `const` field outside its class's constructor body.
    ///
    /// `const` field writes are legal only inside `construct` (ADR-0064 §3,
    /// L-3) — no flow analysis is performed, so this is keyed purely on which
    /// member the write syntactically appears in. The offending field name is
    /// carried for the diagnostic (`field.const_write`).
    #[error("cannot assign to `const` field '{0}' outside a constructor.")]
    ConstFieldWrite(String),

    /// A same-scope redeclaration of a `let` or `const` binding (L-3/L-5).
    ///
    /// One name, one declaration, per scope — for both binding kinds; a
    /// redeclaration cannot release a `const`'s immutability. Nested-scope
    /// shadowing is unaffected (`binding.redeclared`). The offending name is
    /// carried for the diagnostic.
    #[error("'{0}' is already declared in this scope; use assignment, or declare it in a nested scope to shadow.")]
    BindingRedeclared(String),

    /// A branch condition that is a syntactically detectable `Option` literal.
    ///
    /// `Option` has no truth value: `if (None)`, `if (Some(x))` and the
    /// like are compile errors, and any non-`Bool` condition is a hard runtime
    /// type error (no coercion). Reach through `.isSome`/`.isNone` or use
    /// `ifSome`/`ifNone` instead
    /// ([ADR-0007](../../../docs/adr/accepted/0007-option-type.md),
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
    /// ([ADR-0031](../../../docs/adr/accepted/0031-error-handling-surface-syntax.md) §1).
    /// The [`SourceRange`] is the offending literal's own span.
    #[error("`throw` of a non-`Error` literal is a compile error; only `Error` subclasses are throwable.")]
    ThrowNonError(SourceRange),

    /// A product literal whose runtime lowering is still deferred.
    ///
    /// A.1 preserves tuple and record syntax in the AST, but only the legacy
    /// positional tuple compatibility bridge lowers to bytecode here. Any
    /// empty, labeled, or record product literal reaches this explicit error
    /// until A.3 installs direct product construction.
    #[error("product literal lowering is not implemented yet.")]
    ProductLiteralNotLoweredYet(SourceRange),

    /// An `import` statement written anywhere other than a compilation
    /// unit's own top level.
    ///
    /// `import` resolves, loads and binds another `Module` (U15, DEC-U15);
    /// like `class`, it is a program-shape construct, not an ordinary
    /// statement — placing it inside a method/block/constructor/class body
    /// is rejected here rather than silently compiling a per-call reload.
    #[error("`import` is only allowed at a compilation unit's own top level, not inside a method, block, or class body.")]
    ImportNotAtTopLevel,

    /// A second `class X` declaration in the same module, or a `class`
    /// whose name collides with an `import … as Name` already bound in this
    /// unit (PDR-0001 ruling 2, PDR-0002, U-CLASSCLOSE §2.1/§8).
    ///
    /// Classes are closed after definition (PDR-0001): there is no
    /// reopening, so a second declaration of the same name in one module is
    /// always an error, never a merge. Carries **both** spans — this
    /// declaration's own and the original's — plus the original's
    /// pre-resolved 1-based `(line, column)`, since no compile-error
    /// renderer exists to resolve a span against source at print time
    /// (U-CLASSCLOSE §1.2/§3, ruled option A). The message states both
    /// locations directly rather than leaving the second span to be found by
    /// a future renderer.
    #[error("class.already_defined: class '{0}' is already defined in this module (first declared at {3}:{4}).")]
    ClassAlreadyDefined(String, SourceRange, SourceRange, usize, usize),

    /// Two post-expansion members install the same canonical selector on the
    /// same side (U-CTOR §3.2).
    #[error("class.duplicate_selector: '{1}' is already defined in class '{0}' (first declared at {4}:{5}).")]
    DuplicateSelector(String, String, SourceRange, SourceRange, usize, usize),

    /// Two field declarations collide in one class body (U-CTOR §3.2).
    #[error("class.duplicate_field: '{1}' is already defined in class '{0}' (first declared at {4}:{5}).")]
    DuplicateField(String, String, SourceRange, SourceRange, usize, usize),

    /// A kernel class name (e.g. `List`, `Object`, `Number` — the exact set
    /// `VM::install_core`'s `add_class!` binds) declared by a non-core
    /// module (PDR-0001 ruling 3, U-CLASSCLOSE §4).
    ///
    /// Module-scoped class identity alone would already make a user's own
    /// `List` a distinct, harmless local class — literals bind
    /// `universe.classes.list_class` by [`crate::heap::ClassId`], not by
    /// name — but "`class List` is silently not `List`" is a trap users
    /// would only discover at a confusing call site. Reserving the name
    /// makes the closed kernel a stateable rule instead of an emergent
    /// consequence of two other mechanisms.
    #[error("class.reserved_name: '{0}' is a kernel class name, reserved to the core module; declare a differently-named class instead.")]
    ClassReservedName(String, SourceRange),

    /// A full traversal whose unbounded source is proven from syntax and
    /// immutable binding facts (Spec E.3).
    #[error("cannot exhaust a provably unbounded source with `{operation}`")]
    ProvablyUnboundedExhaustion { operation: String, span: SourceRange },

    /// Two static call-site labels collide before any argument bytecode is
    /// emitted. Dynamic packs enforce the same invariant in their builder.
    #[error("duplicate argument label `{label}`")]
    DuplicateArgumentLabel {
        label: String,
        span: SourceRange,
        first_span: SourceRange,
    },

    /// A static product literal exceeds its independent u16 bytecode count.
    #[error("{subject} has {found} entries; bytecode supports at most {limit}")]
    ProductCountLimit {
        subject: &'static str,
        found: usize,
        limit: u16,
        span: SourceRange,
    },

    /// Static lowering was reached with an item that requires the F.2 dynamic
    /// pack lane. Legal source is routed before this diagnostic.
    #[error("internal compiler error: dynamic pack item reached static pack lowering")]
    PackExpansionNotYetSupported(SourceRange),

    /// Static lowering was reached with a computed label. Legal source uses
    /// the F.2 dynamic pack lane.
    #[error("internal compiler error: computed pack label reached static pack lowering")]
    ComputedLabelNotYetSupported(SourceRange),

    /// Constructor/factory and subscript rest capture is outside F.3's method
    /// body ABI. The compiler rejects it before selector creation/installation.
    #[error("rest parameters are not supported on constructors or subscript methods in F.3.")]
    RestModeUnsupportedForMember(SourceRange),

    /// A malformed rest declaration produced or preserved beyond parsing.
    #[error("invalid rest declaration: {kind}")]
    InvalidRestDeclaration {
        kind: RestDeclarationErrorKind,
        span: SourceRange,
    },

    /// Two structurally different rest selectors in one base family/class.
    #[error(
        "class.duplicate_rest_family: class '{class}' already defines rest family '{base}' \
         (first declared at {first_line}:{first_col})."
    )]
    DuplicateRestMethodFamily {
        class: String,
        base: String,
        span: SourceRange,
        first_span: SourceRange,
        first_line: usize,
        first_col: usize,
    },
}

/// Converts an AST-sourced arity to the representation used by selectors and
/// send bytecodes, preserving the original count for diagnostics.
pub(crate) fn checked_send_arity(subject: &'static str, found: usize, span: SourceRange) -> Result<u8, CompilerError> {
    u8::try_from(found).map_err(|_| CompilerError::ArityLimit {
        subject,
        found,
        limit: u8::MAX,
        span,
    })
}

pub(crate) fn checked_product_count(subject: &'static str, found: usize, span: SourceRange) -> Result<u16, CompilerError> {
    u16::try_from(found).map_err(|_| CompilerError::ProductCountLimit {
        subject,
        found,
        limit: u16::MAX,
        span,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_product_count_preserves_the_u16_boundary() {
        assert_eq!(checked_product_count("Tuple", u16::MAX as usize, (0..0).into()).unwrap(), u16::MAX);
        assert!(matches!(
            checked_product_count("Tuple", u16::MAX as usize + 1, (0..0).into()),
            Err(CompilerError::ProductCountLimit {
                found,
                limit: u16::MAX,
                ..
            }) if found == u16::MAX as usize + 1
        ));
    }
}
