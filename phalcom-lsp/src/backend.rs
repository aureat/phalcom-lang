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
//! receiver-aware `textDocument/completion`, resolving the receiver through
//! live semantic facts and offering its source/native semantic surface.
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

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde_json::Value as JsonValue;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionOptions, CompletionParams, CompletionResponse, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    DidChangeWatchedFilesParams, DidChangeWorkspaceFoldersParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, FileChangeType, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverContents, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams, InlayHint,
    InlayHintOptions, InlayHintParams, InlayHintServerCapabilities, Location, MarkupContent, MarkupKind, MessageType, OneOf, Position, PositionEncodingKind,
    ReferenceParams, Registration, SemanticTokensFullOptions, SemanticTokensOptions, SemanticTokensParams, SemanticTokensResult,
    SemanticTokensServerCapabilities, ServerCapabilities, SymbolInformation, SymbolKind, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities, WorkspaceSymbolParams,
};
use tower_lsp::{Client, LanguageServer};

use crate::completion;
use crate::diagnostics::syntax_errors_to_diagnostics;
use crate::documents::DocumentStore;
use crate::hover::{self, SelectorSite};
use crate::index::{self, Occurrence, WorkspaceIndex};
use crate::inlay_hints::HintPolicy;
use crate::line_index::LineIndex;
use crate::semantic::SemanticDb;
use crate::semantic::ValueShape;
use crate::semantic_tokens;

/// Runtime configuration that affects semantic source discovery and hint UI.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Explicit sysroot/core source directory, if configured.
    pub sysroot_path: Option<PathBuf>,
    /// Inlay-hint display policy.
    pub inlay_hints: HintPolicy,
    /// Whether obvious literal initializers should omit their hint.
    pub suppress_obvious: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            sysroot_path: None,
            inlay_hints: HintPolicy::Stable,
            suppress_obvious: false,
        }
    }
}

impl ServerConfig {
    fn from_json(value: Option<&JsonValue>) -> Self {
        let mut config = Self::default();
        let Some(value) = value else { return config };
        let root = value.get("phalcom").unwrap_or(value);
        let lsp = root.get("lsp").unwrap_or(root);
        if let Some(path) = value.get("phalcom.lsp.sysrootPath").and_then(JsonValue::as_str) {
            config.sysroot_path = Some(PathBuf::from(path));
        }
        if let Some(path) = root.get("lsp").and_then(|value| value.get("sysrootPath")).and_then(JsonValue::as_str) {
            config.sysroot_path = Some(PathBuf::from(path));
        } else if let Some(path) = lsp.get("sysrootPath").and_then(JsonValue::as_str) {
            config.sysroot_path = Some(PathBuf::from(path));
        }
        let hints = root.get("inlayHints").or_else(|| lsp.get("inlayHints"));
        if let Some(enabled) = hints.and_then(JsonValue::as_bool) {
            config.inlay_hints = if enabled { HintPolicy::Stable } else { HintPolicy::Off };
        } else if let Some(hints) = hints {
            if let Some(types) = hints.get("types").and_then(JsonValue::as_str) {
                config.inlay_hints = match types {
                    "off" => HintPolicy::Off,
                    "all" => HintPolicy::All,
                    _ => HintPolicy::Stable,
                };
            }
            if let Some(suppress) = hints.get("suppressObvious").and_then(JsonValue::as_bool) {
                config.suppress_obvious = suppress;
            }
        }
        if let Some(types) = value.get("phalcom.inlayHints.types").and_then(JsonValue::as_str) {
            config.inlay_hints = match types {
                "off" => HintPolicy::Off,
                "all" => HintPolicy::All,
                _ => HintPolicy::Stable,
            };
        }
        if let Some(suppress) = value.get("phalcom.inlayHints.suppressObvious").and_then(JsonValue::as_bool) {
            config.suppress_obvious = suppress;
        }
        config
    }
}

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
    /// Current workspace roots advertised by the client.
    workspace_roots: RwLock<Vec<Url>>,
    /// Files currently represented in the workspace index.
    indexed_files: RwLock<BTreeSet<Url>>,
    /// Mutable server configuration.
    config: RwLock<ServerConfig>,
    /// URI for the active on-disk core source, if one replaced bundled core.
    ///
    /// This must be explicit: configured sysroots need the same open-buffer
    /// precedence as the repository's conventional `phalcom-core/core/core.ph`.
    core_source_uris: RwLock<BTreeSet<Url>>,
    /// Whether client requested dynamic watched-file registration.
    watch_registration: RwLock<bool>,
}

