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
    let main = "from .provider import Circle as Shape\nlet value = Shape\nlet second = Shape\nlet third = Shape\n";
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
    assert!(
        definition["result"].as_array().unwrap().iter().all(|location| location["uri"] == provider_uri),
        "alias definition must resolve to the imported declaration: {definition:#?}"
    );

    let import_prefix_definition = lsp
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": main_uri },
                "position": { "line": 0, "character": 7 }
            }),
        )
        .await;
    assert!(
        import_prefix_definition["result"]
            .as_array()
            .is_some_and(|locations| locations.iter().any(|location| location["uri"] == provider_uri)),
        "import-prefix navigation must resolve to the provider module: {import_prefix_definition:#?}"
    );

    let references = lsp
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": main_uri },
                "position": { "line": 0, "character": 33 },
                "context": { "includeDeclaration": true }
            }),
        )
        .await;
    assert!(
        references["result"].as_array().is_some_and(|locations| !locations.is_empty()),
        "references: {references:#?}"
    );
    let lexical_references = references["result"].as_array().unwrap();
    assert!(
        lexical_references.len() == 4
            && lexical_references.iter().filter(|location| location["uri"] == main_uri).count() == 3
            && lexical_references.iter().filter(|location| location["uri"] == provider_uri).count() == 1,
        "alias references must include three local uses plus one canonical declaration: {references:#?}"
    );

    let upstream_references = lsp
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": provider_uri },
                "position": { "line": 0, "character": 7 },
                "context": { "includeDeclaration": false }
            }),
        )
        .await;
    assert!(
        upstream_references["result"]
            .as_array()
            .is_some_and(|locations| locations.iter().any(|location| location["uri"] == main_uri)),
        "semantic upstream references must include the importing module: {upstream_references:#?}"
    );

    let before_symbols = lsp.counter_snapshot();
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

    let after_symbols = lsp.counter_snapshot();
    assert!(
        after_symbols.reference_source_modules_converted <= 5,
        "reference conversion must stay bounded: {after_symbols:#?}"
    );
    assert!(
        after_symbols.reference_line_indexes_built <= 5,
        "line-index construction must stay bounded: {after_symbols:#?}"
    );
    assert_eq!(after_symbols.reference_duplicate_filter_steps, 0);
    assert_eq!(after_symbols.workspace_symbol_source_shard_scans, 0);
    assert_eq!(after_symbols.diagnostic_workspace_source_scans, 0);
    assert_eq!(
        after_symbols.reference_source_modules_converted,
        before_symbols.reference_source_modules_converted
    );
    assert_eq!(after_symbols.reference_line_indexes_built, before_symbols.reference_line_indexes_built);
    lsp.finish().await;
}
