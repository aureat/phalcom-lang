//! Static symbol/import linking for a resolved module universe.

use crate::error::ModuleGraphError;
use crate::graph::{
    DependencyPhase, ModuleGraphs, ReferenceEdge, ReferenceKind, RuntimeDependencyEdge, RuntimeDependencyReason, SemanticEdge, SemanticEdgeKind, SemanticNodeId,
};
use crate::identity::{ModuleId, ModulePath};
use crate::interface::{ImportSurface, LinkedExport, LinkedModuleInterface, UnlinkedExportTarget, UnlinkedModuleInterface};
use crate::project::ProjectUniverse;
use phalcom_ast::ast::{ImportPath, StaticSymbolRef};
use phalcom_common::range::SourceRange;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use thiserror::Error;

/// Semantic identity of a module-owned global declaration.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SymbolId {
    /// Module that owns the declaration.
    pub module: ModuleId,
    /// Declaration name in that module.
    pub name: Box<str>,
}

/// A target used by a linked read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkedReadSpec {
    /// Read the materialized module object.
    Module(ModuleId),
    /// Read a mutable global slot owned by the canonical symbol.
    Binding(SymbolId),
}

/// Linked local binding layout. IDs are symbolic and VM-independent.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModuleBindingLayout {
    /// Module-owned globals.
    pub local_globals: BTreeMap<Box<str>, GlobalBindingId>,
    /// Immutable imported bindings.
    pub imports: BTreeMap<Box<str>, ImportBindingId>,
}

/// Index of a local global in a linked module layout.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GlobalBindingId(pub u32);

/// Index of an imported linked read in a linked module layout.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ImportBindingId(pub u32);

/// A linked module and its VM-independent reads/dependencies.
#[derive(Clone, Debug, PartialEq)]
pub struct LinkedModule {
    /// Canonical public interface.
    pub interface: LinkedModuleInterface,
    /// Local/import binding names.
    pub bindings: ModuleBindingLayout,
    /// Indexed reads emitted by a compiler.
    pub linked_reads: Vec<LinkedReadSpec>,
    /// Runtime dependencies in initialization order-independent form.
    pub runtime_dependencies: Vec<ModuleId>,
}

/// A closed, statically linked program plan.
#[derive(Clone, Debug)]
pub struct LinkedProgram {
    /// Resolved project/package universe.
    pub universe: Arc<ProjectUniverse>,
    /// Every linked reachable module.
    pub modules: BTreeMap<ModuleId, LinkedModule>,
    /// Reference, semantic, and runtime graphs.
    pub graphs: ModuleGraphs,
    /// Selected entry module.
    pub entry: ModuleId,
    /// Deterministic runtime initialization order.
    pub initialization_order: Vec<ModuleId>,
}

/// Linker failure with source ownership retained for diagnostics.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum LinkError {
    /// A path used by an import was not resolved before linking.
    #[error("unresolved import '{path}' from module {module}")]
    UnresolvedImport { module: ModuleId, path: String, range: SourceRange },
    /// A selected name is absent from a target interface.
    #[error("module {module} does not export '{name}'")]
    MissingExport { module: ModuleId, name: String, range: SourceRange },
    /// A local name is not declared or imported.
    #[error("module {module} has no binding '{name}'")]
    MissingBinding { module: ModuleId, name: String, range: SourceRange },
    /// A linked name collides with another top-level binding.
    #[error("module {module} has colliding binding '{name}'")]
    BindingCollision { module: ModuleId, name: String, range: SourceRange },
    /// The selected entry or a statically reached module is absent from the
    /// interface universe supplied to the linker.
    #[error("module {module} is absent from the link universe")]
    MissingModule { module: ModuleId },
    /// A re-export loop cannot produce a canonical symbol.
    #[error("cyclic re-export involving module {module} and '{name}'")]
    CyclicReExport { module: ModuleId, name: String },
    /// Runtime graph validation failed.
    #[error(transparent)]
    RuntimeCycle(#[from] ModuleGraphError),
}

/// Static linker over source-local interfaces and pre-resolved import targets.
pub struct ModuleLinker {
    universe: Arc<ProjectUniverse>,
    interfaces: BTreeMap<ModuleId, UnlinkedModuleInterface>,
}

