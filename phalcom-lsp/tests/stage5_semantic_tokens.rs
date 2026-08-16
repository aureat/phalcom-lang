//! Stage 5 server integration test: drives [`phalcom_lsp::Backend`] through
//! `tower-lsp`'s real JSON-RPC transport over an in-process
//! [`tokio::io::duplex`] pipe (mirrors `tests/stage3_completion.rs`'s
//! transport harness) — no subprocess spawn.
//!
//! Exercises `initialize` (advertising `semanticTokensProvider` with the
//! Stage 5 legend) → `initialized` → `didOpen` → `textDocument/
//! semanticTokens/full`, asserting the returned token stream classifies a
//! representative source snippet (`docs/forge/units/U-LSP/plan.md` § Tests /
//! verification, Stage 5).

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
async fn semantic_tokens_full_classifies_a_representative_document() {
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
    let init_response = read_response(&mut client_end, 1).await;
    let provider = &init_response["result"]["capabilities"]["semanticTokensProvider"];
    assert!(provider.is_object(), "{init_response:#?}");
    assert_eq!(provider["full"], json!(true));
    let token_types = provider["legend"]["tokenTypes"].as_array().expect("legend token types array");
    assert!(token_types.iter().any(|t| t == "selector"), "{token_types:#?}");

    write_message(&mut client_end, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await;

    let uri = "file:///workspace/main.ph";
    let text = "let x = 1\n#move\n";
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

    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/semanticTokens/full",
            "params": {
                "textDocument": { "uri": uri }
            }
        }),
    )
    .await;
    let response = read_response(&mut client_end, 2).await;
    let data = response["result"]["data"].as_array().expect("semantic token data array");

    // 5 fields per token; classified tokens are: let, x, =, 1, #, move.
    // (Newline is uncolored). Selector components retain their lexer token
    // boundaries, so the hash and name are emitted separately.
    assert_eq!(data.len(), 30, "{data:#?}");

    // First token: `let` at (0,0), length 3, token_type 0 (keyword).
    assert_eq!(data[0], json!(0)); // delta_line
    assert_eq!(data[1], json!(0)); // delta_start
    assert_eq!(data[2], json!(3)); // length
    assert_eq!(data[3], json!(0)); // token_type index (keyword)

    // Selector prefix: `#` on line 1, token_type 4 (selector).
    let selector_prefix = &data[20..25];
    assert_eq!(selector_prefix[3], json!(4));

    // Selector base: `move` on line 1, token_type 7 (method / member reference) or 1 (variable in flat pass).
    let selector_base = &data[25..30];
    assert!(selector_base[3] == json!(1) || selector_base[3] == json!(7));

    drop(client_end);
    let _ = server_task.await;
}
