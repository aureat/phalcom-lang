//! Canonical declaration-owned callable signature construction.
//!
//! This module is the one source-to-semantic boundary for source callable
//! declarations. It resolves declaration syntax into range-free semantic type
//! facts first; dispatch/member surfaces are projections of that result.

use super::context::CheckingContext;
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
use crate::dispatch::{CallableParameter, CallableSemanticKind, CallableSignature};
use crate::identity::{CallableId, CallableOwnerId, CallableParameterId, DeclarationId, DispatchSide, FieldId};
use crate::signature::{CallableParameterSemantic, CallableSemanticSignature, FieldSemanticSignature};
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::parameter::TypeParameterOwner;
use phalcom_ast::ast::{ClassMember, EnumBehaviorMember, GetterDef, IndexMethodDef, MethodDef, ParameterDef, SetterDef};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};

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

pub(crate) fn callable_id_for_syntax(
    owner: &CallableOwnerId,
    syntax: CallableSyntaxRef<'_>,
    declared_side: DispatchSide,
) -> Option<CallableId> {
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
        .with_rest(parameter.rest_mode)
        .with_source(crate::diagnostic::SemanticSourceSpan::new(ctx.current_module.clone(), parameter.name_range));
    if let Some(label) = &parameter.label {
        semantic = semantic.with_label(label.clone());
    }
    semantic
}

/// Canonical declaration for the root `Class.new()` allocator behavior.
///
/// The member is inherited by class objects through ordinary instance-side
/// dispatch on `Class`; its constructor result is receiver-specialized `Self`.
/// Standalone checker contexts project this declaration into dispatch, while
/// workspace sessions also retain it in `CallableSignatureTable`.
pub(crate) fn canonical_core_class_new_signature(store: &mut crate::types::store::TypeStore) -> CallableSemanticSignature {
    let owner = DeclarationId::new(crate::identity::ModuleId::core(), "Class".into());
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
    let declared_type = annotation_fact(ctx, &declaration_resolver, field.annotation.as_ref(), UnknownReason::UnannotatedDeclaration);
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

    let declaration_type_parameters = ctx
        .declaration_generic_signature(declaration_owner)
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

    let (generics, parameters, declared_return) = match syntax {
        CallableSyntaxRef::Method(method) => {
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
                    owner: declaration_owner.clone(),
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
        CallableSyntaxRef::Getter(getter) => (
            None,
            Vec::<CallableParameterSemantic>::new().into_boxed_slice(),
            annotation_fact(
                ctx,
                &declaration_resolver,
                getter.return_annotation.as_ref(),
                UnknownReason::UnannotatedDeclaration,
            ),
        ),
        CallableSyntaxRef::Setter(setter) => {
            let parameter = parameter_fact(ctx, &callable, 0, &setter.param, &declaration_resolver, UnknownReason::UnannotatedDeclaration);
            let unit = TypeKnowledge::established(ctx.store.unit(), EvidenceOrigin::DeclarationSemantics);
            (
                None,
                vec![parameter].into_boxed_slice(),
                DeclaredTypeFact::from_knowledge_with_basis(&unit, DeclaredTypeBasis::DeclarationSemantics),
            )
        }
        CallableSyntaxRef::Index(index) => {
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
