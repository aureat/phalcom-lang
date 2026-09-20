//! Declaration checking: classes, methods, getters, setters, indexers, and fields.

use super::context::{CallableReturnContract, CheckingContext};
use super::expression::synthesize_expr;
use super::statement::check_statement;
use crate::diagnostic::DiagnosticCode;
use crate::identity::DeclarationId;
use crate::signature::CallableSemanticSignature;
use crate::surface::{DeclarationSurface, MemberVisibility};
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::substitution::TypeSubstitution;
use phalcom_ast::ast::{ClassDef, ClassMember, ParameterDef, Statement};
use std::collections::HashMap;

/// Validates declaration-owned methods against the dynamically reachable
/// ancestor contract selected for the same selector.
pub(crate) fn validate_class_overrides(ctx: &mut CheckingContext<'_>, class_def: &ClassDef, signatures: &crate::signature::CallableSignatureTable) {
    let owner = DeclarationId::new(ctx.current_module.clone(), class_def.name.clone().into());
    let instance_receiver = ctx.instance_type_of(&owner);
    let class_receiver = ctx.store.class_object_type(owner.clone());

    for member in &class_def.members {
        let Some(callable) = super::declaration_signature::callable_id_for_member(&owner, member) else {
            continue;
        };
        let Some(candidate_semantic) = signatures.get(&callable) else {
            continue;
        };
        let receiver = match callable.side {
            crate::identity::DispatchSide::Instance => {
                let Some(receiver) = instance_receiver else { continue };
                receiver
            }
            crate::identity::DispatchSide::Class => class_receiver,
        };
        let inherited = ctx
            .dispatch_ref()
            .dispatch_owners(ctx.hierarchy.inner(), &owner, callable.side)
            .into_iter()
            .skip(1)
            .find_map(|ancestor| {
                let surface = ctx.dispatch_ref().surface(&ancestor.declaration)?;
                let signature = surface.get_callable(ancestor.side, &callable.selector)?.clone();
                Some((ancestor.declaration, signature))
            });
        let Some((ancestor, inherited)) = inherited else {
            continue;
        };
        let candidate = super::declaration_signature::project_semantic_signature(candidate_semantic);
        let Ok((mut candidate, _)) = ctx.specialize_dispatch_signature(receiver, &owner, candidate) else {
            continue;
        };
        let Ok((inherited, _)) = ctx.specialize_dispatch_signature(receiver, &ancestor, inherited) else {
            continue;
        };

        let generic_contract_matches = align_callable_generic_binders(ctx, &mut candidate, &inherited);
        let shape_matches = generic_contract_matches
            && candidate.kind == inherited.kind
            && candidate.parameters.len() == inherited.parameters.len()
            && candidate
                .parameters
                .iter()
                .zip(&inherited.parameters)
                .all(|(candidate, inherited)| candidate.external_label == inherited.external_label && candidate.rest == inherited.rest);
        let parameters_match = shape_matches
            && candidate.parameters.iter().zip(&inherited.parameters).all(|(candidate, inherited)| {
                let (Some(candidate), Some(inherited)) = (candidate.ty.ty(), inherited.ty.ty()) else {
                    return true;
                };
                ctx.check_knowledge_against_type(&TypeKnowledge::established(inherited, EvidenceOrigin::CallableSignature), candidate)
                    .is_proven()
            });
        let returns_match = match (candidate.return_type.ty(), inherited.return_type.ty()) {
            (Some(candidate), Some(inherited)) => ctx
                .check_knowledge_against_type(&TypeKnowledge::established(candidate, EvidenceOrigin::CallableSignature), inherited)
                .is_proven(),
            _ => true,
        };
        if !parameters_match || !returns_match {
            let range = candidate_semantic.source.as_ref().map(|source| source.range).unwrap_or(class_def.range);
            ctx.diagnostics.push(crate::diagnostic::SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::IncompatibleOverride,
                format!(
                    "override `{}` is incompatible with the inherited contract from `{}`",
                    callable.selector.encode(),
                    ancestor.name
                ),
                range,
            ));
        }
    }
}

