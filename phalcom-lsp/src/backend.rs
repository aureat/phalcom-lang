//! The `tower_lsp::LanguageServer` implementation.
//!
//! Stage 1 (ADR-0056 §3, §6): `initialize`/`initialized`/`shutdown` for the
//! server lifecycle, and `did_open`/`did_change`/`did_close` to maintain the
//! [`DocumentStore`] and publish live, multi-error diagnostics. Nothing
//! else — no symbol index, no completion, no hover, no semantic tokens; those
//! land in later stages (`docs/forge/units/U-LSP/plan.md`).

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, MessageType, PositionEncodingKind,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
};
use tower_lsp::{Client, LanguageServer};

use crate::diagnostics::syntax_errors_to_diagnostics;
use crate::documents::DocumentStore;

/// The Phalcom language server.
///
/// Holds the `tower-lsp` [`Client`] handle (for notifications like
/// `publish_diagnostics` and `show_message`) and the [`DocumentStore`] of
/// currently open `.ph` files.
pub struct Backend {
    /// Handle back to the LSP client, used to send notifications
    /// (`textDocument/publishDiagnostics`, `window/logMessage`, …).
    client: Client,
    /// The open-document store: text + cached parse + cached [`LineIndex`]
    /// per open file.
    ///
    /// [`LineIndex`]: crate::line_index::LineIndex
    documents: DocumentStore,
}

impl Backend {
    /// Creates a new [`Backend`] bound to `client`, with an empty document
    /// store.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DocumentStore::new(),
        }
    }

    /// Reparses the document at `uri` (already updated in the store by the
    /// caller) and publishes its current [`SyntaxError`](phalcom_ast::error::SyntaxError)s
    /// as LSP diagnostics.
    ///
    /// Publishes unconditionally, including an empty list, so a
    /// previously-errored document that becomes clean has its squiggles
    /// cleared.
    async fn publish_diagnostics_for(&self, uri: tower_lsp::lsp_types::Url, version: Option<i32>) {
        let diagnostics = self
            .documents
            .with_document(&uri, |doc| syntax_errors_to_diagnostics(doc.errors(), &doc.line_index))
            .unwrap_or_default();
        self.client
            .publish_diagnostics(uri, diagnostics, version)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    /// Advertises Stage 1 capabilities: full-document sync
    /// (`textDocumentSync=Full` — Stage 1 has no incremental patch logic,
    /// see [`crate::documents`]) and UTF-16 `positionEncoding` (the LSP
    /// default; ADR-0056 §5, DEC-LSP-C).
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                position_encoding: Some(PositionEncodingKind::UTF16),
                ..ServerCapabilities::default()
            },
            server_info: Some(tower_lsp::lsp_types::ServerInfo {
                name: "phalcom-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    /// Logs that the server is ready. No further setup at Stage 1 (later
    /// stages perform the workspace scan here).
    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "phalcom-lsp initialized")
            .await;
    }

    /// Reports readiness to shut down. Holds no resources that need
    /// releasing at Stage 1.
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    /// Parses the newly-opened document and publishes its diagnostics.
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        self.documents
            .open_or_update(uri.clone(), params.text_document.text);
        self.publish_diagnostics_for(uri, Some(version)).await;
    }

    /// Reparses the document from the latest full-text change event and
    /// republishes its diagnostics.
    ///
    /// Sync mode is `Full` (see [`initialize`](Self::initialize)), so the
    /// **last** entry in `content_changes` carries the complete new text;
    /// earlier entries (there should be at most one) are ignored.
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        let Some(change) = params.content_changes.into_iter().next_back() else {
            return;
        };
        self.documents.open_or_update(uri.clone(), change.text);
        self.publish_diagnostics_for(uri, Some(version)).await;
    }

    /// Drops the closed document from the store and clears its diagnostics.
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.close(&uri);
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }
}
