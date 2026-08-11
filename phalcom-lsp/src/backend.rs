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
//! [`crate::core_table`].
//!
//! Stage 4 (ADR-0056 §4, `docs/forge/units/U-LSP/plan.md` "Stage 4"):
//! `textDocument/hover`, composing whichever of [`crate::hover`]'s three
//! independent sources resolve at the cursor — a keyword blurb, or a
//! selector's signature/kind/defining-class plus its harvested Phaldoc
//! summary/tags.
//!
//! Stage 5 (ADR-0056, `docs/forge/units/U-LSP/plan.md` "Stage 5"):
//! `textDocument/semanticTokens/full`, a flat lexer-driven token-coloring
//! pass ([`crate::semantic_tokens`]).

use std::path::{Path, PathBuf};

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents, HoverParams, HoverProviderCapability, InitializeParams,
    InitializeResult, InitializedParams, Location, MarkupContent, MarkupKind, MessageType, OneOf, Position, PositionEncodingKind, ReferenceParams,
    SemanticTokensFullOptions, SemanticTokensOptions, SemanticTokensParams, SemanticTokensResult, SemanticTokensServerCapabilities, ServerCapabilities,
    SymbolInformation, SymbolKind, TextDocumentSyncCapability, TextDocumentSyncKind, Url, WorkspaceSymbolParams,
};
use tower_lsp::{Client, LanguageServer};

use crate::completion;
use crate::core_table::CoreTable;
use crate::diagnostics::syntax_errors_to_diagnostics;
use crate::documents::DocumentStore;
use crate::hover::{self, SelectorSite};
use crate::index::{self, Occurrence, WorkspaceIndex};
use crate::line_index::LineIndex;
use crate::semantic::SemanticDb;
use crate::semantic::ValueShape;
use crate::semantic_tokens;

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
    /// Live VM-free semantic state. Completion migration consumes this after
    /// the Phase A foundation; the legacy index remains for navigation parity.
    semantic: SemanticDb,
}

