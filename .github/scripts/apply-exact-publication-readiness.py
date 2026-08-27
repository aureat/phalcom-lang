from pathlib import Path

# publication.rs: expose only a read-only publication/coherence handle.
p = Path("phalcom-lsp/src/publication.rs")
text = p.read_text()
needle = '''impl SemanticPublication {
'''
insert = '''/// Read-only handle used by protocol scheduling and tests to observe whether
/// an exact source document has reached the canonical semantic publication.
///
/// This handle exposes no semantic lookup or mutation operations. Feature
/// requests continue to query the immutable compiler snapshot through their
/// normal request context.
#[derive(Clone)]
pub struct SemanticPublicationHandle {
    publication: Arc<SemanticPublication>,
}

impl SemanticPublicationHandle {
    pub(crate) fn new(publication: Arc<SemanticPublication>) -> Self {
        Self { publication }
    }

    /// Returns whether the latest canonical publication contains `text` for
    /// the already-produced display path `path`.
    ///
    /// The comparison is exact and performs no filesystem reads or path
    /// canonicalization.
    pub fn contains_exact_source(&self, path: &std::path::Path, text: &str) -> bool {
        let Some(snapshot) = self.publication.load() else {
            return false;
        };
        let Some(module) = snapshot.module_for_display_path(path) else {
            return false;
        };
        snapshot.sources.get(module).is_some_and(|source| source.text.as_ref() == text)
    }
}

'''
if insert not in text:
    if needle not in text:
        raise SystemExit("publication impl anchor missing")
    text = text.replace(needle, insert + needle, 1)
p.write_text(text)

# analysis_service.rs: compiler publication remains private; expose opaque read handle.
p = Path("phalcom-lsp/src/analysis_service.rs")
text = p.read_text()
old = '''    /// Returns the counter set shared by this service and its semantic database.
    pub fn perf_counters(&self) -> PerfCountersHandle {
        self.counters.clone()
    }
'''
new = '''    /// Returns an opaque read-only handle to source coherence in the latest
    /// canonical semantic publication.
    pub(crate) fn semantic_publication_handle(&self) -> crate::publication::SemanticPublicationHandle {
        crate::publication::SemanticPublicationHandle::new(self.publication.clone())
    }

    /// Returns the counter set shared by this service and its semantic database.
    pub fn perf_counters(&self) -> PerfCountersHandle {
        self.counters.clone()
    }
'''
if old not in text:
    raise SystemExit("analysis service perf anchor missing")
text = text.replace(old, new, 1)
p.write_text(text)

# backend.rs: test/protocol harnesses can capture publication readiness without semantic ownership.
p = Path("phalcom-lsp/src/backend.rs")
text = p.read_text()
old = '''    /// Returns this backend's compact performance counters for diagnostics and
    /// benchmark harnesses. The counters are owned by this backend's worker.
    pub fn perf_counters(&self) -> PerfCountersHandle {
        self.analysis.perf_counters()
    }
'''
new = '''    /// Returns this backend's compact performance counters for diagnostics and
    /// benchmark harnesses. The counters are owned by this backend's worker.
    pub fn perf_counters(&self) -> PerfCountersHandle {
        self.analysis.perf_counters()
    }

    /// Returns a read-only publication-coherence handle for integration
    /// scheduling. It cannot perform semantic queries or mutate compiler state.
    pub fn semantic_publication_handle(&self) -> crate::publication::SemanticPublicationHandle {
        self.analysis.semantic_publication_handle()
    }
'''
if old not in text:
    raise SystemExit("backend perf anchor missing")
text = text.replace(old, new, 1)
p.write_text(text)

# lib.rs: re-export opaque handle while keeping publication implementation private.
p = Path("phalcom-lsp/src/lib.rs")
text = p.read_text()
anchor = '''pub use backend::Backend;
'''
replacement = '''pub use backend::Backend;
pub use publication::SemanticPublicationHandle;
'''
if replacement not in text:
    if anchor not in text:
        raise SystemExit("lib backend export anchor missing")
    text = text.replace(anchor, replacement, 1)
p.write_text(text)

# TestLsp: exact source publication, not global batch count, is feature readiness.
p = Path("phalcom-lsp/tests/support/lsp_client.rs")
text = p.read_text()
text = text.replace(
    'use phalcom_lsp::Backend;\n',
    'use phalcom_lsp::{Backend, SemanticPublicationHandle};\n',
    1,
)
old = '''    counters: Arc<Mutex<Option<PerfCountersHandle>>>,
}
'''
new = '''    counters: Arc<Mutex<Option<PerfCountersHandle>>>,
    publication: Arc<Mutex<Option<SemanticPublicationHandle>>>,
}
'''
if old not in text:
    raise SystemExit("TestLsp field anchor missing")
text = text.replace(old, new, 1)
old = '''        let counters = Arc::new(Mutex::new(None));
        let counters_for_backend = counters.clone();
        let (service, socket) = LspService::build(move |client| {
            let backend = Backend::new(client);
            *counters_for_backend.lock().expect("counter capture lock poisoned") = Some(backend.perf_counters());
            backend
        })
'''
new = '''        let counters = Arc::new(Mutex::new(None));
        let counters_for_backend = counters.clone();
        let publication = Arc::new(Mutex::new(None));
        let publication_for_backend = publication.clone();
        let (service, socket) = LspService::build(move |client| {
            let backend = Backend::new(client);
            *counters_for_backend.lock().expect("counter capture lock poisoned") = Some(backend.perf_counters());
            *publication_for_backend.lock().expect("publication capture lock poisoned") = Some(backend.semantic_publication_handle());
            backend
        })
'''
if old not in text:
    raise SystemExit("TestLsp start anchor missing")
