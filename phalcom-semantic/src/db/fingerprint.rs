//! Deterministic semantic input and product fingerprint hashing (Spec 04.5 / Wave 5).
//!
//! Input fingerprints answer "must this query refresh its stored product?" and
//! therefore include source/provenance data that is observable on the product
//! itself. Product fingerprints answer "did semantic meaning change for
//! dependents?" and deliberately omit incidental source movement for contract
//! products such as interfaces, declaration surfaces, and callable signatures.

use crate::checker::analysis::{AnalysisStatus, BodyExitFacts, CallableAnalysis, CallableAnalysisStatus, FlowStateSummary, NormalReturnFact};
use crate::checker::causal::CausalInvalidity;
use crate::checker::flow::graph::{FlowEdgeKind, FlowGraph, FlowNodeKind};
use crate::checker::incident::{BindingContractSummary, InternalSemanticIncident, InternalSemanticIncidentDetails, InternalSemanticIncidentKind};
use crate::db::key::{InputFingerprint, ProductFingerprint};
use crate::declarations::{DeclarationTypeInfo, GenericSupertypeTemplate, TypeDeclarationShell};
use crate::diagnostic::{DiagnosticFix, SemanticDiagnostic, SemanticSourceSpan};
use crate::identity::{CallableId, DeclarationId, ModuleId};
use crate::signature::{CallableSemanticSignature, FieldSemanticSignature, ReturnContractValidation};
use crate::source::ParsedModuleUnit;
use crate::surface::DeclarationSurface;
use crate::type_alias::TypeAliasInfo;
use crate::types::denotation::SemanticDenotation;
use crate::types::evidence::TypeKnowledge;
use crate::types::id::TypeId;
use crate::types::outcome::{BlockReason, BudgetReport};
use crate::types::parameter::{GenericBinderHeader, GenericSignature};
use crate::types::store::TypeStore;
use phalcom_ast::ast::{ClassMember, IndexAccessor, ParameterDef, RestMode, Statement};
use phalcom_common::range::SourceRange;
use phalcom_modules::graph::{ReferenceKind, RuntimeDependencyReason, SemanticEdgeKind};
use phalcom_modules::interface::{LinkedModuleInterface, UnlinkedModuleInterface};
use phalcom_modules::linker::{LinkedProgram, LinkedReadSpec};
use phalcom_modules::manifest::{DependencySpec, ValidatedProjectManifest};
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::ModuleKind;
use std::collections::{BTreeMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub(crate) fn finish_input(hasher: DefaultHasher) -> InputFingerprint {
    InputFingerprint::new(hasher.finish())
}

pub(crate) fn finish_product(hasher: DefaultHasher) -> ProductFingerprint {
    ProductFingerprint::new(hasher.finish())
}

pub(crate) fn hash_range(range: SourceRange, hasher: &mut impl Hasher) {
    range.start.hash(hasher);
    range.end.hash(hasher);
}

/// Hashes parser output while excluding source-location metadata.
///
/// Callable semantic input is stable across trivia edits that move a body in
/// the file. The AST's derived debug form is otherwise a compact structural
/// representation of the parsed body, and its only source-position values are
/// `CopyRange` records. Literal values, selectors, bindings, and all other
/// semantic syntax remain part of the hash.
fn hash_debug_without_source_ranges<T: std::fmt::Debug>(value: &T, hasher: &mut impl Hasher) {
    let rendered = format!("{value:?}");
    let mut normalized = String::with_capacity(rendered.len());
    let mut remaining = rendered.as_str();
    while let Some(start) = remaining.find("CopyRange {") {
        normalized.push_str(&remaining[..start]);
        let Some(close) = remaining[start..].find('}') else {
            normalized.push_str(&remaining[start..]);
            remaining = "";
            break;
        };
        normalized.push_str("CopyRange");
        remaining = &remaining[start + close + 1..];
    }
    normalized.push_str(remaining);
    normalized.hash(hasher);
}

fn hash_module_kind(kind: ModuleKind, hasher: &mut impl Hasher) {
    match kind {
        ModuleKind::Module => 0u8.hash(hasher),
        ModuleKind::Package => 1u8.hash(hasher),
    }
}

fn hash_rest_mode(rest: RestMode, hasher: &mut impl Hasher) {
    match rest {
        RestMode::None => 0u8.hash(hasher),
        RestMode::Positional => 1u8.hash(hasher),
        RestMode::Labeled => 2u8.hash(hasher),
        RestMode::Complete => 3u8.hash(hasher),
    }
}

fn hash_parameter_contract(contract: &crate::signature::ParameterContract, hasher: &mut impl Hasher) {
    match contract {
        crate::signature::ParameterContract::Fixed(fact) => {
            0u8.hash(hasher);
            fact.hash(hasher);
        }
        crate::signature::ParameterContract::Rest { mode, contract } => {
            1u8.hash(hasher);
            hash_rest_mode(*mode, hasher);
            match contract {
                crate::signature::RestValueContract::Unconstrained => 0u8.hash(hasher),
                crate::signature::RestValueContract::Element(fact) => {
                    1u8.hash(hasher);
                    fact.hash(hasher);
                }
                crate::signature::RestValueContract::Lane(lane_id) => {
                    2u8.hash(hasher);
                    lane_id.hash(hasher);
                }
                crate::signature::RestValueContract::Invalid(reason) => {
                    3u8.hash(hasher);
                    reason.hash(hasher);
                }
            }
        }
    }
}

fn hash_source_region(source: &str, range: SourceRange, hasher: &mut impl Hasher) {
    hash_range(range, hasher);
    match source.get(range.start..range.end) {
        Some(region) => {
            0u8.hash(hasher);
            region.as_bytes().hash(hasher);
        }
        None => {
            1u8.hash(hasher);
            0xBAD0u32.hash(hasher);
        }
    }
}

fn hash_optional_source_region(source: &str, range: Option<SourceRange>, hasher: &mut impl Hasher) {
    match range {
        Some(range) => {
            1u8.hash(hasher);
            hash_source_region(source, range, hasher);
        }
        None => 0u8.hash(hasher),
    }
}

fn hash_dispatch_side(side: crate::identity::DispatchSide, hasher: &mut impl Hasher) {
    match side {
        crate::identity::DispatchSide::Instance => 0u8.hash(hasher),
        crate::identity::DispatchSide::Class => 1u8.hash(hasher),
    }
}

fn hash_parameter_source(source: &str, parameter: &ParameterDef, hasher: &mut impl Hasher) {
    parameter.name.hash(hasher);
    parameter.label.hash(hasher);
    hash_rest_mode(parameter.rest_mode, hasher);
    hash_optional_source_region(source, parameter.annotation.as_ref().map(|annotation| annotation.range()), hasher);
}

fn hash_member_attribute_presence(member: &ClassMember, hasher: &mut impl Hasher) {
    member.attributes().iter().any(|attribute| attribute.name == "class").hash(hasher);
    member.attributes().iter().any(|attribute| attribute.name == "constructor").hash(hasher);
}

fn hash_generic_contract_source(
    source: &str,
    parameters: &[phalcom_ast::ast::GenericParameterSyntax],
    where_clause: Option<&phalcom_ast::ast::WhereClauseSyntax>,
    hasher: &mut impl Hasher,
) {
    parameters.len().hash(hasher);
    for parameter in parameters {
        hash_source_region(source, parameter.range, hasher);
    }
    match where_clause {
        Some(where_clause) => {
            1u8.hash(hasher);
            hash_source_region(source, where_clause.range, hasher);
        }
        None => 0u8.hash(hasher),
    }
}

/// Computes the cheap, source-contract input identity for a declaration surface.
///
/// This deliberately excludes member bodies, defaults, whole declaration ranges,
/// and unrelated attributes so it can run before semantic resolution.
pub fn declaration_surface_source_input_fingerprint(
    unit: &phalcom_modules::source::ParsedModuleUnit,
    declaration: &DeclarationId,
    class_def: &phalcom_ast::ast::ClassDef,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    declaration.hash(&mut hasher);
    class_def.members.len().hash(&mut hasher);

    for member in &class_def.members {
        let side = crate::checker::declaration::member_side(member);
        match member {
            ClassMember::Field(field) => {
                0u8.hash(&mut hasher);
                hash_dispatch_side(side, &mut hasher);
                field.name.hash(&mut hasher);
                hash_optional_source_region(&unit.text, field.annotation.as_ref().map(|annotation| annotation.range), &mut hasher);
            }
            ClassMember::Method(method) => {
                1u8.hash(&mut hasher);
                hash_dispatch_side(side, &mut hasher);
                method.name.hash(&mut hasher);
                (method.is_constructor || method.attributes.iter().any(|attribute| attribute.name == "constructor")).hash(&mut hasher);
                hash_member_attribute_presence(member, &mut hasher);
                method.params.len().hash(&mut hasher);
                for parameter in &method.params {
                    hash_parameter_source(&unit.text, parameter, &mut hasher);
                }
                hash_optional_source_region(&unit.text, method.return_annotation.as_ref().map(|annotation| annotation.range), &mut hasher);
                hash_generic_contract_source(&unit.text, &method.generic_parameters, method.where_clause.as_ref(), &mut hasher);
            }
            ClassMember::Getter(getter) => {
                2u8.hash(&mut hasher);
                hash_dispatch_side(side, &mut hasher);
                getter.name.hash(&mut hasher);
                hash_optional_source_region(&unit.text, getter.return_annotation.as_ref().map(|annotation| annotation.range), &mut hasher);
                hash_generic_contract_source(&unit.text, &getter.generic_parameters, getter.where_clause.as_ref(), &mut hasher);
            }
            ClassMember::Setter(setter) => {
                3u8.hash(&mut hasher);
                hash_dispatch_side(side, &mut hasher);
                setter.name.hash(&mut hasher);
                hash_parameter_source(&unit.text, &setter.param, &mut hasher);
                hash_generic_contract_source(&unit.text, &setter.generic_parameters, setter.where_clause.as_ref(), &mut hasher);
            }
            ClassMember::Index(index) => {
                4u8.hash(&mut hasher);
                hash_dispatch_side(side, &mut hasher);
                hash_generic_contract_source(&unit.text, &index.generic_parameters, index.where_clause.as_ref(), &mut hasher);
                match &index.accessor {
                    IndexAccessor::Get => 0u8.hash(&mut hasher),
                    IndexAccessor::Set { value } => {
                        1u8.hash(&mut hasher);
                        hash_parameter_source(&unit.text, value, &mut hasher);
                    }
                }
                index.params.len().hash(&mut hasher);
                for parameter in &index.params {
                    hash_parameter_source(&unit.text, parameter, &mut hasher);
                }
                if matches!(index.accessor, IndexAccessor::Get) {
                    hash_optional_source_region(&unit.text, index.return_annotation.as_ref().map(|annotation| annotation.range), &mut hasher);
                }
            }
            ClassMember::Delegation(delegation) => {
                6u8.hash(&mut hasher);
                delegation.name.hash(&mut hasher);
                delegation.target_field.hash(&mut hasher);
                delegation.kind.hash(&mut hasher);
            }
            ClassMember::TypeMember(type_member) => {
                7u8.hash(&mut hasher);
                match &type_member.selector {
                    phalcom_ast::ast::TypeMemberSelectorSyntax::Named { name, .. } => {
                        0u8.hash(&mut hasher);
                        name.hash(&mut hasher);
                    }
                    phalcom_ast::ast::TypeMemberSelectorSyntax::Subscript { .. } => {
                        1u8.hash(&mut hasher);
                    }
                }
                type_member.implementation.hash(&mut hasher);
                type_member.parameters.len().hash(&mut hasher);
                for param in &type_member.parameters {
                    param.name.hash(&mut hasher);
                }
                hash_optional_source_region(&unit.text, Some(type_member.range), &mut hasher);
            }
        }
    }
    finish_input(hasher)
}

/// Computes source identity for an enum declaration-surface readiness product.
pub fn declaration_surface_enum_input_fingerprint(
    unit: &phalcom_modules::source::ParsedModuleUnit,
    declaration: &DeclarationId,
    enum_def: &phalcom_ast::ast::EnumDef,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    declaration.hash(&mut hasher);
    hash_optional_source_region(&unit.text, Some(enum_def.range), &mut hasher);
    InputFingerprint::new(hasher.finish())
}

fn hash_unlinked_interface(interface: &UnlinkedModuleInterface, include_ranges: bool, hasher: &mut impl Hasher) {
    phalcom_modules::fingerprint::hash_unlinked_interface(interface, include_ranges, hasher);
}

fn hash_linked_interface(interface: &LinkedModuleInterface, include_ranges: bool, hasher: &mut impl Hasher) {
    phalcom_modules::fingerprint::hash_linked_interface(interface, include_ranges, hasher);
}

pub(crate) fn hash_type_knowledge(knowledge: &TypeKnowledge, include_provenance: bool, hasher: &mut impl Hasher) {
    match knowledge {
        TypeKnowledge::Known(evidence) => {
            0u8.hash(hasher);
            evidence.ty().hash(hasher);
            evidence.status().hash(hasher);
            evidence.origin().hash(hasher);
            if include_provenance {
                evidence.provenance().ranges.len().hash(hasher);
                for range in &evidence.provenance().ranges {
                    hash_range(*range, hasher);
                }
                evidence.provenance().descriptions.hash(hasher);
            }
        }
        TypeKnowledge::Unknown(reason) => {
            1u8.hash(hasher);
            reason.hash(hasher);
        }
        TypeKnowledge::Dynamic(reason) => {
            2u8.hash(hasher);
            reason.hash(hasher);
        }
    }
}

pub(crate) fn hash_generic_signature(signature: &GenericSignature, hasher: &mut impl Hasher) {
    signature.owner.hash(hasher);
    if signature.parameter_kind_shapes.len() == signature.parameters.len()
        && signature.parameter_variances.len() == signature.parameters.len()
        && signature.parameter_modes.len() == signature.parameters.len()
    {
        signature.parameter_kind_shapes.hash(hasher);
        signature.parameter_variances.hash(hasher);
        signature.parameter_modes.hash(hasher);
    } else {
        "legacy-generic-parameter-metadata-absent".hash(hasher);
    }
    let mut sorted_constraints = signature.constraints.to_vec();
    sorted_constraints.sort();
    for constraint in &sorted_constraints {
        constraint.hash(hasher);
    }
}

pub(crate) fn inherent_body_context_fingerprint(context: &crate::impls::InherentBodyContext) -> u64 {
    let mut hasher = DefaultHasher::new();
    context.source_target_head.hash(&mut hasher);
    match &context.generic_signature {
        Some(signature) => {
            1u8.hash(&mut hasher);
            hash_generic_signature(signature, &mut hasher);
        }
        None => 0u8.hash(&mut hasher),
    }
    match &context.covering {
        Some(covering) => {
            1u8.hash(&mut hasher);
            covering.impl_to_decl.len().hash(&mut hasher);
            for (implementation, declaration) in covering.impl_to_decl.iter() {
                implementation.hash(&mut hasher);
                declaration.hash(&mut hasher);
            }
            covering.decl_to_impl.len().hash(&mut hasher);
            for (declaration, implementation) in covering.decl_to_impl.iter() {
                declaration.hash(&mut hasher);
                implementation.hash(&mut hasher);
            }
        }
        None => 0u8.hash(&mut hasher),
    }
    hasher.finish()
}

/// Fingerprints body-generation authority without relying on source syntax.
pub(crate) fn callable_definition_body_provenance_fingerprint(provenance: &crate::impls::CallableDefinitionBodyProvenance) -> u64 {
    let mut hasher = DefaultHasher::new();
    match provenance {
        crate::impls::CallableDefinitionBodyProvenance::SourceBody => 0u8.hash(&mut hasher),
        crate::impls::CallableDefinitionBodyProvenance::GeneratedDelegation { target_field } => {
            1u8.hash(&mut hasher);
            target_field.hash(&mut hasher);
        }
    }
    hasher.finish()
}

fn hash_generic_binder_header(header: &GenericBinderHeader, hasher: &mut impl Hasher) {
    header.owner.hash(hasher);
    header.parameter_kinds.hash(hasher);
    header.parameter_variances.hash(hasher);
    header.parameter_modes.hash(hasher);
}

fn hash_generic_supertype_template(template: &GenericSupertypeTemplate, hasher: &mut impl Hasher) {
    template.declaration.hash(hasher);
    match &template.structural_form {
        Some(structural_form) => structural_form.hash(hasher),
        None => "legacy-supertype-structural-metadata-absent".hash(hasher),
    }
}

fn hash_declaration_type_info(info: &DeclarationTypeInfo, hasher: &mut impl Hasher) {
    info.declaration.hash(hasher);
    info.kind.hash(hasher);
    info.binding.hash(hasher);
    match &info.generic_binder_header {
        Some(header) => {
            1u8.hash(hasher);
            hash_generic_binder_header(header, hasher);
        }
        None => 0u8.hash(hasher),
    }
    match &info.generic_signature {
        Some(signature) => {
            1u8.hash(hasher);
            hash_generic_signature(signature, hasher);
        }
        None => 0u8.hash(hasher),
    }
    match &info.supertype_template {
        Some(template) => {
            1u8.hash(hasher);
            hash_generic_supertype_template(template, hasher);
        }
        None => 0u8.hash(hasher),
    }
}

fn hash_type_alias_info(info: &TypeAliasInfo, hasher: &mut impl Hasher) {
    info.declaration.hash(hasher);
    info.kind_shape.hash(hasher);
    match &info.generic_signature {
        Some(signature) => {
            1u8.hash(hasher);
            hash_generic_signature(signature, hasher);
        }
        None => 0u8.hash(hasher),
    }
    info.structural_form.hash(hasher);
    let mut dependencies = info.dependencies.iter().collect::<Vec<_>>();
    dependencies.sort();
    dependencies.hash(hasher);
}

fn hash_type_declaration_shell(shell: &TypeDeclarationShell, hasher: &mut impl Hasher) {
    match shell {
        TypeDeclarationShell::Nominal(info) => {
            0u8.hash(hasher);
            hash_declaration_type_info(info, hasher);
        }
        TypeDeclarationShell::Alias(info) => {
            1u8.hash(hasher);
            hash_type_alias_info(info, hasher);
        }
    }
}

pub trait DeclarationShellFingerprintInput {
    fn hash_declaration_shell(&self, hasher: &mut impl Hasher);
}

impl DeclarationShellFingerprintInput for DeclarationTypeInfo {
    fn hash_declaration_shell(&self, hasher: &mut impl Hasher) {
        hash_declaration_type_info(self, hasher);
    }
}

impl DeclarationShellFingerprintInput for TypeDeclarationShell {
    fn hash_declaration_shell(&self, hasher: &mut impl Hasher) {
        hash_type_declaration_shell(self, hasher);
    }
}

fn hash_type_knowledge_option(value: &Option<TypeKnowledge>, include_provenance: bool, hasher: &mut impl Hasher) {
    match value {
        Some(knowledge) => {
            1u8.hash(hasher);
            hash_type_knowledge(knowledge, include_provenance, hasher);
        }
        None => 0u8.hash(hasher),
    }
}

fn hash_dispatch_callable_signature(signature: &crate::dispatch::CallableSignature, include_provenance: bool, hasher: &mut impl Hasher) {
    signature.selector.hash(hasher);
    match signature.kind {
        crate::dispatch::CallableSemanticKind::Ordinary => 0u8.hash(hasher),
        crate::dispatch::CallableSemanticKind::Constructor => 1u8.hash(hasher),
        crate::dispatch::CallableSemanticKind::Native => 2u8.hash(hasher),
    }
    signature.parameters.len().hash(hasher);
    for parameter in &signature.parameters {
        parameter.external_label.hash(hasher);
        parameter.local_name.hash(hasher);
        parameter.rest.hash(hasher);
        hash_type_knowledge(&parameter.ty, include_provenance, hasher);
    }
    hash_type_knowledge(&signature.return_type, include_provenance, hasher);
    match &signature.generics {
        Some(generics) => {
            1u8.hash(hasher);
            hash_generic_signature(generics, hasher);
        }
        None => 0u8.hash(hasher),
    }
}

fn hash_member_surface(surface: &crate::surface::MemberSurface, include_provenance: bool, hasher: &mut impl Hasher) {
    let mut fields = surface.fields.iter().collect::<Vec<_>>();
    fields.sort_by_key(|(left, _)| *left);
    fields.len().hash(hasher);
    for (name, knowledge) in fields {
        name.hash(hasher);
        hash_type_knowledge(knowledge, include_provenance, hasher);
        surface.field_visibility.get(name).copied().unwrap_or_default().hash(hasher);
    }

    let mut callables = surface.callable_signatures.iter().collect::<Vec<_>>();
    callables.sort_by_key(|(left, _)| *left);
    callables.len().hash(hasher);
    for (selector, signature) in callables {
        selector.hash(hasher);
        hash_dispatch_callable_signature(signature, include_provenance, hasher);
        surface.callable_visibility.get(selector).copied().unwrap_or_default().hash(hasher);
    }
}

fn hash_declaration_surface(surface: &DeclarationSurface, include_provenance: bool, hasher: &mut impl Hasher) {
    surface.id.hash(hasher);
    0u8.hash(hasher);
    hash_member_surface(&surface.instance, include_provenance, hasher);
    1u8.hash(hasher);
    hash_member_surface(&surface.class, include_provenance, hasher);
}

fn hash_source_span(span: &SemanticSourceSpan, hasher: &mut impl Hasher) {
    span.module.hash(hasher);
    hash_range(span.range, hasher);
}

fn hash_callable_source_span(span: &SemanticSourceSpan, hasher: &mut impl Hasher) {
    span.module.hash(hasher);
    hash_range(span.range, hasher);
}

fn hash_return_contract_validation(validation: ReturnContractValidation, hasher: &mut impl Hasher) {
    match validation {
        ReturnContractValidation::NotApplicable => 0u8.hash(hasher),
        ReturnContractValidation::Unchecked => 1u8.hash(hasher),
        ReturnContractValidation::Satisfied(status) => {
            2u8.hash(hasher);
            status.hash(hasher);
        }
        ReturnContractValidation::Refuted => 3u8.hash(hasher),
        ReturnContractValidation::Blocked => 4u8.hash(hasher),
        ReturnContractValidation::DynamicBoundary => 5u8.hash(hasher),
    }
}

fn hash_callable_semantic_signature(signature: &CallableSemanticSignature, include_source: bool, hasher: &mut impl Hasher) {
    signature.callable.hash(hasher);
    signature.owner.hash(hasher);
    signature.side.hash(hasher);
    signature.selector.hash(hasher);
    match &signature.generics {
        Some(generics) => {
            1u8.hash(hasher);
            hash_generic_signature(generics, hasher);
        }
        None => 0u8.hash(hasher),
    }
    signature.parameters.len().hash(hasher);
    for parameter in &signature.parameters {
        parameter.id.hash(hasher);
        parameter.local_name.hash(hasher);
        parameter.external_label.hash(hasher);
        hash_rest_mode(parameter.rest, hasher);
        parameter.declared_type.hash(hasher);
        hash_parameter_contract(&parameter.contract, hasher);
        if include_source {
            match &parameter.source {
                Some(source) => {
                    1u8.hash(hasher);
                    hash_source_span(source, hasher);
                }
                None => 0u8.hash(hasher),
            }
        }
    }
    signature.declared_return.hash(hasher);
    hash_return_contract_validation(signature.return_validation, hasher);
    hash_type_knowledge_option(&signature.inferred_return, false, hasher);
    if include_source {
        match &signature.source {
            Some(source) => {
                1u8.hash(hasher);
                hash_callable_source_span(source, hasher);
            }
            None => 0u8.hash(hasher),
        }
    }
    signature.implementation.hash(hasher);
    signature.native_id.hash(hasher);
    signature.effects.hash(hasher);
    signature.raises.hash(hasher);
    signature.flow.hash(hasher);
    signature.lifecycle.hash(hasher);
}

fn hash_field_semantic_signature(signature: &FieldSemanticSignature, include_source: bool, hasher: &mut impl Hasher) {
    signature.field.hash(hasher);
    signature.owner.hash(hasher);
    signature.side.hash(hasher);
    signature.name.hash(hasher);
    signature.mutable.hash(hasher);
    signature.declared_type.hash(hasher);
    if include_source {
        match &signature.source {
            Some(source) => {
                1u8.hash(hasher);
                hash_source_span(source, hasher);
            }
            None => 0u8.hash(hasher),
        }
    }
}

fn hash_budget_report(report: &BudgetReport, hasher: &mut impl Hasher) {
    report.kind.hash(hasher);
    report.limit.hash(hasher);
    report.used.hash(hasher);
}

fn hash_block_reason(reason: &BlockReason, hasher: &mut impl Hasher) {
    match reason {
        BlockReason::UnknownType(reason) => {
            0u8.hash(hasher);
            reason.hash(hasher);
        }
        BlockReason::UnresolvedDependency(key) => {
            1u8.hash(hasher);
            key.hash(hasher);
        }
        BlockReason::InvalidAnnotation(code) => {
            2u8.hash(hasher);
            code.hash(hasher);
        }
        BlockReason::RecursiveFixpoint => 3u8.hash(hasher),
        BlockReason::InferenceDidNotConverge => 8u8.hash(hasher),
        BlockReason::OpaqueNative(name) => {
            4u8.hash(hasher);
            name.hash(hasher);
        }
        BlockReason::ReflectionBoundary => 5u8.hash(hasher),
        BlockReason::UnavailableSemanticOwner(owner) => {
            9u8.hash(hasher);
            owner.hash(hasher);
        }
        BlockReason::BudgetExceeded(report) => {
            6u8.hash(hasher);
            hash_budget_report(report, hasher);
        }
        BlockReason::SuppressedDependency => 7u8.hash(hasher),
    }
}

fn hash_incident_kind(kind: &InternalSemanticIncidentKind, hasher: &mut impl Hasher) {
    match kind {
        InternalSemanticIncidentKind::FlowInvariantViolation => 0u8.hash(hasher),
        InternalSemanticIncidentKind::RelationInvariantViolation => 1u8.hash(hasher),
        InternalSemanticIncidentKind::InferenceInvariantViolation => 2u8.hash(hasher),
        InternalSemanticIncidentKind::IdentityInvariantViolation => 3u8.hash(hasher),
        InternalSemanticIncidentKind::DatabaseInvariantViolation => 4u8.hash(hasher),
    }
}

fn hash_contract_summary(summary: &BindingContractSummary, hasher: &mut impl Hasher) {
    summary.ty.hash(hasher);
    match summary.origin {
        None => 0u8.hash(hasher),
        Some(crate::checker::binding::BindingContractOrigin::SourceAnnotation) => 1u8.hash(hasher),
        Some(crate::checker::binding::BindingContractOrigin::InferredInitializer) => 2u8.hash(hasher),
        Some(crate::checker::binding::BindingContractOrigin::CallableParameter) => 3u8.hash(hasher),
        Some(crate::checker::binding::BindingContractOrigin::ContextualBlockParameter) => 4u8.hash(hasher),
        Some(crate::checker::binding::BindingContractOrigin::PatternBinding) => 5u8.hash(hasher),
        Some(crate::checker::binding::BindingContractOrigin::StaticParameter) => 6u8.hash(hasher),
    }
}

fn hash_internal_incident_shape(incident: &InternalSemanticIncident, hasher: &mut impl Hasher) {
    hash_incident_kind(&incident.kind, hasher);
    match &incident.details {
        InternalSemanticIncidentDetails::DivergentBindingContract { binding, left, right } => {
            0u8.hash(hasher);
            binding.hash(hasher);
            hash_contract_summary(left, hasher);
            hash_contract_summary(right, hasher);
        }
        InternalSemanticIncidentDetails::DivergentMutability { binding, left, right } => {
            1u8.hash(hasher);
            binding.hash(hasher);
            left.hash(hasher);
            right.hash(hasher);
        }
        InternalSemanticIncidentDetails::DivergentFieldContract { field, left, right } => {
            2u8.hash(hasher);
            field.hash(hasher);
            hash_type_knowledge(left, false, hasher);
            hash_type_knowledge(right, false, hasher);
        }
        InternalSemanticIncidentDetails::Message { message } => {
            3u8.hash(hasher);
            message.hash(hasher);
        }
    }
}

fn hash_analysis_status(status: &AnalysisStatus, incidents: &[InternalSemanticIncident], hasher: &mut impl Hasher) {
    match status {
        AnalysisStatus::Ready => 0u8.hash(hasher),
        AnalysisStatus::Invalid(cause) => {
            1u8.hash(hasher);
            let _ = cause;
        }
        AnalysisStatus::Suppressed(cause) => match cause {
            crate::checker::causal::SuppressionCause::One(_) => 2u8.hash(hasher),
            crate::checker::causal::SuppressionCause::Multiple => 3u8.hash(hasher),
        },
        AnalysisStatus::Blocked(reason) => {
            4u8.hash(hasher);
            hash_block_reason(reason, hasher);
        }
        AnalysisStatus::DynamicBoundary(reason) => {
            5u8.hash(hasher);
            reason.hash(hasher);
        }
        AnalysisStatus::Cancelled => 6u8.hash(hasher),
        AnalysisStatus::BudgetExceeded(report) => {
            7u8.hash(hasher);
            hash_budget_report(report, hasher);
        }
        AnalysisStatus::InternalFailure(incident) => {
            8u8.hash(hasher);
            if let Some(record) = incidents.iter().find(|record| record.id == *incident) {
                hash_internal_incident_shape(record, hasher);
            } else {
                0xBAD1u16.hash(hasher);
            }
        }
    }
}

pub(crate) fn hash_causal_invalidity(causal: crate::checker::causal::CausalInvalidity, hasher: &mut impl Hasher) {
    match causal {
        crate::checker::causal::CausalInvalidity::Clean => 0u8.hash(hasher),
        crate::checker::causal::CausalInvalidity::One(_) => 1u8.hash(hasher),
        crate::checker::causal::CausalInvalidity::Multiple => 2u8.hash(hasher),
    }
}

pub(crate) fn hash_denotation(denotation: &Option<SemanticDenotation>, hasher: &mut impl Hasher) {
    match denotation {
        None => 0u8.hash(hasher),
        Some(SemanticDenotation::TypeForm(ty)) => {
            1u8.hash(hasher);
            ty.hash(hasher);
        }
        Some(SemanticDenotation::Kind(kind)) => {
            2u8.hash(hasher);
            kind.hash(hasher);
        }
        Some(SemanticDenotation::AssociatedValue(assoc)) => {
            3u8.hash(hasher);
            match &**assoc {
                crate::types::denotation::AssociatedValueDenotation::Exact {
                    owner_form,
                    lookup_owner,
                    member,
                    target,
                } => {
                    0u8.hash(hasher);
                    owner_form.hash(hasher);
                    lookup_owner.hash(hasher);
                    member.hash(hasher);
                    target.hash(hasher);
                }
                crate::types::denotation::AssociatedValueDenotation::Family {
                    owner_form,
                    lookup_owner,
                    family,
                    members,
                } => {
                    1u8.hash(hasher);
                    owner_form.hash(hasher);
                    lookup_owner.hash(hasher);
                    family.hash(hasher);
                    for m in members.iter() {
                        m.hash(hasher);
                    }
                }
            }
        }
        Some(SemanticDenotation::BehavioralFamily(family)) => {
            4u8.hash(hasher);
            family.receiver_type.hash(hasher);
            family.receiver_application.hash(hasher);
            family.spec.hash(hasher);
            for member in family.members.iter() {
                member.hash(hasher);
            }
        }
        Some(SemanticDenotation::StaticDomain(domain)) => {
            5u8.hash(hasher);
            domain.hash(hasher);
        }
        Some(SemanticDenotation::StaticTerm(term)) => {
            6u8.hash(hasher);
            term.hash(hasher);
        }
        Some(SemanticDenotation::StaticApplication(application)) => {
            7u8.hash(hasher);
            application.hash(hasher);
        }
    }
}

pub(crate) fn hash_binding_contract(contract: &Option<crate::checker::binding::BindingContract>, hasher: &mut impl Hasher) {
    let Some(contract) = contract else {
        0u8.hash(hasher);
        return;
    };
    1u8.hash(hasher);
    contract.ty.hash(hasher);
    match contract.origin {
        crate::checker::binding::BindingContractOrigin::SourceAnnotation => 0u8.hash(hasher),
        crate::checker::binding::BindingContractOrigin::InferredInitializer => 1u8.hash(hasher),
        crate::checker::binding::BindingContractOrigin::CallableParameter => 2u8.hash(hasher),
        crate::checker::binding::BindingContractOrigin::ContextualBlockParameter => 3u8.hash(hasher),
        crate::checker::binding::BindingContractOrigin::PatternBinding => 4u8.hash(hasher),
        crate::checker::binding::BindingContractOrigin::StaticParameter => 5u8.hash(hasher),
    }
}

pub(crate) fn hash_binding_consistency(consistency: &crate::checker::binding::BindingConsistency, hasher: &mut impl Hasher) {
    match consistency {
        crate::checker::binding::BindingConsistency::Unconstrained => 0u8.hash(hasher),
        crate::checker::binding::BindingConsistency::Validated => 1u8.hash(hasher),
        crate::checker::binding::BindingConsistency::Assumed { basis } => {
            2u8.hash(hasher);
            match basis {
                crate::checker::binding::AssumptionBasis::MissingValueEvidence(reason) => {
                    0u8.hash(hasher);
                    reason.hash(hasher);
                }
                crate::checker::binding::AssumptionBasis::CallableParameterContract => 1u8.hash(hasher),
                crate::checker::binding::AssumptionBasis::ContextualParameterContract => 2u8.hash(hasher),
                crate::checker::binding::AssumptionBasis::DerivedEvidence(origin) => {
                    3u8.hash(hasher);
                    origin.hash(hasher);
                }
            }
        }
        crate::checker::binding::BindingConsistency::Refuted { actual, expected, reason } => {
            3u8.hash(hasher);
            actual.hash(hasher);
            expected.hash(hasher);
            match reason {
                crate::types::relation::RefutationReason::IncompatibleNominal => 0u8.hash(hasher),
                crate::types::relation::RefutationReason::TypeMismatch => 1u8.hash(hasher),
                crate::types::relation::RefutationReason::UnionMemberMismatch => 2u8.hash(hasher),
            }
        }
        crate::checker::binding::BindingConsistency::DynamicBoundary { obligation } => {
            4u8.hash(hasher);
            obligation.reason.hash(hasher);
        }
        crate::checker::binding::BindingConsistency::Blocked(reason) => {
            5u8.hash(hasher);
            hash_block_reason(reason, hasher);
        }
        crate::checker::binding::BindingConsistency::Cancelled => 6u8.hash(hasher),
        crate::checker::binding::BindingConsistency::BudgetExceeded(report) => {
            7u8.hash(hasher);
            report.used.hash(hasher);
            report.limit.hash(hasher);
        }
        crate::checker::binding::BindingConsistency::InternalFailure(message) => {
            8u8.hash(hasher);
            message.hash(hasher);
        }
    }
}

fn hash_flow_node_kind(kind: &FlowNodeKind, hasher: &mut impl Hasher) {
    match kind {
        FlowNodeKind::Entry => 0u8.hash(hasher),
        FlowNodeKind::Exit => 1u8.hash(hasher),
        FlowNodeKind::Statement(index) => {
            2u8.hash(hasher);
            index.hash(hasher);
        }
        FlowNodeKind::BranchCondition => 3u8.hash(hasher),
        FlowNodeKind::LoopHeader => 4u8.hash(hasher),
        FlowNodeKind::Join => 5u8.hash(hasher),
        FlowNodeKind::Throw => 6u8.hash(hasher),
        FlowNodeKind::Unreachable => 7u8.hash(hasher),
    }
}

fn hash_flow_edge_kind(kind: FlowEdgeKind, hasher: &mut impl Hasher) {
    match kind {
        FlowEdgeKind::Normal => 0u8.hash(hasher),
        FlowEdgeKind::TrueBranch => 1u8.hash(hasher),
        FlowEdgeKind::FalseBranch => 2u8.hash(hasher),
        FlowEdgeKind::BackEdge => 3u8.hash(hasher),
        FlowEdgeKind::Break => 4u8.hash(hasher),
        FlowEdgeKind::Continue => 5u8.hash(hasher),
        FlowEdgeKind::Return => 6u8.hash(hasher),
        FlowEdgeKind::Throw => 7u8.hash(hasher),
        FlowEdgeKind::Unreachable => 8u8.hash(hasher),
    }
}

fn hash_flow_graph_semantics(graph: &FlowGraph, hasher: &mut impl Hasher) {
    graph.nodes.len().hash(hasher);
    for (id, node) in &graph.nodes {
        id.hash(hasher);
        node.id.hash(hasher);
        hash_flow_node_kind(&node.kind, hasher);
        node.predecessors.hash(hasher);
        node.successors.hash(hasher);
    }
    graph.edges.len().hash(hasher);
    for (id, edge) in &graph.edges {
        id.hash(hasher);
        edge.id.hash(hasher);
        edge.source.hash(hasher);
        edge.target.hash(hasher);
        hash_flow_edge_kind(edge.kind, hasher);
        edge.predicate.hash(hasher);
    }
    graph.entry.hash(hasher);
    graph.exits.hash(hasher);
}

fn hash_field_validity(validity: &crate::checker::flow::FieldContractValidity, hasher: &mut impl Hasher) {
    match validity {
        crate::checker::flow::FieldContractValidity::Unchecked => 0u8.hash(hasher),
        crate::checker::flow::FieldContractValidity::Validated => 1u8.hash(hasher),
        crate::checker::flow::FieldContractValidity::Assumed => 2u8.hash(hasher),
        crate::checker::flow::FieldContractValidity::Refuted => 3u8.hash(hasher),
        crate::checker::flow::FieldContractValidity::Blocked(reason) => {
            4u8.hash(hasher);
            hash_block_reason(reason, hasher);
        }
        crate::checker::flow::FieldContractValidity::DynamicBoundary(obligation) => {
            5u8.hash(hasher);
            obligation.reason.hash(hasher);
        }
    }
}

fn hash_flow_summary(summary: &FlowStateSummary, hasher: &mut impl Hasher) {
    summary.bindings.len().hash(hasher);
    for (binding, state) in &summary.bindings {
        binding.hash(hasher);
        hash_type_knowledge(&state.knowledge, false, hasher);
        hash_binding_contract(&state.contract, hasher);
        hash_binding_consistency(&state.consistency, hasher);
        state.mutable.hash(hasher);
        state.initialization.hash(hasher);
    }
    summary.fields.len().hash(hasher);
    for (field, state) in &summary.fields {
        field.hash(hasher);
        hash_type_knowledge(&state.contract, false, hasher);
        hash_type_knowledge(&state.current, false, hasher);
        state.initialization.hash(hasher);
        hash_field_validity(&state.validity, hasher);
        hash_causal_invalidity_shape(&state.causal_invalidity, hasher);
    }
    summary.fact_count.hash(hasher);
}

pub(crate) fn hash_generic_conformance_evidence(evidence: &crate::generic_evidence::GenericConformanceEvidence, hasher: &mut impl Hasher) {
    match evidence {
        crate::generic_evidence::GenericConformanceEvidence::Exact(exact) => {
            0u8.hash(hasher);
            exact.fingerprint.hash(hasher);
            exact.source_impl.hash(hasher);
            exact.exact_target.hash(hasher);
            exact.exact_trait_ref.hash(hasher);
            crate::generic_evidence::GenericConformanceKey::new(
                crate::types::parameter::TypeTerm::Canonical(exact.exact_target),
                exact.exact_trait_ref.clone(),
            )
            .hash(hasher);
            match &exact.trait_header_conformance_proofs {
                None => 0u8.hash(hasher),
                Some(proofs) => {
                    1u8.hash(hasher);
                    proofs.len().hash(hasher);
                    for (key, proof) in proofs {
                        key.hash(hasher);
                        hash_generic_conformance_evidence(proof, hasher);
                    }
                }
            }
        }
        crate::generic_evidence::GenericConformanceEvidence::Assumed(assumed) => {
            1u8.hash(hasher);
            assumed.key.hash(hasher);
            assumed.owner.hash(hasher);
        }
        crate::generic_evidence::GenericConformanceEvidence::Derived(derived) => {
            2u8.hash(hasher);
            match &derived.origin {
                crate::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source } => {
                    0u8.hash(hasher);
                    exact_source.fingerprint.hash(hasher);
                }
                crate::generic_evidence::DerivedConformanceOrigin::GuardedRule {
                    rule,
                    exact_subject,
                    exact_trait_ref,
                } => {
                    1u8.hash(hasher);
                    rule.fingerprint.hash(hasher);
                    exact_subject.hash(hasher);
                    exact_trait_ref.hash(hasher);
                }
            }
            let mut bindings = derived.impl_bindings.bindings().iter().collect::<Vec<_>>();
            bindings.sort_by_key(|(parameter, _)| **parameter);
            bindings.hash(hasher);
            for nested in derived.nested.iter() {
                match nested {
                    crate::generic_evidence::NestedGenericEvidence::Conformance(nested) => {
                        0u8.hash(hasher);
                        hash_generic_conformance_evidence(nested, hasher);
                    }
                    crate::generic_evidence::NestedGenericEvidence::Equivalent => 1u8.hash(hasher),
                    crate::generic_evidence::NestedGenericEvidence::Subtype => 2u8.hash(hasher),
                }
            }
        }
        crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(derived) => {
            3u8.hash(hasher);
            derived.key.hash(hasher);
            derived.kind.hash(hasher);
            match &derived.classification {
                crate::structural::StructuralClassification::Unsupported => 0u8.hash(hasher),
                crate::structural::StructuralClassification::OrderedProduct(form) => {
                    1u8.hash(hasher);
                    match form {
                        crate::structural::OrderedProductForm::Unit => 0u8.hash(hasher),
                        crate::structural::OrderedProductForm::Tuple => 1u8.hash(hasher),
                        crate::structural::OrderedProductForm::TupleFromPacks => 2u8.hash(hasher),
                        crate::structural::OrderedProductForm::Pack => 3u8.hash(hasher),
                        crate::structural::OrderedProductForm::PackFromPacks => 4u8.hash(hasher),
                        crate::structural::OrderedProductForm::Data { owner } => {
                            5u8.hash(hasher);
                            owner.hash(hasher);
                        }
                    }
                }
                crate::structural::StructuralClassification::UnorderedProduct(form) => {
                    2u8.hash(hasher);
                    match form {
                        crate::structural::UnorderedProductForm::Record => 0u8.hash(hasher),
                        crate::structural::UnorderedProductForm::Data { owner } => {
                            1u8.hash(hasher);
                            owner.hash(hasher);
                        }
                    }
                }
                crate::structural::StructuralClassification::Sum { owner } => {
                    3u8.hash(hasher);
                    owner.hash(hasher);
                }
            }
        }
    }
}

