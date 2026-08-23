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

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use crate::analysis_service::{AnalysisEvent, AnalysisService, CachedSource, DiskRefresh, SourceCache, WorkspaceScanRequest};
use crate::analysis_status::{AnalysisPhase, AnalysisStatus, AnalysisStatusNotification};

use serde_json::Value as JsonValue;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionOptions, CompletionParams, CompletionResponse, DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidChangeWorkspaceFoldersParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, FileChangeType, GotoDefinitionParams, GotoDefinitionResponse,
    Hover, HoverContents, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams, InlayHint, InlayHintOptions,
    InlayHintParams, InlayHintServerCapabilities, Location, MarkupContent, MarkupKind, MessageType, OneOf, Position, PositionEncodingKind, ReferenceParams,
    Registration, SemanticTokensFullOptions, SemanticTokensOptions, SemanticTokensParams, SemanticTokensResult, SemanticTokensServerCapabilities,
    ServerCapabilities, SymbolInformation, SymbolKind, TextDocumentSyncCapability, TextDocumentSyncKind, Url, WorkspaceFoldersServerCapabilities,
    WorkspaceServerCapabilities, WorkspaceSymbolParams,
};
use tower_lsp::{Client, LanguageServer};

use crate::completion;
use crate::diagnostics::syntax_errors_to_diagnostics;
use crate::documents::{DocumentSnapshot, DocumentStore};
use crate::hover::{self, SelectorSite};
use crate::index::{self, Occurrence, WorkspaceIndex};
use crate::inlay_hints::HintPolicy;
use crate::line_index::LineIndex;
use crate::perf::{PerfCountersHandle, PerfSpan};
use crate::request_context::RequestContext;
use crate::semantic::{FileRevision, OccurrenceRole, SemanticDb, SemanticSnapshot, SemanticTarget, ValueShape};
use crate::semantic_tokens;

use crate::workspace_scan::AnalysisMode;

struct DiagnosticPublication {
    diagnostics: Vec<tower_lsp::lsp_types::Diagnostic>,
    version: Option<i32>,
}

fn combined_diagnostics_for(documents: &DocumentStore, semantic: &SemanticDb, uri: &Url) -> Option<DiagnosticPublication> {
    let document = documents.snapshot(uri)?;
    let mut diagnostics = syntax_errors_to_diagnostics(&document.parse.errors, &document.line_index);
    let syntax_only = |diagnostics| DiagnosticPublication {
        diagnostics,
        version: document.version,
    };
    let advisory = semantic.snapshot();
    let Some(module) = advisory.module_for_uri(uri) else {
        return Some(syntax_only(diagnostics));
    };
    let Some(file) = advisory.file(module) else {
        return Some(syntax_only(diagnostics));
    };
    let Some(static_snapshot) = advisory.static_snapshot.as_ref() else {
        return Some(syntax_only(diagnostics));
    };
    if file.revision != document.revision || static_snapshot.generation != advisory.generation.0 {
        return Some(syntax_only(diagnostics));
    }
    let Some(static_module) = advisory.documents.get_by_uri(uri) else {
        return Some(syntax_only(diagnostics));
    };
    let Some(static_source) = static_snapshot.sources.get(static_module) else {
        return Some(syntax_only(diagnostics));
    };
    if static_source.text.as_ref() != document.text.as_ref() {
        return Some(syntax_only(diagnostics));
    }
    if let Some(semantic_diagnostics) = static_snapshot.diagnostics.get(static_module) {
        diagnostics.extend(crate::diagnostics::semantic_diagnostics_to_lsp_diagnostics(
            semantic_diagnostics,
            &document.line_index,
            uri,
        ));
    }
    Some(DiagnosticPublication {
        diagnostics,
        version: document.version,
    })
}

/// Runtime configuration that affects semantic source discovery and hint UI.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Explicit sysroot/core source directory, if configured.
    pub sysroot_path: Option<PathBuf>,
    /// Analysis scope mode.
    pub analysis_mode: AnalysisMode,
    /// Glob-style workspace paths excluded from Phalcom source indexing.
    pub analysis_exclude: Vec<String>,
    /// Inlay-hint display policy.
    pub inlay_hints: HintPolicy,
    /// Whether obvious literal initializers should omit their hint.
    pub suppress_obvious: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            sysroot_path: None,
            analysis_mode: AnalysisMode::Local,
            analysis_exclude: Vec::new(),
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
        let analysis = root.get("analysis").unwrap_or(root);

        if let Some(path) = value.get("phalcom.lsp.sysrootPath").and_then(JsonValue::as_str) {
            config.sysroot_path = Some(PathBuf::from(path));
        }
        if let Some(path) = root.get("lsp").and_then(|value| value.get("sysrootPath")).and_then(JsonValue::as_str) {
            config.sysroot_path = Some(PathBuf::from(path));
        } else if let Some(path) = lsp.get("sysrootPath").and_then(JsonValue::as_str) {
            config.sysroot_path = Some(PathBuf::from(path));
        }

        if let Some(mode) = value.get("phalcom.analysis.mode").and_then(JsonValue::as_str) {
            config.analysis_mode = mode.parse().unwrap_or(AnalysisMode::Local);
        } else if let Some(mode) = analysis.get("mode").and_then(JsonValue::as_str) {
            config.analysis_mode = mode.parse().unwrap_or(AnalysisMode::Local);
        }

        if let Some(exclude) = value.get("phalcom.analysis.exclude").and_then(JsonValue::as_array) {
            config.analysis_exclude = exclude.iter().filter_map(JsonValue::as_str).map(ToString::to_string).collect();
        } else if let Some(exclude) = analysis.get("exclude").and_then(JsonValue::as_array) {
            config.analysis_exclude = exclude.iter().filter_map(JsonValue::as_str).map(ToString::to_string).collect();
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

/// Semantic analysis configuration subset used to determine invalidation on setting changes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisConfig {
    /// Configured sysroot path.
    pub sysroot_path: Option<PathBuf>,
    /// Analysis scope mode.
    pub mode: AnalysisMode,
    /// Workspace path fragments or file patterns excluded from indexing.
    pub excludes: Vec<String>,
}

