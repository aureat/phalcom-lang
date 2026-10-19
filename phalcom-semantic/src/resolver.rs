//! Linked project-aware type resolver.

use crate::identity::{DeclarationId, ModuleId};
use crate::ordered_map::OrderedMap;
use crate::prelude::PreludeTypeMap;
use crate::types::annotation::TypeResolver;
use phalcom_modules::linker::{LinkedProgram, LinkedReadSpec};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, Default)]
struct AssociatedTypeMemberProducts {
    surfaces: Arc<crate::parent_map::ParentMap<DeclarationId, Arc<crate::associated::AssociatedSurface>>>,
    declarations: crate::parent_map::ParentMap<crate::types::type_member::TypeMemberDeclarationId, Arc<crate::db::product::TypeMemberDeclarationProduct>>,
}

/// A project-aware type resolver that resolves type names through the linker layout
/// and known declaration tables without resorting to global heuristics.
#[derive(Clone, Debug)]
pub struct LinkedTypeResolver {
    linked: Arc<LinkedProgram>,
    canonical_linked: Option<Arc<LinkedProgram>>,
    known_declarations: HashSet<DeclarationId>,
    prelude_types: Arc<PreludeTypeMap>,
    retained_alias_forms: Arc<crate::type_alias::TypeAliasTable>,
    alias_form_overrides: RefCell<BTreeMap<DeclarationId, Option<crate::types::id::TypeId>>>,
    trait_associated_types: Arc<BTreeMap<DeclarationId, BTreeMap<String, crate::traits::AssociatedTypeRequirementId>>>,
    associated_type_members: Arc<RwLock<AssociatedTypeMemberProducts>>,
    static_case_catalogs: Arc<RwLock<OrderedMap<crate::identity::StaticDomainDeclarationId, Arc<crate::static_case::StaticDomainCaseCatalogProduct>>>>,
    static_case_signatures: Arc<RwLock<OrderedMap<crate::identity::StaticCaseId, Arc<crate::static_case::StaticCaseSignatureProduct>>>>,
    static_definitions: Arc<RwLock<crate::static_definition::StaticDefinitionTable>>,
}

impl LinkedTypeResolver {
    /// Constructs a resolver with the canonical source-backed Universe prelude.
    ///
    /// The third parameter is retained temporarily for source compatibility
    /// with session/bootstrap call sites that historically supplied a fake
    /// prelude module. It is deliberately ignored: prelude identity is now
    /// represented by `PreludeTypeMap`, never by synthetic declarations in a
    /// chosen module.
    pub fn new(linked: Arc<LinkedProgram>, known_declarations: HashSet<DeclarationId>, _legacy_prelude_module: ModuleId) -> Self {
        Self::with_prelude(linked, known_declarations, PreludeTypeMap::shared_canonical_universe())
    }

    /// Constructs a resolver with an explicitly shared prelude map.
    pub fn with_prelude(linked: Arc<LinkedProgram>, known_declarations: HashSet<DeclarationId>, prelude_types: Arc<PreludeTypeMap>) -> Self {
        Self {
            linked,
            canonical_linked: None,
            known_declarations,
            prelude_types,
            retained_alias_forms: Arc::new(crate::type_alias::TypeAliasTable::new()),
            alias_form_overrides: RefCell::new(BTreeMap::new()),
            trait_associated_types: Arc::new(BTreeMap::new()),
            associated_type_members: Arc::new(RwLock::new(AssociatedTypeMemberProducts::default())),
            static_case_catalogs: Arc::new(RwLock::new(OrderedMap::default())),
            static_case_signatures: Arc::new(RwLock::new(OrderedMap::default())),
            static_definitions: Arc::new(RwLock::new(crate::static_definition::StaticDefinitionTable::new())),
        }
    }

    /// Constructs a resolver with alias forms retained from the previous
    /// semantic revision. Changed aliases are applied through the small
    /// override map, so an ordinary non-alias edit does not rebuild the full
    /// alias table just to make the resolver usable.
    pub fn with_retained_alias_forms(
        linked: Arc<LinkedProgram>,
        known_declarations: HashSet<DeclarationId>,
        _legacy_prelude_module: ModuleId,
        retained_alias_forms: Arc<crate::type_alias::TypeAliasTable>,
    ) -> Self {
        Self {
            linked,
            canonical_linked: None,
            known_declarations,
            prelude_types: PreludeTypeMap::shared_canonical_universe(),
            retained_alias_forms,
            alias_form_overrides: RefCell::new(BTreeMap::new()),
            trait_associated_types: Arc::new(BTreeMap::new()),
            associated_type_members: Arc::new(RwLock::new(AssociatedTypeMemberProducts::default())),
            static_case_catalogs: Arc::new(RwLock::new(OrderedMap::default())),
            static_case_signatures: Arc::new(RwLock::new(OrderedMap::default())),
            static_definitions: Arc::new(RwLock::new(crate::static_definition::StaticDefinitionTable::new())),
        }
    }

