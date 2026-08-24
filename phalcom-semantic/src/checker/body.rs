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
use phalcom_ast::ast::{Expr, Statement};
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

use crate::dispatch::{CallableSignature, SurfaceDispatchResolver};

/// Returns the declaration signature consumed by a body, including the class-side
/// fallback used for constructor bodies represented on the instance side.
pub(crate) fn signature_consumed_by_body(
    dispatch: &SurfaceDispatchResolver,
    callable: &CallableId,
) -> Option<(CallableId, CallableSignature)> {
    let surface = dispatch.surfaces().get(&callable.owner)?;

    if let Some(signature) = surface.get_callable(callable.side, &callable.selector) {
        let signature_id = surface
            .get_callable_id(callable.side, &callable.selector)
            .cloned()
            .unwrap_or_else(|| callable.clone());
        return Some((signature_id, signature.clone()));
    }

    if callable.side == crate::identity::DispatchSide::Instance {
        let signature = surface.get_callable(crate::identity::DispatchSide::Class, &callable.selector)?;
        let signature_id = surface
            .get_callable_id(crate::identity::DispatchSide::Class, &callable.selector)
            .cloned()
            .unwrap_or_else(|| {
                CallableId::new(
                    callable.owner.clone(),
                    callable.selector.clone(),
                    crate::identity::DispatchSide::Class,
                )
            });
        return Some((signature_id, signature.clone()));
    }

    None
}

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

    // 1. Build flow graph for the body statements
    let flow_graph = Arc::new(FlowGraph::from_statements(body));
    ctx.flow_graph = Some(flow_graph);

    // Bind parameters and expected return from the exact published signature consumed by this body.
    // Constructor bodies are represented as instance-side bodies, while their public constructor
    // signatures live on the class side; record the class-side identity in that fallback case.
    let sig_opt = signature_consumed_by_body(ctx.dispatch_ref(), &callable);

    if let Some((signature_id, sig)) = sig_opt {
        ctx.record_consumed_callable_signature(&signature_id, &sig);
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

    for (statement_index, stmt) in body.iter().enumerate() {
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

        let is_tail = statement_index + 1 == body.len();
        if is_tail {
            if let Statement::Expr { expr, range } = stmt {
                if !matches!(expr, Expr::Assignment(_) | Expr::SetProperty(_) | Expr::SetIndex(_)) {
                    let expected = ctx
                        .expected_return
                        .as_ref()
                        .map(crate::checker::expected::ExpectedType::from_knowledge)
                        .unwrap_or_default();
                    let typed = crate::checker::expression::analyze_expression(&mut ctx, expr, &expected);
                    if let Some(expected_return) = ctx.expected_return.clone() {
                        crate::checker::policy::enforce_assignability(
                            ctx.store,
                            &ctx.hierarchy,
                            &typed.knowledge,
                            &expected_return,
                            &ctx.current_module,
                            crate::diagnostic::DiagnosticCode::ReturnMismatch,
                            "tail expression result is not assignable to method's declared return type",
                            *range,
                            &mut ctx.diagnostics,
                        );
                    }
                    continue;
                }
            }
        }

        check_statement(&mut ctx, stmt);
    }

    ctx.finalize(callable, body_range, status)
}
