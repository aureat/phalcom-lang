//! One coherent document and semantic generation for an editor request.

use std::sync::Arc;

use tower_lsp::lsp_types::Url;

use crate::documents::DocumentSnapshot;
use crate::semantic::{CompilerSemanticSnapshot, FileRevision, FileSemanticSnapshot, ModuleId, SemanticSnapshot};

/// Relationship between the live document and the pinned compiler source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceMatch {
    /// The compiler source text is the same as the document at request time.
    Exact,
    /// A compiler source exists, but it describes an older document revision.
    Stale,
    /// No compiler module/source is published for this URI.
    Unmapped,
}

/// Immutable request inputs pinned at handler entry.
#[derive(Clone)]
pub struct RequestContext {
    /// URI used to resolve the canonical compiler module.
    pub uri: Url,
    /// Current open-document data, including parsed source and line index.
    pub document: DocumentSnapshot,
    /// One published semantic generation used for the full request.
    pub semantic: Arc<SemanticSnapshot>,
    /// The exact compiler publication pinned for this request, when one is
    /// available. It is the semantic source of truth for all new consumers.
    pub compiler: Option<Arc<CompilerSemanticSnapshot>>,
    /// Published module identity corresponding to the request URI.
    pub module: Option<ModuleId>,
    /// Source coherence classification used to suppress stale ranges.
    pub source_match: SourceMatch,
}

impl RequestContext {
    /// Pins one document and one semantic generation.
    pub fn new(document: DocumentSnapshot, semantic: Arc<SemanticSnapshot>, uri: &Url) -> Self {
        let module = semantic.module_for_uri(uri).cloned();
        let compiler = semantic.compiler_snapshot.clone();
        let source_match = classify_source(&document, &semantic, compiler.as_deref(), uri);
        Self {
            uri: uri.clone(),
            document,
            semantic,
            compiler,
            module,
            source_match,
        }
    }

    /// Pins a separately published compiler handle alongside the protocol
    /// adapter snapshot. Both are captured at one request boundary.
    pub fn new_with_compiler(document: DocumentSnapshot, semantic: Arc<SemanticSnapshot>, compiler: Option<Arc<CompilerSemanticSnapshot>>, uri: &Url) -> Self {
        let module = semantic.module_for_uri(uri).cloned();
        let source_match = classify_source(&document, &semantic, compiler.as_deref(), uri);
        Self {
            uri: uri.clone(),
            document,
            semantic,
            compiler,
            module,
            source_match,
        }
    }

    /// Returns current-file semantic products only when their source revision
    /// matches the live document revision.
    pub fn exact_file(&self) -> Option<&FileSemanticSnapshot> {
        let module = self.module.as_ref()?;
        let file = self.semantic.file(module)?;
        (file.revision == self.document.revision).then_some(file)
    }

    /// Returns whether published source products are stale for this request.
    pub fn is_stale(&self) -> bool {
        !matches!(self.source_match, SourceMatch::Exact)
    }

    /// Returns pinned published revision when one exists.
    pub fn published_revision(&self) -> Option<FileRevision> {
        self.module.as_ref().and_then(|module| self.semantic.file_revision(module))
    }

    /// Returns the canonical compiler module identity for this request.
    pub fn compiler_module(&self) -> Option<&phalcom_modules::ModuleId> {
        self.semantic.documents.get_by_uri(&self.uri)
    }
}

fn classify_source(document: &DocumentSnapshot, semantic: &SemanticSnapshot, compiler: Option<&CompilerSemanticSnapshot>, uri: &Url) -> SourceMatch {
    let Some(module) = semantic.documents.get_by_uri(uri) else {
        return SourceMatch::Unmapped;
    };
    let Some(compiler) = compiler else {
        return SourceMatch::Unmapped;
    };
    let Some(source) = compiler.sources.get(module) else {
        return SourceMatch::Unmapped;
    };
    if source.text.as_ref() == document.text.as_ref() {
        SourceMatch::Exact
    } else {
        SourceMatch::Stale
    }
}