#[derive(Clone, Debug)]
struct ResolvedMemberTarget {
    /// Runtime receiver candidate used for dispatch.
    receiver: crate::semantic::ClassId,
    /// Actual declaration selected after inheritance lookup.
    member: crate::semantic::MemberSurface,
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
            workspace_roots: RwLock::new(Vec::new()),
            indexed_files: RwLock::new(BTreeSet::new()),
            config: RwLock::new(ServerConfig::default()),
            core_source_uris: RwLock::new(BTreeSet::new()),
            watch_registration: RwLock::new(false),
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
                let recovered = semantic_recovery_parse(doc.text.as_ref(), &doc.parse);
                let program = recovered.as_ref().map(|parse| &parse.program).unwrap_or(&doc.parse.program);
                self.update_semantic_for_source(&uri, doc.revision, program);
                syntax_errors_to_diagnostics(doc.errors(), &doc.line_index)
            })
            .unwrap_or_default();
        self.client.publish_diagnostics(uri, diagnostics, version).await;
    }

    fn update_semantic_for_source(&self, uri: &Url, revision: crate::semantic::FileRevision, program: &phalcom_ast::ast::Program) {
        if self.is_core_source_uri(uri) {
            self.semantic.update_core(revision, program);
        } else {
            self.semantic.update_file(uri, revision, program);
        }
    }

    fn is_core_source_uri(&self, uri: &Url) -> bool {
        self.core_source_uris.read().expect("core source URI lock poisoned").contains(uri)
    }

    fn set_core_source_uri(&self, uri: Url) {
        let mut core_source_uris = self.core_source_uris.write().expect("core source URI lock poisoned");
        core_source_uris.clear();
        core_source_uris.insert(uri);
    }

    fn clear_core_source_uris(&self) {
        self.core_source_uris.write().expect("core source URI lock poisoned").clear();
    }

    fn record_indexed_file(&self, uri: Url) {
        self.indexed_files.write().expect("indexed file lock poisoned").insert(uri);
    }

    fn remove_indexed_file(&self, uri: &Url) {
        self.index.remove_file(uri);
        self.semantic.remove_file(uri);
        self.indexed_files.write().expect("indexed file lock poisoned").remove(uri);
    }

    fn scan_core_source(&self, roots: &[Url]) {
        let configured = self.config.read().expect("server config lock poisoned").sysroot_path.clone();
        let mut candidates = Vec::new();
        if let Some(path) = configured {
            if path.extension().and_then(|extension| extension.to_str()) == Some("ph") {
                candidates.push(path);
            } else {
                candidates.extend([path.join("core/core.ph"), path.join("phalcom-core/core/core.ph"), path.join("core.ph")]);
            }
        }
        for root in roots {
            if let Ok(path) = root.to_file_path() {
                candidates.extend([path.join("phalcom-core/core/core.ph"), path.join("core/core.ph")]);
            }
        }
        if let Some((path, text)) = candidates
            .into_iter()
            .find_map(|path| std::fs::read_to_string(&path).ok().map(|text| (path, text)))
        {
            let parse = phalcom_ast::parser::parse(&text, 0);
            self.semantic.update_core(crate::semantic::FileRevision(1), &parse.program);
            if let Ok(uri) = Url::from_file_path(path) {
                self.set_core_source_uri(uri.clone());
                self.record_indexed_file(uri);
            }
        }
    }

    /// Scans every `.ph` file under `roots` and (re)builds the workspace
    /// index from scratch.
    ///
    /// Called once from `initialize`. A synchronous filesystem walk —
    /// Stage 2's scan is a one-time startup cost, not a hot path, so no
    /// async I/O or background task is warranted here (plan "Build order"
    /// step 2).
    fn scan_workspace(&self, roots: &[Url]) {
        let mut semantic_files = Vec::new();
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
                semantic_files.push((uri.clone(), crate::semantic::FileRevision(1), parse.program));
                self.record_indexed_file(uri);
            }
        }
        self.semantic.update_files_batch(semantic_files);
        self.scan_core_source(roots);
    }

    fn refresh_closed_file(&self, uri: &Url) {
        if self.documents.with_document(uri, |_| ()).is_some() {
            return;
        }
        let Ok(path) = uri.to_file_path() else { return };
        let Ok(text) = std::fs::read_to_string(path) else {
            self.remove_indexed_file(uri);
            if self.is_core_source_uri(uri) {
                let bundled = crate::semantic::core_source::bundled_parse();
                self.semantic.update_core(crate::semantic::FileRevision(1), &bundled.program);
                self.clear_core_source_uris();
            }
            return;
        };
        let parse = phalcom_ast::parser::parse(&text, 0);
        self.index.update_file(uri.clone(), &parse.program);
        self.update_semantic_for_source(uri, crate::semantic::FileRevision(1), &parse.program);
        self.record_indexed_file(uri.clone());
    }

    fn remove_workspace_root(&self, root: &Url) -> bool {
        let Ok(root_path) = root.to_file_path() else { return false };
        let files = self
            .indexed_files
            .read()
            .expect("indexed file lock poisoned")
            .iter()
            .filter(|uri| uri.to_file_path().ok().is_some_and(|path| path.starts_with(&root_path)))
            .cloned()
            .collect::<Vec<_>>();
        let removed_core_source = files.iter().any(|uri| self.is_core_source_uri(uri));
        for uri in files {
            self.remove_indexed_file(&uri);
        }
        removed_core_source
    }

    /// Resolves a recovered completion target through live semantic facts.
    fn semantic_receiver(&self, uri: &Url, doc: &crate::documents::Document, position: Position) -> Option<completion::SemanticResolvedReceiver> {
        let target = completion::target_at(doc, position)?;
        let receiver = doc.text.get(target.receiver_range.start..target.receiver_range.end)?;
        let offset = target.receiver_range.end;
        if receiver == "self" {
            return self.semantic.class_at(uri, offset).map(|class| completion::SemanticResolvedReceiver {
                alternatives: vec![(class, completion::ReceiverKind::Instance)],
            });
        }
        if receiver == "super" {
            return self
                .semantic
                .class_at(uri, offset)
                .and_then(|class| self.semantic.class_surface(&class)?.superclass)
                .map(|class| completion::SemanticResolvedReceiver {
                    alternatives: vec![(class, completion::ReceiverKind::Instance)],
                });
        }
        if receiver.chars().next().is_some_and(char::is_uppercase) && receiver.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
            return self.semantic.class_for_name(uri, receiver).map(|class| completion::SemanticResolvedReceiver {
                alternatives: vec![(class, completion::ReceiverKind::ClassObject)],
            });
        }
        if receiver.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
            if let Some(value) = self.semantic.binding_at(uri, receiver, offset) {
                if let Some(resolved) = receiver_from_shape(value.shape) {
                    return Some(resolved);
                }
            }
        }
        let parse = phalcom_ast::parser::parse(receiver, target.receiver_range.start);
        let expression = parse.program.statements.iter().find_map(|statement| match statement {
            phalcom_ast::ast::Statement::Expr { expr, .. } => Some(expr),
            _ => None,
        })?;
        receiver_from_shape(self.semantic.infer_expression(uri, expression, offset).shape)
    }

    fn semantic_member_targets(&self, uri: &Url, position: Position, selector: &str) -> Option<Vec<ResolvedMemberTarget>> {
        let receiver_targeted = self
            .documents
            .with_document(uri, |doc| completion::target_at(doc, position).is_some())
            .unwrap_or(false);
        if !receiver_targeted {
            return None;
        }

        let resolved = self.documents.with_document(uri, |doc| self.semantic_receiver(uri, doc, position)).flatten();
        let Some(resolved) = resolved else {
            return Some(Vec::new());
        };

        let mut seen = BTreeSet::new();
        let mut targets = Vec::new();
        for (receiver, receiver_kind) in resolved.alternatives {
            let side = match receiver_kind {
                completion::ReceiverKind::Instance => crate::semantic::DispatchSide::Instance,
                completion::ReceiverKind::ClassObject => crate::semantic::DispatchSide::Class,
            };
            let Some(member) = self.semantic.receiver_member(&receiver, selector, side) else {
                continue;
            };
            if seen.insert(member.callable.clone()) {
                targets.push(ResolvedMemberTarget { receiver, member });
            }
        }
        Some(targets)
    }

    fn member_definition_location(&self, member: &crate::semantic::MemberSurface) -> Option<Location> {
        let owner = &member.callable.owner;
        if owner.module.as_str() == crate::semantic::CORE_MODULE_URI {
            return None;
        }
        let definition_uri = Url::parse(owner.module.as_str()).ok()?;
        let range = self.with_source_snapshot(&definition_uri, |_, _, line_index| {
            line_index.range(member.name_range.start..member.name_range.end)
        })?;
        let location_uri = self
            .index
            .definition_info(&member.callable.selector)
            .into_iter()
            .find(|info| {
                if info.class != owner.name {
                    return false;
                }
                let Some(indexed_path) = info.uri.to_file_path().ok() else { return false };
                let Some(semantic_path) = definition_uri.to_file_path().ok() else {
                    return false;
                };
                std::fs::canonicalize(indexed_path).ok() == std::fs::canonicalize(semantic_path).ok()
            })
            .map(|info| info.uri)
            .unwrap_or(definition_uri);
        Some(Location { uri: location_uri, range })
    }

    fn semantic_definition_locations(&self, uri: &Url, position: Position, selector: &str) -> Vec<Location> {
        let targets = self.semantic_member_targets(uri, position, selector).unwrap_or_default();
        targets.iter().filter_map(|target| self.member_definition_location(&target.member)).collect()
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
    /// so Stage 4's cross-file Phaldoc harvest ([`Self::member_phaldoc`]) can
    /// inspect a declaration's *defining* file even when that file is not the one currently open
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

    fn semantic_class_target(&self, uri: &Url, position: Position) -> Option<(crate::semantic::ClassSurface, tower_lsp::lsp_types::Range)> {
        self.documents
            .with_document(uri, |doc| {
                let offset = doc.line_index.offset(position);
                if let Some(class) = self.semantic.class_name_at(uri, offset) {
                    let range = doc.line_index.range(class.name_range.start..class.name_range.end);
                    return Some((class, range));
                }
                let (name, name_range) = hover::qualified_identifier_at_offset(&doc.text, offset)?;
                let class_id = self.semantic.class_for_name(uri, &name)?;
                let class = self.semantic.class_surface(&class_id)?;
                Some((class, doc.line_index.range(name_range)))
            })
            .flatten()
    }

    fn class_definition_location(&self, class: &crate::semantic::ClassSurface) -> Option<Location> {
        let owner = &class.id;
        if owner.module.as_str() == crate::semantic::CORE_MODULE_URI {
            return None;
        }
        let definition_uri = Url::parse(owner.module.as_str()).ok()?;
        let range = self.with_source_snapshot(&definition_uri, |_, _, line_index| {
            line_index.range(class.name_range.start..class.name_range.end)
        })?;
        Some(Location { uri: definition_uri, range })
    }

    fn member_phaldoc(&self, member: &crate::semantic::MemberSurface) -> Option<hover::PhaldocDoc> {
        let owner = &member.callable.owner;
        if owner.module.as_str() == crate::semantic::CORE_MODULE_URI {
            return None;
        }
        let definition_uri = Url::parse(owner.module.as_str()).ok()?;
        self.with_source_snapshot(&definition_uri, |text, program, line_index| {
            let target = hover::DeclarationDocTarget::Member {
                declaration: member.source_range,
                name: member.name_range,
            };
            hover::harvest_doc_for_declaration(text, line_index, target)
                .or_else(|| hover::harvest_pinned_doc_for_member(text, program, &owner.name, &member.callable.selector, member.source_range))
        })
        .flatten()
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
    /// live semantic core surface, and the harvested Phaldoc doc (from the selector's
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
        let offset = self.documents.with_document(uri, |doc| doc.line_index.offset(position))?;
        let occurrence = self.semantic.occurrence_at(uri, offset)?;
        let span = self.documents.with_document(uri, |doc| doc.line_index.range(occurrence.range.start..occurrence.range.end))?;

        match occurrence.target {
            crate::semantic::SemanticTarget::Binding(binding) => {
                let info = self.semantic.binding_info(uri, binding)?;
                let value = self.semantic.binding_at(uri, &info.name, offset);
                let phaldoc = self.documents.with_document(uri, |doc| {
                    hover::harvest_doc_for_selector(&doc.text, &doc.parse.program, &doc.line_index, &info.name)
                })?;
                Some(Hover {
                    contents: markdown_contents(hover::render_binding_hover(&info, value.as_ref(), phaldoc.as_ref())),
                    range: Some(span),
                })
            }
            crate::semantic::SemanticTarget::Class(class_id) => {
                let class = self.semantic.class_surface(&class_id)?;
                let phaldoc = if class.id.module.as_str() == crate::semantic::CORE_MODULE_URI {
                    None
                } else {
                    Url::parse(class.id.module.as_str()).ok().and_then(|definition_uri| {
                        self.with_source_snapshot(&definition_uri, |text, _, line_index| {
                            hover::harvest_doc_for_declaration(
                                text,
                                line_index,
                                hover::DeclarationDocTarget::Class {
                                    declaration: class.source_range,
                                    name: class.name_range,
                                },
                            )
                        })
                        .flatten()
                    })
                };
                Some(Hover {
                    contents: markdown_contents(hover::render_class_hover(&class.id, class.superclass.as_ref(), phaldoc.as_ref())),
                    range: Some(span),
                })
            }
            crate::semantic::SemanticTarget::Callable(callable) => {
                let member = self.semantic.member_surface(&callable.owner, &callable.selector)?;
                let site = SelectorSite {
                    owner: member.callable.owner.clone(),
                    receiver: None,
                    kind: hover_member_kind(&member),
                };
                let phaldoc = self.member_phaldoc(&member);
                let value = hover::render_selector_hover_with_value(
                    &callable.selector,
                    &[site],
                    phaldoc.as_ref(),
                    self.semantic.return_for_callable(&member.callable).as_ref(),
                )?;
                Some(Hover {
                    contents: markdown_contents(value),
                    range: Some(span),
                })
            }
            crate::semantic::SemanticTarget::Field { owner, name } => {
                let member = self.semantic.member_surface(&owner, &name)?;
                let site = SelectorSite {
                    owner: member.callable.owner.clone(),
                    receiver: None,
                    kind: hover_member_kind(&member),
                };
                let phaldoc = self.member_phaldoc(&member);
                let value = hover::render_selector_hover_with_value(&name, &[site], phaldoc.as_ref(), None)?;
                Some(Hover {
                    contents: markdown_contents(value),
                    range: Some(span),
                })
            }
            crate::semantic::SemanticTarget::Member { .. } => {
                let (selector, selector_span) = self.selector_at_position(uri, position)?;
                if selector_span != span {
                    return None;
                }
                let targets = self.semantic_member_targets(uri, position, &selector)?;
                let mut sites = Vec::new();
                let mut ids = Vec::new();
                let mut docs = Vec::new();
                for target in &targets {
                    sites.push(SelectorSite {
                        owner: target.member.callable.owner.clone(),
                        receiver: Some(target.receiver.clone()),
                        kind: hover_member_kind(&target.member),
                    });
                    ids.push(target.member.callable.clone());
                    if target.member.callable.owner.module.as_str() != crate::semantic::CORE_MODULE_URI
                        && let Some(doc) = self.member_phaldoc(&target.member)
                        && !docs.contains(&doc)
                    {
                        docs.push(doc);
                    }
                }
                let phaldoc = (docs.len() == 1).then(|| docs.remove(0));
                let inferred = self.semantic.returns_for_callables(ids);
                let value = hover::render_selector_hover_with_value(&selector, &sites, phaldoc.as_ref(), inferred.as_ref())?;
                Some(Hover {
                    contents: markdown_contents(value),
                    range: Some(span),
                })
            }
            crate::semantic::SemanticTarget::Operator(_) => None,
        }
    }

}

