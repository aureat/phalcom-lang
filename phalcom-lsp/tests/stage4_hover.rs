//! Stage 4 server integration test: drives [`phalcom_lsp::Backend`] through
//! `tower-lsp`'s real JSON-RPC transport over an in-process
//! [`tokio::io::duplex`] pipe (mirrors `tests/stage3_completion.rs`'s
//! transport harness) — no subprocess spawn.
//!
//! Exercises `textDocument/hover` (`docs/forge/units/U-LSP/plan.md` § Tests,
//! Stage 4):
//! - a keyword hover (`self`),
//! - Phaldoc adjacency (a `///` block immediately above a method attaches;
//!   one separated by a blank line does not),
//! - selector-keying (`foo()`/`foo(_)` each carry their own doc),
//! - cross-file resolution (hovering a call site in one file resolves the
//!   doc written on the declaration in another),
//! - builtin hover (a core-table selector renders kind + selector with no
//!   Phaldoc section).

use std::path::PathBuf;

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

/// A scratch directory on disk holding a small multi-file `.ph` fixture, so
/// `initialize`'s workspace scan has real files to walk. Removed on drop.
struct ScratchWorkspace {
    root: PathBuf,
}

impl ScratchWorkspace {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "phalcom-lsp-stage4-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
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

/// Spawns a `Backend` server bound to an in-process duplex pipe, returning
/// the client-side end of the pipe and the server task handle.
fn spawn_server() -> (tokio::io::DuplexStream, tokio::task::JoinHandle<()>) {
    let (server_end, client_end) = tokio::io::duplex(1 << 16);
    let (server_read, server_write) = tokio::io::split(server_end);
    let (service, socket) = LspService::new(Backend::new);
    let server_task = tokio::spawn(async move {
        Server::new(server_read, server_write, socket).serve(service).await;
    });
    (client_end, server_task)
}

async fn initialize(client_end: &mut tokio::io::DuplexStream, root_uri: Option<&str>) {
    let mut params = json!({ "processId": null, "capabilities": {} });
    if let Some(root) = root_uri {
        params["rootUri"] = json!(root);
    }
    write_message(client_end, &json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": params })).await;
    let init_response = read_response(client_end, 1).await;
    assert_eq!(init_response["result"]["capabilities"]["hoverProvider"], json!(true), "{init_response:#?}");
    write_message(client_end, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await;
}

async fn did_open(client_end: &mut tokio::io::DuplexStream, uri: &str, text: &str) {
    write_message(
        client_end,
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
    let _ = read_message(client_end).await;
}

/// `line`/`character` (0-based, UTF-16) of the first occurrence of `needle`
/// in `text`, for hover requests — every fixture here is pure ASCII, so byte
/// offset and UTF-16 code-unit offset coincide.
fn position_of(text: &str, needle: &str) -> (usize, usize) {
    let offset = text.find(needle).unwrap();
    let line = text[..offset].matches('\n').count();
    let col = offset - text[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    (line, col)
}

async fn hover_at(client_end: &mut tokio::io::DuplexStream, id: i64, uri: &str, text: &str, needle: &str) -> Value {
    let (line, character) = position_of(text, needle);
    write_message(
        client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character }
            }
        }),
    )
    .await;
    read_response(client_end, id).await
}

#[tokio::test]
async fn keyword_hover_renders_the_blurb() {
    let (mut client_end, server_task) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    let text = "class Point {\n  x() { self }\n}\n";
    did_open(&mut client_end, uri, text).await;

    let response = hover_at(&mut client_end, 2, uri, text, "self").await;
    let value = response["result"]["contents"]["value"].as_str().expect("markup contents");
    assert!(value.contains("**self**"), "{value:?}");
    assert!(value.contains("current receiver"), "{value:?}");

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn phaldoc_adjacency_attaches_and_blank_line_breaks_it() {
    let (mut client_end, server_task) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    let text = "class Point {\n  /// Moves the point.\n  move(x) { }\n\n  /// Detached, not adjacent.\n\n  reset() { }\n}\n";
    did_open(&mut client_end, uri, text).await;

    // Adjacent doc attaches.
    let response = hover_at(&mut client_end, 2, uri, text, "move(x)").await;
    let value = response["result"]["contents"]["value"].as_str().expect("markup contents");
    assert!(value.contains("Moves the point."), "{value:?}");
    assert!(value.contains("move(_)"), "{value:?}");

    // A `///` block separated from its target by a blank line is dangling —
    // `reset()` still gets a signature hover (it IS a real definition), but
    // no Phaldoc summary section.
    let response = hover_at(&mut client_end, 3, uri, text, "reset()").await;
    let value = response["result"]["contents"]["value"].as_str().expect("markup contents");
    assert!(value.contains("reset()"), "{value:?}");
    assert!(!value.contains("Detached"), "{value:?}");

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn selector_keying_gives_each_arity_its_own_doc() {
    let (mut client_end, server_task) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    let text = "class Greeter {\n  /// Zero-arg greeting.\n  greet() { }\n  /// Greeting with a name.\n  greet(name) { }\n}\n";
    did_open(&mut client_end, uri, text).await;

    let zero = hover_at(&mut client_end, 2, uri, text, "greet()").await;
    let zero_value = zero["result"]["contents"]["value"].as_str().unwrap();
    assert!(zero_value.contains("Zero-arg greeting."), "{zero_value:?}");
    assert!(!zero_value.contains("Greeting with a name."), "{zero_value:?}");

    let one_offset = text.rfind("greet(name)").unwrap();
    let line = text[..one_offset].matches('\n').count();
    let col = one_offset - text[..one_offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": col }
            }
        }),
    )
    .await;
    let one = read_response(&mut client_end, 3).await;
    let one_value = one["result"]["contents"]["value"].as_str().unwrap();
    assert!(one_value.contains("Greeting with a name."), "{one_value:?}");
    assert!(!one_value.contains("Zero-arg greeting."), "{one_value:?}");

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn cross_file_hover_resolves_the_doc_from_the_declaring_file() {
    let workspace = ScratchWorkspace::new("cross-file");
    workspace.write("mover.ph", "class Mover {\n  /// Moves the mover by `x`.\n  move(x) { }\n}\n");
    let main_text = "let m = Mover.new();\nm.move(1);\n";
    workspace.write("main.ph", main_text);

    let (mut client_end, server_task) = spawn_server();
    initialize(&mut client_end, Some(&workspace.uri())).await;

    let main_uri = url::Url::from_file_path(workspace.root.join("main.ph")).unwrap().to_string();
    did_open(&mut client_end, &main_uri, main_text).await;

    // Hover the call site `m.move(1)` in main.ph — the doc lives in
    // mover.ph, a file never opened by the client.
    let response = hover_at(&mut client_end, 2, &main_uri, main_text, "move(1)").await;
    let value = response["result"]["contents"]["value"].as_str().expect("markup contents");
    assert!(value.contains("Moves the mover by `x`."), "{value:?}");
    assert!(value.contains("method on Mover"), "{value:?}");

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn builtin_hover_has_kind_and_selector_but_no_phaldoc_section() {
    let (mut client_end, server_task) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    // `ifTrue(_)` is a core-table (`Bool`) selector with no local
    // declaration in this file, so it must resolve purely from the builtin
    // table with no Phaldoc section.
    let text = "let x = true.ifTrue { 1 };\n";
    did_open(&mut client_end, uri, text).await;

    let response = hover_at(&mut client_end, 2, uri, text, "ifTrue").await;
    let value = response["result"]["contents"]["value"].as_str().expect("markup contents");
    assert!(value.contains("ifTrue(_)"), "{value:?}");
    assert!(value.contains("on Bool"), "{value:?}");
    // No Phaldoc section: no `---` divider, since there is no local `.ph`
    // source to harvest a doc comment from for a native/core builtin.
    assert!(!value.contains("---"), "{value:?}");

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn selector_hover_sets_range_to_the_resolved_selector_span() {
    let (mut client_end, server_task) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    let text = "class Point {\n  move(x) { }\n}\n";
    did_open(&mut client_end, uri, text).await;

    let response = hover_at(&mut client_end, 2, uri, text, "move(x)").await;
    let range = &response["result"]["range"];
    assert!(!range.is_null(), "{response:#?}");
    let (line, character) = position_of(text, "move(x)");
    assert_eq!(range["start"]["line"].as_u64(), Some(line as u64));
    assert_eq!(range["start"]["character"].as_u64(), Some(character as u64));

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn hover_over_a_top_level_binding_usage_surfaces_its_doc() {
    let (mut client_end, server_task) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    // The usage on the last line is a bare `Expr::Var` read (not a call
    // receiver, whose enclosing `MethodCall` range would otherwise resolve
    // to *that* call's own selector first) — the case
    // `index::top_level_binding_at_offset` is meant to catch.
    let text = "/// The application's shared counter.\nlet counter = Counter.new();\nlet echo = counter;\n";
    did_open(&mut client_end, uri, text).await;

    // Hover the *usage* on the last line, not the declaration.
    let usage_offset = text.rfind("counter").unwrap();
    let line = text[..usage_offset].matches('\n').count();
    let col = usage_offset - text[..usage_offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": col }
            }
        }),
    )
    .await;
    let response = read_response(&mut client_end, 2).await;
    let value = response["result"]["contents"]["value"].as_str().expect("markup contents");
    assert!(value.contains("The application's shared counter."), "{value:?}");

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn hover_on_whitespace_returns_none() {
    let (mut client_end, server_task) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    let text = "let x = 1\n";
    did_open(&mut client_end, uri, text).await;

    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 0, "character": 9 }
            }
        }),
    )
    .await;
    let response = read_response(&mut client_end, 2).await;
    assert!(response["result"].is_null(), "{response:#?}");

    drop(client_end);
    let _ = server_task.await;
}
