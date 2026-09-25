//! Canonical associated-type projection normalization.

use super::evidence::UnknownReason;
use super::family::FamilyMemberType;
use super::id::{RecordRowId, TypeId};
use super::outcome::{BlockReason, BudgetReport, CancellationToken, DynamicBoundaryObligation, QueryBudget};
use super::row::RecordRowField;
use super::store::{AssociatedTypeProjection, CallableParameterType, CallableType, StaticArgument, TupleTypeElement, TypeData, TypeStore};
use crate::declarations::DeclarationTypeTable;
use crate::generic_evidence::{GenericConformanceKey, GenericEvidenceEnvironment};
use crate::identity::ImplId;
use crate::impls::{AssociatedTypeBindingValue, ConformanceAssociatedTypePlan, ConformanceIndex, ConformanceResolution, ConformanceWitnessPlan};
use crate::traits::{TraitRef, TraitSurfaceTable};
use crate::types::parameter::TypeTerm;
use crate::types::type_member::{
    IntrinsicTypeOperation, IntrinsicTypeOwner, ProjectionSubject, TypeLevelProjection, TypeMemberArgument, TypeMemberSelector,
    lookup_intrinsic_type_operation,
};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

/// Authority used to normalize canonical associated projections.
pub enum ProjectionNormalizationMode<'a> {
    /// A trait declaration is being formed. Projections remain symbolic.
    AbstractTrait { trait_ref: &'a TraitRef },
    /// A source conformance is being checked against its staged binding plan.
    SourceConformance {
        trait_ref: &'a TraitRef,
        target: TypeId,
        plan: &'a ConformanceAssociatedTypePlan,
    },
    /// An exact target and trait application consume published conformance evidence.
    Exact {
        conformance_index: &'a ConformanceIndex,
        witness_plans: &'a BTreeMap<ImplId, Arc<ConformanceWitnessPlan>>,
        trait_surfaces: &'a TraitSurfaceTable,
        derived_rules: Option<&'a crate::derived_conformance::DerivedConformanceRuleIndex>,
        declarations: &'a DeclarationTypeTable,
        hierarchy: &'a dyn crate::types::relation::TypeHierarchy,
    },
    /// A generic evidence environment supplies assumed conformance and equality/refinement facts.
    Generic {
        evidence: &'a GenericEvidenceEnvironment,
        conformance_index: Option<&'a ConformanceIndex>,
        witness_plans: Option<&'a BTreeMap<ImplId, Arc<ConformanceWitnessPlan>>>,
        trait_surfaces: &'a TraitSurfaceTable,
        derived_rules: Option<&'a crate::derived_conformance::DerivedConformanceRuleIndex>,
        declarations: &'a DeclarationTypeTable,
        hierarchy: &'a dyn crate::types::relation::TypeHierarchy,
    },
}

/// Bounded state carried through one recursive normalization query.
pub struct ProjectionNormalizationContext<'a> {
    pub mode: ProjectionNormalizationMode<'a>,
    pub budget: &'a mut QueryBudget,
    pub cancel: &'a CancellationToken,
    active: HashSet<AssociatedTypeProjection>,
    current_exact_evidence: Option<CurrentExactEvidence<'a>>,
}

struct CurrentExactEvidence<'a> {
    target: TypeId,
    trait_ref: &'a TraitRef,
    associated_types: &'a BTreeMap<crate::traits::AssociatedTypeRequirementId, crate::impls::ExactAssociatedTypeBinding>,
}

impl<'a> ProjectionNormalizationContext<'a> {
    pub fn new(mode: ProjectionNormalizationMode<'a>, budget: &'a mut QueryBudget, cancel: &'a CancellationToken) -> Self {
        Self {
            mode,
            budget,
            cancel,
            active: HashSet::new(),
            current_exact_evidence: None,
        }
    }

    /// Supplies the exact associated bindings currently being assembled for
    /// one conformance. This breaks the self-evidence construction cycle while
    /// keeping foreign projections on the normal exact-evidence path.
    pub fn with_current_exact_evidence(
        mut self,
        target: TypeId,
        trait_ref: &'a TraitRef,
        associated_types: &'a BTreeMap<crate::traits::AssociatedTypeRequirementId, crate::impls::ExactAssociatedTypeBinding>,
    ) -> Self {
        self.current_exact_evidence = Some(CurrentExactEvidence {
            target,
            trait_ref,
            associated_types,
        });
        self
    }
}

/// Honest result of associated projection normalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectionNormalizationResult {
    Normalized(TypeId),
    Symbolic(TypeId),
    Incomplete,
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Ambiguous(Box<[ImplId]>),
    Recursive,
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}

/// Returns the exact row carried by a source-selected intrinsic `Record::Fields`
/// projection when its subject is already concrete or a canonical applied Record
/// family. Unresolved projections remain represented by their projection TypeId.
pub(crate) fn selected_record_fields_row(store: &TypeStore, form: TypeId) -> Option<RecordRowId> {
    let TypeData::TypeMemberProjection(projection) = store.get(form) else {
        return None;
    };
    let source = projection.source_declaration.as_ref()?;
    let ProjectionSubject::Type(subject) = &projection.subject else {
        return None;
    };
    if source.selector != projection.selector {
        return None;
    }
    let owner = match store.get(*subject) {
        TypeData::Record(_) => IntrinsicTypeOwner::Record,
        TypeData::Nominal { declaration } if declaration == &source.owner => IntrinsicTypeOwner::Record,
        TypeData::Applied { .. } if store.nominal_origin_declaration(*subject) == Some(&source.owner) => IntrinsicTypeOwner::Record,
        _ => return None,
    };
    if lookup_intrinsic_type_operation(owner, &projection.selector) != Some(IntrinsicTypeOperation::RecordFields) {
        return None;
    }

    if let TypeData::Record(row) = store.get(*subject) {
        return Some(*row);
    }
    let (declaration, arguments) = store.applied_nominal_static_parts(*subject)?;
    if declaration != source.owner {
        return None;
    }
    match arguments.as_slice() {
        [StaticArgument::RecordRow(row)] => Some(*row),
        _ => None,
    }
}

