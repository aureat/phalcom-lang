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
use tower_lsp::{LspService, Server};

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

#[tokio::test]
async fn completion_is_receiver_aware_for_a_constructed_user_class() {
    let (server_end, mut client_end) = tokio::io::duplex(1 << 16);
    let (server_read, server_write) = tokio::io::split(server_end);

    let (service, socket) = LspService::new(Backend::new);
    let server_task = tokio::spawn(async move {
        Server::new(server_read, server_write, socket)
            .serve(service)
            .await;
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
    assert!(
        init_response["result"]["capabilities"]["completionProvider"].is_object(),
        "{init_response:#?}"
    );
    assert_eq!(
        init_response["result"]["capabilities"]["completionProvider"]["triggerCharacters"],
        json!(["."])
    );

    write_message(
        &mut client_end,
        &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }),
    )
    .await;

    // `Mover` defines `move(_,to)` and getter `speed`; `m` is constructed from
    // it; the last line is the `m.` member access the completion targets.
    let uri = "file:///workspace/main.ph";
    let text = "class Mover {\n  move(x, to:) { }\n  speed { }\n}\nlet m = Mover.new();\nm.\n";
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
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/completion",
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 5, "character": 2 }
            }
        }),
    )
    .await;
    let response = read_response(&mut client_end, 2).await;
    let items = response["result"].as_array().expect("completion items array");
    let labels: Vec<&str> = items
        .iter()
        .filter_map(|item| item["label"].as_str())
        .collect();

    assert!(labels.contains(&"move(_,to)"), "{labels:#?}");
    assert!(labels.contains(&"speed"), "{labels:#?}");
    // A resolved user receiver must not spill the full builtin surface.
    assert!(!labels.contains(&"ifTrue(_)"), "{labels:#?}");

    // The method item carries a snippet insert text with tab-stops.
    let move_item = items
        .iter()
        .find(|item| item["label"] == json!("move(_,to)"))
        .expect("move item present");
    assert_eq!(
        move_item["insertText"],
        json!("move(${1:_}, to: ${2:_})"),
        "{move_item:#?}"
    );

    drop(client_end);
    let _ = server_task.await;
}