pub(crate) fn generic_conformance_evidence_fingerprint(evidence: &crate::generic_evidence::GenericConformanceEvidence) -> ProductFingerprint {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hash_generic_conformance_evidence(evidence, &mut hasher);
    ProductFingerprint::new(hasher.finish())
}

fn hash_conditional_selected_call_capability(capability: Option<&crate::dispatch::ConditionalSelectedCallCapability>, hasher: &mut impl Hasher) {
    let Some(capability) = capability else {
        0u8.hash(hasher);
        return;
    };
    1u8.hash(hasher);
    capability.selection.impl_id.hash(hasher);
    capability.selection.callable.hash(hasher);
    capability.selection.declaring_owner.hash(hasher);
    capability.selection.side.hash(hasher);
    let applicability = &capability.applicability;
    applicability.impl_id.hash(hasher);
    applicability.receiver.hash(hasher);
    applicability.owner_view.hash(hasher);
    let mut bindings = applicability.bindings.iter().collect::<Vec<_>>();
    bindings.sort_by_key(|(parameter, _)| **parameter);
    bindings.hash(hasher);
    let mut environment_bindings = applicability.environment.bindings.iter().collect::<Vec<_>>();
    environment_bindings.sort_by_key(|(parameter, _)| **parameter);
    environment_bindings.hash(hasher);
    let mut row_bindings = applicability.environment.row_bindings.iter().collect::<Vec<_>>();
    row_bindings.sort_by_key(|(parameter, _)| **parameter);
    row_bindings.hash(hasher);
    applicability.environment.self_binding.hash(hasher);
    match &applicability.conformance_provenance {
        None => 0u8.hash(hasher),
        Some(provenance) => {
            1u8.hash(hasher);
            provenance.len().hash(hasher);
            for binding in provenance.iter() {
                binding.callee_conformance.hash(hasher);
                match &binding.source {
                    crate::checker::analysis::GenericCallConformanceSource::Exact { evidence } => {
                        0u8.hash(hasher);
                        hash_generic_conformance_evidence(&crate::generic_evidence::GenericConformanceEvidence::Exact(evidence.clone()), hasher);
                    }
                    crate::checker::analysis::GenericCallConformanceSource::Forwarded { caller_conformance } => {
                        1u8.hash(hasher);
                        caller_conformance.hash(hasher);
                    }
                    crate::checker::analysis::GenericCallConformanceSource::Derived { evidence } => {
                        2u8.hash(hasher);
                        hash_generic_conformance_evidence(&crate::generic_evidence::GenericConformanceEvidence::Derived(evidence.clone()), hasher);
                    }
                    crate::checker::analysis::GenericCallConformanceSource::CompilerDerived { evidence } => {
                        3u8.hash(hasher);
                        hash_generic_conformance_evidence(&crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(evidence.clone()), hasher);
                    }
                }
            }
        }
    }
}

