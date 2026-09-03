//! The `tower_lsp::LanguageServer` implementation.
//!
//! Stage 1 (ADR-0056 §3, §6): `initialize`/`initialized`/`shutdown` for the
//! server lifecycle, and `did_open`/`did_change`/`did_close` to maintain the
//! [`DocumentStore`] and publish live, multi-error diagnostics.
//!
//! Stage 2 (ADR-0056 §4, `docs/forge/units/U-LSP/plan.md` "Stage 2"): a
//! compiler-owned source indexes for exact definition, reference, and symbol
//! requests, with syntax-only behavior for stale or unmapped buffers.
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

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::analysis_service::{AnalysisEvent, AnalysisService, CachedSource, DiskRefresh, SourceCache, WorkspaceScanRequest, builtin_module_from_uri};
use crate::analysis_status::AnalysisStatusNotification;

use serde::Deserialize;
use serde_json::Value as JsonValue;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionOptions, CompletionParams, CompletionResponse, DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidChangeWorkspaceFoldersParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, FileChangeType, GotoDefinitionParams, GotoDefinitionResponse,
    Hover, HoverContents, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, InitializedParams, InlayHint, InlayHintOptions,
    InlayHintParams, InlayHintServerCapabilities, Location, MarkupContent, MarkupKind, MessageType, OneOf, Position, PositionEncodingKind, ReferenceParams,
    Registration, SemanticTokensFullOptions, SemanticTokensOptions, SemanticTokensParams, SemanticTokensResult, SemanticTokensServerCapabilities,
    ServerCapabilities, SignatureHelp, SignatureHelpOptions, SignatureHelpParams, SymbolInformation, SymbolKind, TextDocumentSyncCapability,
    TextDocumentSyncKind, Url, WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities, WorkspaceSymbolParams,
};
use tower_lsp::{Client, LanguageServer};

use crate::completion;
use crate::diagnostics::{SemanticDiagnosticSource, semantic_diagnostics_to_lsp_diagnostics_with_snapshot, syntax_errors_to_diagnostics};
use crate::documents::DocumentStore;
use crate::hover::{self, SelectorSite};
use crate::inlay_hints::HintPolicy;
use crate::line_index::LineIndex;
use crate::perf::{PerfCountersHandle, PerfSpan};
use crate::request_context::{RequestContext, SourceMatch};
use crate::semantic_tokens;
use crate::signature_help;
use phalcom_semantic::FormalPresentation;
use phalcom_semantic::types::relation::TypeHierarchy;

use crate::workspace_scan::AnalysisMode;

type CompilerCallableHover = (
    String,
    SelectorSite,
    Option<hover::PhaldocDoc>,
    FormalPresentation,
    Option<phalcom_semantic::advisory::AdvisoryFact>,
    Option<phalcom_semantic::NativeCallablePresentation>,
);

fn import_path_range_at_offset(program: &phalcom_ast::ast::Program, offset: usize) -> Option<phalcom_common::range::SourceRange> {
    program.preamble.dependencies.iter().find_map(|dependency| {
        let path = match dependency {
            phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Module(import)) => &import.path,
            phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Selective(import)) => &import.path,
            _ => return None,
        };
        path.range.contains(offset).then_some(path.range)
    })
}

fn compiler_import_definition_location(request: &RequestContext, position: Position) -> Option<Location> {
    let compiler = request.compiler.as_deref()?;
    let importer = request.compiler_module()?;
    let offset = request.document.line_index.offset(position);

    for dependency in &request.document.parse.program.preamble.dependencies {
        let (path, range, binding_names) = match dependency {
            phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Module(import)) => {
                let mut names = Vec::new();
                if let Some(alias) = &import.alias {
                    names.push(alias.name.clone());
                }
                if let Some(segment) = import.path.segments.last() {
                    names.push(segment.name.clone());
                }
                if let phalcom_ast::ast::ImportRoot::Absolute(segment) = &import.path.root {
                    names.push(segment.name.clone());
                }
                (&import.path, import.path.range, names)
            }
            phalcom_ast::ast::DependencyDecl::Import(phalcom_ast::ast::ImportDecl::Selective(import)) => {
                let names = import
                    .items
                    .iter()
                    .flat_map(|item| [Some(item.name.clone()), item.alias.as_ref().map(|alias| alias.name.clone())])
                    .flatten()
                    .collect();
                (&import.path, import.path.range, names)
            }
            _ => continue,
        };
        if !range.contains(offset) {
            continue;
        }

        let queries = compiler.module_queries();
        let target = binding_names
            .iter()
            .find_map(|name| queries.resolved_import_target(importer, name))
            .or_else(|| queries.resolved_import_target(importer, &path.to_string()))?;
        let source = queries.definition_source(target)?;
        let uri = Url::from_file_path(&source.display_path)
            .ok()
            .or_else(|| Url::parse(source.source_id.0.as_ref()).ok())?;
        return Some(Location {
            uri,
            range: tower_lsp::lsp_types::Range::default(),
        });
    }

    None
}

struct DiagnosticPublication {
    diagnostics: Vec<tower_lsp::lsp_types::Diagnostic>,
    version: Option<i32>,
}

