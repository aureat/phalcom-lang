//! The `tower_lsp::LanguageServer` implementation.
//!
//! Stage 1 (ADR-0056 §3, §6): `initialize`/`initialized`/`shutdown` for the
//! server lifecycle, and `did_open`/`did_change`/`did_close` to maintain the
//! [`DocumentStore`] and publish live, multi-error diagnostics.
//!
//! Stage 2 (ADR-0056 §4, `docs/forge/units/U-LSP/plan.md` "Stage 2"): a
//! [`WorkspaceIndex`] scanned from every `.ph` file under the workspace
//! root(s) at `initialize`, kept current per-file on `did_open`/`did_change`,
//! and served through `textDocument/definition`, `textDocument/references`,
//! and `workspace/symbol`.
//!
//! Stage 3 (ADR-0056 §4, `docs/forge/units/U-LSP/plan.md` "Stage 3"):
//! receiver-aware `textDocument/completion`, resolving the receiver's class
//! (via [`crate::completion::ReceiverResolver`]) and offering its selectors —
//! user members from the [`WorkspaceIndex`], builtin members from
//! [`crate::core_table`]. No hover or semantic tokens yet; those land in later
//! stages.

use std::path::{Path, PathBuf};

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionOptions, CompletionParams, CompletionResponse,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    GotoDefinitionParams, GotoDefinitionResponse, InitializeParams, InitializeResult,
    InitializedParams, Location, MessageType, OneOf, Position, PositionEncodingKind,
    ReferenceParams, ServerCapabilities, SymbolInformation, SymbolKind, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url, WorkspaceSymbolParams,
};
use tower_lsp::{Client, LanguageServer};

use crate::completion::{self, ConstructResolver, ReceiverResolver};
use crate::core_table::CoreTable;
use crate::diagnostics::syntax_errors_to_diagnostics;
use crate::documents::DocumentStore;
use crate::index::{self, Occurrence, WorkspaceIndex};
use crate::line_index::LineIndex;

/// The Phalcom language server.
///
/// Holds the `tower-lsp` [`Client`] handle (for notifications like
/// `publish_diagnostics` and `show_message`), the [`DocumentStore`] of
/// currently open `.ph` files, and the [`WorkspaceIndex`] of every `.ph`
/// file under the workspace root(s) (Stage 2).
pub struct Backend {
    /// Handle back to the LSP client, used to send notifications
    /// (`textDocument/publishDiagnostics`, `window/logMessage`, …).
    client: Client,
    /// The open-document store: text + cached parse + cached [`LineIndex`]
    /// per open file.
    documents: DocumentStore,
    /// The workspace-wide selector index (Stage 2): every `ClassMember`
    /// definition and send-site reference, keyed by ADR-0012 comma-form
    /// selector. Backed by a concurrent map internally, so it can be read
    /// and written from concurrent `&self` handlers without a server-wide
    /// lock — no `Arc`/`Mutex` wrapper needed around it here.
    index: WorkspaceIndex,
}

