//! Declaration checking: classes, methods, getters, setters, indexers, and fields.

use super::context::CheckingContext;
use super::expression::synthesize_expr;
use super::statement::check_statement;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::{CallableParameter, CallableSignature};
use crate::identity::DeclarationId;
use crate::surface::DeclarationSurface;
use crate::types::annotation::resolve_type_annotation;
use crate::types::denotation::ValueSemanticFact;
use crate::types::evidence::{EvidenceAuthority, TypeKnowledge, UnknownReason};
use crate::types::relation::{Assignability, check_assignability};
use phalcom_ast::ast::{ClassDef, ClassMember, ParameterDef, Statement};
use phalcom_common::selector::{Selector, SelectorSlot};

fn member_side(member: &ClassMember) -> crate::identity::DispatchSide {
    if member.is_static() || member.attributes().iter().any(|attribute| attribute.name == "class") {
        crate::identity::DispatchSide::Class
    } else {
        crate::identity::DispatchSide::Instance
    }
}

/// Pre-registers a class surface and its callable signatures in the context's dispatch table.
pub fn register_class_surface(ctx: &mut CheckingContext<'_>, class_def: &ClassDef) {
    let decl_id = DeclarationId::new(ctx.current_module.clone(), class_def.name.clone().into());
    let mut surface = DeclarationSurface::new(Some(decl_id.clone()));
    let class_ty = ctx.store.nominal(decl_id.clone());
    ctx.dispatch.register_type(class_ty, decl_id.clone());

    for member in &class_def.members {
        let side = member_side(member);
        match member {
            ClassMember::Field(f) => {
                let declared_k = f
                    .annotation
                    .as_ref()
                    .map(|ann| {
                        let mut diags = Vec::new();
                        let k = resolve_type_annotation(ctx.store, ctx.declarations, ctx.resolver, &ctx.current_module, ann, &mut diags);
                        ctx.diagnostics.extend(diags);
                        k
                    })
                    .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));
                surface.add_field(side, &f.name, declared_k);
            }
            ClassMember::Method(m) => {
                let mut slots = Vec::new();
                let mut params = Vec::new();
                for p in &m.params {
                    let p_k = p
                        .annotation
                        .as_ref()
                        .map(|ann| {
                            let mut diags = Vec::new();
                            let k = resolve_type_annotation(ctx.store, ctx.declarations, ctx.resolver, &ctx.current_module, ann, &mut diags);
                            ctx.diagnostics.extend(diags);
                            k
                        })
                        .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));

                    let mut param = CallableParameter::new(p.name.clone(), p_k).with_rest(p.is_rest());
                    if let Some(ref l) = p.label {
                        slots.push(SelectorSlot::Label(l.clone()));
                        param = param.with_label(l.clone());
                    } else {
                        slots.push(SelectorSlot::Positional);
                    }
                    params.push(param);
                }

                let ret_k = m
                    .return_annotation
                    .as_ref()
                    .map(|ann| {
                        let mut diags = Vec::new();
                        let k = resolve_type_annotation(ctx.store, ctx.declarations, ctx.resolver, &ctx.current_module, ann, &mut diags);
                        ctx.diagnostics.extend(diags);
                        k
                    })
                    .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));

                if let Ok(sel) = Selector::method(&m.name, slots) {
                    surface.add_callable(side, CallableSignature::new(sel, params, ret_k));
                }
            }
            ClassMember::Getter(g) => {
                let ret_k = g
                    .return_annotation
                    .as_ref()
                    .map(|ann| {
                        let mut diags = Vec::new();
                        let k = resolve_type_annotation(ctx.store, ctx.declarations, ctx.resolver, &ctx.current_module, ann, &mut diags);
                        ctx.diagnostics.extend(diags);
                        k
                    })
                    .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));

                if let Ok(sel) = Selector::getter(&g.name) {
                    surface.add_callable(side, CallableSignature::new(sel, Vec::new(), ret_k));
                }
            }
            ClassMember::Setter(s) => {
                let param_k = s
                    .param
                    .annotation
                    .as_ref()
                    .map(|ann| {
                        let mut diags = Vec::new();
                        let k = resolve_type_annotation(ctx.store, ctx.declarations, ctx.resolver, &ctx.current_module, ann, &mut diags);
                        ctx.diagnostics.extend(diags);
                        k
                    })
                    .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));

                if let Ok(sel) = Selector::setter(&s.name) {
                    let param = CallableParameter::new(s.param.name.clone(), param_k);
                    let ret_k = TypeKnowledge::known(ctx.store.unit(), EvidenceAuthority::Declared);
                    surface.add_callable(side, CallableSignature::new(sel, vec![param], ret_k));
                }
            }
            ClassMember::Index(i) => {
                let mut slots = Vec::new();
                let mut params = Vec::new();
                for p in &i.params {
                    let p_k = p
                        .annotation
                        .as_ref()
                        .map(|ann| {
                            let mut diags = Vec::new();
                            let k = resolve_type_annotation(ctx.store, ctx.declarations, ctx.resolver, &ctx.current_module, ann, &mut diags);
                            ctx.diagnostics.extend(diags);
                            k
                        })
                        .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));

                    let mut param = CallableParameter::new(p.name.clone(), p_k).with_rest(p.is_rest());
                    if let Some(ref l) = p.label {
                        slots.push(SelectorSlot::Label(l.clone()));
                        param = param.with_label(l.clone());
                    } else {
                        slots.push(SelectorSlot::Positional);
                    }
                    params.push(param);
                }

                match &i.accessor {
                    phalcom_ast::ast::IndexAccessor::Get => {
                        let ret_k = i
                            .return_annotation
                            .as_ref()
                            .map(|ann| {
                                let mut diags = Vec::new();
                                let k = resolve_type_annotation(ctx.store, ctx.declarations, ctx.resolver, &ctx.current_module, ann, &mut diags);
                                ctx.diagnostics.extend(diags);
                                k
                            })
                            .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));

                        if let Ok(sel) = Selector::subscript_get(slots) {
                            surface.add_callable(side, CallableSignature::new(sel, params, ret_k));
                        }
                    }
                    phalcom_ast::ast::IndexAccessor::Set { put } => {
                        let put_k = put
                            .annotation
                            .as_ref()
                            .map(|ann| {
                                let mut diags = Vec::new();
                                let k = resolve_type_annotation(ctx.store, ctx.declarations, ctx.resolver, &ctx.current_module, ann, &mut diags);
                                ctx.diagnostics.extend(diags);
                                k
                            })
                            .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));

                        params.push(CallableParameter::new(put.name.clone(), put_k.clone()));
                        if let Ok(sel) = Selector::subscript_set(slots) {
                            surface.add_callable(side, CallableSignature::new(sel, params, put_k));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    ctx.register_surface(decl_id, surface);
}

