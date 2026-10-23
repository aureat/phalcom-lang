//! Ownership-scoped artifact projection over a composed semantic view.
//!
//! A [`SemanticSnapshot`] produced by compiling a package against immutable
//! dependency artifacts is a *composed* view: it sees Universe and dependency
//! rows beside the package's own rows. That view is correct for compilation and
//! wrong as a serialization boundary. [`ArtifactProjectionScope`] names the
//! modules whose primary authority an artifact owns, and
//! [`ArtifactSemanticProjection`] exposes only the rows of a snapshot that
//! those modules own.
//!
//! Ownership is always read from canonical native identity (the `ModuleId`
//! embedded in a declaration, callable, impl, variant, field, symbol, or
//! static domain). It is never inferred from display names, stable string
//! prefixes, type shape, source presence, or the identity of a referenced
//! target. In particular a locally owned impl whose target is a dependency
//! declaration is locally owned: ownership follows the contribution, not the
//! target.
//!
//! # Ownership inventory
//!
//! | `SemanticSnapshot` field | Primary owner identity | Projection rule | External refs? |
//! |---|---|---|---|
//! | `declarations` | `DeclarationId::module` | filter | yes (forms) |
//! | `type_aliases` | `DeclarationId::module` | filter | yes |
//! | `hierarchy` (superclass templates) | subclass `DeclarationId::module` | filter | yes (superclass) |
//! | `data_semantics.data_decls` | `DeclarationId::module` | filter | yes |
//! | `enum_semantics.enums` | `DeclarationId::module` | filter | yes |
//! | `enum_semantics.variants` | `VariantId::owner.module` | filter | yes |
//! | `enum_requirements` | owner `DeclarationId::module` | filter | yes |
//! | `callable_signatures` | `CallableId::module()` | filter | yes |
//! | `callable_definitions` | `CallableId::module()` | filter | yes |
//! | `callable_analyses` | `CallableId::module()` | filter | yes |
//! | `constructor_initializations` | factory `CallableId::module()` | filter | yes |
//! | `field_signatures` | `FieldId::owner.module` | filter | yes |
//! | `surfaces` | `DeclarationId::module` | filter | yes |
//! | `trait_headers` / `trait_surfaces` | `DeclarationId::module` | filter | yes |
//! | `conformance_index` | `ImplId::module` | filter | yes (trait, target) |
//! | `conformance_witness_plans` | `ImplId::module` | filter | yes |
//! | `derived_conformance_rules` | rule `ImplId::module` | filter | yes |
//! | `dispatch.conditional_members` | member `ImplId::module` | filter contributions, rebuild buckets | yes (target) |
//! | `trait_dispatch_index` | — | derived; rebuild from owned conformances | yes |
//! | `type_member_declarations` | `TypeMemberDeclarationId::owner.module` | filter | yes |
//! | `associated_static_domains` | owner `DeclarationId::module` | filter | yes |
//! | `associated_surfaces` | `DeclarationId::module` | filter | yes |
//! | `static_case_catalogs` | domain owner `DeclarationId::module` | filter | yes |
//! | `static_case_signatures` | case domain owner `DeclarationId::module` | filter | yes |
//! | `static_definitions` | `DeclarationId::module` | filter | yes |
//! | `static_constants` | `SymbolId::module` | filter | yes |
//! | `module_static_bindings` | `ModuleId` | filter | yes |
//! | `store` / `static_values` | — | shared lineage; closed from owned roots only | — |
//! | `sources`, `presentation_sources`, `source_index`, `advisory`, `diagnostics`, `semantic_graph`, `formal_projection`, `semantic_structure_shards`, `import_phase_products`, `module_products` | — | workspace/session state, not projected here | — |
use crate::identity::{
    AssociatedStaticDomainDeclarationId, CallableId, DeclarationId, FieldId, ImplId, ModuleId, StaticCaseId, StaticDomainDeclarationId, VariantId,
};
use crate::db::QueryKey;
use crate::snapshot::SemanticSnapshot;
use crate::static_constant::StaticConstantId;
use crate::types::type_member::TypeMemberDeclarationId;
use std::collections::BTreeSet;

/// The exact set of modules whose primary semantic authority one artifact owns.
///
/// Owner checks are a single set lookup on the module embedded in a native
/// identity. The whole-image scope exists only for the canonical Universe
/// artifact, whose snapshot is by construction exactly its owned world.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactProjectionScope {
    owned_modules: Option<BTreeSet<ModuleId>>,
}

