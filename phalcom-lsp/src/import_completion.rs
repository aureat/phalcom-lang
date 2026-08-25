//! Canonical module and import completion (Task 11 / DEC-INTEG-011).

use phalcom_modules::identity::{ImportRootTarget, ModuleComponent, ModuleId, ModulePath, ProjectIdentity};
use phalcom_modules::interface::LinkedExportTarget;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat, Url};

use crate::semantic::SemanticSnapshot;

/// Position context within an import or exposure declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportContext {
    /// Top-level import root: `import |` or `import st|`
    ImportRoot {
        /// Partial name typed so far
        partial: String,
    },
    /// Child module path: `import app.shapes.|`
    ImportChild {
        /// Root project or package name
        root: String,
        /// Path segments inside the root
        segments: Vec<String>,
        /// Partial child name typed so far
        partial: String,
    },
    /// Relative module path: `import .shapes.|` or `from . import |`
    RelativeChild {
        /// Number of leading dots in the relative path
        parent_dots: usize,
        /// Segments following the leading dots
        segments: Vec<String>,
        /// Partial child name typed so far
        partial: String,
    },
    /// Selective export member: `from app.shapes.circle import |`
    SelectiveExport {
        /// Root project or package name
        root: String,
        /// Path segments reaching the target module
        segments: Vec<String>,
        /// Partial export name typed so far
        partial: String,
    },
}

/// Detects if cursor is in an import statement and extracts its structural context.
pub fn detect_import_context(line_prefix: &str) -> Option<ImportContext> {
    let trimmed = line_prefix.trim_start();

    // 1. `from <path> import <item>`
    if let Some(rest) = trimmed.strip_prefix("from ") {
        if let Some((path_part, item_part)) = rest.split_once(" import ") {
            let path_segments: Vec<String> = path_part.trim().split('.').filter(|s| !s.is_empty()).map(String::from).collect();
            if let Some(root) = path_segments.first() {
                return Some(ImportContext::SelectiveExport {
                    root: root.clone(),
                    segments: path_segments[1..].to_vec(),
                    partial: item_part.trim().to_string(),
                });
            }
        }
    }

    // 2. `import <path>`
    if let Some(rest) = trimmed.strip_prefix("import ") {
        let path = rest.trim_start();
        if path.starts_with('.') {
            let dots = path.chars().take_while(|&c| c == '.').count();
            let after_dots = &path[dots..];
            let segments: Vec<String> = after_dots.split('.').filter(|s| !s.is_empty()).map(String::from).collect();
            let partial = if path.ends_with('.') {
                String::new()
            } else {
                segments.last().cloned().unwrap_or_default()
            };
            let segs = if path.ends_with('.') {
                segments
            } else if !segments.is_empty() {
                segments[..segments.len() - 1].to_vec()
            } else {
                Vec::new()
            };
            return Some(ImportContext::RelativeChild {
                parent_dots: dots,
                segments: segs,
                partial,
            });
        }

        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() <= 1 && !path.ends_with('.') {
            return Some(ImportContext::ImportRoot {
                partial: path.trim().to_string(),
            });
        }

        let root = parts[0].trim().to_string();
        let is_trailing_dot = path.ends_with('.');
        let (segs, partial) = if is_trailing_dot {
            let s: Vec<String> = parts[1..].iter().filter(|p| !p.is_empty()).map(|p| p.to_string()).collect();
            (s, String::new())
        } else {
            let s: Vec<String> = parts[1..parts.len() - 1].iter().map(|p| p.to_string()).collect();
            let p = parts.last().unwrap_or(&"").to_string();
            (s, p)
        };

        return Some(ImportContext::ImportChild { root, segments: segs, partial });
    }

    // 3. `expose .<path>`
    if let Some(rest) = trimmed.strip_prefix("expose ") {
        let path = rest.trim_start();
        let dots = path.chars().take_while(|&c| c == '.').count();
        let after_dots = &path[dots..];
        let segments: Vec<String> = after_dots.split('.').filter(|s| !s.is_empty()).map(String::from).collect();
        let partial = if path.ends_with('.') {
            String::new()
        } else {
            segments.last().cloned().unwrap_or_default()
        };
        let segs = if path.ends_with('.') {
            segments
        } else if !segments.is_empty() {
            segments[..segments.len() - 1].to_vec()
        } else {
            Vec::new()
        };
        return Some(ImportContext::RelativeChild {
            parent_dots: dots.max(1),
            segments: segs,
            partial,
        });
    }

    None
}