/// Recovers semantic structure lost when a live member-access expression ends
/// at the cursor, e.g. `_client.` inside a method body. The user-facing parse
/// and diagnostics stay untouched; only the semantic snapshot sees the dot
/// replaced by a space, keeping every source range stable.
fn semantic_recovery_parse(text: &str, parsed: &phalcom_ast::parser::Parse) -> Option<phalcom_ast::parser::Parse> {
    if parsed.errors.is_empty() {
        return None;
    }
    let recovered_text = blank_incomplete_member_dots(text);
    if recovered_text == text {
        return None;
    }
    let candidate = phalcom_ast::parser::parse(&recovered_text, 0);
    (candidate.errors.len() <= parsed.errors.len() && candidate.program.statements.len() >= parsed.program.statements.len()).then_some(candidate)
}

fn blank_incomplete_member_dots(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut output = bytes.to_vec();
    let mut quote = None;
    let mut escaped = false;
    for index in 0..bytes.len() {
        let byte = bytes[index];
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
            continue;
        }
        if byte == b'"' || byte == b'\'' {
            quote = Some(byte);
            continue;
        }
        if byte != b'.' || index == 0 || !is_member_receiver_byte(bytes[index - 1]) {
            continue;
        }
        let mut next = index + 1;
        while next < bytes.len() && bytes[next].is_ascii_whitespace() {
            next += 1;
        }
        if next == bytes.len() || matches!(bytes[next], b'}' | b')' | b']' | b',') {
            output[index] = b' ';
        }
    }
    String::from_utf8(output).expect("replacing ASCII bytes preserves UTF-8")
}

