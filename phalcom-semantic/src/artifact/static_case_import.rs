//! Certified P7 static-case semantic authority adapted to current-main owners.
//!
//! These products are reconstructed for the imported semantic generation even
//! when their current query input fingerprints remain source-backed. T4 may
//! publish them into the current-main snapshot tables without pretending that
//! they are independently reusable SemanticDb roots.

use super::identity::StableOwnerCatalog;
use super::type_import::ReconstructedTypeGraph;
use crate::identity::{AssociatedStaticDomainDeclarationId, StaticCaseId, StaticDomainDeclarationId};
use crate::static_case::{StaticCaseHeader, StaticCaseShape, StaticCaseSignatureProduct, StaticDomainCaseCatalogProduct};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorBase, SelectorKind, SelectorSlot};
use phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1;
use phalcom_compiler_artifact::semantic::world::{
    ExactSelectorV1, SelectorKindV1, SelectorSlotV1, StaticCaseDomainV1, StaticCaseShapeV1, StaticCaseSignatureRecordV1, StaticDomainCaseCatalogRecordV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct ImportedStaticCaseAuthority {
    catalogs: Box<[(StaticDomainDeclarationId, Arc<StaticDomainCaseCatalogProduct>)]>,
    signatures: Box<[(StaticCaseId, Arc<StaticCaseSignatureProduct>)]>,
}

impl ImportedStaticCaseAuthority {
    pub fn catalog_len(&self) -> usize {
        self.catalogs.len()
    }

    pub fn signature_len(&self) -> usize {
        self.signatures.len()
    }

    pub(crate) fn catalog(&self, id: &StaticDomainDeclarationId) -> Option<&Arc<StaticDomainCaseCatalogProduct>> {
        self.catalogs
            .binary_search_by(|(candidate, _)| candidate.cmp(id))
            .ok()
            .map(|index| &self.catalogs[index].1)
    }

    pub(crate) fn signature(&self, id: &StaticCaseId) -> Option<&Arc<StaticCaseSignatureProduct>> {
        self.signatures
            .binary_search_by(|(candidate, _)| candidate.cmp(id))
            .ok()
            .map(|index| &self.signatures[index].1)
    }
}

/// Port of P7 `import_static_case_owners` using current project-qualified
/// relocation and the already reconstructed current-main type graph.
pub(crate) fn reconstruct_static_case_authority(
    catalogs: &[StaticDomainCaseCatalogRecordV1],
    signatures: &[StaticCaseSignatureRecordV1],
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedStaticCaseAuthority, ArtifactError> {
    let mut imported_catalogs = allocations
        .vector::<(StaticDomainDeclarationId, Arc<StaticDomainCaseCatalogProduct>)>(catalogs.len())
        .map_err(ArtifactError::Allocation)?;
    for record in catalogs {
        let domain = static_case_domain(&record.domain, owners, allocations)?;
        let mut cases = allocations.vector::<StaticCaseHeader>(record.cases.len()).map_err(ArtifactError::Allocation)?;
        for record_case in &record.cases {
            let selector = exact_selector(&record_case.selector, allocations)?;
            let shape = match &record_case.shape {
                StaticCaseShapeV1::Singleton => StaticCaseShape::Singleton,
                StaticCaseShapeV1::Constructor { slots } => {
                    let mut imported_slots = allocations.vector::<SelectorSlot>(slots.len()).map_err(ArtifactError::Allocation)?;
                    for slot in slots {
                        imported_slots.push(selector_slot(slot, allocations)?);
                    }
                    StaticCaseShape::Constructor {
                        slots: allocations.boxed_slice(imported_slots).map_err(ArtifactError::Allocation)?,
                    }
                }
            };
            cases.push(StaticCaseHeader {
                id: StaticCaseId::with_selector(domain.clone(), selector),
                shape,
                source_range: source_range(record_case.source.start, record_case.source.end)?,
            });
        }
        let cases = allocations.boxed_slice(cases).map_err(ArtifactError::Allocation)?;
        let product = allocations
            .arc(StaticDomainCaseCatalogProduct { domain: domain.clone(), cases })
            .map_err(ArtifactError::Allocation)?;
        imported_catalogs.push((domain, product));
    }
    imported_catalogs.sort_by(|left, right| left.0.cmp(&right.0));
    reject_duplicate_keys(&imported_catalogs, "duplicate static-case catalog after stable-owner relocation")?;

    let mut imported_signatures = allocations
        .vector::<(StaticCaseId, Arc<StaticCaseSignatureProduct>)>(signatures.len())
        .map_err(ArtifactError::Allocation)?;
    for record in signatures {
        let domain = static_case_domain(&record.domain, owners, allocations)?;
        let selector = exact_selector(&record.selector, allocations)?;
        let id = StaticCaseId::with_selector(domain, selector);
        let mut parameter_kinds = allocations
            .vector::<crate::types::id::KindId>(record.parameter_kinds.len())
            .map_err(ArtifactError::Allocation)?;
        for kind in &record.parameter_kinds {
            parameter_kinds.push(
                *types
                    .kinds
                    .get(kind.0 as usize)
                    .ok_or_else(|| invalid("static-case signature references an absent kind row"))?,
            );
        }
        let parameter_kinds = allocations.boxed_slice(parameter_kinds).map_err(ArtifactError::Allocation)?;
        let product = allocations
            .arc(StaticCaseSignatureProduct {
                id: id.clone(),
                parameter_kinds,
                source_range: source_range(record.source.start, record.source.end)?,
            })
            .map_err(ArtifactError::Allocation)?;
        imported_signatures.push((id, product));
    }
    imported_signatures.sort_by(|left, right| left.0.cmp(&right.0));
    reject_duplicate_keys(&imported_signatures, "duplicate static-case signature after stable-owner relocation")?;

    Ok(ImportedStaticCaseAuthority {
        catalogs: allocations.boxed_slice(imported_catalogs).map_err(ArtifactError::Allocation)?,
        signatures: allocations.boxed_slice(imported_signatures).map_err(ArtifactError::Allocation)?,
    })
}

fn static_case_domain(
    domain: &StaticCaseDomainV1,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<StaticDomainDeclarationId, ArtifactError> {
    Ok(match domain {
        StaticCaseDomainV1::Nominal { declaration } => StaticDomainDeclarationId::Nominal(owners.declaration(declaration)?),
        StaticCaseDomainV1::Associated { declaration } => StaticDomainDeclarationId::Associated(AssociatedStaticDomainDeclarationId::new(
            owners.declaration(&declaration.owner)?,
            allocations.boxed_str(&declaration.name).map_err(ArtifactError::Allocation)?,
        )),
    })
}

fn exact_selector(selector: &ExactSelectorV1, allocations: &mut AllocationBudget) -> Result<Selector, ArtifactError> {
    let base = selector_base(&selector.base, allocations)?;
    let kind = selector_kind(&selector.kind);
    let mut slots = allocations.vector::<SelectorSlot>(selector.slots.len()).map_err(ArtifactError::Allocation)?;
    for slot in &selector.slots {
        slots.push(selector_slot(slot, allocations)?);
    }
    Selector::new(base, kind, allocations.boxed_slice(slots).map_err(ArtifactError::Allocation)?).map_err(|_| invalid("static-case selector is not canonical"))
}

fn selector_base(base: &TypeMemberSelectorV1, allocations: &mut AllocationBudget) -> Result<SelectorBase, ArtifactError> {
    Ok(match base {
        TypeMemberSelectorV1::Named { name } => SelectorBase::Named(allocations.string(name).map_err(ArtifactError::Allocation)?),
        TypeMemberSelectorV1::Subscript => SelectorBase::Subscript,
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

fn selector_slot(slot: &SelectorSlotV1, allocations: &mut AllocationBudget) -> Result<SelectorSlot, ArtifactError> {
    Ok(match slot {
        SelectorSlotV1::Positional => SelectorSlot::Positional,
        SelectorSlotV1::Label { label } => SelectorSlot::Label(allocations.string(label).map_err(ArtifactError::Allocation)?),
    })
}

fn source_range(start: u64, end: u64) -> Result<SourceRange, ArtifactError> {
    Ok(SourceRange {
        start: usize::try_from(start).map_err(|_| invalid("static-case source start exceeds address space"))?,
        end: usize::try_from(end).map_err(|_| invalid("static-case source end exceeds address space"))?,
    })
}

fn reject_duplicate_keys<K: Ord, V>(rows: &[(K, V)], message: &'static str) -> Result<(), ArtifactError> {
    if rows.windows(2).any(|pair| &pair[0].0 == &pair[1].0) {
        Err(invalid(message))
    } else {
        Ok(())
    }
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
