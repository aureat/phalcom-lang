#[derive(Debug, Default)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Class(ClassDef),
    Let(LetBinding),
    Return(ReturnStatement),
    Expr(Expr),
}

#[derive(Debug)]
pub struct ClassDef {
    pub name: String,
    pub members: Vec<ClassMember>,
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
}

#[derive(Debug)]
pub struct GetterDef {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct SetterDef {
    pub name: String,
    pub param: String,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub struct LetBinding {
    pub name: String,
    pub value: Option<Expr>,
}

#[derive(Debug)]
pub struct ReturnStatement {
    pub value: Option<Expr>,
}

#[derive(Debug)]
pub enum Expr {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
    Var(String),
    SelfVar,
    SuperVar,
    Assignment(Box<AssignmentExpr>),
    Unary(Box<UnaryExpr>),
    Binary(Box<BinaryExpr>),
    Call(Box<CallExpr>),
    MethodCall(Box<MethodCallExpr>),
    GetProperty(Box<GetPropertyExpr>),
    SetProperty(Box<SetPropertyExpr>),
}

#[derive(Debug)]
pub struct AssignmentExpr {
    pub name: Box<Expr>,
    pub value: Expr,
}

#[derive(Debug)]
pub struct CallExpr {
    pub callee: Expr,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub struct MethodCallExpr {
    pub object: Expr,
    pub method: String,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub struct GetPropertyExpr {
    pub object: Expr,
    pub property: String,
}

#[allow(dead_code)]
enum Suffix {
    Method(String, Vec<Expr>), // .foo(args)
    Property(String),          // .foo
    Call(Vec<Expr>),           // (args)
}

#[derive(Debug)]
pub struct SetPropertyExpr {
    pub object: Expr,
    pub property: String,
    pub value: Expr,
}

#[derive(Debug)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub left: Expr,
    pub right: Expr,
}

#[derive(Debug)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Expr,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum UnaryOp {
    Negate,
    Not,
}
