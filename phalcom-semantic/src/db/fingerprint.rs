//! Deterministic semantic input and product fingerprint hashing (Spec 04.5 / Wave 5).
//!
//! Input fingerprints answer "must this query refresh its stored product?" and
//! therefore include source/provenance data that is observable on the product
//! itself. Product fingerprints answer "did semantic meaning change for
//! dependents?" and deliberately omit incidental source movement for contract
//! products such as interfaces, declaration surfaces, and callable signatures.

use crate::checker::analysis::{
    AnalysisStatus, BodyExitFacts, CallableAnalysis, CallableAnalysisStatus, FlowStateSummary,
};
use crate::checker::flow::graph::{FlowEdgeKind, FlowGraph, FlowNodeKind};
use crate::db::key::{InputFingerprint, ProductFingerprint};
use crate::declarations::{DeclarationTypeInfo, GenericSupertypeTemplate};
use crate::diagnostic::{DiagnosticFix, SemanticDiagnostic, SemanticSourceSpan};
use crate::explain::{DerivationRule, EvidenceRef, ExplanationArena, ExplanationStep, PredicateKind};
use crate::identity::{CallableId, DeclarationId, ExplanationId, ModuleId};
use crate::signature::CallableSemanticSignature;
use crate::surface::DeclarationSurface;
use crate::types::denotation::SemanticDenotation;
use crate::types::evidence::TypeKnowledge;
use crate::types::outcome::{BlockReason, BudgetReport};
use crate::types::parameter::GenericSignature;
use crate::types::store::TypeStore;
use phalcom_ast::ast::{
    ClassMember, ImportPath, ImportRoot, IndexAccessor, MetadataLiteral, ParameterDef, RestMode, Statement,
};
use phalcom_common::range::SourceRange;
use phalcom_modules::graph::{ReferenceKind, RuntimeDependencyReason, SemanticEdgeKind};
use phalcom_modules::interface::{
    ImportSurface, LinkedExportTarget, LinkedModuleInterface, UnlinkedExportTarget, UnlinkedModuleInterface,
};
use phalcom_modules::linker::{LinkedProgram, LinkedReadSpec};
use phalcom_modules::manifest::{DependencySpec, ValidatedProjectManifest};
use phalcom_modules::metadata::{MetadataTarget, ModuleMetadata};
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::ModuleKind;
use std::collections::BTreeSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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

