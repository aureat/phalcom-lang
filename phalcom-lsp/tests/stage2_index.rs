//! Stage 2 server integration test: drives [`phalcom_lsp::Backend`] through
//! `tower-lsp`'s real JSON-RPC transport over an in-process
//! [`tokio::io::duplex`] pipe (mirrors `tests/stage1_diagnostics.rs`'s
//! transport harness) — no subprocess spawn.
//!
//! Exercises `initialize` (with a scratch workspace root on disk, so Stage
//! 2's `initialize`-time scan has something to index) →
//! `initialized` → two `didOpen`s (one defining a method, one calling it)
//! → `textDocument/definition` and `workspace/symbol` — the plan's "golden
//! JSON-RPC" tier for Stage 2 (`docs/forge/units/U-LSP/plan.md` § Tests /
//! verification).

use std::path::PathBuf;

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower_lsp::{LspService, Server};

use phalcom_lsp::Backend;

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

/// Reads messages from `r`, discarding notifications, until one whose
/// `"id"` field equals `id` — i.e. the response to a specific request.
/// Bounded to avoid hanging the test if the server never responds.
async fn read_response(r: &mut (impl AsyncReadExt + Unpin), id: i64) -> Value {
    for _ in 0..32 {
        let msg = read_message(r).await;
        if msg.get("id").and_then(Value::as_i64) == Some(id) {
            return msg;
        }
    }
    panic!("did not observe a response to id {id} within the read budget");
}

/// A scratch directory on disk holding a small multi-file `.ph` fixture, so
/// `initialize`'s workspace scan has real files to walk. Removed on drop.
struct ScratchWorkspace {
    root: PathBuf,
}

impl ScratchWorkspace {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "phalcom-lsp-stage2-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.root.join(relative);
        std::fs::write(&path, contents).unwrap();
        path
    }

    fn uri(&self) -> String {
        url::Url::from_directory_path(&self.root).unwrap().to_string()
    }
}

impl Drop for ScratchWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// `Url::from_file_path` needs an absolute path; a tiny re-export so this
/// test doesn't need its own `url` dependency entry beyond what
/// `tower-lsp`'s `lsp-types` already pulls in transitively.
mod url {
    pub use tower_lsp::lsp_types::Url;
}

#[tokio::test]
async fn goto_definition_and_workspace_symbol_resolve_across_files() {
    let workspace = ScratchWorkspace::new("defs");
    // `Mover` defines `move(_,to,duration)`; `main.ph` calls it. Both are
    // on disk before `initialize` runs, so the startup scan indexes both.
    let def_path = workspace.write(
        "mover.ph",
        "class Mover {\n  move(x, to:, duration:) { }\n}\n",
    );
    workspace.write(
        "main.ph",
        "let m = Mover.new();\nm.move(1, to: 2, duration: 3);\n",
    );

    let (server_end, mut client_end) = tokio::io::duplex(1 << 16);
    let (server_read, server_write) = tokio::io::split(server_end);

    let (service, socket) = LspService::new(Backend::new);
    let server_task = tokio::spawn(async move {
        Server::new(server_read, server_write, socket)
            .serve(service)
            .await;
    });

    // initialize, rooted at the scratch workspace so the startup scan picks
    // up both fixture files.
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": null,
                "rootUri": workspace.uri(),
                "capabilities": {}
            }
        }),
    )
    .await;
    let init_response = read_response(&mut client_end, 1).await;
    assert_eq!(
        init_response["result"]["capabilities"]["definitionProvider"],
        json!(true)
    );
    assert_eq!(
        init_response["result"]["capabilities"]["referencesProvider"],
        json!(true)
    );
    assert_eq!(
        init_response["result"]["capabilities"]["workspaceSymbolProvider"],
        json!(true)
    );

    write_message(
        &mut client_end,
        &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }),
    )
    .await;

    // Open the call-site file (not strictly required — the scan already
    // indexed it from disk — but mirrors a real editor session and proves
    // did_open doesn't disturb the cross-file index).
    let main_text = std::fs::read_to_string(workspace.root.join("main.ph")).unwrap();
    let main_uri = url::Url::from_file_path(workspace.root.join("main.ph"))
        .unwrap()
        .to_string();
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": main_uri,
                    "languageId": "phalcom",
                    "version": 1,
                    "text": main_text,
                }
            }
        }),
    )
    .await;
    // Drain the didOpen's publishDiagnostics notification before issuing
    // the next request, so it doesn't get mistaken for a request response
    // by `read_response`'s scan (it has no "id" field, so it's safely
    // skipped, but draining keeps the stream tidy).
    let _ = read_message(&mut client_end).await;

    // textDocument/definition from the call site (`m.move(...)` in
    // main.ph) must resolve to `move`'s declaration in mover.ph.
    let call_site_offset = main_text.find("move").unwrap();
    let call_site_line = main_text[..call_site_offset].matches('\n').count();
    let call_site_col = call_site_offset
        - main_text[..call_site_offset]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);

    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": { "uri": main_uri },
                "position": { "line": call_site_line, "character": call_site_col }
            }
        }),
    )
    .await;
    let def_response = read_response(&mut client_end, 2).await;
    let locations = def_response["result"].as_array().expect("locations array");
    assert_eq!(locations.len(), 1, "{def_response:#?}");
    let expected_def_uri = url::Url::from_file_path(&def_path).unwrap().to_string();
    assert_eq!(locations[0]["uri"], json!(expected_def_uri));

    // workspace/symbol for "move" must find the declaration.
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "workspace/symbol",
            "params": { "query": "move" }
        }),
    )
    .await;
    let symbol_response = read_response(&mut client_end, 3).await;
    let symbols = symbol_response["result"].as_array().expect("symbols array");
    assert!(
        symbols
            .iter()
            .any(|s| s["name"] == json!("move(_,to,duration)")),
        "{symbols:#?}"
    );

    drop(client_end);
    let _ = server_task.await;
}
