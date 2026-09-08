//! End-to-end canonical module diagnostic publication and repair coverage.

mod support;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;
use tower_lsp::lsp_types::Url;

use support::TestLsp;

static NEXT_WORKSPACE: AtomicU64 = AtomicU64::new(1);

struct ScratchWorkspace {
    root: PathBuf,
}

impl ScratchWorkspace {
    fn new() -> Self {
        let id = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("phalcom-lsp-module-diagnostics-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&root).expect("create module diagnostics workspace");
        std::fs::write(root.join("package.ph"), "").expect("write package marker");
        Self { root }
    }

    fn root_uri(&self) -> String {
        Url::from_directory_path(&self.root).expect("workspace URI").to_string()
    }

    fn file_uri(&self, name: &str) -> String {
        Url::from_file_path(self.root.join(name)).expect("source URI").to_string()
    }
}

impl Drop for ScratchWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn diagnostic_code(params: &Value, code: &str) -> bool {
    params["diagnostics"]
        .as_array()
        .is_some_and(|diagnostics| diagnostics.iter().any(|diagnostic| diagnostic["code"] == code))
}

fn assert_missing_export_diagnostic(params: &Value, main_uri: &str, source: &str, provider_name: &str, imported_name: &str) {
    let diagnostic = params["diagnostics"]
        .as_array()
        .and_then(|diagnostics| diagnostics.iter().find(|diagnostic| diagnostic["code"] == "module.export.missing"))
        .unwrap_or_else(|| panic!("expected module.export.missing diagnostic: {params:#?}"));
    assert_eq!(params["uri"], main_uri);
    assert_eq!(diagnostic["severity"], 1, "missing export is an error");
    assert_eq!(diagnostic["range"]["start"]["line"], 0);
    let start = source.find(imported_name).expect("imported name must occur in source");
    assert_eq!(diagnostic["range"]["start"]["character"], start);
    assert_eq!(diagnostic["range"]["end"]["character"], start + imported_name.len());
    assert!(diagnostic["message"].as_str().is_some_and(|message| message.contains(provider_name)));
    assert!(diagnostic["message"].as_str().is_some_and(|message| message.contains(imported_name)));
}

#[tokio::test]
async fn unresolved_module_diagnostic_uses_canonical_code_and_clears_after_repair() {
    let workspace = ScratchWorkspace::new();
    let main_uri = workspace.file_uri("main.ph");
    let unresolved = "import .missing as missing\n";

    let mut lsp = TestLsp::start().await;
    lsp.initialize(Some(&workspace.root_uri())).await;
    lsp.open_and_wait(&main_uri, unresolved).await;

    let initial = lsp.wait_for_nonempty_publish_diagnostics(&main_uri).await;
    assert!(
        diagnostic_code(&initial, "module.import.unresolved"),
        "unresolved module diagnostic must use the canonical code: {initial:#?}"
    );

    let missing_uri = workspace.file_uri("missing.ph");
    std::fs::write(workspace.root.join("missing.ph"), "class Missing {}\nexport Missing\n").expect("repair missing module");
    let before = lsp.counter_snapshot();
    lsp.watched_file_created(&missing_uri).await;
    lsp.wait_for_semantic_publication_after(before).await;

    let repaired = lsp.wait_for_publish_diagnostics(&main_uri).await;
    assert!(
        repaired["diagnostics"].as_array().is_some_and(Vec::is_empty),
        "repair must clear the canonical module diagnostic: {repaired:#?}"
    );

    lsp.finish().await;
}

#[tokio::test]
async fn private_export_diagnostic_keeps_importer_uri_and_exact_name_range() {
    let workspace = ScratchWorkspace::new();
    let main_uri = workspace.file_uri("main.ph");
    std::fs::write(workspace.root.join("provider.ph"), "class Private {}\n").expect("write private provider");
    let source = "from .provider import Private\n";

    let mut lsp = TestLsp::start().await;
    lsp.initialize(Some(&workspace.root_uri())).await;
    lsp.open_and_wait(&main_uri, source).await;
    let published = lsp.wait_for_nonempty_publish_diagnostics(&main_uri).await;
    assert_missing_export_diagnostic(&published, &main_uri, source, "provider", "Private");
    lsp.finish().await;
}

#[tokio::test]
async fn unknown_export_name_diagnostic_keeps_importer_uri_and_exact_name_range() {
    let workspace = ScratchWorkspace::new();
    let main_uri = workspace.file_uri("main.ph");
    std::fs::write(workspace.root.join("provider.ph"), "class Public {}\nexport Public\n").expect("write provider");
    let source = "from .provider import DoesNotExist\n";

    let mut lsp = TestLsp::start().await;
    lsp.initialize(Some(&workspace.root_uri())).await;
    lsp.open_and_wait(&main_uri, source).await;
    let published = lsp.wait_for_nonempty_publish_diagnostics(&main_uri).await;
    assert_missing_export_diagnostic(&published, &main_uri, source, "provider", "DoesNotExist");
    lsp.finish().await;
}
