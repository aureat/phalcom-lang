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

use dashmap::DashMap;
use phalcom_ast::error::SyntaxError;
use phalcom_ast::parser::{self, Parse};
use tower_lsp::lsp_types::Url;

use crate::line_index::LineIndex;

/// One open document's cached state: source text plus everything derived
/// from it.
///
/// Reparsed and rebuilt wholesale on every text change — see the module
/// docs for why a partial/incremental update is not attempted at Stage 1.
pub struct Document {
    /// The document's full current text, verbatim from the client.
    pub text: String,
    /// The result of parsing [`text`](Self::text) with
    /// [`phalcom_ast::parser::parse`]: the recovered [`Program`] plus every
    /// [`SyntaxError`] found.
    ///
    /// [`Program`]: phalcom_ast::ast::Program
    pub parse: Parse,
    /// The [`LineIndex`] built over [`text`](Self::text), used to map every
    /// byte-offset span in [`parse`](Self::parse) to LSP positions.
    pub line_index: LineIndex,
}

impl Document {
    /// Parses `text` and builds a fresh [`Document`] (parse tree + line
    /// index) from it.
    pub fn new(text: String) -> Self {
        let parse = parser::parse(&text, 0);
        let line_index = LineIndex::new(&text);
        Self {
            text,
            parse,
            line_index,
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
#[derive(Default)]
pub struct DocumentStore {
    documents: DashMap<Url, Document>,
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
    pub fn open_or_update(&self, uri: Url, text: String) {
        self.documents.insert(uri, Document::new(text));
    }

    /// Removes the document for `uri`, e.g. on `did_close`.
    ///
    /// Returns `true` if a document was present and removed.
    pub fn close(&self, uri: &Url) -> bool {
        self.documents.remove(uri).is_some()
    }

    /// Runs `f` against the [`Document`] for `uri`, if open.
    ///
    /// Returns `None` if no document is open at `uri`. Takes a closure
    /// rather than returning a reference so the underlying [`DashMap`] shard
    /// lock is held for the shortest possible scope.
    pub fn with_document<R>(&self, uri: &Url, f: impl FnOnce(&Document) -> R) -> Option<R> {
        self.documents.get(uri).map(|entry| f(&entry))
    }
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
