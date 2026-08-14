//! One coherent document and semantic generation for an editor request.

use std::sync::Arc;

use tower_lsp::lsp_types::Url;

use crate::documents::DocumentSnapshot;
use crate::semantic::{FileSemanticSnapshot, FileRevision, ModuleId, SemanticSnapshot};

/// Immutable request inputs pinned at handler entry.
#[derive(Clone)]
pub struct RequestContext {
    /// Current open-document data, including parsed source and line index.
    pub document: DocumentSnapshot,
    /// One published semantic generation used for the full request.
    pub semantic: Arc<SemanticSnapshot>,
    /// Published module identity corresponding to the request URI.
    pub module: Option<ModuleId>,
}

impl RequestContext {
    /// Pins one document and one semantic generation.
    pub fn new(document: DocumentSnapshot, semantic: Arc<SemanticSnapshot>, uri: &Url) -> Self {
        let module = semantic.module_for_uri(uri).cloned();
        Self { document, semantic, module }
    }

    /// Returns current-file semantic products only when their source revision
    /// matches the live document revision.
    pub fn exact_file(&self) -> Option<&FileSemanticSnapshot> {
        let module = self.module.as_ref()?;
        let file = self.semantic.files.get(module)?;
        (file.revision == self.document.revision).then_some(file.as_ref())
    }

    /// Returns whether published source products are stale for this request.
    pub fn is_stale(&self) -> bool {
        self.exact_file().is_none()
    }

    /// Returns pinned published revision when one exists.
    pub fn published_revision(&self) -> Option<FileRevision> {
        self.module.as_ref().and_then(|module| self.semantic.file_revision(module))
    }
}
