//! Exhaustive relation-policy adapter (Spec 04.5 / E6).
//!
//! Enforces that only Refuted outcomes produce fatal type mismatch diagnostics,
//! while Unknown, Dynamic, Blocked, and BudgetExceeded states propagate
//! honestly without generating spurious secondary errors.

use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::types::evidence::TypeKnowledge;
use crate::types::outcome::RelationOutcome;
use crate::types::relation::{Assignability, TypeHierarchy, check_assignability};
use crate::types::store::TypeStore;
use phalcom_common::range::SourceRange;

/// Shared relation outcome policy enforcement.
/// Returns `true` if the relation holds (or is non-refuting), `false` if refuted.
pub fn handle_relation_outcome<T>(
    outcome: &RelationOutcome<T>,
    code: DiagnosticCode,
    message: impl Into<String>,
    range: SourceRange,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> bool {
    match outcome {
        RelationOutcome::Refuted(_) => {
            diagnostics.push(SemanticDiagnostic::error(code, message.into(), range));
            false
        }
        _ => true,
    }
}

/// Enforces assignability between an actual and expected type knowledge.
/// Emits a diagnostic only when assignability is definitively refuted.
pub fn enforce_assignability(
    store: &TypeStore,
    hierarchy: &dyn TypeHierarchy,
    actual: &TypeKnowledge,
    expected: &TypeKnowledge,
    code: DiagnosticCode,
    message: impl Into<String>,
    range: SourceRange,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) -> bool {
    let outcome = check_assignability(store, hierarchy, actual, expected);
    match outcome {
        Assignability::Refuted { .. } => {
            diagnostics.push(SemanticDiagnostic::error(code, message.into(), range));
            false
        }
        _ => true,
    }
}
