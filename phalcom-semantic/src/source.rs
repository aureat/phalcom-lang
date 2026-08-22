//! Parse-once source artifact representation.

use phalcom_ast::ast::Program;
use phalcom_modules::identity::{ModuleId, SourceLocation};
use phalcom_modules::source::ModuleKind;
use std::sync::Arc;

/// Retained parsed source artifact preventing redundant reparsing.
#[derive(Clone, Debug)]
pub struct ParsedSourceUnit {
    pub module: ModuleId,
    pub kind: ModuleKind,
    pub source: Option<SourceLocation>,
    pub text: Arc<str>,
    pub program: Arc<Program>,
}

impl ParsedSourceUnit {
    pub fn new(module: ModuleId, kind: ModuleKind, source: Option<SourceLocation>, text: Arc<str>, program: Arc<Program>) -> Self {
        Self {
            module,
            kind,
            source,
            text,
            program,
        }
    }
}
