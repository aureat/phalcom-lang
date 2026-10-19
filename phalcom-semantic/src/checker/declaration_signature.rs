//! Canonical declaration-owned callable signature construction.
//!
//! This module is the one source-to-semantic boundary for source callable
//! declarations. It resolves declaration syntax into range-free semantic type
//! facts first; dispatch/member surfaces are projections of that result.

use super::context::CheckingContext;
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::{CallableParameter, CallableParameterContract, CallableSemanticKind, CallableSignature, RestValueContractKnowledge};
use crate::identity::{CallableId, CallableOwnerId, CallableParameterId, DeclarationId, DispatchSide, FieldId};
use crate::signature::{CallableParameterSemantic, CallableSemanticSignature, FieldSemanticSignature, ParameterContract, RestValueContract};
use crate::types::annotation::{TypeFormResolution, TypeFormationOutcome, TypeFormationSite, type_level_binding_for_parameter};
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::id::KindId;
use crate::types::parameter::{TypeParameterOwner, TypeTerm};
use phalcom_ast::ast::{
    BehaviorMember, ClassMember, DelegatedAccessorDef, DelegatedAccessorKind, ExpansionMode, GetterDef, IndexAccessor, IndexMethodDef, MethodDef,
    ParameterAnnotationSyntax, ParameterDef, RestMode, SetterDef,
};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DelegationError {
    FieldNotFound(SourceRange),
    FieldTypeRequired(SourceRange),
    SetterRequiresMutableField(SourceRange),
}

pub(crate) fn delegated_accessor_signatures(
    ctx: &mut CheckingContext<'_>,
    owner: &DeclarationId,
    delegation: &DelegatedAccessorDef,
) -> Result<Vec<CallableSemanticSignature>, DelegationError> {
    let Some(field) = ctx.resolve_field_signature(owner, DispatchSide::Instance, &delegation.target_field) else {
        return Err(DelegationError::FieldNotFound(delegation.target_range));
    };
    if !field.declared_type.is_known() {
        return Err(DelegationError::FieldTypeRequired(delegation.target_range));
    }
    let field_type = field.declared_type.clone();
    if matches!(delegation.kind, DelegatedAccessorKind::Setter | DelegatedAccessorKind::ReadWrite) && !field.mutable {
        return Err(DelegationError::SetterRequiresMutableField(delegation.range));
    }

    let source = crate::diagnostic::SemanticSourceSpan::new(ctx.current_module.clone(), delegation.range);
    let mut signatures = Vec::with_capacity(if delegation.kind == DelegatedAccessorKind::ReadWrite { 2 } else { 1 });

    let make_signature = |ctx: &mut CheckingContext<'_>, setter: bool| {
        let selector = if setter {
            Selector::setter(&delegation.name).expect("parser guarantees a valid delegated setter selector")
        } else {
            Selector::getter(&delegation.name).expect("parser guarantees a valid delegated getter selector")
        };
        let callable = CallableId::new(CallableOwnerId::Declaration(owner.clone()), selector.clone(), DispatchSide::Instance);
        let parameters = if setter {
            vec![CallableParameterSemantic::new(CallableParameterId::new(callable.clone(), 0), "_", field_type.clone()).with_source(source.clone())]
                .into_boxed_slice()
        } else {
            Box::new([])
        };
        let declared_return = if setter {
            DeclaredTypeFact::known(TypeTerm::Canonical(ctx.store.unit()), DeclaredTypeBasis::DeclarationSemantics)
        } else {
            field_type.clone()
        };
        CallableSemanticSignature {
            callable,
            owner: owner.clone(),
            side: DispatchSide::Instance,
            selector,
            generics: None,
            parameters,
            declared_return,
            return_validation: crate::signature::ReturnContractValidation::NotApplicable,
            inferred_return: None,
            source: Some(source.clone()),
            implementation: phalcom_common::modifier::ImplementationKind::Phalcom,
            native_id: None,
            effects: phalcom_native_meta::EffectSpec::Unknown,
            raises: phalcom_native_meta::RaisesSpec::Unknown,
            flow: phalcom_native_meta::ReturnFlowSpec::Value,
            lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
        }
    };

    match delegation.kind {
        DelegatedAccessorKind::Getter => signatures.push(make_signature(ctx, false)),
        DelegatedAccessorKind::Setter => signatures.push(make_signature(ctx, true)),
        DelegatedAccessorKind::ReadWrite => {
            signatures.push(make_signature(ctx, false));
            signatures.push(make_signature(ctx, true));
        }
    }
    Ok(signatures)
}