impl ArtifactProjectionScope {
    /// Scope owning exactly `modules`, typically every module of one package project.
    pub fn new(modules: impl IntoIterator<Item = ModuleId>) -> Self {
        Self {
            owned_modules: Some(modules.into_iter().collect()),
        }
    }

    /// Scope owning every row of the snapshot it projects.
    ///
    /// Valid only when the snapshot is itself the artifact-owned world, as for
    /// the canonical Universe image. Never use it for a package compiled
    /// against dependency artifacts.
    pub fn whole_image() -> Self {
        Self { owned_modules: None }
    }

    /// Whether this scope is the whole-image Universe convenience scope.
    pub fn is_whole_image(&self) -> bool {
        self.owned_modules.is_none()
    }

    /// The explicitly owned modules, or `None` for the whole-image scope.
    pub fn owned_modules(&self) -> Option<&BTreeSet<ModuleId>> {
        self.owned_modules.as_ref()
    }

    /// Whether `module` belongs to this artifact.
    pub fn owns_module(&self, module: &ModuleId) -> bool {
        self.owned_modules.as_ref().is_none_or(|owned| owned.contains(module))
    }

    /// Whether the declaration's defining module belongs to this artifact.
    pub fn owns_declaration(&self, declaration: &DeclarationId) -> bool {
        self.owns_module(&declaration.module)
    }

    /// Whether the callable's owning module belongs to this artifact.
    pub fn owns_callable(&self, callable: &CallableId) -> bool {
        self.owns_module(callable.module())
    }

    /// Whether the impl's defining module belongs to this artifact, regardless of its target.
    pub fn owns_impl(&self, implementation: &ImplId) -> bool {
        self.owns_module(&implementation.module)
    }

    /// Whether the variant's owning enum belongs to this artifact.
    pub fn owns_variant(&self, variant: &VariantId) -> bool {
        self.owns_declaration(&variant.owner)
    }

    /// Whether the field's owning declaration belongs to this artifact.
    pub fn owns_field(&self, field: &FieldId) -> bool {
        self.owns_declaration(&field.owner)
    }

    /// Whether the type member's owning declaration belongs to this artifact.
    pub fn owns_type_member(&self, member: &TypeMemberDeclarationId) -> bool {
        self.owns_declaration(&member.owner)
    }

    /// Whether the associated static domain's owning declaration belongs to this artifact.
    pub fn owns_associated_static_domain(&self, domain: &AssociatedStaticDomainDeclarationId) -> bool {
        self.owns_declaration(&domain.owner)
    }

    /// Whether the static domain's owning declaration belongs to this artifact.
    pub fn owns_static_domain(&self, domain: &StaticDomainDeclarationId) -> bool {
        match domain {
            StaticDomainDeclarationId::Nominal(declaration) => self.owns_declaration(declaration),
            StaticDomainDeclarationId::Associated(associated) => self.owns_associated_static_domain(associated),
        }
    }

    /// Whether the static case's domain belongs to this artifact.
    pub fn owns_static_case(&self, case: &StaticCaseId) -> bool {
        self.owns_static_domain(&case.domain)
    }

    /// Whether the static constant's defining module belongs to this artifact.
    pub fn owns_static_constant(&self, constant: &StaticConstantId) -> bool {
        self.owns_module(&constant.0.module)
    }

    /// Whether a query root belongs to this artifact.
    ///
    /// Queries without a single owning module (target-indexed dispatch buckets,
    /// generic conformance evidence) are derived global products: no package
    /// artifact owns them, though the whole-image scope still does.
    pub fn owns_query(&self, key: &QueryKey) -> bool {
        match query_owner_module(key) {
            Some(module) => self.owns_module(module),
            None => self.is_whole_image(),
        }
    }
}

