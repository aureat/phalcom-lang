//! Import edges and lightweight module-path resolution.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use phalcom_ast::ast::{DependencyDecl, ImportDecl, ImportPath, Program};
use phalcom_common::range::SourceRange;
use tower_lsp::lsp_types::Url;

use super::ids::{DocumentModuleMap, ModuleId};

/// Worker-side adapter over the compiler's project/module resolver.
///
/// Request handlers continue to consume immutable URI-keyed snapshots. This
/// adapter belongs to ingestion/rebuild code, where filesystem/project
/// resolution is allowed, and records the same semantic `ModuleId` mapping
/// used by compiler linking.
pub struct SharedModuleResolver<'a, P: phalcom_modules::SourceProvider> {
    resolver: phalcom_modules::ModuleResolver<'a, P>,
    documents: DocumentModuleMap,
}

impl<'a, P: phalcom_modules::SourceProvider> SharedModuleResolver<'a, P> {
    /// Creates a resolver adapter for one project universe and source provider.
    pub fn new(universe: &'a phalcom_modules::ProjectUniverse, source: &'a P, documents: DocumentModuleMap) -> Self {
        Self {
            resolver: phalcom_modules::ModuleResolver::new(universe, source),
            documents,
        }
    }

    /// Returns the document/semantic identity map maintained by the adapter.
    pub fn documents(&self) -> &DocumentModuleMap {
        &self.documents
    }

    /// Resolves one AST import through `phalcom-modules` and returns the
    /// document-bound LSP key for graph publication.
    pub fn resolve(&mut self, importer: &ModuleId, path: &ImportPath) -> Result<ModuleId, phalcom_modules::ModuleResolutionError> {
        let importer_semantic = self
            .documents
            .semantic_for_lsp(importer)
            .ok_or_else(|| phalcom_modules::ModuleResolutionError::ModuleNotFound(importer.to_string()))?
            .clone();
        let unit = self.resolver.resolve_import(&importer_semantic, path)?;
        let target_uri = Url::from_file_path(&unit.source.display_path)
            .map_err(|_| phalcom_modules::ModuleResolutionError::ModuleNotFound(unit.source.display_path.display().to_string()))?;
        let target_id = unit.id.clone();
        self.documents.insert(target_uri, target_id.clone());
        Ok(ModuleId::new(target_id.to_string()))
    }
}

/// Source-level dependency kind retained by the LSP graph.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ImportEdgeKind {
    /// Whole module object import.
    WholeModule,
    /// Selective value import.
    Selective,
    /// Public re-export.
    ReExport,
}

/// Reverse dependency policy used by invalidation consumers.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReverseDependencyKind {
    /// Interface/name-resolution dependency.
    Interface,
    /// Eager runtime dependency.
    Runtime,
    /// Public re-export dependency.
    ReExport,
}

/// One source import edge, retained even when its target is unresolved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportEdge {
    /// Importing module.
    pub from: ModuleId,
    /// Local module binding.
    pub binding: String,
    /// Source-relative import path, retained so unresolved edges can be
    /// repaired when a provider file is created later.
    pub path: String,
    /// Resolved target, if its source file exists.
    pub target: Option<ModuleId>,
    /// Semantic dependency kind.
    pub kind: ImportEdgeKind,
    /// Phase currently required by v1 semantics.
    pub phase: phalcom_modules::DependencyPhase,
    /// Source span of the import statement.
    pub source_range: SourceRange,
}

/// Current import graph for indexed modules.
#[derive(Clone, Debug, Default)]
pub struct ModuleGraph {
    /// Forward imports contributed by each module.
    forward: BTreeMap<ModuleId, Vec<ImportEdge>>,
    /// Reverse importer index keyed by resolved target module.
    reverse: BTreeMap<ModuleId, BTreeSet<ModuleId>>,
    /// Reverse index of retained import candidates, including unresolved
    /// edges. This lets provider creation/removal repair only possible
    /// importers instead of rescanning every module.
    candidates: BTreeMap<ModuleId, BTreeSet<ModuleId>>,
}

