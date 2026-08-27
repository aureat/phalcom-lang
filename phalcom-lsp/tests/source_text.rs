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
