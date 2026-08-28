//! Immutable document and canonical semantic snapshot pinned for one request.

use std::sync::Arc;

use phalcom_modules::ModuleId;
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

/// Immutable request inputs pinned for one request.
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
    pub fn new(document: DocumentSnapshot, compiler: Option<Arc<SemanticSnapshot>>, uri: &Url) -> Self {
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
    let source_text = snapshot
        .sources
        .get(module)
        .map(|source| source.text.as_ref())
        .or_else(|| snapshot.presentation_source(module));
    let Some(source_text) = source_text else {
        return SourceMatch::Unmapped;
    };
    if source_text == document.text.as_ref() {
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
        let source = crate::source_transport::source_id_for_uri(uri)?;
        return snapshot.module_for_source(&source).cloned();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::DocumentStore;
    use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
    use phalcom_semantic::SemanticWorkspaceSession;
    use std::path::PathBuf;

    fn source() -> (Url, SourceLocation) {
        let path = PathBuf::from("/workspace/request.ph");
        let uri = Url::from_file_path(&path).expect("test source URI");
        let location = SourceLocation {
            source_id: SourceId(path.to_string_lossy().into()),
            display_path: path,
        };
        (uri, location)
    }

    fn published_snapshot(location: SourceLocation, text: &str) -> Arc<SemanticSnapshot> {
        let mut session = SemanticWorkspaceSession::new();
        session
            .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
                source: location,
                text: Arc::from(text),
                revision: SourceRevision(1),
                recovered_program: None,
            }])
            .expect("canonical source publication");
        session.last_snapshot().cloned().expect("published snapshot")
    }

    fn context(uri: &Url, text: &str, snapshot: Arc<SemanticSnapshot>) -> RequestContext {
        let documents = DocumentStore::new();
        documents.open_or_update(uri.clone(), text.to_string());
        RequestContext::new(documents.snapshot(uri).expect("open test document"), Some(snapshot), uri)
    }

    #[test]
    fn exact_source_allows_canonical_semantic_requests() {
        let (uri, location) = source();
        let snapshot = published_snapshot(location, "class Request {}\n");
        let request = context(&uri, "class Request {}\n", snapshot);
        assert_eq!(request.source_match, SourceMatch::Exact);
        assert!(!request.is_stale());
        assert!(request.compiler_module().is_some());
    }

    #[test]
    fn stale_source_fails_closed() {
        let (uri, location) = source();
        let snapshot = published_snapshot(location, "class Request {}\n");
        let request = context(&uri, "class Changed {}\n", snapshot);
        assert_eq!(request.source_match, SourceMatch::Stale);
        assert!(request.is_stale());
    }

    #[test]
    fn unmapped_source_has_no_canonical_module() {
        let (_mapped_uri, location) = source();
        let snapshot = published_snapshot(location, "class Request {}\n");
        let unmapped_uri = Url::parse("file:///workspace/other.ph").expect("unmapped URI");
        let request = context(&unmapped_uri, "class Other {}\n", snapshot);
        assert_eq!(request.source_match, SourceMatch::Unmapped);
        assert!(request.is_stale());
        assert!(request.compiler_module().is_none());
    }

    #[test]
    fn compiler_core_presentation_text_is_an_exact_source() {
        let (_uri, location) = source();
        let snapshot = published_snapshot(location, "class Request {}\n");
        let core = ModuleId::core();
        let text = snapshot
            .presentation_source(&core)
            .expect("semantic publication must retain canonical core presentation text")
            .to_owned();
        let core_uri = Url::parse(crate::core_documents::CORE_MODULE_URI).expect("core URI");

        let request = context(&core_uri, &text, snapshot);
        assert_eq!(request.compiler_module(), Some(&core));
        assert_eq!(
            request.source_match,
            SourceMatch::Exact,
            "the compiler's own presentation source must be coherent with the pinned semantic snapshot"
        );
    }
}
