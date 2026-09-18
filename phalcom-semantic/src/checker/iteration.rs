//! Canonical semantic authority for `for` lane iteration.
//!
//! This module deliberately keeps iteration proof separate from ordinary
//! expression dispatch.  A lane is executable only after the generic evidence
//! engine has proved the canonical `Iterable`/`Iterator` contracts and the
//! associated projections have been normalized by the shared projection
//! normalizer.

use super::call::{CallCheckResult, CallPremise, UnresolvedApplicationReason, analyze_unresolved_application};
use super::context::CheckingContext;
use crate::checker::analysis::{ForIterationLaneResolution, ForIterationRequirementResolution};
use crate::generic_evidence::{GenericConformanceEvidence, GenericConformanceKey, GenericConformanceProof};
use crate::identity::{DeclarationId, DispatchSide, ExpressionId};
use crate::traits::{TraitRef, TraitRequirementId};
use crate::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge};
use crate::types::id::TypeId;
use crate::types::parameter::TypeTerm;
use crate::types::projection::{ProjectionNormalizationMode, ProjectionNormalizationResult, normalize_type};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath};

fn canonical_iteration_declaration(module: &[&str], name: &str) -> DeclarationId {
    let components = module
        .iter()
        .map(|component| ModuleComponent::from_identifier(component).expect("canonical Universe module component"))
        .collect::<Vec<_>>();
    DeclarationId::new(ModuleId::universe(ModulePath::from_components(components)), name.into())
}

fn unavailable(
    ctx: &mut CheckingContext<'_>,
    premise: &CallPremise,
    reason: UnresolvedApplicationReason,
) -> (Option<ForIterationLaneResolution>, CallCheckResult) {
    (None, analyze_unresolved_application(ctx, premise, &[], reason))
}

fn proof_reason(proof: &GenericConformanceProof, range: SourceRange) -> UnresolvedApplicationReason {
    match proof {
        GenericConformanceProof::Missing => UnresolvedApplicationReason::DispatchMissing,
        GenericConformanceProof::Ambiguous(_) | GenericConformanceProof::CoherenceConflict(_) => UnresolvedApplicationReason::DispatchAmbiguous(range),
        GenericConformanceProof::Incomplete(impl_id) => {
            UnresolvedApplicationReason::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Incomplete(impl_id.clone()))
        }
        GenericConformanceProof::Unknown(reason) => {
            UnresolvedApplicationReason::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Unknown(reason.clone()))
        }
        GenericConformanceProof::Blocked(reason) => {
            UnresolvedApplicationReason::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Blocked(reason.clone()))
        }
        GenericConformanceProof::Dynamic(obligation) => {
            UnresolvedApplicationReason::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Dynamic(obligation.clone()))
        }
        GenericConformanceProof::Cancelled => UnresolvedApplicationReason::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::Cancelled),
        GenericConformanceProof::BudgetExceeded(report) => {
            UnresolvedApplicationReason::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::BudgetExceeded(report.clone()))
        }
        GenericConformanceProof::InternalFailure(message) => {
            UnresolvedApplicationReason::TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal::InternalFailure(message.clone()))
        }
        GenericConformanceProof::Proven(_) => UnresolvedApplicationReason::DispatchMissing,
    }
}

fn normalize_projection(ctx: &mut CheckingContext<'_>, ty: TypeId) -> ProjectionNormalizationResult {
    let empty_surfaces = crate::traits::TraitSurfaceTable::new();
    let view = ctx.conformance_semantics;
    ctx.control.relation(|budget, cancellation| {
        let mut normalization = crate::types::ProjectionNormalizationContext::new(
            ProjectionNormalizationMode::Generic {
                evidence: &ctx.generic_evidence,
                conformance_index: view.map(|view| view.conformance_index),
                witness_plans: view.map(|view| view.witness_plans),
                trait_surfaces: view.map(|view| view.trait_surfaces).unwrap_or(&empty_surfaces),
                derived_rules: view.map(|view| view.derived_conformance_rules),
                declarations: ctx.declarations,
                hierarchy: ctx.hierarchy.inner(),
            },
            budget,
            cancellation,
        );
        normalize_type(ctx.store, ty, &mut normalization)
    })
}

