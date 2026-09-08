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

    std::fs::write(workspace.root.join("provider.ph"), "class Provider {}\nexport Provider\n").expect("repair missing module");
    let before = lsp.counter_snapshot();
    lsp.change(&main_uri, "import .provider as provider\n").await;
    lsp.wait_for_semantic_publication_after(before).await;

    let repaired = lsp.wait_for_publish_diagnostics(&main_uri).await;
    assert!(
        repaired["diagnostics"].as_array().is_some_and(Vec::is_empty),
        "repair must clear the canonical module diagnostic: {repaired:#?}"
    );

    lsp.finish().await;
}
