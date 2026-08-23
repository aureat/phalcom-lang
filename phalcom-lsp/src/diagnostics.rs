//! Maps `phalcom-ast` [`SyntaxError`]s to LSP [`Diagnostic`]s.
//!
//! Stage 1 (ADR-0056 §3, §6) publishes **every** recovered
//! [`SyntaxError`] from a [`phalcom_ast::parser::parse`] call, not just the
//! first — the multi-error win over the subprocess path's single-error
//! `parse_source` (`phalcom-core/bin/phalcom/cli.rs:145`).

use std::collections::BTreeMap;

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
    semantic_diagnostic_to_lsp_diagnostic_with_sources(diag, index, uri, &BTreeMap::new())
}

/// Source mapping used to preserve secondary semantic labels across modules.
#[derive(Clone, Debug)]
pub struct SemanticDiagnosticSource {
    /// Canonical editor URI for the source module.
    pub uri: tower_lsp::lsp_types::Url,
    /// Byte-to-LSP position mapping for that module's published source.
    pub line_index: LineIndex,
}

/// Converts one semantic diagnostic while resolving every source-owned label
/// through `sources`. The primary document is supplied separately so callers
/// can use its live unsaved line index.
pub fn semantic_diagnostic_to_lsp_diagnostic_with_sources(
    diag: &phalcom_semantic::SemanticDiagnostic,
    primary_index: &LineIndex,
    primary_uri: &tower_lsp::lsp_types::Url,
    sources: &BTreeMap<phalcom_modules::identity::ModuleId, SemanticDiagnosticSource>,
) -> Diagnostic {
    use phalcom_semantic::DiagnosticSeverity as SemSeverity;
    let severity = match diag.severity {
        SemSeverity::Error => DiagnosticSeverity::ERROR,
        SemSeverity::Warning => DiagnosticSeverity::WARNING,
        SemSeverity::Information => DiagnosticSeverity::INFORMATION,
        SemSeverity::Hint => DiagnosticSeverity::HINT,
    };
    let mut related = Vec::new();
    for label in &diag.labels {
        let source = if label.span.module == diag.primary.module {
            None
        } else {
            sources.get(&label.span.module)
        };
        let (label_uri, label_index) = source.map(|source| (&source.uri, &source.line_index)).unwrap_or((primary_uri, primary_index));
        let message = if source.is_some() {
            label.message.clone()
        } else if label.span.module == diag.primary.module {
            label.message.clone()
        } else {
            format!("{} (source module {})", label.message, label.span.module)
        };
        related.push(tower_lsp::lsp_types::DiagnosticRelatedInformation {
            location: tower_lsp::lsp_types::Location {
                uri: label_uri.clone(),
                range: label_index.range(label.range.start..label.range.end),
            },
            message,
        });
    }
    for note in &diag.notes {
        related.push(tower_lsp::lsp_types::DiagnosticRelatedInformation {
            location: tower_lsp::lsp_types::Location {
                uri: primary_uri.clone(),
                range: primary_index.range(diag.primary_range.start..diag.primary_range.end),
            },
            message: format!("note: {note}"),
        });
    }
    for help in &diag.helps {
        related.push(tower_lsp::lsp_types::DiagnosticRelatedInformation {
            location: tower_lsp::lsp_types::Location {
                uri: primary_uri.clone(),
                range: primary_index.range(diag.primary_range.start..diag.primary_range.end),
            },
            message: format!("help: {help}"),
        });
    }
    let related_information = if !related.is_empty() { Some(related) } else { None };

    Diagnostic {
        range: primary_index.range(diag.primary_range.start..diag.primary_range.end),
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

/// Converts a module's semantic diagnostics with canonical source mappings.
pub fn semantic_diagnostics_to_lsp_diagnostics_with_sources(
    diags: &[phalcom_semantic::SemanticDiagnostic],
    primary_index: &LineIndex,
    primary_uri: &tower_lsp::lsp_types::Url,
    sources: &BTreeMap<phalcom_modules::identity::ModuleId, SemanticDiagnosticSource>,
) -> Vec<Diagnostic> {
    diags
        .iter()
        .map(|diag| semantic_diagnostic_to_lsp_diagnostic_with_sources(diag, primary_index, primary_uri, sources))
        .collect()
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
        let semantic = phalcom_semantic::SemanticDiagnostic::error_in(
            phalcom_modules::identity::ModuleId::core(),
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

    #[test]
    fn semantic_diagnostic_resolves_secondary_label_uri_from_source_module() {
        let primary_source = "let value: Int = \"text\"\n";
        let secondary_source = "const declared: String = 1\n";
        let primary_index = LineIndex::new(primary_source);
        let secondary_index = LineIndex::new(secondary_source);
        let primary_uri = tower_lsp::lsp_types::Url::parse("file:///workspace/main.ph").unwrap();
        let secondary_uri = tower_lsp::lsp_types::Url::parse("file:///workspace/types.ph").unwrap();
        let mut allocator = phalcom_modules::identity::SyntheticProjectIdAllocator;
        let primary_module = phalcom_modules::identity::ModuleId::synthetic(allocator.allocate(), phalcom_modules::identity::ModulePath::root());
        let secondary_module = phalcom_modules::identity::ModuleId::synthetic(allocator.allocate(), phalcom_modules::identity::ModulePath::root());
        let semantic = phalcom_semantic::SemanticDiagnostic::error_in(
            primary_module,
            phalcom_semantic::DiagnosticCode::BindingInitializerMismatch,
            "binding initializer is incompatible",
            (4..9).into(),
        )
        .with_label_in(secondary_module.clone(), (7..15).into(), "declared type");
        let mut sources = BTreeMap::new();
        sources.insert(
            secondary_module,
            SemanticDiagnosticSource {
                uri: secondary_uri.clone(),
                line_index: secondary_index,
            },
        );

        let diagnostic = semantic_diagnostic_to_lsp_diagnostic_with_sources(&semantic, &primary_index, &primary_uri, &sources);
        let related = diagnostic.related_information.expect("related semantic label");
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].location.uri, secondary_uri);
        assert_eq!(related[0].location.range, LineIndex::new(secondary_source).range(7..15));
    }
}
