//! Reedline validator for multi-line REPL input classification.
//!
//! Classifies input buffers using the parser's [`SyntaxErrorKind::UnrecognizedEof`]
//! signal: truncated blocks request continuation (`...`), complete blocks or
//! invalid blocks submit immediately.

use phalcom_ast::error::SyntaxErrorKind;
use phalcom_ast::parser::parse;
use reedline::{ValidationResult, Validator};

/// Classification of a REPL input buffer.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Verdict {
    /// Parsed with no errors — evaluate it.
    Complete,
    /// Ran out of input — prompt for continuation.
    Incomplete,
    /// Malformed token or syntax error — submit to display diagnostic.
    Invalid,
}

/// Classifies `src` according to U-REPL §D7 rules.
///
/// Ensures statement-terminating newline normalization before classification.
pub fn classify(src: &str) -> Verdict {
    let src_norm = if src.ends_with('\n') {
        src.to_string()
    } else {
        format!("{src}\n")
    };

    let parsed = parse(&src_norm, 0);
    if parsed.errors.is_empty() {
        Verdict::Complete
    } else if parsed
        .errors
        .iter()
        .any(|e| matches!(e.kind, SyntaxErrorKind::UnrecognizedEof { .. }))
    {
        Verdict::Incomplete
    } else {
        Verdict::Invalid
    }
}

/// Reedline validator implementing §D7 classification.
pub struct PhalcomValidator;

impl Validator for PhalcomValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        match classify(line) {
            Verdict::Incomplete => ValidationResult::Incomplete,
            _ => ValidationResult::Complete,
        }
    }
}
