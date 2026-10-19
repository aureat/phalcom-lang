//! Certified PERF003 P7 semantic-denotation reconstruction adapted to ARTF001.
//!
//! Denotations are semantic authority carried by callable binding/expression
//! rows. They are reconstructed through the same project-qualified owner and
//! type relocation as the rest of the imported compiler image. Checked static
//! applications remain fail-closed until their application rows are imported.

use super::identity::StableOwnerCatalog;
use super::type_import::ReconstructedTypeGraph;
use crate::associated::AssociatedMemberId;
use crate::checker::associated::BehavioralFamilySpec;
use crate::dispatch::ConditionalDispatchSelection;
use crate::identity::{
    AssociatedFamilyId, AssociatedStaticDomainDeclarationId, DataComponentId, DataConstructorId,
    DispatchSide, InvocationTargetId,
};
use crate::types::denotation::{
    AssociatedValueDenotation, BehavioralFamilyDenotation, CapturedAssociatedMember,
    CapturedBehavioralMember, SemanticDenotation,
};
use crate::types::family::FamilyOperationShape;
use crate::types::type_member::{TypeMemberDeclarationId, TypeMemberSelector};
use phalcom_common::selector::{
    Selector, SelectorBase, SelectorKind, SelectorKindPattern, SelectorPattern, SelectorSlot,
};
use phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1;
use phalcom_compiler_artifact::semantic::world::{
    AssociatedMemberV1, BehavioralFamilySpecV1, ConditionalDispatchSelectionV1,
    ExactSelectorV1, FamilyOperationShapeV1, InvocationTargetV1, SelectorKindPatternV1,
    SelectorKindV1, SelectorPatternSpecV1, SelectorSlotV1, SemanticDenotationV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;

pub(crate) fn reconstruct_semantic_denotation(
    denotation: &SemanticDenotationV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<SemanticDenotation, ArtifactError> {
    Ok(match denotation {
        SemanticDenotationV1::TypeForm { ty } => {
            SemanticDenotation::TypeForm(mapped(&types.types, ty.0, "denotation type")?)
        }
        SemanticDenotationV1::Kind { kind } => {
            SemanticDenotation::Kind(mapped(&types.kinds, kind.0, "denotation kind")?)
        }
        SemanticDenotationV1::StaticDomain { domain } => {
            SemanticDenotation::StaticDomain(mapped(
                &types.static_domains,
                domain.0,
                "denotation static domain",
            )?)
        }
        SemanticDenotationV1::StaticTerm { term } => {
            SemanticDenotation::StaticTerm(mapped(&types.types, term.0, "denotation static term")?)
        }
        SemanticDenotationV1::StaticApplication { .. } => {
            return Err(invalid(
                "static-application denotation has no reconstructed checked-body application authority",
            ));
        }
        SemanticDenotationV1::AssociatedValue { value } => {
            use phalcom_compiler_artifact::semantic::world::AssociatedValueDenotationV1 as Wire;
            let value = match value {
                Wire::Exact {
                    owner_form,
                    lookup_owner,
                    member,
                    target,
                } => {
                    let target = target
                        .as_ref()
                        .map(|target| invocation_target(target, owners))
                        .transpose()?;
                    AssociatedValueDenotation::Exact {
                        owner_form: mapped(&types.types, owner_form.0, "associated owner form")?,
                        lookup_owner: owners.declaration(lookup_owner)?,
                        member: associated_member(member, owners, allocations)?,
                        target: allocations.boxed(target).map_err(ArtifactError::Allocation)?,
                    }
                }
                Wire::Family {
                    owner_form,
                    lookup_owner,
                    family,
                    members,
                } => {
                    let mut imported = allocations
                        .vector::<CapturedAssociatedMember>(members.len())
                        .map_err(ArtifactError::Allocation)?;
                    for member in members {
                        imported.push(CapturedAssociatedMember {
                            operation: family_operation(&member.operation, allocations)?,
                            member: associated_member(&member.member, owners, allocations)?,
                            target: member
                                .target
                                .as_ref()
                                .map(|target| invocation_target(target, owners))
                                .transpose()?,
                        });
                    }
                    AssociatedValueDenotation::Family {
                        owner_form: mapped(&types.types, owner_form.0, "associated family owner form")?,
                        lookup_owner: owners.declaration(lookup_owner)?,
                        family: AssociatedFamilyId::new(
                            owners.declaration(&family.owner)?,
                            selector_base(&family.selector, allocations)?,
                        ),
                        members: allocations.arc_slice(imported).map_err(ArtifactError::Allocation)?,
                    }
                }
            };
            SemanticDenotation::AssociatedValue(allocations.boxed(value).map_err(ArtifactError::Allocation)?)
        }
        SemanticDenotationV1::BehavioralFamily { family } => {
            let mut members = allocations
                .vector::<CapturedBehavioralMember>(family.members.len())
                .map_err(ArtifactError::Allocation)?;
            for member in &family.members {
                members.push(CapturedBehavioralMember {
                    operation: family_operation(&member.operation, allocations)?,
                    target: invocation_target(&member.target, owners)?,
                    conditional: member
                        .conditional
                        .as_ref()
                        .map(|selection| conditional_selection(selection, owners))
                        .transpose()?,
                });
            }
            let family = BehavioralFamilyDenotation {
                receiver_type: mapped(&types.types, family.receiver_type.0, "behavioral family receiver")?,
                receiver_application: family
                    .receiver_application
                    .map(|application| mapped(&types.types, application.0, "behavioral family receiver application"))
                    .transpose()?,
                spec: behavioral_spec(&family.spec, allocations)?,
                members: allocations.arc_slice(members).map_err(ArtifactError::Allocation)?,
            };
            SemanticDenotation::BehavioralFamily(allocations.boxed(family).map_err(ArtifactError::Allocation)?)
        }
    })
}

fn associated_member(
    member: &AssociatedMemberV1,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<AssociatedMemberId, ArtifactError> {
    Ok(match member {
        AssociatedMemberV1::Variant { variant } => AssociatedMemberId::Variant(owners.variant(variant)?),
        AssociatedMemberV1::DataConstructor { owner } => {
            AssociatedMemberId::DataConstructor(DataConstructorId::new(owners.declaration(owner)?))
        }
        AssociatedMemberV1::DataComponent { owner, index } => {
            AssociatedMemberId::DataComponent(DataComponentId::new(owners.declaration(owner)?, *index))
        }
        AssociatedMemberV1::StaticDomain { declaration } => {
            AssociatedMemberId::StaticDomain(AssociatedStaticDomainDeclarationId::new(
                owners.declaration(&declaration.owner)?,
                allocations.boxed_str(&declaration.name).map_err(ArtifactError::Allocation)?,
            ))
        }
        AssociatedMemberV1::TypeMember { owner, selector } => {
            AssociatedMemberId::TypeMember(TypeMemberDeclarationId::new(
                owners.declaration(owner)?,
                type_member_selector(selector, allocations)?,
            ))
        }
    })
}

fn invocation_target(
    target: &InvocationTargetV1,
    owners: &StableOwnerCatalog<'_>,
) -> Result<InvocationTargetId, ArtifactError> {
    Ok(match target {
        InvocationTargetV1::Behavioral { callable } => {
            InvocationTargetId::Behavioral(owners.callable(callable)?)
        }
        InvocationTargetV1::VariantConstructor { variant } => {
            InvocationTargetId::variant_constructor(owners.variant(variant)?)
        }
        InvocationTargetV1::DataConstructor { owner } => {
            InvocationTargetId::data_constructor(owners.declaration(owner)?)
        }
    })
}

fn conditional_selection(
    selection: &ConditionalDispatchSelectionV1,
    owners: &StableOwnerCatalog<'_>,
) -> Result<ConditionalDispatchSelection, ArtifactError> {
    let side = match selection.side {
        0 => DispatchSide::Instance,
        1 => DispatchSide::Class,
        _ => return Err(invalid("conditional dispatch selection has an invalid side")),
    };
    let callable = owners.callable(&selection.callable)?;
    let declaring_owner = owners.declaration(&selection.declaring_owner)?;
    if callable.side != side {
        return Err(invalid("conditional dispatch callable side disagrees with selection"));
    }
    Ok(ConditionalDispatchSelection {
        impl_id: owners.implementation(&selection.impl_id)?,
        callable,
        declaring_owner,
        side,
    })
}

fn family_operation(
    operation: &FamilyOperationShapeV1,
    allocations: &mut AllocationBudget,
) -> Result<FamilyOperationShape, ArtifactError> {
    let mut slots = allocations
        .vector::<SelectorSlot>(operation.slots.len())
        .map_err(ArtifactError::Allocation)?;
    for slot in &operation.slots {
        slots.push(selector_slot(slot, allocations)?);
    }
    Ok(FamilyOperationShape::new(
        selector_kind(&operation.kind),
        allocations.boxed_slice(slots).map_err(ArtifactError::Allocation)?,
    ))
}

fn behavioral_spec(
    spec: &BehavioralFamilySpecV1,
    allocations: &mut AllocationBudget,
) -> Result<BehavioralFamilySpec, ArtifactError> {
    Ok(match spec {
        BehavioralFamilySpecV1::Exact { selector } => {
            BehavioralFamilySpec::Exact(exact_selector(selector, allocations)?)
        }
        BehavioralFamilySpecV1::Pattern { pattern } => {
            BehavioralFamilySpec::Pattern(selector_pattern(pattern, allocations)?)
        }
    })
}

fn exact_selector(
    selector: &ExactSelectorV1,
    allocations: &mut AllocationBudget,
) -> Result<Selector, ArtifactError> {
    let mut slots = allocations
        .vector::<SelectorSlot>(selector.slots.len())
        .map_err(ArtifactError::Allocation)?;
    for slot in &selector.slots {
        slots.push(selector_slot(slot, allocations)?);
    }
    Selector::new(
        selector_base(&selector.base, allocations)?,
        selector_kind(&selector.kind),
        allocations.boxed_slice(slots).map_err(ArtifactError::Allocation)?,
    )
    .map_err(|_| invalid("behavioral family exact selector is not canonical"))
}

fn selector_pattern(
    pattern: &SelectorPatternSpecV1,
    allocations: &mut AllocationBudget,
) -> Result<SelectorPattern, ArtifactError> {
    let mut prefix = allocations
        .vector::<SelectorSlot>(pattern.prefix.len())
        .map_err(ArtifactError::Allocation)?;
    for slot in &pattern.prefix {
        prefix.push(selector_slot(slot, allocations)?);
    }
    let mut suffix = allocations
        .vector::<SelectorSlot>(pattern.suffix.len())
        .map_err(ArtifactError::Allocation)?;
    for slot in &pattern.suffix {
        suffix.push(selector_slot(slot, allocations)?);
    }
    SelectorPattern::new(
        selector_base(&pattern.base, allocations)?,
        selector_kind_pattern(&pattern.kind),
        allocations.boxed_slice(prefix).map_err(ArtifactError::Allocation)?,
        allocations.boxed_slice(suffix).map_err(ArtifactError::Allocation)?,
        pattern.has_gap,
    )
    .map_err(|_| invalid("behavioral family selector pattern is not canonical"))
}

fn selector_base(
    selector: &TypeMemberSelectorV1,
    allocations: &mut AllocationBudget,
) -> Result<SelectorBase, ArtifactError> {
    Ok(match selector {
        TypeMemberSelectorV1::Named { name } => {
            SelectorBase::Named(allocations.string(name).map_err(ArtifactError::Allocation)?)
        }
        TypeMemberSelectorV1::Subscript => SelectorBase::Subscript,
    })
}

fn type_member_selector(
    selector: &TypeMemberSelectorV1,
    allocations: &mut AllocationBudget,
) -> Result<TypeMemberSelector, ArtifactError> {
    Ok(match selector {
        TypeMemberSelectorV1::Named { name } => {
            if name.as_ref() == "At" {
                return Err(invalid("noncanonical type-member selector aliases subscript identity"));
            }
            TypeMemberSelector::Named(allocations.boxed_str(name).map_err(ArtifactError::Allocation)?)
        }
        TypeMemberSelectorV1::Subscript => TypeMemberSelector::Subscript,
    })
}

fn selector_kind(kind: &SelectorKindV1) -> SelectorKind {
    match kind {
        SelectorKindV1::Getter => SelectorKind::Getter,
        SelectorKindV1::Setter => SelectorKind::Setter,
        SelectorKindV1::Method => SelectorKind::Method,
        SelectorKindV1::SubscriptGet => SelectorKind::SubscriptGet,
        SelectorKindV1::SubscriptSet => SelectorKind::SubscriptSet,
    }
}

fn selector_kind_pattern(kind: &SelectorKindPatternV1) -> SelectorKindPattern {
    match kind {
        SelectorKindPatternV1::AnyNamed => SelectorKindPattern::AnyNamed,
        SelectorKindPatternV1::NamedAccessors => SelectorKindPattern::NamedAccessors,
        SelectorKindPatternV1::AnySubscript => SelectorKindPattern::AnySubscript,
        SelectorKindPatternV1::Exact { kind } => SelectorKindPattern::Exact(selector_kind(kind)),
    }
}

fn selector_slot(
    slot: &SelectorSlotV1,
    allocations: &mut AllocationBudget,
) -> Result<SelectorSlot, ArtifactError> {
    Ok(match slot {
        SelectorSlotV1::Positional => SelectorSlot::Positional,
        SelectorSlotV1::Label { label } => {
            SelectorSlot::Label(allocations.string(label).map_err(ArtifactError::Allocation)?)
        }
    })
}

fn mapped<T: Copy>(rows: &[T], raw: u32, family: &'static str) -> Result<T, ArtifactError> {
    rows.get(raw as usize)
        .copied()
        .ok_or_else(|| invalid(format!("{family} references an absent relocated row")))
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
