use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum PhalcomError {
    #[error("SyntaxError: {0}")]
    SyntaxError(SyntaxError),
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SyntaxError {
    #[error("extraneous input: {token:?}")]
    ExtraToken { token: String },
    #[error("invalid input")]
    InvalidToken,
    #[error(r#"init() should not return a value"#)]
    ReturnInInitializer,
    #[error(r#""return" used outside function"#)]
    ReturnOutsideFunction,
    #[error(r#""super" used outside class"#)]
    SuperOutsideClass,
    #[error(r#""super" used in class without a superclass"#)]
    SuperWithoutSuperclass,
    #[error(r#""this" used outside class"#)]
    ThisOutsideClass,
    #[error("unexpected input")]
    UnexpectedInput { token: String },
    #[error("unexpected end of file")]
    UnrecognizedEof { expected: Vec<String> },
    #[error("unexpected {token:?}. Expected one of: {expected:?}")]
    UnrecognizedToken {
        token: String,
        expected: Vec<String>,
    },
    #[error("unterminated string")]
    UnterminatedString,
    #[error("unterminated comment")]
    UnterminatedComment,
    #[error("other")]
    Other,
}
