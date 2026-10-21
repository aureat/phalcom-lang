//! Exhaustive mapping from current semantic query kinds to artifact profile rows.
//!
//! This table classifies where a query's durable product lives. It does not
//! claim that a typed wire reference is already published as a live
//! `SemanticProduct`; the live importer must still reconstruct and publish it.

use crate::db::key::QueryKind;
use phalcom_compiler_artifact::query::{PortableQueryKindV1, QueryProfileSupport};

/// Current-main persistence policy for one query family.
///
/// This is deliberately independent from [`QueryProfileSupport`]: product
/// location says where data is encoded, while persistence says whether a
/// validated product can be reused in a receiving workspace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalQueryPersistence {
    /// A structurally positive product may be retained if its product and
    /// dependency contracts validate during import.
    EligibleStructural,
    /// The result can depend on an extensible local candidate universe.
    OpenWorld,
    /// No current relocation/immutability contract permits importing it.
    Unsupported,
}

/// Canonical Universe compiler-image root requirement.
///
/// This is deliberately explicit rather than derived from structural
/// persistence: a family can be structurally representable without being a
/// reusable compiler root (for example ParsedModule and HierarchyEdge).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalQueryRootRequirement {
    /// Every source-canonical root of this family must survive artifact export.
    Required,
    /// A reusable root may be omitted only when its closure is unavailable.
    Optional,
    /// This family is not a reusable canonical compiler root.
    NotReusable,
}

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

pub const fn query_kind_from_portable(kind: PortableQueryKindV1) -> QueryKind {
    match kind {
        PortableQueryKindV1::ParsedModule => QueryKind::ParsedModule,
        PortableQueryKindV1::UnlinkedInterface => QueryKind::UnlinkedInterface,
        PortableQueryKindV1::ResolvedImports => QueryKind::ResolvedImports,
        PortableQueryKindV1::LinkedInterface => QueryKind::LinkedInterface,
        PortableQueryKindV1::DeclarationShell => QueryKind::DeclarationShell,
        PortableQueryKindV1::HierarchyEdge => QueryKind::HierarchyEdge,
        PortableQueryKindV1::CallableDefinition => QueryKind::CallableDefinition,
        PortableQueryKindV1::CallableSignature => QueryKind::CallableSignature,
        PortableQueryKindV1::FieldSignature => QueryKind::FieldSignature,
        PortableQueryKindV1::DeclarationSurface => QueryKind::DeclarationSurface,
        PortableQueryKindV1::TraitHeader => QueryKind::TraitHeader,
        PortableQueryKindV1::TraitSurface => QueryKind::TraitSurface,
        PortableQueryKindV1::DataDeclaration => QueryKind::DataDeclaration,
        PortableQueryKindV1::EnumDeclaration => QueryKind::EnumDeclaration,
        PortableQueryKindV1::AssociatedSurface => QueryKind::AssociatedSurface,
        PortableQueryKindV1::TypeMemberDeclaration => QueryKind::TypeMemberDeclaration,
        PortableQueryKindV1::ModuleStaticBindingCatalog => QueryKind::ModuleStaticBindingCatalog,
        PortableQueryKindV1::ModuleStaticBindings => QueryKind::ModuleStaticBindings,
        PortableQueryKindV1::SemanticComponent => QueryKind::SemanticComponent,
        PortableQueryKindV1::ConformanceDispatch => QueryKind::ConformanceDispatch,
        PortableQueryKindV1::TraitDispatchBucket => QueryKind::TraitDispatchBucket,
        PortableQueryKindV1::GenericConformance => QueryKind::GenericConformance,
        PortableQueryKindV1::ConstructorInitialization => QueryKind::ConstructorInitialization,
        PortableQueryKindV1::CallableBody => QueryKind::CallableBody,
        PortableQueryKindV1::CallableEffects => QueryKind::CallableEffects,
        PortableQueryKindV1::CallableControl => QueryKind::CallableControl,
        PortableQueryKindV1::CallableTermination => QueryKind::CallableTermination,
        PortableQueryKindV1::CallableContracts => QueryKind::CallableContracts,
        PortableQueryKindV1::AttributeArgument => QueryKind::AttributeArgument,
        PortableQueryKindV1::VerificationConditions => QueryKind::VerificationConditions,
        PortableQueryKindV1::SourceStructure => QueryKind::SourceStructure,
        PortableQueryKindV1::SourceFormalAttachment => QueryKind::SourceFormalAttachment,
        PortableQueryKindV1::AdvisoryCallable => QueryKind::AdvisoryCallable,
        PortableQueryKindV1::AdvisoryModule => QueryKind::AdvisoryModule,
        PortableQueryKindV1::ModuleDiagnostics => QueryKind::ModuleDiagnostics,
        PortableQueryKindV1::ModuleMetadata => QueryKind::ModuleMetadata,
        PortableQueryKindV1::EnumRequirements => QueryKind::EnumRequirements,
        PortableQueryKindV1::StaticDomainCaseCatalog => QueryKind::StaticDomainCaseCatalog,
        PortableQueryKindV1::StaticCaseSignature => QueryKind::StaticCaseSignature,
        PortableQueryKindV1::AssociatedStaticDomainDeclaration => QueryKind::AssociatedStaticDomainDeclaration,
        PortableQueryKindV1::ResolvedImport => QueryKind::ResolvedImport,
        PortableQueryKindV1::LinkedName => QueryKind::LinkedName,
        PortableQueryKindV1::PublicExport => QueryKind::PublicExport,
        PortableQueryKindV1::StaticDefinition => QueryKind::StaticDefinition,
        PortableQueryKindV1::StaticConstant => QueryKind::StaticConstant,
        PortableQueryKindV1::CheckedStaticBody => QueryKind::CheckedStaticBody,
    }
}

