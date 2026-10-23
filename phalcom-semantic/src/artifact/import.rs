//! Semantic-owned artifact import for structural primary contracts (`PERF003.C2.P7.T7.A`).

mod diagnostic;
mod explanation;
pub(super) mod proof;

use crate::artifact::identity::StableOwnerCatalog;
use crate::data_semantics::{
    DataComponentSemantic, DataConstructorParameter, DataConstructorParameterContribution, DataConstructorSignature, DataConstructorTemplate, DataInfo,
    DataProductContribution, DataProductExpansionSemantic, DataProductShapeTemplate, DataSemanticTable, DataShape,
};
use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, DeclaredTypeState};
use crate::declarations::{DeclarationTypeBinding, DeclarationTypeInfo, DeclarationTypeTable};
use crate::diagnostic::{DiagnosticCode, SemanticSourceSpan};
use crate::enum_requirements::{CaseRequirementResult, CaseRequirementStatus, EnumRequirement, EnumRequirementId, EnumRequirementRows, EnumRequirementTable};
use crate::enum_semantics::{
    AlgebraicCasePayloadKind, EnumInfo, EnumSemanticTable, VariantConstructorParameter, VariantConstructorSignature, VariantFieldSemantic, VariantInfo,
    VariantShape, VariantVisibility,
};
use crate::identity::{
    BindingId, CallableId, CallableOwnerId, CallableParameterId, DataComponentId, DataConstructorId, DeclarationId, DiagnosticCauseId, DispatchSide,
    ExplanationId, SourceOwner, SourceSiteId, SourceSiteLocalId, VariantConstructorId, VariantFieldId,
};
use crate::metadata::stable_identity::{
    from_stable_callable, from_stable_declaration, from_stable_dispatch_side, from_stable_impl, from_stable_module, from_stable_project, from_stable_variant,
};
use crate::type_alias::{TypeAliasInfo, TypeAliasTable};
use crate::types::case_environment::CaseTypeEnvironment;
use crate::types::evidence::{DynamicReason, UnknownReason};
use crate::types::parameter::{
    GenericBinderHeader, GenericConstraint, GenericParameterMode, GenericSignature, SelfRole, SelfTypeTerm, TypeParameterOwner, TypeTerm,
};
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::TypeStore;
use crate::types::store::artifact::TypeRelocation;
use crate::types::variance::Variance;
use phalcom_common::modifier::Visibility;
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::semantic::aggregates::*;
use phalcom_compiler_artifact::semantic::type_store::{ParameterModeV1, ParameterOwnerV1, SelfRoleV1, VarianceV1};
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::identity::{StableCallableOwnerRef, StableCallableRef};
use std::collections::BTreeMap;
use std::sync::Arc;

#[cfg(feature = "allocation-diagnostics")]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "allocation-diagnostics")]
const IMPORT_ALLOCATION_DIAGNOSTIC_STAGE_NAMES: [&str; 20] = [
    "start",
    "validation",
    "module execution authority",
    "type graph",
    "structural contracts",
    "static values",
    "callable signatures, definitions, and enum requirements",
    "trait surfaces",
    "dispatch surfaces and field signatures",
    "conditional inherent authority",
    "conformance heads",
    "witness plans",
    "callable analyses",
    "derived conformance and trait dispatch",
    "constructor initializations",
    "associated and static owners",
    "formal projection",
    "snapshot construction",
    "module owner indexes",
    "complete",
];

#[cfg(feature = "allocation-diagnostics")]
static IMPORT_ALLOCATION_DIAGNOSTIC_OBSERVED: [AtomicUsize; IMPORT_ALLOCATION_DIAGNOSTIC_STAGE_NAMES.len()] =
    [const { AtomicUsize::new(usize::MAX) }; IMPORT_ALLOCATION_DIAGNOSTIC_STAGE_NAMES.len()];
#[cfg(feature = "allocation-diagnostics")]
static IMPORT_ALLOCATION_DIAGNOSTIC_LEDGER: [AtomicUsize; IMPORT_ALLOCATION_DIAGNOSTIC_STAGE_NAMES.len()] =
    [const { AtomicUsize::new(usize::MAX) }; IMPORT_ALLOCATION_DIAGNOSTIC_STAGE_NAMES.len()];

#[cfg(feature = "allocation-diagnostics")]
const CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_NAMES: [&str; 59] = [
    "callable table buffers",
    "callable binding and import-phase facts",
    "callable expression facts and statuses",
    "callable dispatch authority",
    "callable specializations",
    "callable denotations",
    "callable expression index",
    "callable resolution authority",
    "callable proof authority",
    "callable flow authority",
    "callable exits and source owners",
    "callable explanation and diagnostics",
    "callable dependency rows",
    "callable product assembly",
    "callable product fingerprint",
    "callable table publication",
    "dispatch setup",
    "dispatch conditional capabilities",
    "dispatch trait sites",
    "dispatch candidates",
    "dispatch contextual realizations",
    "resolution setup",
    "resolution iteration lanes",
    "resolution positional expansions",
    "resolution invalid expansions",
    "resolution sequence literals",
    "resolution associated members",
    "resolution callable references",
    "resolution family applications",
    "Exact evidence setup",
    "Exact evidence requirement views",
    "Exact evidence requirements",
    "Exact evidence associated types",
    "Exact evidence trait contract",
    "Exact evidence header proofs",
    "Exact evidence fingerprint computation",
    "Exact evidence validation and publication",
    "Exact requirement-view Debug formatting",
    "Exact requirement-selection Debug formatting",
    "witness-plan staging buffers",
    "witness-plan identities and generic header",
    "witness-plan requirement views",
    "witness-plan requirement selections and evidence",
    "witness-plan associated types",
    "witness-plan completeness and diagnostics",
    "witness-plan construction and fingerprint",
    "witness-plan source fingerprints and rows",
    "witness-plan root publication",
    "structural contract validation",
    "structural generic-signature index",
    "structural data table",
    "structural enum and variant tables",
    "structural type aliases",
    "structural hierarchy",
    "structural declarations and intrinsic owners",
    "structural root publication",
    "callable signature table",
    "callable definition table",
    "enum requirement table",
];

#[cfg(feature = "allocation-diagnostics")]
static CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_OBSERVED: [AtomicUsize; CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_NAMES.len()] =
    [const { AtomicUsize::new(0) }; CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_NAMES.len()];
#[cfg(feature = "allocation-diagnostics")]
static CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_LEDGER: [AtomicUsize; CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_NAMES.len()] =
    [const { AtomicUsize::new(0) }; CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_NAMES.len()];

/// One allocation-census checkpoint from the explicitly enabled diagnostic importer.
#[cfg(feature = "allocation-diagnostics")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImportAllocationDiagnosticStage {
    pub name: &'static str,
    pub observed_bytes: usize,
    pub ledger_bytes: usize,
}

/// Clears the fixed-size stage census before one diagnostic import.
#[cfg(feature = "allocation-diagnostics")]
pub fn reset_import_allocation_diagnostics() {
    for observed in &IMPORT_ALLOCATION_DIAGNOSTIC_OBSERVED {
        observed.store(usize::MAX, Ordering::Relaxed);
    }
    for ledger in &IMPORT_ALLOCATION_DIAGNOSTIC_LEDGER {
        ledger.store(usize::MAX, Ordering::Relaxed);
    }
    for observed in &CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_OBSERVED {
        observed.store(0, Ordering::Relaxed);
    }
    for ledger in &CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_LEDGER {
        ledger.store(0, Ordering::Relaxed);
    }
}

/// Returns allocation checkpoints after a diagnostic import has completed.
#[cfg(feature = "allocation-diagnostics")]
pub fn import_allocation_diagnostic_stages() -> Vec<ImportAllocationDiagnosticStage> {
    IMPORT_ALLOCATION_DIAGNOSTIC_STAGE_NAMES
        .iter()
        .enumerate()
        .filter_map(|(index, name)| {
            let observed_bytes = IMPORT_ALLOCATION_DIAGNOSTIC_OBSERVED[index].load(Ordering::Relaxed);
            let ledger_bytes = IMPORT_ALLOCATION_DIAGNOSTIC_LEDGER[index].load(Ordering::Relaxed);
            (observed_bytes != usize::MAX && ledger_bytes != usize::MAX).then_some(ImportAllocationDiagnosticStage {
                name,
                observed_bytes,
                ledger_bytes,
            })
        })
        .collect()
}

/// Returns summed allocation intervals for the internal callable importer.
#[cfg(feature = "allocation-diagnostics")]
pub fn callable_import_allocation_diagnostic_stages() -> Vec<ImportAllocationDiagnosticStage> {
    CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_NAMES
        .iter()
        .enumerate()
        .map(|(index, name)| ImportAllocationDiagnosticStage {
            name,
            observed_bytes: CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_OBSERVED[index].load(Ordering::Relaxed),
            ledger_bytes: CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_LEDGER[index].load(Ordering::Relaxed),
        })
        .collect()
}

#[cfg(feature = "allocation-diagnostics")]
fn record_import_allocation_diagnostic_stage(index: usize, allocations: &AllocationBudget) {
    let observed_bytes = phalcom_type_meta::allocation::current_observed_bytes().unwrap_or(usize::MAX);
    IMPORT_ALLOCATION_DIAGNOSTIC_OBSERVED[index].store(observed_bytes, Ordering::Relaxed);
    IMPORT_ALLOCATION_DIAGNOSTIC_LEDGER[index].store(allocations.requested(), Ordering::Relaxed);
}

#[cfg(feature = "allocation-diagnostics")]
fn record_callable_import_allocation_diagnostic_stage(
    index: usize,
    allocations: &AllocationBudget,
    previous_observed: &mut usize,
    previous_ledger: &mut usize,
) {
    let observed = phalcom_type_meta::allocation::current_observed_bytes().unwrap_or(*previous_observed);
    let ledger = allocations.requested();
    CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_OBSERVED[index].fetch_add(observed.saturating_sub(*previous_observed), Ordering::Relaxed);
    CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_LEDGER[index].fetch_add(ledger.saturating_sub(*previous_ledger), Ordering::Relaxed);
    *previous_observed = observed;
    *previous_ledger = ledger;
}

#[cfg(feature = "allocation-diagnostics")]
pub(crate) fn record_callable_import_observed_allocation_diagnostic(index: usize, previous_observed: usize) {
    let observed = phalcom_type_meta::allocation::current_observed_bytes().unwrap_or(previous_observed);
    CALLABLE_IMPORT_ALLOCATION_DIAGNOSTIC_OBSERVED[index].fetch_add(observed.saturating_sub(previous_observed), Ordering::Relaxed);
}

#[cfg(not(feature = "allocation-diagnostics"))]
#[inline(always)]
pub(crate) fn record_callable_import_observed_allocation_diagnostic(_index: usize, _previous_observed: usize) {}

#[cfg(not(feature = "allocation-diagnostics"))]
#[inline(always)]
fn record_callable_import_allocation_diagnostic_stage(
    _index: usize,
    _allocations: &AllocationBudget,
    _previous_observed: &mut usize,
    _previous_ledger: &mut usize,
) {
}

#[cfg(not(feature = "allocation-diagnostics"))]
#[inline(always)]
fn record_import_allocation_diagnostic_stage(_index: usize, _allocations: &AllocationBudget) {}

#[cfg(test)]
use crate::identity::VariantId;

fn referential(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}

pub fn import_parameter_owner(owner: &ParameterOwnerV1) -> Result<TypeParameterOwner, ArtifactError> {
    Ok(match owner {
        ParameterOwnerV1::Declaration { owner } => TypeParameterOwner::Declaration(from_stable_declaration(owner).map_err(referential)?),
        ParameterOwnerV1::Callable { owner } => TypeParameterOwner::Callable(from_stable_callable(owner).map_err(referential)?),
        ParameterOwnerV1::Impl { owner } => TypeParameterOwner::Impl(from_stable_impl(owner).map_err(referential)?),
    })
}

pub fn import_variance(variance: &VarianceV1) -> Variance {
    match variance {
        VarianceV1::Covariant => Variance::Covariant,
        VarianceV1::Contravariant => Variance::Contravariant,
        VarianceV1::Invariant => Variance::Invariant,
    }
}

pub fn import_mode(mode: &ParameterModeV1) -> GenericParameterMode {
    match mode {
        ParameterModeV1::Single => GenericParameterMode::Single,
        ParameterModeV1::PositionalPack => GenericParameterMode::PositionalPack,
        ParameterModeV1::LabeledPack => GenericParameterMode::LabeledPack,
        ParameterModeV1::CompletePack => GenericParameterMode::CompletePack,
    }
}

pub fn import_self_role(role: &SelfRoleV1) -> SelfRole {
    match role {
        SelfRoleV1::InstanceType => SelfRole::InstanceType,
        SelfRoleV1::ReceiverValue => SelfRole::ReceiverValue,
    }
}

pub fn import_type_term(term: &TypeTermV1, relocation: &TypeRelocation) -> Result<TypeTerm, ArtifactError> {
    Ok(match term {
        TypeTermV1::Canonical { ty } => TypeTerm::Canonical(relocation.ty(*ty)?),
        TypeTermV1::SelfType { term } => TypeTerm::SelfType(SelfTypeTerm {
            owner: from_stable_declaration(&term.owner).map_err(referential)?,
            side: from_stable_dispatch_side(term.side),
            role: import_self_role(&term.role),
        }),
    })
}

pub(crate) fn import_trait_ref_identity(
    trait_ref: &TraitRefV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::traits::TraitRef, ArtifactError> {
    let mut arguments = allocations
        .vector::<crate::types::id::TypeId>(trait_ref.arguments.len())
        .map_err(allocation_error)?;
    for argument in &trait_ref.arguments {
        arguments.push(relocation.ty(*argument)?);
    }
    Ok(crate::traits::TraitRef::new(
        import_stable_declaration_with_allocation_budget(&trait_ref.declaration, allocations)?,
        allocations.boxed_slice(arguments).map_err(allocation_error)?,
    ))
}

pub(crate) fn import_stable_module_with_allocation_budget(
    stable: &phalcom_type_meta::identity::StableModuleRef,
    allocations: &mut AllocationBudget,
) -> Result<crate::identity::ModuleId, ArtifactError> {
    let project = from_stable_project(&stable.project).map_err(referential)?;
    let mut components = allocations
        .vector::<crate::identity::ModuleComponent>(stable.path.len())
        .map_err(allocation_error)?;
    for component in stable.path.iter() {
        let component = allocations.boxed_str(component).map_err(allocation_error)?;
        components.push(
            crate::identity::ModuleComponent::from_owned_identifier(component)
                .map_err(|_| referential("callable owner module identity has an invalid path component"))?,
        );
    }
    Ok(crate::identity::ModuleId {
        project,
        path: crate::identity::ModulePath::from_components(allocations.boxed_slice(components).map_err(allocation_error)?),
    })
}

pub(crate) fn import_stable_declaration_with_allocation_budget(
    stable: &phalcom_type_meta::identity::StableDeclarationRef,
    allocations: &mut AllocationBudget,
) -> Result<crate::identity::DeclarationId, ArtifactError> {
    if stable.path.len() != 1 {
        return Err(referential("callable owner declaration identity must contain exactly one declaration name"));
    }
    let module = import_stable_module_with_allocation_budget(&stable.module, allocations)?;
    let name = allocations.boxed_str(&stable.path[0]).map_err(allocation_error)?;
    Ok(crate::identity::DeclarationId::new(module, name))
}

pub(crate) fn import_stable_selector_with_allocation_budget(
    text: &str,
    allocations: &mut AllocationBudget,
) -> Result<phalcom_common::selector::Selector, ArtifactError> {
    use phalcom_common::selector::{Selector, SelectorBase, SelectorKind, SelectorSlot};

    if text.is_empty() {
        return Err(referential("callable owner selector is empty"));
    }
    let (base, kind, slots_text) = if let Some(rest) = text.strip_prefix('[') {
        if let Some(inner) = rest.strip_suffix("]=(_)") {
            (None, SelectorKind::SubscriptSet, Some(inner))
        } else if let Some(inner) = rest.strip_suffix(']') {
            (None, SelectorKind::SubscriptGet, Some(inner))
        } else {
            return Err(referential("callable owner subscript selector is malformed"));
        }
    } else if let Some(open) = text.find('(') {
        if open == 0 || !text.ends_with(')') {
            return Err(referential("callable owner selector is malformed"));
        }
        let head = &text[..open];
        let inner = &text[open + 1..text.len() - 1];
        if inner == "_" && head.ends_with('=') && head.chars().any(|character| character.is_alphanumeric() || character == '_') {
            let name = head
                .strip_suffix('=')
                .ok_or_else(|| referential("callable owner setter selector is malformed"))?;
            (Some(name), SelectorKind::Setter, None)
        } else {
            (Some(head), SelectorKind::Method, Some(inner))
        }
    } else {
        (Some(text), SelectorKind::Getter, None)
    };

    let base = match base {
        Some(name) => SelectorBase::Named(allocations.string(name).map_err(allocation_error)?),
        None => SelectorBase::Subscript,
    };
    let Some(slots_text) = slots_text else {
        return Selector::new(base, kind, allocations.boxed_slice(Vec::new()).map_err(allocation_error)?)
            .map_err(|_| referential("callable owner selector is invalid"));
    };
    if slots_text.trim().is_empty() && !slots_text.is_empty() {
        return Err(referential("callable owner selector is not in canonical encoding"));
    }
    let mut slots = if slots_text.trim().is_empty() {
        Vec::new()
    } else {
        let count = slots_text.split(',').count();
        allocations.vector::<SelectorSlot>(count).map_err(allocation_error)?
    };
    if !slots_text.trim().is_empty() {
        for raw_slot in slots_text.split(',') {
            let slot = raw_slot.trim();
            if slot.is_empty() {
                return Err(referential("callable owner selector contains an empty slot"));
            }
            if raw_slot != slot {
                return Err(referential("callable owner selector is not in canonical encoding"));
            }
            if matches!(slot, "_" | "*" | "**" | "***") {
                if slot != "_" {
                    return Err(referential("callable owner selector is not in canonical encoding"));
                }
                slots.push(SelectorSlot::Positional);
            } else if let Some(hex) = slot.strip_prefix('~') {
                if hex.is_empty() || hex.len() % 2 != 0 || hex.bytes().any(|byte| byte.is_ascii_uppercase()) {
                    return Err(referential("callable owner selector contains an invalid encoded label"));
                }
                let mut bytes = allocations.vector::<u8>(hex.len() / 2).map_err(allocation_error)?;
                for pair in hex.as_bytes().chunks_exact(2) {
                    let high = (pair[0] as char)
                        .to_digit(16)
                        .ok_or_else(|| referential("callable owner selector contains an invalid encoded label"))?;
                    let low = (pair[1] as char)
                        .to_digit(16)
                        .ok_or_else(|| referential("callable owner selector contains an invalid encoded label"))?;
                    bytes.push((high * 16 + low) as u8);
                }
                let label = String::from_utf8(bytes).map_err(|_| referential("callable owner selector label is not UTF-8"))?;
                if is_canonical_unescaped_selector_label(&label) {
                    return Err(referential("callable owner selector is not in canonical encoding"));
                }
                slots.push(SelectorSlot::Label(label));
            } else {
                if !is_canonical_unescaped_selector_label(slot) {
                    return Err(referential("callable owner selector contains an unescaped label"));
                }
                slots.push(SelectorSlot::Label(allocations.string(slot).map_err(allocation_error)?));
            }
        }
    }
    Selector::new(base, kind, allocations.boxed_slice(slots).map_err(allocation_error)?).map_err(|_| referential("callable owner selector is invalid"))
}

fn is_canonical_unescaped_selector_label(label: &str) -> bool {
    !label.is_empty()
        && !label.starts_with('~')
        && !matches!(label, "_" | "*" | "**" | "***")
        && label.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'_' | b'?' | b'!' | b'+' | b'-' | b'*' | b'/' | b'<' | b'>' | b'=' | b'&' | b'|' | b'^' | b'~' | b'%'
                )
        })
}

pub(crate) fn import_stable_variant_with_allocation_budget(
    stable: &phalcom_type_meta::identity::StableVariantRef,
    allocations: &mut AllocationBudget,
) -> Result<crate::identity::VariantId, ArtifactError> {
    let owner = import_stable_declaration_with_allocation_budget(&stable.owner, allocations)?;
    let selector = import_stable_selector_with_allocation_budget(&stable.selector, allocations)?;
    Ok(crate::identity::VariantId::new(owner, selector))
}

pub(crate) fn import_stable_impl_with_allocation_budget(
    stable: &phalcom_type_meta::identity::StableImplRef,
    allocations: &mut AllocationBudget,
) -> Result<crate::identity::ImplId, ArtifactError> {
    Ok(crate::identity::ImplId::new(
        import_stable_module_with_allocation_budget(&stable.module, allocations)?,
        crate::identity::ImplLocalId(stable.local_id),
    ))
}

pub(crate) fn import_stable_callable_with_allocation_budget(
    stable: &StableCallableRef,
    allocations: &mut AllocationBudget,
) -> Result<CallableId, ArtifactError> {
    let owner = match &stable.owner {
        StableCallableOwnerRef::Declaration(owner) => CallableOwnerId::Declaration(import_stable_declaration_with_allocation_budget(owner, allocations)?),
        StableCallableOwnerRef::Variant(variant) => CallableOwnerId::Variant(import_stable_variant_with_allocation_budget(variant, allocations)?),
        StableCallableOwnerRef::Conformance(implementation) => {
            CallableOwnerId::Conformance(import_stable_impl_with_allocation_budget(implementation, allocations)?)
        }
    };
    let selector = import_stable_selector_with_allocation_budget(&stable.selector, allocations)?;
    Ok(CallableId {
        owner,
        selector,
        side: from_stable_dispatch_side(stable.side),
    })
}

fn import_stable_field_identity_with_allocation_budget(
    owner: &phalcom_type_meta::identity::StableDeclarationRef,
    side: phalcom_type_meta::identity::StableDispatchSide,
    name: &str,
    allocations: &mut AllocationBudget,
) -> Result<crate::identity::FieldId, ArtifactError> {
    Ok(crate::identity::FieldId::new(
        import_stable_declaration_with_allocation_budget(owner, allocations)?,
        allocations.boxed_str(name).map_err(allocation_error)?,
        from_stable_dispatch_side(side),
    ))
}

fn import_type_term_with_allocation_budget(
    term: &TypeTermV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<TypeTerm, ArtifactError> {
    Ok(match term {
        TypeTermV1::Canonical { ty } => TypeTerm::Canonical(relocation.ty(*ty)?),
        TypeTermV1::SelfType { term } => TypeTerm::SelfType(SelfTypeTerm {
            owner: import_stable_declaration_with_allocation_budget(&term.owner, allocations)?,
            side: from_stable_dispatch_side(term.side),
            role: import_self_role(&term.role),
        }),
    })
}

pub fn import_callable_query_owner(
    owner: &phalcom_compiler_artifact::semantic::world::CallableQueryOwnerV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::db::QueryKey, ArtifactError> {
    use crate::db::QueryKey;
    use crate::trait_dispatch::{TraitDispatchBucketKey, TraitDispatchTargetFamily};
    use phalcom_compiler_artifact::semantic::world::{CallableQueryOwnerV1 as Portable, ConformanceTargetV1};

    let mut child = allocations.child(allocations.remaining());
    let imported = match owner {
        Portable::CallableSignature { callable } => QueryKey::CallableSignature(import_stable_callable_with_allocation_budget(callable, &mut child)?),
        Portable::FieldSignature { field } => QueryKey::FieldSignature(import_stable_field_identity_with_allocation_budget(
            &field.owner,
            field.side,
            &field.name,
            &mut child,
        )?),
        Portable::GenericConformance { subject, trait_ref } => QueryKey::GenericConformance(crate::generic_evidence::GenericConformanceKey::new(
            import_type_term_with_allocation_budget(subject, relocation, &mut child)?,
            import_trait_ref_identity(trait_ref, relocation, &mut child)?,
        )),
        Portable::TraitDispatchBucket { target_family, selector, side } => {
            let target_family = match target_family {
                ConformanceTargetV1::Declaration { declaration } => {
                    TraitDispatchTargetFamily::Declaration(import_stable_declaration_with_allocation_budget(declaration, &mut child)?)
                }
                ConformanceTargetV1::ExactEnumCase { variant } => {
                    TraitDispatchTargetFamily::ExactEnumCase(import_stable_variant_with_allocation_budget(variant, &mut child)?)
                }
            };
            let base = import_selector_base_with_allocation_budget(&selector.base, &mut child)?;
            let mut slots = child
                .vector::<phalcom_common::selector::SelectorSlot>(selector.slots.len())
                .map_err(allocation_error)?;
            for slot in &selector.slots {
                slots.push(import_selector_slot_with_allocation_budget(slot, &mut child)?);
            }
            let selector =
                phalcom_common::selector::Selector::new(base, import_selector_kind(&selector.kind), child.boxed_slice(slots).map_err(allocation_error)?)
                    .map_err(|_| referential("callable query-owner selector is invalid"))?;
            QueryKey::TraitDispatchBucket(TraitDispatchBucketKey {
                target_family,
                selector,
                side: from_stable_dispatch_side(*side),
            })
        }
    };
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(imported)
}

pub fn import_constraint(constraint: &GenericConstraintV1, relocation: &TypeRelocation) -> Result<GenericConstraint, ArtifactError> {
    Ok(match constraint {
        GenericConstraintV1::Subtype { lower, upper } => GenericConstraint::Subtype {
            lower: import_type_term(lower, relocation)?,
            upper: import_type_term(upper, relocation)?,
        },
        GenericConstraintV1::Equivalent { left, right } => GenericConstraint::Equivalent {
            left: import_type_term(left, relocation)?,
            right: import_type_term(right, relocation)?,
        },
        GenericConstraintV1::Conforms { subject, trait_ref } => GenericConstraint::Conforms {
            subject: import_type_term(subject, relocation)?,
            trait_ref: crate::traits::TraitRef {
                declaration: from_stable_declaration(&trait_ref.declaration).map_err(referential)?,
                arguments: trait_ref
                    .arguments
                    .iter()
                    .map(|&arg| relocation.ty(arg))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
            },
        },
    })
}

fn import_constraint_with_allocation_budget(
    constraint: &GenericConstraintV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<GenericConstraint, ArtifactError> {
    Ok(match constraint {
        GenericConstraintV1::Subtype { lower, upper } => GenericConstraint::Subtype {
            lower: import_type_term_with_allocation_budget(lower, relocation, allocations)?,
            upper: import_type_term_with_allocation_budget(upper, relocation, allocations)?,
        },
        GenericConstraintV1::Equivalent { left, right } => GenericConstraint::Equivalent {
            left: import_type_term_with_allocation_budget(left, relocation, allocations)?,
            right: import_type_term_with_allocation_budget(right, relocation, allocations)?,
        },
        GenericConstraintV1::Conforms { subject, trait_ref } => {
            let subject = import_type_term_with_allocation_budget(subject, relocation, allocations)?;
            let declaration = import_stable_declaration_with_allocation_budget(&trait_ref.declaration, allocations)?;
            let mut arguments = allocations.vector(trait_ref.arguments.len()).map_err(allocation_error)?;
            for &argument in &trait_ref.arguments {
                arguments.push(relocation.ty(argument)?);
            }
            GenericConstraint::Conforms {
                subject,
                trait_ref: crate::traits::TraitRef {
                    declaration,
                    arguments: allocations.boxed_slice(arguments).map_err(allocation_error)?,
                },
            }
        }
    })
}

pub fn import_generic_signature(sig: &GenericSignatureV1, relocation: &TypeRelocation, store: &TypeStore) -> Result<GenericSignature, ArtifactError> {
    let owner = import_parameter_owner(&sig.owner)?;
    let parameters = sig
        .parameters
        .iter()
        .map(|&p| relocation.parameter(p))
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    let parameter_kinds = sig
        .parameter_kinds
        .iter()
        .map(|&k| relocation.kind(k))
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    let parameter_kind_shapes = sig.parameter_kind_shapes.to_vec().into_boxed_slice();
    let parameter_variances = sig.parameter_variances.iter().map(import_variance).collect::<Vec<_>>().into_boxed_slice();
    let parameter_modes = sig.parameter_modes.iter().map(import_mode).collect::<Vec<_>>().into_boxed_slice();
    let constraint_shapes = sig.constraint_shapes.to_vec().into_boxed_slice();
    let constraints = sig
        .constraints
        .iter()
        .map(|c| import_constraint(c, relocation))
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();

    let reconstructed = GenericSignature {
        owner,
        parameters,
        parameter_kinds,
        parameter_kind_shapes,
        parameter_variances,
        parameter_modes,
        constraint_shapes,
        constraints,
    };
    reconstructed.validate_publishable(store).map_err(|e| referential(format!("{e:?}")))?;
    Ok(reconstructed)
}

pub(super) fn import_generic_signature_with_allocation_budget(
    sig: &GenericSignatureV1,
    relocation: &TypeRelocation,
    store: &TypeStore,
    allocations: &mut AllocationBudget,
) -> Result<GenericSignature, ArtifactError> {
    let owner = import_parameter_owner_with_allocation_budget(&sig.owner, allocations)?;
    let mut parameters = allocations.vector(sig.parameters.len()).map_err(allocation_error)?;
    for &parameter in &sig.parameters {
        parameters.push(relocation.parameter(parameter)?);
    }
    let mut parameter_kinds = allocations.vector(sig.parameter_kinds.len()).map_err(allocation_error)?;
    for &kind in &sig.parameter_kinds {
        parameter_kinds.push(relocation.kind(kind)?);
    }
    let mut parameter_kind_shapes = allocations.vector(sig.parameter_kind_shapes.len()).map_err(allocation_error)?;
    for shape in &sig.parameter_kind_shapes {
        parameter_kind_shapes.push(allocations.boxed_str(shape).map_err(allocation_error)?);
    }
    let mut parameter_variances = allocations.vector(sig.parameter_variances.len()).map_err(allocation_error)?;
    parameter_variances.extend(sig.parameter_variances.iter().map(import_variance));
    let mut parameter_modes = allocations.vector(sig.parameter_modes.len()).map_err(allocation_error)?;
    parameter_modes.extend(sig.parameter_modes.iter().map(import_mode));
    let mut constraint_shapes = allocations.vector(sig.constraint_shapes.len()).map_err(allocation_error)?;
    for shape in &sig.constraint_shapes {
        constraint_shapes.push(allocations.boxed_str(shape).map_err(allocation_error)?);
    }
    let mut constraints = allocations.vector(sig.constraints.len()).map_err(allocation_error)?;
    for constraint in &sig.constraints {
        constraints.push(import_constraint_with_allocation_budget(constraint, relocation, allocations)?);
    }

    let reconstructed = GenericSignature {
        owner,
        parameters: allocations.boxed_slice(parameters).map_err(allocation_error)?,
        parameter_kinds: allocations.boxed_slice(parameter_kinds).map_err(allocation_error)?,
        parameter_kind_shapes: allocations.boxed_slice(parameter_kind_shapes).map_err(allocation_error)?,
        parameter_variances: allocations.boxed_slice(parameter_variances).map_err(allocation_error)?,
        parameter_modes: allocations.boxed_slice(parameter_modes).map_err(allocation_error)?,
        constraint_shapes: allocations.boxed_slice(constraint_shapes).map_err(allocation_error)?,
        constraints: allocations.boxed_slice(constraints).map_err(allocation_error)?,
    };
    reconstructed
        .validate_publishable(store)
        .map_err(|_| referential("generic signature is not publishable"))?;
    Ok(reconstructed)
}

fn import_parameter_owner_with_allocation_budget(owner: &ParameterOwnerV1, allocations: &mut AllocationBudget) -> Result<TypeParameterOwner, ArtifactError> {
    Ok(match owner {
        ParameterOwnerV1::Declaration { owner } => TypeParameterOwner::Declaration(import_stable_declaration_with_allocation_budget(owner, allocations)?),
        ParameterOwnerV1::Callable { owner } => TypeParameterOwner::Callable(import_stable_callable_with_allocation_budget(owner, allocations)?),
        ParameterOwnerV1::Impl { owner } => TypeParameterOwner::Impl(import_stable_impl_with_allocation_budget(owner, allocations)?),
    })
}

pub fn import_declared_type_state(state: &DeclaredTypeStateV1, relocation: &TypeRelocation) -> Result<DeclaredTypeState, ArtifactError> {
    Ok(match state {
        DeclaredTypeStateV1::Known { term } => DeclaredTypeState::Known(import_type_term(term, relocation)?),
        DeclaredTypeStateV1::Dynamic { reason } => DeclaredTypeState::Dynamic(match reason {
            DynamicReasonV1::ExplicitEscape => DynamicReason::ExplicitEscape,
            DynamicReasonV1::DynamicRestPack => DynamicReason::DynamicRestPack,
            DynamicReasonV1::RuntimeReflection => DynamicReason::RuntimeReflection,
        }),
        DeclaredTypeStateV1::Unknown { reason } => DeclaredTypeState::Unknown(match reason {
            UnknownReasonV1::NoTypeEvidence => UnknownReason::NoTypeEvidence,
            UnknownReasonV1::MissingInitializer => UnknownReason::MissingInitializer,
            UnknownReasonV1::UnannotatedDeclaration => UnknownReason::UnannotatedDeclaration,
            UnknownReasonV1::UnresolvedName { name } => UnknownReason::UnresolvedName(name.clone()),
            UnknownReasonV1::DynamicMessageSend => UnknownReason::DynamicMessageSend,
            UnknownReasonV1::OpaqueNative => UnknownReason::OpaqueNative,
            UnknownReasonV1::RecursiveFixpoint => UnknownReason::RecursiveFixpoint,
            UnknownReasonV1::UncheckedExpression => UnknownReason::UncheckedExpression,
            UnknownReasonV1::SyntaxError => UnknownReason::SyntaxError,
            UnknownReasonV1::UnderconstrainedTypeVariable => UnknownReason::UnderconstrainedTypeVariable,
            UnknownReasonV1::InferenceConflict => UnknownReason::InferenceConflict,
            UnknownReasonV1::InferenceAmbiguous => UnknownReason::InferenceAmbiguous,
            UnknownReasonV1::InferenceBlocked => UnknownReason::InferenceBlocked,
            UnknownReasonV1::InferenceCancelled => UnknownReason::InferenceCancelled,
            UnknownReasonV1::InferenceBudgetExceeded => UnknownReason::InferenceBudgetExceeded,
            UnknownReasonV1::SuppressedByInvalidCause => UnknownReason::SuppressedByInvalidCause,
            UnknownReasonV1::ExistentialEscape => UnknownReason::ExistentialEscape,
        }),
    })
}

pub fn import_declared_type_basis(basis: &DeclaredTypeBasisV1) -> DeclaredTypeBasis {
    match basis {
        DeclaredTypeBasisV1::Unspecified => DeclaredTypeBasis::Unspecified,
        DeclaredTypeBasisV1::SourceAnnotation => DeclaredTypeBasis::SourceAnnotation,
        DeclaredTypeBasisV1::NativeSignature => DeclaredTypeBasis::NativeSignature,
        DeclaredTypeBasisV1::DeclarationSemantics => DeclaredTypeBasis::DeclarationSemantics,
        DeclaredTypeBasisV1::ConstructorSemantics => DeclaredTypeBasis::ConstructorSemantics,
        DeclaredTypeBasisV1::InitializerInference => DeclaredTypeBasis::InitializerInference,
        DeclaredTypeBasisV1::BodyInference => DeclaredTypeBasis::BodyInference,
        DeclaredTypeBasisV1::ContextualTyping => DeclaredTypeBasis::ContextualTyping,
        DeclaredTypeBasisV1::PatternDecomposition => DeclaredTypeBasis::PatternDecomposition,
    }
}

pub fn import_declared_type_fact(fact: &DeclaredTypeFactV1, relocation: &TypeRelocation) -> Result<DeclaredTypeFact, ArtifactError> {
    Ok(DeclaredTypeFact {
        state: import_declared_type_state(&fact.state, relocation)?,
        basis: import_declared_type_basis(&fact.basis),
    })
}

fn import_declared_type_fact_with_allocation_budget(
    fact: &DeclaredTypeFactV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<DeclaredTypeFact, ArtifactError> {
    use phalcom_compiler_artifact::semantic::aggregates::{DeclaredTypeStateV1, DynamicReasonV1, UnknownReasonV1};

    let state = match &fact.state {
        DeclaredTypeStateV1::Known { term } => DeclaredTypeState::Known(import_type_term_with_allocation_budget(term, relocation, allocations)?),
        DeclaredTypeStateV1::Dynamic { reason } => DeclaredTypeState::Dynamic(match reason {
            DynamicReasonV1::ExplicitEscape => DynamicReason::ExplicitEscape,
            DynamicReasonV1::DynamicRestPack => DynamicReason::DynamicRestPack,
            DynamicReasonV1::RuntimeReflection => DynamicReason::RuntimeReflection,
        }),
        DeclaredTypeStateV1::Unknown { reason } => DeclaredTypeState::Unknown(match reason {
            UnknownReasonV1::NoTypeEvidence => UnknownReason::NoTypeEvidence,
            UnknownReasonV1::MissingInitializer => UnknownReason::MissingInitializer,
            UnknownReasonV1::UnannotatedDeclaration => UnknownReason::UnannotatedDeclaration,
            UnknownReasonV1::UnresolvedName { name } => UnknownReason::UnresolvedName(allocations.boxed_str(name).map_err(allocation_error)?),
            UnknownReasonV1::DynamicMessageSend => UnknownReason::DynamicMessageSend,
            UnknownReasonV1::OpaqueNative => UnknownReason::OpaqueNative,
            UnknownReasonV1::RecursiveFixpoint => UnknownReason::RecursiveFixpoint,
            UnknownReasonV1::UncheckedExpression => UnknownReason::UncheckedExpression,
            UnknownReasonV1::SyntaxError => UnknownReason::SyntaxError,
            UnknownReasonV1::UnderconstrainedTypeVariable => UnknownReason::UnderconstrainedTypeVariable,
            UnknownReasonV1::InferenceConflict => UnknownReason::InferenceConflict,
            UnknownReasonV1::InferenceAmbiguous => UnknownReason::InferenceAmbiguous,
            UnknownReasonV1::InferenceBlocked => UnknownReason::InferenceBlocked,
            UnknownReasonV1::InferenceCancelled => UnknownReason::InferenceCancelled,
            UnknownReasonV1::InferenceBudgetExceeded => UnknownReason::InferenceBudgetExceeded,
            UnknownReasonV1::SuppressedByInvalidCause => UnknownReason::SuppressedByInvalidCause,
            UnknownReasonV1::ExistentialEscape => UnknownReason::ExistentialEscape,
        }),
    };
    Ok(DeclaredTypeFact {
        state,
        basis: import_declared_type_basis(&fact.basis),
    })
}

pub fn import_visibility(visibility: &VisibilityV1) -> Visibility {
    match visibility {
        VisibilityV1::Public => Visibility::Public,
        VisibilityV1::Private => Visibility::Private,
        VisibilityV1::Protected => Visibility::Protected,
        VisibilityV1::Module => Visibility::Module,
        VisibilityV1::Package => Visibility::Package,
        VisibilityV1::Internal => Visibility::Internal,
    }
}

pub fn import_data_shape(shape: &DataShapeV1) -> DataShape {
    match shape {
        DataShapeV1::Tuple => DataShape::Tuple,
        DataShapeV1::Record => DataShape::Record,
    }
}

pub fn import_variant_shape(shape: &VariantShapeV1) -> VariantShape {
    match shape {
        VariantShapeV1::Singleton => VariantShape::Singleton,
        VariantShapeV1::Constructor => VariantShape::Constructor,
    }
}

pub fn import_payload_kind(kind: &PayloadKindV1) -> AlgebraicCasePayloadKind {
    match kind {
        PayloadKindV1::Singleton => AlgebraicCasePayloadKind::Singleton,
        PayloadKindV1::Tuple => AlgebraicCasePayloadKind::Tuple,
        PayloadKindV1::Record => AlgebraicCasePayloadKind::Record,
        PayloadKindV1::ExplicitEmpty => AlgebraicCasePayloadKind::ExplicitEmpty,
    }
}

pub fn import_expansion_mode(mode: &ExpansionModeV1) -> phalcom_ast::ast::ExpansionMode {
    match mode {
        ExpansionModeV1::Positional => phalcom_ast::ast::ExpansionMode::Positional,
        ExpansionModeV1::Labeled => phalcom_ast::ast::ExpansionMode::Labeled,
        ExpansionModeV1::Complete => phalcom_ast::ast::ExpansionMode::Complete,
    }
}

pub fn import_static_data(
    wire: &StaticAlgebraicDeclarationV1,
    relocation: &TypeRelocation,
) -> Result<crate::algebraic_semantics::StaticAlgebraicDeclarationInfo, ArtifactError> {
    Ok(crate::algebraic_semantics::StaticAlgebraicDeclarationInfo {
        domain: relocation.domain(wire.domain)?,
        constructors: wire
            .constructors
            .iter()
            .map(|&c| relocation.constructor(c))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
    })
}

fn import_static_data_with_allocation_budget(
    wire: &StaticAlgebraicDeclarationV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::algebraic_semantics::StaticAlgebraicDeclarationInfo, ArtifactError> {
    let mut constructors = allocations.vector(wire.constructors.len()).map_err(allocation_error)?;
    for &constructor in &wire.constructors {
        constructors.push(relocation.constructor(constructor)?);
    }
    Ok(crate::algebraic_semantics::StaticAlgebraicDeclarationInfo {
        domain: relocation.domain(wire.domain)?,
        constructors: allocations.boxed_slice(constructors).map_err(allocation_error)?,
    })
}

pub fn import_case_environment(env: &CaseEnvironmentV1, relocation: &TypeRelocation) -> Result<CaseTypeEnvironment, ArtifactError> {
    let mut bindings = BTreeMap::new();
    for b in &env.bindings {
        let param = relocation.parameter(b.parameter)?;
        let ty = relocation.ty(b.ty)?;
        bindings.insert(param, ty);
    }
    let equalities = env
        .equalities
        .iter()
        .map(|eq| import_constraint(eq, relocation))
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();

    Ok(CaseTypeEnvironment {
        bindings: bindings.into(),
        equalities,
    })
}

fn import_case_environment_with_allocation_budget(
    env: &CaseEnvironmentV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<CaseTypeEnvironment, ArtifactError> {
    let mut bindings = allocations.vector(env.bindings.len()).map_err(allocation_error)?;
    for binding in &env.bindings {
        bindings.push((relocation.parameter(binding.parameter)?, relocation.ty(binding.ty)?));
    }
    let bindings = crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(bindings, allocations).map_err(|error| match error {
        crate::ordered_map::FrozenOrderedMapError::Allocation(error) => allocation_error(error),
        crate::ordered_map::FrozenOrderedMapError::DuplicateKey => referential("case environment binding parameter is duplicated"),
    })?;
    let mut equalities = allocations.vector(env.equalities.len()).map_err(allocation_error)?;
    for equality in &env.equalities {
        equalities.push(import_constraint_with_allocation_budget(equality, relocation, allocations)?);
    }
    Ok(CaseTypeEnvironment {
        bindings,
        equalities: allocations.boxed_slice(equalities).map_err(allocation_error)?,
    })
}

fn clone_data_component_with_allocation_budget(
    component: &DataComponentSemantic,
    allocations: &mut AllocationBudget,
) -> Result<DataComponentSemantic, ArtifactError> {
    Ok(DataComponentSemantic {
        id: component.id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
        local_name: allocations.boxed_str(&component.local_name).map_err(allocation_error)?,
        external_label: component
            .external_label
            .as_deref()
            .map(|label| allocations.boxed_str(label).map_err(allocation_error))
            .transpose()?,
        declared_type: component.declared_type.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
        source: None,
    })
}

fn clone_data_expansion_with_allocation_budget(
    expansion: &DataProductExpansionSemantic,
    allocations: &mut AllocationBudget,
) -> Result<DataProductExpansionSemantic, ArtifactError> {
    Ok(DataProductExpansionSemantic {
        mode: expansion.mode,
        operand: expansion.operand.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
        source: None,
    })
}

fn clone_variant_family_with_allocation_budget(
    family: &crate::identity::VariantFamilyId,
    allocations: &mut AllocationBudget,
) -> Result<crate::identity::VariantFamilyId, ArtifactError> {
    Ok(crate::identity::VariantFamilyId::new(
        family.owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
        allocations.boxed_str(&family.base_name).map_err(allocation_error)?,
    ))
}

fn debug_boxed_str_with_allocation_budget<T: std::fmt::Debug>(value: &T, allocations: &mut AllocationBudget) -> Result<Box<str>, ArtifactError> {
    use std::fmt::Write as _;

    struct Length(usize);
    impl std::fmt::Write for Length {
        fn write_str(&mut self, value: &str) -> std::fmt::Result {
            self.0 = self.0.checked_add(value.len()).ok_or(std::fmt::Error)?;
            Ok(())
        }
    }
    struct Buffer<'a>(&'a mut Vec<u8>);
    impl std::fmt::Write for Buffer<'_> {
        fn write_str(&mut self, value: &str) -> std::fmt::Result {
            if self.0.capacity().saturating_sub(self.0.len()) < value.len() {
                return Err(std::fmt::Error);
            }
            self.0.extend_from_slice(value.as_bytes());
            Ok(())
        }
    }

    let mut length = Length(0);
    std::write!(&mut length, "{value:?}").map_err(|_| referential("debug-formatted type alias identity is too large"))?;
    let mut bytes = allocations.vector(length.0).map_err(allocation_error)?;
    {
        let mut buffer = Buffer(&mut bytes);
        std::write!(&mut buffer, "{value:?}").map_err(|_| allocation_error(AllocationError::Failed))?;
    }
    if bytes.len() != length.0 {
        return Err(referential("debug-formatted type alias identity changed during materialization"));
    }
    let bytes = Vec::from(allocations.boxed_slice(bytes).map_err(allocation_error)?);
    let formatted = String::from_utf8(bytes).map_err(|_| referential("debug formatter produced invalid UTF-8"))?;
    Ok(formatted.into_boxed_str())
}

fn generic_signature_for_declaration<'a>(
    signatures: &'a crate::ordered_map::OrderedMap<TypeParameterOwner, GenericSignature>,
    declaration: &DeclarationId,
) -> Option<&'a GenericSignature> {
    signatures.iter().find_map(|(owner, signature)| match owner {
        TypeParameterOwner::Declaration(owner) if owner == declaration => Some(signature),
        _ => None,
    })
}

/// Loaded authority populated from structural primary contracts.
#[derive(Clone, Debug)]
pub struct LoadedStructuralContracts {
    pub declaration_table: Arc<DeclarationTypeTable>,
    pub hierarchy: Arc<MapTypeHierarchy>,
    pub type_aliases: Arc<TypeAliasTable>,
    pub data_table: Arc<DataSemanticTable>,
    pub enum_table: Arc<EnumSemanticTable>,
    pub generic_signatures: crate::ordered_map::OrderedMap<TypeParameterOwner, GenericSignature>,
}

/// Reconstructs all structural primary contracts into a fresh set of semantic tables.
pub fn import_structural_primary_contracts(
    contracts: &StructuralPrimaryContractsV2,
    store: &mut TypeStore,
    type_relocation: &TypeRelocation,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<LoadedStructuralContracts, ArtifactError> {
    import_structural_primary_contracts_with_owner_catalog(contracts, store, type_relocation, limits, allocations, None)
}

fn import_structural_primary_contracts_with_owner_catalog(
    contracts: &StructuralPrimaryContractsV2,
    store: &mut TypeStore,
    type_relocation: &TypeRelocation,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
    owner_catalog: Option<&StableOwnerCatalog<'_>>,
) -> Result<LoadedStructuralContracts, ArtifactError> {
    #[cfg(feature = "allocation-diagnostics")]
    let mut diagnostic_previous_observed = phalcom_type_meta::allocation::current_observed_bytes().unwrap_or(0);
    #[cfg(feature = "allocation-diagnostics")]
    let mut diagnostic_previous_ledger = allocations.requested();
    contracts.validate_with_allocation_budget(limits, allocations)?;
    #[cfg(feature = "allocation-diagnostics")]
    record_callable_import_allocation_diagnostic_stage(48, allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
    type_relocation.verify_store(store)?;

    let mut generic_signature_entries = allocations.vector(contracts.generic_signatures.len()).map_err(allocation_error)?;
    for sig_wire in &contracts.generic_signatures {
        let sig = import_generic_signature_with_allocation_budget(sig_wire, type_relocation, store, allocations)?;
        let key = sig.owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        generic_signature_entries.push((key, sig));
    }
    let generic_signatures =
        crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(generic_signature_entries, allocations).map_err(|error| match error {
            crate::ordered_map::FrozenOrderedMapError::Allocation(error) => allocation_error(error),
            crate::ordered_map::FrozenOrderedMapError::DuplicateKey => referential("generic signature owner is duplicated after relocation"),
        })?;
    #[cfg(feature = "allocation-diagnostics")]
    record_callable_import_allocation_diagnostic_stage(49, allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

    let mut data_entries = allocations.vector(contracts.data_contracts.len()).map_err(allocation_error)?;
    for data_wire in &contracts.data_contracts {
        let owner = import_stable_declaration_with_allocation_budget(&data_wire.owner, allocations)?;
        let root_form = type_relocation.ty(data_wire.root_form)?;
        let generic_signature = data_wire
            .generic_signature
            .as_ref()
            .map(|s| import_generic_signature_with_allocation_budget(s, type_relocation, store, allocations))
            .transpose()?;
        let static_data = data_wire
            .static_data
            .as_ref()
            .map(|s| import_static_data_with_allocation_budget(s, type_relocation, allocations))
            .transpose()?;
        let shape = import_data_shape(&data_wire.shape);

        let mut components = allocations.vector(data_wire.components.len()).map_err(allocation_error)?;
        for (idx, c) in data_wire.components.iter().enumerate() {
            let id = DataComponentId::new(owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?, idx as u32);
            let declared_type = import_declared_type_fact_with_allocation_budget(&c.declared_type, type_relocation, allocations)?;
            components.push(DataComponentSemantic {
                id,
                local_name: allocations.boxed_str(&c.local_name).map_err(allocation_error)?,
                external_label: c
                    .external_label
                    .as_deref()
                    .map(|label| allocations.boxed_str(label).map_err(allocation_error))
                    .transpose()?,
                declared_type,
                source: None,
            });
        }

        let constructor_id = DataConstructorId::new(owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?);
        let constructor_visibility = import_visibility(&data_wire.constructor_visibility);
        let mut constructor_parameters = allocations.vector(data_wire.constructor_parameters.len()).map_err(allocation_error)?;
        for (idx, p) in data_wire.constructor_parameters.iter().enumerate() {
            let component = DataComponentId::new(owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?, idx as u32);
            let declared_type = import_declared_type_fact_with_allocation_budget(&p.declared_type, type_relocation, allocations)?;
            constructor_parameters.push(DataConstructorParameter {
                component,
                external_label: p
                    .external_label
                    .as_deref()
                    .map(|label| allocations.boxed_str(label).map_err(allocation_error))
                    .transpose()?,
                local_name: allocations.boxed_str(&p.local_name).map_err(allocation_error)?,
                declared_type,
            });
        }
        let constructor_result_type = type_relocation.ty(data_wire.constructor_result_type)?;
        let constructor = DataConstructorSignature {
            constructor: constructor_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            visibility: constructor_visibility,
            parameters: allocations.boxed_slice(constructor_parameters).map_err(allocation_error)?,
            result_type_template: constructor_result_type,
            source: None,
        };

        let has_expansions = data_wire
            .shape_contributions
            .iter()
            .any(|contribution| matches!(contribution, DataProductContributionV1::Expansion { .. }));
        let mut shape_contributions = allocations.vector(data_wire.shape_contributions.len()).map_err(allocation_error)?;
        let mut constructor_param_contributions = if has_expansions {
            Some(allocations.vector(data_wire.shape_contributions.len()).map_err(allocation_error)?)
        } else {
            None
        };
        let mut component_iter = components.iter();

        for contrib in &data_wire.shape_contributions {
            match contrib {
                DataProductContributionV1::Component { .. } => {
                    let comp = component_iter.next().ok_or_else(|| referential("shape contribution component index error"))?;
                    shape_contributions.push(DataProductContribution::Component(clone_data_component_with_allocation_budget(
                        comp,
                        allocations,
                    )?));
                    if let Some(parameters) = &mut constructor_param_contributions {
                        parameters.push(DataConstructorParameterContribution::Parameter(DataConstructorParameter {
                            component: comp.id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                            external_label: comp
                                .external_label
                                .as_deref()
                                .map(|label| allocations.boxed_str(label).map_err(allocation_error))
                                .transpose()?,
                            local_name: allocations.boxed_str(&comp.local_name).map_err(allocation_error)?,
                            declared_type: comp.declared_type.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                        }));
                    }
                }
                DataProductContributionV1::Expansion { mode, operand } => {
                    let operand = import_declared_type_fact_with_allocation_budget(operand, type_relocation, allocations)?;
                    let exp = DataProductExpansionSemantic {
                        mode: import_expansion_mode(mode),
                        operand,
                        source: None,
                    };
                    shape_contributions.push(DataProductContribution::Expansion(clone_data_expansion_with_allocation_budget(
                        &exp,
                        allocations,
                    )?));
                    if let Some(parameters) = &mut constructor_param_contributions {
                        parameters.push(DataConstructorParameterContribution::Expansion(exp));
                    }
                }
            }
        }

        let shape_template = DataProductShapeTemplate {
            shape,
            contributions: allocations.boxed_slice(shape_contributions).map_err(allocation_error)?,
        };

        let constructor_template = if shape_template.has_expansions() {
            Some(DataConstructorTemplate {
                constructor: constructor_id,
                visibility: constructor_visibility,
                parameters: allocations
                    .boxed_slice(constructor_param_contributions.expect("expansion template parameters were allocated"))
                    .map_err(allocation_error)?,
                result_type_template: constructor_result_type,
                source: None,
            })
        } else {
            None
        };

        let data_info = DataInfo {
            owner,
            root_form,
            generic_signature,
            static_data,
            shape,
            components: allocations.boxed_slice(components).map_err(allocation_error)?,
            constructor,
            shape_template,
            constructor_template,
            source: None,
        };
        let key = data_info.owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        data_entries.push((key, allocations.arc(data_info).map_err(allocation_error)?));
    }

    let data_table = DataSemanticTable::from_frozen_entries_with_allocation_budget(data_entries, allocations).map_err(|error| match error {
        crate::parent_map::FrozenParentMapError::Allocation(error) => allocation_error(error),
        crate::parent_map::FrozenParentMapError::DuplicateKey => referential("data declaration owner is duplicated after relocation"),
    })?;
    #[cfg(feature = "allocation-diagnostics")]
    record_callable_import_allocation_diagnostic_stage(50, allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

    let mut variant_entries = allocations.vector(contracts.variant_contracts.len()).map_err(allocation_error)?;

    for variant_wire in &contracts.variant_contracts {
        let variant_id = import_stable_variant_with_allocation_budget(&variant_wire.variant, allocations)?;
        let type_handle = type_relocation.variant(variant_wire.variant_type)?;
        if store.variant_identity(type_handle) != &variant_id {
            return Err(referential("relocated variant type handle does not match variant identity in TypeStore"));
        }

        let family = match &variant_id.selector.base {
            phalcom_common::selector::SelectorBase::Named(name) => Some(crate::identity::VariantFamilyId::new(
                variant_id.owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                allocations.boxed_str(name).map_err(allocation_error)?,
            )),
            phalcom_common::selector::SelectorBase::Subscript => None,
        };
        let shape = import_variant_shape(&variant_wire.shape);
        let payload_kind = import_payload_kind(&variant_wire.payload_kind);

        let mut fields = allocations.vector(variant_wire.fields.len()).map_err(allocation_error)?;
        for (idx, f) in variant_wire.fields.iter().enumerate() {
            let id = VariantFieldId::new(variant_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?, idx as u32);
            let declared_type = import_declared_type_fact_with_allocation_budget(&f.declared_type, type_relocation, allocations)?;
            fields.push(VariantFieldSemantic {
                id,
                local_name: allocations.boxed_str(&f.local_name).map_err(allocation_error)?,
                external_label: f
                    .external_label
                    .as_deref()
                    .map(|label| allocations.boxed_str(label).map_err(allocation_error))
                    .transpose()?,
                declared_type,
                source: None,
            });
        }

        let result_type_template = type_relocation.ty(variant_wire.result_type)?;
        let exact_case_template = type_relocation.ty(variant_wire.exact_case_type)?;
        let case_environment = import_case_environment_with_allocation_budget(&variant_wire.case_environment, type_relocation, allocations)?;

        let constructor = if let Some(cw) = &variant_wire.constructor {
            let generic_signature = cw
                .generic_signature
                .as_ref()
                .map(|s| import_generic_signature_with_allocation_budget(s, type_relocation, store, allocations))
                .transpose()?;
            let mut parameters = allocations.vector(cw.parameters.len()).map_err(allocation_error)?;
            for (idx, p) in cw.parameters.iter().enumerate() {
                let field = VariantFieldId::new(variant_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?, idx as u32);
                let declared_type = import_declared_type_fact_with_allocation_budget(&p.declared_type, type_relocation, allocations)?;
                parameters.push(VariantConstructorParameter {
                    field,
                    external_label: p
                        .external_label
                        .as_deref()
                        .map(|label| allocations.boxed_str(label).map_err(allocation_error))
                        .transpose()?,
                    local_name: allocations.boxed_str(&p.local_name).map_err(allocation_error)?,
                    declared_type,
                });
            }
            Some(VariantConstructorSignature {
                constructor: VariantConstructorId::new(variant_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?),
                generic_signature,
                parameters: allocations.boxed_slice(parameters).map_err(allocation_error)?,
                result_type_template: type_relocation.ty(cw.result_type)?,
                exact_case_template: type_relocation.ty(cw.exact_case_type)?,
                source: None,
            })
        } else {
            None
        };

        let visibility = VariantVisibility {
            name: import_visibility(&variant_wire.visibility.name),
            construct: import_visibility(&variant_wire.visibility.construct),
            payload: import_visibility(&variant_wire.visibility.payload),
        };

        let variant_info = VariantInfo {
            id: variant_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            type_handle,
            family,
            shape,
            payload_kind,
            fields: allocations.boxed_slice(fields).map_err(allocation_error)?,
            result_type_template,
            exact_case_template,
            case_environment,
            constructor,
            visibility,
            source: None,
        };
        let key = variant_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        variant_entries.push((key, allocations.arc(variant_info).map_err(allocation_error)?));
    }
    variant_entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));

    let mut enum_entries = allocations.vector(contracts.enum_contracts.len()).map_err(allocation_error)?;
    for enum_wire in &contracts.enum_contracts {
        let owner = import_stable_declaration_with_allocation_budget(&enum_wire.owner, allocations)?;
        let root_form = type_relocation.ty(enum_wire.root_form)?;
        let generic_signature = enum_wire
            .generic_signature
            .as_ref()
            .map(|s| import_generic_signature_with_allocation_budget(s, type_relocation, store, allocations))
            .transpose()?;
        let static_data = enum_wire
            .static_data
            .as_ref()
            .map(|s| import_static_data_with_allocation_budget(s, type_relocation, allocations))
            .transpose()?;
        let default_result_type = type_relocation.ty(enum_wire.default_result_type)?;

        let mut variant_ids = allocations.vector(enum_wire.variants.len()).map_err(allocation_error)?;
        let mut variant_families = allocations.vector(enum_wire.variants.len()).map_err(allocation_error)?;

        for v_stable in &enum_wire.variants {
            let v_id = import_stable_variant_with_allocation_budget(v_stable, allocations)?;
            let variant_position = variant_entries.binary_search_by(|(key, _)| key.cmp(&v_id));
            let Ok(variant_position) = variant_position else {
                return Err(referential("enum references a missing structural variant"));
            };
            if let Some(family) = variant_entries[variant_position].1.family.as_ref()
                && !variant_families.iter().any(|existing| existing == family)
            {
                variant_families.push(clone_variant_family_with_allocation_budget(family, allocations)?);
            }
            variant_ids.push(v_id);
        }

        let enum_info = EnumInfo {
            owner,
            root_form,
            generic_signature,
            static_data,
            default_result_type,
            variants: allocations.boxed_slice(variant_ids).map_err(allocation_error)?,
            variant_families: allocations.boxed_slice(variant_families).map_err(allocation_error)?,
            source: None,
        };
        let key = enum_info.owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        enum_entries.push((key, allocations.arc(enum_info).map_err(allocation_error)?));
    }

    let enum_table =
        EnumSemanticTable::from_frozen_entries_with_allocation_budget(enum_entries, variant_entries, allocations).map_err(|error| match error {
            crate::parent_map::FrozenParentMapError::Allocation(error) => allocation_error(error),
            crate::parent_map::FrozenParentMapError::DuplicateKey => referential("enum or variant owner is duplicated after relocation"),
        })?;
    #[cfg(feature = "allocation-diagnostics")]
    record_callable_import_allocation_diagnostic_stage(51, allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

    let mut alias_entries = allocations.vector(contracts.type_aliases.len()).map_err(allocation_error)?;
    for alias in &contracts.type_aliases {
        let decl = import_stable_declaration_with_allocation_budget(&alias.declaration, allocations)?;
        let form = type_relocation.ty(alias.form)?;
        let kind = type_relocation.kind(alias.kind)?;
        let generic_signature = alias
            .generic_signature
            .as_ref()
            .map(|signature| import_generic_signature_with_allocation_budget(signature, type_relocation, store, allocations))
            .transpose()?;
        let mut dependencies = allocations.vector(alias.dependencies.len()).map_err(allocation_error)?;
        for dependency in &alias.dependencies {
            dependencies.push(import_stable_declaration_with_allocation_budget(dependency, allocations)?);
        }
        let start = usize::try_from(alias.source_span.start).map_err(|_| referential("type alias source start exceeds address space"))?;
        let end = usize::try_from(alias.source_span.end).map_err(|_| referential("type alias source end exceeds address space"))?;
        if start > end {
            return Err(referential("type alias source range is inverted"));
        }
        let alias_info = TypeAliasInfo {
            declaration: decl.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            kind,
            kind_shape: allocations.boxed_str(&alias.kind_shape).map_err(allocation_error)?,
            generic_signature,
            form,
            structural_form: allocations.boxed_str(&alias.structural_form).map_err(allocation_error)?,
            dependencies: allocations.boxed_slice(dependencies).map_err(allocation_error)?,
            source: SemanticSourceSpan::new(
                decl.module.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                SourceRange { start, end },
            ),
        };
        let key = decl.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        alias_entries.push((key, alias_info));
    }
    let type_aliases = TypeAliasTable::from_frozen_entries_with_allocation_budget(alias_entries, allocations).map_err(|error| match error {
        crate::type_alias::FrozenTypeAliasTableError::Allocation(error) => allocation_error(error),
        crate::type_alias::FrozenTypeAliasTableError::DuplicateDeclaration => referential("type alias owner is duplicated after relocation"),
    })?;
    #[cfg(feature = "allocation-diagnostics")]
    record_callable_import_allocation_diagnostic_stage(52, allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
    let mut superclass_entries = allocations.vector(contracts.hierarchy_edges.len()).map_err(allocation_error)?;
    let mut template_entries = allocations.vector(contracts.hierarchy_edges.len()).map_err(allocation_error)?;
    for edge in &contracts.hierarchy_edges {
        let declaration = import_stable_declaration_with_allocation_budget(&edge.declaration, allocations)?;
        if let Some(superclass) = &edge.superclass {
            superclass_entries.push((
                declaration.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                import_stable_declaration_with_allocation_budget(superclass, allocations)?,
            ));
        }
        if let Some(template) = &edge.supertype_template {
            let owner = import_stable_declaration_with_allocation_budget(&template.declaration, allocations)?;
            if owner != declaration {
                return Err(referential("supertype template owner differs from its hierarchy edge"));
            }
            template_entries.push((
                owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                crate::declarations::GenericSupertypeTemplate {
                    declaration: owner,
                    supertype: type_relocation.ty(template.supertype)?,
                    structural_form: template
                        .structural_form
                        .as_deref()
                        .map(|form| allocations.boxed_str(form).map_err(allocation_error))
                        .transpose()?,
                },
            ));
        }
    }
    let hierarchy =
        MapTypeHierarchy::from_frozen_entries_with_allocation_budget(superclass_entries, template_entries, contracts.hierarchy_edges.len(), allocations)
            .map_err(|error| match error {
                crate::types::relation::FrozenTypeHierarchyError::Allocation(error) => allocation_error(error),
                crate::types::relation::FrozenTypeHierarchyError::DuplicateSuperclass => {
                    referential("hierarchy declaration is duplicated after identity relocation")
                }
                crate::types::relation::FrozenTypeHierarchyError::DuplicateTemplate => {
                    referential("supertype template owner is duplicated after identity relocation")
                }
            })?;
    #[cfg(feature = "allocation-diagnostics")]
    record_callable_import_allocation_diagnostic_stage(53, allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

    let mut declaration_entries = allocations
        .vector::<(crate::identity::DeclarationId, DeclarationTypeInfo)>(contracts.declaration_shells.len())
        .map_err(allocation_error)?;
    for shell in &contracts.declaration_shells {
        let decl = import_stable_declaration_with_allocation_budget(&shell.declaration, allocations)?;
        let form = type_relocation.ty(shell.self_type)?;
        let class_object_type = type_relocation.ty(shell.class_object_type)?;
        let canonical_class_object_type = store
            .class_object_type_id_with_allocation_budget(&decl, allocations)
            .map_err(allocation_error)?
            .ok_or_else(|| referential("declaration shell has no canonical class-object type"))?;
        if class_object_type != canonical_class_object_type {
            return Err(referential("declaration shell class-object type disagrees with canonical TypeStore identity"));
        }
        let kind = type_relocation.kind(shell.kind)?;
        if kind != store.kind_of(form) {
            return Err(referential("declaration shell kind disagrees with its relocated form"));
        }
        let generic_sig = shell
            .generic_signature
            .as_ref()
            .map(|signature| import_generic_signature_with_allocation_budget(signature, type_relocation, store, allocations))
            .transpose()?;
        let indexed_generic_sig = generic_signature_for_declaration(&generic_signatures, &decl);
        if generic_sig.as_ref() != indexed_generic_sig {
            return Err(referential("declaration shell generic signature disagrees with the canonical signature index"));
        }
        let generic_binder_header = generic_sig
            .as_ref()
            .map(|signature| GenericBinderHeader::from_signature_with_allocation_budget(store, signature, allocations).map_err(allocation_error))
            .transpose()?;
        let supertype_template = shell
            .supertype_template
            .as_ref()
            .map(|template| {
                let owner = import_stable_declaration_with_allocation_budget(&template.declaration, allocations)?;
                if owner != decl {
                    return Err(referential("declaration shell supertype template owner disagrees with its declaration"));
                }
                Ok(crate::declarations::GenericSupertypeTemplate {
                    declaration: owner,
                    supertype: type_relocation.ty(template.supertype)?,
                    structural_form: template
                        .structural_form
                        .as_deref()
                        .map(|form| allocations.boxed_str(form).map_err(allocation_error))
                        .transpose()?,
                })
            })
            .transpose()?;
        if supertype_template.as_ref() != hierarchy.supertype_template(&decl) {
            return Err(referential("declaration shell supertype template disagrees with hierarchy authority"));
        }
        let semantic_form = match shell.binding.semantic_form {
            DeclarationSemanticFormV1::Ordinary => crate::declarations::DeclarationSemanticForm::Ordinary,
            DeclarationSemanticFormV1::IntrinsicTuple => {
                crate::declarations::DeclarationSemanticForm::Intrinsic(crate::declarations::IntrinsicTypeApplication::Tuple)
            }
            DeclarationSemanticFormV1::IntrinsicRecord => {
                crate::declarations::DeclarationSemanticForm::Intrinsic(crate::declarations::IntrinsicTypeApplication::Record)
            }
            DeclarationSemanticFormV1::IntrinsicPack => {
                crate::declarations::DeclarationSemanticForm::Intrinsic(crate::declarations::IntrinsicTypeApplication::Pack)
            }
        };
        let runtime_owner = shell.binding.runtime_owner.as_ref().map(|owner| match owner {
            IntrinsicRuntimeOwnerV1::Tuple => crate::declarations::IntrinsicRuntimeOwner::Tuple,
            IntrinsicRuntimeOwnerV1::Record => crate::declarations::IntrinsicRuntimeOwner::Record,
            IntrinsicRuntimeOwnerV1::Pack => crate::declarations::IntrinsicRuntimeOwner::Pack,
        });
        let binding = DeclarationTypeBinding { semantic_form, runtime_owner };
        let key = decl.clone_with_allocation_budget(allocations).map_err(allocation_error)?;

        let info = DeclarationTypeInfo {
            declaration: decl,
            form,
            class_object_type,
            kind,
            binding,
            generic_binder_header,
            generic_signature: generic_sig,
            supertype_template,
        };
        declaration_entries.push((key, info));
    }
    let mut declaration_table =
        DeclarationTypeTable::from_frozen_entries_with_allocation_budget(declaration_entries, allocations).map_err(|error| match error {
            crate::declarations::FrozenDeclarationTypeTableError::Allocation(error) => allocation_error(error),
            crate::declarations::FrozenDeclarationTypeTableError::DuplicateDeclaration => {
                referential("declaration shell owner is duplicated after stable-identity relocation")
            }
            _ => referential("declaration shell owner is duplicated after stable-identity relocation"),
        })?;
    let mut intrinsic_static_domains = allocations
        .vector::<(crate::identity::DeclarationId, crate::types::id::KindId)>(contracts.intrinsic_static_domains.len())
        .map_err(allocation_error)?;
    let mut intrinsic_type_owners = allocations
        .vector::<(crate::identity::DeclarationId, crate::types::type_member::IntrinsicTypeOwner)>(contracts.intrinsic_type_owners.len())
        .map_err(allocation_error)?;
    if !contracts.intrinsic_static_domains.is_empty() || !contracts.intrinsic_type_owners.is_empty() {
        let owners = owner_catalog.ok_or_else(|| referential("module world and project universe are required for intrinsic projection owners"))?;
        for record in &contracts.intrinsic_static_domains {
            let declaration = owners.declaration_with_allocation_budget(&record.declaration, allocations)?;
            intrinsic_static_domains.push((declaration, type_relocation.kind(record.kind)?));
        }
        for record in &contracts.intrinsic_type_owners {
            let declaration = owners.declaration_with_allocation_budget(&record.declaration, allocations)?;
            let owner = match record.owner {
                IntrinsicTypeOwnerV1::Tuple => crate::types::type_member::IntrinsicTypeOwner::Tuple,
                IntrinsicTypeOwnerV1::Record => crate::types::type_member::IntrinsicTypeOwner::Record,
                IntrinsicTypeOwnerV1::TypePack => crate::types::type_member::IntrinsicTypeOwner::TypePack,
                IntrinsicTypeOwnerV1::LabeledTypePack => crate::types::type_member::IntrinsicTypeOwner::LabeledTypePack,
                IntrinsicTypeOwnerV1::CompleteTypePack => crate::types::type_member::IntrinsicTypeOwner::CompleteTypePack,
                IntrinsicTypeOwnerV1::RecordRow => crate::types::type_member::IntrinsicTypeOwner::RecordRow,
                IntrinsicTypeOwnerV1::UnorderedShape => crate::types::type_member::IntrinsicTypeOwner::UnorderedShape,
            };
            intrinsic_type_owners.push((declaration, owner));
        }
    }
    declaration_table
        .install_frozen_intrinsic_authority_with_allocation_budget(intrinsic_static_domains, intrinsic_type_owners, allocations)
        .map_err(|error| match error {
            crate::declarations::FrozenDeclarationTypeTableError::Allocation(error) => allocation_error(error),
            crate::declarations::FrozenDeclarationTypeTableError::DuplicateIntrinsicStaticDomain => {
                referential("intrinsic static-domain owner is duplicated after stable-identity relocation")
            }
            crate::declarations::FrozenDeclarationTypeTableError::DuplicateIntrinsicTypeOwner => {
                referential("intrinsic type-owner declaration is duplicated after stable-identity relocation")
            }
            crate::declarations::FrozenDeclarationTypeTableError::DuplicateDeclaration => {
                referential("declaration shell owner is duplicated after stable-identity relocation")
            }
        })?;
    #[cfg(feature = "allocation-diagnostics")]
    record_callable_import_allocation_diagnostic_stage(54, allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

    let loaded = LoadedStructuralContracts {
        declaration_table: allocations.arc(declaration_table).map_err(allocation_error)?,
        hierarchy: allocations.arc(hierarchy).map_err(allocation_error)?,
        type_aliases: allocations.arc(type_aliases).map_err(allocation_error)?,
        data_table: allocations.arc(data_table).map_err(allocation_error)?,
        enum_table: allocations.arc(enum_table).map_err(allocation_error)?,
        generic_signatures,
    };
    #[cfg(feature = "allocation-diagnostics")]
    record_callable_import_allocation_diagnostic_stage(55, allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
    Ok(loaded)
}

use crate::artifact::validate::{compute_witness_layout_fingerprint, validate_pending_static_references};
use crate::static_value::{StaticRelocation, StaticValueStore, import_static_value_graph_with_allocation_budget};
use phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1;
use phalcom_compiler_artifact::semantic::world::{
    AssociatedFamilyKindV1, AssociatedMemberV1, AssociatedStaticDomainRecordV1, AssociatedSurfaceRecordV1, ConformanceTargetV1, IntrinsicTypeOperationV1,
    SemanticWorldV4, TypeMemberDeclarationRecordV1, TypeMemberDomainV1, TypeMemberImplementationV1,
};
use phalcom_type_meta::identity::StableDeclarationRef;

pub(crate) fn import_type_member_selector(selector: &TypeMemberSelectorV1) -> crate::types::type_member::TypeMemberSelector {
    match selector {
        TypeMemberSelectorV1::Named { name } => crate::types::type_member::TypeMemberSelector::Named(name.clone().into_boxed_str()),
        TypeMemberSelectorV1::Subscript => crate::types::type_member::TypeMemberSelector::Subscript,
    }
}

fn import_associated_selector(selector: &TypeMemberSelectorV1) -> phalcom_common::selector::SelectorBase {
    match selector {
        TypeMemberSelectorV1::Named { name } => phalcom_common::selector::SelectorBase::Named(name.clone()),
        TypeMemberSelectorV1::Subscript => phalcom_common::selector::SelectorBase::Subscript,
    }
}

fn import_associated_selector_with_allocation_budget(
    selector: &TypeMemberSelectorV1,
    allocations: &mut AllocationBudget,
) -> Result<phalcom_common::selector::SelectorBase, ArtifactError> {
    import_selector_base_with_allocation_budget(selector, allocations)
}

fn ensure_complete_callable_analysis_payload(analyses: &[phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2]) -> Result<(), ArtifactError> {
    for analysis in analyses {
        let has_explanation = |id: u32| (id as usize) < analysis.explanations.len();
        if analysis.return_explanation.is_some_and(|id| !has_explanation(id))
            || analysis
                .expressions
                .iter()
                .filter_map(|expression| expression.explanation)
                .any(|id| !has_explanation(id))
            || analysis
                .bindings
                .iter()
                .filter_map(|binding| binding.explanation)
                .any(|id| !has_explanation(id))
            || analysis.diagnostics.iter().any(|diagnostic| {
                diagnostic.root_cause.is_some_and(|id| id >= analysis.diagnostic_cause_count)
                    || diagnostic
                        .explanations
                        .iter()
                        .any(|reference| reference.callable != analysis.callable || !has_explanation(reference.ordinal))
            })
            || analysis
                .normal_returns
                .iter()
                .filter_map(|exit| exit.direct_result_dependency.as_ref())
                .any(|dependency| !analysis.dependencies.contains(dependency))
        {
            return Err(referential("callable owner payload has a dangling explanation, cause, or dependency"));
        }
    }
    Ok(())
}

/// Reconstructs a callable flow graph into immutable, explicitly budgeted
/// storage. Edge endpoint ordinals determine adjacency, and each node's edge
/// lists follow ascending edge ordinal as in source graphs.
pub fn import_callable_flow_graph_with_allocation_budget(
    flow: &phalcom_compiler_artifact::semantic::world::CallableFlowGraphV1,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<Arc<crate::checker::flow::graph::FlowGraph>, ArtifactError> {
    use crate::checker::flow::graph::{FlowEdge, FlowEdgeKind, FlowGraph, FlowNode, FlowNodeKind};
    use phalcom_compiler_artifact::semantic::world::{CallableFlowEdgeKindV1, CallableFlowNodeKindV1};
    use phalcom_type_meta::allocation::BudgetedSortedMap;

    let node_count = flow.nodes.len();
    let edge_count = flow.edges.len();
    if node_count > limits.max_semantic_entries_per_family || edge_count > limits.max_semantic_entries_per_family {
        return Err(referential("callable flow graph exceeds its owner-local entry limit"));
    }

    // Work in a child ledger so a failed graph import publishes no partial
    // allocation charge to the enclosing semantic-image transaction.
    let mut child = allocations.child(allocations.remaining());
    let budget = &mut child;
    let mut degrees = budget.vector::<(usize, usize)>(node_count).map_err(allocation_error)?;
    degrees.resize(node_count, (0, 0));

    for edge in &flow.edges {
        let source = usize::try_from(edge.source).map_err(|_| referential("flow edge source ordinal exceeds address space"))?;
        let target = usize::try_from(edge.target).map_err(|_| referential("flow edge target ordinal exceeds address space"))?;
        if source >= node_count || target >= node_count {
            return Err(referential("callable flow edge references an absent node ordinal"));
        }
        degrees[source].1 = degrees[source].1.checked_add(1).ok_or_else(|| referential("flow out-degree overflow"))?;
        degrees[target].0 = degrees[target].0.checked_add(1).ok_or_else(|| referential("flow in-degree overflow"))?;
    }

    let entry = flow
        .entry
        .map(|ordinal| {
            let index = usize::try_from(ordinal).map_err(|_| referential("flow entry ordinal exceeds address space"))?;
            if index >= node_count {
                return Err(referential("callable flow entry references an absent node ordinal"));
            }
            if !matches!(&flow.nodes[index].kind, CallableFlowNodeKindV1::Entry) {
                return Err(referential("callable flow entry ordinal does not identify an entry node"));
            }
            Ok(crate::identity::FlowNodeId(ordinal))
        })
        .transpose()?;

    if !flow.exits.is_empty() {
        let mut seen_exits = budget.vector::<u8>(node_count).map_err(allocation_error)?;
        seen_exits.resize(node_count, 0);
        for exit in &flow.exits {
            let index = usize::try_from(*exit).map_err(|_| referential("flow exit ordinal exceeds address space"))?;
            if index >= node_count {
                return Err(referential("callable flow exit references an absent node ordinal"));
            }
            if !matches!(&flow.nodes[index].kind, CallableFlowNodeKindV1::Exit | CallableFlowNodeKindV1::Throw) {
                return Err(referential("callable flow exit ordinal does not identify an exit or throw node"));
            }
            if seen_exits[index] != 0 {
                return Err(referential("callable flow graph contains a duplicate exit"));
            }
            seen_exits[index] = 1;
        }
    }

    let mut nodes = budget.vector::<(crate::identity::FlowNodeId, FlowNode)>(node_count).map_err(allocation_error)?;
    for (index, record) in flow.nodes.iter().enumerate() {
        let ordinal = u32::try_from(index).map_err(|_| referential("flow node count exceeds portable ordinal range"))?;
        if record.range.start > record.range.end {
            return Err(referential("flow node source range is inverted"));
        }
        let range = SourceRange {
            start: usize::try_from(record.range.start).map_err(|_| referential("flow node range start exceeds address space"))?,
            end: usize::try_from(record.range.end).map_err(|_| referential("flow node range end exceeds address space"))?,
        };
        let kind = match &record.kind {
            CallableFlowNodeKindV1::Entry => FlowNodeKind::Entry,
            CallableFlowNodeKindV1::Exit => FlowNodeKind::Exit,
            CallableFlowNodeKindV1::Statement { index } => {
                FlowNodeKind::Statement(usize::try_from(*index).map_err(|_| referential("flow statement index exceeds address space"))?)
            }
            CallableFlowNodeKindV1::BranchCondition => FlowNodeKind::BranchCondition,
            CallableFlowNodeKindV1::LoopHeader => FlowNodeKind::LoopHeader,
            CallableFlowNodeKindV1::Join => FlowNodeKind::Join,
            CallableFlowNodeKindV1::Throw => FlowNodeKind::Throw,
            CallableFlowNodeKindV1::Unreachable => FlowNodeKind::Unreachable,
        };
        let predecessors = budget.vector::<crate::identity::FlowEdgeId>(degrees[index].0).map_err(allocation_error)?;
        let successors = budget.vector::<crate::identity::FlowEdgeId>(degrees[index].1).map_err(allocation_error)?;
        nodes.push((
            crate::identity::FlowNodeId(ordinal),
            FlowNode {
                id: crate::identity::FlowNodeId(ordinal),
                kind,
                range,
                predecessors,
                successors,
            },
        ));
    }

    let mut edges = budget.vector::<(crate::identity::FlowEdgeId, FlowEdge)>(edge_count).map_err(allocation_error)?;
    for (index, record) in flow.edges.iter().enumerate() {
        let ordinal = u32::try_from(index).map_err(|_| referential("flow edge count exceeds portable ordinal range"))?;
        let source = crate::identity::FlowNodeId(record.source);
        let target = crate::identity::FlowNodeId(record.target);
        let id = crate::identity::FlowEdgeId(ordinal);
        let source_index = usize::try_from(record.source).map_err(|_| referential("flow edge source ordinal exceeds address space"))?;
        let target_index = usize::try_from(record.target).map_err(|_| referential("flow edge target ordinal exceeds address space"))?;
        nodes[source_index].1.successors.push(id);
        nodes[target_index].1.predecessors.push(id);
        let kind = match &record.kind {
            CallableFlowEdgeKindV1::Normal => FlowEdgeKind::Normal,
            CallableFlowEdgeKindV1::TrueBranch => FlowEdgeKind::TrueBranch,
            CallableFlowEdgeKindV1::FalseBranch => FlowEdgeKind::FalseBranch,
            CallableFlowEdgeKindV1::BackEdge => FlowEdgeKind::BackEdge,
            CallableFlowEdgeKindV1::Break => FlowEdgeKind::Break,
            CallableFlowEdgeKindV1::Continue => FlowEdgeKind::Continue,
            CallableFlowEdgeKindV1::Return => FlowEdgeKind::Return,
            CallableFlowEdgeKindV1::Throw => FlowEdgeKind::Throw,
            CallableFlowEdgeKindV1::Unreachable => FlowEdgeKind::Unreachable,
        };
        edges.push((
            id,
            FlowEdge {
                id,
                source,
                target,
                kind,
                predicate: None,
            },
        ));
    }

    let mut exits = budget.vector::<crate::identity::FlowNodeId>(flow.exits.len()).map_err(allocation_error)?;
    for exit in &flow.exits {
        exits.push(crate::identity::FlowNodeId(*exit));
    }
    let nodes = BudgetedSortedMap::from_sorted_entries(nodes).map_err(|_| referential("reconstructed flow nodes are not strictly ordered"))?;
    let edges = BudgetedSortedMap::from_sorted_entries(edges).map_err(|_| referential("reconstructed flow edges are not strictly ordered"))?;
    let graph = FlowGraph::from_budgeted(budget, nodes, edges, entry, exits).map_err(allocation_error)?;
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(graph)
}

#[cfg(test)]
fn import_exact_callable(callable: &StableCallableRef) -> Result<CallableId, ArtifactError> {
    let selector =
        phalcom_common::selector::Selector::try_decode_exact(&callable.selector).map_err(|_| referential("artifact callable selector is not canonical"))?;
    if selector.encode() != callable.selector.as_ref() {
        return Err(referential("artifact callable selector is not in canonical encoding"));
    }
    let owner = match &callable.owner {
        StableCallableOwnerRef::Declaration(declaration) => CallableOwnerId::Declaration(from_stable_declaration(declaration).map_err(referential)?),
        StableCallableOwnerRef::Variant(variant) => CallableOwnerId::Variant(from_stable_variant(variant).map_err(referential)?),
        StableCallableOwnerRef::Conformance(implementation) => CallableOwnerId::Conformance(from_stable_impl(implementation).map_err(referential)?),
    };
    Ok(CallableId::new(
        owner,
        selector,
        crate::metadata::stable_identity::from_stable_dispatch_side(callable.side),
    ))
}

fn import_exact_callable_with_allocation_budget(callable: &StableCallableRef, allocations: &mut AllocationBudget) -> Result<CallableId, ArtifactError> {
    // The budgeted stable-identity parser enforces the exact canonical
    // selector grammar while charging every owned identity component.
    import_stable_callable_with_allocation_budget(callable, allocations)
}

#[derive(Clone)]
enum CallableEvidenceImportState {
    Unvisited,
    Visiting,
    Complete(crate::generic_evidence::GenericConformanceEvidence),
}

fn clone_generic_conformance_evidence_with_allocation_budget(
    evidence: &crate::generic_evidence::GenericConformanceEvidence,
    allocations: &mut AllocationBudget,
) -> Result<crate::generic_evidence::GenericConformanceEvidence, ArtifactError> {
    use crate::generic_evidence::{AssumedConformanceEvidence, GenericConformanceEvidence};

    Ok(match evidence {
        GenericConformanceEvidence::Exact(evidence) => GenericConformanceEvidence::Exact(Arc::clone(evidence)),
        GenericConformanceEvidence::Assumed(evidence) => GenericConformanceEvidence::Assumed(AssumedConformanceEvidence {
            key: crate::generic_evidence::GenericConformanceKey {
                subject: crate::types::parameter::clone_type_term_with_allocation_budget(&evidence.key.subject, allocations).map_err(allocation_error)?,
                trait_ref: evidence.key.trait_ref.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            },
            owner: evidence.owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
        }),
        GenericConformanceEvidence::Derived(evidence) => GenericConformanceEvidence::Derived(Arc::clone(evidence)),
        GenericConformanceEvidence::CompilerDerived(evidence) => GenericConformanceEvidence::CompilerDerived(Arc::clone(evidence)),
    })
}

struct CallableEvidenceImporter<'a> {
    relocation: &'a TypeRelocation,
    store: &'a TypeStore,
    source_plan_fingerprints: &'a [(crate::identity::ImplId, crate::db::ProductFingerprint)],
    allocations: &'a mut AllocationBudget,
    states: Vec<CallableEvidenceImportState>,
    max_depth: usize,
}

impl<'a> CallableEvidenceImporter<'a> {
    fn new(
        evidence_count: usize,
        relocation: &'a TypeRelocation,
        store: &'a TypeStore,
        source_plan_fingerprints: &'a [(crate::identity::ImplId, crate::db::ProductFingerprint)],
        allocations: &'a mut AllocationBudget,
        max_depth: usize,
    ) -> Result<Self, ArtifactError> {
        let mut states = allocations.vector(evidence_count).map_err(allocation_error)?;
        states.resize(evidence_count, CallableEvidenceImportState::Unvisited);
        Ok(Self {
            relocation,
            store,
            source_plan_fingerprints,
            allocations,
            states,
            max_depth,
        })
    }

    fn evidence(
        &mut self,
        evidence_nodes: &[phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1],
        ordinal: u32,
        depth: usize,
    ) -> Result<crate::generic_evidence::GenericConformanceEvidence, ArtifactError> {
        use phalcom_compiler_artifact::semantic::world as wire;

        if depth > self.max_depth {
            return Err(referential("callable conformance evidence nesting limit exceeded during import"));
        }
        let index = usize::try_from(ordinal).map_err(|_| referential("callable evidence ordinal exceeds address space"))?;
        let node = evidence_nodes
            .get(index)
            .ok_or_else(|| referential("callable conformance evidence reference is out of range"))?;
        match self.states.get(index) {
            Some(CallableEvidenceImportState::Complete(evidence)) => {
                return clone_generic_conformance_evidence_with_allocation_budget(evidence, self.allocations);
            }
            Some(CallableEvidenceImportState::Visiting) => return Err(referential("callable conformance evidence graph contains a cycle")),
            Some(CallableEvidenceImportState::Unvisited) => {}
            None => return Err(referential("callable conformance evidence reference is out of range")),
        }
        self.states[index] = CallableEvidenceImportState::Visiting;

        let imported = match node {
            wire::CallableConformanceEvidenceNodeV1::Assumed { key, owner } => Ok(crate::generic_evidence::GenericConformanceEvidence::Assumed(
                crate::generic_evidence::AssumedConformanceEvidence {
                    key: import_generic_conformance_key_with_allocation_budget(key, self.relocation, self.allocations)?,
                    owner: import_parameter_owner_with_allocation_budget(owner, self.allocations)?,
                },
            )),
            wire::CallableConformanceEvidenceNodeV1::CompilerDerived { evidence } => self.import_compiler_derived(evidence),
            wire::CallableConformanceEvidenceNodeV1::Exact { evidence } => self.import_exact(evidence_nodes, evidence, depth + 1),
            wire::CallableConformanceEvidenceNodeV1::Derived { evidence } => self.import_derived(evidence_nodes, evidence, depth + 1),
        };

        match imported {
            Ok(evidence) => {
                let cached = match clone_generic_conformance_evidence_with_allocation_budget(&evidence, self.allocations) {
                    Ok(cached) => cached,
                    Err(error) => {
                        self.states[index] = CallableEvidenceImportState::Unvisited;
                        return Err(error);
                    }
                };
                self.states[index] = CallableEvidenceImportState::Complete(cached);
                Ok(evidence)
            }
            Err(error) => {
                self.states[index] = CallableEvidenceImportState::Unvisited;
                Err(error)
            }
        }
    }

    fn import_compiler_derived(
        &mut self,
        evidence: &phalcom_compiler_artifact::semantic::world::CallableCompilerDerivedConformanceEvidenceV1,
    ) -> Result<crate::generic_evidence::GenericConformanceEvidence, ArtifactError> {
        use phalcom_compiler_artifact::semantic::world as wire;

        let key = import_generic_conformance_key_with_allocation_budget(&evidence.key, self.relocation, self.allocations)?;
        let kind = match evidence.kind {
            wire::CallableCompilerDerivedConformanceKindV1::Product => crate::generic_evidence::CompilerDerivedConformanceKind::Product,
            wire::CallableCompilerDerivedConformanceKindV1::OrderedProduct => crate::generic_evidence::CompilerDerivedConformanceKind::OrderedProduct,
            wire::CallableCompilerDerivedConformanceKindV1::UnorderedProduct => crate::generic_evidence::CompilerDerivedConformanceKind::UnorderedProduct,
            wire::CallableCompilerDerivedConformanceKindV1::Sum => crate::generic_evidence::CompilerDerivedConformanceKind::Sum,
        };
        let classification = match &evidence.classification {
            wire::CallableStructuralClassificationV1::Unsupported => crate::structural::StructuralClassification::Unsupported,
            wire::CallableStructuralClassificationV1::OrderedProduct { form } => crate::structural::StructuralClassification::OrderedProduct(match form {
                wire::CallableOrderedProductFormV1::Unit => crate::structural::OrderedProductForm::Unit,
                wire::CallableOrderedProductFormV1::Tuple => crate::structural::OrderedProductForm::Tuple,
                wire::CallableOrderedProductFormV1::TupleFromPacks => crate::structural::OrderedProductForm::TupleFromPacks,
                wire::CallableOrderedProductFormV1::Pack => crate::structural::OrderedProductForm::Pack,
                wire::CallableOrderedProductFormV1::PackFromPacks => crate::structural::OrderedProductForm::PackFromPacks,
                wire::CallableOrderedProductFormV1::Data { owner } => crate::structural::OrderedProductForm::Data {
                    owner: import_stable_declaration_with_allocation_budget(owner, self.allocations)?,
                },
            }),
            wire::CallableStructuralClassificationV1::UnorderedProduct { form } => crate::structural::StructuralClassification::UnorderedProduct(match form {
                wire::CallableUnorderedProductFormV1::Record => crate::structural::UnorderedProductForm::Record,
                wire::CallableUnorderedProductFormV1::Data { owner } => crate::structural::UnorderedProductForm::Data {
                    owner: import_stable_declaration_with_allocation_budget(owner, self.allocations)?,
                },
            }),
            wire::CallableStructuralClassificationV1::Sum { owner } => crate::structural::StructuralClassification::Sum {
                owner: import_stable_declaration_with_allocation_budget(owner, self.allocations)?,
            },
        };
        let evidence = self
            .allocations
            .arc(crate::generic_evidence::CompilerDerivedConformanceEvidence { key, kind, classification })
            .map_err(allocation_error)?;
        Ok(crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(evidence))
    }

    fn import_exact(
        &mut self,
        evidence_nodes: &[phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1],
        evidence: &phalcom_compiler_artifact::semantic::world::CallableExactConformanceEvidenceV1,
        depth: usize,
    ) -> Result<crate::generic_evidence::GenericConformanceEvidence, ArtifactError> {
        use phalcom_compiler_artifact::semantic::world::CallableAssociatedTypeBindingValueV1 as Value;

        let mut diagnostic_previous_observed = phalcom_type_meta::allocation::current_observed_bytes().unwrap_or(0);
        let mut diagnostic_previous_ledger = self.allocations.requested();
        let source_impl = import_stable_impl_with_allocation_budget(&evidence.source_impl, self.allocations)?;
        let source_plan_fingerprint = self
            .source_plan_fingerprints
            .iter()
            .find(|(impl_id, _)| impl_id == &source_impl)
            .map(|(_, fingerprint)| *fingerprint)
            .ok_or_else(|| referential("callable Exact conformance evidence has no reconstructed source witness plan"))?;
        let exact_target = self.relocation.ty(evidence.exact_target)?;
        let exact_trait_ref = import_trait_ref_identity(&evidence.exact_trait_ref, self.relocation, self.allocations)?;
        let impl_environment = import_callable_type_environment_with_budget(&evidence.impl_environment, self.relocation, self.allocations)?;
        record_callable_import_allocation_diagnostic_stage(29, self.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

        let mut requirement_views = self.allocations.vector(evidence.requirement_views.len()).map_err(allocation_error)?;
        for view in &evidence.requirement_views {
            let requirement = import_callable_requirement_identity_with_allocation_budget(&view.requirement, self.allocations)?;
            if requirement.owner != exact_trait_ref.declaration {
                return Err(referential("Exact conformance requirement view belongs to another trait"));
            }
            requirement_views.push((
                requirement.clone_with_allocation_budget(self.allocations).map_err(allocation_error)?,
                crate::traits::InstantiatedTraitRequirement {
                    requirement,
                    callable: import_exact_callable_with_allocation_budget(&view.callable, self.allocations)?,
                    signature: import_callable_signature(&view.signature, self.relocation, self.store, self.allocations)?,
                    visibility: import_visibility(&view.visibility),
                    default_callable: view
                        .default_callable
                        .as_ref()
                        .map(|callable| import_exact_callable_with_allocation_budget(callable, self.allocations))
                        .transpose()?,
                },
            ));
        }
        let requirement_views = import_analysis_map(requirement_views, self.allocations, "Exact conformance requirement views are not canonical")?;
        record_callable_import_allocation_diagnostic_stage(30, self.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

        let mut requirements = self.allocations.vector(evidence.requirements.len()).map_err(allocation_error)?;
        for entry in &evidence.requirements {
            let requirement = import_callable_requirement_identity_with_allocation_budget(&entry.requirement, self.allocations)?;
            if requirement.owner != exact_trait_ref.declaration {
                return Err(referential("Exact conformance requirement selection belongs to another trait"));
            }
            let selection = import_callable_requirement_selection(&entry.selection, evidence_nodes, self, depth)?;
            requirements.push((requirement, selection));
        }
        let requirements = import_analysis_map(requirements, self.allocations, "Exact conformance requirement selections are not canonical")?;
        record_callable_import_allocation_diagnostic_stage(31, self.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

        let mut associated_types = self.allocations.vector(evidence.associated_types.len()).map_err(allocation_error)?;
        for binding in &evidence.associated_types {
            let requirement = import_callable_associated_type_requirement_identity_with_allocation_budget(&binding.requirement, self.allocations)?;
            if requirement.owner != exact_trait_ref.declaration {
                return Err(referential("Exact conformance associated-type binding belongs to another trait"));
            }
            associated_types.push((
                requirement.clone_with_allocation_budget(self.allocations).map_err(allocation_error)?,
                crate::impls::ExactAssociatedTypeBinding {
                    requirement,
                    value: match binding.value {
                        Value::Type { ty } => crate::impls::AssociatedTypeBindingValue::Type(self.relocation.ty(ty)?),
                        Value::Dynamic => crate::impls::AssociatedTypeBindingValue::Dynamic,
                    },
                    source_impl: import_stable_impl_with_allocation_budget(&binding.source_impl, self.allocations)?,
                    source: import_source_span_with_allocation_budget(&binding.source, self.allocations)?,
                },
            ));
        }
        let associated_types = import_analysis_map(associated_types, self.allocations, "Exact conformance associated types are not canonical")?;
        record_callable_import_allocation_diagnostic_stage(32, self.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

        let mut trait_header_contract = self.allocations.vector(evidence.trait_header_contract.len()).map_err(allocation_error)?;
        for constraint in &evidence.trait_header_contract {
            trait_header_contract.push(import_constraint_with_allocation_budget(constraint, self.relocation, self.allocations)?);
        }
        let trait_header_contract = self.allocations.boxed_slice(trait_header_contract).map_err(allocation_error)?;
        record_callable_import_allocation_diagnostic_stage(33, self.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let trait_header_conformance_proofs = evidence
            .trait_header_conformance_proofs
            .as_ref()
            .map(|proofs| {
                let mut imported = self.allocations.vector(proofs.len()).map_err(allocation_error)?;
                for proof in proofs {
                    imported.push((
                        import_generic_conformance_key_with_allocation_budget(&proof.key, self.relocation, self.allocations)?,
                        self.evidence(evidence_nodes, proof.evidence, depth + 1)?,
                    ));
                }
                import_analysis_map(imported, self.allocations, "Exact conformance proof keys are not canonical")
            })
            .transpose()?;
        record_callable_import_allocation_diagnostic_stage(34, self.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

        let fingerprint = crate::impls::conformance_evidence_fingerprint(
            source_plan_fingerprint,
            exact_target,
            &exact_trait_ref,
            &impl_environment,
            &requirement_views,
            &requirements,
            &associated_types,
            &trait_header_contract,
        );
        record_callable_import_allocation_diagnostic_stage(35, self.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let evidence = crate::impls::ConformanceEvidence::from_frozen_parts(
            source_impl,
            exact_target,
            exact_trait_ref,
            impl_environment,
            requirement_views,
            requirements,
            associated_types,
            trait_header_contract,
            trait_header_conformance_proofs,
            fingerprint,
        );
        validate_relocated_conformance_evidence_fingerprint(&evidence, source_plan_fingerprint)?;
        let evidence = self.allocations.arc(evidence).map_err(allocation_error)?;
        record_callable_import_allocation_diagnostic_stage(36, self.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        Ok(crate::generic_evidence::GenericConformanceEvidence::Exact(evidence))
    }

    fn import_derived(
        &mut self,
        evidence_nodes: &[phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1],
        evidence: &phalcom_compiler_artifact::semantic::world::CallableDerivedConformanceEvidenceV1,
        depth: usize,
    ) -> Result<crate::generic_evidence::GenericConformanceEvidence, ArtifactError> {
        use phalcom_compiler_artifact::semantic::world::{CallableDerivedConformanceOriginV1 as Origin, CallableNestedGenericEvidenceV1 as Nested};

        let origin = match &evidence.origin {
            Origin::ConditionalSource { exact_source } => {
                let imported = self.evidence(evidence_nodes, *exact_source, depth + 1)?;
                let crate::generic_evidence::GenericConformanceEvidence::Exact(exact_source) = imported else {
                    return Err(referential("callable Derived conditional source references a node of the wrong kind"));
                };
                crate::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source }
            }
            Origin::GuardedRule {
                rule,
                exact_subject,
                exact_trait_ref,
            } => {
                let rule = import_callable_derived_conformance_rule(rule, evidence_nodes, self, depth + 1)?;
                crate::generic_evidence::DerivedConformanceOrigin::GuardedRule {
                    rule: self.allocations.arc(rule).map_err(allocation_error)?,
                    exact_subject: self.relocation.ty(*exact_subject)?,
                    exact_trait_ref: import_trait_ref_identity(exact_trait_ref, self.relocation, self.allocations)?,
                }
            }
        };

        let mut impl_bindings = self.allocations.vector(evidence.impl_bindings.len()).map_err(allocation_error)?;
        for binding in &evidence.impl_bindings {
            impl_bindings.push((self.relocation.parameter(binding.parameter)?, self.relocation.ty(binding.ty)?));
        }
        impl_bindings.sort_unstable_by_key(|(parameter, _)| *parameter);
        let impl_bindings =
            crate::types::substitution::TypeSubstitution::from_budgeted_entries(impl_bindings, self.allocations).map_err(|error| match error {
                crate::checker::analysis_map::AnalysisMapBuildError::Entries(_) => referential("callable Derived conformance bindings are not canonical"),
                crate::checker::analysis_map::AnalysisMapBuildError::Allocation(error) => allocation_error(error),
            })?;

        let mut nested = self.allocations.vector(evidence.nested.len()).map_err(allocation_error)?;
        for item in &evidence.nested {
            nested.push(match item {
                Nested::Conformance { evidence } => {
                    crate::generic_evidence::NestedGenericEvidence::Conformance(self.evidence(evidence_nodes, *evidence, depth + 1)?)
                }
                Nested::Equivalent => crate::generic_evidence::NestedGenericEvidence::Equivalent,
                Nested::Subtype => crate::generic_evidence::NestedGenericEvidence::Subtype,
            });
        }
        let evidence = crate::generic_evidence::DerivedConformanceEvidence {
            origin,
            impl_bindings,
            nested: self.allocations.boxed_slice(nested).map_err(allocation_error)?,
        };
        Ok(crate::generic_evidence::GenericConformanceEvidence::Derived(
            self.allocations.arc(evidence).map_err(allocation_error)?,
        ))
    }

    fn binding(
        &mut self,
        binding: &phalcom_compiler_artifact::semantic::world::CallableGenericCallConformanceBindingV1,
        evidence_nodes: &[phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1],
    ) -> Result<crate::checker::analysis::GenericCallConformanceBinding, ArtifactError> {
        use phalcom_compiler_artifact::semantic::world::CallableGenericCallConformanceSourceV1 as Source;

        let callee_conformance = import_generic_conformance_key_with_allocation_budget(&binding.callee_conformance, self.relocation, self.allocations)?;
        let source = match &binding.source {
            Source::Exact { evidence } => {
                let imported = self.evidence(evidence_nodes, *evidence, 0)?;
                let crate::generic_evidence::GenericConformanceEvidence::Exact(evidence) = imported else {
                    return Err(referential("callable Exact conformance binding references a node of the wrong kind"));
                };
                crate::checker::analysis::GenericCallConformanceSource::Exact { evidence }
            }
            Source::Forwarded { caller_conformance } => crate::checker::analysis::GenericCallConformanceSource::Forwarded {
                caller_conformance: import_generic_conformance_key_with_allocation_budget(caller_conformance, self.relocation, self.allocations)?,
            },
            Source::Derived { evidence } => {
                let imported = self.evidence(evidence_nodes, *evidence, 0)?;
                let crate::generic_evidence::GenericConformanceEvidence::Derived(evidence) = imported else {
                    return Err(referential("callable Derived conformance binding references a node of the wrong kind"));
                };
                crate::checker::analysis::GenericCallConformanceSource::Derived { evidence }
            }
            Source::CompilerDerived { evidence } => {
                let imported = self.evidence(evidence_nodes, *evidence, 0)?;
                let crate::generic_evidence::GenericConformanceEvidence::CompilerDerived(evidence) = imported else {
                    return Err(referential("callable CompilerDerived conformance binding references a node of the wrong kind"));
                };
                crate::checker::analysis::GenericCallConformanceSource::CompilerDerived { evidence }
            }
        };
        Ok(crate::checker::analysis::GenericCallConformanceBinding { callee_conformance, source })
    }
}

fn import_analysis_map<K: Ord, V>(
    entries: Vec<(K, V)>,
    allocations: &mut AllocationBudget,
    malformed_message: &'static str,
) -> Result<crate::checker::analysis_map::AnalysisMap<K, V>, ArtifactError> {
    crate::checker::analysis_map::AnalysisMap::from_budgeted_entries(entries, allocations).map_err(|error| match error {
        crate::checker::analysis_map::AnalysisMapBuildError::Entries(_) => referential(malformed_message),
        crate::checker::analysis_map::AnalysisMapBuildError::Allocation(error) => allocation_error(error),
    })
}

fn validate_relocated_conformance_evidence_fingerprint(
    evidence: &crate::impls::ConformanceEvidence,
    source_plan_fingerprint: crate::db::ProductFingerprint,
) -> Result<(), ArtifactError> {
    let reconstructed = crate::impls::conformance_evidence_fingerprint(
        source_plan_fingerprint,
        evidence.exact_target,
        &evidence.exact_trait_ref,
        &evidence.impl_environment,
        &evidence.requirement_views,
        &evidence.requirements,
        &evidence.associated_types,
        &evidence.trait_header_contract,
    );
    if evidence.fingerprint != reconstructed {
        return Err(referential("reconstructed Exact conformance evidence fingerprint is stale after relocation"));
    }
    Ok(())
}

/// Relocates selected-call specialization rows and reconstructs every portable
/// conformance variant through one callable-local evidence memo.
pub(crate) fn import_callable_call_specializations(
    expressions: &[phalcom_compiler_artifact::semantic::world::CallableExpressionRowV1],
    evidence_nodes: &[phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1],
    relocation: &TypeRelocation,
    store: &TypeStore,
    source_plan_fingerprints: &[(crate::identity::ImplId, crate::db::ProductFingerprint)],
    allocations: &mut AllocationBudget,
    max_depth: usize,
) -> Result<Vec<(u32, crate::checker::analysis::CallSpecialization)>, ArtifactError> {
    use phalcom_compiler_artifact::semantic::type_store::StaticArgumentV1;
    let mut child = allocations.child(allocations.remaining());
    let mut importer = CallableEvidenceImporter::new(evidence_nodes.len(), relocation, store, source_plan_fingerprints, &mut child, max_depth)?;
    let mut output = importer
        .allocations
        .vector::<(u32, crate::checker::analysis::CallSpecialization)>(expressions.len())
        .map_err(allocation_error)?;
    for (index, expression) in expressions.iter().enumerate() {
        if expression.ordinal as usize != index {
            return Err(referential("callable expression rows are not dense and ordinal-sorted"));
        }
        let Some(record) = &expression.call_specialization else {
            continue;
        };
        let callable = import_stable_callable_with_allocation_budget(&record.callable, importer.allocations)?;
        let mut bindings = importer
            .allocations
            .vector::<(crate::types::id::TypeParameterId, crate::types::store::StaticArgument)>(record.bindings.len())
            .map_err(allocation_error)?;
        for binding in &record.bindings {
            let parameter = relocation.parameter(binding.parameter)?;
            if bindings.iter().any(|(existing, _)| *existing == parameter) {
                return Err(referential("call specialization contains duplicate relocated parameter bindings"));
            }
            let argument = match &binding.argument {
                StaticArgumentV1::Term { term } => crate::types::store::StaticArgument::Term(relocation.ty(*term)?),
                StaticArgumentV1::RecordRow { row } => crate::types::store::StaticArgument::RecordRow(relocation.row(*row)?),
                StaticArgumentV1::DynamicType => crate::types::store::StaticArgument::DynamicType,
            };
            bindings.push((parameter, argument));
        }
        let mut conformances = importer
            .allocations
            .vector::<crate::checker::analysis::GenericCallConformanceBinding>(record.conformances.len())
            .map_err(allocation_error)?;
        for binding in &record.conformances {
            conformances.push(importer.binding(binding, evidence_nodes)?);
        }
        output.push((
            expression.ordinal,
            crate::checker::analysis::CallSpecialization {
                callable,
                bindings: importer.allocations.boxed_slice(bindings).map_err(allocation_error)?,
                conformances: importer.allocations.boxed_slice(conformances).map_err(allocation_error)?,
            },
        ));
    }
    drop(importer);
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(output)
}

fn import_generic_conformance_key_with_allocation_budget(
    key: &phalcom_compiler_artifact::semantic::world::GenericConformanceKeyV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::generic_evidence::GenericConformanceKey, ArtifactError> {
    let subject = import_type_term_with_allocation_budget(&key.subject, relocation, allocations)?;
    let trait_ref = import_trait_ref_identity(&key.trait_ref, relocation, allocations)?;
    Ok(crate::generic_evidence::GenericConformanceKey::new(subject, trait_ref))
}

/// Reconstructs expression statuses, retaining blocked query-owner identities
/// under the same callable-local ordinal used by the row.
pub(crate) fn import_callable_status(
    status: &phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::checker::analysis::AnalysisStatus, ArtifactError> {
    use crate::checker::analysis::AnalysisStatus;
    use crate::checker::causal::SuppressionCause;
    use phalcom_compiler_artifact::semantic::aggregates::DynamicReasonV1;
    use phalcom_compiler_artifact::semantic::world::{CallableExpressionStatusV1 as Portable, CallableSuppressionCauseV1};

    Ok(match status {
        Portable::Ready => AnalysisStatus::Ready,
        Portable::Invalid { cause } => AnalysisStatus::Invalid(DiagnosticCauseId(*cause)),
        Portable::Suppressed { cause } => AnalysisStatus::Suppressed(match cause {
            CallableSuppressionCauseV1::One { cause } => SuppressionCause::One(DiagnosticCauseId(*cause)),
            CallableSuppressionCauseV1::Multiple => SuppressionCause::Multiple,
        }),
        Portable::Blocked { reason } => AnalysisStatus::Blocked(import_callable_block_reason(reason, relocation, allocations)?),
        Portable::DynamicBoundary { reason } => AnalysisStatus::DynamicBoundary(match reason {
            DynamicReasonV1::ExplicitEscape => DynamicReason::ExplicitEscape,
            DynamicReasonV1::DynamicRestPack => DynamicReason::DynamicRestPack,
            DynamicReasonV1::RuntimeReflection => DynamicReason::RuntimeReflection,
        }),
        Portable::Cancelled => AnalysisStatus::Cancelled,
        Portable::BudgetExceeded { report } => AnalysisStatus::BudgetExceeded(import_callable_binding_budget(report)),
    })
}

pub(crate) fn import_callable_expression_statuses(
    expressions: &[phalcom_compiler_artifact::semantic::world::CallableExpressionRowV1],
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<Vec<(u32, crate::checker::analysis::AnalysisStatus)>, ArtifactError> {
    use crate::checker::analysis::AnalysisStatus;

    let mut child = allocations.child(allocations.remaining());
    let mut output = child.vector::<(u32, AnalysisStatus)>(expressions.len()).map_err(allocation_error)?;
    for (index, expression) in expressions.iter().enumerate() {
        if expression.ordinal as usize != index {
            return Err(referential("callable expression rows are not dense and ordinal-sorted"));
        }
        let status = import_callable_status(&expression.status, relocation, &mut child)?;
        output.push((expression.ordinal, status));
    }
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(output)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImportedCallableExpressionFacts {
    pub ordinal: u32,
    pub range: phalcom_common::range::SourceRange,
    pub knowledge: crate::types::evidence::TypeKnowledge,
    pub callable: Option<CallableId>,
    pub conditional_dispatch: Option<crate::dispatch::ConditionalDispatchSelection>,
    pub causal_invalidity: crate::checker::causal::CausalInvalidity,
    pub binding_read_safety: Option<crate::checker::flow::BindingReadSafety>,
    /// Owner-local binding ordinal; the complete callable materializer resolves
    /// this only after validating the record's binding table.
    pub binding_read: Option<u32>,
    /// Owner-local binding ordinal; the complete callable materializer resolves
    /// this only after validating the record's binding table.
    pub binding_write: Option<u32>,
}

/// Reconstructs the relocatable, non-resolution portion of each callable
/// expression row. Local binding links remain ordinals until the complete
/// owner importer builds its reverse binding table.
pub(crate) fn import_callable_expression_facts(
    record: &phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<Vec<ImportedCallableExpressionFacts>, ArtifactError> {
    use crate::checker::causal::CausalInvalidity;
    use crate::checker::flow::BindingReadSafety;
    use phalcom_compiler_artifact::semantic::world::{CallableBindingReadSafetyV1 as PortableSafety, CallableCausalInvalidityV1};

    if record.bindings.iter().enumerate().any(|(index, binding)| binding.ordinal as usize != index) {
        return Err(referential("callable bindings are not dense and ordinal-sorted"));
    }

    let mut child = allocations.child(allocations.remaining());
    let mut output = child
        .vector::<ImportedCallableExpressionFacts>(record.expressions.len())
        .map_err(allocation_error)?;
    for (index, expression) in record.expressions.iter().enumerate() {
        if expression.ordinal as usize != index {
            return Err(referential("callable expression rows are not dense and ordinal-sorted"));
        }
        if expression.range.start > expression.range.end {
            return Err(referential("callable expression source range is inverted"));
        }
        for binding in [expression.binding_read, expression.binding_write].into_iter().flatten() {
            if binding as usize >= record.bindings.len() {
                return Err(referential("callable expression references an absent owner-local binding"));
            }
        }
        let range = phalcom_common::range::SourceRange {
            start: usize::try_from(expression.range.start).map_err(|_| referential("callable expression range start exceeds address space"))?,
            end: usize::try_from(expression.range.end).map_err(|_| referential("callable expression range end exceeds address space"))?,
        };
        let knowledge = import_type_knowledge_with_allocation_budget(&expression.knowledge, relocation, &mut child)?;
        let callable = expression
            .callable
            .as_ref()
            .map(|callable| import_stable_callable_with_allocation_budget(callable, &mut child))
            .transpose()?;
        let conditional_dispatch = expression
            .conditional_dispatch
            .as_ref()
            .map(|selection| import_conditional_dispatch_selection(selection, &mut child))
            .transpose()?;
        let causal_invalidity = match expression.causal_invalidity {
            CallableCausalInvalidityV1::Clean => CausalInvalidity::Clean,
            CallableCausalInvalidityV1::One { cause } => CausalInvalidity::One(DiagnosticCauseId(cause)),
            CallableCausalInvalidityV1::Multiple => CausalInvalidity::Multiple,
        };
        let binding_read_safety = expression.binding_read_safety.as_ref().map(|safety| match safety {
            PortableSafety::ProvenInitialized => BindingReadSafety::ProvenInitialized,
            PortableSafety::DefinitelyUninitialized => BindingReadSafety::DefinitelyUninitialized,
            PortableSafety::RuntimeCheckRequired => BindingReadSafety::RuntimeCheckRequired,
        });
        output.push(ImportedCallableExpressionFacts {
            ordinal: expression.ordinal,
            range,
            knowledge,
            callable,
            conditional_dispatch,
            causal_invalidity,
            binding_read_safety,
            binding_read: expression.binding_read,
            binding_write: expression.binding_write,
        });
    }
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(output)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImportedCallableExpressionDispatchAuthority {
    pub ordinal: u32,
    pub conditional_capability: Option<crate::dispatch::ConditionalSelectedCallCapability>,
    pub trait_dispatch: Option<crate::trait_dispatch::TraitDispatchSite>,
    pub trait_dispatch_candidates: Option<Box<[crate::trait_dispatch::TraitEvidencedMemberCandidate]>>,
    pub contextual_realization: Option<Arc<crate::checker::analysis::ContextualRealization>>,
    pub explanation: Option<ExplanationId>,
}

pub(crate) fn import_callable_expression_dispatch_authority(
    record: &phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2,
    body: crate::identity::BodyId,
    relocation: &TypeRelocation,
    store: &TypeStore,
    source_plan_fingerprints: &[(crate::identity::ImplId, crate::db::ProductFingerprint)],
    allocations: &mut AllocationBudget,
    max_depth: usize,
) -> Result<Vec<ImportedCallableExpressionDispatchAuthority>, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::CallableTraitDispatchSiteV1 as PortableTraitDispatch;

    let mut child = allocations.child(allocations.remaining());
    let mut diagnostic_previous_observed = phalcom_type_meta::allocation::current_observed_bytes().unwrap_or(0);
    let mut diagnostic_previous_ledger = child.requested();
    let mut importer = CallableEvidenceImporter::new(
        record.conformance_evidence.len(),
        relocation,
        store,
        source_plan_fingerprints,
        &mut child,
        max_depth,
    )?;
    let mut output = importer
        .allocations
        .vector::<ImportedCallableExpressionDispatchAuthority>(record.expressions.len())
        .map_err(allocation_error)?;
    record_callable_import_allocation_diagnostic_stage(16, importer.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
    for (index, expression) in record.expressions.iter().enumerate() {
        if expression.ordinal as usize != index {
            return Err(referential("callable expression rows are not dense and ordinal-sorted"));
        }
        let conditional_capability = expression
            .conditional_capability
            .as_ref()
            .map(|capability| -> Result<_, ArtifactError> {
                if expression.conditional_dispatch.as_ref() != Some(&capability.selection) || capability.selection.impl_id != capability.applicability.impl_id {
                    return Err(referential(
                        "conditional call capability disagrees with its expression selection or implementation",
                    ));
                }
                Ok(crate::dispatch::ConditionalSelectedCallCapability {
                    selection: import_conditional_dispatch_selection(&capability.selection, importer.allocations)?,
                    applicability: import_callable_inherent_specialization(&capability.applicability, &record.conformance_evidence, &mut importer)?,
                })
            })
            .transpose()?;
        record_callable_import_allocation_diagnostic_stage(17, importer.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let trait_dispatch = expression
            .trait_dispatch
            .as_deref()
            .map(|site| -> Result<_, ArtifactError> {
                Ok(match site {
                    PortableTraitDispatch::AbstractRequirement { requirement } => crate::trait_dispatch::TraitDispatchSite::AbstractRequirement {
                        requirement: import_callable_requirement_identity_with_allocation_budget(requirement, importer.allocations)?,
                    },
                    PortableTraitDispatch::Assumed {
                        conformance,
                        requirement,
                        signature,
                    } => crate::trait_dispatch::TraitDispatchSite::Assumed(crate::trait_dispatch::AssumedTraitDispatchSelection {
                        conformance: import_generic_conformance_key_with_allocation_budget(conformance, relocation, importer.allocations)?,
                        requirement: import_callable_requirement_identity_with_allocation_budget(requirement, importer.allocations)?,
                        signature: import_callable_signature(signature, relocation, store, importer.allocations)?,
                    }),
                    PortableTraitDispatch::Evidenced { selection } => {
                        let evidence = importer.evidence(&record.conformance_evidence, selection.evidence, 0)?;
                        let evidence_fingerprint = crate::db::fingerprint::generic_conformance_evidence_fingerprint(&evidence);
                        let mut targets = importer
                            .allocations
                            .vector::<(crate::traits::TraitRequirementId, crate::impls::RequirementSelectionTemplate)>(selection.requirement_targets.len())
                            .map_err(allocation_error)?;
                        for target in &selection.requirement_targets {
                            targets.push((
                                import_callable_requirement_identity_with_allocation_budget(&target.requirement, importer.allocations)?,
                                import_callable_requirement_selection(&target.selection, &record.conformance_evidence, &mut importer, 0)?,
                            ));
                        }
                        let requirement_targets = import_analysis_map(targets, importer.allocations, "trait dispatch requirement targets are not canonical")?;
                        let source_impl = import_stable_impl_with_allocation_budget(&selection.source_impl, importer.allocations)?;
                        let exact_target = relocation.ty(selection.exact_target)?;
                        let exact_trait_ref = import_trait_ref_identity(&selection.exact_trait_ref, relocation, importer.allocations)?;
                        let requirement = import_callable_requirement_identity_with_allocation_budget(&selection.requirement, importer.allocations)?;
                        let callable = selection
                            .callable
                            .as_ref()
                            .map(|callable| import_stable_callable_with_allocation_budget(callable, importer.allocations))
                            .transpose()?;
                        let signature = import_callable_signature(&selection.signature, relocation, store, importer.allocations)?;
                        let sel_res = import_callable_requirement_selection(&selection.selection, &record.conformance_evidence, &mut importer, 0)?;
                        crate::trait_dispatch::TraitDispatchSite::Evidenced(crate::trait_dispatch::TraitDispatchSelection {
                            source_impl,
                            exact_target,
                            exact_trait_ref,
                            requirement,
                            callable,
                            signature,
                            evidence_fingerprint,
                            evidence,
                            selection: sel_res,
                            requirement_targets,
                        })
                    }
                })
            })
            .transpose()?;
        record_callable_import_allocation_diagnostic_stage(18, importer.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let trait_dispatch_candidates = expression
            .trait_dispatch_candidates
            .as_ref()
            .map(|candidates| {
                let mut imported = importer.allocations.vector(candidates.len()).map_err(allocation_error)?;
                for candidate in candidates {
                    imported.push(import_callable_trait_dispatch_candidate(
                        candidate,
                        &record.conformance_evidence,
                        &mut importer,
                    )?);
                }
                Ok::<_, ArtifactError>(importer.allocations.boxed_slice(imported).map_err(allocation_error)?)
            })
            .transpose()?;
        record_callable_import_allocation_diagnostic_stage(19, importer.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let contextual_realization = expression
            .contextual_realization
            .as_deref()
            .map(|realization| import_callable_contextual_realization(realization, record, body, &mut importer))
            .transpose()?;
        record_callable_import_allocation_diagnostic_stage(20, importer.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        output.push(ImportedCallableExpressionDispatchAuthority {
            ordinal: expression.ordinal,
            conditional_capability,
            trait_dispatch,
            trait_dispatch_candidates,
            contextual_realization,
            explanation: expression.explanation.map(ExplanationId),
        });
    }
    drop(importer);
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(output)
}

fn import_callable_trait_dispatch_candidate(
    candidate: &phalcom_compiler_artifact::semantic::world::CallableTraitDispatchCandidateV1,
    evidence_nodes: &[phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1],
    importer: &mut CallableEvidenceImporter<'_>,
) -> Result<crate::trait_dispatch::TraitEvidencedMemberCandidate, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::CallableTraitDispatchConvergenceV1 as Portable;

    let evidence = importer.evidence(evidence_nodes, candidate.evidence, 0)?;
    let evidence_fingerprint = crate::db::fingerprint::generic_conformance_evidence_fingerprint(&evidence);
    let mut targets = importer.allocations.vector(candidate.requirement_targets.len()).map_err(allocation_error)?;
    for target in &candidate.requirement_targets {
        targets.push((
            import_callable_requirement_identity_with_allocation_budget(&target.requirement, importer.allocations)?,
            import_callable_requirement_selection(&target.selection, evidence_nodes, importer, 0)?,
        ));
    }
    let requirement_targets = import_analysis_map(targets, importer.allocations, "trait dispatch candidate requirement targets are not canonical")?;
    let convergence = match &candidate.convergence {
        Portable::Inherent { callable } => {
            crate::trait_dispatch::TraitDispatchConvergenceKey::Inherent(import_stable_callable_with_allocation_budget(callable, importer.allocations)?)
        }
        Portable::ConditionalInherent {
            callable,
            implementation,
            applicability_fingerprint,
        } => crate::trait_dispatch::TraitDispatchConvergenceKey::ConditionalInherent {
            callable: import_stable_callable_with_allocation_budget(callable, importer.allocations)?,
            impl_id: import_stable_impl_with_allocation_budget(implementation, importer.allocations)?,
            applicability_fingerprint: crate::db::ProductFingerprint::new(*applicability_fingerprint),
        },
        Portable::DataComponent { owner, index } => crate::trait_dispatch::TraitDispatchConvergenceKey::DataComponent(crate::identity::DataComponentId::new(
            import_stable_declaration_with_allocation_budget(owner, importer.allocations)?,
            *index,
        )),
        Portable::ConformanceCallable { callable } => crate::trait_dispatch::TraitDispatchConvergenceKey::ConformanceCallable(
            import_stable_callable_with_allocation_budget(callable, importer.allocations)?,
        ),
        Portable::TraitDefault {
            callable,
            evidence_fingerprint,
        } => crate::trait_dispatch::TraitDispatchConvergenceKey::TraitDefault {
            callable: import_stable_callable_with_allocation_budget(callable, importer.allocations)?,
            evidence_fingerprint: crate::db::ProductFingerprint::new(*evidence_fingerprint),
        },
    };
    Ok(crate::trait_dispatch::TraitEvidencedMemberCandidate {
        exact_target: importer.relocation.ty(candidate.exact_target)?,
        exact_trait_ref: import_trait_ref_identity(&candidate.exact_trait_ref, importer.relocation, importer.allocations)?,
        requirement: import_callable_requirement_identity_with_allocation_budget(&candidate.requirement, importer.allocations)?,
        source_impl: import_stable_impl_with_allocation_budget(&candidate.source_impl, importer.allocations)?,
        evidence,
        evidence_fingerprint,
        selection: import_callable_requirement_selection(&candidate.selection, evidence_nodes, importer, 0)?,
        signature: import_callable_signature(&candidate.signature, importer.relocation, importer.store, importer.allocations)?,
        requirement_targets,
        convergence,
    })
}

fn import_callable_contextual_realization(
    realization: &phalcom_compiler_artifact::semantic::world::CallableContextualRealizationV1,
    record: &phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2,
    body: crate::identity::BodyId,
    importer: &mut CallableEvidenceImporter<'_>,
) -> Result<Arc<crate::checker::analysis::ContextualRealization>, ArtifactError> {
    use crate::checker::analysis::{
        AlgebraicBasisAxis, ContextualRealization, ContextualRealizationOperation, RealizationCapabilitySelection, RealizationCoefficientSource,
        RealizationEvaluationPlan,
    };
    use crate::identity::{ExpressionId, LocalExpressionId};
    use phalcom_compiler_artifact::semantic::world as wire;

    let expression = |ordinal: u32| -> Result<ExpressionId, ArtifactError> {
        if ordinal as usize >= record.expressions.len() {
            return Err(referential("contextual realization references an absent owner-local expression"));
        }
        Ok(ExpressionId::new(body, LocalExpressionId(ordinal)))
    };
    let range = |range: &phalcom_compiler_artifact::module_world::SpanV1| -> Result<SourceRange, ArtifactError> {
        if range.start > range.end {
            return Err(referential("contextual realization source range is inverted"));
        }
        Ok(SourceRange {
            start: usize::try_from(range.start).map_err(|_| referential("contextual realization source range start exceeds address space"))?,
            end: usize::try_from(range.end).map_err(|_| referential("contextual realization source range end exceeds address space"))?,
        })
    };
    let capability = |selection: &wire::CallableRealizationCapabilityV1,
                      importer: &mut CallableEvidenceImporter<'_>|
     -> Result<RealizationCapabilitySelection, ArtifactError> {
        Ok(match selection {
            wire::CallableRealizationCapabilityV1::Evidenced { selection } => {
                RealizationCapabilitySelection::Evidenced(import_callable_trait_selection(selection, &record.conformance_evidence, importer)?)
            }
            wire::CallableRealizationCapabilityV1::Assumed {
                conformance,
                requirement,
                signature,
            } => RealizationCapabilitySelection::Assumed(crate::trait_dispatch::AssumedTraitDispatchSelection {
                conformance: import_generic_conformance_key_with_allocation_budget(conformance, importer.relocation, importer.allocations)?,
                requirement: import_callable_requirement_identity_with_allocation_budget(requirement, importer.allocations)?,
                signature: import_callable_signature(signature, importer.relocation, importer.store, importer.allocations)?,
            }),
        })
    };
    let operation = match &realization.operation {
        wire::CallableContextualRealizationOperationV1::Basis {
            basis_type,
            axis,
            coefficient,
            evaluation,
        } => {
            let coefficient = match coefficient {
                wire::CallableRealizationCoefficientV1::One {
                    scalar_type,
                    capability: selected,
                } => RealizationCoefficientSource::One {
                    scalar_type: importer.relocation.ty(*scalar_type)?,
                    one: capability(selected, importer)?,
                },
                wire::CallableRealizationCoefficientV1::IntegerLiteral { digits, radix } => RealizationCoefficientSource::IntegerLiteral {
                    digits: importer.allocations.boxed_str(digits).map_err(allocation_error)?,
                    radix: *radix,
                },
                wire::CallableRealizationCoefficientV1::FloatLiteral { bits } => RealizationCoefficientSource::FloatLiteral { bits: *bits },
                wire::CallableRealizationCoefficientV1::Expression {
                    expression: source_expression,
                    scalar_type,
                } => RealizationCoefficientSource::Expression {
                    expression: expression(*source_expression)?,
                    scalar_type: importer.relocation.ty(*scalar_type)?,
                },
            };
            let evaluation = match evaluation {
                wire::CallableRealizationEvaluationV1::LiteralCoefficientThenAxis => RealizationEvaluationPlan::LiteralCoefficientThenAxis,
                wire::CallableRealizationEvaluationV1::MultiplicativeIdentityThenSourceBasis => {
                    RealizationEvaluationPlan::MultiplicativeIdentityThenSourceBasis
                }
                wire::CallableRealizationEvaluationV1::SourceOperands {
                    first,
                    second,
                    coefficient,
                    basis,
                } => RealizationEvaluationPlan::SourceOperands {
                    first: expression(*first)?,
                    second: expression(*second)?,
                    coefficient: expression(*coefficient)?,
                    basis: expression(*basis)?,
                },
            };
            ContextualRealizationOperation::Basis {
                basis_type: importer.relocation.ty(*basis_type)?,
                axis: match axis {
                    wire::CallableAlgebraicBasisAxisV1::I => AlgebraicBasisAxis::I,
                    wire::CallableAlgebraicBasisAxisV1::J => AlgebraicBasisAxis::J,
                    wire::CallableAlgebraicBasisAxisV1::K => AlgebraicBasisAxis::K,
                },
                coefficient,
                evaluation,
            }
        }
        wire::CallableContextualRealizationOperationV1::RealEmbedding { value, scalar_type } => ContextualRealizationOperation::RealEmbedding {
            value: expression(*value)?,
            scalar_type: importer.relocation.ty(*scalar_type)?,
        },
    };
    let realization = ContextualRealization {
        source_expression: expression(realization.source_expression)?,
        source_range: range(&realization.source_range)?,
        source_type: importer.relocation.ty(realization.source_type)?,
        target_type: importer.relocation.ty(realization.target_type)?,
        operation,
        realization: capability(&realization.realization, importer)?,
    };
    importer.allocations.arc(realization).map_err(allocation_error)
}

fn import_unknown_reason_with_allocation_budget(reason: &UnknownReasonV1, allocations: &mut AllocationBudget) -> Result<UnknownReason, ArtifactError> {
    Ok(match reason {
        UnknownReasonV1::NoTypeEvidence => UnknownReason::NoTypeEvidence,
        UnknownReasonV1::MissingInitializer => UnknownReason::MissingInitializer,
        UnknownReasonV1::UnannotatedDeclaration => UnknownReason::UnannotatedDeclaration,
        UnknownReasonV1::UnresolvedName { name } => UnknownReason::UnresolvedName(allocations.boxed_str(name).map_err(allocation_error)?),
        UnknownReasonV1::DynamicMessageSend => UnknownReason::DynamicMessageSend,
        UnknownReasonV1::OpaqueNative => UnknownReason::OpaqueNative,
        UnknownReasonV1::RecursiveFixpoint => UnknownReason::RecursiveFixpoint,
        UnknownReasonV1::UncheckedExpression => UnknownReason::UncheckedExpression,
        UnknownReasonV1::SyntaxError => UnknownReason::SyntaxError,
        UnknownReasonV1::UnderconstrainedTypeVariable => UnknownReason::UnderconstrainedTypeVariable,
        UnknownReasonV1::InferenceConflict => UnknownReason::InferenceConflict,
        UnknownReasonV1::InferenceAmbiguous => UnknownReason::InferenceAmbiguous,
        UnknownReasonV1::InferenceBlocked => UnknownReason::InferenceBlocked,
        UnknownReasonV1::InferenceCancelled => UnknownReason::InferenceCancelled,
        UnknownReasonV1::InferenceBudgetExceeded => UnknownReason::InferenceBudgetExceeded,
        UnknownReasonV1::SuppressedByInvalidCause => UnknownReason::SuppressedByInvalidCause,
        UnknownReasonV1::ExistentialEscape => UnknownReason::ExistentialEscape,
    })
}

fn import_source_span(source: &phalcom_compiler_artifact::semantic::world::EnumRequirementSourceV1) -> Result<SemanticSourceSpan, ArtifactError> {
    let module = from_stable_module(&source.module).map_err(referential)?;
    if source.range.start > source.range.end {
        return Err(referential("callable source span is inverted"));
    }
    Ok(SemanticSourceSpan::new(
        module,
        phalcom_common::range::SourceRange {
            start: usize::try_from(source.range.start).map_err(|_| referential("callable source start exceeds address space"))?,
            end: usize::try_from(source.range.end).map_err(|_| referential("callable source end exceeds address space"))?,
        },
    ))
}

fn import_source_span_with_allocation_budget(
    source: &phalcom_compiler_artifact::semantic::world::EnumRequirementSourceV1,
    allocations: &mut AllocationBudget,
) -> Result<SemanticSourceSpan, ArtifactError> {
    let module = import_stable_module_with_allocation_budget(&source.module, allocations)?;
    if source.range.start > source.range.end {
        return Err(referential("callable source span is inverted"));
    }
    Ok(SemanticSourceSpan::new(
        module,
        phalcom_common::range::SourceRange {
            start: usize::try_from(source.range.start).map_err(|_| referential("callable source start exceeds address space"))?,
            end: usize::try_from(source.range.end).map_err(|_| referential("callable source end exceeds address space"))?,
        },
    ))
}

pub(crate) fn import_trait_surface_diagnostic(
    record: &phalcom_compiler_artifact::semantic::world::CallableEvidenceDiagnosticV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::diagnostic::SemanticDiagnostic, ArtifactError> {
    use crate::diagnostic::{DiagnosticFix, DiagnosticGuidance, DiagnosticLabel, DiagnosticSeverity, ExplanationRef, SemanticDiagnostic};
    use phalcom_compiler_artifact::semantic::world::{CallableDiagnosticSeverityV1, CallableEvidenceDiagnosticGuidanceV1};

    let range = |range: &phalcom_compiler_artifact::module_world::SpanV1| -> Result<SourceRange, ArtifactError> {
        if range.start > range.end {
            return Err(referential("trait surface diagnostic range is inverted"));
        }
        Ok(SourceRange {
            start: usize::try_from(range.start).map_err(|_| referential("trait surface diagnostic range start exceeds address space"))?,
            end: usize::try_from(range.end).map_err(|_| referential("trait surface diagnostic range end exceeds address space"))?,
        })
    };
    let code = DiagnosticCode::from_stable_code(&record.code).ok_or_else(|| referential("trait surface diagnostic has an unknown stable code"))?;
    let primary = import_source_span_with_allocation_budget(&record.primary, allocations)?;
    let primary_range = range(&record.primary_range)?;
    if primary.range != primary_range {
        return Err(referential("trait surface diagnostic primary range disagrees with its source span"));
    }
    let mut labels = allocations.vector(record.labels.len()).map_err(allocation_error)?;
    for label in &record.labels {
        labels.push(DiagnosticLabel {
            span: import_source_span_with_allocation_budget(&label.source, allocations)?,
            range: range(&label.range)?,
            message: allocations.string(&label.message).map_err(allocation_error)?,
        });
    }
    let mut imported_guidance = allocations.vector(record.guidance.len()).map_err(allocation_error)?;
    for guidance in &record.guidance {
        imported_guidance.push(match guidance {
            CallableEvidenceDiagnosticGuidanceV1::ChangeAnnotation { range: source_range, ty } => DiagnosticGuidance::ChangeAnnotation {
                range: range(source_range)?,
                ty: relocation.ty(*ty)?,
            },
            CallableEvidenceDiagnosticGuidanceV1::SupplyAssignableValue { expected } => DiagnosticGuidance::SupplyAssignableValue {
                expected: relocation.ty(*expected)?,
            },
            CallableEvidenceDiagnosticGuidanceV1::UseCallableShape { callable } => DiagnosticGuidance::UseCallableShape {
                callable: import_exact_callable_with_allocation_budget(callable, allocations)?,
            },
            CallableEvidenceDiagnosticGuidanceV1::EstablishTypeEvidence { range: source_range, expected } => DiagnosticGuidance::EstablishTypeEvidence {
                range: range(source_range)?,
                expected: expected.map(|ty| relocation.ty(ty)).transpose()?,
            },
            CallableEvidenceDiagnosticGuidanceV1::ResolveGenericParameter { parameter } => DiagnosticGuidance::ResolveGenericParameter {
                parameter: relocation.parameter(*parameter)?,
            },
        });
    }
    let guidance = imported_guidance;
    let mut explanations = allocations.vector(record.explanations.len()).map_err(allocation_error)?;
    for reference in &record.explanations {
        explanations.push(ExplanationRef::new(
            import_exact_callable_with_allocation_budget(&reference.callable, allocations)?,
            crate::identity::ExplanationId(reference.ordinal),
        ));
    }
    let mut fixes = allocations.vector(record.fixes.len()).map_err(allocation_error)?;
    for fix in &record.fixes {
        let replacement = fix
            .replacement
            .as_ref()
            .map(|replacement| Ok::<_, ArtifactError>((range(&replacement.range)?, allocations.string(&replacement.text).map_err(allocation_error)?)))
            .transpose()?;
        fixes.push(DiagnosticFix {
            message: allocations.string(&fix.message).map_err(allocation_error)?,
            replacement,
        });
    }
    let mut notes = allocations.vector(record.notes.len()).map_err(allocation_error)?;
    for note in &record.notes {
        notes.push(allocations.string(note).map_err(allocation_error)?);
    }
    let mut helps = allocations.vector(record.helps.len()).map_err(allocation_error)?;
    for help in &record.helps {
        helps.push(allocations.string(help).map_err(allocation_error)?);
    }
    Ok(SemanticDiagnostic {
        code,
        severity: match record.severity {
            CallableDiagnosticSeverityV1::Error => DiagnosticSeverity::Error,
            CallableDiagnosticSeverityV1::Warning => DiagnosticSeverity::Warning,
            CallableDiagnosticSeverityV1::Information => DiagnosticSeverity::Information,
            CallableDiagnosticSeverityV1::Hint => DiagnosticSeverity::Hint,
        },
        message: allocations.string(&record.message).map_err(allocation_error)?,
        primary,
        primary_range,
        labels,
        notes,
        helps,
        guidance,
        explanations,
        fixes,
        root_cause: record.root_cause.map(crate::identity::DiagnosticCauseId),
    })
}

fn import_callable_requirement_identity_with_allocation_budget(
    identity: &phalcom_compiler_artifact::semantic::world::CallableRequirementIdentityV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::traits::TraitRequirementId, ArtifactError> {
    let owner = import_stable_declaration_with_allocation_budget(&identity.owner, allocations)?;
    let selector = import_stable_selector_with_allocation_budget(&identity.selector, allocations)?;
    let side = match identity.side {
        0 => DispatchSide::Instance,
        1 => DispatchSide::Class,
        _ => return Err(referential("callable requirement has an unknown dispatch side")),
    };
    Ok(crate::traits::TraitRequirementId { owner, selector, side })
}

fn import_callable_associated_type_requirement_identity(
    identity: &phalcom_compiler_artifact::semantic::world::CallableAssociatedTypeRequirementIdentityV1,
) -> Result<crate::traits::AssociatedTypeRequirementId, ArtifactError> {
    Ok(crate::traits::AssociatedTypeRequirementId::new(
        from_stable_declaration(&identity.owner).map_err(referential)?,
        identity.index,
    ))
}

fn import_callable_associated_type_requirement_identity_with_allocation_budget(
    identity: &phalcom_compiler_artifact::semantic::world::CallableAssociatedTypeRequirementIdentityV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::traits::AssociatedTypeRequirementId, ArtifactError> {
    Ok(crate::traits::AssociatedTypeRequirementId::new(
        import_stable_declaration_with_allocation_budget(&identity.owner, allocations)?,
        identity.index,
    ))
}

fn import_callable_type_environment_with_budget(
    environment: &phalcom_compiler_artifact::semantic::world::CallableTypeEnvironmentV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::types::environment::TypeEnvironment, ArtifactError> {
    use phalcom_compiler_artifact::semantic::type_store::StaticArgumentV1;
    let mut seen = allocations.vector(environment.bindings.len()).map_err(allocation_error)?;
    let mut bindings = allocations.vector(environment.bindings.len()).map_err(allocation_error)?;
    let mut dynamic_bindings = allocations.vector(environment.bindings.len()).map_err(allocation_error)?;
    let mut row_bindings = allocations.vector(environment.bindings.len()).map_err(allocation_error)?;
    for binding in &environment.bindings {
        let parameter = relocation.parameter(binding.parameter)?;
        seen.push(parameter);
        match &binding.argument {
            StaticArgumentV1::Term { term } => bindings.push((parameter, relocation.ty(*term)?)),
            StaticArgumentV1::DynamicType => dynamic_bindings.push(parameter),
            StaticArgumentV1::RecordRow { row } => row_bindings.push((parameter, relocation.row(*row)?)),
        }
    }
    seen.sort_unstable();
    if seen.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(referential("callable type environment binds one parameter more than once"));
    }
    bindings.sort_unstable_by_key(|(parameter, _)| *parameter);
    dynamic_bindings.sort_unstable();
    row_bindings.sort_unstable_by_key(|(parameter, _)| *parameter);
    let bindings = crate::checker::analysis_map::AnalysisMap::from_budgeted_entries(bindings, allocations).map_err(|error| match error {
        crate::checker::analysis_map::AnalysisMapBuildError::Entries(_) => referential("callable type environment bindings are not canonical"),
        crate::checker::analysis_map::AnalysisMapBuildError::Allocation(error) => allocation_error(error),
    })?;
    let dynamic_bindings = crate::checker::analysis_set::AnalysisSet::from_budgeted_entries(dynamic_bindings, allocations).map_err(|error| match error {
        crate::checker::analysis_set::AnalysisSetBuildError::Entries(_) => referential("callable dynamic bindings are not canonical"),
        crate::checker::analysis_set::AnalysisSetBuildError::Allocation(error) => allocation_error(error),
    })?;
    let row_bindings = crate::checker::analysis_map::AnalysisMap::from_budgeted_entries(row_bindings, allocations).map_err(|error| match error {
        crate::checker::analysis_map::AnalysisMapBuildError::Entries(_) => referential("callable row bindings are not canonical"),
        crate::checker::analysis_map::AnalysisMapBuildError::Allocation(error) => allocation_error(error),
    })?;
    Ok(crate::types::environment::TypeEnvironment {
        bindings,
        dynamic_bindings,
        row_bindings,
        self_binding: environment.self_binding.map(|ty| relocation.ty(ty)).transpose()?,
    })
}

fn import_callable_inherent_specialization(
    specialization: &phalcom_compiler_artifact::semantic::world::CallableInherentImplSpecializationV1,
    evidence_nodes: &[phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1],
    importer: &mut CallableEvidenceImporter<'_>,
) -> Result<crate::impls::InherentImplSpecialization, ArtifactError> {
    let impl_id = import_stable_impl_with_allocation_budget(&specialization.impl_id, importer.allocations)?;
    let mut bindings = importer.allocations.vector(specialization.bindings.len()).map_err(allocation_error)?;
    for binding in &specialization.bindings {
        bindings.push((importer.relocation.parameter(binding.parameter)?, importer.relocation.ty(binding.ty)?));
    }
    bindings.sort_unstable_by_key(|(parameter, _)| *parameter);
    let bindings = import_analysis_map(bindings, importer.allocations, "inherent specialization bindings are not canonical")?;
    let environment = import_callable_type_environment_with_budget(&specialization.environment, importer.relocation, importer.allocations)?;
    let conformance_provenance = if let Some(bindings) = &specialization.conformance_provenance {
        let mut imported = importer.allocations.vector(bindings.len()).map_err(allocation_error)?;
        for binding in bindings {
            imported.push(importer.binding(binding, evidence_nodes)?);
        }
        Some(importer.allocations.boxed_slice(imported).map_err(allocation_error)?)
    } else {
        None
    };
    Ok(crate::impls::InherentImplSpecialization {
        impl_id,
        receiver: importer.relocation.ty(specialization.receiver)?,
        owner_view: importer.relocation.ty(specialization.owner_view)?,
        bindings,
        environment,
        conformance_provenance,
    })
}

fn import_callable_effective_inherent_witness(
    witness: &phalcom_compiler_artifact::semantic::world::CallableEffectiveInherentWitnessV1,
    evidence_nodes: &[phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1],
    importer: &mut CallableEvidenceImporter<'_>,
) -> Result<crate::impls::EffectiveInherentWitness, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::CallableInherentImplTargetV1 as Target;
    let conditional_domain = if let Some(domain) = &witness.conditional_domain {
        let target = match &domain.target {
            Target::Declaration { declaration } => {
                crate::impls::InherentImplTarget::Declaration(import_stable_declaration_with_allocation_budget(declaration, importer.allocations)?)
            }
            Target::ExactEnumCase { variant } => {
                crate::impls::InherentImplTarget::ExactEnumCase(import_stable_variant_with_allocation_budget(variant, importer.allocations)?)
            }
        };
        let generic_signature = domain
            .generic_signature
            .as_ref()
            .map(|signature| import_generic_signature_with_allocation_budget(signature, importer.relocation, importer.store, importer.allocations))
            .transpose()?;
        let mut constraints = importer.allocations.vector(domain.constraints.len()).map_err(allocation_error)?;
        for constraint in &domain.constraints {
            constraints.push(import_constraint_with_allocation_budget(constraint, importer.relocation, importer.allocations)?);
        }
        let constraints = importer.allocations.boxed_slice(constraints).map_err(allocation_error)?;
        let impl_id = import_stable_impl_with_allocation_budget(&domain.impl_id, importer.allocations)?;
        Some(
            importer
                .allocations
                .arc(crate::impls::InherentImplDomain {
                    impl_id,
                    target,
                    head_type: importer.relocation.ty(domain.head_type)?,
                    generic_signature,
                    constraints,
                })
                .map_err(allocation_error)?,
        )
    } else {
        None
    };
    let applicability = witness
        .applicability
        .as_ref()
        .map(|specialization| import_callable_inherent_specialization(specialization, evidence_nodes, importer))
        .transpose()?;
    Ok(crate::impls::EffectiveInherentWitness {
        callable: import_exact_callable_with_allocation_budget(&witness.callable, importer.allocations)?,
        signature: import_callable_signature(&witness.signature, importer.relocation, importer.store, importer.allocations)?,
        visibility: import_visibility(&witness.visibility),
        owner: import_stable_declaration_with_allocation_budget(&witness.owner, importer.allocations)?,
        conditional_impl: witness
            .conditional_impl
            .as_ref()
            .map(|id| import_stable_impl_with_allocation_budget(id, importer.allocations))
            .transpose()?,
        applicability,
        conditional_domain,
    })
}

fn import_callable_requirement_selection(
    selection: &phalcom_compiler_artifact::semantic::world::CallableRequirementSelectionV1,
    evidence_nodes: &[phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1],
    importer: &mut CallableEvidenceImporter<'_>,
    depth: usize,
) -> Result<crate::impls::RequirementSelectionTemplate, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::CallableRequirementSelectionV1 as Portable;
    if depth > importer.max_depth {
        return Err(referential("witness requirement-selection nesting limit exceeded"));
    }
    Ok(match selection {
        Portable::ConformanceCallable { callable } => crate::impls::RequirementSelectionTemplate::ConformanceCallable {
            callable: import_exact_callable_with_allocation_budget(callable, importer.allocations)?,
        },
        Portable::InherentCallable {
            callable,
            conditional_impl,
            applicability,
        } => crate::impls::RequirementSelectionTemplate::InherentCallable {
            callable: import_exact_callable_with_allocation_budget(callable, importer.allocations)?,
            conditional_impl: conditional_impl
                .as_ref()
                .map(|id| import_stable_impl_with_allocation_budget(id, importer.allocations))
                .transpose()?,
            applicability: applicability
                .as_ref()
                .map(|specialization| import_callable_inherent_specialization(specialization, evidence_nodes, importer))
                .transpose()?,
        },
        Portable::ConditionalInherent { candidate, pending, fallback } => crate::impls::RequirementSelectionTemplate::ConditionalInherent {
            candidate: import_callable_effective_inherent_witness(candidate, evidence_nodes, importer)?,
            pending: import_callable_conformance_completeness(pending, importer.relocation, importer.allocations)?,
            fallback: fallback
                .as_deref()
                .map(|selection| {
                    let sel = import_callable_requirement_selection(selection, evidence_nodes, importer, depth + 1)?;
                    importer.allocations.boxed(sel).map_err(allocation_error)
                })
                .transpose()?,
        },
        Portable::DataComponent {
            owner,
            index,
            specialized_type,
        } => crate::impls::RequirementSelectionTemplate::DataComponent {
            component: DataComponentId::new(import_stable_declaration_with_allocation_budget(owner, importer.allocations)?, *index),
            specialized_type: importer.relocation.ty(*specialized_type)?,
        },
        Portable::TraitDefault { callable } => crate::impls::RequirementSelectionTemplate::TraitDefault {
            callable: import_exact_callable_with_allocation_budget(callable, importer.allocations)?,
        },
    })
}

fn import_callable_derived_conformance_rule(
    rule: &phalcom_compiler_artifact::semantic::world::CallableDerivedConformanceRuleV1,
    evidence_nodes: &[phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1],
    importer: &mut CallableEvidenceImporter<'_>,
    depth: usize,
) -> Result<crate::derived_conformance::DerivedConformanceRule, ArtifactError> {
    if depth > importer.max_depth {
        return Err(referential("callable derived-rule evidence nesting limit exceeded"));
    }
    let impl_id = import_stable_impl_with_allocation_budget(&rule.impl_id, importer.allocations)?;
    let source_module = import_stable_module_with_allocation_budget(&rule.source_module, importer.allocations)?;
    let target_parameter = importer.relocation.parameter(rule.target_parameter)?;
    let target_template = importer.relocation.ty(rule.target_template)?;
    let trait_ref = import_trait_ref_identity(&rule.trait_ref, importer.relocation, importer.allocations)?;
    let generic_signature =
        import_generic_signature_with_allocation_budget(&rule.generic_signature, importer.relocation, importer.store, importer.allocations)?;
    let associated_type_plan = import_callable_associated_type_plan(
        &rule.associated_type_plan,
        &impl_id,
        &trait_ref.declaration,
        importer.relocation,
        importer.allocations,
    )?;

    let mut requirement_views = importer.allocations.vector(rule.requirement_views.len()).map_err(allocation_error)?;
    for view in &rule.requirement_views {
        let requirement = import_callable_requirement_identity_with_allocation_budget(&view.requirement, importer.allocations)?;
        if requirement.owner != trait_ref.declaration {
            return Err(referential("derived rule requirement view belongs to another trait"));
        }
        requirement_views.push((
            requirement.clone_with_allocation_budget(importer.allocations).map_err(allocation_error)?,
            crate::traits::InstantiatedTraitRequirement {
                requirement,
                callable: import_exact_callable_with_allocation_budget(&view.callable, importer.allocations)?,
                signature: import_callable_signature(&view.signature, importer.relocation, importer.store, importer.allocations)?,
                visibility: import_visibility(&view.visibility),
                default_callable: view
                    .default_callable
                    .as_ref()
                    .map(|callable| import_exact_callable_with_allocation_budget(callable, importer.allocations))
                    .transpose()?,
            },
        ));
    }
    let requirement_views = import_analysis_map(requirement_views, importer.allocations, "derived rule requirement views are not canonical")?;

    let mut requirements = importer.allocations.vector(rule.requirements.len()).map_err(allocation_error)?;
    for entry in &rule.requirements {
        let requirement = import_callable_requirement_identity_with_allocation_budget(&entry.requirement, importer.allocations)?;
        if requirement.owner != trait_ref.declaration {
            return Err(referential("derived rule selection belongs to another trait"));
        }
        let selection = import_callable_requirement_selection(&entry.selection, evidence_nodes, importer, depth + 1)?;
        requirements.push((requirement, selection));
    }
    let requirements = import_analysis_map(requirements, importer.allocations, "derived rule requirements are not canonical")?;

    Ok(crate::derived_conformance::DerivedConformanceRule::from_frozen_parts(
        impl_id,
        source_module,
        target_parameter,
        target_template,
        trait_ref,
        generic_signature,
        associated_type_plan,
        requirement_views,
        requirements,
        import_source_span_with_allocation_budget(&rule.source, importer.allocations)?,
    ))
}

fn import_derived_conformance_rule_record(
    record: &phalcom_compiler_artifact::semantic::world::DerivedConformanceRuleRecordV1,
    relocation: &TypeRelocation,
    store: &TypeStore,
    source_plan_fingerprints: &[(crate::identity::ImplId, crate::db::ProductFingerprint)],
    allocations: &mut AllocationBudget,
    max_depth: usize,
) -> Result<crate::derived_conformance::DerivedConformanceRule, ArtifactError> {
    let mut child = allocations.child(allocations.remaining());
    let mut importer = CallableEvidenceImporter::new(
        record.conformance_evidence.len(),
        relocation,
        store,
        source_plan_fingerprints,
        &mut child,
        max_depth,
    )?;
    let rule = import_callable_derived_conformance_rule(&record.rule, &record.conformance_evidence, &mut importer, 0)?;
    drop(importer);
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(rule)
}

fn import_callable_associated_type_failure(
    failure: &phalcom_compiler_artifact::semantic::world::CallableAssociatedTypeBindingFailureV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::impls::AssociatedTypeBindingFailure, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::CallableAssociatedTypeBindingFailureKindV1 as Kind;
    Ok(crate::impls::AssociatedTypeBindingFailure {
        kind: match failure.kind {
            Kind::Missing => crate::impls::AssociatedTypeBindingFailureKind::Missing,
            Kind::Duplicate => crate::impls::AssociatedTypeBindingFailureKind::Duplicate,
            Kind::Unknown => crate::impls::AssociatedTypeBindingFailureKind::Unknown,
            Kind::Invalid => crate::impls::AssociatedTypeBindingFailureKind::Invalid,
        },
        name: allocations.boxed_str(&failure.name).map_err(allocation_error)?,
        reason: allocations.boxed_str(&failure.reason).map_err(allocation_error)?,
        source: import_source_span_with_allocation_budget(&failure.source, allocations)?,
        requirement: failure
            .requirement
            .as_ref()
            .map(|requirement| import_callable_associated_type_requirement_identity_with_allocation_budget(requirement, allocations))
            .transpose()?,
    })
}

fn import_callable_associated_type_plan(
    plan: &phalcom_compiler_artifact::semantic::world::CallableConformanceAssociatedTypePlanV1,
    expected_impl: &crate::identity::ImplId,
    expected_trait: &crate::identity::DeclarationId,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::impls::ConformanceAssociatedTypePlan, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::CallableAssociatedTypeBindingValueV1 as Value;
    let impl_id = import_stable_impl_with_allocation_budget(&plan.impl_id, allocations)?;
    if &impl_id != expected_impl {
        return Err(referential("associated-type plan implementation disagrees with its witness owner"));
    }
    let mut bindings = allocations.vector(plan.bindings.len()).map_err(allocation_error)?;
    for binding in &plan.bindings {
        let requirement = import_callable_associated_type_requirement_identity_with_allocation_budget(&binding.requirement, allocations)?;
        if requirement.owner != *expected_trait {
            return Err(referential("associated-type binding belongs to another trait"));
        }
        let source_impl = import_stable_impl_with_allocation_budget(&binding.source_impl, allocations)?;
        let source = import_source_span_with_allocation_budget(&binding.source, allocations)?;
        let value_template = match binding.value {
            Value::Type { ty } => crate::impls::AssociatedTypeBindingValue::Type(relocation.ty(ty)?),
            Value::Dynamic => crate::impls::AssociatedTypeBindingValue::Dynamic,
        };
        bindings.push((
            requirement.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            crate::impls::AssociatedTypeBindingTemplate {
                requirement,
                value_template,
                source_impl,
                source,
            },
        ));
    }
    let bindings = crate::checker::analysis_map::AnalysisMap::from_budgeted_entries(bindings, allocations).map_err(|error| match error {
        crate::checker::analysis_map::AnalysisMapBuildError::Entries(_) => referential("associated-type plan bindings are not canonical"),
        crate::checker::analysis_map::AnalysisMapBuildError::Allocation(error) => allocation_error(error),
    })?;
    let mut failures = allocations.vector(plan.failures.len()).map_err(allocation_error)?;
    for failure in &plan.failures {
        let failure = import_callable_associated_type_failure(failure, allocations)?;
        if failure.requirement.as_ref().is_some_and(|requirement| requirement.owner != *expected_trait) {
            return Err(referential("associated-type failure belongs to another trait"));
        }
        failures.push(failure);
    }
    let mut diagnostics = allocations.vector(plan.diagnostics.len()).map_err(allocation_error)?;
    for diagnostic in &plan.diagnostics {
        diagnostics.push(import_trait_surface_diagnostic(diagnostic, relocation, allocations)?);
    }
    let diagnostics = allocations.boxed_slice(diagnostics).map_err(allocation_error)?;
    Ok(crate::impls::ConformanceAssociatedTypePlan::from_parts(
        impl_id,
        bindings,
        allocations.boxed_slice(failures).map_err(allocation_error)?,
        diagnostics,
    ))
}

fn import_callable_conformance_completeness(
    completeness: &phalcom_compiler_artifact::semantic::world::CallableConformanceCompletenessV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::impls::ConformanceCompleteness, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::{CallableConformanceCompletenessV1 as Portable, CallableConformanceFailureV1};
    Ok(match completeness {
        Portable::Complete => crate::impls::ConformanceCompleteness::Complete,
        Portable::Incomplete { failures } => {
            let mut imported = allocations.vector(failures.len()).map_err(allocation_error)?;
            for failure in failures {
                imported.push(match failure {
                    CallableConformanceFailureV1::Behavioral { requirement, reason } => {
                        crate::impls::ConformanceFailure::Behavioral(crate::impls::RequirementFailure {
                            requirement: import_callable_requirement_identity_with_allocation_budget(requirement, allocations)?,
                            reason: allocations.boxed_str(reason).map_err(allocation_error)?,
                        })
                    }
                    CallableConformanceFailureV1::AssociatedType { failure } => {
                        crate::impls::ConformanceFailure::AssociatedType(import_callable_associated_type_failure(failure, allocations)?)
                    }
                });
            }
            crate::impls::ConformanceCompleteness::Incomplete {
                failures: allocations.boxed_slice(imported).map_err(allocation_error)?,
            }
        }
        Portable::Unknown { reason } => crate::impls::ConformanceCompleteness::Unknown(import_unknown_reason_with_allocation_budget(reason, allocations)?),
        Portable::Blocked { reason } => crate::impls::ConformanceCompleteness::Blocked(import_callable_block_reason(reason, relocation, allocations)?),
        Portable::Dynamic { reason } => crate::impls::ConformanceCompleteness::Dynamic(crate::types::outcome::DynamicBoundaryObligation {
            reason: allocations.string(reason).map_err(allocation_error)?,
        }),
        Portable::Cancelled => crate::impls::ConformanceCompleteness::Cancelled,
        Portable::BudgetExceeded { report } => crate::impls::ConformanceCompleteness::BudgetExceeded(import_callable_binding_budget(report)),
        Portable::InternalFailure { message } => {
            crate::impls::ConformanceCompleteness::InternalFailure(allocations.boxed_str(message).map_err(allocation_error)?)
        }
    })
}

fn import_return_flow(flow: &phalcom_compiler_artifact::semantic::world::ReturnFlowV1) -> Result<phalcom_native_meta::ReturnFlowSpec, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::ReturnFlowV1 as Portable;
    Ok(match flow {
        Portable::Value => phalcom_native_meta::ReturnFlowSpec::Value,
        Portable::Receiver => phalcom_native_meta::ReturnFlowSpec::Receiver,
        Portable::Argument { index } => {
            phalcom_native_meta::ReturnFlowSpec::Argument(usize::try_from(*index).map_err(|_| referential("callable return flow index exceeds address space"))?)
        }
        Portable::Never => phalcom_native_meta::ReturnFlowSpec::Never,
        Portable::Unknown => phalcom_native_meta::ReturnFlowSpec::Unknown,
    })
}

fn matches_native_type_expr(spec: &phalcom_native_meta::TypeExprSpec, wire: &phalcom_compiler_artifact::semantic::world::TypeExprSpecV1) -> bool {
    use phalcom_compiler_artifact::semantic::world::TypeExprSpecV1;
    use phalcom_native_meta::TypeExprSpec as Native;
    match (spec, wire) {
        (Native::Unknown, TypeExprSpecV1::Unknown) => true,
        (Native::Never, TypeExprSpecV1::Never) => true,
        (Native::SelfType, TypeExprSpecV1::SelfType) => true,
        (Native::Universe(owner), TypeExprSpecV1::Universe { owner: wire_owner }) => owner.name() == wire_owner.as_ref(),
        (Native::Parameter(name), TypeExprSpecV1::Parameter { name: wire_name }) => *name == wire_name.as_ref(),
        (
            Native::Applied { origin, arguments },
            TypeExprSpecV1::Applied {
                origin: wire_origin,
                arguments: wire_args,
            },
        ) => {
            matches_native_type_expr(origin, wire_origin)
                && arguments.len() == wire_args.len()
                && arguments.iter().zip(wire_args.iter()).all(|(a, w)| matches_native_type_expr(a, w))
        }
        (Native::Union(members), TypeExprSpecV1::Union { members: wire_members }) => {
            members.len() == wire_members.len() && members.iter().zip(wire_members.iter()).all(|(m, w)| matches_native_type_expr(m, w))
        }
        (Native::Tuple(tuple), TypeExprSpecV1::Tuple { tuple: wire_tuple }) => {
            tuple.positional.len() == wire_tuple.positional.len()
                && tuple
                    .positional
                    .iter()
                    .zip(wire_tuple.positional.iter())
                    .all(|(p, w)| matches_native_type_expr(p, w))
                && tuple.labeled.len() == wire_tuple.labeled.len()
                && tuple
                    .labeled
                    .iter()
                    .zip(wire_tuple.labeled.iter())
                    .all(|(l, w)| l.label == w.label.as_ref() && matches_native_type_expr(l.ty, &w.ty))
                && match (tuple.rest, &wire_tuple.rest) {
                    (None, None) => true,
                    (Some(r), Some(wr)) => match (r.ty, &wr.ty) {
                        (None, None) => true,
                        (Some(t), Some(wt)) => matches_native_type_expr(t, wt),
                        _ => false,
                    },
                    _ => false,
                }
        }
        _ => false,
    }
}

fn matches_native_effects(spec: phalcom_native_meta::EffectSpec, wire: &phalcom_compiler_artifact::semantic::world::EffectSpecV1) -> bool {
    use phalcom_compiler_artifact::semantic::world::{EffectSpecV1, NativeEffectV1};
    match (spec, wire) {
        (phalcom_native_meta::EffectSpec::Unknown, EffectSpecV1::Unknown) => true,
        (phalcom_native_meta::EffectSpec::Pure, EffectSpecV1::Pure) => true,
        (phalcom_native_meta::EffectSpec::Known(effects), EffectSpecV1::Known { effects: wire_effects }) => {
            if effects.len() != wire_effects.len() {
                return false;
            }
            effects.iter().zip(wire_effects.iter()).all(|(e, w)| match (e, w) {
                (phalcom_native_meta::NativeEffect::Mutation, NativeEffectV1::Mutation) => true,
                (phalcom_native_meta::NativeEffect::Io, NativeEffectV1::Io) => true,
                (phalcom_native_meta::NativeEffect::Scheduling, NativeEffectV1::Scheduling) => true,
                (phalcom_native_meta::NativeEffect::Reflection, NativeEffectV1::Reflection) => true,
                (phalcom_native_meta::NativeEffect::Nondeterminism, NativeEffectV1::Nondeterminism) => true,
                (phalcom_native_meta::NativeEffect::Blocking, NativeEffectV1::Blocking) => true,
                _ => false,
            })
        }
        _ => false,
    }
}

fn matches_native_raises(spec: phalcom_native_meta::RaisesSpec, wire: &phalcom_compiler_artifact::semantic::world::RaisesSpecV1) -> bool {
    use phalcom_compiler_artifact::semantic::world::RaisesSpecV1;
    match (spec, wire) {
        (phalcom_native_meta::RaisesSpec::Unknown, RaisesSpecV1::Unknown) => true,
        (phalcom_native_meta::RaisesSpec::Known(types), RaisesSpecV1::Known { types: wire_types }) => {
            types.len() == wire_types.len() && types.iter().zip(wire_types.iter()).all(|(t, w)| matches_native_type_expr(t, w))
        }
        _ => false,
    }
}

fn matches_return_flow(spec: phalcom_native_meta::ReturnFlowSpec, wire: &phalcom_compiler_artifact::semantic::world::ReturnFlowV1) -> bool {
    use phalcom_compiler_artifact::semantic::world::ReturnFlowV1;
    use phalcom_native_meta::ReturnFlowSpec;
    match (spec, wire) {
        (ReturnFlowSpec::Value, ReturnFlowV1::Value) => true,
        (ReturnFlowSpec::Receiver, ReturnFlowV1::Receiver) => true,
        (ReturnFlowSpec::Argument(index), ReturnFlowV1::Argument { index: wire_index }) => u64::try_from(index).ok() == Some(*wire_index),
        (ReturnFlowSpec::Never, ReturnFlowV1::Never) => true,
        (ReturnFlowSpec::Unknown, ReturnFlowV1::Unknown) => true,
        _ => false,
    }
}

fn matches_native_lifecycle(spec: phalcom_native_meta::NativeLifecycleSpec, wire: &phalcom_compiler_artifact::semantic::world::NativeLifecycleSpecV1) -> bool {
    spec.since == wire.since.as_deref() && spec.deprecated_since == wire.deprecated_since.as_deref() && spec.replacement == wire.replacement.as_deref()
}

fn import_native_metadata(
    signature: &phalcom_compiler_artifact::semantic::world::CallableSignatureRecordV1,
) -> Result<
    (
        Option<phalcom_native_surface::NativeSurfaceId>,
        phalcom_native_meta::EffectSpec,
        phalcom_native_meta::RaisesSpec,
        phalcom_native_meta::ReturnFlowSpec,
        phalcom_native_meta::NativeLifecycleSpec,
    ),
    ArtifactError,
> {
    use phalcom_compiler_artifact::semantic::world::{EffectSpecV1, RaisesSpecV1};
    if let Some(native_id) = &signature.native_id {
        let owner = phalcom_native_meta::UniverseKey::from_name(&native_id.owner)
            .ok_or_else(|| referential("native surface owner is absent from the trusted catalog"))?;
        let side = match native_id.side {
            0 => phalcom_native_meta::NativeDispatch::Instance,
            1 => phalcom_native_meta::NativeDispatch::Class,
            _ => return Err(referential("native surface dispatch side is invalid")),
        };
        let record = phalcom_native_surface::find_native_surface(owner, side, &native_id.selector)
            .ok_or_else(|| referential("native surface identity is absent from the trusted catalog"))?;
        let expected_key = record.id().0;
        if expected_key.owner != owner || expected_key.side != side || expected_key.selector != native_id.selector.as_ref() {
            return Err(referential("native surface identity does not match its trusted catalog row"));
        }
        if !matches_native_effects(record.effects(), &signature.effects)
            || !matches_native_raises(record.raises(), &signature.raises)
            || !matches_return_flow(record.flow(), &signature.flow)
            || !matches_native_lifecycle(record.lifecycle(), &signature.lifecycle)
        {
            return Err(referential("native signature metadata disagrees with its trusted catalog row"));
        }
        return Ok((Some(record.id()), record.effects(), record.raises(), record.flow(), record.lifecycle()));
    }

    let effects = match &signature.effects {
        EffectSpecV1::Unknown => phalcom_native_meta::EffectSpec::Unknown,
        EffectSpecV1::Pure => phalcom_native_meta::EffectSpec::Pure,
        EffectSpecV1::Known { effects } if effects.is_empty() => phalcom_native_meta::EffectSpec::Known(&[]),
        EffectSpecV1::Known { .. } => return Err(referential("non-catalog native effects require an owned metadata representation")),
    };
    let raises = match &signature.raises {
        RaisesSpecV1::Unknown => phalcom_native_meta::RaisesSpec::Unknown,
        RaisesSpecV1::Known { types } if types.is_empty() => phalcom_native_meta::RaisesSpec::Known(&[]),
        RaisesSpecV1::Known { .. } => return Err(referential("non-catalog raised-type metadata requires an owned representation")),
    };
    if signature.lifecycle.since.is_some() || signature.lifecycle.deprecated_since.is_some() || signature.lifecycle.replacement.is_some() {
        return Err(referential("non-catalog native lifecycle metadata requires an owned representation"));
    }
    Ok((
        None,
        effects,
        raises,
        import_return_flow(&signature.flow)?,
        phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
    ))
}

fn import_type_knowledge_with_allocation_budget(
    knowledge: &phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::types::evidence::TypeKnowledge, ArtifactError> {
    use crate::types::evidence::{EvidenceOrigin, EvidenceSet, EvidenceStatus, TypeKnowledge};
    use phalcom_compiler_artifact::semantic::world::{EvidenceOriginV1, EvidenceStatusV1, TypeKnowledgeV1};

    // Keep a nested import atomic: the caller only pays for evidence that is
    // fully reconstructed, and allocation failure leaves its ledger intact.
    let mut child = allocations.child(allocations.remaining());
    let imported = match knowledge {
        TypeKnowledgeV1::Known {
            ty,
            status,
            origin,
            provenance,
        } => {
            let ty = relocation.ty(*ty)?;
            let status = match status {
                EvidenceStatusV1::Established => EvidenceStatus::Established,
                EvidenceStatusV1::Assumed => EvidenceStatus::Assumed,
            };
            let origin = match origin {
                EvidenceOriginV1::Syntax => EvidenceOrigin::Syntax,
                EvidenceOriginV1::DeclarationSemantics => EvidenceOrigin::DeclarationSemantics,
                EvidenceOriginV1::ConstructorSemantics => EvidenceOrigin::ConstructorSemantics,
                EvidenceOriginV1::CallableSignature => EvidenceOrigin::CallableSignature,
                EvidenceOriginV1::NativeSignature => EvidenceOrigin::NativeSignature,
                EvidenceOriginV1::DeveloperAnnotation => EvidenceOrigin::DeveloperAnnotation,
                EvidenceOriginV1::GenericInference => EvidenceOrigin::GenericInference,
                EvidenceOriginV1::Flow => EvidenceOrigin::Flow,
                EvidenceOriginV1::FieldLifecycle => EvidenceOrigin::FieldLifecycle,
                EvidenceOriginV1::ContextualDerivation => EvidenceOrigin::ContextualDerivation,
                EvidenceOriginV1::PatternDecomposition => EvidenceOrigin::PatternDecomposition,
            };
            let mut ranges = child
                .vector::<phalcom_common::range::SourceRange>(provenance.ranges.len())
                .map_err(allocation_error)?;
            for range in &provenance.ranges {
                if range.start > range.end {
                    return Err(referential("inferred type evidence span is inverted"));
                }
                ranges.push(phalcom_common::range::SourceRange {
                    start: usize::try_from(range.start).map_err(|_| referential("inferred type evidence start exceeds address space"))?,
                    end: usize::try_from(range.end).map_err(|_| referential("inferred type evidence end exceeds address space"))?,
                });
            }
            let mut descriptions = child.vector::<String>(provenance.descriptions.len()).map_err(allocation_error)?;
            for description in &provenance.descriptions {
                descriptions.push(child.string(description).map_err(allocation_error)?);
            }
            let provenance = EvidenceSet { ranges, descriptions };
            TypeKnowledge::known_with_evidence(ty, status, origin, provenance)
        }
        TypeKnowledgeV1::Unknown { reason } => TypeKnowledge::Unknown(import_unknown_reason_with_allocation_budget(reason, &mut child)?),
        TypeKnowledgeV1::Dynamic { reason } => TypeKnowledge::Dynamic(match reason {
            DynamicReasonV1::ExplicitEscape => DynamicReason::ExplicitEscape,
            DynamicReasonV1::DynamicRestPack => DynamicReason::DynamicRestPack,
            DynamicReasonV1::RuntimeReflection => DynamicReason::RuntimeReflection,
        }),
    };
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(imported)
}

fn import_callable_signature(
    record: &phalcom_compiler_artifact::semantic::world::CallableSignatureRecordV1,
    relocation: &TypeRelocation,
    store: &TypeStore,
    allocations: &mut AllocationBudget,
) -> Result<crate::signature::CallableSemanticSignature, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::{ImplementationKindV1, ParameterContractV1, RestValueContractV1, ReturnContractValidationV1};
    let callable = import_exact_callable_with_allocation_budget(&record.callable, allocations)?;
    let owner = import_stable_declaration_with_allocation_budget(&record.owner, allocations)?;
    let side = match record.side {
        0 => DispatchSide::Instance,
        1 => DispatchSide::Class,
        _ => return Err(referential("callable signature dispatch side is invalid")),
    };
    let generics = record
        .generics
        .as_ref()
        .map(|generics| import_generic_signature_with_allocation_budget(generics, relocation, store, allocations))
        .transpose()?;
    let mut parameters = allocations.vector(record.parameters.len()).map_err(allocation_error)?;
    for parameter in &record.parameters {
        let parameter_callable = import_exact_callable_with_allocation_budget(&parameter.callable, allocations)?;
        let declared_type = import_declared_type_fact_with_allocation_budget(&parameter.declared_type, relocation, allocations)?;
        let rest = match parameter.rest_mode {
            0 => phalcom_ast::ast::RestMode::None,
            1 => phalcom_ast::ast::RestMode::Positional,
            2 => phalcom_ast::ast::RestMode::Labeled,
            3 => phalcom_ast::ast::RestMode::Complete,
            _ => return Err(referential("callable parameter rest mode is invalid")),
        };
        let contract = match &parameter.contract {
            ParameterContractV1::Fixed { fact } => {
                crate::signature::ParameterContract::Fixed(import_declared_type_fact_with_allocation_budget(fact, relocation, allocations)?)
            }
            ParameterContractV1::Rest { mode, contract } => {
                let mode = match mode {
                    1 => phalcom_ast::ast::RestMode::Positional,
                    2 => phalcom_ast::ast::RestMode::Labeled,
                    3 => phalcom_ast::ast::RestMode::Complete,
                    _ => return Err(referential("callable rest contract mode is invalid")),
                };
                crate::signature::ParameterContract::Rest {
                    mode,
                    contract: match contract {
                        RestValueContractV1::Unconstrained => crate::signature::RestValueContract::Unconstrained,
                        RestValueContractV1::Element { fact } => {
                            crate::signature::RestValueContract::Element(import_declared_type_fact_with_allocation_budget(fact, relocation, allocations)?)
                        }
                        RestValueContractV1::Lane { ty } => crate::signature::RestValueContract::Lane(relocation.ty(*ty)?),
                        RestValueContractV1::Invalid { reason } => {
                            crate::signature::RestValueContract::Invalid(import_unknown_reason_with_allocation_budget(reason, allocations)?)
                        }
                    },
                }
            }
        };
        let source = parameter
            .source
            .as_ref()
            .map(|source| import_source_span_with_allocation_budget(source, allocations))
            .transpose()?;
        parameters.push(crate::signature::CallableParameterSemantic {
            id: crate::identity::CallableParameterId::new(parameter_callable, parameter.index),
            local_name: allocations.boxed_str(&parameter.local_name).map_err(allocation_error)?,
            external_label: parameter
                .external_label
                .as_deref()
                .map(|label| allocations.boxed_str(label).map_err(allocation_error))
                .transpose()?,
            rest,
            declared_type,
            contract,
            source,
        });
    }
    let declared_return = import_declared_type_fact_with_allocation_budget(&record.declared_return, relocation, allocations)?;
    let return_validation = match &record.return_validation {
        ReturnContractValidationV1::NotApplicable => crate::signature::ReturnContractValidation::NotApplicable,
        ReturnContractValidationV1::Unchecked => crate::signature::ReturnContractValidation::Unchecked,
        ReturnContractValidationV1::Satisfied { status } => crate::signature::ReturnContractValidation::Satisfied(match status {
            phalcom_compiler_artifact::semantic::world::EvidenceStatusV1::Established => crate::types::evidence::EvidenceStatus::Established,
            phalcom_compiler_artifact::semantic::world::EvidenceStatusV1::Assumed => crate::types::evidence::EvidenceStatus::Assumed,
        }),
        ReturnContractValidationV1::Refuted => crate::signature::ReturnContractValidation::Refuted,
        ReturnContractValidationV1::Blocked => crate::signature::ReturnContractValidation::Blocked,
        ReturnContractValidationV1::DynamicBoundary => crate::signature::ReturnContractValidation::DynamicBoundary,
    };
    let inferred_return = record
        .inferred_return
        .as_ref()
        .map(|knowledge| import_type_knowledge_with_allocation_budget(knowledge, relocation, allocations))
        .transpose()?;
    let source = record
        .source
        .as_ref()
        .map(|source| import_source_span_with_allocation_budget(source, allocations))
        .transpose()?;
    let implementation = match record.implementation {
        ImplementationKindV1::Phalcom => phalcom_common::modifier::ImplementationKind::Phalcom,
        ImplementationKindV1::Native => phalcom_common::modifier::ImplementationKind::Native,
        ImplementationKindV1::Foreign => phalcom_common::modifier::ImplementationKind::Foreign,
        ImplementationKindV1::Intrinsic => phalcom_common::modifier::ImplementationKind::Intrinsic,
    };
    let (native_id, effects, raises, flow, lifecycle) = import_native_metadata(record)?;
    let selector = import_stable_selector_with_allocation_budget(&record.callable.selector, allocations)?;
    let parameters_boxed = allocations.boxed_slice(parameters).map_err(allocation_error)?;
    Ok(crate::signature::CallableSemanticSignature {
        callable,
        owner,
        side,
        selector,
        generics,
        parameters: parameters_boxed,
        declared_return,
        return_validation,
        inferred_return,
        source,
        implementation,
        native_id,
        effects,
        raises,
        flow,
        lifecycle,
    })
}

fn import_callable_definition(
    record: &phalcom_compiler_artifact::semantic::world::CallableDefinitionRecordV1,
    relocation: &TypeRelocation,
    store: &TypeStore,
    allocations: &mut AllocationBudget,
) -> Result<crate::impls::EffectiveCallableDefinition, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::{CallableDefinitionBodyProvenanceV1, CallableDefinitionOriginV1};

    let callable = import_stable_callable_with_allocation_budget(&record.callable, allocations)?;
    let origin = match &record.origin {
        CallableDefinitionOriginV1::PrimaryDeclaration => crate::impls::CallableDefinitionOrigin::PrimaryDeclaration,
        CallableDefinitionOriginV1::InherentImpl { impl_id } => {
            crate::impls::CallableDefinitionOrigin::InherentImpl(import_stable_impl_with_allocation_budget(impl_id, allocations)?)
        }
        CallableDefinitionOriginV1::ConformanceWitness { impl_id } => {
            crate::impls::CallableDefinitionOrigin::ConformanceWitness(import_stable_impl_with_allocation_budget(impl_id, allocations)?)
        }
    };
    let signature = import_callable_signature(&record.signature, relocation, store, allocations)?;
    if signature.callable != callable {
        return Err(referential("callable definition row key disagrees with its signature"));
    }
    let body_provenance = match &record.body_provenance {
        CallableDefinitionBodyProvenanceV1::SourceBody => crate::impls::CallableDefinitionBodyProvenance::SourceBody,
        CallableDefinitionBodyProvenanceV1::GeneratedDelegation { target_field } => crate::impls::CallableDefinitionBodyProvenance::GeneratedDelegation {
            target_field: allocations.string(target_field).map_err(allocation_error)?.into(),
        },
    };
    let inherent_body_context = record
        .inherent_body_context
        .as_ref()
        .map(|context| -> Result<_, ArtifactError> {
            let generic_signature = context
                .generic_signature
                .as_ref()
                .map(|signature| import_generic_signature_with_allocation_budget(signature, relocation, store, allocations))
                .transpose()?;
            let covering = context
                .covering
                .as_ref()
                .map(|covering| -> Result<_, ArtifactError> {
                    let mut impl_to_decl = allocations.vector(covering.impl_to_decl.len()).map_err(allocation_error)?;
                    for binding in &covering.impl_to_decl {
                        let implementation = relocation.parameter(binding.impl_parameter)?;
                        let declaration = relocation.parameter(binding.declaration_parameter)?;
                        impl_to_decl.push((implementation, declaration));
                    }
                    let mut decl_to_impl = allocations.vector(covering.decl_to_impl.len()).map_err(allocation_error)?;
                    for binding in &covering.decl_to_impl {
                        let implementation = relocation.parameter(binding.impl_parameter)?;
                        let declaration = relocation.parameter(binding.declaration_parameter)?;
                        decl_to_impl.push((declaration, implementation));
                    }
                    Ok(crate::impls::CoveringImplSubstitution {
                        impl_to_decl: crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(impl_to_decl, allocations)
                            .map_err(frozen_ordered_map_error)?,
                        decl_to_impl: crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(decl_to_impl, allocations)
                            .map_err(frozen_ordered_map_error)?,
                    })
                })
                .transpose()?;
            Ok(crate::impls::InherentBodyContext {
                generic_signature,
                covering,
                source_target_head: relocation.ty(context.source_target_head)?,
            })
        })
        .transpose()?;
    let source_member_index = usize::try_from(record.source_member_index).map_err(|_| referential("callable source member index exceeds address space"))?;
    Ok(crate::impls::EffectiveCallableDefinition {
        callable,
        origin,
        source_member_index,
        signature,
        body_provenance,
        inherent_body_context,
    })
}

fn import_enum_requirements(
    records: &[phalcom_compiler_artifact::semantic::world::EnumRequirementOwnerV1],
    enum_semantics: &EnumSemanticTable,
    relocation: &TypeRelocation,
    store: &TypeStore,
    allocations: &mut AllocationBudget,
) -> Result<EnumRequirementTable, ArtifactError> {
    let mut requirement_rows = allocations
        .vector::<(DeclarationId, EnumRequirementRows<EnumRequirement>)>(records.len())
        .map_err(allocation_error)?;
    let mut status_rows = allocations
        .vector::<(DeclarationId, EnumRequirementRows<CaseRequirementResult>)>(records.len())
        .map_err(allocation_error)?;
    for owner_record in records {
        let owner = import_stable_declaration_with_allocation_budget(&owner_record.owner, allocations)?;
        let enum_info = enum_semantics
            .enum_info(&owner)
            .ok_or_else(|| referential("enum requirement owner has no reconstructed enum authority"))?;
        let mut ordered_requirements = allocations
            .vector::<Option<EnumRequirement>>(owner_record.requirements.len())
            .map_err(allocation_error)?;
        ordered_requirements.resize_with(owner_record.requirements.len(), || None);

        for record in &owner_record.requirements {
            let source_order = usize::try_from(record.source_order).map_err(|_| referential("enum requirement source order exceeds address space"))?;
            let slot = ordered_requirements
                .get_mut(source_order)
                .ok_or_else(|| referential("enum requirement source order is outside its owner table"))?;
            if slot.is_some() {
                return Err(referential("enum requirement source order is duplicated"));
            }
            let callable = import_exact_callable_with_allocation_budget(&record.requirement, allocations)?;
            let crate::identity::CallableOwnerId::Declaration(callable_owner) = &callable.owner else {
                return Err(referential("enum root requirement is not declaration-owned"));
            };
            if callable_owner != &owner || callable.side != DispatchSide::Instance {
                return Err(referential("enum root requirement callable identity disagrees with its enum owner"));
            }
            let signature = import_callable_signature(&record.signature, relocation, store, allocations)?;
            if signature.callable != callable || signature.owner != owner || signature.side != DispatchSide::Instance || signature.selector != callable.selector
            {
                return Err(referential("inline enum requirement signature disagrees with its exact owner"));
            }
            let source = record
                .source
                .as_ref()
                .map(|source| -> Result<_, ArtifactError> {
                    let module = import_stable_module_with_allocation_budget(&source.module, allocations)?;
                    if module != owner.module || source.range.start > source.range.end {
                        return Err(referential("enum requirement source span disagrees with its enum owner"));
                    }
                    Ok(SemanticSourceSpan::new(
                        module,
                        phalcom_common::range::SourceRange {
                            start: usize::try_from(source.range.start).map_err(|_| referential("enum requirement source start exceeds address space"))?,
                            end: usize::try_from(source.range.end).map_err(|_| referential("enum requirement source end exceeds address space"))?,
                        },
                    ))
                })
                .transpose()?;
            *slot = Some(EnumRequirement {
                id: EnumRequirementId::new(
                    owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                    crate::traits::clone_selector_with_allocation_budget(&callable.selector, allocations).map_err(allocation_error)?,
                ),
                signature,
                source,
            });
        }

        let mut requirement_values = allocations
            .vector::<EnumRequirement>(owner_record.requirements.len())
            .map_err(allocation_error)?;
        for requirement in ordered_requirements {
            requirement_values.push(requirement.ok_or_else(|| referential("enum requirement source order has a gap"))?);
        }

        let expected_status_count = requirement_values
            .len()
            .checked_mul(enum_info.variants.len())
            .ok_or_else(|| referential("enum requirement case status count overflow"))?;
        if owner_record.case_statuses.len() != expected_status_count {
            return Err(referential("enum requirement case status count disagrees with owner products"));
        }
        let mut statuses = allocations
            .vector::<CaseRequirementResult>(owner_record.case_statuses.len())
            .map_err(allocation_error)?;
        if !enum_info.variants.is_empty() {
            for (index, record) in owner_record.case_statuses.iter().enumerate() {
                let requirement_order = index / enum_info.variants.len();
                let variant_order = index % enum_info.variants.len();
                let requirement = requirement_values
                    .get(requirement_order)
                    .ok_or_else(|| referential("enum case status requirement order is missing"))?;
                let expected_requirement = owner_record
                    .requirements
                    .iter()
                    .find(|candidate| candidate.source_order as usize == requirement_order)
                    .ok_or_else(|| referential("enum case status requirement source order is missing"))?;
                let variant = import_stable_variant_with_allocation_budget(&record.variant, allocations)?;
                if record.requirement_order as usize != requirement_order
                    || record.variant_order as usize != variant_order
                    || record.requirement != expected_requirement.requirement
                    || variant != enum_info.variants[variant_order]
                {
                    return Err(referential("enum case status order or owner identity disagrees with enum semantics"));
                }
                let status = match &record.status {
                    phalcom_compiler_artifact::semantic::world::CaseRequirementStatusV1::Satisfied { implementation } => {
                        let implementation = import_exact_callable_with_allocation_budget(implementation, allocations)?;
                        if !matches!(&implementation.owner, CallableOwnerId::Variant(v) if v == &variant)
                            || implementation.side != DispatchSide::Instance
                            || implementation.selector != requirement.id.selector
                        {
                            return Err(referential("satisfied enum requirement implementation has the wrong case or selector"));
                        }
                        CaseRequirementStatus::Satisfied { implementation }
                    }
                    phalcom_compiler_artifact::semantic::world::CaseRequirementStatusV1::Missing => CaseRequirementStatus::Missing,
                    phalcom_compiler_artifact::semantic::world::CaseRequirementStatusV1::Incompatible { implementation } => {
                        let implementation = import_exact_callable_with_allocation_budget(implementation, allocations)?;
                        if !matches!(&implementation.owner, CallableOwnerId::Variant(v) if v == &variant)
                            || implementation.side != DispatchSide::Instance
                            || implementation.selector != requirement.id.selector
                        {
                            return Err(referential("incompatible enum requirement implementation has the wrong case or selector"));
                        }
                        CaseRequirementStatus::Incompatible { implementation }
                    }
                    phalcom_compiler_artifact::semantic::world::CaseRequirementStatusV1::Blocked => CaseRequirementStatus::Blocked,
                };
                statuses.push(CaseRequirementResult {
                    variant,
                    requirement: requirement.id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                    status,
                });
            }
        }

        let requirement_values = EnumRequirementRows::from_budgeted(allocations.arc(requirement_values).map_err(allocation_error)?);
        let statuses = EnumRequirementRows::from_budgeted(allocations.arc(statuses).map_err(allocation_error)?);
        requirement_rows.push((owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?, requirement_values));
        status_rows.push((owner, statuses));
    }

    let requirements_by_owner = {
        let mut builder = allocations
            .sorted_map_builder::<DeclarationId, EnumRequirementRows<EnumRequirement>>(records.len())
            .map_err(allocation_error)?;
        for (owner, requirements) in requirement_rows {
            builder.push(owner, requirements).map_err(allocation_error)?;
        }
        builder.finish().map_err(|_| referential("duplicate enum requirement owner row"))?
    };
    let requirements = allocations.arc(requirements_by_owner).map_err(allocation_error)?;

    let case_statuses_by_owner = {
        let mut builder = allocations
            .sorted_map_builder::<DeclarationId, EnumRequirementRows<CaseRequirementResult>>(records.len())
            .map_err(allocation_error)?;
        for (owner, statuses) in status_rows {
            builder.push(owner, statuses).map_err(allocation_error)?;
        }
        builder.finish().map_err(|_| referential("duplicate enum requirement owner row"))?
    };
    let case_statuses = allocations.arc(case_statuses_by_owner).map_err(allocation_error)?;
    EnumRequirementTable::from_frozen(requirements, case_statuses, allocations).map_err(allocation_error)
}

fn import_type_member_domain(domain: &TypeMemberDomainV1, relocation: &TypeRelocation) -> Result<crate::types::type_member::TypeMemberDomain, ArtifactError> {
    Ok(match domain {
        TypeMemberDomainV1::Kind { kind } => crate::types::type_member::TypeMemberDomain::Kind(relocation.kind(*kind)?),
        TypeMemberDomainV1::ValueType { ty } => crate::types::type_member::TypeMemberDomain::ValueType(relocation.ty(*ty)?),
    })
}

fn import_intrinsic_type_operation(operation: &IntrinsicTypeOperationV1) -> crate::types::type_member::IntrinsicTypeOperation {
    use crate::types::type_member::IntrinsicTypeOperation as Native;
    match operation {
        IntrinsicTypeOperationV1::TuplePositionals => Native::TuplePositionals,
        IntrinsicTypeOperationV1::TupleLabeled => Native::TupleLabeled,
        IntrinsicTypeOperationV1::TupleEntries => Native::TupleEntries,
        IntrinsicTypeOperationV1::TupleElement => Native::TupleElement,
        IntrinsicTypeOperationV1::TupleAt => Native::TupleAt,
        IntrinsicTypeOperationV1::TupleMap => Native::TupleMap,
        IntrinsicTypeOperationV1::RecordFields => Native::RecordFields,
        IntrinsicTypeOperationV1::RecordValue => Native::RecordValue,
        IntrinsicTypeOperationV1::RecordMap => Native::RecordMap,
        IntrinsicTypeOperationV1::TypePackMap => Native::TypePackMap,
        IntrinsicTypeOperationV1::LabeledTypePackMap => Native::LabeledTypePackMap,
        IntrinsicTypeOperationV1::RecordRowMap => Native::RecordRowMap,
        IntrinsicTypeOperationV1::CompleteTypePackPositionals => Native::CompleteTypePackPositionals,
        IntrinsicTypeOperationV1::CompleteTypePackLabeled => Native::CompleteTypePackLabeled,
        IntrinsicTypeOperationV1::CompleteTypePackMap => Native::CompleteTypePackMap,
        IntrinsicTypeOperationV1::UnorderedShapeLookup => Native::UnorderedShapeLookup,
        IntrinsicTypeOperationV1::UnorderedShapeView => Native::UnorderedShapeView,
        IntrinsicTypeOperationV1::UnorderedShapeTail => Native::UnorderedShapeTail,
    }
}

fn import_type_member_implementation(implementation: &TypeMemberImplementationV1) -> phalcom_common::modifier::ImplementationKind {
    match implementation {
        TypeMemberImplementationV1::Phalcom => phalcom_common::modifier::ImplementationKind::Phalcom,
        TypeMemberImplementationV1::Native => phalcom_common::modifier::ImplementationKind::Native,
        TypeMemberImplementationV1::Foreign => phalcom_common::modifier::ImplementationKind::Foreign,
        TypeMemberImplementationV1::Intrinsic => phalcom_common::modifier::ImplementationKind::Intrinsic,
    }
}

fn import_associated_family_kind(kind: &AssociatedFamilyKindV1) -> crate::associated::AssociatedFamilyKind {
    match kind {
        AssociatedFamilyKindV1::Variant => crate::associated::AssociatedFamilyKind::Variant,
        AssociatedFamilyKindV1::DataConstructor => crate::associated::AssociatedFamilyKind::DataConstructor,
        AssociatedFamilyKindV1::StaticDomain => crate::associated::AssociatedFamilyKind::StaticDomain,
        AssociatedFamilyKindV1::TypeMember => crate::associated::AssociatedFamilyKind::TypeMember,
        AssociatedFamilyKindV1::IntrinsicTypeMember => crate::associated::AssociatedFamilyKind::IntrinsicTypeMember,
    }
}

fn import_associated_static_domain_declaration(
    declaration: &phalcom_type_meta::identity::StableAssociatedStaticDomainDeclarationRef,
) -> Result<crate::identity::AssociatedStaticDomainDeclarationId, ArtifactError> {
    Ok(crate::identity::AssociatedStaticDomainDeclarationId::new(
        from_stable_declaration(&declaration.owner).map_err(referential)?,
        declaration.name.clone(),
    ))
}

fn import_associated_static_domain_declaration_with_allocation_budget(
    declaration: &phalcom_type_meta::identity::StableAssociatedStaticDomainDeclarationRef,
    allocations: &mut AllocationBudget,
) -> Result<crate::identity::AssociatedStaticDomainDeclarationId, ArtifactError> {
    Ok(crate::identity::AssociatedStaticDomainDeclarationId::new(
        import_stable_declaration_with_allocation_budget(&declaration.owner, allocations)?,
        allocations.boxed_str(&declaration.name).map_err(allocation_error)?,
    ))
}

fn import_static_case_domain_with_allocation_budget(
    domain: &phalcom_compiler_artifact::semantic::world::StaticCaseDomainV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::identity::StaticDomainDeclarationId, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::StaticCaseDomainV1;
    Ok(match domain {
        StaticCaseDomainV1::Nominal { declaration } => {
            crate::identity::StaticDomainDeclarationId::Nominal(import_stable_declaration_with_allocation_budget(declaration, allocations)?)
        }
        StaticCaseDomainV1::Associated { declaration } => crate::identity::StaticDomainDeclarationId::Associated(
            import_associated_static_domain_declaration_with_allocation_budget(declaration, allocations)?,
        ),
    })
}

pub(crate) fn import_static_case_domain(
    domain: &phalcom_compiler_artifact::semantic::world::StaticCaseDomainV1,
) -> Result<crate::identity::StaticDomainDeclarationId, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::StaticCaseDomainV1 as Portable;
    Ok(match domain {
        Portable::Nominal { declaration } => crate::identity::StaticDomainDeclarationId::Nominal(from_stable_declaration(declaration).map_err(referential)?),
        Portable::Associated { declaration } => {
            crate::identity::StaticDomainDeclarationId::Associated(import_associated_static_domain_declaration(declaration)?)
        }
    })
}

pub(crate) fn import_exact_selector_with_budget(
    selector: &phalcom_compiler_artifact::semantic::world::ExactSelectorV1,
    allocations: &mut AllocationBudget,
) -> Result<phalcom_common::selector::Selector, ArtifactError> {
    let base = import_selector_base_with_allocation_budget(&selector.base, allocations)?;
    let mut slots = allocations
        .vector::<phalcom_common::selector::SelectorSlot>(selector.slots.len())
        .map_err(allocation_error)?;
    for slot in &selector.slots {
        slots.push(import_selector_slot_with_allocation_budget(slot, allocations)?);
    }
    phalcom_common::selector::Selector::new(
        base,
        import_selector_kind(&selector.kind),
        allocations.boxed_slice(slots).map_err(allocation_error)?,
    )
    .map_err(|error| referential(format!("static-case selector is malformed: {error}")))
}

fn import_static_semantic_dependency(
    dependency: &phalcom_compiler_artifact::semantic::world::StaticSemanticDependencyV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::checker::analysis::SemanticDependency, ArtifactError> {
    use crate::checker::analysis::SemanticDependency as Dependency;
    use phalcom_compiler_artifact::semantic::world::{ConformanceTargetV1, StaticSemanticDependencyV1 as Portable};
    fn import_domain(
        domain: &phalcom_compiler_artifact::semantic::world::StaticCaseDomainV1,
        allocations: &mut AllocationBudget,
    ) -> Result<crate::identity::StaticDomainDeclarationId, ArtifactError> {
        use phalcom_compiler_artifact::semantic::world::StaticCaseDomainV1;
        Ok(match domain {
            StaticCaseDomainV1::Nominal { declaration } => {
                crate::identity::StaticDomainDeclarationId::Nominal(import_stable_declaration_with_allocation_budget(declaration, allocations)?)
            }
            StaticCaseDomainV1::Associated { declaration } => {
                crate::identity::StaticDomainDeclarationId::Associated(crate::identity::AssociatedStaticDomainDeclarationId::new(
                    import_stable_declaration_with_allocation_budget(&declaration.owner, allocations)?,
                    allocations.boxed_str(&declaration.name).map_err(allocation_error)?,
                ))
            }
        })
    }
    Ok(match dependency {
        Portable::DeclarationShell { owner } => Dependency::DeclarationShell(import_stable_declaration_with_allocation_budget(owner, allocations)?),
        Portable::TraitHeader { owner } => Dependency::TraitHeader(import_stable_declaration_with_allocation_budget(owner, allocations)?),
        Portable::CallableSignature { callable } => Dependency::CallableSignature(import_stable_callable_with_allocation_budget(callable, allocations)?),
        Portable::FieldSignature { field } => Dependency::FieldSignature(crate::identity::FieldId::new(
            import_stable_declaration_with_allocation_budget(&field.owner, allocations)?,
            allocations.boxed_str(&field.name).map_err(allocation_error)?,
            from_stable_dispatch_side(field.side),
        )),
        Portable::DeclarationSurface { owner } => Dependency::DeclarationSurface(import_stable_declaration_with_allocation_budget(owner, allocations)?),
        Portable::TraitSurface { owner } => Dependency::TraitSurface(import_stable_declaration_with_allocation_budget(owner, allocations)?),
        Portable::ConformanceDispatch { implementation } => {
            Dependency::ConformanceDispatch(import_stable_impl_with_allocation_budget(implementation, allocations)?)
        }
        Portable::TraitDispatchBucket { target, selector, side } => {
            let target_family = match target {
                ConformanceTargetV1::Declaration { declaration } => {
                    crate::trait_dispatch::TraitDispatchTargetFamily::Declaration(import_stable_declaration_with_allocation_budget(declaration, allocations)?)
                }
                ConformanceTargetV1::ExactEnumCase { variant } => {
                    crate::trait_dispatch::TraitDispatchTargetFamily::ExactEnumCase(import_stable_variant_with_allocation_budget(variant, allocations)?)
                }
            };
            Dependency::TraitDispatchBucket(crate::trait_dispatch::TraitDispatchBucketKey {
                target_family,
                selector: import_exact_selector_with_budget(selector, allocations)?,
                side: from_stable_dispatch_side(*side),
            })
        }
        Portable::GenericConformance { subject, trait_ref } => {
            let subject = import_type_term_with_allocation_budget(subject, relocation, allocations)?;
            let trait_ref = import_trait_ref_identity(trait_ref, relocation, allocations)?;
            Dependency::GenericConformance(crate::generic_evidence::GenericConformanceKey::new(subject, trait_ref))
        }
        Portable::HierarchyEdge { owner } => Dependency::HierarchyEdge(import_stable_declaration_with_allocation_budget(owner, allocations)?),
        Portable::LinkedInterface { module } => Dependency::LinkedInterface(import_stable_module_with_allocation_budget(module, allocations)?),
        Portable::DataDeclaration { owner } => Dependency::DataDeclaration(import_stable_declaration_with_allocation_budget(owner, allocations)?),
        Portable::EnumDeclaration { owner } => Dependency::EnumDeclaration(import_stable_declaration_with_allocation_budget(owner, allocations)?),
        Portable::AssociatedSurface { owner } => Dependency::AssociatedSurface(import_stable_declaration_with_allocation_budget(owner, allocations)?),
        Portable::TypeMemberDeclaration { owner, selector } => Dependency::TypeMemberDeclaration(crate::types::type_member::TypeMemberDeclarationId::new(
            import_stable_declaration_with_allocation_budget(owner, allocations)?,
            import_type_member_selector_with_allocation_budget(selector, allocations)?,
        )),
        Portable::StaticDomainCaseCatalog { domain } => Dependency::StaticDomainCaseCatalog(import_domain(domain, allocations)?),
        Portable::StaticCaseSignature { domain, selector } => Dependency::StaticCaseSignature(crate::identity::StaticCaseId::with_selector(
            import_domain(domain, allocations)?,
            import_exact_selector_with_budget(selector, allocations)?,
        )),
        Portable::StaticDefinition { owner } => Dependency::StaticDefinition(import_stable_declaration_with_allocation_budget(owner, allocations)?),
        Portable::StaticConstant { symbol } => Dependency::StaticConstant(crate::static_constant::StaticConstantId(phalcom_modules::linker::SymbolId {
            module: import_stable_module_with_allocation_budget(&symbol.module, allocations)?,
            name: allocations.boxed_str(&symbol.name).map_err(allocation_error)?,
        })),
        Portable::ModuleStaticBindingCatalog { module } => {
            Dependency::ModuleStaticBindingCatalog(import_stable_module_with_allocation_budget(module, allocations)?)
        }
        Portable::ConstructorInitialization { owner } => {
            Dependency::ConstructorInitialization(import_stable_declaration_with_allocation_budget(owner, allocations)?)
        }
        Portable::ResolvedImport { module, local_id } => Dependency::ResolvedImport(phalcom_modules::identity::ImportSiteId::new(
            import_stable_module_with_allocation_budget(module, allocations)?,
            phalcom_modules::identity::ImportSiteLocalId(*local_id),
        )),
        Portable::LinkedName { module, name } => Dependency::LinkedName(
            import_stable_module_with_allocation_budget(module, allocations)?,
            allocations.string(name).map_err(allocation_error)?,
        ),
        Portable::PublicExport { module, name } => Dependency::PublicExport(
            import_stable_module_with_allocation_budget(module, allocations)?,
            allocations.string(name).map_err(allocation_error)?,
        ),
        Portable::ModuleStaticBindings { module } => Dependency::ModuleStaticBindings(import_stable_module_with_allocation_budget(module, allocations)?),
    })
}

fn import_static_case_owners(
    catalogs: &[phalcom_compiler_artifact::semantic::world::StaticDomainCaseCatalogRecordV1],
    signatures: &[phalcom_compiler_artifact::semantic::world::StaticCaseSignatureRecordV1],
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<
    (
        crate::ordered_map::OrderedMap<crate::identity::StaticDomainDeclarationId, Arc<crate::static_case::StaticDomainCaseCatalogProduct>>,
        crate::ordered_map::OrderedMap<crate::identity::StaticCaseId, Arc<crate::static_case::StaticCaseSignatureProduct>>,
    ),
    ArtifactError,
> {
    use phalcom_compiler_artifact::semantic::world::StaticCaseShapeV1 as Shape;
    let mut catalog_entries = allocations.vector(catalogs.len()).map_err(allocation_error)?;
    for record in catalogs {
        let domain = import_static_case_domain_with_allocation_budget(&record.domain, allocations)?;
        let mut cases = allocations
            .vector::<crate::static_case::StaticCaseHeader>(record.cases.len())
            .map_err(allocation_error)?;
        for record_case in &record.cases {
            let selector = import_exact_selector_with_budget(&record_case.selector, allocations)?;
            let shape = match &record_case.shape {
                Shape::Singleton => crate::static_case::StaticCaseShape::Singleton,
                Shape::Constructor { slots } => {
                    let mut imported_slots = allocations
                        .vector::<phalcom_common::selector::SelectorSlot>(slots.len())
                        .map_err(allocation_error)?;
                    for slot in slots {
                        imported_slots.push(import_selector_slot_with_allocation_budget(slot, allocations)?);
                    }
                    crate::static_case::StaticCaseShape::Constructor {
                        slots: allocations.boxed_slice(imported_slots).map_err(allocation_error)?,
                    }
                }
            };
            let source_range = phalcom_common::range::SourceRange {
                start: usize::try_from(record_case.source.start).map_err(|_| referential("static-case source start exceeds address space"))?,
                end: usize::try_from(record_case.source.end).map_err(|_| referential("static-case source end exceeds address space"))?,
            };
            cases.push(crate::static_case::StaticCaseHeader {
                id: crate::identity::StaticCaseId::with_selector(import_static_case_domain_with_allocation_budget(&record.domain, allocations)?, selector),
                shape,
                source_range,
            });
        }
        let product = crate::static_case::StaticDomainCaseCatalogProduct {
            domain: import_static_case_domain_with_allocation_budget(&record.domain, allocations)?,
            cases: allocations.boxed_slice(cases).map_err(allocation_error)?,
        };
        let product = allocations.arc(product).map_err(allocation_error)?;
        catalog_entries.push((domain, product));
    }

    let mut signature_entries = allocations.vector(signatures.len()).map_err(allocation_error)?;
    for record in signatures {
        let domain = import_static_case_domain_with_allocation_budget(&record.domain, allocations)?;
        let selector = import_exact_selector_with_budget(&record.selector, allocations)?;
        let id = crate::identity::StaticCaseId::with_selector(domain, selector);
        let mut parameter_kinds = allocations
            .vector::<crate::types::id::KindId>(record.parameter_kinds.len())
            .map_err(allocation_error)?;
        for &kind in &record.parameter_kinds {
            parameter_kinds.push(relocation.kind(kind)?);
        }
        let source_range = phalcom_common::range::SourceRange {
            start: usize::try_from(record.source.start).map_err(|_| referential("static-case signature source start exceeds address space"))?,
            end: usize::try_from(record.source.end).map_err(|_| referential("static-case signature source end exceeds address space"))?,
        };
        let product = crate::static_case::StaticCaseSignatureProduct {
            id: crate::identity::StaticCaseId::with_selector(
                import_static_case_domain_with_allocation_budget(&record.domain, allocations)?,
                import_exact_selector_with_budget(&record.selector, allocations)?,
            ),
            parameter_kinds: allocations.boxed_slice(parameter_kinds).map_err(allocation_error)?,
            source_range,
        };
        let product = allocations.arc(product).map_err(allocation_error)?;
        signature_entries.push((id, product));
    }
    let imported_catalogs =
        crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(catalog_entries, allocations).map_err(frozen_ordered_map_error)?;
    let imported_signatures =
        crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(signature_entries, allocations).map_err(frozen_ordered_map_error)?;
    Ok((imported_catalogs, imported_signatures))
}

fn import_associated_member(member: &AssociatedMemberV1) -> Result<crate::associated::AssociatedMemberId, ArtifactError> {
    Ok(match member {
        AssociatedMemberV1::Variant { variant } => crate::associated::AssociatedMemberId::Variant(from_stable_variant(variant).map_err(referential)?),
        AssociatedMemberV1::DataConstructor { owner } => crate::associated::AssociatedMemberId::DataConstructor(crate::identity::DataConstructorId::new(
            from_stable_declaration(owner).map_err(referential)?,
        )),
        AssociatedMemberV1::DataComponent { owner, index } => crate::associated::AssociatedMemberId::DataComponent(crate::identity::DataComponentId::new(
            from_stable_declaration(owner).map_err(referential)?,
            *index,
        )),
        AssociatedMemberV1::StaticDomain { declaration } => {
            crate::associated::AssociatedMemberId::StaticDomain(import_associated_static_domain_declaration(declaration)?)
        }
        AssociatedMemberV1::TypeMember { owner, selector } => {
            crate::associated::AssociatedMemberId::TypeMember(crate::types::type_member::TypeMemberDeclarationId::new(
                from_stable_declaration(owner).map_err(referential)?,
                import_type_member_selector(selector),
            ))
        }
    })
}

fn import_selector_kind(kind: &phalcom_compiler_artifact::semantic::world::SelectorKindV1) -> phalcom_common::selector::SelectorKind {
    use phalcom_compiler_artifact::semantic::world::SelectorKindV1 as Portable;
    match kind {
        Portable::Getter => phalcom_common::selector::SelectorKind::Getter,
        Portable::Setter => phalcom_common::selector::SelectorKind::Setter,
        Portable::Method => phalcom_common::selector::SelectorKind::Method,
        Portable::SubscriptGet => phalcom_common::selector::SelectorKind::SubscriptGet,
        Portable::SubscriptSet => phalcom_common::selector::SelectorKind::SubscriptSet,
    }
}

fn import_selector_slot_with_allocation_budget(
    slot: &phalcom_compiler_artifact::semantic::world::SelectorSlotV1,
    allocations: &mut AllocationBudget,
) -> Result<phalcom_common::selector::SelectorSlot, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::SelectorSlotV1 as Portable;
    Ok(match slot {
        Portable::Positional => phalcom_common::selector::SelectorSlot::Positional,
        Portable::Label { label } => phalcom_common::selector::SelectorSlot::Label(allocations.string(label).map_err(allocation_error)?),
    })
}

fn import_selector_base_with_allocation_budget(
    base: &TypeMemberSelectorV1,
    allocations: &mut AllocationBudget,
) -> Result<phalcom_common::selector::SelectorBase, ArtifactError> {
    Ok(match base {
        TypeMemberSelectorV1::Named { name } => phalcom_common::selector::SelectorBase::Named(allocations.string(name).map_err(allocation_error)?),
        TypeMemberSelectorV1::Subscript => phalcom_common::selector::SelectorBase::Subscript,
    })
}

fn import_type_member_selector_with_allocation_budget(
    selector: &TypeMemberSelectorV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::types::type_member::TypeMemberSelector, ArtifactError> {
    Ok(match selector {
        TypeMemberSelectorV1::Named { name } => crate::types::type_member::TypeMemberSelector::Named(allocations.boxed_str(name).map_err(allocation_error)?),
        TypeMemberSelectorV1::Subscript => crate::types::type_member::TypeMemberSelector::Subscript,
    })
}

fn import_family_operation_shape_with_allocation_budget(
    shape: &phalcom_compiler_artifact::semantic::world::FamilyOperationShapeV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::types::family::FamilyOperationShape, ArtifactError> {
    let mut slots = allocations
        .vector::<phalcom_common::selector::SelectorSlot>(shape.slots.len())
        .map_err(allocation_error)?;
    for slot in &shape.slots {
        slots.push(import_selector_slot_with_allocation_budget(slot, allocations)?);
    }
    Ok(crate::types::family::FamilyOperationShape::new(
        import_selector_kind(&shape.kind),
        allocations.boxed_slice(slots).map_err(allocation_error)?,
    ))
}

fn import_behavioral_family_spec_with_allocation_budget(
    spec: &phalcom_compiler_artifact::semantic::world::BehavioralFamilySpecV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::checker::associated::BehavioralFamilySpec, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::BehavioralFamilySpecV1;
    Ok(match spec {
        BehavioralFamilySpecV1::Exact { selector } => {
            let base = import_selector_base_with_allocation_budget(&selector.base, allocations)?;
            let mut slots = allocations
                .vector::<phalcom_common::selector::SelectorSlot>(selector.slots.len())
                .map_err(allocation_error)?;
            for slot in &selector.slots {
                slots.push(import_selector_slot_with_allocation_budget(slot, allocations)?);
            }
            let selector = phalcom_common::selector::Selector::new(
                base,
                import_selector_kind(&selector.kind),
                allocations.boxed_slice(slots).map_err(allocation_error)?,
            )
            .map_err(|_| referential("behavioral exact selector is invalid"))?;
            crate::checker::associated::BehavioralFamilySpec::Exact(selector)
        }
        BehavioralFamilySpecV1::Pattern { pattern } => {
            crate::checker::associated::BehavioralFamilySpec::Pattern(import_selector_pattern_with_allocation_budget(pattern, allocations)?)
        }
    })
}

fn import_selector_pattern_with_allocation_budget(
    pattern: &phalcom_compiler_artifact::semantic::world::SelectorPatternSpecV1,
    allocations: &mut AllocationBudget,
) -> Result<phalcom_common::selector::SelectorPattern, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::SelectorKindPatternV1;
    let base = import_selector_base_with_allocation_budget(&pattern.base, allocations)?;
    let kind = match &pattern.kind {
        SelectorKindPatternV1::AnyNamed => phalcom_common::selector::SelectorKindPattern::AnyNamed,
        SelectorKindPatternV1::NamedAccessors => phalcom_common::selector::SelectorKindPattern::NamedAccessors,
        SelectorKindPatternV1::AnySubscript => phalcom_common::selector::SelectorKindPattern::AnySubscript,
        SelectorKindPatternV1::Exact { kind } => phalcom_common::selector::SelectorKindPattern::Exact(import_selector_kind(kind)),
    };
    let mut prefix = allocations
        .vector::<phalcom_common::selector::SelectorSlot>(pattern.prefix.len())
        .map_err(allocation_error)?;
    for slot in &pattern.prefix {
        prefix.push(import_selector_slot_with_allocation_budget(slot, allocations)?);
    }
    let mut suffix = allocations
        .vector::<phalcom_common::selector::SelectorSlot>(pattern.suffix.len())
        .map_err(allocation_error)?;
    for slot in &pattern.suffix {
        suffix.push(import_selector_slot_with_allocation_budget(slot, allocations)?);
    }
    let pattern = phalcom_common::selector::SelectorPattern::new(
        base,
        kind,
        allocations.boxed_slice(prefix).map_err(allocation_error)?,
        allocations.boxed_slice(suffix).map_err(allocation_error)?,
        pattern.has_gap,
    )
    .map_err(|_| referential("behavioral selector pattern is invalid"))?;
    Ok(pattern)
}

fn import_invocation_target(
    target: &phalcom_compiler_artifact::semantic::world::InvocationTargetV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::identity::InvocationTargetId, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::InvocationTargetV1 as Portable;
    Ok(match target {
        Portable::Behavioral { callable } => {
            crate::identity::InvocationTargetId::Behavioral(import_stable_callable_with_allocation_budget(callable, allocations)?)
        }
        Portable::VariantConstructor { variant } => crate::identity::InvocationTargetId::VariantConstructor(VariantConstructorId::new(
            import_stable_variant_with_allocation_budget(variant, allocations)?,
        )),
        Portable::DataConstructor { owner } => {
            crate::identity::InvocationTargetId::DataConstructor(DataConstructorId::new(import_stable_declaration_with_allocation_budget(owner, allocations)?))
        }
    })
}

fn import_conditional_dispatch_selection(
    selection: &phalcom_compiler_artifact::semantic::world::ConditionalDispatchSelectionV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::dispatch::ConditionalDispatchSelection, ArtifactError> {
    if selection.side > 1 {
        return Err(referential("conditional dispatch selection side is invalid"));
    }
    Ok(crate::dispatch::ConditionalDispatchSelection {
        impl_id: import_stable_impl_with_allocation_budget(&selection.impl_id, allocations)?,
        callable: import_stable_callable_with_allocation_budget(&selection.callable, allocations)?,
        declaring_owner: import_stable_declaration_with_allocation_budget(&selection.declaring_owner, allocations)?,
        side: if selection.side == 0 { DispatchSide::Instance } else { DispatchSide::Class },
    })
}

fn import_associated_member_with_allocation_budget(
    member: &AssociatedMemberV1,
    allocations: &mut AllocationBudget,
) -> Result<crate::associated::AssociatedMemberId, ArtifactError> {
    use AssociatedMemberV1 as Portable;
    Ok(match member {
        Portable::Variant { variant } => crate::associated::AssociatedMemberId::Variant(import_stable_variant_with_allocation_budget(variant, allocations)?),
        Portable::DataConstructor { owner } => crate::associated::AssociatedMemberId::DataConstructor(DataConstructorId::new(
            import_stable_declaration_with_allocation_budget(owner, allocations)?,
        )),
        Portable::DataComponent { owner, index } => crate::associated::AssociatedMemberId::DataComponent(DataComponentId::new(
            import_stable_declaration_with_allocation_budget(owner, allocations)?,
            *index,
        )),
        Portable::StaticDomain { declaration } => {
            crate::associated::AssociatedMemberId::StaticDomain(crate::identity::AssociatedStaticDomainDeclarationId::new(
                import_stable_declaration_with_allocation_budget(&declaration.owner, allocations)?,
                allocations.boxed_str(&declaration.name).map_err(allocation_error)?,
            ))
        }
        Portable::TypeMember { owner, selector } => crate::associated::AssociatedMemberId::TypeMember(crate::types::type_member::TypeMemberDeclarationId::new(
            import_stable_declaration_with_allocation_budget(owner, allocations)?,
            import_type_member_selector_with_allocation_budget(selector, allocations)?,
        )),
    })
}

fn budgeted_arc_slice<T>(values: Vec<T>, allocations: &mut AllocationBudget) -> Result<Arc<[T]>, ArtifactError> {
    use std::alloc::Layout;
    use std::sync::atomic::AtomicUsize;

    if values.capacity() > values.len() {
        allocations.reserve_array::<T>(values.len()).map_err(allocation_error)?;
    }
    let payload = Layout::array::<T>(values.len()).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let layout = Layout::new::<[AtomicUsize; 2]>()
        .extend(payload)
        .map_err(|_| allocation_error(AllocationError::Overflow))?
        .0
        .pad_to_align();
    allocations.reserve_layout(layout).map_err(allocation_error)?;
    Ok(Arc::from(values.into_boxed_slice()))
}

fn import_semantic_denotation_inner(
    denotation: &phalcom_compiler_artifact::semantic::world::SemanticDenotationV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::types::denotation::SemanticDenotation, ArtifactError> {
    use crate::types::denotation::{
        AssociatedValueDenotation, BehavioralFamilyDenotation, CapturedAssociatedMember, CapturedBehavioralMember, SemanticDenotation,
    };
    use phalcom_compiler_artifact::semantic::world::{AssociatedValueDenotationV1, SemanticDenotationV1};
    Ok(match denotation {
        SemanticDenotationV1::TypeForm { ty } => SemanticDenotation::TypeForm(relocation.ty(*ty)?),
        SemanticDenotationV1::Kind { kind } => SemanticDenotation::Kind(relocation.kind(*kind)?),
        SemanticDenotationV1::StaticDomain { domain } => SemanticDenotation::StaticDomain(relocation.domain(*domain)?),
        SemanticDenotationV1::StaticTerm { term } => SemanticDenotation::StaticTerm(relocation.ty(*term)?),
        SemanticDenotationV1::StaticApplication { .. } => {
            return Err(referential("static-application denotation has no retained checked-body application row"));
        }
        SemanticDenotationV1::AssociatedValue { value } => {
            let value = match value {
                AssociatedValueDenotationV1::Exact {
                    owner_form,
                    lookup_owner,
                    member,
                    target,
                } => {
                    let target = target.as_ref().map(|target| import_invocation_target(target, allocations)).transpose()?;
                    AssociatedValueDenotation::Exact {
                        owner_form: relocation.ty(*owner_form)?,
                        lookup_owner: import_stable_declaration_with_allocation_budget(lookup_owner, allocations)?,
                        member: import_associated_member_with_allocation_budget(member, allocations)?,
                        target: allocations.boxed(target).map_err(allocation_error)?,
                    }
                }
                AssociatedValueDenotationV1::Family {
                    owner_form,
                    lookup_owner,
                    family,
                    members,
                } => {
                    let mut captured = allocations.vector::<CapturedAssociatedMember>(members.len()).map_err(allocation_error)?;
                    for member in members {
                        captured.push(CapturedAssociatedMember {
                            operation: import_family_operation_shape_with_allocation_budget(&member.operation, allocations)?,
                            member: import_associated_member_with_allocation_budget(&member.member, allocations)?,
                            target: member.target.as_ref().map(|target| import_invocation_target(target, allocations)).transpose()?,
                        });
                    }
                    let family_owner = import_stable_declaration_with_allocation_budget(&family.owner, allocations)?;
                    let family_base = import_associated_selector_with_allocation_budget(&family.selector, allocations)?;
                    AssociatedValueDenotation::Family {
                        owner_form: relocation.ty(*owner_form)?,
                        lookup_owner: import_stable_declaration_with_allocation_budget(lookup_owner, allocations)?,
                        family: crate::identity::AssociatedFamilyId::new(family_owner, family_base),
                        members: budgeted_arc_slice(captured, allocations)?,
                    }
                }
            };
            SemanticDenotation::AssociatedValue(allocations.boxed(value).map_err(allocation_error)?)
        }
        SemanticDenotationV1::BehavioralFamily { family } => {
            let mut members = allocations.vector::<CapturedBehavioralMember>(family.members.len()).map_err(allocation_error)?;
            for member in &family.members {
                members.push(CapturedBehavioralMember {
                    operation: import_family_operation_shape_with_allocation_budget(&member.operation, allocations)?,
                    target: import_invocation_target(&member.target, allocations)?,
                    conditional: member
                        .conditional
                        .as_ref()
                        .map(|selection| import_conditional_dispatch_selection(selection, allocations))
                        .transpose()?,
                });
            }
            let family = BehavioralFamilyDenotation {
                receiver_type: relocation.ty(family.receiver_type)?,
                receiver_application: family.receiver_application.map(|application| relocation.ty(application)).transpose()?,
                spec: import_behavioral_family_spec_with_allocation_budget(&family.spec, allocations)?,
                members: budgeted_arc_slice(members, allocations)?,
            };
            SemanticDenotation::BehavioralFamily(allocations.boxed(family).map_err(allocation_error)?)
        }
    })
}

/// Imports one denotation using the shared artifact type relocation.
///
/// The caller's ledger is committed only if the full denotation is reconstructed.
pub fn import_semantic_denotation(
    denotation: &phalcom_compiler_artifact::semantic::world::SemanticDenotationV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::types::denotation::SemanticDenotation, ArtifactError> {
    let mut child = allocations.child(allocations.remaining());
    let imported = import_semantic_denotation_inner(denotation, relocation, &mut child)?;
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(imported)
}

/// Reconstructed denotations keyed by callable-body-local ordinals.
#[derive(Debug, Eq, PartialEq)]
pub struct ImportedCallableDenotations {
    /// Denotations owned by expression records, in ascending expression ordinal.
    pub expressions: Vec<(u32, crate::types::denotation::SemanticDenotation)>,
    /// Denotations owned by binding records, in ascending binding ordinal.
    pub bindings: Vec<(u32, crate::types::denotation::SemanticDenotation)>,
}

/// Imports expression/binding denotations using the shared type relocation.
/// Static applications remain rejected until their checked-body rows are retained.
pub fn import_callable_denotations(
    record: &phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCallableDenotations, ArtifactError> {
    let mut child = allocations.child(allocations.remaining());
    let mut expressions = child
        .vector::<(u32, crate::types::denotation::SemanticDenotation)>(record.expressions.len())
        .map_err(allocation_error)?;
    for (index, expression) in record.expressions.iter().enumerate() {
        if expression.ordinal as usize != index {
            return Err(referential("callable expression rows are not dense and ordinal-sorted"));
        }
        if let Some(denotation) = &expression.denotation {
            expressions.push((expression.ordinal, import_semantic_denotation_inner(denotation, relocation, &mut child)?));
        }
    }
    let mut bindings = child
        .vector::<(u32, crate::types::denotation::SemanticDenotation)>(record.bindings.len())
        .map_err(allocation_error)?;
    for (index, binding) in record.bindings.iter().enumerate() {
        if binding.ordinal as usize != index {
            return Err(referential("callable binding rows are not dense and ordinal-sorted"));
        }
        if let Some(denotation) = &binding.denotation {
            bindings.push((binding.ordinal, import_semantic_denotation_inner(denotation, relocation, &mut child)?));
        }
    }
    let imported = ImportedCallableDenotations { expressions, bindings };
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(imported)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportedCallableBindingSourceAuthority {
    Source(SourceSiteId),
    Generated(phalcom_compiler_artifact::semantic::world::CallableGeneratedBindingReasonV1),
    ModuleValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedCallableBindingRow {
    pub state: crate::checker::analysis::BindingState,
    pub storage_origin: crate::checker::analysis::CallableBindingOrigin,
    pub source_authority: ImportedCallableBindingSourceAuthority,
    pub formal_range: Option<SourceRange>,
}

fn import_callable_binding_budget(report: &phalcom_compiler_artifact::semantic::world::CallableBudgetReportV1) -> crate::types::outcome::BudgetReport {
    use phalcom_compiler_artifact::semantic::world::CallableBudgetKindV1 as Kind;
    crate::types::outcome::BudgetReport::new(
        match report.kind {
            Kind::Steps => crate::types::outcome::BudgetKind::Steps,
            Kind::RelationPairs => crate::types::outcome::BudgetKind::RelationPairs,
            Kind::SccIterations => crate::types::outcome::BudgetKind::SccIterations,
            Kind::TypeDepth => crate::types::outcome::BudgetKind::TypeDepth,
            Kind::DiagnosticNotes => crate::types::outcome::BudgetKind::DiagnosticNotes,
            Kind::ResidualNodes => crate::types::outcome::BudgetKind::ResidualNodes,
        },
        report.limit,
        report.used,
    )
}

fn import_callable_block_reason(
    reason: &phalcom_compiler_artifact::semantic::world::CallableBlockReasonV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::types::outcome::BlockReason, ArtifactError> {
    use crate::types::outcome::BlockReason;
    use phalcom_compiler_artifact::semantic::world::CallableBlockReasonV1 as Portable;
    Ok(match reason {
        Portable::UnknownType { reason } => BlockReason::UnknownType(import_unknown_reason_with_allocation_budget(reason, allocations)?),
        Portable::UnresolvedDependency { project_source, module_path } => {
            let project_source = allocations.string(project_source).map_err(allocation_error)?;
            let mut components = allocations
                .vector::<phalcom_modules::ModuleComponent>(module_path.len())
                .map_err(allocation_error)?;
            for component in module_path {
                let component = allocations.boxed_str(component).map_err(allocation_error)?;
                components.push(
                    phalcom_modules::ModuleComponent::from_owned_identifier(component)
                        .map_err(|_| referential("callable unresolved-dependency module path contains an invalid component"))?,
                );
            }
            BlockReason::UnresolvedDependency(crate::identity::StableModuleKey::new(
                phalcom_modules::StableProjectKey::from_path(std::path::PathBuf::from(project_source)),
                phalcom_modules::ModulePath::from_components(allocations.boxed_slice(components).map_err(allocation_error)?),
            ))
        }
        Portable::InvalidAnnotation { diagnostic_code } => BlockReason::InvalidAnnotation(
            DiagnosticCode::from_stable_code(diagnostic_code).ok_or_else(|| referential("callable blocked reason has an unknown diagnostic code"))?,
        ),
        Portable::RecursiveFixpoint => BlockReason::RecursiveFixpoint,
        Portable::InferenceDidNotConverge => BlockReason::InferenceDidNotConverge,
        Portable::OpaqueNative { message } => BlockReason::OpaqueNative(allocations.boxed_str(message).map_err(allocation_error)?),
        Portable::ReflectionBoundary => BlockReason::ReflectionBoundary,
        Portable::UnavailableSemanticOwner { owner } => {
            BlockReason::UnavailableSemanticOwner(Box::new(import_callable_query_owner(owner, relocation, allocations)?))
        }
        Portable::BudgetExceeded { report } => BlockReason::BudgetExceeded(import_callable_binding_budget(report)),
        Portable::SuppressedDependency => BlockReason::SuppressedDependency,
    })
}

fn import_callable_binding_consistency(
    consistency: &phalcom_compiler_artifact::semantic::world::CallableBindingConsistencyV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::checker::binding::BindingConsistency, ArtifactError> {
    use crate::checker::binding::{AssumptionBasis, BindingConsistency};
    use phalcom_compiler_artifact::semantic::world::{CallableBindingAssumptionBasisV1 as Assumption, CallableBindingConsistencyV1 as Portable};
    Ok(match consistency {
        Portable::Unconstrained => BindingConsistency::Unconstrained,
        Portable::Validated => BindingConsistency::Validated,
        Portable::Assumed { basis } => BindingConsistency::Assumed {
            basis: match basis {
                Assumption::MissingValueEvidence { reason } => {
                    AssumptionBasis::MissingValueEvidence(import_unknown_reason_with_allocation_budget(reason, allocations)?)
                }
                Assumption::CallableParameterContract => AssumptionBasis::CallableParameterContract,
                Assumption::ContextualParameterContract => AssumptionBasis::ContextualParameterContract,
                Assumption::DerivedEvidence { origin } => AssumptionBasis::DerivedEvidence(match origin {
                    phalcom_compiler_artifact::semantic::world::EvidenceOriginV1::Syntax => crate::types::evidence::EvidenceOrigin::Syntax,
                    phalcom_compiler_artifact::semantic::world::EvidenceOriginV1::DeclarationSemantics => {
                        crate::types::evidence::EvidenceOrigin::DeclarationSemantics
                    }
                    phalcom_compiler_artifact::semantic::world::EvidenceOriginV1::ConstructorSemantics => {
                        crate::types::evidence::EvidenceOrigin::ConstructorSemantics
                    }
                    phalcom_compiler_artifact::semantic::world::EvidenceOriginV1::CallableSignature => {
                        crate::types::evidence::EvidenceOrigin::CallableSignature
                    }
                    phalcom_compiler_artifact::semantic::world::EvidenceOriginV1::NativeSignature => crate::types::evidence::EvidenceOrigin::NativeSignature,
                    phalcom_compiler_artifact::semantic::world::EvidenceOriginV1::DeveloperAnnotation => {
                        crate::types::evidence::EvidenceOrigin::DeveloperAnnotation
                    }
                    phalcom_compiler_artifact::semantic::world::EvidenceOriginV1::GenericInference => crate::types::evidence::EvidenceOrigin::GenericInference,
                    phalcom_compiler_artifact::semantic::world::EvidenceOriginV1::Flow => crate::types::evidence::EvidenceOrigin::Flow,
                    phalcom_compiler_artifact::semantic::world::EvidenceOriginV1::FieldLifecycle => crate::types::evidence::EvidenceOrigin::FieldLifecycle,
                    phalcom_compiler_artifact::semantic::world::EvidenceOriginV1::ContextualDerivation => {
                        crate::types::evidence::EvidenceOrigin::ContextualDerivation
                    }
                    phalcom_compiler_artifact::semantic::world::EvidenceOriginV1::PatternDecomposition => {
                        crate::types::evidence::EvidenceOrigin::PatternDecomposition
                    }
                }),
            },
        },
        Portable::Refuted { actual, expected, reason } => BindingConsistency::Refuted {
            actual: relocation.ty(*actual)?,
            expected: relocation.ty(*expected)?,
            reason: match reason {
                phalcom_compiler_artifact::semantic::world::CallableRefutationReasonV1::IncompatibleNominal => {
                    crate::types::relation::RefutationReason::IncompatibleNominal
                }
                phalcom_compiler_artifact::semantic::world::CallableRefutationReasonV1::TypeMismatch => crate::types::relation::RefutationReason::TypeMismatch,
                phalcom_compiler_artifact::semantic::world::CallableRefutationReasonV1::UnionMemberMismatch => {
                    crate::types::relation::RefutationReason::UnionMemberMismatch
                }
            },
        },
        Portable::DynamicBoundary { reason } => BindingConsistency::DynamicBoundary {
            obligation: crate::types::outcome::DynamicBoundaryObligation {
                reason: allocations.string(reason).map_err(allocation_error)?,
            },
        },
        Portable::Blocked { reason } => BindingConsistency::Blocked(import_callable_block_reason(reason, relocation, allocations)?),
        Portable::Cancelled => BindingConsistency::Cancelled,
        Portable::BudgetExceeded { report } => BindingConsistency::BudgetExceeded(import_callable_binding_budget(report)),
        Portable::InternalFailure { message } => BindingConsistency::InternalFailure(allocations.boxed_str(message).map_err(allocation_error)?),
    })
}

fn import_callable_binding_source_authority(
    authority: &phalcom_compiler_artifact::semantic::world::CallableBindingAuthorityV1,
    allocations: &mut AllocationBudget,
) -> Result<
    (
        crate::checker::analysis::CallableBindingOrigin,
        ImportedCallableBindingSourceAuthority,
        Option<SourceRange>,
    ),
    ArtifactError,
> {
    use phalcom_compiler_artifact::semantic::world::{CallableAnalysisSourceOwnerV1 as SourceOwnerV1, CallableBindingAuthorityV1 as Authority};
    Ok(match authority {
        Authority::BodyLocalSource {
            site_owner,
            site_local,
            declaration_range,
        } => {
            if declaration_range.start > declaration_range.end {
                return Err(referential("callable binding source declaration range is inverted"));
            }
            let owner = match site_owner {
                SourceOwnerV1::Module { module } => SourceOwner::Module(import_stable_module_with_allocation_budget(module, allocations)?),
                SourceOwnerV1::Callable { callable } => SourceOwner::Callable(import_stable_callable_with_allocation_budget(callable, allocations)?),
            };
            let site = SourceSiteId {
                owner,
                local: SourceSiteLocalId(*site_local),
            };
            let range = SourceRange {
                start: usize::try_from(declaration_range.start).map_err(|_| referential("callable binding source start exceeds address space"))?,
                end: usize::try_from(declaration_range.end).map_err(|_| referential("callable binding source end exceeds address space"))?,
            };
            (
                crate::checker::analysis::CallableBindingOrigin::BodyLocal,
                ImportedCallableBindingSourceAuthority::Source(site),
                Some(range),
            )
        }
        Authority::BodyLocalGenerated { reason } => (
            crate::checker::analysis::CallableBindingOrigin::BodyLocal,
            ImportedCallableBindingSourceAuthority::Generated(reason.clone()),
            None,
        ),
        Authority::ModuleValue { module, name, import_binding } => {
            let module = import_stable_module_with_allocation_budget(module, allocations)?;
            let symbol = phalcom_modules::linker::SymbolId {
                module,
                name: allocations.boxed_str(name).map_err(allocation_error)?,
            };
            (
                crate::checker::analysis::CallableBindingOrigin::ModuleValue {
                    symbol,
                    import: import_binding.map(phalcom_modules::linker::ImportBindingId),
                },
                ImportedCallableBindingSourceAuthority::ModuleValue,
                None,
            )
        }
    })
}

/// Imports complete callable binding rows without consulting source or
/// rebuilding formal attachments from names and ranges.
pub fn import_callable_binding_rows(
    record: &phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<Vec<ImportedCallableBindingRow>, ArtifactError> {
    use crate::checker::binding::{BindingContract, BindingContractOrigin};
    use crate::checker::causal::CausalInvalidity;
    use phalcom_compiler_artifact::semantic::world::{CallableBindingContractOriginV1 as ContractOrigin, CallableCausalInvalidityV1 as Invalidity};

    let mut child = allocations.child(allocations.remaining());
    let mut imported = child.vector::<ImportedCallableBindingRow>(record.bindings.len()).map_err(allocation_error)?;
    for (index, row) in record.bindings.iter().enumerate() {
        if usize::try_from(row.ordinal).ok() != Some(index) {
            return Err(referential("callable binding rows are not dense and ordinal-sorted"));
        }
        if row.range.start > row.range.end {
            return Err(referential("callable binding range is inverted"));
        }
        let binding = BindingId(row.ordinal);
        let parameter = row
            .parameter
            .as_ref()
            .map(|parameter| -> Result<CallableParameterId, ArtifactError> {
                Ok(CallableParameterId {
                    callable: import_stable_callable_with_allocation_budget(&parameter.callable, &mut child)?,
                    index: parameter.index,
                })
            })
            .transpose()?;
        let range = SourceRange {
            start: usize::try_from(row.range.start).map_err(|_| referential("callable binding range start exceeds address space"))?,
            end: usize::try_from(row.range.end).map_err(|_| referential("callable binding range end exceeds address space"))?,
        };
        let contract = row
            .contract
            .as_ref()
            .map(|contract| -> Result<_, ArtifactError> {
                let source = contract
                    .source
                    .as_ref()
                    .map(|source| -> Result<_, ArtifactError> {
                        if source.start > source.end {
                            return Err(referential("callable binding contract source range is inverted"));
                        }
                        Ok(SourceRange {
                            start: usize::try_from(source.start).map_err(|_| referential("callable binding contract source start exceeds address space"))?,
                            end: usize::try_from(source.end).map_err(|_| referential("callable binding contract source end exceeds address space"))?,
                        })
                    })
                    .transpose()?;
                Ok(BindingContract {
                    ty: relocation.ty(contract.ty)?,
                    origin: match contract.origin {
                        ContractOrigin::SourceAnnotation => BindingContractOrigin::SourceAnnotation,
                        ContractOrigin::InferredInitializer => BindingContractOrigin::InferredInitializer,
                        ContractOrigin::CallableParameter => BindingContractOrigin::CallableParameter,
                        ContractOrigin::ContextualBlockParameter => BindingContractOrigin::ContextualBlockParameter,
                        ContractOrigin::PatternBinding => BindingContractOrigin::PatternBinding,
                        ContractOrigin::StaticParameter => BindingContractOrigin::StaticParameter,
                    },
                    source,
                })
            })
            .transpose()?;
        let current = import_type_knowledge_with_allocation_budget(&row.current, relocation, &mut child)?;
        let denotation = row
            .denotation
            .as_ref()
            .map(|denotation| import_semantic_denotation_inner(denotation, relocation, &mut child))
            .transpose()?;
        let consistency = import_callable_binding_consistency(&row.consistency, relocation, &mut child)?;
        let causal_invalidity = match row.causal_invalidity {
            Invalidity::Clean => CausalInvalidity::Clean,
            Invalidity::One { cause } => CausalInvalidity::One(DiagnosticCauseId(cause)),
            Invalidity::Multiple => CausalInvalidity::Multiple,
        };
        let explanation = row.explanation.map(ExplanationId);
        let (storage_origin, source_authority, formal_range) = import_callable_binding_source_authority(&row.authority, &mut child)?;
        if formal_range.is_some() && !matches!(storage_origin, crate::checker::analysis::CallableBindingOrigin::BodyLocal) {
            return Err(referential("module-value binding cannot have a callable-local formal source attachment"));
        }
        let name = child.string(&row.name).map_err(allocation_error)?;
        let state = crate::checker::analysis::BindingState {
            binding,
            name,
            parameter,
            range,
            contract,
            current,
            denotation,
            consistency,
            causal_invalidity,
            mutable: row.mutable,
            initialization: match row.initialization {
                phalcom_compiler_artifact::semantic::world::CallableInitializationStateV1::Uninitialized => {
                    crate::checker::flow::InitializationState::Uninitialized
                }
                phalcom_compiler_artifact::semantic::world::CallableInitializationStateV1::MaybeInitialized => {
                    crate::checker::flow::InitializationState::MaybeInitialized
                }
                phalcom_compiler_artifact::semantic::world::CallableInitializationStateV1::DefinitelyInitialized => {
                    crate::checker::flow::InitializationState::DefinitelyInitialized
                }
            },
            version: row.version,
            explanation,
        };
        imported.push(ImportedCallableBindingRow {
            state,
            storage_origin,
            source_authority,
            formal_range,
        });
    }
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(imported)
}

fn import_callable_return_validation(
    validation: &phalcom_compiler_artifact::semantic::world::ReturnContractValidationV1,
) -> crate::signature::ReturnContractValidation {
    use crate::signature::ReturnContractValidation as Native;
    use phalcom_compiler_artifact::semantic::world::{EvidenceStatusV1, ReturnContractValidationV1 as Portable};
    match validation {
        Portable::NotApplicable => Native::NotApplicable,
        Portable::Unchecked => Native::Unchecked,
        Portable::Satisfied {
            status: EvidenceStatusV1::Established,
        } => Native::Satisfied(crate::types::evidence::EvidenceStatus::Established),
        Portable::Satisfied {
            status: EvidenceStatusV1::Assumed,
        } => Native::Satisfied(crate::types::evidence::EvidenceStatus::Assumed),
        Portable::Refuted => Native::Refuted,
        Portable::Blocked => Native::Blocked,
        Portable::DynamicBoundary => Native::DynamicBoundary,
    }
}

fn import_callable_analysis_status(
    status: &phalcom_compiler_artifact::semantic::world::CallableAnalysisStatusV1,
) -> crate::checker::analysis::CallableAnalysisStatus {
    use crate::checker::analysis::CallableAnalysisStatus as Native;
    use phalcom_compiler_artifact::semantic::world::CallableAnalysisStatusV1 as Portable;
    match status {
        Portable::Complete => Native::Complete,
        Portable::Partial => Native::Partial,
        Portable::Blocked => Native::Blocked,
        Portable::Cancelled => Native::Cancelled,
        Portable::BudgetExceeded => Native::BudgetExceeded,
    }
}

fn import_callable_flow_state_summary(
    flow: &phalcom_compiler_artifact::semantic::world::CallableFlowStateSummaryV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::checker::analysis::FlowStateSummary, ArtifactError> {
    use crate::checker::analysis::{FlowBindingSummary, FlowFieldSummary, FlowStateSummary};
    use crate::checker::causal::CausalInvalidity;
    use crate::checker::flow::{FieldContractValidity, InitializationState};
    use phalcom_compiler_artifact::semantic::world::{CallableCausalInvalidityV1, CallableFieldContractValidityV1, CallableInitializationStateV1};

    let mut bindings = allocations.vector(flow.bindings.len()).map_err(allocation_error)?;
    for binding in &flow.bindings {
        bindings.push((
            BindingId(binding.binding),
            FlowBindingSummary {
                knowledge: import_type_knowledge_with_allocation_budget(&binding.knowledge, relocation, allocations)?,
                contract: binding
                    .contract
                    .as_ref()
                    .map(|contract| {
                        Ok::<crate::checker::binding::BindingContract, ArtifactError>(crate::checker::binding::BindingContract {
                            ty: relocation.ty(contract.ty)?,
                            origin: match contract.origin {
                                phalcom_compiler_artifact::semantic::world::CallableBindingContractOriginV1::SourceAnnotation => {
                                    crate::checker::binding::BindingContractOrigin::SourceAnnotation
                                }
                                phalcom_compiler_artifact::semantic::world::CallableBindingContractOriginV1::InferredInitializer => {
                                    crate::checker::binding::BindingContractOrigin::InferredInitializer
                                }
                                phalcom_compiler_artifact::semantic::world::CallableBindingContractOriginV1::CallableParameter => {
                                    crate::checker::binding::BindingContractOrigin::CallableParameter
                                }
                                phalcom_compiler_artifact::semantic::world::CallableBindingContractOriginV1::ContextualBlockParameter => {
                                    crate::checker::binding::BindingContractOrigin::ContextualBlockParameter
                                }
                                phalcom_compiler_artifact::semantic::world::CallableBindingContractOriginV1::PatternBinding => {
                                    crate::checker::binding::BindingContractOrigin::PatternBinding
                                }
                                phalcom_compiler_artifact::semantic::world::CallableBindingContractOriginV1::StaticParameter => {
                                    crate::checker::binding::BindingContractOrigin::StaticParameter
                                }
                            },
                            source: contract
                                .source
                                .as_ref()
                                .map(|range| {
                                    Ok::<SourceRange, ArtifactError>(SourceRange {
                                        start: usize::try_from(range.start).map_err(|_| referential("callable flow contract start exceeds address space"))?,
                                        end: usize::try_from(range.end).map_err(|_| referential("callable flow contract end exceeds address space"))?,
                                    })
                                })
                                .transpose()?,
                        })
                    })
                    .transpose()?,
                consistency: import_callable_binding_consistency(&binding.consistency, relocation, allocations)?,
                mutable: binding.mutable,
                initialization: match binding.initialization {
                    CallableInitializationStateV1::Uninitialized => InitializationState::Uninitialized,
                    CallableInitializationStateV1::MaybeInitialized => InitializationState::MaybeInitialized,
                    CallableInitializationStateV1::DefinitelyInitialized => InitializationState::DefinitelyInitialized,
                },
            },
        ));
    }
    let bindings = import_analysis_map(bindings, allocations, "callable flow bindings are not canonical")?;

    let mut fields = allocations.vector(flow.fields.len()).map_err(allocation_error)?;
    for field in &flow.fields {
        let id = crate::identity::FieldId::new(
            import_stable_declaration_with_allocation_budget(&field.field.owner, allocations)?,
            allocations.boxed_str(&field.field.name).map_err(allocation_error)?,
            from_stable_dispatch_side(field.field.side),
        );
        fields.push((
            id,
            FlowFieldSummary {
                contract: import_type_knowledge_with_allocation_budget(&field.contract, relocation, allocations)?,
                current: import_type_knowledge_with_allocation_budget(&field.current, relocation, allocations)?,
                initialization: match field.initialization {
                    CallableInitializationStateV1::Uninitialized => InitializationState::Uninitialized,
                    CallableInitializationStateV1::MaybeInitialized => InitializationState::MaybeInitialized,
                    CallableInitializationStateV1::DefinitelyInitialized => InitializationState::DefinitelyInitialized,
                },
                validity: match &field.validity {
                    CallableFieldContractValidityV1::Unchecked => FieldContractValidity::Unchecked,
                    CallableFieldContractValidityV1::Validated => FieldContractValidity::Validated,
                    CallableFieldContractValidityV1::Assumed => FieldContractValidity::Assumed,
                    CallableFieldContractValidityV1::Refuted => FieldContractValidity::Refuted,
                    CallableFieldContractValidityV1::Blocked { reason } => {
                        FieldContractValidity::Blocked(import_callable_block_reason(reason, relocation, allocations)?)
                    }
                    CallableFieldContractValidityV1::DynamicBoundary { reason } => {
                        FieldContractValidity::DynamicBoundary(crate::types::outcome::DynamicBoundaryObligation {
                            reason: allocations.string(reason).map_err(allocation_error)?,
                        })
                    }
                },
                causal_invalidity: match field.causal_invalidity {
                    CallableCausalInvalidityV1::Clean => CausalInvalidity::Clean,
                    CallableCausalInvalidityV1::One { cause } => CausalInvalidity::One(DiagnosticCauseId(cause)),
                    CallableCausalInvalidityV1::Multiple => CausalInvalidity::Multiple,
                },
            },
        ));
    }
    Ok(FlowStateSummary {
        bindings,
        fields: import_analysis_map(fields, allocations, "callable flow fields are not canonical")?,
        fact_count: usize::try_from(flow.fact_count).map_err(|_| referential("callable flow fact count exceeds address space"))?,
    })
}

fn take_sparse<T>(rows: &mut std::iter::Peekable<impl Iterator<Item = (u32, T)>>, ordinal: u32) -> Option<T> {
    if rows.peek().is_some_and(|(candidate, _)| *candidate == ordinal) {
        rows.next().map(|(_, value)| value)
    } else {
        None
    }
}

fn import_callable_range(range: &phalcom_compiler_artifact::module_world::SpanV1) -> Result<SourceRange, ArtifactError> {
    if range.start > range.end {
        return Err(referential("callable source range is inverted"));
    }
    Ok(SourceRange {
        start: usize::try_from(range.start).map_err(|_| referential("callable source range start exceeds address space"))?,
        end: usize::try_from(range.end).map_err(|_| referential("callable source range end exceeds address space"))?,
    })
}

fn import_callable_trait_selection(
    selection: &phalcom_compiler_artifact::semantic::world::CallableTraitDispatchSelectionV1,
    evidence_nodes: &[phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1],
    importer: &mut CallableEvidenceImporter<'_>,
) -> Result<crate::trait_dispatch::TraitDispatchSelection, ArtifactError> {
    let evidence = importer.evidence(evidence_nodes, selection.evidence, 0)?;
    let evidence_fingerprint = crate::db::fingerprint::generic_conformance_evidence_fingerprint(&evidence);
    let mut targets = importer.allocations.vector(selection.requirement_targets.len()).map_err(allocation_error)?;
    for target in &selection.requirement_targets {
        targets.push((
            import_callable_requirement_identity_with_allocation_budget(&target.requirement, importer.allocations)?,
            import_callable_requirement_selection(&target.selection, evidence_nodes, importer, 0)?,
        ));
    }
    let requirement_targets = import_analysis_map(targets, importer.allocations, "trait dispatch requirement targets are not canonical")?;
    Ok(crate::trait_dispatch::TraitDispatchSelection {
        source_impl: import_stable_impl_with_allocation_budget(&selection.source_impl, importer.allocations)?,
        exact_target: importer.relocation.ty(selection.exact_target)?,
        exact_trait_ref: import_trait_ref_identity(&selection.exact_trait_ref, importer.relocation, importer.allocations)?,
        requirement: import_callable_requirement_identity_with_allocation_budget(&selection.requirement, importer.allocations)?,
        callable: selection
            .callable
            .as_ref()
            .map(|callable| import_stable_callable_with_allocation_budget(callable, importer.allocations))
            .transpose()?,
        signature: import_callable_signature(&selection.signature, importer.relocation, importer.store, importer.allocations)?,
        evidence_fingerprint,
        evidence,
        selection: import_callable_requirement_selection(&selection.selection, evidence_nodes, importer, 0)?,
        requirement_targets,
    })
}

fn import_callable_iteration_requirement(
    requirement: &phalcom_compiler_artifact::semantic::world::CallableForIterationRequirementV1,
    evidence_nodes: &[phalcom_compiler_artifact::semantic::world::CallableConformanceEvidenceNodeV1],
    importer: &mut CallableEvidenceImporter<'_>,
) -> Result<crate::checker::analysis::ForIterationRequirementResolution, ArtifactError> {
    use crate::checker::analysis::ForIterationRequirementResolution as Native;
    use phalcom_compiler_artifact::semantic::world::CallableForIterationRequirementV1 as Portable;
    Ok(match requirement {
        Portable::AbstractRequirement { requirement, signature } => Native::AbstractRequirement {
            requirement: import_callable_requirement_identity_with_allocation_budget(requirement, importer.allocations)?,
            signature: import_callable_signature(signature, importer.relocation, importer.store, importer.allocations)?,
        },
        Portable::Evidence {
            conformance,
            evidence,
            requirement,
            signature,
        } => Native::Evidence {
            conformance: import_generic_conformance_key_with_allocation_budget(conformance, importer.relocation, importer.allocations)?,
            evidence: importer.evidence(evidence_nodes, *evidence, 0)?,
            requirement: import_callable_requirement_identity_with_allocation_budget(requirement, importer.allocations)?,
            signature: import_callable_signature(signature, importer.relocation, importer.store, importer.allocations)?,
        },
    })
}

pub(super) struct ImportedCallableResolutionAuthority {
    for_iteration_lanes: Arc<crate::checker::analysis::ForIterationLaneResolutionIndex>,
    pub(super) positional_expansions: Arc<crate::checker::analysis::PositionalExpansionIterationResolutionIndex>,
    pub(super) known_invalid_positional_expansions: Arc<crate::checker::analysis::KnownInvalidPositionalExpansionIndex>,
    sequence_literals: Arc<crate::checker::analysis::SequenceLiteralResolutionIndex>,
    associated_resolutions: Arc<crate::checker::associated::AssociatedResolutionIndex>,
    callable_reference_resolutions: Arc<crate::checker::associated::CallableReferenceResolutionIndex>,
    family_applications: Arc<crate::checker::associated::FamilyApplicationResolutionIndex>,
}

pub(super) fn import_callable_family_applications(
    record: &phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2,
    body: crate::identity::BodyId,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<Arc<crate::checker::associated::FamilyApplicationResolutionIndex>, ArtifactError> {
    use crate::checker::associated::{FamilyApplicationCandidate, FamilyApplicationKind, FamilyApplicationResolution, FamilyApplicationSelection};
    use crate::identity::{ExpressionId, LocalExpressionId};
    use phalcom_compiler_artifact::semantic::world::{CallableFamilyApplicationKindV1 as Kind, CallableFamilyApplicationSelectionV1 as Selection};

    let mut child = allocations.child(allocations.remaining());
    let mut rows = child.vector(record.family_applications.len()).map_err(allocation_error)?;
    for application in &record.family_applications {
        if application.expression as usize >= record.expressions.len() {
            return Err(referential("family application references an absent owner-local expression"));
        }
        let selection = match &application.selection {
            Selection::Static {
                operation,
                target,
                callable_type,
                result_type,
            } => FamilyApplicationSelection::Static {
                operation: import_family_operation_shape_with_allocation_budget(operation, &mut child)?,
                target: target.as_ref().map(|target| import_invocation_target(target, &mut child)).transpose()?,
                callable_type: relocation.ty(*callable_type)?,
                result_type: relocation.ty(*result_type)?,
            },
            Selection::Dynamic { candidates, result_type } => {
                let mut imported = child.vector(candidates.len()).map_err(allocation_error)?;
                for candidate in candidates {
                    imported.push(FamilyApplicationCandidate {
                        operation: import_family_operation_shape_with_allocation_budget(&candidate.operation, &mut child)?,
                        target: candidate
                            .target
                            .as_ref()
                            .map(|target| import_invocation_target(target, &mut child))
                            .transpose()?,
                        callable_type: relocation.ty(candidate.callable_type)?,
                    });
                }
                FamilyApplicationSelection::Dynamic {
                    candidates: child.boxed_slice(imported).map_err(allocation_error)?,
                    result_type: result_type.map(|ty| relocation.ty(ty)).transpose()?,
                }
            }
        };
        rows.push((
            ExpressionId::new(body, LocalExpressionId(application.expression)),
            FamilyApplicationResolution {
                family_type: relocation.ty(application.family_type)?,
                kind: match application.kind {
                    Kind::Associated => FamilyApplicationKind::Associated,
                    Kind::Behavioral => FamilyApplicationKind::Behavioral,
                },
                selection,
            },
        ));
    }
    let index = import_analysis_map(rows, &mut child, "family application expressions are not canonical")?;
    let index = child.arc(index).map_err(allocation_error)?;
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(index)
}

pub(super) fn import_callable_resolution_authority(
    record: &phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2,
    body: crate::identity::BodyId,
    relocation: &TypeRelocation,
    store: &TypeStore,
    source_plan_fingerprints: &[(crate::identity::ImplId, crate::db::ProductFingerprint)],
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCallableResolutionAuthority, ArtifactError> {
    use crate::checker::analysis::{
        ForIterationLaneResolution, SequenceConstructionPlan, SequenceExpansionResolution, SequenceLiteralFamily, SequenceLiteralResolution,
    };
    use crate::checker::associated::{
        AssociatedResolution, AssociatedResolutionKind, BoundBehavioralMember, CallableReferenceResolution, CallableReferenceResolutionKind,
    };
    use phalcom_compiler_artifact::semantic::world as wire;

    let mut child = allocations.child(allocations.remaining());
    let mut diagnostic_previous_observed = phalcom_type_meta::allocation::current_observed_bytes().unwrap_or(0);
    let mut diagnostic_previous_ledger = child.requested();
    let mut importer = CallableEvidenceImporter::new(
        record.conformance_evidence.len(),
        relocation,
        store,
        source_plan_fingerprints,
        &mut child,
        limits.max_recursion_depth,
    )?;
    record_callable_import_allocation_diagnostic_stage(21, importer.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
    let expression = |ordinal: u32| -> Result<crate::identity::ExpressionId, ArtifactError> {
        if ordinal as usize >= record.expressions.len() {
            return Err(referential("callable resolution references an absent expression"));
        }
        Ok(crate::identity::ExpressionId::new(body, crate::identity::LocalExpressionId(ordinal)))
    };
    let mut lanes = importer.allocations.vector(record.for_iteration_lanes.len()).map_err(allocation_error)?;
    for lane in &record.for_iteration_lanes {
        let id = expression(lane.expression)?;
        lanes.push((
            id,
            ForIterationLaneResolution {
                source_expression: id,
                source_range: import_callable_range(&lane.source_range)?,
                source_type: relocation.ty(lane.source_type)?,
                item_type: relocation.ty(lane.item_type)?,
                iterator_type: relocation.ty(lane.iterator_type)?,
                iterator: import_callable_iteration_requirement(&lane.iterator, &record.conformance_evidence, &mut importer)?,
                next: import_callable_iteration_requirement(&lane.next, &record.conformance_evidence, &mut importer)?,
            },
        ));
    }
    let for_iteration_lanes = import_analysis_map(lanes, importer.allocations, "callable iteration lanes are not canonical")?;
    let for_iteration_lanes = importer.allocations.arc(for_iteration_lanes).map_err(allocation_error)?;
    record_callable_import_allocation_diagnostic_stage(22, importer.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

    let mut expansions = importer.allocations.vector(record.positional_expansions.len()).map_err(allocation_error)?;
    for expansion in &record.positional_expansions {
        let id = expression(expansion.expression)?;
        expansions.push((
            id,
            crate::checker::analysis::PositionalExpansionIterationResolution {
                source_expression: id,
                source_range: import_callable_range(&expansion.source_range)?,
                source_type: relocation.ty(expansion.source_type)?,
                item_type: relocation.ty(expansion.item_type)?,
                iterator_type: relocation.ty(expansion.iterator_type)?,
                iterator: import_callable_iteration_requirement(&expansion.iterator, &record.conformance_evidence, &mut importer)?,
                next: import_callable_iteration_requirement(&expansion.next, &record.conformance_evidence, &mut importer)?,
            },
        ));
    }
    let positional_expansions = import_analysis_map(expansions, importer.allocations, "callable positional expansions are not canonical")?;
    let positional_expansions = importer.allocations.arc(positional_expansions).map_err(allocation_error)?;
    record_callable_import_allocation_diagnostic_stage(23, importer.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
    let mut invalid = importer
        .allocations
        .vector(record.known_invalid_positional_expansions.len())
        .map_err(allocation_error)?;
    for expansion in &record.known_invalid_positional_expansions {
        let id = expression(expansion.expression)?;
        invalid.push((
            id,
            crate::checker::analysis::KnownInvalidPositionalExpansion {
                source_expression: id,
                source_range: import_callable_range(&expansion.source_range)?,
                source_type: relocation.ty(expansion.source_type)?,
            },
        ));
    }
    let known_invalid_positional_expansions = import_analysis_map(invalid, importer.allocations, "known-invalid positional expansions are not canonical")?;
    let known_invalid_positional_expansions = importer.allocations.arc(known_invalid_positional_expansions).map_err(allocation_error)?;
    record_callable_import_allocation_diagnostic_stage(24, importer.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

    let mut literals = importer.allocations.vector(record.sequence_literals.len()).map_err(allocation_error)?;
    for literal in &record.sequence_literals {
        let id = expression(literal.expression)?;
        let mut expansions = importer.allocations.vector(literal.exact_expansions.len()).map_err(allocation_error)?;
        for expansion in &literal.exact_expansions {
            expansions.push((
                import_callable_range(&expansion.source_range)?,
                match expansion.expansion {
                    wire::CallableSequenceExpansionV1::TuplePositionals { extent } => SequenceExpansionResolution::TuplePositionals {
                        extent: relocation.ty(extent)?,
                    },
                    wire::CallableSequenceExpansionV1::VectorStorage { extent } => SequenceExpansionResolution::VectorStorage {
                        extent: relocation.ty(extent)?,
                    },
                },
            ));
        }
        let exact_expansions = import_analysis_map(expansions, importer.allocations, "callable sequence expansion ranges are not canonical")?;
        literals.push((
            id,
            SequenceLiteralResolution {
                source_expression: id,
                source_range: import_callable_range(&literal.source_range)?,
                family: match literal.family {
                    wire::CallableSequenceFamilyV1::List => SequenceLiteralFamily::List,
                    wire::CallableSequenceFamilyV1::Array => SequenceLiteralFamily::Array,
                    wire::CallableSequenceFamilyV1::Vector => SequenceLiteralFamily::Vector,
                },
                result_type: relocation.ty(literal.result_type)?,
                element_type: relocation.ty(literal.element_type)?,
                extent: literal.extent.map(|extent| relocation.ty(extent)).transpose()?,
                construction: match &literal.construction {
                    wire::CallableSequenceConstructionV1::Fixed { element_count } => SequenceConstructionPlan::Fixed {
                        element_count: usize::try_from(*element_count).map_err(|_| referential("callable sequence element count exceeds address space"))?,
                    },
                    wire::CallableSequenceConstructionV1::Expanded => SequenceConstructionPlan::Expanded,
                    wire::CallableSequenceConstructionV1::VectorIdentitySpread { source_range, extent } => SequenceConstructionPlan::VectorIdentitySpread {
                        source_range: import_callable_range(source_range)?,
                        extent: relocation.ty(*extent)?,
                    },
                },
                exact_expansions,
            },
        ));
    }
    let sequence_literals = import_analysis_map(literals, importer.allocations, "callable sequence literals are not canonical")?;
    let sequence_literals = importer.allocations.arc(sequence_literals).map_err(allocation_error)?;
    record_callable_import_allocation_diagnostic_stage(25, importer.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

    let mut associated = importer.allocations.vector(record.associated_resolutions.len()).map_err(allocation_error)?;
    for resolution in &record.associated_resolutions {
        let family = resolution
            .family
            .as_ref()
            .map(|family| -> Result<_, ArtifactError> {
                Ok(crate::identity::AssociatedFamilyId::new(
                    import_stable_declaration_with_allocation_budget(&family.owner, importer.allocations)?,
                    import_associated_selector_with_allocation_budget(&family.selector, importer.allocations)?,
                ))
            })
            .transpose()?;
        let kind = match &resolution.kind {
            wire::CallableAssociatedResolutionKindV1::ExactValue { member, value_type } => AssociatedResolutionKind::ExactValue {
                member: import_associated_member_with_allocation_budget(member, importer.allocations)?,
                value_type: relocation.ty(*value_type)?,
            },
            wire::CallableAssociatedResolutionKindV1::StaticInvoke {
                member,
                target,
                result_type,
                argument_mapping,
            } => {
                let argument_mapping = argument_mapping
                    .as_ref()
                    .map(|mapping| -> Result<_, ArtifactError> {
                        let mut imported = importer.allocations.vector(mapping.len()).map_err(allocation_error)?;
                        imported.extend_from_slice(mapping);
                        Ok(importer.allocations.boxed_slice(imported).map_err(allocation_error)?)
                    })
                    .transpose()?;
                AssociatedResolutionKind::StaticInvoke {
                    member: import_associated_member_with_allocation_budget(member, importer.allocations)?,
                    target: import_invocation_target(target, importer.allocations)?,
                    result_type: relocation.ty(*result_type)?,
                    argument_mapping,
                }
            }
        };
        associated.push((
            expression(resolution.expression)?,
            AssociatedResolution {
                owner_form: relocation.ty(resolution.owner_form)?,
                lookup_owner: import_stable_declaration_with_allocation_budget(&resolution.lookup_owner, importer.allocations)?,
                family,
                kind,
            },
        ));
    }
    let associated_resolutions = import_analysis_map(associated, importer.allocations, "callable associated resolutions are not canonical")?;
    let associated_resolutions = importer.allocations.arc(associated_resolutions).map_err(allocation_error)?;
    record_callable_import_allocation_diagnostic_stage(26, importer.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

    let mut references = importer
        .allocations
        .vector(record.callable_reference_resolutions.len())
        .map_err(allocation_error)?;
    for resolution in &record.callable_reference_resolutions {
        let mut members = importer.allocations.vector(resolution.members.len()).map_err(allocation_error)?;
        for member in &resolution.members {
            let conditional_capability = member
                .conditional_capability
                .as_ref()
                .map(|capability| -> Result<_, ArtifactError> {
                    if member.conditional.as_ref() != Some(&capability.selection) || capability.selection.impl_id != capability.applicability.impl_id {
                        return Err(referential("callable reference capability disagrees with its selected implementation"));
                    }
                    Ok(crate::dispatch::ConditionalSelectedCallCapability {
                        selection: import_conditional_dispatch_selection(&capability.selection, importer.allocations)?,
                        applicability: import_callable_inherent_specialization(&capability.applicability, &record.conformance_evidence, &mut importer)?,
                    })
                })
                .transpose()?;
            members.push(BoundBehavioralMember {
                operation: import_family_operation_shape_with_allocation_budget(&member.operation, importer.allocations)?,
                member_kind: match member.member_kind {
                    wire::CallableFamilyMemberKindV1::Value => crate::types::family::FamilyMemberTypeKind::Value,
                    wire::CallableFamilyMemberKindV1::Callable => crate::types::family::FamilyMemberTypeKind::Callable,
                },
                target: import_invocation_target(&member.target, importer.allocations)?,
                callable_type: relocation.ty(member.callable_type)?,
                conditional: member
                    .conditional
                    .as_ref()
                    .map(|selection| import_conditional_dispatch_selection(selection, importer.allocations))
                    .transpose()?,
                conditional_capability,
                trait_dispatch: member
                    .trait_dispatch
                    .as_ref()
                    .map(|selection| import_callable_trait_selection(selection, &record.conformance_evidence, &mut importer))
                    .transpose()?,
            });
        }
        references.push((
            expression(resolution.expression)?,
            CallableReferenceResolution {
                kind: CallableReferenceResolutionKind::BoundFamily {
                    family_type: relocation.ty(resolution.family_type)?,
                    spec: import_behavioral_family_spec_with_allocation_budget(&resolution.spec, importer.allocations)?,
                    members: importer.allocations.boxed_slice(members).map_err(allocation_error)?,
                },
            },
        ));
    }
    let callable_reference_resolutions = import_analysis_map(references, importer.allocations, "callable reference resolutions are not canonical")?;
    let callable_reference_resolutions = importer.allocations.arc(callable_reference_resolutions).map_err(allocation_error)?;
    record_callable_import_allocation_diagnostic_stage(27, importer.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
    let family_applications = import_callable_family_applications(record, body, relocation, importer.allocations)?;
    record_callable_import_allocation_diagnostic_stage(28, importer.allocations, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
    drop(importer);
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok(ImportedCallableResolutionAuthority {
        for_iteration_lanes,
        positional_expansions,
        known_invalid_positional_expansions,
        sequence_literals,
        associated_resolutions,
        callable_reference_resolutions,
        family_applications,
    })
}

fn import_callable_observed_import_phases(
    record: &phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2,
    callable: &CallableId,
    module_world: &phalcom_compiler_artifact::module_world::ModuleWorldV2,
    import_phase_products: &impl phalcom_modules::linker::ModuleImportPhaseProductLookup,
    allocations: &mut AllocationBudget,
) -> Result<crate::checker::analysis_map::AnalysisMap<phalcom_modules::identity::SourceImportBindingId, phalcom_modules::graph::DependencyPhase>, ArtifactError>
{
    use phalcom_compiler_artifact::semantic::world::DependencyPhaseV1;
    use phalcom_modules::graph::DependencyPhase;
    use phalcom_modules::identity::{ImportBindingLocalId, ImportSiteId, ImportSiteLocalId, SourceImportBindingId};

    let stable_module = stable_callable_module(&record.callable);
    let native_module = import_stable_module_with_allocation_budget(stable_module, allocations)?;
    if &native_module != callable.owner.module() {
        return Err(referential("callable phase owner module disagrees with its canonical callable"));
    }
    let product = import_phase_products
        .get(&native_module)
        .ok_or_else(|| referential("callable phase owner has no module import phase product"))?;
    let mut phases = allocations.vector(record.observed_import_phases.len()).map_err(allocation_error)?;
    for observed in &record.observed_import_phases {
        let (site, local) = module_world
            .source_import_binding_at(stable_module, observed.binding)?
            .ok_or_else(|| referential("callable observed import phase references an absent source binding"))?;
        let binding = SourceImportBindingId {
            site: ImportSiteId::new(
                native_module.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                ImportSiteLocalId::new(site),
            ),
            local: ImportBindingLocalId(local),
        };
        let phase = match observed.phase {
            DependencyPhaseV1::InterfaceOnly => DependencyPhase::InterfaceOnly,
            DependencyPhaseV1::Runtime => DependencyPhase::Runtime,
        };
        let Some(summary) = product.bindings.get(&binding) else {
            return Err(referential("callable observed phase has no matching module phase summary"));
        };
        // The module phase is the join of observations from all semantic
        // owners. A Runtime summary can therefore cover an individual
        // InterfaceOnly callable observation; the reverse is invalid.
        let phase_is_covered = summary.observed
            && matches!(
                (summary.phase, phase),
                (DependencyPhase::Runtime, _) | (DependencyPhase::InterfaceOnly, DependencyPhase::InterfaceOnly)
            );
        if !phase_is_covered {
            return Err(referential("callable observed phase disagrees with module phase summary"));
        }
        phases.push((binding, phase));
    }
    if phases.is_empty() {
        return Ok(crate::checker::analysis_map::AnalysisMap::new());
    }
    phases.sort_unstable_by(|left, right| left.0.cmp(&right.0));
    crate::checker::analysis_map::AnalysisMap::from_budgeted_entries(phases, allocations).map_err(|error| match error {
        crate::checker::analysis_map::AnalysisMapBuildError::Allocation(error) => allocation_error(error),
        crate::checker::analysis_map::AnalysisMapBuildError::Entries(phalcom_type_meta::allocation::SortedMapEntriesError::DuplicateKey) => {
            referential("callable observed import phase is duplicated after identity relocation")
        }
        crate::checker::analysis_map::AnalysisMapBuildError::Entries(phalcom_type_meta::allocation::SortedMapEntriesError::OutOfOrder) => {
            referential("callable observed import phases are not canonical after identity relocation")
        }
    })
}

fn stable_callable_module(callable: &phalcom_type_meta::identity::StableCallableRef) -> &phalcom_type_meta::identity::StableModuleRef {
    use phalcom_type_meta::identity::StableCallableOwnerRef;
    match &callable.owner {
        StableCallableOwnerRef::Declaration(owner) => &owner.module,
        StableCallableOwnerRef::Variant(owner) => &owner.owner.module,
        StableCallableOwnerRef::Conformance(owner) => &owner.module,
    }
}

fn import_module_execution_authority(
    authority: &phalcom_compiler_artifact::semantic::world::ModuleExecutionAuthorityV1,
    module_world: &phalcom_compiler_artifact::module_world::ModuleWorldV2,
    allocations: &mut AllocationBudget,
) -> Result<
    (
        phalcom_modules::authority::ModuleAuthorityMap<crate::identity::ModuleId, phalcom_modules::linker::ModuleImportPhaseProduct>,
        Vec<phalcom_modules::linker::RuntimeDependencyRequirement>,
    ),
    ArtifactError,
> {
    use phalcom_compiler_artifact::semantic::world as wire;
    use phalcom_modules::graph::{
        DependencyPhase, RuntimeDependencyReason, RuntimeRequirementProducer, RuntimeRequirementProduct, RuntimeRequirementProvenance,
        RuntimeRequirementTarget, RuntimeRequirementTiming,
    };
    use phalcom_modules::identity::{ImportBindingLocalId, ImportSiteId, ImportSiteLocalId, SourceImportBindingId};
    use phalcom_modules::linker::{
        ImportPhaseSummary, LinkedReadSpec, ModuleImportPhaseProduct, PhaseEvidenceCoverage, RuntimeDependencyRequirement, SymbolId,
    };

    if authority.import_phase_products.len() != module_world.modules.len() {
        return Err(referential("module import phase authority must contain one product for every module-world row"));
    }
    let mut phase_product_entries = allocations
        .vector::<(crate::identity::ModuleId, phalcom_modules::linker::ModuleImportPhaseProduct)>(authority.import_phase_products.len())
        .map_err(allocation_error)?;
    for product in &authority.import_phase_products {
        if !module_world.modules.iter().any(|row| row.identity == product.module) {
            return Err(referential("module import phase product owner is absent from the module world"));
        }
        let expected_count = module_world
            .source_import_binding_count(&product.module)?
            .ok_or_else(|| referential("module import phase product owner is absent from the module world"))?;
        if product.bindings.len() != expected_count as usize {
            return Err(referential("module import phase product does not cover the exact source-binding range"));
        }
        let module = import_stable_module_with_allocation_budget(&product.module, allocations)?;
        let mut binding_entries = allocations
            .vector::<(SourceImportBindingId, ImportPhaseSummary)>(product.bindings.len())
            .map_err(allocation_error)?;
        for (expected_ordinal, row) in product.bindings.iter().enumerate() {
            let expected_ordinal = u32::try_from(expected_ordinal).map_err(|_| referential("module import phase ordinal exceeds u32"))?;
            if row.binding != expected_ordinal {
                return Err(referential("module import phase product bindings are not a dense ascending ordinal range"));
            }
            let (site, local) = module_world
                .source_import_binding_at(&product.module, row.binding)?
                .ok_or_else(|| referential("module import phase product references an absent source binding"))?;
            let source_binding = SourceImportBindingId {
                site: ImportSiteId::new(
                    module.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                    ImportSiteLocalId::new(site),
                ),
                local: ImportBindingLocalId(local),
            };
            let summary = ImportPhaseSummary {
                phase: match row.summary.phase {
                    wire::DependencyPhaseV1::InterfaceOnly => DependencyPhase::InterfaceOnly,
                    wire::DependencyPhaseV1::Runtime => DependencyPhase::Runtime,
                },
                observed: row.summary.observed,
            };
            binding_entries.push((source_binding, summary));
        }
        let bindings = phalcom_modules::authority::ModuleAuthorityMap::from_budgeted_entries(binding_entries, allocations).map_err(|error| match error {
            phalcom_modules::authority::ModuleAuthorityBuildError::Allocation(error) => allocation_error(error),
            _ => referential("module import phase binding is duplicated after identity relocation"),
        })?;
        let imported = ModuleImportPhaseProduct {
            module: module.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            bindings,
            coverage: match product.coverage {
                wire::PhaseEvidenceCoverageV1::Full => PhaseEvidenceCoverage::Complete,
                wire::PhaseEvidenceCoverageV1::Incomplete => PhaseEvidenceCoverage::Incomplete,
            },
        };
        phase_product_entries.push((module, imported));
    }
    let phase_products =
        phalcom_modules::authority::ModuleAuthorityMap::from_budgeted_entries(phase_product_entries, allocations).map_err(|error| match error {
            phalcom_modules::authority::ModuleAuthorityBuildError::Allocation(error) => allocation_error(error),
            _ => referential("module import phase product owner is duplicated after identity relocation"),
        })?;

    let mut runtime_requirements = allocations
        .vector::<RuntimeDependencyRequirement>(authority.runtime_requirements.len())
        .map_err(allocation_error)?;
    for requirement in &authority.runtime_requirements {
        if !module_world.modules.iter().any(|row| row.identity == requirement.importer)
            || !module_world.modules.iter().any(|row| row.identity == requirement.dependency)
        {
            return Err(referential("runtime dependency requirement references a module absent from the module world"));
        }
        if requirement.provenance.consumer != requirement.importer {
            return Err(referential("runtime dependency consumer disagrees with its importer"));
        }
        let (target_module, target_binding) = match &requirement.provenance.target {
            wire::RuntimeRequirementTargetV1::Module { module } => (module, None),
            wire::RuntimeRequirementTargetV1::Binding { module, name } => (module, Some(name.as_ref())),
        };
        if target_module != &requirement.dependency {
            return Err(referential("runtime dependency target disagrees with its dependency"));
        }
        let importer = import_stable_module_with_allocation_budget(&requirement.importer, allocations)?;
        let dependency = import_stable_module_with_allocation_budget(&requirement.dependency, allocations)?;
        let range = SourceRange {
            start: usize::try_from(requirement.range.start).map_err(|_| referential("runtime dependency range start exceeds address space"))?,
            end: usize::try_from(requirement.range.end).map_err(|_| referential("runtime dependency range end exceeds address space"))?,
        };
        if range.start > range.end {
            return Err(referential("runtime dependency range is inverted"));
        }
        let read_target = requirement
            .read_target
            .as_ref()
            .map(|target| -> Result<LinkedReadSpec, ArtifactError> {
                Ok(match target {
                    wire::LinkedReadSpecV1::Module { module } => {
                        if module != &requirement.dependency || target_binding.is_some() {
                            return Err(referential("runtime module read target disagrees with its provenance"));
                        }
                        LinkedReadSpec::Module(import_stable_module_with_allocation_budget(module, allocations)?)
                    }
                    wire::LinkedReadSpecV1::Binding { symbol } => {
                        let symbol = import_stable_declaration_with_allocation_budget(symbol, allocations)?;
                        if symbol.module != dependency || target_binding != Some(symbol.name.as_ref()) {
                            return Err(referential("runtime binding read target disagrees with its provenance"));
                        }
                        LinkedReadSpec::Binding(SymbolId {
                            module: symbol.module,
                            name: allocations.boxed_str(&symbol.name).map_err(allocation_error)?,
                        })
                    }
                })
            })
            .transpose()?;
        let target = match &requirement.provenance.target {
            wire::RuntimeRequirementTargetV1::Module { module } => {
                RuntimeRequirementTarget::Module(import_stable_module_with_allocation_budget(module, allocations)?)
            }
            wire::RuntimeRequirementTargetV1::Binding { module, name } => RuntimeRequirementTarget::Binding {
                module: import_stable_module_with_allocation_budget(module, allocations)?,
                name: allocations.boxed_str(name).map_err(allocation_error)?,
            },
        };
        runtime_requirements.push(RuntimeDependencyRequirement {
            importer: importer.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            dependency,
            range,
            reason: match requirement.reason {
                wire::RuntimeDependencyReasonV1::WholeModuleImport => RuntimeDependencyReason::WholeModuleImport,
                wire::RuntimeDependencyReasonV1::SelectiveValueImport => RuntimeDependencyReason::SelectiveValueImport,
                wire::RuntimeDependencyReasonV1::ReExport => RuntimeDependencyReason::ReExport,
                wire::RuntimeDependencyReasonV1::RuntimeDeclarationReference => RuntimeDependencyReason::RuntimeDeclarationReference,
            },
            read_target,
            provenance: RuntimeRequirementProvenance {
                consumer: importer,
                producer: match requirement.provenance.producer {
                    wire::RuntimeRequirementProducerV1::DeclarationRead => RuntimeRequirementProducer::DeclarationRead,
                    wire::RuntimeRequirementProducerV1::ImportRead => RuntimeRequirementProducer::ImportRead,
                    wire::RuntimeRequirementProducerV1::CompilerGeneratedRead => RuntimeRequirementProducer::CompilerGeneratedRead,
                    wire::RuntimeRequirementProducerV1::CallableOwner => RuntimeRequirementProducer::CallableOwner,
                    wire::RuntimeRequirementProducerV1::PatternOwner => RuntimeRequirementProducer::PatternOwner,
                    wire::RuntimeRequirementProducerV1::ConformanceTypeOwner => RuntimeRequirementProducer::ConformanceTypeOwner,
                    wire::RuntimeRequirementProducerV1::ContextualReceiverRead => RuntimeRequirementProducer::ContextualReceiverRead,
                    wire::RuntimeRequirementProducerV1::MetadataTypeOwner => RuntimeRequirementProducer::MetadataTypeOwner,
                    wire::RuntimeRequirementProducerV1::StaticConstantRead => RuntimeRequirementProducer::StaticConstantRead,
                },
                target,
                product: match requirement.provenance.product {
                    wire::RuntimeRequirementProductV1::CanonicalIdentity => RuntimeRequirementProduct::CanonicalIdentity,
                    wire::RuntimeRequirementProductV1::MetadataDescriptor => RuntimeRequirementProduct::MetadataDescriptor,
                    wire::RuntimeRequirementProductV1::RuntimeObject => RuntimeRequirementProduct::RuntimeObject,
                    wire::RuntimeRequirementProductV1::SourceInitializedValue => RuntimeRequirementProduct::SourceInitializedValue,
                },
                timing: match requirement.provenance.timing {
                    wire::RuntimeRequirementTimingV1::BeforeSourceExecution => RuntimeRequirementTiming::BeforeSourceExecution,
                    wire::RuntimeRequirementTimingV1::DuringModuleInitializer => RuntimeRequirementTiming::DuringModuleInitializer,
                    wire::RuntimeRequirementTimingV1::AfterBootstrap => RuntimeRequirementTiming::AfterBootstrap,
                },
            },
        });
    }
    Ok((phase_products, runtime_requirements))
}

pub(super) fn import_callable_analysis_records(
    records: &[phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2],
    relocation: &TypeRelocation,
    store: &TypeStore,
    source_plan_fingerprints: &[(crate::identity::ImplId, crate::db::ProductFingerprint)],
    module_world: Option<&phalcom_compiler_artifact::module_world::ModuleWorldV2>,
    import_phase_products: &impl phalcom_modules::linker::ModuleImportPhaseProductLookup,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<
    (
        crate::checker::analysis::CallableAnalysisTable,
        crate::parent_map::ParentMap<CallableId, crate::ordered_map::OrderedMap<BindingId, SourceRange>>,
    ),
    ArtifactError,
> {
    use crate::checker::analysis::{BodyExitFacts, CallableAnalysis, ExpressionAnalysis, NormalReturnFact};
    use crate::identity::{BodyId, ExpressionId, LocalExpressionId};

    ensure_complete_callable_analysis_payload(records)?;
    let mut child = allocations.child(allocations.remaining());
    let mut diagnostic_previous_observed = phalcom_type_meta::allocation::current_observed_bytes().unwrap_or(0);
    let mut diagnostic_previous_ledger = child.requested();
    let mut rows = child.vector(records.len()).map_err(allocation_error)?;
    let mut binding_range_entries = child.vector(records.len()).map_err(allocation_error)?;
    record_callable_import_allocation_diagnostic_stage(0, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
    for (body_index, record) in records.iter().enumerate() {
        let callable = import_stable_callable_with_allocation_budget(&record.callable, &mut child)?;
        let observed_import_phases = match module_world {
            Some(module_world) => import_callable_observed_import_phases(record, &callable, module_world, import_phase_products, &mut child)?,
            None if record.observed_import_phases.is_empty() => crate::checker::analysis_map::AnalysisMap::new(),
            None => return Err(referential("module world is required to resolve callable source-import phase authority")),
        };
        let body = BodyId(u32::try_from(body_index).map_err(|_| referential("callable body ordinal exceeds portable range"))?);

        let imported_bindings = import_callable_binding_rows(record, relocation, &mut child)?;
        let mut binding_entries = child.vector(imported_bindings.len()).map_err(allocation_error)?;
        let mut origin_entries = child.vector(imported_bindings.len()).map_err(allocation_error)?;
        let mut formal_range_entries = child.vector(imported_bindings.len()).map_err(allocation_error)?;
        for imported in imported_bindings {
            let binding = imported.state.binding;
            if let Some(range) = imported.formal_range {
                formal_range_entries.push((binding, range));
            }
            origin_entries.push((binding, imported.storage_origin));
            binding_entries.push((binding, imported.state));
        }
        let bindings = import_analysis_map(binding_entries, &mut child, "callable binding rows are not canonical")?;
        let binding_origins = import_analysis_map(origin_entries, &mut child, "callable binding origins are not canonical")?;
        let formal_ranges =
            crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(formal_range_entries, &mut child).map_err(|error| match error {
                crate::ordered_map::FrozenOrderedMapError::Allocation(error) => allocation_error(error),
                crate::ordered_map::FrozenOrderedMapError::DuplicateKey => referential("callable binding source range is duplicated"),
            })?;
        binding_range_entries.push((callable.clone_with_allocation_budget(&mut child).map_err(allocation_error)?, formal_ranges));
        record_callable_import_allocation_diagnostic_stage(1, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);

        let facts = import_callable_expression_facts(record, relocation, &mut child)?;
        let statuses = import_callable_expression_statuses(&record.expressions, relocation, &mut child)?;
        record_callable_import_allocation_diagnostic_stage(2, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let dispatch = import_callable_expression_dispatch_authority(
            record,
            body,
            relocation,
            store,
            source_plan_fingerprints,
            &mut child,
            limits.max_recursion_depth,
        )?;
        record_callable_import_allocation_diagnostic_stage(3, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let specializations = import_callable_call_specializations(
            &record.expressions,
            &record.conformance_evidence,
            relocation,
            store,
            source_plan_fingerprints,
            &mut child,
            limits.max_recursion_depth,
        )?;
        record_callable_import_allocation_diagnostic_stage(4, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let denotations = import_callable_denotations(record, relocation, &mut child)?;
        record_callable_import_allocation_diagnostic_stage(5, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let mut specializations = specializations.into_iter().peekable();
        let mut denotations = denotations.expressions.into_iter().peekable();
        let mut expression_entries = child.vector(record.expressions.len()).map_err(allocation_error)?;
        for (((wire, fact), (_, status)), authority) in record.expressions.iter().zip(facts).zip(statuses).zip(dispatch) {
            if wire.ordinal != fact.ordinal || wire.ordinal != authority.ordinal {
                return Err(referential("callable expression helper ordinals disagree"));
            }
            let id = ExpressionId::new(body, LocalExpressionId(wire.ordinal));
            expression_entries.push((
                id,
                ExpressionAnalysis {
                    id,
                    range: fact.range,
                    knowledge: fact.knowledge,
                    callable: fact.callable,
                    call_specialization: take_sparse(&mut specializations, wire.ordinal),
                    conditional_dispatch: fact.conditional_dispatch,
                    conditional_capability: authority.conditional_capability,
                    trait_dispatch: authority.trait_dispatch,
                    trait_dispatch_candidates: authority.trait_dispatch_candidates,
                    contextual_realization: authority.contextual_realization,
                    denotation: take_sparse(&mut denotations, wire.ordinal),
                    status,
                    causal_invalidity: fact.causal_invalidity,
                    explanation: authority.explanation,
                    call: None,
                    binding_read_safety: fact.binding_read_safety,
                    binding_read: fact.binding_read.map(BindingId),
                    binding_write: fact.binding_write.map(BindingId),
                },
            ));
        }
        if specializations.next().is_some() || denotations.next().is_some() {
            return Err(referential("callable sparse expression authority references an absent expression"));
        }
        let expressions = import_analysis_map(expression_entries, &mut child, "callable expression rows are not canonical")?;
        record_callable_import_allocation_diagnostic_stage(6, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let resolutions = import_callable_resolution_authority(record, body, relocation, store, source_plan_fingerprints, limits, &mut child)?;
        record_callable_import_allocation_diagnostic_stage(7, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let proofs = proof::import_callable_proofs_v2(record, body, relocation, limits, &mut child)?;
        record_callable_import_allocation_diagnostic_stage(8, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let flow_graph = import_callable_flow_graph_with_allocation_budget(&record.flow_graph, limits, &mut child)?;
        let entry_flow = import_callable_flow_state_summary(&record.entry_flow, relocation, &mut child)?;
        record_callable_import_allocation_diagnostic_stage(9, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let mut normal_returns = child.vector(record.normal_returns.len()).map_err(allocation_error)?;
        for exit in &record.normal_returns {
            let status = import_callable_status(&exit.status, relocation, &mut child)?;
            normal_returns.push(NormalReturnFact {
                knowledge: import_type_knowledge_with_allocation_budget(&exit.knowledge, relocation, &mut child)?,
                flow: import_callable_flow_state_summary(&exit.flow, relocation, &mut child)?,
                status,
                causal_invalidity: match exit.causal_invalidity {
                    phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean => crate::checker::causal::CausalInvalidity::Clean,
                    phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::One { cause } => {
                        crate::checker::causal::CausalInvalidity::One(DiagnosticCauseId(cause))
                    }
                    phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Multiple => crate::checker::causal::CausalInvalidity::Multiple,
                },
                direct_result_dependency: exit
                    .direct_result_dependency
                    .as_ref()
                    .map(|dependency| import_stable_callable_with_allocation_budget(dependency, &mut child))
                    .transpose()?,
            });
        }
        let mut throws = child.vector(record.throws.len()).map_err(allocation_error)?;
        for flow in &record.throws {
            throws.push(import_callable_flow_state_summary(flow, relocation, &mut child)?);
        }
        let body_source_owner = record
            .body_source_owner
            .as_ref()
            .map(|owner| match owner {
                phalcom_compiler_artifact::semantic::world::CallableAnalysisSourceOwnerV1::Module { module } => {
                    Ok::<SourceOwner, ArtifactError>(SourceOwner::Module(import_stable_module_with_allocation_budget(module, &mut child)?))
                }
                phalcom_compiler_artifact::semantic::world::CallableAnalysisSourceOwnerV1::Callable { callable } => {
                    Ok::<SourceOwner, ArtifactError>(SourceOwner::Callable(import_stable_callable_with_allocation_budget(callable, &mut child)?))
                }
            })
            .transpose()?;
        let body_range = SourceRange {
            start: usize::try_from(record.body_range.start).map_err(|_| referential("callable body start exceeds address space"))?,
            end: usize::try_from(record.body_range.end).map_err(|_| referential("callable body end exceeds address space"))?,
        };
        let constructor_super_initializer = record
            .constructor_super_initializer
            .as_ref()
            .map(|resolution| {
                Ok::<crate::checker::analysis::ConstructorSuperCallResolution, ArtifactError>(crate::checker::analysis::ConstructorSuperCallResolution {
                    call_range: SourceRange {
                        start: usize::try_from(resolution.call_range.start).map_err(|_| referential("constructor super-call start exceeds address space"))?,
                        end: usize::try_from(resolution.call_range.end).map_err(|_| referential("constructor super-call end exceeds address space"))?,
                    },
                    initializer: import_stable_callable_with_allocation_budget(&resolution.initializer, &mut child)?,
                })
            })
            .transpose()?;
        record_callable_import_allocation_diagnostic_stage(10, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let explanations = explanation::import_explanations(&record.explanations, body, relocation, &mut child)?;
        let mut diagnostics = child.vector(record.diagnostics.len()).map_err(allocation_error)?;
        for diagnostic in &record.diagnostics {
            diagnostics.push(diagnostic::import_diagnostic_with_allocation_budget(diagnostic, relocation, &mut child)?);
        }
        let diagnostics = budgeted_arc_slice(diagnostics, &mut child)?;
        record_callable_import_allocation_diagnostic_stage(11, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let mut dependencies = child.vector(record.dependencies.len()).map_err(allocation_error)?;
        for dependency in &record.dependencies {
            dependencies.push(import_stable_callable_with_allocation_budget(dependency, &mut child)?);
        }
        let dependencies = budgeted_arc_slice(dependencies, &mut child)?;
        let mut semantic_dependencies = child.vector(record.semantic_dependencies.len()).map_err(allocation_error)?;
        for dependency in &record.semantic_dependencies {
            semantic_dependencies.push(import_static_semantic_dependency(dependency, relocation, &mut child)?);
        }
        let semantic_dependencies = budgeted_arc_slice(semantic_dependencies, &mut child)?;
        record_callable_import_allocation_diagnostic_stage(12, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        let mut analysis = CallableAnalysis {
            callable: callable.clone_with_allocation_budget(&mut child).map_err(allocation_error)?,
            body_range,
            body_source_owner,
            expressions,
            bindings,
            binding_origins,
            for_iteration_lanes: resolutions.for_iteration_lanes,
            positional_expansions: resolutions.positional_expansions,
            known_invalid_positional_expansions: resolutions.known_invalid_positional_expansions,
            sequence_literals: resolutions.sequence_literals,
            associated_resolutions: resolutions.associated_resolutions,
            callable_reference_resolutions: resolutions.callable_reference_resolutions,
            family_applications: resolutions.family_applications,
            match_resolutions: proofs.matches,
            pattern_site_resolutions: proofs.sites,
            local_proof_graph: proofs.graph,
            flow_graph,
            entry_flow,
            exits: BodyExitFacts {
                normal_returns,
                throws,
                unreachable: record.exits_unreachable,
            },
            return_validation: import_callable_return_validation(&record.return_validation),
            diagnostics,
            internal_incidents: budgeted_arc_slice(Vec::new(), &mut child)?,
            explanations,
            return_explanation: record.return_explanation.map(ExplanationId),
            dependencies,
            semantic_dependencies,
            observed_import_phases: child.arc(observed_import_phases).map_err(allocation_error)?,
            constructor_super_initializer,
            dependency_fingerprint: crate::db::ProductFingerprint::new(0),
            status: import_callable_analysis_status(&record.status),
        };
        record_callable_import_allocation_diagnostic_stage(13, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        analysis.dependency_fingerprint = crate::db::fingerprint::callable_body_product_fingerprint(&analysis);
        record_callable_import_allocation_diagnostic_stage(14, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
        rows.push((callable, child.arc(analysis).map_err(allocation_error)?));
    }
    let table = crate::checker::analysis::CallableAnalysisTable::from_budgeted_entries(rows, &mut child).map_err(|error| match error {
        crate::checker::analysis::CallableAnalysisTableBuildError::DuplicateCallable => referential("callable analysis owner is duplicated after relocation"),
        crate::checker::analysis::CallableAnalysisTableBuildError::Allocation(error) => allocation_error(error),
        crate::checker::analysis::CallableAnalysisTableBuildError::ModuleIndex(_) => referential("callable analysis module index is not canonical"),
    })?;
    let binding_ranges =
        crate::parent_map::ParentMap::from_frozen_entries_with_allocation_budget(binding_range_entries, &mut child).map_err(|error| match error {
            crate::parent_map::FrozenParentMapError::Allocation(error) => allocation_error(error),
            crate::parent_map::FrozenParentMapError::DuplicateKey => referential("callable binding range owner is duplicated"),
        })?;
    record_callable_import_allocation_diagnostic_stage(15, &child, &mut diagnostic_previous_observed, &mut diagnostic_previous_ledger);
    allocations.commit_child(child).map_err(allocation_error)?;
    Ok((table, binding_ranges))
}

pub(super) struct ImportedAssociatedAuthority {
    pub associated_surfaces: crate::associated::AssociatedFamilyTable,
    pub type_member_declarations:
        crate::parent_map::ParentMap<crate::types::type_member::TypeMemberDeclarationId, Arc<crate::db::product::TypeMemberDeclarationProduct>>,
    pub associated_static_domains:
        crate::ordered_map::OrderedMap<crate::identity::AssociatedStaticDomainDeclarationId, Arc<crate::db::product::AssociatedStaticDomainDeclarationProduct>>,
}

pub(super) fn import_associated_authority(
    type_member_records: &[TypeMemberDeclarationRecordV1],
    static_domain_records: &[AssociatedStaticDomainRecordV1],
    surface_records: &[AssociatedSurfaceRecordV1],
    type_relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<ImportedAssociatedAuthority, ArtifactError> {
    let mut type_member_entries = allocations.vector(type_member_records.len()).map_err(allocation_error)?;
    for rec in type_member_records {
        let owner = import_stable_declaration_with_allocation_budget(&rec.owner, allocations)?;
        let selector = import_type_member_selector_with_allocation_budget(&rec.selector, allocations)?;
        let id_selector = import_type_member_selector_with_allocation_budget(&rec.selector, allocations)?;
        let id = crate::types::type_member::TypeMemberDeclarationId::new(owner, id_selector);
        let mut parameters = allocations
            .vector::<crate::types::type_member::TypeMemberParameter>(rec.parameters.len())
            .map_err(allocation_error)?;
        for parameter in &rec.parameters {
            parameters.push(crate::types::type_member::TypeMemberParameter {
                name: allocations.boxed_str(&parameter.name).map_err(allocation_error)?,
                domain: import_type_member_domain(&parameter.domain, type_relocation)?,
            });
        }
        let key = crate::types::type_member::TypeMemberDeclarationId::new(
            import_stable_declaration_with_allocation_budget(&rec.owner, allocations)?,
            import_type_member_selector_with_allocation_budget(&rec.selector, allocations)?,
        );
        let product = crate::db::product::TypeMemberDeclarationProduct {
            id,
            signature: crate::types::type_member::TypeMemberSignature {
                selector,
                parameters: allocations.boxed_slice(parameters).map_err(allocation_error)?,
                result_domain: import_type_member_domain(&rec.result_domain, type_relocation)?,
                intrinsic: rec.intrinsic.as_ref().map(import_intrinsic_type_operation),
                implementation: import_type_member_implementation(&rec.implementation),
            },
            source_range: phalcom_common::range::SourceRange::default(),
        };
        let product = allocations.arc(product).map_err(allocation_error)?;
        type_member_entries.push((key, product));
    }
    let type_member_declarations =
        crate::parent_map::ParentMap::from_frozen_entries_with_allocation_budget(type_member_entries, allocations).map_err(|error| match error {
            crate::parent_map::FrozenParentMapError::Allocation(error) => allocation_error(error),
            crate::parent_map::FrozenParentMapError::DuplicateKey => referential("duplicate type-member declaration in semantic artifact"),
        })?;

    let mut static_domain_entries = allocations.vector(static_domain_records.len()).map_err(allocation_error)?;
    for rec in static_domain_records {
        let id = import_associated_static_domain_declaration_with_allocation_budget(&rec.declaration, allocations)?;
        let key = import_associated_static_domain_declaration_with_allocation_budget(&rec.declaration, allocations)?;
        let domain = type_relocation.domain(rec.domain)?;
        let mut constructors = allocations
            .vector::<crate::types::id::AlgebraicStaticConstructorId>(rec.constructors.len())
            .map_err(allocation_error)?;
        for &constructor in &rec.constructors {
            constructors.push(type_relocation.constructor(constructor)?);
        }
        let product = crate::db::product::AssociatedStaticDomainDeclarationProduct {
            id,
            domain,
            constructors: allocations.boxed_slice(constructors).map_err(allocation_error)?,
            source_range: phalcom_common::range::SourceRange::default(),
        };
        let product = allocations.arc(product).map_err(allocation_error)?;
        static_domain_entries.push((key, product));
    }
    let associated_static_domains =
        crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(static_domain_entries, allocations).map_err(frozen_ordered_map_error)?;

    let mut surface_entries = allocations.vector(surface_records.len()).map_err(allocation_error)?;
    for rec in surface_records {
        let owner = import_stable_declaration_with_allocation_budget(&rec.owner, allocations)?;
        let surface_owner = owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        let mut family_entries = allocations.vector(rec.families.len()).map_err(allocation_error)?;
        for family in &rec.families {
            let base = import_associated_selector_with_allocation_budget(&family.selector, allocations)?;
            let family_base = import_associated_selector_with_allocation_budget(&family.selector, allocations)?;
            let mut members = allocations
                .vector::<crate::associated::AssociatedMemberId>(family.members.len())
                .map_err(allocation_error)?;
            for member in &family.members {
                members.push(import_associated_member_with_allocation_budget(member, allocations)?);
            }
            let family_info = crate::associated::AssociatedFamilyInfo {
                id: crate::identity::AssociatedFamilyId::new(owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?, family_base),
                kind: import_associated_family_kind(&family.kind),
                members: allocations.boxed_slice(members).map_err(allocation_error)?,
            };
            family_entries.push((base, family_info));
        }
        let families =
            crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(family_entries, allocations).map_err(frozen_ordered_map_error)?;
        let surface = crate::associated::AssociatedSurface {
            owner: surface_owner,
            families,
        };
        let surface = allocations.arc(surface).map_err(allocation_error)?;
        surface_entries.push((owner, surface));
    }
    let surfaces = crate::parent_map::ParentMap::from_frozen_entries_with_allocation_budget(surface_entries, allocations).map_err(|error| match error {
        crate::parent_map::FrozenParentMapError::Allocation(error) => allocation_error(error),
        crate::parent_map::FrozenParentMapError::DuplicateKey => referential("duplicate associated surface in semantic artifact"),
    })?;
    let associated_surfaces = crate::associated::AssociatedFamilyTable {
        surfaces: allocations.arc(surfaces).map_err(allocation_error)?,
    };

    Ok(ImportedAssociatedAuthority {
        associated_surfaces,
        type_member_declarations,
        associated_static_domains,
    })
}

#[derive(Debug, Default)]
pub struct ReconstructedModuleOwnerIndexes {
    pub declarations_by_module: crate::ordered_map::OrderedMap<String, Vec<StableDeclarationRef>>,
    pub data_by_module: crate::ordered_map::OrderedMap<String, Vec<StableDeclarationRef>>,
    pub enums_by_module: crate::ordered_map::OrderedMap<String, Vec<StableDeclarationRef>>,
}

fn import_module_owner_index(
    mut declarations: Vec<StableDeclarationRef>,
    allocations: &mut AllocationBudget,
) -> Result<crate::ordered_map::OrderedMap<String, Vec<StableDeclarationRef>>, ArtifactError> {
    declarations.sort_unstable();
    let count = declarations.chunk_by(|left, right| left.module == right.module).count();
    let mut entries = allocations.vector(count).map_err(allocation_error)?;
    let mut declarations = declarations.into_iter();
    while let Some(first) = declarations.as_slice().first() {
        let count = declarations.as_slice().partition_point(|declaration| declaration.module == first.module);
        let key = String::from(debug_boxed_str_with_allocation_budget(&first.module, allocations)?);
        let mut owners = allocations.vector(count).map_err(allocation_error)?;
        owners.extend(declarations.by_ref().take(count));
        entries.push((key, owners));
    }
    crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(entries, allocations).map_err(frozen_ordered_map_error)
}

fn import_static_definition_owners(
    records: &[phalcom_compiler_artifact::semantic::world::StaticDefinitionRecordV1],
    structural_contracts: &LoadedStructuralContracts,
    static_relocation: &StaticRelocation,
    type_relocation: &TypeRelocation,
    type_store: &TypeStore,
    allocations: &mut AllocationBudget,
) -> Result<crate::static_definition::StaticDefinitionTable, ArtifactError> {
    let mut entries = allocations.vector(records.len()).map_err(allocation_error)?;
    for record in records {
        let id = import_stable_declaration_with_allocation_budget(&record.id, allocations)?;
        let signature = crate::artifact::operational::import_static_definition_signature(&record.signature, type_relocation, type_store, allocations)?;
        let mut dependencies = allocations
            .vector::<crate::checker::analysis::SemanticDependency>(record.canonical_program.dependencies.len())
            .map_err(allocation_error)?;
        for dependency in &record.canonical_program.dependencies {
            dependencies.push(import_static_semantic_dependency(dependency, type_relocation, allocations)?);
        }
        let program = crate::artifact::operational::import_canonical_static_program(
            &record.canonical_program,
            static_relocation,
            type_relocation,
            type_store,
            dependencies,
            allocations,
        )?;
        if program.owner.0 != id {
            return Err(referential("imported static program owner disagrees with its definition record"));
        }
        let program = allocations.arc(program).map_err(allocation_error)?;
        let body = crate::static_definition::StaticDefinitionBodyRepr::Operational(program.clone());
        let mut diagnostics = allocations.vector(record.diagnostics.len()).map_err(allocation_error)?;
        for diagnostic in &record.diagnostics {
            diagnostics.push(import_trait_surface_diagnostic(diagnostic, type_relocation, allocations)?);
        }
        let source_range = SourceRange {
            start: usize::try_from(record.source.start).map_err(|_| referential("static definition source start exceeds address space"))?,
            end: usize::try_from(record.source.end).map_err(|_| referential("static definition source end exceeds address space"))?,
        };
        if source_range.start > source_range.end {
            return Err(referential("static definition source range is inverted"));
        }
        let alias_info = match &record.alias_info {
            Some(alias) if alias == &record.id => {
                let alias = structural_contracts
                    .type_aliases
                    .get(&id)
                    .ok_or_else(|| referential("static definition alias link has no imported structural alias owner"))?
                    .clone_with_allocation_budget(allocations)
                    .map_err(allocation_error)?;
                Some(allocations.arc(alias).map_err(allocation_error)?)
            }
            Some(_) => return Err(referential("static definition alias link belongs to another declaration")),
            None => None,
        };
        let key = id.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        let definition = allocations
            .arc(crate::static_definition::StaticDefinition {
                id,
                signature,
                body,
                source_range,
            })
            .map_err(allocation_error)?;
        let diagnostics = allocations.arc_slice(diagnostics).map_err(allocation_error)?;
        let product = allocations
            .arc(crate::static_definition::StaticDefinitionProduct {
                definition,
                diagnostics,
                alias_info,
                canonical_program: Some(program),
                checked_body: None,
            })
            .map_err(allocation_error)?;
        entries.push((key, product));
    }
    crate::static_definition::StaticDefinitionTable::from_frozen_entries_with_allocation_budget(entries, allocations).map_err(frozen_ordered_map_error)
}

fn import_static_constant_symbol(
    symbol: &phalcom_compiler_artifact::semantic::world::StaticConstantSymbolV1,
    allocations: &mut AllocationBudget,
) -> Result<phalcom_modules::linker::SymbolId, ArtifactError> {
    Ok(phalcom_modules::linker::SymbolId {
        module: import_stable_module_with_allocation_budget(&symbol.module, allocations)?,
        name: allocations.boxed_str(&symbol.name).map_err(allocation_error)?,
    })
}

fn clone_static_symbol_with_allocation_budget(
    symbol: &phalcom_modules::linker::SymbolId,
    allocations: &mut AllocationBudget,
) -> Result<phalcom_modules::linker::SymbolId, ArtifactError> {
    Ok(phalcom_modules::linker::SymbolId {
        module: symbol.module.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
        name: allocations.boxed_str(&symbol.name).map_err(allocation_error)?,
    })
}

fn import_static_constant_owners(
    records: &[phalcom_compiler_artifact::semantic::world::StaticConstantRecordV1],
    static_relocation: &StaticRelocation,
    type_relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::static_constant::StaticConstantTable, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::StaticConstantOutcomeV1;
    let mut entries = allocations.vector(records.len()).map_err(allocation_error)?;
    for record in records {
        let symbol = import_static_constant_symbol(&record.symbol, allocations)?;
        let id = crate::static_constant::StaticConstantId(symbol);
        let value = match &record.outcome {
            StaticConstantOutcomeV1::Reduced { value } => static_relocation.value(*value)?,
        };
        let mut dependencies = allocations
            .vector::<crate::checker::analysis::SemanticDependency>(record.dependencies.len())
            .map_err(allocation_error)?;
        for dependency in &record.dependencies {
            dependencies.push(import_static_semantic_dependency(dependency, type_relocation, allocations)?);
        }
        let mut static_constant_reads = allocations
            .vector::<phalcom_modules::linker::SymbolId>(record.static_constant_reads.len())
            .map_err(allocation_error)?;
        for read in &record.static_constant_reads {
            static_constant_reads.push(import_static_constant_symbol(read, allocations)?);
        }
        let mut record_key_static_constant_reads = allocations
            .vector::<phalcom_modules::linker::SymbolId>(record.record_key_static_constant_reads.len())
            .map_err(allocation_error)?;
        for read in &record.record_key_static_constant_reads {
            record_key_static_constant_reads.push(import_static_constant_symbol(read, allocations)?);
        }
        let authority = crate::static_constant::StaticConstantAuthority {
            input_fingerprint: crate::db::InputFingerprint::new(record.input_fingerprint),
            dependencies: allocations.arc_slice(dependencies).map_err(allocation_error)?,
            static_constant_reads: allocations.arc_slice(static_constant_reads).map_err(allocation_error)?,
            record_key_static_constant_reads: allocations.arc_slice(record_key_static_constant_reads).map_err(allocation_error)?,
            result_type: record.result_type.map(|ty| type_relocation.ty(ty)).transpose()?,
        };
        let declaration_range = SourceRange {
            start: usize::try_from(record.declaration_source.start).map_err(|_| referential("static constant declaration start exceeds address space"))?,
            end: usize::try_from(record.declaration_source.end).map_err(|_| referential("static constant declaration end exceeds address space"))?,
        };
        let initializer_range = SourceRange {
            start: usize::try_from(record.initializer_source.start).map_err(|_| referential("static constant initializer start exceeds address space"))?,
            end: usize::try_from(record.initializer_source.end).map_err(|_| referential("static constant initializer end exceeds address space"))?,
        };
        if declaration_range.start > declaration_range.end || initializer_range.start > initializer_range.end {
            return Err(referential("static constant source range is inverted"));
        }
        let lkg_value = record.lkg_value.map(|value| static_relocation.value(value)).transpose()?;
        let mut diagnostics = allocations.vector(record.diagnostics.len()).map_err(allocation_error)?;
        for diagnostic in &record.diagnostics {
            diagnostics.push(import_trait_surface_diagnostic(diagnostic, type_relocation, allocations)?);
        }
        let key = crate::static_constant::StaticConstantId(clone_static_symbol_with_allocation_budget(&id.0, allocations)?);
        let product = crate::static_constant::StaticConstantProduct {
            id,
            declaration_range,
            initializer_range,
            checked: None,
            authority: Some(authority),
            outcome: crate::static_eval::StaticEvalOutcome::Reduced(value),
            residuals: None,
            value: Some(value),
            lkg_value,
            diagnostics: allocations.arc_slice(diagnostics).map_err(allocation_error)?,
            imported: None,
        };
        entries.push((key, allocations.arc(product).map_err(allocation_error)?));
    }
    crate::static_constant::StaticConstantTable::from_frozen_entries_with_allocation_budget(entries, allocations).map_err(frozen_ordered_map_error)
}

fn frozen_ordered_map_error(error: crate::ordered_map::FrozenOrderedMapError) -> ArtifactError {
    match error {
        crate::ordered_map::FrozenOrderedMapError::Allocation(error) => allocation_error(error),
        crate::ordered_map::FrozenOrderedMapError::DuplicateKey => referential("duplicate relocated semantic owner"),
    }
}

fn import_module_static_binding_indexes(
    records: &[phalcom_compiler_artifact::semantic::world::ModuleStaticBindingIndexV1],
    callable_records: &[phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2],
    type_relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<crate::parent_map::ParentMap<crate::identity::ModuleId, Arc<crate::module_bindings::ModuleStaticBindingIndex>>, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::ModuleStaticBindingPhaseV1;
    let body_reference_count = records
        .iter()
        .try_fold(0usize, |count, record| count.checked_add(record.bindings.len()))
        .ok_or_else(|| allocation_error(AllocationError::Overflow))?;
    let mut body_refs = allocations
        .vector::<&phalcom_type_meta::identity::StableCallableRef>(body_reference_count)
        .map_err(allocation_error)?;
    for record in records {
        body_refs.extend(record.bindings.iter().map(|binding| &binding.source_body));
    }
    body_refs.sort_unstable();
    body_refs.dedup();
    let mut rows_by_body = allocations
        .vector::<(&phalcom_type_meta::identity::StableCallableRef, Vec<ImportedCallableBindingRow>)>(body_refs.len())
        .map_err(allocation_error)?;
    for body in body_refs {
        let source_record = callable_records
            .binary_search_by(|candidate| candidate.callable.cmp(body))
            .ok()
            .map(|position| &callable_records[position])
            .ok_or_else(|| referential("module static binding references an absent callable-analysis owner"))?;
        rows_by_body.push((body, import_callable_binding_rows(source_record, type_relocation, allocations)?));
    }
    let mut indexes = allocations
        .vector::<(crate::identity::ModuleId, Arc<crate::module_bindings::ModuleStaticBindingIndex>)>(records.len())
        .map_err(allocation_error)?;
    for record in records {
        let module = import_stable_module_with_allocation_budget(&record.module, allocations)?;
        let mut binding_entries = allocations
            .vector::<(phalcom_modules::linker::SymbolId, crate::module_bindings::ModuleBindingFact)>(record.bindings.len())
            .map_err(allocation_error)?;
        for binding in &record.bindings {
            let body_index = rows_by_body
                .binary_search_by(|(body, _)| (**body).cmp(&binding.source_body))
                .map_err(|_| referential("module static binding callable rows were not reconstructed"))?;
            let imported_body = &rows_by_body[body_index].1;
            let row = imported_body
                .get(binding.source_binding_ordinal as usize)
                .ok_or_else(|| referential("module static binding ordinal is out of range"))?;
            if row.state.binding.0 != binding.source_binding_ordinal
                || !matches!(&row.storage_origin, crate::checker::analysis::CallableBindingOrigin::BodyLocal)
            {
                return Err(referential("module static binding does not resolve to its body-local binding owner"));
            }
            let symbol = import_static_constant_symbol(&binding.symbol, allocations)?;
            let symbol_for_entry = clone_static_symbol_with_allocation_budget(&symbol, allocations)?;
            let fact = crate::module_bindings::ModuleBindingFact {
                symbol,
                source_body: import_stable_callable_with_allocation_budget(&binding.source_body, allocations)?,
                source_binding: row.state.binding,
                phase: match binding.phase {
                    ModuleStaticBindingPhaseV1::StaticConstant => crate::module_bindings::ModuleBindingPhase::StaticConstant,
                    ModuleStaticBindingPhaseV1::Runtime => crate::module_bindings::ModuleBindingPhase::Runtime,
                },
                state: row.state.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                declaration_range: SourceRange {
                    start: usize::try_from(binding.declaration_source.start)
                        .map_err(|_| referential("module static-binding declaration start exceeds address space"))?,
                    end: usize::try_from(binding.declaration_source.end)
                        .map_err(|_| referential("module static-binding declaration end exceeds address space"))?,
                },
                initializer_range: binding
                    .initializer_source
                    .as_ref()
                    .map(|source| -> Result<SourceRange, ArtifactError> {
                        Ok(SourceRange {
                            start: usize::try_from(source.start).map_err(|_| referential("module static-binding initializer start exceeds address space"))?,
                            end: usize::try_from(source.end).map_err(|_| referential("module static-binding initializer end exceeds address space"))?,
                        })
                    })
                    .transpose()?,
            };
            if fact.declaration_range.start > fact.declaration_range.end || fact.initializer_range.is_some_and(|range| range.start > range.end) {
                return Err(referential("module static-binding owner is duplicate or has an invalid range"));
            }
            binding_entries.push((symbol_for_entry, fact));
        }
        let mut visible_entries = allocations
            .vector::<(Box<str>, phalcom_modules::linker::SymbolId)>(record.visible_names.len())
            .map_err(allocation_error)?;
        for visible in &record.visible_names {
            let symbol = import_static_constant_symbol(&visible.symbol, allocations)?;
            let name = allocations.boxed_str(&visible.name).map_err(allocation_error)?;
            visible_entries.push((name, symbol));
        }
        let bindings =
            crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(binding_entries, allocations).map_err(|error| match error {
                crate::ordered_map::FrozenOrderedMapError::Allocation(error) => allocation_error(error),
                crate::ordered_map::FrozenOrderedMapError::DuplicateKey => referential("module static-binding owner is duplicated"),
            })?;
        let visible_names =
            crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(visible_entries, allocations).map_err(|error| match error {
                crate::ordered_map::FrozenOrderedMapError::Allocation(error) => allocation_error(error),
                crate::ordered_map::FrozenOrderedMapError::DuplicateKey => referential("module visible static-binding name is duplicated"),
            })?;
        let module_for_index = module.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        let index =
            crate::module_bindings::ModuleStaticBindingIndex::from_frozen_artifact_parts(module_for_index, bindings, visible_names).map_err(referential)?;
        indexes.push((module, allocations.arc(index).map_err(allocation_error)?));
    }
    crate::parent_map::ParentMap::from_frozen_entries_with_allocation_budget(indexes, allocations).map_err(|error| match error {
        crate::parent_map::FrozenParentMapError::Allocation(error) => allocation_error(error),
        crate::parent_map::FrozenParentMapError::DuplicateKey => referential("module static-binding index owner is duplicated"),
    })
}

pub struct ValidatedSemanticParts {
    pub structural_contracts: LoadedStructuralContracts,
    pub type_store: Arc<TypeStore>,
    pub type_relocation: TypeRelocation,
    pub static_values: Arc<StaticValueStore>,
    pub static_relocation: StaticRelocation,
    pub owner_indexes: ReconstructedModuleOwnerIndexes,
    pub snapshot: Arc<crate::snapshot::SemanticSnapshot>,
}

/// Executes stages 3–7 of the P7 loader construction order (`T4/T5` -> `T7.A` -> `T6` -> `T7.B` -> `T8`).
pub fn validate_and_import_semantic_world(world: SemanticWorldV4, limits: &ValidationLimits) -> Result<ValidatedSemanticParts, ArtifactError> {
    let budget_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = AllocationBudget::new(budget_limit);
    validate_and_import_semantic_world_with_allocation_budget(world, limits, &mut allocations)
}

/// Imports semantic authority using the caller's cumulative load transaction.
pub fn validate_and_import_semantic_world_with_allocation_budget(
    world: SemanticWorldV4,
    limits: &ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<ValidatedSemanticParts, ArtifactError> {
    validate_and_import_semantic_world_with_optional_module_world(world, None, None, limits, transaction)
}

/// Imports semantic authority while resolving source-import ordinals against
/// the exact module rows that accompany the semantic world.
pub fn validate_and_import_semantic_world_with_module_world_and_allocation_budget(
    world: SemanticWorldV4,
    module_world: &phalcom_compiler_artifact::module_world::ModuleWorldV2,
    limits: &ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<ValidatedSemanticParts, ArtifactError> {
    let projects = phalcom_modules::ProjectUniverse::new();
    validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget(world, module_world, &projects, limits, transaction)
}

/// Imports semantic authority while resolving every project-qualified owner
/// against the exact project graph retained by the loaded module world.
pub fn validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget(
    world: SemanticWorldV4,
    module_world: &phalcom_compiler_artifact::module_world::ModuleWorldV2,
    projects: &phalcom_modules::ProjectUniverse,
    limits: &ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<ValidatedSemanticParts, ArtifactError> {
    validate_and_import_semantic_world_with_optional_module_world(world, Some(module_world), Some(projects), limits, transaction)
}

/// Runs semantic reconstruction with stage reporting available to focused
/// allocation diagnostics. This API exists only with the feature enabled.
#[cfg(feature = "allocation-diagnostics")]
pub fn validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget_for_diagnostics(
    world: SemanticWorldV4,
    module_world: &phalcom_compiler_artifact::module_world::ModuleWorldV2,
    projects: &phalcom_modules::ProjectUniverse,
    limits: &ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<ValidatedSemanticParts, ArtifactError> {
    validate_and_import_semantic_world_with_optional_module_world(world, Some(module_world), Some(projects), limits, transaction)
}

fn validate_and_import_semantic_world_with_optional_module_world(
    world: SemanticWorldV4,
    module_world: Option<&phalcom_compiler_artifact::module_world::ModuleWorldV2>,
    projects: Option<&phalcom_modules::ProjectUniverse>,
    limits: &ValidationLimits,
    transaction: &mut AllocationBudget,
) -> Result<ValidatedSemanticParts, ArtifactError> {
    let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
    let mut allocations = transaction.child(section_limit);
    let parts = validate_and_import_semantic_world_inner(world, module_world, projects, limits, &mut allocations)?;
    transaction.commit_child(allocations).map_err(allocation_error)?;
    Ok(parts)
}

fn allocation_error(error: AllocationError) -> ArtifactError {
    // Keep budget failure reporting allocation-free. `LimitExceeded(String)`
    // would request a new owned string precisely when the budget may already
    // be exhausted; the typed allocation variant carries the cause directly.
    ArtifactError::Allocation(error)
}

fn validate_and_import_semantic_world_inner(
    world: SemanticWorldV4,
    module_world: Option<&phalcom_compiler_artifact::module_world::ModuleWorldV2>,
    projects: Option<&phalcom_modules::ProjectUniverse>,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ValidatedSemanticParts, ArtifactError> {
    record_import_allocation_diagnostic_stage(0, allocations);
    world.validate_with_allocation_budget(limits, allocations)?;
    ensure_complete_callable_analysis_payload(&world.callable_analyses)?;
    record_import_allocation_diagnostic_stage(1, allocations);
    let (import_phase_products, generated_runtime_requirements) = match module_world {
        Some(module_world) => import_module_execution_authority(&world.module_execution_authority, module_world, allocations)?,
        None if world.module_execution_authority.import_phase_products.is_empty()
            && world.module_execution_authority.runtime_requirements.is_empty()
            && world.callable_analyses.iter().all(|analysis| analysis.observed_import_phases.is_empty()) =>
        {
            (phalcom_modules::authority::ModuleAuthorityMap::new(), Vec::new())
        }
        None => return Err(referential("module world is required to resolve source-import phase authority")),
    };
    let has_intrinsic_projection_rows =
        !world.structural_contracts.intrinsic_static_domains.is_empty() || !world.structural_contracts.intrinsic_type_owners.is_empty();
    // Project-qualified type graph nodes require the same exact owner catalog
    // as intrinsic projection rows. Do not resolve them from display names or
    // construct a synthetic project identity during import.
    let owner_catalog = match (module_world, projects) {
        (Some(module_world), Some(projects)) => Some(StableOwnerCatalog::new(module_world, projects)?),
        (None, None) if !has_intrinsic_projection_rows => None,
        _ => return Err(referential("module world and project universe are required to resolve artifact semantic owners")),
    };
    record_import_allocation_diagnostic_stage(2, allocations);
    // 3. Fresh TypeStore with checked TypeRelocation
    let (mut type_store, type_relocation) = TypeStore::import_type_graph_with_owner_catalog_and_allocation_budget(
        &world.type_graph, owner_catalog.as_ref(), limits, allocations,
    )?;
    record_import_allocation_diagnostic_stage(3, allocations);

    // 4. Structural primary contracts (shells, hierarchy, aliases, data/enum contracts, signatures)
    let structural_contracts = import_structural_primary_contracts_with_owner_catalog(
        &world.structural_contracts,
        &mut type_store,
        &type_relocation,
        limits,
        allocations,
        owner_catalog.as_ref(),
    )?;
    record_import_allocation_diagnostic_stage(4, allocations);

    // 5. Fresh StaticValueStore through validated factories
    let (static_values, static_relocation) = import_static_value_graph_with_allocation_budget(
        &world.static_values,
        &type_store,
        &type_relocation,
        &structural_contracts.data_table,
        &structural_contracts.enum_table,
        &|decl_ref, allocations| import_stable_declaration_with_allocation_budget(decl_ref, allocations),
        &|var_ref, allocations| import_stable_variant_with_allocation_budget(var_ref, allocations),
        limits,
        allocations,
    )?;
    record_import_allocation_diagnostic_stage(5, allocations);

    // Cross-validate pending static references
    validate_pending_static_references(&world.structural_contracts, &static_relocation)?;

    // 7. Verify witness plans and layout fingerprints (P7.T8)
    for witness in &world.witness_plans {
        let actual = compute_witness_layout_fingerprint(&witness.ordered_selected_callables);
        if actual != witness.slot_layout_fingerprint {
            return Err(referential("witness plan slot layout fingerprint mismatch"));
        }
    }

    // 8. Reconstruct callable signatures and definitions (P7.T7.B)
    #[cfg(feature = "allocation-diagnostics")]
    let mut callable_table_previous_observed = phalcom_type_meta::allocation::current_observed_bytes().unwrap_or(0);
    #[cfg(feature = "allocation-diagnostics")]
    let mut callable_table_previous_ledger = allocations.requested();
    let mut callable_signature_entries = allocations.vector(world.callable_signatures.len()).map_err(allocation_error)?;
    for rec in &world.callable_signatures {
        let sig = import_callable_signature(rec, &type_relocation, &type_store, allocations)?;
        callable_signature_entries.push(sig);
    }
    let callable_signatures = crate::signature::CallableSignatureTable::from_frozen_entries_with_allocation_budget(callable_signature_entries, allocations)
        .map_err(|error| match error {
            crate::signature::FrozenCallableSignaturesError::Allocation(error) => allocation_error(error),
            crate::signature::FrozenCallableSignaturesError::DuplicateCallable => {
                referential("callable signature owner is duplicated after stable-identity relocation")
            }
        })?;
    #[cfg(feature = "allocation-diagnostics")]
    record_callable_import_allocation_diagnostic_stage(56, allocations, &mut callable_table_previous_observed, &mut callable_table_previous_ledger);
    let mut callable_definition_entries = allocations
        .vector::<(crate::identity::CallableId, crate::impls::EffectiveCallableDefinition)>(world.callable_definitions.len())
        .map_err(allocation_error)?;
    for record in &world.callable_definitions {
        let definition = import_callable_definition(record, &type_relocation, &type_store, allocations)?;
        let key = definition.callable.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        callable_definition_entries.push((key, definition));
    }
    let callable_definitions_map =
        crate::parent_map::ParentMap::from_frozen_entries_with_allocation_budget(callable_definition_entries, allocations).map_err(|error| match error {
            crate::parent_map::FrozenParentMapError::Allocation(error) => allocation_error(error),
            crate::parent_map::FrozenParentMapError::DuplicateKey => referential("callable definition owner is duplicated after stable-identity relocation"),
        })?;
    #[cfg(feature = "allocation-diagnostics")]
    record_callable_import_allocation_diagnostic_stage(57, allocations, &mut callable_table_previous_observed, &mut callable_table_previous_ledger);

    let enum_requirements = import_enum_requirements(
        &world.enum_requirements,
        &structural_contracts.enum_table,
        &type_relocation,
        &type_store,
        allocations,
    )?;
    #[cfg(feature = "allocation-diagnostics")]
    record_callable_import_allocation_diagnostic_stage(58, allocations, &mut callable_table_previous_observed, &mut callable_table_previous_ledger);
    record_import_allocation_diagnostic_stage(6, allocations);

    // 10. Reconstruct trait surfaces and trait headers (P7.T7.B)
    let mut trait_surface_entries = allocations
        .vector::<(DeclarationId, Arc<crate::traits::TraitSurface>)>(world.trait_surfaces.len())
        .map_err(allocation_error)?;
    let mut trait_header_entries = allocations
        .vector::<(DeclarationId, Arc<crate::traits::TraitHeader>)>(world.trait_surfaces.len())
        .map_err(allocation_error)?;
    for rec in &world.trait_surfaces {
        let decl = import_stable_declaration_with_allocation_budget(&rec.declaration, allocations)?;
        let generic_signature = if let Some(g) = &rec.generic_signature {
            Some(import_generic_signature_with_allocation_budget(g, &type_relocation, &type_store, allocations)?)
        } else {
            None
        };
        let mut associated_entries = allocations
            .vector::<(crate::traits::AssociatedTypeRequirementId, crate::traits::TraitAssociatedTypeRequirement)>(rec.associated_types.len())
            .map_err(allocation_error)?;
        for associated in &rec.associated_types {
            let owner = import_stable_declaration_with_allocation_budget(&associated.requirement.owner, allocations)?;
            if owner != decl {
                return Err(referential("trait associated-type owner does not match its surface"));
            }
            let requirement = crate::traits::AssociatedTypeRequirementId::new(owner, associated.requirement.index);
            let domain = match &associated.domain {
                TypeMemberDomainV1::Kind { kind } => crate::types::type_member::TypeMemberDomain::Kind(type_relocation.kind(*kind)?),
                TypeMemberDomainV1::ValueType { ty } => crate::types::type_member::TypeMemberDomain::ValueType(type_relocation.ty(*ty)?),
            };
            let key = requirement.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
            associated_entries.push((
                key,
                crate::traits::TraitAssociatedTypeRequirement {
                    requirement,
                    name: allocations.boxed_str(&associated.name).map_err(allocation_error)?,
                    domain,
                    source: import_source_span_with_allocation_budget(&associated.source, allocations)?,
                },
            ));
        }
        let mut member_entries = allocations
            .vector::<(crate::traits::TraitRequirementId, crate::traits::TraitSurfaceMember)>(rec.members.len())
            .map_err(allocation_error)?;
        for member in &rec.members {
            let callable = import_stable_callable_with_allocation_budget(&member.callable, allocations)?;
            let owner = import_stable_declaration_with_allocation_budget(&member.requirement.owner, allocations)?;
            let side = match member.requirement.side {
                0 => crate::identity::DispatchSide::Instance,
                1 => crate::identity::DispatchSide::Class,
                _ => return Err(referential("trait member dispatch side is invalid")),
            };
            let requirement = crate::traits::TraitRequirementId {
                owner,
                selector: import_stable_selector_with_allocation_budget(&member.requirement.selector, allocations)?,
                side,
            };
            if requirement.owner != decl
                || !matches!(&callable.owner, CallableOwnerId::Declaration(owner) if owner == &requirement.owner)
                || requirement.selector != callable.selector
                || requirement.side != callable.side
            {
                return Err(referential("trait requirement identity does not match its canonical callable"));
            }
            let key = requirement.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
            let signature = import_callable_signature(&member.signature, &type_relocation, &type_store, allocations)?;
            let source_member_index =
                usize::try_from(member.source_member_index).map_err(|_| referential("trait source member index exceeds address space"))?;
            let default_source = member
                .default_source
                .as_ref()
                .map(|source| import_source_span_with_allocation_budget(source, allocations))
                .transpose()?;
            member_entries.push((
                key,
                crate::traits::TraitSurfaceMember {
                    requirement,
                    callable,
                    signature,
                    visibility: import_visibility(&member.visibility),
                    source: import_source_span_with_allocation_budget(&member.source, allocations)?,
                    source_member_index,
                    default_present: member.default_present,
                    default_source,
                },
            ));
        }
        let members = crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(member_entries, allocations).map_err(|error| match error {
            crate::ordered_map::FrozenOrderedMapError::Allocation(error) => allocation_error(error),
            crate::ordered_map::FrozenOrderedMapError::DuplicateKey => referential("trait member owner is duplicated after identity relocation"),
        })?;
        let associated_types =
            crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(associated_entries, allocations).map_err(|error| match error {
                crate::ordered_map::FrozenOrderedMapError::Allocation(error) => allocation_error(error),
                crate::ordered_map::FrozenOrderedMapError::DuplicateKey => referential("trait associated-type owner is duplicated after identity relocation"),
            })?;
        let mut diagnostics = allocations.vector(rec.diagnostics.len()).map_err(allocation_error)?;
        for diagnostic in &rec.diagnostics {
            diagnostics.push(import_trait_surface_diagnostic(diagnostic, &type_relocation, allocations)?);
        }
        let diagnostics = allocations.boxed_slice(diagnostics).map_err(allocation_error)?;
        let surface_decl = decl.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        let surface =
            crate::traits::TraitSurface::from_frozen_import_parts(surface_decl, generic_signature, members, associated_types, diagnostics, allocations)
                .map_err(|error| match error {
                    crate::ordered_map::FrozenOrderedMapError::Allocation(error) => allocation_error(error),
                    crate::ordered_map::FrozenOrderedMapError::DuplicateKey => referential("trait requirement layout contains a duplicate identity"),
                })?;
        let surface_key = decl.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        trait_surface_entries.push((surface_key, allocations.arc(surface).map_err(allocation_error)?));
        let header_source = import_source_span_with_allocation_budget(&rec.header_source, allocations)?;
        let header_declaration = decl.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        let header_generic_signature = rec
            .generic_signature
            .as_ref()
            .map(|signature| import_generic_signature_with_allocation_budget(signature, &type_relocation, &type_store, allocations))
            .transpose()?;
        let header = allocations
            .arc(crate::traits::TraitHeader {
                declaration: header_declaration,
                generic_signature: header_generic_signature,
                source: header_source,
            })
            .map_err(allocation_error)?;
        let header_key = decl.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        trait_header_entries.push((header_key, header));
    }
    let trait_surfaces =
        crate::traits::TraitSurfaceTable::from_frozen_entries_with_allocation_budget(trait_surface_entries, allocations).map_err(|error| match error {
            crate::parent_map::FrozenParentMapError::Allocation(error) => allocation_error(error),
            crate::parent_map::FrozenParentMapError::DuplicateKey => referential("trait surface owner is duplicated after identity relocation"),
        })?;
    let trait_headers =
        crate::traits::TraitHeaderTable::from_frozen_entries_with_allocation_budget(trait_header_entries, allocations).map_err(|error| match error {
            crate::parent_map::FrozenParentMapError::Allocation(error) => allocation_error(error),
            crate::parent_map::FrozenParentMapError::DuplicateKey => referential("trait header owner is duplicated after identity relocation"),
        })?;
    record_import_allocation_diagnostic_stage(7, allocations);

    // 11. Reconstruct surfaces, dispatch resolver, and field signatures (P7.T7.B)
    let field_signature_capacity = world
        .dispatch_surfaces
        .iter()
        .try_fold(0usize, |total, surface| total.checked_add(surface.fields.len()))
        .ok_or_else(|| allocation_error(AllocationError::Overflow))?;
    let mut field_signature_entries = allocations
        .vector::<(crate::identity::FieldId, crate::signature::FieldSemanticSignature)>(field_signature_capacity)
        .map_err(allocation_error)?;
    let mut dispatch_surface_entries = allocations
        .vector::<(DeclarationId, crate::surface::DeclarationSurface)>(world.dispatch_surfaces.len())
        .map_err(allocation_error)?;
    let mut type_declaration_entries = allocations
        .vector::<(crate::types::id::TypeId, DeclarationId)>(world.structural_contracts.declaration_shells.len())
        .map_err(allocation_error)?;
    for rec in &world.dispatch_surfaces {
        let decl = import_stable_declaration_with_allocation_budget(&rec.declaration, allocations)?;
        let mut surface = crate::surface::DeclarationSurface::new(Some(decl.clone_with_allocation_budget(allocations).map_err(allocation_error)?));
        for (side, facts) in [
            (crate::identity::DispatchSide::Instance, &rec.instance_callables),
            (crate::identity::DispatchSide::Class, &rec.class_callables),
        ] {
            let side_wire = match side {
                crate::identity::DispatchSide::Instance => 0,
                crate::identity::DispatchSide::Class => 1,
            };
            let field_count = rec.fields.iter().filter(|field| field.side == side_wire).count();
            let mut members = crate::surface::FrozenMemberSurfaceBuilder::new(field_count, facts.len(), allocations).map_err(allocation_error)?;
            for fact in facts {
                let cid = import_stable_callable_with_allocation_budget(&fact.callable, allocations)?;
                let semantic = callable_signatures
                    .get(&cid)
                    .ok_or_else(|| referential("declaration dispatch callable has no imported semantic signature"))?;
                let mut projected = crate::checker::declaration_signature::project_semantic_signature_with_allocation_budget(semantic, allocations)
                    .map_err(allocation_error)?;
                projected.return_type = import_type_knowledge_with_allocation_budget(&fact.knowledge, &type_relocation, allocations)?;
                members
                    .add_callable(cid, projected, import_visibility(&fact.visibility), allocations)
                    .map_err(allocation_error)?;
            }
            for field in rec.fields.iter().filter(|field| field.side == side_wire) {
                let owner = import_stable_declaration_with_allocation_budget(&field.owner, allocations)?;
                let field_side = side;
                let field_id = crate::identity::FieldId::new(
                    owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                    allocations.boxed_str(&field.name).map_err(allocation_error)?,
                    field_side,
                );
                let knowledge = import_type_knowledge_with_allocation_budget(&field.knowledge, &type_relocation, allocations)?;
                members
                    .add_field(
                        field_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                        knowledge,
                        import_visibility(&field.visibility),
                        allocations,
                    )
                    .map_err(allocation_error)?;
                if let (Some(declared_type), Some(mutable)) = (&field.declared_type, field.mutable) {
                    let declared_type = import_declared_type_fact_with_allocation_budget(declared_type, &type_relocation, allocations)?;
                    let source = field
                        .source
                        .as_ref()
                        .map(|source| {
                            if source.start > source.end {
                                return Err(referential("declaration field source span is inverted"));
                            }
                            Ok(SemanticSourceSpan::new(
                                owner.module.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                                SourceRange {
                                    start: usize::try_from(source.start).map_err(|_| referential("declaration field source start exceeds address space"))?,
                                    end: usize::try_from(source.end).map_err(|_| referential("declaration field source end exceeds address space"))?,
                                },
                            ))
                        })
                        .transpose()?;
                    let signature = crate::signature::FieldSemanticSignature {
                        field: field_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                        owner: owner.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                        side: field_side,
                        name: allocations.boxed_str(&field.name).map_err(allocation_error)?,
                        mutable,
                        declared_type,
                        source,
                    };
                    field_signature_entries.push((field_id, signature));
                }
            }
            let members = members.finish(allocations).map_err(|error| match error {
                crate::surface::FrozenSurfaceMapError::Allocation(error) => allocation_error(error),
                crate::surface::FrozenSurfaceMapError::DuplicateKey => referential("declaration member surface contains duplicate relocated identities"),
            })?;
            *surface.surface_mut(side) = members;
        }
        if let Some(type_form) = structural_contracts.declaration_table.form(&decl) {
            type_declaration_entries.push((type_form, decl.clone_with_allocation_budget(allocations).map_err(allocation_error)?));
        }
        let surface_key = decl.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        dispatch_surface_entries.push((surface_key, surface));
    }
    let dispatch_resolver = crate::dispatch::SurfaceDispatchResolver::from_frozen_import_entries_with_allocation_budget(
        dispatch_surface_entries,
        type_declaration_entries,
        allocations,
    )
    .map_err(|error| match error {
        crate::dispatch::FrozenDispatchImportError::Allocation(error) => allocation_error(error),
        crate::dispatch::FrozenDispatchImportError::DuplicateOwner => referential("dispatch surface owner is duplicated after relocation"),
        crate::dispatch::FrozenDispatchImportError::DuplicateType => referential("dispatch type registration is duplicated after relocation"),
        crate::dispatch::FrozenDispatchImportError::DuplicateConditionalTarget => {
            referential("conditional inherent target is duplicated after identity relocation")
        }
        crate::dispatch::FrozenDispatchImportError::DuplicateDomain => referential("inherent implementation domain is duplicated after identity relocation"),
    })?;
    let field_signatures = crate::signature::FieldSignatureTable::from_frozen_entries_with_allocation_budget(field_signature_entries, allocations).map_err(
        |error| match error {
            crate::signature::FrozenFieldSignatureTableError::Allocation(error) => allocation_error(error),
            crate::signature::FrozenFieldSignatureTableError::DuplicateField => referential("field signature owner is duplicated after identity relocation"),
        },
    )?;
    record_import_allocation_diagnostic_stage(8, allocations);

    let domain_count = world
        .conditional_inherent_sets
        .iter()
        .try_fold(0usize, |count, set| count.checked_add(set.domains.len()))
        .ok_or_else(|| allocation_error(AllocationError::Overflow))?;
    let mut conditional_member_entries = allocations
        .vector::<(crate::impls::InherentImplTarget, Arc<crate::impls::ConditionalInherentMemberSet>)>(world.conditional_inherent_sets.len())
        .map_err(allocation_error)?;
    let mut inherent_domain_entries = allocations
        .vector::<(crate::identity::ImplId, Arc<crate::impls::InherentImplDomain>)>(domain_count)
        .map_err(allocation_error)?;
    for set_record in &world.conditional_inherent_sets {
        use phalcom_compiler_artifact::semantic::world::InherentImplTargetV1;
        let target = match &set_record.target {
            InherentImplTargetV1::Declaration { declaration } => {
                crate::impls::InherentImplTarget::Declaration(import_stable_declaration_with_allocation_budget(declaration, allocations)?)
            }
            InherentImplTargetV1::ExactEnumCase { variant } => {
                crate::impls::InherentImplTarget::ExactEnumCase(import_stable_variant_with_allocation_budget(variant, allocations)?)
            }
        };
        let mut domain_entries = allocations
            .vector::<(crate::identity::ImplId, Arc<crate::impls::InherentImplDomain>)>(set_record.domains.len())
            .map_err(allocation_error)?;
        for domain in &set_record.domains {
            let impl_id = import_stable_impl_with_allocation_budget(&domain.impl_id, allocations)?;
            let generic_signature = domain
                .generic_signature
                .as_ref()
                .map(|signature| import_generic_signature_with_allocation_budget(signature, &type_relocation, &type_store, allocations))
                .transpose()?;
            let mut constraints = allocations.vector(domain.constraints.len()).map_err(allocation_error)?;
            for constraint in &domain.constraints {
                constraints.push(import_constraint_with_allocation_budget(constraint, &type_relocation, allocations)?);
            }
            let domain_target = match &target {
                crate::impls::InherentImplTarget::Declaration(declaration) => {
                    crate::impls::InherentImplTarget::Declaration(declaration.clone_with_allocation_budget(allocations).map_err(allocation_error)?)
                }
                crate::impls::InherentImplTarget::ExactEnumCase(variant) => {
                    crate::impls::InherentImplTarget::ExactEnumCase(variant.clone_with_allocation_budget(allocations).map_err(allocation_error)?)
                }
            };
            let domain_impl_id = impl_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
            let constraints = allocations.boxed_slice(constraints).map_err(allocation_error)?;
            let imported = allocations
                .arc(crate::impls::InherentImplDomain {
                    impl_id: domain_impl_id,
                    target: domain_target,
                    head_type: type_relocation.ty(domain.head_type)?,
                    generic_signature,
                    constraints,
                })
                .map_err(allocation_error)?;
            inherent_domain_entries.push((
                impl_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                Arc::clone(&imported),
            ));
            domain_entries.push((impl_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?, imported));
        }
        let domains = crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(domain_entries, allocations).map_err(|error| match error {
            crate::ordered_map::FrozenOrderedMapError::Allocation(error) => allocation_error(error),
            crate::ordered_map::FrozenOrderedMapError::DuplicateKey => referential("conditional inherent domain is duplicated after identity relocation"),
        })?;
        let mut members = allocations.vector(set_record.members.len()).map_err(allocation_error)?;
        for member in &set_record.members {
            let impl_id = import_stable_impl_with_allocation_budget(&member.impl_id, allocations)?;
            let domain = domains
                .get(&impl_id)
                .cloned()
                .ok_or_else(|| referential("conditional inherent member has no reconstructed domain"))?;
            let signature_template = import_callable_signature(&member.signature, &type_relocation, &type_store, allocations)?;
            let source_member =
                usize::try_from(member.source_member_index).map_err(|_| referential("conditional inherent source ordinal exceeds address space"))?;
            members.push(crate::impls::ConditionalInherentMember {
                impl_id,
                domain,
                is_conditionally_applicable: member.is_conditionally_applicable,
                callable: import_stable_callable_with_allocation_budget(&member.callable, allocations)?,
                signature_template,
                visibility: import_visibility(&member.visibility),
                source_member,
                is_requirement: member.is_requirement,
            });
        }
        let set_target = match &target {
            crate::impls::InherentImplTarget::Declaration(declaration) => {
                crate::impls::InherentImplTarget::Declaration(declaration.clone_with_allocation_budget(allocations).map_err(allocation_error)?)
            }
            crate::impls::InherentImplTarget::ExactEnumCase(variant) => {
                crate::impls::InherentImplTarget::ExactEnumCase(variant.clone_with_allocation_budget(allocations).map_err(allocation_error)?)
            }
        };
        let members = crate::impls::ConditionalInherentMemberSet::from_frozen_import_parts_with_allocation_budget(Some(set_target), members, allocations)
            .map_err(|error| match error {
                crate::ordered_map::FrozenOrderedMapError::Allocation(error) => allocation_error(error),
                crate::ordered_map::FrozenOrderedMapError::DuplicateKey => referential("conditional inherent member selector is duplicated"),
            })?;
        let target_key = match &target {
            crate::impls::InherentImplTarget::Declaration(declaration) => {
                crate::impls::InherentImplTarget::Declaration(declaration.clone_with_allocation_budget(allocations).map_err(allocation_error)?)
            }
            crate::impls::InherentImplTarget::ExactEnumCase(variant) => {
                crate::impls::InherentImplTarget::ExactEnumCase(variant.clone_with_allocation_budget(allocations).map_err(allocation_error)?)
            }
        };
        conditional_member_entries.push((target_key, allocations.arc(members).map_err(allocation_error)?));
    }
    let dispatch_resolver = dispatch_resolver
        .with_frozen_conditional_import_entries(conditional_member_entries, inherent_domain_entries, allocations)
        .map_err(|error| match error {
            crate::dispatch::FrozenDispatchImportError::Allocation(error) => allocation_error(error),
            crate::dispatch::FrozenDispatchImportError::DuplicateOwner => referential("dispatch surface owner is duplicated after relocation"),
            crate::dispatch::FrozenDispatchImportError::DuplicateType => referential("dispatch type registration is duplicated after relocation"),
            crate::dispatch::FrozenDispatchImportError::DuplicateConditionalTarget => {
                referential("conditional inherent target is duplicated after identity relocation")
            }
            crate::dispatch::FrozenDispatchImportError::DuplicateDomain => {
                referential("inherent implementation domain is duplicated after identity relocation")
            }
        })?;
    record_import_allocation_diagnostic_stage(9, allocations);
    // 12. Reconstruct immutable conformance heads without source B-tree indexes.
    let mut conformance_entries = allocations
        .vector::<(crate::identity::ImplId, crate::impls::ConformanceContribution)>(world.conformance_heads.len())
        .map_err(allocation_error)?;
    for rec in &world.conformance_heads {
        let impl_id = import_stable_impl_with_allocation_budget(&rec.impl_id, allocations)?;
        let source_module = import_stable_module_with_allocation_budget(&rec.source_module, allocations)?;
        let target = match &rec.target {
            ConformanceTargetV1::Declaration { declaration } => {
                crate::impls::ConformanceTarget::Declaration(import_stable_declaration_with_allocation_budget(declaration, allocations)?)
            }
            ConformanceTargetV1::ExactEnumCase { variant } => {
                crate::impls::ConformanceTarget::ExactEnumCase(import_stable_variant_with_allocation_budget(variant, allocations)?)
            }
        };
        let target_head = type_relocation.ty(rec.target_type)?;
        let trait_decl = import_stable_declaration_with_allocation_budget(&rec.trait_ref.declaration, allocations)?;
        let mut args = allocations.vector(rec.trait_ref.arguments.len()).map_err(allocation_error)?;
        for &a in &rec.trait_ref.arguments {
            args.push(type_relocation.ty(a)?);
        }
        let generic_signature = if let Some(g) = &rec.generic_signature {
            Some(import_generic_signature_with_allocation_budget(g, &type_relocation, &type_store, allocations)?)
        } else {
            None
        };
        let key = impl_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        let source_span_module = source_module.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        conformance_entries.push((
            key,
            crate::impls::ConformanceContribution {
                impl_id,
                source_module,
                trait_ref: crate::traits::TraitRef::new(trait_decl, allocations.boxed_slice(args).map_err(allocation_error)?),
                target,
                target_head,
                generic_signature,
                authorized: rec.authorized,
                eligible: rec.eligible,
                source: crate::diagnostic::SemanticSourceSpan::new(source_span_module, phalcom_common::range::SourceRange::default()),
                diagnostics: Box::new([]),
            },
        ));
    }
    let conformance_index =
        crate::impls::ConformanceIndex::from_frozen_entries_with_allocation_budget(conformance_entries, allocations).map_err(|error| match error {
            crate::parent_map::FrozenParentMapError::Allocation(error) => allocation_error(error),
            crate::parent_map::FrozenParentMapError::DuplicateKey => referential("conformance head is duplicated after identity relocation"),
        })?;
    record_import_allocation_diagnostic_stage(10, allocations);

    #[cfg(feature = "allocation-diagnostics")]
    let mut witness_diagnostic_previous_observed = phalcom_type_meta::allocation::current_observed_bytes().unwrap_or(0);
    #[cfg(feature = "allocation-diagnostics")]
    let mut witness_diagnostic_previous_ledger = allocations.requested();
    let mut witness_plan_entries = allocations
        .vector::<(crate::identity::ImplId, Arc<crate::impls::ConformanceWitnessPlan>)>(world.witness_plans.len())
        .map_err(allocation_error)?;
    let mut source_plan_fingerprints = allocations
        .vector::<(crate::identity::ImplId, crate::db::ProductFingerprint)>(world.witness_plans.len())
        .map_err(allocation_error)?;
    #[cfg(feature = "allocation-diagnostics")]
    record_callable_import_allocation_diagnostic_stage(
        39,
        allocations,
        &mut witness_diagnostic_previous_observed,
        &mut witness_diagnostic_previous_ledger,
    );
    for witness in &world.witness_plans {
        let impl_id = import_stable_impl_with_allocation_budget(&witness.impl_id, allocations)?;
        let source_module = import_stable_module_with_allocation_budget(&witness.source_module, allocations)?;
        let trait_decl = import_stable_declaration_with_allocation_budget(&witness.trait_ref_template.declaration, allocations)?;
        let mut trait_args = allocations.vector(witness.trait_ref_template.arguments.len()).map_err(allocation_error)?;
        for &a in &witness.trait_ref_template.arguments {
            trait_args.push(type_relocation.ty(a)?);
        }
        let trait_ref_template = crate::traits::TraitRef::new(
            trait_decl.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            allocations.boxed_slice(trait_args).map_err(allocation_error)?,
        );
        let target_template = type_relocation.ty(witness.target_template)?;
        let generic_signature = if let Some(g) = &witness.generic_signature {
            Some(import_generic_signature_with_allocation_budget(g, &type_relocation, &type_store, allocations)?)
        } else {
            None
        };
        #[cfg(feature = "allocation-diagnostics")]
        record_callable_import_allocation_diagnostic_stage(
            40,
            allocations,
            &mut witness_diagnostic_previous_observed,
            &mut witness_diagnostic_previous_ledger,
        );
        let mut requirement_view_entries = allocations
            .vector::<(crate::traits::TraitRequirementId, crate::traits::InstantiatedTraitRequirement)>(witness.requirement_views.len())
            .map_err(allocation_error)?;
        for view in &witness.requirement_views {
            let requirement = import_callable_requirement_identity_with_allocation_budget(&view.requirement, allocations)?;
            if requirement.owner != trait_decl {
                return Err(referential("witness requirement view belongs to another trait"));
            }
            let key = requirement.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
            let signature = import_callable_signature(&view.signature, &type_relocation, &type_store, allocations)?;
            let callable = import_exact_callable_with_allocation_budget(&view.callable, allocations)?;
            let default_callable = view
                .default_callable
                .as_ref()
                .map(|callable| import_exact_callable_with_allocation_budget(callable, allocations))
                .transpose()?;
            let imported = crate::traits::InstantiatedTraitRequirement {
                requirement,
                callable,
                signature,
                visibility: import_visibility(&view.visibility),
                default_callable,
            };
            requirement_view_entries.push((key, imported));
        }
        let requirement_views =
            crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(requirement_view_entries, allocations).map_err(|error| match error {
                crate::ordered_map::FrozenOrderedMapError::Allocation(error) => allocation_error(error),
                crate::ordered_map::FrozenOrderedMapError::DuplicateKey => referential("witness requirement view is duplicated"),
            })?;
        #[cfg(feature = "allocation-diagnostics")]
        record_callable_import_allocation_diagnostic_stage(
            41,
            allocations,
            &mut witness_diagnostic_previous_observed,
            &mut witness_diagnostic_previous_ledger,
        );

        let mut requirement_entries = allocations
            .vector::<(crate::traits::TraitRequirementId, crate::impls::RequirementSelectionTemplate)>(witness.requirements.len())
            .map_err(allocation_error)?;
        let mut evidence_importer = CallableEvidenceImporter::new(
            witness.conformance_evidence.len(),
            &type_relocation,
            &type_store,
            &source_plan_fingerprints,
            allocations,
            limits.max_recursion_depth,
        )?;
        for entry in &witness.requirements {
            let requirement = import_callable_requirement_identity_with_allocation_budget(&entry.requirement, evidence_importer.allocations)?;
            if requirement.owner != trait_decl {
                return Err(referential("witness requirement selection belongs to another trait"));
            }
            let key = requirement
                .clone_with_allocation_budget(evidence_importer.allocations)
                .map_err(allocation_error)?;
            let selection = import_callable_requirement_selection(&entry.selection, &witness.conformance_evidence, &mut evidence_importer, 0)?;
            requirement_entries.push((key, selection));
        }
        drop(evidence_importer);
        let requirements =
            crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(requirement_entries, allocations).map_err(|error| match error {
                crate::ordered_map::FrozenOrderedMapError::Allocation(error) => allocation_error(error),
                crate::ordered_map::FrozenOrderedMapError::DuplicateKey => referential("witness requirement selection is duplicated"),
            })?;
        #[cfg(feature = "allocation-diagnostics")]
        record_callable_import_allocation_diagnostic_stage(
            42,
            allocations,
            &mut witness_diagnostic_previous_observed,
            &mut witness_diagnostic_previous_ledger,
        );
        let associated_type_plan = import_callable_associated_type_plan(&witness.associated_type_plan, &impl_id, &trait_decl, &type_relocation, allocations)?;
        #[cfg(feature = "allocation-diagnostics")]
        record_callable_import_allocation_diagnostic_stage(
            43,
            allocations,
            &mut witness_diagnostic_previous_observed,
            &mut witness_diagnostic_previous_ledger,
        );
        let completeness = import_callable_conformance_completeness(&witness.completeness, &type_relocation, allocations)?;
        let mut diagnostics = allocations.vector(witness.diagnostics.len()).map_err(allocation_error)?;
        for diagnostic in &witness.diagnostics {
            diagnostics.push(import_trait_surface_diagnostic(diagnostic, &type_relocation, allocations)?);
        }
        let diagnostics = allocations.boxed_slice(diagnostics).map_err(allocation_error)?;
        #[cfg(feature = "allocation-diagnostics")]
        record_callable_import_allocation_diagnostic_stage(
            44,
            allocations,
            &mut witness_diagnostic_previous_observed,
            &mut witness_diagnostic_previous_ledger,
        );
        let plan = crate::impls::ConformanceWitnessPlan::from_parts(
            impl_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            source_module,
            target_template,
            trait_ref_template,
            generic_signature,
            requirement_views,
            requirements,
            associated_type_plan,
            witness.invalid_explicit_members,
            completeness,
            diagnostics,
        );
        #[cfg(feature = "allocation-diagnostics")]
        record_callable_import_allocation_diagnostic_stage(
            45,
            allocations,
            &mut witness_diagnostic_previous_observed,
            &mut witness_diagnostic_previous_ledger,
        );

        source_plan_fingerprints.push((impl_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?, plan.fingerprint));
        witness_plan_entries.push((
            impl_id.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
            allocations.arc(plan).map_err(allocation_error)?,
        ));
        #[cfg(feature = "allocation-diagnostics")]
        record_callable_import_allocation_diagnostic_stage(
            46,
            allocations,
            &mut witness_diagnostic_previous_observed,
            &mut witness_diagnostic_previous_ledger,
        );
    }

    let witness_plans_map =
        crate::impls::ConformanceWitnessPlans::from_frozen_entries_with_allocation_budget(witness_plan_entries, allocations).map_err(|error| match error {
            crate::parent_map::FrozenParentMapError::Allocation(error) => allocation_error(error),
            crate::parent_map::FrozenParentMapError::DuplicateKey => referential("witness plan is duplicated after identity relocation"),
        })?;
    #[cfg(feature = "allocation-diagnostics")]
    record_callable_import_allocation_diagnostic_stage(
        47,
        allocations,
        &mut witness_diagnostic_previous_observed,
        &mut witness_diagnostic_previous_ledger,
    );
    record_import_allocation_diagnostic_stage(11, allocations);

    let (callable_analyses, callable_binding_ranges) = import_callable_analysis_records(
        &world.callable_analyses,
        &type_relocation,
        &type_store,
        &source_plan_fingerprints,
        module_world,
        &import_phase_products,
        limits,
        allocations,
    )?;
    record_import_allocation_diagnostic_stage(12, allocations);

    let mut imported_derived_conformance_rules = allocations
        .vector::<crate::derived_conformance::DerivedConformanceRule>(world.derived_conformance_rules.len())
        .map_err(allocation_error)?;
    for record in &world.derived_conformance_rules {
        let rule = import_derived_conformance_rule_record(
            record,
            &type_relocation,
            &type_store,
            &source_plan_fingerprints,
            allocations,
            limits.max_recursion_depth,
        )?;
        imported_derived_conformance_rules.push(rule);
    }
    let derived_conformance_rules =
        crate::derived_conformance::DerivedConformanceRuleIndex::from_frozen_rules_with_allocation_budget(imported_derived_conformance_rules, allocations)
            .map_err(|error| match error {
                crate::derived_conformance::FrozenDerivedConformanceRuleIndexError::Allocation(error) => allocation_error(error),
                crate::derived_conformance::FrozenDerivedConformanceRuleIndexError::DuplicateKey => {
                    referential("derived conformance rule implementation is duplicated after identity relocation")
                }
            })?;
    let trait_dispatch_index = crate::trait_dispatch::TraitDispatchIndex::build_with_allocation_budget(&conformance_index, &trait_surfaces, allocations)
        .map_err(|error| match error {
            crate::trait_dispatch::FrozenTraitDispatchIndexError::Allocation(error) => allocation_error(error),
            crate::trait_dispatch::FrozenTraitDispatchIndexError::DuplicateBucket => referential("trait dispatch index contains a duplicate bucket"),
        })?;
    record_import_allocation_diagnostic_stage(13, allocations);

    let mut constructor_entries = allocations.vector(world.constructor_initializations.len()).map_err(allocation_error)?;
    for rec in &world.constructor_initializations {
        let factory = import_stable_callable_with_allocation_budget(&rec.factory, allocations)?;
        let initializer = import_stable_callable_with_allocation_budget(&rec.initializer, allocations)?;
        let origin = match rec.origin {
            0 => crate::signature::ConstructorOrigin::Source,
            1 => crate::signature::ConstructorOrigin::Construct,
            2 => crate::signature::ConstructorOrigin::Data,
            _ => crate::signature::ConstructorOrigin::DefaultOnly,
        };
        let field_input_fingerprint = crate::db::ProductFingerprint(rec.field_input_fingerprint);
        let mut default_fields = allocations.vector(rec.default_fields.len()).map_err(allocation_error)?;
        for name in &rec.default_fields {
            default_fields.push(crate::identity::FieldId::new(
                initializer
                    .owner
                    .declaration()
                    .clone_with_allocation_budget(allocations)
                    .map_err(allocation_error)?,
                allocations.boxed_str(name).map_err(allocation_error)?,
                crate::identity::DispatchSide::Instance,
            ));
        }
        let mut parameter_fields = allocations.vector(rec.parameter_fields.len()).map_err(allocation_error)?;
        for pf in &rec.parameter_fields {
            let side = match pf.field_side {
                0 => crate::identity::DispatchSide::Instance,
                _ => crate::identity::DispatchSide::Class,
            };
            parameter_fields.push(crate::signature::ConstructorParameterField {
                parameter: crate::identity::CallableParameterId::new(
                    factory.clone_with_allocation_budget(allocations).map_err(allocation_error)?,
                    pf.parameter_index,
                ),
                field: crate::identity::FieldId::new(
                    initializer
                        .owner
                        .declaration()
                        .clone_with_allocation_budget(allocations)
                        .map_err(allocation_error)?,
                    allocations.boxed_str(&pf.field_name).map_err(allocation_error)?,
                    side,
                ),
            });
        }
        let key = factory.clone_with_allocation_budget(allocations).map_err(allocation_error)?;
        constructor_entries.push((
            key,
            crate::signature::ConstructorInitializationProduct {
                factory,
                initializer,
                origin,
                field_input_fingerprint,
                default_fields: allocations.boxed_slice(default_fields).map_err(allocation_error)?,
                parameter_fields: allocations.boxed_slice(parameter_fields).map_err(allocation_error)?,
            },
        ));
    }
    let constructor_initializations =
        crate::signature::ConstructorInitializationTable::from_frozen_entries_with_allocation_budget(constructor_entries, allocations)
            .map_err(frozen_ordered_map_error)?;
    record_import_allocation_diagnostic_stage(14, allocations);

    let ImportedAssociatedAuthority {
        associated_surfaces,
        type_member_declarations,
        associated_static_domains,
    } = import_associated_authority(
        &world.type_member_declarations,
        &world.associated_static_domains,
        &world.associated_surfaces,
        &type_relocation,
        allocations,
    )?;
    let (static_case_catalogs, static_case_signatures) =
        import_static_case_owners(&world.static_case_catalogs, &world.static_case_signatures, &type_relocation, allocations)?;
    let static_definitions = import_static_definition_owners(
        &world.static_definitions,
        &structural_contracts,
        &static_relocation,
        &type_relocation,
        &type_store,
        allocations,
    )?;
    let static_constants = import_static_constant_owners(&world.static_constants, &static_relocation, &type_relocation, allocations)?;
    let module_static_bindings = import_module_static_binding_indexes(&world.module_static_bindings, &world.callable_analyses, &type_relocation, allocations)?;
    record_import_allocation_diagnostic_stage(15, allocations);

    let surfaces = dispatch_resolver.surfaces().clone();
    let formal_projection = crate::presentation::FormalSemanticProjection::from_callable_analysis_table_with_binding_ranges_and_allocation_budget(
        &callable_analyses,
        &callable_binding_ranges,
        allocations,
    )
    .map_err(|error| match error {
        crate::presentation::FormalSemanticProjectionBuildError::Allocation(error) => allocation_error(error),
        crate::presentation::FormalSemanticProjectionBuildError::DuplicateFact => referential("formal projection contains a duplicate callable fact"),
        crate::presentation::FormalSemanticProjectionBuildError::DuplicateModule => referential("formal projection contains a duplicate module"),
    })?;
    let formal_projection = allocations.arc(formal_projection).map_err(allocation_error)?;
    record_import_allocation_diagnostic_stage(16, allocations);
    let type_store = allocations.arc(type_store).map_err(allocation_error)?;
    let static_values = allocations.arc(static_values).map_err(allocation_error)?;
    let snap_sources = allocations.arc(BTreeMap::new()).map_err(allocation_error)?;
    let snap_presentation_sources = allocations.arc(BTreeMap::new()).map_err(allocation_error)?;
    let snap_dispatch = allocations.arc(dispatch_resolver).map_err(allocation_error)?;
    let snap_callable_signatures = allocations.arc(callable_signatures).map_err(allocation_error)?;
    let snap_constructor_initializations = allocations.arc(constructor_initializations).map_err(allocation_error)?;
    let snap_callable_definitions = allocations.arc(callable_definitions_map).map_err(allocation_error)?;
    let snap_conformance_index = allocations.arc(conformance_index).map_err(allocation_error)?;
    let snap_conformance_witness_plans = allocations.arc(witness_plans_map).map_err(allocation_error)?;
    let snap_derived_conformance_rules = allocations.arc(derived_conformance_rules).map_err(allocation_error)?;
    let snap_trait_dispatch_index = allocations.arc(trait_dispatch_index).map_err(allocation_error)?;
    let snap_field_signatures = allocations.arc(field_signatures).map_err(allocation_error)?;

    let snap_trait_headers = allocations.arc(trait_headers).map_err(allocation_error)?;
    let snap_trait_surfaces = allocations.arc(trait_surfaces).map_err(allocation_error)?;
    let snap_diagnostics = allocations.arc(BTreeMap::new()).map_err(allocation_error)?;
    let snap_semantic_graph = allocations.arc(phalcom_modules::graph::SemanticGraph::default()).map_err(allocation_error)?;
    let snap_callable_analyses = allocations.arc(callable_analyses).map_err(allocation_error)?;
    let snap_internal_incidents = allocations.arc_slice(Vec::new()).map_err(allocation_error)?;
    let ssi = crate::source_index::SourceSemanticIndex::empty_with_allocation_budget(allocations).map_err(allocation_error)?;
    let snap_source_index = allocations.arc(ssi).map_err(allocation_error)?;
    let adv = crate::advisory::AdvisoryWorkspace::empty_with_allocation_budget(allocations).map_err(allocation_error)?;
    let snap_advisory = allocations.arc(adv).map_err(allocation_error)?;
    let mp = crate::snapshot::ModuleQueryProducts::empty_with_allocation_budget(allocations).map_err(allocation_error)?;
    let snap_module_products = allocations.arc(mp).map_err(allocation_error)?;

    let snap_enum_requirements = allocations.arc(enum_requirements).map_err(allocation_error)?;
    let snap_associated_surfaces = allocations.arc(associated_surfaces).map_err(allocation_error)?;
    let snap_type_member_declarations = allocations.arc(type_member_declarations).map_err(allocation_error)?;
    let snap_associated_static_domains = allocations.arc(associated_static_domains).map_err(allocation_error)?;
    let snap_static_case_catalogs = allocations.arc(static_case_catalogs).map_err(allocation_error)?;
    let snap_static_case_signatures = allocations.arc(static_case_signatures).map_err(allocation_error)?;
    let snap_static_definitions = allocations.arc(static_definitions).map_err(allocation_error)?;
    let snap_static_constants = allocations.arc(static_constants).map_err(allocation_error)?;
    let snap_module_static_bindings = allocations.arc(module_static_bindings).map_err(allocation_error)?;
    let snap_semantic_structure_shards = allocations.arc(BTreeMap::new()).map_err(allocation_error)?;
    let snap_import_phase_products = allocations.arc(import_phase_products).map_err(allocation_error)?;
    let snap_generated_runtime_requirements = allocations.arc_slice(generated_runtime_requirements).map_err(allocation_error)?;

    let snapshot = crate::snapshot::SemanticSnapshot {
        id: crate::identity::SnapshotId::new(crate::identity::WorkspaceId(1), crate::identity::SemanticRevision(1), type_store.id()),
        generation: 1,
        store: Arc::clone(&type_store),
        sources: snap_sources,
        presentation_sources: snap_presentation_sources,
        surfaces,
        dispatch: snap_dispatch,
        callable_signatures: snap_callable_signatures,
        constructor_initializations: snap_constructor_initializations,
        callable_definitions: snap_callable_definitions,
        conformance_index: snap_conformance_index,
        conformance_witness_plans: snap_conformance_witness_plans,
        derived_conformance_rules: snap_derived_conformance_rules,
        trait_dispatch_index: snap_trait_dispatch_index,
        field_signatures: snap_field_signatures,
        declarations: Arc::clone(&structural_contracts.declaration_table),
        type_aliases: Arc::clone(&structural_contracts.type_aliases),
        trait_headers: snap_trait_headers,
        trait_surfaces: snap_trait_surfaces,
        hierarchy: Arc::clone(&structural_contracts.hierarchy),
        diagnostics: snap_diagnostics,
        semantic_graph: snap_semantic_graph,
        callable_analyses: snap_callable_analyses,
        internal_incidents: snap_internal_incidents,
        formal_projection,
        source_index: snap_source_index,
        advisory: snap_advisory,
        module_products: snap_module_products,
        data_semantics: Arc::clone(&structural_contracts.data_table),
        enum_semantics: Arc::clone(&structural_contracts.enum_table),
        enum_requirements: snap_enum_requirements,
        associated_surfaces: snap_associated_surfaces,
        type_member_declarations: snap_type_member_declarations,
        associated_static_domains: snap_associated_static_domains,
        static_case_catalogs: snap_static_case_catalogs,
        static_case_signatures: snap_static_case_signatures,
        static_definitions: snap_static_definitions,
        static_constants: snap_static_constants,
        static_values: Arc::clone(&static_values),
        module_static_bindings: snap_module_static_bindings,
        semantic_structure_shards: snap_semantic_structure_shards,
        import_phase_products: snap_import_phase_products,
        generated_runtime_requirements: snap_generated_runtime_requirements,
        // Imported rows are not promoted to Complete until every canonical
        // source-free owner family passes the final completeness check below.
        status: crate::snapshot::SnapshotStatus::Partial { blocked_modules: 0 },
    };

    // PATCH2 reconstructs this semantic owner graph, but PATCH6 owns the
    // source-free image completeness check and final snapshot promotion.
    let snapshot = allocations.arc(snapshot).map_err(allocation_error)?;
    record_import_allocation_diagnostic_stage(17, allocations);

    // Build secondary owner indexes in a single O(N) pass (P7.T8)
    let mut declarations = allocations
        .vector(world.structural_contracts.declaration_shells.len())
        .map_err(allocation_error)?;
    declarations.extend(world.structural_contracts.declaration_shells.into_iter().map(|shell| shell.declaration));
    let mut data = allocations.vector(world.structural_contracts.data_contracts.len()).map_err(allocation_error)?;
    data.extend(world.structural_contracts.data_contracts.into_iter().map(|data| data.owner));
    let mut enums = allocations.vector(world.structural_contracts.enum_contracts.len()).map_err(allocation_error)?;
    enums.extend(world.structural_contracts.enum_contracts.into_iter().map(|enm| enm.owner));
    let owner_indexes = ReconstructedModuleOwnerIndexes {
        declarations_by_module: import_module_owner_index(declarations, allocations)?,
        data_by_module: import_module_owner_index(data, allocations)?,
        enums_by_module: import_module_owner_index(enums, allocations)?,
    };
    record_import_allocation_diagnostic_stage(18, allocations);

    record_import_allocation_diagnostic_stage(19, allocations);
    Ok(ValidatedSemanticParts {
        structural_contracts,
        type_store,
        type_relocation,
        static_values,
        static_relocation,
        owner_indexes,
        snapshot,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::associated::{AssociatedFamilyInfo, AssociatedFamilyKind, AssociatedFamilyTable, AssociatedMemberId, AssociatedSurface};
    use crate::db::product::{AssociatedStaticDomainDeclarationProduct, TypeMemberDeclarationProduct};
    use crate::identity::{AssociatedFamilyId, AssociatedStaticDomainDeclarationId, CallableOwnerId, DataConstructorId, VariantId};
    use crate::signature::{CallableSemanticSignature, ReturnContractValidation};
    use crate::types::store::artifact::TypeExportContext;
    use crate::types::type_member::{
        IntrinsicTypeOperation, TypeMemberDeclarationId, TypeMemberDomain, TypeMemberParameter, TypeMemberSelector, TypeMemberSignature,
    };
    use phalcom_common::modifier::ImplementationKind;
    use phalcom_common::range::SourceRange;
    use phalcom_common::selector::{Selector, SelectorBase};
    use phalcom_modules::{DeclarationId, ModuleComponent, ModuleId, ModulePath};
    use phalcom_native_meta::{EffectSpec, NativeLifecycleSpec, RaisesSpec, ReturnFlowSpec};
    use phalcom_type_meta::identity::{StableCallableOwnerRef, StableCallableRef, StableDispatchSide, StableModuleRef, StableProjectRef};

    fn round_trip_denotation(
        denotation: &crate::types::denotation::SemanticDenotation,
        store: &TypeStore,
        type_roots: &[crate::types::id::TypeId],
    ) -> (crate::types::denotation::SemanticDenotation, usize, Vec<crate::types::id::TypeId>) {
        use phalcom_compiler_artifact::semantic::type_store::{CanonicalRootFamilyTag, CanonicalTypeRootKey, StableOwnerRef};

        let root_callable = crate::identity::CallableId::method(owner(), Selector::getter("artifact_denotation_root").unwrap(), DispatchSide::Class);
        let mut types = TypeExportContext::new(store);
        let lineage = types.expected_lineage();
        for (index, ty) in type_roots.iter().copied().enumerate() {
            types
                .register_type_root(
                    lineage,
                    CanonicalTypeRootKey {
                        family_tag: CanonicalRootFamilyTag::CallableAnalysis,
                        owner: StableOwnerRef::Callable(crate::metadata::stable_identity::to_stable_callable(&root_callable)),
                        product_subkey: index as u32,
                    },
                    ty,
                )
                .unwrap();
        }
        types.close_graph(store, &ValidationLimits::default()).unwrap();
        let portable = crate::artifact::export::export_semantic_denotation(denotation, &types).unwrap();
        let graph = types.project_graph(store, &ValidationLimits::default()).unwrap();
        let (_loaded_store, relocation) = TypeStore::import_type_graph(&graph, &ValidationLimits::default()).unwrap();
        let relocated_roots = type_roots.iter().map(|&ty| relocation.ty(types.ty(ty).unwrap()).unwrap()).collect::<Vec<_>>();
        let mut allocations = AllocationBudget::new(1 << 20);
        let imported = import_semantic_denotation(&portable, &relocation, &mut allocations).unwrap();
        (imported, allocations.requested(), relocated_roots)
    }

    #[test]
    fn callable_type_knowledge_import_budgets_owned_names_and_rolls_back() {
        let source_name = "unresolved-type-name";
        let portable = phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
            reason: UnknownReasonV1::UnresolvedName { name: source_name.into() },
        };
        let store = TypeStore::new();
        let relocation = TypeRelocation {
            store_id: store.id(),
            types: Vec::new(),
            kinds: Vec::new(),
            parameters: Vec::new(),
            rows: Vec::new(),
            families: Vec::new(),
            variants: Vec::new(),
            domains: Vec::new(),
            constructors: Vec::new(),
            scoped: Vec::new(),
            lambdas: Vec::new(),
        };

        let mut exact = AllocationBudget::new(source_name.len());
        let imported = import_type_knowledge_with_allocation_budget(&portable, &relocation, &mut exact).unwrap();
        assert_eq!(exact.requested(), source_name.len());
        assert_eq!(
            imported,
            crate::types::evidence::TypeKnowledge::Unknown(UnknownReason::UnresolvedName(source_name.into()))
        );

        let mut insufficient = AllocationBudget::new(source_name.len() - 1);
        let error = import_type_knowledge_with_allocation_budget(&portable, &relocation, &mut insufficient).unwrap_err();
        assert!(matches!(error, ArtifactError::Allocation(AllocationError::Exhausted)));
        assert_eq!(insufficient.requested(), 0, "failed nested reconstruction must not debit the caller");
    }

    #[test]
    fn associated_denotations_round_trip_optional_targets_and_captured_member_sequence() {
        use crate::types::denotation::{AssociatedValueDenotation, CapturedAssociatedMember, SemanticDenotation};
        use crate::types::family::FamilyOperationShape;
        use phalcom_common::selector::{SelectorBase, SelectorSlot};
        use std::sync::Arc;

        let owner = owner();
        let mut store = TypeStore::new();
        let owner_form = store.nominal(owner.clone());
        let variant = VariantId::new(owner.clone(), Selector::getter("Ready").unwrap());
        let exact_without_target = SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::exact(
            owner_form,
            owner.clone(),
            AssociatedMemberId::Variant(variant.clone()),
            None,
        )));
        let (restored, requested, relocated_roots) = round_trip_denotation(&exact_without_target, &store, &[owner_form]);
        let SemanticDenotation::AssociatedValue(restored) = restored else {
            panic!("associated exact denotation")
        };
        let AssociatedValueDenotation::Exact {
            owner_form: restored_owner,
            member,
            target,
            ..
        } = &*restored
        else {
            panic!("associated exact payload")
        };
        assert_eq!(*restored_owner, relocated_roots[0]);
        assert_eq!(member, &AssociatedMemberId::Variant(variant.clone()));
        assert_eq!(target.as_ref(), &None);
        assert!(requested > 0, "the owned exact-target box is charged");

        let exact_with_target = SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::exact(
            owner_form,
            owner.clone(),
            AssociatedMemberId::DataConstructor(DataConstructorId::new(owner.clone())),
            Some(crate::identity::InvocationTargetId::DataConstructor(DataConstructorId::new(owner.clone()))),
        )));
        let (restored, _, relocated_roots) = round_trip_denotation(&exact_with_target, &store, &[owner_form]);
        let SemanticDenotation::AssociatedValue(restored) = restored else {
            panic!("associated exact denotation")
        };
        let AssociatedValueDenotation::Exact {
            owner_form: restored_owner,
            target,
            ..
        } = &*restored
        else {
            panic!("associated exact payload")
        };
        assert_eq!(*restored_owner, relocated_roots[0]);
        assert_eq!(
            target.as_ref(),
            &Some(crate::identity::InvocationTargetId::DataConstructor(DataConstructorId::new(owner.clone())))
        );

        let captured = vec![
            CapturedAssociatedMember {
                operation: FamilyOperationShape::getter(),
                member: AssociatedMemberId::Variant(variant.clone()),
                target: None,
            },
            CapturedAssociatedMember {
                operation: FamilyOperationShape::method(vec![SelectorSlot::Label("payload".into())].into_boxed_slice()),
                member: AssociatedMemberId::DataConstructor(DataConstructorId::new(owner.clone())),
                target: Some(crate::identity::InvocationTargetId::DataConstructor(DataConstructorId::new(owner.clone()))),
            },
        ];
        let family_id = AssociatedFamilyId::new(owner.clone(), SelectorBase::Named("selected".into()));
        let subset = SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::Family {
            owner_form,
            lookup_owner: owner.clone(),
            family: family_id,
            members: Arc::from([captured[1].clone()]),
        }));
        let (restored, requested, relocated_roots) = round_trip_denotation(&subset, &store, &[owner_form]);
        let SemanticDenotation::AssociatedValue(restored) = restored else {
            panic!("associated family denotation")
        };
        let AssociatedValueDenotation::Family {
            owner_form: restored_owner,
            members,
            ..
        } = &*restored
        else {
            panic!("associated family payload")
        };
        assert_eq!(*restored_owner, relocated_roots[0]);
        assert_eq!(
            members.as_ref(),
            &[captured[1].clone()],
            "the selected subset and its source order are denotation-owned"
        );
        assert!(requested > 0, "captured member buffers are charged");
    }

    #[test]
    fn behavioral_family_round_trip_preserves_specs_targets_and_optional_selections() {
        use crate::checker::associated::BehavioralFamilySpec;
        use crate::types::denotation::{BehavioralFamilyDenotation, CapturedBehavioralMember, SemanticDenotation};
        use crate::types::family::FamilyOperationShape;
        use phalcom_common::selector::{SelectorKindPattern, SelectorSlot};
        use std::sync::Arc;

        let owner = owner();
        let callable = crate::identity::CallableId::method(
            owner.clone(),
            Selector::method("run", [SelectorSlot::Positional]).unwrap(),
            DispatchSide::Instance,
        );
        let impl_id = crate::identity::ImplId::new(owner.module.clone(), crate::identity::ImplLocalId(9));
        let conditional = crate::dispatch::ConditionalDispatchSelection {
            impl_id,
            callable: callable.clone(),
            declaring_owner: owner.clone(),
            side: DispatchSide::Instance,
        };
        let mut store = TypeStore::new();
        let receiver_type = store.nominal_type(owner.clone());
        let constructor_kind = store.arrow_kind(vec![crate::types::id::KindId::TYPE].into_boxed_slice(), crate::types::id::KindId::TYPE);
        let receiver_form = store.nominal_form(owner.clone(), constructor_kind);
        let receiver_application = store
            .apply_static_form(receiver_form, &[crate::types::store::StaticArgument::Term(receiver_type)])
            .unwrap();
        let spec = BehavioralFamilySpec::Pattern(
            phalcom_common::selector::SelectorPattern::named(
                "run",
                SelectorKindPattern::Exact(phalcom_common::selector::SelectorKind::Method),
                Vec::<SelectorSlot>::new().into_boxed_slice(),
                Vec::<SelectorSlot>::new().into_boxed_slice(),
                true,
            )
            .unwrap(),
        );
        let source = SemanticDenotation::BehavioralFamily(Box::new(BehavioralFamilyDenotation {
            receiver_type,
            receiver_application: Some(receiver_application),
            spec,
            members: Arc::from([
                CapturedBehavioralMember {
                    operation: FamilyOperationShape::method(vec![SelectorSlot::Positional].into_boxed_slice()),
                    target: crate::identity::InvocationTargetId::Behavioral(callable.clone()),
                    conditional: None,
                },
                CapturedBehavioralMember {
                    operation: FamilyOperationShape::method(vec![SelectorSlot::Positional].into_boxed_slice()),
                    target: crate::identity::InvocationTargetId::Behavioral(callable.clone()),
                    conditional: Some(conditional.clone()),
                },
            ]),
        }));
        let (restored, requested, relocated_roots) = round_trip_denotation(&source, &store, &[receiver_type, receiver_application]);
        let SemanticDenotation::BehavioralFamily(restored) = restored else {
            panic!("behavioral-family denotation")
        };
        assert_eq!(restored.receiver_type, relocated_roots[0]);
        assert_eq!(restored.receiver_application, Some(relocated_roots[1]));
        assert_eq!(
            restored.spec,
            match &source {
                SemanticDenotation::BehavioralFamily(family) => family.spec.clone(),
                _ => unreachable!(),
            }
        );
        assert_eq!(restored.members[0].conditional, None);
        assert_eq!(restored.members[1].conditional.as_ref(), Some(&conditional));
        assert!(requested > 0, "captured behavioral members and publication storage are charged");
    }

    #[test]
    fn callable_denotation_import_preserves_ordinals_and_commits_only_complete_records() {
        use crate::types::denotation::{AssociatedValueDenotation, SemanticDenotation};
        use crate::types::id::{TypeId, TypeStoreId};
        use phalcom_compiler_artifact::module_world::SpanV1;
        use phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId;
        use phalcom_compiler_artifact::semantic::world::{
            AssociatedFamilyIdentityV1, AssociatedMemberV1, AssociatedValueDenotationV1, CallableAnalysisRecordV1, CallableAnalysisStatusV1,
            CallableBindingAuthorityV1, CallableBindingConsistencyV1, CallableBindingRowV1, CallableCausalInvalidityV1, CallableExpressionRowV1,
            CallableFlowGraphV1, CallableInitializationStateV1, FamilyOperationShapeV1, InvocationTargetV1, SelectorKindV1, SelectorSlotV1,
            SemanticDenotationV1,
        };

        let owner = owner();
        let callable = crate::identity::CallableId::method(owner.clone(), Selector::method("analyze", []).unwrap(), DispatchSide::Instance);
        let variant = VariantId::new(owner.clone(), Selector::getter("Ready").unwrap());
        let stable_owner = crate::metadata::stable_identity::to_stable_declaration(&owner);
        let make_binding_row = |ordinal: u32, denotation| CallableBindingRowV1 {
            ordinal,
            name: format!("binding_{ordinal}"),
            parameter: None,
            range: SpanV1 { start: 0, end: 0 },
            contract: None,
            current: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
                reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
            },
            denotation,
            consistency: CallableBindingConsistencyV1::Unconstrained,
            causal_invalidity: CallableCausalInvalidityV1::Clean,
            mutable: false,
            initialization: CallableInitializationStateV1::DefinitelyInitialized,
            version: 0,
            explanation: None,
            authority: CallableBindingAuthorityV1::BodyLocalSource {
                site_owner: phalcom_compiler_artifact::semantic::world::CallableAnalysisSourceOwnerV1::Module {
                    module: stable_owner.module.clone(),
                },
                site_local: ordinal,
                declaration_range: SpanV1 { start: 0, end: 0 },
            },
        };
        let record = CallableAnalysisRecordV1 {
            explanations: Vec::new(),
            return_explanation: None,
            diagnostic_cause_count: 0,
            diagnostics: Vec::new(),
            dependencies: Vec::new(),
            semantic_dependencies: Vec::new(),
            callable: crate::metadata::stable_identity::to_stable_callable(&callable),
            body_source_owner: None,
            body_range: SpanV1 { start: 0, end: 3 },
            status: CallableAnalysisStatusV1::Complete,
            exits_unreachable: false,
            flow_graph: CallableFlowGraphV1 {
                nodes: Vec::new(),
                edges: Vec::new(),
                entry: None,
                exits: Vec::new(),
            },
            constructor_super_initializer: None,
            expressions: vec![
                CallableExpressionRowV1 {
                    ordinal: 0,
                    range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
                    knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
                        reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
                    },
                    callable: None,
                    conditional_dispatch: None,
                    conditional_capability: None,
                    trait_dispatch: None,
                    trait_dispatch_candidates: None,
                    contextual_realization: None,
                    causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
                    explanation: None,
                    binding_read_safety: None,
                    binding_read: None,
                    binding_write: None,
                    status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
                    denotation: None,
                    call_specialization: None,
                },
                CallableExpressionRowV1 {
                    ordinal: 1,
                    range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
                    knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
                        reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
                    },
                    callable: None,
                    conditional_dispatch: None,
                    conditional_capability: None,
                    trait_dispatch: None,
                    trait_dispatch_candidates: None,
                    contextual_realization: None,
                    causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
                    explanation: None,
                    binding_read_safety: None,
                    binding_read: None,
                    binding_write: None,
                    status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
                    denotation: None,
                    call_specialization: None,
                },
                CallableExpressionRowV1 {
                    ordinal: 2,
                    range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
                    knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
                        reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
                    },
                    callable: None,
                    conditional_dispatch: None,
                    conditional_capability: None,
                    trait_dispatch: None,
                    trait_dispatch_candidates: None,
                    contextual_realization: None,
                    causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
                    explanation: None,
                    binding_read_safety: None,
                    binding_read: None,
                    binding_write: None,
                    status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
                    denotation: Some(SemanticDenotationV1::TypeForm { ty: ArtifactTypeId(0) }),
                    call_specialization: None,
                },
            ],
            conformance_evidence: vec![],
            for_iteration_lanes: vec![],
            positional_expansions: vec![],
            known_invalid_positional_expansions: vec![],
            sequence_literals: vec![],
            associated_resolutions: vec![],
            callable_reference_resolutions: vec![],
            family_applications: vec![],
            local_proof_graph: None,
            match_resolutions: vec![],
            pattern_site_resolutions: vec![],
            entry_flow: phalcom_compiler_artifact::semantic::world::CallableFlowStateSummaryV1 {
                bindings: vec![],
                fields: vec![],
                fact_count: 0,
            },
            normal_returns: vec![],
            throws: vec![],
            return_validation: phalcom_compiler_artifact::semantic::world::ReturnContractValidationV1::NotApplicable,
            bindings: vec![
                make_binding_row(
                    0,
                    Some(SemanticDenotationV1::AssociatedValue {
                        value: AssociatedValueDenotationV1::Exact {
                            owner_form: ArtifactTypeId(0),
                            lookup_owner: stable_owner.clone(),
                            member: AssociatedMemberV1::Variant {
                                variant: crate::metadata::stable_identity::to_stable_variant(&variant),
                            },
                            target: None,
                        },
                    }),
                ),
                make_binding_row(
                    1,
                    Some(SemanticDenotationV1::AssociatedValue {
                        value: AssociatedValueDenotationV1::Family {
                            owner_form: ArtifactTypeId(0),
                            lookup_owner: stable_owner.clone(),
                            family: AssociatedFamilyIdentityV1 {
                                owner: stable_owner.clone(),
                                selector: TypeMemberSelectorV1::Named { name: "selected".into() },
                            },
                            members: vec![phalcom_compiler_artifact::semantic::world::CapturedAssociatedMemberV1 {
                                operation: FamilyOperationShapeV1 {
                                    kind: SelectorKindV1::Method,
                                    slots: vec![SelectorSlotV1::Label { label: "payload".into() }],
                                },
                                member: AssociatedMemberV1::DataConstructor { owner: stable_owner.clone() },
                                target: Some(InvocationTargetV1::DataConstructor { owner: stable_owner.clone() }),
                            }],
                        },
                    }),
                ),
            ],
        };
        let mut record = phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2::from_v1_with_observed_import_phases(record, Vec::new());
        let relocation = TypeRelocation {
            store_id: TypeStoreId(0),
            types: vec![TypeId(81)],
            kinds: Vec::new(),
            parameters: Vec::new(),
            rows: Vec::new(),
            families: Vec::new(),
            variants: Vec::new(),
            domains: Vec::new(),
            constructors: Vec::new(),
            scoped: Vec::new(),
            lambdas: Vec::new(),
        };

        let expected = crate::types::denotation::SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::exact(
            TypeId(81),
            owner.clone(),
            crate::associated::AssociatedMemberId::Variant(variant),
            None,
        )));
        let expected_family = crate::types::denotation::SemanticDenotation::AssociatedValue(Box::new(AssociatedValueDenotation::Family {
            owner_form: TypeId(81),
            lookup_owner: owner.clone(),
            family: crate::identity::AssociatedFamilyId::new(owner.clone(), phalcom_common::selector::SelectorBase::Named("selected".into())),
            members: Arc::from([crate::types::denotation::CapturedAssociatedMember {
                operation: crate::types::family::FamilyOperationShape::method(
                    vec![phalcom_common::selector::SelectorSlot::Label("payload".into())].into_boxed_slice(),
                ),
                member: crate::associated::AssociatedMemberId::DataConstructor(crate::identity::DataConstructorId::new(owner.clone())),
                target: Some(crate::identity::InvocationTargetId::DataConstructor(crate::identity::DataConstructorId::new(
                    owner.clone(),
                ))),
            }]),
        }));
        let mut measured = AllocationBudget::new(1 << 20);
        let imported = import_callable_denotations(&record, &relocation, &mut measured).unwrap();
        let required = measured.requested();
        assert!(required > 0, "owner vectors and the captured associated value are charged");
        assert_eq!(imported.expressions, vec![(2, SemanticDenotation::TypeForm(TypeId(81)))]);
        assert_eq!(imported.bindings, vec![(0, expected), (1, expected_family)]);

        let mut exact = AllocationBudget::new(required);
        import_callable_denotations(&record, &relocation, &mut exact).unwrap();
        assert_eq!(exact.requested(), required);

        let mut insufficient = AllocationBudget::new(required - 1);
        let error = import_callable_denotations(&record, &relocation, &mut insufficient).unwrap_err();
        assert!(matches!(error, ArtifactError::Allocation(AllocationError::Exhausted)));
        assert_eq!(insufficient.requested(), 0, "a rejected owner graph does not debit the caller");

        record.bindings.push(make_binding_row(
            2,
            Some(SemanticDenotationV1::StaticApplication {
                body: stable_owner.clone(),
                ordinal: 0,
            }),
        ));
        let mut rejected = AllocationBudget::new(1 << 20);
        assert!(import_callable_denotations(&record, &relocation, &mut rejected).is_err());
        assert_eq!(rejected.requested(), 0, "a later fail-closed denotation rolls back earlier owner allocations");
    }

    #[test]
    fn callable_binding_row_import_preserves_full_state_and_exact_source_identity() {
        use crate::checker::analysis::{BindingState, CallableBindingOrigin};
        use crate::checker::binding::{BindingConsistency, BindingContract, BindingContractOrigin};
        use crate::checker::causal::CausalInvalidity;
        use crate::checker::flow::InitializationState;
        use crate::identity::{CallableParameterId, DiagnosticCauseId, ExplanationId, SourceOwner, SourceSiteId, SourceSiteLocalId};
        use crate::types::denotation::SemanticDenotation;
        use crate::types::evidence::{EvidenceOrigin, EvidenceSet, EvidenceStatus, TypeKnowledge};
        use crate::types::id::{TypeId, TypeStoreId};
        use phalcom_compiler_artifact::module_world::SpanV1;
        use phalcom_compiler_artifact::semantic::aggregates::{TraitRefV1, TypeTermV1};
        use phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId;
        use phalcom_compiler_artifact::semantic::world::{
            CallableAnalysisRecordV1, CallableAnalysisRecordV2, CallableAnalysisSourceOwnerV1, CallableAnalysisStatusV1, CallableBindingAuthorityV1,
            CallableBindingConsistencyV1, CallableBindingContractOriginV1, CallableBindingContractV1, CallableBindingRowV1, CallableCausalInvalidityV1,
            CallableFlowGraphV1, CallableQueryOwnerV1, SemanticDenotationV1, TypeKnowledgeV1,
        };

        let declaration = owner();
        let callable = CallableId::method(
            declaration.clone(),
            Selector::method("read", [phalcom_common::selector::SelectorSlot::Label("payload".into())]).unwrap(),
            DispatchSide::Instance,
        );
        let stable_callable = crate::metadata::stable_identity::to_stable_callable(&callable);
        let stable_declaration = crate::metadata::stable_identity::to_stable_declaration(&declaration);
        let module = stable_declaration.module.clone();
        let record = CallableAnalysisRecordV2::from_v1_with_observed_import_phases(
            CallableAnalysisRecordV1 {
                explanations: Vec::new(),
                return_explanation: None,
                diagnostic_cause_count: 0,
                diagnostics: Vec::new(),
                dependencies: Vec::new(),
                semantic_dependencies: Vec::new(),
                callable: stable_callable.clone(),
                body_source_owner: None,
                body_range: SpanV1 { start: 0, end: 12 },
                status: CallableAnalysisStatusV1::Complete,
                exits_unreachable: false,
                flow_graph: CallableFlowGraphV1 {
                    nodes: Vec::new(),
                    edges: Vec::new(),
                    entry: None,
                    exits: Vec::new(),
                },
                constructor_super_initializer: None,
                expressions: vec![phalcom_compiler_artifact::semantic::world::CallableExpressionRowV1 {
                    ordinal: 0,
                    range: SpanV1 { start: 1, end: 4 },
                    knowledge: TypeKnowledgeV1::Known {
                        ty: ArtifactTypeId(0),
                        status: phalcom_compiler_artifact::semantic::world::EvidenceStatusV1::Assumed,
                        origin: phalcom_compiler_artifact::semantic::world::EvidenceOriginV1::Flow,
                        provenance: phalcom_compiler_artifact::semantic::world::EvidenceSetV1 {
                            ranges: vec![SpanV1 { start: 2, end: 3 }],
                            descriptions: vec!["flow fact".into()],
                        },
                    },
                    callable: Some(stable_callable.clone()),
                    conditional_dispatch: Some(phalcom_compiler_artifact::semantic::world::ConditionalDispatchSelectionV1 {
                        impl_id: crate::metadata::stable_identity::to_stable_impl(&crate::identity::ImplId::new(
                            declaration.module.clone(),
                            crate::identity::ImplLocalId(4),
                        )),
                        callable: stable_callable.clone(),
                        declaring_owner: stable_declaration.clone(),
                        side: 0,
                    }),
                    conditional_capability: None,
                    trait_dispatch: None,
                    trait_dispatch_candidates: None,
                    contextual_realization: None,
                    causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::One { cause: 7 },
                    explanation: None,
                    binding_read_safety: Some(phalcom_compiler_artifact::semantic::world::CallableBindingReadSafetyV1::RuntimeCheckRequired),
                    binding_read: Some(0),
                    binding_write: Some(0),
                    status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Blocked {
                        reason: phalcom_compiler_artifact::semantic::world::CallableBlockReasonV1::UnavailableSemanticOwner {
                            owner: CallableQueryOwnerV1::GenericConformance {
                                subject: TypeTermV1::Canonical { ty: ArtifactTypeId(0) },
                                trait_ref: TraitRefV1 {
                                    declaration: stable_declaration.clone(),
                                    arguments: vec![ArtifactTypeId(0)],
                                },
                            },
                        },
                    },
                    denotation: None,
                    call_specialization: None,
                }],
                conformance_evidence: vec![],
                for_iteration_lanes: vec![],
                positional_expansions: vec![],
                known_invalid_positional_expansions: vec![],
                sequence_literals: vec![],
                associated_resolutions: vec![],
                callable_reference_resolutions: vec![],
                family_applications: vec![],
                local_proof_graph: None,
                match_resolutions: vec![],
                pattern_site_resolutions: vec![],
                entry_flow: phalcom_compiler_artifact::semantic::world::CallableFlowStateSummaryV1 {
                    bindings: vec![],
                    fields: vec![],
                    fact_count: 0,
                },
                normal_returns: vec![],
                throws: vec![],
                return_validation: phalcom_compiler_artifact::semantic::world::ReturnContractValidationV1::NotApplicable,
                bindings: vec![CallableBindingRowV1 {
                    ordinal: 0,
                    name: "text".into(),
                    parameter: Some(phalcom_compiler_artifact::semantic::world::CallableParameterIdentityV1 {
                        callable: stable_callable.clone(),
                        index: 3,
                    }),
                    range: SpanV1 { start: 5, end: 9 },
                    contract: Some(CallableBindingContractV1 {
                        ty: ArtifactTypeId(0),
                        origin: CallableBindingContractOriginV1::SourceAnnotation,
                        source: Some(SpanV1 { start: 5, end: 9 }),
                    }),
                    current: TypeKnowledgeV1::Known {
                        ty: ArtifactTypeId(0),
                        status: phalcom_compiler_artifact::semantic::world::EvidenceStatusV1::Assumed,
                        origin: phalcom_compiler_artifact::semantic::world::EvidenceOriginV1::DeveloperAnnotation,
                        provenance: phalcom_compiler_artifact::semantic::world::EvidenceSetV1 {
                            ranges: vec![SpanV1 { start: 1, end: 2 }],
                            descriptions: vec!["annotation".into()],
                        },
                    },
                    denotation: Some(SemanticDenotationV1::TypeForm { ty: ArtifactTypeId(0) }),
                    consistency: CallableBindingConsistencyV1::Blocked {
                        reason: phalcom_compiler_artifact::semantic::world::CallableBlockReasonV1::UnavailableSemanticOwner {
                            owner: CallableQueryOwnerV1::GenericConformance {
                                subject: TypeTermV1::Canonical { ty: ArtifactTypeId(0) },
                                trait_ref: TraitRefV1 {
                                    declaration: stable_declaration.clone(),
                                    arguments: vec![ArtifactTypeId(0)],
                                },
                            },
                        },
                    },
                    causal_invalidity: CallableCausalInvalidityV1::One { cause: 7 },
                    mutable: true,
                    initialization: phalcom_compiler_artifact::semantic::world::CallableInitializationStateV1::MaybeInitialized,
                    version: 4,
                    explanation: Some(9),
                    authority: CallableBindingAuthorityV1::BodyLocalSource {
                        site_owner: CallableAnalysisSourceOwnerV1::Module { module: module.clone() },
                        site_local: 12,
                        declaration_range: SpanV1 { start: 5, end: 9 },
                    },
                }],
            },
            Vec::new(),
        );
        let relocation = TypeRelocation {
            store_id: TypeStoreId(0),
            types: vec![TypeId(81)],
            kinds: Vec::new(),
            parameters: Vec::new(),
            rows: Vec::new(),
            families: Vec::new(),
            variants: Vec::new(),
            domains: Vec::new(),
            constructors: Vec::new(),
            scoped: Vec::new(),
            lambdas: Vec::new(),
        };
        let mut allocations = AllocationBudget::new(1 << 20);
        let rows = import_callable_binding_rows(&record, &relocation, &mut allocations).unwrap();
        let required = allocations.requested();
        assert!(required > 0);
        assert_eq!(rows.len(), 1);

        let conformance = crate::db::QueryKey::GenericConformance(crate::generic_evidence::GenericConformanceKey::new(
            crate::types::parameter::TypeTerm::Canonical(TypeId(81)),
            crate::traits::TraitRef::new(declaration.clone(), vec![TypeId(81)].into_boxed_slice()),
        ));
        let expected = BindingState {
            binding: BindingId(0),
            name: "text".into(),
            parameter: Some(CallableParameterId::new(callable.clone(), 3)),
            range: SourceRange { start: 5, end: 9 },
            contract: Some(BindingContract {
                ty: TypeId(81),
                origin: BindingContractOrigin::SourceAnnotation,
                source: Some(SourceRange { start: 5, end: 9 }),
            }),
            current: TypeKnowledge::known_with_evidence(
                TypeId(81),
                EvidenceStatus::Assumed,
                EvidenceOrigin::DeveloperAnnotation,
                EvidenceSet {
                    ranges: vec![SourceRange { start: 1, end: 2 }],
                    descriptions: vec!["annotation".into()],
                },
            ),
            denotation: Some(SemanticDenotation::TypeForm(TypeId(81))),
            consistency: BindingConsistency::Blocked(crate::types::outcome::BlockReason::UnavailableSemanticOwner(Box::new(conformance))),
            causal_invalidity: CausalInvalidity::One(DiagnosticCauseId(7)),
            mutable: true,
            initialization: InitializationState::MaybeInitialized,
            version: 4,
            explanation: Some(ExplanationId(9)),
        };
        assert_eq!(rows[0].state, expected);
        assert_eq!(rows[0].storage_origin, CallableBindingOrigin::BodyLocal);
        assert_eq!(
            rows[0].source_authority,
            ImportedCallableBindingSourceAuthority::Source(SourceSiteId {
                owner: SourceOwner::Module(declaration.module.clone()),
                local: SourceSiteLocalId(12),
            })
        );
        assert_eq!(rows[0].formal_range, Some(SourceRange { start: 5, end: 9 }));

        let mut status_allocations = AllocationBudget::new(1 << 20);
        let statuses = import_callable_expression_statuses(&record.expressions, &relocation, &mut status_allocations).unwrap();
        let status_required = status_allocations.requested();
        assert_eq!(
            statuses,
            vec![(
                0,
                crate::checker::analysis::AnalysisStatus::Blocked(crate::types::outcome::BlockReason::UnavailableSemanticOwner(match &expected.consistency {
                    BindingConsistency::Blocked(reason) => match reason {
                        crate::types::outcome::BlockReason::UnavailableSemanticOwner(owner) => owner.clone(),
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }),)
            )]
        );
        assert!(status_required > 0);
        let mut status_exact = AllocationBudget::new(status_required);
        import_callable_expression_statuses(&record.expressions, &relocation, &mut status_exact).unwrap();
        assert_eq!(status_exact.requested(), status_required);
        let mut status_short = AllocationBudget::new(status_required - 1);
        let error = import_callable_expression_statuses(&record.expressions, &relocation, &mut status_short).unwrap_err();
        assert!(matches!(error, ArtifactError::Allocation(AllocationError::Exhausted)));
        assert_eq!(status_short.requested(), 0, "a rejected expression status does not debit its parent");

        let mut expression_allocations = AllocationBudget::new(1 << 20);
        let expression_facts = import_callable_expression_facts(&record, &relocation, &mut expression_allocations).unwrap();
        let expression_required = expression_allocations.requested();
        assert!(expression_required > 0);
        assert_eq!(
            expression_facts,
            vec![ImportedCallableExpressionFacts {
                ordinal: 0,
                range: SourceRange { start: 1, end: 4 },
                knowledge: TypeKnowledge::known_with_evidence(
                    TypeId(81),
                    EvidenceStatus::Assumed,
                    EvidenceOrigin::Flow,
                    EvidenceSet {
                        ranges: vec![SourceRange { start: 2, end: 3 }],
                        descriptions: vec!["flow fact".into()],
                    },
                ),
                callable: Some(callable.clone()),
                conditional_dispatch: Some(crate::dispatch::ConditionalDispatchSelection {
                    impl_id: crate::identity::ImplId::new(declaration.module.clone(), crate::identity::ImplLocalId(4)),
                    callable: callable.clone(),
                    declaring_owner: declaration.clone(),
                    side: DispatchSide::Instance,
                }),
                causal_invalidity: CausalInvalidity::One(DiagnosticCauseId(7)),
                binding_read_safety: Some(crate::checker::flow::BindingReadSafety::RuntimeCheckRequired),
                binding_read: Some(0),
                binding_write: Some(0),
            }]
        );
        let mut expression_exact = AllocationBudget::new(expression_required);
        assert_eq!(
            import_callable_expression_facts(&record, &relocation, &mut expression_exact).unwrap(),
            expression_facts
        );
        assert_eq!(expression_exact.requested(), expression_required);
        let mut expression_short = AllocationBudget::new(expression_required - 1);
        let error = import_callable_expression_facts(&record, &relocation, &mut expression_short).unwrap_err();
        assert!(matches!(error, ArtifactError::Allocation(AllocationError::Exhausted)));
        assert_eq!(expression_short.requested(), 0, "a rejected expression owner does not debit its parent");

        let mut exact = AllocationBudget::new(required);
        assert_eq!(import_callable_binding_rows(&record, &relocation, &mut exact).unwrap(), rows);
        assert_eq!(exact.requested(), required);

        let mut one_short = AllocationBudget::new(required - 1);
        let error = import_callable_binding_rows(&record, &relocation, &mut one_short).unwrap_err();
        assert!(matches!(error, ArtifactError::Allocation(AllocationError::Exhausted)));
        assert_eq!(one_short.requested(), 0, "a rejected callable binding owner does not debit its parent");

        let mut retry = AllocationBudget::new(required);
        assert_eq!(import_callable_binding_rows(&record, &relocation, &mut retry).unwrap(), rows);
        assert_eq!(retry.requested(), required);
    }

    fn owner() -> DeclarationId {
        let path = ModulePath::from_components(vec![ModuleComponent::from_identifier("artifact_parity").unwrap()]);
        DeclarationId::new(ModuleId::universe(path), "Packet".into())
    }

    #[test]
    fn callable_query_owner_identity_round_trips_every_admitted_key_kind() {
        use crate::db::QueryKey;
        use crate::generic_evidence::GenericConformanceKey;
        use crate::identity::{CallableId, DispatchSide, FieldId, VariantId};
        use crate::trait_dispatch::{TraitDispatchBucketKey, TraitDispatchTargetFamily};
        use crate::traits::TraitRef;
        use crate::types::parameter::{SelfRole, SelfTypeTerm, TypeTerm};
        use phalcom_common::selector::{Selector, SelectorSlot};

        let declaration = owner();
        let callable = CallableId::method(
            declaration.clone(),
            Selector::method("dispatch", [SelectorSlot::Label("payload".into())]).unwrap(),
            DispatchSide::Instance,
        );
        let keys = [
            QueryKey::CallableSignature(callable),
            QueryKey::FieldSignature(FieldId::new(declaration.clone(), "cached", DispatchSide::Class)),
            QueryKey::GenericConformance(GenericConformanceKey::new(
                TypeTerm::SelfType(SelfTypeTerm {
                    owner: declaration.clone(),
                    side: DispatchSide::Instance,
                    role: SelfRole::ReceiverValue,
                }),
                TraitRef::new(declaration.clone(), Box::<[_]>::default()),
            )),
            QueryKey::TraitDispatchBucket(TraitDispatchBucketKey {
                target_family: TraitDispatchTargetFamily::ExactEnumCase(VariantId::new(declaration.clone(), Selector::getter("case").unwrap())),
                selector: Selector::method("perform", [SelectorSlot::Label("value".into())]).unwrap(),
                side: DispatchSide::Class,
            }),
            QueryKey::TraitDispatchBucket(TraitDispatchBucketKey {
                target_family: TraitDispatchTargetFamily::Declaration(declaration.clone()),
                selector: Selector::getter("perform").unwrap(),
                side: DispatchSide::Instance,
            }),
        ];

        let store = TypeStore::new();
        let types = TypeExportContext::new(&store);
        let relocation = TypeRelocation {
            store_id: store.id(),
            types: Vec::new(),
            kinds: Vec::new(),
            parameters: Vec::new(),
            rows: Vec::new(),
            families: Vec::new(),
            variants: Vec::new(),
            domains: Vec::new(),
            constructors: Vec::new(),
            scoped: Vec::new(),
            lambdas: Vec::new(),
        };

        for key in keys {
            let portable = crate::artifact::export::export_callable_query_owner(&key, &types).unwrap();
            let mut measured = AllocationBudget::new(1 << 20);
            let restored = import_callable_query_owner(&portable, &relocation, &mut measured).unwrap();
            assert_eq!(restored, key);
            let required = measured.requested();
            assert!(required > 0);

            let mut exact = AllocationBudget::new(required);
            assert_eq!(import_callable_query_owner(&portable, &relocation, &mut exact).unwrap(), key);
            assert_eq!(exact.requested(), required);

            let mut one_short = AllocationBudget::new(required - 1);
            let error = import_callable_query_owner(&portable, &relocation, &mut one_short).unwrap_err();
            assert!(matches!(error, ArtifactError::Allocation(AllocationError::Exhausted)));
            assert_eq!(one_short.requested(), 0, "query-owner import rolls back its parent on allocation failure");
        }

        let malformed = QueryKey::GenericConformance(GenericConformanceKey::new(
            TypeTerm::Infer(crate::types::id::InferVarId(7)),
            TraitRef::new(declaration, Box::<[_]>::default()),
        ));
        assert!(crate::artifact::export::export_callable_query_owner(&malformed, &types).is_err());

        let unsupported = QueryKey::ParsedModule(crate::identity::ModuleId::universe(crate::identity::ModulePath::from_components(vec![
            crate::identity::ModuleComponent::from_identifier("query_owner_reject").unwrap(),
        ])));
        assert!(crate::artifact::export::export_callable_query_owner(&unsupported, &types).is_err());
    }

    fn signature(callable: crate::identity::CallableId, owner: &DeclarationId) -> CallableSemanticSignature {
        CallableSemanticSignature {
            owner: owner.clone(),
            side: callable.side,
            selector: callable.selector.clone(),
            callable,
            generics: None,
            parameters: Box::new([]),
            declared_return: crate::declaration_type::DeclaredTypeFact::unknown(UnknownReason::NoTypeEvidence),
            return_validation: ReturnContractValidation::NotApplicable,
            inferred_return: None,
            source: None,
            implementation: ImplementationKind::Phalcom,
            native_id: None,
            effects: EffectSpec::Unknown,
            raises: RaisesSpec::Unknown,
            flow: ReturnFlowSpec::Value,
            lifecycle: NativeLifecycleSpec::UNKNOWN,
        }
    }

    #[test]
    fn typed_call_specialization_arguments_survive_relocation() {
        use crate::types::store::artifact::TypeRelocation;
        use phalcom_compiler_artifact::semantic::type_store::{ArtifactRecordRowId, ArtifactTypeId, ArtifactTypeParameterId, StaticArgumentV1};
        use phalcom_compiler_artifact::semantic::world::{
            CallableCallSpecializationV1, CallableCompilerDerivedConformanceEvidenceV1, CallableCompilerDerivedConformanceKindV1,
            CallableConformanceEvidenceNodeV1, CallableDerivedConformanceEvidenceV1, CallableDerivedConformanceOriginV1, CallableExactConformanceEvidenceV1,
            CallableExpressionRowV1, CallableGenericCallConformanceBindingV1, CallableGenericCallConformanceSourceV1, CallableNestedGenericEvidenceV1,
            CallableOrderedProductFormV1, CallableStaticArgumentBindingV1, CallableStructuralClassificationV1, CallableTypeEnvironmentV1,
            GenericConformanceKeyV1,
        };

        let owner = owner();
        let callable = crate::identity::CallableId::new(
            CallableOwnerId::Declaration(owner),
            Selector::method("ping", []).unwrap(),
            crate::identity::DispatchSide::Instance,
        );
        let trait_declaration = crate::identity::DeclarationId::new(callable.module().clone(), "ConformanceTrait".into());
        let portable_key = || GenericConformanceKeyV1 {
            subject: phalcom_compiler_artifact::semantic::aggregates::TypeTermV1::Canonical { ty: ArtifactTypeId(0) },
            trait_ref: phalcom_compiler_artifact::semantic::aggregates::TraitRefV1 {
                declaration: crate::metadata::stable_identity::to_stable_declaration(&trait_declaration),
                arguments: vec![ArtifactTypeId(0)],
            },
        };
        let compiler_key = portable_key();
        let caller_key = portable_key();
        let source_impl = crate::identity::ImplId::new(callable.module().clone(), crate::identity::ImplLocalId(0));
        let evidence_nodes = vec![
            CallableConformanceEvidenceNodeV1::CompilerDerived {
                evidence: CallableCompilerDerivedConformanceEvidenceV1 {
                    key: compiler_key.clone(),
                    kind: CallableCompilerDerivedConformanceKindV1::OrderedProduct,
                    classification: CallableStructuralClassificationV1::OrderedProduct {
                        form: CallableOrderedProductFormV1::Unit,
                    },
                },
            },
            CallableConformanceEvidenceNodeV1::Exact {
                evidence: CallableExactConformanceEvidenceV1 {
                    source_impl: crate::metadata::stable_identity::to_stable_impl(&source_impl),
                    exact_target: ArtifactTypeId(0),
                    exact_trait_ref: compiler_key.trait_ref.clone(),
                    impl_environment: CallableTypeEnvironmentV1 {
                        bindings: vec![],
                        self_binding: None,
                    },
                    requirement_views: vec![],
                    requirements: vec![],
                    associated_types: vec![],
                    trait_header_contract: vec![],
                    trait_header_conformance_proofs: Some(vec![]),
                },
            },
            CallableConformanceEvidenceNodeV1::Derived {
                evidence: CallableDerivedConformanceEvidenceV1 {
                    origin: CallableDerivedConformanceOriginV1::ConditionalSource { exact_source: 1 },
                    impl_bindings: vec![],
                    nested: vec![
                        CallableNestedGenericEvidenceV1::Conformance { evidence: 0 },
                        CallableNestedGenericEvidenceV1::Equivalent,
                    ],
                },
            },
        ];
        let expressions = vec![
            CallableExpressionRowV1 {
                ordinal: 0,
                range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
                knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
                    reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
                },
                callable: None,
                conditional_dispatch: None,
                conditional_capability: None,
                trait_dispatch: None,
                trait_dispatch_candidates: None,
                contextual_realization: None,
                causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
                explanation: None,
                binding_read_safety: None,
                binding_read: None,
                binding_write: None,
                status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
                denotation: None,
                call_specialization: None,
            },
            CallableExpressionRowV1 {
                ordinal: 1,
                range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
                knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
                    reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
                },
                callable: None,
                conditional_dispatch: None,
                conditional_capability: None,
                trait_dispatch: None,
                trait_dispatch_candidates: None,
                contextual_realization: None,
                causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
                explanation: None,
                binding_read_safety: None,
                binding_read: None,
                binding_write: None,
                status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
                denotation: None,
                call_specialization: None,
            },
            CallableExpressionRowV1 {
                ordinal: 2,
                range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
                knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
                    reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
                },
                callable: None,
                conditional_dispatch: None,
                conditional_capability: None,
                trait_dispatch: None,
                trait_dispatch_candidates: None,
                contextual_realization: None,
                causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
                explanation: None,
                binding_read_safety: None,
                binding_read: None,
                binding_write: None,
                status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
                denotation: None,
                call_specialization: None,
            },
            CallableExpressionRowV1 {
                ordinal: 3,
                range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 0 },
                knowledge: phalcom_compiler_artifact::semantic::world::TypeKnowledgeV1::Unknown {
                    reason: phalcom_compiler_artifact::semantic::aggregates::UnknownReasonV1::NoTypeEvidence,
                },
                callable: None,
                conditional_dispatch: None,
                conditional_capability: None,
                trait_dispatch: None,
                trait_dispatch_candidates: None,
                contextual_realization: None,
                causal_invalidity: phalcom_compiler_artifact::semantic::world::CallableCausalInvalidityV1::Clean,
                explanation: None,
                binding_read_safety: None,
                binding_read: None,
                binding_write: None,
                status: phalcom_compiler_artifact::semantic::world::CallableExpressionStatusV1::Ready,
                denotation: None,
                call_specialization: Some(CallableCallSpecializationV1 {
                    callable: crate::metadata::stable_identity::to_stable_callable(&callable),
                    bindings: vec![
                        CallableStaticArgumentBindingV1 {
                            parameter: ArtifactTypeParameterId(0),
                            argument: StaticArgumentV1::DynamicType,
                        },
                        CallableStaticArgumentBindingV1 {
                            parameter: ArtifactTypeParameterId(1),
                            argument: StaticArgumentV1::Term { term: ArtifactTypeId(0) },
                        },
                        CallableStaticArgumentBindingV1 {
                            parameter: ArtifactTypeParameterId(2),
                            argument: StaticArgumentV1::RecordRow { row: ArtifactRecordRowId(0) },
                        },
                    ],
                    conformances: vec![
                        CallableGenericCallConformanceBindingV1 {
                            callee_conformance: compiler_key,
                            source: CallableGenericCallConformanceSourceV1::CompilerDerived { evidence: 0 },
                        },
                        CallableGenericCallConformanceBindingV1 {
                            callee_conformance: portable_key(),
                            source: CallableGenericCallConformanceSourceV1::Forwarded {
                                caller_conformance: caller_key,
                            },
                        },
                        CallableGenericCallConformanceBindingV1 {
                            callee_conformance: portable_key(),
                            source: CallableGenericCallConformanceSourceV1::Exact { evidence: 1 },
                        },
                        CallableGenericCallConformanceBindingV1 {
                            callee_conformance: portable_key(),
                            source: CallableGenericCallConformanceSourceV1::Derived { evidence: 2 },
                        },
                    ],
                }),
            },
        ];
        let relocated_parameters = [
            crate::types::id::TypeParameterId(71),
            crate::types::id::TypeParameterId(72),
            crate::types::id::TypeParameterId(73),
        ];
        let relocated_type = crate::types::id::TypeId(81);
        let relocated_row = crate::types::id::RecordRowId(91);
        let relocation = TypeRelocation {
            store_id: crate::types::id::TypeStoreId(0),
            types: vec![relocated_type],
            kinds: vec![],
            parameters: relocated_parameters.into(),
            rows: vec![relocated_row],
            families: vec![],
            variants: vec![],
            domains: vec![],
            constructors: vec![],
            scoped: vec![],
            lambdas: vec![],
        };
        let store = TypeStore::new();
        let source_plan_fingerprints = vec![(source_impl.clone(), crate::db::ProductFingerprint::new(37))];
        let mut allocations = AllocationBudget::new(8192);
        let restored = import_callable_call_specializations(
            &expressions,
            &evidence_nodes,
            &relocation,
            &store,
            &source_plan_fingerprints,
            &mut allocations,
            16,
        )
        .unwrap();
        assert_eq!(restored[0].0, 3);
        assert_eq!(restored[0].1.callable, callable);
        assert_eq!(
            restored[0].1.bindings.as_ref(),
            &[
                (relocated_parameters[0], crate::types::store::StaticArgument::DynamicType),
                (relocated_parameters[1], crate::types::store::StaticArgument::Term(relocated_type)),
                (relocated_parameters[2], crate::types::store::StaticArgument::RecordRow(relocated_row)),
            ]
        );
        assert_eq!(restored[0].1.conformances.len(), 4);
        let crate::checker::analysis::GenericCallConformanceSource::CompilerDerived { evidence } = &restored[0].1.conformances[0].source else {
            panic!("compiler-derived evidence source must be preserved");
        };
        assert_eq!(evidence.key.subject, crate::types::parameter::TypeTerm::Canonical(relocated_type));
        assert_eq!(evidence.key.trait_ref.arguments.as_ref(), &[relocated_type]);
        assert_eq!(
            evidence.classification,
            crate::structural::StructuralClassification::OrderedProduct(crate::structural::OrderedProductForm::Unit)
        );
        let crate::checker::analysis::GenericCallConformanceSource::Forwarded { caller_conformance } = &restored[0].1.conformances[1].source else {
            panic!("forwarded evidence source must preserve its caller key");
        };
        assert_eq!(caller_conformance.subject, crate::types::parameter::TypeTerm::Canonical(relocated_type));
        let crate::checker::analysis::GenericCallConformanceSource::Exact { evidence } = &restored[0].1.conformances[2].source else {
            panic!("exact evidence source must be reconstructed");
        };
        assert_eq!(evidence.source_impl, source_impl);
        assert_eq!(evidence.exact_target, relocated_type);
        assert_eq!(evidence.exact_trait_ref.arguments.as_ref(), &[relocated_type]);
        let source_plan_fingerprint = source_plan_fingerprints.iter().find(|(impl_id, _)| impl_id == &source_impl).unwrap().1;
        validate_relocated_conformance_evidence_fingerprint(evidence, source_plan_fingerprint).unwrap();
        let mut stale_evidence = evidence.as_ref().clone();
        stale_evidence.fingerprint = crate::db::ProductFingerprint::new(stale_evidence.fingerprint.raw().wrapping_add(1));
        let error = validate_relocated_conformance_evidence_fingerprint(&stale_evidence, source_plan_fingerprint)
            .expect_err("a stale post-relocation Exact fingerprint must be rejected");
        assert!(error.to_string().contains("fingerprint is stale after relocation"));
        let crate::checker::analysis::GenericCallConformanceSource::Derived { evidence: derived } = &restored[0].1.conformances[3].source else {
            panic!("derived evidence source must be reconstructed");
        };
        let crate::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source } = &derived.origin else {
            panic!("derived conditional source must be preserved");
        };
        assert!(
            std::sync::Arc::ptr_eq(exact_source, evidence),
            "shared exact evidence must remain shared after import"
        );
        let mut wrong_kind = expressions.clone();
        wrong_kind[3].call_specialization.as_mut().unwrap().conformances[0].source = CallableGenericCallConformanceSourceV1::Exact { evidence: 0 };
        let mut negative_allocations = AllocationBudget::new(8192);
        let error = import_callable_call_specializations(
            &wrong_kind,
            &evidence_nodes,
            &relocation,
            &store,
            &source_plan_fingerprints,
            &mut negative_allocations,
            16,
        )
        .expect_err("a compiler-derived node cannot satisfy an Exact binding");
        assert!(error.to_string().contains("references a node of the wrong kind"));

        let mut dangling = expressions.clone();
        dangling[3].call_specialization.as_mut().unwrap().conformances[2].source = CallableGenericCallConformanceSourceV1::Exact { evidence: u32::MAX };
        let mut negative_allocations = AllocationBudget::new(8192);
        let error = import_callable_call_specializations(
            &dangling,
            &evidence_nodes,
            &relocation,
            &store,
            &source_plan_fingerprints,
            &mut negative_allocations,
            16,
        )
        .expect_err("a dangling evidence ordinal must be rejected");
        assert!(error.to_string().contains("reference is out of range"));

        let mut cyclic_nodes = evidence_nodes.clone();
        let CallableConformanceEvidenceNodeV1::Derived { evidence } = &mut cyclic_nodes[2] else {
            panic!("fixture node must remain Derived");
        };
        evidence.nested = vec![CallableNestedGenericEvidenceV1::Conformance { evidence: 2 }];
        let mut negative_allocations = AllocationBudget::new(8192);
        let error = import_callable_call_specializations(
            &expressions,
            &cyclic_nodes,
            &relocation,
            &store,
            &source_plan_fingerprints,
            &mut negative_allocations,
            16,
        )
        .expect_err("a recursive evidence graph must be rejected");
        assert!(error.to_string().contains("contains a cycle"));

        let mut negative_allocations = AllocationBudget::new(8192);
        let error = import_callable_call_specializations(&expressions, &evidence_nodes, &relocation, &store, &[], &mut negative_allocations, 16)
            .expect_err("Exact evidence without its source witness fingerprint must be rejected");
        assert!(error.to_string().contains("no reconstructed source witness plan"));
        assert!(import_callable_call_specializations(&[], &[], &relocation, &store, &source_plan_fingerprints, &mut allocations, 16).is_ok());
    }

    #[test]
    fn enum_requirements_round_trip_source_order_and_all_case_statuses() {
        let owner = owner();
        let mut store = TypeStore::new();
        let owner_type = store.nominal(owner.clone());
        let lane = store.type_pack(vec![owner_type].into_boxed_slice()).unwrap();
        let area = crate::identity::CallableId::new(
            CallableOwnerId::Declaration(owner.clone()),
            Selector::method("area", []).unwrap(),
            crate::identity::DispatchSide::Instance,
        );
        let width = crate::identity::CallableId::new(
            CallableOwnerId::Declaration(owner.clone()),
            Selector::method("width", []).unwrap(),
            crate::identity::DispatchSide::Instance,
        );
        let circle = VariantId::new(owner.clone(), Selector::getter("Circle").unwrap());
        let square = VariantId::new(owner.clone(), Selector::getter("Square").unwrap());
        let circle_area = crate::identity::CallableId::new(
            CallableOwnerId::Variant(circle.clone()),
            Selector::method("area", []).unwrap(),
            crate::identity::DispatchSide::Instance,
        );
        let circle_width = crate::identity::CallableId::new(
            CallableOwnerId::Variant(circle.clone()),
            Selector::method("width", []).unwrap(),
            crate::identity::DispatchSide::Instance,
        );

        let area_signature = signature(area.clone(), &owner);
        let fixed_fact = crate::declaration_type::DeclaredTypeFact::known(
            crate::types::parameter::TypeTerm::Canonical(owner_type),
            crate::declaration_type::DeclaredTypeBasis::SourceAnnotation,
        );
        let lane_declared = crate::declaration_type::DeclaredTypeFact::unknown(UnknownReason::NoTypeEvidence);
        let invalid_reason = UnknownReason::InferenceBlocked;
        let parameter_source = |start, end| Some(SemanticSourceSpan::new(owner.module.clone(), SourceRange { start, end }));
        let width_signature_source = SemanticSourceSpan::new(owner.module.clone(), SourceRange { start: 3, end: 10 });
        let mut width_signature = signature(width.clone(), &owner);
        width_signature.parameters = vec![
            crate::signature::CallableParameterSemantic {
                id: crate::identity::CallableParameterId::new(width.clone(), 0),
                local_name: "value".into(),
                external_label: Some("value".into()),
                rest: phalcom_ast::ast::RestMode::None,
                declared_type: fixed_fact.clone(),
                contract: crate::signature::ParameterContract::Fixed(fixed_fact.clone()),
                source: parameter_source(11, 16),
            },
            crate::signature::CallableParameterSemantic {
                id: crate::identity::CallableParameterId::new(width.clone(), 1),
                local_name: "items".into(),
                external_label: None,
                rest: phalcom_ast::ast::RestMode::Positional,
                declared_type: fixed_fact.clone(),
                contract: crate::signature::ParameterContract::Rest {
                    mode: phalcom_ast::ast::RestMode::Positional,
                    contract: crate::signature::RestValueContract::Element(fixed_fact.clone()),
                },
                source: parameter_source(17, 23),
            },
            crate::signature::CallableParameterSemantic {
                id: crate::identity::CallableParameterId::new(width.clone(), 2),
                local_name: "named".into(),
                external_label: Some("named".into()),
                rest: phalcom_ast::ast::RestMode::Labeled,
                declared_type: lane_declared.clone(),
                contract: crate::signature::ParameterContract::Rest {
                    mode: phalcom_ast::ast::RestMode::Labeled,
                    contract: crate::signature::RestValueContract::Lane(lane),
                },
                source: parameter_source(24, 30),
            },
            crate::signature::CallableParameterSemantic {
                id: crate::identity::CallableParameterId::new(width.clone(), 3),
                local_name: "all".into(),
                external_label: None,
                rest: phalcom_ast::ast::RestMode::Complete,
                declared_type: crate::declaration_type::DeclaredTypeFact::unknown(invalid_reason.clone()),
                contract: crate::signature::ParameterContract::Rest {
                    mode: phalcom_ast::ast::RestMode::Complete,
                    contract: crate::signature::RestValueContract::Invalid(invalid_reason),
                },
                source: parameter_source(31, 35),
            },
        ]
        .into_boxed_slice();
        width_signature.declared_return = fixed_fact;
        width_signature.return_validation = ReturnContractValidation::Satisfied(crate::types::evidence::EvidenceStatus::Assumed);
        width_signature.inferred_return = Some(
            crate::types::evidence::TypeKnowledge::established(owner_type, crate::types::evidence::EvidenceOrigin::Flow)
                .with_range(SourceRange { start: 36, end: 41 }),
        );
        width_signature.source = Some(width_signature_source);
        width_signature.flow = ReturnFlowSpec::Argument(1);

        let source_range = SourceRange { start: 11, end: 19 };
        let requirements: Arc<[EnumRequirement]> = Arc::from(
            vec![
                EnumRequirement {
                    id: crate::enum_requirements::EnumRequirementId::new(owner.clone(), width.selector.clone()),
                    signature: width_signature,
                    source: Some(SemanticSourceSpan::new(owner.module.clone(), source_range)),
                },
                EnumRequirement {
                    id: crate::enum_requirements::EnumRequirementId::new(owner.clone(), area.selector.clone()),
                    signature: area_signature,
                    source: None,
                },
            ]
            .into_boxed_slice(),
        );
        let case_statuses: Arc<[CaseRequirementResult]> = Arc::from(
            vec![
                CaseRequirementResult {
                    variant: circle.clone(),
                    requirement: requirements[0].id.clone(),
                    status: CaseRequirementStatus::Satisfied {
                        implementation: circle_width.clone(),
                    },
                },
                CaseRequirementResult {
                    variant: square.clone(),
                    requirement: requirements[0].id.clone(),
                    status: CaseRequirementStatus::Missing,
                },
                CaseRequirementResult {
                    variant: circle.clone(),
                    requirement: requirements[1].id.clone(),
                    status: CaseRequirementStatus::Incompatible {
                        implementation: circle_area.clone(),
                    },
                },
                CaseRequirementResult {
                    variant: square.clone(),
                    requirement: requirements[1].id.clone(),
                    status: CaseRequirementStatus::Blocked,
                },
            ]
            .into_boxed_slice(),
        );
        let mut source_table = EnumRequirementTable::new();
        source_table.insert(owner.clone(), requirements.clone(), case_statuses.clone());

        let unit = store.unit();
        let mut enum_semantics = EnumSemanticTable::new();
        enum_semantics.insert_enum(Arc::new(crate::enum_semantics::EnumInfo {
            owner: owner.clone(),
            root_form: unit,
            generic_signature: None,
            static_data: None,
            default_result_type: unit,
            variants: vec![circle.clone(), square.clone()].into_boxed_slice(),
            variant_families: Box::new([]),
            source: None,
        }));

        let mut export_types = TypeExportContext::new(&store);
        crate::artifact::roots::register_callable_signature_roots(&mut export_types, &requirements[0].signature).unwrap();
        crate::artifact::roots::register_callable_signature_roots(&mut export_types, &requirements[1].signature).unwrap();
        export_types.close_graph(&store, &ValidationLimits::default()).unwrap();
        let type_graph = export_types.project_graph(&store, &ValidationLimits::default()).unwrap();
        let portable = crate::artifact::export::export_enum_requirements(&source_table, &enum_semantics, &export_types).unwrap();
        assert_eq!(portable[0].requirements[0].requirement.selector.as_ref(), "area()");
        assert_eq!(portable[0].requirements[0].source_order, 1);
        assert_eq!(portable[0].requirements[1].source_order, 0);
        let (restored_store, relocation) = TypeStore::import_type_graph(&type_graph, &ValidationLimits::default()).unwrap();
        let mut allocation_budget = AllocationBudget::new(usize::MAX);
        let restored = import_enum_requirements(&portable, &enum_semantics, &relocation, &restored_store, &mut allocation_budget).unwrap();
        let required = allocation_budget.requested();
        assert!(required > 0);

        let mut exact_budget = AllocationBudget::new(required);
        import_enum_requirements(&portable, &enum_semantics, &relocation, &restored_store, &mut exact_budget).unwrap();
        assert_eq!(
            exact_budget.requested(),
            required,
            "budgeted vector rows and Arc owners fit at the measured boundary"
        );

        let mut short_budget = AllocationBudget::new(required - 1);
        let error = import_enum_requirements(&portable, &enum_semantics, &relocation, &restored_store, &mut short_budget).unwrap_err();
        assert!(matches!(error, ArtifactError::Allocation(AllocationError::Exhausted)));

        let mut restored_types = TypeExportContext::new(&restored_store);
        for requirement in restored.requirements.get(&owner).unwrap().iter() {
            crate::artifact::roots::register_callable_signature_roots(&mut restored_types, &requirement.signature).unwrap();
        }
        restored_types.close_graph(&restored_store, &ValidationLimits::default()).unwrap();
        let restored_portable = crate::artifact::export::export_enum_requirements(&restored, &enum_semantics, &restored_types).unwrap();
        assert_eq!(restored_portable, portable, "every owner-held callable signature field survives relocation");
        assert_eq!(restored.case_statuses.get(&owner).unwrap().as_ref(), case_statuses.as_ref());
        assert_eq!(allocation_budget.requested(), required);
    }

    #[test]
    fn callable_native_metadata_must_match_the_trusted_catalog_row() {
        let owner = owner();
        let callable = crate::identity::CallableId::new(
            CallableOwnerId::Declaration(owner.clone()),
            Selector::method("native", []).unwrap(),
            crate::identity::DispatchSide::Instance,
        );
        let native = phalcom_native_surface::NATIVE_SURFACES.first().expect("native catalog has a row");
        let mut signature = signature(callable, &owner);
        signature.implementation = ImplementationKind::Native;
        signature.native_id = Some(native.id());
        signature.effects = native.effects();
        signature.raises = native.raises();
        signature.flow = native.flow();
        signature.lifecycle = native.lifecycle();

        let store = TypeStore::new();
        let types = TypeExportContext::new(&store);
        let portable = crate::artifact::export::export_callable_signature(&signature, &types).unwrap();
        assert!(import_native_metadata(&portable).is_ok());

        let mut tampered = portable;
        tampered.effects = if matches!(tampered.effects, phalcom_compiler_artifact::semantic::world::EffectSpecV1::Unknown) {
            phalcom_compiler_artifact::semantic::world::EffectSpecV1::Pure
        } else {
            phalcom_compiler_artifact::semantic::world::EffectSpecV1::Unknown
        };
        assert!(
            import_native_metadata(&tampered).is_err(),
            "catalog identity cannot authorize mismatched native metadata"
        );
    }

    fn minimal_callable_analysis_record() -> phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2 {
        let callable = StableCallableRef {
            owner: StableCallableOwnerRef::Declaration(StableDeclarationRef {
                module: StableModuleRef {
                    project: StableProjectRef::Builtin {
                        namespace: "universe".into(),
                        version: "0.1.0".into(),
                    },
                    path: vec!["test".into()].into_boxed_slice(),
                },
                path: vec!["Owner".into()].into_boxed_slice(),
            }),
            side: StableDispatchSide::Instance,
            selector: "method".into(),
        };
        phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV2::from_v1_with_observed_import_phases(
            phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV1 {
                explanations: Vec::new(),
                return_explanation: None,
                diagnostic_cause_count: 0,
                diagnostics: Vec::new(),
                dependencies: Vec::new(),
                semantic_dependencies: Vec::new(),
                callable,
                body_source_owner: None,
                body_range: phalcom_compiler_artifact::module_world::SpanV1 { start: 0, end: 8 },
                status: phalcom_compiler_artifact::semantic::world::CallableAnalysisStatusV1::Complete,
                exits_unreachable: false,
                flow_graph: phalcom_compiler_artifact::semantic::world::CallableFlowGraphV1 {
                    nodes: Vec::new(),
                    edges: Vec::new(),
                    entry: None,
                    exits: Vec::new(),
                },
                constructor_super_initializer: None,
                expressions: Vec::new(),
                conformance_evidence: Vec::new(),
                for_iteration_lanes: Vec::new(),
                positional_expansions: Vec::new(),
                known_invalid_positional_expansions: Vec::new(),
                sequence_literals: Vec::new(),
                associated_resolutions: Vec::new(),
                callable_reference_resolutions: Vec::new(),
                family_applications: Vec::new(),
                local_proof_graph: None,
                match_resolutions: Vec::new(),
                pattern_site_resolutions: Vec::new(),
                entry_flow: phalcom_compiler_artifact::semantic::world::CallableFlowStateSummaryV1 {
                    bindings: Vec::new(),
                    fields: Vec::new(),
                    fact_count: 0,
                },
                normal_returns: Vec::new(),
                throws: Vec::new(),
                return_validation: phalcom_compiler_artifact::semantic::world::ReturnContractValidationV1::NotApplicable,
                bindings: Vec::new(),
            },
            Vec::new(),
        )
    }

    #[test]
    fn callable_interface_observation_is_covered_by_runtime_module_summary() {
        use phalcom_compiler_artifact::module_world::{
            AliasV1, ArtifactModuleId, ImportPathV1, ImportRootV1, ImportV1, ModuleKindV1, ModuleV2, ModuleWorldV2, SpanV1,
        };
        use phalcom_compiler_artifact::semantic::world::{CallableObservedImportPhaseV1, DependencyPhaseV1};
        use phalcom_modules::graph::DependencyPhase;
        use phalcom_modules::identity::{ImportBindingLocalId, ImportSiteId, ImportSiteLocalId, SourceImportBindingId};
        use phalcom_modules::linker::{ImportPhaseSummary, ModuleImportPhaseProduct, PhaseEvidenceCoverage};

        let mut record = minimal_callable_analysis_record();
        record.observed_import_phases.push(CallableObservedImportPhaseV1 {
            binding: 0,
            phase: DependencyPhaseV1::InterfaceOnly,
        });
        let stable_module = stable_callable_module(&record.callable).clone();
        let span = SpanV1 { start: 0, end: 0 };
        let module_world = ModuleWorldV2 {
            entry: ArtifactModuleId(0),
            initialization_order: Vec::new(),
            modules: vec![ModuleV2 {
                identity: stable_module.clone(),
                path: vec!["test".into()],
                kind: ModuleKindV1::Module,
                declarations: Vec::new(),
                exports: Vec::new(),
                imports: vec![ImportV1::Module {
                    path: ImportPathV1 {
                        root: ImportRootV1::Relative { dots: 1, span: span.clone() },
                        segments: Vec::new(),
                        span: span.clone(),
                    },
                    alias: Some(AliasV1 {
                        name: "dependency".into(),
                        span: span.clone(),
                    }),
                    span,
                }],
                exposed_children: Vec::new(),
                metadata: Vec::new(),
                linked_exports: Vec::new(),
                globals: Vec::new(),
                import_bindings: Vec::new(),
                linked_reads: Vec::new(),
                runtime_dependencies: Vec::new(),
                references: Vec::new(),
                runtime_edges: Vec::new(),
                topology_parent: None,
                topology_children: Vec::new(),
                interface_fingerprint: 0,
                linked_interface_fingerprint: 0,
                dependency_fingerprint: 0,
            }],
            semantic_rows: Vec::new(),
            external_projects: Vec::new(),
            topology_fingerprint: 0,
        };
        let module = import_stable_module_with_allocation_budget(&stable_module, &mut AllocationBudget::new(4096)).unwrap();
        let binding = SourceImportBindingId {
            site: ImportSiteId::new(module.clone(), ImportSiteLocalId::new(0)),
            local: ImportBindingLocalId(0),
        };
        let product = ModuleImportPhaseProduct {
            module: module.clone(),
            bindings: std::collections::BTreeMap::from([(
                binding.clone(),
                ImportPhaseSummary {
                    phase: DependencyPhase::Runtime,
                    observed: true,
                },
            )])
            .into(),
            coverage: PhaseEvidenceCoverage::Complete,
        };
        let products = std::collections::BTreeMap::from([(module.clone(), product)]);
        let mut allocations = AllocationBudget::new(4096);
        let callable = import_stable_callable_with_allocation_budget(&record.callable, &mut allocations).unwrap();
        let phases = import_callable_observed_import_phases(&record, &callable, &module_world, &products, &mut allocations)
            .expect("a runtime aggregate summary covers an interface-only callable observation");
        assert_eq!(phases.get(&binding), Some(&DependencyPhase::InterfaceOnly));

        let stricter_product = ModuleImportPhaseProduct {
            module: module.clone(),
            bindings: std::collections::BTreeMap::from([(
                binding,
                ImportPhaseSummary {
                    phase: DependencyPhase::InterfaceOnly,
                    observed: true,
                },
            )])
            .into(),
            coverage: PhaseEvidenceCoverage::Complete,
        };
        let stricter_products = std::collections::BTreeMap::from([(module, stricter_product)]);
        let mut uncovered_record = record.clone();
        uncovered_record.observed_import_phases[0].phase = DependencyPhaseV1::Runtime;
        assert!(
            import_callable_observed_import_phases(
                &uncovered_record,
                &callable,
                &module_world,
                &stricter_products,
                &mut AllocationBudget::new(4096),
            )
            .is_err()
        );
    }

    #[test]
    fn callable_analysis_import_validates_complete_owner_links() {
        let mut record = minimal_callable_analysis_record();
        assert!(ensure_complete_callable_analysis_payload(std::slice::from_ref(&record)).is_ok());
        record.return_explanation = Some(0);
        assert!(ensure_complete_callable_analysis_payload(&[record]).is_err());
        assert!(ensure_complete_callable_analysis_payload(&[]).is_ok());
    }

    #[test]
    fn reconstructed_callable_table_retains_lowering_readers_and_relocated_fingerprint() {
        use phalcom_compiler_artifact::module_world::SpanV1;
        use phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId;
        use phalcom_compiler_artifact::semantic::world as wire;

        let limits = ValidationLimits::default();
        let mut source_store = TypeStore::new();
        let source_type = source_store.nominal(owner());
        let types = TypeExportContext::new_full(&source_store).unwrap();
        let graph = types.project_graph(&source_store, &limits).unwrap();
        let (store, relocation) = TypeStore::import_type_graph(&graph, &limits).unwrap();
        let portable_type = types.ty(source_type).unwrap();
        let relocated_type = relocation.ty(portable_type).unwrap();
        let mut record = minimal_callable_analysis_record();
        let callable = import_exact_callable(&record.callable).unwrap();
        let knowledge = wire::TypeKnowledgeV1::Known {
            ty: portable_type,
            status: wire::EvidenceStatusV1::Established,
            origin: wire::EvidenceOriginV1::Syntax,
            provenance: wire::EvidenceSetV1 {
                ranges: vec![],
                descriptions: vec![],
            },
        };
        record.bindings.push(wire::CallableBindingRowV1 {
            ordinal: 0,
            name: "value".into(),
            parameter: None,
            range: SpanV1 { start: 2, end: 7 },
            contract: None,
            current: knowledge.clone(),
            denotation: None,
            consistency: wire::CallableBindingConsistencyV1::Validated,
            causal_invalidity: wire::CallableCausalInvalidityV1::Clean,
            mutable: true,
            initialization: wire::CallableInitializationStateV1::DefinitelyInitialized,
            version: 1,
            explanation: None,
            authority: wire::CallableBindingAuthorityV1::BodyLocalSource {
                site_owner: wire::CallableAnalysisSourceOwnerV1::Callable {
                    callable: record.callable.clone(),
                },
                site_local: 0,
                declaration_range: SpanV1 { start: 2, end: 7 },
            },
        });
        record.expressions.push(wire::CallableExpressionRowV1 {
            ordinal: 0,
            range: SpanV1 { start: 7, end: 8 },
            knowledge,
            callable: None,
            conditional_dispatch: None,
            conditional_capability: None,
            trait_dispatch: None,
            trait_dispatch_candidates: None,
            contextual_realization: None,
            status: wire::CallableExpressionStatusV1::Ready,
            causal_invalidity: wire::CallableCausalInvalidityV1::Clean,
            explanation: None,
            binding_read_safety: Some(wire::CallableBindingReadSafetyV1::ProvenInitialized),
            binding_read: Some(0),
            binding_write: Some(0),
            denotation: Some(wire::SemanticDenotationV1::TypeForm { ty: portable_type }),
            call_specialization: None,
        });
        record.sequence_literals.push(wire::CallableSequenceLiteralV1 {
            expression: 0,
            source_range: SpanV1 { start: 7, end: 8 },
            family: wire::CallableSequenceFamilyV1::List,
            result_type: portable_type,
            element_type: portable_type,
            extent: None,
            construction: wire::CallableSequenceConstructionV1::Fixed { element_count: 1 },
            exact_expansions: vec![],
        });
        let mut budget = AllocationBudget::new(1 << 20);
        let (table, ranges) = import_callable_analysis_records(
            std::slice::from_ref(&record),
            &relocation,
            &store,
            &[],
            None,
            &BTreeMap::new(),
            &limits,
            &mut budget,
        )
        .unwrap();
        let analysis = table.get(&callable).unwrap();
        assert_eq!(table.iter_module(callable.module()).count(), 1);
        assert!(analysis.expressions.is_frozen());
        assert!(analysis.bindings.is_frozen());
        assert_eq!(analysis.expressions.values().next().unwrap().knowledge.ty(), Some(relocated_type));
        assert_eq!(analysis.expressions.values().next().unwrap().binding_read, Some(BindingId(0)));
        assert_eq!(analysis.expressions.values().next().unwrap().binding_write, Some(BindingId(0)));
        assert_eq!(analysis.sequence_literals.values().next().unwrap().element_type, relocated_type);
        assert_eq!(
            analysis.dependency_fingerprint,
            crate::db::fingerprint::callable_body_product_fingerprint(analysis)
        );
        let projection =
            crate::presentation::FormalSemanticProjection::from_callable_analysis_table_with_binding_ranges_and_allocation_budget(&table, &ranges, &mut budget)
                .unwrap();
        assert_eq!(
            projection.binding_attachment_range(&callable, BindingId(0)),
            Some(SourceRange { start: 2, end: 7 })
        );

        record.expressions[0].binding_write = Some(1);
        let mut rejected = AllocationBudget::new(1 << 20);
        assert!(import_callable_analysis_records(&[record], &relocation, &store, &[], None, &BTreeMap::new(), &limits, &mut rejected).is_err());
        assert_eq!(rejected.requested(), 0);
        let _ = ArtifactTypeId(0);
    }

    fn insert_family(
        surface: &mut AssociatedSurface,
        owner: &DeclarationId,
        base: SelectorBase,
        kind: AssociatedFamilyKind,
        members: Box<[AssociatedMemberId]>,
    ) {
        surface.families.insert(
            base.clone(),
            AssociatedFamilyInfo {
                id: AssociatedFamilyId::new(owner.clone(), base),
                kind,
                members,
            },
        );
    }

    #[test]
    fn associated_authority_round_trips_through_type_relocation() {
        let owner = owner();
        let mut store = TypeStore::new();
        let unit = store.unit();

        let member_id = TypeMemberDeclarationId::new(owner.clone(), TypeMemberSelector::named("Map"));
        let member_product = Arc::new(TypeMemberDeclarationProduct {
            id: member_id.clone(),
            signature: TypeMemberSignature {
                selector: member_id.selector.clone(),
                parameters: Box::new([
                    TypeMemberParameter {
                        name: "mapper".into(),
                        domain: TypeMemberDomain::Kind(crate::types::id::KindId::TYPE),
                    },
                    TypeMemberParameter {
                        name: "fallback".into(),
                        domain: TypeMemberDomain::ValueType(unit),
                    },
                ]),
                result_domain: TypeMemberDomain::Kind(crate::types::id::KindId::TYPE),
                intrinsic: Some(IntrinsicTypeOperation::RecordMap),
                implementation: ImplementationKind::Intrinsic,
            },
            source_range: SourceRange::default(),
        });
        let type_members = [(member_id.clone(), member_product)]
            .into_iter()
            .collect::<crate::parent_map::ParentMap<_, _>>();

        let static_id = AssociatedStaticDomainDeclarationId::new(owner.clone(), "State");
        let static_domain = store.associated_static_domain(static_id.clone(), Box::new([]));
        let static_constructor = store.intern_static_constructor(static_domain, owner.clone(), "Ready", Box::new([]));
        let static_product = Arc::new(AssociatedStaticDomainDeclarationProduct {
            id: static_id.clone(),
            domain: static_domain,
            constructors: Box::new([static_constructor]),
            source_range: SourceRange::default(),
        });
        let static_domains = crate::ordered_map::OrderedMap::from_source([(static_id.clone(), static_product)].into_iter().collect::<BTreeMap<_, _>>());

        let mut surface = AssociatedSurface::new(owner.clone());
        let variant = VariantId::new(owner.clone(), Selector::getter("Ready").unwrap());
        insert_family(
            &mut surface,
            &owner,
            SelectorBase::Named("Ready".into()),
            AssociatedFamilyKind::Variant,
            Box::new([AssociatedMemberId::Variant(variant)]),
        );
        insert_family(
            &mut surface,
            &owner,
            SelectorBase::Named("Construct".into()),
            AssociatedFamilyKind::DataConstructor,
            Box::new([AssociatedMemberId::DataConstructor(DataConstructorId::new(owner.clone()))]),
        );
        insert_family(
            &mut surface,
            &owner,
            SelectorBase::Named("State".into()),
            AssociatedFamilyKind::StaticDomain,
            Box::new([AssociatedMemberId::StaticDomain(static_id.clone())]),
        );
        insert_family(
            &mut surface,
            &owner,
            SelectorBase::Named("Map".into()),
            AssociatedFamilyKind::IntrinsicTypeMember,
            Box::new([AssociatedMemberId::TypeMember(member_id.clone())]),
        );
        let mut surfaces = AssociatedFamilyTable::new();
        surfaces.insert(owner.clone(), Arc::new(surface));

        let type_export = TypeExportContext::new_full(&store).unwrap();
        let exported = crate::artifact::export::export_associated_authority(&type_members, &static_domains, &surfaces, &type_export).unwrap();
        let graph = type_export.project_graph(&store, &ValidationLimits::default()).unwrap();
        let (_imported_store, relocation) = TypeStore::import_type_graph(&graph, &ValidationLimits::default()).unwrap();

        let imported = import_associated_authority(
            &exported.type_member_declarations,
            &exported.associated_static_domains,
            &exported.associated_surfaces,
            &relocation,
            &mut AllocationBudget::new(usize::MAX),
        )
        .unwrap();

        let imported_member = imported.type_member_declarations.get(&member_id).unwrap();
        assert_eq!(imported_member.id, member_id);
        assert_eq!(imported_member.signature.selector, TypeMemberSelector::named("Map"));
        assert_eq!(imported_member.signature.intrinsic, Some(IntrinsicTypeOperation::RecordMap));
        assert_eq!(imported_member.signature.implementation, ImplementationKind::Intrinsic);
        assert_eq!(
            imported_member.signature.parameters[1].domain,
            TypeMemberDomain::ValueType(relocation.ty(type_export.ty(unit).unwrap()).unwrap())
        );

        let imported_static = imported.associated_static_domains.get(&static_id).unwrap();
        assert_eq!(imported_static.id, static_id);
        assert_eq!(imported_static.domain, relocation.domain(type_export.domain(static_domain).unwrap()).unwrap());
        assert_eq!(
            imported_static.constructors.as_ref(),
            [relocation.constructor(type_export.constructor(static_constructor).unwrap()).unwrap()]
        );
        assert_eq!(imported.associated_surfaces, surfaces);
    }

    fn runtime_requirement_module_world() -> (phalcom_compiler_artifact::module_world::ModuleWorldV2, ModuleId, ModuleId) {
        use phalcom_compiler_artifact::module_world::{ArtifactModuleId, ModuleKindV1, ModuleV2, ModuleWorldV2};

        let root = ModuleId::universe_root();
        let dependency = ModuleId::universe(ModulePath::from_components(vec![ModuleComponent::from_identifier("core").unwrap()]));
        let project = StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        };
        let module = |identity: StableModuleRef,
                      path: Vec<String>,
                      kind: ModuleKindV1,
                      topology_parent: Option<ArtifactModuleId>,
                      topology_children: Vec<ArtifactModuleId>| ModuleV2 {
            identity,
            path,
            kind,
            declarations: vec![],
            exports: vec![],
            imports: vec![],
            exposed_children: vec![],
            metadata: vec![],
            linked_exports: vec![],
            globals: vec![],
            import_bindings: vec![],
            linked_reads: vec![],
            runtime_dependencies: vec![],
            references: vec![],
            runtime_edges: vec![],
            topology_parent,
            topology_children,
            interface_fingerprint: 0,
            linked_interface_fingerprint: 0,
            dependency_fingerprint: 0,
        };
        let mut world = ModuleWorldV2 {
            entry: ArtifactModuleId(0),
            initialization_order: vec![ArtifactModuleId(0), ArtifactModuleId(1)],
            modules: vec![
                module(
                    StableModuleRef {
                        project: project.clone(),
                        path: Box::new([]),
                    },
                    vec![],
                    ModuleKindV1::Package,
                    None,
                    vec![ArtifactModuleId(1)],
                ),
                module(
                    StableModuleRef {
                        project,
                        path: Box::new(["core".into()]),
                    },
                    vec!["core".into()],
                    ModuleKindV1::Module,
                    Some(ArtifactModuleId(0)),
                    vec![],
                ),
            ],
            semantic_rows: vec![],
            external_projects: vec![],
            topology_fingerprint: 0,
        };
        world.refresh_portable_topology_fingerprint().unwrap();
        (world, root, dependency)
    }

    fn empty_semantic_world_for_module_world(
        module_world: &phalcom_compiler_artifact::module_world::ModuleWorldV2,
    ) -> phalcom_compiler_artifact::semantic::world::SemanticWorldV4 {
        use phalcom_compiler_artifact::semantic::aggregates::StructuralPrimaryContractsV2;
        use phalcom_compiler_artifact::semantic::static_value::{StaticValueGraphV1, StaticValueV1};
        use phalcom_compiler_artifact::semantic::type_store::{ArtifactKindId, KindV1, TypeEntryV1, TypeGraphV1, TypeV1};
        use phalcom_compiler_artifact::semantic::world::{ModuleExecutionAuthorityV1, ModuleImportPhaseProductV1, PhaseEvidenceCoverageV1, SemanticWorldV4};

        let mut import_phase_products = module_world
            .modules
            .iter()
            .map(|module| ModuleImportPhaseProductV1 {
                module: module.identity.clone(),
                bindings: vec![],
                coverage: PhaseEvidenceCoverageV1::Full,
            })
            .collect::<Vec<_>>();
        import_phase_products.sort_unstable_by(|left, right| left.module.cmp(&right.module));
        SemanticWorldV4 {
            type_graph: TypeGraphV1 {
                types: vec![TypeEntryV1 {
                    kind: ArtifactKindId(0),
                    form: TypeV1::Unit,
                }],
                kinds: vec![KindV1::Type],
                parameters: vec![],
                current_parameters: vec![],
                rows: vec![],
                families: vec![],
                variants: vec![],
                static_domains: vec![],
                static_constructors: vec![],
                scoped: vec![],
                lambdas: vec![],
                variance_overrides: vec![],
            },
            structural_contracts: StructuralPrimaryContractsV2 {
                declaration_shells: vec![],
                intrinsic_static_domains: vec![],
                intrinsic_type_owners: vec![],
                hierarchy_edges: vec![],
                type_aliases: vec![],
                generic_signatures: vec![],
                data_contracts: vec![],
                enum_contracts: vec![],
                variant_contracts: vec![],
            },
            enum_requirements: vec![],
            static_values: StaticValueGraphV1 {
                values: vec![StaticValueV1::Unit],
                strings: vec![],
                symbols: vec![],
                tuples: vec![],
                sequences: vec![],
                records: vec![],
                data_values: vec![],
                variant_values: vec![],
                pack_backings: vec![],
                packs: vec![],
            },
            static_definitions: vec![],
            static_constants: vec![],
            module_static_bindings: vec![],
            witness_plans: vec![],
            derived_conformance_rules: vec![],
            callable_signatures: vec![],
            callable_definitions: vec![],
            callable_analyses: vec![],
            trait_surfaces: vec![],
            dispatch_surfaces: vec![],
            conditional_inherent_sets: vec![],
            conformance_heads: vec![],
            constructor_initializations: vec![],
            type_member_declarations: vec![],
            associated_static_domains: vec![],
            static_case_catalogs: vec![],
            static_case_signatures: vec![],
            associated_surfaces: vec![],
            module_execution_authority: ModuleExecutionAuthorityV1 {
                import_phase_products,
                runtime_requirements: vec![],
            },
        }
    }

    #[test]
    fn runtime_requirement_multiset_survives_native_export_v4_and_canonical_import() {
        use phalcom_compiler_artifact::limits::ValidationLimits;
        use phalcom_modules::graph::{
            RuntimeDependencyReason, RuntimeRequirementProducer, RuntimeRequirementProduct, RuntimeRequirementProvenance, RuntimeRequirementTarget,
            RuntimeRequirementTiming,
        };
        use phalcom_modules::linker::{LinkedReadSpec, RuntimeDependencyRequirement, SymbolId};

        let (module_world, importer, dependency) = runtime_requirement_module_world();
        fn make_requirement(
            importer: &ModuleId,
            dependency: &ModuleId,
            name: &str,
            reason: RuntimeDependencyReason,
            product: RuntimeRequirementProduct,
            timing: RuntimeRequirementTiming,
            with_read_target: bool,
        ) -> RuntimeDependencyRequirement {
            RuntimeDependencyRequirement {
                importer: importer.clone(),
                dependency: dependency.clone(),
                range: SourceRange { start: 12, end: 18 },
                reason,
                read_target: with_read_target.then(|| {
                    LinkedReadSpec::Binding(SymbolId {
                        module: dependency.clone(),
                        name: name.into(),
                    })
                }),
                provenance: RuntimeRequirementProvenance {
                    consumer: importer.clone(),
                    producer: RuntimeRequirementProducer::DeclarationRead,
                    target: RuntimeRequirementTarget::Binding {
                        module: dependency.clone(),
                        name: name.into(),
                    },
                    product,
                    timing,
                },
            }
        }
        let native_requirements = vec![
            make_requirement(
                &importer,
                &dependency,
                "First",
                RuntimeDependencyReason::RuntimeDeclarationReference,
                RuntimeRequirementProduct::RuntimeObject,
                RuntimeRequirementTiming::DuringModuleInitializer,
                true,
            ),
            make_requirement(
                &importer,
                &dependency,
                "Second",
                RuntimeDependencyReason::RuntimeDeclarationReference,
                RuntimeRequirementProduct::RuntimeObject,
                RuntimeRequirementTiming::DuringModuleInitializer,
                true,
            ),
            make_requirement(
                &importer,
                &dependency,
                "First",
                RuntimeDependencyReason::RuntimeDeclarationReference,
                RuntimeRequirementProduct::MetadataDescriptor,
                RuntimeRequirementTiming::DuringModuleInitializer,
                true,
            ),
            make_requirement(
                &importer,
                &dependency,
                "First",
                RuntimeDependencyReason::RuntimeDeclarationReference,
                RuntimeRequirementProduct::RuntimeObject,
                RuntimeRequirementTiming::AfterBootstrap,
                true,
            ),
            make_requirement(
                &importer,
                &dependency,
                "First",
                RuntimeDependencyReason::SelectiveValueImport,
                RuntimeRequirementProduct::RuntimeObject,
                RuntimeRequirementTiming::DuringModuleInitializer,
                false,
            ),
        ];

        let mut world = empty_semantic_world_for_module_world(&module_world);
        world.module_execution_authority.runtime_requirements =
            crate::artifact::export::export_runtime_dependency_requirements(&native_requirements).expect("export native runtime requirements");
        let encoded = world.encode().expect("encode validated SemanticWorldV4");
        let decoded = phalcom_compiler_artifact::semantic::world::SemanticWorldV4::decode(&encoded, &ValidationLimits::default())
            .expect("decode and validate SemanticWorldV4");
        let (_, imported_requirements) =
            import_module_execution_authority(&decoded.module_execution_authority, &module_world, &mut AllocationBudget::new(1 << 20))
                .expect("import canonical runtime requirements");
        let mut expected = native_requirements;
        expected.sort_unstable_by(RuntimeDependencyRequirement::canonical_cmp);
        assert_eq!(
            imported_requirements, expected,
            "the complete same-span obligation multiset must survive transport"
        );
    }
}