impl ProjectionNormalizationResult {
    pub fn type_id(&self) -> Option<TypeId> {
        match self {
            Self::Normalized(ty) | Self::Symbolic(ty) => Some(*ty),
            _ => None,
        }
    }

    fn is_symbolic(&self) -> bool {
        matches!(self, Self::Symbolic(_))
    }
}

pub fn normalize_type(store: &mut TypeStore, ty: TypeId, context: &mut ProjectionNormalizationContext<'_>) -> ProjectionNormalizationResult {
    if let Err(report) = context.budget.charge_step() {
        return ProjectionNormalizationResult::BudgetExceeded(report);
    }
    if context.cancel.is_cancelled() {
        return ProjectionNormalizationResult::Cancelled;
    }

    match store.get(ty).clone() {
        TypeData::AssociatedProjection(projection) => normalize_projection(store, ty, projection, context),
        TypeData::Applied { origin, arguments } => {
            let origin = match normalize_type(store, origin, context) {
                result @ (ProjectionNormalizationResult::Normalized(_) | ProjectionNormalizationResult::Symbolic(_)) => result,
                result => return result,
            };
            let mut symbolic = origin.is_symbolic();
            let origin = origin.type_id().expect("successful normalization has a type");
            let mut normalized_arguments = Vec::with_capacity(arguments.len());
            for argument in arguments {
                let Some(argument) = argument.as_type_id() else {
                    return ProjectionNormalizationResult::Symbolic(ty);
                };
                let result = normalize_type(store, argument, context);
                symbolic |= result.is_symbolic();
                let Some(argument) = result.type_id() else { return result };
                normalized_arguments.push(argument);
            }
            let normalized = store.apply_type_form(origin, &normalized_arguments).unwrap_or(ty);
            if symbolic {
                ProjectionNormalizationResult::Symbolic(normalized)
            } else {
                ProjectionNormalizationResult::Normalized(normalized)
            }
        }
        TypeData::ExactCase { variant, enum_type } => {
            let result = normalize_type(store, enum_type, context);
            let symbolic = result.is_symbolic();
            let Some(enum_type) = result.type_id() else { return result };
            let variant_identity = store.variant_identity(variant).clone();
            let normalized = match store.exact_case_type(&variant_identity, enum_type) {
                Ok(normalized) => normalized,
                Err(error) => return ProjectionNormalizationResult::InternalFailure(format!("exact-case rebuilding failed: {error:?}").into_boxed_str()),
            };
            if symbolic {
                ProjectionNormalizationResult::Symbolic(normalized)
            } else {
                ProjectionNormalizationResult::Normalized(normalized)
            }
        }
        TypeData::Union(members) => {
            let mut symbolic = false;
            let mut normalized_members = Vec::with_capacity(members.len());
            for member in members {
                let result = normalize_type(store, member, context);
                symbolic |= result.is_symbolic();
                let Some(member) = result.type_id() else { return result };
                normalized_members.push(member);
            }
            let normalized = store.union(&normalized_members);
            if symbolic {
                ProjectionNormalizationResult::Symbolic(normalized)
            } else {
                ProjectionNormalizationResult::Normalized(normalized)
            }
        }
        TypeData::Tuple(elements) => {
            let mut symbolic = false;
            let mut normalized_elements = Vec::with_capacity(elements.len());
            for element in elements {
                let result = normalize_type(store, element.ty, context);
                symbolic |= result.is_symbolic();
                let Some(ty) = result.type_id() else { return result };
                normalized_elements.push(TupleTypeElement { label: element.label, ty });
            }
            let normalized = store.tuple(normalized_elements.into_boxed_slice());
            if symbolic {
                ProjectionNormalizationResult::Symbolic(normalized)
            } else {
                ProjectionNormalizationResult::Normalized(normalized)
            }
        }
        TypeData::Record(row_id) => {
            let row = store.record_row(row_id).clone();
            let mut symbolic = false;
            let mut fields = Vec::with_capacity(row.fields.len());
            for field in row.fields {
                let result = normalize_type(store, field.ty, context);
                symbolic |= result.is_symbolic();
                let Some(ty) = result.type_id() else { return result };
                fields.push(RecordRowField { name: field.name, ty });
            }
            let normalized = match store.record_row_type_checked(fields, row.tail) {
                Ok(normalized) => normalized,
                Err(error) => return ProjectionNormalizationResult::InternalFailure(format!("record rebuilding failed: {error:?}").into_boxed_str()),
            };
            if symbolic {
                ProjectionNormalizationResult::Symbolic(normalized)
            } else {
                ProjectionNormalizationResult::Normalized(normalized)
            }
        }
        TypeData::Callable(callable) => {
            let mut symbolic = false;
            let mut parameters = Vec::with_capacity(callable.parameters.len());
            for parameter in callable.parameters {
                let result = normalize_type(store, parameter.ty, context);
                symbolic |= result.is_symbolic();
                let Some(ty) = result.type_id() else { return result };
                parameters.push(CallableParameterType {
                    label: parameter.label,
                    ty,
                    rest: parameter.rest,
                });
            }
            let result = normalize_type(store, callable.return_type, context);
            symbolic |= result.is_symbolic();
            let Some(return_type) = result.type_id() else { return result };
            let normalized = store.callable(CallableType {
                parameters: parameters.into_boxed_slice(),
                return_type,
            });
            if symbolic {
                ProjectionNormalizationResult::Symbolic(normalized)
            } else {
                ProjectionNormalizationResult::Normalized(normalized)
            }
        }
        TypeData::Family(family_id) => {
            let family = store.get_family(family_id).clone();
            let mut symbolic = false;
            let mut members = Vec::with_capacity(family.members.len());
            for member in family.members {
                let result = normalize_type(store, member.ty, context);
                symbolic |= result.is_symbolic();
                let Some(ty) = result.type_id() else { return result };
                members.push(FamilyMemberType {
                    operation: member.operation,
                    member_kind: member.member_kind,
                    ty,
                });
            }
            let normalized = match store.family_type(members) {
                Ok(normalized) => normalized,
                Err(error) => return ProjectionNormalizationResult::InternalFailure(format!("family rebuilding failed: {error:?}").into_boxed_str()),
            };
            if symbolic {
                ProjectionNormalizationResult::Symbolic(normalized)
            } else {
                ProjectionNormalizationResult::Normalized(normalized)
            }
        }
        TypeData::TypePack(elements) => {
            let mut symbolic = false;
            let mut normalized_elements = Vec::with_capacity(elements.len());
            for &element in elements.iter() {
                let result = normalize_type(store, element, context);
                symbolic |= result.is_symbolic();
                let Some(element) = result.type_id() else { return result };
                normalized_elements.push(element);
            }
            let normalized = store.type_pack(normalized_elements.into_boxed_slice()).unwrap_or(ty);
            if symbolic {
                ProjectionNormalizationResult::Symbolic(normalized)
            } else {
                ProjectionNormalizationResult::Normalized(normalized)
            }
        }
        TypeData::LabeledTypePack(elements) => {
            let mut symbolic = false;
            let mut normalized_elements = Vec::with_capacity(elements.len());
            for element in elements.iter() {
                let result = normalize_type(store, element.element, context);
                symbolic |= result.is_symbolic();
                let Some(el) = result.type_id() else { return result };
                normalized_elements.push(super::store::LabeledTypePackElement {
                    label: element.label.clone(),
                    element: el,
                });
            }
            let normalized = store.labeled_type_pack(normalized_elements.into_boxed_slice()).unwrap_or(ty);
            if symbolic {
                ProjectionNormalizationResult::Symbolic(normalized)
            } else {
                ProjectionNormalizationResult::Normalized(normalized)
            }
        }
        TypeData::TupleFromPacks { positionals, labeled } => {
            let res_pos = normalize_type(store, positionals, context);
            let symbolic_pos = res_pos.is_symbolic();
            let Some(pos) = res_pos.type_id() else { return res_pos };
            let res_lab = normalize_type(store, labeled, context);
            let symbolic_lab = res_lab.is_symbolic();
            let Some(lab) = res_lab.type_id() else { return res_lab };
            let normalized = store.tuple_from_packs(pos, lab).unwrap_or(ty);
            if symbolic_pos || symbolic_lab {
                ProjectionNormalizationResult::Symbolic(normalized)
            } else {
                ProjectionNormalizationResult::Normalized(normalized)
            }
        }
        TypeData::CallableFromPacks {
            positionals,
            labeled,
            return_type,
        } => {
            let res_pos = normalize_type(store, positionals, context);
            let symbolic_pos = res_pos.is_symbolic();
            let Some(pos) = res_pos.type_id() else { return res_pos };
            let res_lab = normalize_type(store, labeled, context);
            let symbolic_lab = res_lab.is_symbolic();
            let Some(lab) = res_lab.type_id() else { return res_lab };
            let res_ret = normalize_type(store, return_type, context);
            let symbolic_ret = res_ret.is_symbolic();
            let Some(ret) = res_ret.type_id() else { return res_ret };
            let normalized = store.callable_from_packs(pos, lab, ret).unwrap_or(ty);
            if symbolic_pos || symbolic_lab || symbolic_ret {
                ProjectionNormalizationResult::Symbolic(normalized)
            } else {
                ProjectionNormalizationResult::Normalized(normalized)
            }
        }
        TypeData::CompleteTypePack { positionals, labeled } => {
            let positionals = match normalize_type(store, positionals, context) {
                result @ (ProjectionNormalizationResult::Normalized(_) | ProjectionNormalizationResult::Symbolic(_)) => result,
                result => return result,
            };
            let labeled = match normalize_type(store, labeled, context) {
                result @ (ProjectionNormalizationResult::Normalized(_) | ProjectionNormalizationResult::Symbolic(_)) => result,
                result => return result,
            };
            let symbolic = positionals.is_symbolic() || labeled.is_symbolic();
            let positionals = positionals.type_id().expect("successful normalization has a type");
            let labeled = labeled.type_id().expect("successful normalization has a type");
            match store.complete_type_pack(positionals, labeled) {
                Ok(normalized) if symbolic => ProjectionNormalizationResult::Symbolic(normalized),
                Ok(normalized) => ProjectionNormalizationResult::Normalized(normalized),
                Err(error) => ProjectionNormalizationResult::InternalFailure(format!("complete pack rebuilding failed: {error:?}").into_boxed_str()),
            }
        }
        TypeData::TypePackConcat(segments) => normalize_pack_concat(store, ty, &segments, true, context),
        TypeData::LabeledTypePackConcat(segments) => normalize_pack_concat(store, ty, &segments, false, context),
        TypeData::TypeMemberProjection(projection) => normalize_type_member_projection(store, ty, projection, context),
        _ => ProjectionNormalizationResult::Normalized(ty),
    }
}