pub(crate) fn delegated_member_visibility(_name: &str, _attributes: &[phalcom_ast::ast::Attribute]) -> crate::surface::MemberVisibility {
    crate::surface::MemberVisibility::Public
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum CallableSyntaxRef<'a> {
    Method(&'a MethodDef),
    Getter(&'a GetterDef),
    Setter(&'a SetterDef),
    Index(&'a IndexMethodDef),
}

impl<'a> CallableSyntaxRef<'a> {
    pub(crate) fn implementation(&self) -> phalcom_common::modifier::ImplementationKind {
        match self {
            CallableSyntaxRef::Method(method) => method.implementation,
            CallableSyntaxRef::Getter(getter) => getter.implementation,
            CallableSyntaxRef::Setter(setter) => setter.implementation,
            CallableSyntaxRef::Index(index) => index.implementation,
        }
    }

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
            CallableSyntaxRef::Index(index) => index.body.statements().is_some(),
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

impl<'a> From<&'a BehaviorMember> for CallableSyntaxRef<'a> {
    fn from(member: &'a BehaviorMember) -> Self {
        match member {
            BehaviorMember::Method(m) => CallableSyntaxRef::Method(m),
            BehaviorMember::Getter(g) => CallableSyntaxRef::Getter(g),
            BehaviorMember::Setter(s) => CallableSyntaxRef::Setter(s),
            BehaviorMember::Index(i) => CallableSyntaxRef::Index(i),
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
        ClassMember::Field(_) | ClassMember::Variant(_) | ClassMember::Delegation(_) | ClassMember::TypeMember(_) => return None,
    };
    callable_id_for_syntax(&CallableOwnerId::Declaration(owner.clone()), syntax, declared_side)
}

pub(crate) fn callable_type_annotations(member: &ClassMember) -> Vec<&phalcom_ast::ast::TypeAnnotation> {
    let syntax = match member {
        ClassMember::Method(method) => CallableSyntaxRef::Method(method),
        ClassMember::Getter(getter) => CallableSyntaxRef::Getter(getter),
        ClassMember::Setter(setter) => CallableSyntaxRef::Setter(setter),
        ClassMember::Index(index) => CallableSyntaxRef::Index(index),
        ClassMember::Field(_) | ClassMember::Variant(_) | ClassMember::Delegation(_) | ClassMember::TypeMember(_) => return Vec::new(),
    };
    callable_syntax_type_annotations(syntax)
}

pub(crate) fn callable_syntax_type_annotations(syntax: CallableSyntaxRef<'_>) -> Vec<&phalcom_ast::ast::TypeAnnotation> {
    fn add_parameter<'a>(annotations: &mut Vec<&'a phalcom_ast::ast::TypeAnnotation>, parameter: &'a ParameterDef) {
        if let Some(annotation) = parameter.type_annotation() {
            annotations.push(annotation);
        } else if let Some(lane) = parameter.rest_lane_annotation() {
            annotations.push(&lane.operand);
        }
    }

    let mut annotations = Vec::new();
    match syntax {
        CallableSyntaxRef::Method(method) => {
            for parameter in &method.params {
                add_parameter(&mut annotations, parameter);
            }
            annotations.extend(method.return_annotation.as_ref());
        }
        CallableSyntaxRef::Getter(getter) => annotations.extend(getter.return_annotation.as_ref()),
        CallableSyntaxRef::Setter(setter) => {
            add_parameter(&mut annotations, &setter.param);
            annotations.extend(setter.return_annotation.as_ref());
        }
        CallableSyntaxRef::Index(index) => {
            for parameter in &index.params {
                add_parameter(&mut annotations, parameter);
            }
            if let IndexAccessor::Set { value } = &index.accessor {
                add_parameter(&mut annotations, value);
            }
            annotations.extend(index.return_annotation.as_ref());
        }
    }
    annotations
}

pub(crate) fn annotation_has_computed_record_key(annotation: &phalcom_ast::ast::TypeAnnotation) -> bool {
    !crate::static_key::type_record_key_expressions(annotation, 0).is_empty()
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
    let contract = if parameter.rest_mode == RestMode::None {
        let declared = match &parameter.annotation {
            Some(ParameterAnnotationSyntax::Type(ty)) => annotation_fact(ctx, resolver, site, Some(ty), missing),
            Some(ParameterAnnotationSyntax::RestLane(lane)) => {
                ctx.publish_diagnostics(vec![SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::AnnotationUnsupported,
                    "rest lane annotation is only permitted on rest parameters",
                    lane.range,
                )]);
                DeclaredTypeFact::unknown(UnknownReason::SuppressedByInvalidCause)
            }
            None => DeclaredTypeFact::unknown(missing),
        };
        ParameterContract::Fixed(declared)
    } else {
        match &parameter.annotation {
            None => ParameterContract::Rest {
                mode: parameter.rest_mode,
                contract: RestValueContract::Unconstrained,
            },
            Some(ParameterAnnotationSyntax::Type(ann)) => {
                let mut diagnostics = Vec::new();
                let form = crate::types::annotation::resolve_type_form(ctx.store, ctx.declarations, resolver, site, ann, &mut diagnostics);
                let declared = match form {
                    TypeFormResolution::Ready(ty) => {
                        let kind = ctx.store.kind_of(ty);
                        if kind != KindId::TYPE {
                            diagnostics.push(SemanticDiagnostic::error_in(
                                ctx.current_module.clone(),
                                DiagnosticCode::KindExpectedType,
                                format!(
                                    "rest element contract requires a proper Type; `{}` has domain {kind:?}",
                                    ann.origin_symbol_ref().map(|s| s.root.as_str()).unwrap_or("operand")
                                ),
                                ann.range,
                            ));
                            DeclaredTypeFact::unknown(UnknownReason::SuppressedByInvalidCause)
                        } else {
                            DeclaredTypeFact::known(TypeTerm::Canonical(ty), DeclaredTypeBasis::SourceAnnotation)
                        }
                    }
                    _ => DeclaredTypeFact::unknown(missing),
                };
                ctx.publish_diagnostics(diagnostics);
                ParameterContract::Rest {
                    mode: parameter.rest_mode,
                    contract: RestValueContract::Element(declared),
                }
            }
            Some(ParameterAnnotationSyntax::RestLane(lane)) => {
                let expansion_mode = match lane.mode {
                    ExpansionMode::Positional => RestMode::Positional,
                    ExpansionMode::Labeled => RestMode::Labeled,
                    ExpansionMode::Complete => RestMode::Complete,
                };
                if expansion_mode != parameter.rest_mode {
                    ctx.publish_diagnostics(vec![SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::AnnotationUnsupported,
                        "rest parameter mode and lane annotation mode do not match",
                        lane.range,
                    )]);
                    ParameterContract::Rest {
                        mode: parameter.rest_mode,
                        contract: RestValueContract::Invalid(UnknownReason::SuppressedByInvalidCause),
                    }
                } else {
                    let mut diagnostics = Vec::new();
                    let form = crate::types::annotation::resolve_type_form(ctx.store, ctx.declarations, resolver, site, &lane.operand, &mut diagnostics);
                    let lane_id = match form {
                        TypeFormResolution::Ready(operand_ty) => {
                            let projected = match parameter.rest_mode {
                                RestMode::Positional => ctx.store.project_positionals(operand_ty),
                                RestMode::Labeled => ctx.store.project_labeled(operand_ty),
                                RestMode::Complete => ctx.store.project_complete(operand_ty),
                                RestMode::None => unreachable!(),
                            };
                            match projected {
                                Ok(p) => Some(p),
                                Err(err) => {
                                    diagnostics.push(SemanticDiagnostic::error_in(
                                        ctx.current_module.clone(),
                                        DiagnosticCode::AnnotationUnsupported,
                                        format!("failed to project {:?}: {err}", parameter.rest_mode),
                                        lane.range,
                                    ));
                                    None
                                }
                            }
                        }
                        _ => None,
                    };
                    ctx.publish_diagnostics(diagnostics);
                    match lane_id {
                        Some(lane_ty) => ParameterContract::Rest {
                            mode: parameter.rest_mode,
                            contract: RestValueContract::Lane(lane_ty),
                        },
                        None => ParameterContract::Rest {
                            mode: parameter.rest_mode,
                            contract: RestValueContract::Invalid(UnknownReason::SuppressedByInvalidCause),
                        },
                    }
                }
            }
        }
    };

    let mut semantic = CallableParameterSemantic::new(
        CallableParameterId::new(callable.clone(), index as u32),
        parameter.name.clone(),
        contract.declared_type(),
    )
    .with_contract(contract)
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
        implementation: phalcom_common::modifier::ImplementationKind::Phalcom,
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
    implementation: phalcom_common::modifier::ImplementationKind,
) -> crate::signature::ReturnContractValidation {
    if !matches!(implementation, phalcom_common::modifier::ImplementationKind::Phalcom) || declared_return.basis != DeclaredTypeBasis::SourceAnnotation {
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
    let declaration_owner = owner.declaration();
    let is_constructor = matches!(syntax, CallableSyntaxRef::Method(method) if method.is_constructor || method.attributes.iter().any(|attribute| attribute.name == "constructor"));
    let is_class_side = declared_side == DispatchSide::Class;
    let formation_side = if is_constructor || is_class_side {
        DispatchSide::Class
    } else {
        DispatchSide::Instance
    };
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
    semantic_signature_for_syntax_with_resolver(ctx, owner, &declaration_resolver, syntax, declared_side)
}

pub(crate) fn semantic_signature_for_syntax_with_resolver(
    ctx: &mut CheckingContext<'_>,
    owner: &CallableOwnerId,
    declaration_resolver: &dyn crate::types::annotation::TypeResolver,
    syntax: CallableSyntaxRef<'_>,
    declared_side: DispatchSide,
) -> Option<CallableSemanticSignature> {
    let declaration_owner = owner.declaration().clone();
    semantic_signature_for_syntax_with_owner(ctx, owner, &declaration_owner, declaration_resolver, syntax, declared_side)
}

/// Builds a callable signature with an explicit nominal/presentation owner.
///
/// Conformance witnesses use a conformance-owned [`CallableId`], but the
/// existing signature product still carries a declaration owner for receiver
/// and presentation projections. Keeping that input explicit prevents the
/// conformance owner from being mistaken for a lexical declaration owner.
pub(crate) fn semantic_signature_for_syntax_with_owner(
    ctx: &mut CheckingContext<'_>,
    owner: &CallableOwnerId,
    declaration_owner: &DeclarationId,
    declaration_resolver: &dyn crate::types::annotation::TypeResolver,
    syntax: CallableSyntaxRef<'_>,
    declared_side: DispatchSide,
) -> Option<CallableSemanticSignature> {
    let formation_side = if matches!(syntax, CallableSyntaxRef::Method(method) if method.is_constructor || method.attributes.iter().any(|attribute| attribute.name == "constructor"))
    {
        DispatchSide::Class
    } else {
        declared_side
    };
    let formation_site = TypeFormationSite::member(ctx.current_module.clone(), declaration_owner.clone(), formation_side);
    semantic_signature_for_syntax_with_owner_and_site(ctx, owner, declaration_owner, declaration_resolver, &formation_site, syntax, declared_side)
}

fn compose_signature_evidence(
    ctx: &CheckingContext<'_>,
    declaration_owner: &DeclarationId,
    generic_signature: Option<&crate::types::parameter::GenericSignature>,
) -> crate::generic_evidence::GenericEvidenceEnvironment {
    let empty_surfaces = crate::traits::TraitSurfaceTable::default();
    let trait_surfaces = ctx.conformance_semantics.map(|v| v.trait_surfaces).unwrap_or(&empty_surfaces);

    let mut method_env = crate::generic_evidence::GenericEvidenceEnvironment::empty();
    if let Some(generics) = generic_signature {
        if let Ok(env) =
            crate::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(&generics.constraints, generics.owner.clone(), trait_surfaces)
        {
            method_env = env;
        }
    }
    let owner_env = ctx
        .declaration_generic_signature(declaration_owner)
        .and_then(|sig| crate::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(&sig.constraints, sig.owner.clone(), trait_surfaces).ok())
        .unwrap_or_default();
    owner_env.try_compose(&method_env, trait_surfaces).unwrap_or(method_env)
}

pub(crate) fn semantic_signature_for_syntax_with_owner_and_site(
    ctx: &mut CheckingContext<'_>,
    owner: &CallableOwnerId,
    declaration_owner: &DeclarationId,
    declaration_resolver: &dyn crate::types::annotation::TypeResolver,
    formation_site: &TypeFormationSite,
    syntax: CallableSyntaxRef<'_>,
    declared_side: DispatchSide,
) -> Option<CallableSemanticSignature> {
    let callable = callable_id_for_syntax(owner, syntax, declared_side)?;

    let (generics, parameters, declared_return) = match syntax {
        CallableSyntaxRef::Method(method) => {
            let generic_signature = resolve_callable_local_generics(
                ctx,
                declaration_resolver,
                formation_site,
                &callable,
                &method.generic_parameters,
                method.where_clause.as_ref(),
                method.range,
                "method",
            );
            let member_site = formation_site
                .clone()
                .with_generic_evidence(compose_signature_evidence(ctx, declaration_owner, generic_signature.as_ref()));
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
                parent: declaration_resolver,
                type_parameters: method_type_parameters,
            };
            let parameters = method
                .params
                .iter()
                .enumerate()
                .map(|(index, parameter)| parameter_fact(ctx, &callable, index, parameter, &method_resolver, &member_site, UnknownReason::NoTypeEvidence))
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
                    &member_site,
                    method.return_annotation.as_ref(),
                    UnknownReason::UnannotatedDeclaration,
                )
            };
            (generic_signature, parameters, declared_return)
        }
        CallableSyntaxRef::Getter(getter) => {
            let generic_signature = resolve_callable_local_generics(
                ctx,
                declaration_resolver,
                formation_site,
                &callable,
                &getter.generic_parameters,
                getter.where_clause.as_ref(),
                getter.range,
                "getter",
            );
            let member_site = formation_site
                .clone()
                .with_generic_evidence(compose_signature_evidence(ctx, declaration_owner, generic_signature.as_ref()));
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
                parent: declaration_resolver,
                type_parameters: getter_type_parameters,
            };
            (
                generic_signature,
                Vec::<CallableParameterSemantic>::new().into_boxed_slice(),
                annotation_fact(
                    ctx,
                    &getter_resolver,
                    &member_site,
                    getter.return_annotation.as_ref(),
                    UnknownReason::UnannotatedDeclaration,
                ),
            )
        }
        CallableSyntaxRef::Setter(setter) => {
            let generic_signature = resolve_callable_local_generics(
                ctx,
                declaration_resolver,
                formation_site,
                &callable,
                &setter.generic_parameters,
                setter.where_clause.as_ref(),
                setter.range,
                "setter",
            );
            let member_site = formation_site
                .clone()
                .with_generic_evidence(compose_signature_evidence(ctx, declaration_owner, generic_signature.as_ref()));
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
                parent: declaration_resolver,
                type_parameters: setter_type_parameters,
            };
            let parameter = parameter_fact(
                ctx,
                &callable,
                0,
                &setter.param,
                &setter_resolver,
                &member_site,
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
                declaration_resolver,
                formation_site,
                &callable,
                &index.generic_parameters,
                index.where_clause.as_ref(),
                index.range,
                "index member",
            );
            let member_site = formation_site
                .clone()
                .with_generic_evidence(compose_signature_evidence(ctx, declaration_owner, generic_signature.as_ref()));
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
                parent: declaration_resolver,
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
                        &member_site,
                        UnknownReason::NoTypeEvidence,
                    )
                })
                .collect::<Vec<_>>();
            let declared_return = match &index.accessor {
                phalcom_ast::ast::IndexAccessor::Get => annotation_fact(
                    ctx,
                    &index_resolver,
                    &member_site,
                    index.return_annotation.as_ref(),
                    UnknownReason::NoTypeEvidence,
                ),
                phalcom_ast::ast::IndexAccessor::Set { value } => {
                    let value_semantic = parameter_fact(
                        ctx,
                        &callable,
                        parameters.len(),
                        value,
                        &index_resolver,
                        &member_site,
                        UnknownReason::NoTypeEvidence,
                    );
                    let result = value_semantic.declared_type.clone();
                    parameters.push(value_semantic);
                    result
                }
            };
            (generic_signature, parameters.into_boxed_slice(), declared_return)
        }
    };

    let implementation = syntax.implementation();
    let return_validation = initial_return_validation(&declared_return, implementation);

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
        implementation,
        native_id: None,
        effects: phalcom_native_meta::EffectSpec::Unknown,
        raises: phalcom_native_meta::RaisesSpec::Unknown,
        flow: phalcom_native_meta::ReturnFlowSpec::Value,
        lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
    })
}