impl ServerConfig {
    /// Extracts the semantic analysis configuration subset.
    pub fn analysis_config(&self) -> AnalysisConfig {
        AnalysisConfig {
            sysroot_path: self.sysroot_path.clone(),
            mode: self.analysis_mode,
            excludes: self.analysis_exclude.clone(),
        }
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
    index: Arc<WorkspaceIndex>,
    /// Live VM-free semantic state snapshot reader.
    semantic: Arc<SemanticDb>,
    /// Background semantic analysis service.
    analysis: AnalysisService,
    /// Receiver for worker analysis events (taken in `initialized`).
    analysis_events: Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<AnalysisEvent>>>,
    /// Current workspace roots advertised by the client.
    workspace_roots: RwLock<Vec<Url>>,
    /// Files currently represented in the workspace index.
    indexed_files: Arc<RwLock<BTreeSet<Url>>>,
    /// Closed-file text, parse, and line metadata populated by worker/index events.
    closed_sources: SourceCache,
    /// Mutable server configuration.
    config: RwLock<ServerConfig>,
    /// URI for the active on-disk core source, if one replaced bundled core.
    ///
    /// This must be explicit: configured sysroots need the same open-buffer
    /// precedence as repository's conventional canonical universe package source.
    core_source_uris: Arc<RwLock<BTreeSet<Url>>>,
    /// Whether client requested dynamic watched-file registration.
    watch_registration: RwLock<bool>,
    inlay_refresh: Arc<PublicationRefresh>,
    semantic_token_refresh: Arc<PublicationRefresh>,
}

#[derive(Default)]
struct PublicationRefresh {
    state: Mutex<PublicationRefreshState>,
}

#[derive(Default)]
struct PublicationRefreshState {
    in_flight: bool,
    pending: bool,
}

impl PublicationRefresh {
    fn request(&self) -> bool {
        let mut state = self.state.lock().expect("publication refresh lock poisoned");
        state.pending = true;
        if state.in_flight {
            false
        } else {
            state.in_flight = true;
            state.pending = false;
            true
        }
    }

    fn finished_refresh(&self) -> bool {
        let mut state = self.state.lock().expect("publication refresh lock poisoned");
        if state.pending {
            state.pending = false;
            true
        } else {
            state.in_flight = false;
            false
        }
    }
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
        let counters = Arc::new(crate::perf::PerfCounters::new());
        let _span = PerfSpan::start_with_counters("backend_construction", counters.clone());
        let db = Arc::new(SemanticDb::with_counters(counters));
        let index = Arc::new(WorkspaceIndex::new());
        let closed_sources = Arc::new(RwLock::new(BTreeMap::new()));
        let (analysis, event_rx) = AnalysisService::new_with_index_and_cache(db.clone(), Some(index.clone()), Some(closed_sources.clone()));
        Self {
            client,
            documents: DocumentStore::new(),
            index,
            semantic: db,
            analysis,
            analysis_events: Mutex::new(Some(event_rx)),
            workspace_roots: RwLock::new(Vec::new()),
            indexed_files: Arc::new(RwLock::new(BTreeSet::new())),
            closed_sources,
            config: RwLock::new(ServerConfig::default()),
            core_source_uris: Arc::new(RwLock::new(BTreeSet::new())),
            watch_registration: RwLock::new(false),
            inlay_refresh: Arc::new(PublicationRefresh::default()),
            semantic_token_refresh: Arc::new(PublicationRefresh::default()),
        }
    }

    /// Returns this backend's compact performance counters for diagnostics and
    /// benchmark harnesses. The counters are owned by this backend's worker.
    pub fn perf_counters(&self) -> PerfCountersHandle {
        self.semantic.perf_counters()
    }

    /// Pins open-document data and one published semantic generation before
    /// any request-local work begins. The document map guard is released by
    /// `DocumentStore::snapshot` before this returns.
    fn request_context(&self, uri: &Url) -> Option<RequestContext> {
        let document = self.documents.snapshot(uri)?;
        Some(RequestContext::new(document, self.semantic.snapshot(), uri))
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
        self.documents.with_document(&uri, |doc| {
            self.index.update_file(uri.clone(), &doc.parse.program);
        });
        let diagnostics = combined_diagnostics_for(&self.documents, &self.semantic, &uri)
            .map(|publication| publication.diagnostics)
            .unwrap_or_default();
        self.client.publish_diagnostics(uri, diagnostics, version).await;
    }

    fn update_semantic_for_source(&self, uri: &Url, revision: crate::semantic::FileRevision, text: Arc<str>, program: &phalcom_ast::ast::Program) {
        if self.is_core_source_uri(uri) {
            self.analysis.enqueue_core_update_with_source(revision, text, program.clone());
        } else {
            self.analysis.enqueue_file_update_with_source(uri.clone(), revision, text, program.clone());
        }
    }

    fn is_core_source_uri(&self, uri: &Url) -> bool {
        self.core_source_uris.read().expect("core source URI lock poisoned").contains(uri)
    }

    fn clear_core_source_uris(&self) {
        self.core_source_uris.write().expect("core source URI lock poisoned").clear();
    }

    fn cache_source(&self, uri: Url, revision: FileRevision, text: impl Into<Arc<str>>, program: impl Into<Arc<phalcom_ast::ast::Program>>) {
        let text = text.into();
        let program = program.into();
        let source = CachedSource {
            revision,
            line_index: Arc::new(LineIndex::new(&text)),
            text,
            program,
        };
        let canonical = uri
            .to_file_path()
            .ok()
            .and_then(|path| path.canonicalize().ok())
            .and_then(|path| Url::from_file_path(path).ok());
        let mut cache = self.closed_sources.write().expect("closed source cache lock poisoned");
        cache.insert(uri, source.clone());
        if let Some(canonical) = canonical {
            cache.insert(canonical, source);
        }
    }

    fn remove_indexed_file(&self, uri: &Url) {
        self.index.remove_file(uri);
        self.analysis.enqueue_file_removal(uri.clone());
        self.closed_sources.write().expect("closed source cache lock poisoned").remove(uri);
        self.indexed_files.write().expect("indexed file lock poisoned").remove(uri);
    }

    fn schedule_workspace_scan(&self, roots: &[Url]) {
        let config = self.config.read().expect("server config lock poisoned").clone();
        self.clear_core_source_uris();
        let filesystem_roots = roots.iter().filter_map(|root| root.to_file_path().ok()).collect();
        self.analysis.configure_workspace(WorkspaceScanRequest {
            roots: filesystem_roots,
            mode: config.analysis_mode,
            excludes: config.analysis_exclude,
            core_source_path: config.sysroot_path,
        });
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
        for uri in &files {
            self.index.remove_file(uri);
            self.closed_sources.write().expect("closed source cache lock poisoned").remove(uri);
        }
        self.indexed_files
            .write()
            .expect("indexed file lock poisoned")
            .retain(|uri| !files.contains(uri));
        self.analysis.enqueue_file_removals(files);
        removed_core_source
    }

