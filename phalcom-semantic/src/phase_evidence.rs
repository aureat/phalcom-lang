//! Semantic phase evidence aggregation and generated runtime requirement extraction.
//!
//! Semantic analysis owns the proof of whether an imported binding is interface-only
//! or runtime (ruling 2026-10-13 §4.2). This module joins observed use phases
//! across callable bodies, signatures, declarations, and AST structures into
//! canonical `ModuleImportPhaseProduct`s consumed by executable finalization.

use crate::checker::analysis::CallableAnalysis;
use crate::identity::{CallableId, ModuleId};
use crate::source::ParsedModuleUnit;
use phalcom_ast::ast::{
    BehaviorMember, CasePayloadSyntax, ClassMember, GenericArgumentSyntax, GenericConstraintSyntax,
    Statement, TraitRequirementSyntax, TypeAnnotation, TypeAnnotationExpr, TypeCallableItem,
    TypeTupleItem, WhereClauseSyntax,
};
use phalcom_modules::graph::{DependencyPhase, ReferenceKind, RuntimeDependencyEdge, RuntimeDependencyReason};
use phalcom_modules::identity::SourceImportBindingId;
use phalcom_modules::linker::{
    ImportPhaseSummary, ModuleImportPhaseProduct, PhaseEvidenceCoverage, SemanticLinkedProgram,
    SymbolicImportTarget,
};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

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
        TypeAnnotationExpr::ExactEnumCase { enum_target, generic_arguments, .. } => {
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

/// Aggregates joined phase evidence and generated runtime requirements for all modules in a workspace.
pub fn compute_module_import_phase_products(
    semantic_linked: &SemanticLinkedProgram,
    sources: &BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    callable_analyses: &HashMap<CallableId, Arc<CallableAnalysis>>,
    is_complete: bool,
) -> (BTreeMap<ModuleId, ModuleImportPhaseProduct>, Vec<RuntimeDependencyEdge>) {
    let mut products = BTreeMap::new();
    let mut generated_requirements = Vec::new();

    for (module_id, sem_mod) in &semantic_linked.modules {
        let mut observed_uses: BTreeMap<SourceImportBindingId, DependencyPhase> = BTreeMap::new();

        // 1. Whole-module imports and direct re-exports are always Runtime (§9.2).
        for binding in sem_mod.imports.bindings.values() {
            match binding.kind {
                ReferenceKind::WholeModuleImport | ReferenceKind::ReExport => {
                    observed_uses.insert(binding.id.clone(), DependencyPhase::Runtime);
                }
                _ => {}
            }
        }

        // 2. Body-level uses: expressions, calls, and local type annotations.
        for (callable_id, analysis) in callable_analyses {
            if callable_id.module() == module_id {
                for (import_id, phase) in analysis.observed_import_phases.as_ref() {
                    observed_uses
                        .entry(import_id.clone())
                        .and_modify(|p| *p = p.join(*phase))
                        .or_insert(*phase);
                }
            }
        }

        // 3. Module-level declarations: superclasses (Runtime + generated req),
        // field annotations (InterfaceOnly), signature annotations (InterfaceOnly),
        // type aliases (InterfaceOnly), ADT/Enum payload types (InterfaceOnly).
        if let Some(parsed) = sources.get(module_id) {
            let mut decl_type_roots = Vec::new();

            for stmt in &parsed.program.statements {
                match stmt {
                    Statement::Class(class_def) => {
                        // Superclass reference is ALWAYS Runtime (§9.3).
                        if let Some(super_ref) = class_def.superclass_ref() {
                            if let Some(binding) = sem_mod.imports.get(&super_ref.root) {
                                observed_uses
                                    .entry(binding.id.clone())
                                    .and_modify(|p| *p = p.join(DependencyPhase::Runtime))
                                    .or_insert(DependencyPhase::Runtime);

                                let target_mod = match &binding.target {
                                    SymbolicImportTarget::Module(m) => m.clone(),
                                    SymbolicImportTarget::Binding(s) => s.module.clone(),
                                };
                                generated_requirements.push(RuntimeDependencyEdge {
                                    importer: module_id.clone(),
                                    dependency: target_mod,
                                    range: super_ref.range,
                                    reason: RuntimeDependencyReason::RuntimeDeclarationReference,
                                });
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
        for (id, _binding) in &sem_mod.imports.bindings {
            let summary = if let Some(phase) = observed_uses.get(id) {
                ImportPhaseSummary {
                    phase: *phase,
                    observed: true,
                }
            } else {
                // §9.5: Unused selective imports remain Runtime in P4.
                ImportPhaseSummary {
                    phase: DependencyPhase::Runtime,
                    observed: false,
                }
            };
            binding_summaries.insert(id.clone(), summary);
        }

        let coverage = if is_complete {
            PhaseEvidenceCoverage::Complete
        } else {
            PhaseEvidenceCoverage::Incomplete
        };

        products.insert(
            module_id.clone(),
            ModuleImportPhaseProduct {
                module: module_id.clone(),
                bindings: binding_summaries,
                coverage,
            },
        );
    }

    (products, generated_requirements)
}