    pub fn with_canonical_linked(mut self, canonical_linked: Arc<LinkedProgram>) -> Self {
        self.canonical_linked = Some(canonical_linked);
        self
    }

    fn linked_module(&self, module: &ModuleId) -> Option<&phalcom_modules::linker::LinkedModule> {
        self.linked
            .modules
            .get(module)
            .or_else(|| self.canonical_linked.as_ref().and_then(|canonical| canonical.modules.get(module)))
    }

    pub fn with_trait_associated_types(
        mut self,
        trait_associated_types: Arc<BTreeMap<DeclarationId, BTreeMap<String, crate::traits::AssociatedTypeRequirementId>>>,
    ) -> Self {
        self.trait_associated_types = trait_associated_types;
        self
    }

    pub fn prelude_types(&self) -> &Arc<PreludeTypeMap> {
        &self.prelude_types
    }

    pub fn insert_alias_form(&self, declaration: DeclarationId, form: crate::types::id::TypeId) {
        self.alias_form_overrides.borrow_mut().insert(declaration, Some(form));
    }

    pub fn remove_alias_form(&self, declaration: DeclarationId) {
        self.alias_form_overrides.borrow_mut().insert(declaration, None);
    }

    /// Replaces the associated member query view after declaration surfaces and
    /// their checked member products have been published for this snapshot.
    pub fn update_associated_type_members(
        &self,
        surfaces: &crate::associated::AssociatedFamilyTable,
        declarations: &crate::parent_map::ParentMap<crate::types::type_member::TypeMemberDeclarationId, Arc<crate::db::product::TypeMemberDeclarationProduct>>,
    ) {
        if let Ok(mut products) = self.associated_type_members.write() {
            products.surfaces = surfaces.surfaces.clone();
            products.declarations = declarations.clone();
        }
    }

    pub fn update_static_case_catalogs(
        &self,
        catalogs: &OrderedMap<crate::identity::StaticDomainDeclarationId, Arc<crate::static_case::StaticDomainCaseCatalogProduct>>,
    ) {
        if let Ok(mut current) = self.static_case_catalogs.write() {
            *current = catalogs.clone();
        }
    }

    pub fn update_static_case_signatures(&self, signatures: &OrderedMap<crate::identity::StaticCaseId, Arc<crate::static_case::StaticCaseSignatureProduct>>) {
        if let Ok(mut current) = self.static_case_signatures.write() {
            *current = signatures.clone();
        }
    }

    /// Publishes the snapshot's exact static-definition signature products to
    /// type-form resolution. Checked body products may be composed later; the
    /// stable declaration and signature are already final at this stage.
    pub fn update_static_definitions(&self, definitions: &crate::static_definition::StaticDefinitionTable) {
        if let Ok(mut current) = self.static_definitions.write() {
            *current = definitions.clone();
        }
    }
}

impl TypeResolver for LinkedTypeResolver {
    fn resolve_static_definition(&self, declaration: &DeclarationId) -> Option<Arc<crate::static_definition::StaticDefinitionProduct>> {
        self.static_definitions.read().ok()?.get(declaration).cloned()
    }

    fn resolve_associated_static_domain(&self, owner: &DeclarationId, name: &str) -> Option<crate::identity::AssociatedStaticDomainDeclarationId> {
        let products = self.associated_type_members.read().ok()?;
        let family = products
            .surfaces
            .get(owner)?
            .families
            .get(&phalcom_common::selector::SelectorBase::Named(name.to_string()))?;
        if family.kind != crate::associated::AssociatedFamilyKind::StaticDomain {
            return None;
        }
        let crate::associated::AssociatedMemberId::StaticDomain(id) = family.members.first()? else {
            return None;
        };
        Some(id.clone())
    }

    fn resolve_static_case_catalog(
        &self,
        domain: &crate::identity::StaticDomainDeclarationId,
    ) -> Option<Arc<crate::static_case::StaticDomainCaseCatalogProduct>> {
        self.static_case_catalogs.read().ok()?.get(domain).cloned()
    }

    fn resolve_static_case_signature(&self, case: &crate::identity::StaticCaseId) -> Option<Arc<crate::static_case::StaticCaseSignatureProduct>> {
        self.static_case_signatures.read().ok()?.get(case).cloned()
    }
    fn resolve_associated_type_member(
        &self,
        owner: &DeclarationId,
        selector: &crate::types::type_member::TypeMemberSelector,
    ) -> Option<Arc<crate::db::product::TypeMemberDeclarationProduct>> {
        let id = self.resolve_associated_type_member_id(owner, selector)?;
        self.associated_type_members.read().ok()?.declarations.get(&id).cloned()
    }