    /// Resolves a recovered completion target through live semantic facts.
    fn semantic_receiver(
        &self,
        semantic: &SemanticSnapshot,
        uri: &Url,
        doc: &DocumentSnapshot,
        position: Position,
    ) -> Option<completion::SemanticResolvedReceiver> {
        let target = completion::target_at_snapshot(doc, position)?;
        let receiver = doc.text.get(target.receiver_range.start..target.receiver_range.end)?;
        let offset = target.receiver_range.end;
        if receiver == "self" {
            return semantic.class_at(uri, offset).map(|class| completion::SemanticResolvedReceiver {
                alternatives: vec![(class, completion::ReceiverKind::Instance)],
            });
        }
        if receiver == "super" {
            return semantic
                .class_at(uri, offset)
                .and_then(|class| semantic.class_surface(&class)?.superclass.clone())
                .map(|class| completion::SemanticResolvedReceiver {
                    alternatives: vec![(class, completion::ReceiverKind::Instance)],
                });
        }
        if receiver.chars().next().is_some_and(char::is_uppercase) && receiver.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
            return semantic.class_for_name(uri, receiver).map(|class| completion::SemanticResolvedReceiver {
                alternatives: vec![(class, completion::ReceiverKind::ClassObject)],
            });
        }
        if receiver.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
            if let Some(formal_type) = semantic.formal_binding_type_at(uri, receiver, offset) {
                if let Some(class) = semantic.class_for_name(uri, &formal_type) {
                    let formal_resolved = completion::SemanticResolvedReceiver {
                        alternatives: vec![(class, completion::ReceiverKind::Instance)],
                    };
                    return Some(formal_resolved);
                }
            }
            if let Some(value) = semantic.binding_at(uri, receiver, offset) {
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
        if let Some(formal_expr_type) = semantic.formal_expression_type_at(uri, target.receiver_range.start) {
            if let Some(class) = semantic.class_for_name(uri, &formal_expr_type) {
                return Some(completion::SemanticResolvedReceiver {
                    alternatives: vec![(class, completion::ReceiverKind::Instance)],
                });
            }
        }
        receiver_from_shape(semantic.infer_expression(uri, expression, offset).shape)
    }

