//! Immutable semantic session seed composed from multiple artifact generations.
//!
//! Query products retain their original generation/provenance. This module only
//! composes the semantic aggregate tables required to start a mutable workspace
//! over a closed external artifact base. Every row is admitted by exact module
//! ownership and duplicate authority is rejected even when payloads compare
//! equal.

use crate::associated::AssociatedFamilyTable;
use crate::canonical_image::{CanonicalImageError, ExternalSemanticQueryCatalogue};
use crate::data_semantics::DataSemanticTable;
use crate::declarations::DeclarationTypeTable;
use crate::dispatch::SurfaceDispatchResolver;
use crate::enum_requirements::{EnumRequirementRows, EnumRequirementTable};
use crate::enum_semantics::EnumSemanticTable;
use crate::identity::{ModuleId, StaticDomainDeclarationId};
use crate::impls::{ConformanceIndex, ConformanceWitnessPlans};
use crate::signature::{CallableSignatureTable, ConstructorInitializationTable, FieldSignatureTable};
use crate::static_constant::StaticConstantTable;
use crate::static_definition::StaticDefinitionTable;
use crate::static_value::StaticValueStore;
use crate::traits::{TraitHeaderTable, TraitSurfaceTable};
use crate::type_alias::TypeAliasTable;
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_modules::{ExternalModuleAuthority, ExternalModuleAuthoritySet};
use std::sync::Arc;

fn incomplete(message: impl Into<String>) -> CanonicalImageError {
    CanonicalImageError::IncompletePublication(message.into())
}

fn validate_owner(
    modules: &ExternalModuleAuthoritySet,
    generation: usize,
    module: &ModuleId,
    family: &'static str,
) -> Result<(), CanonicalImageError> {
    let Some(origin) = modules.origin(module) else {
        return Err(incomplete(format!("{family} row is owned by a module outside the external artifact base: {module}")));
    };
    if origin.index() != generation {
        return Err(incomplete(format!(
            "{family} row is published by immutable generation {generation} but module {module} belongs to generation {}",
            origin.index()
        )));
    }
    Ok(())
}

fn static_domain_module(domain: &StaticDomainDeclarationId) -> &ModuleId {
    match domain {
        StaticDomainDeclarationId::Nominal(owner) => &owner.module,
        StaticDomainDeclarationId::Associated(owner) => &owner.owner.module,
    }
}

fn shared_rows<T: Clone>(rows: &EnumRequirementRows<T>) -> Arc<[T]> {
    Arc::from(rows.as_ref().to_vec().into_boxed_slice())
}

/// Exact aggregate authority used to seed a mutable semantic workspace.
///
/// The original artifact snapshots remain independently owned by
/// `ExternalSemanticQueryCatalogue`; these tables are a checked composed view,
/// not a synthetic artifact generation.
#[derive(Debug)]
pub struct ExternalSemanticSessionSeed {
    modules: Arc<ExternalModuleAuthoritySet>,
    queries: Arc<ExternalSemanticQueryCatalogue>,
    store: Arc<TypeStore>,
    static_values: Arc<StaticValueStore>,

