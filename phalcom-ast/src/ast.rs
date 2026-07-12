//! Abstract syntax tree produced by [`crate::parser`].
//!
//! These node types are the parser's output and the compiler's input. Every
//! node carries a [`SourceRange`] so later phases can attach precise
//! diagnostics. Surface absence is modelled as the `Option` type rather than a
//! `nil` literal (ADR-0007): there is deliberately no `Expr::Nil` variant, and
//! `??`/`?.` are desugared to ordinary `Option` message sends by the parser
//! (see [`crate::parser`]).

use phalcom_common::range::SourceRange;

#[derive(Debug, Default)]
pub struct Module {
    pub program: Program,
    pub range: SourceRange,
}

#[derive(Debug, Default)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Class(ClassDef),
    Let(LetBinding),
    Return(ReturnStatement),
    Expr { expr: Expr, range: SourceRange },
    /// `for (binding in iter) { body }` — the cursor loop (ADR-0035 §2,
    /// iteration.md §2). Lowered by the compiler to an inlined cursor `while`
    /// over the two-selector protocol `iterate(_)` / `iteratorValue(_)` —
    /// **never** to `coll.each { … }` (ADR-0035 §2), so a `for` body inside a
    /// fiber can `yield` freely. `for` is a statement consumed for effect; its
    /// value is unspecified (see the U-ITER specification §1.2).
    For(ForStatement),
    /// `break` — leave the innermost enclosing `for` loop (ADR-0035 §3,
    /// iteration.md §3). Resolved lexically against the compiler's
    /// loop-context stack; a `break` outside any loop is a compile error. It
    /// carries no operand and no label (unlabelled form only).
    Break {
        /// The source span of the `break` keyword, for the out-of-loop
        /// compile-error diagnostic.
        range: SourceRange,
    },
    /// `continue` — jump to the innermost enclosing loop's cursor-step, so the
    /// next `iterate(_)` runs (ADR-0035 §3, iteration.md §3). Resolved
    /// lexically like [`Statement::Break`]; a `continue` outside any loop is a
    /// compile error. Carries no operand and no label.
    Continue {
        /// The source span of the `continue` keyword, for the out-of-loop
        /// compile-error diagnostic.
        range: SourceRange,
    },
    /// `throw expr` — unwind the stack raising `expr`
    /// ([error-handling.md §1](../../../docs/spec/v0.2/error-handling.md),
    /// [ADR-0031](../../../docs/adr/0031-error-handling-surface-syntax.md) §1).
    /// Surface sugar for `expr.raise()`; the compiler additionally rejects a
    /// syntactically-detectable non-`Error` literal operand at compile time
    /// (error-handling.md §1: "`throw "oops"` is a compile error") — a
    /// non-literal operand (a variable, a user `Error` construction) cannot be
    /// statically classified and defers to the runtime `doesNotUnderstand` miss
    /// on `raise()` instead.
    Throw {
        /// The raised expression, evaluated then sent `raise()`.
        expr: Expr,
        /// The source span of the whole `throw` statement.
        range: SourceRange,
    },
    /// `import "path" as Name` — resolves, loads and binds another
    /// compilation unit as a first-class [`Module`](../../../phalcom-core/src/module.rs)
    /// namespace (U15, DEC-U15 A+A: relative file-path resolution + whole-
    /// module binding, [object-model.md §4](../../../docs/spec/v0.2/object-model.md)).
    ///
    /// Only valid at a compilation unit's own top level (never inside a
    /// method/block/class body) — the compiler rejects any other placement.
    /// There is deliberately no bare `import "path"` (no binding) and no
    /// selective `import { a, b } from "path"` form in Draft 0.1 (DEC-U15);
    /// `from` is reserved for that future selective-import grammar.
    Import(ImportStatement),
}

/// `import "path" as Name` (U15, DEC-U15 A+A).
///
/// `path` is resolved relative to the *importing file's* directory with a
/// `.ph` extension implied (DEC-U15 resolution choice A); `binding` is the
/// name the imported unit's [`Module`](../../../phalcom-core/src/module.rs)
/// is bound to in the importing scope — an ordinary global, not a namespace
/// merge (no global-namespace pollution, U15 plan §1). `as Name` is
/// mandatory in Draft 0.1: whole-module binding only (DEC-U15 binding choice
/// A), no selective form yet.
#[derive(Debug, Clone)]
pub struct ImportStatement {
    /// The literal path string as written, e.g. `"./geometry/point"`.
    pub path: String,
    /// The local name the imported `Module` is bound to (`as Name`).
    pub binding: String,
    /// The source span of the whole `import` statement.
    pub range: SourceRange,
}

