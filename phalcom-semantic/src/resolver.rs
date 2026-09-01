//! Linked project-aware type resolver.

use crate::identity::{DeclarationId, ModuleId};
use crate::types::annotation::TypeResolver;
use phalcom_modules::linker::{LinkedProgram, LinkedReadSpec};
use std::collections::HashSet;
use std::sync::Arc;

/// A project-aware type resolver that resolves type names through the linker layout
/// and known declaration tables without resorting to global heuristics.
#[derive(Clone, Debug)]
pub struct LinkedTypeResolver {
    linked: Arc<LinkedProgram>,
    known_declarations: HashSet<DeclarationId>,
    prelude_module: ModuleId,
}

impl LinkedTypeResolver {
    pub fn new(linked: Arc<LinkedProgram>, known_declarations: HashSet<DeclarationId>, prelude_module: ModuleId) -> Self {
        Self {
            linked,
            known_declarations,
            prelude_module,
        }
    }
}

impl TypeResolver for LinkedTypeResolver {
    fn resolve_type_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId> {
        if members.is_empty() {
            // 1. Local declaration in current_module
            let local_decl = DeclarationId::new(current_module.clone(), root.into());
            if self.known_declarations.contains(&local_decl) {
                return Some(local_decl);
            }

            // 2. Selective import binding in current_module
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

            // 3. Re-export in current_module
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

            // 4. Builtin / prelude declaration
            let prelude_decl = DeclarationId::new(self.prelude_module.clone(), root.into());
            if self.known_declarations.contains(&prelude_decl) {
                return Some(prelude_decl);
            }

            let core_decl = DeclarationId::new(ModuleId::universe_root(), root.into());
            if self.known_declarations.contains(&core_decl) {
                return Some(core_decl);
            }

            None
        } else {
            // Qualified path: e.g. root is a module alias in current_module
            if let Some(linked_mod) = self.linked.modules.get(current_module) {
                if let Some(&import_id) = linked_mod.bindings.imports.get::<str>(root) {
                    if let Some(LinkedReadSpec::Module(target_mod)) = linked_mod.linked_reads.get(import_id.0 as usize) {
                        let leaf_name = members.last().unwrap();
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
