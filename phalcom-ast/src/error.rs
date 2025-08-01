use std::fmt::{Display, Formatter};
use std::ops::Range;
use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub struct SyntaxError {
    pub kind: SyntaxErrorKind,
    pub range: Range<usize>,
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub fn format_expected(expected: &[String]) -> String {
    if expected.len() == 1 {
        format!("Expected {}", expected[0])
    } else {
        let mut s = String::from("Expected one of ");
        for (i, e) in expected.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(e);
        }
        s
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SyntaxErrorKind {
    #[error("Extraneous input: {token:?}")]
    ExtraToken { token: String },

    #[error("Invalid token")]
    InvalidToken,

    #[error(r#"init() should not return a value"#)]
    ReturnInInitializer,

    #[error(r#""return" used outside function"#)]
    ReturnOutsideFunction,

    #[error(r#""super" used outside class"#)]
    SuperOutsideClass,

    #[error(r#""super" used in class without a superclass"#)]
    SuperWithoutSuperclass,

    #[error(r#""self" used outside class"#)]
    SelfOutsideClass,

    #[error("Unexpected input: {token:?}")]
    UnexpectedInput { token: String },

    #[error("Unexpected end of file. {}", format_expected(expected))]
    UnrecognizedEof { expected: Vec<String> },

    #[error("{}", format_expected(expected))]
    UnrecognizedToken { token: String, expected: Vec<String> },

    #[error("Invalid integer")]
    InvalidInteger,

    #[error("Invalid integer")]
    InvalidFloat,

    #[error("Unterminated string")]
    UnterminatedString,

    #[error("Unterminated comment")]
    UnterminatedComment,

    #[error("Unknown error")]
    Unknown,

    #[error("{0}")]
    Message(String),
}