    fn resolve_associated_type_member_id(
        &self,
        owner: &DeclarationId,
        selector: &crate::types::type_member::TypeMemberSelector,
    ) -> Option<crate::types::type_member::TypeMemberDeclarationId> {
        let base = match selector {
            crate::types::type_member::TypeMemberSelector::Named(name) => phalcom_common::selector::SelectorBase::Named(name.to_string()),
            crate::types::type_member::TypeMemberSelector::Subscript => phalcom_common::selector::SelectorBase::Subscript,
        };
        let products = self.associated_type_members.read().ok()?;
        let family = products.surfaces.get(owner)?.families.get(&base)?;
        let crate::associated::AssociatedMemberId::TypeMember(id) = family.members.first()? else {
            return None;
        };
        Some(id.clone())
    }

    fn resolve_module_alias(&self, current_module: &ModuleId, alias: &str) -> Option<ModuleId> {
        if let Some(linked_mod) = self.linked_module(current_module) {
            if let Some(&import_id) = linked_mod.bindings.imports.get::<str>(alias) {
                if let Some(LinkedReadSpec::Module(target_mod)) = linked_mod.linked_reads.get(import_id.0 as usize) {
                    return Some(target_mod.clone());
                }
            }
        }
        None
    }

    fn resolve_alias_form(&self, declaration: &DeclarationId) -> Option<crate::types::id::TypeId> {
        if let Some(form) = self.alias_form_overrides.borrow().get(declaration) {
            return *form;
        }
        self.retained_alias_forms.form(declaration)
    }

    fn resolve_associated_type_requirement(&self, trait_declaration: &DeclarationId, name: &str) -> Option<crate::traits::AssociatedTypeRequirementId> {
        self.trait_associated_types
            .get(trait_declaration)
            .and_then(|members| members.get(name))
            .cloned()
    }

    fn resolve_type_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId> {
        if members.is_empty() {
            // 1. Local declaration in current_module.
            let local_decl = DeclarationId::new(current_module.clone(), root.into());
            if self.known_declarations.contains(&local_decl) {
                return Some(local_decl);
            }

            // 2. Selective import binding in current_module.
            if let Some(linked_mod) = self.linked_module(current_module) {
                if let Some(&import_id) = linked_mod.bindings.imports.get::<str>(root) {
                    if let Some(LinkedReadSpec::Binding(sym)) = linked_mod.linked_reads.get(import_id.0 as usize) {
                        let decl = DeclarationId::new(sym.module.clone(), sym.name.clone());
                        if self.known_declarations.contains(&decl) {
                            return Some(decl);
                        }
                    }
                }
            }

            // 3. Re-export/current linked namespace.
            if let Some(linked_mod) = self.linked_module(current_module) {
                if let Some(export) = linked_mod.interface.exports.get::<str>(root) {
                    match &export.target {
                        phalcom_modules::interface::LinkedExportTarget::Binding(sym) => {
                            let decl = DeclarationId::new(sym.module.clone(), sym.name.clone());
                            if self.known_declarations.contains(&decl) {
                                return Some(decl);
                            }
                        }
                        phalcom_modules::interface::LinkedExportTarget::Module(_) => {}
                    }
                }
            }

            // 4. Canonical prelude declaration. The map contains only names
            // explicitly admitted by prelude policy and points at their real
            // source-owned DeclarationId.
            if let Some(decl) = self.prelude_types.get(root) {
                if self.known_declarations.contains(decl) {
                    return Some(decl.clone());
                }
            }

            // 5. Canonical Universe internal references. Source modules in the Universe
            // have no explicit preamble imports and may reference other un-preluded Universe declarations
            // (such as Behavior) directly without exposing them through the public user prelude.
            if current_module.project == phalcom_modules::identity::ProjectIdentity::Universe {
                if let Some(key) = phalcom_native_meta::UniverseKey::from_name(root) {
                    let decl = crate::core_surface::universe_declaration(key);
                    if self.known_declarations.contains(&decl) {
                        return Some(decl);
                    }
                }
            }

            None
        } else {
            // Qualified lookup can currently resolve only one declaration
            // member directly from an imported module alias. Until linked
            // products expose namespace traversal for every intermediate
            // component, fail closed rather than reinterpret
            // `root.a.Leaf` as `root.Leaf`.
            if members.len() != 1 {
                return None;
            }
            if let Some(linked_mod) = self.linked_module(current_module) {
                if let Some(&import_id) = linked_mod.bindings.imports.get::<str>(root) {
                    if let Some(LinkedReadSpec::Module(target_mod)) = linked_mod.linked_reads.get(import_id.0 as usize) {
                        let target = self.linked_module(target_mod)?;
                        let export = target.interface.exports.get::<str>(&members[0])?;
                        let phalcom_modules::interface::LinkedExportTarget::Binding(symbol) = &export.target else {
                            return None;
                        };
                        let declaration = DeclarationId::new(symbol.module.clone(), symbol.name.clone());
                        if self.known_declarations.contains(&declaration) {
                            return Some(declaration);
                        }
                    }
                }
            }

            None
        }
    }
}
