//! Immutable document and canonical semantic snapshot pinned for one request.

use std::sync::Arc;

use phalcom_modules::{ModuleId, SourceId, SourceRevision};
use phalcom_semantic::SemanticSnapshot;
use tower_lsp::lsp_types::Url;

use crate::documents::DocumentSnapshot;

/// Relationship between live document text and the pinned compiler source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceMatch {
    /// Canonical source text matches the document at request time.
    Exact,
    /// Canonical source exists, but describes older text.
    Stale,
    /// No canonical module/source is published for this URI.
    Unmapped,
}

/// Immutable request inputs pinned at handler entry.
#[derive(Clone)]
pub struct RequestContext {
    /// URI used by the protocol request.
    pub uri: Url,
    /// Current open-document data, including recovered syntax and line index.
    pub document: DocumentSnapshot,
    /// One canonical immutable semantic snapshot, when published.
    pub compiler: Option<Arc<SemanticSnapshot>>,
    /// Canonical module identity corresponding to the request URI.
    pub canonical_module: Option<ModuleId>,
    /// Source coherence classification used to suppress stale semantic ranges.
    pub source_match: SourceMatch,
}

impl RequestContext {
    /// Pins document data and one canonical snapshot for the request lifetime.
    pub fn new_with_compiler(document: DocumentSnapshot, compiler: Option<Arc<SemanticSnapshot>>, uri: &Url) -> Self {
        let canonical_module = compiler.as_deref().and_then(|snapshot| canonical_module_for_uri(snapshot, uri));
        let source_match = classify_source(&document, compiler.as_deref(), canonical_module.as_ref());
        Self {
            uri: uri.clone(),
            document,
            compiler,
            canonical_module,
            source_match,
        }
    }

    /// Returns the canonical module identity for this request.
    pub fn compiler_module(&self) -> Option<&ModuleId> {
        self.canonical_module.as_ref()
    }

    /// Whether canonical products are unavailable or stale for this document.
    pub fn is_stale(&self) -> bool {
        !matches!(self.source_match, SourceMatch::Exact)
    }
}

fn classify_source(document: &DocumentSnapshot, snapshot: Option<&SemanticSnapshot>, module: Option<&ModuleId>) -> SourceMatch {
    let Some(snapshot) = snapshot else {
        return SourceMatch::Unmapped;
    };
    let Some(module) = module else {
        return SourceMatch::Unmapped;
    };
    let Some(source) = snapshot.sources.get(module) else {
        return SourceMatch::Unmapped;
    };
    if source.text.as_ref() == document.text.as_ref() {
        SourceMatch::Exact
    } else {
        SourceMatch::Stale
    }
}

fn canonical_module_for_uri(snapshot: &SemanticSnapshot, uri: &Url) -> Option<ModuleId> {
    if let Ok(path) = uri.to_file_path()
        && let Some(module) = snapshot.module_for_display_path(&path)
    {
        return Some(module.clone());
    }
    if uri.as_str() == crate::core_documents::CORE_MODULE_URI {
        return Some(ModuleId::core());
    }
    if let Some(module) = crate::analysis_service::builtin_module_from_uri(uri)
        && snapshot.sources.contains_key(&module)
    {
        return Some(module);
    }
    if uri.scheme() == "file" {
        let source = SourceId(uri.to_file_path().ok()?.to_string_lossy().into());
        return snapshot.module_for_source(&source).cloned();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_modules::{SourceLocation, WorkspaceSourceMutation};
    use phalcom_semantic::SemanticWorkspaceSession;
    use std::path::PathBuf;

    #[test]
    fn old_request_keeps_immutable_compiler_snapshot_after_new_publication() {
        let path = PathBuf::from(format!("/tmp/phalcom-request-context-{}.ph", std::process::id()));
        let uri = Url::from_file_path(&path).expect("test path must be a file URI");
        let location = SourceLocation {
            source_id: SourceId(path.to_string_lossy().into()),
            display_path: path,
        };
        let mut session = SemanticWorkspaceSession::new();
        let first = session
            .apply_module_mutation(WorkspaceSourceMutation::SetOverlay {
                source: location.clone(),
                text: Arc::from("class Main { old() {} }\n"),
                revision: SourceRevision(1),
            })
            .expect("first publication");
        let old = first.snapshot;
        let old_id = old.id;
        let old_text = old.sources.values().next().expect("source publication").text.clone();

        let document = crate::documents::Document::new_with_revision("class Main { old() {} }\n".to_string(), SourceRevision(1));
        let document = crate::documents::DocumentSnapshot {
            text: document.text,
            parse: document.parse,
            line_index: document.line_index,
            revision: document.revision,
            version: document.version,
        };
        let request = RequestContext::new_with_compiler(document, Some(old.clone()), &uri);
        let reader = std::thread::spawn(move || {
            let pinned = request.compiler.expect("request must retain compiler snapshot");
            (pinned.id, pinned.sources.values().next().expect("pinned source").text.clone())
        });

        let second = session
            .apply_module_mutation(WorkspaceSourceMutation::SetOverlay {
                source: location,
                text: Arc::from("class Main { new() {} }\n"),
                revision: SourceRevision(2),
            })
            .expect("second publication");
        assert_ne!(second.snapshot.id, old_id);

        let (pinned_id, pinned_text) = reader.join().expect("reader must complete");
        assert_eq!(pinned_id, old_id);
        assert_eq!(pinned_text, old_text);
    }
}
