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

#[derive(Debug)]
pub enum Statement {
    Class(ClassDef),
    Let(LetBinding),
    Return(ReturnStatement),
    Expr { expr: Expr, range: SourceRange },
}

#[derive(Debug)]
pub struct ClassDef {
    pub name: String,
    pub members: Vec<ClassMember>,
    pub range: SourceRange,
}

#[derive(Debug)]
pub enum ClassMember {
    Method(MethodDef),
    Getter(GetterDef),
    Setter(SetterDef),
}

#[derive(Debug)]
pub struct MethodDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
    pub is_static: bool,
    pub range: SourceRange,
}

#[derive(Debug)]
pub struct GetterDef {
    pub name: String,
    pub body: Vec<Statement>,
    pub is_static: bool,
    pub range: SourceRange,
}

#[derive(Debug)]
pub struct SetterDef {
    pub name: String,
    pub param: String,
    pub body: Vec<Statement>,
    pub is_static: bool,
    pub range: SourceRange,
}

#[derive(Debug)]
pub struct LetBinding {
    pub name: String,
    pub value: Option<Expr>,
    pub range: SourceRange,
}

#[derive(Debug)]
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
pub struct MethodCallExpr {
    pub object: Expr,
    pub method: String,
    pub args: Vec<Expr>,
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