/// The single module that owns `key`'s product, or `None` for derived global queries.
pub fn query_owner_module(key: &QueryKey) -> Option<&ModuleId> {
    use crate::attribute_analysis::AttributeOwnerId;
    use QueryKey as Q;
    Some(match key {
        Q::ParsedModule(module)
        | Q::UnlinkedInterface(module)
        | Q::ResolvedImports(module)
        | Q::LinkedInterface(module)
        | Q::SemanticComponent(module)
        | Q::SourceStructure(module)
        | Q::AdvisoryModule(module)
        | Q::ModuleDiagnostics(module)
        | Q::ModuleMetadata(module)
        | Q::LinkedName(module, _)
        | Q::PublicExport(module, _)
        | Q::ModuleStaticBindingCatalog(module)
        | Q::ModuleStaticBindings(module) => module,
        Q::DeclarationShell(declaration)
        | Q::DeclarationSurface(declaration)
        | Q::TraitHeader(declaration)
        | Q::TraitSurface(declaration)
        | Q::HierarchyEdge(declaration)
        | Q::ConstructorInitialization(declaration)
        | Q::DataDeclaration(declaration)
        | Q::EnumDeclaration(declaration)
        | Q::EnumRequirements(declaration)
        | Q::AssociatedSurface(declaration)
        | Q::StaticDefinition(declaration) => &declaration.module,
        Q::CheckedStaticBody(body) => &body.0.module,
        Q::ConformanceDispatch(implementation) => &implementation.module,
        Q::CallableDefinition(callable)
        | Q::CallableSignature(callable)
        | Q::CallableBody(callable)
        | Q::CallableEffects(callable)
        | Q::CallableControl(callable)
        | Q::CallableTermination(callable)
        | Q::CallableContracts(callable)
        | Q::VerificationConditions(callable)
        | Q::SourceFormalAttachment(callable)
        | Q::AdvisoryCallable(callable) => callable.module(),
        Q::FieldSignature(field) => &field.owner.module,
        Q::TypeMemberDeclaration(member) => &member.owner.module,
        Q::StaticDomainCaseCatalog(domain) => match domain {
            StaticDomainDeclarationId::Nominal(declaration) => &declaration.module,
            StaticDomainDeclarationId::Associated(associated) => &associated.owner.module,
        },
        Q::StaticCaseSignature(case) => match &case.domain {
            StaticDomainDeclarationId::Nominal(declaration) => &declaration.module,
            StaticDomainDeclarationId::Associated(associated) => &associated.owner.module,
        },
        Q::AssociatedStaticDomainDeclaration(domain) => &domain.owner.module,
        Q::StaticConstant(constant) => &constant.0.module,
        Q::ResolvedImport(site) => &site.importer,
        Q::AttributeArgument(argument) => match &argument.owner {
            AttributeOwnerId::Module(module) => module,
            AttributeOwnerId::Declaration(declaration) => &declaration.module,
            AttributeOwnerId::Callable(callable) => callable.module(),
            AttributeOwnerId::Field(field) => &field.owner.module,
        },
        Q::TraitDispatchBucket(_) | Q::GenericConformance(_) => return None,
    })
}

/// Read-only export view of the rows of one snapshot owned by one scope.
///
/// The snapshot is never mutated or rebuilt; the projection only decides
/// which primary rows enter the artifact. Rows it yields may still reference
/// dependency-owned identities, which the exporter preserves as references.
#[derive(Clone, Copy, Debug)]
pub struct ArtifactSemanticProjection<'a> {
    snapshot: &'a SemanticSnapshot,
    scope: &'a ArtifactProjectionScope,
}

impl<'a> ArtifactSemanticProjection<'a> {
    /// Projects `snapshot` through `scope`.
    pub fn new(snapshot: &'a SemanticSnapshot, scope: &'a ArtifactProjectionScope) -> Self {
        Self { snapshot, scope }
    }

    /// The composed snapshot being projected.
    pub fn snapshot(&self) -> &'a SemanticSnapshot {
        self.snapshot
    }

    /// The ownership scope applied to the snapshot.
    pub fn scope(&self) -> &'a ArtifactProjectionScope {
        self.scope
    }
}

#[cfg(test)]
mod tests {
    use super::ArtifactProjectionScope;
    use crate::identity::{DeclarationId, ImplId, ImplLocalId, ModuleId, ModulePath, ResolvedProjectId};

    #[test]
    fn impl_ownership_follows_contribution_module_not_target() {
        let local = ModuleId::resolved(ResolvedProjectId::from_raw(8), ModulePath::root());
        let scope = ArtifactProjectionScope::new([local.clone()]);
        let external_target = DeclarationId::new(ModuleId::universe_root(), "Int".into());
        assert!(!scope.owns_declaration(&external_target));
        assert!(scope.owns_impl(&ImplId::new(local, ImplLocalId(0))));
        assert!(!scope.owns_impl(&ImplId::new(ModuleId::universe_root(), ImplLocalId(0))));
    }

    #[test]
    fn whole_image_scope_owns_every_module() {
        let scope = ArtifactProjectionScope::whole_image();
        assert!(scope.is_whole_image());
        assert!(scope.owns_module(&ModuleId::universe_root()));
    }
}
