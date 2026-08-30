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
use crate::declarations::{DeclarationTypeInfo, GenericSupertypeTemplate};
use crate::diagnostic::{DiagnosticFix, SemanticDiagnostic, SemanticSourceSpan};
use crate::identity::{CallableId, DeclarationId, ModuleId};
use crate::signature::{CallableSemanticSignature, FieldSemanticSignature, ReturnContractValidation};
use crate::source::ParsedModuleUnit;
use crate::surface::DeclarationSurface;
use crate::types::denotation::SemanticDenotation;
use crate::types::evidence::TypeKnowledge;
use crate::types::outcome::{BlockReason, BudgetReport};
use crate::types::parameter::GenericSignature;
use crate::types::store::TypeStore;
use phalcom_ast::ast::{ClassMember, ImportPath, ImportRoot, IndexAccessor, MetadataLiteral, ParameterDef, RestMode, Statement};
use phalcom_common::range::SourceRange;
use phalcom_modules::graph::{ReferenceKind, RuntimeDependencyReason, SemanticEdgeKind};
use phalcom_modules::interface::{ImportSurface, InterfaceBuilder, LinkedExportTarget, LinkedModuleInterface, UnlinkedExportTarget, UnlinkedModuleInterface};
use phalcom_modules::linker::{LinkedProgram, LinkedReadSpec};
use phalcom_modules::manifest::{DependencySpec, ValidatedProjectManifest};
use phalcom_modules::metadata::{MetadataTarget, ModuleMetadata};
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::ModuleKind;
use std::collections::{BTreeMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

fn finish_input(hasher: DefaultHasher) -> InputFingerprint {
    InputFingerprint::new(hasher.finish())
}

fn finish_product(hasher: DefaultHasher) -> ProductFingerprint {
    ProductFingerprint::new(hasher.finish())
}

fn hash_range(range: SourceRange, hasher: &mut impl Hasher) {
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
    hash_optional_source_region(source, parameter.annotation.as_ref().map(|annotation| annotation.range), hasher);
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
            }
            ClassMember::Setter(setter) => {
                3u8.hash(&mut hasher);
                hash_dispatch_side(side, &mut hasher);
                setter.name.hash(&mut hasher);
                hash_parameter_source(&unit.text, &setter.param, &mut hasher);
            }
            ClassMember::Index(index) => {
                4u8.hash(&mut hasher);
                hash_dispatch_side(side, &mut hasher);
                match &index.accessor {
                    IndexAccessor::Get => 0u8.hash(&mut hasher),
                    IndexAccessor::Set { put } => {
                        1u8.hash(&mut hasher);
                        hash_parameter_source(&unit.text, put, &mut hasher);
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
            ClassMember::Variant(_) => {
                5u8.hash(&mut hasher);
            }
        }
    }
    finish_input(hasher)
}

fn hash_import_path(path: &ImportPath, include_ranges: bool, hasher: &mut impl Hasher) {
    match &path.root {
        ImportRoot::Absolute(segment) => {
            0u8.hash(hasher);
            segment.name.hash(hasher);
            if include_ranges {
                hash_range(segment.range, hasher);
            }
        }
        ImportRoot::Relative { dots, range } => {
            1u8.hash(hasher);
            dots.hash(hasher);
            if include_ranges {
                hash_range(*range, hasher);
            }
        }
    }
    path.segments.len().hash(hasher);
    for segment in &path.segments {
        segment.name.hash(hasher);
        if include_ranges {
            hash_range(segment.range, hasher);
        }
    }
    if include_ranges {
        hash_range(path.range, hasher);
    }
}

fn hash_metadata_literal(literal: &MetadataLiteral, hasher: &mut impl Hasher) {
    match literal {
        MetadataLiteral::Unit => 0u8.hash(hasher),
        MetadataLiteral::Bool(value) => {
            1u8.hash(hasher);
            value.hash(hasher);
        }
        MetadataLiteral::Int(value) => {
            2u8.hash(hasher);
            value.hash(hasher);
        }
        MetadataLiteral::Float(value) => {
            3u8.hash(hasher);
            value.to_bits().hash(hasher);
        }
        MetadataLiteral::String(value) => {
            4u8.hash(hasher);
            value.hash(hasher);
        }
        MetadataLiteral::Symbol(value) => {
            5u8.hash(hasher);
            value.hash(hasher);
        }
        MetadataLiteral::Tuple(values) => {
            6u8.hash(hasher);
            values.len().hash(hasher);
            for value in values {
                hash_metadata_literal(value, hasher);
            }
        }
        MetadataLiteral::Record(fields) => {
            7u8.hash(hasher);
            fields.len().hash(hasher);
            for (name, value) in fields {
                name.hash(hasher);
                hash_metadata_literal(value, hasher);
            }
        }
    }
}

fn hash_metadata(metadata: &ModuleMetadata, include_ranges: bool, hasher: &mut impl Hasher) {
    metadata.attributes.len().hash(hasher);
    for attribute in &metadata.attributes {
        match attribute.target {
            MetadataTarget::Module => 0u8.hash(hasher),
            MetadataTarget::Package => 1u8.hash(hasher),
            MetadataTarget::Project => 2u8.hash(hasher),
        }
        attribute.name.hash(hasher);
        attribute.arguments.len().hash(hasher);
        for argument in &attribute.arguments {
            hash_metadata_literal(argument, hasher);
        }
        if include_ranges {
            hash_range(attribute.range, hasher);
        }
    }
}

fn hash_unlinked_interface(interface: &UnlinkedModuleInterface, include_ranges: bool, hasher: &mut impl Hasher) {
    interface.id.hash(hasher);
    hash_module_kind(interface.kind, hasher);

    interface.declarations.len().hash(hasher);
    for (name, declaration) in &interface.declarations {
        name.hash(hasher);
        declaration.name.hash(hasher);
        declaration.is_const.hash(hasher);
        if include_ranges {
            hash_range(declaration.range, hasher);
        }
    }

    interface.imports.len().hash(hasher);
    for import in &interface.imports {
        match import {
            ImportSurface::Module(module) => {
                0u8.hash(hasher);
                hash_import_path(&module.path, include_ranges, hasher);
                module.alias.as_ref().map(|alias| alias.name.as_str()).hash(hasher);
                if include_ranges {
                    if let Some(alias) = &module.alias {
                        hash_range(alias.range, hasher);
                    }
                    hash_range(module.range, hasher);
                }
            }
            ImportSurface::Selective(selective) => {
                1u8.hash(hasher);
                hash_import_path(&selective.path, include_ranges, hasher);
                selective.items.len().hash(hasher);
                for item in &selective.items {
                    item.name.hash(hasher);
                    item.alias.as_ref().map(|alias| alias.name.as_str()).hash(hasher);
                    if include_ranges {
                        hash_range(item.name_range, hasher);
                        if let Some(alias) = &item.alias {
                            hash_range(alias.range, hasher);
                        }
                        hash_range(item.range, hasher);
                    }
                }
                if include_ranges {
                    hash_range(selective.range, hasher);
                }
            }
            ImportSurface::ReExport(reexport) => {
                2u8.hash(hasher);
                hash_import_path(&reexport.path, include_ranges, hasher);
                reexport.items.len().hash(hasher);
                for item in &reexport.items {
                    item.local_or_remote_name.hash(hasher);
                    item.alias.as_ref().map(|alias| alias.name.as_str()).hash(hasher);
                    if include_ranges {
                        hash_range(item.name_range, hasher);
                        if let Some(alias) = &item.alias {
                            hash_range(alias.range, hasher);
                        }
                        hash_range(item.range, hasher);
                    }
                }
                if include_ranges {
                    hash_range(reexport.range, hasher);
                }
            }
        }
    }

    interface.exports.len().hash(hasher);
    for (name, export) in &interface.exports {
        name.hash(hasher);
        export.exported_name.hash(hasher);
        export.internal_name.hash(hasher);
        match &export.target {
            UnlinkedExportTarget::Local(local) => {
                0u8.hash(hasher);
                local.hash(hasher);
            }
            UnlinkedExportTarget::ReExport { path, remote } => {
                1u8.hash(hasher);
                hash_import_path(path, include_ranges, hasher);
                remote.hash(hasher);
            }
        }
        if include_ranges {
            hash_range(export.range, hasher);
        }
    }

    interface.exposed_children.len().hash(hasher);
    for child in &interface.exposed_children {
        child.hash(hasher);
    }
    hash_metadata(&interface.metadata, include_ranges, hasher);
}

fn hash_linked_interface(interface: &LinkedModuleInterface, include_ranges: bool, hasher: &mut impl Hasher) {
    interface.module.hash(hasher);
    hash_module_kind(interface.kind, hasher);
    interface.exports.len().hash(hasher);
    for (name, export) in &interface.exports {
        name.hash(hasher);
        export.public_name.hash(hasher);
        match &export.target {
            LinkedExportTarget::Binding(symbol) => {
                0u8.hash(hasher);
                symbol.hash(hasher);
            }
            LinkedExportTarget::Module(module) => {
                1u8.hash(hasher);
                module.hash(hasher);
            }
        }
        if include_ranges {
            hash_range(export.range, hasher);
        }
    }
    hash_metadata(&interface.metadata, include_ranges, hasher);
}

fn hash_type_knowledge(knowledge: &TypeKnowledge, include_provenance: bool, hasher: &mut impl Hasher) {
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

fn hash_generic_signature(signature: &GenericSignature, hasher: &mut impl Hasher) {
    signature.owner.hash(hasher);
    signature.parameters.hash(hasher);
    signature.constraints.hash(hasher);
}

fn hash_generic_supertype_template(template: &GenericSupertypeTemplate, hasher: &mut impl Hasher) {
    template.declaration.hash(hasher);
    template.supertype.hash(hasher);
}

fn hash_declaration_type_info(info: &DeclarationTypeInfo, hasher: &mut impl Hasher) {
    info.declaration.hash(hasher);
    info.form.hash(hasher);
    info.class_object_type.hash(hasher);
    info.kind.hash(hasher);
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
                hash_source_span(source, hasher);
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
        BlockReason::OpaqueNative(name) => {
            4u8.hash(hasher);
            name.hash(hasher);
        }
        BlockReason::ReflectionBoundary => 5u8.hash(hasher),
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

fn hash_causal_invalidity(causal: crate::checker::causal::CausalInvalidity, hasher: &mut impl Hasher) {
    match causal {
        crate::checker::causal::CausalInvalidity::Clean => 0u8.hash(hasher),
        crate::checker::causal::CausalInvalidity::One(_) => 1u8.hash(hasher),
        crate::checker::causal::CausalInvalidity::Multiple => 2u8.hash(hasher),
    }
}

fn hash_denotation(denotation: &Option<SemanticDenotation>, hasher: &mut impl Hasher) {
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
            match assoc {
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
    }
}

fn hash_binding_contract(contract: &Option<crate::checker::binding::BindingContract>, hasher: &mut impl Hasher) {
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
    }
}

fn hash_binding_consistency(consistency: &crate::checker::binding::BindingConsistency, hasher: &mut impl Hasher) {
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
    let mut hasher = DefaultHasher::new();
    hash_unlinked_interface(interface, true, &mut hasher);
    finish_input(hasher)
}

/// Computes semantic product fingerprint for an unlinked module interface.
pub fn unlinked_interface_product_fingerprint(interface: &UnlinkedModuleInterface) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_unlinked_interface(interface, false, &mut hasher);
    finish_product(hasher)
}

/// Computes the source/provenance-sensitive input identity of a linked module interface.
pub fn linked_interface_input_fingerprint(interface: &LinkedModuleInterface) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_linked_interface(interface, true, &mut hasher);
    finish_input(hasher)
}

