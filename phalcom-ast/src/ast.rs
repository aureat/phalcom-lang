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
    /// [ADR-0031](../../../docs/adr/accepted/0031-error-handling-surface-syntax.md) §1).
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
    /// `@name(args…)` attributes attached directly to the class header
    /// (e.g. a future class-level decorator), distinct from
    /// [`invariants`](Self::invariants) — `@invariant` is a parse-time
    /// carve-out that is *never* collected here (see
    /// [`Attribute`]/DEC-ANNOT-B in `annotations-legality-grammar.md`).
    pub attributes: Vec<Attribute>,
    /// Standalone `@invariant(pred)` class-body predicates, in declaration
    /// order, paired with each predicate's own source span.
    ///
    /// Per DEC-ANNOT-B (`docs/spec/v0.2/experimental/annotations-contracts.md`,
    /// `docs/forge/units/U-ANNOT-CONTRACTS/plan.md` §3.1), `@invariant` is a
    /// one-off parse-time exception to the normal "attribute binds to the
    /// following member" rule: it stands alone in the class body with no
    /// following member to attach to, so the parser routes it here directly
    /// instead of into a [`ClassMember`]'s [`Attribute`] list. The compiler
    /// conjoins these in order into one synthesized `__check_invariant()`
    /// method, woven receiver-scoped per
    /// [ADR-0052](../../../docs/adr/accepted/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md).
    pub invariants: Vec<(Expr, SourceRange)>,
    pub range: SourceRange,
    /// The source span of just the class name identifier (e.g. the `Foo` in
    /// `class Foo extends Bar { … }`), distinct from [`Self::range`]'s
    /// whole-declaration span. Lets a downstream consumer (e.g.
    /// `phalcom-lsp`'s `semanticTokens/full` pass) highlight the declared
    /// name itself with a `class`-token type without heuristically
    /// re-scanning `range` for the first matching identifier — a prior
    /// attempt at that heuristic was rejected as unsound (nothing guarantees
    /// the first occurrence of `name` inside `range` is the declaration
    /// token rather than an incidental earlier one).
    pub name_range: SourceRange,
}

/// A `@name(args…)` attribute attached to a class or class member.
///
/// Attributes bind to the *following* class member by default (the parser's
/// newline-tolerant attribute-collection loop, `crate::parser`'s
/// `parse_class_body`); the sole exception is a
/// standalone `@invariant(...)`, which is diverted at parse time into
/// [`ClassDef::invariants`] instead of ever becoming an `Attribute` here (see
/// that field's doc and DEC-ANNOT-B). The compiler resolves `name` against a
/// registry of [`AttributeExpander`](../../../phalcom-core/src/compiler/attributes.rs)
/// implementations (`docs/spec/v0.2/experimental/annotations-core.md`,
/// `annotations-legality-grammar.md`) that desugar it into ordinary AST before
/// the rest of compilation runs.
/// Builtin attribute variants recognized by the compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinAttr {
    Construct,
    Constructor,
    Class,
    Get,
    Set,
    Data,
    Sealed,
    Variant,
    Invariant,
    Requires,
    Ensures,
    On,
    Native,
    Ignore,
}

