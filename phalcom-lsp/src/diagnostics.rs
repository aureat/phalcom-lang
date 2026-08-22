//! Maps `phalcom-ast` [`SyntaxError`]s to LSP [`Diagnostic`]s.
//!
//! Stage 1 (ADR-0056 §3, §6) publishes **every** recovered
//! [`SyntaxError`] from a [`phalcom_ast::parser::parse`] call, not just the
//! first — the multi-error win over the subprocess path's single-error
//! `parse_source` (`phalcom-core/bin/phalcom/cli.rs:145`).

use phalcom_ast::error::SyntaxError;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

use crate::line_index::LineIndex;

/// Converts one [`SyntaxError`] into an LSP [`Diagnostic`] using `index` to
/// map its byte [`range`](SyntaxError::range) to a `(line, character)` span.
///
/// The diagnostic's message is `kind.to_string()` (the error's `#[error(..)]`
/// text via its [`std::fmt::Display`] impl); severity is always
/// [`DiagnosticSeverity::ERROR`] — `phalcom-ast` does not currently emit
/// warnings.
pub fn syntax_error_to_diagnostic(error: &SyntaxError, index: &LineIndex) -> Diagnostic {
    Diagnostic {
        range: index.range(error.range.clone()),
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("phalcom".to_string()),
        message: error.kind.to_string(),
        ..Diagnostic::default()
    }
}

/// Converts every [`SyntaxError`] in `errors` into LSP [`Diagnostic`]s.
pub fn syntax_errors_to_diagnostics(errors: &[SyntaxError], index: &LineIndex) -> Vec<Diagnostic> {
    errors.iter().map(|e| syntax_error_to_diagnostic(e, index)).collect()
}

/// Converts a [`phalcom_semantic::SemanticDiagnostic`] into an LSP [`Diagnostic`].
pub fn semantic_diagnostic_to_lsp_diagnostic(diag: &phalcom_semantic::SemanticDiagnostic, index: &LineIndex, uri: &tower_lsp::lsp_types::Url) -> Diagnostic {
    use phalcom_semantic::DiagnosticSeverity as SemSeverity;
    let severity = match diag.severity {
        SemSeverity::Error => DiagnosticSeverity::ERROR,
        SemSeverity::Warning => DiagnosticSeverity::WARNING,
        SemSeverity::Information => DiagnosticSeverity::INFORMATION,
        SemSeverity::Hint => DiagnosticSeverity::HINT,
    };
    let related_information = if !diag.labels.is_empty() {
        Some(
            diag.labels
                .iter()
                .map(|label| tower_lsp::lsp_types::DiagnosticRelatedInformation {
                    location: tower_lsp::lsp_types::Location {
                        uri: uri.clone(),
                        range: index.range(label.range.start..label.range.end),
                    },
                    message: label.message.clone(),
                })
                .collect(),
        )
    } else {
        None
    };

    Diagnostic {
        range: index.range(diag.primary_range.start..diag.primary_range.end),
        severity: Some(severity),
        code: Some(tower_lsp::lsp_types::NumberOrString::String(diag.code.as_str().to_string())),
        source: Some("phalcom-typecheck".to_string()),
        message: diag.message.clone(),
        related_information,
        ..Diagnostic::default()
    }
}

/// Converts every [`SemanticDiagnostic`] into LSP [`Diagnostic`]s.
pub fn semantic_diagnostics_to_lsp_diagnostics(
    diags: &[phalcom_semantic::SemanticDiagnostic],
    index: &LineIndex,
    uri: &tower_lsp::lsp_types::Url,
) -> Vec<Diagnostic> {
    diags.iter().map(|d| semantic_diagnostic_to_lsp_diagnostic(d, index, uri)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::parser::parse;

    #[test]
    fn clean_parse_yields_zero_diagnostics() {
        let source = "let x = 1\nlet y = 2\n";
        let parsed = parse(source, 0);
        assert!(parsed.errors.is_empty(), "expected a clean parse");
        let index = LineIndex::new(source);
        let diags = syntax_errors_to_diagnostics(&parsed.errors, &index);
        assert_eq!(diags.len(), 0);
    }

    #[test]
    fn n_syntax_errors_yield_n_diagnostics() {
        // Two independent malformed statements, separated by newlines so the
        // parser's synchronize-and-resume recovery can find both.
        let source = "let = \nlet = \n";
        let parsed = parse(source, 0);
        assert_eq!(
            parsed.errors.len(),
            2,
            "fixture expected to produce exactly 2 recovered syntax errors, got {:?}",
            parsed.errors
        );
        let index = LineIndex::new(source);
        let diags = syntax_errors_to_diagnostics(&parsed.errors, &index);
        assert_eq!(diags.len(), parsed.errors.len());
        for (diag, err) in diags.iter().zip(parsed.errors.iter()) {
            assert_eq!(diag.message, err.kind.to_string());
            assert_eq!(diag.range, index.range(err.range.clone()));
            assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
        }
    }

    #[test]
    fn semantic_diagnostic_preserves_code_source_and_real_related_uri() {
        let source = "let value: Int = \"text\"\n";
        let index = LineIndex::new(source);
        let uri = tower_lsp::lsp_types::Url::parse("file:///workspace/main.ph").unwrap();
        let semantic = phalcom_semantic::SemanticDiagnostic::error(
            phalcom_semantic::DiagnosticCode::BindingInitializerMismatch,
            "binding initializer is incompatible",
            (4..9).into(),
        )
        .with_label((11..14).into(), "declared type");

        let diagnostic = semantic_diagnostic_to_lsp_diagnostic(&semantic, &index, &uri);

        assert_eq!(diagnostic.source.as_deref(), Some("phalcom-typecheck"));
        assert_eq!(
            diagnostic.code,
            Some(tower_lsp::lsp_types::NumberOrString::String("type.binding.initializer_mismatch".to_string()))
        );
        let related = diagnostic.related_information.expect("related semantic label");
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].location.uri, uri);
        assert_ne!(related[0].location.uri.as_str(), "file:///");
    }
}
