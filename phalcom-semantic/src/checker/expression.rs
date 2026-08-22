//! Expression type synthesis engine.

use super::context::CheckingContext;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::types::evidence::{EvidenceAuthority, TypeKnowledge, UnknownReason};
use crate::types::relation::check_assignability;
use phalcom_ast::ast::Expr;

/// Synthesizes epistemic type knowledge for an expression.
pub fn synthesize_expr(ctx: &mut CheckingContext<'_>, expr: &Expr) -> TypeKnowledge {
    match expr {
        Expr::Int { range, .. } => {
            if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "Int", &[]) {
                let ty = ctx.store.nominal(decl);
                TypeKnowledge::known(ty, EvidenceAuthority::ExactSyntax).with_range(*range)
            } else {
                TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Float { range, .. } => {
            if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "Float", &[]) {
                let ty = ctx.store.nominal(decl);
                TypeKnowledge::known(ty, EvidenceAuthority::ExactSyntax).with_range(*range)
            } else {
                TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::String { range, .. } => {
            if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "String", &[]) {
                let ty = ctx.store.nominal(decl);
                TypeKnowledge::known(ty, EvidenceAuthority::ExactSyntax).with_range(*range)
            } else {
                TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Boolean { range, .. } => {
            if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "Bool", &[]) {
                let ty = ctx.store.nominal(decl);
                TypeKnowledge::known(ty, EvidenceAuthority::ExactSyntax).with_range(*range)
            } else {
                TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Var { value, range } => {
            if let Some(k) = ctx.lookup_local(value) {
                k.clone().with_range(*range)
            } else if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, value, &[]) {
                let ty = ctx.store.nominal(decl);
                TypeKnowledge::known(ty, EvidenceAuthority::Declared).with_range(*range)
            } else {
                TypeKnowledge::Unknown(UnknownReason::UnresolvedName(value.as_str().into()))
            }
        }
        Expr::SelfVar { range } => {
            if let Some(class_decl) = ctx.current_class.clone() {
                let ty = ctx.store.nominal(class_decl);
                TypeKnowledge::known(ty, EvidenceAuthority::Proven).with_range(*range)
            } else {
                TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
            }
        }
        Expr::Assignment(assign) => {
            let val_k = synthesize_expr(ctx, &assign.value);
            if let Expr::Var { value: var_name, .. } = &*assign.name {
                if let Some(target_k) = ctx.lookup_local(var_name).cloned() {
                    let assignability = check_assignability(ctx.store, ctx.hierarchy, &val_k, &target_k);
                    if let crate::types::relation::Assignability::Refuted { .. } = assignability {
                        ctx.diagnostics.push(SemanticDiagnostic::error(
                            DiagnosticCode::AssignmentMismatch,
                            format!("assigned value is not assignable to `{}`", var_name),
                            assign.range,
                        ));
                    }
                }
            }
            val_k
        }
        Expr::Block(block) => {
            ctx.push_scope();
            let last_k = TypeKnowledge::known(ctx.store.unit(), EvidenceAuthority::ExactSyntax);
            for stmt in &block.body {
                super::statement::check_statement(ctx, stmt);
            }
            ctx.pop_scope();
            last_k
        }
        Expr::Binary(b) => {
            let left_k = synthesize_expr(ctx, &b.left);
            let _right_k = synthesize_expr(ctx, &b.right);
            match b.op {
                phalcom_ast::ast::BinaryOp::Equal
                | phalcom_ast::ast::BinaryOp::Same
                | phalcom_ast::ast::BinaryOp::NotEqual
                | phalcom_ast::ast::BinaryOp::LessThan
                | phalcom_ast::ast::BinaryOp::LessThanOrEqual
                | phalcom_ast::ast::BinaryOp::GreaterThan
                | phalcom_ast::ast::BinaryOp::GreaterThanOrEqual
                | phalcom_ast::ast::BinaryOp::And
                | phalcom_ast::ast::BinaryOp::Or => {
                    if let Some(decl) = ctx.resolver.resolve_type_name(&ctx.current_module, "Bool", &[]) {
                        let ty = ctx.store.nominal(decl);
                        TypeKnowledge::known(ty, EvidenceAuthority::Proven).with_range(b.range)
                    } else {
                        TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
                    }
                }
                _ => {
                    // Arithmetic defaults to left type if known
                    if left_k.is_known() {
                        left_k
                    } else {
                        TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend)
                    }
                }
            }
        }
        _ => TypeKnowledge::Unknown(UnknownReason::UncheckedExpression),
    }
}
