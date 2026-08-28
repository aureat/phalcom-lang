use phalcom_lsp::line_index::LineIndex;
use serde_json::json;

use crate::support::TestLsp;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn virtual_builtin_source_text_request_returns_canonical_document() {
    let mut lsp = TestLsp::start().await;
    let initialize = lsp.initialize(None).await;
    assert!(initialize.get("result").is_some(), "server initializes before source request");

    let response = lsp.request("phalcom/sourceText", json!({ "uri": "phalcom://universe/object/object" })).await;
    let text = response["result"].as_str().expect("source provider returns text");
    assert!(!text.is_empty());
    assert!(text.contains("class Object") || text.contains("class object"));
    lsp.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn core_source_text_matches_compiler_presentation_publication() {
    let mut lsp = TestLsp::start().await;
    let initialize = lsp.initialize(None).await;
    assert!(initialize.get("result").is_some(), "server initializes before source request");

    lsp.open_and_wait("file:///core-presentation-probe.ph", "class Probe {}\n").await;

    let response = lsp.request("phalcom/sourceText", json!({ "uri": "phalcom://core" })).await;
    let actual = response["result"].as_str().expect("core source provider returns text");
    let expected = phalcom_semantic::core_surface::render_canonical_core_source();
    assert_eq!(
        actual,
        expected.as_ref(),
        "the editor must receive the exact compiler-owned core presentation text whose ranges are published in the semantic snapshot"
    );
    lsp.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn builtin_definition_range_indexes_the_compiler_core_presentation() {
    let mut lsp = TestLsp::start().await;
    let initialize = lsp.initialize(None).await;
    assert!(initialize.get("result").is_some(), "server initializes before definition request");

    let uri = "file:///core-definition-range-probe.ph";
    lsp.open_and_wait(uri, "const value: Int = 42\n").await;

    let definition = lsp
        .request(
            "textDocument/definition",
            json!({
                "textDocument": { "uri": uri },
                "position": { "line": 0, "character": 13 }
            }),
        )
        .await;
    let locations = definition["result"].as_array().expect("builtin annotation definition returns a location array");
    let location = locations.first().expect("Int annotation has a canonical definition location");
    assert_eq!(location["uri"].as_str(), Some("phalcom://core"));

    let source = lsp.request("phalcom/sourceText", json!({ "uri": "phalcom://core" })).await;
    let core_text = source["result"].as_str().expect("core presentation source is available");
    let range: tower_lsp::lsp_types::Range = serde_json::from_value(location["range"].clone()).expect("definition range is valid LSP data");
    let index = LineIndex::new(core_text);
    let start = index.offset(range.start);
    let end = index.offset(range.end);
    assert_eq!(
        core_text.get(start..end),
        Some("Int"),
        "the definition range must select the canonical Int declaration in the same compiler-owned text returned to the editor"
    );

    lsp.finish().await;
}