pub(crate) fn semantic_signature_for_conformance_syntax(
    ctx: &mut CheckingContext<'_>,
    owner: &CallableOwnerId,
    target_owner: &DeclarationId,
    impl_signature: Option<&crate::types::parameter::GenericSignature>,
    syntax: CallableSyntaxRef<'_>,
    declared_side: DispatchSide,
) -> Option<CallableSemanticSignature> {
    let formation_site = TypeFormationSite::member(ctx.current_module.clone(), target_owner.clone(), declared_side);
    semantic_signature_for_conformance_syntax_with_site(ctx, owner, target_owner, impl_signature, &formation_site, syntax, declared_side)
}

pub(crate) fn semantic_signature_for_conformance_syntax_with_site(
    ctx: &mut CheckingContext<'_>,
    owner: &CallableOwnerId,
    target_owner: &DeclarationId,
    impl_signature: Option<&crate::types::parameter::GenericSignature>,
    formation_site: &TypeFormationSite,
    syntax: CallableSyntaxRef<'_>,
    declared_side: DispatchSide,
) -> Option<CallableSemanticSignature> {
    let type_parameters = impl_signature
        .map(|signature| {
            signature
                .parameters
                .iter()
                .map(|&parameter| {
                    let name = ctx.store.type_parameter(parameter).name.to_string();
                    (name, type_level_binding_for_parameter(ctx.store, parameter))
                })
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    let parent = ctx.resolver.clone();
    let resolver = crate::types::annotation::ScopedTypeResolver {
        parent: &parent,
        type_parameters,
    };
    semantic_signature_for_syntax_with_owner_and_site(ctx, owner, target_owner, &resolver, formation_site, syntax, declared_side)
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
        ClassMember::Field(_) | ClassMember::Variant(_) | ClassMember::Delegation(_) | ClassMember::TypeMember(_) => return None,
    };
    semantic_signature_for_syntax(ctx, &CallableOwnerId::Declaration(owner.clone()), syntax, declared_side)
}

pub(crate) fn project_semantic_signature(signature: &CallableSemanticSignature) -> CallableSignature {
    let parameters = signature
        .parameters
        .iter()
        .map(|parameter| {
            let contract_knowledge = match &parameter.contract {
                ParameterContract::Fixed(fact) => CallableParameterContract::Fixed(fact.to_knowledge()),
                ParameterContract::Rest { mode, contract } => CallableParameterContract::Rest {
                    mode: *mode,
                    contract: match contract {
                        RestValueContract::Unconstrained => RestValueContractKnowledge::Unconstrained,
                        RestValueContract::Element(fact) => RestValueContractKnowledge::Element(fact.to_knowledge()),
                        RestValueContract::Lane(lane_id) => RestValueContractKnowledge::Lane(*lane_id),
                        RestValueContract::Invalid(reason) => RestValueContractKnowledge::Invalid(reason.clone()),
                    },
                },
            };
            let mut projected =
                CallableParameter::new(parameter.local_name.to_string(), parameter.declared_type.to_knowledge()).with_contract(contract_knowledge);
            if let Some(label) = &parameter.external_label {
                projected = projected.with_label(label.to_string());
            }
            projected
        })
        .collect::<Vec<_>>();

    let kind = if signature.declared_return.basis == DeclaredTypeBasis::ConstructorSemantics {
        CallableSemanticKind::Constructor
    } else if signature.implementation != phalcom_common::modifier::ImplementationKind::Phalcom {
        CallableSemanticKind::Native
    } else {
        CallableSemanticKind::Ordinary
    };

    let mut projected = CallableSignature::new(signature.selector.clone(), parameters, signature.published_return_knowledge()).with_kind(kind);
    if let Some(generics) = &signature.generics {
        projected = projected.with_generics(generics.clone());
    }
    projected
}

pub(crate) fn project_semantic_signature_with_allocation_budget(
    signature: &CallableSemanticSignature,
    allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
) -> Result<CallableSignature, phalcom_type_meta::allocation::AllocationError> {
    use crate::dispatch::{CallableParameterContract, RestValueContractKnowledge};
    use crate::signature::{ParameterContract, RestValueContract};
    use phalcom_common::selector::{SelectorBase, SelectorSlot};

    fn clone_selector(
        selector: &Selector,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Selector, phalcom_type_meta::allocation::AllocationError> {
        let base = match &selector.base {
            SelectorBase::Named(name) => SelectorBase::Named(allocations.string(name)?),
            SelectorBase::Subscript => SelectorBase::Subscript,
        };
        let mut slots = allocations.vector(selector.slots.len())?;
        for slot in selector.slots.iter() {
            slots.push(match slot {
                SelectorSlot::Positional => SelectorSlot::Positional,
                SelectorSlot::Label(label) => SelectorSlot::Label(allocations.string(label)?),
            });
        }
        Ok(Selector {
            base,
            kind: selector.kind,
            slots: allocations.boxed_slice(slots)?,
        })
    }

    let mut parameters = allocations.vector(signature.parameters.len())?;
    for parameter in signature.parameters.iter() {
        let contract = match &parameter.contract {
            ParameterContract::Fixed(fact) => CallableParameterContract::Fixed(fact.to_knowledge_with_allocation_budget(allocations)?),
            ParameterContract::Rest { mode, contract } => CallableParameterContract::Rest {
                mode: *mode,
                contract: match contract {
                    RestValueContract::Unconstrained => RestValueContractKnowledge::Unconstrained,
                    RestValueContract::Element(fact) => RestValueContractKnowledge::Element(fact.to_knowledge_with_allocation_budget(allocations)?),
                    RestValueContract::Lane(lane) => RestValueContractKnowledge::Lane(*lane),
                    RestValueContract::Invalid(reason) => RestValueContractKnowledge::Invalid(reason.clone_with_allocation_budget(allocations)?),
                },
            },
        };
        let ty = match &contract {
            CallableParameterContract::Fixed(ty) => ty.clone_with_allocation_budget(allocations)?,
            CallableParameterContract::Rest {
                contract: RestValueContractKnowledge::Element(ty),
                ..
            } => ty.clone_with_allocation_budget(allocations)?,
            CallableParameterContract::Rest {
                contract: RestValueContractKnowledge::Invalid(reason),
                ..
            } => crate::types::evidence::TypeKnowledge::Unknown(reason.clone_with_allocation_budget(allocations)?),
            CallableParameterContract::Rest { .. } => crate::types::evidence::TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::NoTypeEvidence),
        };
        parameters.push(CallableParameter {
            external_label: parameter.external_label.as_ref().map(|label| allocations.string(label)).transpose()?,
            local_name: allocations.string(&parameter.local_name)?,
            ty,
            rest: contract.rest_mode(),
            contract,
        });
    }

    let kind = if signature.declared_return.basis == DeclaredTypeBasis::ConstructorSemantics {
        CallableSemanticKind::Constructor
    } else if signature.implementation != phalcom_common::modifier::ImplementationKind::Phalcom {
        CallableSemanticKind::Native
    } else {
        CallableSemanticKind::Ordinary
    };
    let return_type = signature.published_return_knowledge_with_allocation_budget(allocations)?;
    let generics = signature
        .generics
        .as_ref()
        .map(|generics| generics.clone_with_allocation_budget(allocations))
        .transpose()?;

    Ok(CallableSignature {
        selector: clone_selector(&signature.selector, allocations)?,
        parameters,
        return_type,
        generics,
        kind,
    })
}
