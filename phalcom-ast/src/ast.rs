//! Abstract syntax tree produced by [`crate::parser`].
//!
//! These node types are the parser's output and the compiler's input. Every
//! node carries a [`SourceRange`] so later phases can attach precise
//! diagnostics. Surface absence is modelled as the `Option` type rather than a
//! `nil` literal (ADR-0007): there is deliberately no `Expr::Nil` variant, and
//! `??`/`?.` are desugared to ordinary `Option` message sends by the parser
//! (see [`crate::parser`]).

use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorError, SelectorKind, SelectorKindPattern, SelectorPattern, SelectorSlot};

#[derive(Debug, Default)]
pub struct Module {
    pub program: Program,
    pub range: SourceRange,
}

#[derive(Debug, Default, Clone)]
pub struct Program {
    pub preamble: ModulePreamble,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Class(ClassDef),
    Enum(EnumDef),
    TypeAlias(TypeAliasDef),
    Let(LetBinding),
    Return(ReturnStatement),
    Expr {
        expr: Expr,
        range: SourceRange,
    },
    /// `for pattern [at index] in iter { body }` — the cursor loop (ADR-0035 §2,
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
    Throw {
        /// The raised expression, evaluated then sent `raise()`.
        expr: Expr,
        /// The source span of the whole `throw` statement.
        range: SourceRange,
    },
    /// Local export declaration in the module body: `export Name, Other as Alias`.
    Export(ExportDecl),
}

// ── Logical Import / Export / Preamble AST ────────────────────────────────

/// Logical import path (e.g. `geometry.point` or `.point` or `..units`).
#[derive(Clone, Debug, PartialEq)]
pub struct ImportPath {
    pub root: ImportRoot,
    pub segments: Vec<PathSegment>,
    pub range: SourceRange,
}

