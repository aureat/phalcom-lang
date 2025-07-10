use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Variable '{0}' not found.")]
    NotFound(String),
    #[error("Invalid assignment target.")]
    InvalidAssignment,
}