fn hash_iteration_requirement(requirement: &crate::checker::analysis::ForIterationRequirementResolution, hasher: &mut impl Hasher) {
    match requirement {
        crate::checker::analysis::ForIterationRequirementResolution::Evidence {
            conformance,
            evidence,
            requirement,
            signature,
        } => {
            0u8.hash(hasher);
            conformance.hash(hasher);
            hash_generic_conformance_evidence(evidence, hasher);
            requirement.hash(hasher);
            callable_signature_product_fingerprint(signature).raw().hash(hasher);
        }
        crate::checker::analysis::ForIterationRequirementResolution::AbstractRequirement { requirement, signature } => {
            1u8.hash(hasher);
            requirement.hash(hasher);
            callable_signature_product_fingerprint(signature).raw().hash(hasher);
        }
    }
}

fn hash_iteration_lanes(lanes: &crate::checker::analysis::ForIterationLaneResolutionIndex, hasher: &mut impl Hasher) {
    lanes.len().hash(hasher);
    for (expression, lane) in lanes {
        expression.hash(hasher);
        lane.source_expression.hash(hasher);
        lane.source_type.hash(hasher);
        lane.item_type.hash(hasher);
        lane.iterator_type.hash(hasher);
        hash_iteration_requirement(&lane.iterator, hasher);
        hash_iteration_requirement(&lane.next, hasher);
    }
}

fn hash_positional_expansions(expansions: &crate::checker::analysis::PositionalExpansionIterationResolutionIndex, hasher: &mut impl Hasher) {
    expansions.len().hash(hasher);
    for (expression, expansion) in expansions {
        expression.hash(hasher);
        expansion.source_expression.hash(hasher);
        expansion.source_type.hash(hasher);
        expansion.item_type.hash(hasher);
        expansion.iterator_type.hash(hasher);
        hash_iteration_requirement(&expansion.iterator, hasher);
        hash_iteration_requirement(&expansion.next, hasher);
    }
}

