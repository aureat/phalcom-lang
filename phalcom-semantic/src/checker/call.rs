//! Message send and callable argument verification.

use super::context::CheckingContext;
use super::expression::synthesize_expr;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::CallableSignature;
use crate::types::evidence::TypeKnowledge;
use crate::types::relation::{Assignability, check_assignability};
use phalcom_ast::ast::{Expr, PackItem, PackLabel};
use phalcom_common::range::SourceRange;

/// Checks arguments passed to a callable against expected parameter type knowledge.
pub fn check_arguments(ctx: &mut CheckingContext<'_>, args: &[Expr], param_types: &[TypeKnowledge], call_range: SourceRange) {
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

/// Matches call argument pack items against a callable signature, validating labels and types.
pub fn match_callable_arguments(ctx: &mut CheckingContext<'_>, args: &[PackItem], signature: &CallableSignature, call_range: SourceRange) -> TypeKnowledge {
    let mut positional_idx = 0;

    for arg in args {
        match arg {
            PackItem::Positional { expr, range } => {
                let arg_k = synthesize_expr(ctx, expr);
                while positional_idx < signature.parameters.len() {
                    let param = &signature.parameters[positional_idx];
                    positional_idx += 1;
                    if param.external_label.is_none() {
                        let assignability = check_assignability(ctx.store, ctx.hierarchy, &arg_k, &param.ty);
                        if let Assignability::Refuted { .. } = assignability {
                            ctx.diagnostics.push(SemanticDiagnostic::error(
                                DiagnosticCode::ArgumentMismatch,
                                format!("positional argument `{}` does not match expected parameter type", param.local_name),
                                *range,
                            ));
                        }
                        break;
                    }
                }
            }
            PackItem::Labeled { label, value, range } => {
                let arg_k = synthesize_expr(ctx, value);
                if let PackLabel::Static { text, .. } = label {
                    for param in &signature.parameters {
                        if let Some(ref ext_label) = param.external_label {
                            if ext_label == text {
                                let assignability = check_assignability(ctx.store, ctx.hierarchy, &arg_k, &param.ty);
                                if let Assignability::Refuted { .. } = assignability {
                                    ctx.diagnostics.push(SemanticDiagnostic::error(
                                        DiagnosticCode::ArgumentMismatch,
                                        format!("argument for label `{}:` does not match expected parameter type", text),
                                        *range,
                                    ));
                                }
                                break;
                            }
                        }
                    }
                }
            }
            PackItem::Expand { expr, .. } => {
                synthesize_expr(ctx, expr);
            }
        }
    }

    signature.return_type.clone().with_range(call_range)
}
