//! Production-path acceptance checks for Plan B's indexed LSP adaptation.

mod support;

use std::path::PathBuf;

use serde_json::json;
use tower_lsp::lsp_types::Url;

use support::TestLsp;

struct ScratchWorkspace {
    root: PathBuf,
}

impl ScratchWorkspace {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("phalcom-lsp-plan-b-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create Plan B workspace");
        std::fs::write(root.join("package.ph"), "").expect("write package marker");
        Self { root }
    }

    fn uri(&self, name: &str) -> String {
        Url::from_file_path(self.root.join(name)).expect("source URI").to_string()
    }

    fn root_uri(&self) -> String {
        Url::from_directory_path(&self.root).expect("workspace URI").to_string()
    }
}

impl Drop for ScratchWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[tokio::test]
async fn indexed_navigation_symbols_and_request_work_are_bounded() {
    let workspace = ScratchWorkspace::new();
    let provider_uri = workspace.uri("provider.ph");
    let main_uri = workspace.uri("main.ph");
    let provider = "class Circle { area() -> Int { 1 } }\nenum Mood { @variant Happy }\ntype Alias = Circle\nexport Circle\n";
    let main = "from .provider import Circle as Shape\nlet value = Shape\n";
    std::fs::write(workspace.root.join("provider.ph"), provider).expect("write provider");
    std::fs::write(workspace.root.join("main.ph"), main).expect("write main");

    let mut lsp = TestLsp::start().await;
    let initialize = lsp.initialize(Some(&workspace.root_uri())).await;
    assert!(initialize["result"]["capabilities"]["workspaceSymbolProvider"].is_boolean());
    lsp.open_and_wait(&provider_uri, provider).await;
    lsp.open_and_wait(&main_uri, main).await;

    let definition = lsp
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": main_uri },
                "position": { "line": 1, "character": 13 }
            }),
        )
        .await;
    assert!(
        definition["result"].as_array().is_some_and(|locations| !locations.is_empty()),
        "definition: {definition:#?}"
    );

    let references = lsp
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": main_uri },
                "position": { "line": 1, "character": 13 },
                "context": { "includeDeclaration": true }
            }),
        )
        .await;
    assert!(
        references["result"].as_array().is_some_and(|locations| !locations.is_empty()),
        "references: {references:#?}"
    );

    let symbols = lsp.request("workspace/symbol", json!({ "query": "Circle" })).await;
    assert!(
        symbols["result"]
            .as_array()
            .is_some_and(|entries| entries.iter().any(|entry| entry["name"] == "Circle")),
        "symbols: {symbols:#?}"
    );
    let enum_symbols = lsp.request("workspace/symbol", json!({ "query": "Mood" })).await;
    assert!(
        enum_symbols["result"]
            .as_array()
            .is_some_and(|entries| entries.iter().any(|entry| entry["kind"] == 10)),
        "enum symbols: {enum_symbols:#?}"
    );
    let alias_symbols = lsp.request("workspace/symbol", json!({ "query": "Alias" })).await;
    assert!(
        alias_symbols["result"]
            .as_array()
            .is_some_and(|entries| entries.iter().any(|entry| entry["kind"] == 26)),
        "alias symbols: {alias_symbols:#?}"
    );

    let counters = lsp.counter_snapshot();
    assert!(counters.reference_source_modules_converted >= 1);
    assert!(counters.reference_line_indexes_built >= 1);
    assert_eq!(counters.reference_duplicate_filter_steps, 0);
    assert_eq!(counters.workspace_symbol_source_shard_scans, 0);
    assert_eq!(counters.diagnostic_workspace_source_scans, 0);
    lsp.finish().await;
}
