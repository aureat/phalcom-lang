use thiserror::Error;

#[derive(Error, Debug)]
pub enum PhalcomError {
    #[error("Type Error: {0}")]
    TypeError(String),

    #[error("Method Not Found: No method with selector '{selector}' found for class '{class_name}'.")]
    MethodNotFound {
        selector: String,
        class_name: String,
    },

    #[error("Argument Error: {0}")]
    ArgumentError(String),

    // You will add many more as the VM and compiler are built
    #[error("Compilation Error: {0}")]
    CompilationError(String),

    #[error("Internal VM Error: {0}")]
    InternalVMError(String),
}