    fn semantic_member_targets_for_request(
        &self,
        request: &RequestContext,
        uri: &Url,
        position: Position,
        selector: &str,
    ) -> Option<Vec<ResolvedMemberTarget>> {
        let receiver_targeted = completion::target_at_snapshot(&request.document, position).is_some();
        if !receiver_targeted {
            return None;
        }

        let resolved = self.semantic_receiver(&request.semantic, uri, &request.document, position);
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
            let Some(member) = request.semantic.receiver_member(&receiver, selector, side) else {
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
                info.uri == definition_uri
            })
            .map(|info| info.uri)
            .unwrap_or(definition_uri);
        Some(Location { uri: location_uri, range })
    }

    fn semantic_definition_locations_for_request(&self, request: &RequestContext, uri: &Url, position: Position, selector: &str) -> Vec<Location> {
        let targets = self.semantic_member_targets_for_request(request, uri, position, selector).unwrap_or_default();
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
    fn selector_at_document(&self, doc: &DocumentSnapshot, position: Position) -> Option<(String, tower_lsp::lsp_types::Range)> {
        let offset = doc.line_index.offset(position);
        index::selector_at_offset(&doc.parse.program, offset).map(|(selector, range)| (selector, doc.line_index.range(range.start..range.end)))
    }

    /// Maps one index [`Occurrence`] to an LSP [`Location`].
    ///
    /// The occurrence's file may or may not be currently open in the
    /// client:
    /// - **Open** (tracked in [`DocumentStore`]): map through that
    ///   document's own [`LineIndex`], which reflects the live/unsaved
    ///   buffer — the same text the index was last built from (Stage 2's
    ///   `did_change` wiring keeps them in lockstep).
    /// - **Not open**: maps through the worker-maintained [`CachedSource`]
    ///   metadata for the indexed closed file.
    ///
    /// Returns `None` if the file is not open and has no cached source
    /// metadata.
    fn occurrence_to_location(&self, occurrence: &Occurrence) -> Option<Location> {
        if let Some(doc) = self.documents.snapshot(&occurrence.uri) {
            return Some(Location {
                uri: occurrence.uri.clone(),
                range: doc.line_index.range(occurrence.range.start..occurrence.range.end),
            });
        }

        let source = self.cached_source(&occurrence.uri)?;
        Some(Location {
            uri: occurrence.uri.clone(),
            range: source.line_index.range(occurrence.range.start..occurrence.range.end),
        })
    }

    /// Maps every occurrence in `occurrences` to an LSP [`Location`],
    /// dropping any whose file could not be resolved
    /// ([`Self::occurrence_to_location`]).
    fn occurrences_to_locations(&self, occurrences: &[Occurrence]) -> Vec<Location> {
        occurrences.iter().filter_map(|occ| self.occurrence_to_location(occ)).collect()
    }

    fn cached_source(&self, uri: &Url) -> Option<CachedSource> {
        let source = self.closed_sources.read().expect("closed source cache lock poisoned").get(uri).cloned()?;
        let snapshot = self.semantic.snapshot();
        if let Some(module) = snapshot.module_for_uri(uri)
            && let Some(revision) = snapshot.file_revision(module)
            && revision != source.revision
        {
            return None;
        }
        Some(source)
    }

    /// Runs `f` against a snapshot of `uri`'s source text, parsed
    /// [`phalcom_ast::ast::Program`], and [`LineIndex`] — the same live or
    /// cached path [`Self::occurrence_to_location`] uses, generalized
    /// so Stage 4's cross-file Phaldoc harvest ([`Self::member_phaldoc`]) can
    /// inspect a declaration's *defining* file even when that file is not the one currently open
    /// under the cursor:
    ///
    /// - **Open**: borrows the live/unsaved buffer straight out of the
    ///   [`DocumentStore`] (no reparse).
    /// - **Not open**: reads the worker-maintained [`CachedSource`] entry.
    ///
    /// Returns `None` if `uri` is not open and has no cached source entry.
    fn with_source_snapshot<R>(&self, uri: &Url, f: impl FnOnce(&str, &phalcom_ast::ast::Program, &LineIndex) -> R) -> Option<R> {
        if let Some(doc) = self.documents.snapshot(uri) {
            return Some(f(&doc.text, &doc.parse.program, &doc.line_index));
        }

        let source = self.cached_source(uri)?;
        Some(f(&source.text, &source.program, &source.line_index))
    }

    fn semantic_class_target(
        &self,
        request: &RequestContext,
        uri: &Url,
        position: Position,
    ) -> Option<(crate::semantic::ClassSurface, tower_lsp::lsp_types::Range)> {
        let doc = &request.document;
        let offset = doc.line_index.offset(position);
        if let Some(class) = request.semantic.class_name_at(uri, offset).filter(|_| request.exact_file().is_some()) {
            let range = doc.line_index.range(class.name_range.start..class.name_range.end);
            return Some((class, range));
        }
        let (name, name_range) = hover::qualified_identifier_at_offset(&doc.text, offset)?;
        let class_id = request.semantic.class_for_name(uri, &name)?;
        let class = request.semantic.class_surface(&class_id)?;
        Some((class.clone(), doc.line_index.range(name_range)))
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
        let request = self.request_context(uri)?;
        self.hover_at_request(&request, uri, position)
    }

    fn hover_at_request(&self, request: &RequestContext, uri: &Url, position: Position) -> Option<Hover> {
        let offset = request.document.line_index.offset(position);
        let Some(occurrence) = request.exact_file().and_then(|_| request.semantic.occurrence_at(uri, offset)) else {
            return self.legacy_hover_at(uri, position, offset);
        };
        let span = request.document.line_index.range(occurrence.range.start..occurrence.range.end);

        match occurrence.target {
            crate::semantic::SemanticTarget::Binding(binding) => {
                let info = request.semantic.binding_info(uri, binding)?;
                let value = request.semantic.binding_at(uri, &info.name, offset);
                let formal_type = request.semantic.formal_binding_type_at(uri, &info.name, offset);

                let advisory_str = value.as_ref().map(|v| crate::semantic::render_value_shape(&v.shape));
                crate::parity::ShadowParityHarness::new().record_hover_parity(
                    &info.name,
                    formal_type.as_deref(),
                    advisory_str.as_deref(),
                );

                let phaldoc = hover::harvest_doc_for_selector(
                    &request.document.text,
                    &request.document.parse.program,
                    &request.document.line_index,
                    &info.name,
                );
                Some(Hover {
                    contents: markdown_contents(hover::render_binding_hover_with_formal(&info, formal_type.as_deref(), value.as_ref(), phaldoc.as_ref())),
                    range: Some(span),
                })
            }
            crate::semantic::SemanticTarget::Class(class_id) => {
                let class = request.semantic.class_surface(&class_id)?;
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
                let member = request.semantic.member_surface(&callable)?;
                let site = SelectorSite {
                    owner: member.callable.owner.clone(),
                    receiver: None,
                    kind: hover_member_kind(member),
                };
                let phaldoc = self.member_phaldoc(member);
                let value = hover::render_selector_hover_with_value(
                    &callable.selector,
                    &[site],
                    phaldoc.as_ref(),
                    request.semantic.return_for_callable(&member.callable).as_ref(),
                )?;
                Some(Hover {
                    contents: markdown_contents(value),
                    range: Some(span),
                })
            }
            crate::semantic::SemanticTarget::Field(field) => {
                let callable = crate::semantic::CallableId {
                    owner: field.owner.clone(),
                    selector: field.name.clone(),
                    side: field.side,
                };
                let member = request.semantic.member_surface(&callable)?;
                let site = SelectorSite {
                    owner: member.callable.owner.clone(),
                    receiver: None,
                    kind: hover_member_kind(member),
                };
                let phaldoc = self.member_phaldoc(member);
                let inferred = request.semantic.field_value(&field.owner, &field.name, field.side);
                let value = hover::render_selector_hover_with_value(&field.name, &[site], phaldoc.as_ref(), inferred.as_ref())?;
                Some(Hover {
                    contents: markdown_contents(value),
                    range: Some(span),
                })
            }
            crate::semantic::SemanticTarget::Member { .. } => {
                let (selector, selector_span) = self.selector_at_document(&request.document, position)?;
                if selector_span != span {
                    return None;
                }
                let targets = self.semantic_member_targets_for_request(request, uri, position, &selector)?;
                if targets.is_empty() {
                    let receiver_targeted = completion::target_at_snapshot(&request.document, position).is_some();
                    if receiver_targeted {
                        return None;
                    }
                    let infos = self.index.definition_info(&selector);
                    let mut sites = Vec::new();
                    let mut docs = Vec::new();
                    for info in infos {
                        let module = request
                            .semantic
                            .module_for_uri(&info.uri)
                            .cloned()
                            .unwrap_or_else(|| crate::semantic::ModuleId::new(info.uri.to_string()));
                        sites.push(SelectorSite {
                            owner: crate::semantic::ClassId::new(module, info.class.clone()),
                            receiver: None,
                            kind: info.kind,
                        });
                        if let Some(doc) = self
                            .with_source_snapshot(&info.uri, |text, program, line_index| {
                                hover::harvest_doc_for_declaration(
                                    text,
                                    line_index,
                                    hover::DeclarationDocTarget::Member {
                                        declaration: info.range,
                                        name: info.range,
                                    },
                                )
                                .or_else(|| hover::harvest_pinned_doc_for_member(text, program, &info.class, &selector, info.range))
                            })
                            .flatten()
                        {
                            if !docs.contains(&doc) {
                                docs.push(doc);
                            }
                        }
                    }
                    let phaldoc = (docs.len() == 1).then(|| docs.remove(0));
                    let value = hover::render_selector_hover_with_value(&selector, &sites, phaldoc.as_ref(), None)?;
                    return Some(Hover {
                        contents: markdown_contents(value),
                        range: Some(span),
                    });
                }
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
                let inferred = request.semantic.returns_for_callables(ids);
                let value = hover::render_selector_hover_with_value(&selector, &sites, phaldoc.as_ref(), inferred.as_ref())?;
                Some(Hover {
                    contents: markdown_contents(value),
                    range: Some(span),
                })
            }
            crate::semantic::SemanticTarget::Operator(_) => None,
        }
    }

    /// Serves hover from live source/index/cache data while semantic analysis
    /// is still pending. This path performs no inference and keeps requests
    /// useful during the publication gap.
    fn legacy_hover_at(&self, uri: &Url, _position: Position, offset: usize) -> Option<Hover> {
        let document = self.documents.snapshot(uri)?;
        let text = document.text;
        let program = document.parse.program.clone();
        let line_index = document.line_index;
        if let Some((selector, selector_range)) = index::selector_at_offset(&program, offset) {
            let infos = self.index.definition_info(&selector);
            let receiver = text[..selector_range.start]
                .trim_end()
                .strip_suffix('.')
                .and_then(|prefix| prefix.rsplit(|character: char| !character.is_ascii_alphanumeric() && character != '_').next());
            let infos = if let Some(receiver) = receiver {
                infos.into_iter().filter(|info| info.class == receiver).collect::<Vec<_>>()
            } else {
                let local = infos
                    .iter()
                    .filter(|info| info.uri == *uri && info.range.contains(offset))
                    .cloned()
                    .collect::<Vec<_>>();
                if local.is_empty() { infos } else { local }
            };
            let mut sites = Vec::new();
            let mut docs = Vec::new();
            let snapshot = self.semantic.snapshot();
            for info in infos {
                let module = snapshot
                    .module_for_uri(&info.uri)
                    .cloned()
                    .unwrap_or_else(|| crate::semantic::ModuleId::new(info.uri.to_string()));
                sites.push(SelectorSite {
                    owner: crate::semantic::ClassId::new(module, info.class.clone()),
                    receiver: None,
                    kind: info.kind,
                });
                if let Some(doc) = self
                    .with_source_snapshot(&info.uri, |source, program, line_index| {
                        hover::harvest_doc_for_selector(source, program, line_index, &selector)
                            .or_else(|| hover::harvest_pinned_doc_for_member(source, program, &info.class, &selector, info.range))
                    })
                    .flatten()
                {
                    if !docs.contains(&doc) {
                        docs.push(doc);
                    }
                }
            }
            if sites.is_empty() {
                let module = snapshot
                    .module_for_uri(uri)
                    .cloned()
                    .unwrap_or_else(|| crate::semantic::ModuleId::new(uri.to_string()));
                let surface = crate::semantic::build_module_surface(module, &program);
                for member in surface
                    .classes
                    .values()
                    .flat_map(|class| class.all_members())
                    .filter(|member| member.callable.selector == selector)
                {
                    sites.push(SelectorSite {
                        owner: member.callable.owner.clone(),
                        receiver: None,
                        kind: hover_member_kind(member),
                    });
                }
                if !sites.is_empty() {
                    if let Some(doc) = hover::harvest_doc_for_selector(&text, &program, &line_index, &selector) {
                        docs.push(doc);
                    }
                }
            }
            if sites.is_empty() {
                if let Some(native) = phalcom_native_surface::find_native_surface_by_selector(&selector) {
                    let kind = match native.kind {
                        phalcom_native_surface::NativeMemberKind::Getter => crate::index::MemberKind::Getter,
                        phalcom_native_surface::NativeMemberKind::Setter => crate::index::MemberKind::Setter,
                        phalcom_native_surface::NativeMemberKind::Method => crate::index::MemberKind::Method,
                    };
                    sites.push(SelectorSite {
                        owner: crate::semantic::ClassId::new(crate::semantic::ModuleId::new(crate::semantic::CORE_MODULE_URI), native.owner().name()),
                        receiver: None,
                        kind,
                    });
                }
            }
            let phaldoc = (docs.len() == 1).then(|| docs.remove(0));
            let value = hover::render_selector_hover_with_value(&selector, &sites, phaldoc.as_ref(), None)?;
            return Some(Hover {
                contents: markdown_contents(value),
                range: Some(line_index.range(selector_range.start..selector_range.end)),
            });
        }

        let (name, range) = hover::identifier_at_offset(&text, offset)?;
        if index::top_level_binding_at_offset(&program, offset).is_some() {
            let phaldoc = hover::harvest_doc_for_selector(&text, &program, &line_index, &name);
            return Some(Hover {
                contents: markdown_contents(format!(
                    "`{name}` — mutable binding{}",
                    phaldoc.map(|doc| format!("\n\n{}", doc.summary)).unwrap_or_default()
                )),
                range: Some(line_index.range(range)),
            });
        }
        if shallow_parameter_at(&program, &name, offset) {
            return Some(Hover {
                contents: markdown_contents(format!("`{name}` — parameter")),
                range: Some(line_index.range(range)),
            });
        }
        if shallow_local_binding_at(&program, &name, offset) {
            return Some(Hover {
                contents: markdown_contents(format!("`{name}` — mutable binding")),
                range: Some(line_index.range(range)),
            });
        }
        None
    }
}

