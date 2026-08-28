//! Canonical declaration-owned callable signature construction.
//!
//! This module is the one source-to-semantic boundary for source callable
//! declarations. It resolves declaration syntax into range-free semantic type
//! facts first; dispatch/member surfaces are projections of that result.

use super::context::CheckingContext;
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
use crate::dispatch::{CallableParameter, CallableSemanticKind, CallableSignature};
use crate::identity::{CallableId, CallableParameterId, DeclarationId, DispatchSide};
use crate::signature::{CallableParameterSemantic, CallableSemanticSignature};
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::parameter::TypeParameterOwner;
use phalcom_ast::ast::{ClassMember, ParameterDef, RestMode};
use phalcom_common::selector::{Selector, SelectorSlot};

pub(crate) fn callable_id_for_member(owner: &DeclarationId, member: &ClassMember) -> Option<CallableId> {
    let declared_side = super::declaration::member_side(member);
    match member {
        ClassMember::Method(method) => {
            let slots = method
                .params
                .iter()
                .map(|parameter| {
                    parameter
                        .label
                        .as_ref()
                        .map(|label| SelectorSlot::Label(label.clone()))
                        .unwrap_or(SelectorSlot::Positional)
                })
                .collect::<Vec<_>>();
            let selector = Selector::method(&method.name, slots).ok()?;
            let is_constructor = method.is_constructor || method.attributes.iter().any(|attribute| attribute.name == "constructor");
            let side = if is_constructor { DispatchSide::Class } else { declared_side };
            Some(CallableId::new(owner.clone(), selector, side))
        }
        ClassMember::Getter(getter) => Selector::getter(&getter.name)
            .ok()
            .map(|selector| CallableId::new(owner.clone(), selector, declared_side)),
        ClassMember::Setter(setter) => Selector::setter(&setter.name)
            .ok()
            .map(|selector| CallableId::new(owner.clone(), selector, declared_side)),
        ClassMember::Index(index) => {
            let slots = index
                .params
                .iter()
                .map(|parameter| {
                    parameter
                        .label
                        .as_ref()
                        .map(|label| SelectorSlot::Label(label.clone()))
                        .unwrap_or(SelectorSlot::Positional)
                })
                .collect::<Vec<_>>();
            let selector = match &index.accessor {
                phalcom_ast::ast::IndexAccessor::Get => Selector::subscript_get(slots).ok()?,
                phalcom_ast::ast::IndexAccessor::Set { .. } => Selector::subscript_set(slots).ok()?,
            };
            Some(CallableId::new(owner.clone(), selector, declared_side))
        }
        ClassMember::Field(_) | ClassMember::Variant(_) => None,
    }
}

fn annotation_fact(
    ctx: &mut CheckingContext<'_>,
    resolver: &dyn crate::types::annotation::TypeResolver,
    annotation: Option<&phalcom_ast::ast::TypeAnnotation>,
    missing: UnknownReason,
) -> DeclaredTypeFact {
    let Some(annotation) = annotation else {
        return DeclaredTypeFact::unknown(missing);
    };
    let (knowledge, _) = ctx.resolve_type_annotation(resolver, annotation);
    DeclaredTypeFact::from_knowledge_with_basis(&knowledge, DeclaredTypeBasis::SourceAnnotation)
}

fn parameter_fact(
    ctx: &mut CheckingContext<'_>,
    callable: &CallableId,
    index: usize,
    parameter: &ParameterDef,
    resolver: &dyn crate::types::annotation::TypeResolver,
    missing: UnknownReason,
) -> CallableParameterSemantic {
    let declared_type = annotation_fact(ctx, resolver, parameter.annotation.as_ref(), missing);
    let mut semantic = CallableParameterSemantic::new(CallableParameterId::new(callable.clone(), index as u32), parameter.name.clone(), declared_type)
        .with_rest(parameter.rest_mode);
    if let Some(label) = &parameter.label {
        semantic = semantic.with_label(label.clone());
    }
    semantic
}