impl Backend {
    /// Creates a new [`Backend`] bound to `client`, with an empty document
    /// store and an empty workspace index.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DocumentStore::new(),
            index: WorkspaceIndex::new(),
        }
    }

    /// Reparses the document at `uri` (already updated in the store by the
    /// caller), refreshes its slice of the [`WorkspaceIndex`], and publishes
    /// its current [`SyntaxError`](phalcom_ast::error::SyntaxError)s as LSP
    /// diagnostics.
    ///
    /// Publishes unconditionally, including an empty list, so a
    /// previously-errored document that becomes clean has its squiggles
    /// cleared.
    async fn publish_diagnostics_for(&self, uri: Url, version: Option<i32>) {
        let diagnostics = self
            .documents
            .with_document(&uri, |doc| {
                self.index.update_file(uri.clone(), &doc.parse.program);
                syntax_errors_to_diagnostics(doc.errors(), &doc.line_index)
            })
            .unwrap_or_default();
        self.client
            .publish_diagnostics(uri, diagnostics, version)
            .await;
    }

    /// Scans every `.ph` file under `roots` and (re)builds the workspace
    /// index from scratch.
    ///
    /// Called once from `initialize`. A synchronous filesystem walk —
    /// Stage 2's scan is a one-time startup cost, not a hot path, so no
    /// async I/O or background task is warranted here (plan "Build order"
    /// step 2).
    fn scan_workspace(&self, roots: &[Url]) {
        for root in roots {
            let Ok(root_path) = root.to_file_path() else {
                continue;
            };
            for file in collect_ph_files(&root_path) {
                let Ok(text) = std::fs::read_to_string(&file) else {
                    continue;
                };
                let Ok(uri) = Url::from_file_path(&file) else {
                    continue;
                };
                let parse = phalcom_ast::parser::parse(&text, 0);
                self.index.update_file(uri, &parse.program);
            }
        }
    }

    /// Resolves the selector under `position` in the open document `uri`,
    /// via that document's own cached parse and [`LineIndex`].
    ///
    /// Returns `None` if `uri` is not open, or if `position` sits on no
    /// selector-bearing node (see [`index::selector_at_offset`]).
    fn selector_at_position(&self, uri: &Url, position: Position) -> Option<String> {
        self.documents
            .with_document(uri, |doc| {
                let offset = doc.line_index.offset(position);
                index::selector_at_offset(&doc.parse.program, offset)
            })
            .flatten()
    }

    /// Maps one index [`Occurrence`] to an LSP [`Location`].
    ///
    /// The occurrence's file may or may not be currently open in the
    /// client:
    /// - **Open** (tracked in [`DocumentStore`]): map through that
    ///   document's own [`LineIndex`], which reflects the live/unsaved
    ///   buffer — the same text the index was last built from (Stage 2's
    ///   `did_change` wiring keeps them in lockstep).
    /// - **Not open**: the index's byte range was computed against the file
    ///   as last read from disk (the `initialize`-time scan, or whenever it
    ///   was last open), so re-reading the file and building a fresh
    ///   [`LineIndex`] on demand is correct as long as the on-disk file
    ///   hasn't changed since. This crate deliberately does **not** cache a
    ///   `LineIndex` per indexed-but-unopened file — that cache would need
    ///   its own invalidation story (a filesystem watch) that Stage 2 does
    ///   not build; re-reading a small `.ph` file on an occasional
    ///   go-to-def/find-refs/workspace-symbol request is simple and cheap
    ///   enough not to need one.
    ///
    /// Returns `None` if the file is not open and cannot be read from disk
    /// (deleted since the last scan, or not a `file://` URI).
    fn occurrence_to_location(&self, occurrence: &Occurrence) -> Option<Location> {
        if let Some(range) = self.documents.with_document(&occurrence.uri, |doc| {
            doc.line_index
                .range(occurrence.range.start..occurrence.range.end)
        }) {
            return Some(Location {
                uri: occurrence.uri.clone(),
                range,
            });
        }

        let path = occurrence.uri.to_file_path().ok()?;
        let text = std::fs::read_to_string(path).ok()?;
        let line_index = LineIndex::new(&text);
        Some(Location {
            uri: occurrence.uri.clone(),
            range: line_index.range(occurrence.range.start..occurrence.range.end),
        })
    }

    /// Maps every occurrence in `occurrences` to an LSP [`Location`],
    /// dropping any whose file could not be resolved
    /// ([`Self::occurrence_to_location`]).
    fn occurrences_to_locations(&self, occurrences: &[Occurrence]) -> Vec<Location> {
        occurrences
            .iter()
            .filter_map(|occ| self.occurrence_to_location(occ))
            .collect()
    }
}

/// Recursively collects every `.ph` file under `root`, skipping VCS and
/// dependency/build directories (`.git`, `target`, `node_modules`) and any
/// other hidden (dot-prefixed) directory, so a workspace scan rooted at a
/// large repo doesn't walk into unrelated, potentially huge trees.
///
/// A plain, dependency-free recursive walk (no `walkdir`) — `phalcom-lsp`'s
/// `Cargo.toml` intentionally carries no new dependency for this (U-LSP
/// plan, "do NOT add a new heavy dependency without checking `Cargo.toml`
/// first").
fn collect_ph_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_ph_files_into(root, &mut files);
    files
}

