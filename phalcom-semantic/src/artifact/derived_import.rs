//! Certified PERF003 P7 top-level derived-conformance reconstruction adapted to ARTF001.
//!
//! A derived rule is staged together with the relocated proof graph that its
//! requirement selections reference. Native BTreeMap/Arc proof structures are
//! still deferred to T4 so P2 remains allocation-accounted and atomic.

use super::callable_import::ImportedCallableAuthority;
use super::conformance_evidence_import::{
    ImportedConformanceEvidenceGraph, ImportedDerivedConformanceRule,
    reconstruct_conformance_evidence_graph, reconstruct_derived_rule,
};
use super::conformance_import::ImportedConformanceAuthority;
use super::identity::StableOwnerCatalog;
use super::trait_import::ImportedTraitAuthority;
use super::type_import::ReconstructedTypeGraph;
use super::witness_import::ImportedWitnessPlanPreflight;
use crate::identity::ImplId;
use phalcom_compiler_artifact::semantic::world::DerivedConformanceRuleRecordV1;
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::AllocationBudget;

#[derive(Clone, Debug)]
pub struct ImportedDerivedConformanceRuleRecord {
    pub rule: ImportedDerivedConformanceRule,
    pub conformance_evidence: ImportedConformanceEvidenceGraph,
}

/// Immutable sorted derived-rule authority staged for the imported generation.
#[derive(Clone, Debug, Default)]
pub struct ImportedDerivedConformanceAuthority {
    records: Box<[ImportedDerivedConformanceRuleRecord]>,
}

impl ImportedDerivedConformanceAuthority {
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn evidence_node_len(&self) -> usize {
        self.records
            .iter()
            .map(|record| record.conformance_evidence.len())
            .sum()
    }

    pub(crate) fn get(&self, implementation: &ImplId) -> Option<&ImportedDerivedConformanceRuleRecord> {
        self.records
            .binary_search_by(|record| record.rule.impl_id.cmp(implementation))
            .ok()
            .map(|index| &self.records[index])
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &ImportedDerivedConformanceRuleRecord> {
        self.records.iter()
    }
}

pub(crate) fn reconstruct_derived_conformance_authority(
    records: &[DerivedConformanceRuleRecordV1],
    source_witnesses: &[ImportedWitnessPlanPreflight],
    conformances: &ImportedConformanceAuthority,
    traits: &ImportedTraitAuthority,
    callables: &ImportedCallableAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ImportedDerivedConformanceAuthority, ArtifactError> {
    let mut imported = allocations
        .vector::<ImportedDerivedConformanceRuleRecord>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for record in records {
        let rule = reconstruct_derived_rule(
            &record.rule,
            record.conformance_evidence.len(),
            traits,
            callables,
            types,
            owners,
            limits,
            allocations,
        )?;
        let conformance_evidence = reconstruct_conformance_evidence_graph(
            &record.conformance_evidence,
            conformances,
            source_witnesses,
            traits,
            callables,
            types,
            owners,
            limits,
            allocations,
        )?;
        imported.push(ImportedDerivedConformanceRuleRecord {
            rule,
            conformance_evidence,
        });
    }

    imported.sort_by(|left, right| left.rule.impl_id.cmp(&right.rule.impl_id));
    if imported
        .windows(2)
        .any(|pair| pair[0].rule.impl_id == pair[1].rule.impl_id)
    {
        return Err(invalid(
            "derived conformance rule is duplicated after stable-owner relocation",
        ));
    }

    Ok(ImportedDerivedConformanceAuthority {
        records: allocations.boxed_slice(imported).map_err(ArtifactError::Allocation)?,
    })
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
