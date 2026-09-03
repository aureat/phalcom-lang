//! Exact semantic diagnostic lock for canonical Universe bootstrap.

use phalcom_semantic::SemanticSnapshot;
use phalcom_semantic::diagnostic::DiagnosticSeverity;
use thiserror::Error;

const EXPECTED: &str = include_str!("../../core/universe/semantic-diagnostics-baseline.txt");

/// Stable canonical semantic-error identity, rendered one diagnostic per line.
pub(crate) fn canonical_error_baseline(snapshot: &SemanticSnapshot) -> String {
    let mut lines = snapshot
        .diagnostics
        .values()
        .flat_map(|diagnostics| diagnostics.iter())
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .map(|diagnostic| {
            format!(
                "{}\t{}\t{}\t{}",
                diagnostic.primary.module,
                diagnostic.code.as_str(),
                diagnostic.primary_range.start,
                diagnostic.primary_range.end,
            )
        })
        .collect::<Vec<_>>();
    lines.sort();

    if lines.is_empty() { String::new() } else { format!("{}\n", lines.join("\n")) }
}

#[derive(Debug, Error)]
pub(crate) enum CanonicalSemanticBaselineError {
    #[error("canonical Universe semantic error baseline changed\n--- expected ---\n{expected}--- actual ---\n{actual}")]
    Mismatch { expected: Box<str>, actual: Box<str> },
}

pub(crate) fn validate_canonical_error_baseline(snapshot: &SemanticSnapshot) -> Result<(), CanonicalSemanticBaselineError> {
    compare_baselines(EXPECTED, &canonical_error_baseline(snapshot))
}

fn compare_baselines(expected: &str, actual: &str) -> Result<(), CanonicalSemanticBaselineError> {
    if expected == actual {
        Ok(())
    } else {
        Err(CanonicalSemanticBaselineError::Mismatch {
            expected: expected.into(),
            actual: actual.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::compare_baselines;

    #[test]
    fn identical_baseline_is_accepted() {
        assert!(compare_baselines("a\n", "a\n").is_ok());
    }

    #[test]
    fn additional_canonical_error_is_rejected() {
        let error = compare_baselines("a\n", "a\nb\n").expect_err("new error must fail");
        assert!(error.to_string().contains("b"));
    }

    #[test]
    fn removed_canonical_error_requires_baseline_update() {
        assert!(compare_baselines("a\nb\n", "a\n").is_err());
    }
}