fn normalize_pack_concat(
    store: &mut TypeStore,
    _original: TypeId,
    segments: &[TypeId],
    positional: bool,
    context: &mut ProjectionNormalizationContext<'_>,
) -> ProjectionNormalizationResult {
    let mut normalized_segments = Vec::with_capacity(segments.len());
    let mut symbolic = false;
    for &segment in segments {
        let result = match normalize_type(store, segment, context) {
            result @ (ProjectionNormalizationResult::Normalized(_) | ProjectionNormalizationResult::Symbolic(_)) => result,
            result => return result,
        };
        symbolic |= result.is_symbolic();
        normalized_segments.push(result.type_id().expect("successful normalization has a type"));
    }
    let normalized = if positional {
        store.type_pack_concat(&normalized_segments)
    } else {
        store.labeled_type_pack_concat(&normalized_segments)
    };
    match normalized {
        Ok(type_id) if symbolic => ProjectionNormalizationResult::Symbolic(type_id),
        Ok(type_id) => ProjectionNormalizationResult::Normalized(type_id),
        Err(error) => ProjectionNormalizationResult::InternalFailure(format!("pack concat rebuilding failed: {error:?}").into_boxed_str()),
    }
}

fn normalize_associated_binding(
    store: &mut TypeStore,
    binding: &AssociatedTypeBindingValue,
    context: &mut ProjectionNormalizationContext<'_>,
) -> ProjectionNormalizationResult {
    match binding {
        AssociatedTypeBindingValue::Type(ty) => normalize_type(store, *ty, context),
        AssociatedTypeBindingValue::Dynamic => ProjectionNormalizationResult::Dynamic(DynamicBoundaryObligation {
            reason: "associated type binding is dynamic".into(),
        }),
    }
}