/// Alpha-aligns corresponding method-generic binders for one override relation.
///
/// Receiver specialization has already substituted declaration-owned parameters
/// before this runs. Callable-owned binders remain rigid identities, so a direct
/// relation check would incorrectly reject equivalent declarations such as
/// `map<A, B>` overriding `map<X, Y>`. This operation substitutes only the
/// candidate callable's binders into the inherited callable's binders. It keeps
/// declaration-owner identities, `Self`, and all other free parameters intact.
///
/// Constraints are compared after the same alpha-renaming. Requiring the same
/// normalized constraints is conservative: it prevents an override from
/// strengthening obligations imposed on callers while avoiding a second
/// constraint-implication engine at this soundness boundary.
fn align_callable_generic_binders(
    ctx: &mut CheckingContext<'_>,
    candidate: &mut crate::dispatch::CallableSignature,
    inherited: &crate::dispatch::CallableSignature,
) -> bool {
    let (Some(candidate_generics), Some(inherited_generics)) = (candidate.generics.as_mut(), inherited.generics.as_ref()) else {
        return candidate.generics.is_none() && inherited.generics.is_none();
    };
    if candidate_generics.parameter_count() != inherited_generics.parameter_count() {
        return false;
    }

    let mut substitution = TypeSubstitution::new();
    for (&candidate_parameter, &inherited_parameter) in candidate_generics.parameters.iter().zip(&inherited_generics.parameters) {
        let candidate_data = ctx.store.type_parameter(candidate_parameter);
        let inherited_data = ctx.store.type_parameter(inherited_parameter);
        if candidate_data.kind != inherited_data.kind || candidate_data.variance != inherited_data.variance {
            return false;
        }
        substitution.bind(candidate_parameter, ctx.store.parameter_form(inherited_parameter));
    }

    for parameter in &mut candidate.parameters {
        parameter.ty = parameter.ty.map_type(|ty| substitution.apply(ctx.store, ty));
    }
    candidate.return_type = candidate.return_type.map_type(|ty| substitution.apply(ctx.store, ty));
    candidate_generics.constraints = candidate_generics
        .constraints
        .iter()
        .map(|constraint| substitute_generic_constraint(ctx, &substitution, constraint))
        .collect();

    let mut candidate_constraints = candidate_generics.constraints.to_vec();
    let mut inherited_constraints = inherited_generics.constraints.to_vec();
    candidate_constraints.sort();
    inherited_constraints.sort();
    candidate_constraints == inherited_constraints
}