impl ModuleLinker {
    /// Creates a linker. The interface map is the complete source universe for
    /// this linking generation; no source is parsed during linking.
    pub fn new(universe: Arc<ProjectUniverse>, interfaces: BTreeMap<ModuleId, UnlinkedModuleInterface>) -> Self {
        Self { universe, interfaces }
    }

    /// Links all interfaces using `(importer, logical path)` resolutions.
    ///
    /// Resolution remains a separate phase so filesystem/package policy cannot
    /// accidentally leak into symbol linking.
    pub fn link(&self, entry: ModuleId, resolved: &BTreeMap<(ModuleId, String), ModuleId>) -> Result<LinkedProgram, LinkError> {
        self.link_inner(entry, resolved, false)
    }

    /// Links the currently available source universe while retaining modules
    /// whose imports are temporarily unresolved. This is used by persistent
    /// workspace updates so a deleted dependency can publish the importer and
    /// its semantic diagnostics instead of retaining the removed module in the
    /// last-known-good snapshot.
    pub fn link_with_unresolved_imports(&self, entry: ModuleId, resolved: &BTreeMap<(ModuleId, String), ModuleId>) -> Result<LinkedProgram, LinkError> {
        self.link_inner(entry, resolved, true)
    }

    /// Links every interface supplied to this linker, retaining the complete
    /// source universe for workspace-wide semantic analysis.
    pub fn link_all(&self, entry: ModuleId, resolved: &BTreeMap<(ModuleId, String), ModuleId>) -> Result<LinkedProgram, LinkError> {
        let mut context = LinkContext::new(self, resolved, false);
        context.build().map(|(modules, graphs, initialization_order)| LinkedProgram {
            universe: self.universe.clone(),
            modules,
            graphs,
            entry,
            initialization_order,
        })
    }

    fn link_inner(
        &self,
        entry: ModuleId,
        resolved: &BTreeMap<(ModuleId, String), ModuleId>,
        allow_unresolved_imports: bool,
    ) -> Result<LinkedProgram, LinkError> {
        let reachable = self.reachable_interfaces(&entry, resolved, allow_unresolved_imports)?;
        let reachable_interfaces = self
            .interfaces
            .iter()
            .filter(|(module, _)| reachable.contains(*module))
            .map(|(module, interface)| (module.clone(), interface.clone()))
            .collect();
        let reachable_linker = ModuleLinker::new(self.universe.clone(), reachable_interfaces);
        let mut context = LinkContext::new(&reachable_linker, resolved, allow_unresolved_imports);
        context.build().map(|(modules, graphs, initialization_order)| LinkedProgram {
            universe: self.universe.clone(),
            modules,
            graphs,
            entry,
            initialization_order,
        })
    }

