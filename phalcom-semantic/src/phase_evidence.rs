//! Semantic phase evidence aggregation and generated runtime requirement extraction.
//!
//! Semantic analysis owns the proof of whether an imported binding is interface-only
//! or runtime (ruling 2026-10-13 §4.2). This module joins observed use phases
//! across callable bodies, signatures, declarations, and AST structures into
//! canonical `ModuleImportPhaseProduct`s consumed by executable finalization.

use crate::checker::analysis::CallableAnalysis;
use crate::identity::{CallableId, DeclarationId, ModuleId};
use crate::prelude::PreludeTypeMap;
use crate::source::ParsedModuleUnit;
use crate::source_index::{OccurrenceHint, OccurrenceRole, SourceDeclarationKind, SourceNameResolution, SourceSemanticIndex};
use crate::static_constant::StaticConstantTable;
use crate::types::store::TypeStore;
use phalcom_ast::ast::{
    AttrKind, Attribute, BehaviorMember, CasePayloadSyntax, ClassMember, Expr, GenericArgumentSyntax, GenericConstraintSyntax, ImplKind, Statement,
    TraitMember, TraitRequirementSyntax, TypeAnnotation, TypeAnnotationExpr, TypeCallableItem, TypeTupleItem, WhereClauseSyntax,
};
use phalcom_modules::graph::{DependencyPhase, ReferenceKind, RuntimeDependencyReason};
use phalcom_modules::identity::SourceImportBindingId;
use phalcom_modules::linker::{
    ImportPhaseSummary, LinkedReadSpec, ModuleImportPhaseProduct, PhaseEvidenceCoverage, RuntimeDependencyRequirement, SemanticLinkedProgram, SymbolId,
    SymbolicImportTarget,
};
use phalcom_modules::{
    RuntimeRequirementProducer, RuntimeRequirementProduct, RuntimeRequirementProvenance, RuntimeRequirementTarget, RuntimeRequirementTiming,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

/// Read-only callable-analysis input accepted by phase-product projection.
pub trait CallableAnalysisCollection {
    fn for_each_analysis(&self, visitor: &mut dyn FnMut(&CallableAnalysis));
}

impl CallableAnalysisCollection for HashMap<CallableId, Arc<CallableAnalysis>> {
    fn for_each_analysis(&self, visitor: &mut dyn FnMut(&CallableAnalysis)) {
        self.values().for_each(|analysis| visitor(analysis));
    }
}

impl CallableAnalysisCollection for crate::checker::CallableAnalysisTable {
    fn for_each_analysis(&self, visitor: &mut dyn FnMut(&CallableAnalysis)) {
        self.values().for_each(|analysis| visitor(analysis));
    }
}

/// Collects root symbol names referenced by a [`TypeAnnotation`].
pub fn collect_type_annotation_roots(annotation: &TypeAnnotation, roots: &mut Vec<String>) {
    match &annotation.expr {
        TypeAnnotationExpr::Reference(sym) => roots.push(sym.root.clone()),
        TypeAnnotationExpr::Application { origin, arguments, .. } => {
            collect_type_annotation_roots(origin, roots);
            for arg in arguments {
                if let GenericArgumentSyntax::Single(ty) = arg {
                    collect_type_annotation_roots(ty, roots);
                }
            }
        }
        TypeAnnotationExpr::ExactEnumCase {
            enum_target,
            generic_arguments,
            ..
        } => {
            collect_type_annotation_roots(enum_target, roots);
            for arg in generic_arguments {
                collect_type_annotation_roots(arg, roots);
            }
        }
        TypeAnnotationExpr::Union { members, .. } => {
            for member in members {
                collect_type_annotation_roots(member, roots);
            }
        }
        TypeAnnotationExpr::Tuple { elements, .. } => {
            for elem in elements {
                if let TypeTupleItem::Element(el) = elem {
                    collect_type_annotation_roots(&el.ty, roots);
                }
            }
        }
        TypeAnnotationExpr::Callable { parameters, result, .. } => {
            for param in parameters {
                if let TypeCallableItem::Parameter(p) = param {
                    collect_type_annotation_roots(&p.ty, roots);
                }
            }
            collect_type_annotation_roots(result, roots);
        }
        TypeAnnotationExpr::AssociatedTypeProjection { subject, .. } => {
            collect_type_annotation_roots(subject, roots);
        }
        TypeAnnotationExpr::QualifiedAssociatedTypeProjection { subject, trait_ref, .. } => {
            collect_type_annotation_roots(subject, roots);
            collect_trait_requirement_roots(trait_ref, roots);
        }
        TypeAnnotationExpr::Record { fields, .. } => {
            for field in fields {
                collect_type_annotation_roots(&field.ty, roots);
            }
        }
        _ => {}
    }
}

/// Collects root symbol names referenced by a [`TraitRequirementSyntax`].
pub fn collect_trait_requirement_roots(trait_ref: &TraitRequirementSyntax, roots: &mut Vec<String>) {
    collect_type_annotation_roots(&trait_ref.trait_type, roots);
    for assoc in &trait_ref.associated_specializations {
        collect_type_annotation_roots(&assoc.value, roots);
    }
}

/// Collects root symbol names referenced by a [`WhereClauseSyntax`].
pub fn collect_where_clause_roots(where_clause: &WhereClauseSyntax, roots: &mut Vec<String>) {
    for constraint in &where_clause.constraints {
        match constraint {
            GenericConstraintSyntax::Subtype { lower, upper, .. } => {
                collect_type_annotation_roots(lower, roots);
                collect_type_annotation_roots(upper, roots);
            }
            GenericConstraintSyntax::Equivalent { left, right, .. } => {
                collect_type_annotation_roots(left, roots);
                collect_type_annotation_roots(right, roots);
            }
            GenericConstraintSyntax::Conforms { subject, bounds, .. } => {
                collect_type_annotation_roots(subject, roots);
                for bound in bounds {
                    collect_trait_requirement_roots(bound, roots);
                }
            }
            _ => {}
        }
    }
}

fn collect_behavior_member_roots(behavior: &BehaviorMember, roots: &mut Vec<String>) {
    match behavior {
        BehaviorMember::Method(m) => {
            for param in &m.params {
                if let Some(ann) = param.type_annotation() {
                    collect_type_annotation_roots(ann, roots);
                }
            }
            if let Some(ret) = &m.return_annotation {
                collect_type_annotation_roots(ret, roots);
            }
            if let Some(wc) = &m.where_clause {
                collect_where_clause_roots(wc, roots);
            }
        }
        BehaviorMember::Getter(g) => {
            if let Some(ret) = &g.return_annotation {
                collect_type_annotation_roots(ret, roots);
            }
            if let Some(wc) = &g.where_clause {
                collect_where_clause_roots(wc, roots);
            }
        }
        BehaviorMember::Setter(s) => {
            if let Some(ann) = s.param.type_annotation() {
                collect_type_annotation_roots(ann, roots);
            }
            if let Some(ret) = &s.return_annotation {
                collect_type_annotation_roots(ret, roots);
            }
            if let Some(wc) = &s.where_clause {
                collect_where_clause_roots(wc, roots);
            }
        }
        BehaviorMember::Index(idx) => {
            for param in &idx.params {
                if let Some(ann) = param.type_annotation() {
                    collect_type_annotation_roots(ann, roots);
                }
            }
            if let Some(ret) = &idx.return_annotation {
                collect_type_annotation_roots(ret, roots);
            }
            if let Some(wc) = &idx.where_clause {
                collect_where_clause_roots(wc, roots);
            }
        }
    }
}

fn push_runtime_requirement(
    importer: &ModuleId,
    dependency: &ModuleId,
    range: phalcom_common::range::SourceRange,
    read_target: Option<LinkedReadSpec>,
    producer: phalcom_modules::RuntimeRequirementProducer,
    requirements: &mut Vec<RuntimeDependencyRequirement>,
) {
    if importer == dependency {
        return;
    }
    let target = match read_target.as_ref() {
        Some(LinkedReadSpec::Module(module)) => phalcom_modules::RuntimeRequirementTarget::Module(module.clone()),
        Some(LinkedReadSpec::Binding(symbol)) => phalcom_modules::RuntimeRequirementTarget::Binding {
            module: symbol.module.clone(),
            name: symbol.name.clone(),
        },
        None => phalcom_modules::RuntimeRequirementTarget::Module(dependency.clone()),
    };
    let product = target.conservative_product();
    let requirement = RuntimeDependencyRequirement {
        importer: importer.clone(),
        dependency: dependency.clone(),
        range,
        reason: RuntimeDependencyReason::RuntimeDeclarationReference,
        read_target,
        provenance: phalcom_modules::RuntimeRequirementProvenance {
            consumer: importer.clone(),
            producer,
            target,
            product,
            timing: phalcom_modules::RuntimeRequirementTiming::DuringModuleInitializer,
        },
    };
    // Several checked expression views can project the same declaration read.
    // Only this producer owns that overlap; other mechanisms keep their own
    // publication rules and distinct authority must remain visible.
    if producer != RuntimeRequirementProducer::DeclarationRead || !requirements.contains(&requirement) {
        requirements.push(requirement);
    }
}

fn push_declaration_read_requirement(
    importer: &ModuleId,
    declaration: &crate::identity::DeclarationId,
    range: phalcom_common::range::SourceRange,
    requirements: &mut Vec<RuntimeDependencyRequirement>,
) {
    push_runtime_requirement(
        importer,
        &declaration.module,
        range,
        Some(LinkedReadSpec::Binding(SymbolId {
            module: declaration.module.clone(),
            name: declaration.name.clone(),
        })),
        phalcom_modules::RuntimeRequirementProducer::DeclarationRead,
        requirements,
    );
}

fn push_import_read_requirement(
    importer: &ModuleId,
    target: &SymbolicImportTarget,
    range: phalcom_common::range::SourceRange,
    source_index: &SourceSemanticIndex,
    requirements: &mut Vec<RuntimeDependencyRequirement>,
) {
    let (dependency, read_target) = match target {
        SymbolicImportTarget::Module(module) => (module.clone(), LinkedReadSpec::Module(module.clone())),
        SymbolicImportTarget::Binding(symbol) => (symbol.module.clone(), LinkedReadSpec::Binding(symbol.clone())),
    };
    let start_len = requirements.len();
    push_runtime_requirement(
        importer,
        &dependency,
        range,
        Some(read_target),
        phalcom_modules::RuntimeRequirementProducer::ImportRead,
        requirements,
    );
    if requirements.len() == start_len {
        return;
    }
    if let SymbolicImportTarget::Binding(symbol) = target {
        let declaration = DeclarationId::new(symbol.module.clone(), symbol.name.clone());
        if source_index
            .declaration_source(&declaration)
            .is_some_and(|source| matches!(source.kind, SourceDeclarationKind::Trait | SourceDeclarationKind::TypeAlias))
        {
            let provenance = &mut requirements.last_mut().expect("runtime import requirement was appended").provenance;
            provenance.product = phalcom_modules::RuntimeRequirementProduct::CanonicalIdentity;
            provenance.timing = phalcom_modules::RuntimeRequirementTiming::BeforeSourceExecution;
        }
    }
}

/// Projects source-visible references that the compiler resolves directly
/// through the canonical native binding catalog (for example `None`). The
/// spelling hint only selects a catalog row; lexical resolution must also say
/// the name is unresolved, matching the compiler's fallback order exactly.
fn add_native_prelude_read_requirements(importer: &ModuleId, source_index: &SourceSemanticIndex, requirements: &mut Vec<RuntimeDependencyRequirement>) {
    let Some(source) = source_index.module(importer) else {
        return;
    };
    let mut seen = BTreeSet::new();
    for occurrence in source.occurrences.all() {
        if matches!(occurrence.role, OccurrenceRole::Declaration | OccurrenceRole::Write) {
            continue;
        }
        let Some(OccurrenceHint::Name(name)) = occurrence.hint.as_ref() else {
            continue;
        };
        if source_index.target_for(&occurrence.site).is_some()
            || !matches!(
                source
                    .structure
                    .resolve_name(source.structure.scope_at(occurrence.range.start), name, occurrence.range.start),
                SourceNameResolution::Unresolved
            )
        {
            continue;
        }
        let Some(binding) = phalcom_native_meta::UNIVERSE_BINDINGS.iter().find(|binding| binding.name == name.as_ref()) else {
            continue;
        };
        let components = binding
            .key
            .source_path()
            .iter()
            .map(|component| phalcom_modules::ModuleComponent::from_identifier(component).ok())
            .collect::<Option<Vec<_>>>();
        let Some(components) = components else {
            continue;
        };
        let dependency = ModuleId::universe(phalcom_modules::ModulePath::from_components(components));
        let target = RuntimeRequirementTarget::Binding {
            module: dependency.clone(),
            name: binding.name.into(),
        };
        if !seen.insert((target.clone(), occurrence.range)) {
            continue;
        }
        let requirement = RuntimeDependencyRequirement {
            importer: importer.clone(),
            dependency: dependency.clone(),
            range: occurrence.range,
            reason: RuntimeDependencyReason::RuntimeDeclarationReference,
            read_target: Some(LinkedReadSpec::Binding(SymbolId {
                module: dependency.clone(),
                name: binding.name.into(),
            })),
            provenance: RuntimeRequirementProvenance {
                consumer: importer.clone(),
                producer: RuntimeRequirementProducer::CompilerGeneratedRead,
                target,
                product: RuntimeRequirementProduct::RuntimeObject,
                timing: RuntimeRequirementTiming::DuringModuleInitializer,
            },
        };
        // A checked type-form read owns publication when it already describes
        // this exact native binding and availability requirement. The catalog
        // projection is only a fallback; a shared source span alone is not proof
        // that two reads have the same authority.
        if requirements.iter().any(|existing| {
            existing.provenance.producer == RuntimeRequirementProducer::DeclarationRead
                && existing.importer == requirement.importer
                && existing.dependency == requirement.dependency
                && existing.range == requirement.range
                && existing.reason == requirement.reason
                && existing.read_target == requirement.read_target
                && existing.provenance.consumer == requirement.provenance.consumer
                && existing.provenance.target == requirement.provenance.target
                && existing.provenance.product == requirement.provenance.product
                && existing.provenance.timing == requirement.provenance.timing
        }) {
            continue;
        }
        requirements.push(requirement);
    }
}

fn push_compiler_generated_read_requirement(
    importer: &ModuleId,
    name: &str,
    range: phalcom_common::range::SourceRange,
    prelude: &PreludeTypeMap,
    requirements: &mut Vec<RuntimeDependencyRequirement>,
) -> bool {
    let declaration = match name {
        "Error" => prelude.get("Error").cloned(),
        "ZipLengthMismatchError" => Some(crate::identity::DeclarationId::new(
            crate::core_surface::identity::universe_module(&["errors", "iteration"]),
            "ZipLengthMismatchError".into(),
        )),
        "PreconditionError" | "PostconditionError" | "InvariantError" => Some(crate::identity::DeclarationId::new(
            crate::core_surface::identity::universe_module(&["errors", "contracts"]),
            name.into(),
        )),
        "Tuple" | "List" | "Record" | "Map" => prelude.get(name).cloned(),
        _ => None,
    };
    if let Some(declaration) = declaration {
        push_declaration_read_requirement(importer, &declaration, range, requirements);
        true
    } else {
        false
    }
}

fn push_callable_owner_requirement(
    importer: &ModuleId,
    dependency: &ModuleId,
    range: phalcom_common::range::SourceRange,
    requirements: &mut Vec<RuntimeDependencyRequirement>,
) {
    if importer == dependency {
        return;
    }
    let requirement = RuntimeDependencyRequirement {
        importer: importer.clone(),
        dependency: dependency.clone(),
        range,
        reason: RuntimeDependencyReason::RuntimeDeclarationReference,
        read_target: None,
        provenance: RuntimeRequirementProvenance {
            consumer: importer.clone(),
            producer: phalcom_modules::RuntimeRequirementProducer::CallableOwner,
            target: RuntimeRequirementTarget::Module(dependency.clone()),
            product: phalcom_modules::RuntimeRequirementProduct::CanonicalIdentity,
            timing: phalcom_modules::RuntimeRequirementTiming::BeforeSourceExecution,
        },
    };
    // A selected callable, its specialization, and its impl can share an owner.
    // They require one identical module identity capability at this source site.
    if !requirements.contains(&requirement) {
        requirements.push(requirement);
    }
}

fn push_invocation_target_owner_requirement(
    importer: &ModuleId,
    target: &crate::identity::InvocationTargetId,
    range: phalcom_common::range::SourceRange,
    requirements: &mut Vec<RuntimeDependencyRequirement>,
) {
    let owner = match target {
        crate::identity::InvocationTargetId::Behavioral(callable) => callable.module(),
        crate::identity::InvocationTargetId::VariantConstructor(constructor) => &constructor.variant.owner.module,
        crate::identity::InvocationTargetId::DataConstructor(constructor) => &constructor.owner.module,
    };
    push_callable_owner_requirement(importer, owner, range, requirements);
}

fn add_associated_resolution_owner_requirements(
    importer: &ModuleId,
    resolution: &crate::checker::associated::AssociatedResolution,
    range: phalcom_common::range::SourceRange,
    requirements: &mut Vec<RuntimeDependencyRequirement>,
) {
    use crate::checker::associated::{AssociatedResolutionKind, SpecializedAssociatedMember};

    fn add_member_owner(
        importer: &ModuleId,
        member: &crate::associated::AssociatedMemberId,
        range: phalcom_common::range::SourceRange,
        requirements: &mut Vec<RuntimeDependencyRequirement>,
    ) {
        let owner = match member {
            crate::associated::AssociatedMemberId::Variant(variant) => Some(&variant.owner.module),
            crate::associated::AssociatedMemberId::DataConstructor(constructor) => Some(&constructor.owner.module),
            crate::associated::AssociatedMemberId::DataComponent(_)
            | crate::associated::AssociatedMemberId::StaticDomain(_)
            | crate::associated::AssociatedMemberId::TypeMember(_) => None,
        };
        if let Some(owner) = owner {
            push_callable_owner_requirement(importer, owner, range, requirements);
        }
    }

    fn add_specialized_member(
        importer: &ModuleId,
        member: &SpecializedAssociatedMember,
        range: phalcom_common::range::SourceRange,
        requirements: &mut Vec<RuntimeDependencyRequirement>,
    ) {
        add_member_owner(importer, &member.member, range, requirements);
        if let Some(target) = &member.target {
            push_invocation_target_owner_requirement(importer, target, range, requirements);
        }
    }

    match &resolution.kind {
        AssociatedResolutionKind::ExactValue { member, .. } => add_member_owner(importer, member, range, requirements),
        AssociatedResolutionKind::ExactCallable { target, .. } | AssociatedResolutionKind::StaticInvoke { target, .. } => {
            push_invocation_target_owner_requirement(importer, target, range, requirements);
        }
        AssociatedResolutionKind::Family { members, .. } => {
            for member in members.iter() {
                add_specialized_member(importer, member, range, requirements);
            }
        }
        AssociatedResolutionKind::DynamicInvoke { candidates, .. } => {
            for candidate in candidates.iter() {
                add_specialized_member(importer, candidate, range, requirements);
            }
        }
        AssociatedResolutionKind::DataExpansionField { .. } | AssociatedResolutionKind::StaticDomain { .. } | AssociatedResolutionKind::StaticTerm { .. } => {}
    }
}

fn add_family_application_owner_requirements(
    importer: &ModuleId,
    resolution: &crate::checker::associated::FamilyApplicationResolution,
    range: phalcom_common::range::SourceRange,
    requirements: &mut Vec<RuntimeDependencyRequirement>,
) {
    use crate::checker::associated::FamilyApplicationSelection;

    let mut add_target = |target: &Option<crate::identity::InvocationTargetId>| {
        if let Some(target) = target {
            push_invocation_target_owner_requirement(importer, target, range, requirements);
        }
    };
    match &resolution.selection {
        FamilyApplicationSelection::Static { target, .. } => add_target(target),
        FamilyApplicationSelection::Dynamic { candidates, .. } => {
            for candidate in candidates.iter() {
                add_target(&candidate.target);
            }
        }
    }
}

fn push_binding_owner_requirement(
    importer: &ModuleId,
    declaration: &DeclarationId,
    range: phalcom_common::range::SourceRange,
    producer: phalcom_modules::RuntimeRequirementProducer,
    requirements: &mut Vec<RuntimeDependencyRequirement>,
) {
    if importer == &declaration.module {
        return;
    }
    let target = RuntimeRequirementTarget::Binding {
        module: declaration.module.clone(),
        name: declaration.name.clone(),
    };
    let requirement = RuntimeDependencyRequirement {
        importer: importer.clone(),
        dependency: declaration.module.clone(),
        range,
        reason: RuntimeDependencyReason::RuntimeDeclarationReference,
        read_target: None,
        provenance: RuntimeRequirementProvenance {
            consumer: importer.clone(),
            producer,
            product: target.conservative_product(),
            target,
            timing: phalcom_modules::RuntimeRequirementTiming::DuringModuleInitializer,
        },
    };
    // Pattern sites/resolved patterns and expression/iteration conformance
    // views can observe the same owner binding. Coalesce exact repetitions
    // within those producer families only; other producers remain independent.
    if !matches!(producer, RuntimeRequirementProducer::PatternOwner | RuntimeRequirementProducer::ConformanceTypeOwner)
        || !requirements.contains(&requirement)
    {
        requirements.push(requirement);
    }
}

fn add_conformance_type_owner_requirements(
    importer: &ModuleId,
    range: phalcom_common::range::SourceRange,
    evidence: &crate::generic_evidence::GenericConformanceEvidence,
    type_store: &TypeStore,
    requirements: &mut Vec<RuntimeDependencyRequirement>,
) {
    let bindings = match evidence {
        crate::generic_evidence::GenericConformanceEvidence::Exact(evidence) => evidence.impl_environment.bindings.values().copied().collect::<Vec<_>>(),
        crate::generic_evidence::GenericConformanceEvidence::Derived(evidence) => evidence.impl_bindings.bindings().values().copied().collect(),
        crate::generic_evidence::GenericConformanceEvidence::Assumed(_) | crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(_) => Vec::new(),
    };
    for ty in bindings {
        if let Some(declaration) = type_store.nominal_origin_declaration(ty) {
            push_binding_owner_requirement(
                importer,
                declaration,
                range,
                phalcom_modules::RuntimeRequirementProducer::ConformanceTypeOwner,
                requirements,
            );
        }
    }
}

fn add_pattern_requirements<R, S, K>(
    importer: &ModuleId,
    range: phalcom_common::range::SourceRange,
    pattern: &crate::match_semantics::PatternResolution<R, S, K>,
    prelude: &PreludeTypeMap,
    requirements: &mut Vec<RuntimeDependencyRequirement>,
    complete: &mut bool,
) {
    use crate::match_semantics::PatternResolution;

    match pattern {
        PatternResolution::Variant(resolved) => {
            for owner in resolved.owner_candidates.iter() {
                push_binding_owner_requirement(importer, owner, range, phalcom_modules::RuntimeRequirementProducer::PatternOwner, requirements);
            }
            for candidate in resolved.candidates.iter() {
                push_binding_owner_requirement(
                    importer,
                    &candidate.variant.owner,
                    range,
                    phalcom_modules::RuntimeRequirementProducer::PatternOwner,
                    requirements,
                );
                for field in candidate.fields.iter() {
                    add_pattern_requirements(importer, range, &field.child, prelude, requirements, complete);
                }
            }
        }
        PatternResolution::Or(resolved) => {
            for alternative in resolved.alternatives.iter() {
                add_pattern_requirements(importer, range, alternative, prelude, requirements, complete);
            }
        }
        PatternResolution::Tuple(items) => {
            add_pattern_class_requirement(importer, "Tuple", range, prelude, requirements, complete);
            add_pattern_class_requirement(importer, "Error", range, prelude, requirements, complete);
            for item in items.iter() {
                add_pattern_requirements(importer, range, item, prelude, requirements, complete);
            }
        }
        PatternResolution::List(list) => {
            add_pattern_class_requirement(importer, "List", range, prelude, requirements, complete);
            add_pattern_class_requirement(importer, "Error", range, prelude, requirements, complete);
            for item in list.prefix.iter() {
                add_pattern_requirements(importer, range, item, prelude, requirements, complete);
            }
            if let Some(rest) = list.rest.as_deref() {
                add_pattern_requirements(importer, range, rest, prelude, requirements, complete);
            }
        }
        PatternResolution::Record(fields) => {
            add_pattern_class_requirement(importer, "Record", range, prelude, requirements, complete);
            add_pattern_class_requirement(importer, "Error", range, prelude, requirements, complete);
            for field in fields.iter() {
                add_pattern_requirements(importer, range, &field.child, prelude, requirements, complete);
            }
        }
        PatternResolution::Map(entries) => {
            add_pattern_class_requirement(importer, "Map", range, prelude, requirements, complete);
            add_pattern_class_requirement(importer, "Error", range, prelude, requirements, complete);
            for entry in entries.iter() {
                add_pattern_requirements(importer, range, &entry.child, prelude, requirements, complete);
            }
        }
        PatternResolution::Wildcard | PatternResolution::Char(_) | PatternResolution::Binding { .. } => {}
    }
}

fn add_pattern_class_requirement(
    importer: &ModuleId,
    name: &str,
    range: phalcom_common::range::SourceRange,
    prelude: &PreludeTypeMap,
    requirements: &mut Vec<RuntimeDependencyRequirement>,
    complete: &mut bool,
) {
    if let Some(declaration) = prelude.get(name) {
        push_declaration_read_requirement(importer, declaration, range, requirements);
    } else {
        *complete = false;
    }
}

/// Collects source roots in an executable attribute argument. Attribute
/// expressions are compiled by core but are not part of callable analysis, so
/// this syntax walk supplies only canonical prelude read targets; imported
/// names are still conservatively retained as Runtime below.
fn collect_attribute_pack_roots(items: &[phalcom_ast::ast::PackItem], roots: &mut Vec<String>, complete: &mut bool) {
    for item in items {
        match item {
            phalcom_ast::ast::PackItem::Positional { expr, .. }
            | phalcom_ast::ast::PackItem::Labeled { value: expr, .. }
            | phalcom_ast::ast::PackItem::Expand { expr, .. } => collect_attribute_expr_roots(expr, roots, complete),
        }
    }
}

fn collect_attribute_expr_roots(expression: &Expr, roots: &mut Vec<String>, complete: &mut bool) {
    use phalcom_ast::ast::{BracketLiteralElement, IfElseBody, MapLiteralKey, MatchArmBody, RecordLiteralEntry, SetLiteralEntry, TupleLiteralEntry};

    match expression {
        Expr::Var { value, .. } => roots.push(value.clone()),
        Expr::UnqualifiedCall(call) => {
            roots.push(call.name.clone());
            collect_attribute_pack_roots(&call.args, roots, complete);
        }
        Expr::TypeForm(annotation) => collect_type_annotation_roots(annotation, roots),
        Expr::Assignment(assignment) => {
            collect_attribute_expr_roots(&assignment.name, roots, complete);
            collect_attribute_expr_roots(&assignment.value, roots, complete);
        }
        Expr::Range(range) => {
            if let Some(lower) = &range.lower {
                collect_attribute_expr_roots(lower, roots, complete);
            }
            if let Some(upper) = &range.upper {
                collect_attribute_expr_roots(upper, roots, complete);
            }
        }
        Expr::Unary(unary) => collect_attribute_expr_roots(&unary.expr, roots, complete),
        Expr::Binary(binary) => {
            collect_attribute_expr_roots(&binary.left, roots, complete);
            collect_attribute_expr_roots(&binary.right, roots, complete);
        }
        Expr::ComparisonChain(chain) => {
            for operand in &chain.operands {
                collect_attribute_expr_roots(operand, roots, complete);
            }
        }
        Expr::IfLet(expr) => {
            *complete = false;
            collect_pattern_syntax_roots(&expr.pattern, roots);
            collect_attribute_expr_roots(&expr.value, roots, complete);
            collect_attribute_block_roots(&expr.then_body.body, roots, complete);
            if let Some(else_body) = &expr.else_body {
                collect_attribute_block_roots(&else_body.body, roots, complete);
            }
        }
        Expr::WhileLet(expr) => {
            *complete = false;
            collect_pattern_syntax_roots(&expr.pattern, roots);
            collect_attribute_expr_roots(&expr.value, roots, complete);
            collect_attribute_block_roots(&expr.body, roots, complete);
        }
        Expr::If(expr) => {
            collect_attribute_expr_roots(&expr.condition, roots, complete);
            collect_attribute_block_roots(&expr.then_body.body, roots, complete);
            if let Some(else_body) = &expr.else_body {
                match else_body {
                    IfElseBody::Block(block) => collect_attribute_block_roots(&block.body, roots, complete),
                    IfElseBody::ElseIf(expr) => collect_attribute_expr_roots(expr, roots, complete),
                }
            }
        }
        Expr::While(expr) => {
            collect_attribute_expr_roots(&expr.condition, roots, complete);
            collect_attribute_block_roots(&expr.body.body, roots, complete);
        }
        Expr::MethodCall(call) => {
            collect_attribute_expr_roots(&call.object, roots, complete);
            collect_attribute_pack_roots(&call.args, roots, complete);
        }
        Expr::GetProperty(property) => collect_attribute_expr_roots(&property.object, roots, complete),
        Expr::SetProperty(property) => {
            collect_attribute_expr_roots(&property.object, roots, complete);
            collect_attribute_expr_roots(&property.value, roots, complete);
        }
        Expr::Index(index) => {
            collect_attribute_expr_roots(&index.object, roots, complete);
            collect_attribute_pack_roots(&index.args, roots, complete);
        }
        Expr::SetIndex(index) => {
            collect_attribute_expr_roots(&index.object, roots, complete);
            collect_attribute_pack_roots(&index.args, roots, complete);
            collect_attribute_expr_roots(&index.value, roots, complete);
        }
        Expr::Block(block) => {
            *complete = false;
            collect_attribute_block_roots(&block.body, roots, complete);
        }
        Expr::AssociatedLookup(lookup) => collect_attribute_expr_roots(&lookup.receiver, roots, complete),
        Expr::AssociatedInvoke(invoke) => {
            collect_attribute_expr_roots(&invoke.receiver, roots, complete);
            collect_attribute_pack_roots(&invoke.args, roots, complete);
        }
        Expr::CallableReference(reference) => {
            *complete = false;
            match &reference.target {
                phalcom_ast::ast::CallableReferenceTarget::Bound { receiver, .. }
                | phalcom_ast::ast::CallableReferenceTarget::Associated { receiver, .. }
                | phalcom_ast::ast::CallableReferenceTarget::PrimaryDataConstructor { receiver, .. } => {
                    collect_attribute_expr_roots(receiver, roots, complete);
                }
            }
        }
        Expr::TupleLiteral(tuple) => {
            for entry in &tuple.entries {
                match entry {
                    TupleLiteralEntry::Positional { expr, .. } | TupleLiteralEntry::Expand { expr, .. } => {
                        collect_attribute_expr_roots(expr, roots, complete);
                    }
                    TupleLiteralEntry::Labeled { label, value, .. } => {
                        if let phalcom_ast::ast::ProductLabel::Computed { expr, .. } = label {
                            collect_attribute_expr_roots(expr, roots, complete);
                        }
                        collect_attribute_expr_roots(value, roots, complete);
                    }
                }
            }
        }
        Expr::RecordLiteral(record) => {
            for entry in &record.entries {
                match entry {
                    RecordLiteralEntry::Field(field) => {
                        if let phalcom_ast::ast::ProductLabel::Computed { expr, .. } = &field.label {
                            collect_attribute_expr_roots(expr, roots, complete);
                        }
                        collect_attribute_expr_roots(&field.value, roots, complete);
                    }
                    RecordLiteralEntry::Expansion { expr, .. } => collect_attribute_expr_roots(expr, roots, complete),
                }
            }
        }
        Expr::RecordConstruction(record) => {
            roots.push(record.target.root.clone());
            for annotation in &record.type_arguments {
                collect_type_annotation_roots(annotation, roots);
            }
            for entry in &record.entries {
                collect_attribute_expr_roots(&entry.value, roots, complete);
            }
        }
        Expr::MapLiteral(map) => {
            roots.push(map.head.root.clone());
            if let Some(arguments) = &map.type_arguments {
                for annotation in arguments {
                    collect_type_annotation_roots(annotation, roots);
                }
            }
            for entry in &map.entries {
                match entry {
                    phalcom_ast::ast::MapLiteralEntry::Association { key, value, .. } => {
                        if let MapLiteralKey::Computed { expr, .. } = key {
                            collect_attribute_expr_roots(expr, roots, complete);
                        }
                        collect_attribute_expr_roots(value, roots, complete);
                    }
                    phalcom_ast::ast::MapLiteralEntry::Expansion { expr, .. } => collect_attribute_expr_roots(expr, roots, complete),
                }
            }
        }
        Expr::SetLiteral(set) => {
            roots.push("Set".to_owned());
            for entry in &set.entries {
                match entry {
                    SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => {
                        collect_attribute_expr_roots(expr, roots, complete);
                    }
                }
            }
        }
        Expr::BracketLiteral(list) => {
            roots.push("List".to_owned());
            for element in &list.elements {
                match element {
                    BracketLiteralElement::Element { expr, .. } | BracketLiteralElement::Expansion { expr, .. } => {
                        collect_attribute_expr_roots(expr, roots, complete);
                    }
                }
            }
        }
        Expr::Membership(expr) => {
            collect_attribute_expr_roots(&expr.left, roots, complete);
            collect_attribute_expr_roots(&expr.right, roots, complete);
        }
        Expr::IsMembership(expr) => {
            collect_attribute_expr_roots(&expr.left, roots, complete);
            collect_attribute_expr_roots(&expr.candidates, roots, complete);
        }
        Expr::Match(expr) => {
            *complete = false;
            collect_attribute_expr_roots(&expr.value, roots, complete);
            for arm in &expr.arms {
                collect_pattern_syntax_roots(&arm.pattern, roots);
                match &arm.branch {
                    MatchArmBody::Expression(expr) => collect_attribute_expr_roots(expr, roots, complete),
                    MatchArmBody::Block(block) => collect_attribute_block_roots(&block.body, roots, complete),
                }
            }
        }
        Expr::Char { .. }
        | Expr::Int { .. }
        | Expr::Float { .. }
        | Expr::Imaginary { .. }
        | Expr::String { .. }
        | Expr::Boolean { .. }
        | Expr::Field { .. }
        | Expr::SelfVar { .. }
        | Expr::SuperVar { .. }
        | Expr::Symbol(_)
        | Expr::Ellipsis { .. } => {}
    }
}

fn collect_attribute_block_roots(statements: &[Statement], roots: &mut Vec<String>, complete: &mut bool) {
    for statement in statements {
        match statement {
            Statement::Expr { expr, .. } | Statement::Throw { expr, .. } => collect_attribute_expr_roots(expr, roots, complete),
            Statement::Let(binding) => {
                collect_pattern_syntax_roots(&binding.pattern, roots);
                if let Some(annotation) = &binding.annotation {
                    collect_type_annotation_roots(annotation, roots);
                }
                if let Some(value) = &binding.value {
                    collect_attribute_expr_roots(value, roots, complete);
                }
            }
            Statement::Return(return_stmt) => {
                if let Some(value) = &return_stmt.value {
                    collect_attribute_expr_roots(value, roots, complete);
                }
            }
            Statement::For(for_stmt) => {
                // Core emits these canonical reads while lowering every
                // executable `for`; strict lockstep zip adds its dedicated
                // mismatch error class.
                roots.push("Error".to_owned());
                if for_stmt.lanes.len() > 1 {
                    roots.push("ZipLengthMismatchError".to_owned());
                }
                for lane in &for_stmt.lanes {
                    collect_pattern_syntax_roots(&lane.pattern, roots);
                    collect_attribute_expr_roots(&lane.iter, roots, complete);
                }
                collect_attribute_block_roots(&for_stmt.body, roots, complete);
                *complete = false;
            }
            Statement::Break { .. } | Statement::Continue { .. } => {}
            _ => *complete = false,
        }
    }
}

/// Finds loop-control statements whose target loop lives in an outer compiler
/// function. Core emits `Error.new(...).raise()` for those materialized-block
/// paths. Direct loop control remains a local jump and must not create an
/// `Error` runtime dependency.
fn collect_materialized_loop_control_trap_ranges(statements: &[Statement]) -> Vec<phalcom_common::range::SourceRange> {
    use phalcom_common::range::SourceRange;

    fn scan_pack(items: &[phalcom_ast::ast::PackItem], function_depth: usize, loop_depths: &mut Vec<usize>, traps: &mut Vec<SourceRange>) {
        for item in items {
            let expression = match item {
                phalcom_ast::ast::PackItem::Positional { expr, .. }
                | phalcom_ast::ast::PackItem::Labeled { value: expr, .. }
                | phalcom_ast::ast::PackItem::Expand { expr, .. } => expr,
            };
            scan_expr(expression, function_depth, loop_depths, traps);
        }
    }

    fn scan_statements(statements: &[Statement], function_depth: usize, loop_depths: &mut Vec<usize>, traps: &mut Vec<SourceRange>) {
        for statement in statements {
            #[allow(deprecated)]
            match statement {
                Statement::Expr { expr, .. } | Statement::Throw { expr, .. } => scan_expr(expr, function_depth, loop_depths, traps),
                Statement::Let(binding) => {
                    if let Some(value) = &binding.value {
                        scan_expr(value, function_depth, loop_depths, traps);
                    }
                }
                Statement::Return(return_stmt) => {
                    if let Some(value) = &return_stmt.value {
                        scan_expr(value, function_depth, loop_depths, traps);
                    }
                }
                Statement::For(for_statement) => {
                    for lane in &for_statement.lanes {
                        scan_expr(&lane.iter, function_depth, loop_depths, traps);
                    }
                    loop_depths.push(function_depth);
                    scan_statements(&for_statement.body, function_depth, loop_depths, traps);
                    loop_depths.pop();
                }
                Statement::Break { range } | Statement::Continue { range } => {
                    if loop_depths.last().is_some_and(|target_depth| function_depth > *target_depth) {
                        traps.push(*range);
                    }
                }
                Statement::Class(_)
                | Statement::Data(_)
                | Statement::Trait(_)
                | Statement::Impl(_)
                | Statement::TypeDefinition(_)
                | Statement::TypeAlias(_)
                | Statement::Export(_) => {}
            }
        }
    }

    fn scan_expr(expression: &Expr, function_depth: usize, loop_depths: &mut Vec<usize>, traps: &mut Vec<SourceRange>) {
        use phalcom_ast::ast::{BracketLiteralElement, IfElseBody, MapLiteralKey, MatchArmBody, RecordLiteralEntry, SetLiteralEntry, TupleLiteralEntry};

        match expression {
            Expr::Assignment(assignment) => {
                scan_expr(&assignment.name, function_depth, loop_depths, traps);
                scan_expr(&assignment.value, function_depth, loop_depths, traps);
            }
            Expr::Range(range) => {
                if let Some(lower) = &range.lower {
                    scan_expr(lower, function_depth, loop_depths, traps);
                }
                if let Some(upper) = &range.upper {
                    scan_expr(upper, function_depth, loop_depths, traps);
                }
            }
            Expr::Unary(unary) => scan_expr(&unary.expr, function_depth, loop_depths, traps),
            Expr::Binary(binary) => {
                scan_expr(&binary.left, function_depth, loop_depths, traps);
                scan_expr(&binary.right, function_depth, loop_depths, traps);
            }
            Expr::ComparisonChain(chain) => {
                for operand in &chain.operands {
                    scan_expr(operand, function_depth, loop_depths, traps);
                }
            }
            Expr::IfLet(expr) => {
                scan_expr(&expr.value, function_depth, loop_depths, traps);
                scan_statements(&expr.then_body.body, function_depth, loop_depths, traps);
                if let Some(else_body) = &expr.else_body {
                    scan_statements(&else_body.body, function_depth, loop_depths, traps);
                }
            }
            Expr::WhileLet(expr) => {
                loop_depths.push(function_depth);
                scan_expr(&expr.value, function_depth, loop_depths, traps);
                scan_statements(&expr.body, function_depth, loop_depths, traps);
                loop_depths.pop();
            }
            Expr::If(expr) => {
                scan_expr(&expr.condition, function_depth, loop_depths, traps);
                scan_statements(&expr.then_body.body, function_depth, loop_depths, traps);
                if let Some(else_body) = &expr.else_body {
                    match else_body {
                        IfElseBody::Block(block) => scan_statements(&block.body, function_depth, loop_depths, traps),
                        IfElseBody::ElseIf(expr) => scan_expr(expr, function_depth, loop_depths, traps),
                    }
                }
            }
            Expr::While(expr) => {
                // The guarded `whileTrue` lowering compiles the same condition
                // and body again in a materialized fallback closure. In that
                // copy the while context is one function shallower, while any
                // nested loop establishes its own target at the current depth.
                loop_depths.push(function_depth);
                scan_expr(&expr.condition, function_depth + 1, loop_depths, traps);
                scan_statements(&expr.body.body, function_depth + 1, loop_depths, traps);
                loop_depths.pop();
            }
            Expr::UnqualifiedCall(call) => scan_pack(&call.args, function_depth, loop_depths, traps),
            Expr::MethodCall(call) => {
                scan_expr(&call.object, function_depth, loop_depths, traps);
                scan_pack(&call.args, function_depth, loop_depths, traps);
            }
            Expr::GetProperty(property) => scan_expr(&property.object, function_depth, loop_depths, traps),
            Expr::SetProperty(property) => {
                scan_expr(&property.object, function_depth, loop_depths, traps);
                scan_expr(&property.value, function_depth, loop_depths, traps);
            }
            Expr::Index(index) => {
                scan_expr(&index.object, function_depth, loop_depths, traps);
                scan_pack(&index.args, function_depth, loop_depths, traps);
            }
            Expr::SetIndex(index) => {
                scan_expr(&index.object, function_depth, loop_depths, traps);
                scan_pack(&index.args, function_depth, loop_depths, traps);
                scan_expr(&index.value, function_depth, loop_depths, traps);
            }
            Expr::Block(block) => scan_statements(&block.body, function_depth + 1, loop_depths, traps),
            Expr::AssociatedLookup(lookup) => scan_expr(&lookup.receiver, function_depth, loop_depths, traps),
            Expr::AssociatedInvoke(invoke) => {
                scan_expr(&invoke.receiver, function_depth, loop_depths, traps);
                scan_pack(&invoke.args, function_depth, loop_depths, traps);
            }
            Expr::CallableReference(reference) => match &reference.target {
                phalcom_ast::ast::CallableReferenceTarget::Bound { receiver, .. }
                | phalcom_ast::ast::CallableReferenceTarget::Associated { receiver, .. }
                | phalcom_ast::ast::CallableReferenceTarget::PrimaryDataConstructor { receiver, .. } => scan_expr(receiver, function_depth, loop_depths, traps),
            },
            Expr::TupleLiteral(tuple) => {
                for entry in &tuple.entries {
                    match entry {
                        TupleLiteralEntry::Positional { expr, .. } | TupleLiteralEntry::Expand { expr, .. } => {
                            scan_expr(expr, function_depth, loop_depths, traps)
                        }
                        TupleLiteralEntry::Labeled { label, value, .. } => {
                            if let phalcom_ast::ast::ProductLabel::Computed { expr, .. } = label {
                                scan_expr(expr, function_depth, loop_depths, traps);
                            }
                            scan_expr(value, function_depth, loop_depths, traps);
                        }
                    }
                }
            }
            Expr::RecordLiteral(record) => {
                for entry in &record.entries {
                    match entry {
                        RecordLiteralEntry::Field(field) => {
                            if let phalcom_ast::ast::ProductLabel::Computed { expr, .. } = &field.label {
                                scan_expr(expr, function_depth, loop_depths, traps);
                            }
                            scan_expr(&field.value, function_depth, loop_depths, traps);
                        }
                        RecordLiteralEntry::Expansion { expr, .. } => scan_expr(expr, function_depth, loop_depths, traps),
                    }
                }
            }
            Expr::RecordConstruction(record) => {
                for entry in &record.entries {
                    scan_expr(&entry.value, function_depth, loop_depths, traps);
                }
            }
            Expr::MapLiteral(map) => {
                for entry in &map.entries {
                    match entry {
                        phalcom_ast::ast::MapLiteralEntry::Association { key, value, .. } => {
                            if let MapLiteralKey::Computed { expr, .. } = key {
                                scan_expr(expr, function_depth, loop_depths, traps);
                            }
                            scan_expr(value, function_depth, loop_depths, traps);
                        }
                        phalcom_ast::ast::MapLiteralEntry::Expansion { expr, .. } => scan_expr(expr, function_depth, loop_depths, traps),
                    }
                }
            }
            Expr::SetLiteral(set) => {
                for entry in &set.entries {
                    match entry {
                        SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => scan_expr(expr, function_depth, loop_depths, traps),
                    }
                }
            }
            Expr::BracketLiteral(list) => {
                for entry in &list.elements {
                    match entry {
                        BracketLiteralElement::Element { expr, .. } | BracketLiteralElement::Expansion { expr, .. } => {
                            scan_expr(expr, function_depth, loop_depths, traps)
                        }
                    }
                }
            }
            Expr::Membership(expr) => {
                scan_expr(&expr.left, function_depth, loop_depths, traps);
                scan_expr(&expr.right, function_depth, loop_depths, traps);
            }
            Expr::IsMembership(expr) => {
                scan_expr(&expr.left, function_depth, loop_depths, traps);
                scan_expr(&expr.candidates, function_depth, loop_depths, traps);
            }
            Expr::Match(expr) => {
                scan_expr(&expr.value, function_depth, loop_depths, traps);
                for arm in &expr.arms {
                    match &arm.branch {
                        MatchArmBody::Expression(expr) => scan_expr(expr, function_depth, loop_depths, traps),
                        MatchArmBody::Block(block) => scan_statements(&block.body, function_depth, loop_depths, traps),
                    }
                }
            }
            Expr::Char { .. }
            | Expr::Int { .. }
            | Expr::Float { .. }
            | Expr::Imaginary { .. }
            | Expr::String { .. }
            | Expr::Boolean { .. }
            | Expr::Var { .. }
            | Expr::Field { .. }
            | Expr::SelfVar { .. }
            | Expr::SuperVar { .. }
            | Expr::Symbol(_)
            | Expr::Ellipsis { .. }
            | Expr::TypeForm(_) => {}
        }
    }

    let mut traps = Vec::new();
    scan_statements(statements, 0, &mut Vec::new(), &mut traps);
    traps.sort();
    traps.dedup();
    traps
}

fn collect_materialized_loop_control_trap_ranges_in_expr(expression: &Expr) -> Vec<phalcom_common::range::SourceRange> {
    collect_materialized_loop_control_trap_ranges(&[Statement::Expr {
        expr: expression.clone(),
        range: expression.range(),
    }])
}

/// Adds implicit runtime class roots emitted by structural pattern lowering.
fn collect_pattern_syntax_roots(pattern: &phalcom_ast::ast::Pattern, roots: &mut Vec<String>) {
    use phalcom_ast::ast::Pattern;

    match pattern {
        Pattern::Tuple { elements, .. } => {
            roots.extend(["Tuple".to_owned(), "Error".to_owned()]);
            for element in elements {
                collect_pattern_syntax_roots(element, roots);
            }
        }
        Pattern::List { elements, rest, .. } => {
            roots.extend(["List".to_owned(), "Error".to_owned()]);
            for element in elements {
                collect_pattern_syntax_roots(element, roots);
            }
            if let Some(rest) = rest {
                collect_pattern_syntax_roots(rest, roots);
            }
        }
        Pattern::Record { entries, rest, .. } => {
            roots.extend(["Record".to_owned(), "Error".to_owned()]);
            for entry in entries {
                collect_pattern_syntax_roots(&entry.pattern, roots);
            }
            if let Some(rest) = rest {
                collect_pattern_syntax_roots(rest, roots);
            }
        }
        Pattern::Map { entries, .. } => {
            roots.extend(["Map".to_owned(), "Error".to_owned()]);
            for entry in entries {
                collect_pattern_syntax_roots(&entry.pattern, roots);
            }
        }
        Pattern::Variant(variant) => {
            if let phalcom_ast::ast::VariantPatternMode::ExactCall { arguments }
            | phalcom_ast::ast::VariantPatternMode::CallablePattern { prefix: arguments, .. } = &variant.mode
            {
                for argument in arguments {
                    collect_pattern_syntax_roots(&argument.pattern, roots);
                }
            }
            if let phalcom_ast::ast::VariantPatternMode::CallablePattern { suffix, .. } = &variant.mode {
                for argument in suffix {
                    collect_pattern_syntax_roots(&argument.pattern, roots);
                }
            }
        }
        Pattern::Or { alternatives, .. } => {
            for alternative in alternatives {
                collect_pattern_syntax_roots(alternative, roots);
            }
        }
        Pattern::Wildcard { .. } | Pattern::Char { .. } | Pattern::Name { .. } => {}
    }
}

/// Adds exact canonical reads for compiler-generated loop/control-flow paths.
fn add_lowering_runtime_requirements(
    importer: &ModuleId,
    parsed: &phalcom_ast::ast::Program,
    prelude: &PreludeTypeMap,
    requirements: &mut Vec<RuntimeDependencyRequirement>,
    module_complete: &mut bool,
) {
    use phalcom_ast::ast::{ClassMember, ImplMember, MemberBody, Statement, TraitMember};

    fn collect_body(body: &MemberBody, roots: &mut Vec<(String, phalcom_common::range::SourceRange)>, complete: &mut bool) {
        if let MemberBody::Block(statements) = body {
            let mut found_roots = Vec::new();
            collect_attribute_block_roots(statements, &mut found_roots, complete);
            for root in found_roots {
                if matches!(root.as_str(), "Error" | "ZipLengthMismatchError" | "Tuple" | "List" | "Record" | "Map") {
                    roots.push((root, Default::default()));
                }
            }
            roots.extend(
                collect_materialized_loop_control_trap_ranges(statements)
                    .into_iter()
                    .map(|range| ("Error".to_owned(), range)),
            );
        }
    }

    fn collect_statement_roots(statements: &[Statement], roots: &mut Vec<(String, phalcom_common::range::SourceRange)>, complete: &mut bool) {
        let mut names = Vec::new();
        collect_attribute_block_roots(statements, &mut names, complete);
        for name in names {
            if matches!(name.as_str(), "Error" | "ZipLengthMismatchError" | "Tuple" | "List" | "Record" | "Map") {
                roots.push((name, Default::default()));
            }
        }
        roots.extend(
            collect_materialized_loop_control_trap_ranges(statements)
                .into_iter()
                .map(|range| ("Error".to_owned(), range)),
        );
    }

    let mut roots = Vec::new();
    let mut ignored_complete = true;
    for statement in &parsed.statements {
        #[allow(deprecated)]
        match statement {
            // These declaration forms are traversed below through their
            // callable bodies. Passing them to the block walker would mark
            // every module incomplete because declarations are not executable
            // statements in that context.
            Statement::Class(_)
            | Statement::Trait(_)
            | Statement::Impl(_)
            | Statement::Data(_)
            | Statement::TypeDefinition(_)
            | Statement::TypeAlias(_)
            | Statement::Export(_) => {}
            _ => collect_statement_roots(std::slice::from_ref(statement), &mut roots, &mut ignored_complete),
        }
    }
    for statement in &parsed.statements {
        match statement {
            Statement::Class(class) => {
                for member in &class.members {
                    match member {
                        ClassMember::Method(method) => collect_body(&method.body, &mut roots, &mut ignored_complete),
                        ClassMember::Getter(getter) => collect_body(&getter.body, &mut roots, &mut ignored_complete),
                        ClassMember::Setter(setter) => collect_body(&setter.body, &mut roots, &mut ignored_complete),
                        ClassMember::Index(index) => collect_body(&index.body, &mut roots, &mut ignored_complete),
                        ClassMember::Field(field) => {
                            if let Some(default) = &field.default {
                                let mut found_roots = Vec::new();
                                collect_attribute_expr_roots(default, &mut found_roots, &mut ignored_complete);
                                roots.extend(
                                    found_roots
                                        .into_iter()
                                        .filter(|root| matches!(root.as_str(), "Error" | "ZipLengthMismatchError" | "Tuple" | "List" | "Record" | "Map"))
                                        .map(|root| (root, default.range())),
                                );
                                roots.extend(
                                    collect_materialized_loop_control_trap_ranges_in_expr(default)
                                        .into_iter()
                                        .map(|range| ("Error".to_owned(), range)),
                                );
                            }
                        }
                        ClassMember::Delegation(_) | ClassMember::TypeMember(_) => {}
                    }
                }
            }
            Statement::Trait(trait_def) => {
                for member in &trait_def.members {
                    if let TraitMember::Behavior(behavior) = member {
                        match behavior {
                            phalcom_ast::ast::BehaviorMember::Method(method) => collect_body(&method.body, &mut roots, &mut ignored_complete),
                            phalcom_ast::ast::BehaviorMember::Getter(getter) => collect_body(&getter.body, &mut roots, &mut ignored_complete),
                            phalcom_ast::ast::BehaviorMember::Setter(setter) => collect_body(&setter.body, &mut roots, &mut ignored_complete),
                            phalcom_ast::ast::BehaviorMember::Index(index) => collect_body(&index.body, &mut roots, &mut ignored_complete),
                        }
                    }
                }
            }
            Statement::Impl(implementation) => {
                for member in &implementation.members {
                    if let ImplMember::Behavior(behavior) = member {
                        match behavior {
                            phalcom_ast::ast::BehaviorMember::Method(method) => collect_body(&method.body, &mut roots, &mut ignored_complete),
                            phalcom_ast::ast::BehaviorMember::Getter(getter) => collect_body(&getter.body, &mut roots, &mut ignored_complete),
                            phalcom_ast::ast::BehaviorMember::Setter(setter) => collect_body(&setter.body, &mut roots, &mut ignored_complete),
                            phalcom_ast::ast::BehaviorMember::Index(index) => collect_body(&index.body, &mut roots, &mut ignored_complete),
                        }
                    }
                }
            }
            _ => {}
        }
    }

    for (root, range) in roots {
        if !push_compiler_generated_read_requirement(importer, &root, range, prelude, requirements) {
            *module_complete = false;
        }
    }
    // Break/continue lowered through materialized blocks can emit an Error
    // trap. The semantic product does not currently distinguish that boundary.
    // We project Error conservatively above and retain incomplete coverage.
    if !ignored_complete {
        *module_complete = false;
    }
}

fn statement_attributes(statement: &Statement) -> Vec<&Attribute> {
    let mut attributes = Vec::new();
    match statement {
        Statement::Class(class) => {
            attributes.extend(class.attributes.iter());
            for associated_type_data in &class.associated_type_data {
                attributes.extend(associated_type_data.attributes.iter());
            }
            for member in &class.members {
                attributes.extend(member.attributes().iter());
            }
        }
        Statement::Data(data) => {
            attributes.extend(data.attributes.iter());
            if let Some(cases) = data.cases() {
                for case in cases {
                    attributes.extend(case.attributes.iter());
                }
            }
        }
        Statement::Trait(trait_def) => {
            attributes.extend(trait_def.attributes.iter());
            for member in &trait_def.members {
                match member {
                    TraitMember::Behavior(behavior) => attributes.extend(behavior.attributes().iter()),
                    TraitMember::Property(property) => attributes.extend(property.attributes.iter()),
                    TraitMember::AssociatedType(associated) => attributes.extend(associated.attributes.iter()),
                }
            }
        }
        Statement::Impl(impl_def) => {
            for member in &impl_def.members {
                if let Some(behavior) = member.behavior() {
                    attributes.extend(behavior.attributes().iter());
                }
                match member {
                    phalcom_ast::ast::ImplMember::Delegation(delegation) => attributes.extend(delegation.attributes.iter()),
                    phalcom_ast::ast::ImplMember::AssociatedTypeBinding(binding) => attributes.extend(binding.attributes.iter()),
                    phalcom_ast::ast::ImplMember::TypeMember(type_member) => attributes.extend(type_member.attributes.iter()),
                    phalcom_ast::ast::ImplMember::Behavior(_) => {}
                }
            }
        }
        _ => {}
    }
    attributes
}

/// Whether retained attribute analysis proves a closed runtime dependency
/// set for one argument. A completed query may still contain an unknown or
/// deliberately dynamic send; those expressions can select owners that the
/// semantic product did not resolve, so they must keep conservative import
/// fallback active.
fn attribute_argument_has_closed_runtime_dependencies(product: Option<&Arc<crate::attribute_analysis::AttributeArgumentAnalysis>>) -> bool {
    product.is_some_and(|product| {
        product.analysis.status == crate::checker::CallableAnalysisStatus::Complete
            && !product.analysis.expressions.is_empty()
            && product
                .analysis
                .expressions
                .values()
                .all(|expression| expression.status.is_ready() && !expression.knowledge.is_unknown() && !expression.knowledge.is_dynamic())
    })
}

/// Aggregates joined phase evidence and generated runtime requirements for all modules in a workspace.
pub fn compute_module_import_phase_products(
    semantic_linked: &SemanticLinkedProgram,
    sources: &BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    source_index: &SourceSemanticIndex,
    callable_analyses: &HashMap<CallableId, Arc<CallableAnalysis>>,
    type_store: Option<&TypeStore>,
    static_constants: Option<&StaticConstantTable>,
    is_complete: bool,
) -> (BTreeMap<ModuleId, ModuleImportPhaseProduct>, Vec<RuntimeDependencyRequirement>) {
    compute_module_import_phase_products_with_attribute_analyses(
        semantic_linked,
        sources,
        source_index,
        callable_analyses,
        &BTreeMap::new(),
        type_store,
        static_constants,
        is_complete,
    )
}

/// Aggregates ordinary callable and attribute-argument semantic products into
/// joined import phases and generated runtime requirements.
pub fn compute_module_import_phase_products_with_attribute_analyses(
    semantic_linked: &SemanticLinkedProgram,
    sources: &BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    source_index: &SourceSemanticIndex,
    callable_analyses: &HashMap<CallableId, Arc<CallableAnalysis>>,
    attribute_analyses: &BTreeMap<crate::attribute_analysis::AttributeArgumentId, Arc<crate::attribute_analysis::AttributeArgumentAnalysis>>,
    type_store: Option<&TypeStore>,
    static_constants: Option<&StaticConstantTable>,
    is_complete: bool,
) -> (BTreeMap<ModuleId, ModuleImportPhaseProduct>, Vec<RuntimeDependencyRequirement>) {
    compute_module_import_phase_products_with_attribute_analyses_and_mode(
        semantic_linked,
        sources,
        source_index,
        callable_analyses,
        attribute_analyses,
        type_store,
        static_constants,
        is_complete,
        crate::attribute_analysis::AttributeExecutionMode::Debug,
    )
}

/// Aggregates semantic products and projects only the attribute requirements
/// emitted by `execution_mode` into the executable import graph.
pub fn compute_module_import_phase_products_with_attribute_analyses_and_mode(
    semantic_linked: &SemanticLinkedProgram,
    sources: &BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    source_index: &SourceSemanticIndex,
    callable_analyses: &dyn CallableAnalysisCollection,
    attribute_analyses: &BTreeMap<crate::attribute_analysis::AttributeArgumentId, Arc<crate::attribute_analysis::AttributeArgumentAnalysis>>,
    type_store: Option<&TypeStore>,
    static_constants: Option<&StaticConstantTable>,
    is_complete: bool,
    execution_mode: crate::attribute_analysis::AttributeExecutionMode,
) -> (BTreeMap<ModuleId, ModuleImportPhaseProduct>, Vec<RuntimeDependencyRequirement>) {
    let mut products = BTreeMap::new();
    let mut generated_requirements = Vec::new();
    let prelude = PreludeTypeMap::shared_canonical_universe();

    for (module_id, sem_mod) in &semantic_linked.modules {
        let mut observed_uses: BTreeMap<SourceImportBindingId, DependencyPhase> = BTreeMap::new();
        let mut module_complete = is_complete;

        // 1. Whole-module imports and direct re-exports are always Runtime (§9.2).
        for binding in sem_mod.imports.bindings.values() {
            match binding.kind {
                ReferenceKind::WholeModuleImport | ReferenceKind::ReExport => {
                    observed_uses.insert(binding.id.clone(), DependencyPhase::Runtime);
                }
                _ => {}
            }
        }

        // Attribute expressions are semantically checked in every mode, but
        // stripped contracts and invariants do not contribute executable
        // reads. Retain their import observations as interface-only so a
        // complete mode-specific product does not mistake them for unused
        // selective imports (which remain Runtime by the P4 conservative
        // rule). Any emitted body use below still joins to Runtime.
        for product in attribute_analyses
            .values()
            .filter(|product| !product.id.context.is_emitted_in(execution_mode))
            .filter(|product| product.id.module() == module_id)
        {
            for import_id in product.analysis.observed_import_phases.keys() {
                observed_uses
                    .entry(import_id.clone())
                    .and_modify(|phase| *phase = phase.join(DependencyPhase::InterfaceOnly))
                    .or_insert(DependencyPhase::InterfaceOnly);
            }
        }

        // 2. Body-level uses: expressions, calls, and local type annotations.
        let mut process_analysis = |analysis_module: &ModuleId, analysis: &CallableAnalysis| {
            if analysis_module == module_id {
                for (import_id, phase) in analysis.observed_import_phases.as_ref() {
                    observed_uses.entry(import_id.clone()).and_modify(|p| *p = p.join(*phase)).or_insert(*phase);
                }

                for expression in analysis.expressions.values() {
                    if let Some(crate::types::denotation::SemanticDenotation::TypeForm(ty)) = expression.denotation.as_ref()
                        && let Some(declaration) = type_store.and_then(|store| store.nominal_origin_declaration(*ty))
                    {
                        // Keep this in parity with core's `project_type_form_values`:
                        // nominal and applied class forms both emit a class-object read.
                        push_declaration_read_requirement(module_id, declaration, expression.range, &mut generated_requirements);
                    }

                    if let Some(specialization) = expression.call_specialization.as_ref() {
                        push_callable_owner_requirement(module_id, specialization.callable.module(), expression.range, &mut generated_requirements);
                        if let Some(store) = type_store {
                            for (_, argument) in specialization.bindings.iter() {
                                if let crate::types::store::StaticArgument::Term(ty) = argument
                                    && let Some(declaration) = store.nominal_origin_declaration(*ty)
                                {
                                    push_callable_owner_requirement(module_id, &declaration.module, expression.range, &mut generated_requirements);
                                }
                            }
                        }
                        if let Some(store) = type_store {
                            for conformance in specialization.conformances.iter() {
                                match &conformance.source {
                                    crate::checker::analysis::GenericCallConformanceSource::Exact { evidence } => {
                                        push_callable_owner_requirement(module_id, &evidence.source_impl.module, expression.range, &mut generated_requirements);
                                        add_conformance_type_owner_requirements(
                                            module_id,
                                            expression.range,
                                            &crate::generic_evidence::GenericConformanceEvidence::Exact(evidence.clone()),
                                            store,
                                            &mut generated_requirements,
                                        );
                                    }
                                    crate::checker::analysis::GenericCallConformanceSource::Derived { evidence } => {
                                        match &evidence.origin {
                                            crate::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source } => {
                                                push_callable_owner_requirement(
                                                    module_id,
                                                    &exact_source.source_impl.module,
                                                    expression.range,
                                                    &mut generated_requirements,
                                                );
                                            }
                                            crate::generic_evidence::DerivedConformanceOrigin::GuardedRule { rule, .. } => {
                                                push_callable_owner_requirement(module_id, &rule.impl_id.module, expression.range, &mut generated_requirements);
                                            }
                                        }
                                        add_conformance_type_owner_requirements(
                                            module_id,
                                            expression.range,
                                            &crate::generic_evidence::GenericConformanceEvidence::Derived(evidence.clone()),
                                            store,
                                            &mut generated_requirements,
                                        );
                                    }
                                    crate::checker::analysis::GenericCallConformanceSource::Forwarded { .. }
                                    | crate::checker::analysis::GenericCallConformanceSource::CompilerDerived { .. } => {}
                                }
                            }
                        }
                    }
                    if let Some(callable) = expression.callable.as_ref() {
                        push_callable_owner_requirement(module_id, callable.module(), expression.range, &mut generated_requirements);
                    }
                    if let Some(capability) = expression.conditional_capability.as_ref() {
                        push_callable_owner_requirement(module_id, &capability.selection.impl_id.module, expression.range, &mut generated_requirements);
                        push_callable_owner_requirement(module_id, capability.selection.callable.module(), expression.range, &mut generated_requirements);
                    }
                    if let Some(selection) = expression.conditional_dispatch.as_ref() {
                        push_callable_owner_requirement(module_id, &selection.impl_id.module, expression.range, &mut generated_requirements);
                        push_callable_owner_requirement(module_id, selection.callable.module(), expression.range, &mut generated_requirements);
                    }
                    if let Some(crate::trait_dispatch::TraitDispatchSite::Evidenced(selection)) = expression.trait_dispatch.as_ref() {
                        push_callable_owner_requirement(module_id, &selection.source_impl.module, expression.range, &mut generated_requirements);
                        if let Some(callable) = selection.callable.as_ref() {
                            push_callable_owner_requirement(module_id, callable.module(), expression.range, &mut generated_requirements);
                        }
                        if let Some(store) = type_store {
                            add_conformance_type_owner_requirements(module_id, expression.range, &selection.evidence, store, &mut generated_requirements);
                        }
                    }
                    if let Some(realization) = expression.contextual_realization.as_ref() {
                        let mut add_selection = |selection: &crate::checker::analysis::RealizationCapabilitySelection| {
                            if let crate::checker::analysis::RealizationCapabilitySelection::Evidenced(selection) = selection {
                                push_callable_owner_requirement(
                                    module_id,
                                    &selection.source_impl.module,
                                    realization.source_range,
                                    &mut generated_requirements,
                                );
                                if let Some(callable) = selection.callable.as_ref() {
                                    push_callable_owner_requirement(module_id, callable.module(), realization.source_range, &mut generated_requirements);
                                }
                                if let Some(store) = type_store {
                                    let bindings = match &selection.evidence {
                                        crate::generic_evidence::GenericConformanceEvidence::Exact(evidence) => {
                                            evidence.impl_environment.bindings.values().copied().collect::<Vec<_>>()
                                        }
                                        crate::generic_evidence::GenericConformanceEvidence::Derived(evidence) => {
                                            evidence.impl_bindings.bindings().values().copied().collect::<Vec<_>>()
                                        }
                                        crate::generic_evidence::GenericConformanceEvidence::Assumed(_)
                                        | crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(_) => Vec::new(),
                                    };
                                    for ty in bindings {
                                        if let Some(declaration) = store.nominal_origin_declaration(ty) {
                                            push_callable_owner_requirement(
                                                module_id,
                                                &declaration.module,
                                                realization.source_range,
                                                &mut generated_requirements,
                                            );
                                        }
                                    }
                                }
                            }
                        };
                        add_selection(&realization.realization);
                        if let crate::checker::analysis::ContextualRealizationOperation::Basis {
                            coefficient: crate::checker::analysis::RealizationCoefficientSource::One { one, .. },
                            ..
                        } = &realization.operation
                        {
                            add_selection(one);
                        }
                    }
                }

                for (expression_id, resolution) in analysis.associated_resolutions.iter() {
                    let Some(expression) = analysis.expressions.get(expression_id) else {
                        module_complete = false;
                        continue;
                    };
                    add_associated_resolution_owner_requirements(module_id, resolution, expression.range, &mut generated_requirements);
                }

                for (expression_id, resolution) in analysis.callable_reference_resolutions.iter() {
                    let range = analysis.expressions.get(expression_id).map(|expression| expression.range).unwrap_or_else(|| {
                        module_complete = false;
                        Default::default()
                    });
                    match &resolution.kind {
                        crate::checker::associated::CallableReferenceResolutionKind::Associated(associated) => {
                            add_associated_resolution_owner_requirements(module_id, associated, range, &mut generated_requirements);
                        }
                        crate::checker::associated::CallableReferenceResolutionKind::BoundFamily { members, .. } => {
                            for member in members.iter() {
                                push_invocation_target_owner_requirement(module_id, &member.target, range, &mut generated_requirements);
                                if let Some(selection) = member.conditional.as_ref() {
                                    push_callable_owner_requirement(module_id, &selection.impl_id.module, range, &mut generated_requirements);
                                }
                                if let Some(selection) = member.conditional_capability.as_ref() {
                                    push_callable_owner_requirement(module_id, &selection.selection.impl_id.module, range, &mut generated_requirements);
                                    push_callable_owner_requirement(module_id, selection.selection.callable.module(), range, &mut generated_requirements);
                                }
                                if let Some(selection) = member.trait_dispatch.as_ref() {
                                    push_callable_owner_requirement(module_id, &selection.source_impl.module, range, &mut generated_requirements);
                                    if let Some(callable) = selection.callable.as_ref() {
                                        push_callable_owner_requirement(module_id, callable.module(), range, &mut generated_requirements);
                                    }
                                }
                            }
                        }
                    }
                }

                for (expression_id, resolution) in analysis.family_applications.iter() {
                    let range = analysis.expressions.get(expression_id).map(|expression| expression.range).unwrap_or_else(|| {
                        module_complete = false;
                        Default::default()
                    });
                    add_family_application_owner_requirements(module_id, resolution, range, &mut generated_requirements);
                }

                for (expression_id, resolution) in analysis.match_resolutions.iter() {
                    let range = analysis.expressions.get(expression_id).map(|expression| expression.range).unwrap_or_default();
                    for arm in resolution.arms.iter() {
                        add_pattern_requirements(module_id, range, &arm.pattern, &prelude, &mut generated_requirements, &mut module_complete);
                    }
                }
                for site in analysis.pattern_site_resolutions.values() {
                    add_pattern_requirements(
                        module_id,
                        site.range,
                        &site.pattern,
                        &prelude,
                        &mut generated_requirements,
                        &mut module_complete,
                    );
                }
                for lane in analysis.for_iteration_lanes.values() {
                    for resolution in [&lane.iterator, &lane.next] {
                        if let crate::checker::ForIterationRequirementResolution::Evidence { evidence, .. } = resolution {
                            match evidence {
                                crate::generic_evidence::GenericConformanceEvidence::Exact(exact) => {
                                    push_callable_owner_requirement(module_id, &exact.source_impl.module, lane.source_range, &mut generated_requirements);
                                }
                                crate::generic_evidence::GenericConformanceEvidence::Derived(derived) => match &derived.origin {
                                    crate::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source } => {
                                        push_callable_owner_requirement(
                                            module_id,
                                            &exact_source.source_impl.module,
                                            lane.source_range,
                                            &mut generated_requirements,
                                        );
                                    }
                                    crate::generic_evidence::DerivedConformanceOrigin::GuardedRule { rule, .. } => {
                                        push_callable_owner_requirement(module_id, &rule.impl_id.module, lane.source_range, &mut generated_requirements);
                                    }
                                },
                                crate::generic_evidence::GenericConformanceEvidence::Assumed(_)
                                | crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(_) => {}
                            }
                            if let Some(store) = type_store {
                                add_conformance_type_owner_requirements(module_id, lane.source_range, evidence, store, &mut generated_requirements);
                            }
                        }
                    }
                }
            }
        };
        callable_analyses.for_each_analysis(&mut |analysis| process_analysis(analysis.callable.module(), analysis));
        for product in attribute_analyses.values().filter(|product| product.id.context.is_emitted_in(execution_mode)) {
            process_analysis(product.id.module(), product.analysis.as_ref());
        }

        // 3. Module-level declarations: superclasses (Runtime + generated req),
        // field annotations (InterfaceOnly), signature annotations (InterfaceOnly),
        // type aliases (InterfaceOnly), ADT/Enum payload types (InterfaceOnly).
        if let Some(parsed) = sources.get(module_id) {
            add_lowering_runtime_requirements(module_id, &parsed.program, &prelude, &mut generated_requirements, &mut module_complete);
            let mut decl_type_roots = Vec::new();
            let supported_attribute_sites = crate::attribute_analysis::collect_supported_attribute_argument_sites(module_id, &parsed.program);

            for stmt in &parsed.program.statements {
                for attribute in statement_attributes(stmt) {
                    if matches!(&attribute.kind, AttrKind::User(_)) {
                        if let Some(binding) = sem_mod.imports.get(&attribute.name) {
                            observed_uses
                                .entry(binding.id.clone())
                                .and_modify(|phase| *phase = phase.join(DependencyPhase::Runtime))
                                .or_insert(DependencyPhase::Runtime);
                            push_import_read_requirement(module_id, &binding.target, attribute.range, source_index, &mut generated_requirements);
                        } else if let Some(declaration) = prelude.get(&attribute.name) {
                            push_declaration_read_requirement(module_id, declaration, attribute.range, &mut generated_requirements);
                        } else if !sem_mod.local_globals.contains_key(attribute.name.as_str()) {
                            // Runtime user attributes are lowered by core today.
                            // An unclassified constructor cannot prove any selected
                            // import interface-only.
                            module_complete = false;
                        }
                    }
                    if !attribute.args.is_empty() {
                        let attribute_context = attribute.args.iter().find_map(|argument| {
                            supported_attribute_sites
                                .iter()
                                .find(|site| site.id.attribute_name.as_ref() == attribute.name && site.expression.range() == argument.range())
                                .map(|site| site.id.context)
                        });
                        if attribute_context.is_some_and(|context| !context.is_emitted_in(execution_mode)) {
                            continue;
                        }
                        let contract_error = match attribute_context {
                            Some(crate::attribute_analysis::AttributeAnalysisContext::RequiresContract) => Some("PreconditionError"),
                            Some(crate::attribute_analysis::AttributeAnalysisContext::EnsuresContract) => Some("PostconditionError"),
                            _ => None,
                        };
                        if let Some(error_name) = contract_error
                            && !push_compiler_generated_read_requirement(module_id, error_name, attribute.range, &prelude, &mut generated_requirements)
                        {
                            module_complete = false;
                        }
                        for argument in &attribute.args {
                            for range in collect_materialized_loop_control_trap_ranges_in_expr(argument) {
                                if !push_compiler_generated_read_requirement(module_id, "Error", range, &prelude, &mut generated_requirements) {
                                    module_complete = false;
                                }
                            }
                        }
                        let mut uncovered = Vec::new();
                        for argument in &attribute.args {
                            let site = supported_attribute_sites
                                .iter()
                                .find(|site| site.id.attribute_name.as_ref() == attribute.name && site.expression.range() == argument.range());
                            let covered = attribute_argument_has_closed_runtime_dependencies(site.and_then(|site| attribute_analyses.get(&site.id)));
                            if !covered {
                                uncovered.push(argument);
                            }
                        }
                        if uncovered.is_empty() {
                            continue;
                        }
                        // User-attribute arguments and builtin contract/invariant
                        // arguments without a complete retained semantic product
                        // remain conservative.
                        module_complete = false;
                        for binding in sem_mod.imports.bindings.values() {
                            observed_uses
                                .entry(binding.id.clone())
                                .and_modify(|phase| *phase = phase.join(DependencyPhase::Runtime))
                                .or_insert(DependencyPhase::Runtime);
                        }
                        let mut argument_roots = Vec::new();
                        for argument in uncovered {
                            collect_attribute_expr_roots(argument, &mut argument_roots, &mut module_complete);
                        }
                        for root in argument_roots {
                            if let Some(binding) = sem_mod.imports.get(&root) {
                                observed_uses
                                    .entry(binding.id.clone())
                                    .and_modify(|phase| *phase = phase.join(DependencyPhase::Runtime))
                                    .or_insert(DependencyPhase::Runtime);
                                push_import_read_requirement(module_id, &binding.target, attribute.range, source_index, &mut generated_requirements);
                            } else if let Some(declaration) = prelude.get(&root) {
                                push_declaration_read_requirement(module_id, declaration, attribute.range, &mut generated_requirements);
                            } else if root == "ZipLengthMismatchError" {
                                push_compiler_generated_read_requirement(module_id, &root, attribute.range, &prelude, &mut generated_requirements);
                            }
                        }
                    }
                }
                match stmt {
                    Statement::Class(class_def) => {
                        if execution_mode == crate::attribute_analysis::AttributeExecutionMode::Debug {
                            let declaration = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                            for (occurrence, (expression, range)) in class_def.invariants.iter().enumerate() {
                                let sites = supported_attribute_sites
                                    .iter()
                                    .filter(|site| {
                                        site.id.owner == crate::attribute_analysis::AttributeOwnerId::Declaration(declaration.clone())
                                            && site.id.attribute_name.as_ref() == "invariant"
                                            && site.id.context == crate::attribute_analysis::AttributeAnalysisContext::ClassInvariant
                                            && site.id.occurrence == occurrence as u32
                                    })
                                    .collect::<Vec<_>>();
                                // Invariants are woven only into callable bodies.
                                // A class with no eligible receiver callable has
                                // no runtime invariant expression to project.
                                if sites.is_empty() {
                                    continue;
                                }
                                if !push_compiler_generated_read_requirement(module_id, "InvariantError", *range, &prelude, &mut generated_requirements) {
                                    module_complete = false;
                                }
                                for trap_range in collect_materialized_loop_control_trap_ranges_in_expr(expression) {
                                    if !push_compiler_generated_read_requirement(module_id, "Error", trap_range, &prelude, &mut generated_requirements) {
                                        module_complete = false;
                                    }
                                }
                                let covered = sites
                                    .iter()
                                    .all(|site| attribute_argument_has_closed_runtime_dependencies(attribute_analyses.get(&site.id)));
                                if !covered {
                                    // Preserve the P4 conservative rule if any
                                    // receiving callable lacks complete semantic
                                    // evidence for the invariant expression.
                                    module_complete = false;
                                    for binding in sem_mod.imports.bindings.values() {
                                        observed_uses
                                            .entry(binding.id.clone())
                                            .and_modify(|phase| *phase = phase.join(DependencyPhase::Runtime))
                                            .or_insert(DependencyPhase::Runtime);
                                    }
                                    let mut roots = Vec::new();
                                    collect_attribute_expr_roots(expression, &mut roots, &mut module_complete);
                                    for root in roots {
                                        if let Some(binding) = sem_mod.imports.get(&root) {
                                            observed_uses
                                                .entry(binding.id.clone())
                                                .and_modify(|phase| *phase = phase.join(DependencyPhase::Runtime))
                                                .or_insert(DependencyPhase::Runtime);
                                            push_import_read_requirement(module_id, &binding.target, *range, source_index, &mut generated_requirements);
                                        } else if let Some(declaration) = prelude.get(&root) {
                                            push_declaration_read_requirement(module_id, declaration, *range, &mut generated_requirements);
                                        } else if root == "ZipLengthMismatchError" {
                                            push_compiler_generated_read_requirement(module_id, &root, *range, &prelude, &mut generated_requirements);
                                        }
                                    }
                                }
                            }
                        }
                        // Superclass reference is ALWAYS Runtime (§9.3).
                        if let Some(super_ref) = class_def.superclass_ref() {
                            if let Some(binding) = sem_mod.imports.get(&super_ref.root) {
                                observed_uses
                                    .entry(binding.id.clone())
                                    .and_modify(|p| *p = p.join(DependencyPhase::Runtime))
                                    .or_insert(DependencyPhase::Runtime);

                                push_import_read_requirement(module_id, &binding.target, super_ref.range, source_index, &mut generated_requirements);
                            } else if let Some(declaration) = prelude.get(&super_ref.root) {
                                push_declaration_read_requirement(module_id, declaration, super_ref.range, &mut generated_requirements);
                            }
                        }

                        if let Some(wc) = &class_def.where_clause {
                            collect_where_clause_roots(wc, &mut decl_type_roots);
                        }

                        // Field and member annotations in classes.
                        for member in &class_def.members {
                            match member {
                                ClassMember::Field(f) => {
                                    if let Some(ann) = &f.annotation {
                                        collect_type_annotation_roots(ann, &mut decl_type_roots);
                                    }
                                }
                                ClassMember::Method(m) => {
                                    for param in &m.params {
                                        if let Some(ann) = param.type_annotation() {
                                            collect_type_annotation_roots(ann, &mut decl_type_roots);
                                        }
                                    }
                                    if let Some(ret) = &m.return_annotation {
                                        collect_type_annotation_roots(ret, &mut decl_type_roots);
                                    }
                                    if let Some(wc) = &m.where_clause {
                                        collect_where_clause_roots(wc, &mut decl_type_roots);
                                    }
                                }
                                ClassMember::Getter(g) => {
                                    if let Some(ret) = &g.return_annotation {
                                        collect_type_annotation_roots(ret, &mut decl_type_roots);
                                    }
                                    if let Some(wc) = &g.where_clause {
                                        collect_where_clause_roots(wc, &mut decl_type_roots);
                                    }
                                }
                                ClassMember::Setter(s) => {
                                    if let Some(ann) = s.param.type_annotation() {
                                        collect_type_annotation_roots(ann, &mut decl_type_roots);
                                    }
                                    if let Some(ret) = &s.return_annotation {
                                        collect_type_annotation_roots(ret, &mut decl_type_roots);
                                    }
                                    if let Some(wc) = &s.where_clause {
                                        collect_where_clause_roots(wc, &mut decl_type_roots);
                                    }
                                }
                                ClassMember::Index(idx) => {
                                    for param in &idx.params {
                                        if let Some(ann) = param.type_annotation() {
                                            collect_type_annotation_roots(ann, &mut decl_type_roots);
                                        }
                                    }
                                    if let Some(ret) = &idx.return_annotation {
                                        collect_type_annotation_roots(ret, &mut decl_type_roots);
                                    }
                                    if let Some(wc) = &idx.where_clause {
                                        collect_where_clause_roots(wc, &mut decl_type_roots);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Statement::Data(data_def) => {
                        if let Some(wc) = &data_def.where_clause {
                            collect_where_clause_roots(wc, &mut decl_type_roots);
                        }
                        if let Some(components) = data_def.components() {
                            for comp in components {
                                collect_type_annotation_roots(&comp.annotation, &mut decl_type_roots);
                            }
                        }
                        if let Some(cases) = data_def.cases() {
                            for case in cases {
                                if let Some(res) = &case.result_annotation {
                                    collect_type_annotation_roots(res, &mut decl_type_roots);
                                }
                                match &case.payload {
                                    CasePayloadSyntax::Singleton => {}
                                    CasePayloadSyntax::Tuple { parameters, .. } => {
                                        for param in parameters {
                                            if let Some(ann) = param.type_annotation() {
                                                collect_type_annotation_roots(ann, &mut decl_type_roots);
                                            }
                                        }
                                    }
                                    CasePayloadSyntax::Record { components, .. } => {
                                        for comp in components {
                                            collect_type_annotation_roots(&comp.annotation, &mut decl_type_roots);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Statement::Trait(trait_def) => {
                        if let Some(wc) = &trait_def.where_clause {
                            collect_where_clause_roots(wc, &mut decl_type_roots);
                        }
                        for member in &trait_def.members {
                            if let Some(behavior) = member.behavior() {
                                collect_behavior_member_roots(behavior, &mut decl_type_roots);
                            }
                        }
                    }
                    Statement::Impl(impl_def) => {
                        if let ImplKind::Conformance { trait_ref, .. } = &impl_def.kind {
                            collect_type_annotation_roots(trait_ref, &mut decl_type_roots);
                        }
                        collect_type_annotation_roots(&impl_def.target, &mut decl_type_roots);
                        if let Some(wc) = &impl_def.where_clause {
                            collect_where_clause_roots(wc, &mut decl_type_roots);
                        }
                        for member in &impl_def.members {
                            if let Some(behavior) = member.behavior() {
                                collect_behavior_member_roots(behavior, &mut decl_type_roots);
                            }
                        }
                    }
                    #[allow(deprecated)]
                    Statement::TypeDefinition(alias_def) | Statement::TypeAlias(alias_def) => {
                        if let Some(ann) = alias_def.type_form_body() {
                            collect_type_annotation_roots(ann, &mut decl_type_roots);
                        }
                        if let Some(wc) = &alias_def.where_clause {
                            collect_where_clause_roots(wc, &mut decl_type_roots);
                        }
                    }
                    Statement::Let(let_binding) => {
                        if let Some(ann) = &let_binding.annotation {
                            collect_type_annotation_roots(ann, &mut decl_type_roots);
                        }
                    }
                    _ => {}
                }
            }

            for root in decl_type_roots {
                if let Some(binding) = sem_mod.imports.get(&root) {
                    observed_uses
                        .entry(binding.id.clone())
                        .and_modify(|p| *p = p.join(DependencyPhase::InterfaceOnly))
                        .or_insert(DependencyPhase::InterfaceOnly);
                }
            }
        }

        // 4. Construct final ImportPhaseSummary for each binding.
        let mut binding_summaries = BTreeMap::new();
        for (id, binding) in &sem_mod.imports.bindings {
            if observed_uses.get(id).is_some_and(|phase| *phase == DependencyPhase::Runtime) {
                push_import_read_requirement(module_id, &binding.target, binding.range, source_index, &mut generated_requirements);
            }
            let summary = if let Some(phase) = observed_uses.get(id) {
                ImportPhaseSummary { phase: *phase, observed: true }
            } else {
                // §9.5: Unused selective imports remain Runtime in P4.
                ImportPhaseSummary {
                    phase: DependencyPhase::Runtime,
                    observed: false,
                }
            };
            binding_summaries.insert(id.clone(), summary);
        }

        let coverage = if module_complete {
            PhaseEvidenceCoverage::Complete
        } else {
            PhaseEvidenceCoverage::Incomplete
        };

        add_native_prelude_read_requirements(module_id, source_index, &mut generated_requirements);

        products.insert(
            module_id.clone(),
            ModuleImportPhaseProduct {
                module: module_id.clone(),
                bindings: binding_summaries.into(),
                coverage,
            },
        );
    }

    if let Some(static_constants) = static_constants {
        for (_, product) in static_constants.iter() {
            let importer = &product.id.0.module;
            let Some(checked) = product.checked.as_ref() else {
                // Missing checked authority cannot prove interface-only use.
                if let Some(phase_product) = products.get_mut(importer) {
                    phase_product.coverage = PhaseEvidenceCoverage::Incomplete;
                }
                continue;
            };
            for symbol in checked.static_constant_reads.iter() {
                if &symbol.module == importer {
                    continue;
                }
                if let Some(module) = semantic_linked.modules.get(importer) {
                    for binding in module
                        .imports
                        .bindings
                        .values()
                        .filter(|binding| matches!(&binding.target, SymbolicImportTarget::Binding(target) if target == symbol))
                    {
                        if let Some(phase_product) = products.get_mut(importer)
                            && let Some(summary) = phase_product.bindings.get_mut(&binding.id)
                        {
                            summary.phase = summary.phase.join(DependencyPhase::Runtime);
                            summary.observed = true;
                        }
                    }
                }
                push_runtime_requirement(
                    importer,
                    &symbol.module,
                    product.initializer_range,
                    Some(LinkedReadSpec::Binding(symbol.clone())),
                    phalcom_modules::RuntimeRequirementProducer::StaticConstantRead,
                    &mut generated_requirements,
                );
            }
        }
    }

    (products, generated_requirements)
}

#[cfg(test)]
mod native_prelude_tests {
    use super::*;
    use crate::source_index::{SourceIndexContext, build_source_scope_index};
    use phalcom_modules::{ModuleComponent, ModulePath};

    fn fixture() -> (ModuleId, SourceSemanticIndex, DeclarationId, phalcom_common::range::SourceRange) {
        let module = ModuleId::universe(ModulePath::from_components(vec![ModuleComponent::from_identifier("consumer").unwrap()]));
        let parsed = phalcom_ast::parse("Error\n", 0);
        assert!(parsed.errors.is_empty());
        let program = Arc::new(parsed.program);
        let scope = build_source_scope_index(module.clone(), &program, &SourceIndexContext::default());
        let source = Arc::new(ParsedModuleUnit::new(
            module.clone(),
            phalcom_modules::source::ModuleKind::Module,
            None,
            Arc::from("Error\n"),
            program,
        ));
        let index =
            SourceSemanticIndex::from_scope_indices_with_programs(BTreeMap::from([(module.clone(), scope)]), &BTreeMap::from([(module.clone(), source)]));
        let dependency = ModuleId::universe(ModulePath::from_components(
            ["errors", "error"].map(|name| ModuleComponent::from_identifier(name).unwrap()),
        ));
        (module, index, DeclarationId::new(dependency, "Error".into()), (0..5).into())
    }

    #[test]
    fn overlapping_owner_views_publish_each_exact_obligation_once() {
        let (module, _, declaration, range) = fixture();
        let mut requirements = Vec::new();
        for _ in 0..3 {
            push_declaration_read_requirement(&module, &declaration, range, &mut requirements);
            push_callable_owner_requirement(&module, &declaration.module, range, &mut requirements);
            push_binding_owner_requirement(&module, &declaration, range, RuntimeRequirementProducer::PatternOwner, &mut requirements);
        }
        assert_eq!(
            requirements.len(),
            3,
            "distinct read, module capability, and pattern capability must survive repeated views"
        );
        let distinct = DeclarationId::new(declaration.module.clone(), "OtherError".into());
        push_declaration_read_requirement(&module, &distinct, range, &mut requirements);
        push_binding_owner_requirement(&module, &distinct, range, RuntimeRequirementProducer::PatternOwner, &mut requirements);
        assert_eq!(requirements.len(), 5, "different exact bindings at one span are distinct obligations");
    }

    #[test]
    fn overlapping_conformance_type_owner_views_publish_one_exact_obligation() {
        let (module, _, declaration, range) = fixture();
        let mut requirements = Vec::new();
        for _ in 0..4 {
            push_binding_owner_requirement(&module, &declaration, range, RuntimeRequirementProducer::ConformanceTypeOwner, &mut requirements);
        }
        assert_eq!(requirements.len(), 1, "repeated exact conformance type-owner views are one obligation");
        assert_eq!(requirements[0].provenance.producer, RuntimeRequirementProducer::ConformanceTypeOwner);

        let distinct = DeclarationId::new(declaration.module.clone(), "OtherError".into());
        push_binding_owner_requirement(&module, &distinct, range, RuntimeRequirementProducer::ConformanceTypeOwner, &mut requirements);
        assert_eq!(requirements.len(), 2, "a distinct binding at the same span remains a separate obligation");
    }

    #[test]
    fn native_prelude_fallback_defers_to_same_semantic_read() {
        let (module, index, declaration, range) = fixture();
        let mut requirements = Vec::new();
        push_declaration_read_requirement(&module, &declaration, range, &mut requirements);
        let semantic = requirements.clone();
        add_native_prelude_read_requirements(&module, &index, &mut requirements);
        assert_eq!(requirements, semantic, "one canonical declaration read must own the overlapping observation");
    }

    #[test]
    fn native_prelude_fallback_preserves_distinct_same_span_targets() {
        let (module, index, declaration, range) = fixture();
        let distinct = DeclarationId::new(declaration.module.clone(), "OtherError".into());
        let mut requirements = Vec::new();
        push_declaration_read_requirement(&module, &distinct, range, &mut requirements);
        add_native_prelude_read_requirements(&module, &index, &mut requirements);
        assert_eq!(requirements.len(), 2);
        assert_ne!(requirements[0].read_target, requirements[1].read_target);
        assert_eq!(requirements[0].range, requirements[1].range);
        assert_eq!(requirements[0].dependency, requirements[1].dependency);
    }

    #[test]
    fn native_prelude_fallback_preserves_distinct_same_target_authority() {
        let (module, index, declaration, range) = fixture();
        let mut requirements = Vec::new();
        push_declaration_read_requirement(&module, &declaration, range, &mut requirements);
        requirements[0].provenance.product = RuntimeRequirementProduct::CanonicalIdentity;
        requirements[0].provenance.timing = RuntimeRequirementTiming::BeforeSourceExecution;
        add_native_prelude_read_requirements(&module, &index, &mut requirements);
        assert_eq!(requirements.len(), 2);
        assert_eq!(requirements[0].read_target, requirements[1].read_target);
        assert_ne!(requirements[0].provenance.product, requirements[1].provenance.product);
        assert_ne!(requirements[0].provenance.timing, requirements[1].provenance.timing);
    }
}
