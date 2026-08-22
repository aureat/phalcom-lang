//! Message send and callable argument verification.

use super::context::CheckingContext;
use super::expression::synthesize_expr;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::types::evidence::TypeKnowledge;
use crate::types::relation::{check_assignability, Assignability};
use phalcom_ast::ast::Expr;
use phalcom_common::range::SourceRange;

/// Checks arguments passed to a callable against expected parameter type knowledge.
pub fn check_arguments(
    ctx: &mut CheckingContext<'_>,
    args: &[Expr],
    param_types: &[TypeKnowledge],
    call_range: SourceRange,
) {
    for (i, arg) in args.iter().enumerate() {
        let arg_k = synthesize_expr(ctx, arg);
        if let Some(param_k) = param_types.get(i) {
            let assignability = check_assignability(ctx.store, ctx.hierarchy, &arg_k, param_k);
            if let Assignability::Refuted { .. } = assignability {
                ctx.diagnostics.push(SemanticDiagnostic::error(
                    DiagnosticCode::ArgumentMismatch,
                    format!("argument at position {} does not match expected parameter type", i + 1),
                    call_range,
                ));
            }
        }
    }
}
