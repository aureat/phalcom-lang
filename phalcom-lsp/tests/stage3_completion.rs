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

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{Duration, sleep};
use tower_lsp::lsp_types::Position;
use tower_lsp::{LspService, Server};

use crate::support::{TestLsp, TestWorkspace, completion_labels};
use phalcom_lsp::Backend;

/// Writes one JSON-RPC message to `w` using the LSP `Content-Length` framing.
async fn write_message(w: &mut (impl AsyncWriteExt + Unpin), value: &Value) {
    let body = serde_json::to_string(value).unwrap();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    w.write_all(header.as_bytes()).await.unwrap();
    w.write_all(body.as_bytes()).await.unwrap();
}

/// Reads one JSON-RPC message from `r`, parsing the `Content-Length` header
/// then the JSON body it announces.
async fn read_message(r: &mut (impl AsyncReadExt + Unpin)) -> Value {
    let mut header = Vec::new();
    loop {
        let mut byte = [0u8; 1];
        r.read_exact(&mut byte).await.unwrap();
        header.push(byte[0]);
        if header.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let header_str = String::from_utf8(header).unwrap();
    let content_length: usize = header_str
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .expect("Content-Length header present")
        .trim()
        .parse()
        .unwrap();
    let mut body = vec![0u8; content_length];
    r.read_exact(&mut body).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

/// Reads messages from `r`, discarding notifications, until one whose `"id"`
/// equals `id`. Bounded to avoid hanging if the server never responds.
async fn read_response(r: &mut (impl AsyncReadExt + Unpin), id: i64) -> Value {
    for _ in 0..32 {
        let msg = read_message(r).await;
        if msg.get("id").and_then(Value::as_i64) == Some(id) {
            return msg;
        }
    }
    panic!("did not observe a response to id {id} within the read budget");
}

async fn completion_items(client_end: &mut (impl AsyncReadExt + AsyncWriteExt + Unpin), uri: &str, line: u32, character: u32) -> Value {
    for id in 2..=32 {
        write_message(
            client_end,
            &json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "textDocument/completion",
                "params": {
                    "textDocument": { "uri": uri },
                    "position": { "line": line, "character": character }
                }
            }),
        )
        .await;
        let response = read_response(client_end, id).await;
        if response["result"].as_array().is_some_and(|items| !items.is_empty()) {
            return response;
        }
        sleep(Duration::from_millis(2)).await;
    }
    panic!("compiler publication did not produce completion items");
}

