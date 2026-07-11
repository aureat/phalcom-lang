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

#[derive(Debug, Clone)]
pub struct LetBinding {
    pub name: String,
    pub value: Option<Expr>,
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
    Nil { range: SourceRange },
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
            | Expr::Nil { range }
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
