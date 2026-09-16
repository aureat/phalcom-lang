//! Stage 3 server integration test: drives [`phalcom_lsp::Backend`] through
//! `tower-lsp`'s real JSON-RPC transport over an in-process
//! [`tokio::io::duplex`] pipe (mirrors `tests/stage2_index.rs`'s transport
//! harness) — no subprocess spawn.
//!
//! Exercises `initialize` (advertising `completionProvider`) → `initialized`
//! → `didOpen` (a file defining a class, constructing an instance of it, and a
//! `receiver.` member-access line) → `textDocument/completion`, asserting the
//! receiver-aware completion surface contains the class's own selectors and
//! not an unrelated builtin (`docs/forge/units/U-LSP/plan.md` § Tests /
//! verification, Stage 3).

use serde_json::json;
use tower_lsp::lsp_types::Position;

use crate::support::{TestLsp, TestWorkspace, completion_labels};

#[tokio::test]
async fn completion_is_receiver_aware_for_a_constructed_user_class() {
    let mut lsp = TestLsp::start().await;

    let init = lsp.initialize(None).await;
    assert!(init["result"]["capabilities"]["completionProvider"].is_object(), "{init:#?}");

    let uri = "file:///workspace/main.ph";
    let text = concat!(
        "class Mover {\n",
        "  @constructor new() {}\n",
        "  move(_ x, to) { }\n",
        "  speed { }\n",
        "}\n",
        "let m = Mover.new();\n",
        "m.\n",
    );

    lsp.open_and_wait(uri, text).await;

    let response = lsp.completion(uri, Position { line: 6, character: 2 }).await;

    let items = response["result"].as_array().expect("completion items array");

    let labels = completion_labels(&response);

    assert!(labels.iter().any(|label| label == "move(_,to)"), "{labels:#?}");
    assert!(labels.iter().any(|label| label == "speed"), "{labels:#?}");
    assert!(!labels.iter().any(|label| label == "ifTrue(_)"), "{labels:#?}");

    let move_item = items.iter().find(|item| item["label"] == json!("move(_,to)")).expect("move item present");

    assert_eq!(move_item["insertText"], json!("move(${1:_}, to: ${2:_})"));

    for item in items {
        let rendered = item.to_string();
        assert!(!rendered.contains('≈'));
        assert!(!rendered.contains("Confidence"));
        assert!(!rendered.contains("Observed"));
    }

    lsp.finish().await;
}

#[tokio::test]
async fn completion_follows_constructor_assigned_field() {
    let mut lsp = TestLsp::start().await;
    lsp.initialize(None).await;

    let uri = "file:///workspace/service.ph";
    let text = concat!(
        "class Client {\n",
        "  @constructor new() {}\n",
        "  send() { }\n",
        "}\n",
        "class Service {\n",
        "  @constructor new() { _client = Client.new() }\n",
        "  run() {\n",
        "    _client.\n",
        "  }\n",
        "}\n",
    );

    lsp.open_and_wait(uri, text).await;

    let response = lsp.completion(uri, Position { line: 7, character: 12 }).await;

    let labels = completion_labels(&response);

    assert!(labels.iter().any(|label| label == "send()"), "{labels:#?}");
    assert!(!labels.iter().any(|label| label == "ifTrue(_)"), "{labels:#?}");

    lsp.finish().await;
}

#[tokio::test]
async fn completion_marks_partial_union_coverage() {
    let uri = "file:///workspace/union.ph";
    let text = "class Circle {\n  @constructor new() {}\n  stroke() { }\n}\nclass Rectangle {\n  @constructor new() {}\n  fill() { }\n}\nclass Canvas {\n  @constructor new() {}\n  draw(_ shape) {\n    shape.\n  }\n}\nCanvas.new().draw(Circle.new())\nCanvas.new().draw(Rectangle.new())\n";

    let mut lsp = TestLsp::start().await;
    lsp.initialize(None).await;
    lsp.open_and_wait(uri, text).await;

    let response = lsp.completion(uri, Position::new(11, 10)).await;
    let items = response["result"].as_array().expect("completion items array");
    let stroke = items.iter().find(|item| item["label"] == "stroke()").expect("Circle member present");
    let fill = items.iter().find(|item| item["label"] == "fill()").expect("Rectangle member present");
    assert!(stroke["detail"].as_str().unwrap_or_default().contains("1/2"));
    assert!(fill["detail"].as_str().unwrap_or_default().contains("1/2"));
    let shared = items.iter().find(|item| item["label"] == "==(_)").expect("shared Object member present");
    assert!(shared["detail"].as_str().unwrap_or_default().contains("2/2"));

    lsp.finish().await;
}

#[tokio::test]
async fn import_completion_uses_published_module_queries() {
    let workspace = TestWorkspace::from_fixture_dir("import_completion");
    let uri = workspace.file_uri("main.ph");
    let text = "import .a as A\nimport .b as B\n";
    workspace.write("main.ph", text);

    let mut lsp = TestLsp::start().await;
    let before = lsp.counter_snapshot();
    lsp.initialize(Some(&workspace.uri())).await;
    lsp.open_and_wait(&uri, text).await;
    lsp.wait_for_publication_after(before).await;

    let response = lsp.completion(&uri, Position::new(1, 8)).await;
    let labels = completion_labels(&response);
    assert!(
        labels.iter().any(|label| label == "a"),
        "relative import completion must expose module a: {labels:#?}"
    );
    assert!(
        labels.iter().any(|label| label == "b"),
        "relative import completion must expose module b: {labels:#?}"
    );

    lsp.finish().await;
}
