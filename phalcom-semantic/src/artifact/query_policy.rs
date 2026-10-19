//! Exhaustive mapping from current semantic query kinds to artifact profile rows.
//!
//! This table classifies where a query's durable product lives. It does not
//! claim that a typed wire reference is already published as a live
//! `SemanticProduct`; the live importer must still reconstruct and publish it.

use crate::db::key::QueryKind;
use phalcom_compiler_artifact::query::{PortableQueryKindV1, QueryProfileSupport};

pub const fn portable_query_kind(kind: QueryKind) -> PortableQueryKindV1 {
    match kind {
        QueryKind::ParsedModule => PortableQueryKindV1::ParsedModule,
        QueryKind::UnlinkedInterface => PortableQueryKindV1::UnlinkedInterface,
        QueryKind::ResolvedImports => PortableQueryKindV1::ResolvedImports,
        QueryKind::LinkedInterface => PortableQueryKindV1::LinkedInterface,
        QueryKind::DeclarationShell => PortableQueryKindV1::DeclarationShell,
        QueryKind::SemanticComponent => PortableQueryKindV1::SemanticComponent,
        QueryKind::DeclarationSurface => PortableQueryKindV1::DeclarationSurface,
        QueryKind::TraitHeader => PortableQueryKindV1::TraitHeader,
        QueryKind::TraitSurface => PortableQueryKindV1::TraitSurface,
        QueryKind::ConformanceDispatch => PortableQueryKindV1::ConformanceDispatch,
        QueryKind::TraitDispatchBucket => PortableQueryKindV1::TraitDispatchBucket,
        QueryKind::GenericConformance => PortableQueryKindV1::GenericConformance,
        QueryKind::CallableDefinition => PortableQueryKindV1::CallableDefinition,
        QueryKind::HierarchyEdge => PortableQueryKindV1::HierarchyEdge,
        QueryKind::FieldSignature => PortableQueryKindV1::FieldSignature,
        QueryKind::ConstructorInitialization => PortableQueryKindV1::ConstructorInitialization,
        QueryKind::CallableSignature => PortableQueryKindV1::CallableSignature,
        QueryKind::CallableBody => PortableQueryKindV1::CallableBody,
        QueryKind::CallableEffects => PortableQueryKindV1::CallableEffects,
        QueryKind::CallableControl => PortableQueryKindV1::CallableControl,
        QueryKind::CallableTermination => PortableQueryKindV1::CallableTermination,
        QueryKind::CallableContracts => PortableQueryKindV1::CallableContracts,
        QueryKind::AttributeArgument => PortableQueryKindV1::AttributeArgument,
        QueryKind::VerificationConditions => PortableQueryKindV1::VerificationConditions,
        QueryKind::SourceStructure => PortableQueryKindV1::SourceStructure,
        QueryKind::SourceFormalAttachment => PortableQueryKindV1::SourceFormalAttachment,
        QueryKind::AdvisoryCallable => PortableQueryKindV1::AdvisoryCallable,
        QueryKind::AdvisoryModule => PortableQueryKindV1::AdvisoryModule,
        QueryKind::ModuleDiagnostics => PortableQueryKindV1::ModuleDiagnostics,
        QueryKind::ModuleMetadata => PortableQueryKindV1::ModuleMetadata,
        QueryKind::DataDeclaration => PortableQueryKindV1::DataDeclaration,
        QueryKind::EnumDeclaration => PortableQueryKindV1::EnumDeclaration,
        QueryKind::EnumRequirements => PortableQueryKindV1::EnumRequirements,
        QueryKind::TypeMemberDeclaration => PortableQueryKindV1::TypeMemberDeclaration,
        QueryKind::StaticDomainCaseCatalog => PortableQueryKindV1::StaticDomainCaseCatalog,
        QueryKind::StaticCaseSignature => PortableQueryKindV1::StaticCaseSignature,
        QueryKind::AssociatedStaticDomainDeclaration => PortableQueryKindV1::AssociatedStaticDomainDeclaration,
        QueryKind::AssociatedSurface => PortableQueryKindV1::AssociatedSurface,
        QueryKind::ResolvedImport => PortableQueryKindV1::ResolvedImport,
        QueryKind::LinkedName => PortableQueryKindV1::LinkedName,
        QueryKind::PublicExport => PortableQueryKindV1::PublicExport,
        QueryKind::StaticDefinition => PortableQueryKindV1::StaticDefinition,
        QueryKind::StaticConstant => PortableQueryKindV1::StaticConstant,
        QueryKind::ModuleStaticBindingCatalog => PortableQueryKindV1::ModuleStaticBindingCatalog,
        QueryKind::ModuleStaticBindings => PortableQueryKindV1::ModuleStaticBindings,
        QueryKind::CheckedStaticBody => PortableQueryKindV1::CheckedStaticBody,
    }
}

pub const fn query_kind_profile_support(kind: QueryKind) -> QueryProfileSupport {
    portable_query_kind(kind).profile_support()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn current_query_kind_inventory_has_one_explicit_portable_disposition_each() {
        let mapped = QueryKind::ALL.map(portable_query_kind);
        let unique = mapped.into_iter().collect::<BTreeSet<_>>();
        assert_eq!(QueryKind::count(), 46);
        assert_eq!(unique.len(), QueryKind::count());
        assert!(unique.iter().all(|kind| PortableQueryKindV1::from_raw(*kind as u16) == Some(*kind)));
    }

    #[test]
    fn static_query_families_have_explicit_semantic_world_dispositions() {
        for kind in [QueryKind::StaticDefinition, QueryKind::StaticConstant, QueryKind::CheckedStaticBody] {
            assert_eq!(query_kind_profile_support(kind), QueryProfileSupport::RepresentedBySemanticWorld);
        }
    }
}