fn prove(ctx: &mut CheckingContext<'_>, subject: TypeTerm, trait_ref: &TraitRef) -> GenericConformanceProof {
    let Some(view) = ctx.conformance_semantics else {
        return GenericConformanceProof::InternalFailure("conformance semantics are unavailable for canonical iteration".into());
    };
    let mut proof_context = crate::generic_evidence::ConformanceProofContext::new(
        Some(&ctx.generic_evidence),
        Some(view.conformance_index),
        Some(view.witness_plans),
        Some(view.trait_surfaces),
        Some(ctx.declarations),
        Some(ctx.hierarchy.inner()),
        Some(&mut *ctx.store),
    )
    .with_derived_rules(Some(view.derived_conformance_rules));
    crate::generic_evidence::prove_conformance_with_context(&GenericConformanceKey::new(subject, trait_ref.clone()), &mut proof_context)
}

fn evidence_status(evidence: &GenericConformanceEvidence) -> EvidenceStatus {
    match evidence {
        GenericConformanceEvidence::Exact(_) => EvidenceStatus::Established,
        GenericConformanceEvidence::Assumed(_) | GenericConformanceEvidence::Derived(_) => EvidenceStatus::Assumed,
    }
}

fn record_evidence_dependencies(ctx: &mut CheckingContext<'_>, evidence: &GenericConformanceEvidence) {
    match evidence {
        GenericConformanceEvidence::Exact(exact) => {
            ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::ConformanceDispatch(exact.source_impl.clone()));
            ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                exact.exact_trait_ref.declaration.clone(),
            ));
        }
        GenericConformanceEvidence::Assumed(assumed) => {
            ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                assumed.key.trait_ref.declaration.clone(),
            ));
        }
        GenericConformanceEvidence::Derived(derived) => {
            match &derived.origin {
                crate::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source } => {
                    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::ConformanceDispatch(
                        exact_source.source_impl.clone(),
                    ));
                    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(
                        exact_source.exact_trait_ref.declaration.clone(),
                    ));
                }
                crate::generic_evidence::DerivedConformanceOrigin::GuardedRule { rule, exact_trait_ref, .. } => {
                    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::ConformanceDispatch(rule.impl_id.clone()));
                    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(exact_trait_ref.declaration.clone()));
                }
            }
            for nested in &derived.nested {
                if let crate::generic_evidence::NestedGenericEvidence::Conformance(nested) = nested {
                    record_evidence_dependencies(ctx, nested);
                }
            }
        }
    }
}

fn requirement_resolution(
    ctx: &mut CheckingContext<'_>,
    conformance: GenericConformanceKey,
    evidence: GenericConformanceEvidence,
    requirement: TraitRequirementId,
    fallback_signature: crate::signature::CallableSemanticSignature,
) -> ForIterationRequirementResolution {
    record_evidence_dependencies(ctx, &evidence);
    let signature = match &evidence {
        GenericConformanceEvidence::Exact(exact) => exact
            .requirement_views
            .get(&requirement)
            .map(|view| view.signature.clone())
            .unwrap_or(fallback_signature),
        GenericConformanceEvidence::Derived(derived) => derived
            .requirement_views()
            .get(&requirement)
            .map(|view| view.signature.clone())
            .unwrap_or(fallback_signature),
        GenericConformanceEvidence::Assumed(_) => fallback_signature,
    };
    ForIterationRequirementResolution::Evidence {
        conformance,
        evidence,
        requirement,
        signature,
    }
}