fn combined_diagnostics_for(
    documents: &DocumentStore,
    compiler_snapshot: Option<&phalcom_semantic::SemanticSnapshot>,
    uri: &Url,
) -> Option<DiagnosticPublication> {
    let document = documents.snapshot(uri)?;
    let mut diagnostics = syntax_errors_to_diagnostics(&document.parse.errors, &document.line_index);
    let syntax_only = |diagnostics| DiagnosticPublication {
        diagnostics,
        version: document.version,
    };
    let Some(compiler_snapshot) = compiler_snapshot else {
        return Some(syntax_only(diagnostics));
    };
    let Some(module) = compiler_module_for_uri(compiler_snapshot, uri) else {
        return Some(syntax_only(diagnostics));
    };
    let Some(static_source) = compiler_snapshot.sources.get(module) else {
        return Some(syntax_only(diagnostics));
    };
    if static_source.text.as_ref() != document.text.as_ref() {
        return Some(syntax_only(diagnostics));
    }
    if let Some(semantic_diagnostics) = compiler_snapshot.diagnostics.get(module) {
        let mut diagnostic_sources = BTreeMap::new();
        for (module, source) in compiler_snapshot.sources.iter() {
            if let Some(source_uri) = compiler_uri_for_module(compiler_snapshot, module) {
                diagnostic_sources.insert(
                    module.clone(),
                    SemanticDiagnosticSource {
                        uri: source_uri,
                        line_index: LineIndex::new(&source.text),
                    },
                );
            }
        }
        diagnostic_sources.insert(
            module.clone(),
            SemanticDiagnosticSource {
                uri: uri.clone(),
                line_index: (*document.line_index).clone(),
            },
        );
        diagnostics.extend(semantic_diagnostics_to_lsp_diagnostics_with_snapshot(
            semantic_diagnostics,
            compiler_snapshot,
            &document.line_index,
            uri,
            &diagnostic_sources,
        ));
    }
    Some(DiagnosticPublication {
        diagnostics,
        version: document.version,
    })
}

fn compiler_module_for_uri<'a>(compiler: &'a phalcom_semantic::SemanticSnapshot, uri: &Url) -> Option<&'a phalcom_modules::ModuleId> {
    if let Ok(path) = uri.to_file_path() {
        return compiler.module_for_display_path(&path);
    }
    let module = crate::analysis_service::builtin_module_from_uri(uri)?;
    compiler
        .sources
        .keys()
        .find(|candidate| **candidate == module)
        .or_else(|| compiler.presentation_sources.keys().find(|candidate| **candidate == module))
}

fn compiler_uri_for_module(compiler: &phalcom_semantic::SemanticSnapshot, module: &phalcom_modules::ModuleId) -> Option<Url> {
    if let Some(raw) = phalcom_modules::universe_module_uri(module) {
        return Url::parse(&raw).ok();
    }
    let source = compiler.sources.get(module)?.source.as_ref()?;
    Url::from_file_path(&source.display_path)
        .ok()
        .or_else(|| Url::parse(source.source_id.0.as_ref()).ok())
        .or_else(|| phalcom_modules::universe_module_uri(module).and_then(|raw| Url::parse(&raw).ok()))
}

/// Parameters for the read-only virtual source provider request.
#[derive(Clone, Debug, Deserialize)]
pub struct SourceTextParams {
    /// Virtual or physical source URI requested by the editor.
    pub uri: Url,
}

/// Runtime configuration that affects semantic source discovery and hint UI.
#[derive(Clone, Debug)]
pub struct ServerConfig {
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
    /// Analysis scope mode.
    pub mode: AnalysisMode,
    /// Workspace path fragments or file patterns excluded from indexing.
    pub excludes: Vec<String>,
}

impl ServerConfig {
    /// Extracts the semantic analysis configuration subset.
    pub fn analysis_config(&self) -> AnalysisConfig {
        AnalysisConfig {
            mode: self.analysis_mode,
            excludes: self.analysis_exclude.clone(),
        }
    }
}

/// The Phalcom language server.
///
/// Holds the `tower-lsp` [`Client`] handle, open documents, and the
/// compiler-owned analysis service.
pub struct Backend {
    /// Handle back to the LSP client, used to send notifications
    /// (`textDocument/publishDiagnostics`, `window/logMessage`, …).
    client: Client,
    /// The open-document store: text + cached parse + cached [`LineIndex`]
    /// per open file.
    documents: DocumentStore,
    /// Background semantic analysis service.
    analysis: AnalysisService,
    /// Receiver for worker analysis events (taken in `initialized`).
    analysis_events: Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<AnalysisEvent>>>,
    /// Current workspace roots advertised by the client.
    workspace_roots: RwLock<Vec<Url>>,
    /// Closed-file text, parse, and line metadata populated by worker/index events.
    closed_sources: SourceCache,
    /// Mutable server configuration.
    config: RwLock<ServerConfig>,
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

impl Backend {
    /// Creates a new [`Backend`] bound to `client`, with an empty document
    /// store and an empty workspace index.
    pub fn new(client: Client) -> Self {
        let _span = PerfSpan::start_with_counters("backend_construction", Arc::new(crate::perf::PerfCounters::new()));
        let closed_sources = Arc::new(RwLock::new(BTreeMap::new()));
        let (analysis, event_rx) = AnalysisService::new_with_source_cache(Some(closed_sources.clone()));
        Self {
            client,
            documents: DocumentStore::new(),
            analysis,
            analysis_events: Mutex::new(Some(event_rx)),
            workspace_roots: RwLock::new(Vec::new()),
            closed_sources,
            config: RwLock::new(ServerConfig::default()),
            watch_registration: RwLock::new(false),
            inlay_refresh: Arc::new(PublicationRefresh::default()),
            semantic_token_refresh: Arc::new(PublicationRefresh::default()),
        }
    }

