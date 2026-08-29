//! Declaration checking: classes, methods, getters, setters, indexers, and fields.

use super::context::{CallableReturnContract, CheckingContext};
use super::expression::synthesize_expr;
use super::statement::check_statement;
use crate::TypeResolver;
use crate::diagnostic::DiagnosticCode;
use crate::identity::DeclarationId;
use crate::surface::{DeclarationSurface, MemberVisibility};
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use phalcom_ast::ast::{ClassDef, ClassMember, ParameterDef, Statement};

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

    for member in &class_def.members {
        let visibility = member_visibility(member);
        match member {
            ClassMember::Field(_) => {
                let Some(signature) = super::declaration_signature::semantic_field_signature_for_member(ctx, &decl_id, member) else {
                    continue;
                };
                surface.add_field_with_visibility(
                    signature.side,
                    signature.name.as_ref(),
                    super::declaration_signature::project_field_signature(&signature),
                    visibility,
                );
            }
            ClassMember::Method(_) | ClassMember::Getter(_) | ClassMember::Setter(_) | ClassMember::Index(_) => {
                let Some(signature) = super::declaration_signature::semantic_signature_for_member(ctx, &decl_id, member) else {
                    continue;
                };
                let side = signature.side;
                let projection = super::declaration_signature::project_semantic_signature(&signature);
                surface.add_callable_with_visibility(side, projection, visibility);
            }
            ClassMember::Variant(_) => {}
        }
    }

    ctx.register_surface(decl_id, surface);
}

fn member_visibility(member: &ClassMember) -> MemberVisibility {
    let (name, attributes, is_field) = match member {
        ClassMember::Method(item) => (Some(item.name.as_str()), item.attributes.as_slice(), false),
        ClassMember::Getter(item) => (Some(item.name.as_str()), item.attributes.as_slice(), false),
        ClassMember::Setter(item) => (Some(item.name.as_str()), item.attributes.as_slice(), false),
        ClassMember::Field(item) => (Some(item.name.as_str()), item.attributes.as_slice(), true),
        ClassMember::Variant(item) => (Some(item.name.as_str()), item.attributes.as_slice(), false),
        ClassMember::Index(item) => (None, item.attributes.as_slice(), false),
    };
    if name.is_some_and(|name| name.starts_with("_$")) {
        MemberVisibility::Internal
    } else if is_field || attributes.iter().any(|attribute| attribute.name == "private") {
        MemberVisibility::Private
    } else if attributes.iter().any(|attribute| attribute.name == "protected") {
        MemberVisibility::Protected
    } else {
        MemberVisibility::Public
    }
}

fn check_field_initializer_against_declared(ctx: &mut CheckingContext<'_>, field: &phalcom_ast::ast::FieldDef, declared: &TypeKnowledge) {
    let Some(default_expr) = &field.default else {
        return;
    };
    let initializer = synthesize_expr(ctx, default_expr);
    ctx.apply_assignability(
        &initializer,
        declared,
        DiagnosticCode::FieldMismatch,
        format!("default value for field `{}` does not match declared type", field.name),
        field.range,
    );
}

fn check_field_initializer(ctx: &mut CheckingContext<'_>, resolver: &dyn crate::types::annotation::TypeResolver, field: &phalcom_ast::ast::FieldDef) {
    let declared_k = field.annotation.as_ref().map(|annotation| ctx.resolve_type_annotation(resolver, annotation).0);

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
        let (param_k, param_invalidity) = if let Some(ann) = &param.annotation {
            ctx.resolve_type_annotation(resolver, ann)
        } else {
            (
                TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence),
                crate::checker::causal::CausalInvalidity::Clean,
            )
        };
        ctx.bind_callable_parameter_with_causal(param.name.clone(), param_k, param.range, param_invalidity);
    }

    // Resolve return annotation
    let expected_return = return_annotation.and_then(|ann| {
        let (knowledge, _) = ctx.resolve_type_annotation(resolver, ann);
        knowledge.ty().map(|ty| CallableReturnContract {
            ty,
            basis: crate::declaration_type::DeclaredTypeBasis::SourceAnnotation,
            origin: EvidenceOrigin::DeveloperAnnotation,
            is_dynamic: false,
            source: Some(ann.range),
        })
    });

    let old_return = ctx.expected_return.take();
    ctx.expected_return = expected_return.clone();

    for (i, stmt) in body.iter().enumerate() {
        if i == body.len() - 1 {
            if let Statement::Expr { expr, range } = stmt {
                let expected_ret_type = expected_return
                    .as_ref()
                    .map(|contract| {
                        crate::checker::expected::ExpectedType::proper_from(contract.ty, crate::checker::expected::ExpectationOrigin::ReturnContract)
                    })
                    .unwrap_or_default();
                let tail_typed = crate::checker::expression::analyze_expression(ctx, expr, &expected_ret_type);
                if let Some(expected) = &expected_return {
                    ctx.apply_knowledge_against_type(
                        &tail_typed.knowledge,
                        expected.ty,
                        DiagnosticCode::ReturnMismatch,
                        "tail expression result is not assignable to method's declared return type",
                        *range,
                    );
                }
            } else if let Statement::Let(binding) = stmt {
                let unit = TypeKnowledge::established(ctx.store.unit(), EvidenceOrigin::Flow);
                if let Some(expected) = &expected_return {
                    ctx.apply_knowledge_against_type(
                        &unit,
                        expected.ty,
                        DiagnosticCode::ReturnMismatch,
                        "tail let/const completes with Unit, which is not assignable to method's declared return type",
                        binding.range,
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
