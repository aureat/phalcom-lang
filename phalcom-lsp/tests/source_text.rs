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
