//! Certified PERF003 P7 static semantic-dependency reconstruction.
//!
//! Static definitions/constants and full callable analyses share this exact
//! dependency vocabulary. Keeping one importer prevents each owner family from
//! drifting in identity relocation or query-key semantics.

use super::identity::StableOwnerCatalog;
use super::type_import::ReconstructedTypeGraph;
use crate::checker::analysis::SemanticDependency;
use crate::generic_evidence::GenericConformanceKey;
use crate::identity::{
    AssociatedStaticDomainDeclarationId, FieldId, StaticCaseId, StaticDomainDeclarationId,
};
use crate::static_constant::StaticConstantId;
use crate::trait_dispatch::{TraitDispatchBucketKey, TraitDispatchTargetFamily};
use crate::types::type_member::{TypeMemberDeclarationId, TypeMemberSelector};
use phalcom_compiler_artifact::semantic::type_store::TypeMemberSelectorV1;
use phalcom_compiler_artifact::semantic::world::{
    ConformanceTargetV1, StaticCaseDomainV1, StaticSemanticDependencyV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_modules::identity::{ImportSiteId, ImportSiteLocalId};
use phalcom_modules::linker::SymbolId;
use phalcom_type_meta::allocation::AllocationBudget;

pub(crate) fn reconstruct_static_dependency(
    dependency: &StaticSemanticDependencyV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<SemanticDependency, ArtifactError> {
    use StaticSemanticDependencyV1 as Wire;
    Ok(match dependency {
        Wire::DeclarationShell { owner } => SemanticDependency::DeclarationShell(owners.declaration(owner)?),
        Wire::TraitHeader { owner } => SemanticDependency::TraitHeader(owners.declaration(owner)?),
        Wire::CallableSignature { callable } => SemanticDependency::CallableSignature(owners.callable(callable)?),
        Wire::FieldSignature { field } => SemanticDependency::FieldSignature(FieldId::new(
            owners.declaration(&field.owner)?,
            allocations.boxed_str(&field.name).map_err(ArtifactError::Allocation)?,
            super::identity::dispatch_side(field.side),
        )),
        Wire::DeclarationSurface { owner } => SemanticDependency::DeclarationSurface(owners.declaration(owner)?),
        Wire::TraitSurface { owner } => SemanticDependency::TraitSurface(owners.declaration(owner)?),
        Wire::ConformanceDispatch { implementation } => {
            SemanticDependency::ConformanceDispatch(owners.implementation(implementation)?)
        }
        Wire::TraitDispatchBucket { target, selector, side } => {
            let target_family = match target {
                ConformanceTargetV1::Declaration { declaration } => {
                    TraitDispatchTargetFamily::Declaration(owners.declaration(declaration)?)
                }
                ConformanceTargetV1::ExactEnumCase { variant } => {
                    TraitDispatchTargetFamily::ExactEnumCase(owners.variant(variant)?)
                }
            };
            SemanticDependency::TraitDispatchBucket(TraitDispatchBucketKey {
                target_family,
                selector: super::denotation_import::exact_selector(selector, allocations)?,
                side: super::identity::dispatch_side(*side),
            })
        }
        Wire::GenericConformance { subject, trait_ref } => {
            let mut arguments = allocations
                .vector::<crate::types::id::TypeId>(trait_ref.arguments.len())
                .map_err(ArtifactError::Allocation)?;
            for argument in &trait_ref.arguments {
                arguments.push(
                    *types
                        .types
                        .get(argument.0 as usize)
                        .ok_or_else(|| invalid("static dependency trait reference uses an absent type"))?,
                );
            }
            SemanticDependency::GenericConformance(GenericConformanceKey::new(
                super::semantic_import::reconstruct_term(subject, types, owners)?,
                crate::traits::TraitRef::new(
                    owners.declaration(&trait_ref.declaration)?,
                    allocations.boxed_slice(arguments).map_err(ArtifactError::Allocation)?,
                ),
            ))
        }
        Wire::HierarchyEdge { owner } => SemanticDependency::HierarchyEdge(owners.declaration(owner)?),
        Wire::LinkedInterface { module } => SemanticDependency::LinkedInterface(owners.module(module)?),
        Wire::DataDeclaration { owner } => SemanticDependency::DataDeclaration(owners.declaration(owner)?),
        Wire::EnumDeclaration { owner } => SemanticDependency::EnumDeclaration(owners.declaration(owner)?),
        Wire::AssociatedSurface { owner } => SemanticDependency::AssociatedSurface(owners.declaration(owner)?),
        Wire::TypeMemberDeclaration { owner, selector } => {
            SemanticDependency::TypeMemberDeclaration(TypeMemberDeclarationId::new(
                owners.declaration(owner)?,
                type_member_selector(selector, allocations)?,
            ))
        }
        Wire::StaticDomainCaseCatalog { domain } => {
            SemanticDependency::StaticDomainCaseCatalog(static_domain(domain, owners, allocations)?)
        }
        Wire::StaticCaseSignature { domain, selector } => {
            SemanticDependency::StaticCaseSignature(StaticCaseId::with_selector(
                static_domain(domain, owners, allocations)?,
                super::denotation_import::exact_selector(selector, allocations)?,
            ))
        }
        Wire::StaticDefinition { owner } => SemanticDependency::StaticDefinition(owners.declaration(owner)?),
        Wire::StaticConstant { symbol } => SemanticDependency::StaticConstant(StaticConstantId(SymbolId {
            module: owners.module(&symbol.module)?,
            name: allocations.boxed_str(&symbol.name).map_err(ArtifactError::Allocation)?,
        })),
        Wire::ModuleStaticBindingCatalog { module } => {
            SemanticDependency::ModuleStaticBindingCatalog(owners.module(module)?)
        }
        Wire::ConstructorInitialization { owner } => {
            SemanticDependency::ConstructorInitialization(owners.declaration(owner)?)
        }
        Wire::ResolvedImport { module, local_id } => SemanticDependency::ResolvedImport(ImportSiteId::new(
            owners.module(module)?,
            ImportSiteLocalId(*local_id),
        )),
        Wire::LinkedName { module, name } => SemanticDependency::LinkedName(
            owners.module(module)?,
            allocations.string(name).map_err(ArtifactError::Allocation)?,
        ),
        Wire::PublicExport { module, name } => SemanticDependency::PublicExport(
            owners.module(module)?,
            allocations.string(name).map_err(ArtifactError::Allocation)?,
        ),
        Wire::ModuleStaticBindings { module } => {
            SemanticDependency::ModuleStaticBindings(owners.module(module)?)
        }
    })
}

fn static_domain(
    domain: &StaticCaseDomainV1,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<StaticDomainDeclarationId, ArtifactError> {
    Ok(match domain {
        StaticCaseDomainV1::Nominal { declaration } => {
            StaticDomainDeclarationId::Nominal(owners.declaration(declaration)?)
        }
        StaticCaseDomainV1::Associated { declaration } => {
            StaticDomainDeclarationId::Associated(AssociatedStaticDomainDeclarationId::new(
                owners.declaration(&declaration.owner)?,
                allocations.boxed_str(&declaration.name).map_err(ArtifactError::Allocation)?,
            ))
        }
    })
}

fn type_member_selector(
    selector: &TypeMemberSelectorV1,
    allocations: &mut AllocationBudget,
) -> Result<TypeMemberSelector, ArtifactError> {
    Ok(match selector {
        TypeMemberSelectorV1::Named { name } => {
            if name.as_str() == "At" {
                return Err(invalid("noncanonical type-member selector aliases subscript identity"));
            }
            TypeMemberSelector::Named(
                allocations.boxed_str(name).map_err(ArtifactError::Allocation)?,
            )
        }
        TypeMemberSelectorV1::Subscript => TypeMemberSelector::Subscript,
    })
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
