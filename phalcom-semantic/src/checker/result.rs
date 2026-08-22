//! Type check results and reports.

use crate::diagnostic::SemanticDiagnostic;

#[derive(Clone, Debug, Default)]
pub struct TypeCheckReport {
    pub diagnostics: Vec<SemanticDiagnostic>,
}

impl TypeCheckReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == crate::diagnostic::DiagnosticSeverity::Error)
    }

    pub fn add(&mut self, diagnostic: SemanticDiagnostic) {
        self.diagnostics.push(diagnostic);
    }
}