fn normalize_projection(
    store: &mut TypeStore,
    _original: TypeId,
    projection: AssociatedTypeProjection,
    context: &mut ProjectionNormalizationContext<'_>,
) -> ProjectionNormalizationResult {
    let subject = match normalize_type(store, projection.subject, context) {
        result @ (ProjectionNormalizationResult::Normalized(_) | ProjectionNormalizationResult::Symbolic(_)) => result,
        result => return result,
    };
    let subject = subject.type_id().expect("successful normalization has a type");
    let mut arguments = Vec::with_capacity(projection.trait_ref.arguments.len());
    for argument in projection.trait_ref.arguments.iter().copied() {
        let result = normalize_type(store, argument, context);
        let Some(argument) = result.type_id() else { return result };
        arguments.push(argument);
    }
    let trait_ref = TraitRef::new(projection.trait_ref.declaration.clone(), arguments.into_boxed_slice());
    let rewritten = store.associated_projection(subject, trait_ref.clone(), projection.requirement.clone());

    match &context.mode {
        ProjectionNormalizationMode::AbstractTrait { .. } => ProjectionNormalizationResult::Symbolic(rewritten),
        ProjectionNormalizationMode::SourceConformance {
            trait_ref: current,
            target,
            plan,
        } => {
            if &trait_ref != *current || subject != *target {
                return ProjectionNormalizationResult::Symbolic(rewritten);
            }
            let Some(binding) = plan.bindings.get(&projection.requirement) else {
                // During source-plan construction the later completeness pass
                // owns the Missing failure. Preserve the unresolved sibling as
                // a symbolic residual instead of making source order or the
                // builder's partial map observable.
                return ProjectionNormalizationResult::Symbolic(rewritten);
            };
            if !context.active.insert(projection.clone()) {
                return ProjectionNormalizationResult::Recursive;
            }
            let result = normalize_associated_binding(store, &binding.value_template, context);
            context.active.remove(&projection);
            result
        }
        ProjectionNormalizationMode::Exact {
            conformance_index,
            witness_plans,
            trait_surfaces,
            derived_rules,
            declarations,
            hierarchy,
        } => {
            if let Some(current) = context.current_exact_evidence.as_ref()
                && current.target == subject
                && current.trait_ref == &trait_ref
            {
                let Some(binding) = current.associated_types.get(&projection.requirement) else {
                    return ProjectionNormalizationResult::Incomplete;
                };
                if !context.active.insert(projection.clone()) {
                    return ProjectionNormalizationResult::Recursive;
                }
                let result = normalize_associated_binding(store, &binding.value, context);
                context.active.remove(&projection);
                return result;
            }
            if !trait_surfaces.contains(&trait_ref.declaration) {
                return ProjectionNormalizationResult::InternalFailure("trait surface is unavailable for projection normalization".into());
            }
            let resolution = crate::impls::resolve_conformance_evidence_with_surfaces(
                conformance_index,
                witness_plans,
                trait_surfaces,
                declarations,
                store,
                *hierarchy,
                subject,
                &trait_ref,
            );
            match resolution {
                ConformanceResolution::Proven(evidence) => {
                    let Some(binding) = evidence.associated_types.get(&projection.requirement) else {
                        return ProjectionNormalizationResult::Incomplete;
                    };
                    if !context.active.insert(projection.clone()) {
                        return ProjectionNormalizationResult::Recursive;
                    }
                    let result = normalize_associated_binding(store, &binding.value, context);
                    context.active.remove(&projection);
                    result
                }
                ConformanceResolution::NotDeclared => {
                    if let Some(rules) = derived_rules {
                        let candidate_rules = rules.rules_for_trait(&trait_ref.declaration);
                        for rule in candidate_rules {
                            let mut impl_bindings = crate::types::substitution::TypeSubstitution::new();
                            impl_bindings.bind(rule.target_parameter, subject);

                            let instantiated_trait_ref = crate::generic_evidence::substitute_trait_ref(&rule.trait_ref, &impl_bindings, store);
                            if instantiated_trait_ref == trait_ref {
                                let mut proof_context = crate::generic_evidence::ConformanceProofContext::new(
                                    None,
                                    Some(conformance_index),
                                    Some(witness_plans),
                                    Some(trait_surfaces),
                                    Some(declarations),
                                    Some(*hierarchy),
                                    Some(store),
                                )
                                .with_derived_rules(Some(rules));

                                let mut all_constraints_satisfied = true;
                                for constraint in &rule.generic_signature.constraints {
                                    let proof = crate::generic_evidence::prove_constraint(constraint, &impl_bindings, &mut proof_context);
                                    if !proof.is_proven() {
                                        all_constraints_satisfied = false;
                                        break;
                                    }
                                }

                                if all_constraints_satisfied {
                                    if let Some(binding) = rule.associated_type_plan.bindings.get(&projection.requirement) {
                                        let AssociatedTypeBindingValue::Type(value_template) = &binding.value_template else {
                                            return ProjectionNormalizationResult::Dynamic(DynamicBoundaryObligation {
                                                reason: "associated type binding is dynamic".into(),
                                            });
                                        };
                                        let instantiated_val = impl_bindings.apply(store, *value_template);
                                        if !context.active.insert(projection.clone()) {
                                            return ProjectionNormalizationResult::Recursive;
                                        }
                                        let result = normalize_type(store, instantiated_val, context);
                                        context.active.remove(&projection);
                                        return result;
                                    } else {
                                        return ProjectionNormalizationResult::Incomplete;
                                    }
                                }
                            }
                        }
                    }
                    ProjectionNormalizationResult::Unknown(UnknownReason::UnderconstrainedTypeVariable)
                }
                ConformanceResolution::InvalidSource(_) | ConformanceResolution::Incomplete(_) => ProjectionNormalizationResult::Incomplete,
                ConformanceResolution::CoherenceConflict(candidates) => ProjectionNormalizationResult::Ambiguous(candidates),
                ConformanceResolution::Unknown(reason) => ProjectionNormalizationResult::Unknown(reason),
                ConformanceResolution::Blocked(reason) => ProjectionNormalizationResult::Blocked(reason),
                ConformanceResolution::Dynamic(obligation) => ProjectionNormalizationResult::Dynamic(obligation),
                ConformanceResolution::Cancelled => ProjectionNormalizationResult::Cancelled,
                ConformanceResolution::BudgetExceeded(report) => ProjectionNormalizationResult::BudgetExceeded(report),
                ConformanceResolution::InternalFailure(message) => ProjectionNormalizationResult::InternalFailure(message),
            }
        }
        ProjectionNormalizationMode::Generic {
            evidence,
            conformance_index,
            witness_plans,
            trait_surfaces,
            derived_rules,
            declarations,
            hierarchy,
        } => {
            // 1. Check assumed conformance for the subject (or any equivalent subject)
            let key = GenericConformanceKey::new(TypeTerm::Canonical(subject), trait_ref.clone());
            let has_assumed = evidence.assumed_conformance(&key).is_some()
                || evidence.equivalent_terms(&TypeTerm::Canonical(subject)).iter().any(|equiv| {
                    evidence
                        .assumed_conformance(&GenericConformanceKey::new(equiv.clone(), trait_ref.clone()))
                        .is_some()
                });

            if has_assumed {
                // 2. Check if there is a unique lexical refinement for this projection
                let proj_term = TypeTerm::Canonical(rewritten);
                if let Some(TypeTerm::Canonical(refinement_ty)) = evidence.unique_refinement(&proj_term) {
                    if !context.active.insert(projection.clone()) {
                        return ProjectionNormalizationResult::Recursive;
                    }
                    let result = normalize_type(store, *refinement_ty, context);
                    context.active.remove(&projection);
                    return result;
                }
                // Assumed conformance without refinement remains symbolic
                return ProjectionNormalizationResult::Symbolic(rewritten);
            }

            // 1b. Check if assumed evidence or candidate rules entail conformance via a derived rule
            if let Some(rules) = derived_rules {
                let candidate_rules = rules.rules_for_trait(&trait_ref.declaration);
                for rule in candidate_rules {
                    let mut impl_bindings = crate::types::substitution::TypeSubstitution::new();
                    impl_bindings.bind(rule.target_parameter, subject);

                    let instantiated_trait_ref = crate::generic_evidence::substitute_trait_ref(&rule.trait_ref, &impl_bindings, store);
                    if instantiated_trait_ref == trait_ref {
                        let mut proof_context = crate::generic_evidence::ConformanceProofContext::new(
                            Some(evidence),
                            *conformance_index,
                            *witness_plans,
                            Some(trait_surfaces),
                            Some(declarations),
                            Some(*hierarchy),
                            Some(store),
                        )
                        .with_derived_rules(Some(rules));

                        let mut all_satisfied = true;
                        for constraint in &rule.generic_signature.constraints {
                            let proof = crate::generic_evidence::prove_constraint(constraint, &impl_bindings, &mut proof_context);
                            if !proof.is_proven() {
                                all_satisfied = false;
                                break;
                            }
                        }

                        if all_satisfied {
                            if let Some(binding) = rule.associated_type_plan.bindings.get(&projection.requirement) {
                                let AssociatedTypeBindingValue::Type(value_template) = &binding.value_template else {
                                    return ProjectionNormalizationResult::Dynamic(DynamicBoundaryObligation {
                                        reason: "associated type binding is dynamic".into(),
                                    });
                                };
                                let instantiated_val = impl_bindings.apply(store, *value_template);
                                if !context.active.insert(projection.clone()) {
                                    return ProjectionNormalizationResult::Recursive;
                                }
                                let result = normalize_type(store, instantiated_val, context);
                                context.active.remove(&projection);
                                return result;
                            } else {
                                return ProjectionNormalizationResult::Incomplete;
                            }
                        }
                    }
                }
            }

            // 3. Exact fallback for canonical types
            if let (Some(conformance_index), Some(witness_plans)) = (conformance_index, witness_plans) {
                if !trait_surfaces.contains(&trait_ref.declaration) {
                    return ProjectionNormalizationResult::InternalFailure("trait surface is unavailable for projection normalization".into());
                }
                let resolution = crate::impls::resolve_conformance_evidence_with_surfaces(
                    conformance_index,
                    witness_plans,
                    trait_surfaces,
                    declarations,
                    store,
                    *hierarchy,
                    subject,
                    &trait_ref,
                );
                match resolution {
                    ConformanceResolution::Proven(exact_evidence) => {
                        let Some(binding) = exact_evidence.associated_types.get(&projection.requirement) else {
                            return ProjectionNormalizationResult::Incomplete;
                        };
                        if !context.active.insert(projection.clone()) {
                            return ProjectionNormalizationResult::Recursive;
                        }
                        let result = normalize_associated_binding(store, &binding.value, context);
                        context.active.remove(&projection);
                        result
                    }
                    ConformanceResolution::NotDeclared => {
                        if let Some(rules) = derived_rules {
                            let candidate_rules = rules.rules_for_trait(&trait_ref.declaration);
                            for rule in candidate_rules {
                                let mut impl_bindings = crate::types::substitution::TypeSubstitution::new();
                                impl_bindings.bind(rule.target_parameter, subject);

                                let instantiated_trait_ref = crate::generic_evidence::substitute_trait_ref(&rule.trait_ref, &impl_bindings, store);
                                if instantiated_trait_ref == trait_ref {
                                    let mut proof_context = crate::generic_evidence::ConformanceProofContext::new(
                                        None,
                                        Some(conformance_index),
                                        Some(witness_plans),
                                        Some(trait_surfaces),
                                        Some(declarations),
                                        Some(*hierarchy),
                                        Some(store),
                                    )
                                    .with_derived_rules(Some(rules));

                                    let mut all_constraints_satisfied = true;
                                    for constraint in &rule.generic_signature.constraints {
                                        let proof = crate::generic_evidence::prove_constraint(constraint, &impl_bindings, &mut proof_context);
                                        if !proof.is_proven() {
                                            all_constraints_satisfied = false;
                                            break;
                                        }
                                    }

                                    if all_constraints_satisfied {
                                        if let Some(binding) = rule.associated_type_plan.bindings.get(&projection.requirement) {
                                            let AssociatedTypeBindingValue::Type(value_template) = &binding.value_template else {
                                                return ProjectionNormalizationResult::Dynamic(DynamicBoundaryObligation {
                                                    reason: "associated type binding is dynamic".into(),
                                                });
                                            };
                                            let instantiated_val = impl_bindings.apply(store, *value_template);
                                            if !context.active.insert(projection.clone()) {
                                                return ProjectionNormalizationResult::Recursive;
                                            }
                                            let result = normalize_type(store, instantiated_val, context);
                                            context.active.remove(&projection);
                                            return result;
                                        } else {
                                            return ProjectionNormalizationResult::Incomplete;
                                        }
                                    }
                                }
                            }
                        }
                        ProjectionNormalizationResult::Symbolic(rewritten)
                    }
                    ConformanceResolution::InvalidSource(_) | ConformanceResolution::Incomplete(_) => ProjectionNormalizationResult::Incomplete,
                    ConformanceResolution::CoherenceConflict(candidates) => ProjectionNormalizationResult::Ambiguous(candidates),
                    ConformanceResolution::Unknown(reason) => ProjectionNormalizationResult::Unknown(reason),
                    ConformanceResolution::Blocked(reason) => ProjectionNormalizationResult::Blocked(reason),
                    ConformanceResolution::Dynamic(obligation) => ProjectionNormalizationResult::Dynamic(obligation),
                    ConformanceResolution::Cancelled => ProjectionNormalizationResult::Cancelled,
                    ConformanceResolution::BudgetExceeded(report) => ProjectionNormalizationResult::BudgetExceeded(report),
                    ConformanceResolution::InternalFailure(message) => ProjectionNormalizationResult::InternalFailure(message),
                }
            } else {
                ProjectionNormalizationResult::Symbolic(rewritten)
            }
        }
    }
}