fn hash_known_invalid_positional_expansions(expansions: &crate::checker::analysis::KnownInvalidPositionalExpansionIndex, hasher: &mut impl Hasher) {
    expansions.len().hash(hasher);
    for (expression, expansion) in expansions {
        expression.hash(hasher);
        expansion.source_expression.hash(hasher);
        expansion.source_range.hash(hasher);
        expansion.source_type.hash(hasher);
    }
}

fn hash_sequence_literals(literals: &crate::checker::analysis::SequenceLiteralResolutionIndex, hasher: &mut impl Hasher) {
    literals.len().hash(hasher);
    for (expression, literal) in literals {
        expression.hash(hasher);
        literal.source_expression.hash(hasher);
        literal.family.hash(hasher);
        literal.result_type.hash(hasher);
        literal.element_type.hash(hasher);
        literal.extent.hash(hasher);
        literal.construction.hash(hasher);
        literal.exact_expansions.hash(hasher);
    }
}

fn hash_causal_invalidity_shape(causal: &CausalInvalidity, hasher: &mut impl Hasher) {
    match causal {
        CausalInvalidity::Clean => 0u8.hash(hasher),
        CausalInvalidity::One(_) => 1u8.hash(hasher),
        CausalInvalidity::Multiple => 2u8.hash(hasher),
    }
}

fn hash_normal_return_fact(fact: &NormalReturnFact, hasher: &mut impl Hasher) {
    hash_type_knowledge(&fact.knowledge, false, hasher);
    hash_flow_summary(&fact.flow, hasher);
    hash_analysis_status(&fact.status, &[], hasher);
    hash_causal_invalidity_shape(&fact.causal_invalidity, hasher);
    fact.direct_result_dependency.hash(hasher);
}

fn hash_exit_facts(exits: &BodyExitFacts, hasher: &mut impl Hasher) {
    exits.normal_returns.len().hash(hasher);
    for fact in &exits.normal_returns {
        hash_normal_return_fact(fact, hasher);
    }
    exits.throws.len().hash(hasher);
    for summary in &exits.throws {
        hash_flow_summary(summary, hasher);
    }
    exits.unreachable.hash(hasher);
}

fn hash_diagnostic_fix(fix: &DiagnosticFix, hasher: &mut impl Hasher) {
    fix.message.hash(hasher);
    match &fix.replacement {
        Some((range, text)) => {
            1u8.hash(hasher);
            hash_range(*range, hasher);
            text.hash(hasher);
        }
        None => 0u8.hash(hasher),
    }
}

fn hash_semantic_diagnostic(diagnostic: &SemanticDiagnostic, hasher: &mut impl Hasher) {
    diagnostic.code.hash(hasher);
    diagnostic.severity.hash(hasher);
    diagnostic.message.hash(hasher);
    hash_source_span(&diagnostic.primary, hasher);
    hash_range(diagnostic.primary_range, hasher);
    diagnostic.labels.len().hash(hasher);
    for label in &diagnostic.labels {
        hash_source_span(&label.span, hasher);
        hash_range(label.range, hasher);
        label.message.hash(hasher);
    }
    diagnostic.notes.hash(hasher);
    diagnostic.helps.hash(hasher);
    diagnostic.explanations.hash(hasher);
    diagnostic.fixes.len().hash(hasher);
    for fix in &diagnostic.fixes {
        hash_diagnostic_fix(fix, hasher);
    }
    // Cause numbers are snapshot-local allocator identities, not semantic
    // diagnostic content. Preserve ownership shape without hashing the ID.
    diagnostic.root_cause.is_some().hash(hasher);
}

/// Adds diagnostic content to a query's source-sensitive input identity while
/// keeping it out of the semantic product fingerprint and its dependents.
pub fn input_fingerprint_with_diagnostics(input: InputFingerprint, diagnostics: &[SemanticDiagnostic]) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    input.raw().hash(&mut hasher);
    diagnostics.len().hash(&mut hasher);
    for diagnostic in diagnostics {
        hash_semantic_diagnostic(diagnostic, &mut hasher);
    }
    finish_input(hasher)
}

fn hash_callable_analysis_status(status: CallableAnalysisStatus, incidents: &[InternalSemanticIncident], hasher: &mut impl Hasher) {
    match status {
        CallableAnalysisStatus::Complete => 0u8.hash(hasher),
        CallableAnalysisStatus::Partial => 1u8.hash(hasher),
        CallableAnalysisStatus::Blocked => 2u8.hash(hasher),
        CallableAnalysisStatus::Cancelled => 3u8.hash(hasher),
        CallableAnalysisStatus::BudgetExceeded => 4u8.hash(hasher),
        CallableAnalysisStatus::InternalFailure(incident) => {
            5u8.hash(hasher);
            if let Some(record) = incidents.iter().find(|record| record.id == incident) {
                hash_internal_incident_shape(record, hasher);
            } else {
                0xBAD1u16.hash(hasher);
            }
        }
    }
}

fn hash_project_universe_semantics(universe: &ProjectUniverse, hasher: &mut impl Hasher) {
    universe.projects().len().hash(hasher);
    for project in universe.projects() {
        project.id.hash(hasher);
        project.name.hash(hasher);
        project.namespace.hash(hasher);
        project.entry.hash(hasher);
        project.dependencies.len().hash(hasher);
        for (alias, target) in &project.dependencies {
            alias.hash(hasher);
            target.hash(hasher);
        }
        project.import_roots.len().hash(hasher);
        for (root, (target, is_self)) in &project.import_roots {
            root.hash(hasher);
            target.hash(hasher);
            is_self.hash(hasher);
        }
        project.persistent_project.hash(hasher);
    }
}

fn hash_dependency_spec(spec: &DependencySpec, hasher: &mut impl Hasher) {
    match spec {
        DependencySpec::Path { path } => {
            0u8.hash(hasher);
            path.hash(hasher);
        }
        DependencySpec::Package { package, version } => {
            1u8.hash(hasher);
            package.hash(hasher);
            version.hash(hasher);
        }
    }
}

fn hash_validated_manifest(manifest: &ValidatedProjectManifest, hasher: &mut impl Hasher) {
    manifest.name.hash(hasher);
    manifest.raw_name.hash(hasher);
    manifest.namespace.hash(hasher);
    manifest.version.hash(hasher);
    manifest.authors.hash(hasher);
    manifest.description.hash(hasher);
    manifest.license.hash(hasher);
    manifest.homepage.hash(hasher);
    manifest.repository.hash(hasher);
    manifest.source.hash(hasher);
    manifest.entry.hash(hasher);
    manifest.default_entry.hash(hasher);
    manifest.dependencies.len().hash(hasher);
    for (alias, (raw_alias, spec)) in &manifest.dependencies {
        alias.hash(hasher);
        raw_alias.hash(hasher);
        hash_dependency_spec(spec, hasher);
    }
}

fn hash_project_universe_input(universe: &ProjectUniverse, hasher: &mut impl Hasher) {
    hash_project_universe_semantics(universe, hasher);
    for project in universe.projects() {
        project.root_dir.hash(hasher);
        project.source_root.hash(hasher);
        project.source_identity.hash(hasher);
        match &project.manifest {
            Some(manifest) => {
                1u8.hash(hasher);
                hash_validated_manifest(manifest, hasher);
            }
            None => 0u8.hash(hasher),
        }
    }
}

fn hash_reference_kind(kind: ReferenceKind, hasher: &mut impl Hasher) {
    match kind {
        ReferenceKind::WholeModuleImport => 0u8.hash(hasher),
        ReferenceKind::SelectiveImport => 1u8.hash(hasher),
        ReferenceKind::ReExport => 2u8.hash(hasher),
        ReferenceKind::InterfaceOnly => 3u8.hash(hasher),
    }
}

fn hash_semantic_edge_kind(kind: SemanticEdgeKind, hasher: &mut impl Hasher) {
    match kind {
        SemanticEdgeKind::ModuleInterface => 0u8.hash(hasher),
        SemanticEdgeKind::TypeReference => 1u8.hash(hasher),
        SemanticEdgeKind::Superclass => 2u8.hash(hasher),
        SemanticEdgeKind::ProtocolReference => 3u8.hash(hasher),
        SemanticEdgeKind::ConstraintReference => 4u8.hash(hasher),
        SemanticEdgeKind::CallbackSignature => 5u8.hash(hasher),
        SemanticEdgeKind::AdtReference => 6u8.hash(hasher),
    }
}

fn hash_runtime_dependency_reason(reason: RuntimeDependencyReason, hasher: &mut impl Hasher) {
    match reason {
        RuntimeDependencyReason::WholeModuleImport => 0u8.hash(hasher),
        RuntimeDependencyReason::SelectiveValueImport => 1u8.hash(hasher),
        RuntimeDependencyReason::ReExport => 2u8.hash(hasher),
        RuntimeDependencyReason::RuntimeDeclarationReference => 3u8.hash(hasher),
    }
}

fn hash_linked_program(program: &LinkedProgram, hasher: &mut impl Hasher) {
    hash_project_universe_semantics(&program.universe, hasher);
    program.entry.hash(hasher);
    program.modules.len().hash(hasher);
    for (module_id, module) in &program.modules {
        module_id.hash(hasher);
        hash_linked_interface(&module.interface, false, hasher);

        module.bindings.local_globals.len().hash(hasher);
        for (name, binding) in &module.bindings.local_globals {
            name.hash(hasher);
            binding.0.hash(hasher);
        }
        module.bindings.imports.len().hash(hasher);
        for (name, binding) in &module.bindings.imports {
            name.hash(hasher);
            binding.0.hash(hasher);
        }

        module.linked_reads.len().hash(hasher);
        for read in &module.linked_reads {
            match read {
                LinkedReadSpec::Module(module) => {
                    0u8.hash(hasher);
                    module.hash(hasher);
                }
                LinkedReadSpec::Binding(symbol) => {
                    1u8.hash(hasher);
                    symbol.hash(hasher);
                }
            }
        }
        module.runtime_dependencies.hash(hasher);
    }

    let reference_nodes = program.graphs.references.nodes();
    reference_nodes.len().hash(hasher);
    for node in reference_nodes {
        node.hash(hasher);
        let edges = program.graphs.references.edges_from(&node);
        edges.len().hash(hasher);
        for edge in edges {
            edge.from.hash(hasher);
            edge.to.hash(hasher);
            hash_reference_kind(edge.kind, hasher);
        }
    }

    let semantic_nodes = program.graphs.semantics.nodes();
    semantic_nodes.len().hash(hasher);
    for node in semantic_nodes {
        node.hash(hasher);
        let edges = program.graphs.semantics.edges_from(&node);
        edges.len().hash(hasher);
        for edge in edges {
            edge.from.hash(hasher);
            edge.to.hash(hasher);
            hash_semantic_edge_kind(edge.kind, hasher);
        }
    }

    let runtime_nodes = program.graphs.runtime.nodes();
    runtime_nodes.len().hash(hasher);
    for node in runtime_nodes {
        node.hash(hasher);
        let edges = program.graphs.runtime.edges_from(&node);
        edges.len().hash(hasher);
        for edge in edges {
            edge.importer.hash(hasher);
            edge.dependency.hash(hasher);
            hash_runtime_dependency_reason(edge.reason, hasher);
        }
    }
    program.initialization_order.hash(hasher);
}

/// Computes input fingerprint for a parsed module query.
pub fn parsed_module_input_fingerprint(module: &ModuleId, kind: ModuleKind, source: &str) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    module.hash(&mut hasher);
    hash_module_kind(kind, &mut hasher);
    source.as_bytes().hash(&mut hasher);
    finish_input(hasher)
}

/// Computes the source/provenance-sensitive input identity of an unlinked module interface.
pub fn unlinked_interface_input_fingerprint(interface: &UnlinkedModuleInterface) -> InputFingerprint {
    InputFingerprint::new(phalcom_modules::fingerprint::unlinked_interface_input_fingerprint(interface).raw())
}

/// Computes semantic product fingerprint for an unlinked module interface.
pub fn unlinked_interface_product_fingerprint(interface: &UnlinkedModuleInterface) -> ProductFingerprint {
    ProductFingerprint::new(phalcom_modules::fingerprint::interface_fingerprint(interface).raw())
}

/// Computes the source/provenance-sensitive input identity of a linked module interface.
pub fn linked_interface_input_fingerprint(interface: &LinkedModuleInterface) -> InputFingerprint {
    InputFingerprint::new(phalcom_modules::fingerprint::linked_interface_input_fingerprint(interface).raw())
}

/// Computes semantic product fingerprint for a linked module interface.
pub fn linked_interface_product_fingerprint(interface: &LinkedModuleInterface) -> ProductFingerprint {
    ProductFingerprint::new(phalcom_modules::fingerprint::linked_interface_fingerprint(interface).raw())
}

/// Computes the direct semantic input identity of declaration type metadata.
pub fn declaration_shell_input_fingerprint<T: DeclarationShellFingerprintInput>(info: &T) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    info.hash_declaration_shell(&mut hasher);
    finish_input(hasher)
}

/// Computes the semantic product identity of declaration type metadata.
pub fn declaration_shell_product_fingerprint<T: DeclarationShellFingerprintInput>(info: &T) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    info.hash_declaration_shell(&mut hasher);
    finish_product(hasher)
}

/// Computes the provenance-sensitive input identity of a declaration surface.
pub fn declaration_surface_input_fingerprint(surface: &DeclarationSurface) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_declaration_surface(surface, true, &mut hasher);
    finish_input(hasher)
}

/// Computes the complete source-sensitive input identity for a declaration-surface query.
///
/// Diagnostics participate in query input identity because unresolved annotations may keep
/// the same semantic `Unknown` surface while their source ranges or explanatory details move.
/// They deliberately do not participate in the declaration-surface product fingerprint, so
/// diagnostic-only refreshes do not invalidate semantic consumers.
pub fn declaration_surface_query_input_fingerprint(surface: &DeclarationSurface, diagnostics: &[SemanticDiagnostic]) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_declaration_surface(surface, true, &mut hasher);
    diagnostics.len().hash(&mut hasher);
    for diagnostic in diagnostics {
        hash_semantic_diagnostic(diagnostic, &mut hasher);
    }
    finish_input(hasher)
}

/// Computes semantic product fingerprint for a declaration surface.
pub fn declaration_surface_product_fingerprint(surface: &DeclarationSurface) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_declaration_surface(surface, false, &mut hasher);
    finish_product(hasher)
}

/// Computes the source-sensitive input identity of a callable semantic signature.
pub fn callable_signature_input_fingerprint(signature: &CallableSemanticSignature) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_callable_semantic_signature(signature, true, &mut hasher);
    finish_input(hasher)
}

/// Computes semantic product fingerprint for a callable semantic signature.
pub fn callable_signature_product_fingerprint(signature: &CallableSemanticSignature) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_callable_semantic_signature(signature, false, &mut hasher);
    finish_product(hasher)
}

fn callable_definition_fingerprint_value(definition: &crate::impls::EffectiveCallableDefinition) -> u64 {
    let signature_fingerprint = callable_signature_product_fingerprint(&definition.signature);
    let origin_tag = match &definition.origin {
        crate::impls::CallableDefinitionOrigin::PrimaryDeclaration => 0_u64,
        crate::impls::CallableDefinitionOrigin::InherentImpl(impl_id) => {
            let mut hasher = DefaultHasher::new();
            impl_id.hash(&mut hasher);
            hasher.finish()
        }
        crate::impls::CallableDefinitionOrigin::ConformanceWitness(impl_id) => {
            let mut hasher = DefaultHasher::new();
            impl_id.hash(&mut hasher);
            hasher.finish().rotate_left(7)
        }
    };
    let body_context_tag = definition
        .inherent_body_context
        .as_ref()
        .map_or(0, inherent_body_context_fingerprint);
    let body_provenance_tag = callable_definition_body_provenance_fingerprint(&definition.body_provenance);
    signature_fingerprint.raw()
        ^ origin_tag.rotate_left(17)
        ^ (definition.source_member_index as u64).rotate_left(33)
        ^ body_context_tag.rotate_left(11)
        ^ body_provenance_tag.rotate_left(23)
}