pub const fn query_kind_profile_support(kind: QueryKind) -> QueryProfileSupport {
    portable_query_kind(kind).profile_support()
}

/// Returns whether a query family may be imported as a reusable root.
///
/// Persistence eligibility and wire product location are independent gates:
/// `ParsedModule` is structurally classified by PERF003 but source-dependent,
/// while `HierarchyEdge` is only dependency evidence in this artifact profile.
pub const fn query_root_is_reusable(kind: QueryKind) -> bool {
    matches!(canonical_query_persistence(kind), CanonicalQueryPersistence::EligibleStructural)
        && matches!(
            query_kind_profile_support(kind),
            QueryProfileSupport::RepresentedByModuleWorld | QueryProfileSupport::RepresentedBySemanticWorld
        )
}

/// Explicit current compiler-image root profile required by ARTF001.C1.P2.R4.T4.
///
/// Required families are consumed as exact immutable query roots by ordinary
/// image-backed semantic compilation. Optional families may still contribute
/// reconstructible dependency evidence, but their root identity is not required
/// when that identity depends on source-only provenance.
pub const fn canonical_query_root_requirement(kind: QueryKind) -> CanonicalQueryRootRequirement {
    match kind {
        QueryKind::DeclarationShell | QueryKind::CallableSignature | QueryKind::DeclarationSurface => CanonicalQueryRootRequirement::Required,
        QueryKind::UnlinkedInterface
        | QueryKind::ResolvedImports
        | QueryKind::LinkedInterface
        | QueryKind::CallableDefinition
        | QueryKind::FieldSignature
        | QueryKind::TraitHeader
        | QueryKind::TraitSurface
        | QueryKind::DataDeclaration
        | QueryKind::EnumDeclaration
        | QueryKind::AssociatedSurface
        | QueryKind::TypeMemberDeclaration
        | QueryKind::ModuleStaticBindingCatalog
        | QueryKind::ModuleStaticBindings => CanonicalQueryRootRequirement::Optional,
        _ => CanonicalQueryRootRequirement::NotReusable,
    }
}

