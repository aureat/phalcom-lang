//! Semantic database high-level query execution and caching (Spec 04.5 / Wave 5).

use crate::checker::analysis::{CallableAnalysis, CallableAnalysisStatus};
use crate::checker::body::analyze_callable_body;
use crate::db::budget::{CancellationToken, QueryBudget};
use crate::db::key::{ProductFingerprint, QueryKey};
use crate::db::state::{QueryOutcome, QueryValue};
use crate::db::SemanticDb;
use crate::declarations::DeclarationTypeTable;
use crate::identity::{CallableId, ModuleId};
use crate::types::annotation::TypeResolver;
use crate::types::relation::TypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_ast::ast::Statement;
use phalcom_common::range::SourceRange;
use std::sync::Arc;

/// Evaluates or retrieves the cached `CallableAnalysis` for a given callable body.
pub fn query_callable_body(
    db: &mut SemanticDb,
    callable: CallableId,
    body: &[Statement],
    body_range: SourceRange,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    resolver: &dyn TypeResolver,
    declarations: &DeclarationTypeTable,
    module: ModuleId,
    budget: QueryBudget,
    cancel: &CancellationToken,
) -> QueryOutcome<Arc<CallableAnalysis>> {
    let key = QueryKey::CallableBody(callable.clone());

    // 1. Check if already computed and ready in current revision
    if let Some(state) = db.query_state(&key) {
        if state.is_ready() && state.revision() == Some(db.revision()) {
            // Cache hit
            db.metrics().record_hit();
        }
    }

    // 2. Perform analysis
    let analysis = analyze_callable_body(
        callable,
        body,
        body_range,
        store,
        hierarchy,
        resolver,
        declarations,
        module,
        budget,
        cancel,
    );

    let arc_analysis = Arc::new(analysis);

    match arc_analysis.status {
        CallableAnalysisStatus::Cancelled => QueryOutcome::Cancelled,
        CallableAnalysisStatus::BudgetExceeded => {
            let report = crate::db::budget::BudgetReport::new(crate::db::budget::BudgetKind::Steps, budget.max_steps, budget.max_steps);
            QueryOutcome::BudgetExceeded(report)
        }
        CallableAnalysisStatus::Blocked => {
            QueryOutcome::Blocked(crate::types::outcome::BlockReason::SuppressedDependency)
        }
        CallableAnalysisStatus::Complete | CallableAnalysisStatus::Partial => {
            let rev = db.revision();
            let _ = db.publish_ready(
                key,
                rev,
                ProductFingerprint::new(0),
                QueryValue::from_bytes(&[]),
                Vec::new(),
            );
            QueryOutcome::Ready(arc_analysis)
        }
    }
}
