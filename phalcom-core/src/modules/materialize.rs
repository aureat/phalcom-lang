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

        // Phase 2: Materialize lexical context intrinsics (__module__, __package__, __root__, __project__) and ownership.
        let module_sym = self.interner.intern("__module__");
        let package_sym = self.interner.intern("__package__");
        let root_sym = self.interner.intern("__root__");
        let project_sym = self.interner.intern("__project__");

        for (id, compiled_mod) in &program.modules {
            let obj_ref = self.module_registry.get(id).expect("module allocated").object;

            // 1. __module__ is bound to the current module object.
            self.define_global(obj_ref, module_sym, crate::value::Value::obj(obj_ref))?;

            // 2. Ownership and context resolution according to ModuleKind and ProjectIdentity.
            let (nearest_pkg_id, root_pkg_id, proj_pid) = match &id.project {
                phalcom_modules::ProjectIdentity::Resolved(pid) => {
                    let is_standalone = program.project_universe.get_project(*pid).map(|p| p.is_standalone_package()).unwrap_or(false);
                    let parent = if id.path.is_root() {
                        if compiled_mod.kind == phalcom_modules::ModuleKind::Package {
                            None
                        } else {
                            Some(phalcom_modules::ModuleId::resolved(*pid, phalcom_modules::ModulePath::root()))
                        }
                    } else {
                        id.path.parent().map(|p| phalcom_modules::ModuleId::resolved(*pid, p))
                    };
                    let root = if is_standalone {
                        None
                    } else {
                        Some(phalcom_modules::ModuleId::resolved(*pid, phalcom_modules::ModulePath::root()))
                    };
                    (parent, root, if is_standalone { None } else { Some(*pid) })
                }
                phalcom_modules::ProjectIdentity::Builtin(bid) => {
                    let parent = if id.path.is_root() {
                        if compiled_mod.kind == phalcom_modules::ModuleKind::Package {
                            None
                        } else {
                            Some(phalcom_modules::ModuleId::builtin(*bid, phalcom_modules::ModulePath::root()))
                        }
                    } else {
                        id.path.parent().map(|p| phalcom_modules::ModuleId::builtin(*bid, p))
                    };
                    let root = Some(phalcom_modules::ModuleId::builtin(*bid, phalcom_modules::ModulePath::root()));
                    (parent, root, None)
                }
                phalcom_modules::ProjectIdentity::Synthetic(sid) => {
                    let parent = id.path.parent().map(|p| phalcom_modules::ModuleId::synthetic(*sid, p));
                    (parent, None, None)
                }
            };

            // Set package
            let pkg_val = if compiled_mod.kind == phalcom_modules::ModuleKind::Package {
                if let Some(parent_id) = nearest_pkg_id {
                    if let Some(record) = self.module_registry.get(&parent_id) {
                        self.heap.module_mut(obj_ref).package = Some(record.object);
                    }
                } else {
                    self.heap.module_mut(obj_ref).package = Some(obj_ref);
                }
                crate::value::Value::obj(obj_ref).wrap_some()?
            } else {
                if let Some(pkg_id) = nearest_pkg_id {
                    if let Some(record) = self.module_registry.get(&pkg_id) {
                        let pkg_obj = record.object;
                        self.heap.module_mut(obj_ref).package = Some(pkg_obj);
                        crate::value::Value::obj(pkg_obj).wrap_some()?
                    } else {
                        crate::value::Value::none()
                    }
                } else {
                    self.heap.module_mut(obj_ref).package = None;
                    crate::value::Value::none()
                }
            };
            self.define_global(obj_ref, package_sym, pkg_val)?;

            // Set root_package and __root__
            let root_val = if let Some(root_id) = root_pkg_id {
                if let Some(record) = self.module_registry.get(&root_id) {
                    let root_obj = record.object;
                    self.heap.module_mut(obj_ref).root_package = Some(root_obj);
                    crate::value::Value::obj(root_obj).wrap_some()?
                } else {
                    crate::value::Value::none()
                }
            } else {
                self.heap.module_mut(obj_ref).root_package = None;
                crate::value::Value::none()
            };
            self.define_global(obj_ref, root_sym, root_val)?;

            // Set __project__ (defined only in active development project context)
            let project_val = if let Some(pid) = proj_pid {
                if let Some(resolved_proj) = program.project_universe.get_project(pid) {
                    if let Some(manifest) = &resolved_proj.manifest {
                        let root_pkg_id = phalcom_modules::ModuleId::resolved(pid, phalcom_modules::ModulePath::root());
                        if let Some(root_record) = self.module_registry.get(&root_pkg_id) {
                            let root_pkg_ref = root_record.object;
                            let namespace_sym = self.interner.intern(resolved_proj.namespace.as_str());
                            let manifest_ref = crate::modules::reflection_cache::ReflectionCache::get_or_create_project_manifest(self, manifest);
                            let identity_ref = crate::modules::reflection_cache::ReflectionCache::get_or_create_project_identity(self, &resolved_proj.name);
                            let dev_entry = manifest.entry.as_ref().map(|entry| {
                                let entry_mod_id = phalcom_modules::ModuleId::resolved(
                                    pid,
                                    phalcom_modules::ModulePath::from_components(
                                        entry
                                            .split('.')
                                            .skip(1)
                                            .filter_map(|c| phalcom_modules::ModuleComponent::from_identifier(c).ok())
                                            .collect::<Vec<_>>(),
                                    ),
                                );
                                crate::modules::reflection_cache::ReflectionCache::get_or_create_module_identity(self, &entry_mod_id)
                            });

                            let mut dep_values: Vec<crate::value::Value> = Vec::new();
                            for (comp, (orig_alias, spec)) in &manifest.dependencies {
                                let req_desc = match spec {
                                    phalcom_modules::manifest::DependencySpec::Package { package, version } => {
                                        phalcom_modules::package_info::PackageRequirementDescriptor {
                                            alias: comp.as_str().to_string().into_boxed_str(),
                                            package: package.clone(),
                                            version_requirement: version.clone(),
                                            optional: false,
                                        }
                                    }
                                    phalcom_modules::manifest::DependencySpec::Path { .. } => phalcom_modules::package_info::PackageRequirementDescriptor {
                                        alias: comp.as_str().to_string().into_boxed_str(),
                                        package: orig_alias.clone(),
                                        version_requirement: "*".to_string(),
                                        optional: false,
                                    },
                                };
                                let req_ref = crate::modules::reflection_cache::ReflectionCache::get_or_create_package_requirement(self, &req_desc);
                                let pkg_info_desc = phalcom_modules::package_info::PackageInfoDescriptor::standalone(comp.as_str());
                                let pkg_info_ref = crate::modules::reflection_cache::ReflectionCache::get_or_create_package_info(self, &pkg_info_desc);
                                let dep_alias = self.interner.intern(comp.as_str());
                                let origin_sym = self.interner.intern("#workspace");

                                let dep_obj = self.heap.alloc(crate::heap::Object::ResolvedProjectDependency(Box::new(
                                    crate::heap::reflection::ResolvedProjectDependencyObject {
                                        alias: dep_alias,
                                        requirement: Some(req_ref),
                                        package_info: pkg_info_ref,
                                        root_package: root_pkg_ref,
                                        origin_sym,
                                    },
                                )));
                                dep_values.push(crate::value::Value::obj(dep_obj));
                            }
                            let deps_tuple = self.heap.alloc(crate::heap::Object::Tuple(crate::heap::TupleObject::positional(dep_values)));

                            let proj_obj = crate::modules::reflection_cache::ReflectionCache::get_or_create_project(
                                self,
                                &resolved_proj.name,
                                namespace_sym,
                                manifest_ref,
                                root_pkg_ref,
                                deps_tuple,
                                dev_entry,
                                identity_ref,
                            );
                            crate::value::Value::obj(proj_obj).wrap_some()?
                        } else {
                            crate::value::Value::none()
                        }
                    } else {
                        crate::value::Value::none()
                    }
                } else {
                    crate::value::Value::none()
                }
            } else {
                crate::value::Value::none()
            };
            self.define_global(obj_ref, project_sym, project_val)?;
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
                        if symbol_id.module.project.as_builtin().is_some() {
                            if let Some(class_id) = self.resolve_builtin_class_name(&symbol_id.name) {
                                self.heap.module_mut(target_obj).set_global(slot, crate::value::Value::obj(class_id))?;
                            }
                        }
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
                        if symbol.module.project.as_builtin().is_some() {
                            if let Some(class_id) = self.resolve_builtin_class_name(&symbol.name) {
                                self.heap.module_mut(target_mod_obj).set_global(slot, crate::value::Value::obj(class_id))?;
                            }
                        }
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

        // Phase 8: Load immutable semantic metadata pool into typing registry if present.
        if let Some(ref metadata_bundle) = program.semantic_metadata {
            let pool = crate::typing::loader::load_metadata_bundle(
                crate::typing::handle::MetadataPoolId(0),
                metadata_bundle.clone(),
                &phalcom_type_meta::validate::ValidationLimits::default(),
            )?;
            self.typing_registry.register_pool(pool);

            // Register nominal class bindings for all universe/core classes
            for binding in phalcom_native_meta::UNIVERSE_BINDINGS.iter() {
                let class_id = self.universe.classes.resolve(binding.key);
                let decl_ref = phalcom_type_meta::identity::StableDeclarationRef {
                    module: phalcom_type_meta::identity::StableModuleRef {
                        project: phalcom_type_meta::identity::StableProjectRef::Builtin {
                            namespace: "std".into(),
                            version: "0.1.0".into(),
                        },
                        path: Box::new([binding.name.into()]),
                    },
                    path: Box::new([binding.name.into()]),
                };
                self.typing_registry.register_nominal_binding(decl_ref, class_id);
            }
        }

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
                self.compiler_error(err, obj_ref, source_id);
            })?;
        self.heap.module_mut(obj_ref).closure = Some(closure);
        Ok(closure)
    }
}
