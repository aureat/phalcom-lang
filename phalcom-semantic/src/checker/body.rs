//! Full callable body type checking and CallableAnalysis generation (Spec 04.5 / Wave 5).

use crate::checker::analysis::{CallableAnalysis, CallableAnalysisStatus};
use crate::checker::context::CheckingContext;
use crate::checker::flow::graph::FlowGraph;
use crate::checker::statement::check_statement;
use crate::db::budget::{CancellationToken, QueryBudget};
use crate::declarations::DeclarationTypeTable;
use crate::identity::{CallableId, ModuleId};
use crate::types::annotation::TypeResolver;
use crate::types::relation::TypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_ast::ast::Statement;
use phalcom_common::range::SourceRange;
use std::sync::Arc;

fn stmt_range(stmt: &Statement) -> SourceRange {
    match stmt {
        Statement::Class(c) => c.range,
        Statement::TypeAlias(t) => t.range,
        Statement::Let(l) => l.range,
        Statement::Return(r) => r.range,
        Statement::Expr { range, .. } => *range,
        Statement::For(f) => f.range,
        Statement::Break { range } => *range,
        Statement::Continue { range } => *range,
        Statement::Throw { range, .. } => *range,
        Statement::Export(e) => e.range,
    }
}

use crate::dispatch::SurfaceDispatchResolver;

/// Context holding canonical published semantic inputs for callable body checking.
pub struct BodyAnalysisContext<'a> {
    pub store: &'a mut TypeStore,
    pub hierarchy: &'a dyn TypeHierarchy,
    pub resolver: &'a dyn TypeResolver,
    pub declarations: &'a DeclarationTypeTable,
    pub dispatch: &'a SurfaceDispatchResolver,
    pub module: ModuleId,
}

/// Analyzes a single callable body and returns a complete [`CallableAnalysis`].
pub fn analyze_callable_body(
    callable: CallableId,
    body: &[Statement],
    body_range: SourceRange,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    resolver: &dyn TypeResolver,
    declarations: &DeclarationTypeTable,
    dispatch: &SurfaceDispatchResolver,
    module: ModuleId,
    mut budget: QueryBudget,
    cancel: &CancellationToken,
) -> CallableAnalysis {
    let mut ctx = CheckingContext::new_with_dispatch_ref(store, hierarchy, resolver, declarations, dispatch, module);
    ctx.current_class = Some(callable.owner.clone());
    ctx.current_side = callable.side;
    ctx.semantic_dependencies.insert(crate::checker::analysis::SemanticDependency::CallableSignature(callable.clone()));

    // 1. Build flow graph for the body statements
    let flow_graph = Arc::new(FlowGraph::from_statements(body));
    ctx.flow_graph = Some(flow_graph);

    // Bind parameters and expected return if available from dispatch surface
    let sig_opt = ctx.dispatch.get().surfaces().get(&callable.owner).and_then(|surface| {
        let member_surface = match callable.side {
            crate::identity::DispatchSide::Instance => &surface.instance,
            crate::identity::DispatchSide::Class => &surface.class,
        };
        member_surface
            .callable_signatures
            .get(&callable.selector)
            .cloned()
            .or_else(|| surface.class.callable_signatures.get(&callable.selector).cloned())
    });

    if let Some(sig) = sig_opt {
        ctx.push_scope();
        for param in &sig.parameters {
            let ty_opt = param.ty.ty();
            ctx.bind_local_var(param.local_name.clone(), ty_opt, param.ty.clone(), false, None, body_range);
        }
        if let Some(ret_ty) = sig.return_type.ty() {
            ctx.expected_return = Some(crate::types::evidence::TypeKnowledge::known(
                ret_ty,
                crate::types::evidence::EvidenceAuthority::Declared,
            ));
        }
    }

    // 2. Check each statement while charging budget and checking cancellation
    let mut status = CallableAnalysisStatus::Complete;

    for stmt in body {
        if cancel.is_cancelled() {
            status = CallableAnalysisStatus::Cancelled;
            break;
        }

        if let Err(report) = budget.charge_step() {
            ctx.diagnostics.push(crate::diagnostic::SemanticDiagnostic::warning_in(
                ctx.current_module.clone(),
                crate::diagnostic::DiagnosticCode::AnalysisBudgetExceeded,
                format!("callable body analysis exceeded step budget ({}/{})", report.used, report.limit),
                stmt_range(stmt),
            ));
            status = CallableAnalysisStatus::BudgetExceeded;
            break;
        }

        check_statement(&mut ctx, stmt);
    }

    ctx.finalize(callable, body_range, status)
}
