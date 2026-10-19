//! Certified PERF003 P7 callable-local conformance evidence authority.
//!
//! Callable analyses carry their own ordinal conformance-evidence graph. The
//! graph is reconstructed only after conformance heads, trait authority, and
//! source witness plans are staged, matching P7's proof dependency order.
//! Expression trait-dispatch sidecars consume this authority by ordinal.

use super::callable_import::ImportedCallableAuthority;
use super::conformance_evidence_import::ImportedConformanceEvidenceGraph;
use super::conformance_import::ImportedConformanceAuthority;
use super::identity::StableOwnerCatalog;
use super::trait_import::ImportedTraitAuthority;
use super::type_import::ReconstructedTypeGraph;
use super::witness_import::ImportedWitnessPreflightAuthority;
use crate::identity::{BodyId, CallableId};
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::semantic::world::CallableAnalysisRecordV1;
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;

/// One callable body's fully relocated P7 conformance-evidence graph.
#[derive(Clone, Debug)]
pub struct ImportedCallableConformanceEvidence {
    pub callable: CallableId,
    pub body: BodyId,
    pub graph: ImportedConformanceEvidenceGraph,
}

/// Sorted callable-keyed proof authority. This is a staging graph, not a
/// second semantic database and not by itself a Ready CallableBody product.
#[derive(Clone, Debug, Default)]
pub struct ImportedCallableConformanceEvidenceAuthority {
    owners: Box<[ImportedCallableConformanceEvidence]>,
}

impl ImportedCallableConformanceEvidenceAuthority {
    pub fn callable_len(&self) -> usize {
        self.owners.len()
    }

    pub fn evidence_node_len(&self) -> usize {
        self.owners.iter().map(|owner| owner.graph.len()).sum()
    }

    pub fn graph(&self, callable: &CallableId) -> Option<&ImportedConformanceEvidenceGraph> {
        self.owners
            .binary_search_by(|owner| owner.callable.cmp(callable))
            .ok()
            .map(|index| &self.owners[index].graph)
    }

    pub(crate) fn owner(&self, callable: &CallableId) -> Option<&ImportedCallableConformanceEvidence> {
        self.owners
            .binary_search_by(|owner| owner.callable.cmp(callable))
            .ok()
            .map(|index| &self.owners[index])
    }
}

/// Reconstructs every callable-local evidence graph using the same recursive
/// P7 importer already shared by witness plans.
pub(crate) fn reconstruct_callable_conformance_evidence(
    records: &[CallableAnalysisRecordV1],
    conformances: &ImportedConformanceAuthority,
    witnesses: &ImportedWitnessPreflightAuthority,
    traits: &ImportedTraitAuthority,
    callables: &ImportedCallableAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCallableConformanceEvidenceAuthority, ArtifactError> {
    let mut imported = allocations
        .vector::<ImportedCallableConformanceEvidence>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for (body_index, record) in records.iter().enumerate() {
        let callable = owners.callable(&record.callable)?;
        if callables.signature(&callable).is_none() {
            return Err(invalid(
                "callable analysis evidence owner has no imported canonical signature",
            ));
        }
        let body = BodyId(
            u32::try_from(body_index)
                .map_err(|_| invalid("callable body ordinal exceeds portable range"))?,
        );
        let graph =
            super::conformance_evidence_import::reconstruct_conformance_evidence_graph(
                &record.conformance_evidence,
                conformances,
                witnesses.plans(),
                traits,
                callables,
                types,
                owners,
                limits,
                allocations,
            )?;
        imported.push(ImportedCallableConformanceEvidence {
            callable,
            body,
            graph,
        });
    }

    imported.sort_by(|left, right| left.callable.cmp(&right.callable));
    if imported
        .windows(2)
        .any(|pair| pair[0].callable == pair[1].callable)
    {
        return Err(invalid(
            "duplicate callable conformance-evidence owner after stable-owner relocation",
        ));
    }

    Ok(ImportedCallableConformanceEvidenceAuthority {
        owners: allocations
            .boxed_slice(imported)
            .map_err(ArtifactError::Allocation)?,
    })
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