fn is_member_receiver_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b')' | b']')
}

fn receiver_from_shape(shape: ValueShape) -> Option<completion::SemanticResolvedReceiver> {
    let alternatives = match shape {
        ValueShape::Instance(class) => vec![(class, completion::ReceiverKind::Instance)],
        ValueShape::ClassObject(class) => vec![(class, completion::ReceiverKind::ClassObject)],
        ValueShape::Union(shapes) => shapes
            .into_iter()
            .filter_map(|shape| match shape {
                ValueShape::Instance(class) => Some((class, completion::ReceiverKind::Instance)),
                ValueShape::ClassObject(class) => Some((class, completion::ReceiverKind::ClassObject)),
                _ => None,
            })
            .collect(),
        _ => return None,
    };
    (!alternatives.is_empty()).then_some(completion::SemanticResolvedReceiver { alternatives })
}

fn hover_member_kind(member: &crate::semantic::MemberSurface) -> crate::index::MemberKind {
    match member.kind {
        crate::semantic::MemberKind::Getter => crate::index::MemberKind::Getter,
        crate::semantic::MemberKind::Setter => crate::index::MemberKind::Setter,
        crate::semantic::MemberKind::Field => crate::index::MemberKind::Getter,
        crate::semantic::MemberKind::Method | crate::semantic::MemberKind::Index | crate::semantic::MemberKind::Variant => {
            if member.side == crate::semantic::DispatchSide::Class && member.is_constructor {
                crate::index::MemberKind::Construct
            } else if member.side == crate::semantic::DispatchSide::Class {
                crate::index::MemberKind::StaticMethod
            } else {
                crate::index::MemberKind::Method
            }
        }
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
        let config = ServerConfig::from_json(params.initialization_options.as_ref());
        *self.config.write().expect("server config lock poisoned") = config;
        let dynamic_watch = params
            .capabilities
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.did_change_watched_files.as_ref())
            .and_then(|watch| watch.dynamic_registration)
            .unwrap_or(false);
        *self.watch_registration.write().expect("watch registration lock poisoned") = dynamic_watch;
        let mut roots: Vec<Url> = params.workspace_folders.unwrap_or_default().into_iter().map(|folder| folder.uri).collect();
        #[allow(deprecated)]
        if let Some(root_uri) = params.root_uri {
            if !roots.contains(&root_uri) {
                roots.push(root_uri);
            }
        }
        *self.workspace_roots.write().expect("workspace root lock poisoned") = roots.clone();
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
                inlay_hint_provider: Some(OneOf::Right(InlayHintServerCapabilities::Options(InlayHintOptions {
                    resolve_provider: Some(false),
                    ..InlayHintOptions::default()
                }))),
                semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                    legend: semantic_tokens::legend(),
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                    ..SemanticTokensOptions::default()
                })),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    ..WorkspaceServerCapabilities::default()
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
        self.client.log_message(MessageType::INFO, "phalcom-lsp initialized").await;
        if !*self.watch_registration.read().expect("watch registration lock poisoned") {
            return;
        }
        let _ = self
            .client
            .register_capability(vec![Registration {
                id: "phalcom-ph-source-watch".to_string(),
                method: "workspace/didChangeWatchedFiles".to_string(),
                register_options: Some(serde_json::json!({
                    "watchers": [{ "globPattern": "**/*.ph" }]
                })),
            }])
            .await;
    }

    /// Applies changed settings and refreshes the configured core source.
    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let config = ServerConfig::from_json(Some(&params.settings));
        *self.config.write().expect("server config lock poisoned") = config;
        self.clear_core_source_uris();
        let bundled = crate::semantic::core_source::bundled_parse();
        self.semantic.update_core(crate::semantic::FileRevision(1), &bundled.program);
        let roots = self.workspace_roots.read().expect("workspace root lock poisoned").clone();
        self.scan_core_source(&roots);
    }

    /// Refreshes semantic/index state when workspace folders are added or removed.
    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        let mut removed_core_source = false;
        for folder in &params.event.removed {
            removed_core_source |= self.remove_workspace_root(&folder.uri);
        }
        let roots = {
            let mut roots = self.workspace_roots.write().expect("workspace root lock poisoned");
            roots.retain(|root| !params.event.removed.iter().any(|folder| folder.uri == *root));
            for folder in &params.event.added {
                if !roots.contains(&folder.uri) {
                    roots.push(folder.uri.clone());
                }
            }
            roots.clone()
        };
        if removed_core_source {
            self.clear_core_source_uris();
            let bundled = crate::semantic::core_source::bundled_parse();
            self.semantic.update_core(crate::semantic::FileRevision(1), &bundled.program);
            self.scan_core_source(&roots);
        }
        let added = params.event.added.iter().map(|folder| folder.uri.clone()).collect::<Vec<_>>();
        self.scan_workspace(&added);
    }

    /// Refreshes closed-file contributions for watched `.ph` changes.
    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        for change in params.changes {
            if change.typ == FileChangeType::DELETED {
                self.remove_indexed_file(&change.uri);
                if self.is_core_source_uri(&change.uri) {
                    let bundled = crate::semantic::core_source::bundled_parse();
                    self.semantic.update_core(crate::semantic::FileRevision(1), &bundled.program);
                    self.clear_core_source_uris();
                }
            } else {
                self.refresh_closed_file(&change.uri);
            }
        }
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
                self.update_semantic_for_source(&uri, revision, &parse.program);
                self.record_indexed_file(uri.clone());
            } else {
                self.remove_indexed_file(&uri);
                if self.is_core_source_uri(&uri) {
                    let bundled = crate::semantic::core_source::bundled_parse();
                    self.semantic.update_core(revision, &bundled.program);
                    self.clear_core_source_uris();
                }
            }
        } else {
            self.remove_indexed_file(&uri);
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

        if let Some((class, _)) = self.semantic_class_target(&uri, position) {
            return Ok(self
                .class_definition_location(&class)
                .map(|location| GotoDefinitionResponse::Array(vec![location])));
        }

        let Some((selector, _range)) = self.selector_at_position(&uri, position) else {
            return Ok(None);
        };

        let offset = self.documents.with_document(&uri, |doc| doc.line_index.offset(position));
        if let Some(member) = offset
            .and_then(|offset| self.semantic.member_at(&uri, offset))
            .filter(|member| member.callable.selector == selector)
        {
            return Ok(self
                .member_definition_location(&member)
                .map(|location| GotoDefinitionResponse::Array(vec![location])));
        }

        let receiver_targeted = self
            .documents
            .with_document(&uri, |doc| completion::target_at(doc, position).is_some())
            .unwrap_or(false);
        if receiver_targeted {
            let semantic_locations = self.semantic_definition_locations(&uri, position, &selector);
            return if semantic_locations.is_empty() {
                Ok(None)
            } else {
                Ok(Some(GotoDefinitionResponse::Array(semantic_locations)))
            };
        }

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
    /// Resolves the receiver under the cursor through the semantic database,
    /// then renders live source/native selectors as snippet completion items.
    /// Unknown receivers use the bounded live workspace surface.
    ///
    /// Returns `Ok(None)` if the document is not open.
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let items: Option<Vec<CompletionItem>> = self.documents.with_document(&uri, |doc| {
            let resolved = self.semantic_receiver(&uri, doc, position);
            let offset = doc.line_index.offset(position);
            let privileged = self.is_core_source_uri(&uri);
            let lexical_class = self.semantic.class_at(&uri, offset);
            let mut items = completion::semantic_contextual_completions(
                &self.semantic,
                completion::SemanticCompletionContext {
                    resolved: resolved.as_ref(),
                    lexical_class: lexical_class.as_ref(),
                    privileged,
                    uri: &uri,
                    program: &doc.parse.program,
                    text: &doc.text,
                    offset,
                },
            );
            if let Some(target) = completion::target_at(doc, position) {
                if !target.partial_member.is_empty() {
                    items.retain(|item| item.label.starts_with(&target.partial_member));
                }
            }
            items
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

    /// Answers standard inlay-hint requests from the live semantic database.
    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let config = self.config.read().expect("server config lock poisoned").clone();
        let uri = params.text_document.uri.clone();
        let hints = self.documents.with_document(&uri, |doc| {
            crate::inlay_hints::hints_for_params_with_policy(&self.semantic, &uri, doc, &params, config.inlay_hints, config.suppress_obvious)
        });
        Ok(hints)
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{HintPolicy, ServerConfig};

    #[test]
    fn server_config_parses_nested_stable_hint_policy_and_sysroot_directory() {
        let settings = json!({
            "phalcom": {
                "lsp": { "sysrootPath": "/opt/phalcom" },
                "inlayHints": { "types": "stable", "suppressObvious": false }
            }
        });

        let config = ServerConfig::from_json(Some(&settings));

        assert_eq!(config.sysroot_path.as_deref(), Some(std::path::Path::new("/opt/phalcom")));
        assert_eq!(config.inlay_hints, HintPolicy::Stable);
        assert!(!config.suppress_obvious);
    }

    #[test]
    fn server_config_parses_dotted_off_hint_policy() {
        let settings = json!({
            "phalcom.lsp.sysrootPath": "/opt/phalcom/core/core.ph",
            "phalcom.inlayHints.types": "off"
        });

        let config = ServerConfig::from_json(Some(&settings));

        assert_eq!(config.sysroot_path.as_deref(), Some(std::path::Path::new("/opt/phalcom/core/core.ph")));
        assert_eq!(config.inlay_hints, HintPolicy::Off);
    }

    #[test]
    fn server_config_parses_all_hints_and_obvious_initializer_suppression() {
        let settings = json!({
            "phalcom": {
                "inlayHints": { "types": "all", "suppressObvious": true }
            }
        });

        let config = ServerConfig::from_json(Some(&settings));

        assert_eq!(config.inlay_hints, HintPolicy::All);
        assert!(config.suppress_obvious);
    }
}
