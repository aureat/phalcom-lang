//! Linked project-aware type resolver.

use crate::identity::{DeclarationId, ModuleId};
use crate::prelude::PreludeTypeMap;
use crate::types::annotation::TypeResolver;
use phalcom_modules::linker::{LinkedProgram, LinkedReadSpec};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

/// A project-aware type resolver that resolves type names through the linker layout
/// and known declaration tables without resorting to global heuristics.
#[derive(Clone, Debug)]
pub struct LinkedTypeResolver {
    linked: Arc<LinkedProgram>,
    known_declarations: HashSet<DeclarationId>,
    prelude_types: Arc<PreludeTypeMap>,
    alias_forms: RefCell<BTreeMap<DeclarationId, crate::types::id::TypeId>>,
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
    pub fn with_prelude(
        linked: Arc<LinkedProgram>,
        known_declarations: HashSet<DeclarationId>,
        prelude_types: Arc<PreludeTypeMap>,
    ) -> Self {
        Self {
            linked,
            known_declarations,
            prelude_types,
            alias_forms: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn prelude_types(&self) -> &Arc<PreludeTypeMap> {
        &self.prelude_types
    }

    pub fn insert_alias_form(&self, declaration: DeclarationId, form: crate::types::id::TypeId) {
        self.alias_forms.borrow_mut().insert(declaration, form);
    }
}

impl TypeResolver for LinkedTypeResolver {
    fn resolve_alias_form(&self, declaration: &DeclarationId) -> Option<crate::types::id::TypeId> {
        self.alias_forms.borrow().get(declaration).copied()
    }

    fn resolve_type_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId> {
        if members.is_empty() {
            // 1. Local declaration in current_module.
            let local_decl = DeclarationId::new(current_module.clone(), root.into());
            if self.known_declarations.contains(&local_decl) {
                return Some(local_decl);
            }

            // 2. Selective import binding in current_module.
            if let Some(linked_mod) = self.linked.modules.get(current_module) {
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
            if let Some(linked_mod) = self.linked.modules.get(current_module) {
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
            if let Some(linked_mod) = self.linked.modules.get(current_module) {
                if let Some(&import_id) = linked_mod.bindings.imports.get::<str>(root) {
                    if let Some(LinkedReadSpec::Module(target_mod)) = linked_mod.linked_reads.get(import_id.0 as usize) {
                        let leaf_name = &members[0];
                        let decl = DeclarationId::new(target_mod.clone(), leaf_name.clone().into());
                        if self.known_declarations.contains(&decl) {
                            return Some(decl);
                        }
                    }
                }
            }

            None
        }
    }
}