impl std::fmt::Display for ImportPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.root {
            ImportRoot::Absolute(seg) => {
                f.write_str(&seg.name)?;
            }
            ImportRoot::Relative { dots, .. } => {
                for _ in 0..*dots {
                    f.write_str(".")?;
                }
            }
        }
        for (i, seg) in self.segments.iter().enumerate() {
            if i > 0 || matches!(self.root, ImportRoot::Absolute(_)) {
                f.write_str(".")?;
            }
            f.write_str(&seg.name)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportRoot {
    Absolute(PathSegment),
    Relative { dots: u16, range: SourceRange },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathSegment {
    pub name: String,
    pub range: SourceRange,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModulePreamble {
    pub metadata: Vec<ModuleMetadataAttribute>,
    pub dependencies: Vec<DependencyDecl>,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DependencyDecl {
    Import(ImportDecl),
    ReExport(ReExportDecl),
    Expose(ExposeDecl),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportDecl {
    Module(ModuleImportDecl),
    Selective(SelectiveImportDecl),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModuleImportDecl {
    pub path: ImportPath,
    pub alias: Option<ImportAlias>,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectiveImportDecl {
    pub path: ImportPath,
    pub items: Vec<ImportItem>,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportAlias {
    pub name: String,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportItem {
    pub name: String,
    pub name_range: SourceRange,
    pub alias: Option<ImportAlias>,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReExportDecl {
    pub path: ImportPath,
    pub items: Vec<ExportItem>,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExportDecl {
    pub items: Vec<ExportItem>,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExportAlias {
    pub name: String,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExportItem {
    pub local_or_remote_name: String,
    pub name_range: SourceRange,
    pub alias: Option<ExportAlias>,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExposeDecl {
    pub child: PathSegment,
    pub range: SourceRange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModuleMetadataAttribute {
    pub name: String,
    pub arguments: Vec<MetadataLiteral>,
    pub range: SourceRange,
}

/// Inert literal value in a module/package header attribute.
#[derive(Clone, Debug, PartialEq)]
pub enum MetadataLiteral {
    Unit,
    Bool(bool),
    Int(String),
    Float(f64),
    String(String),
    Symbol(String),
    Tuple(Vec<MetadataLiteral>),
    Record(Vec<(String, MetadataLiteral)>),
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
    /// Generic parameter binders for this class (Spec 04).
    pub generic_parameters: Vec<GenericParameterSyntax>,
    /// The explicit superclass template named by an `is` clause, if any (Spec 04).
    ///
    /// `None` means no `is` clause was written, so the class implicitly
    /// inherits from `Object` (object-model.md §5.1). `Some(_)` carries the
    /// superclass type template and its span for compile-time resolution and
    /// diagnostics.
    pub superclass: Option<TypeAnnotation>,
    /// Generic `where` constraints attached to this class (Spec 04).
    pub where_clause: Option<WhereClauseSyntax>,
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
    /// `class Foo is Bar { … }`), distinct from [`Self::range`]'s
    /// whole-declaration span. Lets a downstream consumer (e.g.
    /// `phalcom-lsp`'s `semanticTokens/full` pass) highlight the declared
    /// name itself with a `class`-token type without heuristically
    /// re-scanning `range` for the first matching identifier — a prior
    /// attempt at that heuristic was rejected as unsound (nothing guarantees
    /// the first occurrence of `name` inside `range` is the declaration
    /// token rather than an incidental earlier one).
    pub name_range: SourceRange,
}

impl ClassDef {
    /// Returns the static symbol reference of the superclass origin, if any.
    pub fn superclass_ref(&self) -> Option<&StaticSymbolRef> {
        self.superclass.as_ref().and_then(|sc| sc.origin_symbol_ref())
    }
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub name_range: SourceRange,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    pub members: Vec<EnumMember>,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub enum EnumMember {
    Variant(VariantDecl),
    Behavior(EnumBehaviorMember),
}

#[derive(Debug, Clone)]
pub enum EnumBehaviorMember {
    Method(MethodDef),
    Getter(GetterDef),
    Setter(SetterDef),
    Index(IndexMethodDef),
}

impl EnumBehaviorMember {
    pub fn attributes(&self) -> &[Attribute] {
        match self {
            EnumBehaviorMember::Method(m) => &m.attributes,
            EnumBehaviorMember::Getter(g) => &g.attributes,
            EnumBehaviorMember::Setter(s) => &s.attributes,
            EnumBehaviorMember::Index(i) => &i.attributes,
        }
    }

    pub fn attributes_mut(&mut self) -> &mut Vec<Attribute> {
        match self {
            EnumBehaviorMember::Method(m) => &mut m.attributes,
            EnumBehaviorMember::Getter(g) => &mut g.attributes,
            EnumBehaviorMember::Setter(s) => &mut s.attributes,
            EnumBehaviorMember::Index(i) => &mut i.attributes,
        }
    }

    pub fn range(&self) -> SourceRange {
        match self {
            EnumBehaviorMember::Method(m) => m.range,
            EnumBehaviorMember::Getter(g) => g.range,
            EnumBehaviorMember::Setter(s) => s.range,
            EnumBehaviorMember::Index(i) => i.range,
        }
    }

    pub fn name_range(&self) -> SourceRange {
        match self {
            EnumBehaviorMember::Method(m) => m.name_range,
            EnumBehaviorMember::Getter(g) => g.name_range,
            EnumBehaviorMember::Setter(s) => s.name_range,
            EnumBehaviorMember::Index(i) => i.name_range,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VariantDecl {
    pub name: String,
    pub name_range: SourceRange,
    /// Span of the explicit `@variant` marker.
    pub variant_marker_range: SourceRange,
    /// `None` means getter-shaped singleton variant `#name`.
    pub payload: Option<VariantPayloadSyntax>,
    /// GADT result specialization, if written.
    pub result_annotation: Option<TypeAnnotation>,
    /// Case-specific behavior.
    pub body: Option<VariantBody>,
    /// Non-marker attributes preserved for later visibility/metadata semantics.
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct VariantPayloadSyntax {
    pub parameters: Vec<ParameterDef>,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct VariantBody {
    pub members: Vec<EnumBehaviorMember>,
    pub range: SourceRange,
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
    Private,
    Protected,
    Total,
    Internal,
}

impl BuiltinAttr {
    pub const ALL: &'static [BuiltinAttr] = &[
        BuiltinAttr::Construct,
        BuiltinAttr::Constructor,
        BuiltinAttr::Class,
        BuiltinAttr::Get,
        BuiltinAttr::Set,
        BuiltinAttr::Data,
        BuiltinAttr::Sealed,
        BuiltinAttr::Variant,
        BuiltinAttr::Invariant,
        BuiltinAttr::Requires,
        BuiltinAttr::Ensures,
        BuiltinAttr::On,
        BuiltinAttr::Native,
        BuiltinAttr::Ignore,
        BuiltinAttr::Private,
        BuiltinAttr::Protected,
        BuiltinAttr::Total,
        BuiltinAttr::Internal,
    ];

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
            BuiltinAttr::Private => "private",
            BuiltinAttr::Protected => "protected",
            BuiltinAttr::Total => "total",
            BuiltinAttr::Internal => "internal",
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
            "private" => Some(BuiltinAttr::Private),
            "protected" => Some(BuiltinAttr::Protected),
            "total" => Some(BuiltinAttr::Total),
            "internal" => Some(BuiltinAttr::Internal),
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

/// A statically-resolved symbol reference used by declarations.
///
/// The parser accepts a bare root (`Shape`) or a qualified path
/// (`base.Shape`). Linkers decide whether the root is a valid module alias and
/// whether the final exported declaration exists; this node never denotes an
/// arbitrary runtime member send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticSymbolRef {
    /// Root identifier as written in source.
    pub root: String,
    /// Source span of the root identifier.
    pub root_range: SourceRange,
    /// Qualified member segments after the root.
    pub members: Vec<PathSegment>,
    /// Source span of the complete static reference.
    pub range: SourceRange,
}

/// Compatibility name for callers that still speak specifically about
/// superclass references.
pub type SuperclassRef = StaticSymbolRef;

impl StaticSymbolRef {
    /// Returns the final source name in this reference.
    pub fn leaf_name(&self) -> &str {
        self.members.last().map(|segment| segment.name.as_str()).unwrap_or(&self.root)
    }

    /// Returns whether this is an unqualified bare reference.
    pub fn is_bare(&self) -> bool {
        self.members.is_empty()
    }
}

/// Explicit source-level type annotation / type syntax.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAnnotation {
    pub expr: TypeAnnotationExpr,
    pub range: SourceRange,
}

impl TypeAnnotation {
    /// Extracts the static symbol reference of the origin if this is a reference or applied reference.
    pub fn origin_symbol_ref(&self) -> Option<&StaticSymbolRef> {
        match &self.expr {
            TypeAnnotationExpr::Reference(sym) => Some(sym),
            TypeAnnotationExpr::Application { origin, .. } => origin.origin_symbol_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotationExpr {
    /// Nominal / static symbol reference (e.g. `Int` or `geometry.Point`).
    Reference(StaticSymbolRef),
    /// Generic type application (e.g. `List<Int>`).
    Application {
        origin: Box<TypeAnnotation>,
        arguments: Vec<TypeAnnotation>,
        range: SourceRange,
    },
    /// Union type (e.g. `Int | String`).
    Union { members: Vec<TypeAnnotation>, range: SourceRange },
    /// Tuple type (e.g. `(Int, String)`).
    Tuple { elements: Vec<TypeTupleElement>, range: SourceRange },
    /// Callable / block signature (e.g. `(Int) -> String`).
    Callable {
        parameters: Vec<TypeCallableParameter>,
        result: Box<TypeAnnotation>,
        range: SourceRange,
    },
    /// Unit type `()`.
    Unit { range: SourceRange },
    /// Dynamic boundary type `Dynamic`.
    Dynamic { range: SourceRange },
    /// Bottom type `Never`.
    Never { range: SourceRange },
    /// Owner-relative type `Self`.
    SelfType { range: SourceRange },
    /// Structural record type (e.g. `#{ name: String, age: Int }` or `#{ name: String, | R }`).
    Record {
        fields: Vec<RecordTypeField>,
        tail: Option<RecordRowTail>,
        range: SourceRange,
    },
    /// Type lambda expression (e.g. `<T> =>> Result<T, Error>`).
    TypeLambda {
        parameters: Vec<TypeLambdaParameter>,
        body: Box<TypeAnnotation>,
        range: SourceRange,
    },
    /// Recovered invalid type expression.
    Invalid { message: String, range: SourceRange },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeTupleElement {
    pub label: Option<String>,
    pub ty: TypeAnnotation,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeCallableParameter {
    pub label: Option<String>,
    pub ty: TypeAnnotation,
    pub rest: bool,
    pub range: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VarianceSyntax {
    Invariant,
    Covariant,
    Contravariant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KindSyntax {
    Type(SourceRange),
    RecordRow(SourceRange),
    Arrow {
        parameter: Box<KindSyntax>,
        result: Box<KindSyntax>,
        range: SourceRange,
    },
    Grouped {
        inner: Box<KindSyntax>,
        range: SourceRange,
    },
    Invalid {
        message: String,
        range: SourceRange,
    },
}

impl KindSyntax {
    pub fn range(&self) -> SourceRange {
        match self {
            KindSyntax::Type(r) => *r,
            KindSyntax::RecordRow(r) => *r,
            KindSyntax::Arrow { range, .. } => *range,
            KindSyntax::Grouped { range, .. } => *range,
            KindSyntax::Invalid { range, .. } => *range,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericParameterSyntax {
    pub variance: VarianceSyntax,
    pub name: String,
    pub name_range: SourceRange,
    pub kind: Option<KindSyntax>,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GenericConstraintSyntax {
    Subtype {
        lower: TypeAnnotation,
        upper: TypeAnnotation,
        range: SourceRange,
    },
    Equivalent {
        left: TypeAnnotation,
        right: TypeAnnotation,
        range: SourceRange,
    },
    Invalid {
        message: String,
        range: SourceRange,
    },
}

impl GenericConstraintSyntax {
    pub fn range(&self) -> SourceRange {
        match self {
            GenericConstraintSyntax::Subtype { range, .. } => *range,
            GenericConstraintSyntax::Equivalent { range, .. } => *range,
            GenericConstraintSyntax::Invalid { range, .. } => *range,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhereClauseSyntax {
    pub constraints: Vec<GenericConstraintSyntax>,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordTypeField {
    pub name: String,
    pub ty: TypeAnnotation,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordRowTail {
    pub name: String,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeLambdaParameter {
    pub name: String,
    pub name_range: SourceRange,
    pub kind: Option<KindSyntax>,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAliasDef {
    pub name: String,
    pub name_range: SourceRange,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    pub body: TypeAnnotation,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub enum ClassMember {
    Method(MethodDef),
    Getter(GetterDef),
    Setter(SetterDef),
    /// A declared source/implementation field at class-body position. See
    /// [`FieldDef`].
    Field(FieldDef),
    /// A `@variant Name(labels...)` arm inside a `@sealed` class body
    /// (U-ANNOT-LAYOUT §3.4, `annotations-data.md` §"`@variant`"). See
    /// [`VariantDef`]. Never compiled directly — stripped and expanded into a
    /// sibling top-level [`Statement::Class`] by `phalcom-core`'s
    /// `compiler::attributes::expand_class_attributes`.
    Variant(VariantDef),
    /// A bracket-delimited subscript method — `[_ idx] { ... }` (read) or
    /// `[_ idx]=(put value) { ... }` (write), U-INDEX,
    /// [ADR-0060](../../../docs/adr/accepted/0060-index-operator-as-real-selector.md).
    /// See [`IndexMethodDef`].
    Index(IndexMethodDef),
}

impl ClassMember {
    pub fn is_static(&self) -> bool {
        match self {
            ClassMember::Method(m) => m.is_static,
            ClassMember::Getter(g) => g.is_static,
            ClassMember::Setter(s) => s.is_static,
            ClassMember::Index(_) => false,
            ClassMember::Variant(_) => false,
            ClassMember::Field(f) => f.is_static,
        }
    }

    pub fn is_constructor(&self) -> bool {
        match self {
            ClassMember::Method(m) => m.is_constructor,
            _ => false,
        }
    }

    pub fn attributes(&self) -> &[Attribute] {
        match self {
            ClassMember::Method(m) => &m.attributes,
            ClassMember::Getter(g) => &g.attributes,
            ClassMember::Setter(s) => &s.attributes,
            ClassMember::Index(i) => &i.attributes,
            ClassMember::Variant(v) => &v.attributes,
            ClassMember::Field(f) => &f.attributes,
        }
    }

    pub fn attributes_mut(&mut self) -> &mut Vec<Attribute> {
        match self {
            ClassMember::Method(m) => &mut m.attributes,
            ClassMember::Getter(g) => &mut g.attributes,
            ClassMember::Setter(s) => &mut s.attributes,
            ClassMember::Index(i) => &mut i.attributes,
            ClassMember::Variant(v) => &mut v.attributes,
            ClassMember::Field(f) => &mut f.attributes,
        }
    }

    pub fn range(&self) -> SourceRange {
        match self {
            ClassMember::Method(m) => m.range,
            ClassMember::Getter(g) => g.range,
            ClassMember::Setter(s) => s.range,
            ClassMember::Index(i) => i.range,
            ClassMember::Variant(v) => v.range,
            ClassMember::Field(f) => f.range,
        }
    }

    pub fn name_range(&self) -> SourceRange {
        match self {
            ClassMember::Method(m) => m.name_range,
            ClassMember::Getter(g) => g.name_range,
            ClassMember::Setter(s) => s.name_range,
            ClassMember::Index(i) => i.name_range,
            ClassMember::Variant(v) => v.name_range,
            ClassMember::Field(f) => f.name_range,
        }
    }
}

/// A bracket-delimited subscript method definition — `[_ idx] { ... }` /
/// `[_ idx]=(put value) { ... }` / `[] { ... }` / `[]=(put value) { ... }` (U-INDEX,
/// [ADR-0060](../../../docs/adr/accepted/0060-index-operator-as-real-selector.md)).
///
/// Unlike every other [`ClassMember`], this selector carries no separate name
/// token at all — the brackets themselves are the whole of the member's
/// grammar, and `params` (parsed by the ordinary `Parser::parse_param_list`,
/// substituting `[`/`]` for `(`/`)`) live *inside* them rather than in a
/// following `(...)` slot.
/// Getter identity is bracket arity + labels; setter identity appends the
/// fixed assignment role. Thus `[_ idx, default fallback]=(put value)` is
/// `[_,default]=(put)`.
#[derive(Debug, Clone)]
pub enum IndexAccessor {
    Get,
    Set { put: Box<ParameterDef> },
}

#[derive(Debug, Clone)]
pub struct IndexMethodDef {
    /// Indexing arguments only. The assignment value is not included here.
    pub params: Vec<ParameterDef>,
    pub accessor: IndexAccessor,
    pub return_annotation: Option<TypeAnnotation>,
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
    /// The source span of just the variant name token.
    pub name_range: SourceRange,
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
    /// The source span of just the field name token.
    pub name_range: SourceRange,
    /// Whether the field was declared *without* `const` (`true`, mutable) or
    /// with `const` (`false`, immutable) — ADR-0064's field mutability
    /// distinction (L-2). Mutable fields take no keyword at all (`_x`, not
    /// `let _x`); only `const _x` spells the immutable form.
    pub mutable: bool,
    /// Whether field storage belongs to declaring class object (`@class`).
    pub is_static: bool,
    /// Explicit type annotation, if provided.
    pub annotation: Option<TypeAnnotation>,
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
/// An ordinary positional parameter is a bare identifier; a declaration label
/// uses `external local` (or `external` when both names match). F.1 parses
/// all [`RestMode`] values. F.3 normalizes rest modes into lane-aware method
/// signatures; constructor/factory and subscript rest remain outside that
/// method-body scope.
#[derive(Debug, Clone)]
pub struct ParameterDef {
    /// The parameter's local binding name.
    pub name: String,
    /// The source span of the local binding name, excluding labels and rest markers.
    pub name_range: SourceRange,
    /// The keyword label this parameter is called under, if any. `None` for
    /// an ordinary positional parameter.
    pub label: Option<String>,
    /// The source span of the external label when it is written separately.
    pub label_range: Option<SourceRange>,
    /// Parsed rest lane. The compiler normalizes this into runtime rest
    /// metadata without adding a second parser-side representation.
    pub rest_mode: RestMode,
    /// Explicit type annotation, if provided.
    pub annotation: Option<TypeAnnotation>,
    /// The parameter's source span.
    pub range: SourceRange,
}

impl ParameterDef {
    pub fn is_rest(&self) -> bool {
        self.rest_mode != RestMode::None
    }
}

/// The expansion lane of a pack contribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpansionMode {
    Positional,
    Labeled,
    Complete,
}

/// A parameter's rest-binding lane. Binding semantics remain deferred to F.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RestMode {
    None,
    Positional,
    Labeled,
    Complete,
}

/// Callable body representation preserving the difference between declaration-only
/// and executable body forms.
#[derive(Clone, Debug)]
pub enum MemberBody {
    Declaration,
    Block(Vec<Statement>),
}

impl MemberBody {
    pub fn is_declaration(&self) -> bool {
        matches!(self, Self::Declaration)
    }

    pub fn statements(&self) -> Option<&[Statement]> {
        match self {
            Self::Block(statements) => Some(statements),
            Self::Declaration => None,
        }
    }

    pub fn statements_mut(&mut self) -> Option<&mut Vec<Statement>> {
        match self {
            Self::Block(statements) => Some(statements),
            Self::Declaration => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MethodDef {
    pub name: String,
    /// Generic parameter binders for this method (Spec 04).
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub params: Vec<ParameterDef>,
    pub return_annotation: Option<TypeAnnotation>,
    /// Generic `where` constraints attached to this method (Spec 04).
    pub where_clause: Option<WhereClauseSyntax>,
    pub body: MemberBody,
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
    pub return_annotation: Option<TypeAnnotation>,
    pub body: MemberBody,
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
    pub param: ParameterDef,
    pub return_annotation: Option<TypeAnnotation>,
    pub body: MemberBody,
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
    /// Explicit type annotation attached to the binding pattern.
    pub annotation: Option<TypeAnnotation>,
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
    /// rest sub-pattern. A rest sub-pattern, if
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
    /// A sealed/unit or payload-bearing variant pattern such as `None()` or
    /// `Some(value)`.
    Variant {
        /// Constructor/variant class name.
        constructor: String,
        /// Positional payload patterns in declaration order.
        arguments: Vec<Pattern>,
        /// Source span of the complete constructor pattern.
        range: SourceRange,
    },
    /// An open record pattern such as `#{name: value}`.
    Record {
        /// Required field patterns.
        entries: Vec<RecordPatternEntry>,
        /// Source span of the complete record pattern.
        range: SourceRange,
    },
    /// An open map pattern such as `{#name: value}`.
    Map {
        /// Required key/value patterns.
        entries: Vec<MapPatternEntry>,
        /// Source span of the complete map pattern.
        range: SourceRange,
    },
}

impl Pattern {
    /// Returns this pattern's source span.
    pub fn range(&self) -> SourceRange {
        match self {
            Pattern::Name { range, .. }
            | Pattern::Tuple { range, .. }
            | Pattern::List { range, .. }
            | Pattern::Variant { range, .. }
            | Pattern::Record { range, .. }
            | Pattern::Map { range, .. } => *range,
        }
    }
}

/// A required record field in a pattern.
#[derive(Debug, Clone)]
pub struct RecordPatternEntry {
    pub label: String,
    pub pattern: Pattern,
    pub range: SourceRange,
}

/// A stable literal key and required value pattern in a map pattern.
#[derive(Debug, Clone)]
pub struct MapPatternEntry {
    pub key: MapPatternKey,
    pub pattern: Pattern,
    pub range: SourceRange,
}

/// Keys accepted by the first map-pattern implementation. Arbitrary key
/// expressions remain deliberately outside pattern syntax.
#[derive(Debug, Clone)]
pub enum MapPatternKey {
    Symbol(String),
    String(String),
    Int { digits: String, radix: u32 },
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
    /// Non-empty lockstep lanes, each evaluated once from left to right.
    pub lanes: Vec<ForLane>,
    /// The loop body statements, run once per element under jump-based control
    /// flow (no `block_call` on the taken path, ADR-0035 §2).
    pub body: Vec<Statement>,
    /// The source span covering the whole `for` statement.
    pub range: SourceRange,
}

/// One `for` lane: `pattern [at ordinal] in iterable`.
#[derive(Debug, Clone)]
pub struct ForLane {
    pub pattern: Pattern,
    pub index: Option<ForIndexBinding>,
    pub iter: Expr,
    pub range: SourceRange,
}

/// A source binding for the zero-based ordinal of one `for` lane.
#[derive(Debug, Clone)]
pub struct ForIndexBinding {
    pub name: String,
    pub range: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldKind {
    Source,
    Implementation,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int {
        digits: String,
        radix: u32,
        range: SourceRange,
    },
    Float {
        value: f64,
        range: SourceRange,
    },
    String {
        value: String,
        range: SourceRange,
    },
    Boolean {
        value: bool,
        range: SourceRange,
    },
    Var {
        value: String,
        range: SourceRange,
    },
    Field {
        value: String,
        kind: FieldKind,
        range: SourceRange,
    },
    SelfVar {
        range: SourceRange,
    },
    SuperVar {
        range: SourceRange,
    },
    Assignment(Box<AssignmentExpr>),
    /// A native range bound descriptor. It stays explicit until direct bytecode
    /// construction and never desugars through overridable class creation.
    Range(Box<RangeExpr>),
    Unary(Box<UnaryExpr>),
    Binary(Box<BinaryExpr>),
    /// A relational chain whose middle operands are evaluated once.
    ComparisonChain(Box<ComparisonChainExpr>),
    /// A refutable pattern conditional.
    IfLet(Box<IfLetExpr>),
    /// A refutable pattern loop.
    WhileLet(Box<WhileLetExpr>),
    /// The ordinary expression value represented by `...`.
    Ellipsis {
        range: SourceRange,
    },
    /// A call written without an explicit receiver, e.g. `foo(value)`.
    ///
    /// This remains distinct from a call to a value's `call` protocol until
    /// compilation, where lexical bindings take precedence over an implicit
    /// receiver send.
    UnqualifiedCall(Box<UnqualifiedCallExpr>),
    MethodCall(Box<MethodCallExpr>),
    /// An implementation selector written without an explicit receiver,
    /// e.g. `_$rawAt`.
    ImplementationSelector {
        value: String,
        range: SourceRange,
    },
    GetProperty(Box<GetPropertyExpr>),
    SetProperty(Box<SetPropertyExpr>),
    /// A postfix subscript read `object[index]` (U-INDEX). See [`IndexExpr`].
    Index(Box<IndexExpr>),
    /// A postfix subscript write `object[index] = value` (U-INDEX). See [`SetIndexExpr`].
    SetIndex(Box<SetIndexExpr>),
    Block(Box<BlockExpr>),
    AssociatedLookup(Box<AssociatedLookupExpr>),
    AssociatedInvoke(Box<AssociatedInvokeExpr>),
    /// A `#`-prefixed symbol literal (selectors.md §2, U-LEX-HASH). See
    /// [`SymbolExpr`].
    Symbol(Box<SymbolExpr>),
    /// A tuple literal written with `(` `)` product syntax.
    TupleLiteral(Box<TupleLiteralExpr>),
    /// A record literal written with `#{` `}` product syntax.
    RecordLiteral(Box<RecordLiteralExpr>),
    /// An association Map literal written with `{ key: value }` syntax.
    MapLiteral(Box<MapLiteralExpr>),
    /// A Set literal written with `{ value, ... }` syntax.
    SetLiteral(Box<SetLiteralExpr>),
    /// A List literal written with `[ value, ... ]` syntax.
    ListLiteral(Box<ListLiteralExpr>),
    /// A membership test `left in right` or `left not in right`.
    Membership(Box<MembershipExpr>),
    /// A type membership test `left is in candidates` / `left is! in candidates` / `left is not in candidates` / `left is! not in candidates`.
    IsMembership(Box<IsMembershipExpr>),
    /// A value-space type form expression (e.g. `List<Int>` or `<T> =>> Result<T, Error>`) (Spec 04).
    TypeForm(Box<TypeAnnotation>),
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
            Expr::Int { range, .. }
            | Expr::Float { range, .. }
            | Expr::String { range, .. }
            | Expr::Boolean { range, .. }
            | Expr::Var { range, .. }
            | Expr::Field { range, .. }
            | Expr::SelfVar { range }
            | Expr::SuperVar { range } => *range,
            Expr::Assignment(e) => e.range,
            Expr::Range(e) => e.range,
            Expr::Unary(e) => e.range,
            Expr::Binary(e) => e.range,
            Expr::ComparisonChain(e) => e.range,
            Expr::IfLet(e) => e.range,
            Expr::WhileLet(e) => e.range,
            Expr::Ellipsis { range } => *range,
            Expr::UnqualifiedCall(e) => e.range,
            Expr::MethodCall(e) => e.range,
            Expr::ImplementationSelector { range, .. } => *range,
            Expr::GetProperty(e) => e.range,
            Expr::SetProperty(e) => e.range,
            Expr::Index(e) => e.range,
            Expr::SetIndex(e) => e.range,
            Expr::Block(e) => e.range,
            Expr::AssociatedLookup(e) => e.range,
            Expr::AssociatedInvoke(e) => e.range,
            Expr::Symbol(e) => e.range,
            Expr::TupleLiteral(e) => e.range,
            Expr::RecordLiteral(e) => e.range,
            Expr::MapLiteral(e) => e.range,
            Expr::SetLiteral(e) => e.range,
            Expr::ListLiteral(e) => e.range,
            Expr::Membership(e) => e.range,
            Expr::IsMembership(e) => e.range,
            Expr::TypeForm(t) => t.range,
        }
    }
}

/// Range bounds as written in source. Omitted bounds are structurally absent,
/// never a surface `None` expression.
#[derive(Debug, Clone)]
pub struct RangeExpr {
    pub lower: Option<Expr>,
    pub upper: Option<Expr>,
    pub upper_inclusive: bool,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    /// The written operator token. `None` for parser/compiler-generated sends.
    pub op_range: Option<SourceRange>,
    pub left: Expr,
    pub right: Expr,
    pub range: SourceRange,
}

/// A Python-style chained relation, e.g. `a < b <= c`.
#[derive(Debug, Clone)]
pub struct ComparisonChainExpr {
    pub operands: Vec<Expr>,
    pub operators: Vec<RelationOp>,
    pub range: SourceRange,
}

/// Boolean-producing relation operators allowed in comparison chains.
#[derive(Debug, Clone)]
pub enum RelationOp {
    Binary(BinaryOp),
    Matches,
    Understands,
}

/// `if let pattern = value { then } else { otherwise }`.
#[derive(Debug, Clone)]
pub struct IfLetExpr {
    pub pattern: Pattern,
    pub value: Expr,
    pub then_body: BlockExpr,
    pub else_body: Option<BlockExpr>,
    pub range: SourceRange,
}

/// `while let pattern = value { body }`.
#[derive(Debug, Clone)]
pub struct WhileLetExpr {
    pub pattern: Pattern,
    pub value: Expr,
    pub body: Vec<Statement>,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct SetPropertyExpr {
    pub object: Expr,
    pub property: String,
    /// The written property token, if this setter came from source syntax.
    pub property_range: Option<SourceRange>,
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
    pub args: Vec<PackItem>,
    /// The written bracket selector span, excluding the receiver.
    pub selector_range: Option<SourceRange>,
    /// Source span from `object` start through `]`.
    pub range: SourceRange,
}

/// A postfix subscript write — `object[args...] = value` (U-INDEX,
/// [ADR-0060](../../../docs/adr/accepted/0060-index-operator-as-real-selector.md)).
///
/// Sends the bracket-write selector `args`' arity/labels encode with `put:
/// value` appended in the fixed setter role (e.g. `xs[i] = v` sends
/// `[_]=(put)`) — **no** `at(_,put)` lowering (ADR-0060 supersedes ADR-0055).
/// Produced by `parse_assignment` when it sees an `Expr::Index` on the left
/// of `=`, parallel to `SetProperty`'s production from `GetProperty`.
#[derive(Debug, Clone)]
pub struct SetIndexExpr {
    /// The collection being mutated.
    pub object: Expr,
    /// The bracketed argument list — positional and/or `label:` arguments,
    /// in source order (the index/key side only; `value` is appended as the
    /// selector's trailing `put:` argument by the compiler, not stored here).
    pub args: Vec<PackItem>,
    /// The written bracket selector span, excluding the receiver and RHS.
    pub selector_range: Option<SourceRange>,
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

/// An unqualified call whose receiver resolution is deferred to the compiler.
#[derive(Debug, Clone)]
pub struct UnqualifiedCallExpr {
    pub name: String,
    /// The written callee name, if this call came from source syntax.
    pub name_range: Option<SourceRange>,
    pub args: Vec<PackItem>,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub enum PackItem {
    Positional { expr: Expr, range: SourceRange },
    Labeled { label: PackLabel, value: Expr, range: SourceRange },
    Expand { mode: ExpansionMode, expr: Expr, range: SourceRange },
}

/// A label contributed by a call/subscript argument pack.
#[derive(Debug, Clone)]
pub enum PackLabel {
    Static { text: String, range: SourceRange },
    Computed { expr: Box<Expr>, range: SourceRange },
}

#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub object: Expr,
    pub method: String,
    /// The written selector/property token, if this call came from source syntax.
    pub method_range: Option<SourceRange>,
    pub args: Vec<PackItem>,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct GetPropertyExpr {
    pub object: Expr,
    pub property: String,
    /// The written property token, if this access came from source syntax.
    pub property_range: Option<SourceRange>,
    pub range: SourceRange,
}

/// A method reference expression, `receiver::name` / `receiver::#sel(...)`
/// (selectors.md §3, U16-Open + U16-Pinned — the **bound** forms only; the
/// unbound `Type::name` / `Type::#sel(...)` "receiver is the first argument"
#[derive(Debug, Clone)]
pub struct AssociatedLookupExpr {
    pub receiver: Expr,
    pub first_separator_range: SourceRange,
    pub member: AssociatedMemberSyntax,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub enum AssociatedMemberSyntax {
    Named(AssociatedNamedMemberSyntax),
    Operator(ExactSelectorSyntax),
    Subscript(ExactSelectorSyntax),
}

#[derive(Debug, Clone)]
pub struct AssociatedNamedMemberSyntax {
    pub base: String,
    pub base_range: SourceRange,
    pub mode: AssociatedNamedMode,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub enum AssociatedNamedMode {
    /// `owner::name` or `owner::name::`.
    Getter { explicit_separator_range: Option<SourceRange> },
    /// `owner::name::shape`.
    Exact {
        second_separator_range: SourceRange,
        residual: AssociatedResidualSelectorSyntax,
    },
    /// `owner::name::*`.
    Family {
        second_separator_range: SourceRange,
        star_range: SourceRange,
    },
}

#[derive(Debug, Clone)]
pub enum AssociatedResidualSelectorSyntax {
    Method { slots: Vec<SelectorSlotSyntax>, range: SourceRange },
    Setter { put_range: SourceRange, range: SourceRange },
}

#[derive(Debug, Clone)]
pub struct AssociatedInvokeExpr {
    pub receiver: Expr,
    pub first_separator_range: SourceRange,
    pub base: String,
    pub base_range: SourceRange,
    pub args: Vec<PackItem>,
    pub range: SourceRange,
}

/// Source-oriented selector specification. Unlike the runtime selector model,
/// this keeps component ranges so diagnostics and editor features can point at
/// the base, slots, labels, and gap independently.
#[derive(Debug, Clone)]
pub enum SelectorSpecSyntax {
    Exact(ExactSelectorSyntax),
    Pattern(SelectorPatternSyntax),
}

#[derive(Debug, Clone)]
pub struct ExactSelectorSyntax {
    pub base: String,
    pub kind: SelectorKind,
    pub slots: Vec<SelectorSlotSyntax>,
    /// `true` for bracket/index selectors such as `#[_]`.
    pub is_subscript: bool,
    pub base_range: SourceRange,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct SelectorPatternSyntax {
    pub base: String,
    pub kind: SelectorKindPattern,
    pub prefix: Vec<SelectorSlotSyntax>,
    pub suffix: Vec<SelectorSlotSyntax>,
    /// `true` for bracket/index patterns such as `#[...]`.
    pub is_subscript: bool,
    pub gap_range: SourceRange,
    pub base_range: SourceRange,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct SelectorSlotSyntax {
    pub slot: SelectorSlot,
    pub range: SourceRange,
}

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NormalizedSelectorSpec {
    Exact(Selector),
    Pattern(SelectorPattern),
}

impl NormalizedSelectorSpec {
    pub fn try_decode(text: &str) -> Result<Self, SelectorError> {
        if phalcom_common::selector::is_selector_pattern_syntax(text) {
            SelectorPattern::try_decode_pattern(text).map(Self::Pattern)
        } else {
            Selector::try_decode_exact(text).map(Self::Exact)
        }
    }
}

impl ExactSelectorSyntax {
    pub fn normalize(&self) -> Result<Selector, SelectorError> {
        Selector::new(
            if self.is_subscript {
                phalcom_common::selector::SelectorBase::Subscript
            } else {
                phalcom_common::selector::SelectorBase::Named(self.base.clone())
            },
            self.kind,
            self.slots.iter().map(|slot| slot.slot.clone()).collect::<Vec<_>>().into_boxed_slice(),
        )
    }
}

impl SelectorPatternSyntax {
    pub fn normalize(&self) -> Result<SelectorPattern, SelectorError> {
        SelectorPattern::new(
            if self.is_subscript {
                phalcom_common::selector::SelectorBase::Subscript
            } else {
                phalcom_common::selector::SelectorBase::Named(self.base.clone())
            },
            self.kind.clone(),
            self.prefix.iter().map(|slot| slot.slot.clone()).collect::<Vec<_>>().into_boxed_slice(),
            self.suffix.iter().map(|slot| slot.slot.clone()).collect::<Vec<_>>().into_boxed_slice(),
            true,
        )
    }
}

impl SelectorSpecSyntax {
    pub fn range(&self) -> SourceRange {
        match self {
            Self::Exact(spec) => spec.range,
            Self::Pattern(spec) => spec.range,
        }
    }

    pub fn base(&self) -> &str {
        match self {
            Self::Exact(spec) => &spec.base,
            Self::Pattern(spec) => &spec.base,
        }
    }

    pub fn normalize(&self) -> Result<NormalizedSelectorSpec, SelectorError> {
        match self {
            Self::Exact(spec) => spec.normalize().map(NormalizedSelectorSpec::Exact),
            Self::Pattern(spec) => spec.normalize().map(NormalizedSelectorSpec::Pattern),
        }
    }
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
    /// An exact bracket selector such as `#[_]` or `#[_]=(put)`.
    Subscript {
        /// Index slots, with `None` representing `_`.
        labels: Vec<Option<String>>,
        /// Whether this is the bracket setter spelling `]=(put)`.
        setter: bool,
    },
    /// A structural selector pattern such as `#name(...)` or
    /// `#name(_, ..., tail)`. It remains an AST value until the compiler
    /// materializes its immutable runtime pattern object.
    Pattern(SelectorPatternSyntax),
}

/// A product-label syntax family for tuple entries and record fields.
#[derive(Debug, Clone)]
pub enum ProductLabel {
    /// A statically-known label symbol.
    Static {
        /// The symbol literal shape recorded for the label spelling.
        symbol: SymbolLiteralKind,
        /// The syntax family that produced the label.
        syntax: ProductLabelSyntax,
        /// The source span covering the label head and trailing `:`.
        range: SourceRange,
    },
    /// A computed label expression written as `[expr]:`.
    Computed {
        /// The expression inside the brackets.
        expr: Box<Expr>,
        /// The source span covering the full computed label.
        range: SourceRange,
    },
}

/// Describes how a static product label was written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductLabelSyntax {
    /// A bare label head such as `name:` or `+(other):`.
    Bare,
    /// An explicit symbol label such as `#name:` or `#":":`.
    ExplicitSymbol,
}

/// A tuple literal written with product syntax.
#[derive(Debug, Clone)]
pub struct TupleLiteralExpr {
    /// The tuple entries in source order.
    pub entries: Vec<TupleLiteralEntry>,
    /// The source span covering the full tuple literal.
    pub range: SourceRange,
}

/// A tuple entry, either positional or labeled.
#[derive(Debug, Clone)]
pub enum TupleLiteralEntry {
    /// A positional tuple element.
    Positional {
        /// The entry expression.
        expr: Expr,
        /// The source span of the entry expression.
        range: SourceRange,
    },
    /// A labeled tuple element.
    Labeled {
        /// The entry label.
        label: ProductLabel,
        /// The entry value.
        value: Expr,
        /// The source span covering the labeled entry.
        range: SourceRange,
    },
    /// An expansion contribution. Runtime assembly is deferred to F.2.
    Expand { mode: ExpansionMode, expr: Expr, range: SourceRange },
}

/// A record literal written with `#{...}` product syntax.
#[derive(Debug, Clone)]
pub struct RecordLiteralExpr {
    /// The record entries in source order.
    pub entries: Vec<RecordLiteralEntry>,
    /// The source span covering the full record literal.
    pub range: SourceRange,
}

/// A Record literal entry: an explicit field or a labeled-lane expansion.
#[derive(Debug, Clone)]
pub enum RecordLiteralEntry {
    /// An explicit Record field.
    Field(RecordLiteralField),
    /// A `**source` labeled-lane expansion.
    Expansion { expr: Expr, range: SourceRange },
}

/// A record field entry.
#[derive(Debug, Clone)]
pub struct RecordLiteralField {
    /// The field label.
    pub label: ProductLabel,
    /// The field value.
    pub value: Expr,
    /// The source span covering the field.
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct MapLiteralExpr {
    pub entries: Vec<MapLiteralEntry>,
    pub range: SourceRange,
}

/// `Expansion` reserves a structural seam for Spec F without implementing it.
#[derive(Debug, Clone)]
pub enum MapLiteralEntry {
    Association { key: MapLiteralKey, value: Expr, range: SourceRange },
    Expansion { expr: Expr, range: SourceRange },
}

#[derive(Debug, Clone)]
pub enum MapLiteralKey {
    BareSymbol { name: String, range: SourceRange },
    Computed { expr: Expr, range: SourceRange },
}

#[derive(Debug, Clone)]
pub struct SetLiteralExpr {
    pub entries: Vec<SetLiteralEntry>,
    pub range: SourceRange,
}

/// `Expansion` is reserved for Spec F, parallel to Map literal entries.
#[derive(Debug, Clone)]
pub enum SetLiteralEntry {
    Element { expr: Expr, range: SourceRange },
    Expansion { expr: Expr, range: SourceRange },
}

#[derive(Debug, Clone)]
pub struct ListLiteralExpr {
    pub elements: Vec<ListLiteralElement>,
    pub range: SourceRange,
}

/// `Expansion` is reserved for Spec F, parallel to Map/Set literal elements/entries.
#[derive(Debug, Clone)]
pub enum ListLiteralElement {
    Element { expr: Expr, range: SourceRange },
    Expansion { expr: Expr, range: SourceRange },
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    /// The written unary operator token. `None` for generated negation.
    pub op_range: Option<SourceRange>,
    pub expr: Expr,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    IntegerDivide,
    Power,
    Modulo,
    ShiftLeft,
    ShiftRight,
    BitAnd,
    BitXor,
    BitOr,
    Equal,
    Same,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Compare,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    /// Unary `+x` — lowers to the `+` getter send.
    Plus,
    /// Unary `-x` — lowers to the `-` getter send.
    Minus,
    /// Prefix `not x` — lowers to the `not` getter send.
    Not,
    /// Prefix `~x` — lowers to the `~` getter send.
    BitNot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureParameter {
    /// The parameter's local binding name.
    pub name: String,
    /// The source span of the parameter name, excluding `*`/`_` markers.
    pub range: SourceRange,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClosureParameters {
    /// Fixed positional parameters, in source order.
    pub fixed: Vec<ClosureParameter>,
    /// Optional terminal positional-rest parameter.
    pub positional_rest: Option<ClosureParameter>,
}

impl ClosureParameters {
    pub fn fixed(names: Vec<String>) -> Self {
        Self {
            fixed: names
                .into_iter()
                .map(|name| ClosureParameter {
                    name,
                    range: SourceRange::default(),
                })
                .collect(),
            positional_rest: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockExpr {
    pub params: ClosureParameters,
    pub body: Vec<Statement>,
    pub expr_body: bool,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct MembershipExpr {
    pub left: Expr,
    pub right: Expr,
    pub negated: bool,
    pub op_range: Option<SourceRange>,
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct IsMembershipExpr {
    pub left: Expr,
    pub candidates: Expr,
    pub strict: bool,
    pub negated: bool,
    pub op_range: Option<SourceRange>,
    pub range: SourceRange,
}
