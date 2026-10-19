//! Exact conversion from project-qualified wire query owners to live keys.

use crate::db::key::QueryKey;
use crate::identity::{AssociatedStaticDomainDeclarationId, DispatchSide, FieldId, StaticDomainDeclarationId};
use crate::static_constant::StaticConstantId;
use crate::static_definition::StaticBodyId;
use crate::types::type_member::{TypeMemberDeclarationId, TypeMemberSelector};
use phalcom_compiler_artifact::query::{PortableQueryKindV1 as Kind, PortableQueryOwnerKeyV1 as Owner, QueryImageV3};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_modules::identity::{ImportSiteId, ImportSiteLocalId};
use phalcom_type_meta::identity::StableStaticDomainDeclarationRef;

/// Converts a portable owner to its exact live query key.
///
/// This performs identity relocation only. Callers must separately check
/// `query_root_is_reusable`, product location, row ownership, fingerprints,
/// current validation, and dependency closure before publishing a result.
impl QueryKey {
    pub fn from_portable_owner(kind: Kind, owner: &Owner, owners: &super::identity::StableOwnerCatalog<'_>) -> Result<Self, ArtifactError> {
        use Kind as K;
        use Owner as O;

        let key = match kind {
            K::ParsedModule => Self::ParsedModule(module_owner(owner, owners)?),
            K::UnlinkedInterface => Self::UnlinkedInterface(module_owner(owner, owners)?),
            K::ResolvedImports => Self::ResolvedImports(module_owner(owner, owners)?),
            K::LinkedInterface => Self::LinkedInterface(module_owner(owner, owners)?),
            K::DeclarationShell => Self::DeclarationShell(declaration_owner(owner, owners)?),
            K::HierarchyEdge => Self::HierarchyEdge(declaration_owner(owner, owners)?),
            K::CallableDefinition => Self::CallableDefinition(callable_owner(owner, owners)?),
            K::CallableSignature => Self::CallableSignature(callable_owner(owner, owners)?),
            K::FieldSignature => Self::FieldSignature(field_owner(owner, owners)?),
            K::DeclarationSurface => Self::DeclarationSurface(declaration_owner(owner, owners)?),
            K::TraitHeader => Self::TraitHeader(declaration_owner(owner, owners)?),
            K::TraitSurface => Self::TraitSurface(declaration_owner(owner, owners)?),
            K::DataDeclaration => Self::DataDeclaration(declaration_owner(owner, owners)?),
            K::EnumDeclaration => Self::EnumDeclaration(declaration_owner(owner, owners)?),
            K::AssociatedSurface => Self::AssociatedSurface(declaration_owner(owner, owners)?),
            K::TypeMemberDeclaration => {
                let O::TypeMember(reference) = owner else {
                    return Err(owner_mismatch(kind, "type member"));
                };
                let selector = match reference.selector.as_ref() {
                    "[]" => TypeMemberSelector::Subscript,
                    "At" => return Err(invalid("noncanonical stable type-member selector aliases the subscript identity")),
                    "" => return Err(invalid("empty stable type-member selector")),
                    name => TypeMemberSelector::Named(name.to_owned().into_boxed_str()),
                };
                Self::TypeMemberDeclaration(TypeMemberDeclarationId::new(owners.declaration(&reference.owner)?, selector))
            }
            K::ModuleStaticBindingCatalog => Self::ModuleStaticBindingCatalog(module_owner(owner, owners)?),
            K::ModuleStaticBindings => Self::ModuleStaticBindings(module_owner(owner, owners)?),
            K::SemanticComponent => Self::SemanticComponent(module_owner(owner, owners)?),
            K::ConformanceDispatch => {
                let O::Impl(reference) = owner else {
                    return Err(owner_mismatch(kind, "impl"));
                };
                Self::ConformanceDispatch(owners.implementation(reference)?)
            }
            // The wire owner has no exact enum-case discriminator. Mapping its
            // declaration projection to a declaration-family bucket would alias
            // distinct `ExactEnumCase` keys.
            K::TraitDispatchBucket => {
                if !matches!(owner, O::TraitDispatchBucket { .. }) {
                    return Err(owner_mismatch(kind, "trait-dispatch bucket"));
                }
                return Err(invalid("portable trait-dispatch owner cannot distinguish declaration and exact-case targets"));
            }
            // The pair (impl, context ordinal) does not encode the exact subject
            // and trait proposition required by GenericConformanceKey.
            K::GenericConformance => {
                if !matches!(owner, O::GenericConformance { .. }) {
                    return Err(owner_mismatch(kind, "generic-conformance context"));
                }
                return Err(invalid("portable generic-conformance owner does not identify an exact proposition"));
            }
            K::ConstructorInitialization => Self::ConstructorInitialization(declaration_owner(owner, owners)?),
            K::CallableBody => Self::CallableBody(callable_owner(owner, owners)?),
            K::CallableEffects => Self::CallableEffects(callable_owner(owner, owners)?),
            K::CallableControl => Self::CallableControl(callable_owner(owner, owners)?),
            K::CallableTermination => Self::CallableTermination(callable_owner(owner, owners)?),
            K::CallableContracts => Self::CallableContracts(callable_owner(owner, owners)?),
            K::AttributeArgument => {
                if !matches!(owner, O::AttributeArgument { .. }) {
                    return Err(owner_mismatch(kind, "attribute argument"));
                }
                return Err(invalid("portable attribute-argument owner omits the canonical analysis context"));
            }
            K::VerificationConditions => Self::VerificationConditions(callable_owner(owner, owners)?),
            K::SourceStructure => Self::SourceStructure(module_owner(owner, owners)?),
            K::SourceFormalAttachment => Self::SourceFormalAttachment(callable_owner(owner, owners)?),
            K::AdvisoryCallable => Self::AdvisoryCallable(callable_owner(owner, owners)?),
            K::AdvisoryModule => Self::AdvisoryModule(module_owner(owner, owners)?),
            K::ModuleDiagnostics => Self::ModuleDiagnostics(module_owner(owner, owners)?),
            K::ModuleMetadata => Self::ModuleMetadata(module_owner(owner, owners)?),
            K::EnumRequirements => Self::EnumRequirements(declaration_owner(owner, owners)?),
            K::StaticDomainCaseCatalog => {
                let O::StaticDomain(reference) = owner else {
                    return Err(owner_mismatch(kind, "static domain"));
                };
                let domain = match reference {
                    StableStaticDomainDeclarationRef::Nominal(declaration) => StaticDomainDeclarationId::Nominal(owners.declaration(declaration)?),
                    StableStaticDomainDeclarationRef::Associated(associated) => StaticDomainDeclarationId::Associated(
                        AssociatedStaticDomainDeclarationId::new(owners.declaration(&associated.owner)?, associated.name.clone()),
                    ),
                };
                Self::StaticDomainCaseCatalog(domain)
            }
            // A StableVariantRef cannot distinguish two associated static
            // domains declared on the same owner. Keep this unsupported until
            // the wire identity carries the complete StaticCaseId domain.
            K::StaticCaseSignature => {
                if !matches!(owner, O::StaticCase(_)) {
                    return Err(owner_mismatch(kind, "static case"));
                }
                return Err(invalid("portable static-case owner omits the complete static-domain identity"));
            }
            K::AssociatedStaticDomainDeclaration => {
                let O::AssociatedStaticDomain(reference) = owner else {
                    return Err(owner_mismatch(kind, "associated static domain"));
                };
                Self::AssociatedStaticDomainDeclaration(AssociatedStaticDomainDeclarationId::new(
                    owners.declaration(&reference.owner)?,
                    reference.name.clone(),
                ))
            }
            K::ResolvedImport => {
                let O::ImportSite { module, site_id } = owner else {
                    return Err(owner_mismatch(kind, "import site"));
                };
                Self::ResolvedImport(ImportSiteId::new(owners.module(module)?, ImportSiteLocalId(*site_id)))
            }
            K::LinkedName => {
                let (module, name) = module_name_owner(kind, owner, owners)?;
                Self::LinkedName(module, name)
            }
            K::PublicExport => {
                let (module, name) = module_name_owner(kind, owner, owners)?;
                Self::PublicExport(module, name)
            }
            K::StaticDefinition => Self::StaticDefinition(declaration_owner(owner, owners)?),
            K::StaticConstant => {
                let O::StaticConstant(reference) = owner else {
                    return Err(owner_mismatch(kind, "static constant"));
                };
                Self::StaticConstant(StaticConstantId::new(owners.module(&reference.module)?, reference.name.clone()))
            }
            K::CheckedStaticBody => {
                let O::CheckedStaticBody {
                    owner: declaration,
                    body_index,
                } = owner
                else {
                    return Err(owner_mismatch(kind, "checked static body"));
                };
                if *body_index != 0 {
                    return Err(invalid("portable checked-static-body index is not representable by the current owner identity"));
                }
                Self::CheckedStaticBody(StaticBodyId(owners.declaration(declaration)?))
            }
        };

        if super::query_policy::portable_query_kind(key.kind()) != kind {
            return Err(invalid("portable query kind does not match the reconstructed live key"));
        }
        Ok(key)
    }
}