/// Computes the current-main input identity of one complete accepted callable
/// definition. The same semantic fields define the product identity because
/// incidental source ranges live in the embedded signature's product hash.
pub fn callable_definition_input_fingerprint(definition: &crate::impls::EffectiveCallableDefinition) -> InputFingerprint {
    InputFingerprint::new(callable_definition_fingerprint_value(definition))
}

pub fn callable_definition_product_fingerprint(definition: &crate::impls::EffectiveCallableDefinition) -> ProductFingerprint {
    ProductFingerprint::new(callable_definition_fingerprint_value(definition))
}

/// Computes the source-sensitive input identity of a canonical field signature.
pub fn field_signature_input_fingerprint(signature: &FieldSemanticSignature) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_field_semantic_signature(signature, true, &mut hasher);
    finish_input(hasher)
}

/// Computes the range-free semantic product identity of a canonical field signature.
pub fn field_signature_product_fingerprint(signature: &FieldSemanticSignature) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_field_semantic_signature(signature, false, &mut hasher);
    finish_product(hasher)
}

/// Computes the input identity of one owner's published constructor paths.
pub fn constructor_initialization_owner_input_fingerprint(product: &crate::db::product::ConstructorInitializationOwnerProduct) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    product.owner.hash(&mut hasher);
    product.paths.hash(&mut hasher);
    finish_input(hasher)
}

/// Computes the semantic identity of one owner's published constructor paths.
pub fn constructor_initialization_owner_product_fingerprint(product: &crate::db::product::ConstructorInitializationOwnerProduct) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    product.owner.hash(&mut hasher);
    product.paths.hash(&mut hasher);
    finish_product(hasher)
}

/// Computes direct input identity for a hierarchy-edge query.
///
/// The source spelling is hashed together with the currently resolved target.
/// The latter is intentionally transitional: until resolved-import/link products
/// expose every name-resolution binding consumed by formal queries, including
/// the resolver outcome here prevents an unchanged superclass spelling from
/// reusing a stale edge after external resolution changes.
pub fn hierarchy_edge_input_fingerprint(
    class_decl: &DeclarationId,
    superclass_syntax: Option<&str>,
    resolved_super_decl: &Option<DeclarationId>,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    class_decl.hash(&mut hasher);
    superclass_syntax.map(str::as_bytes).hash(&mut hasher);
    resolved_super_decl.hash(&mut hasher);
    finish_input(hasher)
}

/// Computes product fingerprint for a hierarchy edge.
///
/// The direct declaration identity alone is insufficient for generic
/// inheritance: `Parent<List<T>>` and `Parent<Set<T>>` resolve to the same
/// superclass declaration while publishing different owner environments to
/// inherited call sites. Keep the source template in the semantic product
/// fingerprint so those callers cannot reuse a stale specialization.
pub fn hierarchy_edge_product_fingerprint(
    class_decl: &DeclarationId,
    super_decl: &Option<DeclarationId>,
    superclass_syntax: Option<&str>,
) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    class_decl.hash(&mut hasher);
    super_decl.hash(&mut hasher);
    superclass_syntax.map(str::as_bytes).hash(&mut hasher);
    finish_product(hasher)
}

/// Computes the source-contract input identity for a trait header. The
/// header product deliberately excludes member bodies; member contract shape
/// belongs to the separate TraitSurface product.
pub fn trait_header_input_fingerprint(header: &crate::traits::TraitHeader) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    header.declaration.hash(&mut hasher);
    header.source.module.hash(&mut hasher);
    header.source.range.hash(&mut hasher);
    match &header.generic_signature {
        Some(signature) => {
            1u8.hash(&mut hasher);
            hash_generic_signature(signature, &mut hasher);
        }
        None => 0u8.hash(&mut hasher),
    }
    finish_input(hasher)
}

/// Computes semantic identity for a trait header without source coordinates.
pub fn trait_header_product_fingerprint(header: &crate::traits::TraitHeader) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    header.declaration.hash(&mut hasher);
    match &header.generic_signature {
        Some(signature) => {
            1u8.hash(&mut hasher);
            hash_generic_signature(signature, &mut hasher);
        }
        None => 0u8.hash(&mut hasher),
    }
    finish_product(hasher)
}

/// Computes the source-sensitive input identity for a complete trait surface.
/// Source spans and diagnostics may refresh the product, but default statement
/// contents are absent by construction and therefore cannot enter this hash.
pub fn trait_surface_input_fingerprint(surface: &crate::traits::TraitSurface) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_trait_surface(surface, true, &mut hasher);
    surface.diagnostics.len().hash(&mut hasher);
    for diagnostic in surface.diagnostics.iter() {
        format!("{diagnostic:?}").hash(&mut hasher);
    }
    finish_input(hasher)
}

/// Computes semantic product identity for a complete trait surface. Full
/// default bodies are intentionally not present in this product.
pub fn trait_surface_product_fingerprint(surface: &crate::traits::TraitSurface) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_trait_surface(surface, false, &mut hasher);
    finish_product(hasher)
}

fn hash_trait_surface(surface: &crate::traits::TraitSurface, include_source: bool, hasher: &mut impl Hasher) {
    surface.declaration.hash(hasher);
    match &surface.generic_signature {
        Some(signature) => {
            1u8.hash(hasher);
            hash_generic_signature(signature, hasher);
        }
        None => 0u8.hash(hasher),
    }
    surface.members.len().hash(hasher);
    for (requirement, member) in &surface.members {
        requirement.hash(hasher);
        member.callable.hash(hasher);
        callable_signature_product_fingerprint(&member.signature).raw().hash(hasher);
        member.visibility.hash(hasher);
        member.default_present.hash(hasher);
        member.default_source.is_some().hash(hasher);
        if include_source {
            member.source.hash(hasher);
            member.default_source.hash(hasher);
        }
    }
    surface.associated_types.len().hash(hasher);
    for (requirement, associated_type) in &surface.associated_types {
        requirement.hash(hasher);
        associated_type.name.hash(hasher);
        associated_type.domain.hash(hasher);
        if include_source {
            associated_type.source.hash(hasher);
        }
    }
}

/// Computes input fingerprint for a callable body query.
pub fn callable_body_input_fingerprint(callable: &CallableId, body: &[Statement], _body_range: SourceRange, store: &TypeStore) -> InputFingerprint {
    callable_body_input_fingerprint_with_owner_generics(callable, body, _body_range, store, None)
}

/// Computes callable-body input identity while including an explicit owner
/// generic signature when the callable owner is not represented by nominal
/// declaration metadata.
pub fn callable_body_input_fingerprint_with_owner_generics(
    callable: &CallableId,
    body: &[Statement],
    _body_range: SourceRange,
    store: &TypeStore,
    owner_generic_signature: Option<&GenericSignature>,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    callable.hash(&mut hasher);
    store.id().hash(&mut hasher);
    match owner_generic_signature {
        Some(signature) => {
            1u8.hash(&mut hasher);
            hash_generic_signature(signature, &mut hasher);
        }
        None => 0u8.hash(&mut hasher),
    }
    // Body input identity is syntax-sensitive but source-location agnostic.
    // Product hashing below never relies on Debug representation; a future
    // parser-owned syntax fingerprint can replace this without affecting
    // dependency semantics.
    hash_debug_without_source_ranges(&body, &mut hasher);
    finish_input(hasher)
}

pub fn callable_body_input_fingerprint_with_fields(
    callable: &CallableId,
    body: &[Statement],
    body_range: SourceRange,
    store: &TypeStore,
    lifecycle: &crate::checker::field_lifecycle::FieldLifecycleTable,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    callable_body_input_fingerprint(callable, body, body_range, store).0.hash(&mut hasher);
    for (field, fact) in lifecycle
        .fields
        .iter()
        .filter(|(field, _)| callable.try_declaration_owner().is_some_and(|owner| &field.owner == owner))
    {
        field.hash(&mut hasher);
        hash_type_knowledge(&fact.contract, true, &mut hasher);
        hash_type_knowledge(&fact.read_knowledge, true, &mut hasher);
        fact.initialization.hash(&mut hasher);
    }
    finish_input(hasher)
}

/// Computes callable-body input identity from direct body-relative inputs.
///
/// Exact semantic dependencies are recorded separately and are validated by
/// the query layer before reuse. Broad workspace and linked-program products
/// are intentionally not hashed here because they are not direct inputs to a
/// callable body.
pub struct CallableBodyFormalInputFingerprint<'a> {
    pub sources: &'a BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub source_resolution_input: InputFingerprint,
    pub linked_component_product: ProductFingerprint,
    pub lifecycle: Option<&'a crate::checker::field_lifecycle::FieldLifecycleTable>,
    pub owner_generic_signature: Option<&'a GenericSignature>,
    pub declared_signature: Option<&'a CallableSemanticSignature>,
    pub instance_self_override: Option<TypeId>,
    pub trait_surface: Option<&'a crate::traits::TraitSurface>,
    pub conformance_semantics: Option<&'a crate::trait_dispatch::ConformanceSemanticView<'a>>,
    pub has_module_value_environment: bool,
}

pub fn callable_body_input_fingerprint_with_formal_inputs(
    callable: &CallableId,
    body: &[Statement],
    body_range: SourceRange,
    store: &TypeStore,
    inputs: CallableBodyFormalInputFingerprint<'_>,
) -> InputFingerprint {
    let CallableBodyFormalInputFingerprint {
        sources: _sources,
        source_resolution_input: _source_resolution_input,
        linked_component_product: _linked_component_product,
        lifecycle,
        owner_generic_signature,
        declared_signature: _declared_signature,
        instance_self_override,
        trait_surface,
        conformance_semantics: _conformance_semantics,
        has_module_value_environment,
    } = inputs;
    let mut hasher = DefaultHasher::new();
    let base = callable_body_input_fingerprint_with_owner_generics(callable, body, body_range, store, owner_generic_signature);
    base.0.hash(&mut hasher);
    let has_body_relative_contract = instance_self_override.is_some() || trait_surface.is_some() || has_module_value_environment;
    let has_lifecycle_inputs = lifecycle.is_some_and(|lifecycle| {
        lifecycle
            .fields
            .keys()
            .any(|field| callable.try_declaration_owner().is_some_and(|owner| &field.owner == owner))
    });
    if !has_body_relative_contract && !has_lifecycle_inputs {
        return base;
    }
    if has_body_relative_contract {
        1u8.hash(&mut hasher);
    }
    has_module_value_environment.hash(&mut hasher);
    instance_self_override.hash(&mut hasher);
    if let Some(surface) = trait_surface {
        1u8.hash(&mut hasher);
        trait_surface_product_fingerprint(surface).raw().hash(&mut hasher);
    } else {
        0u8.hash(&mut hasher);
    }
    if let Some(lifecycle) = lifecycle {
        let lifecycle_owner = callable.try_declaration_owner().cloned();
        for (field, fact) in lifecycle
            .fields
            .iter()
            .filter(|(field, _)| lifecycle_owner.as_ref().is_some_and(|owner| &field.owner == owner))
        {
            field.hash(&mut hasher);
            hash_type_knowledge(&fact.contract, true, &mut hasher);
            hash_type_knowledge(&fact.read_knowledge, true, &mut hasher);
            fact.initialization.hash(&mut hasher);
        }
    }
    finish_input(hasher)
}

/// Computes canonical declaration namespace identity consumed by body
/// resolution. Callable bodies are intentionally excluded so body-only edits
/// can still reuse callers when their semantic products remain unchanged.
pub fn source_resolution_input_fingerprint(interfaces: &BTreeMap<ModuleId, Arc<UnlinkedModuleInterface>>) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    interfaces.len().hash(&mut hasher);
    for (module, interface) in interfaces {
        module.hash(&mut hasher);
        unlinked_interface_product_fingerprint(interface).raw().hash(&mut hasher);
    }
    finish_input(hasher)
}

/// Computes product fingerprint for a callable body analysis result.
pub fn callable_body_product_fingerprint(analysis: &CallableAnalysis) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    analysis.callable.hash(&mut hasher);

    analysis.expressions.len().hash(&mut hasher);
    for (expression_id, expression) in &analysis.expressions {
        expression_id.hash(&mut hasher);
        hash_type_knowledge(&expression.knowledge, false, &mut hasher);
        expression.callable.hash(&mut hasher);
        expression.conditional_dispatch.hash(&mut hasher);
        hash_conditional_selected_call_capability(expression.conditional_capability.as_ref(), &mut hasher);
        hash_trait_dispatch_site(expression.trait_dispatch.as_ref(), &mut hasher);
        hash_contextual_realization(expression.contextual_realization.as_deref(), &mut hasher);
        expression
            .trait_dispatch_candidates
            .as_ref()
            .map(|candidates| {
                candidates
                    .iter()
                    .map(|candidate| {
                        let mut candidate_hasher = DefaultHasher::new();
                        candidate.exact_trait_ref.hash(&mut candidate_hasher);
                        candidate.requirement.hash(&mut candidate_hasher);
                        candidate.source_impl.hash(&mut candidate_hasher);
                        candidate.evidence_fingerprint.raw().hash(&mut candidate_hasher);
                        hash_requirement_selection(&candidate.selection, &mut candidate_hasher);
                        candidate_hasher.finish()
                    })
                    .collect::<Vec<_>>()
            })
            .hash(&mut hasher);
        hash_denotation(&expression.denotation, &mut hasher);
        hash_analysis_status(&expression.status, &analysis.internal_incidents, &mut hasher);
        hash_causal_invalidity(expression.causal_invalidity, &mut hasher);
        expression.binding_read_safety.hash(&mut hasher);
        expression.binding_read.hash(&mut hasher);
    }

    let expansion_field_count = analysis
        .associated_resolutions
        .values()
        .filter(|resolution| {
            matches!(
                &resolution.kind,
                crate::checker::associated::AssociatedResolutionKind::DataExpansionField { .. }
            )
        })
        .count();
    expansion_field_count.hash(&mut hasher);
    for (expression_id, resolution) in analysis.associated_resolutions.iter() {
        if let crate::checker::associated::AssociatedResolutionKind::DataExpansionField { logical_index, value_type } = &resolution.kind {
            expression_id.hash(&mut hasher);
            resolution.owner_form.hash(&mut hasher);
            resolution.lookup_owner.hash(&mut hasher);
            logical_index.hash(&mut hasher);
            value_type.hash(&mut hasher);
        }
    }

    analysis.bindings.len().hash(&mut hasher);
    for (binding_id, binding) in &analysis.bindings {
        binding_id.hash(&mut hasher);
        hash_binding_contract(&binding.contract, &mut hasher);
        hash_type_knowledge(&binding.current, false, &mut hasher);
        hash_denotation(&binding.denotation, &mut hasher);
        hash_binding_consistency(&binding.consistency, &mut hasher);
        hash_causal_invalidity(binding.causal_invalidity, &mut hasher);
        binding.mutable.hash(&mut hasher);
        binding.initialization.hash(&mut hasher);
        binding.version.hash(&mut hasher);
    }

    hash_match_resolutions(&analysis.match_resolutions, &mut hasher);
    hash_pattern_site_resolutions(&analysis.pattern_site_resolutions, &mut hasher);
    hash_iteration_lanes(&analysis.for_iteration_lanes, &mut hasher);
    hash_positional_expansions(&analysis.positional_expansions, &mut hasher);
    hash_known_invalid_positional_expansions(&analysis.known_invalid_positional_expansions, &mut hasher);
    hash_sequence_literals(&analysis.sequence_literals, &mut hasher);
    analysis.callable_reference_resolutions.len().hash(&mut hasher);
    for (expression, resolution) in analysis.callable_reference_resolutions.iter() {
        expression.hash(&mut hasher);
        match &resolution.kind {
            crate::checker::associated::CallableReferenceResolutionKind::BoundFamily { members, .. } => {
                0u8.hash(&mut hasher);
                members.len().hash(&mut hasher);
                for member in members.iter() {
                    member.operation.hash(&mut hasher);
                    member.target.hash(&mut hasher);
                    hash_conditional_selected_call_capability(member.conditional_capability.as_ref(), &mut hasher);
                }
            }
            crate::checker::associated::CallableReferenceResolutionKind::Associated(_) => 1u8.hash(&mut hasher),
        }
    }
    hash_flow_graph_semantics(&analysis.flow_graph, &mut hasher);
    hash_flow_summary(&analysis.entry_flow, &mut hasher);
    hash_exit_facts(&analysis.exits, &mut hasher);

    analysis.internal_incidents.len().hash(&mut hasher);
    for incident in analysis.internal_incidents.iter() {
        hash_internal_incident_shape(incident, &mut hasher);
    }

    analysis.dependencies.hash(&mut hasher);
    analysis.constructor_super_initializer.hash(&mut hasher);
    hash_callable_analysis_status(analysis.status, &analysis.internal_incidents, &mut hasher);
    hash_return_contract_validation(analysis.return_validation, &mut hasher);
    // `dependency_fingerprint` is assigned from this fingerprint after
    // computation. Hashing it would make the product definition recursive.
    finish_product(hasher)
}