fn normalize_type_member_projection(
    store: &mut TypeStore,
    original: TypeId,
    projection: TypeLevelProjection,
    context: &mut ProjectionNormalizationContext<'_>,
) -> ProjectionNormalizationResult {
    let normalized_subject = match &projection.subject {
        ProjectionSubject::Type(ty) => {
            let res = normalize_type(store, *ty, context);
            let Some(norm_ty) = res.type_id() else { return res };
            ProjectionSubject::Type(norm_ty)
        }
        ProjectionSubject::RecordRow(r) => ProjectionSubject::RecordRow(*r),
    };

    let mut normalized_arguments = Vec::with_capacity(projection.arguments.len());
    for arg in projection.arguments.iter() {
        match arg {
            TypeMemberArgument::Type(ty) => {
                let res = normalize_type(store, *ty, context);
                let Some(norm_ty) = res.type_id() else { return res };
                normalized_arguments.push(TypeMemberArgument::Type(norm_ty));
            }
            TypeMemberArgument::Const(super::type_member::ConstTerm::Int(value)) => {
                normalized_arguments.push(TypeMemberArgument::Type(store.const_int(*value)));
            }
            TypeMemberArgument::RecordRow(r) => normalized_arguments.push(TypeMemberArgument::RecordRow(*r)),
        }
    }

    let orig_kind = store.kind_of(original);
    let rewritten_proj = TypeLevelProjection {
        subject: normalized_subject.clone(),
        selector: projection.selector.clone(),
        arguments: normalized_arguments.clone().into_boxed_slice(),
        source_declaration: projection.source_declaration.clone(),
    };
    let rewritten = store.type_member_projection(rewritten_proj, orig_kind);

    // Try intrinsic normalization
    if let Some(res) = normalize_intrinsic_projection(store, &normalized_subject, &projection.selector, &normalized_arguments, rewritten, context) {
        return res;
    }

    ProjectionNormalizationResult::Symbolic(rewritten)
}

