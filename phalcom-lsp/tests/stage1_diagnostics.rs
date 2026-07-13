//! Stage 1 server integration test: drives [`phalcom_lsp::Backend`] through
//! `tower-lsp`'s real JSON-RPC transport over an in-process
//! [`tokio::io::duplex`] pipe — no subprocess spawn.
//!
//! Exercises `initialize` → `initialized` → `didOpen` and asserts the
//! resulting `textDocument/publishDiagnostics` notification carries exactly
//! **2** diagnostics (the fixture's 2 recovered syntax errors), each at the
//! position [`phalcom_lsp::line_index::LineIndex`] independently computes —
//! the plan's "golden JSON-RPC" tier for Stage 1
//! (`docs/forge/units/U-LSP/plan.md` § Tests / verification).

use phalcom_lsp::Backend;
use phalcom_lsp::line_index::LineIndex;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower_lsp::{LspService, Server};

/// Writes one JSON-RPC message to `w` using the LSP `Content-Length` framing
/// (`tower_lsp`'s `LanguageServerCodec` wire format).
async fn write_message(w: &mut (impl AsyncWriteExt + Unpin), value: &Value) {
    let body = serde_json::to_string(value).unwrap();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    w.write_all(header.as_bytes()).await.unwrap();
    w.write_all(body.as_bytes()).await.unwrap();
}

/// Reads one JSON-RPC message from `r`, parsing the `Content-Length` header
/// then the JSON body it announces.
async fn read_message(r: &mut (impl AsyncReadExt + Unpin)) -> Value {
    // Read the header line-by-line (terminated by "\r\n\r\n") one byte at a
    // time — simple and adequate for a test harness reading small,
    // well-formed frames.
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

/// Reads messages from `r` until one whose `"method"` field equals `method`,
/// returning that message. Bounded to avoid hanging the test if the server
/// never sends it.
async fn read_until_method(r: &mut (impl AsyncReadExt + Unpin), method: &str) -> Value {
    for _ in 0..16 {
        let msg = read_message(r).await;
        if msg.get("method").and_then(Value::as_str) == Some(method) {
            return msg;
        }
    }
    panic!("did not observe a {method:?} message within the read budget");
}

#[tokio::test]
async fn did_open_with_two_syntax_errors_publishes_two_diagnostics() {
    // One in-process duplex pipe stands in for the stdio transport. The
    // server's end is split into independent read/write halves (its
    // `stdin`/`stdout`); the client's end is used directly to write requests
    // and read responses/notifications.
    let (server_end, mut client_end) = tokio::io::duplex(1 << 16);
    let (server_read, server_write) = tokio::io::split(server_end);

    let (service, socket) = LspService::new(Backend::new);
    let server_task = tokio::spawn(async move {
        Server::new(server_read, server_write, socket)
            .serve(service)
            .await;
    });

    // initialize
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "processId": null, "rootUri": null, "capabilities": {} }
        }),
    )
    .await;
    let init_response = read_message(&mut client_end).await;
    assert_eq!(init_response["id"], json!(1));
    assert_eq!(
        init_response["result"]["capabilities"]["positionEncoding"],
        json!("utf-16")
    );

    // initialized
    write_message(
        &mut client_end,
        &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }),
    )
    .await;

    // A fixture with exactly 2 recovered syntax errors (mirrors
    // `diagnostics::tests::n_syntax_errors_yield_n_diagnostics`).
    let source = "let = \nlet = \n";

    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///test.ph",
                    "languageId": "phalcom",
                    "version": 1,
                    "text": source,
                }
            }
        }),
    )
    .await;

    let publish = read_until_method(&mut client_end, "textDocument/publishDiagnostics").await;
    assert_eq!(publish["params"]["uri"], json!("file:///test.ph"));
    let diagnostics = publish["params"]["diagnostics"]
        .as_array()
        .expect("diagnostics array");
    assert_eq!(diagnostics.len(), 2, "expected 2 diagnostics, got {diagnostics:#?}");

    // Cross-check each diagnostic's range against an independently built
    // `LineIndex` over the same source, using the same underlying parse to
    // locate each error's byte range.
    let index = LineIndex::new(source);
    let parsed = phalcom_ast::parser::parse(source, 0);
    assert_eq!(parsed.errors.len(), 2);
    for (diag, err) in diagnostics.iter().zip(parsed.errors.iter()) {
        let expected = index.range(err.range.clone());
        let expected_json = json!({
            "start": { "line": expected.start.line, "character": expected.start.character },
            "end": { "line": expected.end.line, "character": expected.end.character },
        });
        assert_eq!(diag["range"], expected_json);
        assert_eq!(diag["message"], json!(err.kind.to_string()));
    }

    drop(client_end);
    let _ = server_task.await;
}
