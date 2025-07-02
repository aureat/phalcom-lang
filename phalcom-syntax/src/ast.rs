use phalcom_vm::value::Value;

// The root of a program is a sequence of statements.
pub type Program = Vec<Stmt>;

// Statements do not produce values.
#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    ClassDef {
        name: String,
        body: Vec<Stmt>, // For now, methods are statements
    },
    MethodDef {
        name: String,
        params: Vec<String>,
        body: Vec<Expr>, // Method body is a list of expressions
    },
    Let {
        name: String,
        value: Expr,
    },
    Expr(Expr),
}

// Expressions evaluate to a value.
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Literal(Value),
    // A message send, like `receiver.add(arg)` or `receiver + arg`
    Message {
        receiver: Box<Expr>,
        selector: String,
        args: Vec<Expr>,
    },
    Variable(String),
}