pub(crate) fn semantic_signature_for_member(ctx: &mut CheckingContext<'_>, owner: &DeclarationId, member: &ClassMember) -> Option<CallableSemanticSignature> {
    let callable = callable_id_for_member(owner, member)?;

    let declaration_type_parameters = ctx
        .declaration_generic_signature(owner)
        .map(|signature| {
            signature
                .parameters
                .iter()
                .map(|&parameter_id| {
                    let name = ctx.store.type_parameter(parameter_id).name.to_string();
                    let form = ctx.store.parameter_form(parameter_id);
                    (name, form)
                })
                .collect()
        })
        .unwrap_or_default();
    let parent_resolver = ctx.resolver.clone();
    let declaration_resolver = crate::types::annotation::ScopedTypeResolver {
        parent: &parent_resolver,
        type_parameters: declaration_type_parameters,
    };

    let (generics, parameters, declared_return) = match member {
        ClassMember::Method(method) => {
            let generic_signature = if method.generic_parameters.is_empty() {
                None
            } else {
                let mut diagnostics = Vec::new();
                let signature = crate::types::annotation::resolve_generic_signature(
                    ctx.store,
                    ctx.declarations,
                    &declaration_resolver,
                    &ctx.current_module,
                    TypeParameterOwner::Callable(callable.clone()),
                    &method.generic_parameters,
                    method.where_clause.as_ref(),
                    &mut diagnostics,
                );
                ctx.publish_diagnostics(diagnostics);
                Some(signature)
            };
            let method_type_parameters = generic_signature
                .as_ref()
                .map(|signature| {
                    signature
                        .parameters
                        .iter()
                        .map(|&parameter_id| {
                            let name = ctx.store.type_parameter(parameter_id).name.to_string();
                            let form = ctx.store.parameter_form(parameter_id);
                            (name, form)
                        })
                        .collect()
                })
                .unwrap_or_default();
            let method_resolver = crate::types::annotation::ScopedTypeResolver {
                parent: &declaration_resolver,
                type_parameters: method_type_parameters,
            };
            let parameters = method
                .params
                .iter()
                .enumerate()
                .map(|(index, parameter)| parameter_fact(ctx, &callable, index, parameter, &method_resolver, UnknownReason::NoTypeEvidence))
                .collect::<Vec<_>>()
                .into_boxed_slice();
            let is_constructor = method.is_constructor || method.attributes.iter().any(|attribute| attribute.name == "constructor");
            let declared_return = if is_constructor {
                let self_type = ctx.store.self_type(crate::types::parameter::SelfTypeTerm {
                    owner: owner.clone(),
                    side: DispatchSide::Class,
                    role: crate::types::parameter::SelfRole::InstanceType,
                });
                let knowledge = TypeKnowledge::established(self_type, EvidenceOrigin::ConstructorSemantics);
                DeclaredTypeFact::from_knowledge_with_basis(&knowledge, DeclaredTypeBasis::ConstructorSemantics)
            } else {
                annotation_fact(ctx, &method_resolver, method.return_annotation.as_ref(), UnknownReason::UnannotatedDeclaration)
            };
            (generic_signature, parameters, declared_return)
        }
        ClassMember::Getter(getter) => (
            None,
            Vec::<CallableParameterSemantic>::new().into_boxed_slice(),
            annotation_fact(
                ctx,
                &declaration_resolver,
                getter.return_annotation.as_ref(),
                UnknownReason::UnannotatedDeclaration,
            ),
        ),
        ClassMember::Setter(setter) => {
            let parameter = parameter_fact(ctx, &callable, 0, &setter.param, &declaration_resolver, UnknownReason::UnannotatedDeclaration);
            let unit = TypeKnowledge::established(ctx.store.unit(), EvidenceOrigin::DeclarationSemantics);
            (
                None,
                vec![parameter].into_boxed_slice(),
                DeclaredTypeFact::from_knowledge_with_basis(&unit, DeclaredTypeBasis::DeclarationSemantics),
            )
        }
        ClassMember::Index(index) => {
            let mut parameters = index
                .params
                .iter()
                .enumerate()
                .map(|(parameter_index, parameter)| {
                    parameter_fact(ctx, &callable, parameter_index, parameter, &declaration_resolver, UnknownReason::NoTypeEvidence)
                })
                .collect::<Vec<_>>();
            let declared_return = match &index.accessor {
                phalcom_ast::ast::IndexAccessor::Get => {
                    annotation_fact(ctx, &declaration_resolver, index.return_annotation.as_ref(), UnknownReason::NoTypeEvidence)
                }
                phalcom_ast::ast::IndexAccessor::Set { put } => {
                    let put_semantic = parameter_fact(ctx, &callable, parameters.len(), put, &declaration_resolver, UnknownReason::NoTypeEvidence);
                    let result = put_semantic.declared_type.clone();
                    parameters.push(put_semantic);
                    result
                }
            };
            (None, parameters.into_boxed_slice(), declared_return)
        }
        ClassMember::Field(_) | ClassMember::Variant(_) => return None,
    };

    Some(CallableSemanticSignature {
        callable: callable.clone(),
        owner: owner.clone(),
        side: callable.side,
        selector: callable.selector.clone(),
        generics,
        parameters,
        declared_return,
        inferred_return: None,
        source: None,
        implementation: phalcom_native_meta::ImplementationKind::Source,
        native_id: None,
        effects: phalcom_native_meta::EffectSpec::Unknown,
        raises: phalcom_native_meta::RaisesSpec::Unknown,
        flow: phalcom_native_meta::ReturnFlowSpec::Value,
        lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
    })
}

pub(crate) fn project_semantic_signature(signature: &CallableSemanticSignature) -> CallableSignature {
    let parameters = signature
        .parameters
        .iter()
        .map(|parameter| {
            let mut projected =
                CallableParameter::new(parameter.local_name.to_string(), parameter.declared_type.to_knowledge()).with_rest(parameter.rest != RestMode::None);
            if let Some(label) = &parameter.external_label {
                projected = projected.with_label(label.to_string());
            }
            projected
        })
        .collect::<Vec<_>>();

    let kind = if signature.declared_return.basis == DeclaredTypeBasis::ConstructorSemantics {
        CallableSemanticKind::Constructor
    } else if signature.implementation != phalcom_native_meta::ImplementationKind::Source {
        CallableSemanticKind::Native
    } else {
        CallableSemanticKind::Ordinary
    };

    let mut projected = CallableSignature::new(signature.selector.clone(), parameters, signature.declared_return.to_knowledge()).with_kind(kind);
    if let Some(generics) = &signature.generics {
        projected = projected.with_generics(generics.clone());
    }
    projected
}