fn normalize_intrinsic_projection(
    store: &mut TypeStore,
    subject: &ProjectionSubject,
    selector: &TypeMemberSelector,
    arguments: &[TypeMemberArgument],
    symbolic_fallback: TypeId,
    context: &mut ProjectionNormalizationContext<'_>,
) -> Option<ProjectionNormalizationResult> {
    let (owner, subject_data) = match subject {
        ProjectionSubject::Type(ty) => match store.get(*ty).clone() {
            TypeData::Tuple(elems) => (Some(IntrinsicTypeOwner::Tuple), Some(TypeData::Tuple(elems))),
            TypeData::TupleFromPacks { positionals, labeled } => match (store.get(positionals).clone(), store.get(labeled).clone()) {
                (TypeData::TypePack(pos), TypeData::LabeledTypePack(lab)) => {
                    let mut elems = Vec::with_capacity(pos.len() + lab.len());
                    for p in pos.iter() {
                        elems.push(TupleTypeElement::unlabeled(*p));
                    }
                    for l in lab.iter() {
                        elems.push(TupleTypeElement::labeled(l.label.clone(), l.element));
                    }
                    (Some(IntrinsicTypeOwner::Tuple), Some(TypeData::Tuple(elems.into_boxed_slice())))
                }
                _ => (Some(IntrinsicTypeOwner::Tuple), Some(TypeData::TupleFromPacks { positionals, labeled })),
            },
            TypeData::Unit => (Some(IntrinsicTypeOwner::Tuple), Some(TypeData::Tuple(Box::new([])))),
            TypeData::Record(row_id) => (Some(IntrinsicTypeOwner::Record), Some(TypeData::Record(row_id))),
            TypeData::TypePack(elems) => (Some(IntrinsicTypeOwner::TypePack), Some(TypeData::TypePack(elems))),
            TypeData::LabeledTypePack(elems) => (Some(IntrinsicTypeOwner::LabeledTypePack), Some(TypeData::LabeledTypePack(elems))),
            TypeData::CompleteTypePack { positionals, labeled } => {
                (Some(IntrinsicTypeOwner::CompleteTypePack), Some(TypeData::CompleteTypePack { positionals, labeled }))
            }
            TypeData::TypeMemberProjection(_) if store.kind_of(*ty) == crate::types::id::KindId::RECORD_ROW => {
                (Some(IntrinsicTypeOwner::RecordRow), None)
            }
            TypeData::Nominal { declaration } if declaration == crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Tuple) => {
                (Some(IntrinsicTypeOwner::Tuple), None)
            }
            TypeData::Nominal { declaration } if declaration == crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Record) => {
                (Some(IntrinsicTypeOwner::Record), None)
            }
            _ => (None, None),
        },
        ProjectionSubject::RecordRow(_) => (Some(IntrinsicTypeOwner::RecordRow), None),
    };

    let intrinsic = lookup_intrinsic_type_operation(owner?, selector)?;

    match intrinsic {
        IntrinsicTypeOperation::TuplePositionals => match subject {
            ProjectionSubject::Type(subject_ty) => match store.project_positionals(*subject_ty) {
                Ok(pack) => Some(ProjectionNormalizationResult::Normalized(pack)),
                Err(_) => Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            },
            ProjectionSubject::RecordRow(_) => Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
        },
        IntrinsicTypeOperation::TupleLabeled => match subject {
            ProjectionSubject::Type(subject_ty) => match store.project_labeled(*subject_ty) {
                Ok(pack) => Some(ProjectionNormalizationResult::Normalized(pack)),
                Err(_) => Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            },
            ProjectionSubject::RecordRow(_) => Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
        },
        IntrinsicTypeOperation::TupleEntries => match subject {
            ProjectionSubject::Type(subject_ty) => match store.project_complete(*subject_ty) {
                Ok(entries) => Some(ProjectionNormalizationResult::Normalized(entries)),
                Err(_) => Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            },
            ProjectionSubject::RecordRow(_) => Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
        },
        IntrinsicTypeOperation::TupleElement => {
            let elems = match subject_data? {
                TypeData::Tuple(elems) => elems,
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            if elems.is_empty() {
                Some(ProjectionNormalizationResult::Normalized(store.never()))
            } else {
                let elem_types: Vec<TypeId> = elems.iter().map(|e| e.ty).collect();
                let union_ty = store.union(&elem_types);
                Some(ProjectionNormalizationResult::Normalized(union_ty))
            }
        },
        IntrinsicTypeOperation::TupleAt => {
            let idx = match arguments.first()? {
                TypeMemberArgument::Type(argument) => match store.get(*argument) {
                    TypeData::ConstInt(value) => *value,
                    _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
                },
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let elems = match subject_data? {
                TypeData::Tuple(elems) => elems,
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let n = elems.len() as i64;
            let actual_idx = if idx < 0 { n + idx } else { idx };
            if actual_idx >= 0 && (actual_idx as usize) < elems.len() {
                let target_ty = elems[actual_idx as usize].ty;
                Some(normalize_type(store, target_ty, context))
            } else {
                Some(ProjectionNormalizationResult::InternalFailure(
                    format!("tuple index out of bounds: {idx} (length {n})").into_boxed_str(),
                ))
            }
        }
        IntrinsicTypeOperation::TupleMap => {
            let transform_fn = match arguments.first()? {
                TypeMemberArgument::Type(ty) => *ty,
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let elems = match subject_data? {
                TypeData::Tuple(elems) => elems,
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let mut transformed_elements = Vec::with_capacity(elems.len());
            for elem in elems.iter() {
                let applied = store.apply_type_form(transform_fn, &[elem.ty]).unwrap_or(elem.ty);
                let norm_res = normalize_type(store, applied, context);
                let Some(norm_ty) = norm_res.type_id() else { return Some(norm_res) };
                transformed_elements.push(TupleTypeElement {
                    label: elem.label.clone(),
                    ty: norm_ty,
                });
            }
            let normalized = store.tuple(transformed_elements.into_boxed_slice());
            Some(ProjectionNormalizationResult::Normalized(normalized))
        }
        IntrinsicTypeOperation::RecordMap => {
            let transform_fn = match arguments.first()? {
                TypeMemberArgument::Type(ty) => *ty,
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let row_id = match subject_data? {
                TypeData::Record(row_id) => row_id,
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let row = store.record_row(row_id).clone();
            let mut transformed_fields = Vec::with_capacity(row.fields.len());
            for field in row.fields.iter() {
                let applied = store.apply_type_form(transform_fn, &[field.ty]).unwrap_or(field.ty);
                let norm_res = normalize_type(store, applied, context);
                let Some(norm_ty) = norm_res.type_id() else { return Some(norm_res) };
                transformed_fields.push(RecordRowField {
                    name: field.name.clone(),
                    ty: norm_ty,
                });
            }
            let normalized_rec = store.record_row_type_checked(transformed_fields, row.tail).ok()?;
            Some(ProjectionNormalizationResult::Normalized(normalized_rec))
        }
        IntrinsicTypeOperation::TypePackMap => {
            let transform_fn = match arguments.first()? {
                TypeMemberArgument::Type(ty) => *ty,
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let elems = match subject_data? {
                TypeData::TypePack(elems) => elems,
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let mut transformed_elements = Vec::with_capacity(elems.len());
            for &elem in elems.iter() {
                let applied = store.apply_type_form(transform_fn, &[elem]).unwrap_or(elem);
                let norm_res = normalize_type(store, applied, context);
                let Some(norm_ty) = norm_res.type_id() else { return Some(norm_res) };
                transformed_elements.push(norm_ty);
            }
            let normalized = store.type_pack(transformed_elements.into_boxed_slice()).ok()?;
            Some(ProjectionNormalizationResult::Normalized(normalized))
        }
        IntrinsicTypeOperation::LabeledTypePackMap => {
            let transform_fn = match arguments.first()? {
                TypeMemberArgument::Type(ty) => *ty,
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let elems = match subject_data? {
                TypeData::LabeledTypePack(elems) => elems,
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let mut transformed_elements = Vec::with_capacity(elems.len());
            for elem in elems.iter() {
                let applied = store.apply_type_form(transform_fn, &[elem.element]).unwrap_or(elem.element);
                let norm_res = normalize_type(store, applied, context);
                let Some(norm_ty) = norm_res.type_id() else { return Some(norm_res) };
                transformed_elements.push(super::store::LabeledTypePackElement {
                    label: elem.label.clone(),
                    element: norm_ty,
                });
            }
            let normalized = store.labeled_type_pack(transformed_elements.into_boxed_slice()).ok()?;
            Some(ProjectionNormalizationResult::Normalized(normalized))
        }
        IntrinsicTypeOperation::RecordFields => {
            // The projection remains a TypeId here because normalization is
            // TypeId-valued. Record-row consumers recover an exact known row
            // through the row-conversion seam in type annotation formation.
            Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback))
        }
        IntrinsicTypeOperation::RecordValue => {
            let row_id = match subject_data? {
                TypeData::Record(row_id) => row_id,
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let row = store.record_row(row_id).clone();
            if row.tail != crate::types::row::RecordRowTail::Closed {
                Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback))
            } else if row.fields.is_empty() {
                Some(ProjectionNormalizationResult::Normalized(store.never()))
            } else {
                let field_types: Vec<TypeId> = row.fields.iter().map(|f| f.ty).collect();
                let union_ty = store.union(&field_types);
                Some(ProjectionNormalizationResult::Normalized(union_ty))
            }
        }
        IntrinsicTypeOperation::RecordRowMap => Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
        IntrinsicTypeOperation::CompleteTypePackPositionals => {
            match subject_data? {
                TypeData::CompleteTypePack { positionals, .. } => {
                    Some(ProjectionNormalizationResult::Normalized(positionals))
                }
                _ => Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            }
        }
        IntrinsicTypeOperation::CompleteTypePackLabeled => {
            match subject_data? {
                TypeData::CompleteTypePack { labeled, .. } => {
                    Some(ProjectionNormalizationResult::Normalized(labeled))
                }
                _ => Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            }
        }
        IntrinsicTypeOperation::CompleteTypePackMap => {
            let transform_fn = match arguments.first()? {
                TypeMemberArgument::Type(ty) => *ty,
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let (pos_ty, lab_ty) = match subject_data? {
                TypeData::CompleteTypePack { positionals, labeled } => (positionals, labeled),
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let pos_elems = match store.get(pos_ty).clone() {
                TypeData::TypePack(elems) => elems,
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let mut transformed_pos = Vec::with_capacity(pos_elems.len());
            for &elem in pos_elems.iter() {
                let applied = store.apply_type_form(transform_fn, &[elem]).unwrap_or(elem);
                let norm_res = normalize_type(store, applied, context);
                let Some(norm_ty) = norm_res.type_id() else { return Some(norm_res) };
                transformed_pos.push(norm_ty);
            }
            let norm_pos = store.type_pack(transformed_pos.into_boxed_slice()).ok()?;

            let lab_elems = match store.get(lab_ty).clone() {
                TypeData::LabeledTypePack(elems) => elems,
                _ => return Some(ProjectionNormalizationResult::Symbolic(symbolic_fallback)),
            };
            let mut transformed_lab = Vec::with_capacity(lab_elems.len());
            for elem in lab_elems.iter() {
                let applied = store.apply_type_form(transform_fn, &[elem.element]).unwrap_or(elem.element);
                let norm_res = normalize_type(store, applied, context);
                let Some(norm_ty) = norm_res.type_id() else { return Some(norm_res) };
                transformed_lab.push(super::store::LabeledTypePackElement {
                    label: elem.label.clone(),
                    element: norm_ty,
                });
            }
            let norm_lab = store.labeled_type_pack(transformed_lab.into_boxed_slice()).ok()?;

            let normalized = store.complete_type_pack(norm_pos, norm_lab).ok()?;
            Some(ProjectionNormalizationResult::Normalized(normalized))
        }
    }
}