fn hash_trait_dispatch_site(site: Option<&crate::trait_dispatch::TraitDispatchSite>, hasher: &mut impl Hasher) {
    match site {
        None => 0u8.hash(hasher),
        Some(crate::trait_dispatch::TraitDispatchSite::AbstractRequirement { requirement }) => {
            1u8.hash(hasher);
            requirement.hash(hasher);
        }
        Some(crate::trait_dispatch::TraitDispatchSite::Evidenced(selection)) => {
            2u8.hash(hasher);
            selection.source_impl.hash(hasher);
            selection.exact_target.hash(hasher);
            selection.exact_trait_ref.hash(hasher);
            selection.requirement.hash(hasher);
            selection.evidence_fingerprint.raw().hash(hasher);
            hash_requirement_selection(&selection.selection, hasher);
            selection.requirement_targets.len().hash(hasher);
            for (requirement, target) in &selection.requirement_targets {
                requirement.hash(hasher);
                hash_requirement_selection(target, hasher);
            }
        }
        Some(crate::trait_dispatch::TraitDispatchSite::Assumed(selection)) => {
            3u8.hash(hasher);
            selection.conformance.hash(hasher);
            selection.requirement.hash(hasher);
        }
    }
}

fn hash_contextual_realization(realization: Option<&crate::checker::ContextualRealization>, hasher: &mut impl Hasher) {
    let Some(realization) = realization else {
        0u8.hash(hasher);
        return;
    };
    1u8.hash(hasher);
    realization.source_expression.hash(hasher);
    realization.source_range.hash(hasher);
    realization.source_type.hash(hasher);
    realization.target_type.hash(hasher);
    match &realization.operation {
        crate::checker::ContextualRealizationOperation::Basis {
            basis_type,
            axis,
            coefficient,
            evaluation,
        } => {
            0u8.hash(hasher);
            basis_type.hash(hasher);
            axis.hash(hasher);
            match evaluation {
                crate::checker::RealizationEvaluationPlan::LiteralCoefficientThenAxis => 0u8.hash(hasher),
                crate::checker::RealizationEvaluationPlan::MultiplicativeIdentityThenSourceBasis => 1u8.hash(hasher),
                crate::checker::RealizationEvaluationPlan::SourceOperands {
                    first,
                    second,
                    coefficient,
                    basis,
                } => {
                    2u8.hash(hasher);
                    first.hash(hasher);
                    second.hash(hasher);
                    coefficient.hash(hasher);
                    basis.hash(hasher);
                }
            }
            match coefficient {
                crate::checker::RealizationCoefficientSource::One { scalar_type, one } => {
                    0u8.hash(hasher);
                    scalar_type.hash(hasher);
                    hash_realization_capability_selection(one, hasher);
                }
                crate::checker::RealizationCoefficientSource::IntegerLiteral { digits, radix } => {
                    1u8.hash(hasher);
                    digits.hash(hasher);
                    radix.hash(hasher);
                }
                crate::checker::RealizationCoefficientSource::FloatLiteral { bits } => {
                    2u8.hash(hasher);
                    bits.hash(hasher);
                }
                crate::checker::RealizationCoefficientSource::Expression { expression, scalar_type } => {
                    3u8.hash(hasher);
                    expression.hash(hasher);
                    scalar_type.hash(hasher);
                }
            }
        }
        crate::checker::ContextualRealizationOperation::RealEmbedding { value, scalar_type } => {
            1u8.hash(hasher);
            value.hash(hasher);
            scalar_type.hash(hasher);
        }
    }
    hash_realization_capability_selection(&realization.realization, hasher);
}

fn hash_realization_capability_selection(selection: &crate::checker::RealizationCapabilitySelection, hasher: &mut impl Hasher) {
    match selection {
        crate::checker::RealizationCapabilitySelection::Evidenced(selection) => {
            0u8.hash(hasher);
            hash_trait_dispatch_site(Some(&crate::trait_dispatch::TraitDispatchSite::Evidenced(selection.clone())), hasher);
        }
        crate::checker::RealizationCapabilitySelection::Assumed(selection) => {
            1u8.hash(hasher);
            selection.conformance.hash(hasher);
            selection.requirement.hash(hasher);
            hash_callable_semantic_signature(&selection.signature, false, hasher);
        }
    }
}

fn hash_requirement_selection(selection: &crate::impls::RequirementSelectionTemplate, hasher: &mut impl Hasher) {
    match selection {
        crate::impls::RequirementSelectionTemplate::ConformanceCallable { callable } => {
            0u8.hash(hasher);
            callable.hash(hasher);
        }
        crate::impls::RequirementSelectionTemplate::InherentCallable {
            callable,
            conditional_impl,
            applicability,
        } => {
            1u8.hash(hasher);
            callable.hash(hasher);
            conditional_impl.hash(hasher);
            applicability
                .as_ref()
                .map(crate::trait_dispatch::inherent_specialization_fingerprint)
                .hash(hasher);
        }
        crate::impls::RequirementSelectionTemplate::ConditionalInherent { candidate, pending, fallback } => {
            2u8.hash(hasher);
            candidate.callable.hash(hasher);
            candidate.owner.hash(hasher);
            candidate.conditional_impl.hash(hasher);
            candidate
                .applicability
                .as_ref()
                .map(crate::trait_dispatch::inherent_specialization_fingerprint)
                .hash(hasher);
            // The retained terminal state is an intermediate proof carrier;
            // the selected conditional specialization/fallback identities
            // below are the published semantic meaning.
            std::mem::discriminant(pending).hash(hasher);
            fallback
                .as_deref()
                .map(|selection| {
                    let mut nested = DefaultHasher::new();
                    hash_requirement_selection(selection, &mut nested);
                    nested.finish()
                })
                .hash(hasher);
        }
        crate::impls::RequirementSelectionTemplate::DataComponent { component, specialized_type } => {
            3u8.hash(hasher);
            component.hash(hasher);
            specialized_type.hash(hasher);
        }
        crate::impls::RequirementSelectionTemplate::TraitDefault { callable } => {
            4u8.hash(hasher);
            callable.hash(hasher);
        }
    }
}

fn hash_match_resolutions(resolutions: &crate::match_semantics::MatchResolutionIndex, hasher: &mut impl Hasher) {
    resolutions.len().hash(hasher);
    for (expr_id, resolution) in resolutions.iter() {
        expr_id.hash(hasher);
        hash_match_resolution(resolution, hasher);
    }
}

fn hash_match_resolution(res: &crate::match_semantics::MatchResolution, hasher: &mut impl Hasher) {
    res.expression.hash(hasher);
    hash_type_knowledge(&res.scrutinee, false, hasher);
    hash_pattern_space_summary(&res.initial_space, hasher);
    res.arms.len().hash(hasher);
    for arm in res.arms.iter() {
        hash_match_arm_resolution(arm, hasher);
    }
    hash_type_knowledge(&res.result, false, hasher);
    hash_exhaustiveness_result(&res.exhaustiveness, hasher);
}

fn hash_match_arm_resolution(arm: &crate::match_semantics::MatchArmResolution, hasher: &mut impl Hasher) {
    arm.arm_index.hash(hasher);
    hash_pattern_resolution(&arm.pattern, hasher);
    hash_pattern_space_summary(&arm.reachable_space, hasher);
    hash_pattern_space_summary(&arm.residual_after, hasher);
    arm.bindings.len().hash(hasher);
    for b in arm.bindings.iter() {
        b.binding.hash(hasher);
        b.name.hash(hasher);
        hash_type_knowledge(&b.knowledge, false, hasher);
        hash_range(b.source, hasher);
    }
    hash_branch_proof_environment(&arm.proof, hasher);
    match arm.usefulness {
        crate::match_semantics::PatternUsefulness::Useful => 0u8.hash(hasher),
        crate::match_semantics::PatternUsefulness::Redundant => 1u8.hash(hasher),
        crate::match_semantics::PatternUsefulness::Impossible => 2u8.hash(hasher),
    }
    hash_type_knowledge(&arm.branch_result, false, hasher);
}

fn hash_pattern_site_resolutions(resolutions: &crate::match_semantics::PatternSiteResolutionIndex, hasher: &mut impl Hasher) {
    resolutions.len().hash(hasher);
    for (key, resolution) in resolutions.iter() {
        match key.kind {
            crate::match_semantics::PatternSiteKind::IfLet => 0u8.hash(hasher),
            crate::match_semantics::PatternSiteKind::WhileLet => 1u8.hash(hasher),
            crate::match_semantics::PatternSiteKind::ForLane => 2u8.hash(hasher),
        }
        hash_range(key.range, hasher);
        hash_pattern_site_resolution(resolution, hasher);
    }
}

fn hash_pattern_site_resolution(res: &crate::match_semantics::PatternSiteResolution, hasher: &mut impl Hasher) {
    match res.kind {
        crate::match_semantics::PatternSiteKind::IfLet => 0u8.hash(hasher),
        crate::match_semantics::PatternSiteKind::WhileLet => 1u8.hash(hasher),
        crate::match_semantics::PatternSiteKind::ForLane => 2u8.hash(hasher),
    }
    hash_range(res.range, hasher);
    hash_type_knowledge(&res.subject, false, hasher);
    hash_pattern_resolution(&res.pattern, hasher);
    res.bindings.len().hash(hasher);
    for b in res.bindings.iter() {
        b.binding.hash(hasher);
        b.name.hash(hasher);
        hash_type_knowledge(&b.knowledge, false, hasher);
        hash_range(b.source, hasher);
    }
    match res.usefulness {
        crate::match_semantics::PatternUsefulness::Useful => 0u8.hash(hasher),
        crate::match_semantics::PatternUsefulness::Redundant => 1u8.hash(hasher),
        crate::match_semantics::PatternUsefulness::Impossible => 2u8.hash(hasher),
    }
}

fn hash_pattern_resolution(pat: &crate::match_semantics::PatternResolution, hasher: &mut impl Hasher) {
    match pat {
        crate::match_semantics::PatternResolution::Wildcard => 0u8.hash(hasher),
        crate::match_semantics::PatternResolution::Binding { binding, name, knowledge } => {
            1u8.hash(hasher);
            binding.hash(hasher);
            name.hash(hasher);
            hash_type_knowledge(knowledge, false, hasher);
        }
        crate::match_semantics::PatternResolution::Variant(v) => {
            2u8.hash(hasher);
            v.owner.hash(hasher);
            v.family.hash(hasher);
            v.owner_candidates.len().hash(hasher);
            for owner in v.owner_candidates.iter() {
                owner.hash(hasher);
            }
            match &v.selector {
                crate::match_semantics::VariantSelectorConstraint::Exact(sel) => {
                    0u8.hash(hasher);
                    sel.hash(hasher);
                }
                crate::match_semantics::VariantSelectorConstraint::Pattern(pat) => {
                    1u8.hash(hasher);
                    pat.hash(hasher);
                }
                crate::match_semantics::VariantSelectorConstraint::WholeFamily => 2u8.hash(hasher),
            }
            v.candidates.len().hash(hasher);
            for c in v.candidates.iter() {
                c.variant.hash(hasher);
                c.exact_case.hash(hasher);
                c.fields.len().hash(hasher);
                for f in c.fields.iter() {
                    f.field.hash(hasher);
                    hash_type_knowledge(&f.field_type, false, hasher);
                    hash_pattern_resolution(&f.child, hasher);
                }
                hash_branch_proof_environment(&c.proof, hasher);
            }
        }
        crate::match_semantics::PatternResolution::Or(or_pat) => {
            3u8.hash(hasher);
            or_pat.alternatives.len().hash(hasher);
            for alt in or_pat.alternatives.iter() {
                hash_pattern_resolution(alt, hasher);
            }
        }
        crate::match_semantics::PatternResolution::Tuple(elems) => {
            4u8.hash(hasher);
            elems.len().hash(hasher);
            for e in elems.iter() {
                hash_pattern_resolution(e, hasher);
            }
        }
        crate::match_semantics::PatternResolution::List(list) => {
            5u8.hash(hasher);
            list.prefix.len().hash(hasher);
            for p in list.prefix.iter() {
                hash_pattern_resolution(p, hasher);
            }
            list.rest.is_some().hash(hasher);
            if let Some(ref r) = list.rest {
                hash_pattern_resolution(r, hasher);
            }
        }
        crate::match_semantics::PatternResolution::Record(fields) => {
            6u8.hash(hasher);
            fields.len().hash(hasher);
            for f in fields.iter() {
                f.label.hash(hasher);
                hash_pattern_resolution(&f.child, hasher);
            }
        }
        crate::match_semantics::PatternResolution::Map(entries) => {
            7u8.hash(hasher);
            entries.len().hash(hasher);
            for e in entries.iter() {
                e.key.hash(hasher);
                hash_pattern_resolution(&e.child, hasher);
            }
        }
        crate::match_semantics::PatternResolution::Char(value) => {
            8u8.hash(hasher);
            (*value as u32).hash(hasher);
        }
    }
}

fn hash_pattern_space_summary(space: &crate::match_semantics::PatternSpaceSummary, hasher: &mut impl Hasher) {
    match space {
        crate::match_semantics::PatternSpaceSummary::Empty => 0u8.hash(hasher),
        crate::match_semantics::PatternSpaceSummary::Blocked(reason) => {
            6u8.hash(hasher);
            hash_block_reason(reason, hasher);
        }
        crate::match_semantics::PatternSpaceSummary::Opaque(ty) => {
            1u8.hash(hasher);
            ty.hash(hasher);
        }
        crate::match_semantics::PatternSpaceSummary::Union(members) => {
            2u8.hash(hasher);
            members.len().hash(hasher);
            for m in members.iter() {
                hash_pattern_space_summary(m, hasher);
            }
        }
        crate::match_semantics::PatternSpaceSummary::Variant { variant, exact_case, fields } => {
            3u8.hash(hasher);
            variant.hash(hasher);
            exact_case.hash(hasher);
            fields.len().hash(hasher);
            for f in fields.iter() {
                hash_pattern_space_summary(f, hasher);
            }
        }
        crate::match_semantics::PatternSpaceSummary::Tuple(elems) => {
            4u8.hash(hasher);
            elems.len().hash(hasher);
            for e in elems.iter() {
                hash_pattern_space_summary(e, hasher);
            }
        }
        crate::match_semantics::PatternSpaceSummary::List => 5u8.hash(hasher),
        crate::match_semantics::PatternSpaceSummary::Char(value) => {
            7u8.hash(hasher);
            (*value as u32).hash(hasher);
        }
    }
}