/// Computes module and import completion items for an import context.
pub fn import_completions(snapshot: &SemanticSnapshot, uri: &Url, context: &ImportContext) -> Vec<CompletionItem> {
    let Some(static_snap) = &snapshot.static_snapshot else {
        return Vec::new();
    };

    let importer_module = snapshot.documents.by_uri.get(uri);
    let facade = static_snap.module_queries();

    let default_importer = ModuleId::core();
    let importer = importer_module.unwrap_or(&default_importer);
    let mut items = Vec::new();

    match context {
        ImportContext::ImportRoot { partial } => {
            let roots = facade.import_root_entries(importer);
            for (comp, _target) in roots {
                let name = comp.as_str();
                if partial.is_empty() || name.starts_with(partial) {
                    items.push(CompletionItem {
                        label: name.to_string(),
                        kind: Some(CompletionItemKind::MODULE),
                        insert_text: Some(name.to_string()),
                        insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                        detail: Some("module import root".to_string()),
                        ..CompletionItem::default()
                    });
                }
            }
        }
        ImportContext::ImportChild { root, segments, partial } => {
            let roots = facade.import_root_entries(importer);
            if let Some(comp) = ModuleComponent::from_identifier(root).ok()
                && let Some(root_target) = roots.get(&comp)
            {
                let project = match root_target.target {
                    ImportRootTarget::Builtin(b) => ProjectIdentity::Builtin(b),
                    ImportRootTarget::Resolved(r) => ProjectIdentity::Resolved(r),
                };
                let path_components: Vec<ModuleComponent> = segments.iter().filter_map(|s| ModuleComponent::from_identifier(s).ok()).collect();
                let prefix = ModulePath::from_components(path_components);
                let children = if root_target.is_self {
                    facade.module_children(project, &prefix)
                } else {
                    facade.external_import_children(project, &prefix)
                };
                for child in children {
                    if let Some(last) = child.path.components().last() {
                        let name = last.as_str();
                        if partial.is_empty() || name.starts_with(partial) {
                            items.push(CompletionItem {
                                label: name.to_string(),
                                kind: Some(CompletionItemKind::MODULE),
                                insert_text: Some(name.to_string()),
                                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                                detail: Some(format!("module in {root}")),
                                ..CompletionItem::default()
                            });
                        }
                    }
                }
            }
        }
        ImportContext::RelativeChild {
            parent_dots,
            segments,
            partial,
        } => {
            let path_components: Vec<ModuleComponent> = segments.iter().filter_map(|s| ModuleComponent::from_identifier(s).ok()).collect();
            let prefix = match facade.resolve_relative_prefix(importer, *parent_dots, &path_components) {
                Ok(p) => p,
                Err(_) => ModulePath::from_components(path_components),
            };
            let children = facade.module_children(importer.project, &prefix);
            for child in children {
                if let Some(last) = child.path.components().last() {
                    let name = last.as_str();
                    if partial.is_empty() || name.starts_with(partial) {
                        items.push(CompletionItem {
                            label: name.to_string(),
                            kind: Some(CompletionItemKind::MODULE),
                            insert_text: Some(name.to_string()),
                            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                            detail: Some("relative module".to_string()),
                            ..CompletionItem::default()
                        });
                    }
                }
            }
        }
        ImportContext::SelectiveExport { root, segments, partial } => {
            let roots = facade.import_root_entries(importer);
            if let Some(comp) = ModuleComponent::from_identifier(root).ok()
                && let Some(root_target) = roots.get(&comp)
            {
                let project = match root_target.target {
                    ImportRootTarget::Builtin(b) => ProjectIdentity::Builtin(b),
                    ImportRootTarget::Resolved(r) => ProjectIdentity::Resolved(r),
                };
                let path_components: Vec<ModuleComponent> = segments.iter().filter_map(|s| ModuleComponent::from_identifier(s).ok()).collect();
                let target_mod = ModuleId {
                    project,
                    path: ModulePath::from_components(path_components),
                };
                if let Some(exports) = facade.public_exports(&target_mod) {
                    for (name, export) in exports {
                        if partial.is_empty() || name.starts_with(partial.as_str()) {
                            let kind = match &export.target {
                                LinkedExportTarget::Binding(_) => CompletionItemKind::CLASS,
                                LinkedExportTarget::Module(_) => CompletionItemKind::MODULE,
                            };
                            items.push(CompletionItem {
                                label: name.to_string(),
                                kind: Some(kind),
                                insert_text: Some(name.to_string()),
                                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                                detail: Some(format!("export from {root}")),
                                ..CompletionItem::default()
                            });
                        }
                    }
                }
            }
        }
    }

    items.sort_by(|a, b| a.label.cmp(&b.label));
    items.dedup_by(|a, b| a.label == b.label);
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_import_context_root() {
        assert_eq!(detect_import_context("import "), Some(ImportContext::ImportRoot { partial: String::new() }));
        assert_eq!(
            detect_import_context("import st"),
            Some(ImportContext::ImportRoot { partial: "st".to_string() })
        );
    }

    #[test]
    fn test_detect_import_context_child() {
        assert_eq!(
            detect_import_context("import app."),
            Some(ImportContext::ImportChild {
                root: "app".to_string(),
                segments: Vec::new(),
                partial: String::new(),
            })
        );
        assert_eq!(
            detect_import_context("import app.shapes.ci"),
            Some(ImportContext::ImportChild {
                root: "app".to_string(),
                segments: vec!["shapes".to_string()],
                partial: "ci".to_string(),
            })
        );
    }

    #[test]
    fn test_detect_import_context_relative() {
        assert_eq!(
            detect_import_context("import ."),
            Some(ImportContext::RelativeChild {
                parent_dots: 1,
                segments: Vec::new(),
                partial: String::new(),
            })
        );
        assert_eq!(
            detect_import_context("expose .shapes"),
            Some(ImportContext::RelativeChild {
                parent_dots: 1,
                segments: Vec::new(),
                partial: "shapes".to_string(),
            })
        );
    }

    #[test]
    fn test_detect_import_context_selective() {
        assert_eq!(
            detect_import_context("from app.shapes.circle import Point"),
            Some(ImportContext::SelectiveExport {
                root: "app".to_string(),
                segments: vec!["shapes".to_string(), "circle".to_string()],
                partial: "Point".to_string(),
            })
        );
    }
}
