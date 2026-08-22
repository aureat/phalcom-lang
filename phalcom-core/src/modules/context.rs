//! Shared semantic execution context for modules, projects, and the interactive REPL.

use crate::error::{PhResult, RuntimeError};
use crate::heap::ObjRef;
use crate::modules::compile::{EntrySelection, ProgramCompiler};
use crate::modules::linkage::{BindingRef, CompileBindings, LinkedImportInfo, RuntimeLinkedRead, TopLevelBindingInfo, TopLevelBindingKind};
use crate::vm::VM;
use phalcom_ast::ast::{DependencyDecl, ImportDecl, ImportRoot, Program};
use phalcom_modules::builtin::BuiltinProjectSourceProvider;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ProjectIdentity, ResolvedProjectId, SyntheticProjectIdAllocator};
use phalcom_modules::linker::{ImportBindingId, LinkedReadSpec, SymbolId};
use phalcom_modules::project::{ProjectUniverse, discover_owning_project};
use phalcom_modules::resolver::ModuleResolver;
use phalcom_modules::source::FilesystemSourceProvider;
use phalcom_modules::stabilization::ResolverGeneration;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Persistent execution context coordinating module resolution, artifact compilation,
/// and live VM module linkage across cell and file execution boundaries.
pub struct ModuleExecutionContext {
    /// Canonical semantic identity of the active session or entry module.
    pub session_id: ModuleId,
    /// Root working directory.
    pub cwd: PathBuf,
    /// Authoritative project universe if running within a workspace/project.
    pub project_universe: Option<ProjectUniverse>,
    /// Incremental resolver generation token.
    pub resolver_generation: ResolverGeneration,
    /// Cumulative compiler bindings visible in the session scope.
    pub bindings: CompileBindings,
    /// Cumulative runtime linked reads materialized on the session module.
    pub linked_reads: Vec<RuntimeLinkedRead>,
    /// Number of synthetic allocations made in this execution context.
    pub synthetic_allocator: SyntheticProjectIdAllocator,
}

impl ModuleExecutionContext {
    /// Creates a new execution context for a REPL or script rooted at `cwd`.
    pub fn new(cwd: PathBuf, session_module_name: &str) -> Self {
        let mut universe = ProjectUniverse::new();
        let project_universe = if let Ok(Some(project_root)) = discover_owning_project(&cwd) {
            let manifest_path = project_root.join("project.toml");
            if universe.load_root(&manifest_path).is_ok() { Some(universe) } else { None }
        } else {
            None
        };

        let mut synthetic_allocator = SyntheticProjectIdAllocator;
        let session_id = if let Some(ref _u) = project_universe {
            let root_id = ResolvedProjectId::from_raw(1);
            let path = ModulePath::from_components(vec![
                ModuleComponent::from_identifier(session_module_name).unwrap_or_else(|_| ModuleComponent::from_identifier("main").unwrap()),
            ]);
            ModuleId::resolved(root_id, path)
        } else {
            let sid = synthetic_allocator.allocate();
            let path = ModulePath::from_components(vec![
                ModuleComponent::from_identifier(session_module_name).unwrap_or_else(|_| ModuleComponent::from_identifier("main").unwrap()),
            ]);
            ModuleId::synthetic(sid, path)
        };

        Self {
            session_id,
            cwd,
            project_universe,
            resolver_generation: ResolverGeneration::default(),
            bindings: CompileBindings::default(),
            linked_reads: Vec::new(),
            synthetic_allocator,
        }
    }