/// Computes semantic product fingerprint for a linked module interface.
pub fn linked_interface_product_fingerprint(interface: &LinkedModuleInterface) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_linked_interface(interface, false, &mut hasher);
    finish_product(hasher)
}

/// Computes the direct semantic input identity of declaration type metadata.
pub fn declaration_shell_input_fingerprint(info: &DeclarationTypeInfo) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_declaration_type_info(info, &mut hasher);
    finish_input(hasher)
}

/// Computes the semantic product identity of declaration type metadata.
pub fn declaration_shell_product_fingerprint(info: &DeclarationTypeInfo) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_declaration_type_info(info, &mut hasher);
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
pub fn hierarchy_edge_product_fingerprint(class_decl: &DeclarationId, super_decl: &Option<DeclarationId>) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    class_decl.hash(&mut hasher);
    super_decl.hash(&mut hasher);
    finish_product(hasher)
}

/// Computes input fingerprint for a callable body query.
pub fn callable_body_input_fingerprint(callable: &CallableId, body: &[Statement], _body_range: SourceRange, store: &TypeStore) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    callable.hash(&mut hasher);
    store.id().hash(&mut hasher);
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
    for (field, fact) in lifecycle.fields.iter().filter(|(field, _)| &field.owner == callable.declaration_owner()) {
        field.hash(&mut hasher);
        hash_type_knowledge(&fact.contract, true, &mut hasher);
        hash_type_knowledge(&fact.read_knowledge, true, &mut hasher);
        fact.initialization.hash(&mut hasher);
    }
    finish_input(hasher)
}