    pub(crate) declarations: Arc<DeclarationTypeTable>,
    pub(crate) hierarchy: Arc<MapTypeHierarchy>,
    pub(crate) dispatch: Arc<SurfaceDispatchResolver>,
    pub(crate) callable_signatures: Arc<CallableSignatureTable>,
    pub(crate) constructor_initializations: Arc<ConstructorInitializationTable>,
    pub(crate) callable_definitions:
        Arc<crate::parent_map::ParentMap<crate::identity::CallableId, crate::impls::EffectiveCallableDefinition>>,
    pub(crate) conformance_index: Arc<ConformanceIndex>,
    pub(crate) conformance_witness_plans: Arc<ConformanceWitnessPlans>,
    pub(crate) derived_conformance_rules: Arc<crate::derived_conformance::DerivedConformanceRuleIndex>,
    pub(crate) field_signatures: Arc<FieldSignatureTable>,
    pub(crate) type_aliases: Arc<TypeAliasTable>,
    pub(crate) trait_headers: Arc<TraitHeaderTable>,
    pub(crate) trait_surfaces: Arc<TraitSurfaceTable>,
    pub(crate) data_semantics: Arc<DataSemanticTable>,
    pub(crate) enum_semantics: Arc<EnumSemanticTable>,
    pub(crate) enum_requirements: Arc<EnumRequirementTable>,
    pub(crate) associated_surfaces: Arc<AssociatedFamilyTable>,
    pub(crate) type_member_declarations:
        Arc<crate::parent_map::ParentMap<crate::types::type_member::TypeMemberDeclarationId, Arc<crate::db::product::TypeMemberDeclarationProduct>>>,
    pub(crate) associated_static_domains: Arc<
        crate::ordered_map::OrderedMap<
            crate::identity::AssociatedStaticDomainDeclarationId,
            Arc<crate::db::product::AssociatedStaticDomainDeclarationProduct>,
        >,
    >,
    pub(crate) static_case_catalogs: Arc<
        crate::ordered_map::OrderedMap<
            crate::identity::StaticDomainDeclarationId,
            Arc<crate::static_case::StaticDomainCaseCatalogProduct>,
        >,
    >,
    pub(crate) static_case_signatures:
        Arc<crate::ordered_map::OrderedMap<crate::identity::StaticCaseId, Arc<crate::static_case::StaticCaseSignatureProduct>>>,
    pub(crate) static_definitions: Arc<StaticDefinitionTable>,
    pub(crate) static_constants: Arc<StaticConstantTable>,
    pub(crate) module_static_bindings:
        Arc<crate::parent_map::ParentMap<ModuleId, Arc<crate::module_bindings::ModuleStaticBindingIndex>>>,
}