// Note: `try { P } (on T e { … })* (catch e { … })? (ensure { … })?`
// (error-handling.md §2, ADR-0031 §3) has **no** dedicated `Statement`
// variant. The parser desugars it directly to nested [`Expr::MethodCall`]/
// [`Expr::Block`] sends at parse time — the exact shape [`Statement::Expr`]
// would hold for a hand-written `{ P }.on(T){e=>…}.ensure{…}` chain — mirroring
// how `if`/`while` desugar to sends in [`crate::parser`] (U5) rather than
// carrying their own AST node. See `Parser::parse_try`'s doc for the nested
// re-wrapping desugar.

#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    /// The explicit superclass named by an `extends` clause, if any.
    ///
    /// `None` means no `extends` clause was written, so the class implicitly
    /// inherits from `Object` (object-model.md §5.1). `Some(_)` carries the
    /// superclass identifier and its span for compile-time resolution and
    /// diagnostics (U-INH, DEC-INH-A: `extends` is a contextual keyword
    /// recognised only in this class-header position).
    pub superclass: Option<SuperclassRef>,
    pub members: Vec<ClassMember>,
    pub range: SourceRange,
}

/// A reference to a superclass named in a `class N extends S { … }` header.
///
/// Holds the raw identifier `S` and its source span. The name is resolved to a
/// class value at compile time as an ordinary global lookup (object-model.md
/// §5.1, U-INH); the span anchors an "unknown superclass" / self-inheritance
/// diagnostic.
#[derive(Debug, Clone)]
pub struct SuperclassRef {
    /// The superclass identifier as written in source.
    pub name: String,
    /// The source span of the identifier, for diagnostics.
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub enum ClassMember {
    Method(MethodDef),
    Getter(GetterDef),
    Setter(SetterDef),
    Construct(ConstructDef),
}

#[derive(Debug, Clone)]
pub struct ConstructDef {
    pub name: String,
    pub params: Vec<ParameterDef>,
    pub body: Vec<Statement>,
    pub range: SourceRange,
}

/// A single parameter in a method/constructor parameter list.
///
/// See `messages-and-selectors.md` §4 for the surface grammar: an ordinary
/// positional parameter is a bare identifier, a labeled (keyword) parameter is
/// `name:`, and a rest parameter is `*name` (U9, encoded on the runtime side
/// as `SignatureKind::Variadic` in `phalcom-core`).
#[derive(Debug, Clone)]
pub struct ParameterDef {
    /// The parameter's local binding name.
    pub name: String,
    /// The keyword label this parameter is called under, if any (`name:` in
    /// source). `None` for an ordinary positional parameter.
    pub label: Option<String>,
    /// Whether this is the rest parameter (`*name`), collecting every
    /// trailing positional argument into a single `List` (U9). At most one
    /// parameter per list may set this, and only as the list's last entry
    /// (enforced by the parser); it may not also carry a [`label`](Self::label).
    pub is_rest: bool,
    /// The parameter's source span.
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct MethodDef {
    pub name: String,
    pub params: Vec<ParameterDef>,
    pub body: Vec<Statement>,
    pub is_static: bool,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct GetterDef {
    pub name: String,
    pub body: Vec<Statement>,
    pub is_static: bool,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct SetterDef {
    pub name: String,
    pub param: String,
    pub body: Vec<Statement>,
    pub is_static: bool,
    pub range: SourceRange,
}

/// Whether a binding is immutable (`let`) or mutable (`var`).
///
/// Per ADR-0014: a `let` binding cannot be reassigned (reassignment is a
/// compile error) and requires an initializer, whereas a `var` binding is
/// mutable and may be declared without one — an uninitialized `var` reads the
/// surface `None` value (ADR-0007). Enforcement of these rules lives in the
/// compiler; the AST only records which form was written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    /// An immutable `let` binding.
    Let,
    /// A mutable `var` binding.
    Var,
}

/// A `let`/`var` binding, optionally with an initializer.
///
/// The [`kind`](LetBinding::kind) distinguishes the immutable `let` form from
/// the mutable `var` form (ADR-0014). A missing [`value`](LetBinding::value)
/// means no initializer was written; the compiler rejects that for `let` and
/// surfaces `None` for `var`. The [`pattern`](LetBinding::pattern) is either a
/// bare name (the pre-U14 case) or a destructuring [`Pattern`] — a tuple or
/// list pattern that positionally unpacks the initializer (open-questions.md
/// Q7, [ADR-0046](../../../docs/adr/0046-destructuring-bindings.md)). A
/// destructuring `pattern` always requires an initializer, regardless of
/// `kind` — there is nothing to unpack from an absent value.
#[derive(Debug, Clone)]
pub struct LetBinding {
    /// Whether this is an immutable `let` or a mutable `var` binding.
    pub kind: BindingKind,
    /// The bound name or destructuring pattern.
    pub pattern: Pattern,
    /// The initializer expression, or `None` if the binding has no `= expr`.
    pub value: Option<Expr>,
    /// The source span covering the whole binding statement.
    pub range: SourceRange,
}

/// A `let`/`var` binding's left-hand side — a bare name or a destructuring
/// pattern (open-questions.md Q7, U14,
/// [ADR-0046](../../../docs/adr/0046-destructuring-bindings.md)).
///
/// Patterns nest recursively (`let ((a, b), c) = …`), reusing the collection
/// literal's `(…)`/`[…]` delimiters in binding-target position — a grammar
/// path distinct from an RHS tuple/list *literal* (`Self::parse_paren_or_tuple`
/// vs the pattern parser in `crate::parser`), so `(a, b)` never parses
/// ambiguously between the two positions.
///
/// Every destructuring binding compiles to a single evaluation of the
/// initializer followed by positional element reads through the *same*
/// `at(_)` selector `List`/`Tuple` already expose (ADR-0020) — there is no
/// separate `_0`/`_1` accessor protocol. The bind is **irrefutable**: a shape
/// mismatch (wrong arity) raises a runtime error rather than failing silently
/// or falling back — see ADR-0046 §2. This node is intentionally reused by a
/// future refutable `match`/`if let` (ADR-0046 §4): the "raise on mismatch"
/// behavior lives in the compiler's lowering, not in this node's shape.
#[derive(Debug, Clone)]
pub enum Pattern {
    /// A single bound name — the non-destructuring case (`let x = …`).
    Name {
        /// The bound name.
        name: String,
        /// The source span of the name.
        range: SourceRange,
    },
    /// A tuple pattern `(p1, …, pn)` — binds against a `Tuple` of the exact
    /// same arity; any other arity is a runtime error (ADR-0046 §2).
    Tuple {
        /// The sub-patterns, one per tuple slot, in order.
        elements: Vec<Pattern>,
        /// The source span of the whole `(…)` pattern.
        range: SourceRange,
    },
    /// A list pattern `[p1, …, pn]`, or `[p1, …, pn, *rest]` with a trailing
    /// rest sub-pattern (U9's `*name` spelling reused verbatim —
    /// messages-and-selectors.md §5 spread parity). A rest sub-pattern, if
    /// present, **must be the pattern's last element**; the parser rejects an
    /// interior `*` (mirrors [`ParameterDef::is_rest`]'s "last parameter"
    /// rule).
    List {
        /// The fixed leading sub-patterns, in order.
        elements: Vec<Pattern>,
        /// The trailing `*rest` sub-pattern, or `None` if the list pattern has
        /// no rest. When present, the scrutinee must have **at least**
        /// `elements.len()` items; the rest sub-pattern binds a fresh `List`
        /// holding everything from index `elements.len()` onward.
        rest: Option<Box<Pattern>>,
        /// The source span of the whole `[…]` pattern.
        range: SourceRange,
    },
}

impl Pattern {
    /// Returns this pattern's source span.
    pub fn range(&self) -> SourceRange {
        match self {
            Pattern::Name { range, .. } | Pattern::Tuple { range, .. } | Pattern::List { range, .. } => *range,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub value: Option<Expr>,
    pub range: SourceRange,
}

/// A `for (binding in iter) { body }` cursor loop (ADR-0035 §2,
/// iteration.md §2).
///
/// The parser produces this node directly (unlike `while`, which desugars to a
/// `whileTrue` [`Expr::MethodCall`] at parse time); the compiler lowers it to
/// an inlined cursor `while` driving the `iterate(_)` / `iteratorValue(_)`
/// protocol sends, evaluating [`iter`](ForStatement::iter) exactly once. See
/// the U-ITER specification §3 for the exact desugar.
#[derive(Debug, Clone)]
pub struct ForStatement {
    /// The loop variable, freshly bound to each element in turn (behaves as a
    /// `let` per iteration; iteration.md §2).
    pub binding: String,
    /// The iterable expression, evaluated **once** into a synthetic temporary
    /// before the loop (U-ITER specification §3.3).
    pub iter: Expr,
    /// The loop body statements, run once per element under jump-based control
    /// flow (no `block_call` on the taken path, ADR-0035 §2).
    pub body: Vec<Statement>,
    /// The source span covering the whole `for` statement.
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number { value: f64, range: SourceRange },
    String { value: String, range: SourceRange },
    Boolean { value: bool, range: SourceRange },
    Var { value: String, range: SourceRange },
    Field { value: String, range: SourceRange },
    SelfVar { range: SourceRange },
    SuperVar { range: SourceRange },
    Assignment(Box<AssignmentExpr>),
    Unary(Box<UnaryExpr>),
    Binary(Box<BinaryExpr>),
    MethodCall(Box<MethodCallExpr>),
    GetProperty(Box<GetPropertyExpr>),
    SetProperty(Box<SetPropertyExpr>),
    Block(Box<BlockExpr>),
    /// A `::` method reference — `receiver::name` (selectors.md §3, U16-Open,
    /// Open form only). See [`MethodRefExpr`].
    MethodRef(Box<MethodRefExpr>),
}

impl Expr {
    /// Returns this expression's source span.
    ///
    /// Added for U5's `if`/`while` desugaring (`phalcom-ast/src/parser.rs`),
    /// which needs a uniform way to span-wrap an arbitrary sub-expression
    /// (e.g. an `else if`'s nested [`Expr::MethodCall`]) into a synthetic
    /// block literal.
    pub fn range(&self) -> SourceRange {
        match self {
            Expr::Number { range, .. }
            | Expr::String { range, .. }
            | Expr::Boolean { range, .. }
            | Expr::Var { range, .. }
            | Expr::Field { range, .. }
            | Expr::SelfVar { range }
            | Expr::SuperVar { range } => *range,
            Expr::Assignment(e) => e.range,
            Expr::Unary(e) => e.range,
            Expr::Binary(e) => e.range,
            Expr::MethodCall(e) => e.range,
            Expr::GetProperty(e) => e.range,
            Expr::SetProperty(e) => e.range,
            Expr::Block(e) => e.range,
            Expr::MethodRef(e) => e.range,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub left: Expr,
    pub right: Expr,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct SetPropertyExpr {
    pub object: Expr,
    pub property: String,
    pub value: Expr,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct AssignmentExpr {
    pub name: Box<Expr>,
    pub value: Expr,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Expr,
    pub args: Vec<Expr>,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub label: Option<String>,
    pub expr: Expr,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub object: Expr,
    pub method: String,
    pub args: Vec<Argument>,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct GetPropertyExpr {
    pub object: Expr,
    pub property: String,
    pub range: SourceRange,
}

/// A method reference expression, `receiver::name` (selectors.md §3,
/// U16-Open — the **Open** form only; the Pinned `recv::#sel(...)` form is
/// deferred to U-LEX-HASH, which needs `#`-symbol-literal lexing).
///
/// Both `obj::name` and `Type::name` parse to this node — `receiver` is
/// whatever postfix expression preceded `::`, evaluated and bound as the
/// resulting [`Family`](https://en.wikipedia.org/wiki/Message_passing)
/// value's receiver at reference time. There is no unbound form in this
/// unit's scope: the compiled `Family` always carries a concrete receiver
/// value (`phalcom-core::heap::Object::Family`).
#[derive(Debug, Clone)]
pub struct MethodRefExpr {
    /// The expression evaluated to produce the family's bound receiver.
    pub receiver: Expr,
    /// The base method name after `::` (not a full selector — the call-time
    /// selector is built from this name plus the call site's argument
    /// labels, selectors.md §3 "Open families resolve at call time").
    pub name: String,
    /// The source span from the start of `receiver` through the name.
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Expr,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone)]
pub struct BlockExpr {
    pub params: Vec<String>,
    pub body: Vec<Statement>,
    pub expr_body: bool,
    pub range: SourceRange,
}