    /// Returns this backend's compact performance counters for diagnostics and
    /// benchmark harnesses. The counters are owned by this backend's worker.
    pub fn perf_counters(&self) -> PerfCountersHandle {
        self.analysis.perf_counters()
    }

    /// Returns a read-only publication-coherence handle for integration
    /// scheduling. It cannot perform semantic queries or mutate compiler state.
    pub fn semantic_publication_handle(&self) -> crate::publication::SemanticPublicationHandle {
        self.analysis.semantic_publication_handle()
    }

    /// Serves canonical Universe source text to an editor content
    /// provider without mutating or refreshing semantic state.
    pub async fn source_text(&self, params: SourceTextParams) -> Result<Option<String>> {
        if let Some(text) = virtual_source_text(&params.uri) {
            return Ok(Some(text));
        }

        if let Some(document) = self.documents.snapshot(&params.uri) {
            return Ok(Some(document.text.to_string()));
        }
        Ok(self.cached_source(&params.uri).map(|source| source.text.to_string()))
    }

    /// Pins open-document data and one published semantic generation before
    /// any request-local work begins. The document map guard is released by
    /// `DocumentStore::snapshot` before this returns.
    fn request_context(&self, uri: &Url) -> Option<RequestContext> {
        let document = self.documents.snapshot(uri)?;
        Some(RequestContext::new(document, self.analysis.snapshot(), uri))
    }

    /// Reparses the document at `uri` (already updated in the store by the
    /// caller), refreshes its compiler-owned source slice, and publishes
    /// its current [`SyntaxError`](phalcom_ast::error::SyntaxError)s as LSP
    /// diagnostics.
    ///
    /// Publishes unconditionally, including an empty list, so a
    /// previously-errored document that becomes clean has its squiggles
    /// cleared.
    async fn publish_diagnostics_for(&self, uri: Url, version: Option<i32>) {
        let compiler = self.analysis.snapshot();
        let diagnostics = combined_diagnostics_for(&self.documents, compiler.as_deref(), &uri)
            .map(|publication| publication.diagnostics)
            .unwrap_or_default();
        self.client.publish_diagnostics(uri, diagnostics, version).await;
    }

    fn update_semantic_for_source(&self, uri: &Url, revision: phalcom_modules::SourceRevision, text: Arc<str>, program: &phalcom_ast::ast::Program) {
        if crate::source_transport::source_location_for_uri(uri).is_none() {
            return;
        }
        self.analysis.enqueue_file_update(uri.clone(), revision, text, Arc::new(program.clone()));
    }

    fn cache_source(
        &self,
        uri: Url,
        _revision: phalcom_modules::SourceRevision,
        text: impl Into<Arc<str>>,
        program: impl Into<Arc<phalcom_ast::ast::Program>>,
    ) {
        let text = text.into();
        let program = program.into();
        let source = CachedSource {
            line_index: Arc::new(LineIndex::new(&text)),
            text,
            program,
        };
        let mut cache = self.closed_sources.write().expect("closed source cache lock poisoned");
        cache.insert(uri, source);
    }

    fn remove_indexed_file(&self, uri: &Url) {
        self.analysis.enqueue_file_removal(uri.clone());
        self.closed_sources.write().expect("closed source cache lock poisoned").remove(uri);
    }

    fn schedule_workspace_scan(&self, roots: &[Url]) {
        let config = self.config.read().expect("server config lock poisoned").clone();
        let filesystem_roots = roots.iter().filter_map(|root| root.to_file_path().ok()).collect();
        self.analysis.configure_workspace(WorkspaceScanRequest {
            roots: filesystem_roots,
            mode: config.analysis_mode,
            excludes: config.analysis_exclude,
        });
    }

    fn remove_workspace_root(&self, root: &Url) {
        let Ok(root_path) = root.to_file_path() else { return };
        let files = self
            .closed_sources
            .read()
            .expect("closed source cache lock poisoned")
            .keys()
            .filter(|uri| uri.to_file_path().ok().is_some_and(|path| path.starts_with(&root_path)))
            .cloned()
            .collect::<Vec<_>>();
        for uri in &files {
            self.closed_sources.write().expect("closed source cache lock poisoned").remove(uri);
        }
        self.analysis.enqueue_file_removals(files);
    }

    /// Resolves a completion receiver through the pinned compiler snapshot.
    ///
    /// Formal expression/binding knowledge is authoritative. Advisory runtime
    /// shapes are consulted only when the formal product has no concrete class
    /// denotation. This path intentionally does not call the legacy semantic
    /// database or reconstruct a module surface from the request buffer.
    fn compiler_receiver(&self, request: &RequestContext, position: Position) -> Option<phalcom_semantic::ResolvedReceiver> {
        if !matches!(request.source_match, SourceMatch::Exact) {
            return None;
        }
        let target = completion::target_at_snapshot(&request.document, position)?;
        self.compiler_receiver_for_range(request, target.receiver_range)
    }

    fn compiler_receiver_for_range(
        &self,
        request: &RequestContext,
        receiver_range: phalcom_common::range::SourceRange,
    ) -> Option<phalcom_semantic::ResolvedReceiver> {
        let compiler = request.compiler.as_deref()?;
        let module = request.compiler_module()?;
        compiler.editor().resolve_receiver_at(module, receiver_range)
    }

