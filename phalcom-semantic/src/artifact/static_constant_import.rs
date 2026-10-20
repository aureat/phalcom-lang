//! Certified PERF003 P7 static-constant authority adapted to ARTF001.
//!
//! Current-main's `StaticConstantProduct` separates source-checked bodies from
//! externally imported bundle origins, while P7 rows carry the already-checked
//! source authority directly. This module preserves that complete typed
//! authority without misclassifying it as the newer imported-origin envelope.

use super::identity::StableOwnerCatalog;
use super::static_import::ImportedStaticValueGraph;
use super::type_import::ReconstructedTypeGraph;
use crate::checker::analysis::SemanticDependency;
use crate::db::InputFingerprint;
use crate::diagnostic::SemanticDiagnostic;
use crate::static_constant::StaticConstantId;
use crate::static_value::StaticValueRef;
use crate::types::id::TypeId;
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::semantic::world::{StaticConstantOutcomeV1, StaticConstantRecordV1, StaticConstantSymbolV1};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_modules::linker::SymbolId;
use phalcom_type_meta::allocation::AllocationBudget;
use std::sync::Arc;

/// Exact checked P7 authority for one static constant.
#[derive(Clone, Debug)]
pub struct ImportedStaticConstant {
    pub id: StaticConstantId,
    pub declaration_range: SourceRange,
    pub initializer_range: SourceRange,
    pub input_fingerprint: InputFingerprint,
    pub dependencies: Arc<[SemanticDependency]>,
    pub static_constant_reads: Arc<[SymbolId]>,
    pub record_key_static_constant_reads: Arc<[SymbolId]>,
    pub result_type: Option<TypeId>,
    pub value: StaticValueRef,
    pub lkg_value: Option<StaticValueRef>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}

/// Sorted immutable static-constant authority staged for T4.
#[derive(Clone, Debug, Default)]
pub struct ImportedStaticConstantAuthority {
    constants: Box<[ImportedStaticConstant]>,
}

impl ImportedStaticConstantAuthority {
    pub fn len(&self) -> usize {
        self.constants.len()
    }

    pub fn is_empty(&self) -> bool {
        self.constants.is_empty()
    }

    pub(crate) fn get(&self, id: &StaticConstantId) -> Option<&ImportedStaticConstant> {
        self.constants
            .binary_search_by(|candidate| candidate.id.cmp(id))
            .ok()
            .map(|index| &self.constants[index])
    }
}

/// Port of P7 `import_static_constant_owners` against the already-relocated
/// current-main type/static-value graphs.
pub(crate) fn reconstruct_static_constants(
    records: &[StaticConstantRecordV1],
    static_values: &ImportedStaticValueGraph,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedStaticConstantAuthority, ArtifactError> {
    let mut constants = allocations.vector::<ImportedStaticConstant>(records.len()).map_err(ArtifactError::Allocation)?;

    for record in records {
        let id = StaticConstantId(static_symbol(&record.symbol, owners, allocations)?);
        let value = match &record.outcome {
            StaticConstantOutcomeV1::Reduced { value } => static_values
                .value(*value)
                .ok_or_else(|| invalid("static constant outcome references an absent static value"))?,
        };

        let mut dependencies = allocations
            .vector::<SemanticDependency>(record.dependencies.len())
            .map_err(ArtifactError::Allocation)?;
        for dependency in &record.dependencies {
            dependencies.push(super::static_dependency_import::reconstruct_static_dependency(
                dependency,
                types,
                owners,
                allocations,
            )?);
        }

        let mut static_constant_reads = allocations
            .vector::<SymbolId>(record.static_constant_reads.len())
            .map_err(ArtifactError::Allocation)?;
        for read in &record.static_constant_reads {
            static_constant_reads.push(static_symbol(read, owners, allocations)?);
        }

        let mut record_key_reads = allocations
            .vector::<SymbolId>(record.record_key_static_constant_reads.len())
            .map_err(ArtifactError::Allocation)?;
        for read in &record.record_key_static_constant_reads {
            record_key_reads.push(static_symbol(read, owners, allocations)?);
        }

        let result_type = record
            .result_type
            .map(|ty| {
                types
                    .types
                    .get(ty.0 as usize)
                    .copied()
                    .ok_or_else(|| invalid("static constant result type references an absent type"))
            })
            .transpose()?;
        let lkg_value = record
            .lkg_value
            .map(|value| {
                static_values
                    .value(value)
                    .ok_or_else(|| invalid("static constant LKG value references an absent static value"))
            })
            .transpose()?;

        let mut diagnostics = allocations
            .vector::<SemanticDiagnostic>(record.diagnostics.len())
            .map_err(ArtifactError::Allocation)?;
        for diagnostic in &record.diagnostics {
            diagnostics.push(super::diagnostic_import::reconstruct_semantic_diagnostic(
                diagnostic,
                types,
                owners,
                allocations,
            )?);
        }

        constants.push(ImportedStaticConstant {
            id,
            declaration_range: super::diagnostic_import::range(&record.declaration_source, "static constant declaration")?,
            initializer_range: super::diagnostic_import::range(&record.initializer_source, "static constant initializer")?,
            input_fingerprint: InputFingerprint::new(record.input_fingerprint),
            dependencies: allocations.arc_slice(dependencies).map_err(ArtifactError::Allocation)?,
            static_constant_reads: allocations.arc_slice(static_constant_reads).map_err(ArtifactError::Allocation)?,
            record_key_static_constant_reads: allocations.arc_slice(record_key_reads).map_err(ArtifactError::Allocation)?,
            result_type,
            value,
            lkg_value,
            diagnostics: allocations.arc_slice(diagnostics).map_err(ArtifactError::Allocation)?,
        });
    }

    constants.sort_by(|left, right| left.id.cmp(&right.id));
    if constants.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(invalid("static constant owner is duplicated after stable-owner relocation"));
    }

    Ok(ImportedStaticConstantAuthority {
        constants: allocations.boxed_slice(constants).map_err(ArtifactError::Allocation)?,
    })
}

fn static_symbol(symbol: &StaticConstantSymbolV1, owners: &StableOwnerCatalog<'_>, allocations: &mut AllocationBudget) -> Result<SymbolId, ArtifactError> {
    Ok(SymbolId {
        module: owners.module(&symbol.module)?,
        name: allocations.boxed_str(&symbol.name).map_err(ArtifactError::Allocation)?,
    })
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
