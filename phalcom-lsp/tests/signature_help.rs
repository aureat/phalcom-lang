use serde_json::Value;
use tower_lsp::lsp_types::Position;

use crate::support::TestLsp;

#[tokio::test]
async fn signature_help_recovers_incomplete_receiver_call() {
    let mut client = TestLsp::start().await;
    let init = client.initialize(None).await;
    assert!(init["result"]["capabilities"]["signatureHelpProvider"].is_object(), "{init:#?}");

    let uri = "file:///signature-help.ph";
    let source = "class Service { compute(_ x: Int, label y: Int) -> Int { x } run() { self.compute(1) } }\n";
    client.open_and_wait(uri, source).await;
    let position = Position::new(0, source.find("self.compute(").unwrap() as u32 + "self.compute(".len() as u32);
    let response = client.signature_help(uri, position).await;
    let signatures = response["result"]["signatures"].as_array().unwrap_or_else(|| panic!("{response:#?}"));
    assert!(!signatures.is_empty(), "signature help must resolve incomplete receiver call: {response:#?}");
    assert!(signatures[0]["label"].as_str().is_some_and(|label| label.contains("compute")));
    assert_eq!(signatures[0]["parameters"].as_array().map(Vec::len), Some(2));
    assert!(
        signatures[0]["label"].as_str().is_some_and(|label| label.contains("Int")),
        "compiler signature should retain formal parameter/return types: {response:#?}"
    );
    assert_eq!(response["result"]["activeParameter"], Value::from(0));

    client.finish().await;
}
