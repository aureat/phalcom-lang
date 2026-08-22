//! Statement type checking engine.

use super::context::CheckingContext;
use super::expression::synthesize_expr;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::types::annotation::resolve_type_annotation;
use crate::types::evidence::{EvidenceAuthority, TypeKnowledge, UnknownReason};
use crate::types::relation::{check_assignability, Assignability};
use phalcom_ast::ast::{Pattern, Statement};

/// Checks a single statement, updating context bindings and recording diagnostics.
pub fn check_statement(ctx: &mut CheckingContext<'_>, statement: &Statement) {
    match statement {
        Statement::Let(binding) => {
            let declared_k = binding.annotation.as_ref().map(|ann| {
                let mut diags = Vec::new();
                let k = resolve_type_annotation(ctx.store, ctx.resolver, &ctx.current_module, ann, &mut diags);
                ctx.diagnostics.extend(diags);
                k
            });

            let val_k = if let Some(expr) = &binding.value {
                synthesize_expr(ctx, expr)
            } else {
                TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
            };

            if let Some(ref decl_k) = declared_k {
                if binding.value.is_some() {
                    let assignability = check_assignability(ctx.store, ctx.hierarchy, &val_k, decl_k);
                    if let Assignability::Refuted { .. } = assignability {
                        let mut diag = SemanticDiagnostic::error(
                            DiagnosticCode::BindingInitializerMismatch,
                            "initializer expression is not assignable to declared type",
                            binding.range,
                        );
                        if let Some(ann) = &binding.annotation {
                            diag = diag.with_label(ann.range, "declared type");
                        }
                        if let Some(val) = &binding.value {
                            diag = diag.with_label(val.range(), "inferred type");
                        }
                        ctx.diagnostics.push(diag);
                    }
                }
            }

            let effective_k = declared_k.unwrap_or(val_k);

            match &binding.pattern {
                Pattern::Name { name, .. } => {
                    ctx.bind_local(name.clone(), effective_k);
                }
                _ => {}
            }
        }
        Statement::Return(ret) => {
            let val_k = if let Some(expr) = &ret.value {
                synthesize_expr(ctx, expr)
            } else {
                TypeKnowledge::known(ctx.store.unit(), EvidenceAuthority::ExactSyntax)
            };

            if let Some(expected) = ctx.expected_return.clone() {
                let assignability = check_assignability(ctx.store, ctx.hierarchy, &val_k, &expected);
                if let Assignability::Refuted { .. } = assignability {
                    ctx.diagnostics.push(SemanticDiagnostic::error(
                        DiagnosticCode::ReturnMismatch,
                        "returned value is not assignable to method's declared return type",
                        ret.range,
                    ));
                }
            }
        }
        Statement::Expr { expr, .. } => {
            synthesize_expr(ctx, expr);
        }
        Statement::Throw { expr, .. } => {
            synthesize_expr(ctx, expr);
        }
        Statement::Class(class_def) => {
            super::declaration::check_class(ctx, class_def);
        }
        Statement::For(for_stmt) => {
            for lane in &for_stmt.lanes {
                synthesize_expr(ctx, &lane.iter);
            }
            ctx.push_scope();
            for s in &for_stmt.body {
                check_statement(ctx, s);
            }
            ctx.pop_scope();
        }
        _ => {}
    }
}
