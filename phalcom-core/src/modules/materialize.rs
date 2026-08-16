//! Program materialization: populates VM runtime structures from a CompiledProgram.

use crate::error::{PhResult, RuntimeError};
use crate::heap::{ModuleObject, Object, RuntimeExportRef};
use crate::modules::compile::CompiledProgram;
use crate::modules::linkage::{BindingRef, CompileBindings, RuntimeLinkedRead};
use crate::modules::registry::ModuleRecord;
use crate::vm::{RuntimeRoots, VM};
use phalcom_modules::LinkedReadSpec;
use std::collections::HashMap;

impl VM {
    /// Materializes a closed, linked compiled program into VM module objects,
    /// global layouts, linked read slots, and export tables without running initializers.
    pub fn materialize_program(&mut self, program: &CompiledProgram) -> PhResult<()> {
        // Phase 1: Allocate all module/package objects in the Prepared state.
        for (id, compiled_mod) in &program.modules {
            let display_name = id.to_string();
            let name_sym = self.interner.intern(&display_name);
            let path = compiled_mod
                .source
                .as_ref()
                .map(|s| s.display_path.display().to_string())
                .unwrap_or_else(|| display_name.clone());

            let module_obj = ModuleObject::new(id.clone(), compiled_mod.kind, display_name, name_sym, path, None, false);
            let obj_ref = self.heap.alloc(Object::Module(Box::new(module_obj)));
            self.module_registry.insert(id.clone(), ModuleRecord::prepared(obj_ref));
        }

        // Phase 2: Global layouts are populated dynamically by top-level execution.

        // Phase 3: Materialize declaration blueprints (classes/globals from artifact).
        for (id, compiled_mod) in &program.modules {
            let obj_ref = self.module_registry.get(id).expect("module allocated").object;
            for decl in &compiled_mod.artifact.declarations {
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
                let target_mod_obj = self
                    .module_registry
                    .get(&linked_export.symbol.module)
                    .ok_or_else(|| RuntimeError::Internal(format!("export target module {} not registered", linked_export.symbol.module)))?
                    .object;
                let target_sym = self.interner.intern(&linked_export.symbol.name);

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
            self.heap.module_mut(obj_ref).exports = exports;
        }

        // Phase 6: Install initializer closures (if available).
        for (id, compiled_mod) in &program.modules {
            let obj_ref = self.module_registry.get(id).expect("module allocated").object;
            if let Some(init_closure) = compiled_mod.artifact.initializer {
                self.heap.module_mut(obj_ref).closure = Some(init_closure);
            }
        }

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
