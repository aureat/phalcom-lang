//! Semantic database high-level query execution and caching (Spec 04.5 / Wave 5).

use crate::checker::analysis::{CallableAnalysis, CallableAnalysisStatus};
use crate::checker::body::analyze_callable_body;
use crate::db::SemanticDb;
use crate::db::SemanticProduct;
use crate::db::budget::{CancellationToken, QueryBudget};
use crate::db::key::{ProductFingerprint, QueryKey};
use crate::db::state::{QueryOutcome, QueryState};
use crate::declarations::DeclarationTypeTable;
use crate::dispatch::SurfaceDispatchResolver;
use crate::identity::{CallableId, ModuleId};
use crate::types::annotation::TypeResolver;
use crate::types::relation::TypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_ast::ast::Statement;
use phalcom_common::range::SourceRange;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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
    dispatch: &SurfaceDispatchResolver,
    module: ModuleId,
    budget: QueryBudget,
    cancel: &CancellationToken,
) -> QueryOutcome<Arc<CallableAnalysis>> {
    let key = QueryKey::CallableBody(callable.clone());

    let input_fingerprint = callable_input_fingerprint(&callable, body, body_range, store);

    // 1. Check if already computed and ready for the same callable input.
    // The query key identifies the callable, while this fingerprint identifies
    // the current body/product request. Without the second check, an edit to a
    // body with the same CallableId would return a stale typed product.
    let reusable = db
        .query_state(&key)
        .map(|state| state.is_ready() && state.fingerprint() == Some(input_fingerprint))
        .unwrap_or(false);
    if reusable {
        if let Some(product) = db.product(&key).and_then(|product| product.as_callable_body()) {
            db.metrics().record_hit();
            return QueryOutcome::Ready(product.clone());
        }
    }

    // A ready product with a different input, or a non-ready state from an
    // earlier attempt, cannot remain in the dependency index while this
    // generation recomputes it. Invalidation also clears dependents that
    // consumed the old callable result.
    if db.query_state(&key).is_some() {
        db.invalidate([key.clone()]);
    }
    db.metrics().record_miss();

    // 2. Perform analysis
    let analysis = analyze_callable_body(callable, body, body_range, store, hierarchy, resolver, declarations, dispatch, module, budget, cancel);

    let mut analysis = analysis;
    analysis.dependency_fingerprint = callable_fingerprint(&analysis);
    let arc_analysis = Arc::new(analysis);

    match arc_analysis.status {
        CallableAnalysisStatus::Cancelled => {
            db.metrics().record_cancellation();
            db.set_state(key, QueryState::Cancelled { revision: db.revision() });
            QueryOutcome::Cancelled
        }
        CallableAnalysisStatus::BudgetExceeded => {
            let report = crate::db::budget::BudgetReport::new(crate::db::budget::BudgetKind::Steps, budget.max_steps, budget.max_steps);
            db.metrics().record_budget_exhaustion();
            db.set_state(
                key,
                QueryState::BudgetExceeded {
                    revision: db.revision(),
                    report: report.clone(),
                },
            );
            QueryOutcome::BudgetExceeded(report)
        }
        CallableAnalysisStatus::Blocked => {
            let reason = crate::types::outcome::BlockReason::SuppressedDependency;
            db.set_state(
                key,
                QueryState::Blocked {
                    revision: db.revision(),
                    reason: reason.clone(),
                },
            );
            QueryOutcome::Blocked(reason)
        }
        CallableAnalysisStatus::Complete | CallableAnalysisStatus::Partial => {
            let rev = db.revision();
            let _ = db.publish_product_ready(key, rev, input_fingerprint, SemanticProduct::CallableBody(arc_analysis.clone()), Vec::new());
            QueryOutcome::Ready(arc_analysis)
        }
    }
}

fn callable_input_fingerprint(callable: &CallableId, body: &[Statement], body_range: SourceRange, store: &TypeStore) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    callable.hash(&mut hasher);
    body_range.start.hash(&mut hasher);
    body_range.end.hash(&mut hasher);
    store.id().hash(&mut hasher);
    for statement in body {
        // AST nodes intentionally do not expose a second semantic cache key.
        // Their canonical Debug form is stable for the in-memory query input
        // and captures edits that preserve callable identity and range.
        format!("{statement:?}").hash(&mut hasher);
    }
    ProductFingerprint::new(hasher.finish())
}

fn callable_fingerprint(analysis: &CallableAnalysis) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    analysis.callable.hash(&mut hasher);
    analysis.body_range.start.hash(&mut hasher);
    analysis.body_range.end.hash(&mut hasher);
    analysis.expressions.len().hash(&mut hasher);
    analysis.bindings.len().hash(&mut hasher);
    analysis.dependencies.len().hash(&mut hasher);
    analysis.status.hash(&mut hasher);
    for dependency in analysis.dependencies.iter() {
        dependency.hash(&mut hasher);
    }
    ProductFingerprint::new(hasher.finish())
}
