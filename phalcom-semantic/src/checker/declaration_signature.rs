//! Canonical declaration-owned callable signature construction.
//!
//! This module is the one source-to-semantic boundary for source callable
//! declarations. It resolves declaration syntax into range-free semantic type
//! facts first; dispatch/member surfaces are projections of that result.

use super::context::CheckingContext;
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::{CallableParameter, CallableSemanticKind, CallableSignature};
use crate::identity::{CallableId, CallableOwnerId, CallableParameterId, DeclarationId, DispatchSide, FieldId};
use crate::signature::{CallableParameterSemantic, CallableSemanticSignature, FieldSemanticSignature};
use crate::types::annotation::{TypeFormationOutcome, TypeFormationSite, type_level_binding_for_parameter};
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::parameter::TypeParameterOwner;
use phalcom_ast::ast::{ClassMember, EnumBehaviorMember, GetterDef, IndexMethodDef, MethodDef, ParameterDef, SetterDef};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub(crate) enum CallableSyntaxRef<'a> {
    Method(&'a MethodDef),
    Getter(&'a GetterDef),
    Setter(&'a SetterDef),
    Index(&'a IndexMethodDef),
}

impl<'a> CallableSyntaxRef<'a> {
    pub(crate) fn range(&self) -> SourceRange {
        match self {
            CallableSyntaxRef::Method(m) => m.range,
            CallableSyntaxRef::Getter(g) => g.range,
            CallableSyntaxRef::Setter(s) => s.range,
            CallableSyntaxRef::Index(i) => i.range,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn name_range(&self) -> SourceRange {
        match self {
            CallableSyntaxRef::Method(m) => m.name_range,
            CallableSyntaxRef::Getter(g) => g.name_range,
            CallableSyntaxRef::Setter(s) => s.name_range,
            CallableSyntaxRef::Index(i) => i.name_range,
        }
    }

    pub(crate) fn has_body(&self) -> bool {
        match self {
            CallableSyntaxRef::Method(m) => m.body.statements().is_some(),
            CallableSyntaxRef::Getter(g) => g.body.statements().is_some(),
            CallableSyntaxRef::Setter(s) => s.body.statements().is_some(),
            CallableSyntaxRef::Index(_) => true,
        }
    }

    pub(crate) fn selector_base(&self) -> phalcom_common::selector::SelectorBase {
        match self {
            CallableSyntaxRef::Method(m) => phalcom_common::selector::SelectorBase::Named(m.name.clone()),
            CallableSyntaxRef::Getter(g) => phalcom_common::selector::SelectorBase::Named(g.name.clone()),
            CallableSyntaxRef::Setter(s) => phalcom_common::selector::SelectorBase::Named(s.name.clone()),
            CallableSyntaxRef::Index(_) => phalcom_common::selector::SelectorBase::Subscript,
        }
    }

    pub(crate) fn attributes(&self) -> &[phalcom_ast::ast::Attribute] {
        match self {
            CallableSyntaxRef::Method(m) => &m.attributes,
            CallableSyntaxRef::Getter(g) => &g.attributes,
            CallableSyntaxRef::Setter(s) => &s.attributes,
            CallableSyntaxRef::Index(i) => &i.attributes,
        }
    }
}

impl<'a> From<&'a EnumBehaviorMember> for CallableSyntaxRef<'a> {
    fn from(member: &'a EnumBehaviorMember) -> Self {
        match member {
            EnumBehaviorMember::Method(m) => CallableSyntaxRef::Method(m),
            EnumBehaviorMember::Getter(g) => CallableSyntaxRef::Getter(g),
            EnumBehaviorMember::Setter(s) => CallableSyntaxRef::Setter(s),
            EnumBehaviorMember::Index(i) => CallableSyntaxRef::Index(i),
        }
    }
}

pub(crate) fn callable_id_for_syntax(owner: &CallableOwnerId, syntax: CallableSyntaxRef<'_>, declared_side: DispatchSide) -> Option<CallableId> {
    match syntax {
        CallableSyntaxRef::Method(method) => {
            let slots = method
                .params
                .iter()
                .filter(|parameter| parameter.rest_mode == phalcom_ast::ast::RestMode::None)
                .map(|parameter| {
                    parameter
                        .label
                        .as_ref()
                        .map(|label| {
                            if label == "_" {
                                SelectorSlot::Positional
                            } else {
                                SelectorSlot::Label(label.clone())
                            }
                        })
                        .unwrap_or(SelectorSlot::Positional)
                })
                .collect::<Vec<_>>();
            let selector = Selector::method(&method.name, slots).ok()?;
            let is_constructor = method.is_constructor || method.attributes.iter().any(|attribute| attribute.name == "constructor");
            let side = if is_constructor { DispatchSide::Class } else { declared_side };
            Some(CallableId::new(owner.clone(), selector, side))
        }
        CallableSyntaxRef::Getter(getter) => Selector::getter(&getter.name)
            .ok()
            .map(|selector| CallableId::new(owner.clone(), selector, declared_side)),
        CallableSyntaxRef::Setter(setter) => Selector::setter(&setter.name)
            .ok()
            .map(|selector| CallableId::new(owner.clone(), selector, declared_side)),
        CallableSyntaxRef::Index(index) => {
            let slots = index
                .params
                .iter()
                .map(|parameter| {
                    parameter
                        .label
                        .as_ref()
                        .map(|label| {
                            if label == "_" {
                                SelectorSlot::Positional
                            } else {
                                SelectorSlot::Label(label.clone())
                            }
                        })
                        .unwrap_or(SelectorSlot::Positional)
                })
                .collect::<Vec<_>>();
            let selector = match &index.accessor {
                phalcom_ast::ast::IndexAccessor::Get => Selector::subscript_get(slots).ok()?,
                phalcom_ast::ast::IndexAccessor::Set { .. } => Selector::subscript_set(slots).ok()?,
            };
            Some(CallableId::new(owner.clone(), selector, declared_side))
        }
    }
}

pub(crate) fn callable_id_for_member(owner: &DeclarationId, member: &ClassMember) -> Option<CallableId> {
    let declared_side = super::declaration::member_side(member);
    let syntax = match member {
        ClassMember::Method(m) => CallableSyntaxRef::Method(m),
        ClassMember::Getter(g) => CallableSyntaxRef::Getter(g),
        ClassMember::Setter(s) => CallableSyntaxRef::Setter(s),
        ClassMember::Index(i) => CallableSyntaxRef::Index(i),
        ClassMember::Field(_) | ClassMember::Variant(_) => return None,
    };
    callable_id_for_syntax(&CallableOwnerId::Declaration(owner.clone()), syntax, declared_side)
}

fn annotation_fact(
    ctx: &mut CheckingContext<'_>,
    resolver: &dyn crate::types::annotation::TypeResolver,
    site: &TypeFormationSite,
    annotation: Option<&phalcom_ast::ast::TypeAnnotation>,
    missing: UnknownReason,
) -> DeclaredTypeFact {
    let Some(annotation) = annotation else {
        return DeclaredTypeFact::unknown(missing);
    };
    let mut diagnostics = Vec::new();
    let knowledge = crate::types::annotation::resolve_type_annotation(ctx.store, ctx.declarations, resolver, site, annotation, &mut diagnostics);
    ctx.publish_diagnostics(diagnostics);
    DeclaredTypeFact::from_knowledge_with_basis(&knowledge, DeclaredTypeBasis::SourceAnnotation)
}

fn parameter_fact(
    ctx: &mut CheckingContext<'_>,
    callable: &CallableId,
    index: usize,
    parameter: &ParameterDef,
    resolver: &dyn crate::types::annotation::TypeResolver,
    site: &TypeFormationSite,
    missing: UnknownReason,
) -> CallableParameterSemantic {
    let declared_type = annotation_fact(ctx, resolver, site, parameter.annotation.as_ref(), missing);
    let mut semantic = CallableParameterSemantic::new(CallableParameterId::new(callable.clone(), index as u32), parameter.name.clone(), declared_type)
        .with_rest(parameter.rest_mode)
        .with_source(crate::diagnostic::SemanticSourceSpan::new(ctx.current_module.clone(), parameter.name_range));
    if let Some(label) = &parameter.label {
        semantic = semantic.with_label(label.clone());
    }
    semantic
}

pub(crate) fn declaration_type_level_bindings_for_side(
    ctx: &mut CheckingContext<'_>,
    owner: &DeclarationId,
    _side: DispatchSide,
) -> HashMap<String, crate::types::annotation::TypeLevelBinding> {
    declaration_type_level_bindings(ctx, owner)
}

fn declaration_type_level_bindings(ctx: &mut CheckingContext<'_>, owner: &DeclarationId) -> HashMap<String, crate::types::annotation::TypeLevelBinding> {
    let parameter_ids = ctx
        .declaration_generic_signature(owner)
        .map(|signature| signature.parameters.to_vec())
        .unwrap_or_default();
    let parameters = parameter_ids
        .into_iter()
        .map(|parameter_id| {
            let data = ctx.store.type_parameter(parameter_id);
            (data.name.to_string(), parameter_id)
        })
        .collect::<Vec<_>>();
    parameters
        .into_iter()
        .map(|(name, parameter_id)| (name, type_level_binding_for_parameter(ctx.store, parameter_id)))
        .collect()
}

/// Canonical declaration for the root `Class.new()` allocator behavior.
///
/// The member is inherited by class objects through ordinary instance-side
/// dispatch on `Class`; its constructor result is receiver-specialized `Self`.
/// Standalone checker contexts project this declaration into dispatch, while
/// workspace sessions also retain it in `CallableSignatureTable`.
pub(crate) fn canonical_core_class_new_signature(store: &mut crate::types::store::TypeStore) -> CallableSemanticSignature {
    let owner = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Class);
    let selector = Selector::method("new", Vec::new()).expect("root Class.new selector must be valid");
    let callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);
    let self_type = store.self_type(crate::types::parameter::SelfTypeTerm {
        owner: owner.clone(),
        side: DispatchSide::Instance,
        role: crate::types::parameter::SelfRole::InstanceType,
    });
    let knowledge = TypeKnowledge::established(self_type, EvidenceOrigin::ConstructorSemantics);
    CallableSemanticSignature {
        callable,
        owner,
        side: DispatchSide::Instance,
        selector,
        generics: None,
        parameters: Vec::<CallableParameterSemantic>::new().into_boxed_slice(),
        declared_return: DeclaredTypeFact::from_knowledge_with_basis(&knowledge, DeclaredTypeBasis::ConstructorSemantics),
        return_validation: crate::signature::ReturnContractValidation::NotApplicable,
        inferred_return: None,
        source: None,
        implementation: phalcom_native_meta::ImplementationKind::Generated,
        native_id: None,
        effects: phalcom_native_meta::EffectSpec::Unknown,
        raises: phalcom_native_meta::RaisesSpec::Unknown,
        flow: phalcom_native_meta::ReturnFlowSpec::Value,
        lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
    }
}

pub(crate) fn field_id_for_member(owner: &DeclarationId, member: &ClassMember) -> Option<FieldId> {
    let ClassMember::Field(field) = member else {
        return None;
    };
    Some(FieldId::new(owner.clone(), field.name.clone(), super::declaration::member_side(member)))
}

pub(crate) fn semantic_field_signature_for_member(
    ctx: &mut CheckingContext<'_>,
    owner: &DeclarationId,
    member: &ClassMember,
) -> Option<FieldSemanticSignature> {
    let ClassMember::Field(field) = member else {
        return None;
    };
    let field_id = field_id_for_member(owner, member)?;
    let side = field_id.side;
    let declaration_type_parameters = declaration_type_level_bindings_for_side(ctx, owner, side);
    let parent_resolver = ctx.resolver.clone();
    let declaration_resolver = crate::types::annotation::ScopedTypeResolver {
        parent: &parent_resolver,
        type_parameters: declaration_type_parameters,
    };
    let formation_site = TypeFormationSite::member(ctx.current_module.clone(), owner.clone(), side);
    let declared_type = annotation_fact(
        ctx,
        &declaration_resolver,
        &formation_site,
        field.annotation.as_ref(),
        UnknownReason::UnannotatedDeclaration,
    );
    Some(FieldSemanticSignature {
        field: field_id,
        owner: owner.clone(),
        side,
        name: field.name.clone().into(),
        mutable: field.mutable,
        declared_type,
        source: None,
    })
}

pub(crate) fn project_field_signature(signature: &FieldSemanticSignature) -> TypeKnowledge {
    signature.declared_type.to_knowledge()
}

fn initial_return_validation(
    declared_return: &DeclaredTypeFact,
    implementation: phalcom_native_meta::ImplementationKind,
) -> crate::signature::ReturnContractValidation {
    if !matches!(implementation, phalcom_native_meta::ImplementationKind::Source) || declared_return.basis != DeclaredTypeBasis::SourceAnnotation {
        return crate::signature::ReturnContractValidation::NotApplicable;
    }
    if declared_return.is_dynamic() {
        crate::signature::ReturnContractValidation::DynamicBoundary
    } else if declared_return.is_known() {
        crate::signature::ReturnContractValidation::Unchecked
    } else {
        crate::signature::ReturnContractValidation::NotApplicable
    }
}

pub(crate) fn semantic_signature_for_syntax(
    ctx: &mut CheckingContext<'_>,
    owner: &CallableOwnerId,
    syntax: CallableSyntaxRef<'_>,
    declared_side: DispatchSide,
) -> Option<CallableSemanticSignature> {
    let callable = callable_id_for_syntax(owner, syntax, declared_side)?;
    let declaration_owner = owner.declaration();

    let formation_side = callable.side;
    let is_constructor = matches!(syntax, CallableSyntaxRef::Method(method) if method.is_constructor || method.attributes.iter().any(|attribute| attribute.name == "constructor"));
    let declaration_type_parameters = if is_constructor {
        declaration_type_level_bindings(ctx, declaration_owner)
    } else {
        declaration_type_level_bindings_for_side(ctx, declaration_owner, formation_side)
    };
    let parent_resolver = ctx.resolver.clone();
    let declaration_resolver = crate::types::annotation::ScopedTypeResolver {
        parent: &parent_resolver,
        type_parameters: declaration_type_parameters,
    };
    let formation_site = TypeFormationSite::member(ctx.current_module.clone(), declaration_owner.clone(), formation_side);

    let (generics, parameters, declared_return) = match syntax {
        CallableSyntaxRef::Method(method) => {
            let generic_signature = resolve_callable_local_generics(
                ctx,
                &declaration_resolver,
                &formation_site,
                &callable,
                &method.generic_parameters,
                method.where_clause.as_ref(),
                method.range,
                "method",
            );
            let method_type_parameters = generic_signature
                .as_ref()
                .map(|signature| signature.parameters.to_vec())
                .unwrap_or_default()
                .into_iter()
                .map(|parameter_id| {
                    let name = ctx.store.type_parameter(parameter_id).name.to_string();
                    (name, parameter_id)
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|(name, parameter_id)| (name, type_level_binding_for_parameter(ctx.store, parameter_id)))
                .collect();
            let method_resolver = crate::types::annotation::ScopedTypeResolver {
                parent: &declaration_resolver,
                type_parameters: method_type_parameters,
            };
            let parameters = method
                .params
                .iter()
                .enumerate()
                .map(|(index, parameter)| {
                    parameter_fact(
                        ctx,
                        &callable,
                        index,
                        parameter,
                        &method_resolver,
                        &formation_site,
                        UnknownReason::NoTypeEvidence,
                    )
                })
                .collect::<Vec<_>>()
                .into_boxed_slice();
            let is_constructor = method.is_constructor || method.attributes.iter().any(|attribute| attribute.name == "constructor");
            let declared_return = if is_constructor {
                let self_type = ctx.store.self_type(crate::types::parameter::SelfTypeTerm {
                    owner: declaration_owner.clone(),
                    side: DispatchSide::Class,
                    role: crate::types::parameter::SelfRole::InstanceType,
                });
                let knowledge = TypeKnowledge::established(self_type, EvidenceOrigin::ConstructorSemantics);
                DeclaredTypeFact::from_knowledge_with_basis(&knowledge, DeclaredTypeBasis::ConstructorSemantics)
            } else {
                annotation_fact(
                    ctx,
                    &method_resolver,
                    &formation_site,
                    method.return_annotation.as_ref(),
                    UnknownReason::UnannotatedDeclaration,
                )
            };
            (generic_signature, parameters, declared_return)
        }
        CallableSyntaxRef::Getter(getter) => {
            let generic_signature = resolve_callable_local_generics(
                ctx,
                &declaration_resolver,
                &formation_site,
                &callable,
                &getter.generic_parameters,
                getter.where_clause.as_ref(),
                getter.range,
                "getter",
            );
            let getter_type_parameters = generic_signature
                .as_ref()
                .map(|signature| signature.parameters.to_vec())
                .unwrap_or_default()
                .into_iter()
                .map(|parameter_id| {
                    let name = ctx.store.type_parameter(parameter_id).name.to_string();
                    (name, parameter_id)
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|(name, parameter_id)| (name, type_level_binding_for_parameter(ctx.store, parameter_id)))
                .collect();
            let getter_resolver = crate::types::annotation::ScopedTypeResolver {
                parent: &declaration_resolver,
                type_parameters: getter_type_parameters,
            };
            (
                generic_signature,
                Vec::<CallableParameterSemantic>::new().into_boxed_slice(),
                annotation_fact(
                    ctx,
                    &getter_resolver,
                    &formation_site,
                    getter.return_annotation.as_ref(),
                    UnknownReason::UnannotatedDeclaration,
                ),
            )
        }
        CallableSyntaxRef::Setter(setter) => {
            let generic_signature = resolve_callable_local_generics(
                ctx,
                &declaration_resolver,
                &formation_site,
                &callable,
                &setter.generic_parameters,
                setter.where_clause.as_ref(),
                setter.range,
                "setter",
            );
            let setter_type_parameters = generic_signature
                .as_ref()
                .map(|signature| signature.parameters.to_vec())
                .unwrap_or_default()
                .into_iter()
                .map(|parameter_id| {
                    let name = ctx.store.type_parameter(parameter_id).name.to_string();
                    (name, type_level_binding_for_parameter(ctx.store, parameter_id))
                })
                .collect();
            let setter_resolver = crate::types::annotation::ScopedTypeResolver {
                parent: &declaration_resolver,
                type_parameters: setter_type_parameters,
            };
            let parameter = parameter_fact(
                ctx,
                &callable,
                0,
                &setter.param,
                &setter_resolver,
                &formation_site,
                UnknownReason::UnannotatedDeclaration,
            );
            let unit = TypeKnowledge::established(ctx.store.unit(), EvidenceOrigin::DeclarationSemantics);
            (
                generic_signature,
                vec![parameter].into_boxed_slice(),
                DeclaredTypeFact::from_knowledge_with_basis(&unit, DeclaredTypeBasis::DeclarationSemantics),
            )
        }
        CallableSyntaxRef::Index(index) => {
            let generic_signature = resolve_callable_local_generics(
                ctx,
                &declaration_resolver,
                &formation_site,
                &callable,
                &index.generic_parameters,
                index.where_clause.as_ref(),
                index.range,
                "index member",
            );
            let index_type_parameters = generic_signature
                .as_ref()
                .map(|signature| signature.parameters.to_vec())
                .unwrap_or_default()
                .into_iter()
                .map(|parameter_id| {
                    let name = ctx.store.type_parameter(parameter_id).name.to_string();
                    (name, type_level_binding_for_parameter(ctx.store, parameter_id))
                })
                .collect();
            let index_resolver = crate::types::annotation::ScopedTypeResolver {
                parent: &declaration_resolver,
                type_parameters: index_type_parameters,
            };
            let mut parameters = index
                .params
                .iter()
                .enumerate()
                .map(|(parameter_index, parameter)| {
                    parameter_fact(
                        ctx,
                        &callable,
                        parameter_index,
                        parameter,
                        &index_resolver,
                        &formation_site,
                        UnknownReason::NoTypeEvidence,
                    )
                })
                .collect::<Vec<_>>();
            let declared_return = match &index.accessor {
                phalcom_ast::ast::IndexAccessor::Get => annotation_fact(
                    ctx,
                    &index_resolver,
                    &formation_site,
                    index.return_annotation.as_ref(),
                    UnknownReason::NoTypeEvidence,
                ),
                phalcom_ast::ast::IndexAccessor::Set { put } => {
                    let put_semantic = parameter_fact(
                        ctx,
                        &callable,
                        parameters.len(),
                        put,
                        &index_resolver,
                        &formation_site,
                        UnknownReason::NoTypeEvidence,
                    );
                    let result = put_semantic.declared_type.clone();
                    parameters.push(put_semantic);
                    result
                }
            };
            (generic_signature, parameters.into_boxed_slice(), declared_return)
        }
    };

    let return_validation = initial_return_validation(&declared_return, phalcom_native_meta::ImplementationKind::Source);

    Some(CallableSemanticSignature {
        callable: callable.clone(),
        owner: declaration_owner.clone(),
        side: callable.side,
        selector: callable.selector.clone(),
        generics,
        parameters,
        declared_return,
        return_validation,
        inferred_return: None,
        source: Some(crate::diagnostic::SemanticSourceSpan::new(ctx.current_module.clone(), syntax.range())),
        implementation: phalcom_native_meta::ImplementationKind::Source,
        native_id: None,
        effects: phalcom_native_meta::EffectSpec::Unknown,
        raises: phalcom_native_meta::RaisesSpec::Unknown,
        flow: phalcom_native_meta::ReturnFlowSpec::Value,
        lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
    })
}

#[allow(clippy::too_many_arguments)]
fn resolve_callable_local_generics(
    ctx: &mut CheckingContext<'_>,
    declaration_resolver: &dyn crate::types::annotation::TypeResolver,
    formation_site: &TypeFormationSite,
    callable: &CallableId,
    generic_parameters: &[phalcom_ast::ast::GenericParameterSyntax],
    where_clause: Option<&phalcom_ast::ast::WhereClauseSyntax>,
    range: SourceRange,
    callable_kind: &str,
) -> Option<crate::types::parameter::GenericSignature> {
    if generic_parameters.is_empty() {
        return None;
    }

    let mut diagnostics = Vec::new();
    let signature = crate::types::annotation::resolve_generic_signature(
        ctx.store,
        ctx.declarations,
        declaration_resolver,
        formation_site,
        TypeParameterOwner::Callable(callable.clone()),
        crate::types::annotation::GenericBinderSite::Callable,
        generic_parameters,
        where_clause,
        &mut diagnostics,
    );
    let signature = match signature {
        TypeFormationOutcome::Ready(signature) => Some(signature),
        TypeFormationOutcome::Dynamic => {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::AnnotationUnsupported,
                format!("generic {callable_kind} signature depends on a dynamic type-form boundary"),
                range,
            ));
            None
        }
        TypeFormationOutcome::Missing(reason) => {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::AnnotationUnresolved,
                format!("generic {callable_kind} signature publication missing: {reason:?}"),
                range,
            ));
            None
        }
        TypeFormationOutcome::Unresolved(_) | TypeFormationOutcome::Invalid(_) => None,
        TypeFormationOutcome::Blocked(reason) => {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::AnalysisBlocked,
                format!("generic {callable_kind} signature publication blocked: {reason:?}"),
                range,
            ));
            None
        }
        TypeFormationOutcome::Cancelled => {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::AnalysisBlocked,
                format!("generic {callable_kind} signature publication cancelled"),
                range,
            ));
            None
        }
        TypeFormationOutcome::BudgetExceeded(report) => {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::AnalysisBudgetExceeded,
                format!("generic {callable_kind} signature publication exceeded budget: {report:?}"),
                range,
            ));
            None
        }
        TypeFormationOutcome::InternalFailure(failure) => {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::AnalysisInternalFailure,
                format!("generic {callable_kind} signature publication failed: {failure}"),
                range,
            ));
            None
        }
    };
    ctx.publish_diagnostics(diagnostics);
    signature
}

pub(crate) fn semantic_signature_for_member(ctx: &mut CheckingContext<'_>, owner: &DeclarationId, member: &ClassMember) -> Option<CallableSemanticSignature> {
    let declared_side = super::declaration::member_side(member);
    let syntax = match member {
        ClassMember::Method(m) => CallableSyntaxRef::Method(m),
        ClassMember::Getter(g) => CallableSyntaxRef::Getter(g),
        ClassMember::Setter(s) => CallableSyntaxRef::Setter(s),
        ClassMember::Index(i) => CallableSyntaxRef::Index(i),
        ClassMember::Field(_) | ClassMember::Variant(_) => return None,
    };
    semantic_signature_for_syntax(ctx, &CallableOwnerId::Declaration(owner.clone()), syntax, declared_side)
}

pub(crate) fn project_semantic_signature(signature: &CallableSemanticSignature) -> CallableSignature {
    let parameters = signature
        .parameters
        .iter()
        .map(|parameter| {
            let mut projected = CallableParameter::new(parameter.local_name.to_string(), parameter.declared_type.to_knowledge()).with_rest(parameter.rest);
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
