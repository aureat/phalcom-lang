use crate::compiler::lib::CompilerError;
use phalcom_ast::error::SyntaxError;
use std::io::Error as IoError;
use thiserror::Error;

pub type PhResult<T> = Result<T, PhError>;

#[derive(Error, Debug)]
pub enum PhError {
    #[error(transparent)]
    Runtime(#[from] RuntimeError),

    #[error(transparent)]
    Io(#[from] IoError),

    #[error("VM Error: {message}\nStack Trace:\n{stack_trace}")]
    VMError { message: String, stack_trace: String },

    #[error(transparent)]
    Parse(SyntaxError),

    #[error(transparent)]
    Compile(CompilerError),

    #[error("{0}")]
    StringError(String),

    #[error("{0}")]
    StrError(&'static str),
}

impl From<&'static str> for PhError {
    fn from(err: &'static str) -> Self {
        PhError::StrError(err)
    }
}

fn format_num_arguments<'a>(args: usize) -> String {
    if args == 1 {
        String::from("1 argument")
    } else {
        format!("{} arguments", args)
    }
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Method {signature} expected {}, got {found}", format_num_arguments(*expected))]
    Arity { signature: &'static str, expected: usize, found: usize },

    #[error("Expected {expected}, got {found}")]
    Type { expected: &'static str, found: &'static str },

    #[error("Can't set superclass of a class")]
    InvalidSetSuper,

    #[error("Can't set class of an object")]
    InvalidSetClass,

    #[error("Undefined variable `{0}`")]
    UndefinedVar(String),

    #[error("Division by zero")]
    ZeroDivision,

    #[error("Can't convert {found} to {expected}")]
    TypeConversion { expected: &'static str, found: &'static str },

    #[error("{0}")]
    NotAllowed(String),

    #[error("Invalid argument: {0}")]
    ArgumentError(String),

    #[error("{0}")]
    Message(String),
}

#[macro_export]
macro_rules! ensure_arity {
    ($signature: expr, $args:expr, $expected:expr) => {
        if $args.len() != $expected {
            return Err(RuntimeError::Arity {
                signature: $signature,
                expected: $expected,
                found: $args.len(),
            }
            .into());
        }
    };
}

#[macro_export]
macro_rules! expect_value {
    ($value:expr, String) => {{
        match $value {
            Value::String(s) => s.clone(),
            found => {
                return Err(RuntimeError::Type {
                    expected: "String",
                    found: found.type_name(),
                }
                .into());
            }
        }
    }};
    ($value:expr, Number) => {{
        match $value {
            Value::Number(n) => *n,
            other => {
                return Err(RuntimeError::Type {
                    expected: "Number",
                    found: other.type_name(),
                }
                .into());
            }
        }
    }};
    ($value:expr, Bool) => {{
        match $value {
            Value::Bool(b) => b,
            other => {
                return Err(RuntimeError::Type {
                    expected: "Bool",
                    found: other.type_name(),
                }
                .into());
            }
        }
    }};
    ($value:expr, Symbol) => {{
        match $value {
            Value::Symbol(s) => s,
            other => {
                return Err(RuntimeError::Type {
                    expected: "Symbol",
                    found: other.type_name(),
                }
                .into());
            }
        }
    }};
    ($value:expr, Nil) => {{
        match $value {
            Value::Nil => (),
            other => {
                return Err(RuntimeError::Type {
                    expected: "Nil",
                    found: other.type_name(),
                }
                .into());
            }
        }
    }};
    ($value:expr, Instance) => {{
        match $value {
            Value::Instance(inst) => inst,
            other => {
                return Err(RuntimeError::Type {
                    expected: "Instance",
                    found: other.type_name(),
                }
                .into());
            }
        }
    }};
    ($value:expr, Class) => {{
        match $value {
            Value::Class(class) => class,
            other => {
                return Err(RuntimeError::Type {
                    expected: "Class",
                    found: other.type_name(),
                }
                .into());
            }
        }
    }};
    ($value:expr, Method) => {{
        match $value {
            Value::Method(method) => method,
            other => {
                return Err(RuntimeError::Type {
                    expected: "Method",
                    found: other.type_name(),
                }
                .into());
            }
        }
    }}; // ($value:expr, $type:ident) => {{
        //     return Err(RuntimeError::Type {
        //         expected: stringify!($type),
        //         found: $value.type_name(),
        //     }
        //     .into());
        // }};
}

// #[macro_export]
// macro_rules! ensure_instance_of {
//     ($vm:expr, $val:expr, $class_id:expr) => {{
//         let inst = expect!($val, Instance);
//         if inst.borrow().class().borrow().symbol != $class_id {
//             return Err(RuntimeError::Type {
//                 expected: $vm.symbol_to_string($class_id).borrow().as_str(),
//                 found: inst.borrow().class().borrow().name.as_str(),
//             }
//             .into());
//         }
//         inst
//     }};
// }
