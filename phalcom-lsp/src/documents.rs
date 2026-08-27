//! Open-document store.
//!
//! Keeps, per open [`Url`], the current text, its cached
//! [`Parse`] result, and its cached
//! [`LineIndex`] — all three rebuilt together on every `didOpen`/`didChange`
//! (this server negotiates `textDocumentSync=Full`: each change replaces the
//! entire document text, so there is no incremental patch to apply here).
//!
//! Backed by a [`DashMap`] so `did_open`/`did_change`/`did_close` and any
//! future read-only capability (go-to-def, hover, …) can run concurrently
//! without a single global lock.

use std::sync::Arc;

use dashmap::DashMap;
use phalcom_ast::error::SyntaxError;
use phalcom_ast::parser::{self, Parse};
use tower_lsp::lsp_types::Url;

use crate::line_index::LineIndex;
use phalcom_modules::SourceRevision;

/// One open document's cached state: source text plus everything derived
/// from it.
///
/// Reparsed and rebuilt wholesale on every text change — see the module
/// docs for why a partial/incremental update is not attempted at Stage 1.
pub struct Document {
    /// The document's full current text, verbatim from the client.
    pub text: Arc<str>,
    /// The result of parsing [`text`](Self::text) with
    /// [`phalcom_ast::parser::parse`]: the recovered [`Program`] plus every
    /// [`SyntaxError`] found.
    ///
    /// [`Program`]: phalcom_ast::ast::Program
    pub parse: Arc<Parse>,
    /// The [`LineIndex`] built over [`text`](Self::text), used to map every
    /// byte-offset span in [`parse`](Self::parse) to LSP positions.
    pub line_index: Arc<LineIndex>,
    /// Monotonic semantic revision for this document.
    pub revision: SourceRevision,
    /// Client-owned LSP document version, when opened through the protocol.
    pub version: Option<i32>,
}

impl Document {
    /// Parses `text` and builds a fresh [`Document`] (parse tree + line
    /// index) from it.
    pub fn new(text: String) -> Self {
        Self::new_with_revision_and_version(text, SourceRevision(1), None)
    }

    /// Parses `text` at an explicitly assigned semantic revision.
    pub fn new_with_revision(text: String, revision: SourceRevision) -> Self {
        Self::new_with_revision_and_version(text, revision, None)
    }

    /// Parses `text` with semantic and client-visible LSP revisions.
    pub fn new_with_revision_and_version(text: String, revision: SourceRevision, version: Option<i32>) -> Self {
        let parse = Arc::new(parser::parse(&text, 0));
        let line_index = Arc::new(LineIndex::new(&text));
        Self {
            text: Arc::from(text),
            parse,
            line_index,
            revision,
            version,
        }
    }

    /// The [`SyntaxError`]s recovered from the current parse, in discovery
    /// order.
    pub fn errors(&self) -> &[SyntaxError] {
        &self.parse.errors
    }
}

/// A concurrent map of open documents, keyed by their LSP [`Url`].
///
/// Wraps a [`DashMap`] so the backend's `did_open`/`did_change`/`did_close`
/// handlers (each `&self`, per `tower_lsp::LanguageServer`) can mutate
/// distinct documents concurrently.
#[derive(Clone, Default)]
pub struct DocumentStore {
    documents: Arc<DashMap<Url, Document>>,
    revisions: Arc<DashMap<Url, u64>>,
}

impl DocumentStore {
    /// Creates an empty document store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses `text` and inserts (or replaces) the [`Document`] for `uri`.
    ///
    /// Used by both `did_open` (first insert) and `did_change` (full-text
    /// replace, since sync mode is `Full`) — the operation is identical:
    /// reparse and overwrite.
    pub fn open_or_update(&self, uri: Url, text: String) -> SourceRevision {
        self.open_or_update_versioned(uri, text, None)
    }

    /// Parses and stores one protocol document with its client version.
    pub fn open_or_update_versioned(&self, uri: Url, text: String, version: Option<i32>) -> SourceRevision {
        let revision = {
            let mut entry = self.revisions.entry(uri.clone()).or_insert(0);
            *entry += 1;
            SourceRevision(*entry)
        };
        self.documents.insert(uri, Document::new_with_revision_and_version(text, revision, version));
        revision
    }

    /// Removes the document for `uri`, e.g. on `did_close`.
    ///
    /// Returns `true` if a document was present and removed.
    pub fn close(&self, uri: &Url) -> bool {
        self.documents.remove(uri).is_some()
    }

    /// Advances a file revision after a close or disk-backed refresh.
    pub fn bump_revision(&self, uri: &Url) -> SourceRevision {
        let mut entry = self.revisions.entry(uri.clone()).or_insert(0);
        *entry += 1;
        SourceRevision(*entry)
    }

    /// Runs `f` against the [`Document`] for `uri`, if open.
    ///
    /// Returns `None` if no document is open at `uri`. Takes a closure
    /// rather than returning a reference so the underlying [`DashMap`] shard
    /// lock is held for the shortest possible scope.
    pub fn with_document<R>(&self, uri: &Url, f: impl FnOnce(&Document) -> R) -> Option<R> {
        self.documents.get(uri).map(|entry| f(&entry))
    }

    /// Returns a cheap immutable snapshot without retaining a map guard.
    pub fn snapshot(&self, uri: &Url) -> Option<DocumentSnapshot> {
        self.documents.get(uri).map(|entry| DocumentSnapshot {
            text: Arc::clone(&entry.text),
            parse: Arc::clone(&entry.parse),
            line_index: Arc::clone(&entry.line_index),
            revision: entry.revision,
            version: entry.version,
        })
    }

    /// Returns a list of all currently open document URIs.
    pub fn open_uris(&self) -> Vec<Url> {
        let mut uris = self.documents.iter().map(|entry| entry.key().clone()).collect::<Vec<_>>();
        uris.sort();
        uris
    }
}

/// Immutable open-document data suitable for semantic work outside the map
/// shard lock.
#[derive(Clone)]
pub struct DocumentSnapshot {
    /// Full live source text.
    pub text: Arc<str>,
    /// Recovered parse tree and diagnostics.
    pub parse: Arc<Parse>,
    /// UTF-16 line/offset index.
    pub line_index: Arc<LineIndex>,
    /// Semantic revision of this snapshot.
    pub revision: SourceRevision,
    /// Client-owned LSP version of this snapshot.
    pub version: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uri(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn open_then_read_back() {
        let store = DocumentStore::new();
        let u = uri("file:///a.ph");
        store.open_or_update(u.clone(), "let x = 1\n".to_string());
        let errs = store.with_document(&u, |d| d.errors().len());
        assert_eq!(errs, Some(0));
    }

    #[test]
    fn update_replaces_wholesale() {
        let store = DocumentStore::new();
        let u = uri("file:///a.ph");
        store.open_or_update(u.clone(), "let x = 1\n".to_string());
        store.open_or_update(u.clone(), "let = \n".to_string());
        let errs = store.with_document(&u, |d| d.errors().len());
        assert!(errs.unwrap() > 0);
    }

    #[test]
    fn close_removes_document() {
        let store = DocumentStore::new();
        let u = uri("file:///a.ph");
        store.open_or_update(u.clone(), "let x = 1\n".to_string());
        assert!(store.close(&u));
        assert_eq!(store.with_document(&u, |_| ()), None);
    }

    #[test]
    fn missing_document_returns_none() {
        let store = DocumentStore::new();
        let u = uri("file:///missing.ph");
        assert_eq!(store.with_document(&u, |_| ()), None);
    }
}
