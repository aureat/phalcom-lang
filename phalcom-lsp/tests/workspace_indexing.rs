//! Production-path acceptance checks for Plan B's indexed LSP adaptation.

mod support;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use phalcom_lsp::diagnostics::{SemanticDiagnosticSource, semantic_diagnostic_to_lsp_diagnostic_with_snapshot};
use phalcom_lsp::line_index::LineIndex;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::{DiagnosticCode, SemanticDiagnostic};
use serde_json::json;
use tower_lsp::lsp_types::Url;

use support::TestLsp;

static NEXT_WORKSPACE: AtomicU64 = AtomicU64::new(1);

struct ScratchWorkspace {
    root: PathBuf,
}

impl ScratchWorkspace {
    fn new() -> Self {
        let id = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("phalcom-lsp-plan-b-{}-{id}", std::process::id()));
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

async fn wait_for_discovered(lsp: &TestLsp, minimum: u64) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if lsp.counter_snapshot().workspace_files_discovered >= minimum {
            return;
        }
        tokio::task::yield_now().await;
    }
    panic!("workspace scan did not discover {minimum} files within the 30-second yield budget");
}

#[tokio::test]
async fn many_references_in_few_modules_build_only_few_line_indexes() {
    let workspace = ScratchWorkspace::new();
    let provider_uri = workspace.uri("provider.ph");
    let main_uri = workspace.uri("main.ph");
    let provider = "class Circle {}\nexport Circle\n";
    let mut main = String::from("from .provider import Circle as Shape\n");
    for index in 0..128 {
        main.push_str(&format!("let value{index} = Shape\n"));
    }
    std::fs::write(workspace.root.join("provider.ph"), provider).expect("write provider");
    std::fs::write(workspace.root.join("main.ph"), &main).expect("write main");

    let mut lsp = TestLsp::start().await;
    lsp.initialize(Some(&workspace.root_uri())).await;
    lsp.open_and_wait(&provider_uri, provider).await;
    lsp.open_and_wait(&main_uri, &main).await;
    let before = lsp.counter_snapshot();
    let references = lsp
        .request(
            "textDocument/references",
            json!({
                "textDocument": { "uri": main_uri },
                "position": { "line": 1, "character": 15 },
                "context": { "includeDeclaration": true }
            }),
        )
        .await;
    let locations = references["result"].as_array().expect("reference result array");
    assert_eq!(locations.iter().filter(|location| location["uri"] == main_uri).count(), 128);
    assert_eq!(locations.iter().filter(|location| location["uri"] == provider_uri).count(), 1);
    let after = lsp.counter_snapshot();
    assert!(after.reference_source_modules_converted - before.reference_source_modules_converted <= 2);
    assert!(after.reference_line_indexes_built - before.reference_line_indexes_built <= 2);
    assert_eq!(after.reference_duplicate_filter_steps, 0);
    lsp.finish().await;
}

fn resolved_test_module(project: ResolvedProjectId, name: &str) -> ModuleId {
    ModuleId::resolved(
        project,
        ModulePath::root().join(ModuleComponent::from_identifier(name).expect("valid test module name")),
    )
}

fn empty_semantic_snapshot() -> phalcom_semantic::SemanticSnapshot {
    phalcom_semantic::SemanticSnapshot::new(
        phalcom_semantic::WorkspaceId::from_raw(1),
        phalcom_semantic::SemanticRevision::from_raw(1),
        1,
        Arc::new(phalcom_semantic::TypeStore::new()),
        Arc::new(Default::default()),
        Arc::new(Default::default()),
        Arc::new(phalcom_semantic::SurfaceDispatchResolver::new()),
        Arc::new(phalcom_semantic::CallableSignatureTable::new()),
        Arc::new(phalcom_semantic::DeclarationTypeTable::new()),
        Arc::new(phalcom_semantic::MapTypeHierarchy::new()),
        Arc::new(Default::default()),
        Arc::new(phalcom_modules::graph::SemanticGraph::default()),
    )
}

#[test]
fn related_diagnostic_maps_only_its_named_secondary_module_in_large_source_map() {
    let project = ResolvedProjectId::from_raw(1);
    let primary_module = resolved_test_module(project, "main");
    let secondary_module = resolved_test_module(project, "provider");
    let primary_uri = Url::parse("file:///workspace/main.ph").expect("primary URI");
    let secondary_uri = Url::parse("file:///workspace/provider.ph").expect("secondary URI");
    let primary_text = "let x: Int = value\n";
    let secondary_text = "class Value {}\n";

    let mut sources = BTreeMap::new();
    sources.insert(
        secondary_module.clone(),
        SemanticDiagnosticSource {
            uri: secondary_uri.clone(),
            line_index: LineIndex::new(secondary_text),
        },
    );
    for index in 0..256 {
        let name = format!("extra{index:03}");
        sources.insert(
            resolved_test_module(project, &name),
            SemanticDiagnosticSource {
                uri: Url::parse(&format!("file:///workspace/{name}.ph")).expect("extra URI"),
                line_index: LineIndex::new("class Extra {}\n"),
            },
        );
    }

    let diagnostic = SemanticDiagnostic::error_in(
        primary_module,
        DiagnosticCode::BindingInitializerMismatch,
        "binding initializer is incompatible",
        (4..9).into(),
    )
    .with_label_in(secondary_module, (6..11).into(), "declared in provider");
    let snapshot = empty_semantic_snapshot();
    let mapped = semantic_diagnostic_to_lsp_diagnostic_with_snapshot(&diagnostic, &snapshot, &LineIndex::new(primary_text), &primary_uri, &sources);

    let related = mapped.related_information.expect("cross-module related information");
    assert_eq!(related.len(), 1, "only the explicit secondary label should be adapted");
    assert_eq!(related[0].location.uri, secondary_uri);
    assert_eq!(related[0].message, "declared in provider");
    assert!(
        related
            .iter()
            .all(|item| item.location.uri != Url::parse("file:///workspace/extra000.ph").unwrap()),
        "unrelated source-map entries must not participate in diagnostic adaptation"
    );
}

#[tokio::test]
async fn diagnostic_adaptation_is_exercised_and_scan_free_in_large_workspace() {
    let workspace = ScratchWorkspace::new();
    for index in 0..256 {
        std::fs::write(
            workspace.root.join(format!("extra{index:03}.ph")),
            format!("class Extra{index:03} {{ value() -> Int {{ {index} }} }}\n"),
        )
        .expect("write extra source");
    }
    let main_uri = workspace.uri("main.ph");
    let main = "let x: Int = \"string\"\n";
    std::fs::write(workspace.root.join("main.ph"), main).expect("write diagnostic source");

    let mut lsp = TestLsp::start().await;
    lsp.initialize_with_options(Some(&workspace.root_uri()), json!({ "phalcom": { "analysis": { "mode": "workspace" } } }))
        .await;
    wait_for_discovered(&lsp, 256).await;
    lsp.open_and_wait(&main_uri, main).await;
    let published = lsp.wait_for_nonempty_publish_diagnostics(&main_uri).await;
    assert!(published["diagnostics"].as_array().is_some_and(|diagnostics| !diagnostics.is_empty()));
    let counters = lsp.counter_snapshot();
    assert_eq!(counters.diagnostic_workspace_source_scans, 0);
    assert_eq!(counters.workspace_symbol_source_shard_scans, 0);
    lsp.finish().await;
}
