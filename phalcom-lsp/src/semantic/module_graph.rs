//! Import edges and lightweight module-path resolution.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use phalcom_ast::ast::{Program, Statement};
use phalcom_common::range::SourceRange;
use tower_lsp::lsp_types::Url;

use super::ids::ModuleId;

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
}

impl ModuleGraph {
    /// Replaces all import edges contributed by `module`.
    pub fn update(&mut self, module: ModuleId, program: &Program, available: &BTreeSet<ModuleId>) {
        self.remove_reverse_edges(&module);
        let edges: Vec<ImportEdge> = program
            .statements
            .iter()
            .filter_map(|statement| {
                let Statement::Import(import) = statement else { return None };
                Some(ImportEdge {
                    from: module.clone(),
                    binding: import.binding.clone(),
                    path: import.path.clone(),
                    target: resolve_import(&module, &import.path, available),
                    source_range: import.range,
                })
            })
            .collect();
        for edge in &edges {
            if let Some(target) = &edge.target {
                self.reverse.entry(target.clone()).or_default().insert(module.clone());
            }
        }
        self.forward.insert(module, edges);
    }

    /// Removes all edges contributed by `module`.
    pub fn remove(&mut self, module: &ModuleId) {
        self.remove_reverse_edges(module);
        self.forward.remove(module);
        for importers in self.reverse.values_mut() {
            importers.remove(module);
        }
    }

    /// Returns imports declared by `module`.
    pub fn imports(&self, module: &ModuleId) -> &[ImportEdge] {
        self.forward.get(module).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Returns modules whose imports point at `target`.
    pub fn dependents_of(&self, target: &ModuleId) -> Vec<ModuleId> {
        self.reverse.get(target).into_iter().flatten().cloned().collect()
    }

    /// Re-resolves imports after a file is created, removed, or moved.
    /// Returns importers whose resolved target changed.
    pub fn refresh_resolutions(&mut self, available: &BTreeSet<ModuleId>) -> Vec<ModuleId> {
        let mut changed = Vec::new();
        for (module, edges) in &mut self.forward {
            for edge in edges {
                let old_target = edge.target.clone();
                let target = resolve_import(module, &edge.path, available);
                if edge.target != target {
                    edge.target = target;
                    changed.push(module.clone());
                    if let Some(old_target) = old_target {
                        if let Some(importers) = self.reverse.get_mut(&old_target) {
                            importers.remove(module);
                        }
                    }
                    if let Some(target) = &edge.target {
                        self.reverse.entry(target.clone()).or_default().insert(module.clone());
                    }
                }
            }
        }
        changed
    }

    fn remove_reverse_edges(&mut self, module: &ModuleId) {
        let Some(edges) = self.forward.get(module) else { return };
        for edge in edges {
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
    let candidate = import_candidate(module, import)?;
    Some(candidate).filter(|id| available.contains(id))
}

fn import_candidate(module: &ModuleId, import: &str) -> Option<ModuleId> {
    // TODO(module-path-common): extract this VM-free normalization into phalcom-common for compiler and LSP reuse.
    let uri = Url::parse(module.as_str()).ok()?;
    let source = uri.to_file_path().ok()?;
    let mut candidate = source.parent()?.join(import);
    if candidate.extension().is_none() {
        candidate.set_extension("ph");
    }
    let normalized = normalize_path(candidate);
    Url::from_file_path(normalized).ok().map(|uri| ModuleId::from_uri(&uri))
}

fn normalize_path(path: PathBuf) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }
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
        let program = parse("import \"./missing\" as Missing\n", 0).program;
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
        let program = parse("import \"./provider.ph\" as Provider\n", 0).program;
        let mut graph = ModuleGraph::default();
        graph.update(main.clone(), &program, &BTreeSet::from([main.clone(), provider.clone()]));
        assert_eq!(graph.imports(&main)[0].target, Some(provider));
    }
}
