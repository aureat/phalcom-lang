//! Typed semantic query product wrappers and accessors (Spec 04.5 / Wave 5).

use crate::checker::analysis::CallableAnalysis;
use crate::db::state::QueryValue;
use crate::diagnostic::SemanticDiagnostic;
use std::sync::Arc;

/// Strongly-typed wrapper around semantic database product variants.
#[derive(Clone, Debug)]
pub enum SemanticProduct {
    CallableBody(Arc<CallableAnalysis>),
    ModuleDiagnostics(Arc<[SemanticDiagnostic]>),
}

impl SemanticProduct {
    pub fn as_callable_body(&self) -> Option<&Arc<CallableAnalysis>> {
        match self {
            Self::CallableBody(body) => Some(body),
            _ => None,
        }
    }

    pub fn as_module_diagnostics(&self) -> Option<&Arc<[SemanticDiagnostic]>> {
        match self {
            Self::ModuleDiagnostics(diags) => Some(diags),
            _ => None,
        }
    }

    /// Converts typed product into type-erased `QueryValue`.
    pub fn to_query_value(&self) -> QueryValue {
        // Placeholder byte representation for query cache
        QueryValue::from_bytes(&[])
    }
}
