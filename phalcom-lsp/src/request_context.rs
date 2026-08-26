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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::Document;
    use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceMutation};
    use phalcom_semantic::SemanticWorkspaceSession;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::thread;

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

        let document = Document::new_with_revision("class Main { old() {} }\n".to_string(), FileRevision(1));
        let document = DocumentSnapshot {
            text: document.text,
            parse: document.parse,
            line_index: document.line_index,
            revision: document.revision,
            version: document.version,
        };
        let request = RequestContext::new_with_compiler(
            document,
            Arc::new(SemanticSnapshot::default()),
            Some(old.clone()),
            &uri,
        );
        let reader = thread::spawn(move || {
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