fn hash_generic_contract_source(source: &str, parameters: &[phalcom_ast::ast::GenericParameterSyntax], where_clause: Option<&phalcom_ast::ast::WhereClauseSyntax>, hasher: &mut impl Hasher) {
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
                (method.is_constructor || method.attributes.iter().any(|attribute| attribute.name == "constructor"))
                    .hash(&mut hasher);
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
            evidence.ty.hash(hasher);
            evidence.authority.hash(hasher);
            if include_provenance {
                evidence.provenance.ranges.len().hash(hasher);
                for range in &evidence.provenance.ranges {
                    hash_range(*range, hasher);
                }
                evidence.provenance.descriptions.hash(hasher);
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

fn hash_dispatch_callable_signature(
    signature: &crate::dispatch::CallableSignature,
    include_provenance: bool,
    hasher: &mut impl Hasher,
) {
    signature.selector.hash(hasher);
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
    fields.sort_by(|(left, _), (right, _)| left.cmp(right));
    fields.len().hash(hasher);
    for (name, knowledge) in fields {
        name.hash(hasher);
        hash_type_knowledge(knowledge, include_provenance, hasher);
    }

    let mut callables = surface.callable_signatures.iter().collect::<Vec<_>>();
    callables.sort_by(|(left, _), (right, _)| left.cmp(right));
    callables.len().hash(hasher);
    for (selector, signature) in callables {
        selector.hash(hasher);
        hash_dispatch_callable_signature(signature, include_provenance, hasher);
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
        parameter.index.hash(hasher);
        parameter.local_name.hash(hasher);
        parameter.external_label.hash(hasher);
        hash_rest_mode(parameter.rest, hasher);
        parameter.ty.hash(hasher);
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
    signature.return_type.hash(hasher);
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

fn hash_analysis_status(status: &AnalysisStatus, hasher: &mut impl Hasher) {
    match status {
        AnalysisStatus::Ready => 0u8.hash(hasher),
        AnalysisStatus::Invalid(cause) => {
            1u8.hash(hasher);
            cause.hash(hasher);
        }
        AnalysisStatus::Blocked(reason) => {
            2u8.hash(hasher);
            hash_block_reason(reason, hasher);
        }
        AnalysisStatus::DynamicBoundary(reason) => {
            3u8.hash(hasher);
            reason.hash(hasher);
        }
        AnalysisStatus::Cancelled => 4u8.hash(hasher),
        AnalysisStatus::BudgetExceeded(report) => {
            5u8.hash(hasher);
            hash_budget_report(report, hasher);
        }
        AnalysisStatus::InternalFailure(incident) => {
            6u8.hash(hasher);
            incident.hash(hasher);
        }
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

fn hash_flow_graph(graph: &FlowGraph, hasher: &mut impl Hasher) {
    graph.nodes.len().hash(hasher);
    for (id, node) in &graph.nodes {
        id.hash(hasher);
        node.id.hash(hasher);
        hash_flow_node_kind(&node.kind, hasher);
        hash_range(node.range, hasher);
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

fn hash_flow_summary(summary: &FlowStateSummary, hasher: &mut impl Hasher) {
    summary.known_bindings.len().hash(hasher);
    for (binding, ty) in &summary.known_bindings {
        binding.hash(hasher);
        ty.hash(hasher);
    }
    summary.fact_count.hash(hasher);
}

fn hash_exit_facts(exits: &BodyExitFacts, hasher: &mut impl Hasher) {
    exits.returns.len().hash(hasher);
    for summary in &exits.returns {
        hash_flow_summary(summary, hasher);
    }
    exits.throws.len().hash(hasher);
    for summary in &exits.throws {
        hash_flow_summary(summary, hasher);
    }
    exits.unreachable.hash(hasher);
}

fn hash_predicate_kind(kind: &PredicateKind, hasher: &mut impl Hasher) {
    match kind {
        PredicateKind::IsInstance => 0u8.hash(hasher),
        PredicateKind::IsNotInstance => 1u8.hash(hasher),
        PredicateKind::IsNil => 2u8.hash(hasher),
        PredicateKind::NotNil => 3u8.hash(hasher),
        PredicateKind::EqualLiteral => 4u8.hash(hasher),
        PredicateKind::NotEqualLiteral => 5u8.hash(hasher),
        PredicateKind::Ordered => 6u8.hash(hasher),
        PredicateKind::Truthy => 7u8.hash(hasher),
        PredicateKind::Falsy => 8u8.hash(hasher),
    }
}

fn hash_derivation_rule(rule: &DerivationRule, hasher: &mut impl Hasher) {
    match rule {
        DerivationRule::LiteralSynthesis => 0u8.hash(hasher),
        DerivationRule::AnnotationConstraint => 1u8.hash(hasher),
        DerivationRule::MethodCallReturn { selector } => {
            2u8.hash(hasher);
            selector.hash(hasher);
        }
        DerivationRule::GenericInstantiation { type_args } => {
            3u8.hash(hasher);
            type_args.hash(hasher);
        }
        DerivationRule::FlowRefinement { predicate_kind } => {
            4u8.hash(hasher);
            hash_predicate_kind(predicate_kind, hasher);
        }
        DerivationRule::BranchJoin { branch_count } => {
            5u8.hash(hasher);
            branch_count.hash(hasher);
        }
        DerivationRule::PolicyEnforcement { code } => {
            6u8.hash(hasher);
            code.hash(hasher);
        }
        DerivationRule::IterationElementResolution => 7u8.hash(hasher),
        DerivationRule::AssignmentPropagation => 8u8.hash(hasher),
        DerivationRule::ReturnTypeCheck => 9u8.hash(hasher),
        DerivationRule::InternalBlocked { reason } => {
            10u8.hash(hasher);
            reason.hash(hasher);
        }
    }
}

fn hash_evidence_ref(evidence: &EvidenceRef, hasher: &mut impl Hasher) {
    match evidence {
        EvidenceRef::SourceSpan(range) => {
            0u8.hash(hasher);
            hash_range(*range, hasher);
        }
        EvidenceRef::TypeId(ty) => {
            1u8.hash(hasher);
            ty.hash(hasher);
        }
        EvidenceRef::CallResolution(call) => {
            2u8.hash(hasher);
            call.hash(hasher);
        }
        EvidenceRef::BindingVersion { binding, version } => {
            3u8.hash(hasher);
            binding.hash(hasher);
            version.hash(hasher);
        }
        EvidenceRef::Suppressed { cause } => {
            4u8.hash(hasher);
            cause.hash(hasher);
        }
    }
}

fn hash_explanation_step(step: &ExplanationStep, hasher: &mut impl Hasher) {
    match step {
        ExplanationStep::Literal { expression, ty } => {
            0u8.hash(hasher);
            expression.hash(hasher);
            ty.hash(hasher);
        }
        ExplanationStep::Declared { binding, range, ty } => {
            1u8.hash(hasher);
            binding.hash(hasher);
            hash_range(*range, hasher);
            ty.hash(hasher);
        }
        ExplanationStep::MethodCall { call, callable, return_ty } => {
            2u8.hash(hasher);
            call.hash(hasher);
            callable.hash(hasher);
            return_ty.hash(hasher);
        }
        ExplanationStep::FlowRefinement { binding, prior, refined } => {
            3u8.hash(hasher);
            binding.hash(hasher);
            hash_type_knowledge(prior, true, hasher);
            hash_type_knowledge(refined, true, hasher);
        }
        ExplanationStep::BranchJoin { binding, branches, joined } => {
            4u8.hash(hasher);
            binding.hash(hasher);
            branches.len().hash(hasher);
            for branch in branches {
                hash_type_knowledge(branch, true, hasher);
            }
            hash_type_knowledge(joined, true, hasher);
        }
        ExplanationStep::Subtyping { actual, expected, proven } => {
            5u8.hash(hasher);
            actual.hash(hasher);
            expected.hash(hasher);
            proven.hash(hasher);
        }
    }
}

fn hash_explanation_arena(arena: &ExplanationArena, roots: impl IntoIterator<Item = ExplanationId>, hasher: &mut impl Hasher) {
    let mut reachable = BTreeSet::new();
    let mut pending = roots.into_iter().collect::<Vec<_>>();
    while let Some(id) = pending.pop() {
        if !reachable.insert(id) {
            continue;
        }
        if let Some(node) = arena.get(id) {
            pending.extend(node.parents.iter().copied());
        }
    }

    reachable.len().hash(hasher);
    for id in reachable {
        id.hash(hasher);
        match arena.get(id) {
            Some(node) => {
                true.hash(hasher);
                node.id.hash(hasher);
                hash_explanation_step(&node.step, hasher);
                hash_derivation_rule(&node.rule, hasher);
                node.authority.hash(hasher);
                node.evidence.len().hash(hasher);
                for evidence in &node.evidence {
                    hash_evidence_ref(evidence, hasher);
                }
                node.parents.hash(hasher);
            }
            None => false.hash(hasher),
        }
    }
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
    diagnostic.root_cause.hash(hasher);
}

fn hash_callable_analysis_status(status: CallableAnalysisStatus, hasher: &mut impl Hasher) {
    match status {
        CallableAnalysisStatus::Complete => 0u8.hash(hasher),
        CallableAnalysisStatus::Partial => 1u8.hash(hasher),
        CallableAnalysisStatus::Blocked => 2u8.hash(hasher),
        CallableAnalysisStatus::Cancelled => 3u8.hash(hasher),
        CallableAnalysisStatus::BudgetExceeded => 4u8.hash(hasher),
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
pub fn declaration_surface_query_input_fingerprint(
    surface: &DeclarationSurface,
    diagnostics: &[SemanticDiagnostic],
) -> InputFingerprint {
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
pub fn callable_body_input_fingerprint(
    callable: &CallableId,
    body: &[Statement],
    body_range: SourceRange,
    store: &TypeStore,
) -> InputFingerprint {
    let mut hasher = DefaultHasher::new();
    callable.hash(&mut hasher);
    hash_range(body_range, &mut hasher);
    store.id().hash(&mut hasher);
    // Body input identity is intentionally syntax-sensitive. Product hashing
    // below never relies on Debug representation; a future parser-owned syntax
    // fingerprint can replace this without affecting dependency semantics.
    for statement in body {
        format!("{statement:?}").hash(&mut hasher);
    }
    finish_input(hasher)
}

/// Computes product fingerprint for a callable body analysis result.
pub fn callable_body_product_fingerprint(analysis: &CallableAnalysis) -> ProductFingerprint {
    let mut hasher = DefaultHasher::new();
    analysis.callable.hash(&mut hasher);
    hash_range(analysis.body_range, &mut hasher);

    analysis.expressions.len().hash(&mut hasher);
    for (expression_id, expression) in &analysis.expressions {
        expression_id.hash(&mut hasher);
        expression.id.hash(&mut hasher);
        hash_range(expression.range, &mut hasher);
        hash_type_knowledge(&expression.knowledge, true, &mut hasher);
        hash_denotation(&expression.denotation, &mut hasher);
        hash_analysis_status(&expression.status, &mut hasher);
        expression.explanation.hash(&mut hasher);
        expression.call.hash(&mut hasher);
    }

    analysis.bindings.len().hash(&mut hasher);
    for (binding_id, binding) in &analysis.bindings {
        binding_id.hash(&mut hasher);
        binding.binding.hash(&mut hasher);
        binding.name.hash(&mut hasher);
        hash_range(binding.range, &mut hasher);
        binding.declared.hash(&mut hasher);
        hash_type_knowledge(&binding.current, true, &mut hasher);
        binding.mutable.hash(&mut hasher);
        binding.version.hash(&mut hasher);
        binding.explanation.hash(&mut hasher);
    }

    hash_flow_graph(&analysis.flow_graph, &mut hasher);
    hash_flow_summary(&analysis.entry_flow, &mut hasher);
    hash_exit_facts(&analysis.exits, &mut hasher);

    analysis.diagnostics.len().hash(&mut hasher);
    for diagnostic in analysis.diagnostics.iter() {
        hash_semantic_diagnostic(diagnostic, &mut hasher);
    }

    let explanation_roots = analysis
        .expressions
        .values()
        .filter_map(|expression| expression.explanation)
        .chain(analysis.bindings.values().filter_map(|binding| binding.explanation))
        .chain(analysis.diagnostics.iter().flat_map(|diagnostic| diagnostic.explanations.iter().copied()));
    hash_explanation_arena(&analysis.explanations, explanation_roots, &mut hasher);

    analysis.dependencies.hash(&mut hasher);
    analysis.semantic_dependencies.hash(&mut hasher);
    hash_callable_analysis_status(analysis.status, &mut hasher);
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
