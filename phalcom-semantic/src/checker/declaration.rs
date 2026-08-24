//! Declaration checking: classes, methods, getters, setters, indexers, and fields.

use super::context::CheckingContext;
use super::expression::synthesize_expr;
use super::statement::check_statement;
use crate::TypeResolver;
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

pub(crate) fn member_side(member: &ClassMember) -> crate::identity::DispatchSide {
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
    let class_ty = ctx.nominal_type_of(&decl_id);
    ctx.dispatch.make_mut().register_type(class_ty, decl_id.clone());

    let type_params_map = if let Some(sig) = ctx.declaration_generic_signature(&decl_id) {
        let mut map = std::collections::HashMap::new();
        for &param_id in sig.parameters.iter() {
            let name = ctx.store.type_parameter(param_id).name.to_string();
            let param_form = ctx.store.parameter_form(param_id);
            map.insert(name, param_form);
        }
        map
    } else {
        std::collections::HashMap::new()
    };
    let scoped_resolver = crate::types::annotation::ScopedTypeResolver {
        parent: &ctx.resolver,
        type_parameters: type_params_map,
    };
    let resolver = &scoped_resolver;

    for member in &class_def.members {
        let side = member_side(member);
        match member {
            ClassMember::Field(f) => {
                let declared_k = f
                    .annotation
                    .as_ref()
                    .map(|ann| {
                        let mut diags = Vec::new();
                        let k = resolve_type_annotation(ctx.store, ctx.declarations, resolver, &ctx.current_module, ann, &mut diags);
                        ctx.diagnostics.extend(diags);
                        k
                    })
                    .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));
                surface.add_field(side, &f.name, declared_k);
            }
            ClassMember::Method(m) => {
                let is_constructor = m.is_constructor || m.attributes.iter().any(|a| a.name == "constructor");
                let mut slots = Vec::new();
                let mut params = Vec::new();
                for p in &m.params {
                    let p_k = p
                        .annotation
                        .as_ref()
                        .map(|ann| {
                            let mut diags = Vec::new();
                            let k = resolve_type_annotation(ctx.store, ctx.declarations, resolver, &ctx.current_module, ann, &mut diags);
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

                let (effective_side, ret_k) = if is_constructor {
                    let self_type = ctx.store.self_type(crate::types::parameter::SelfTypeTerm {
                        owner: decl_id.clone(),
                        side: crate::identity::DispatchSide::Class,
                        role: crate::types::parameter::SelfRole::InstanceType,
                    });
                    (
                        crate::identity::DispatchSide::Class,
                        TypeKnowledge::known(self_type, EvidenceAuthority::Declared),
                    )
                } else {
                    let ret_k = m
                        .return_annotation
                        .as_ref()
                        .map(|ann| {
                            let mut diags = Vec::new();
                            let k = resolve_type_annotation(ctx.store, ctx.declarations, resolver, &ctx.current_module, ann, &mut diags);
                            ctx.diagnostics.extend(diags);
                            k
                        })
                        .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration));
                    (side, ret_k)
                };

                if let Ok(sel) = Selector::method(&m.name, slots) {
                    let mut callable_sig = CallableSignature::new(sel.clone(), params, ret_k);
                    if !m.generic_parameters.is_empty() {
                        let mut diags = Vec::new();
                        let callable_id = crate::identity::CallableId::new(decl_id.clone(), sel, effective_side);
                        let sig = crate::types::annotation::resolve_generic_signature(
                            ctx.store,
                            ctx.declarations,
                            resolver,
                            &ctx.current_module,
                            crate::types::parameter::TypeParameterOwner::Callable(callable_id),
                            &m.generic_parameters,
                            m.where_clause.as_ref(),
                            &mut diags,
                        );
                        ctx.diagnostics.extend(diags);
                        callable_sig = callable_sig.with_generics(sig);
                    }
                    surface.add_callable(effective_side, callable_sig);
                }
            }
            ClassMember::Getter(g) => {
                let ret_k = g
                    .return_annotation
                    .as_ref()
                    .map(|ann| {
                        let mut diags = Vec::new();
                        let k = resolve_type_annotation(ctx.store, ctx.declarations, resolver, &ctx.current_module, ann, &mut diags);
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
                        let k = resolve_type_annotation(ctx.store, ctx.declarations, resolver, &ctx.current_module, ann, &mut diags);
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
                            let k = resolve_type_annotation(ctx.store, ctx.declarations, resolver, &ctx.current_module, ann, &mut diags);
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
                                let k = resolve_type_annotation(ctx.store, ctx.declarations, resolver, &ctx.current_module, ann, &mut diags);
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
                                let k = resolve_type_annotation(ctx.store, ctx.declarations, resolver, &ctx.current_module, ann, &mut diags);
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

fn check_field_initializer_against_declared(ctx: &mut CheckingContext<'_>, field: &phalcom_ast::ast::FieldDef, declared: &TypeKnowledge) {
    let Some(default_expr) = &field.default else {
        return;
    };
    let initializer = synthesize_expr(ctx, default_expr);
    let assignability = check_assignability(ctx.store, &ctx.hierarchy, &initializer, declared);
    if let Assignability::Refuted { .. } = assignability {
        ctx.diagnostics.push(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::FieldMismatch,
            format!("default value for field `{}` does not match declared type", field.name),
            field.range,
        ));
    }
}

fn check_field_initializer(ctx: &mut CheckingContext<'_>, resolver: &dyn crate::types::annotation::TypeResolver, field: &phalcom_ast::ast::FieldDef) {
    let declared_k = field.annotation.as_ref().map(|annotation| {
        let mut diagnostics = Vec::new();
        let knowledge = resolve_type_annotation(ctx.store, ctx.declarations, resolver, &ctx.current_module, annotation, &mut diagnostics);
        ctx.diagnostics.extend(diagnostics);
        knowledge
    });

    if let Some(declared) = declared_k {
        check_field_initializer_against_declared(ctx, field, &declared);
    }
}

/// Checks only field initializer expressions for an already-registered class.
///
/// Workspace-session callable bodies are analyzed by `CallableBody` DB queries.
/// This narrower pass keeps non-callable field initialization diagnostics without
/// re-running every method/getter/setter/index body after query evaluation.
pub fn check_class_field_initializers(ctx: &mut CheckingContext<'_>, class_def: &ClassDef) {
    let decl_id = DeclarationId::new(ctx.current_module.clone(), class_def.name.clone().into());
    let old_class = ctx.current_class.replace(decl_id.clone());

    for member in &class_def.members {
        let ClassMember::Field(field) = member else {
            continue;
        };
        let side = member_side(member);
        let old_side = ctx.current_side;
        ctx.current_side = side;
        if let Some(declared) = ctx.get_field(&decl_id, side, &field.name) {
            check_field_initializer_against_declared(ctx, field, &declared);
        }
        ctx.current_side = old_side;
    }

    ctx.current_class = old_class;
}

/// Checks the member bodies of an already-registered class declaration.
pub fn check_class_bodies(ctx: &mut CheckingContext<'_>, class_def: &ClassDef) {
    let decl_id = DeclarationId::new(ctx.current_module.clone(), class_def.name.clone().into());
    let type_params_map = if let Some(sig) = ctx.declaration_generic_signature(&decl_id) {
        let mut map = std::collections::HashMap::new();
        for &param_id in sig.parameters.iter() {
            let name = ctx.store.type_parameter(param_id).name.to_string();
            let param_form = ctx.store.parameter_form(param_id);
            map.insert(name, param_form);
        }
        map
    } else {
        std::collections::HashMap::new()
    };
    // Keep resolver ownership independent from `ctx` while checking bodies. The
    // body checker mutably borrows the full context, so a scoped resolver that
    // directly borrows `ctx.resolver` would create an overlapping borrow.
    let parent_resolver = ctx.resolver.clone();
    let scoped_resolver = crate::types::annotation::ScopedTypeResolver {
        parent: &parent_resolver,
        type_parameters: type_params_map,
    };
    let resolver = &scoped_resolver;
    let old_class = ctx.current_class.replace(decl_id);

    for member in &class_def.members {
        let side = member_side(member);
        let old_side = ctx.current_side;
        ctx.current_side = side;
        match member {
            ClassMember::Field(f) => {
                check_field_initializer(ctx, resolver, f);
            }
            ClassMember::Method(m) => {
                check_callable_body(ctx, resolver, &m.params, m.return_annotation.as_ref(), m.body.statements().unwrap_or(&[]));
            }
            ClassMember::Getter(g) => {
                check_callable_body(ctx, resolver, &[], g.return_annotation.as_ref(), g.body.statements().unwrap_or(&[]));
            }
            ClassMember::Setter(s) => {
                check_callable_body(
                    ctx,
                    resolver,
                    std::slice::from_ref(&s.param),
                    s.return_annotation.as_ref(),
                    s.body.statements().unwrap_or(&[]),
                );
            }
            ClassMember::Index(i) => {
                let mut params = i.params.clone();
                if let phalcom_ast::ast::IndexAccessor::Set { put } = &i.accessor {
                    params.push((**put).clone());
                }
                check_callable_body(ctx, resolver, &params, i.return_annotation.as_ref(), &i.body);
            }
            _ => {}
        }
        ctx.current_side = old_side;
    }

    ctx.current_class = old_class;
}

/// Checks a class declaration and all its members.
pub fn check_class(ctx: &mut CheckingContext<'_>, class_def: &ClassDef) {
    register_class_surface(ctx, class_def);
    check_class_bodies(ctx, class_def);
}

fn check_callable_body(
    ctx: &mut CheckingContext<'_>,
    resolver: &dyn TypeResolver,
    params: &[ParameterDef],
    return_annotation: Option<&phalcom_ast::ast::TypeAnnotation>,
    body: &[Statement],
) {
    ctx.push_scope();

    // Bind parameters
    for param in params {
        let param_k = if let Some(ann) = &param.annotation {
            let mut diags = Vec::new();
            let k = resolve_type_annotation(ctx.store, ctx.declarations, resolver, &ctx.current_module, ann, &mut diags);
            ctx.diagnostics.extend(diags);
            k
        } else {
            TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
        };
        ctx.bind_local(param.name.clone(), ValueSemanticFact::new(param_k), param.range);
    }

    // Resolve return annotation
    let expected_return = return_annotation.map(|ann| {
        let mut diags = Vec::new();
        let k = resolve_type_annotation(ctx.store, ctx.declarations, resolver, &ctx.current_module, ann, &mut diags);
        ctx.diagnostics.extend(diags);
        k
    });

    let old_return = ctx.expected_return.take();
    ctx.expected_return = expected_return.clone();

    for (i, stmt) in body.iter().enumerate() {
        if i == body.len() - 1 {
            if let Statement::Expr { expr, range } = stmt {
                let expected_ret_type = expected_return
                    .as_ref()
                    .map(crate::checker::expected::ExpectedType::from_knowledge)
                    .unwrap_or_default();
                let tail_typed = crate::checker::expression::analyze_expression(ctx, expr, &expected_ret_type);
                if let Some(expected) = &expected_return {
                    crate::checker::policy::enforce_assignability(
                        ctx.store,
                        &ctx.hierarchy,
                        &tail_typed.knowledge,
                        expected,
                        &ctx.current_module,
                        DiagnosticCode::ReturnMismatch,
                        "tail expression result is not assignable to method's declared return type",
                        *range,
                        &mut ctx.diagnostics,
                    );
                }
            } else if let Statement::Let(binding) = stmt {
                let unit = TypeKnowledge::known(ctx.store.unit(), EvidenceAuthority::Proven);
                if let Some(expected) = &expected_return {
                    crate::checker::policy::enforce_assignability(
                        ctx.store,
                        &ctx.hierarchy,
                        &unit,
                        expected,
                        &ctx.current_module,
                        DiagnosticCode::ReturnMismatch,
                        "tail let/const completes with Unit, which is not assignable to method's declared return type",
                        binding.range,
                        &mut ctx.diagnostics,
                    );
                }
                check_statement(ctx, stmt);
            } else {
                check_statement(ctx, stmt);
            }
        } else {
            check_statement(ctx, stmt);
        }
    }

    ctx.expected_return = old_return;
    ctx.pop_scope();
}