    fn compiler_callable_owner_at(
        &self,
        compiler: &phalcom_semantic::SemanticSnapshot,
        module: &phalcom_modules::ModuleId,
        offset: usize,
    ) -> Option<phalcom_semantic::DeclarationId> {
        let source = compiler.source_index().module(module)?;
        for offset in [offset, offset.saturating_sub(1)] {
            if let Some(site) = source.expression_site_at(offset)
                && let phalcom_semantic::SourceOwner::Callable(callable) = &site.id.owner
            {
                return Some(callable.owner.declaration().clone());
            }
        }
        source.structure.sites.values().filter_map(|site| {
            let phalcom_semantic::SourceSiteKind::Callable(callable) = &site.kind else {
                return None;
            };
            site.range.contains(offset).then_some(callable.owner.declaration().clone())
        }).min_by_key(|owner| {
            source
                .structure
                .sites
                .values()
                .find(|site| matches!(&site.kind, phalcom_semantic::SourceSiteKind::Callable(callable) if callable.owner.declaration() == owner && site.range.contains(offset)))
                .map_or(usize::MAX, |site| site.range.len())
        })
    }

    fn compiler_target_at_request(&self, request: &RequestContext, offset: usize) -> Option<phalcom_semantic::SemanticTargetId> {
        if !matches!(request.source_match, SourceMatch::Exact) {
            return None;
        }
        let compiler = request.compiler.as_deref()?;
        let module = request.compiler_module()?;
        compiler.editor().target_at(module, offset)
    }

    /// Builds selector-hover inputs from one canonical compiler callable.
    ///
    /// The compiler snapshot owns target identity, selector, dispatch side,
    /// signature, and return facts. The protocol member surface is consulted
    /// only for presentation metadata that has no compiler source-index
    /// equivalent yet (member kind and Phaldoc source ranges).
    fn compiler_callable_hover(&self, request: &RequestContext, callable: &phalcom_semantic::identity::CallableId) -> Option<CompilerCallableHover> {
        let compiler = request.compiler.as_deref()?;
        let signature = compiler.callable_signatures.get(callable);
        let source = compiler.source_index().callable_source(callable);
        if signature.is_none() && source.is_none() {
            return None;
        }
        let kind = source.map_or(hover::MemberKind::Method, |source| match source.kind {
            phalcom_semantic::SourceCallableKind::Getter => hover::MemberKind::Getter,
            phalcom_semantic::SourceCallableKind::Setter => hover::MemberKind::Setter,
            phalcom_semantic::SourceCallableKind::Constructor => hover::MemberKind::Construct,
            phalcom_semantic::SourceCallableKind::IndexGet => hover::MemberKind::Method,
            phalcom_semantic::SourceCallableKind::IndexSet => hover::MemberKind::Method,
            phalcom_semantic::SourceCallableKind::Method => match callable.side {
                phalcom_semantic::DispatchSide::Instance => hover::MemberKind::Method,
                phalcom_semantic::DispatchSide::Class => hover::MemberKind::StaticMethod,
            },
        });
        let phaldoc = source.and_then(|source| self.member_phaldoc(compiler, source));
        let presenter = phalcom_semantic::TypePresenter::new(&compiler.store);
        let formal = signature.map_or(FormalPresentation::Unknown, |signature| {
            phalcom_semantic::CallablePresentation::from_signature(signature, source, &presenter).return_type
        });
        Some((
            callable.selector.encode(),
            SelectorSite {
                owner: callable.owner.declaration().clone(),
                receiver: None,
                kind,
            },
            phaldoc,
            formal,
            compiler.advisory_callable(callable).map(|summary| summary.return_fact.clone()),
            compiler.editor().native_callable_presentation(callable),
        ))
    }

    /// Builds class-hover inputs from compiler declaration identity and
    /// hierarchy products. Source metadata is used only to harvest Phaldoc.
    fn compiler_class_hover(
        &self,
        request: &RequestContext,
        declaration: &phalcom_semantic::identity::DeclarationId,
    ) -> Option<(
        phalcom_semantic::DeclarationId,
        Option<phalcom_semantic::DeclarationId>,
        Option<hover::PhaldocDoc>,
    )> {
        let compiler = request.compiler.as_deref()?;
        compiler.surfaces.get(declaration)?;
        let superclass = compiler.hierarchy.superclass(declaration).cloned();
        let phaldoc = compiler.source_index().declaration_source(declaration).and_then(|metadata| {
            let definition_uri = compiler_uri_for_module(compiler, &declaration.module)?;
            self.with_source_snapshot(&definition_uri, |text, _, line_index| {
                hover::harvest_doc_for_declaration(
                    text,
                    line_index,
                    hover::DeclarationDocTarget::Class {
                        declaration: metadata.declaration_range,
                        name: metadata.name_range,
                    },
                )
            })
            .flatten()
        });
        Some((declaration.clone(), superclass, phaldoc))
    }

    /// Builds field-hover inputs from compiler field identity and advisory
    /// field products. Protocol surfaces contribute no lookup decision.
    fn compiler_field_hover(
        &self,
        request: &RequestContext,
        field: &phalcom_semantic::identity::FieldId,
    ) -> Option<(String, SelectorSite, Option<phalcom_semantic::advisory::AdvisoryFact>)> {
        let compiler = request.compiler.as_deref()?;
        let side = field.side;
        let surface = compiler.surfaces.get(&field.owner)?.surface(side);
        surface.get_field(&field.name)?;
        Some((
            field.name.to_string(),
            SelectorSite {
                owner: field.owner.clone(),
                receiver: None,
                kind: hover::MemberKind::Field,
            },
            compiler.advisory().field(field).cloned(),
        ))
    }

