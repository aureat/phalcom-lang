//! Declaration checking: classes, methods, getters, setters, indexers, and fields.

use super::context::CheckingContext;
use super::expression::synthesize_expr;
use super::statement::check_statement;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::identity::DeclarationId;
use crate::types::annotation::resolve_type_annotation;
use crate::types::relation::{check_assignability, Assignability};
use phalcom_ast::ast::{ClassDef, ClassMember, ParameterDef, Statement};

/// Checks a class declaration and all its members.
pub fn check_class(ctx: &mut CheckingContext<'_>, class_def: &ClassDef) {
    let decl_id = DeclarationId::new(ctx.current_module.clone(), class_def.name.clone().into());
    let old_class = ctx.current_class.replace(decl_id);

    for member in &class_def.members {
        match member {
            ClassMember::Field(f) => {
                let declared_k = f.annotation.as_ref().map(|ann| {
                    let mut diags = Vec::new();
                    let k = resolve_type_annotation(ctx.store, ctx.resolver, &ctx.current_module, ann, &mut diags);
                    ctx.diagnostics.extend(diags);
                    k
                });

                if let (Some(decl_k), Some(default_expr)) = (declared_k, &f.default) {
                    let init_k = synthesize_expr(ctx, default_expr);
                    let assignability = check_assignability(ctx.store, ctx.hierarchy, &init_k, &decl_k);
                    if let Assignability::Refuted { .. } = assignability {
                        ctx.diagnostics.push(SemanticDiagnostic::error(
                            DiagnosticCode::FieldMismatch,
                            format!("default value for field `{}` does not match declared type", f.name),
                            f.range,
                        ));
                    }
                }
            }
            ClassMember::Method(m) => {
                check_callable_body(
                    ctx,
                    &m.params,
                    m.return_annotation.as_ref(),
                    &m.body,
                );
            }
            ClassMember::Getter(g) => {
                check_callable_body(
                    ctx,
                    &[],
                    g.return_annotation.as_ref(),
                    &g.body,
                );
            }
            ClassMember::Setter(s) => {
                check_callable_body(
                    ctx,
                    std::slice::from_ref(&s.param),
                    s.return_annotation.as_ref(),
                    &s.body,
                );
            }
            ClassMember::Index(i) => {
                let mut params = i.params.clone();
                if let phalcom_ast::ast::IndexAccessor::Set { put } = &i.accessor {
                    params.push(put.clone());
                }
                check_callable_body(
                    ctx,
                    &params,
                    i.return_annotation.as_ref(),
                    &i.body,
                );
            }
            _ => {}
        }
    }

    ctx.current_class = old_class;
}

fn check_callable_body(
    ctx: &mut CheckingContext<'_>,
    params: &[ParameterDef],
    return_annotation: Option<&phalcom_ast::ast::TypeAnnotation>,
    body: &[Statement],
) {
    ctx.push_scope();

    // Bind parameters
    for param in params {
        let param_k = if let Some(ann) = &param.annotation {
            let mut diags = Vec::new();
            let k = resolve_type_annotation(ctx.store, ctx.resolver, &ctx.current_module, ann, &mut diags);
            ctx.diagnostics.extend(diags);
            k
        } else {
            crate::types::evidence::TypeKnowledge::Unknown(
                crate::types::evidence::UnknownReason::UnannotatedDeclaration,
            )
        };
        ctx.bind_local(param.name.clone(), param_k);
    }

    // Resolve return annotation
    let expected_return = return_annotation.map(|ann| {
        let mut diags = Vec::new();
        let k = resolve_type_annotation(ctx.store, ctx.resolver, &ctx.current_module, ann, &mut diags);
        ctx.diagnostics.extend(diags);
        k
    });

    let old_return = ctx.expected_return.take();
    ctx.expected_return = expected_return.clone();

    for (i, stmt) in body.iter().enumerate() {
        check_statement(ctx, stmt);

        // Tail expression checking: if this is the last statement and it is Statement::Expr
        if i == body.len() - 1 {
            if let Statement::Expr { expr, range } = stmt {
                if let Some(expected) = &expected_return {
                    let tail_k = synthesize_expr(ctx, expr);
                    let assignability = check_assignability(ctx.store, ctx.hierarchy, &tail_k, expected);
                    if let Assignability::Refuted { .. } = assignability {
                        ctx.diagnostics.push(SemanticDiagnostic::error(
                            DiagnosticCode::ReturnMismatch,
                            "tail expression result is not assignable to method's declared return type",
                            *range,
                        ));
                    }
                }
            }
        }
    }

    ctx.expected_return = old_return;
    ctx.pop_scope();
}
