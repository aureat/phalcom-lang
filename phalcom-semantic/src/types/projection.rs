//! Canonical associated-type projection normalization.

use super::evidence::UnknownReason;
use super::id::TypeId;
use super::outcome::{BlockReason, BudgetReport, CancellationToken, DynamicBoundaryObligation, QueryBudget};
use super::row::RecordRowField;
use super::family::FamilyMemberType;
use super::store::{AssociatedTypeProjection, CallableParameterType, CallableType, TupleTypeElement, TypeData, TypeStore};
use crate::declarations::DeclarationTypeTable;
use crate::identity::ImplId;
use crate::impls::{ConformanceAssociatedTypePlan, ConformanceIndex, ConformanceResolution, ConformanceWitnessPlan};
use crate::traits::{TraitRef, TraitSurfaceTable};
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

impl ProjectionNormalizationResult {
    fn type_id(&self) -> Option<TypeId> {
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
                let result = normalize_type(store, argument, context);
                symbolic |= result.is_symbolic();
                let Some(argument) = result.type_id() else { return result };
                normalized_arguments.push(argument);
            }
            let normalized = store.apply_type_form(origin, &normalized_arguments).unwrap_or(ty);
            if symbolic { ProjectionNormalizationResult::Symbolic(normalized) } else { ProjectionNormalizationResult::Normalized(normalized) }
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
            if symbolic { ProjectionNormalizationResult::Symbolic(normalized) } else { ProjectionNormalizationResult::Normalized(normalized) }
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
            if symbolic { ProjectionNormalizationResult::Symbolic(normalized) } else { ProjectionNormalizationResult::Normalized(normalized) }
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
            if symbolic { ProjectionNormalizationResult::Symbolic(normalized) } else { ProjectionNormalizationResult::Normalized(normalized) }
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
            if symbolic { ProjectionNormalizationResult::Symbolic(normalized) } else { ProjectionNormalizationResult::Normalized(normalized) }
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
            let normalized = store.callable(CallableType { parameters: parameters.into_boxed_slice(), return_type });
            if symbolic { ProjectionNormalizationResult::Symbolic(normalized) } else { ProjectionNormalizationResult::Normalized(normalized) }
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
            if symbolic { ProjectionNormalizationResult::Symbolic(normalized) } else { ProjectionNormalizationResult::Normalized(normalized) }
        }
        _ => ProjectionNormalizationResult::Normalized(ty),
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
        ProjectionNormalizationMode::SourceConformance { trait_ref: current, target, plan } => {
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
            let result = normalize_type(store, binding.value_template, context);
            context.active.remove(&projection);
            result
        }
        ProjectionNormalizationMode::Exact {
            conformance_index,
            witness_plans,
            trait_surfaces,
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
                let result = normalize_type(store, binding.value, context);
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
            let evidence = match resolution {
                ConformanceResolution::Proven(evidence) => evidence,
                ConformanceResolution::NotDeclared => return ProjectionNormalizationResult::Unknown(UnknownReason::UnderconstrainedTypeVariable),
                ConformanceResolution::InvalidSource(_) | ConformanceResolution::Incomplete(_) => return ProjectionNormalizationResult::Incomplete,
                ConformanceResolution::CoherenceConflict(candidates) => return ProjectionNormalizationResult::Ambiguous(candidates),
                ConformanceResolution::Unknown(reason) => return ProjectionNormalizationResult::Unknown(reason),
                ConformanceResolution::Blocked(reason) => return ProjectionNormalizationResult::Blocked(reason),
                ConformanceResolution::Dynamic(obligation) => return ProjectionNormalizationResult::Dynamic(obligation),
                ConformanceResolution::Cancelled => return ProjectionNormalizationResult::Cancelled,
                ConformanceResolution::BudgetExceeded(report) => return ProjectionNormalizationResult::BudgetExceeded(report),
                ConformanceResolution::InternalFailure(message) => return ProjectionNormalizationResult::InternalFailure(message),
            };
            let Some(binding) = evidence.associated_types.get(&projection.requirement) else {
                return ProjectionNormalizationResult::Incomplete;
            };
            if !context.active.insert(projection.clone()) {
                return ProjectionNormalizationResult::Recursive;
            }
            let result = normalize_type(store, binding.value, context);
            context.active.remove(&projection);
            result
        }
    }
}