fn substitute_generic_constraint(
    ctx: &mut CheckingContext<'_>,
    substitution: &TypeSubstitution,
    constraint: &crate::types::parameter::GenericConstraint,
) -> crate::types::parameter::GenericConstraint {
    use crate::types::parameter::{GenericConstraint, TypeTerm};

    let mut substitute_term = |term: &TypeTerm| match term {
        TypeTerm::Canonical(ty) => TypeTerm::Canonical(substitution.apply(ctx.store, *ty)),
        TypeTerm::SelfType(term) => TypeTerm::SelfType(term.clone()),
        TypeTerm::Infer(variable) => TypeTerm::Infer(*variable),
    };

    match constraint {
        GenericConstraint::Subtype { lower, upper } => GenericConstraint::Subtype {
            lower: substitute_term(lower),
            upper: substitute_term(upper),
        },
        GenericConstraint::Equivalent { left, right } => GenericConstraint::Equivalent {
            left: substitute_term(left),
            right: substitute_term(right),
        },
        GenericConstraint::Conforms { subject, trait_ref } => GenericConstraint::Conforms {
            subject: substitute_term(subject),
            trait_ref: crate::traits::TraitRef::new(
                trait_ref.declaration.clone(),
                trait_ref
                    .arguments
                    .iter()
                    .map(|&argument| substitution.apply(ctx.store, argument))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
        },
    }
}

pub(crate) fn member_side(member: &ClassMember) -> crate::identity::DispatchSide {
    if member.is_static() || member.attributes().iter().any(|attribute| attribute.name == "class") {
        crate::identity::DispatchSide::Class
    } else {
        crate::identity::DispatchSide::Instance
    }
}

/// Computes the primary declared surface and signature table for a class.
pub fn build_declared_class_surface(
    ctx: &mut CheckingContext<'_>,
    class_def: &ClassDef,
) -> (DeclarationSurface, HashMap<crate::identity::CallableId, CallableSemanticSignature>) {
    let decl_id = DeclarationId::new(ctx.current_module.clone(), class_def.name.clone().into());
    let mut surface = DeclarationSurface::new(Some(decl_id.clone()));
    let mut signatures = HashMap::new();

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
                signatures.insert(signature.callable.clone(), signature);
            }
            ClassMember::Variant(_) | ClassMember::Delegation(_) => {}
        }
    }

    // A delegation is source shorthand for one or two ordinary accessors.
    // Expand it after explicit members have been published so duplicate
    // selectors are diagnosed regardless of source order.
    for member in &class_def.members {
        let ClassMember::Delegation(delegation) = member else {
            continue;
        };
        let generated = match super::declaration_signature::delegated_accessor_signatures(ctx, &decl_id, delegation) {
            Ok(signatures) => signatures,
            Err(error) => {
                let (code, message, range) = match error {
                    super::declaration_signature::DelegationError::FieldNotFound(range) => (
                        DiagnosticCode::DelegationFieldNotFound,
                        format!("delegation field `{}` was not found on `{}`", delegation.target_field, decl_id.name),
                        range,
                    ),
                    super::declaration_signature::DelegationError::FieldTypeRequired(range) => (
                        DiagnosticCode::DelegationFieldTypeRequired,
                        format!("delegation field `{}` requires an explicit usable type", delegation.target_field),
                        range,
                    ),
                    super::declaration_signature::DelegationError::SetterRequiresMutableField(range) => (
                        DiagnosticCode::DelegationSetterRequiresMutableField,
                        format!("delegation field `{}` must be mutable for a setter", delegation.target_field),
                        range,
                    ),
                };
                ctx.diagnostics.push(crate::diagnostic::SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    code,
                    message,
                    range,
                ));
                continue;
            }
        };
        for signature in generated {
            if surface.get_callable(signature.side, &signature.selector).is_some() || signatures.contains_key(&signature.callable) {
                ctx.diagnostics.push(crate::diagnostic::SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplMemberConflict,
                    format!("delegated accessor `{}` conflicts with an existing member", signature.selector.encode()),
                    delegation.range,
                ));
                continue;
            }
            let side = signature.side;
            let projection = super::declaration_signature::project_semantic_signature(&signature);
            surface.add_callable_with_visibility(side, projection, member_visibility(member));
            signatures.insert(signature.callable.clone(), signature);
        }
    }

    (surface, signatures)
}

/// Pre-registers a class surface and its callable signatures in the context's dispatch table.
pub fn register_class_surface(ctx: &mut CheckingContext<'_>, class_def: &ClassDef) -> HashMap<crate::identity::CallableId, CallableSemanticSignature> {
    let decl_id = DeclarationId::new(ctx.current_module.clone(), class_def.name.clone().into());
    let class_ty = ctx.nominal_type_of(&decl_id).unwrap_or_else(|| ctx.store.nominal(decl_id.clone()));
    ctx.dispatch.make_mut().register_type(class_ty, decl_id.clone());

    let (surface, signatures) = build_declared_class_surface(ctx, class_def);
    ctx.register_surface(decl_id, surface);
    signatures
}