fn hash_branch_proof_environment(proof: &crate::match_semantics::BranchProofEnvironment, hasher: &mut impl Hasher) {
    proof.bindings.len().hash(hasher);
    for (param, ty) in &proof.bindings {
        param.hash(hasher);
        ty.hash(hasher);
    }
    proof.equalities.len().hash(hasher);
    for eq in proof.equalities.iter() {
        match eq {
            crate::types::TypeConstraint::Equal(a, b) => {
                0u8.hash(hasher);
                a.hash(hasher);
                b.hash(hasher);
            }
            crate::types::TypeConstraint::Subtype(sub, sup) => {
                1u8.hash(hasher);
                sub.hash(hasher);
                sup.hash(hasher);
            }
            crate::types::TypeConstraint::HasMember(ty, sel) => {
                2u8.hash(hasher);
                ty.hash(hasher);
                sel.hash(hasher);
            }
        }
    }
}

fn hash_exhaustiveness_result(res: &crate::match_semantics::ExhaustivenessResult, hasher: &mut impl Hasher) {
    match res {
        crate::match_semantics::ExhaustivenessResult::Proven => 0u8.hash(hasher),
        crate::match_semantics::ExhaustivenessResult::Missing(witnesses) => {
            1u8.hash(hasher);
            witnesses.len().hash(hasher);
            for w in witnesses.iter() {
                hash_coverage_witness(w, hasher);
            }
        }
        crate::match_semantics::ExhaustivenessResult::Blocked(b) => {
            2u8.hash(hasher);
            hash_block_reason(b, hasher);
        }
    }
}

fn hash_coverage_witness(w: &crate::match_semantics::CoverageWitness, hasher: &mut impl Hasher) {
    match w {
        crate::match_semantics::CoverageWitness::Variant { variant, exact_case, fields } => {
            0u8.hash(hasher);
            variant.hash(hasher);
            exact_case.hash(hasher);
            fields.len().hash(hasher);
            for f in fields.iter() {
                hash_coverage_witness(f, hasher);
            }
        }
        crate::match_semantics::CoverageWitness::Tuple(elems) => {
            1u8.hash(hasher);
            elems.len().hash(hasher);
            for e in elems.iter() {
                hash_coverage_witness(e, hasher);
            }
        }
        crate::match_semantics::CoverageWitness::List(elems) => {
            2u8.hash(hasher);
            elems.len().hash(hasher);
            for e in elems.iter() {
                hash_coverage_witness(e, hasher);
            }
        }
        crate::match_semantics::CoverageWitness::Opaque(ty) => {
            3u8.hash(hasher);
            ty.hash(hasher);
        }
        crate::match_semantics::CoverageWitness::Wildcard => 4u8.hash(hasher),
        crate::match_semantics::CoverageWitness::Char(value) => {
            5u8.hash(hasher);
            (*value as u32).hash(hasher);
        }
    }
}

/// Computes product fingerprint for resolved imports.
pub fn resolved_imports_product_fingerprint(product: &crate::module_product::ResolvedImportsProduct) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    product.module.hash(&mut hasher);
    product.imports.len().hash(&mut hasher);
    for (path, target) in &product.imports {
        path.hash(&mut hasher);
        target.hash(&mut hasher);
    }
    product.unresolved_diagnostics.len().hash(&mut hasher);
    for (error, range) in &product.unresolved_diagnostics {
        error.hash(&mut hasher);
        hash_range(*range, &mut hasher);
    }
    finish_product(hasher)
}

/// Computes product fingerprint for module diagnostics.
pub fn module_diagnostics_product_fingerprint(module: &ModuleId, diagnostics: &[SemanticDiagnostic]) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    module.hash(&mut hasher);
    diagnostics.len().hash(&mut hasher);
    for diagnostic in diagnostics {
        hash_semantic_diagnostic(diagnostic, &mut hasher);
    }
    finish_product(hasher)
}

/// Computes the full structural input identity used to link one semantic component.
pub fn semantic_component_input_fingerprint(
    entry: &ModuleId,
    universe: &ProjectUniverse,
    interfaces: &std::collections::BTreeMap<ModuleId, UnlinkedModuleInterface>,
    resolved: &std::collections::BTreeMap<(ModuleId, String), ModuleId>,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    entry.hash(&mut hasher);
    hash_project_universe_input(universe, &mut hasher);
    interfaces.len().hash(&mut hasher);
    for (module, interface) in interfaces {
        module.hash(&mut hasher);
        hash_unlinked_interface(interface, true, &mut hasher);
    }
    resolved.len().hash(&mut hasher);
    for ((importer, path), target) in resolved {
        importer.hash(&mut hasher);
        path.hash(&mut hasher);
        target.hash(&mut hasher);
    }
    finish_input(hasher)
}

/// Computes semantic product fingerprint for a linked component.
pub fn semantic_component_product_fingerprint(program: &LinkedProgram) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_linked_program(program, &mut hasher);
    finish_product(hasher)
}

/// Fingerprints for compiler-owned source/advisory query products.
pub fn source_structure_input_fingerprint(product: &crate::source_index::ModuleSourceIndex) -> InputFingerprint {
    InputFingerprint::new(product.fingerprints().presentation.raw())
}

pub fn source_structure_product_fingerprint(product: &crate::source_index::ModuleSourceIndex) -> ProductFingerprint {
    product.fingerprints().semantic
}

pub fn source_formal_attachment_fingerprint(product: &crate::source_index::CallableSourceAttachment) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    product.callable.hash(&mut hasher);
    for (binding, site) in &product.formal_bindings {
        binding.hash(&mut hasher);
        site.hash(&mut hasher);
    }
    for (expression, site) in &product.formal_expressions {
        expression.hash(&mut hasher);
        site.hash(&mut hasher);
    }
    product.exact_targets.hash(&mut hasher);
    finish_product(hasher)
}

pub fn advisory_callable_input_fingerprint(product: &crate::advisory::AdvisoryCallableSummary) -> InputFingerprint {
    InputFingerprint::new(product.fingerprint.raw())
}

pub fn advisory_callable_product_fingerprint(product: &crate::advisory::AdvisoryCallableSummary) -> ProductFingerprint {
    product.fingerprint
}

pub fn advisory_module_input_fingerprint(product: &crate::advisory::AdvisoryModuleProduct) -> InputFingerprint {
    InputFingerprint::new(product.fingerprint.raw())
}

pub fn advisory_module_product_fingerprint(product: &crate::advisory::AdvisoryModuleProduct) -> ProductFingerprint {
    product.fingerprint
}

pub fn data_declaration_input_fingerprint(product: &crate::db::product::DataDeclarationProduct) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    product.info.owner.hash(&mut hasher);
    product.info.root_form.hash(&mut hasher);
    product.info.shape.hash(&mut hasher);
    match &product.info.generic_signature {
        Some(signature) => {
            1u8.hash(&mut hasher);
            hash_generic_signature(signature, &mut hasher);
        }
        None => 0u8.hash(&mut hasher),
    }
    product.info.static_data.hash(&mut hasher);
    for c in product.info.components.iter() {
        c.id.hash(&mut hasher);
        c.external_label.hash(&mut hasher);
        c.local_name.hash(&mut hasher);
        c.declared_type.hash(&mut hasher);
    }
    product.info.constructor.constructor.hash(&mut hasher);
    product.info.constructor.visibility.hash(&mut hasher);
    product.info.constructor.result_type_template.hash(&mut hasher);
    finish_input(hasher)
}

pub fn data_declaration_product_fingerprint(product: &crate::db::product::DataDeclarationProduct) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    product.info.owner.hash(&mut hasher);
    product.info.shape.hash(&mut hasher);
    match &product.info.generic_signature {
        Some(signature) => {
            1u8.hash(&mut hasher);
            hash_generic_signature(signature, &mut hasher);
        }
        None => 0u8.hash(&mut hasher),
    }
    product.info.static_data.hash(&mut hasher);
    for c in product.info.components.iter() {
        c.id.hash(&mut hasher);
        c.external_label.hash(&mut hasher);
        c.local_name.hash(&mut hasher);
        c.declared_type.hash(&mut hasher);
    }
    product.info.constructor.constructor.hash(&mut hasher);
    product.info.constructor.visibility.hash(&mut hasher);
    product.info.constructor.result_type_template.hash(&mut hasher);
    finish_product(hasher)
}

pub fn enum_declaration_input_fingerprint(product: &crate::db::product::EnumDeclarationProduct) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    product.info.owner.hash(&mut hasher);
    product.info.root_form.hash(&mut hasher);
    product.info.static_data.hash(&mut hasher);
    product.info.variants.hash(&mut hasher);
    for v in product.variants.iter() {
        v.id.hash(&mut hasher);
        v.type_handle.hash(&mut hasher);
        v.shape.hash(&mut hasher);
        v.result_type_template.hash(&mut hasher);
        v.exact_case_template.hash(&mut hasher);
        if let Some(constructor) = &v.constructor {
            constructor.constructor.hash(&mut hasher);
            match &constructor.generic_signature {
                Some(signature) => {
                    1u8.hash(&mut hasher);
                    hash_generic_signature(signature, &mut hasher);
                }
                None => 0u8.hash(&mut hasher),
            }
        } else {
            0u8.hash(&mut hasher);
        }
        for f in &v.fields {
            f.id.hash(&mut hasher);
            f.external_label.hash(&mut hasher);
            f.local_name.hash(&mut hasher);
            f.declared_type.hash(&mut hasher);
        }
    }
    finish_input(hasher)
}

pub fn enum_declaration_product_fingerprint(product: &crate::db::product::EnumDeclarationProduct) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    product.info.owner.hash(&mut hasher);
    product.info.static_data.hash(&mut hasher);
    product.info.variants.hash(&mut hasher);
    for v in product.variants.iter() {
        v.id.hash(&mut hasher);
        v.type_handle.hash(&mut hasher);
        v.shape.hash(&mut hasher);
        v.result_type_template.hash(&mut hasher);
        v.exact_case_template.hash(&mut hasher);
        if let Some(constructor) = &v.constructor {
            constructor.constructor.hash(&mut hasher);
            match &constructor.generic_signature {
                Some(signature) => {
                    1u8.hash(&mut hasher);
                    hash_generic_signature(signature, &mut hasher);
                }
                None => 0u8.hash(&mut hasher),
            }
        } else {
            0u8.hash(&mut hasher);
        }
        for f in &v.fields {
            f.id.hash(&mut hasher);
            f.external_label.hash(&mut hasher);
            f.local_name.hash(&mut hasher);
            f.declared_type.hash(&mut hasher);
        }
    }
    finish_product(hasher)
}

pub fn enum_requirements_input_fingerprint(owner: &DeclarationId) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    owner.hash(&mut hasher);
    finish_input(hasher)
}

pub fn enum_requirements_product_fingerprint(product: &crate::db::product::EnumRequirementsProduct) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    for req in product.requirements.iter() {
        req.id.hash(&mut hasher);
    }
    for status in product.case_statuses.iter() {
        status.variant.hash(&mut hasher);
        status.requirement.hash(&mut hasher);
    }
    finish_product(hasher)
}

pub fn associated_surface_input_fingerprint(surface: &crate::associated::AssociatedSurface) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    surface.owner.hash(&mut hasher);
    for (base, family) in &surface.families {
        base.hash(&mut hasher);
        family.id.hash(&mut hasher);
        family.kind.hash(&mut hasher);
        family.members.hash(&mut hasher);
    }
    finish_input(hasher)
}

pub fn associated_surface_product_fingerprint(surface: &crate::associated::AssociatedSurface) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    surface.owner.hash(&mut hasher);
    for (base, family) in &surface.families {
        base.hash(&mut hasher);
        family.id.hash(&mut hasher);
        family.kind.hash(&mut hasher);
        family.members.hash(&mut hasher);
    }
    finish_product(hasher)
}

/// Fingerprints the exact source slice that declares one type-member signature.
pub fn type_member_declaration_input_fingerprint(
    unit: Option<&phalcom_modules::source::ParsedModuleUnit>,
    product: &crate::db::product::TypeMemberDeclarationProduct,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    product.id.hash(&mut hasher);
    if let Some(unit) = unit {
        hash_optional_source_region(&unit.text, Some(product.source_range), &mut hasher);
    } else {
        product.source_range.hash(&mut hasher);
    }
    product.signature.hash(&mut hasher);
    finish_input(hasher)
}

/// Fingerprints checked member semantics independently of source locations.
pub fn type_member_declaration_product_fingerprint(product: &crate::db::product::TypeMemberDeclarationProduct) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    product.id.hash(&mut hasher);
    product.signature.hash(&mut hasher);
    finish_product(hasher)
}

/// Structural case membership changes independently of payload annotations.
pub fn static_domain_case_catalog_input_fingerprint(product: &crate::static_case::StaticDomainCaseCatalogProduct) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    product.domain.hash(&mut hasher);
    for case in product.cases.iter() {
        case.id.hash(&mut hasher);
        case.shape.hash(&mut hasher);
        case.source_range.hash(&mut hasher);
    }
    finish_input(hasher)
}

pub fn static_domain_case_catalog_product_fingerprint(product: &crate::static_case::StaticDomainCaseCatalogProduct) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    product.domain.hash(&mut hasher);
    for case in product.cases.iter() {
        case.id.hash(&mut hasher);
        case.shape.hash(&mut hasher);
    }
    finish_product(hasher)
}

pub fn static_case_signature_input_fingerprint(
    unit: &phalcom_modules::source::ParsedModuleUnit,
    product: &crate::static_case::StaticCaseSignatureProduct,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    product.id.hash(&mut hasher);
    hash_optional_source_region(&unit.text, Some(product.source_range), &mut hasher);
    product.parameter_kinds.hash(&mut hasher);
    finish_input(hasher)
}

pub fn static_case_signature_product_fingerprint(product: &crate::static_case::StaticCaseSignatureProduct) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    product.id.hash(&mut hasher);
    product.parameter_kinds.hash(&mut hasher);
    finish_product(hasher)
}

/// Fingerprints the checked source contribution and canonical case facts for one associated static domain.
pub fn associated_static_domain_declaration_input_fingerprint(
    unit: &phalcom_modules::source::ParsedModuleUnit,
    product: &crate::db::product::AssociatedStaticDomainDeclarationProduct,
    store: &crate::types::store::TypeStore,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    product.id.hash(&mut hasher);
    hash_optional_source_region(&unit.text, Some(product.source_range), &mut hasher);
    hash_associated_static_domain_facts(product, store, &mut hasher);
    finish_input(hasher)
}

/// Fingerprints the canonical identity and checked cases of one associated static domain.
pub fn associated_static_domain_declaration_product_fingerprint(
    product: &crate::db::product::AssociatedStaticDomainDeclarationProduct,
    store: &crate::types::store::TypeStore,
) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    product.id.hash(&mut hasher);
    hash_associated_static_domain_facts(product, store, &mut hasher);
    finish_product(hasher)
}

fn hash_associated_static_domain_facts(
    product: &crate::db::product::AssociatedStaticDomainDeclarationProduct,
    store: &crate::types::store::TypeStore,
    hasher: &mut impl Hasher,
) {
    if let Some(domain) = store.static_domain(product.domain) {
        domain.hash(hasher);
    }
    for constructor in product.constructors.iter() {
        if let Some(constructor) = store.static_constructor(*constructor) {
            constructor.case_name.hash(hasher);
            constructor.parameter_kinds.hash(hasher);
        }
    }
}

pub fn resolved_import_input_fingerprint(product: &phalcom_modules::resolver::ImportResolutionProduct) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    product.site.importer.hash(&mut hasher);
    product.site.local.0.hash(&mut hasher);
    product.fingerprint.raw().hash(&mut hasher);
    finish_input(hasher)
}

pub fn resolved_import_product_fingerprint(product: &phalcom_modules::resolver::ImportResolutionProduct) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    match &product.target {
        Ok(target) => {
            0u8.hash(&mut hasher);
            target.hash(&mut hasher);
            product.prefixes.len().hash(&mut hasher);
            for prefix in product.prefixes.iter() {
                prefix.prefix.hash(&mut hasher);
                prefix.module.hash(&mut hasher);
            }
        }
        Err(error) => {
            1u8.hash(&mut hasher);
            hash_resolution_failure_category(error, &mut hasher);
        }
    }
    finish_product(hasher)
}