    fn compiler_binding_hover(&self, request: &RequestContext, binding: &phalcom_semantic::SourceSiteId) -> Option<String> {
        let compiler = request.compiler.as_deref()?;
        let source = compiler.source_index().module_for_site(binding)?;
        let info = source.structure.bindings.get(binding)?;
        let kind = match info.kind {
            phalcom_semantic::SourceBindingKind::MethodParameter
            | phalcom_semantic::SourceBindingKind::SetterParameter
            | phalcom_semantic::SourceBindingKind::IndexParameter
            | phalcom_semantic::SourceBindingKind::ClosureParameter => "parameter",
            _ => "mutable binding",
        };
        let mut sections = vec![format!("`{}` — {kind}", info.name)];
        if let Some(state) = compiler.formal_binding_at(binding) {
            match phalcom_semantic::TypePresenter::new(&compiler.store).present_knowledge(&state.current) {
                FormalPresentation::Known(text) => sections.push(format!("type: `{text}`")),
                FormalPresentation::Dynamic => sections.push("type: `Dynamic`".to_string()),
                _ => {}
            }
        }
        if let Some(advisory) = compiler.advisory_fact(binding)
            && !matches!(advisory.shape, phalcom_semantic::ValueShape::Unknown)
        {
            sections.push(format!(
                "runtime value: `{}`",
                phalcom_semantic::AdvisoryPresenter::present_shape(&advisory.shape)
            ));
        }
        let module = match &binding.owner {
            phalcom_semantic::SourceOwner::Module(module) => module,
            phalcom_semantic::SourceOwner::Callable(callable) => &callable.owner.module,
        };
        if let Some(uri) = compiler_uri_for_module(compiler, module)
            && let Some(doc) = self.with_source_snapshot(&uri, |text, program, line_index| {
                hover::harvest_doc_for_selector(text, program, line_index, &info.name)
            })
            && let Some(doc) = doc
        {
            if !doc.summary.is_empty() {
                sections.push(doc.summary);
            }
            sections.extend(doc.tags.into_iter().map(|(tag, payload)| format!("- **@{tag}** {payload}")));
        }
        Some(sections.join("\n\n---\n\n"))
    }

    fn compiler_uri_for_module(&self, compiler: &phalcom_semantic::SemanticSnapshot, module: &phalcom_modules::ModuleId) -> Option<Url> {
        compiler_uri_for_module(compiler, module)
    }

    fn compiler_site_location(&self, compiler: &phalcom_semantic::SemanticSnapshot, site: &phalcom_semantic::SourceSiteId) -> Option<Location> {
        let source = compiler.source_site(site)?;
        let module = match &site.owner {
            phalcom_semantic::SourceOwner::Module(module) => module,
            phalcom_semantic::SourceOwner::Callable(callable) => &callable.owner.module,
        };
        let uri = self.compiler_uri_for_module(compiler, module)?;
        let text = compiler
            .sources
            .get(module)
            .map(|published| published.text.as_ref())
            .or_else(|| compiler.presentation_source(module))?;
        let line_index = LineIndex::new(text);
        let range = line_index.range(source.range.start..source.range.end);
        Some(Location { uri, range })
    }

    fn compiler_target_locations(&self, compiler: &phalcom_semantic::SemanticSnapshot, target: &phalcom_semantic::SemanticTargetId) -> Vec<Location> {
        self.compiler_sites_locations(compiler, compiler.editor().definition_sites(target))
    }

    fn compiler_reference_locations(
        &self,
        compiler: &phalcom_semantic::SemanticSnapshot,
        target: &phalcom_semantic::SemanticTargetId,
        include_declaration: bool,
    ) -> Vec<Location> {
        let mut sites = compiler.editor().reference_sites(target);
        if include_declaration {
            sites.extend(compiler.editor().definition_sites(target));
        }
        sites.sort();
        sites.dedup();
        self.compiler_sites_locations(compiler, sites)
    }

    fn compiler_sites_locations(&self, compiler: &phalcom_semantic::SemanticSnapshot, sites: Vec<phalcom_semantic::SourceSiteId>) -> Vec<Location> {
        let mut locations = Vec::new();
        for site in sites {
            if let Some(location) = self.compiler_site_location(compiler, &site)
                && !locations
                    .iter()
                    .any(|existing: &Location| existing.uri == location.uri && existing.range == location.range)
            {
                locations.push(location);
            }
        }
        locations
    }

    fn compiler_workspace_symbols(&self, compiler: &phalcom_semantic::SemanticSnapshot, query: &str) -> Vec<SymbolInformation> {
        let query = query.to_lowercase();
        let mut symbols = Vec::new();
        for module in compiler.source_index().modules.values() {
            for site in module.structure.sites.values() {
                let (name, kind, container_name) = match &site.kind {
                    phalcom_semantic::SourceSiteKind::Declaration(declaration) => (declaration.name.to_string(), SymbolKind::CLASS, None),
                    phalcom_semantic::SourceSiteKind::Callable(callable) => {
                        (callable.selector.encode(), SymbolKind::METHOD, Some(callable.owner.name.to_string()))
                    }
                    phalcom_semantic::SourceSiteKind::Field(field) => (field.name.to_string(), SymbolKind::FIELD, Some(field.owner.name.to_string())),
                    _ => continue,
                };
                if !name.to_lowercase().contains(&query) {
                    continue;
                }
                let Some(location) = self.compiler_site_location(compiler, &site.id) else {
                    continue;
                };
                #[allow(deprecated)]
                symbols.push(SymbolInformation {
                    name,
                    kind,
                    tags: None,
                    deprecated: None,
                    location,
                    container_name,
                });
            }
        }
        symbols.sort_by(|left, right| left.name.cmp(&right.name).then_with(|| left.location.uri.cmp(&right.location.uri)));
        symbols.dedup_by(|left, right| left.name == right.name && left.location == right.location);
        symbols
    }