impl Backend {
    /// Creates a new [`Backend`] bound to `client`, with an empty document
    /// store and an empty workspace index.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DocumentStore::new(),
            index: WorkspaceIndex::new(),
            semantic: SemanticDb::new(),
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
                self.semantic.update_file(&uri, doc.revision, &doc.parse.program);
                syntax_errors_to_diagnostics(doc.errors(), &doc.line_index)
            })
            .unwrap_or_default();
        self.client.publish_diagnostics(uri, diagnostics, version).await;
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
                self.index.update_file(uri.clone(), &parse.program);
                self.semantic.update_file(&uri, crate::semantic::FileRevision(1), &parse.program);
            }
        }
    }

    /// Resolves a recovered completion target through live semantic facts.
    fn semantic_receiver(&self, uri: &Url, doc: &crate::documents::Document, position: Position) -> Option<(String, completion::ReceiverKind)> {
        let target = completion::target_at(doc, position)?;
        let receiver = doc.text.get(target.receiver_range.start..target.receiver_range.end)?;
        let offset = target.receiver_range.end;
        if receiver == "self" {
            return self
                .semantic
                .class_at(uri, offset)
                .map(|class| (class.name, completion::ReceiverKind::Instance));
        }
        if receiver.chars().next().is_some_and(char::is_uppercase) && receiver.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
            return self
                .semantic
                .class_for_name(uri, receiver)
                .map(|class| (class.name, completion::ReceiverKind::ClassObject));
        }
        if receiver.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
            return match self.semantic.binding_at(uri, receiver, offset)?.shape {
                ValueShape::Instance(class) => Some((class.name, completion::ReceiverKind::Instance)),
                ValueShape::ClassObject(class) => Some((class.name, completion::ReceiverKind::ClassObject)),
                ValueShape::Union(classes) => classes.into_iter().find_map(|shape| match shape {
                    ValueShape::Instance(class) => Some((class.name, completion::ReceiverKind::Instance)),
                    ValueShape::ClassObject(class) => Some((class.name, completion::ReceiverKind::ClassObject)),
                    _ => None,
                }),
                _ => None,
            };
        }
        None
    }

    /// Resolves the selector under `position` in the open document `uri`,
    /// via that document's own cached parse and [`LineIndex`], together with
    /// the matched node's own LSP [`tower_lsp::lsp_types::Range`] (so a
    /// caller like `hover_at` can underline the exact span a selector was
    /// resolved from, the way its keyword branch already does).
    ///
    /// Returns `None` if `uri` is not open, or if `position` sits on no
    /// selector-bearing node (see [`index::selector_at_offset`]).
    fn selector_at_position(&self, uri: &Url, position: Position) -> Option<(String, tower_lsp::lsp_types::Range)> {
        self.documents
            .with_document(uri, |doc| {
                let offset = doc.line_index.offset(position);
                index::selector_at_offset(&doc.parse.program, offset).map(|(selector, range)| (selector, doc.line_index.range(range.start..range.end)))
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
        if let Some(range) = self
            .documents
            .with_document(&occurrence.uri, |doc| doc.line_index.range(occurrence.range.start..occurrence.range.end))
        {
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
        occurrences.iter().filter_map(|occ| self.occurrence_to_location(occ)).collect()
    }

    /// Runs `f` against a snapshot of `uri`'s source text, parsed
    /// [`phalcom_ast::ast::Program`], and [`LineIndex`] — the same "open or
    /// on-disk" dual path [`Self::occurrence_to_location`] uses, generalized
    /// so Stage 4's cross-file Phaldoc harvest
    /// ([`crate::hover::harvest_doc_for_selector`]) can inspect a selector's
    /// *defining* file even when that file is not the one currently open
    /// under the cursor:
    ///
    /// - **Open**: borrows the live/unsaved buffer straight out of the
    ///   [`DocumentStore`] (no reparse).
    /// - **Not open**: reads the file from disk and parses it fresh, on the
    ///   same "small `.ph` file, occasional request" cost basis
    ///   [`Self::occurrence_to_location`] already accepts — no cache is kept
    ///   for the on-disk path.
    ///
    /// Returns `None` if `uri` is not open and cannot be read from disk.
    fn with_source_snapshot<R>(&self, uri: &Url, f: impl FnOnce(&str, &phalcom_ast::ast::Program, &LineIndex) -> R) -> Option<R> {
        let is_open = self.documents.with_document(uri, |_| ()).is_some();
        if is_open {
            return self.documents.with_document(uri, |doc| f(&doc.text, &doc.parse.program, &doc.line_index));
        }

        let path = uri.to_file_path().ok()?;
        let text = std::fs::read_to_string(path).ok()?;
        let parse = phalcom_ast::parser::parse(&text, 0);
        let line_index = LineIndex::new(&text);
        Some(f(&text, &parse.program, &line_index))
    }

    /// Answers `textDocument/hover` (Stage 4): the pluggable composition of
    /// [`crate::hover`]'s sources.
    ///
    /// A keyword/contextual-word hit at the cursor
    /// ([`hover::keyword_at_offset`]) short-circuits straight to its blurb —
    /// mirrors `hover.ts`'s keyword branch taking priority over the selector
    /// layer. Otherwise resolves the selector under the cursor
    /// ([`index::selector_at_offset`]) and renders whichever of the
    /// selector layer's sources are present: user-class sites from
    /// [`WorkspaceIndex::definition_info`], builtin sites from
    /// [`CoreTable`], and the harvested Phaldoc doc (from the selector's
    /// *defining* file, which may not be the currently open one —
    /// [`Self::with_source_snapshot`]) attached to a user-class definition —
    /// with [`Hover::range`] set to the resolved selector's own span
    /// ([`Self::selector_at_position`]), matching the keyword branch.
    ///
    /// If the cursor sits on neither a keyword nor a selector, falls back to
    /// resolving a top-level `let`/`var` binding usage
    /// ([`index::top_level_binding_at_offset`]) and rendering its harvested
    /// Phaldoc doc, if any (doc-comments-phaldoc.md §5's `let`/`var`
    /// placement-legality case).
    ///
    /// Returns `None` if nothing at the cursor resolves to a keyword, a
    /// known selector, or a documented top-level binding.
    fn hover_at(&self, uri: &Url, position: Position) -> Option<Hover> {
        if let Some((word, span)) = self
            .documents
            .with_document(uri, |doc| {
                let offset = doc.line_index.offset(position);
                hover::keyword_at_offset(&doc.text, offset).map(|(word, range)| (word, doc.line_index.range(range)))
            })
            .flatten()
        {
            let blurb = hover::keyword_blurb(word)?;
            return Some(Hover {
                contents: markdown_contents(hover::render_keyword_hover(word, blurb)),
                range: Some(span),
            });
        }

        let Some((selector, span)) = self.selector_at_position(uri, position) else {
            return self.hover_for_top_level_binding(uri, position);
        };

        let mut sites: Vec<SelectorSite> = self
            .index
            .definition_info(&selector)
            .into_iter()
            .map(|info| SelectorSite {
                class: info.class,
                kind: info.kind,
            })
            .collect();

        if sites.is_empty() {
            let table = CoreTable::bundled();
            for (class, members) in &table.classes {
                if let Some(member) = members.iter().find(|m| m.selector == selector) {
                    sites.push(SelectorSite {
                        class: class.clone(),
                        kind: member.kind,
                    });
                }
            }
            sites.sort_by(|a, b| a.class.cmp(&b.class));
        }

        // The Phaldoc harvest only applies to a user-class definition — a
        // builtin selector has no `.ph` source to scan (Stage 4 test
        // "Builtin hover ... no Phaldoc section").
        let defs = self.index.definition_info(&selector);
        let phaldoc = defs.first().and_then(|def| {
            self.with_source_snapshot(&def.uri, |text, program, line_index| {
                hover::harvest_doc_for_selector(text, program, line_index, &selector)
            })
            .flatten()
        });

        let value = hover::render_selector_hover(&selector, &sites, phaldoc.as_ref())?;
        Some(Hover {
            contents: markdown_contents(value),
            range: Some(span),
        })
    }

    /// The `hover_at` fallback for a bare identifier that resolves to none of
    /// keyword, contextual word, or a selector: a top-level `let`/`var`
    /// binding usage ([`index::top_level_binding_at_offset`]), rendered from
    /// its own file's harvested Phaldoc doc, if any.
    ///
    /// Unlike the selector layer, a top-level binding is same-file only (no
    /// cross-file `DefinitionInfo` is tracked for it), so this reads straight
    /// off `uri`'s own cached parse — no [`Self::with_source_snapshot`] hop.
    /// Renders with an empty [`SelectorSite`] list ([`hover::
    /// render_selector_hover`]'s "purely local, Phaldoc-only hover" case), so
    /// a binding with no doc above it renders no hover at all rather than a
    /// bare, uninformative label.
    ///
    /// Returns `None` if `uri` is not open, the cursor resolves to no
    /// top-level binding, or that binding carries no Phaldoc doc.
    fn hover_for_top_level_binding(&self, uri: &Url, position: Position) -> Option<Hover> {
        let (name, doc) = self
            .documents
            .with_document(uri, |doc| {
                let offset = doc.line_index.offset(position);
                let name = index::top_level_binding_at_offset(&doc.parse.program, offset)?;
                let phaldoc = hover::harvest_doc_for_selector(&doc.text, &doc.parse.program, &doc.line_index, &name)?;
                Some((name, phaldoc))
            })
            .flatten()?;

        let value = hover::render_selector_hover(&name, &[], Some(&doc))?;
        Some(Hover {
            contents: markdown_contents(value),
            range: None,
        })
    }
}

/// Wraps `value` as an LSP [`HoverContents::Markup`] block of
/// [`MarkupKind::Markdown`] — the one place `hover_at` builds a [`Hover`]'s
/// contents, so every hover renders through the same markdown wrapper.
fn markdown_contents(value: String) -> HoverContents {
    HoverContents::Markup(MarkupContent {
        kind: MarkupKind::Markdown,
        value,
    })
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
    /// `workspace_symbol_provider`, Stage 3's `completion_provider` (with
    /// `.` as a trigger character), and Stage 4's `hover_provider`.
    ///
    /// Also performs Stage 2's one-time workspace scan
    /// (`Self::scan_workspace`) over every root named in `params`
    /// (`root_uri` and/or `workspace_folders` — plan "P4": no single-root
    /// assumption, every named root is scanned).
    ///
    /// Also advertises Stage 5's `semanticTokensProvider` (full-document
    /// only, no `range`/`delta` support yet), with the legend built by
    /// [`semantic_tokens::legend`].
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let mut roots: Vec<Url> = params.workspace_folders.unwrap_or_default().into_iter().map(|folder| folder.uri).collect();
        #[allow(deprecated)]
        if let Some(root_uri) = params.root_uri {
            if !roots.contains(&root_uri) {
                roots.push(root_uri);
            }
        }
        self.scan_workspace(&roots);

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
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
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                    legend: semantic_tokens::legend(),
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                    ..SemanticTokensOptions::default()
                })),
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
        self.client.log_message(MessageType::INFO, "phalcom-lsp initialized").await;
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
        self.documents.open_or_update(uri.clone(), params.text_document.text);
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
        let revision = self.documents.bump_revision(&uri);
        if let Ok(path) = uri.to_file_path() {
            if let Ok(text) = std::fs::read_to_string(path) {
                let parse = phalcom_ast::parser::parse(&text, 0);
                self.index.update_file(uri.clone(), &parse.program);
                self.semantic.update_file(&uri, revision, &parse.program);
            } else {
                self.semantic.remove_file(&uri);
            }
        } else {
            self.semantic.remove_file(&uri);
        }
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
    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let Some((selector, _range)) = self.selector_at_position(&uri, position) else {
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
        let Some((selector, _range)) = self.selector_at_position(&uri, position) else {
            return Ok(None);
        };

        let mut occurrences = self.index.references(&selector);
        if params.context.include_declaration {
            occurrences.extend(self.index.definitions(&selector));
        }

        let locations = self.occurrences_to_locations(&occurrences);
        if locations.is_empty() { Ok(None) } else { Ok(Some(locations)) }
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
    async fn symbol(&self, params: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
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
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let items: Option<Vec<CompletionItem>> = self.documents.with_document(&uri, |doc| {
            let resolved = self.semantic_receiver(&uri, doc, position);
            let resolved = resolved.as_ref().map(|(class, kind)| (class.as_str(), *kind));
            let offset = doc.line_index.offset(position);
            let privileged = uri.path().ends_with("/phalcom-core/core/core.ph");
            completion::contextual_completions(completion::ContextualCompletionContext {
                resolved,
                program: &doc.parse.program,
                text: &doc.text,
                offset,
                privileged,
                uri: &uri,
                index: &self.index,
                table: CoreTable::bundled(),
            })
        });

        Ok(items.map(CompletionResponse::Array))
    }

    /// Answers `textDocument/hover` (Stage 4). See `Self::hover_at` for the
    /// resolution/composition logic; this is a thin `async` shim over it, as
    /// `hover_at`'s I/O (an on-disk read for a not-currently-open defining
    /// file) is synchronous, small, and occasional, matching every other
    /// cross-file lookup in this backend (`Self::occurrence_to_location`).
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        Ok(self.hover_at(&uri, position))
    }

    /// Answers `textDocument/semanticTokens/full` (Stage 5): a flat,
    /// lexer-driven token-coloring pass over the whole document
    /// ([`semantic_tokens::tokens_for`]).
    ///
    /// Runs directly off the document's cached text and [`LineIndex`] — no
    /// parse tree involved, so this works even on a currently-unparseable
    /// buffer (diagnostics squiggle it red; tokens still color it).
    ///
    /// Returns `Ok(None)` if the document is not open.
    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let data = self
            .documents
            .with_document(&uri, |doc| semantic_tokens::tokens_for(&doc.text, &doc.line_index));
        Ok(data.map(|data| SemanticTokensResult::Tokens(tower_lsp::lsp_types::SemanticTokens { result_id: None, data })))
    }
}