    fn reachable_interfaces(
        &self,
        entry: &ModuleId,
        resolved: &BTreeMap<(ModuleId, String), ModuleId>,
        allow_unresolved_imports: bool,
    ) -> Result<BTreeSet<ModuleId>, LinkError> {
        if !self.interfaces.contains_key(entry) {
            return Err(LinkError::MissingModule { module: entry.clone() });
        }
        let mut reachable = BTreeSet::from([entry.clone()]);
        let mut pending = vec![entry.clone()];

        if let Some(project_id) = entry.project.as_resolved() {
            let root_id = ModuleId::resolved(project_id, ModulePath::root());
            if self.interfaces.contains_key(&root_id) && reachable.insert(root_id.clone()) {
                pending.push(root_id);
            }
        } else if let Some(sid) = entry.project.as_synthetic() {
            let root_id = ModuleId::synthetic(sid, ModulePath::root());
            if self.interfaces.contains_key(&root_id) && reachable.insert(root_id.clone()) {
                pending.push(root_id);
            }
        }

        while let Some(module) = pending.pop() {
            let interface = self.interfaces.get(&module).expect("reachable interface exists");
            for import in &interface.imports {
                let path = match import {
                    ImportSurface::Module(decl) => (&decl.path, decl.range),
                    ImportSurface::Selective(decl) => (&decl.path, decl.range),
                    ImportSurface::ReExport(decl) => (&decl.path, decl.range),
                };
                let Some(target) = resolved
                    .get(&(module.clone(), path.0.to_string()))
                    .cloned()
                    .filter(|target| self.interfaces.contains_key(target))
                else {
                    if allow_unresolved_imports {
                        continue;
                    }
                    return Err(LinkError::UnresolvedImport {
                        module: module.clone(),
                        path: path.0.to_string(),
                        range: path.1,
                    });
                };
                if reachable.insert(target.clone()) {
                    pending.push(target.clone());
                }
                if let Some(project_id) = target.project.as_resolved() {
                    let mut curr_path = target.path.parent();
                    while let Some(parent) = curr_path {
                        let pkg_id = ModuleId::resolved(project_id, parent.clone());
                        if self.interfaces.contains_key(&pkg_id) && reachable.insert(pkg_id.clone()) {
                            pending.push(pkg_id);
                        }
                        curr_path = parent.parent();
                    }
                    let root_id = ModuleId::resolved(project_id, ModulePath::root());
                    if self.interfaces.contains_key(&root_id) && reachable.insert(root_id.clone()) {
                        pending.push(root_id);
                    }
                }
            }
            for export in interface.exports.values() {
                let UnlinkedExportTarget::CanonicalDeclaration { module: target, .. } = &export.target else {
                    continue;
                };
                if !self.interfaces.contains_key(target) {
                    return Err(LinkError::MissingModule {
                        module: target.clone(),
                    });
                }
                if reachable.insert(target.clone()) {
                    pending.push(target.clone());
                }
            }
        }
        Ok(reachable)
    }

    /// Resolves a declaration-only static symbol through the current module's
    /// local declarations or a whole-module import alias.
    pub fn resolve_static_symbol(
        &self,
        module: &ModuleId,
        reference: &StaticSymbolRef,
        resolved: &BTreeMap<(ModuleId, String), ModuleId>,
    ) -> Result<SymbolId, LinkError> {
        let interface = self.interfaces.get(module).ok_or_else(|| LinkError::MissingBinding {
            module: module.clone(),
            name: reference.root.clone(),
            range: reference.range,
        })?;
        if reference.is_bare() {
            if interface.declarations.contains_key(&reference.root) {
                return Ok(SymbolId {
                    module: module.clone(),
                    name: reference.root.clone().into_boxed_str(),
                });
            }
            let mut context = LinkContext::new(self, resolved, false);
            context.collect_imports_and_graphs()?;
            for import in &interface.imports {
                let Some((path, remote, range)) = (match import {
                    ImportSurface::Selective(decl) => decl.items.iter().find_map(|item| {
                        let local = item.alias.as_ref().map(|alias| alias.name.as_str()).unwrap_or(item.name.as_str());
                        (local == reference.root).then_some((&decl.path, item.name.as_str(), item.range))
                    }),
                    ImportSurface::ReExport(decl) => decl.items.iter().find_map(|item| {
                        let local = item.local_or_remote_name.as_str();
                        (local == reference.root).then_some((&decl.path, item.local_or_remote_name.as_str(), item.range))
                    }),
                    ImportSurface::Module(_) => None,
                }) else {
                    continue;
                };
                let target = context.target(module, path, range)?;
                let export = context.resolve_export(&target, remote)?;
                let symbol = export.symbol().cloned().ok_or_else(|| LinkError::MissingBinding {
                    module: target.clone(),
                    name: remote.to_string(),
                    range,
                })?;
                return Ok(symbol);
            }
            return Err(LinkError::MissingBinding {
                module: module.clone(),
                name: reference.root.clone(),
                range: reference.root_range,
            });
        }

        let import = interface
            .imports
            .iter()
            .find_map(|import| {
                let ImportSurface::Module(decl) = import else { return None };
                let binding = decl
                    .alias
                    .as_ref()
                    .map(|alias| alias.name.as_str())
                    .or_else(|| default_module_binding(&decl.path))?;
                (binding == reference.root).then_some(decl)
            })
            .ok_or_else(|| LinkError::MissingBinding {
                module: module.clone(),
                name: reference.root.clone(),
                range: reference.root_range,
            })?;
        let target = resolved
            .get(&(module.clone(), import.path.to_string()))
            .cloned()
            .ok_or_else(|| LinkError::UnresolvedImport {
                module: module.clone(),
                path: import.path.to_string(),
                range: import.range,
            })?;
        if reference.members.len() != 1 {
            return Err(LinkError::MissingBinding {
                module: target,
                name: reference.leaf_name().to_string(),
                range: reference.range,
            });
        }
        let name = reference.leaf_name();
        let mut context = LinkContext::new(self, resolved, false);
        context.collect_imports_and_graphs()?;
        let export = context.resolve_export(&target, name)?;
        let symbol = export.symbol().cloned().ok_or_else(|| LinkError::MissingBinding {
            module: target,
            name: name.to_string(),
            range: reference.range,
        })?;
        Ok(symbol)
    }
}