#[tokio::test]
async fn completion_is_receiver_aware_for_a_constructed_user_class() {
    let (server_end, mut client_end) = tokio::io::duplex(1 << 16);
    let (server_read, server_write) = tokio::io::split(server_end);

    let (service, socket) = LspService::new(Backend::new);
    let server_task = tokio::spawn(async move {
        Server::new(server_read, server_write, socket).serve(service).await;
    });

    // No workspace root: this test drives the index purely through `didOpen`,
    // proving completion works off the live buffer.
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "processId": null, "capabilities": {} }
        }),
    )
    .await;
    let init_response = read_response(&mut client_end, 1).await;
    assert!(init_response["result"]["capabilities"]["completionProvider"].is_object(), "{init_response:#?}");
    assert_eq!(init_response["result"]["capabilities"]["completionProvider"]["triggerCharacters"], json!(["."]));

    write_message(&mut client_end, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await;

    // `Mover` defines `move(_,to)` and getter `speed`; `m` is constructed from
    // it; the last line is the `m.` member access the completion targets.
    let uri = "file:///workspace/main.ph";
    let text = "class Mover {\n  @constructor new() {}\n  move(_ x, to) { }\n  speed { }\n}\nlet m = Mover.new();\nm.\n";
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "phalcom",
                    "version": 1,
                    "text": text,
                }
            }
        }),
    )
    .await;
    // Drain the didOpen's publishDiagnostics notification.
    let _ = read_message(&mut client_end).await;

    // Completion at the end of the `m.` line (line 5, character 2).
    let response = completion_items(&mut client_end, uri, 6, 2).await;
    let items = response["result"].as_array().expect("completion items array");
    let labels: Vec<&str> = items.iter().filter_map(|item| item["label"].as_str()).collect();

    for item in items {
        let rendered = item.to_string();
        assert!(!rendered.contains('≈'), "completion item must not expose advisory decoration: {item}");
        assert!(!rendered.contains("Confidence"), "completion item must not expose confidence taxonomy: {item}");
        assert!(
            !rendered.contains("Observed"),
            "completion item must not expose observed-value boilerplate: {item}"
        );
    }

    assert!(labels.contains(&"move(_,to)"), "{labels:#?}");
    assert!(labels.contains(&"speed"), "{labels:#?}");
    // A resolved user receiver must not spill the full builtin surface.
    assert!(!labels.contains(&"ifTrue(_)"), "{labels:#?}");

    // The method item carries a snippet insert text with tab-stops.
    let move_item = items.iter().find(|item| item["label"] == json!("move(_,to)")).expect("move item present");
    assert_eq!(move_item["insertText"], json!("move(${1:_}, to: ${2:_})"), "{move_item:#?}");

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn completion_follows_constructor_assigned_field() {
    let (server_end, mut client_end) = tokio::io::duplex(1 << 16);
    let (server_read, server_write) = tokio::io::split(server_end);
    let (service, socket) = LspService::new(Backend::new);
    let server_task = tokio::spawn(async move {
        Server::new(server_read, server_write, socket).serve(service).await;
    });

    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "processId": null, "capabilities": {} }
        }),
    )
    .await;
    let _ = read_response(&mut client_end, 1).await;
    write_message(&mut client_end, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await;

    let uri = "file:///workspace/service.ph";
    let text = "class Client {\n  @constructor new() {}\n  send() { }\n}\nclass Service {\n  @constructor new() { _client = Client.new() }\n  run() {\n    _client.\n  }\n}\n";
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": { "uri": uri, "languageId": "phalcom", "version": 1, "text": text } }
        }),
    )
    .await;
    let _ = read_message(&mut client_end).await;

    let response = completion_items(&mut client_end, uri, 7, 12).await;
    let items = response["result"].as_array().expect("completion items array");
    let labels: Vec<&str> = items.iter().filter_map(|item| item["label"].as_str()).collect();
    assert!(labels.contains(&"send()"), "{labels:#?}");
    assert!(!labels.contains(&"ifTrue(_)"), "{labels:#?}");

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn completion_marks_partial_union_coverage() {
    let (server_end, mut client_end) = tokio::io::duplex(1 << 16);
    let (server_read, server_write) = tokio::io::split(server_end);
    let (service, socket) = LspService::new(Backend::new);
    let server_task = tokio::spawn(async move {
        Server::new(server_read, server_write, socket).serve(service).await;
    });

    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "processId": null, "capabilities": {} }
        }),
    )
    .await;
    let _ = read_response(&mut client_end, 1).await;
    write_message(&mut client_end, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await;

    let uri = "file:///workspace/union.ph";
    let text = "class Circle {\n  @constructor new() {}\n  stroke() { }\n}\nclass Rectangle {\n  @constructor new() {}\n  fill() { }\n}\nclass Canvas {\n  @constructor new() {}\n  draw(_ shape) {\n    shape.\n  }\n}\nCanvas.new().draw(Circle.new())\nCanvas.new().draw(Rectangle.new())\n";
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": { "textDocument": { "uri": uri, "languageId": "phalcom", "version": 1, "text": text } }
        }),
    )
    .await;
    let _ = read_message(&mut client_end).await;

    let response = completion_items(&mut client_end, uri, 11, 10).await;
    let items = response["result"].as_array().expect("completion items array");
    let stroke = items.iter().find(|item| item["label"] == "stroke()").expect("Circle member present");
    let fill = items.iter().find(|item| item["label"] == "fill()").expect("Rectangle member present");
    assert!(stroke["detail"].as_str().unwrap_or_default().contains("1/2"));
    assert!(fill["detail"].as_str().unwrap_or_default().contains("1/2"));
    let shared = items.iter().find(|item| item["label"] == "==(_)").expect("shared Object member present");
    assert!(shared["detail"].as_str().unwrap_or_default().contains("2/2"));

    drop(client_end);
    let _ = server_task.await;
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