text = text.replace(old, new, 1)
old = '''            next_version: 1,
            counters,
        }
'''
new = '''            next_version: 1,
            counters,
            publication,
        }
'''
if old not in text:
    raise SystemExit("TestLsp constructor anchor missing")
text = text.replace(old, new, 1)
old = '''    pub async fn open_and_wait(&mut self, uri: &str, text: &str) {
        let before = self.counter_snapshot();
        self.open(uri, text).await;
        self.wait_for_semantic_publication_after(before).await;
    }
'''
new = '''    pub async fn open_and_wait(&mut self, uri: &str, text: &str) {
        self.open(uri, text).await;
        self.wait_for_exact_source_publication(uri, text).await;
    }

    pub async fn change_and_wait(&mut self, uri: &str, text: &str) {
        self.change(uri, text).await;
        self.wait_for_exact_source_publication(uri, text).await;
    }

    async fn wait_for_exact_source_publication(&self, uri: &str, text: &str) {
        let uri = tower_lsp::lsp_types::Url::parse(uri).expect("test URI");
        let path = uri.to_file_path().expect("semantic test source must use a file URI");
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            let published = self
                .publication
                .lock()
                .expect("publication capture lock poisoned")
                .as_ref()
                .expect("backend publication handle captured during start")
                .contains_exact_source(&path, text);
            if published {
                return;
            }
            tokio::task::yield_now().await;
        }
        panic!("exact semantic source was not published within the 30-second yield budget: {uri}");
    }
'''
if old not in text:
    raise SystemExit("TestLsp open_and_wait anchor missing")
text = text.replace(old, new, 1)
p.write_text(text)

# Stage4 harness: capture the same opaque handle and wait for exact source after didOpen.
p = Path("phalcom-lsp/tests/stage4_hover.rs")
text = p.read_text()
text = text.replace(
    'use std::path::PathBuf;\nuse std::time::{Duration, Instant};\n',
    'use std::path::PathBuf;\nuse std::sync::{Arc, Mutex};\nuse std::time::{Duration, Instant};\n',
    1,
)
text = text.replace(
    'use phalcom_lsp::Backend;\n',
    'use phalcom_lsp::{Backend, SemanticPublicationHandle};\n',
    1,
)
old = '''fn spawn_server() -> (tokio::io::DuplexStream, tokio::task::JoinHandle<()>) {
    let (server_end, client_end) = tokio::io::duplex(1 << 16);
    let (server_read, server_write) = tokio::io::split(server_end);
    let (service, socket) = LspService::new(Backend::new);
    let server_task = tokio::spawn(async move {
        Server::new(server_read, server_write, socket).serve(service).await;
    });
    (client_end, server_task)
}
'''
new = '''fn spawn_server() -> (tokio::io::DuplexStream, tokio::task::JoinHandle<()>, SemanticPublicationHandle) {
    let (server_end, client_end) = tokio::io::duplex(1 << 16);
    let (server_read, server_write) = tokio::io::split(server_end);
    let publication = Arc::new(Mutex::new(None));
    let publication_for_backend = publication.clone();
    let (service, socket) = LspService::build(move |client| {
        let backend = Backend::new(client);
        *publication_for_backend.lock().expect("publication capture lock poisoned") = Some(backend.semantic_publication_handle());
        backend
    })
    .finish();
    let handle = publication
        .lock()
        .expect("publication capture lock poisoned")
        .as_ref()
        .expect("backend publication handle captured")
        .clone();
    let server_task = tokio::spawn(async move {
        Server::new(server_read, server_write, socket).serve(service).await;
    });
    (client_end, server_task, handle)
}
'''
if old not in text:
    raise SystemExit("stage4 spawn_server anchor missing")
text = text.replace(old, new, 1)
old = '''async fn did_open(client_end: &mut tokio::io::DuplexStream, uri: &str, text: &str) {
'''
new = '''async fn did_open(
    client_end: &mut tokio::io::DuplexStream,
    publication: &SemanticPublicationHandle,
    uri: &str,
    text: &str,
) {
'''
if old not in text:
    raise SystemExit("stage4 did_open signature anchor missing")
text = text.replace(old, new, 1)
old = '''    // Drain the didOpen's publishDiagnostics notification.
    let _ = read_message(client_end).await;
}
'''
new = '''    // Drain the immediate syntax-diagnostics publication; it is not a
    // semantic readiness signal.
    let _ = read_message(client_end).await;

    let uri = url::Url::parse(uri).expect("test URI");
    let path = uri.to_file_path().expect("Stage 4 semantic source must use a file URI");
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if publication.contains_exact_source(&path, text) {
            return;
        }
        tokio::task::yield_now().await;
    }
    panic!("exact Stage 4 semantic source was not published within the 30-second yield budget: {uri}");
}
'''
if old not in text:
    raise SystemExit("stage4 did_open tail anchor missing")
text = text.replace(old, new, 1)
# All stage4 tests bind spawn_server the same way; add publication and pass it to did_open.
text = text.replace(
    'let (mut client_end, server_task) = spawn_server();',
    'let (mut client_end, server_task, publication) = spawn_server();',
)
text = text.replace(
    'did_open(&mut client_end, uri, text).await;',
    'did_open(&mut client_end, &publication, uri, text).await;',
)
text = text.replace(
    'did_open(&mut client_end, &main_uri, main_text).await;',
    'did_open(&mut client_end, &publication, &main_uri, main_text).await;',
)
p.write_text(text)

# Cleanup warning made obsolete by canonical method-family owner traversal.
p = Path("phalcom-semantic/src/session.rs")
text = p.read_text().replace(
    'use crate::types::relation::{MapTypeHierarchy, TypeHierarchy};',
    'use crate::types::relation::MapTypeHierarchy;',
    1,
)
p.write_text(text)

print("exact publication readiness candidate applied")