/// Checks project-qualified owner authority before retaining the wire query
/// image. Only exact keys for families that can become reusable are converted
/// into live `QueryKey`s; other products remain transport input for snapshot
/// reconstruction and cannot be published as `Ready` by this path.
pub(crate) fn validate_query_owner_authority(image: &QueryImageV3, owners: &super::identity::StableOwnerCatalog<'_>) -> Result<(), ArtifactError> {
    for root in &image.supported_roots {
        validate_owner_authority(&root.key.owner, owners)?;
        let kind = super::query_policy::query_kind_from_portable(root.key.kind);
        if super::query_policy::query_root_is_reusable(kind) {
            QueryKey::from_portable_owner(root.key.kind, &root.key.owner, owners)?;
        }
    }
    for dependency in &image.dependency_evidence {
        validate_owner_authority(&dependency.key.owner, owners)?;
        validate_owner_authority(&dependency.stable_aggregate_owner, owners)?;
        let kind = super::query_policy::query_kind_from_portable(dependency.key.kind);
        if matches!(
            super::query_policy::canonical_query_persistence(kind),
            super::query_policy::CanonicalQueryPersistence::EligibleStructural
        ) {
            QueryKey::from_portable_owner(dependency.key.kind, &dependency.key.owner, owners)?;
        }
    }
    Ok(())
}