impl ExternalSemanticSessionSeed {
    /// Composes exact semantic aggregate authority from already validated
    /// immutable artifact generations.
    ///
    /// Query images are re-bound to the supplied final shared stores before
    /// any aggregate table is published. Module ownership and generation order
    /// must agree exactly with `ExternalModuleAuthoritySet`.
    pub fn from_validated_authorities(
        modules: Arc<ExternalModuleAuthoritySet>,
        queries: Arc<ExternalSemanticQueryCatalogue>,
        store: Arc<TypeStore>,
        static_values: Arc<StaticValueStore>,
    ) -> Result<Self, CanonicalImageError> {
        let generation_count = queries.generations().len();
        for module in modules.module_ids() {
            let origin = modules
                .origin(module)
                .ok_or_else(|| incomplete(format!("external module has no composed artifact origin: {module}")))?;
            if origin.index() >= generation_count {
                return Err(incomplete(format!(
                    "external module {module} belongs to generation {} but only {generation_count} semantic generations exist",
                    origin.index()
                )));
            }
        }

        let mut declarations = DeclarationTypeTable::new();
        let mut hierarchy = MapTypeHierarchy::new();
        let mut dispatch = SurfaceDispatchResolver::new();
        let mut callable_signatures = CallableSignatureTable::new();
        let mut constructor_initializations = ConstructorInitializationTable::default();
        let mut callable_definitions = crate::parent_map::ParentMap::default();
        let mut conformance_index = ConformanceIndex::new();
        let mut conformance_witness_plans = ConformanceWitnessPlans::default();
        let mut derived_conformance_rules = crate::derived_conformance::DerivedConformanceRuleIndex::new();
        let mut field_signatures = FieldSignatureTable::new();
        let mut type_aliases = TypeAliasTable::new();
        let mut trait_headers = TraitHeaderTable::new();
        let mut trait_surfaces = TraitSurfaceTable::new();
        let mut data_semantics = DataSemanticTable::new();
        let mut enum_semantics = EnumSemanticTable::new();
        let mut enum_requirements = EnumRequirementTable::new();
        let mut associated_surfaces = AssociatedFamilyTable::new();
        let mut type_member_declarations = crate::parent_map::ParentMap::default();
        let mut associated_static_domains = crate::ordered_map::OrderedMap::default();
        let mut static_case_catalogs = crate::ordered_map::OrderedMap::default();
        let mut static_case_signatures = crate::ordered_map::OrderedMap::default();
        let mut static_definitions = StaticDefinitionTable::new();
        let mut static_constants = StaticConstantTable::new();
        let mut module_static_bindings = crate::parent_map::ParentMap::default();

        for (generation_index, generation) in queries.generations().iter().enumerate() {
            generation
                .queries()
                .reader(generation.world(), store.as_ref(), static_values.as_ref())?;
            let snapshot = generation.snapshot();

            for (owner, info) in snapshot.declarations.iter() {
                validate_owner(&modules, generation_index, &owner.module, "declaration")?;
                if declarations.contains(owner) {
                    return Err(incomplete(format!("declaration is owned by multiple immutable artifact generations: {owner:?}")));
                }
                declarations.insert(info.clone());

                if let Some(template) = snapshot.hierarchy.supertype_template(owner) {
                    if hierarchy.supertype_template(owner).is_some() {
                        return Err(incomplete(format!("supertype template is owned by multiple immutable artifact generations: {owner:?}")));
                    }
                    hierarchy.insert_template(template.clone());
                }

            }

            for declaration in snapshot.declarations.intrinsic_static_domain_declarations() {
                validate_owner(&modules, generation_index, &declaration.module, "intrinsic static-domain")?;
                let kind = snapshot
                    .declarations
                    .intrinsic_static_domain_kind(declaration)
                    .ok_or_else(|| incomplete("intrinsic static-domain owner has no kind"))?;
                if declarations.intrinsic_static_domain_kind(declaration).is_some() {
                    return Err(incomplete(format!(
                        "intrinsic static-domain authority is owned by multiple immutable artifact generations: {:?}",
                        declaration
                    )));
                }
                declarations.register_imported_intrinsic_static_domain(declaration.clone(), kind);
            }
            for declaration in snapshot.declarations.intrinsic_type_owner_declarations() {
                validate_owner(&modules, generation_index, &declaration.module, "intrinsic type-owner")?;
                let owner = snapshot
                    .declarations
                    .intrinsic_type_owner(declaration)
                    .ok_or_else(|| incomplete("intrinsic type-owner declaration has no owner"))?;
                if let Some(existing) = declarations.intrinsic_type_owner(declaration) {
                    if existing != owner {
                        return Err(incomplete(format!(
                            "intrinsic type-owner authority disagrees for immutable declaration {:?}: {:?} versus {:?}",
                            declaration, existing, owner
                        )));
                    }
                } else {
                    declarations.register_intrinsic_type_owner(declaration.clone(), owner);
                }
            }

            for (owner, superclass) in snapshot.hierarchy.iter_superclasses() {
                validate_owner(&modules, generation_index, &owner.module, "hierarchy")?;
                if hierarchy.superclass(owner).is_some() {
                    return Err(incomplete(format!("hierarchy edge is owned by multiple immutable artifact generations: {owner:?}")));
                }
                hierarchy.insert(owner.clone(), superclass.clone());
            }

            for (ty, owner) in snapshot.dispatch.type_declarations().iter() {
                validate_owner(&modules, generation_index, &owner.module, "dispatch type registration")?;
                if let Some(existing) = dispatch.declaration_for_type(*ty) {
                    return Err(incomplete(format!(
                        "dispatch type registration is owned by multiple immutable artifact generations: {:?} -> {:?} (already {:?})",
                        ty, owner, existing
                    )));
                }
                dispatch.register_type(*ty, owner.clone());
            }
            for (impl_id, domain) in snapshot.dispatch.inherent_impl_domains().iter() {
                validate_owner(&modules, generation_index, &impl_id.module, "inherent implementation domain")?;
                if dispatch.inherent_impl_domain(impl_id).is_some() {
                    return Err(incomplete(format!(
                        "inherent implementation domain is owned by multiple immutable artifact generations: {:?}",
                        impl_id
                    )));
                }
                dispatch.register_inherent_impl_domain(impl_id.clone(), Arc::clone(domain));
            }

            for (owner, surface) in snapshot.dispatch.surfaces().iter() {
                validate_owner(&modules, generation_index, &owner.module, "dispatch surface")?;
                if dispatch.get_surface(owner).is_some() {
                    return Err(incomplete(format!("dispatch surface is owned by multiple immutable artifact generations: {owner:?}")));
                }
                dispatch.register_surface(owner.clone(), surface.clone());
            }
            for (target, members) in snapshot.dispatch.conditional_members().iter() {
                let module = &target.declaration().module;
                validate_owner(&modules, generation_index, module, "conditional inherent dispatch")?;
                if dispatch.get_conditional_members(target).is_some() {
                    return Err(incomplete(format!(
                        "conditional inherent dispatch target is owned by multiple immutable artifact generations: {:?}",
                        target
                    )));
                }
                dispatch.register_conditional_members(target.clone(), Arc::clone(members));
            }

            for (callable, signature) in snapshot.callable_signatures.iter() {
                validate_owner(&modules, generation_index, callable.module(), "callable signature")?;
                if callable_signatures.get(callable).is_some() {
                    return Err(incomplete(format!("callable signature is owned by multiple immutable artifact generations: {callable:?}")));
                }
                callable_signatures.insert(signature.clone());
            }
            for (factory, product) in snapshot.constructor_initializations.iter() {
                validate_owner(&modules, generation_index, factory.module(), "constructor initialization")?;
                if constructor_initializations.get(factory).is_some() {
                    return Err(incomplete(format!("constructor path is owned by multiple immutable artifact generations: {factory:?}")));
                }
                constructor_initializations.insert(product.clone());
            }
            for (callable, definition) in snapshot.callable_definitions.iter() {
                validate_owner(&modules, generation_index, callable.module(), "callable definition")?;
                if callable_definitions.get(callable).is_some() {
                    return Err(incomplete(format!("callable definition is owned by multiple immutable artifact generations: {callable:?}")));
                }
                callable_definitions.insert(callable.clone(), definition.clone());
            }
            for (field, signature) in snapshot.field_signatures.iter() {
                validate_owner(&modules, generation_index, &field.owner.module, "field signature")?;
                if field_signatures.get(field).is_some() {
                    return Err(incomplete(format!("field signature is owned by multiple immutable artifact generations: {:?}", field)));
                }
                field_signatures.insert(signature.clone());
            }

            for (owner, alias) in snapshot.type_aliases.iter() {
                validate_owner(&modules, generation_index, &owner.module, "type alias")?;
                if type_aliases.contains_key(owner) {
                    return Err(incomplete(format!("type alias is owned by multiple immutable artifact generations: {owner:?}")));
                }
                type_aliases.insert(alias.clone());
            }
            for (owner, header) in snapshot.trait_headers.iter() {
                validate_owner(&modules, generation_index, &owner.module, "trait header")?;
                if trait_headers.contains(owner) {
                    return Err(incomplete(format!("trait header is owned by multiple immutable artifact generations: {owner:?}")));
                }
                trait_headers.insert(Arc::clone(header));
            }
            for (owner, surface) in snapshot.trait_surfaces.iter() {
                validate_owner(&modules, generation_index, &owner.module, "trait surface")?;
                if trait_surfaces.contains(owner) {
                    return Err(incomplete(format!("trait surface is owned by multiple immutable artifact generations: {owner:?}")));
                }
                trait_surfaces.insert(Arc::clone(surface));
            }

            for (owner, info) in snapshot.data_semantics.data_decls.iter() {
                validate_owner(&modules, generation_index, &owner.module, "data semantics")?;
                if data_semantics.get(owner).is_some() {
                    return Err(incomplete(format!("data semantics are owned by multiple immutable artifact generations: {owner:?}")));
                }
                data_semantics.insert_data(Arc::clone(info));
            }
            for (owner, info) in snapshot.enum_semantics.enums.iter() {
                validate_owner(&modules, generation_index, &owner.module, "enum semantics")?;
                if enum_semantics.enum_info(owner).is_some() {
                    return Err(incomplete(format!("enum semantics are owned by multiple immutable artifact generations: {owner:?}")));
                }
                enum_semantics.insert_enum(Arc::clone(info));
            }
            for (variant, info) in snapshot.enum_semantics.variants.iter() {
                validate_owner(&modules, generation_index, &variant.owner.module, "enum variant semantics")?;
                if enum_semantics.variant_info(variant).is_some() {
                    return Err(incomplete(format!("enum variant is owned by multiple immutable artifact generations: {variant:?}")));
                }
                enum_semantics.insert_variant(Arc::clone(info));
            }
            for (owner, requirements) in snapshot.enum_requirements.requirements.iter() {
                validate_owner(&modules, generation_index, &owner.module, "enum requirements")?;
                if enum_requirements.requirements.contains_key(owner) {
                    return Err(incomplete(format!("enum requirements are owned by multiple immutable artifact generations: {owner:?}")));
                }
                let statuses = snapshot
                    .enum_requirements
                    .case_statuses
                    .get(owner)
                    .ok_or_else(|| incomplete(format!("enum requirement owner has no case-status rows: {owner:?}")))?;
                enum_requirements.insert(owner.clone(), shared_rows(requirements), shared_rows(statuses));
            }

            for (owner, surface) in snapshot.associated_surfaces.surfaces.iter() {
                validate_owner(&modules, generation_index, &owner.module, "associated surface")?;
                if associated_surfaces.surfaces.get(owner).is_some() {
                    return Err(incomplete(format!("associated surface is owned by multiple immutable artifact generations: {owner:?}")));
                }
                associated_surfaces.insert(owner.clone(), Arc::clone(surface));
            }
            for (member, product) in snapshot.type_member_declarations.iter() {
                validate_owner(&modules, generation_index, &member.owner.module, "type-member declaration")?;
                if type_member_declarations.get(member).is_some() {
                    return Err(incomplete(format!("type-member declaration is owned by multiple immutable artifact generations: {member:?}")));
                }
                type_member_declarations.insert(member.clone(), Arc::clone(product));
            }

            for (id, product) in snapshot.associated_static_domains.iter() {
                validate_owner(&modules, generation_index, &id.owner.module, "associated static domain")?;
                if associated_static_domains.contains_key(id) {
                    return Err(incomplete(format!("associated static domain is owned by multiple immutable artifact generations: {id:?}")));
                }
                associated_static_domains.insert(id.clone(), Arc::clone(product));
            }
            for (domain, product) in snapshot.static_case_catalogs.iter() {
                validate_owner(&modules, generation_index, static_domain_module(domain), "static case catalog")?;
                if static_case_catalogs.contains_key(domain) {
                    return Err(incomplete(format!("static case catalog is owned by multiple immutable artifact generations: {domain:?}")));
                }
                static_case_catalogs.insert(domain.clone(), Arc::clone(product));
            }
            for (case, product) in snapshot.static_case_signatures.iter() {
                validate_owner(&modules, generation_index, static_domain_module(&case.domain), "static case signature")?;
                if static_case_signatures.contains_key(case) {
                    return Err(incomplete(format!("static case signature is owned by multiple immutable artifact generations: {case:?}")));
                }
                static_case_signatures.insert(case.clone(), Arc::clone(product));
            }
            for (owner, product) in snapshot.static_definitions.iter() {
                validate_owner(&modules, generation_index, &owner.module, "static definition")?;
                if static_definitions.contains_key(owner) {
                    return Err(incomplete(format!("static definition is owned by multiple immutable artifact generations: {owner:?}")));
                }
                static_definitions.insert(Arc::clone(product));
            }
            for (id, product) in snapshot.static_constants.iter() {
                validate_owner(&modules, generation_index, id.module(), "static constant")?;
                if static_constants.get(id).is_some() {
                    return Err(incomplete(format!("static constant is owned by multiple immutable artifact generations: {id}")));
                }
                static_constants.insert(Arc::clone(product));
            }
            for (module, bindings) in snapshot.module_static_bindings.iter() {
                validate_owner(&modules, generation_index, module, "module static bindings")?;
                if module_static_bindings.get(module).is_some() {
                    return Err(incomplete(format!("module static bindings are owned by multiple immutable artifact generations: {module}")));
                }
                module_static_bindings.insert(module.clone(), Arc::clone(bindings));
            }

            for (impl_id, contribution) in snapshot.conformance_index.iter() {
                validate_owner(&modules, generation_index, &impl_id.module, "conformance")?;
                if conformance_index.get(impl_id).is_some() {
                    return Err(incomplete(format!("conformance is owned by multiple immutable artifact generations: {impl_id:?}")));
                }
                conformance_index.insert(contribution.clone());
            }
            for (impl_id, plan) in snapshot.conformance_witness_plans.iter() {
                validate_owner(&modules, generation_index, &impl_id.module, "conformance witness plan")?;
                if conformance_witness_plans.get(impl_id).is_some() {
                    return Err(incomplete(format!("conformance witness plan is owned by multiple immutable artifact generations: {impl_id:?}")));
                }
                conformance_witness_plans.insert(impl_id.clone(), Arc::clone(plan));
            }
            for rule in snapshot.derived_conformance_rules.iter() {
                validate_owner(&modules, generation_index, &rule.source_module, "derived conformance rule")?;
                if derived_conformance_rules.get_rule(&rule.impl_id).is_some() {
                    return Err(incomplete(format!(
                        "derived conformance rule is owned by multiple immutable artifact generations: {:?}",
                        rule.impl_id
                    )));
                }
                derived_conformance_rules.insert((**rule).clone());
            }
        }

        // Dispatch buckets are derived from the complete composed primary
        // conformance + trait surface authority. They are intentionally not
        // concatenated from generation-local precomputed indexes.
        let _trait_dispatch_index = crate::trait_dispatch::TraitDispatchIndex::build(&conformance_index, &trait_surfaces);

        Ok(Self {
            modules,
            queries,
            store,
            static_values,
            declarations: Arc::new(declarations),
            hierarchy: Arc::new(hierarchy),
            dispatch: Arc::new(dispatch),
            callable_signatures: Arc::new(callable_signatures),
            constructor_initializations: Arc::new(constructor_initializations),
            callable_definitions: Arc::new(callable_definitions),
            conformance_index: Arc::new(conformance_index),
            conformance_witness_plans: Arc::new(conformance_witness_plans),
            derived_conformance_rules: Arc::new(derived_conformance_rules),
            field_signatures: Arc::new(field_signatures),
            type_aliases: Arc::new(type_aliases),
            trait_headers: Arc::new(trait_headers),
            trait_surfaces: Arc::new(trait_surfaces),
            data_semantics: Arc::new(data_semantics),
            enum_semantics: Arc::new(enum_semantics),
            enum_requirements: Arc::new(enum_requirements),
            associated_surfaces: Arc::new(associated_surfaces),
            type_member_declarations: Arc::new(type_member_declarations),
            associated_static_domains: Arc::new(associated_static_domains),
            static_case_catalogs: Arc::new(static_case_catalogs),
            static_case_signatures: Arc::new(static_case_signatures),
            static_definitions: Arc::new(static_definitions),
            static_constants: Arc::new(static_constants),
            module_static_bindings: Arc::new(module_static_bindings),
        })
    }

    pub fn modules(&self) -> &Arc<ExternalModuleAuthoritySet> {
        &self.modules
    }

    pub fn queries(&self) -> &Arc<ExternalSemanticQueryCatalogue> {
        &self.queries
    }

    pub fn store(&self) -> &Arc<TypeStore> {
        &self.store
    }

    pub fn static_values(&self) -> &Arc<StaticValueStore> {
        &self.static_values
    }
}