impl BuiltinAttr {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltinAttr::Construct => "construct",
            BuiltinAttr::Constructor => "constructor",
            BuiltinAttr::Class => "class",
            BuiltinAttr::Get => "get",
            BuiltinAttr::Set => "set",
            BuiltinAttr::Data => "data",
            BuiltinAttr::Sealed => "sealed",
            BuiltinAttr::Variant => "variant",
            BuiltinAttr::Invariant => "invariant",
            BuiltinAttr::Requires => "requires",
            BuiltinAttr::Ensures => "ensures",
            BuiltinAttr::On => "On",
            BuiltinAttr::Native => "native",
            BuiltinAttr::Ignore => "ignore",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "construct" => Some(BuiltinAttr::Construct),
            "constructor" => Some(BuiltinAttr::Constructor),
            "class" => Some(BuiltinAttr::Class),
            "get" => Some(BuiltinAttr::Get),
            "set" => Some(BuiltinAttr::Set),
            "data" => Some(BuiltinAttr::Data),
            "sealed" => Some(BuiltinAttr::Sealed),
            "variant" => Some(BuiltinAttr::Variant),
            "invariant" => Some(BuiltinAttr::Invariant),
            "requires" => Some(BuiltinAttr::Requires),
            "ensures" => Some(BuiltinAttr::Ensures),
            "On" => Some(BuiltinAttr::On),
            "native" => Some(BuiltinAttr::Native),
            "ignore" => Some(BuiltinAttr::Ignore),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrKind {
    Builtin(BuiltinAttr),
    User(String),
}

impl AttrKind {
    pub fn name(&self) -> &str {
        match self {
            AttrKind::Builtin(b) => b.name(),
            AttrKind::User(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub kind: AttrKind,
    pub name: String,
    pub args: Vec<Expr>,
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
    /// A declared field (`let`/`var` at class-body position, U-ANNOT-LAYOUT
    /// §3.1). See [`FieldDef`].
    Field(FieldDef),
    /// A `@variant Name(labels...)` arm inside a `@sealed` class body
    /// (U-ANNOT-LAYOUT §3.4, `annotations-data.md` §"`@variant`"). See
    /// [`VariantDef`]. Never compiled directly — stripped and expanded into a
    /// sibling top-level [`Statement::Class`] by `phalcom-core`'s
    /// `compiler::attributes::expand_class_attributes`.
    Variant(VariantDef),
    /// A bracket-delimited subscript method — `[idx] { ... }` (read) or
    /// `[idx, put:] { ... }` (write), U-INDEX,
    /// [ADR-0060](../../../docs/adr/accepted/0060-index-operator-as-real-selector.md).
    /// See [`IndexMethodDef`].
    Index(IndexMethodDef),
}

/// A bracket-delimited subscript method definition — `[idx] { ... }` /
/// `[idx, put:] { ... }` / `[] { ... }` / `[put:] { ... }` (U-INDEX,
/// [ADR-0060](../../../docs/adr/accepted/0060-index-operator-as-real-selector.md)).
///
/// Unlike every other [`ClassMember`], this selector carries no separate name
/// token at all — the brackets themselves are the whole of the member's
/// grammar, and `params` (parsed by the ordinary `Parser::parse_param_list`,
/// substituting `[`/`]` for `(`/`)`) live *inside* them rather than in a
/// following `(...)` slot.
/// Identity is arity + labels alone: `phalcom-core`'s
/// `SignatureKind::Subscript` encodes this to `[_]`, `[_,put]`, `[]`,
/// `[put]`, etc., mirroring `comma_form_slots` but bracket-delimited instead
/// of `name(...)`-delimited.
#[derive(Debug, Clone)]
pub struct IndexMethodDef {
    /// The bracketed parameter list — positional and/or `label:` parameters,
    /// in source order (e.g. `[idx, put:]`'s `idx` positional, `put`
    /// labeled).
    pub params: Vec<ParameterDef>,
    /// The method body.
    pub body: Vec<Statement>,
    /// `@name(args…)` attributes attached to this member, in declaration
    /// order. See [`MethodDef::attributes`].
    pub attributes: Vec<Attribute>,
    /// The source span of the whole declaration.
    pub range: SourceRange,
    /// The source span of just the `[...]` bracket-and-params portion,
    /// distinct from [`Self::range`]'s whole-declaration span. See
    /// [`ClassDef::name_range`]'s doc for why this exists as its own field.
    pub name_range: SourceRange,
}

/// A single `@variant Name(label1:, label2:, ...)` arm declared inside a
/// `@sealed` class body (U-ANNOT-LAYOUT §3.4, `annotations-data.md`
/// §"`@variant`").
///
/// Distinct grammar from every other [`ClassMember`]: `Name(labels...)` has
/// no body and no expression value per label — each label is a bare
/// identifier followed by `:` (no type, no default), naming one field the
/// generated sibling class will carry. The `@variant` [`Attribute`] itself is
/// always present in [`Self::attributes`] (it is what tells the parser's
/// `parse_class_body` to parse this production instead of an ordinary
/// member); a bare `Name(labels...)` with no `@variant` prefix is not valid
/// grammar and never produces this node.
#[derive(Debug, Clone)]
pub struct VariantDef {
    /// The variant's name (e.g. `"Circle"`) — becomes the generated sibling
    /// class's name, an ordinary **global** class name (Draft 0.1 has no
    /// nested/namespaced variant naming, `annotations-data.md`'s own
    /// simplification).
    pub name: String,
    /// The declared field labels, in declaration order (e.g. `["radius"]`
    /// for `@variant Circle(radius:)`) — R3, field order is API. Each label
    /// becomes one `FieldDef` (named `"_" + label`) on the generated sibling
    /// class, and one keyword-labeled parameter on the enclosing sealed
    /// class's generated `match(...)` visitor.
    pub labels: Vec<String>,
    /// `@name(args…)` attributes attached to this variant declaration — in
    /// practice always exactly one entry, the `@variant` attribute itself.
    pub attributes: Vec<Attribute>,
    /// The source span of the whole `@variant Name(labels...)` declaration.
    pub range: SourceRange,
}

/// A declared class field: `let _name [= expr]` or `var _name [= expr]` at
/// class-body position (`docs/spec/v0.2/experimental/annotations-construct.md`
/// "Prerequisite 1", U-ANNOT-LAYOUT §3.1).
///
/// Distinct from [`LetBinding`] (the statement-position form of `let`/`const`,
/// ADR-0064) purely by parse *position* — the parser
/// disambiguates in [`crate::parser`]'s `parse_class_member`, which only ever
/// runs inside a class body, so no lookahead ambiguity exists.
///
/// **Field order is API** (R3, `selectors.md` §1): `FieldDef`s appear in
/// [`ClassMember`] order exactly as written, and this order is what the
/// layout-derive attributes (`@construct`, `@data`, once built) key their
/// generated parameter lists off. A class using at least one `FieldDef`
/// switches its whole instance-field layout onto this declared list; the
/// legacy implicit-by-assignment inference is skipped entirely for that class
/// (U-ANNOT-LAYOUT §3 "Rubric" hazard, DEC-ANNOT-H) — mixing declared and
/// inferred fields within one class is unsupported.
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// The field's name, as written (conventionally leading-underscore,
    /// e.g. `"_x"`, though the grammar does not enforce this).
    pub name: String,
    /// Whether the field was declared *without* `const` (`true`, mutable) or
    /// with `const` (`false`, immutable) — ADR-0064's field mutability
    /// distinction (L-2). Mutable fields take no keyword at all (`_x`, not
    /// `let _x`); only `const _x` spells the immutable form.
    pub mutable: bool,
    /// Whether field storage belongs to declaring class object (`@class`).
    pub is_static: bool,
    /// The field's default-value expression (`= expr`), or `None` if the
    /// field was declared with no initializer. A layout-derive attribute
    /// (e.g. `@construct`) that omits a defaulted field from its generated
    /// parameter list evaluates this expression per instance, at
    /// construct time, before the derived body's own labeled-parameter
    /// assignments (`annotations-construct-inheritance.md`'s "supply-and-
    /// default is mutually exclusive per field").
    pub default: Option<Expr>,
    /// `@name(args…)` attributes attached to this field, in declaration
    /// order (e.g. `@get`/`@set` — U-ANNOT-LAYOUT §3.2). See
    /// [`MethodDef::attributes`].
    pub attributes: Vec<Attribute>,
    /// The source span of the whole field declaration.
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
    /// Marks a source constructor before compiler lowering splits it into a
    /// class-side factory and an instance-side initializer.
    pub is_constructor: bool,
    /// `@name(args…)` attributes attached to this method, in declaration
    /// order (e.g. `@requires`/`@ensures` — U-ANNOT-CONTRACTS). Consumed and
    /// cleared by [`AttributeExpander::expand`](../../../phalcom-core/src/compiler/attributes.rs)
    /// during class expansion.
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
    /// The source span of just the method's name/selector token, distinct
    /// from [`Self::range`]'s whole-declaration span. See
    /// [`ClassDef::name_range`]'s doc for why this exists as its own field
    /// rather than being re-derived from `range`.
    pub name_range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct GetterDef {
    pub name: String,
    pub body: Vec<Statement>,
    pub is_static: bool,
    /// `@name(args…)` attributes attached to this getter, in declaration
    /// order. See [`MethodDef::attributes`].
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
    /// The source span of just the getter's name token, distinct from
    /// [`Self::range`]'s whole-declaration span. See
    /// [`ClassDef::name_range`]'s doc for why this exists as its own field
    /// rather than being re-derived from `range`.
    pub name_range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct SetterDef {
    pub name: String,
    pub param: String,
    pub body: Vec<Statement>,
    pub is_static: bool,
    /// `@name(args…)` attributes attached to this setter, in declaration
    /// order. See [`MethodDef::attributes`].
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
    /// The source span of just the setter's name token (excluding the
    /// trailing `=`), distinct from [`Self::range`]'s whole-declaration span.
    /// See [`ClassDef::name_range`]'s doc for why this exists as its own
    /// field rather than being re-derived from `range`.
    pub name_range: SourceRange,
}

/// Whether a binding is mutable (`let`) or immutable (`const`).
///
/// Per [ADR-0064](../../../docs/adr/accepted/0064-let-const-bindings-and-field-mutability.md):
/// a `let` binding is mutable and may be declared without an initializer — an
/// uninitialized `let` reads the surface `None` value (ADR-0007) — whereas a
/// `const` binding cannot be reassigned (reassignment is a compile error,
/// `AssignToImmutable`) and requires an initializer. Same-scope redeclaration
/// is rejected for both kinds (`binding.redeclared`, ruling L-3/L-5); nested
/// shadowing stays legal. Enforcement of these rules lives in the compiler;
/// the AST only records which form was written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    /// A mutable `let` binding.
    Let,
    /// An immutable `const` binding.
    Const,
}

/// A `let`/`const` binding, optionally with an initializer.
///
/// The [`kind`](LetBinding::kind) distinguishes the mutable `let` form from
/// the immutable `const` form (ADR-0064). A missing [`value`](LetBinding::value)
/// means no initializer was written; the compiler surfaces `None` for `let`
/// and rejects it for `const`. The [`pattern`](LetBinding::pattern) is either a
/// bare name (the pre-U14 case) or a destructuring [`Pattern`] — a tuple or
/// list pattern that positionally unpacks the initializer (open-questions.md
/// Q7, [ADR-0046](../../../docs/adr/accepted/0046-destructuring-bindings.md)). A
/// destructuring `pattern` always requires an initializer, regardless of
/// `kind` — there is nothing to unpack from an absent value.
#[derive(Debug, Clone)]
pub struct LetBinding {
    /// Whether this is a mutable `let` or an immutable `const` binding.
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
/// [ADR-0046](../../../docs/adr/accepted/0046-destructuring-bindings.md)).
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
    /// A postfix subscript read `object[index]` (U-INDEX). See [`IndexExpr`].
    Index(Box<IndexExpr>),
    /// A postfix subscript write `object[index] = value` (U-INDEX). See [`SetIndexExpr`].
    SetIndex(Box<SetIndexExpr>),
    Block(Box<BlockExpr>),
    /// A `::` method reference — `receiver::name` (selectors.md §3, U16-Open,
    /// Open form only). See [`MethodRefExpr`].
    MethodRef(Box<MethodRefExpr>),
    /// A `#`-prefixed symbol literal (selectors.md §2, U-LEX-HASH). See
    /// [`SymbolExpr`].
    Symbol(Box<SymbolExpr>),
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
            Expr::Index(e) => e.range,
            Expr::SetIndex(e) => e.range,
            Expr::Block(e) => e.range,
            Expr::MethodRef(e) => e.range,
            Expr::Symbol(e) => e.range,
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

/// A postfix subscript read — `object[args...]` (U-INDEX,
/// [ADR-0060](../../../docs/adr/accepted/0060-index-operator-as-real-selector.md)).
///
/// `args` is a full call-shaped argument list (positional + `label:`,
/// identical grammar to a call's `(...)` — produced by the same
/// `Parser::parse_arg_list` call-arg parses use), not a single expression:
/// `xs[i, j]` sends the distinct selector `[_,_]`,
/// `cache[key, default: fallback]` sends `[_,default]`, and empty `xs[]`
/// sends the zero-arity `[]`. Compiles to a direct send against the bracket
/// selector the args' arity/labels encode (`phalcom-core`'s
/// `SignatureKind::Subscript`) — **no** `at`/`at(_,put:)` lowering (ADR-0060
/// supersedes ADR-0055's sugar-over-`at` draft).
///
/// Kept as a distinct node (rather than an immediate `MethodCall` desugar) so
/// `parse_assignment` can distinguish the read form from the write form —
/// the same reason `GetProperty`/`SetProperty` are distinct from
/// `MethodCall`.
#[derive(Debug, Clone)]
pub struct IndexExpr {
    /// The collection being indexed.
    pub object: Expr,
    /// The bracketed argument list — positional and/or `label:` arguments,
    /// in source order.
    pub args: Vec<Argument>,
    /// Source span from `object` start through `]`.
    pub range: SourceRange,
}

/// A postfix subscript write — `object[args...] = value` (U-INDEX,
/// [ADR-0060](../../../docs/adr/accepted/0060-index-operator-as-real-selector.md)).
///
/// Sends the bracket-write selector `args`' arity/labels encode with `put:
/// value` appended (e.g. `xs[i] = v` sends `[_,put]`, `xs[] = v` sends
/// `[put]`) — **no** `at(_,put:)` lowering (ADR-0060 supersedes ADR-0055).
/// Produced by `parse_assignment` when it sees an `Expr::Index` on the left
/// of `=`, parallel to `SetProperty`'s production from `GetProperty`.
#[derive(Debug, Clone)]
pub struct SetIndexExpr {
    /// The collection being mutated.
    pub object: Expr,
    /// The bracketed argument list — positional and/or `label:` arguments,
    /// in source order (the index/key side only; `value` is appended as the
    /// selector's trailing `put:` argument by the compiler, not stored here).
    pub args: Vec<Argument>,
    /// The new value.
    pub value: Expr,
    /// Source span from `object` start through the RHS.
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

/// A method reference expression, `receiver::name` / `receiver::#sel(...)`
/// (selectors.md §3, U16-Open + U16-Pinned — the **bound** forms only; the
/// unbound `Type::name` / `Type::#sel(...)` "receiver is the first argument"
/// forms are out of this unit's scope).
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
    /// Which of the two reference shapes this is — Open (`::name`) or
    /// Pinned (`::#sel(...)`). See [`MethodRefKind`].
    pub kind: MethodRefKind,
    /// The source span from the start of `receiver` through the name/selector.
    pub range: SourceRange,
}

/// The two `::` method-reference shapes (selectors.md §3), carried by
/// [`MethodRefExpr`].
#[derive(Debug, Clone)]
pub enum MethodRefKind {
    /// `obj::name` — a bare base name. The call-time selector is built from
    /// this name plus the call site's argument labels (selectors.md §3
    /// "Open families resolve at call time"), U16-Open.
    Open {
        /// The base method name after `::` (not a full selector).
        name: String,
    },
    /// `obj::#name(_,to,duration)` — a full selector form. Pins the exact
    /// selector at the reference site: the call-time labels are ignored and
    /// only the argument *count* is validated (selectors.md §3 "Pinned
    /// families have their selector fully known at compile time"), U16-Pinned.
    Pinned {
        /// The selector's base name (`"move"`, `"square"`, ...).
        name: String,
        /// Per-argument labels in declared order; `None` is the positional
        /// placeholder `_`. Lowered by the compiler through the same
        /// `encode_selector` routine a matching method definition uses
        /// (`phalcom-core::method::encode_selector`), so the pinned selector
        /// interns to the *same* `Symbol` as its target method (ADR-0012).
        labels: Vec<Option<String>>,
    },
}

/// A `#`-prefixed symbol literal (selectors.md §2, U-LEX-HASH): a name symbol
/// (`#move`) or a selector symbol (`#move(_,to,duration)`, `#+`, `#==`).
///
/// Both shapes lower to a `Value::Symbol` constant (`phalcom-core::compiler`)
/// — only the interned string differs.
#[derive(Debug, Clone)]
pub struct SymbolExpr {
    /// Which of the two symbol shapes this literal is.
    pub kind: SymbolLiteralKind,
    /// The source span of the whole `#...` literal.
    pub range: SourceRange,
}

/// The two symbol-literal shapes (selectors.md §2), carried by [`SymbolExpr`].
#[derive(Debug, Clone)]
pub enum SymbolLiteralKind {
    /// `#move` — a bare name symbol. Identifies a method-name *family*, not
    /// a complete method identity; used for map keys, `respondsTo`, and
    /// other reflection queries that key on a base name alone.
    Name(String),
    /// `#move(_,to,duration)` / `#+` / `#==` — a complete selector symbol.
    /// Lowered by the compiler through the same `encode_selector` routine a
    /// matching method definition uses, so the two intern to the same
    /// `Symbol` identity (ADR-0012).
    Selector {
        /// The selector's base name (`"move"`, `"size"`, `"+"`, `"=="`, ...).
        name: String,
        /// Per-argument labels in declared order; `None` is the positional
        /// placeholder `_`.
        labels: Vec<Option<String>>,
    },
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
