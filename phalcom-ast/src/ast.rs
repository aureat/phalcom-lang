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
}

#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    pub members: Vec<ClassMember>,
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

#[derive(Debug, Clone)]
pub struct ParameterDef {
    pub name: String,
    pub label: Option<String>,
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

/// A `let`/`var` name binding, optionally with an initializer.
///
/// The [`kind`](LetBinding::kind) distinguishes the immutable `let` form from
/// the mutable `var` form (ADR-0014). A missing [`value`](LetBinding::value)
/// means no initializer was written; the compiler rejects that for `let` and
/// surfaces `None` for `var`.
#[derive(Debug, Clone)]
pub struct LetBinding {
    /// Whether this is an immutable `let` or a mutable `var` binding.
    pub kind: BindingKind,
    /// The bound name.
    pub name: String,
    /// The initializer expression, or `None` if the binding has no `= expr`.
    pub value: Option<Expr>,
    /// The source span covering the whole binding statement.
    pub range: SourceRange,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub value: Option<Expr>,
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
