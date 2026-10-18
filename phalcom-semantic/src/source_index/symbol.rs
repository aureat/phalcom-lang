//! Protocol-neutral workspace symbol types and incrementally maintained search index.

use std::sync::Arc;

use crate::identity::{ModuleId, SemanticTargetId, SourceSiteId};
use crate::source_index::SourceIndexUpdateStats;

/// Protocol-neutral unique identifier for one workspace symbol entry.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorkspaceSymbolId {
    pub target: SemanticTargetId,
    pub site: SourceSiteId,
}

/// Category of a workspace symbol.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EditorSymbolKind {
    Module,
    Class,
    Data,
    TypeData,
    Trait,
    DataComponent,
    TypeAlias,
    Callable,
    Field,
    Variant,
    VariantField,
    Binding,
}

/// One compiler-owned workspace symbol entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceSymbolEntry {
    pub id: WorkspaceSymbolId,
    pub name: Box<str>,
    pub normalized_name: Box<str>,
    pub target: SemanticTargetId,
    pub declaration_site: SourceSiteId,
    pub kind: EditorSymbolKind,
    pub container_name: Option<Box<str>>,
}

/// Incrementally maintained persistent workspace symbol index.
#[derive(Clone, Debug, Default)]
pub struct WorkspaceSymbolIndex {
    entries: Option<im::OrdMap<WorkspaceSymbolId, Arc<WorkspaceSymbolEntry>>>,
    by_module: Option<im::OrdMap<ModuleId, Arc<[WorkspaceSymbolId]>>>,
    trigrams: Option<im::OrdMap<[char; 3], im::OrdSet<WorkspaceSymbolId>>>,
}

impl WorkspaceSymbolIndex {
    /// Creates an empty workspace symbol index.
    pub fn new() -> Self {
        Self {
            entries: None,
            by_module: None,
            trigrams: None,
        }
    }

    /// Number of indexed workspace symbols.
    pub fn len(&self) -> usize {
        self.entries.as_ref().map_or(0, |e| e.len())
    }

    /// Whether the index is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.as_ref().map_or(true, |e| e.is_empty())
    }

    /// Replaces or removes symbols contributed by one module.
    pub fn replace_module(&self, module: &ModuleId, old: &[WorkspaceSymbolEntry], new: &[WorkspaceSymbolEntry], stats: &mut SourceIndexUpdateStats) -> Self {
        stats.workspace_symbol_contributions_replaced += 1;
        stats.workspace_symbol_entries_removed += old.len();
        stats.workspace_symbol_entries_added += new.len();

        let mut entries = self.entries.clone().unwrap_or_default();
        let mut by_module = self.by_module.clone().unwrap_or_default();
        let mut trigrams = self.trigrams.clone().unwrap_or_default();

        // 1. Remove old entries
        for entry in old {
            entries.remove(&entry.id);
            for tri in trigrams_for(&entry.normalized_name) {
                if let Some(mut set) = trigrams.get(&tri).cloned() {
                    set.remove(&entry.id);
                    if set.is_empty() {
                        trigrams.remove(&tri);
                    } else {
                        trigrams.insert(tri, set);
                    }
                }
            }
        }

        // 2. Insert new entries
        let mut new_ids = Vec::with_capacity(new.len());
        for entry in new {
            let id = entry.id.clone();
            new_ids.push(id.clone());
            let entry_arc = Arc::new(entry.clone());
            entries.insert(id.clone(), entry_arc);
            for tri in trigrams_for(&entry.normalized_name) {
                let mut set = trigrams.get(&tri).cloned().unwrap_or_default();
                set.insert(id.clone());
                trigrams.insert(tri, set);
            }
        }

        if new_ids.is_empty() {
            by_module.remove(module);
        } else {
            by_module.insert(module.clone(), Arc::from(new_ids.into_boxed_slice()));
        }

        Self {
            entries: if entries.is_empty() { None } else { Some(entries) },
            by_module: if by_module.is_empty() { None } else { Some(by_module) },
            trigrams: if trigrams.is_empty() { None } else { Some(trigrams) },
        }
    }

    /// Searches symbols using case-insensitive substring matching.
    pub fn search(&self, query: &str, limit: usize) -> Vec<&WorkspaceSymbolEntry> {
        if limit == 0 {
            return Vec::new();
        }
        let (Some(entries), Some(trigrams)) = (&self.entries, &self.trigrams) else {
            return Vec::new();
        };
        let normalized_query = query.trim().to_lowercase();
        let chars: Vec<char> = normalized_query.chars().collect();

        if chars.len() >= 3 {
            // Use trigram intersection to prune candidate IDs
            let mut candidate_set: Option<im::OrdSet<WorkspaceSymbolId>> = None;
            for i in 0..=chars.len() - 3 {
                let tri = [chars[i], chars[i + 1], chars[i + 2]];
                if let Some(set) = trigrams.get(&tri) {
                    candidate_set = match candidate_set {
                        None => Some(set.clone()),
                        Some(current) => Some(current.intersection(set.clone())),
                    };
                } else {
                    return Vec::new();
                }
            }

            let mut results = Vec::new();
            if let Some(candidates) = candidate_set {
                for id in &candidates {
                    if let Some(entry) = entries.get(id) {
                        if entry.normalized_name.contains(&normalized_query) {
                            results.push(entry.as_ref());
                            if results.len() >= limit {
                                break;
                            }
                        }
                    }
                }
            }
            results
        } else if normalized_query.is_empty() {
            entries.values().take(limit).map(AsRef::as_ref).collect()
        } else {
            // Short query (1 or 2 chars): iterate indexed symbol entries directly without touching source shards
            let mut results = Vec::new();
            for entry in entries.values() {
                if entry.normalized_name.contains(&normalized_query) {
                    results.push(entry.as_ref());
                    if results.len() >= limit {
                        break;
                    }
                }
            }
            results
        }
    }
}

fn trigrams_for(s: &str) -> Vec<[char; 3]> {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 3 {
        return Vec::new();
    }
    let mut tris = Vec::with_capacity(chars.len() - 2);
    for i in 0..=chars.len() - 3 {
        tris.push([chars[i], chars[i + 1], chars[i + 2]]);
    }
    tris.sort();
    tris.dedup();
    tris
}
