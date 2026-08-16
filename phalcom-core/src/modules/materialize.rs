//! Program materialization: populate runtime structures from an immutable linked program plan.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{ModuleObject, Object, RuntimeExportRef};
use crate::modules::compile::CompiledProgram;
use crate::modules::linkage::{BindingRef, CompileBindings, RuntimeLinkedRead};
use crate::modules::registry::{ModuleOwner, ModuleRecord, ModuleRegistryError};
use crate::vm::{RuntimeRoots, VM};
use phalcom_modules::{LinkedReadSpec, ProjectIdentity};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

fn registry_error(error: ModuleRegistryError) -> crate::error::PhError {
    RuntimeError::Internal(error.to_string()).into()
}

impl VM {
    /// Materialize only modules that are absent for this exact runtime program
    /// and immutable plan. Existing matching records are a strict no-op.
    pub fn materialize_program(&mut self, program: &CompiledProgram) -> PhResult<()> {
        let mut materialize = HashSet::new();

        // Phase 1: validate ownership/fingerprint before mutating any existing
        // record, then allocate only genuinely absent module objects.
        for (id, compiled_mod) in &program.modules {
            if let Some(record) = self.module_registry.get(id) {
                match id.project {
                    ProjectIdentity::Builtin(_) => {
                        if record.owner != ModuleOwner::Builtin {
                            return Err(registry_error(ModuleRegistryError::ProgramOwnershipConflict { module: id.clone() }));
                        }
                        match record.plan_fingerprint {
                            Some(existing) if existing != compiled_mod.plan_fingerprint => {
                                return Err(registry_error(ModuleRegistryError::PlanFingerprintMismatch { module: id.clone() }));
                            }
                            Some(_) => continue,
                            None => {
                                let record = self
                                    .module_registry
                                    .get_mut(id)
                                    .ok_or_else(|| RuntimeError::Internal(format!("builtin module {id} disappeared during materialization")))?;
                                record.plan_fingerprint = Some(compiled_mod.plan_fingerprint);
                                materialize.insert(id.clone());
                                continue;
                            }
                        }
                    }
                    _ => {
                        if record.owner != ModuleOwner::Program(program.runtime_id) {
                            return Err(registry_error(ModuleRegistryError::ProgramOwnershipConflict { module: id.clone() }));
                        }
                        if record.plan_fingerprint != Some(compiled_mod.plan_fingerprint) {
                            return Err(registry_error(ModuleRegistryError::PlanFingerprintMismatch { module: id.clone() }));
                        }
                        continue;
                    }
                }
            }

            let display_name = id.to_string();
            let name_sym = self.interner.intern(&display_name);
            let path = compiled_mod
                .source
                .as_ref()
                .map(|source| source.display_path.display().to_string())
                .unwrap_or_else(|| display_name.clone());
            let mut module_obj = ModuleObject::new(id.clone(), compiled_mod.kind, display_name, name_sym, path, None, false);
            module_obj.metadata = Some(Arc::new(compiled_mod.interface.metadata.clone()));
            let obj_ref = self.heap.alloc(Object::Module(Box::new(module_obj)));
            let record = match id.project {
                ProjectIdentity::Builtin(_) => ModuleRecord::builtin_prepared(obj_ref, compiled_mod.plan_fingerprint),
                _ => ModuleRecord::prepared_for(obj_ref, program.runtime_id, compiled_mod.plan_fingerprint),
            };
            self.module_registry.register_new(id.clone(), record).map_err(registry_error)?;
            materialize.insert(id.clone());
        }

        // Phase 2: predeclare every module-owned global/class name before any
        // source closure can execute. Class blueprints are refined by the
        // source-aware plan producer; both variants reserve the canonical slot.
        for id in &materialize {
            let compiled_mod = &program.modules[id];
            let obj_ref = self
                .module_registry
                .get(id)
                .ok_or_else(|| RuntimeError::Internal(format!("newly materialized module {id} disappeared")))?
                .object;
            for decl in &compiled_mod.artifact.declarations {
                let symbol = match decl {
                    crate::modules::RuntimeDeclarationBlueprint::Global { symbol, .. } => symbol,
                    crate::modules::RuntimeDeclarationBlueprint::Class(class) => &class.symbol,
                };
                let sym = self.interner.intern(&symbol.name);
                if self.heap.module(obj_ref).slot_of(sym).is_none() {
                    self.heap.module_mut(obj_ref).declare(sym)?;
                }
            }
        }

        // Phase 3: materialize symbolic linked reads only for new records.
        for id in &materialize {
            let compiled_mod = &program.modules[id];
            let obj_ref = self
                .module_registry
                .get(id)
                .ok_or_else(|| RuntimeError::Internal(format!("new module {id} not registered")))?
                .object;
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
                            Some(slot) => slot,
                            None => self.heap.module_mut(target_obj).declare(sym)?,
                        };
                        RuntimeLinkedRead::Binding(BindingRef { module: target_obj, slot: slot as u16 })
                    }
                };
                materialized_reads.push(runtime_read);
            }
            self.heap.module_mut(obj_ref).linked_reads = materialized_reads;
        }

        // Phase 4: materialize linked export tables only for new records.
        for id in &materialize {
            let compiled_mod = &program.modules[id];
            let obj_ref = self
                .module_registry
                .get(id)
                .ok_or_else(|| RuntimeError::Internal(format!("new module {id} not registered")))?
                .object;
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
                            Some(slot) => slot,
                            None => self.heap.module_mut(target_mod_obj).declare(target_sym)?,
                        };
                        exports.insert(public_sym, RuntimeExportRef::Binding(BindingRef { module: target_mod_obj, slot: slot as u16 }));
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

        let entry_obj = self
            .module_registry
            .get(&program.entry)
            .ok_or_else(|| RuntimeError::Internal(format!("entry module {} not registered", program.entry)))?
            .object;
        let core_obj = self.core_module().unwrap_or(entry_obj);
        self.runtime_roots = Some(RuntimeRoots { core: core_obj, entry: Some(entry_obj) });
        Ok(())
    }

    /// Compile and attach a source closure exactly once for a Prepared module.
    pub fn compile_program_module_closure(
        &mut self,
        id: &phalcom_modules::ModuleId,
        source: &str,
        program: &CompiledProgram,
    ) -> PhResult<crate::heap::ObjRef> {
        let obj_ref = self
            .module_registry
            .get(id)
            .ok_or_else(|| RuntimeError::Internal(format!("module {id} not found in registry")))?
            .object;
        if let Some(existing) = self.heap.module(obj_ref).closure {
            return Ok(existing);
        }
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