/// Computes callable-body input identity from the formal workspace inputs.
///
/// Linked semantic availability is a direct input even when the callable body
/// did not record a dependency on a declaration that was absent in an earlier
/// link. The linked-program product is semantic and therefore does not make
/// source-position movement invalidate the body query.
pub fn callable_body_input_fingerprint_with_formal_inputs(
    callable: &CallableId,
    body: &[Statement],
    body_range: SourceRange,
    store: &TypeStore,
    sources: &BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    linked: &LinkedProgram,
    lifecycle: Option<&crate::checker::field_lifecycle::FieldLifecycleTable>,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    callable_body_input_fingerprint(callable, body, body_range, store).0.hash(&mut hasher);
    if let Some(unit) = sources.get(callable.module()) {
        unit.text.get(body_range.start..body_range.end).map(str::as_bytes).hash(&mut hasher);
    }
    source_resolution_input_fingerprint(sources).raw().hash(&mut hasher);
    semantic_component_product_fingerprint(linked).raw().hash(&mut hasher);
    if let Some(lifecycle) = lifecycle {
        for (field, fact) in lifecycle.fields.iter().filter(|(field, _)| &field.owner == callable.declaration_owner()) {
            field.hash(&mut hasher);
            hash_type_knowledge(&fact.contract, true, &mut hasher);
            hash_type_knowledge(&fact.read_knowledge, true, &mut hasher);
            fact.initialization.hash(&mut hasher);
        }
    }
    finish_input(hasher)
}