    /// Resolves, compiles, and links imports for a new program unit in this context.
    pub fn process_cell_dependencies(&mut self, vm: &mut VM, session_obj: ObjRef, program: &Program) -> PhResult<()> {
        if program.preamble.dependencies.is_empty() {
            return Ok(());
        }

        let fs_provider = FilesystemSourceProvider::new();
        let dummy_universe = ProjectUniverse::new();
        let universe_ref = self.project_universe.as_ref().unwrap_or(&dummy_universe);
        let mut resolver = ModuleResolver::new(universe_ref, &fs_provider);

        for dep in &program.preamble.dependencies {
            match dep {
                DependencyDecl::Import(ImportDecl::Module(mod_decl)) => {
                    let resolved_target = resolver
                        .resolve_import(&self.session_id, &mod_decl.path)
                        .map_err(|e| RuntimeError::Internal(format!("import resolution failed: {e}")))?;

                    self.ensure_module_materialized(vm, &resolved_target.id)?;

                    let local_name = if let Some(alias) = &mod_decl.alias {
                        alias.name.clone()
                    } else if mod_decl.path.segments.is_empty() {
                        match &mod_decl.path.root {
                            ImportRoot::Absolute(seg) => seg.name.clone(),
                            ImportRoot::Relative { .. } => {
                                return Err(RuntimeError::Internal("relative import without segments".to_string()).into());
                            }
                        }
                    } else {
                        mod_decl.path.segments.last().unwrap().name.clone()
                    };

                    let target_obj = vm
                        .module_registry
                        .get(&resolved_target.id)
                        .ok_or_else(|| RuntimeError::Internal(format!("module {} not registered", resolved_target.id)))?
                        .object;

                    let binding_index = self.linked_reads.len() as u32;
                    let binding_id = ImportBindingId(binding_index);
                    self.linked_reads.push(RuntimeLinkedRead::Module(target_obj));

                    let local_boxed: Box<str> = local_name.into_boxed_str();
                    self.bindings.entries.insert(
                        local_boxed.clone(),
                        TopLevelBindingInfo {
                            kind: TopLevelBindingKind::Import(binding_id),
                            declared_at: Some(mod_decl.range),
                        },
                    );
                    self.bindings.imports.insert(
                        local_boxed.clone(),
                        LinkedImportInfo {
                            local_name: local_boxed,
                            binding: binding_id,
                            target: LinkedReadSpec::Module(resolved_target.id),
                            symbol: None,
                        },
                    );
                }
                DependencyDecl::Import(ImportDecl::Selective(sel_decl)) => {
                    let resolved_target = resolver
                        .resolve_import(&self.session_id, &sel_decl.path)
                        .map_err(|e| RuntimeError::Internal(format!("selective import resolution failed: {e}")))?;

                    self.ensure_module_materialized(vm, &resolved_target.id)?;

                    let target_obj = vm
                        .module_registry
                        .get(&resolved_target.id)
                        .ok_or_else(|| RuntimeError::Internal(format!("module {} not registered", resolved_target.id)))?
                        .object;

                    for item in &sel_decl.items {
                        let local_name = if let Some(alias) = &item.alias {
                            alias.name.clone()
                        } else {
                            item.name.clone()
                        };

                        let item_sym = vm.interner.intern(&item.name);
                        let exports = &vm.heap.module(target_obj).exports;
                        if !exports.contains_key(&item_sym) {
                            return Err(RuntimeError::Internal(format!(
                                "selective import resolution failed: module {} does not export '{}'",
                                resolved_target.id, item.name
                            ))
                            .into());
                        }

                        let slot = vm
                            .heap
                            .module(target_obj)
                            .slot_of(item_sym)
                            .expect("export is confirmed; slot must exist after materialization");

                        let binding_index = self.linked_reads.len() as u32;
                        let binding_id = ImportBindingId(binding_index);
                        let symbol_id = SymbolId {
                            module: resolved_target.id.clone(),
                            name: item.name.clone().into_boxed_str(),
                        };

                        self.linked_reads.push(RuntimeLinkedRead::Binding(BindingRef {
                            module: target_obj,
                            slot: slot as u16,
                        }));

                        let local_boxed: Box<str> = local_name.into_boxed_str();
                        self.bindings.entries.insert(
                            local_boxed.clone(),
                            TopLevelBindingInfo {
                                kind: TopLevelBindingKind::Import(binding_id),
                                declared_at: Some(item.range),
                            },
                        );
                        self.bindings.imports.insert(
                            local_boxed.clone(),
                            LinkedImportInfo {
                                local_name: local_boxed,
                                binding: binding_id,
                                target: LinkedReadSpec::Binding(symbol_id.clone()),
                                symbol: Some(symbol_id),
                            },
                        );
                    }
                }
                _ => {}
            }
        }

        // Attach updated linked_reads to the session module object
        vm.heap.module_mut(session_obj).linked_reads = self.linked_reads.clone();

        Ok(())
    }