    /// Maps source metadata from closed files through the current canonical
    /// publication.
    fn cached_source(&self, uri: &Url) -> Option<CachedSource> {
        let source = self.closed_sources.read().expect("closed source cache lock poisoned").get(uri).cloned()?;
        let Some(snapshot) = self.analysis.snapshot() else {
            return Some(source);
        };
        if let Some(module) = compiler_module_for_uri(&snapshot, uri)
            && snapshot
                .sources
                .get(module)
                .is_some_and(|published| published.text.as_ref() != source.text.as_ref())
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

        if let Some(source) = self.cached_source(uri) {
            return Some(f(&source.text, &source.program, &source.line_index));
        }

        if let Some(text) = virtual_source_text(uri) {
            let parse_result = phalcom_ast::parser::parse(&text, 0);
            let line_index = LineIndex::new(&text);
            return Some(f(&text, &parse_result.program, &line_index));
        }

        None
    }

    fn member_phaldoc(
        &self,
        compiler: &phalcom_semantic::SemanticSnapshot,
        member: &phalcom_semantic::source_index::CallableSourceInfo,
    ) -> Option<hover::PhaldocDoc> {
        let owner = &member.id.owner;
        let definition_uri = compiler_uri_for_module(compiler, &owner.module)?;
        self.with_source_snapshot(&definition_uri, |text, program, line_index| {
            let target = hover::DeclarationDocTarget::Member {
                declaration: member.declaration_range,
                name: member.name_range,
            };
            hover::harvest_doc_for_declaration(text, line_index, target)
                .or_else(|| hover::harvest_pinned_doc_for_member(text, program, &owner.name, &member.id.selector.encode(), member.declaration_range))
        })
        .flatten()
    }

    /// Answers `textDocument/hover` from compiler-owned declaration, callable,
    /// field, source, and advisory metadata.
    fn hover_at(&self, uri: &Url, position: Position) -> Option<Hover> {
        let request = self.request_context(uri)?;
        self.hover_at_request(&request, uri, position)
    }

    /// Resolves read-only signature help from the pinned source and semantic snapshot.
    fn signature_help_at(&self, uri: &Url, position: Position) -> Option<SignatureHelp> {
        let request = self.request_context(uri)?;
        let offset = request.document.line_index.offset(position);
        let site = signature_help::call_site_at(&request.document.text, offset)?;

        if matches!(request.source_match, SourceMatch::Exact)
            && let Some(compiler) = request.compiler.as_deref()
            && let Some(module) = request.compiler_module()
            && let Some(signature) = compiler_signature_for_call(compiler, module, &site)
        {
            let advisory = compiler.advisory_callable(&signature.callable);
            return Some(signature_help::render_signature_help(
                signature,
                &compiler.store,
                advisory,
                site.active_parameter,
            ));
        }
        None
    }

    fn hover_at_request(&self, request: &RequestContext, _uri: &Url, position: Position) -> Option<Hover> {
        let offset = request.document.line_index.offset(position);
        if let Some(import_range) = import_path_range_at_offset(&request.document.parse.program, offset)
            && let Some(location) = compiler_import_definition_location(request, position)
        {
            return Some(Hover {
                contents: markdown_contents(format!("module: `{}`", location.uri)),
                range: Some(request.document.line_index.range(import_range.start..import_range.end)),
            });
        }
        let compiler = request.compiler.as_deref()?;
        let module = request.compiler_module()?;
        if !matches!(request.source_match, SourceMatch::Exact) {
            return None;
        }
        let occurrence = compiler.occurrence_at(module, offset);
        let Some(occurrence) = occurrence else {
            // Stale or unmapped text is syntax-only. Semantic fallbacks would
            // make an older index an accidental second authority.
            return None;
        };
        let span = request
            .document
            .line_index
            .range(occurrence.occurrence.range.start..occurrence.occurrence.range.end);

        if matches!(request.source_match, SourceMatch::Exact)
            && let Some(target) = self.compiler_target_at_request(request, offset)
        {
            match target {
                phalcom_semantic::SemanticTargetId::Callable(callable) => {
                    if let Some((selector, site, phaldoc, formal, advisory, native)) = self.compiler_callable_hover(request, &callable)
                        && let Some(mut contents) =
                            hover::render_selector_hover_with_formal_value(&selector, &[site], phaldoc.as_ref(), Some(&formal), advisory.as_ref())
                    {
                        if let Some(native) = native {
                            contents.push_str("\n\n---\n\n");
                            contents.push_str(&hover::render_native_callable_details(native.documentation));
                        }
                        return Some(Hover {
                            contents: markdown_contents(contents),
                            range: Some(span),
                        });
                    }
                }
                phalcom_semantic::SemanticTargetId::Declaration(declaration) => {
                    if let Some((class, superclass, phaldoc)) = self.compiler_class_hover(request, &declaration) {
                        return Some(Hover {
                            contents: markdown_contents(hover::render_class_hover(&class, superclass.as_ref(), phaldoc.as_ref())),
                            range: Some(span),
                        });
                    }
                }
                phalcom_semantic::SemanticTargetId::Field(field) => {
                    if let Some((name, site, advisory)) = self.compiler_field_hover(request, &field)
                        && let Some(contents) = hover::render_selector_hover_with_value(&name, &[site], None, advisory.as_ref())
                    {
                        return Some(Hover {
                            contents: markdown_contents(contents),
                            range: Some(span),
                        });
                    }
                }
                phalcom_semantic::SemanticTargetId::Binding(binding) => {
                    if let Some(contents) = self.compiler_binding_hover(request, &binding) {
                        return Some(Hover {
                            contents: markdown_contents(contents),
                            range: Some(span),
                        });
                    }
                }
                _ => {}
            }
        }

        None
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

fn compiler_signature_for_call<'a>(
    compiler: &'a phalcom_semantic::SemanticSnapshot,
    module: &phalcom_modules::ModuleId,
    site: &signature_help::CallSite,
) -> Option<&'a phalcom_semantic::CallableSemanticSignature> {
    let exact = compiler
        .occurrence_at(module, site.name_range.start)
        .filter(|occurrence| occurrence.occurrence.role == phalcom_semantic::OccurrenceRole::Call)
        .and_then(|occurrence| match occurrence.target {
            Some(phalcom_semantic::SemanticTargetId::Callable(callable)) => Some(callable.clone()),
            _ => None,
        });
    if let Some(callable) = exact {
        return compiler.callable_signatures().get(&callable);
    }