fn hash_resolution_failure_category(error: &phalcom_modules::error::ModuleResolutionError, hasher: &mut impl Hasher) {
    use phalcom_modules::error::ModuleResolutionError;

    let category = match error {
        ModuleResolutionError::ModuleNotFound(_) => 0u8,
        ModuleResolutionError::PackageNotFoundError(_) => 1,
        ModuleResolutionError::InvalidModuleLayout(_) => 2,
        ModuleResolutionError::AmbiguousModule { .. } => 3,
        ModuleResolutionError::InvalidModuleName(_, _) => 4,
        ModuleResolutionError::NonCanonicalPhysicalName { .. } => 5,
        ModuleResolutionError::Parse { .. } => 6,
        ModuleResolutionError::Interface { .. } => 7,
        ModuleResolutionError::UnknownImportRoot(_) => 8,
        ModuleResolutionError::LegacyCoreImportRemoved => 9,
        ModuleResolutionError::LegacyStdImportRemoved => 10,
        ModuleResolutionError::RelativeImportBeyondRoot { .. } => 11,
        ModuleResolutionError::ImportOutsideSourceRoot(_, _) => 12,
        ModuleResolutionError::NestedProjectBoundary(_) => 13,
        ModuleResolutionError::DuplicateSourceIdentity(_) => 14,
        ModuleResolutionError::ModulePathNotExposed { .. } => 15,
        ModuleResolutionError::PackageSurface(_) => 16,
        ModuleResolutionError::Source(_) => 17,
    };
    category.hash(hasher);
}

pub fn linked_name_input_fingerprint(product: &crate::db::product::LinkedNameProduct) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    product.module.hash(&mut hasher);
    product.name.hash(&mut hasher);
    finish_input(hasher)
}

/// Computes linked-name query input including current public and private
/// linkage inputs. The stable query identity remains in `product`; these
/// canonical fingerprints make refresh sensitive to the namespace and local
/// import layout that produced the fact.
pub(crate) fn linked_name_input_fingerprint_with_module(
    product: &crate::db::product::LinkedNameProduct,
    linked_module: Option<&phalcom_modules::linker::LinkedModule>,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    product.module.hash(&mut hasher);
    product.name.hash(&mut hasher);
    if let Some(linked_module) = linked_module {
        linked_interface_input_fingerprint(&linked_module.interface).raw().hash(&mut hasher);
        phalcom_modules::fingerprint::linked_dependency_fingerprint(linked_module)
            .raw()
            .hash(&mut hasher);
    }
    finish_input(hasher)
}

pub fn linked_name_product_fingerprint(product: &crate::db::product::LinkedNameProduct) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    product.module.hash(&mut hasher);
    product.name.hash(&mut hasher);
    match &product.fact {
        crate::db::product::LinkedNameFact::Absent => 0u8.hash(&mut hasher),
        crate::db::product::LinkedNameFact::Local(decl) => {
            1u8.hash(&mut hasher);
            decl.module.hash(&mut hasher);
            decl.name.hash(&mut hasher);
        }
        crate::db::product::LinkedNameFact::ImportedModule(mod_id) => {
            2u8.hash(&mut hasher);
            mod_id.hash(&mut hasher);
        }
        crate::db::product::LinkedNameFact::ImportedBinding(decl) => {
            3u8.hash(&mut hasher);
            decl.module.hash(&mut hasher);
            decl.name.hash(&mut hasher);
        }
    }
    finish_product(hasher)
}

pub fn public_export_input_fingerprint(product: &crate::db::product::PublicExportProduct) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    product.module.hash(&mut hasher);
    product.name.hash(&mut hasher);
    finish_input(hasher)
}

/// Computes public-export query input including the current linked interface
/// that owns the export namespace.
pub(crate) fn public_export_input_fingerprint_with_interface(
    product: &crate::db::product::PublicExportProduct,
    linked_interface: Option<&LinkedModuleInterface>,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    product.module.hash(&mut hasher);
    product.name.hash(&mut hasher);
    if let Some(linked_interface) = linked_interface {
        linked_interface_input_fingerprint(linked_interface).raw().hash(&mut hasher);
    }
    finish_input(hasher)
}

pub fn public_export_product_fingerprint(product: &crate::db::product::PublicExportProduct) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    product.module.hash(&mut hasher);
    product.name.hash(&mut hasher);
    match &product.fact {
        crate::db::product::PublicExportFact::Absent => 0u8.hash(&mut hasher),
        crate::db::product::PublicExportFact::Present(export) => {
            1u8.hash(&mut hasher);
            export.public_name.hash(&mut hasher);
            export.target.hash(&mut hasher);
        }
    }
    finish_product(hasher)
}

pub fn static_definition_input_fingerprint(product: &crate::static_definition::StaticDefinitionProduct) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    product.definition.id.hash(&mut hasher);
    format!("{:?}{:?}", product.definition.signature, product.definition.body).hash(&mut hasher);
    finish_input(hasher)
}

pub fn static_definition_product_fingerprint(product: &crate::static_definition::StaticDefinitionProduct) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    let def = &product.definition;
    def.id.hash(&mut hasher);
    def.signature.result_domain.hash(&mut hasher);
    def.signature.parameters.len().hash(&mut hasher);
    for p in def.signature.parameters.iter() {
        p.name.hash(&mut hasher);
        p.domain.hash(&mut hasher);
        p.mode.hash(&mut hasher);
        p.index.hash(&mut hasher);
        p.type_parameter_id.is_some().hash(&mut hasher);
    }
    match &def.signature.generic_signature {
        Some(sig) => {
            1u8.hash(&mut hasher);
            hash_generic_signature(sig, &mut hasher);
        }
        None => 0u8.hash(&mut hasher),
    }
    match &def.body {
        crate::static_definition::StaticDefinitionBodyRepr::TypeForm(ann) => {
            0u8.hash(&mut hasher);
            hash_debug_without_source_ranges(ann, &mut hasher);
        }
        crate::static_definition::StaticDefinitionBodyRepr::Expression(expr) => {
            1u8.hash(&mut hasher);
            hash_debug_without_source_ranges(expr, &mut hasher);
        }
        crate::static_definition::StaticDefinitionBodyRepr::Block(block) => {
            2u8.hash(&mut hasher);
            hash_debug_without_source_ranges(block, &mut hasher);
        }
    }
    match product.checked_body.as_ref() {
        Some(body) => {
            1u8.hash(&mut hasher);
            checked_static_body_product_fingerprint(body).hash(&mut hasher);
        }
        None => 0u8.hash(&mut hasher),
    }
    finish_product(hasher)
}

fn hash_module_static_binding_index(index: &crate::module_bindings::ModuleStaticBindingIndex, hasher: &mut impl Hasher) {
    index.module().hash(hasher);
    for (visible_name, fact) in index.visible_bindings() {
        visible_name.hash(hasher);
        fact.symbol.hash(hasher);
        fact.source_body.hash(hasher);
        fact.source_binding.hash(hasher);
        fact.phase.hash(hasher);
        let state = &fact.state;
        state.name.hash(hasher);
        hash_range(state.range, hasher);
        state.parameter.hash(hasher);
        hash_binding_contract(&state.contract, hasher);
        hash_type_knowledge(&state.current, false, hasher);
        hash_denotation(&state.denotation, hasher);
        hash_binding_consistency(&state.consistency, hasher);
        hash_causal_invalidity(state.causal_invalidity, hasher);
        state.mutable.hash(hasher);
        state.initialization.hash(hasher);
        state.version.hash(hasher);
    }
}

/// Fingerprints the source-facing name mapping and checked facts used to seed
/// ordinary module bindings in static-body analysis.
pub fn module_static_binding_input_fingerprint(index: &crate::module_bindings::ModuleStaticBindingIndex) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_module_static_binding_index(index, &mut hasher);
    finish_input(hasher)
}

/// Fingerprints stable identities, phase categories, and checked binding
/// states published to semantic consumers.
pub fn module_static_binding_product_fingerprint(index: &crate::module_bindings::ModuleStaticBindingIndex) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_module_static_binding_index(index, &mut hasher);
    finish_product(hasher)
}

/// Fingerprints a static definition's source/signature and the module binding
/// product consumed by its checked body.
pub fn checked_static_body_source_input_fingerprint(
    definition: &crate::static_definition::StaticDefinition,
    module_binding_fingerprint: Option<ProductFingerprint>,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    definition.id.hash(&mut hasher);
    format!("{:?}{:?}", definition.signature, definition.body).hash(&mut hasher);
    module_binding_fingerprint.hash(&mut hasher);
    finish_input(hasher)
}

pub fn checked_static_body_input_fingerprint(body: &crate::static_definition::CheckedStaticBody) -> InputFingerprint {
    body.input_fingerprint
}

/// Fingerprints the complete checked-body product, including ordinary checker
/// resolutions, evaluator layout, dependencies, and diagnostics.
pub fn checked_static_body_product_fingerprint(body: &crate::static_definition::CheckedStaticBody) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    format!("{body:?}").hash(&mut hasher);
    finish_product(hasher)
}

/// Fingerprints a static constant's targeted initializer source and declared type.
pub fn static_constant_input_fingerprint(
    id: &crate::static_constant::StaticConstantId,
    initializer: &phalcom_ast::ast::Expr,
    annotation: Option<&phalcom_ast::ast::TypeAnnotation>,
    expected_type: Option<crate::types::id::TypeId>,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    format!("{initializer:?}").hash(&mut hasher);
    format!("{annotation:?}").hash(&mut hasher);
    expected_type.hash(&mut hasher);
    finish_input(hasher)
}

/// Fingerprints the reduced static constant product, including evaluated content,
/// outcome, and diagnostics.
pub fn static_constant_product_fingerprint(
    product: &crate::static_constant::StaticConstantProduct,
    store: &crate::static_value::StaticValueStore,
    types: &crate::types::store::TypeStore,
    projects: &phalcom_modules::ProjectUniverse,
) -> Result<ProductFingerprint, String> {
    let mut hasher = DefaultHasher::new();
    if !matches!(product.outcome, crate::static_eval::StaticEvalOutcome::Symbolic(_)) {
        product.id.hash(&mut hasher);
    }
    if let Some(checked) = &product.checked {
        checked.record_key_static_constant_reads.hash(&mut hasher);
    }
    if let Some(imported) = &product.imported {
        imported.value_fingerprint.hash(&mut hasher);
        imported.dependency_fingerprint.hash(&mut hasher);
    }
    match &product.outcome {
        crate::static_eval::StaticEvalOutcome::Reduced(val_ref) => {
            0u8.hash(&mut hasher);
            if let Ok(content_hash) = store.content_fingerprint(*val_ref) {
                content_hash.hash(&mut hasher);
            } else {
                val_ref.hash(&mut hasher);
            }
        }
        crate::static_eval::StaticEvalOutcome::ReducedKinded(type_id) => {
            1u8.hash(&mut hasher);
            type_id.hash(&mut hasher);
        }
        crate::static_eval::StaticEvalOutcome::Symbolic(residual_ref) => {
            2u8.hash(&mut hasher);
            let mut semantic = crate::db::static_fingerprint::StaticSemanticFingerprinter::new(types, store, product.residuals.as_deref(), projects);
            semantic.constant_owner(&product.id)?.hash(&mut hasher);
            semantic.residual(*residual_ref)?.hash(&mut hasher);
        }
        crate::static_eval::StaticEvalOutcome::Invalid(err) => {
            3u8.hash(&mut hasher);
            format!("{err:?}").hash(&mut hasher);
        }
        crate::static_eval::StaticEvalOutcome::Recursive(cycle) => {
            4u8.hash(&mut hasher);
            cycle.cycle_start.hash(&mut hasher);
            cycle.applications.len().hash(&mut hasher);
            for frame in cycle.applications.iter() {
                frame.definition.hash(&mut hasher);
            }
        }
        crate::static_eval::StaticEvalOutcome::BudgetExceeded(_) => {
            5u8.hash(&mut hasher);
        }
        crate::static_eval::StaticEvalOutcome::Cancelled => {
            6u8.hash(&mut hasher);
        }
        crate::static_eval::StaticEvalOutcome::Unknown(reason) => {
            7u8.hash(&mut hasher);
            reason.hash(&mut hasher);
        }
        crate::static_eval::StaticEvalOutcome::Blocked(reason) => {
            8u8.hash(&mut hasher);
            format!("{reason:?}").hash(&mut hasher);
        }
        crate::static_eval::StaticEvalOutcome::Dynamic(obligation) => {
            9u8.hash(&mut hasher);
            format!("{obligation:?}").hash(&mut hasher);
        }
    }
    for diag in product.diagnostics.iter() {
        format!("{diag:?}").hash(&mut hasher);
    }
    Ok(finish_product(hasher))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::evidence::EvidenceStatus;

    #[test]
    fn return_contract_validation_fingerprint_distinguishes_variants() {
        let variants = [
            ReturnContractValidation::NotApplicable,
            ReturnContractValidation::Unchecked,
            ReturnContractValidation::Satisfied(EvidenceStatus::Assumed),
            ReturnContractValidation::Satisfied(EvidenceStatus::Established),
            ReturnContractValidation::Refuted,
            ReturnContractValidation::Blocked,
            ReturnContractValidation::DynamicBoundary,
        ];
        let mut hashes = std::collections::BTreeSet::new();
        for variant in variants {
            let mut hasher = DefaultHasher::new();
            hash_return_contract_validation(variant, &mut hasher);
            let h = hasher.finish();
            assert!(hashes.insert(h), "hash collision for {variant:?}");
        }
    }

    #[test]
    fn field_validity_fingerprint_distinguishes_variants() {
        use crate::checker::flow::FieldContractValidity;
        use crate::types::outcome::{BlockReason, DynamicBoundaryObligation};

        let variants = [
            FieldContractValidity::Unchecked,
            FieldContractValidity::Validated,
            FieldContractValidity::Assumed,
            FieldContractValidity::Refuted,
            FieldContractValidity::Blocked(BlockReason::SuppressedDependency),
            FieldContractValidity::DynamicBoundary(DynamicBoundaryObligation { reason: "dyn".into() }),
        ];
        let mut hashes = std::collections::BTreeSet::new();
        for variant in &variants {
            let mut hasher = DefaultHasher::new();
            hash_field_validity(variant, &mut hasher);
            let h = hasher.finish();
            assert!(hashes.insert(h), "hash collision for {variant:?}");
        }
    }

    #[test]
    fn flow_summary_fingerprint_changes_on_field_validity_change() {
        use crate::checker::analysis::{FlowFieldSummary, FlowStateSummary};
        use crate::checker::flow::{FieldContractValidity, FieldInitialization};
        use crate::identity::{DeclarationId, DispatchSide, FieldId, ModuleId};

        let field = FieldId::new(DeclarationId::new(ModuleId::universe_root(), "Cell".into()), "_value", DispatchSide::Instance);
        let contract = TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::MissingInitializer);
        let current = TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::MissingInitializer);

        let make_summary = |validity: FieldContractValidity, causal: CausalInvalidity| {
            let mut summary = FlowStateSummary::default();
            summary.fields.insert(
                field.clone(),
                FlowFieldSummary {
                    contract: contract.clone(),
                    current: current.clone(),
                    initialization: FieldInitialization::DefinitelyInitialized,
                    validity,
                    causal_invalidity: causal,
                },
            );
            let mut hasher = DefaultHasher::new();
            hash_flow_summary(&summary, &mut hasher);
            hasher.finish()
        };

        let h_validated = make_summary(FieldContractValidity::Validated, CausalInvalidity::Clean);
        let h_assumed = make_summary(FieldContractValidity::Assumed, CausalInvalidity::Clean);
        assert_ne!(h_validated, h_assumed);

        // Same shape causal invalidity (One(cause0) vs One(cause1)) produces same fingerprint
        let h_cause0 = make_summary(FieldContractValidity::Refuted, CausalInvalidity::One(crate::identity::DiagnosticCauseId(0)));
        let h_cause1 = make_summary(FieldContractValidity::Refuted, CausalInvalidity::One(crate::identity::DiagnosticCauseId(1)));
        assert_eq!(h_cause0, h_cause1);
    }
}