/// Checks a class declaration and all its members.
pub fn check_class(ctx: &mut CheckingContext<'_>, class_def: &ClassDef) {
    register_class_surface(ctx, class_def);

    let decl_id = DeclarationId::new(ctx.current_module.clone(), class_def.name.clone().into());
    let old_class = ctx.current_class.replace(decl_id);

    for member in &class_def.members {
        let side = member_side(member);
        let old_side = ctx.current_side;
        ctx.current_side = side;
        match member {
            ClassMember::Field(f) => {
                let declared_k = f.annotation.as_ref().map(|ann| {
                    let mut diags = Vec::new();
                    let k = resolve_type_annotation(ctx.store, ctx.declarations, ctx.resolver, &ctx.current_module, ann, &mut diags);
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
                check_callable_body(ctx, &m.params, m.return_annotation.as_ref(), &m.body);
            }
            ClassMember::Getter(g) => {
                check_callable_body(ctx, &[], g.return_annotation.as_ref(), &g.body);
            }
            ClassMember::Setter(s) => {
                check_callable_body(ctx, std::slice::from_ref(&s.param), s.return_annotation.as_ref(), &s.body);
            }
            ClassMember::Index(i) => {
                let mut params = i.params.clone();
                if let phalcom_ast::ast::IndexAccessor::Set { put } = &i.accessor {
                    params.push(put.clone());
                }
                check_callable_body(ctx, &params, i.return_annotation.as_ref(), &i.body);
            }
            _ => {}
        }
        ctx.current_side = old_side;
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
            let k = resolve_type_annotation(ctx.store, ctx.declarations, ctx.resolver, &ctx.current_module, ann, &mut diags);
            ctx.diagnostics.extend(diags);
            k
        } else {
            TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
        };
        ctx.bind_local(param.name.clone(), ValueSemanticFact::new(param_k));
    }

    // Resolve return annotation
    let expected_return = return_annotation.map(|ann| {
        let mut diags = Vec::new();
        let k = resolve_type_annotation(ctx.store, ctx.declarations, ctx.resolver, &ctx.current_module, ann, &mut diags);
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