    let receiver_range = site.receiver_range?;
    let receiver = compiler.editor().resolve_receiver_at(module, receiver_range)?;
    let selector = phalcom_common::selector::Selector::try_decode_exact(&site.selector).ok()?;
    let pattern = phalcom_semantic::PartialCallPattern::from_selector_prefix(&selector);
    let access = compiler.editor().access_context_at(module, site.name_range.start);
    let candidates = compiler.editor().callable_candidates(&receiver, &pattern, &access);
    let [callable] = candidates.as_slice() else {
        return None;
    };
    compiler.callable_signatures().get(callable)
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
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: Some(vec![",".to_string()]),
                    ..SignatureHelpOptions::default()
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
            let closed_sources = self.closed_sources.clone();
            let inlay_refresh = self.inlay_refresh.clone();
            let semantic_token_refresh = self.semantic_token_refresh.clone();
            let counters = self.perf_counters();
            let documents = self.documents.clone();
            let publication = self.analysis.publication_handle();
            tokio::spawn(async move {
                while let Some(event) = events.recv().await {
                    match event {
                        AnalysisEvent::WorkspaceFileIndexed { uri, text: _, revision: _ } => {
                            let source = closed_sources.read().expect("closed source cache lock poisoned").get(&uri).cloned();
                            let Some(source) = source else { continue };
                            let mut cache = closed_sources.write().expect("closed source cache lock poisoned");
                            cache.insert(uri, source);
                        }
                        AnalysisEvent::WorkspaceFileRemoved { uri } => {
                            let mut cache = closed_sources.write().expect("closed source cache lock poisoned");
                            cache.remove(&uri);
                        }
                        AnalysisEvent::Published { effects, .. } => {
                            for uri in documents.open_uris() {
                                let Some(publication) = combined_diagnostics_for(&documents, publication.load().as_deref(), &uri) else {
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
                        AnalysisEvent::Log(log) => {
                            client.send_notification::<crate::analysis_log::AnalysisLogNotification>(*log).await;
                        }
                        AnalysisEvent::Error { message } => {
                            client.log_message(MessageType::ERROR, message).await;
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

    /// Applies changed settings and refreshes semantic analysis when configuration changes.
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
        for folder in &params.event.removed {
            self.remove_workspace_root(&folder.uri);
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
        self.schedule_workspace_scan(&roots);
    }

    /// Refreshes closed-file contributions for watched `.ph` changes.
    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        let mut removals = Vec::new();
        let mut refreshes = Vec::new();
        for change in params.changes {
            if change.typ == FileChangeType::DELETED {
                self.closed_sources.write().expect("closed source cache lock poisoned").remove(&change.uri);
                removals.push(change.uri.clone());
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
    /// the selector has no recorded definition (e.g. a builtin Universe class
    /// method — the index only covers user `.ph` source; `core-table.json`
    /// lookup is a later stage, plan DEC-LSP-B).
    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let Some(request) = self.request_context(&uri) else { return Ok(None) };

        let offset = request.document.line_index.offset(position);
        if let Some(target) = self.compiler_target_at_request(&request, offset)
            && let Some(compiler) = request.compiler.as_deref()
        {
            let locations = self.compiler_target_locations(compiler, &target);
            if !locations.is_empty() {
                return Ok(Some(GotoDefinitionResponse::Array(locations)));
            }
        }

        if let Some(location) = compiler_import_definition_location(&request, position) {
            return Ok(Some(GotoDefinitionResponse::Array(vec![location])));
        }

        Ok(None)
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
        if let Some(target) = self.compiler_target_at_request(&request, offset)
            && let Some(compiler) = request.compiler.as_deref()
        {
            let locations = self.compiler_reference_locations(compiler, &target, params.context.include_declaration);
            if !locations.is_empty() {
                return Ok(Some(locations));
            }
        }

        Ok(None)
    }

    /// Answers `workspace/symbol` from compiler-owned source declarations
    /// whose names contain `params.query`.
    ///
    /// Every result reports [`SymbolKind::METHOD`] — the index does not yet
    /// distinguish getter/setter/construct/field kinds in a
    /// `SymbolKind`-shaped way; refining this is left to a later stage
    /// (hover, Stage 4, already needs that per-kind rendering and is the
    /// natural place to add it once).
    async fn symbol(&self, params: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
        let symbols = self
            .analysis
            .snapshot()
            .map(|compiler| self.compiler_workspace_symbols(&compiler, &params.query))
            .unwrap_or_default();
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
        let offset = request.document.line_index.offset(position);
        let privileged = false;
        let line_prefix = request.document.text[..offset].rsplit('\n').next().unwrap_or_default();
        let import_context = crate::import_completion::detect_import_context(line_prefix);
        let mut items = if let Some(context) = import_context {
            match (request.compiler.as_deref(), request.compiler_module(), request.source_match) {
                (Some(compiler), Some(module), SourceMatch::Exact) => crate::import_completion::import_completions(compiler, module, &context),
                _ => Vec::new(),
            }
        } else if let (Some(compiler), Some(module)) = (request.compiler.as_deref(), request.compiler_module())
            && matches!(request.source_match, SourceMatch::Exact)
        {
            let resolved = self.compiler_receiver(&request, position);
            let lexical_class = self.compiler_callable_owner_at(compiler, module, offset);
            completion::compiler_contextual_completions(
                compiler,
                completion::CompilerCompletionContext {
                    resolved: resolved.as_ref(),
                    lexical_class: lexical_class.as_ref(),
                    privileged,
                    module,
                    text: &request.document.text,
                    offset,
                },
            )
        } else {
            completion::syntax_visible_completions(&request.document.parse.program, &request.document.text, offset)
        };
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

    /// Answers `textDocument/signatureHelp` from the pinned read-only snapshot.
    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let _span = PerfSpan::start_with_counters("signature_help", self.perf_counters());
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        Ok(self.signature_help_at(&uri, position))
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

fn virtual_source_text(uri: &Url) -> Option<String> {
    let module = builtin_module_from_uri(uri)?;
    phalcom_modules::UniverseSourceProvider::new()
        .source_text(&module)
        .ok()
        .map(|text| text.to_string())
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
    use crate::workspace_scan::AnalysisMode;
    use phalcom_modules::SourceRevision;

    #[test]
    fn server_config_parses_nested_stable_hint_policy() {
        let settings = json!({
            "phalcom": {
                "inlayHints": { "types": "stable", "suppressObvious": false }
            }
        });

        let config = ServerConfig::from_json(Some(&settings));

        assert_eq!(config.inlay_hints, HintPolicy::Stable);
        assert!(!config.suppress_obvious);
    }

    #[test]
    fn virtual_source_text_serves_canonical_builtin_module() {
        let uri = Url::parse("phalcom://universe/object/object").unwrap();
        let text = super::virtual_source_text(&uri).expect("canonical builtin source must be available");
        assert!(!text.is_empty());
    }

    #[tokio::test]
    async fn canonical_class_definition_location_resolves() {
        let (service, _) = LspService::new(super::Backend::new);
        let backend = service.inner();
        let uri = Url::parse("file:///main.ph").unwrap();
        let text = "class MyClass is Object {\n}\n".to_string();
        backend.documents.open_or_update_versioned(uri.clone(), text.clone(), Some(1));
        let parse_result = phalcom_ast::parser::parse(&text, 0);
        let revision = SourceRevision(1);
        backend.update_semantic_for_source(&uri, revision, std::sync::Arc::from(text.as_str()), &parse_result.program);
        backend.analysis.flush();

        let position = tower_lsp::lsp_types::Position { line: 0, character: 10 };
        let params = tower_lsp::lsp_types::GotoDefinitionParams {
            text_document_position_params: tower_lsp::lsp_types::TextDocumentPositionParams {
                text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        let response = backend.goto_definition(params).await.unwrap();
        assert!(response.is_some());
    }

    #[test]
    fn server_config_parses_dotted_off_hint_policy() {
        let settings = json!({
            "phalcom.inlayHints.types": "off"
        });

        let config = ServerConfig::from_json(Some(&settings));

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
        assert!(backend.analysis.snapshot().is_none(), "construction must not publish a source snapshot");
        assert_eq!(backend.perf_counters().snapshot().semantic_batches_started, 0);

        let gate = Arc::new(TestBatchGate::default());
        backend.analysis.install_test_batch_gate(gate.clone());
        let uri = Url::parse("file:///blocked-request.ph").unwrap();
        let source = "let value = 1\n";
        let parsed = phalcom_ast::parser::parse(source, 0);
        backend.documents.open_or_update(uri.clone(), source.to_string());
        backend.analysis.mark_open(uri.clone());
        let text: Arc<str> = Arc::from(source);
        backend
            .analysis
            .enqueue_file_update(uri.clone(), SourceRevision(1), text, Arc::new(parsed.program));
        gate.wait_until_before_entered();

        let initialize: tower_lsp::lsp_types::InitializeParams = serde_json::from_value(json!({
            "processId": null,
            "rootUri": null,
            "capabilities": {}
        }))
        .expect("initialize params");
        let response = backend.initialize(initialize).await.expect("initialize response");
        assert!(response.capabilities.inlay_hint_provider.is_some());
        assert!(response.capabilities.signature_help_provider.is_some());
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
    async fn presentation_configuration_does_not_rebuild_semantics() {
        let (service, _socket) = LspService::new(super::Backend::new);
        let backend = service.inner();
        backend
            .did_change_configuration(tower_lsp::lsp_types::DidChangeConfigurationParams {
                settings: json!({ "phalcom": { "inlayHints": { "types": "all" } } }),
            })
            .await;
        backend.analysis.flush();
        let snapshot = backend.perf_counters().snapshot();
        assert_eq!(
            snapshot.semantic_batches_started, 0,
            "presentation-only configuration must not rebuild semantics"
        );
        assert!(backend.analysis.snapshot().is_none());
        drop(service);
    }
}