struct LinkContext<'a> {
    linker: &'a ModuleLinker,
    resolved: &'a BTreeMap<(ModuleId, String), ModuleId>,
    allow_unresolved_imports: bool,
    import_targets: BTreeMap<(ModuleId, String), LinkedReadSpec>,
    import_symbols: BTreeMap<(ModuleId, String), Option<SymbolId>>,
    linked_exports: BTreeMap<(ModuleId, String), LinkedExport>,
    resolving_exports: BTreeSet<(ModuleId, String)>,
    graphs: ModuleGraphs,
}

type LinkBuild = (BTreeMap<ModuleId, LinkedModule>, ModuleGraphs, Vec<ModuleId>);

impl<'a> LinkContext<'a> {
    fn new(linker: &'a ModuleLinker, resolved: &'a BTreeMap<(ModuleId, String), ModuleId>, allow_unresolved_imports: bool) -> Self {
        Self {
            linker,
            resolved,
            allow_unresolved_imports,
            import_targets: BTreeMap::new(),
            import_symbols: BTreeMap::new(),
            linked_exports: BTreeMap::new(),
            resolving_exports: BTreeSet::new(),
            graphs: ModuleGraphs::default(),
        }
    }

    fn build(&mut self) -> Result<LinkBuild, LinkError> {
        self.collect_imports_and_graphs()?;
        let module_ids = self.linker.interfaces.keys().cloned().collect::<Vec<_>>();
        for module in &module_ids {
            let names = self
                .linker
                .interfaces
                .get(module)
                .map(|interface| interface.exports.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            for name in names {
                self.resolve_export(module, &name)?;
            }
        }

        let initialization_order = self.graphs.runtime.initialization_order()?;
        let mut modules = BTreeMap::new();
        for module in module_ids {
            let interface = self.linked_interface(&module)?;
            let mut bindings = ModuleBindingLayout::default();
            let mut linked_reads = Vec::new();
            if let Some(unlinked) = self.linker.interfaces.get(&module) {
                for name in unlinked.declarations.keys() {
                    let next = bindings.local_globals.len() as u32;
                    bindings.local_globals.insert(name.clone().into_boxed_str(), GlobalBindingId(next));
                }
                for (local, read) in self
                    .import_targets
                    .iter()
                    .filter_map(|((owner, local), read)| (owner == &module).then_some((local, read)))
                {
                    let index = linked_reads.len() as u32;
                    linked_reads.push(read.clone());
                    bindings.imports.insert(local.clone().into_boxed_str(), ImportBindingId(index));
                }
            }
            let mut runtime_dependencies = self
                .graphs
                .runtime
                .edges_from(&module)
                .iter()
                .map(|edge| edge.dependency.clone())
                .collect::<Vec<_>>();
            runtime_dependencies.sort();
            runtime_dependencies.dedup();
            modules.insert(
                module,
                LinkedModule {
                    interface,
                    bindings,
                    linked_reads,
                    runtime_dependencies,
                },
            );
        }
        Ok((modules, std::mem::take(&mut self.graphs), initialization_order))
    }

    fn collect_imports_and_graphs(&mut self) -> Result<(), LinkError> {
        for (module, interface) in &self.linker.interfaces {
            self.graphs.references.add_node(module.clone());
            self.graphs.runtime.add_node(module.clone());
            let mut reference_edges = Vec::new();
            for import in &interface.imports {
                match import {
                    ImportSurface::Module(decl) => {
                        let Some(target) = self.target_if_present(module, &decl.path, decl.range)? else {
                            continue;
                        };
                        reference_edges.push(ReferenceEdge {
                            from: module.clone(),
                            to: target.clone(),
                            kind: ReferenceKind::WholeModuleImport,
                            range: decl.range,
                        });
                        self.graphs.semantics.add(SemanticEdge {
                            from: SemanticNodeId::Module(module.clone()),
                            to: SemanticNodeId::Module(target.clone()),
                            kind: SemanticEdgeKind::ModuleInterface,
                            range: decl.range,
                        });
                        self.graphs.runtime.add(RuntimeDependencyEdge {
                            importer: module.clone(),
                            dependency: target.clone(),
                            range: decl.range,
                            reason: RuntimeDependencyReason::WholeModuleImport,
                        });
                        let local = decl
                            .alias
                            .as_ref()
                            .map(|alias| alias.name.clone())
                            .or_else(|| default_module_binding(&decl.path).map(str::to_owned))
                            .unwrap_or_else(|| target.path.to_string());
                        self.add_import(module, local, LinkedReadSpec::Module(target), None, decl.range)?;
                    }
                    ImportSurface::Selective(decl) => {
                        let Some(target) = self.target_if_present(module, &decl.path, decl.range)? else {
                            continue;
                        };
                        for item in &decl.items {
                            reference_edges.push(ReferenceEdge {
                                from: module.clone(),
                                to: target.clone(),
                                kind: ReferenceKind::SelectiveImport,
                                range: item.range,
                            });
                            self.graphs.semantics.add(SemanticEdge {
                                from: SemanticNodeId::Module(module.clone()),
                                to: SemanticNodeId::Module(target.clone()),
                                kind: SemanticEdgeKind::ModuleInterface,
                                range: item.range,
                            });
                            self.graphs.runtime.add(RuntimeDependencyEdge {
                                importer: module.clone(),
                                dependency: target.clone(),
                                range: item.range,
                                reason: RuntimeDependencyReason::SelectiveValueImport,
                            });
                            let local = item.alias.as_ref().map(|alias| alias.name.clone()).unwrap_or_else(|| item.name.clone());
                            let provisional = SymbolId {
                                module: target.clone(),
                                name: item.name.clone().into_boxed_str(),
                            };
                            self.add_import(module, local, LinkedReadSpec::Binding(provisional.clone()), Some(provisional), item.range)?;
                        }
                    }
                    ImportSurface::ReExport(decl) => {
                        let Some(target) = self.target_if_present(module, &decl.path, decl.range)? else {
                            continue;
                        };
                        for item in &decl.items {
                            reference_edges.push(ReferenceEdge {
                                from: module.clone(),
                                to: target.clone(),
                                kind: ReferenceKind::ReExport,
                                range: item.range,
                            });
                            self.graphs.semantics.add(SemanticEdge {
                                from: SemanticNodeId::Module(module.clone()),
                                to: SemanticNodeId::Module(target.clone()),
                                kind: SemanticEdgeKind::ModuleInterface,
                                range: item.range,
                            });
                            self.graphs.runtime.add(RuntimeDependencyEdge {
                                importer: module.clone(),
                                dependency: target.clone(),
                                range: item.range,
                                reason: RuntimeDependencyReason::ReExport,
                            });
                            let local = item.local_or_remote_name.clone();
                            let provisional = SymbolId {
                                module: target.clone(),
                                name: item.local_or_remote_name.clone().into_boxed_str(),
                            };
                            self.add_import(module, local, LinkedReadSpec::Binding(provisional.clone()), Some(provisional), item.range)?;
                        }
                    }
                }
            }
            self.graphs.references.replace(module.clone(), reference_edges);
        }
        // All local import names now exist, so canonicalize selected imports
        // and re-exports in a second pass. This keeps interface-map iteration
        // order from affecting linked symbol identity.
        for (module, interface) in &self.linker.interfaces {
            for import in &interface.imports {
                match import {
                    ImportSurface::Selective(decl) => {
                        let Some(target) = self.target_if_present(module, &decl.path, decl.range)? else {
                            continue;
                        };
                        for item in &decl.items {
                            let local = item.alias.as_ref().map(|alias| alias.name.clone()).unwrap_or_else(|| item.name.clone());
                            let linked_exp = self.resolve_export(&target, &item.name)?;
                            match &linked_exp.target {
                                crate::interface::LinkedExportTarget::Binding(symbol) => {
                                    self.import_targets
                                        .insert((module.clone(), local.clone()), LinkedReadSpec::Binding(symbol.clone()));
                                    self.import_symbols.insert((module.clone(), local), Some(symbol.clone()));
                                }
                                crate::interface::LinkedExportTarget::Module(mod_id) => {
                                    self.import_targets
                                        .insert((module.clone(), local.clone()), LinkedReadSpec::Module(mod_id.clone()));
                                    self.import_symbols.insert((module.clone(), local), None);
                                }
                            }
                        }
                    }
                    ImportSurface::ReExport(decl) => {
                        let Some(target) = self.target_if_present(module, &decl.path, decl.range)? else {
                            continue;
                        };
                        for item in &decl.items {
                            let local = item.local_or_remote_name.clone();
                            let linked_exp = self.resolve_export(&target, &item.local_or_remote_name)?;
                            match &linked_exp.target {
                                crate::interface::LinkedExportTarget::Binding(symbol) => {
                                    self.import_targets
                                        .insert((module.clone(), local.clone()), LinkedReadSpec::Binding(symbol.clone()));
                                    self.import_symbols.insert((module.clone(), local), Some(symbol.clone()));
                                }
                                crate::interface::LinkedExportTarget::Module(mod_id) => {
                                    self.import_targets
                                        .insert((module.clone(), local.clone()), LinkedReadSpec::Module(mod_id.clone()));
                                    self.import_symbols.insert((module.clone(), local), None);
                                }
                            }
                        }
                    }
                    ImportSurface::Module(_) => {}
                }
            }
        }
        Ok(())
    }

    fn add_import(&mut self, module: &ModuleId, local: String, target: LinkedReadSpec, symbol: Option<SymbolId>, range: SourceRange) -> Result<(), LinkError> {
        let key = (module.clone(), local.clone());
        if self.import_targets.contains_key(&key) {
            return Err(LinkError::BindingCollision {
                module: module.clone(),
                name: local,
                range,
            });
        }
        self.import_targets.insert(key.clone(), target);
        self.import_symbols.insert(key, symbol);
        Ok(())
    }

    fn target(&self, module: &ModuleId, path: &ImportPath, range: SourceRange) -> Result<ModuleId, LinkError> {
        self.resolved
            .get(&(module.clone(), path.to_string()))
            .cloned()
            .filter(|target| self.linker.interfaces.contains_key(target))
            .ok_or_else(|| LinkError::UnresolvedImport {
                module: module.clone(),
                path: path.to_string(),
                range,
            })
    }

    fn target_if_present(&self, module: &ModuleId, path: &ImportPath, range: SourceRange) -> Result<Option<ModuleId>, LinkError> {
        match self.target(module, path, range) {
            Ok(target) => Ok(Some(target)),
            Err(LinkError::UnresolvedImport { .. }) if self.allow_unresolved_imports => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn resolve_export(&mut self, module: &ModuleId, name: &str) -> Result<LinkedExport, LinkError> {
        let key = (module.clone(), name.to_string());
        if let Some(export) = self.linked_exports.get(&key) {
            return Ok(export.clone());
        }
        if !self.resolving_exports.insert(key.clone()) {
            return Err(LinkError::CyclicReExport {
                module: module.clone(),
                name: name.to_string(),
            });
        }
        let interface = self.linker.interfaces.get(module).ok_or_else(|| LinkError::MissingExport {
            module: module.clone(),
            name: name.to_string(),
            range: SourceRange::default(),
        })?;
        let surface = interface.exports.get(name).ok_or_else(|| LinkError::MissingExport {
            module: module.clone(),
            name: name.to_string(),
            range: SourceRange::default(),
        })?;
        let (target, range) = match &surface.target {
            UnlinkedExportTarget::Local(local) => {
                if let Some(declaration) = interface.declarations.get(local) {
                    (
                        crate::interface::LinkedExportTarget::Binding(SymbolId {
                            module: module.clone(),
                            name: local.clone().into_boxed_str(),
                        }),
                        declaration.range,
                    )
                } else if let Some(Some(symbol)) = self.import_symbols.get(&(module.clone(), local.clone())) {
                    (crate::interface::LinkedExportTarget::Binding(symbol.clone()), surface.range)
                } else if let Some(LinkedReadSpec::Module(target_mod)) = self.import_targets.get(&(module.clone(), local.clone())) {
                    (crate::interface::LinkedExportTarget::Module(target_mod.clone()), surface.range)
                } else {
                    return Err(LinkError::MissingBinding {
                        module: module.clone(),
                        name: local.clone(),
                        range: surface.range,
                    });
                }
            }
            UnlinkedExportTarget::ReExport { path, remote } => {
                let target_mod = self.target(module, path, surface.range)?;
                let linked = self.resolve_export(&target_mod, remote)?;
                (linked.target, surface.range)
            }
            UnlinkedExportTarget::CanonicalDeclaration { module: target_module, name } => {
                let target_interface = self.linker.interfaces.get(target_module).ok_or_else(|| LinkError::MissingModule {
                    module: target_module.clone(),
                })?;
                if !target_interface.declarations.contains_key(name) {
                    return Err(LinkError::MissingBinding {
                        module: target_module.clone(),
                        name: name.clone(),
                        range: surface.range,
                    });
                }
                (
                    crate::interface::LinkedExportTarget::Binding(SymbolId {
                        module: target_module.clone(),
                        name: name.clone().into_boxed_str(),
                    }),
                    surface.range,
                )
            }
        };
        self.resolving_exports.remove(&key);
        let linked = LinkedExport {
            public_name: name.to_owned().into_boxed_str(),
            target,
            range,
        };
        self.linked_exports.insert(key, linked.clone());
        Ok(linked)
    }

    fn linked_interface(&mut self, module: &ModuleId) -> Result<LinkedModuleInterface, LinkError> {
        let unlinked = self.linker.interfaces.get(module).ok_or_else(|| LinkError::MissingExport {
            module: module.clone(),
            name: "<module>".to_string(),
            range: SourceRange::default(),
        })?;
        let mut exports = BTreeMap::new();
        for name in unlinked.exports.keys() {
            let linked = self.resolve_export(module, name)?;
            exports.insert(name.clone().into_boxed_str(), linked);
        }
        Ok(LinkedModuleInterface {
            module: module.clone(),
            kind: unlinked.kind,
            exports,
            metadata: unlinked.metadata.clone(),
        })
    }
}

/// Classifies a source reference into the strongest dependency phase needed by
/// its current v1 runtime representation.
pub fn dependency_phase(kind: ReferenceKind) -> DependencyPhase {
    match kind {
        ReferenceKind::InterfaceOnly => DependencyPhase::InterfaceOnly,
        ReferenceKind::WholeModuleImport | ReferenceKind::SelectiveImport | ReferenceKind::ReExport => DependencyPhase::Runtime,
    }
}

/// Helper for constructing a project-relative path resolution table.
pub fn resolution_key(module: &ModuleId, path: &ImportPath) -> (ModuleId, String) {
    (module.clone(), path.to_string())
}

/// Returns a human-readable module path for diagnostics that do not have a
/// source provider available.
pub fn module_path(module: &ModuleId) -> String {
    ModulePath::from_components(module.path.components().to_vec()).to_string()
}

fn default_module_binding(path: &ImportPath) -> Option<&str> {
    path.segments.last().map(|segment| segment.name.as_str()).or(match &path.root {
        phalcom_ast::ast::ImportRoot::Absolute(segment) => Some(segment.name.as_str()),
        phalcom_ast::ast::ImportRoot::Relative { .. } => None,
    })
}
