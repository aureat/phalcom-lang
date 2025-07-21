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
    pub is_static: bool,
}

#[derive(Debug)]
pub struct GetterDef {
    pub name: String,
    pub body: Vec<Statement>,
    pub is_static: bool,
}

#[derive(Debug)]
pub struct SetterDef {
    pub name: String,
    pub param: String,
    pub body: Vec<Statement>,
    pub is_static: bool,
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

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
    Var(String),
    Field(String),
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

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub left: Expr,
    pub right: Expr,
}

#[derive(Debug, Clone)]
pub struct SetPropertyExpr {
    pub object: Expr,
    pub property: String,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct AssignmentExpr {
    pub name: Box<Expr>,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Expr,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub object: Expr,
    pub method: String,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct GetPropertyExpr {
    pub object: Expr,
    pub property: String,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Expr,
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
