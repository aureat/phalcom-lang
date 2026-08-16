//! Program materialization: populates VM runtime structures from a CompiledProgram.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{ModuleObject, Object, RuntimeExportRef};
use crate::modules::compile::CompiledProgram;
use crate::modules::linkage::{BindingRef, CompileBindings, RuntimeLinkedRead};
use crate::modules::registry::ModuleRecord;
use crate::vm::{RuntimeRoots, VM};
use phalcom_modules::LinkedReadSpec;
use std::collections::HashMap;
use std::sync::Arc;

impl VM {
    /// Materializes a closed, linked compiled program into VM module objects,
    /// global layouts, linked read slots, and export tables without running initializers.
    pub fn materialize_program(&mut self, program: &CompiledProgram) -> PhResult<()> {
        // Phase 1: Allocate all module/package objects in the Prepared state (idempotent).
        for (id, compiled_mod) in &program.modules {
            if !self.module_registry.contains_key(id) {
                let display_name = id.to_string();
                let name_sym = self.interner.intern(&display_name);
                let path = compiled_mod
                    .source
                    .as_ref()
                    .map(|s| s.display_path.display().to_string())
                    .unwrap_or_else(|| display_name.clone());

                let mut module_obj = ModuleObject::new(id.clone(), compiled_mod.kind, display_name, name_sym, path, None, false);
                module_obj.metadata = Some(Arc::new(compiled_mod.interface.metadata.clone()));
                let obj_ref = self.heap.alloc(Object::Module(Box::new(module_obj)));
                self.module_registry
                    .register_new(
                        id.clone(),
                        ModuleRecord::prepared(
                            obj_ref,
                            crate::modules::registry::RuntimeProgramId(1),
                            crate::modules::registry::ModulePlanFingerprint(0),
                        ),
                    )
                    .map_err(|e| RuntimeError::Internal(e.to_string()))?;
            }
        }

        // Phase 2: Materialize lexical context intrinsics (__module__, __package__, __project__) and ownership.
        let module_sym = self.interner.intern("__module__");
        let package_sym = self.interner.intern("__package__");
        let project_sym = self.interner.intern("__project__");

        for (id, compiled_mod) in &program.modules {
            let obj_ref = self.module_registry.get(id).expect("module allocated").object;

            // 1. __module__ is bound to the current module object.
            self.define_global(obj_ref, module_sym, crate::value::Value::Obj(obj_ref))?;

            // 2. Ownership and context resolution according to ModuleKind and ProjectIdentity.
            match compiled_mod.kind {
                phalcom_modules::ModuleKind::ProjectRoot => {
                    // Spec §6 & §15.2: Project is its own root Package.
                    // owning_package = None (it is root), owning_project = Some(self).
                    self.heap.module_mut(obj_ref).owning_package = None;
                    self.heap.module_mut(obj_ref).owning_project = Some(obj_ref);

                    let self_val = crate::value::Value::Obj(obj_ref).wrap_some()?;
                    self.define_global(obj_ref, package_sym, self_val)?;
                    self.define_global(obj_ref, project_sym, self_val)?;
                }
                phalcom_modules::ModuleKind::Package => {
                    // Package: determine parent package and owning project.
                    let (parent_pkg_id, root_proj_id) = match &id.project {
                        phalcom_modules::ProjectIdentity::Resolved(pid) => {
                            let is_standalone = program.project_universe.get_project(*pid).map(|p| p.is_standalone_package()).unwrap_or(false);
                            let parent = if id.path.is_root() {
                                None
                            } else {
                                id.path.parent().map(|p| phalcom_modules::ModuleId::resolved(*pid, p))
                            };
                            let root = if is_standalone {
                                None
                            } else {
                                Some(phalcom_modules::ModuleId::resolved(*pid, phalcom_modules::ModulePath::root()))
                            };
                            (parent, root)
                        }
                        phalcom_modules::ProjectIdentity::Builtin(bid) => {
                            let parent = if id.path.is_root() {
                                None
                            } else {
                                id.path.parent().map(|p| phalcom_modules::ModuleId::builtin(*bid, p))
                            };
                            let root = Some(phalcom_modules::ModuleId::builtin(*bid, phalcom_modules::ModulePath::root()));
                            (parent, root)
                        }
                        phalcom_modules::ProjectIdentity::Synthetic(sid) => {
                            let parent = if id.path.is_root() {
                                None
                            } else {
                                id.path.parent().map(|p| phalcom_modules::ModuleId::synthetic(*sid, p))
                            };
                            (parent, None)
                        }
                    };

                    // Set owning_package
                    let pkg_val = if let Some(parent_id) = parent_pkg_id {
                        if let Some(record) = self.module_registry.get(&parent_id) {
                            let parent_obj = record.object;
                            self.heap.module_mut(obj_ref).owning_package = Some(parent_obj);
                            crate::value::Value::Obj(parent_obj).wrap_some()?
                        } else {
                            crate::value::Value::None
                        }
                    } else if id.path.is_root() {
                        // Spec §15.3: Standalone root package.ph has __package__ = Some(self).
                        crate::value::Value::Obj(obj_ref).wrap_some()?
                    } else {
                        crate::value::Value::None
                    };
                    self.define_global(obj_ref, package_sym, pkg_val)?;

                    // Set owning_project
                    let proj_val = if let Some(root_id) = root_proj_id {
                        if let Some(record) = self.module_registry.get(&root_id) {
                            let proj_obj = record.object;
                            self.heap.module_mut(obj_ref).owning_project = Some(proj_obj);
                            crate::value::Value::Obj(proj_obj).wrap_some()?
                        } else {
                            crate::value::Value::None
                        }
                    } else {
                        self.heap.module_mut(obj_ref).owning_project = None;
                        crate::value::Value::None
                    };
                    self.define_global(obj_ref, project_sym, proj_val)?;
                }
                phalcom_modules::ModuleKind::Module => {
                    // Ordinary Module: determine nearest package and owning project.
                    let (nearest_pkg_id, root_proj_id) = match &id.project {
                        phalcom_modules::ProjectIdentity::Resolved(pid) => {
                            let is_standalone = program.project_universe.get_project(*pid).map(|p| p.is_standalone_package()).unwrap_or(false);
                            let pkg = id
                                .path
                                .parent()
                                .map(|p| phalcom_modules::ModuleId::resolved(*pid, p))
                                .unwrap_or_else(|| phalcom_modules::ModuleId::resolved(*pid, phalcom_modules::ModulePath::root()));
                            let root = if is_standalone {
                                None
                            } else {
                                Some(phalcom_modules::ModuleId::resolved(*pid, phalcom_modules::ModulePath::root()))
                            };
                            (Some(pkg), root)
                        }
                        phalcom_modules::ProjectIdentity::Builtin(bid) => {
                            let pkg = id
                                .path
                                .parent()
                                .map(|p| phalcom_modules::ModuleId::builtin(*bid, p))
                                .unwrap_or_else(|| phalcom_modules::ModuleId::builtin(*bid, phalcom_modules::ModulePath::root()));
                            let root = Some(phalcom_modules::ModuleId::builtin(*bid, phalcom_modules::ModulePath::root()));
                            (Some(pkg), root)
                        }
                        phalcom_modules::ProjectIdentity::Synthetic(sid) => {
                            // Standalone module has no package if root path or no package.ph
                            let pkg = id.path.parent().map(|p| phalcom_modules::ModuleId::synthetic(*sid, p));
                            (pkg, None)
                        }
                    };

                    // Set owning_package
                    let pkg_val = if let Some(pkg_id) = nearest_pkg_id {
                        if let Some(record) = self.module_registry.get(&pkg_id) {
                            let pkg_obj = record.object;
                            self.heap.module_mut(obj_ref).owning_package = Some(pkg_obj);
                            crate::value::Value::Obj(pkg_obj).wrap_some()?
                        } else {
                            crate::value::Value::None
                        }
                    } else {
                        self.heap.module_mut(obj_ref).owning_package = None;
                        crate::value::Value::None
                    };
                    self.define_global(obj_ref, package_sym, pkg_val)?;

                    // Set owning_project
                    let proj_val = if let Some(root_id) = root_proj_id {
                        if let Some(record) = self.module_registry.get(&root_id) {
                            let proj_obj = record.object;
                            self.heap.module_mut(obj_ref).owning_project = Some(proj_obj);
                            crate::value::Value::Obj(proj_obj).wrap_some()?
                        } else {
                            crate::value::Value::None
                        }
                    } else {
                        self.heap.module_mut(obj_ref).owning_project = None;
                        crate::value::Value::None
                    };
                    self.define_global(obj_ref, project_sym, proj_val)?;
                }
            }
        }

        // Phase 3: Materialize declaration blueprints (classes/globals from artifact).
        for (id, compiled_mod) in &program.modules {
            let obj_ref = self.module_registry.get(id).expect("module allocated").object;
            for decl in &compiled_mod.plan.declarations {
                match decl {
                    crate::modules::RuntimeDeclarationBlueprint::Global { symbol, .. } => {
                        let sym = self.interner.intern(&symbol.name);
                        self.heap.module_mut(obj_ref).declare(sym)?;
                    }
                    crate::modules::RuntimeDeclarationBlueprint::Class(_class_bp) => {}
                }
            }
        }

        // Phase 4: Materialize linked reads (resolve LinkedReadSpec -> RuntimeLinkedRead).
        for (id, compiled_mod) in &program.modules {
            let obj_ref = self.module_registry.get(id).expect("module allocated").object;
            let mut materialized_reads = Vec::with_capacity(compiled_mod.linked_reads.len());

            for read_spec in &compiled_mod.linked_reads {
                let runtime_read = match read_spec {
                    LinkedReadSpec::Module(target_id) => {
                        let target_obj = self
                            .module_registry
                            .get(target_id)
                            .ok_or_else(|| RuntimeError::Internal(format!("linked read target module {target_id} not registered")))?
                            .object;
                        RuntimeLinkedRead::Module(target_obj)
                    }
                    LinkedReadSpec::Binding(symbol_id) => {
                        let target_obj = self
                            .module_registry
                            .get(&symbol_id.module)
                            .ok_or_else(|| RuntimeError::Internal(format!("linked read target module {} not registered", symbol_id.module)))?
                            .object;
                        let sym = self.interner.intern(&symbol_id.name);
                        let slot = match self.heap.module(target_obj).slot_of(sym) {
                            Some(s) => s,
                            None => self.heap.module_mut(target_obj).declare(sym)?,
                        };
                        RuntimeLinkedRead::Binding(BindingRef {
                            module: target_obj,
                            slot: slot as u16,
                        })
                    }
                };
                materialized_reads.push(runtime_read);
            }
            self.heap.module_mut(obj_ref).linked_reads = materialized_reads;
        }

        // Phase 5: Materialize export table on ModuleObject.
        for (id, compiled_mod) in &program.modules {
            let obj_ref = self.module_registry.get(id).expect("module allocated").object;
            let mut exports = HashMap::new();

            for (exported_name, linked_export) in &compiled_mod.interface.exports {
                let public_sym = self.interner.intern(exported_name);
                match &linked_export.target {
                    phalcom_modules::LinkedExportTarget::Binding(symbol) => {
                        let target_mod_obj = self
                            .module_registry
                            .get(&symbol.module)
                            .ok_or_else(|| RuntimeError::Internal(format!("export target module {} not registered", symbol.module)))?
                            .object;
                        let target_sym = self.interner.intern(&symbol.name);

                        let slot = match self.heap.module(target_mod_obj).slot_of(target_sym) {
                            Some(s) => s,
                            None => self.heap.module_mut(target_mod_obj).declare(target_sym)?,
                        };
                        exports.insert(
                            public_sym,
                            RuntimeExportRef::Binding(BindingRef {
                                module: target_mod_obj,
                                slot: slot as u16,
                            }),
                        );
                    }
                    phalcom_modules::LinkedExportTarget::Module(target_mod_id) => {
                        let target_mod_obj = self
                            .module_registry
                            .get(target_mod_id)
                            .ok_or_else(|| RuntimeError::Internal(format!("export target module {target_mod_id} not registered")))?
                            .object;
                        exports.insert(public_sym, RuntimeExportRef::Module(target_mod_obj));
                    }
                }
            }
            self.heap.module_mut(obj_ref).exports = exports;
        }

        // Phase 6: Top-level closures are compiled on-demand by run_compiled rather than pre-stored on plans.

        // Phase 7: Record entry handle on VM runtime roots.
        let entry_obj = self
            .module_registry
            .get(&program.entry)
            .ok_or_else(|| RuntimeError::Internal(format!("entry module {} not registered", program.entry)))?
            .object;
        let core_obj = self.core_module().unwrap_or(entry_obj);
        self.runtime_roots = Some(RuntimeRoots {
            core: core_obj,
            entry: Some(entry_obj),
        });

        Ok(())
    }

    /// Compiles and attaches a source closure for `id` using precomputed bindings.
    pub fn compile_program_module_closure(&mut self, id: &phalcom_modules::ModuleId, source: &str, program: &CompiledProgram) -> PhResult<crate::heap::ObjRef> {
        let obj_ref = self
            .module_registry
            .get(id)
            .ok_or_else(|| RuntimeError::Internal(format!("module {id} not found in registry")))?
            .object;
        let bindings = program.linked.modules.get(id).map(CompileBindings::from_linked_module);
        let closure = self
            .compile_closure_as_with_bindings(obj_ref, source, crate::compiler::lib::UnitKind::File, bindings)
            .inspect_err(|err| {
                let source_id = self.heap.module(obj_ref).sources.len().saturating_sub(1) as u32;
                self.compiler_error(err.clone(), obj_ref, source_id);
            })?;
        self.heap.module_mut(obj_ref).closure = Some(closure);
        Ok(closure)
    }
}