/// Computes the source-local declaration namespace identity consumed by body
/// resolution. Declaration bodies are intentionally absent; adding/removing a
/// declaration still changes this identity so a previously unresolved caller
/// cannot survive a later re-addition without recomputation.
pub fn source_resolution_input_fingerprint(sources: &BTreeMap<ModuleId, Arc<ParsedModuleUnit>>) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    sources.len().hash(&mut hasher);
    for (module, unit) in sources {
        module.hash(&mut hasher);
        match InterfaceBuilder::build(module.clone(), unit.kind, &unit.program) {
            Ok(interface) => unlinked_interface_product_fingerprint(&interface).raw().hash(&mut hasher),
            Err(_) => {
                // Invalid interfaces must not reuse a product across a
                // namespace change. Keep this fallback location agnostic.
                1u8.hash(&mut hasher);
                hash_debug_without_source_ranges(&unit.program.statements, &mut hasher);
            }
        }
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
        hash_denotation(&expression.denotation, &mut hasher);
        hash_analysis_status(&expression.status, &analysis.internal_incidents, &mut hasher);
        hash_causal_invalidity(expression.causal_invalidity, &mut hasher);
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
        binding.version.hash(&mut hasher);
    }

    hash_flow_graph_semantics(&analysis.flow_graph, &mut hasher);
    hash_flow_summary(&analysis.entry_flow, &mut hasher);
    hash_exit_facts(&analysis.exits, &mut hasher);

    analysis.internal_incidents.len().hash(&mut hasher);
    for incident in analysis.internal_incidents.iter() {
        hash_internal_incident_shape(incident, &mut hasher);
    }

    analysis.dependencies.hash(&mut hasher);
    hash_callable_analysis_status(analysis.status, &analysis.internal_incidents, &mut hasher);
    hash_return_contract_validation(analysis.return_validation, &mut hasher);
    // `dependency_fingerprint` is assigned from this fingerprint after
    // computation. Hashing it would make the product definition recursive.
    finish_product(hasher)
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

pub fn enum_declaration_input_fingerprint(info: &crate::enum_semantics::EnumInfo) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    info.owner.hash(&mut hasher);
    info.root_form.hash(&mut hasher);
    info.variants.hash(&mut hasher);
    finish_input(hasher)
}

pub fn enum_declaration_product_fingerprint(product: &crate::db::product::EnumDeclarationProduct) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    product.info.owner.hash(&mut hasher);
    product.info.variants.hash(&mut hasher);
    for v in product.variants.iter() {
        v.id.hash(&mut hasher);
        v.type_handle.hash(&mut hasher);
        v.shape.hash(&mut hasher);
        v.result_type_template.hash(&mut hasher);
        v.exact_case_template.hash(&mut hasher);
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

pub fn associated_surface_input_fingerprint(owner: &DeclarationId) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    owner.hash(&mut hasher);
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

        let field = FieldId::new(DeclarationId::new(ModuleId::core(), "Cell".into()), "_value", DispatchSide::Instance);
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
