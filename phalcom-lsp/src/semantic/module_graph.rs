//! Import edges and lightweight module-path resolution.

use std::collections::BTreeMap;
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
    /// Resolved target, if its source file exists.
    pub target: Option<ModuleId>,
    /// Source span of the import statement.
    pub source_range: SourceRange,
}

/// Current import graph for indexed modules.
#[derive(Clone, Debug, Default)]
pub struct ModuleGraph {
    edges: BTreeMap<ModuleId, Vec<ImportEdge>>,
}

impl ModuleGraph {
    /// Replaces all import edges contributed by `module`.
    pub fn update(&mut self, module: ModuleId, program: &Program) {
        let edges = program
            .statements
            .iter()
            .filter_map(|statement| {
                let Statement::Import(import) = statement else { return None };
                Some(ImportEdge {
                    from: module.clone(),
                    binding: import.binding.clone(),
                    target: resolve_import(&module, &import.path),
                    source_range: import.range,
                })
            })
            .collect();
        self.edges.insert(module, edges);
    }

    /// Removes all edges contributed by `module`.
    pub fn remove(&mut self, module: &ModuleId) {
        self.edges.remove(module);
    }

    /// Returns imports declared by `module`.
    pub fn imports(&self, module: &ModuleId) -> &[ImportEdge] {
        self.edges.get(module).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Returns modules whose imports point at `target`.
    pub fn dependents_of(&self, target: &ModuleId) -> Vec<ModuleId> {
        self.edges
            .iter()
            .filter(|(_, edges)| edges.iter().any(|edge| edge.target.as_ref() == Some(target)))
            .map(|(module, _)| module.clone())
            .collect()
    }
}

fn resolve_import(module: &ModuleId, import: &str) -> Option<ModuleId> {
    let uri = Url::parse(module.as_str()).ok()?;
    let source = uri.to_file_path().ok()?;
    let candidate = source.parent()?.join(import).with_extension("ph");
    if !candidate.is_file() {
        return None;
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
        graph.update(module.clone(), &program);
        assert_eq!(graph.imports(&module).len(), 1);
        assert!(graph.imports(&module)[0].target.is_none());
    }
}