fn validate_owner_authority(owner: &Owner, owners: &super::identity::StableOwnerCatalog<'_>) -> Result<(), ArtifactError> {
    owners.module_parts(owner.project(), owner.module_path()).map(|_| ())
}

fn module_owner(owner: &Owner, owners: &super::identity::StableOwnerCatalog<'_>) -> Result<crate::identity::ModuleId, ArtifactError> {
    let Owner::Module(module) = owner else {
        return Err(invalid("portable query owner is not a module"));
    };
    owners.module(module)
}

fn declaration_owner(owner: &Owner, owners: &super::identity::StableOwnerCatalog<'_>) -> Result<crate::identity::DeclarationId, ArtifactError> {
    let Owner::Declaration(declaration) = owner else {
        return Err(invalid("portable query owner is not a declaration"));
    };
    owners.declaration(declaration)
}

fn callable_owner(owner: &Owner, owners: &super::identity::StableOwnerCatalog<'_>) -> Result<crate::identity::CallableId, ArtifactError> {
    let Owner::Callable(callable) = owner else {
        return Err(invalid("portable query owner is not a callable"));
    };
    owners.callable(callable)
}

fn field_owner(owner: &Owner, owners: &super::identity::StableOwnerCatalog<'_>) -> Result<FieldId, ArtifactError> {
    let Owner::Field(field) = owner else {
        return Err(invalid("portable query owner is not a field"));
    };
    Ok(FieldId::new(
        owners.declaration(&field.owner)?,
        field.name.clone(),
        match field.side {
            phalcom_type_meta::identity::StableDispatchSide::Instance => DispatchSide::Instance,
            phalcom_type_meta::identity::StableDispatchSide::Class => DispatchSide::Class,
        },
    ))
}

fn module_name_owner(
    kind: Kind,
    owner: &Owner,
    owners: &super::identity::StableOwnerCatalog<'_>,
) -> Result<(crate::identity::ModuleId, String), ArtifactError> {
    let Owner::ModuleName { module, name } = owner else {
        return Err(owner_mismatch(kind, "module name"));
    };
    Ok((owners.module(module)?, name.to_string()))
}

fn owner_mismatch(kind: Kind, expected: &str) -> ArtifactError {
    invalid(format!("portable query owner for {kind:?} is not a {expected}"))
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
