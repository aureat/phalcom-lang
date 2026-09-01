use phalcom_lsp::line_index::LineIndex;
use phalcom_modules::{UniverseSourceProvider, universe_module_from_uri};
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
async fn canonical_universe_source_text_matches_compiler_presentation_publication() {
    let mut lsp = TestLsp::start().await;
    let initialize = lsp.initialize(None).await;
    assert!(initialize.get("result").is_some(), "server initializes before source request");

    lsp.open_and_wait("file:///core-presentation-probe.ph", "class Probe {}\n").await;

    let uri = "phalcom://universe/object/object";
    let response = lsp.request("phalcom/sourceText", json!({ "uri": uri })).await;
    let actual = response["result"].as_str().expect("canonical Universe source provider returns text");
    let module = universe_module_from_uri(uri).expect("canonical Universe URI");
    let expected = UniverseSourceProvider::new().source_text(&module).expect("canonical Universe source exists");
    assert_eq!(actual, expected.as_ref(), "the editor must receive the exact canonical Universe source text");
    lsp.finish().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn builtin_definition_range_indexes_canonical_universe_presentation() {
    let mut lsp = TestLsp::start().await;
    let initialize = lsp.initialize(None).await;
    assert!(initialize.get("result").is_some(), "server initializes before definition request");

    let uri = "file:///core-definition-range-probe.ph";
    let int_source_uri = "phalcom://universe/scalar/number";

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

    assert_eq!(location["uri"].as_str(), Some(int_source_uri));

    let source = lsp.request("phalcom/sourceText", json!({ "uri": int_source_uri })).await;

    let int_source = source["result"].as_str().expect("canonical Int source is available");

    let range: tower_lsp::lsp_types::Range = serde_json::from_value(location["range"].clone()).expect("definition range is valid LSP data");

    let index = LineIndex::new(int_source);
    let start = index.offset(range.start);
    let end = index.offset(range.end);

    assert_eq!(
        int_source.get(start..end),
        Some("Int"),
        "definition range must select Int in canonical scalar.number source"
    );

    lsp.finish().await;
}