fn shallow_parameter_at(program: &phalcom_ast::ast::Program, name: &str, offset: usize) -> bool {
    program.statements.iter().any(|statement| {
        let phalcom_ast::ast::Statement::Class(class) = statement else { return false };
        class.members.iter().any(|member| match member {
            phalcom_ast::ast::ClassMember::Method(method) => method.range.contains(offset) && method.params.iter().any(|param| param.name == name),
            _ => false,
        })
    })
}

fn shallow_local_binding_at(program: &phalcom_ast::ast::Program, name: &str, offset: usize) -> bool {
    program.statements.iter().any(|statement| {
        let phalcom_ast::ast::Statement::Class(class) = statement else { return false };
        class.members.iter().any(|member| match member {
            phalcom_ast::ast::ClassMember::Method(method) => {
                method.range.contains(offset)
                    && method
                        .body
                        .statements()
                        .unwrap_or_default()
                        .iter()
                        .any(|statement| statement_has_binding(statement, name))
            }
            _ => false,
        })
    })
}

fn statement_has_binding(statement: &phalcom_ast::ast::Statement, name: &str) -> bool {
    match statement {
        phalcom_ast::ast::Statement::Let(binding) => {
            matches!(&binding.pattern, phalcom_ast::ast::Pattern::Name { name: binding_name, .. } if binding_name == name)
        }
        phalcom_ast::ast::Statement::For(for_statement) => for_statement.body.iter().any(|statement| statement_has_binding(statement, name)),
        phalcom_ast::ast::Statement::Expr { expr, .. } | phalcom_ast::ast::Statement::Throw { expr, .. } => expr_has_binding(expr, name),
        phalcom_ast::ast::Statement::Return(_)
        | phalcom_ast::ast::Statement::Class(_)
        | phalcom_ast::ast::Statement::TypeAlias(_)
        | phalcom_ast::ast::Statement::Export(_)
        | phalcom_ast::ast::Statement::Break { .. }
        | phalcom_ast::ast::Statement::Continue { .. } => false,
    }
}