impl ModuleGraph {
    /// Replaces all import edges contributed by `module`.
    pub fn update(&mut self, module: ModuleId, program: &Program, available: &BTreeSet<ModuleId>) {
        self.remove_edges(&module);
        let mut edges = Vec::new();
        for dep in &program.preamble.dependencies {
            match dep {
                phalcom_ast::ast::DependencyDecl::Import(imp) => match imp {
                    phalcom_ast::ast::ImportDecl::Module(m) => {
                        let binding = if let Some(alias) = &m.alias {
                            alias.name.clone()
                        } else if m.path.segments.is_empty() {
                            match &m.path.root {
                                phalcom_ast::ast::ImportRoot::Absolute(seg) => seg.name.clone(),
                                phalcom_ast::ast::ImportRoot::Relative { .. } => String::new(),
                            }
                        } else {
                            m.path.segments.last().unwrap().name.clone()
                        };
                        let path_str = m.path.to_string();
                        edges.push(ImportEdge {
                            from: module.clone(),
                            binding,
                            path: path_str.clone(),
                            target: resolve_import(&module, &path_str, available),
                            kind: ImportEdgeKind::WholeModule,
                            phase: phalcom_modules::DependencyPhase::Runtime,
                            source_range: m.range,
                        });
                    }
                    phalcom_ast::ast::ImportDecl::Selective(s) => {
                        let path_str = s.path.to_string();
                        let target = resolve_import(&module, &path_str, available);
                        for item in &s.items {
                            let binding = if let Some(alias) = &item.alias {
                                alias.name.clone()
                            } else {
                                item.name.clone()
                            };
                            edges.push(ImportEdge {
                                from: module.clone(),
                                binding,
                                path: path_str.clone(),
                                target: target.clone(),
                                kind: ImportEdgeKind::Selective,
                                phase: phalcom_modules::DependencyPhase::Runtime,
                                source_range: item.range,
                            });
                        }
                    }
                },
                phalcom_ast::ast::DependencyDecl::ReExport(r) => {
                    let path_str = r.path.to_string();
                    let target = resolve_import(&module, &path_str, available);
                    for item in &r.items {
                        edges.push(ImportEdge {
                            from: module.clone(),
                            binding: item.local_or_remote_name.clone(),
                            path: path_str.clone(),
                            target: target.clone(),
                            kind: ImportEdgeKind::ReExport,
                            phase: phalcom_modules::DependencyPhase::Runtime,
                            source_range: item.range,
                        });
                    }
                }
                phalcom_ast::ast::DependencyDecl::Expose(_) => {}
            }
        }
        for edge in &edges {
            for candidate in import_candidates(&module, &edge.path) {
                self.candidates.entry(candidate).or_default().insert(module.clone());
            }
            if let Some(target) = &edge.target {
                self.reverse.entry(target.clone()).or_default().insert(module.clone());
            }
        }
        self.forward.insert(module, edges);
    }

    /// Rebuilds one module's graph edges through the shared project resolver.
    /// This is intended for worker/source-ingestion paths; unresolved imports
    /// remain an explicit resolver error rather than being guessed from URIs.
    pub fn update_with_shared_resolver(
        &mut self,
        module: ModuleId,
        program: &Program,
        resolver: &mut SharedModuleResolver<'_, impl phalcom_modules::SourceProvider>,
    ) -> Result<(), phalcom_modules::ModuleResolutionError> {
        self.remove_edges(&module);
        let mut edges = Vec::new();
        for dependency in &program.preamble.dependencies {
            match dependency {
                DependencyDecl::Import(import) => match import {
                    ImportDecl::Module(decl) => {
                        let target = resolver.resolve(&module, &decl.path)?;
                        let binding = decl
                            .alias
                            .as_ref()
                            .map(|alias| alias.name.clone())
                            .or_else(|| decl.path.segments.last().map(|segment| segment.name.clone()))
                            .unwrap_or_default();
                        edges.push(ImportEdge {
                            from: module.clone(),
                            binding,
                            path: decl.path.to_string(),
                            target: Some(target),
                            kind: ImportEdgeKind::WholeModule,
                            phase: phalcom_modules::DependencyPhase::Runtime,
                            source_range: decl.range,
                        });
                    }
                    ImportDecl::Selective(decl) => {
                        let target = resolver.resolve(&module, &decl.path)?;
                        for item in &decl.items {
                            edges.push(ImportEdge {
                                from: module.clone(),
                                binding: item.alias.as_ref().map(|alias| alias.name.clone()).unwrap_or_else(|| item.name.clone()),
                                path: decl.path.to_string(),
                                target: Some(target.clone()),
                                kind: ImportEdgeKind::Selective,
                                phase: phalcom_modules::DependencyPhase::Runtime,
                                source_range: item.range,
                            });
                        }
                    }
                },
                DependencyDecl::ReExport(decl) => {
                    let target = resolver.resolve(&module, &decl.path)?;
                    for item in &decl.items {
                        edges.push(ImportEdge {
                            from: module.clone(),
                            binding: item.local_or_remote_name.clone(),
                            path: decl.path.to_string(),
                            target: Some(target.clone()),
                            kind: ImportEdgeKind::ReExport,
                            phase: phalcom_modules::DependencyPhase::Runtime,
                            source_range: item.range,
                        });
                    }
                }
                DependencyDecl::Expose(_) => {}
            }
        }
        for edge in &edges {
            self.reverse
                .entry(edge.target.clone().expect("shared resolution produces target"))
                .or_default()
                .insert(module.clone());
        }
        self.forward.insert(module, edges);
        Ok(())
    }

