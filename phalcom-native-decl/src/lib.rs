//! Shared parser and validator for native primitive declarations.

mod normalized;
mod parse;
mod validate;

pub use normalized::{NormalizedPrimitiveDecl, PrimitiveDeclField, PrimitiveDeclKey};
pub use parse::{docs_from_attributes, parse_primitive_attribute};
pub use validate::validate_decl;

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DeclError {
    #[error("invalid primitive declaration: {0}")]
    Parse(String),
    #[error("invalid primitive declaration field `{0}`")]
    UnknownField(String),
    #[error("duplicate primitive declaration field `{0}`")]
    DuplicateField(String),
    #[error("invalid primitive selector: {0}")]
    InvalidSelector(String),
    #[error("invalid primitive metadata: {0}")]
    InvalidMetadata(String),
}