fn expr_has_binding(expr: &phalcom_ast::ast::Expr, name: &str) -> bool {
    match expr {
        phalcom_ast::ast::Expr::Block(block) => block.body.iter().any(|statement| statement_has_binding(statement, name)),
        _ => false,
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
        crate::semantic::MemberKind::Field => crate::index::MemberKind::Field,
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
    /// Schedules progressive workspace discovery for every root named in
    /// `params`; discovery continues on the analysis worker after this
    /// response is returned.
    ///
    /// Also advertises Stage 5's `semanticTokensProvider` (full-document
    /// only, no `range`/`delta` support yet), with the legend built by
    /// [`semantic_tokens::legend`].
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let _span = PerfSpan::start_with_counters("initialize", self.perf_counters());
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
        self.schedule_workspace_scan(&roots);

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

    /// Logs that the server is ready and starts consuming worker events.
    async fn initialized(&self, _params: InitializedParams) {
        let _span = PerfSpan::start_with_counters("initialized", self.perf_counters());
        if let Some(mut events) = self.analysis_events.lock().expect("analysis events lock poisoned").take() {
            let client = self.client.clone();
            let indexed_files = self.indexed_files.clone();
            let closed_sources = self.closed_sources.clone();
            let core_source_uris = self.core_source_uris.clone();
            let inlay_refresh = self.inlay_refresh.clone();
            let semantic_token_refresh = self.semantic_token_refresh.clone();
            let counters = self.perf_counters();
            let documents = self.documents.clone();
            let semantic = self.semantic.clone();
            tokio::spawn(async move {
                while let Some(event) = events.recv().await {
                    match event {
                        AnalysisEvent::CoreSourceSelected { uri } => {
                            let mut core_source_uris = core_source_uris.write().expect("core source URI lock poisoned");
                            core_source_uris.clear();
                            if let Some(uri) = uri {
                                core_source_uris.insert(uri);
                            }
                        }
                        AnalysisEvent::WorkspaceFileIndexed { uri, text: _, revision } => {
                            let cached_uri = uri.clone();
                            let canonical_uri = cached_uri
                                .to_file_path()
                                .ok()
                                .and_then(|path| path.canonicalize().ok())
                                .and_then(|path| Url::from_file_path(path).ok());
                            let source = {
                                let cache = closed_sources.read().expect("closed source cache lock poisoned");
                                cache
                                    .get(&cached_uri)
                                    .cloned()
                                    .or_else(|| canonical_uri.as_ref().and_then(|uri| cache.get(uri).cloned()))
                            };
                            let Some(source) = source else { continue };
                            indexed_files.write().expect("indexed file lock poisoned").insert(uri);
                            let mut cache = closed_sources.write().expect("closed source cache lock poisoned");
                            cache.insert(cached_uri, CachedSource { revision, ..source.clone() });
                            if let Some(canonical_uri) = canonical_uri {
                                cache.insert(canonical_uri, CachedSource { revision, ..source });
                            }
                        }
                        AnalysisEvent::WorkspaceFileRemoved { uri } => {
                            indexed_files.write().expect("indexed file lock poisoned").remove(&uri);
                            let canonical_uri = uri
                                .to_file_path()
                                .ok()
                                .and_then(|path| path.canonicalize().ok())
                                .and_then(|path| Url::from_file_path(path).ok());
                            let mut cache = closed_sources.write().expect("closed source cache lock poisoned");
                            cache.remove(&uri);
                            if let Some(canonical_uri) = canonical_uri {
                                cache.remove(&canonical_uri);
                            }
                        }
                        AnalysisEvent::Published { effects, .. } => {
                            for uri in documents.open_uris() {
                                let Some(publication) = combined_diagnostics_for(&documents, &semantic, &uri) else {
                                    continue;
                                };
                                client.publish_diagnostics(uri, publication.diagnostics, publication.version).await;
                            }
                            if effects.inlay_hints_changed && inlay_refresh.request() {
                                let client = client.clone();
                                let refresh = inlay_refresh.clone();
                                counters.inlay_refresh_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                tokio::spawn(async move {
                                    loop {
                                        let _ = client.inlay_hint_refresh().await;
                                        if !refresh.finished_refresh() {
                                            break;
                                        }
                                    }
                                });
                            }
                            if effects.semantic_tokens_changed && semantic_token_refresh.request() {
                                let client = client.clone();
                                let refresh = semantic_token_refresh.clone();
                                counters.semantic_token_refresh_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                tokio::spawn(async move {
                                    loop {
                                        let _ = client.semantic_tokens_refresh().await;
                                        if !refresh.finished_refresh() {
                                            break;
                                        }
                                    }
                                });
                            }
                        }
                        AnalysisEvent::Status(status) => {
                            client.send_notification::<AnalysisStatusNotification>(status).await;
                        }
                        AnalysisEvent::Error { message } => {
                            let status = AnalysisStatus {
                                session: 0,
                                sequence: 0,
                                phase: AnalysisPhase::Error,
                                step: None,
                                mode: crate::workspace_scan::AnalysisMode::Local,
                                current_uri: None,
                                discovered_files: 0,
                                indexed_files: 0,
                                analyzed_files: 0,
                                generation: None,
                                complete: false,
                                message: Some(message),
                            };
                            client.send_notification::<AnalysisStatusNotification>(status).await;
                        }
                        AnalysisEvent::StaleBatchDiscarded { .. } => {}
                    }
                }
            });
        }
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

    /// Applies changed settings and refreshes the configured core source if analysis configuration changed.
    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let new_config = ServerConfig::from_json(Some(&params.settings));
        let old_analysis_config = self.config.read().expect("server config lock poisoned").analysis_config();
        let new_analysis_config = new_config.analysis_config();
        *self.config.write().expect("server config lock poisoned") = new_config;

        if old_analysis_config != new_analysis_config {
            let roots = self.workspace_roots.read().expect("workspace root lock poisoned").clone();
            self.schedule_workspace_scan(&roots);
        }
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
            self.analysis.enqueue_core_update(crate::semantic::FileRevision(1), bundled.program);
        }
        self.schedule_workspace_scan(&roots);
    }

    /// Refreshes closed-file contributions for watched `.ph` changes.
    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        let mut removals = Vec::new();
        let mut refreshes = Vec::new();
        for change in params.changes {
            if change.typ == FileChangeType::DELETED {
                self.index.remove_file(&change.uri);
                self.closed_sources.write().expect("closed source cache lock poisoned").remove(&change.uri);
                self.indexed_files.write().expect("indexed file lock poisoned").remove(&change.uri);
                if self.is_core_source_uri(&change.uri) {
                    let bundled = crate::semantic::core_source::bundled_parse();
                    self.analysis.enqueue_core_update(crate::semantic::FileRevision(1), bundled.program);
                    self.clear_core_source_uris();
                } else {
                    removals.push(change.uri.clone());
                }
            } else {
                refreshes.push(DiskRefresh { uri: change.uri });
            }
        }
        self.analysis.enqueue_file_mutations(removals, refreshes);
    }

    /// Reports readiness to shut down.
    async fn shutdown(&self) -> Result<()> {
        self.analysis.shutdown();
        Ok(())
    }

    /// Parses the newly-opened document, publishes its diagnostics, and
    /// refreshes its slice of the workspace index (via
    /// `Self::publish_diagnostics_for`).
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let _span = PerfSpan::start_with_counters("did_open_source_parse", self.perf_counters());
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        self.analysis.mark_open(uri.clone());
        self.closed_sources.write().expect("closed source cache lock poisoned").remove(&uri);
        self.documents.open_or_update_versioned(uri.clone(), params.text_document.text, Some(version));
        if let Some((revision, text, program)) = self.documents.with_document(&uri, |doc| {
            let recovered = semantic_recovery_parse(doc.text.as_ref(), &doc.parse);
            let program = recovered.map(|p| p.program).unwrap_or_else(|| doc.parse.program.clone());
            self.cache_source(uri.clone(), doc.revision, doc.text.clone(), Arc::new(program.clone()));
            (doc.revision, doc.text.clone(), program)
        }) {
            self.update_semantic_for_source(&uri, revision, text, &program);
        }
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
        let _span = PerfSpan::start_with_counters("did_change_source_parse", self.perf_counters());
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        let Some(change) = params.content_changes.into_iter().next_back() else {
            return;
        };
        self.documents.open_or_update_versioned(uri.clone(), change.text, Some(version));
        self.analysis.mark_open(uri.clone());
        if let Some((revision, text, program)) = self.documents.with_document(&uri, |doc| {
            let recovered = semantic_recovery_parse(doc.text.as_ref(), &doc.parse);
            let program = recovered.map(|p| p.program).unwrap_or_else(|| doc.parse.program.clone());
            self.cache_source(uri.clone(), doc.revision, doc.text.clone(), Arc::new(program.clone()));
            (doc.revision, doc.text.clone(), program)
        }) {
            self.update_semantic_for_source(&uri, revision, text, &program);
        }
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
        self.analysis.mark_closed(&uri);
        let _revision = self.documents.bump_revision(&uri);
        if uri.to_file_path().is_ok() {
            self.analysis.enqueue_disk_refresh(uri.clone());
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
        let Some(request) = self.request_context(&uri) else { return Ok(None) };

        let offset = request.document.line_index.offset(position);
        if let Some(occurrence) = request.exact_file().and_then(|_| request.semantic.occurrence_at(&uri, offset)) {
            match &occurrence.target {
                SemanticTarget::Binding(binding_id) => {
                    if let Some(info) = request.semantic.binding_info(&uri, *binding_id) {
                        let decl_uri = uri.clone();
                        let range = request.document.line_index.range(info.declaration_range.start..info.declaration_range.end);
                        return Ok(Some(GotoDefinitionResponse::Array(vec![Location { uri: decl_uri, range }])));
                    }
                }
                SemanticTarget::Class(class_id) => {
                    if let Some(class) = request.semantic.class_surface(class_id) {
                        if let Some(loc) = self.class_definition_location(class) {
                            return Ok(Some(GotoDefinitionResponse::Array(vec![loc])));
                        }
                    }
                }
                SemanticTarget::Callable(callable_id) => {
                    if let Some(member) = request.semantic.member_surface(callable_id) {
                        if let Some(loc) = self.member_definition_location(member) {
                            return Ok(Some(GotoDefinitionResponse::Array(vec![loc])));
                        }
                    }
                }
                SemanticTarget::Field(field) => {
                    let callable = crate::semantic::CallableId {
                        owner: field.owner.clone(),
                        selector: field.name.clone(),
                        side: field.side,
                    };
                    if let Some(member) = request.semantic.member_surface(&callable) {
                        if let Some(loc) = self.member_definition_location(member) {
                            return Ok(Some(GotoDefinitionResponse::Array(vec![loc])));
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some((class, _)) = self.semantic_class_target(&request, &uri, position) {
            return Ok(self
                .class_definition_location(&class)
                .map(|location| GotoDefinitionResponse::Array(vec![location])));
        }

        let Some((selector, _range)) = self.selector_at_document(&request.document, position) else {
            return Ok(None);
        };

        if let Some(member) = request
            .exact_file()
            .and_then(|_| request.semantic.member_at(&uri, offset))
            .filter(|member| member.callable.selector == selector)
        {
            return Ok(self
                .member_definition_location(&member)
                .map(|location| GotoDefinitionResponse::Array(vec![location])));
        }

        let receiver_targeted = completion::target_at_snapshot(&request.document, position).is_some();
        if receiver_targeted {
            let semantic_locations = self.semantic_definition_locations_for_request(&request, &uri, position, &selector);
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
        let Some(request) = self.request_context(&uri) else { return Ok(None) };

        let offset = request.document.line_index.offset(position);
        if let Some(occurrence) = request.exact_file().and_then(|_| request.semantic.occurrence_at(&uri, offset)) {
            let refs = request.semantic.references_for_target(&uri, &occurrence.target);
            let locations: Vec<Location> = refs
                .into_iter()
                .filter_map(|(file_uri, range, role)| {
                    if !params.context.include_declaration && role == OccurrenceRole::Declaration {
                        return None;
                    }
                    self.with_source_snapshot(&file_uri, |_, _, line_index| line_index.range(range.start..range.end))
                        .map(|range| Location { uri: file_uri, range })
                })
                .collect();
            return Ok(if locations.is_empty() { None } else { Some(locations) });
        }

        let Some((selector, _range)) = self.selector_at_document(&request.document, position) else {
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
        let _span = PerfSpan::start_with_counters("completion", self.perf_counters());
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let Some(request) = self.request_context(&uri) else { return Ok(None) };
        let recovered = semantic_recovery_parse(request.document.text.as_ref(), &request.document.parse);
        let source_program = recovered.as_ref().map(|parse| &parse.program).unwrap_or(&request.document.parse.program);
        let resolved = self.semantic_receiver(&request.semantic, &uri, &request.document, position);
        let offset = request.document.line_index.offset(position);
        let privileged = self.is_core_source_uri(&uri);
        let lexical_class = request.semantic.class_at(&uri, offset);
        let mut items = completion::semantic_contextual_completions(
            &request.semantic,
            completion::SemanticCompletionContext {
                resolved: resolved.as_ref(),
                lexical_class: lexical_class.as_ref(),
                privileged,
                uri: &uri,
                program: &request.document.parse.program,
                text: &request.document.text,
                offset,
            },
        );
        if resolved.is_none()
            && let Some(shallow) = completion::shallow_receiver_completions_from_snapshot(&self.index, &uri, &request.document, source_program, position)
        {
            items = shallow;
        }
        if let Some(target) = completion::target_at_snapshot(&request.document, position)
            && !target.partial_member.is_empty()
        {
            items.retain(|item| item.label.starts_with(&target.partial_member));
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    /// Answers `textDocument/hover` (Stage 4). See `Self::hover_at` for the
    /// resolution/composition logic; this is a thin `async` shim over it.
    /// Cross-file source metadata comes from the worker-maintained cache, so
    /// this request never performs disk I/O or waits for semantic analysis.
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let _span = PerfSpan::start_with_counters("hover", self.perf_counters());
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        Ok(self.hover_at(&uri, position))
    }

    /// Answers standard inlay-hint requests from the live semantic database.
    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let _span = PerfSpan::start_with_counters("inlay", self.perf_counters());
        let config = self.config.read().expect("server config lock poisoned").clone();
        let uri = params.text_document.uri.clone();
        let Some(request) = self.request_context(&uri) else { return Ok(None) };
        Ok(Some(crate::inlay_hints::hints_for_request(
            &request,
            params.range,
            config.inlay_hints,
            config.suppress_obvious,
        )))
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
        let Some(request) = self.request_context(&uri) else { return Ok(None) };
        let data = semantic_tokens::tokens_for_request(&request);
        Ok(Some(SemanticTokensResult::Tokens(tower_lsp::lsp_types::SemanticTokens {
            result_id: None,
            data,
        })))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::fs;
    use std::sync::Arc;
    use tower_lsp::lsp_types::{DidChangeWatchedFilesParams, FileChangeType, FileEvent, Url};
    use tower_lsp::{LanguageServer, LspService};

    use super::super::analysis_service::{AnalysisEvent, TestBatchGate, TestScanGate};
    use super::{HintPolicy, PublicationRefresh, ServerConfig};
    use crate::semantic::{FileRevision, SemanticGeneration};
    use crate::workspace_scan::AnalysisMode;

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
            "phalcom.lsp.sysrootPath": "/opt/phalcom/phalcom-core/core/universe/src/package.ph",
            "phalcom.inlayHints.types": "off"
        });

        let config = ServerConfig::from_json(Some(&settings));

        assert_eq!(
            config.sysroot_path.as_deref(),
            Some(std::path::Path::new("/opt/phalcom/phalcom-core/core/universe/src/package.ph"))
        );
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

    #[test]
    fn server_config_defaults_to_local_analysis() {
        let config = ServerConfig::from_json(None);

        assert_eq!(config.analysis_mode, AnalysisMode::Local);
        assert!(config.analysis_exclude.is_empty());
    }

    #[test]
    fn server_config_parses_analysis_mode_and_exclusions() {
        let settings = serde_json::json!({
            "phalcom": {
                "analysis": {
                    "mode": "workspace",
                    "exclude": ["**/generated/**", "fixtures"]
                }
            }
        });

        let config = ServerConfig::from_json(Some(&settings));

        assert_eq!(config.analysis_mode, AnalysisMode::Workspace);
        assert_eq!(config.analysis_exclude, ["**/generated/**", "fixtures"]);
    }

    #[test]
    fn analysis_config_diffing_ignores_presentation_settings() {
        let base = json!({
            "phalcom": {
                "analysis": { "mode": "local" },
                "inlayHints": { "types": "stable" }
            }
        });
        let changed_presentation = json!({
            "phalcom": {
                "analysis": { "mode": "local" },
                "inlayHints": { "types": "all" }
            }
        });
        let changed_analysis = json!({
            "phalcom": {
                "analysis": { "mode": "workspace" },
                "inlayHints": { "types": "stable" }
            }
        });

        let base_cfg = ServerConfig::from_json(Some(&base)).analysis_config();
        let presentation_cfg = ServerConfig::from_json(Some(&changed_presentation)).analysis_config();
        let analysis_cfg = ServerConfig::from_json(Some(&changed_analysis)).analysis_config();

        assert_eq!(base_cfg, presentation_cfg, "presentation changes should not affect analysis config");
        assert_ne!(base_cfg, analysis_cfg, "analysis mode changes should affect analysis config");
    }

    #[test]
    fn publication_refresh_coalesces_compatible_events_and_resets() {
        let refresh = PublicationRefresh::default();

        assert!(refresh.request());
        for _ in 0..100 {
            assert!(!refresh.request());
        }
        assert!(refresh.finished_refresh(), "publication during refresh must schedule another pass");
        assert!(!refresh.finished_refresh(), "refresh state must reset after the final pass");
        assert!(refresh.request(), "a later publication starts a fresh refresh");
        assert!(!refresh.finished_refresh());
    }

    #[test]
    fn publication_refresh_requests_during_completion_are_not_lost() {
        let refresh = Arc::new(PublicationRefresh::default());
        assert!(refresh.request());

        let during_completion = refresh.clone();
        let thread = std::thread::spawn(move || {
            assert!(!during_completion.request());
        });
        thread.join().expect("refresh request thread must complete");

        assert!(refresh.finished_refresh());
        assert!(!refresh.finished_refresh());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn construction_initialize_hover_and_inlay_do_not_wait_for_semantic_batch() {
        let (service, _socket) = LspService::new(super::Backend::new);
        let backend = service.inner();
        assert_eq!(backend.semantic.generation(), SemanticGeneration(0), "construction must not analyze core");
        assert_eq!(backend.perf_counters().snapshot().semantic_batches_started, 0);

        let gate = Arc::new(TestBatchGate::default());
        backend.analysis.install_test_batch_gate(gate.clone());
        let uri = Url::parse("file:///blocked-request.ph").unwrap();
        let source = "let value = 1\n";
        let parsed = phalcom_ast::parser::parse(source, 0);
        backend.documents.open_or_update(uri.clone(), source.to_string());
        backend.analysis.mark_open(uri.clone());
        backend.analysis.enqueue_file_update(uri.clone(), FileRevision(1), parsed.program);
        gate.wait_until_before_entered();

        let initialize: tower_lsp::lsp_types::InitializeParams = serde_json::from_value(json!({
            "processId": null,
            "rootUri": null,
            "capabilities": {}
        }))
        .expect("initialize params");
        let response = backend.initialize(initialize).await.expect("initialize response");
        assert!(response.capabilities.inlay_hint_provider.is_some());
        assert_eq!(
            backend.perf_counters().snapshot().semantic_batches_started,
            0,
            "worker remains blocked before batch start"
        );

        let hover: tower_lsp::lsp_types::HoverParams = serde_json::from_value(json!({
            "textDocument": { "uri": uri },
            "position": { "line": 0, "character": 4 }
        }))
        .expect("hover params");
        assert!(backend.hover(hover).await.is_ok());

        let inlay: tower_lsp::lsp_types::InlayHintParams = serde_json::from_value(json!({
            "textDocument": { "uri": uri },
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 1, "character": 0 }
            }
        }))
        .expect("inlay params");
        assert!(backend.inlay_hint(inlay).await.is_ok());

        gate.release_before();
        drop(service);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn initialize_returns_before_recursive_scan_or_deep_solve() {
        let root = std::env::temp_dir().join(format!("phalcom-lsp-initialize-scan-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create initialize test root");
        fs::write(root.join("closed.ph"), "class Closed { marker() {} }\n").expect("write closed source");

        let (service, _socket) = LspService::new(super::Backend::new);
        let backend = service.inner();
        let gate = Arc::new(TestScanGate::default());
        backend.analysis.install_test_scan_gate(gate.clone());
        let root_uri = Url::from_directory_path(&root).expect("root URI");
        let params: tower_lsp::lsp_types::InitializeParams = serde_json::from_value(json!({
            "processId": null,
            "rootUri": root_uri,
            "workspaceFolders": [{ "uri": root_uri, "name": "scan-test" }],
            "capabilities": {}
        }))
        .expect("initialize params");

        let response = backend.initialize(params).await.expect("initialize response");
        assert!(response.capabilities.workspace_symbol_provider.is_some());
        gate.wait_until_entered();
        let counters = backend.perf_counters().snapshot();
        assert_eq!(counters.semantic_batches_started, 0, "initialize must not wait for deep solving");
        assert_eq!(counters.workspace_files_discovered, 0, "recursive scan must remain on worker after response");

        gate.release();
        backend.analysis.flush();
        drop(service);
        let _ = fs::remove_dir_all(root);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn watched_file_batch_publishes_one_semantic_transaction() {
        let root = std::env::temp_dir().join(format!("phalcom-lsp-watched-batch-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create watched-file test root");
        let first = root.join("first.ph");
        let second = root.join("second.ph");
        fs::write(&first, "class First {}\n").expect("write first watched file");
        fs::write(&second, "class Second {}\n").expect("write second watched file");

        let (service, _socket) = LspService::new(super::Backend::new);
        let backend = service.inner();
        let mut events = backend
            .analysis_events
            .lock()
            .expect("analysis events lock poisoned")
            .take()
            .expect("event receiver");
        backend
            .did_change_watched_files(DidChangeWatchedFilesParams {
                changes: vec![
                    FileEvent {
                        uri: Url::from_file_path(&first).unwrap(),
                        typ: FileChangeType::CHANGED,
                    },
                    FileEvent {
                        uri: Url::from_file_path(&second).unwrap(),
                        typ: FileChangeType::CHANGED,
                    },
                ],
            })
            .await;

        backend.analysis.flush();
        let publications = std::iter::from_fn(|| events.try_recv().ok())
            .filter(|event| matches!(event, AnalysisEvent::Published { .. }))
            .count();
        assert_eq!(publications, 1, "watched-file batch must publish one transaction");
        assert_eq!(backend.perf_counters().snapshot().semantic_batches_published, 1);

        drop(service);
        let _ = fs::remove_dir_all(root);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn presentation_configuration_does_not_rebuild_core() {
        let (service, _socket) = LspService::new(super::Backend::new);
        let backend = service.inner();
        backend
            .did_change_configuration(tower_lsp::lsp_types::DidChangeConfigurationParams {
                settings: json!({ "phalcom": { "inlayHints": { "types": "all" } } }),
            })
            .await;
        backend.analysis.flush();
        let snapshot = backend.perf_counters().snapshot();
        assert_eq!(snapshot.semantic_batches_started, 0, "presentation-only configuration must not rebuild core");
        assert_eq!(backend.semantic.generation(), SemanticGeneration(0));
        drop(service);
    }
}