/// Resolves one `for` source through the canonical Iterator/Iterable evidence
/// products and returns both its published lane (when usable) and the normal
/// call result used by the existing flow/diagnostic machinery.
pub(crate) fn resolve_for_iteration_lane(
    ctx: &mut CheckingContext<'_>,
    premise: &CallPremise,
    source_expression: Option<ExpressionId>,
    receiver_ty: TypeId,
    source_range: SourceRange,
) -> (Option<ForIterationLaneResolution>, CallCheckResult) {
    let Some(source_expression) = source_expression else {
        return unavailable(ctx, premise, UnresolvedApplicationReason::DispatchMissing);
    };
    let iterator_decl = canonical_iteration_declaration(&["collections", "iterator_laws"], "Iterator");
    let iterable_decl = canonical_iteration_declaration(&["collections", "iterator_laws"], "Iterable");
    let Some(view) = ctx.conformance_semantics else {
        return unavailable(ctx, premise, UnresolvedApplicationReason::DispatchMissing);
    };
    let Some(iterable_surface) = view.trait_surfaces.get(&iterable_decl) else {
        return unavailable(ctx, premise, UnresolvedApplicationReason::DispatchMissing);
    };
    let Some(iterator_surface) = view.trait_surfaces.get(&iterator_decl) else {
        return unavailable(ctx, premise, UnresolvedApplicationReason::DispatchMissing);
    };
    let Some(item_requirement) = iterable_surface.associated_type_by_name("Item").map(|item| item.requirement.clone()) else {
        return unavailable(ctx, premise, UnresolvedApplicationReason::DispatchMissing);
    };
    let Some(iterator_requirement) = iterable_surface.associated_type_by_name("Iterator").map(|item| item.requirement.clone()) else {
        return unavailable(ctx, premise, UnresolvedApplicationReason::DispatchMissing);
    };
    let Ok(iterator_selector) = Selector::getter("iterator") else {
        return unavailable(ctx, premise, UnresolvedApplicationReason::DispatchMissing);
    };
    let Some(iterator_member) = iterable_surface.get_by_selector(&iterator_selector, DispatchSide::Instance) else {
        return unavailable(ctx, premise, UnresolvedApplicationReason::DispatchMissing);
    };
    let Ok(next_selector) = Selector::getter("next") else {
        return unavailable(ctx, premise, UnresolvedApplicationReason::DispatchMissing);
    };
    let Some(next_member) = iterator_surface.get_by_selector(&next_selector, DispatchSide::Instance) else {
        return unavailable(ctx, premise, UnresolvedApplicationReason::DispatchMissing);
    };

    let iterable_ref = TraitRef::new(iterable_decl.clone(), Vec::<TypeId>::new());
    let iterable_key = GenericConformanceKey::new(TypeTerm::Canonical(receiver_ty), iterable_ref.clone());
    let iterable_proof = prove(ctx, TypeTerm::Canonical(receiver_ty), &iterable_ref);
    let GenericConformanceProof::Proven(iterable_evidence) = iterable_proof.clone() else {
        return unavailable(ctx, premise, proof_reason(&iterable_proof, source_range));
    };

    let item_projection = ctx.store.associated_projection(receiver_ty, iterable_ref.clone(), item_requirement.clone());
    let iterator_projection = ctx.store.associated_projection(receiver_ty, iterable_ref.clone(), iterator_requirement.clone());
    let Some(item_type) = normalize_projection(ctx, item_projection).type_id() else {
        return unavailable(ctx, premise, UnresolvedApplicationReason::DispatchMissing);
    };
    let Some(iterator_type) = normalize_projection(ctx, iterator_projection).type_id() else {
        return unavailable(ctx, premise, UnresolvedApplicationReason::DispatchMissing);
    };

    let iterator_ref = TraitRef::new(iterator_decl.clone(), Vec::<TypeId>::new());
    let iterator_key = GenericConformanceKey::new(TypeTerm::Canonical(iterator_type), iterator_ref);
    let iterator_proof = prove(ctx, TypeTerm::Canonical(iterator_type), &iterator_key.trait_ref);
    let GenericConformanceProof::Proven(iterator_evidence) = iterator_proof.clone() else {
        return unavailable(ctx, premise, proof_reason(&iterator_proof, source_range));
    };

    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(iterable_decl));
    ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::TraitSurface(iterator_decl));
    let item_status = evidence_status(&iterable_evidence);
    let item_knowledge = match item_status {
        EvidenceStatus::Established => TypeKnowledge::established(item_type, EvidenceOrigin::ContextualDerivation),
        EvidenceStatus::Assumed => TypeKnowledge::assumed(item_type, EvidenceOrigin::ContextualDerivation),
    };
    let resolution = ForIterationLaneResolution {
        source_expression,
        source_range,
        source_type: receiver_ty,
        item_type,
        iterator_type,
        iterator: requirement_resolution(
            ctx,
            iterable_key,
            iterable_evidence,
            iterator_member.requirement.clone(),
            iterator_member.signature.clone(),
        ),
        next: requirement_resolution(
            ctx,
            iterator_key,
            iterator_evidence,
            next_member.requirement.clone(),
            next_member.signature.clone(),
        ),
    };
    let result = CallCheckResult {
        knowledge: item_knowledge,
        status: super::analysis::AnalysisStatus::Ready,
        causal_invalidity: super::causal::CausalInvalidity::Clean,
        explanation_parents: Vec::new(),
        callable: None,
        local_type: None,
        symbolic_result: None,
    };
    (Some(resolution), result)
}
