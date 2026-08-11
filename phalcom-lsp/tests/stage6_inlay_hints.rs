//! Standard LSP inlay-hint integration coverage.

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower_lsp::{LspService, Server};

use phalcom_lsp::Backend;

async fn write_message(w: &mut (impl AsyncWriteExt + Unpin), value: &Value) {
    let body = serde_json::to_string(value).unwrap();
    w.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes()).await.unwrap();
    w.write_all(body.as_bytes()).await.unwrap();
}

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
    let header = String::from_utf8(header).unwrap();
    let length = header
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length: "))
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let mut body = vec![0; length];
    r.read_exact(&mut body).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

async fn read_response(r: &mut (impl AsyncReadExt + Unpin), id: i64) -> Value {
    loop {
        let message = read_message(r).await;
        if message.get("id").and_then(Value::as_i64) == Some(id) {
            return message;
        }
    }
}

#[tokio::test]
async fn inlay_hint_returns_runtime_value_for_literal_binding() {
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
    let init = read_response(&mut client_end, 1).await;
    assert!(init["result"]["capabilities"]["inlayHintProvider"].is_object(), "{init:#?}");

    write_message(&mut client_end, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await;
    let uri = "file:///workspace/main.ph";
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
                    "text": "let text = \"hello\"\n"
                }
            }
        }),
    )
    .await;
    let _diagnostics = read_message(&mut client_end).await;

    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/inlayHint",
            "params": {
                "textDocument": { "uri": uri },
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 1, "character": 0 } }
            }
        }),
    )
    .await;
    let response = read_response(&mut client_end, 2).await;
    let hints = response["result"].as_array().expect("inlay hint array");
    assert_eq!(hints.len(), 1);
    assert_eq!(hints[0]["label"], json!(": String"));

    drop(client_end);
    let _ = server_task.await;
}