    /// Removes all edges contributed by `module`.
    pub fn remove(&mut self, module: &ModuleId) {
        self.remove_edges(module);
        self.forward.remove(module);
        for importers in self.reverse.values_mut() {
            importers.remove(module);
        }
        for importers in self.candidates.values_mut() {
            importers.remove(module);
        }
        self.reverse.retain(|_, importers| !importers.is_empty());
        self.candidates.retain(|_, importers| !importers.is_empty());
    }

    /// Returns imports declared by `module`.
    pub fn imports(&self, module: &ModuleId) -> &[ImportEdge] {
        self.forward.get(module).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Returns modules whose imports point at `target`.
    pub fn dependents_of(&self, target: &ModuleId) -> Vec<ModuleId> {
        self.reverse.get(target).into_iter().flatten().cloned().collect()
    }

    /// Returns dependent modules filtered by semantic reverse-edge kind.
    pub fn dependents_of_kind(&self, target: &ModuleId, kind: ReverseDependencyKind) -> Vec<ModuleId> {
        self.forward
            .values()
            .flat_map(|edges| edges.iter())
            .filter(|edge| {
                edge.target.as_ref() == Some(target)
                    && match kind {
                        ReverseDependencyKind::Interface => edge.phase == phalcom_modules::DependencyPhase::InterfaceOnly,
                        ReverseDependencyKind::Runtime => edge.phase == phalcom_modules::DependencyPhase::Runtime,
                        ReverseDependencyKind::ReExport => edge.kind == ImportEdgeKind::ReExport,
                    }
            })
            .map(|edge| edge.from.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Repairs importers whose retained path can resolve to `provider`.
    /// Returns only importers whose resolved edge changed.
    pub fn repair_provider(&mut self, provider: &ModuleId, available: &BTreeSet<ModuleId>) -> Vec<ModuleId> {
        let modules = self.candidates.get(provider).cloned().unwrap_or_default();
        let mut changed = BTreeSet::new();
        for module in modules {
            let Some(edges) = self.forward.get_mut(&module) else { continue };
            for edge in edges {
                if !import_candidates(&module, &edge.path).iter().any(|candidate| candidate == provider) {
                    continue;
                }
                let target = available.contains(provider).then(|| provider.clone());
                if edge.target == target {
                    continue;
                }
                if let Some(old_target) = std::mem::replace(&mut edge.target, target.clone())
                    && let Some(importers) = self.reverse.get_mut(&old_target)
                {
                    importers.remove(&module);
                }
                if let Some(target) = target {
                    self.reverse.entry(target).or_default().insert(module.clone());
                }
                changed.insert(module.clone());
            }
        }
        self.reverse.retain(|_, importers| !importers.is_empty());
        changed.into_iter().collect()
    }

    /// Compatibility helper for callers that still need a complete repair.
    /// New mutation paths should call [`Self::repair_provider`] per changed
    /// provider.
    pub fn refresh_resolutions(&mut self, available: &BTreeSet<ModuleId>) -> Vec<ModuleId> {
        let providers = self.candidates.keys().cloned().collect::<Vec<_>>();
        let mut changed = BTreeSet::new();
        for provider in providers {
            changed.extend(self.repair_provider(&provider, available));
        }
        changed.into_iter().collect()
    }

    fn remove_edges(&mut self, module: &ModuleId) {
        let Some(edges) = self.forward.get(module).cloned() else { return };
        for edge in edges {
            for candidate in import_candidates(module, &edge.path) {
                if let Some(importers) = self.candidates.get_mut(&candidate) {
                    importers.remove(module);
                    if importers.is_empty() {
                        self.candidates.remove(&candidate);
                    }
                }
            }
            if let Some(target) = &edge.target {
                if let Some(importers) = self.reverse.get_mut(target) {
                    importers.remove(module);
                    if importers.is_empty() {
                        self.reverse.remove(target);
                    }
                }
            }
        }
    }

    /// Returns all transitive import dependents of `target`.
    pub fn dependent_closure(&self, target: &ModuleId) -> Vec<ModuleId> {
        let mut pending = vec![target.clone()];
        let mut seen = std::collections::BTreeSet::new();
        while let Some(module) = pending.pop() {
            for dependent in self.dependents_of(&module) {
                if seen.insert(dependent.clone()) {
                    pending.push(dependent);
                }
            }
        }
        seen.into_iter().collect()
    }
}

fn resolve_import(module: &ModuleId, import: &str, available: &BTreeSet<ModuleId>) -> Option<ModuleId> {
    for candidate in import_candidates(module, import) {
        if available.contains(&candidate) {
            return Some(candidate);
        }
        let canonical = Url::parse(candidate.as_str())
            .ok()
            .and_then(|uri| uri.to_file_path().ok())
            .and_then(|path| path.canonicalize().ok())
            .and_then(|path| Url::from_file_path(path).ok())
            .map(|uri| ModuleId::new(uri.to_string()));
        if canonical.as_ref().is_some_and(|id| available.contains(id)) {
            return canonical;
        }
    }
    None
}

fn import_candidates(module: &ModuleId, import: &str) -> Vec<ModuleId> {
    // TODO(module-path-common): extract this VM-free normalization into phalcom-common for compiler and LSP reuse.
    let Some(uri) = Url::parse(module.as_str()).ok() else { return Vec::new() };
    let Ok(source) = uri.to_file_path() else { return Vec::new() };
    let dot_count = import.bytes().take_while(|byte| *byte == b'.').count();
    if dot_count == 0 {
        // Absolute logical roots need ProjectUniverse context. The Part I LSP
        // seam only resolves relative paths against the importing document.
        return Vec::new();
    }
    let Some(logical_path) = import.get(dot_count..) else { return Vec::new() };
    if logical_path.is_empty() {
        return Vec::new();
    }
    let Some(parent) = source.parent() else { return Vec::new() };
    let mut base = parent.to_path_buf();
    for _ in 1..dot_count {
        base.pop();
    }
    let segments: Vec<&str> = logical_path.split('.').collect();
    if segments.iter().any(|segment| segment.is_empty()) {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for kebab in [false, true] {
        let mut candidate = base.clone();
        for segment in &segments {
            let segment = if kebab { segment.replace('_', "-") } else { (*segment).to_string() };
            candidate.push(segment);
        }
        if candidate.extension().is_none() {
            candidate.set_extension("ph");
        }
        let normalized = normalize_path(candidate);
        if let Ok(uri) = Url::from_file_path(normalized) {
            let id = ModuleId::new(uri.to_string());
            if !candidates.contains(&id) {
                candidates.push(id);
            }
        }
    }
    candidates
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {}
            component => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

#[allow(dead_code)]
fn _path(_: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::parser::parse;

    #[test]
    fn unresolved_import_stays_in_graph() {
        let program = parse("import .missing as Missing\n", 0).program;
        let module = ModuleId::new("file:///tmp/main.ph");
        let mut graph = ModuleGraph::default();
        graph.update(module.clone(), &program, &BTreeSet::from([module.clone()]));
        assert_eq!(graph.imports(&module).len(), 1);
        assert!(graph.imports(&module)[0].target.is_none());
    }

    #[test]
    fn existing_ph_extension_is_preserved() {
        let main = ModuleId::new("file:///tmp/main.ph");
        let provider = ModuleId::new("file:///tmp/provider.ph");
        let program = parse("import .provider as Provider\n", 0).program;
        let mut graph = ModuleGraph::default();
        graph.update(main.clone(), &program, &BTreeSet::from([main.clone(), provider.clone()]));
        assert_eq!(graph.imports(&main)[0].target, Some(provider));
    }

    #[test]
    fn provider_repair_returns_only_retained_candidate_importers() {
        let provider = ModuleId::new("file:///tmp/provider.ph");
        let other = ModuleId::new("file:///tmp/other.ph");
        let consumer = ModuleId::new("file:///tmp/consumer.ph");
        let unrelated = ModuleId::new("file:///tmp/unrelated.ph");
        let mut graph = ModuleGraph::default();
        graph.update(
            consumer.clone(),
            &parse("import .provider as Provider\n", 0).program,
            &BTreeSet::from([consumer.clone()]),
        );
        graph.update(
            unrelated.clone(),
            &parse("import .other as Other\n", 0).program,
            &BTreeSet::from([unrelated.clone()]),
        );

        let affected = graph.repair_provider(&provider, &BTreeSet::from([consumer.clone(), provider.clone(), other]));
        assert_eq!(affected, vec![consumer.clone()]);
        assert_eq!(graph.dependents_of(&provider), vec![consumer]);
        assert!(graph.dependents_of(&ModuleId::new("file:///tmp/other.ph")).is_empty());
    }

    #[test]
    fn replacing_imports_removes_stale_reverse_edges() {
        let main = ModuleId::new("file:///tmp/main.ph");
        let first = ModuleId::new("file:///tmp/first.ph");
        let second = ModuleId::new("file:///tmp/second.ph");
        let available = BTreeSet::from([main.clone(), first.clone(), second.clone()]);
        let mut graph = ModuleGraph::default();
        graph.update(main.clone(), &parse("import .first as First\n", 0).program, &available);
        graph.update(main.clone(), &parse("import .second as Second\n", 0).program, &available);
        assert!(graph.dependents_of(&first).is_empty());
        assert_eq!(graph.dependents_of(&second), vec![main]);
    }
}
