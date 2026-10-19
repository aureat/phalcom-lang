//! Certified PERF003 P7 callable-signature reconstruction adapted to ARTF001.
//!
//! This module restores callable semantic authority into the imported canonical
//! generation. It deliberately does not mark CallableSignature queries Ready:
//! current-main source queries still depend on declaration shells, linked
//! interfaces, source syntax/effective definitions, and captured semantic
//! dependencies. Query-image promotion belongs after that closure is imported.

use super::identity::StableOwnerCatalog;
use super::type_import::ReconstructedTypeGraph;
use crate::declaration_type::DeclaredTypeFact;
use crate::diagnostic::SemanticSourceSpan;
use crate::identity::{CallableId, CallableParameterId, DispatchSide};
use crate::impls::{
    CallableDefinitionBodyProvenance, CallableDefinitionOrigin, CoveringImplSubstitution, EffectiveCallableDefinition, InherentBodyContext,
};
use crate::signature::{
    CallableParameterSemantic, CallableSemanticSignature, ParameterContract, RestValueContract, ReturnContractValidation,
};
use crate::types::evidence::{
    DynamicReason, EvidenceOrigin, EvidenceSet, EvidenceStatus, TypeKnowledge, UnknownReason,
};
use phalcom_ast::ast::RestMode;
use phalcom_common::modifier::ImplementationKind;
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::semantic::aggregates::{
    DeclaredTypeFactV1, DynamicReasonV1, UnknownReasonV1,
};
use phalcom_compiler_artifact::semantic::world::{
    CallableDefinitionBodyProvenanceV1, CallableDefinitionOriginV1, CallableDefinitionRecordV1, CallableSignatureRecordV1, EffectSpecV1,
    EvidenceOriginV1, EvidenceStatusV1, ImplementationKindV1, ParameterContractV1,
    RaisesSpecV1, RestValueContractV1, ReturnContractValidationV1, ReturnFlowV1, TypeExprSpecV1, TypeKnowledgeV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct ImportedCallableAuthority {
    signatures: Box<[(CallableId, Arc<CallableSemanticSignature>)]>,
    definitions: Box<[(CallableId, Arc<EffectiveCallableDefinition>)]>,
}

impl ImportedCallableAuthority {
    pub fn signature_len(&self) -> usize {
        self.signatures.len()
    }

    pub fn definition_len(&self) -> usize {
        self.definitions.len()
    }

    pub(crate) fn signature(&self, id: &CallableId) -> Option<&Arc<CallableSemanticSignature>> {
        self.signatures
            .binary_search_by(|(candidate, _)| candidate.cmp(id))
            .ok()
            .map(|index| &self.signatures[index].1)
    }

    pub(crate) fn definition(&self, id: &CallableId) -> Option<&Arc<EffectiveCallableDefinition>> {
        self.definitions
            .binary_search_by(|(candidate, _)| candidate.cmp(id))
            .ok()
            .map(|index| &self.definitions[index].1)
    }
}

/// Transplants P7 callable-signature reconstruction while resolving every
/// stable owner through the current exact project-qualified owner catalog.
pub(crate) fn reconstruct_callable_authority(
    rows: &[CallableSignatureRecordV1],
    definition_rows: &[CallableDefinitionRecordV1],
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCallableAuthority, ArtifactError> {
    let mut signatures = allocations
        .vector::<(CallableId, Arc<CallableSemanticSignature>)>(rows.len())
        .map_err(ArtifactError::Allocation)?;
    for row in rows {
        let signature = reconstruct_callable_signature(row, types, owners, allocations)?;
        signatures.push((signature.callable.clone(), allocations.arc(signature).map_err(ArtifactError::Allocation)?));
    }
    signatures.sort_by(|left, right| left.0.cmp(&right.0));
    if signatures.windows(2).any(|pair| &pair[0].0 == &pair[1].0) {
        return Err(referential("duplicate callable signature after stable-owner relocation"));
    }
    let mut definitions = allocations
        .vector::<(CallableId, Arc<EffectiveCallableDefinition>)>(definition_rows.len())
        .map_err(ArtifactError::Allocation)?;
    for row in definition_rows {
        let definition = reconstruct_callable_definition(row, types, owners, allocations)?;
        let Some((_, canonical_signature)) = signatures
            .binary_search_by(|(candidate, _)| candidate.cmp(&definition.callable))
            .ok()
            .and_then(|index| signatures.get(index))
        else {
            return Err(referential("callable definition has no canonical callable-signature authority"));
        };
        if canonical_signature.as_ref() != &definition.signature {
            return Err(referential("callable definition signature disagrees with canonical callable-signature authority"));
        }
        definitions.push((
            definition.callable.clone(),
            allocations.arc(definition).map_err(ArtifactError::Allocation)?,
        ));
    }
    definitions.sort_by(|left, right| left.0.cmp(&right.0));
    if definitions.windows(2).any(|pair| &pair[0].0 == &pair[1].0) {
        return Err(referential("duplicate callable definition after stable-owner relocation"));
    }

    Ok(ImportedCallableAuthority {
        signatures: allocations.boxed_slice(signatures).map_err(ArtifactError::Allocation)?,
        definitions: allocations.boxed_slice(definitions).map_err(ArtifactError::Allocation)?,
    })
}

fn reconstruct_callable_definition(
    record: &CallableDefinitionRecordV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<EffectiveCallableDefinition, ArtifactError> {
    let callable = owners.callable(&record.callable)?;
    let origin = match &record.origin {
        CallableDefinitionOriginV1::PrimaryDeclaration => CallableDefinitionOrigin::PrimaryDeclaration,
        CallableDefinitionOriginV1::InherentImpl { impl_id } => {
            CallableDefinitionOrigin::InherentImpl(owners.implementation(impl_id)?)
        }
        CallableDefinitionOriginV1::ConformanceWitness { impl_id } => {
            CallableDefinitionOrigin::ConformanceWitness(owners.implementation(impl_id)?)
        }
    };
    let signature = reconstruct_callable_signature(&record.signature, types, owners, allocations)?;
    if signature.callable != callable {
        return Err(referential("callable definition row key disagrees with its signature"));
    }
    let body_provenance = match &record.body_provenance {
        CallableDefinitionBodyProvenanceV1::SourceBody => CallableDefinitionBodyProvenance::SourceBody,
        CallableDefinitionBodyProvenanceV1::GeneratedDelegation { target_field } => {
            CallableDefinitionBodyProvenance::GeneratedDelegation {
                target_field: allocations.boxed_str(target_field).map_err(ArtifactError::Allocation)?,
            }
        }
    };
    let inherent_body_context = record
        .inherent_body_context
        .as_ref()
        .map(|context| -> Result<_, ArtifactError> {
            let generic_signature = context
                .generic_signature
                .as_ref()
                .map(|signature| super::semantic_import::reconstruct_signature(signature, types, owners, allocations))
                .transpose()?;
            let covering = context
                .covering
                .as_ref()
                .map(|covering| -> Result<_, ArtifactError> {
                    let mut impl_to_decl = allocations
                        .vector(covering.impl_to_decl.len())
                        .map_err(ArtifactError::Allocation)?;
                    for binding in &covering.impl_to_decl {
                        let implementation = types
                            .parameters
                            .get(binding.impl_parameter.0 as usize)
                            .copied()
                            .ok_or_else(|| referential("covering impl substitution references an absent implementation parameter"))?;
                        let declaration = types
                            .parameters
                            .get(binding.declaration_parameter.0 as usize)
                            .copied()
                            .ok_or_else(|| referential("covering impl substitution references an absent declaration parameter"))?;
                        impl_to_decl.push((implementation, declaration));
                    }
                    let mut decl_to_impl = allocations
                        .vector(covering.decl_to_impl.len())
                        .map_err(ArtifactError::Allocation)?;
                    for binding in &covering.decl_to_impl {
                        let implementation = types
                            .parameters
                            .get(binding.impl_parameter.0 as usize)
                            .copied()
                            .ok_or_else(|| referential("covering impl substitution references an absent implementation parameter"))?;
                        let declaration = types
                            .parameters
                            .get(binding.declaration_parameter.0 as usize)
                            .copied()
                            .ok_or_else(|| referential("covering impl substitution references an absent declaration parameter"))?;
                        decl_to_impl.push((declaration, implementation));
                    }
                    Ok(CoveringImplSubstitution {
                        impl_to_decl: crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(
                            impl_to_decl,
                            allocations,
                        )
                        .map_err(frozen_ordered_map_error)?,
                        decl_to_impl: crate::ordered_map::OrderedMap::from_frozen_entries_with_allocation_budget(
                            decl_to_impl,
                            allocations,
                        )
                        .map_err(frozen_ordered_map_error)?,
                    })
                })
                .transpose()?;
            let source_target_head = mapped_type(context.source_target_head, types)?;
            Ok(InherentBodyContext {
                generic_signature,
                covering,
                source_target_head,
            })
        })
        .transpose()?;
    let source_member_index = usize::try_from(record.source_member_index)
        .map_err(|_| referential("callable source member index exceeds address space"))?;
    Ok(EffectiveCallableDefinition {
        callable,
        origin,
        source_member_index,
        signature,
        body_provenance,
        inherent_body_context,
    })
}

fn frozen_ordered_map_error(error: crate::ordered_map::FrozenOrderedMapError) -> ArtifactError {
    match error {
        crate::ordered_map::FrozenOrderedMapError::Allocation(error) => ArtifactError::Allocation(error),
        crate::ordered_map::FrozenOrderedMapError::DuplicateKey => {
            referential("covering impl substitution contains duplicate parameter identities")
        }
    }
}

fn reconstruct_callable_signature(
    record: &CallableSignatureRecordV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<CallableSemanticSignature, ArtifactError> {
    let callable = owners.callable(&record.callable)?;
    let owner = owners.declaration(&record.owner)?;
    let side = match record.side {
        0 => DispatchSide::Instance,
        1 => DispatchSide::Class,
        _ => return Err(referential("callable signature dispatch side is invalid")),
    };
    if callable.side != side {
        return Err(referential("callable signature side disagrees with its stable callable identity"));
    }
    if record.selector.as_ref() != record.callable.selector.as_ref() {
        return Err(referential("callable signature selector disagrees with its stable callable identity"));
    }

    let generics = record
        .generics
        .as_ref()
        .map(|signature| super::semantic_import::reconstruct_signature(signature, types, owners, allocations))
        .transpose()?;

    let mut parameters = allocations
        .vector::<CallableParameterSemantic>(record.parameters.len())
        .map_err(ArtifactError::Allocation)?;
    for parameter in &record.parameters {
        let parameter_callable = owners.callable(&parameter.callable)?;
        if parameter_callable != callable {
            return Err(referential("callable parameter identity belongs to another callable"));
        }
        let declared_type = reconstruct_declared_type(&parameter.declared_type, types, owners, allocations)?;
        let rest = rest_mode(parameter.rest_mode, true)?;
        let contract = match &parameter.contract {
            ParameterContractV1::Fixed { fact } => {
                ParameterContract::Fixed(reconstruct_declared_type(fact, types, owners, allocations)?)
            }
            ParameterContractV1::Rest { mode, contract } => {
                let mode = rest_mode(*mode, false)?;
                ParameterContract::Rest {
                    mode,
                    contract: match contract {
                        RestValueContractV1::Unconstrained => RestValueContract::Unconstrained,
                        RestValueContractV1::Element { fact } => {
                            RestValueContract::Element(reconstruct_declared_type(fact, types, owners, allocations)?)
                        }
                        RestValueContractV1::Lane { ty } => RestValueContract::Lane(mapped_type(*ty, types)?),
                        RestValueContractV1::Invalid { reason } => {
                            RestValueContract::Invalid(import_unknown_reason(reason, allocations)?)
                        }
                    },
                }
            }
        };
        if contract.rest_mode() != rest {
            return Err(referential("callable parameter rest mode disagrees with its semantic contract"));
        }
        parameters.push(CallableParameterSemantic {
            id: CallableParameterId::new(parameter_callable, parameter.index),
            local_name: allocations.boxed_str(&parameter.local_name).map_err(ArtifactError::Allocation)?,
            external_label: parameter
                .external_label
                .as_deref()
                .map(|label| allocations.boxed_str(label).map_err(ArtifactError::Allocation))
                .transpose()?,
            rest,
            declared_type,
            contract,
            source: parameter
                .source
                .as_ref()
                .map(|source| import_source_span(source, owners))
                .transpose()?,
        });
    }

    let declared_return = reconstruct_declared_type(&record.declared_return, types, owners, allocations)?;
    let return_validation = match &record.return_validation {
        ReturnContractValidationV1::NotApplicable => ReturnContractValidation::NotApplicable,
        ReturnContractValidationV1::Unchecked => ReturnContractValidation::Unchecked,
        ReturnContractValidationV1::Satisfied { status } => ReturnContractValidation::Satisfied(match status {
            EvidenceStatusV1::Established => EvidenceStatus::Established,
            EvidenceStatusV1::Assumed => EvidenceStatus::Assumed,
        }),
        ReturnContractValidationV1::Refuted => ReturnContractValidation::Refuted,
        ReturnContractValidationV1::Blocked => ReturnContractValidation::Blocked,
        ReturnContractValidationV1::DynamicBoundary => ReturnContractValidation::DynamicBoundary,
    };
    let inferred_return = record
        .inferred_return
        .as_ref()
        .map(|knowledge| import_type_knowledge(knowledge, types, allocations))
        .transpose()?;
    let source = record
        .source
        .as_ref()
        .map(|source| import_source_span(source, owners))
        .transpose()?;
    let implementation = match &record.implementation {
        ImplementationKindV1::Phalcom => ImplementationKind::Phalcom,
        ImplementationKindV1::Native => ImplementationKind::Native,
        ImplementationKindV1::Foreign => ImplementationKind::Foreign,
        ImplementationKindV1::Intrinsic => ImplementationKind::Intrinsic,
    };
    let (native_id, effects, raises, flow, lifecycle) = import_native_metadata(record)?;

    Ok(CallableSemanticSignature {
        callable,
        owner,
        side,
        selector: super::type_import::parse_selector(&record.callable.selector)?,
        generics,
        parameters: allocations.boxed_slice(parameters).map_err(ArtifactError::Allocation)?,
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

fn reconstruct_declared_type(
    row: &DeclaredTypeFactV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<DeclaredTypeFact, ArtifactError> {
    // Use the shared P7-budgeted structural reconstruction. Keeping one helper
    // prevents callable and algebraic products from drifting semantically.
    super::semantic_import::reconstruct_declared_type(row, types, owners, allocations)
}

fn import_type_knowledge(
    knowledge: &TypeKnowledgeV1,
    types: &ReconstructedTypeGraph,
    allocations: &mut AllocationBudget,
) -> Result<TypeKnowledge, ArtifactError> {
    // Preserve P7's nested transaction: partially materialized provenance must
    // not debit the parent artifact budget.
    let mut child = allocations.child(allocations.remaining());
    let imported = match knowledge {
        TypeKnowledgeV1::Known {
            ty,
            status,
            origin,
            provenance,
        } => {
            let ty = mapped_type(*ty, types)?;
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
                .vector::<SourceRange>(provenance.ranges.len())
                .map_err(ArtifactError::Allocation)?;
            for range in &provenance.ranges {
                ranges.push(source_range(range.start, range.end, "callable evidence")?);
            }
            let mut descriptions = child
                .vector::<String>(provenance.descriptions.len())
                .map_err(ArtifactError::Allocation)?;
            for description in &provenance.descriptions {
                descriptions.push(child.string(description).map_err(ArtifactError::Allocation)?);
            }
            TypeKnowledge::known_with_evidence(
                ty,
                status,
                origin,
                EvidenceSet {
                    ranges,
                    descriptions,
                },
            )
        }
        TypeKnowledgeV1::Unknown { reason } => TypeKnowledge::Unknown(import_unknown_reason(reason, &mut child)?),
        TypeKnowledgeV1::Dynamic { reason } => TypeKnowledge::Dynamic(import_dynamic_reason(reason)),
    };
    allocations.commit_child(child).map_err(ArtifactError::Allocation)?;
    Ok(imported)
}

fn import_source_span(
    source: &phalcom_compiler_artifact::semantic::world::EnumRequirementSourceV1,
    owners: &StableOwnerCatalog<'_>,
) -> Result<SemanticSourceSpan, ArtifactError> {
    Ok(SemanticSourceSpan::new(
        owners.module(&source.module)?,
        source_range(source.range.start, source.range.end, "callable source")?,
    ))
}

fn rest_mode(raw: u8, allow_none: bool) -> Result<RestMode, ArtifactError> {
    match raw {
        0 if allow_none => Ok(RestMode::None),
        1 => Ok(RestMode::Positional),
        2 => Ok(RestMode::Labeled),
        3 => Ok(RestMode::Complete),
        _ => Err(referential("callable rest mode is invalid")),
    }
}

fn mapped_type(
    id: phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId,
    types: &ReconstructedTypeGraph,
) -> Result<crate::types::id::TypeId, ArtifactError> {
    types
        .types
        .get(id.0 as usize)
        .copied()
        .ok_or_else(|| referential("callable semantic product references an absent type row"))
}

fn source_range(start: u64, end: u64, family: &str) -> Result<SourceRange, ArtifactError> {
    if start > end {
        return Err(referential(format!("{family} span is inverted")));
    }
    Ok(SourceRange {
        start: usize::try_from(start).map_err(|_| referential(format!("{family} start exceeds address space")))?,
        end: usize::try_from(end).map_err(|_| referential(format!("{family} end exceeds address space")))?,
    })
}

fn import_unknown_reason(reason: &UnknownReasonV1, allocations: &mut AllocationBudget) -> Result<UnknownReason, ArtifactError> {
    Ok(match reason {
        UnknownReasonV1::NoTypeEvidence => UnknownReason::NoTypeEvidence,
        UnknownReasonV1::MissingInitializer => UnknownReason::MissingInitializer,
        UnknownReasonV1::UnannotatedDeclaration => UnknownReason::UnannotatedDeclaration,
        UnknownReasonV1::UnresolvedName { name } => {
            UnknownReason::UnresolvedName(allocations.boxed_str(name).map_err(ArtifactError::Allocation)?)
        }
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

fn import_dynamic_reason(reason: &DynamicReasonV1) -> DynamicReason {
    match reason {
        DynamicReasonV1::ExplicitEscape => DynamicReason::ExplicitEscape,
        DynamicReasonV1::DynamicRestPack => DynamicReason::DynamicRestPack,
        DynamicReasonV1::RuntimeReflection => DynamicReason::RuntimeReflection,
    }
}

fn import_return_flow(flow: &ReturnFlowV1) -> Result<phalcom_native_meta::ReturnFlowSpec, ArtifactError> {
    Ok(match flow {
        ReturnFlowV1::Value => phalcom_native_meta::ReturnFlowSpec::Value,
        ReturnFlowV1::Receiver => phalcom_native_meta::ReturnFlowSpec::Receiver,
        ReturnFlowV1::Argument { index } => phalcom_native_meta::ReturnFlowSpec::Argument(
            usize::try_from(*index).map_err(|_| referential("callable return flow index exceeds address space"))?,
        ),
        ReturnFlowV1::Never => phalcom_native_meta::ReturnFlowSpec::Never,
        ReturnFlowV1::Unknown => phalcom_native_meta::ReturnFlowSpec::Unknown,
    })
}

fn matches_native_type_expr(spec: &phalcom_native_meta::TypeExprSpec, wire: &TypeExprSpecV1) -> bool {
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
                && arguments
                    .iter()
                    .zip(wire_args.iter())
                    .all(|(argument, wire)| matches_native_type_expr(argument, wire))
        }
        (Native::Union(members), TypeExprSpecV1::Union { members: wire_members }) => {
            members.len() == wire_members.len()
                && members
                    .iter()
                    .zip(wire_members.iter())
                    .all(|(member, wire)| matches_native_type_expr(member, wire))
        }
        (Native::Tuple(tuple), TypeExprSpecV1::Tuple { tuple: wire_tuple }) => {
            tuple.positional.len() == wire_tuple.positional.len()
                && tuple
                    .positional
                    .iter()
                    .zip(wire_tuple.positional.iter())
                    .all(|(position, wire)| matches_native_type_expr(position, wire))
                && tuple.labeled.len() == wire_tuple.labeled.len()
                && tuple
                    .labeled
                    .iter()
                    .zip(wire_tuple.labeled.iter())
                    .all(|(label, wire)| label.label == wire.label.as_ref() && matches_native_type_expr(label.ty, &wire.ty))
                && match (tuple.rest, &wire_tuple.rest) {
                    (None, None) => true,
                    (Some(rest), Some(wire_rest)) => match (rest.ty, &wire_rest.ty) {
                        (None, None) => true,
                        (Some(ty), Some(wire_ty)) => matches_native_type_expr(ty, wire_ty),
                        _ => false,
                    },
                    _ => false,
                }
        }
        _ => false,
    }
}

fn matches_native_effects(spec: phalcom_native_meta::EffectSpec, wire: &EffectSpecV1) -> bool {
    use phalcom_compiler_artifact::semantic::world::NativeEffectV1;
    match (spec, wire) {
        (phalcom_native_meta::EffectSpec::Unknown, EffectSpecV1::Unknown) => true,
        (phalcom_native_meta::EffectSpec::Pure, EffectSpecV1::Pure) => true,
        (phalcom_native_meta::EffectSpec::Known(effects), EffectSpecV1::Known { effects: wire_effects }) => {
            effects.len() == wire_effects.len()
                && effects.iter().zip(wire_effects.iter()).all(|(effect, wire)| {
                    matches!(
                        (effect, wire),
                        (phalcom_native_meta::NativeEffect::Mutation, NativeEffectV1::Mutation)
                            | (phalcom_native_meta::NativeEffect::Io, NativeEffectV1::Io)
                            | (phalcom_native_meta::NativeEffect::Scheduling, NativeEffectV1::Scheduling)
                            | (phalcom_native_meta::NativeEffect::Reflection, NativeEffectV1::Reflection)
                            | (phalcom_native_meta::NativeEffect::Nondeterminism, NativeEffectV1::Nondeterminism)
                            | (phalcom_native_meta::NativeEffect::Blocking, NativeEffectV1::Blocking)
                    )
                })
        }
        _ => false,
    }
}

fn matches_native_raises(spec: phalcom_native_meta::RaisesSpec, wire: &RaisesSpecV1) -> bool {
    match (spec, wire) {
        (phalcom_native_meta::RaisesSpec::Unknown, RaisesSpecV1::Unknown) => true,
        (phalcom_native_meta::RaisesSpec::Known(types), RaisesSpecV1::Known { types: wire_types }) => {
            types.len() == wire_types.len()
                && types
                    .iter()
                    .zip(wire_types.iter())
                    .all(|(ty, wire)| matches_native_type_expr(ty, wire))
        }
        _ => false,
    }
}

fn matches_return_flow(spec: phalcom_native_meta::ReturnFlowSpec, wire: &ReturnFlowV1) -> bool {
    use phalcom_native_meta::ReturnFlowSpec;
    match (spec, wire) {
        (ReturnFlowSpec::Value, ReturnFlowV1::Value) => true,
        (ReturnFlowSpec::Receiver, ReturnFlowV1::Receiver) => true,
        (ReturnFlowSpec::Argument(index), ReturnFlowV1::Argument { index: wire_index }) => {
            u64::try_from(index).ok() == Some(*wire_index)
        }
        (ReturnFlowSpec::Never, ReturnFlowV1::Never) => true,
        (ReturnFlowSpec::Unknown, ReturnFlowV1::Unknown) => true,
        _ => false,
    }
}

fn matches_native_lifecycle(
    spec: phalcom_native_meta::NativeLifecycleSpec,
    wire: &phalcom_compiler_artifact::semantic::world::NativeLifecycleSpecV1,
) -> bool {
    spec.since == wire.since.as_deref()
        && spec.deprecated_since == wire.deprecated_since.as_deref()
        && spec.replacement == wire.replacement.as_deref()
}

fn import_native_metadata(
    signature: &CallableSignatureRecordV1,
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
        EffectSpecV1::Known { .. } => {
            return Err(referential("non-catalog native effects require an owned metadata representation"));
        }
    };
    let raises = match &signature.raises {
        RaisesSpecV1::Unknown => phalcom_native_meta::RaisesSpec::Unknown,
        RaisesSpecV1::Known { types } if types.is_empty() => phalcom_native_meta::RaisesSpec::Known(&[]),
        RaisesSpecV1::Known { .. } => {
            return Err(referential("non-catalog raised-type metadata requires an owned representation"));
        }
    };
    if signature.lifecycle.since.is_some()
        || signature.lifecycle.deprecated_since.is_some()
        || signature.lifecycle.replacement.is_some()
    {
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

fn referential(message: impl Into<String>) -> ArtifactError {
    ArtifactError::Validation(ValidationError::ReferentialIntegrity(message.into()))
}
