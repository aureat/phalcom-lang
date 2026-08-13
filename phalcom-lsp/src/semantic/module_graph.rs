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
    /// Reverse index of retained import candidates, including unresolved
    /// edges. This lets provider creation/removal repair only possible
    /// importers instead of rescanning every module.
    candidates: BTreeMap<ModuleId, BTreeSet<ModuleId>>,
}

impl ModuleGraph {
    /// Replaces all import edges contributed by `module`.
    pub fn update(&mut self, module: ModuleId, program: &Program, available: &BTreeSet<ModuleId>) {
        self.remove_edges(&module);
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
            if let Some(candidate) = import_candidate(&module, &edge.path) {
                self.candidates.entry(candidate).or_default().insert(module.clone());
            }
            if let Some(target) = &edge.target {
                self.reverse.entry(target.clone()).or_default().insert(module.clone());
            }
        }
        self.forward.insert(module, edges);
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

    /// Repairs importers whose retained path can resolve to `provider`.
    /// Returns only importers whose resolved edge changed.
    pub fn repair_provider(&mut self, provider: &ModuleId, available: &BTreeSet<ModuleId>) -> Vec<ModuleId> {
        let modules = self.candidates.get(provider).cloned().unwrap_or_default();
        let mut changed = BTreeSet::new();
        for module in modules {
            let Some(edges) = self.forward.get_mut(&module) else { continue };
            for edge in edges {
                if import_candidate(&module, &edge.path).as_ref() != Some(provider) {
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
            if let Some(candidate) = import_candidate(module, &edge.path)
                && let Some(importers) = self.candidates.get_mut(&candidate)
            {
                importers.remove(module);
                if importers.is_empty() {
                    self.candidates.remove(&candidate);
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

    #[test]
    fn provider_repair_returns_only_retained_candidate_importers() {
        let provider = ModuleId::new("file:///tmp/provider.ph");
        let other = ModuleId::new("file:///tmp/other.ph");
        let consumer = ModuleId::new("file:///tmp/consumer.ph");
        let unrelated = ModuleId::new("file:///tmp/unrelated.ph");
        let mut graph = ModuleGraph::default();
        graph.update(
            consumer.clone(),
            &parse("import \"./provider\" as Provider\n", 0).program,
            &BTreeSet::from([consumer.clone()]),
        );
        graph.update(
            unrelated.clone(),
            &parse("import \"./other\" as Other\n", 0).program,
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
        graph.update(main.clone(), &parse("import \"./first\" as First\n", 0).program, &available);
        graph.update(main.clone(), &parse("import \"./second\" as Second\n", 0).program, &available);
        assert!(graph.dependents_of(&first).is_empty());
        assert_eq!(graph.dependents_of(&second), vec![main]);
    }
}