    /// Ensures a resolved module is compiled, registered, and materialized in the VM.
    fn ensure_module_materialized(&self, vm: &mut VM, id: &ModuleId) -> PhResult<()> {
        if vm.module_registry.get(id).is_some() {
            return Ok(());
        }

        // Compile and materialize the target module
        match &id.project {
            ProjectIdentity::Builtin(bid) => {
                let provider = BuiltinProjectSourceProvider::new(*bid);
                let iface = provider
                    .load_interface(id)
                    .map_err(|e| RuntimeError::Internal(format!("failed to load builtin interface for {id}: {e}")))?;

                let display_name = id.to_string();
                let name_sym = vm.interner.intern(
                    &id.path
                        .components()
                        .last()
                        .map(|c| c.as_str().to_string())
                        .unwrap_or_else(|| "package".to_string()),
                );
                let path = format!("<builtin:{id}>");

                let mut module_obj = crate::heap::ModuleObject::new(id.clone(), iface.kind, display_name, name_sym, path, None, true);
                module_obj.metadata = Some(Arc::new(iface.metadata.clone()));

                let obj_ref = vm.heap.alloc(crate::heap::Object::Module(Box::new(module_obj)));
                vm.module_registry
                    .register_new(
                        id.clone(),
                        crate::modules::registry::ModuleRecord::prepared(
                            obj_ref,
                            crate::modules::registry::RuntimeProgramId(0),
                            crate::modules::registry::ModulePlanFingerprint(0),
                        ),
                    )
                    .map_err(|e| RuntimeError::Internal(e.to_string()))?;

                // Populate exports from iface
                let mut exports = HashMap::new();
                for name in iface.exports.keys() {
                    let sym = vm.interner.intern(name);
                    let slot = vm.heap.module_mut(obj_ref).declare(sym)?;
                    if let Some(key) = phalcom_native_meta::UniverseKey::from_name(name) {
                        let class_id = vm.universe.classes.resolve(key);
                        vm.heap.module_mut(obj_ref).set_global(slot, crate::value::Value::obj(class_id))?;
                    }
                    exports.insert(
                        sym,
                        crate::heap::RuntimeExportRef::Binding(BindingRef {
                            module: obj_ref,
                            slot: slot as u16,
                        }),
                    );
                }
                vm.heap.module_mut(obj_ref).exports = exports;

                // For leaf modules, compile and run the source initializer to populate
                // class bindings. Package .ph files contain only expose/re-export — no body
                // to run — so we restrict this to Module kind.
                if iface.kind == phalcom_modules::source::ModuleKind::Module {
                    if let Ok(source_text) = provider.source_text(id) {
                        let closure = vm
                            .compile_closure_as_with_bindings(obj_ref, &source_text, crate::compiler::lib::UnitKind::File, None)
                            .map_err(|e| RuntimeError::Internal(format!("failed to compile builtin module {id}: {e}")))?;
                        vm.heap.module_mut(obj_ref).closure = Some(closure);
                        vm.run_in_module(obj_ref, closure)
                            .map_err(|e| RuntimeError::Internal(format!("failed to initialize builtin module {id}: {e}")))?;
                        if let Some(rec) = vm.module_registry.get_mut(id) {
                            rec.state = crate::modules::registry::ModuleState::Initialized;
                        }
                    }
                }
            }
            _ => {
                // User module compilation and materialization
                let selection = EntrySelection::Module(self.cwd.join(id.path.to_string()));
                if let Ok(program) = ProgramCompiler::compile_entry_selection(selection) {
                    vm.materialize_program(&program)?;
                }
            }
        }

        Ok(())
    }
}