/// The recursive worker behind [`collect_ph_files`], accumulating into
/// `out` rather than returning per-call `Vec`s.
fn collect_ph_files_into(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let skip = name.starts_with('.') || name == "target" || name == "node_modules";
            if !skip {
                collect_ph_files_into(&path, out);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("ph") {
            out.push(path);
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    /// Advertises Stage 1 + Stage 2 capabilities: full-document sync
    /// (`textDocumentSync=Full` — Stage 1 has no incremental patch logic,
    /// see [`crate::documents`]), UTF-16 `positionEncoding` (the LSP
    /// default; ADR-0056 §5, DEC-LSP-C), Stage 2's
    /// `definition_provider`/`references_provider`/
    /// `workspace_symbol_provider`, and Stage 3's `completion_provider` (with
    /// `.` as a trigger character).
    ///
    /// Also performs Stage 2's one-time workspace scan
    /// (`Self::scan_workspace`) over every root named in `params`
    /// (`root_uri` and/or `workspace_folders` — plan "P4": no single-root
    /// assumption, every named root is scanned).
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let mut roots: Vec<Url> = params
            .workspace_folders
            .unwrap_or_default()
            .into_iter()
            .map(|folder| folder.uri)
            .collect();
        #[allow(deprecated)]
        if let Some(root_uri) = params.root_uri {
            if !roots.contains(&root_uri) {
                roots.push(root_uri);
            }
        }
        self.scan_workspace(&roots);

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                position_encoding: Some(PositionEncodingKind::UTF16),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    // `.` triggers member completion; the client also
                    // re-requests on identifier characters as the user types.
                    trigger_characters: Some(vec![".".to_string()]),
                    ..CompletionOptions::default()
                }),
                ..ServerCapabilities::default()
            },
            server_info: Some(tower_lsp::lsp_types::ServerInfo {
                name: "phalcom-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    /// Logs that the server is ready. The workspace scan already ran in
    /// [`Self::initialize`] (per the LSP spec, `initialize` may do this kind
    /// of setup work before responding).
    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "phalcom-lsp initialized")
            .await;
    }

    /// Reports readiness to shut down. Holds no resources that need
    /// releasing.
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    /// Parses the newly-opened document, publishes its diagnostics, and
    /// refreshes its slice of the workspace index (via
    /// `Self::publish_diagnostics_for`).
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        self.documents
            .open_or_update(uri.clone(), params.text_document.text);
        self.publish_diagnostics_for(uri, Some(version)).await;
    }

    /// Reparses the document from the latest full-text change event,
    /// republishes its diagnostics, and refreshes its slice of the
    /// workspace index — so the index tracks the live/unsaved buffer, not
    /// just what's on disk (plan "Build order" step 3).
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
    ///
    /// The document's contribution to the workspace index is **not**
    /// removed here: an index entry survives a close, tracking the file's
    /// last-known (on-disk) content, until either it is reopened and
    /// reparsed, or the workspace is rescanned. Closing a buffer is not the
    /// same as deleting the file.
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.documents.close(&uri);
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    /// Resolves the selector under the cursor
    /// (`Self::selector_at_position`) and returns every recorded
    /// definition site for it, mapped to LSP [`Location`]s
    /// (`Self::occurrences_to_locations`).
    ///
    /// Returns `Ok(None)` if the cursor sits on no selector-bearing node, or
    /// the selector has no recorded definition (e.g. a builtin core-class
    /// method — the index only covers user `.ph` source; `core-table.json`
    /// lookup is a later stage, plan DEC-LSP-B).
    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let Some(selector) = self.selector_at_position(&uri, position) else {
            return Ok(None);
        };

        let occurrences = self.index.definitions(&selector);
        let locations = self.occurrences_to_locations(&occurrences);
        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(GotoDefinitionResponse::Array(locations)))
        }
    }

    /// Resolves the selector under the cursor and returns every recorded
    /// send-site reference to it, mapped to LSP [`Location`]s. Includes the
    /// definition site(s) too when `context.include_declaration` is set
    /// (the LSP `references` convention).
    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let Some(selector) = self.selector_at_position(&uri, position) else {
            return Ok(None);
        };

        let mut occurrences = self.index.references(&selector);
        if params.context.include_declaration {
            occurrences.extend(self.index.definitions(&selector));
        }

        let locations = self.occurrences_to_locations(&occurrences);
        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }

    /// Answers `workspace/symbol`: every indexed selector containing
    /// `params.query` as a case-insensitive substring
    /// ([`WorkspaceIndex::symbols_matching`]), rendered as
    /// [`SymbolInformation`] at its first definition site.
    ///
    /// Every result reports [`SymbolKind::METHOD`] — the index does not yet
    /// distinguish getter/setter/construct/field kinds in a
    /// `SymbolKind`-shaped way; refining this is left to a later stage
    /// (hover, Stage 4, already needs that per-kind rendering and is the
    /// natural place to add it once).
    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let matches = self.index.symbols_matching(&params.query);
        let symbols: Vec<SymbolInformation> = matches
            .into_iter()
            .filter_map(|(name, occurrence)| {
                let location = self.occurrence_to_location(&occurrence)?;
                #[allow(deprecated)] // `deprecated` field has no non-deprecated replacement to construct with here; `tags` is left unset.
                Some(SymbolInformation {
                    name,
                    kind: SymbolKind::METHOD,
                    tags: None,
                    deprecated: None,
                    location,
                    container_name: None,
                })
            })
            .collect();
        Ok(Some(symbols))
    }

    /// Answers `textDocument/completion` (Stage 3): receiver-aware member
    /// completion.
    ///
    /// Resolves the class of the receiver under the cursor via the pluggable
    /// [`ReceiverResolver`] ([`ConstructResolver`] here), then renders that
    /// class's selectors — user members from the [`WorkspaceIndex`], builtin
    /// members from [`CoreTable`] — as snippet [`CompletionItem`]s
    /// ([`completion::completions`]). When the receiver type cannot be
    /// resolved, it degrades to the full builtin surface rather than
    /// suppressing completion (never worse than the pre-LSP client behavior).
    ///
    /// Returns `Ok(None)` if the document is not open.
    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let items: Option<Vec<CompletionItem>> = self.documents.with_document(&uri, |doc| {
            let resolved = ConstructResolver.resolve(doc, position);
            completion::completions(resolved.as_deref(), &self.index, CoreTable::bundled())
        });

        Ok(items.map(CompletionResponse::Array))
    }
}
