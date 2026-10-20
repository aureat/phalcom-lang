//! Certified PERF003 P7 callable-binding reconstruction adapted to ARTF001.
//!
//! This is deliberately narrower than a complete CallableAnalysis import. It
//! reconstructs exact checked binding states and their source/module authority
//! so later stages can restore module-static binding facts without rereading
//! source. Expression/proof/flow authority remains a separate transplant.

use super::identity::StableOwnerCatalog;
use super::type_import::ReconstructedTypeGraph;
use crate::checker::analysis::BindingState;
use crate::checker::binding::{AssumptionBasis, BindingConsistency, BindingContract, BindingContractOrigin};
use crate::checker::causal::CausalInvalidity;
use crate::checker::flow::InitializationState;
use crate::db::QueryKey;
use crate::generic_evidence::GenericConformanceKey;
use crate::identity::{BindingId, CallableId, CallableParameterId, DiagnosticCauseId, ExplanationId, FieldId, SourceOwner, SourceSiteId, SourceSiteLocalId};
use crate::trait_dispatch::{TraitDispatchBucketKey, TraitDispatchTargetFamily};
use crate::types::evidence::{EvidenceOrigin, UnknownReason};
use crate::types::outcome::{BlockReason, BudgetKind, BudgetReport, DynamicBoundaryObligation};
use crate::types::relation::RefutationReason;
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::semantic::world::{
    CallableAnalysisRecordV2, CallableAnalysisSourceOwnerV1, CallableBindingAssumptionBasisV1, CallableBindingAuthorityV1, CallableBindingConsistencyV1,
    CallableBindingContractOriginV1, CallableBindingRowV1, CallableBlockReasonV1, CallableBudgetKindV1, CallableBudgetReportV1, CallableCausalInvalidityV1,
    CallableGeneratedBindingReasonV1, CallableInitializationStateV1, CallableQueryOwnerV1, CallableRefutationReasonV1, ConformanceTargetV1, EvidenceOriginV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportedGeneratedBindingReason {
    ImplicitReceiver,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportedCallableBindingSourceAuthority {
    BodyLocalSource(SourceSiteId),
    BodyLocalGenerated(ImportedGeneratedBindingReason),
    ModuleValue {
        symbol: phalcom_modules::linker::SymbolId,
        import: Option<phalcom_modules::linker::ImportBindingId>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedCallableBindingRow {
    pub state: BindingState,
    pub source_authority: ImportedCallableBindingSourceAuthority,
    pub formal_range: Option<SourceRange>,
}

#[derive(Clone, Debug)]
pub struct ImportedCallableBindings {
    pub callable: CallableId,
    pub rows: Arc<[ImportedCallableBindingRow]>,
}

/// Immutable callable-keyed binding authority staged for later full analysis
/// and module-static publication.
#[derive(Clone, Debug, Default)]
pub struct ImportedCallableBindingAuthority {
    callables: Box<[ImportedCallableBindings]>,
}

impl ImportedCallableBindingAuthority {
    pub fn callable_len(&self) -> usize {
        self.callables.len()
    }

    pub fn binding_len(&self) -> usize {
        self.callables.iter().map(|entry| entry.rows.len()).sum()
    }

    pub(crate) fn rows(&self, callable: &CallableId) -> Option<&Arc<[ImportedCallableBindingRow]>> {
        self.callables
            .binary_search_by(|entry| entry.callable.cmp(callable))
            .ok()
            .map(|index| &self.callables[index].rows)
    }

    pub fn binding(&self, callable: &CallableId, ordinal: u32) -> Option<&ImportedCallableBindingRow> {
        let rows = self.rows(callable)?;
        rows.get(ordinal as usize).filter(|row| row.state.binding == BindingId(ordinal))
    }
}

pub(crate) fn reconstruct_callable_bindings(
    records: &[CallableAnalysisRecordV2],
    types: &ReconstructedTypeGraph,
    static_definitions: &super::static_definition_import::ImportedStaticDefinitionAuthority,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCallableBindingAuthority, ArtifactError> {
    let mut imported = allocations
        .vector::<ImportedCallableBindings>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for record in records {
        let callable = owners.callable(&record.callable)?;
        let mut rows = allocations
            .vector::<ImportedCallableBindingRow>(record.bindings.len())
            .map_err(ArtifactError::Allocation)?;
        for (index, row) in record.bindings.iter().enumerate() {
            if usize::try_from(row.ordinal).ok() != Some(index) {
                return Err(invalid("callable binding rows are not dense and ordinal-sorted"));
            }
            rows.push(reconstruct_binding_row(row, types, static_definitions, owners, allocations)?);
        }
        imported.push(ImportedCallableBindings {
            callable,
            rows: allocations.arc_slice(rows).map_err(ArtifactError::Allocation)?,
        });
    }

    imported.sort_by(|left, right| left.callable.cmp(&right.callable));
    if imported.windows(2).any(|pair| pair[0].callable == pair[1].callable) {
        return Err(invalid("duplicate callable binding authority after stable-owner relocation"));
    }

    Ok(ImportedCallableBindingAuthority {
        callables: allocations.boxed_slice(imported).map_err(ArtifactError::Allocation)?,
    })
}

fn reconstruct_binding_row(
    row: &CallableBindingRowV1,
    types: &ReconstructedTypeGraph,
    static_definitions: &super::static_definition_import::ImportedStaticDefinitionAuthority,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCallableBindingRow, ArtifactError> {
    let range = source_range(&row.range, "callable binding")?;
    let parameter = row
        .parameter
        .as_ref()
        .map(|parameter| Ok::<_, ArtifactError>(CallableParameterId::new(owners.callable(&parameter.callable)?, parameter.index)))
        .transpose()?;

    let contract = row
        .contract
        .as_ref()
        .map(|contract| {
            let source = contract
                .source
                .as_ref()
                .map(|range| source_range(range, "callable binding contract"))
                .transpose()?;
            Ok::<_, ArtifactError>(BindingContract {
                ty: mapped_type(contract.ty.0, types, "binding contract type")?,
                origin: match contract.origin {
                    CallableBindingContractOriginV1::SourceAnnotation => BindingContractOrigin::SourceAnnotation,
                    CallableBindingContractOriginV1::InferredInitializer => BindingContractOrigin::InferredInitializer,
                    CallableBindingContractOriginV1::CallableParameter => BindingContractOrigin::CallableParameter,
                    CallableBindingContractOriginV1::ContextualBlockParameter => BindingContractOrigin::ContextualBlockParameter,
                    CallableBindingContractOriginV1::PatternBinding => BindingContractOrigin::PatternBinding,
                    CallableBindingContractOriginV1::StaticParameter => BindingContractOrigin::StaticParameter,
                },
                source,
            })
        })
        .transpose()?;

    let current = super::callable_import::import_type_knowledge(&row.current, types, allocations)?;
    let denotation = row
        .denotation
        .as_ref()
        .map(|denotation| super::denotation_import::reconstruct_semantic_denotation(denotation, types, static_definitions, owners, allocations))
        .transpose()?;
    let consistency = binding_consistency(&row.consistency, types, owners, allocations)?;
    let causal_invalidity = match row.causal_invalidity {
        CallableCausalInvalidityV1::Clean => CausalInvalidity::Clean,
        CallableCausalInvalidityV1::One { cause } => CausalInvalidity::One(DiagnosticCauseId(cause)),
        CallableCausalInvalidityV1::Multiple => CausalInvalidity::Multiple,
    };
    let (source_authority, formal_range) = binding_source_authority(&row.authority, owners, allocations)?;

    Ok(ImportedCallableBindingRow {
        state: BindingState {
            binding: BindingId(row.ordinal),
            name: allocations.string(&row.name).map_err(ArtifactError::Allocation)?,
            parameter,
            range,
            contract,
            current,
            denotation,
            consistency,
            causal_invalidity,
            mutable: row.mutable,
            initialization: match row.initialization {
                CallableInitializationStateV1::Uninitialized => InitializationState::Uninitialized,
                CallableInitializationStateV1::MaybeInitialized => InitializationState::MaybeInitialized,
                CallableInitializationStateV1::DefinitelyInitialized => InitializationState::DefinitelyInitialized,
            },
            version: row.version,
            explanation: row.explanation.map(ExplanationId),
        },
        source_authority,
        formal_range,
    })
}

fn binding_source_authority(
    authority: &CallableBindingAuthorityV1,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<(ImportedCallableBindingSourceAuthority, Option<SourceRange>), ArtifactError> {
    Ok(match authority {
        CallableBindingAuthorityV1::BodyLocalSource {
            site_owner,
            site_local,
            declaration_range,
        } => {
            let owner = match site_owner {
                CallableAnalysisSourceOwnerV1::Module { module } => SourceOwner::Module(owners.module(module)?),
                CallableAnalysisSourceOwnerV1::Callable { callable } => SourceOwner::Callable(owners.callable(callable)?),
            };
            (
                ImportedCallableBindingSourceAuthority::BodyLocalSource(SourceSiteId {
                    owner,
                    local: SourceSiteLocalId(*site_local),
                }),
                Some(source_range(declaration_range, "callable binding declaration")?),
            )
        }
        CallableBindingAuthorityV1::BodyLocalGenerated { reason } => (
            ImportedCallableBindingSourceAuthority::BodyLocalGenerated(match reason {
                CallableGeneratedBindingReasonV1::ImplicitReceiver => ImportedGeneratedBindingReason::ImplicitReceiver,
            }),
            None,
        ),
        CallableBindingAuthorityV1::ModuleValue { module, name, import_binding } => (
            ImportedCallableBindingSourceAuthority::ModuleValue {
                symbol: phalcom_modules::linker::SymbolId {
                    module: owners.module(module)?,
                    name: allocations.boxed_str(name).map_err(ArtifactError::Allocation)?,
                },
                import: import_binding.map(phalcom_modules::linker::ImportBindingId),
            },
            None,
        ),
    })
}

fn binding_consistency(
    consistency: &CallableBindingConsistencyV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<BindingConsistency, ArtifactError> {
    Ok(match consistency {
        CallableBindingConsistencyV1::Unconstrained => BindingConsistency::Unconstrained,
        CallableBindingConsistencyV1::Validated => BindingConsistency::Validated,
        CallableBindingConsistencyV1::Assumed { basis } => BindingConsistency::Assumed {
            basis: assumption_basis(basis, allocations)?,
        },
        CallableBindingConsistencyV1::Refuted { actual, expected, reason } => BindingConsistency::Refuted {
            actual: mapped_type(actual.0, types, "binding refutation actual type")?,
            expected: mapped_type(expected.0, types, "binding refutation expected type")?,
            reason: match reason {
                CallableRefutationReasonV1::IncompatibleNominal => RefutationReason::IncompatibleNominal,
                CallableRefutationReasonV1::TypeMismatch => RefutationReason::TypeMismatch,
                CallableRefutationReasonV1::UnionMemberMismatch => RefutationReason::UnionMemberMismatch,
            },
        },
        CallableBindingConsistencyV1::DynamicBoundary { reason } => BindingConsistency::DynamicBoundary {
            obligation: DynamicBoundaryObligation {
                reason: allocations.string(reason).map_err(ArtifactError::Allocation)?,
            },
        },
        CallableBindingConsistencyV1::Blocked { reason } => BindingConsistency::Blocked(block_reason(reason, types, owners, allocations)?),
        CallableBindingConsistencyV1::Cancelled => BindingConsistency::Cancelled,
        CallableBindingConsistencyV1::BudgetExceeded { report } => BindingConsistency::BudgetExceeded(budget_report(report)),
        CallableBindingConsistencyV1::InternalFailure { message } => {
            BindingConsistency::InternalFailure(allocations.boxed_str(message).map_err(ArtifactError::Allocation)?)
        }
    })
}

fn assumption_basis(basis: &CallableBindingAssumptionBasisV1, allocations: &mut AllocationBudget) -> Result<AssumptionBasis, ArtifactError> {
    Ok(match basis {
        CallableBindingAssumptionBasisV1::MissingValueEvidence { reason } => {
            AssumptionBasis::MissingValueEvidence(super::callable_import::import_unknown_reason(reason, allocations)?)
        }
        CallableBindingAssumptionBasisV1::CallableParameterContract => AssumptionBasis::CallableParameterContract,
        CallableBindingAssumptionBasisV1::ContextualParameterContract => AssumptionBasis::ContextualParameterContract,
        CallableBindingAssumptionBasisV1::DerivedEvidence { origin } => AssumptionBasis::DerivedEvidence(evidence_origin(origin)),
    })
}

pub(crate) fn block_reason(
    reason: &CallableBlockReasonV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<BlockReason, ArtifactError> {
    Ok(match reason {
        CallableBlockReasonV1::UnknownType { reason } => BlockReason::UnknownType(super::callable_import::import_unknown_reason(reason, allocations)?),
        CallableBlockReasonV1::UnresolvedDependency { .. } => {
            // Current StableProjectKey owns a PathBuf. The allocation ledger
            // has no fallible/budgeted PathBuf constructor, so recreating P7's
            // direct PathBuf::from would violate exact whole-load accounting.
            return Err(ArtifactError::Allocation(AllocationError::UnaccountedBoundary));
        }
        CallableBlockReasonV1::InvalidAnnotation { diagnostic_code } => BlockReason::InvalidAnnotation(
            crate::diagnostic::DiagnosticCode::from_stable_code(diagnostic_code)
                .ok_or_else(|| invalid("callable blocked reason has an unknown diagnostic code"))?,
        ),
        CallableBlockReasonV1::RecursiveFixpoint => BlockReason::RecursiveFixpoint,
        CallableBlockReasonV1::InferenceDidNotConverge => BlockReason::InferenceDidNotConverge,
        CallableBlockReasonV1::OpaqueNative { message } => BlockReason::OpaqueNative(allocations.boxed_str(message).map_err(ArtifactError::Allocation)?),
        CallableBlockReasonV1::ReflectionBoundary => BlockReason::ReflectionBoundary,
        CallableBlockReasonV1::UnavailableSemanticOwner { owner } => {
            let key = callable_query_owner(owner, types, owners, allocations)?;
            BlockReason::UnavailableSemanticOwner(allocations.boxed(key).map_err(ArtifactError::Allocation)?)
        }
        CallableBlockReasonV1::BudgetExceeded { report } => BlockReason::BudgetExceeded(budget_report(report)),
        CallableBlockReasonV1::SuppressedDependency => BlockReason::SuppressedDependency,
    })
}

pub(crate) fn callable_query_owner(
    owner: &CallableQueryOwnerV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<QueryKey, ArtifactError> {
    Ok(match owner {
        CallableQueryOwnerV1::CallableSignature { callable } => QueryKey::CallableSignature(owners.callable(callable)?),
        CallableQueryOwnerV1::FieldSignature { field } => QueryKey::FieldSignature(FieldId::new(
            owners.declaration(&field.owner)?,
            allocations.boxed_str(&field.name).map_err(ArtifactError::Allocation)?,
            super::identity::dispatch_side(field.side),
        )),
        CallableQueryOwnerV1::GenericConformance { subject, trait_ref } => {
            let mut arguments = allocations
                .vector::<crate::types::id::TypeId>(trait_ref.arguments.len())
                .map_err(ArtifactError::Allocation)?;
            for argument in &trait_ref.arguments {
                arguments.push(mapped_type(argument.0, types, "generic conformance trait argument")?);
            }
            QueryKey::GenericConformance(GenericConformanceKey::new(
                super::semantic_import::reconstruct_term(subject, types, owners)?,
                crate::traits::TraitRef::new(
                    owners.declaration(&trait_ref.declaration)?,
                    allocations.boxed_slice(arguments).map_err(ArtifactError::Allocation)?,
                ),
            ))
        }
        CallableQueryOwnerV1::TraitDispatchBucket { target_family, selector, side } => {
            let target_family = match target_family {
                ConformanceTargetV1::Declaration { declaration } => TraitDispatchTargetFamily::Declaration(owners.declaration(declaration)?),
                ConformanceTargetV1::ExactEnumCase { variant } => TraitDispatchTargetFamily::ExactEnumCase(owners.variant(variant)?),
            };
            QueryKey::TraitDispatchBucket(TraitDispatchBucketKey {
                target_family,
                selector: super::denotation_import::exact_selector(selector, allocations)?,
                side: super::identity::dispatch_side(*side),
            })
        }
    })
}

fn evidence_origin(origin: &EvidenceOriginV1) -> EvidenceOrigin {
    match origin {
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
    }
}

pub(crate) fn budget_report(report: &CallableBudgetReportV1) -> BudgetReport {
    BudgetReport::new(
        match report.kind {
            CallableBudgetKindV1::Steps => BudgetKind::Steps,
            CallableBudgetKindV1::RelationPairs => BudgetKind::RelationPairs,
            CallableBudgetKindV1::SccIterations => BudgetKind::SccIterations,
            CallableBudgetKindV1::TypeDepth => BudgetKind::TypeDepth,
            CallableBudgetKindV1::DiagnosticNotes => BudgetKind::DiagnosticNotes,
            CallableBudgetKindV1::ResidualNodes => BudgetKind::ResidualNodes,
        },
        report.limit,
        report.used,
    )
}

fn source_range(range: &phalcom_compiler_artifact::module_world::SpanV1, family: &'static str) -> Result<SourceRange, ArtifactError> {
    if range.start > range.end {
        return Err(invalid(format!("{family} range is inverted")));
    }
    Ok(SourceRange {
        start: usize::try_from(range.start).map_err(|_| invalid(format!("{family} start exceeds address space")))?,
        end: usize::try_from(range.end).map_err(|_| invalid(format!("{family} end exceeds address space")))?,
    })
}

fn mapped_type(raw: u32, types: &ReconstructedTypeGraph, family: &'static str) -> Result<crate::types::id::TypeId, ArtifactError> {
    types
        .types
        .get(raw as usize)
        .copied()
        .ok_or_else(|| invalid(format!("{family} references an absent relocated type row")))
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