fn member_visibility(member: &ClassMember) -> MemberVisibility {
    member.visibility()
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
pub fn check_class_bodies(ctx: &mut CheckingContext<'_>, class_def: &ClassDef, signatures: &HashMap<crate::identity::CallableId, CallableSemanticSignature>) {
    let decl_id = DeclarationId::new(ctx.current_module.clone(), class_def.name.clone().into());
    // Keep resolver ownership independent from `ctx` while checking bodies. The
    // body checker mutably borrows the full context, so a scoped resolver that
    // directly borrows `ctx.resolver` would create an overlapping borrow.
    let parent_resolver = ctx.resolver.clone();
    let old_class = ctx.current_class.replace(decl_id.clone());

    for member in &class_def.members {
        let side = member_side(member);
        let scoped_resolver = crate::types::annotation::ScopedTypeResolver {
            parent: &parent_resolver,
            type_parameters: super::declaration_signature::declaration_type_level_bindings_for_side(ctx, &decl_id, side),
        };
        let old_side = ctx.current_side;
        ctx.current_side = side;
        match member {
            ClassMember::Field(f) => {
                check_field_initializer(ctx, &scoped_resolver, f);
            }
            ClassMember::Method(m) => {
                let callable = super::declaration_signature::callable_id_for_member(&decl_id, member);
                if let Some(signature) = callable.as_ref().and_then(|id| signatures.get(id)) {
                    check_callable_body(ctx, &m.params, signature, m.body.statements().unwrap_or(&[]));
                }
            }
            ClassMember::Getter(g) => {
                let callable = super::declaration_signature::callable_id_for_member(&decl_id, member);
                if let Some(signature) = callable.as_ref().and_then(|id| signatures.get(id)) {
                    check_callable_body(ctx, &[], signature, g.body.statements().unwrap_or(&[]));
                }
            }
            ClassMember::Setter(s) => {
                let callable = super::declaration_signature::callable_id_for_member(&decl_id, member);
                if let Some(signature) = callable.as_ref().and_then(|id| signatures.get(id)) {
                    check_callable_body(ctx, std::slice::from_ref(&s.param), signature, s.body.statements().unwrap_or(&[]));
                }
            }
            ClassMember::Index(i) => {
                let mut params = i.params.clone();
                if let phalcom_ast::ast::IndexAccessor::Set { value } = &i.accessor {
                    params.push((**value).clone());
                }
                let callable = super::declaration_signature::callable_id_for_member(&decl_id, member);
                if let Some(signature) = callable.as_ref().and_then(|id| signatures.get(id)) {
                    check_callable_body(ctx, &params, signature, i.body.statements().unwrap_or(&[]));
                }
            }
            _ => {}
        }
        ctx.current_side = old_side;
    }

    ctx.current_class = old_class;
}

/// Checks a class declaration and all its members.
pub fn check_class(ctx: &mut CheckingContext<'_>, class_def: &ClassDef) {
    let signatures = register_class_surface(ctx, class_def);
    check_class_bodies(ctx, class_def, &signatures);
}

fn check_callable_body(ctx: &mut CheckingContext<'_>, params: &[ParameterDef], signature: &CallableSemanticSignature, body: &[Statement]) {
    ctx.push_scope();

    // Bind parameters
    for (index, param) in params.iter().enumerate() {
        let knowledge = signature
            .parameter_at(index)
            .map(|parameter| parameter.declared_type.to_knowledge())
            .unwrap_or_else(|| TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence));
        let param_k = knowledge;
        let param_invalidity = crate::checker::causal::CausalInvalidity::Clean;
        ctx.bind_callable_parameter_with_causal(param.name.clone(), param_k, param.range, param_invalidity);
    }

    // Reuse declaration-owned return contract. Signature formation already
    // resolved source annotations and generic binders before body analysis.
    let expected_return = {
        let knowledge = signature.declared_return.to_knowledge();
        knowledge.ty().map(|ty| CallableReturnContract {
            ty,
            basis: signature.declared_return.basis,
            origin: knowledge.origin().unwrap_or(EvidenceOrigin::CallableSignature),
            is_dynamic: knowledge.is_dynamic(),
            source: signature.source.as_ref().map(|source| source.range),
        })
    };

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