/// Exhaustive current-main query reuse contract, ported from certified
/// PERF003.C2.P7's `canonical_query_persistence` at `077d93d3`.
///
/// Artifact product location and per-root checks are additional gates: an
/// eligible family still requires a typed product, matching fingerprints,
/// current owner authority, and a closed acyclic dependency graph. Open-world
/// families are retained as semantic source rows when needed but are never
/// imported as unconditionally reusable `Ready` products.
pub const fn canonical_query_persistence(kind: QueryKind) -> CanonicalQueryPersistence {
    match kind {
        QueryKind::ParsedModule
        | QueryKind::UnlinkedInterface
        | QueryKind::ResolvedImports
        | QueryKind::LinkedInterface
        | QueryKind::DeclarationShell
        | QueryKind::HierarchyEdge
        | QueryKind::CallableDefinition
        | QueryKind::CallableSignature
        | QueryKind::FieldSignature
        | QueryKind::DeclarationSurface
        | QueryKind::TraitHeader
        | QueryKind::TraitSurface
        | QueryKind::DataDeclaration
        | QueryKind::EnumDeclaration
        | QueryKind::AssociatedSurface
        | QueryKind::TypeMemberDeclaration
        | QueryKind::ModuleStaticBindingCatalog
        | QueryKind::ModuleStaticBindings => CanonicalQueryPersistence::EligibleStructural,
        QueryKind::GenericConformance
        | QueryKind::ConformanceDispatch
        | QueryKind::TraitDispatchBucket
        | QueryKind::CallableBody
        | QueryKind::CheckedStaticBody => CanonicalQueryPersistence::OpenWorld,
        QueryKind::SemanticComponent
        | QueryKind::ConstructorInitialization
        | QueryKind::CallableEffects
        | QueryKind::CallableControl
        | QueryKind::CallableTermination
        | QueryKind::CallableContracts
        | QueryKind::AttributeArgument
        | QueryKind::VerificationConditions
        | QueryKind::SourceStructure
        | QueryKind::SourceFormalAttachment
        | QueryKind::AdvisoryCallable
        | QueryKind::AdvisoryModule
        | QueryKind::ModuleDiagnostics
        | QueryKind::ModuleMetadata
        | QueryKind::EnumRequirements
        | QueryKind::StaticDomainCaseCatalog
        | QueryKind::StaticCaseSignature
        | QueryKind::AssociatedStaticDomainDeclaration
        | QueryKind::ResolvedImport
        | QueryKind::LinkedName
        | QueryKind::PublicExport
        | QueryKind::StaticDefinition
        | QueryKind::StaticConstant => CanonicalQueryPersistence::Unsupported,
    }
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
        assert!(QueryKind::ALL.iter().all(|kind| query_kind_from_portable(portable_query_kind(*kind)) == *kind));
    }

    #[test]
    fn static_query_families_have_explicit_semantic_world_dispositions() {
        for kind in [QueryKind::StaticDefinition, QueryKind::StaticConstant, QueryKind::CheckedStaticBody] {
            assert_eq!(query_kind_profile_support(kind), QueryProfileSupport::RepresentedBySemanticWorld);
        }
    }

    #[test]
    fn all_46_query_kinds_have_an_explicit_persistence_contract() {
        assert_eq!(QueryKind::ALL.len(), 46);
        for kind in QueryKind::ALL {
            let _ = canonical_query_persistence(kind);
        }
        assert_eq!(
            canonical_query_persistence(QueryKind::AttributeArgument),
            CanonicalQueryPersistence::Unsupported
        );
    }

    #[test]
    fn open_world_families_are_never_classified_as_structurally_reusable() {
        for kind in [
            QueryKind::GenericConformance,
            QueryKind::ConformanceDispatch,
            QueryKind::TraitDispatchBucket,
            QueryKind::CallableBody,
            QueryKind::CheckedStaticBody,
        ] {
            assert_eq!(canonical_query_persistence(kind), CanonicalQueryPersistence::OpenWorld);
        }
    }

    #[test]
    fn required_compiler_root_profile_is_explicit_and_matches_reusable_families() {
        let required = QueryKind::ALL
            .into_iter()
            .filter(|kind| canonical_query_root_requirement(*kind) == CanonicalQueryRootRequirement::Required)
            .collect::<BTreeSet<_>>();
        let expected = BTreeSet::from([QueryKind::DeclarationShell, QueryKind::CallableSignature, QueryKind::DeclarationSurface]);
        assert_eq!(required, expected);

        let optional = QueryKind::ALL
            .into_iter()
            .filter(|kind| canonical_query_root_requirement(*kind) == CanonicalQueryRootRequirement::Optional)
            .collect::<BTreeSet<_>>();
        let expected_optional = BTreeSet::from([
            QueryKind::UnlinkedInterface,
            QueryKind::ResolvedImports,
            QueryKind::LinkedInterface,
            QueryKind::CallableDefinition,
            QueryKind::FieldSignature,
            QueryKind::TraitHeader,
            QueryKind::TraitSurface,
            QueryKind::DataDeclaration,
            QueryKind::EnumDeclaration,
            QueryKind::AssociatedSurface,
            QueryKind::TypeMemberDeclaration,
            QueryKind::ModuleStaticBindingCatalog,
            QueryKind::ModuleStaticBindings,
        ]);
        assert_eq!(optional, expected_optional);
        for kind in QueryKind::ALL {
            let requirement = canonical_query_root_requirement(kind);
            assert_eq!(
                requirement != CanonicalQueryRootRequirement::NotReusable,
                query_root_is_reusable(kind),
                "{kind:?}"
            );
            if requirement == CanonicalQueryRootRequirement::Required {
                assert_eq!(canonical_query_persistence(kind), CanonicalQueryPersistence::EligibleStructural);
            }
        }
    }

    #[test]
    fn product_location_and_persistence_both_gate_reusable_roots() {
        for kind in QueryKind::ALL {
            let expected = canonical_query_persistence(kind) == CanonicalQueryPersistence::EligibleStructural
                && matches!(
                    query_kind_profile_support(kind),
                    QueryProfileSupport::RepresentedByModuleWorld | QueryProfileSupport::RepresentedBySemanticWorld
                );
            assert_eq!(query_root_is_reusable(kind), expected, "{kind:?}");
        }
        assert!(!query_root_is_reusable(QueryKind::ParsedModule));
        assert!(!query_root_is_reusable(QueryKind::HierarchyEdge));
    }
}
