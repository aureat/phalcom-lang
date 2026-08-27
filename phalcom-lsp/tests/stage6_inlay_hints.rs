//! Standard LSP inlay-hint integration coverage.

use serde_json::json;

use crate::support::TestLsp;

#[tokio::test]
async fn inlay_hint_returns_runtime_value_for_literal_binding() {
    let mut lsp = TestLsp::start().await;
    let init = lsp.initialize(None).await;
    assert!(init["result"]["capabilities"]["inlayHintProvider"].is_object(), "{init:#?}");
    assert_eq!(init["result"]["capabilities"]["inlayHintProvider"]["resolveProvider"], json!(false));

    let uri = "file:///workspace/main.ph";
    lsp.open_and_wait(uri, "let text = \"hello\"\n").await;

    let response = lsp.inlay_hints(uri, 1).await;
    let hints = response["result"].as_array().expect("inlay hint array");
    assert_eq!(hints.len(), 1);
    assert_eq!(hints[0]["label"], json!(": String"));

    lsp.finish().await;
}

#[tokio::test]
async fn inlay_hint_skips_explicit_binding_annotation() {
    let mut lsp = TestLsp::start().await;
    lsp.initialize(None).await;

    let uri = "file:///workspace/annotated.ph";
    lsp.open_and_wait(uri, "let annotated: Int = 1\nlet inferred = 2\n").await;

    let response = lsp.inlay_hints(uri, 2).await;
    let hints = response["result"].as_array().expect("inlay hint array");
    assert_eq!(hints.len(), 1, "only unannotated binding should receive a hint: {response:#?}");
    assert_eq!(hints[0]["label"], json!(": Int"));

    lsp.finish().await;
}
