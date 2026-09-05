//! Stage 4 server integration test: drives [`phalcom_lsp::Backend`] through
//! `tower-lsp`'s real JSON-RPC transport over an in-process
//! [`tokio::io::duplex`] pipe (mirrors `tests/stage3_completion.rs`'s
//! transport harness) — no subprocess spawn.
//!
//! Exercises `textDocument/hover` (`docs/forge/units/U-LSP/plan.md` § Tests,
//! Stage 4):
//! - no hover for keywords/literals,
//! - Phaldoc adjacency (a `///` block immediately above a method attaches;
//!   one separated by a blank line does not),
//! - selector-keying (`foo()`/`foo(_)` each carry their own doc),
//! - cross-file resolution (hovering a call site in one file resolves the
//!   doc written on the declaration in another),
//! - builtin hover (a core-table selector renders kind + selector with no
//!   Phaldoc section).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower_lsp::{LspService, Server};

use phalcom_lsp::{Backend, SemanticPublicationHandle};

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

async fn wait_for_workspace_symbol(client: &mut tokio::io::DuplexStream, id: &mut i64) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        *id += 1;
        let request_id = *id;
        write_message(
            client,
            &json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "workspace/symbol",
                "params": { "query": "move" }
            }),
        )
        .await;
        let response = read_response(client, request_id).await;
        if response["result"]
            .as_array()
            .is_some_and(|symbols| symbols.iter().any(|symbol| symbol["name"] == json!("move(_)")))
        {
            return;
        }
        tokio::task::yield_now().await;
    }
    panic!("workspace symbol did not become available within the 30-second yield budget");
}

async fn wait_for_hover(client: &mut tokio::io::DuplexStream, id: &mut i64, uri: &str, text: &str, position_needle: &str, content_needle: &str) -> Value {
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut last_response = Value::Null;
    while Instant::now() < deadline {
        *id += 1;
        let response = hover_at(client, *id, uri, text, position_needle).await;
        last_response = response.clone();
        if response["result"]["contents"]["value"]
            .as_str()
            .is_some_and(|value| value.contains(content_needle))
        {
            return response;
        }
        tokio::task::yield_now().await;
    }
    panic!("cross-file hover did not become available within the 30-second yield budget; last response: {last_response:#?}");
}

/// A scratch directory on disk holding a small multi-file `.ph` fixture, so
/// `initialize`'s workspace scan has real files to walk. Removed on drop.
struct ScratchWorkspace {
    root: PathBuf,
}

impl ScratchWorkspace {
    fn new(name: &str) -> Self {
        let base = std::env::temp_dir();
        let base = base.canonicalize().unwrap_or(base);
        let root = base.join(format!(
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
fn spawn_server() -> (tokio::io::DuplexStream, tokio::task::JoinHandle<()>, SemanticPublicationHandle) {
    let (server_end, client_end) = tokio::io::duplex(1 << 16);
    let (server_read, server_write) = tokio::io::split(server_end);
    let publication = Arc::new(Mutex::new(None));
    let publication_for_backend = publication.clone();
    let (service, socket) = LspService::build(move |client| {
        let backend = Backend::new(client);
        *publication_for_backend.lock().expect("publication capture lock poisoned") = Some(backend.semantic_publication_handle());
        backend
    })
    .finish();
    let handle = publication
        .lock()
        .expect("publication capture lock poisoned")
        .as_ref()
        .expect("backend publication handle captured")
        .clone();
    let server_task = tokio::spawn(async move {
        Server::new(server_read, server_write, socket).serve(service).await;
    });
    (client_end, server_task, handle)
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

async fn did_open(client_end: &mut tokio::io::DuplexStream, publication: &SemanticPublicationHandle, uri: &str, text: &str) {
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
    // Drain the immediate syntax-diagnostics publication; it is not a
    // semantic readiness signal.
    let _ = read_message(client_end).await;

    let uri = url::Url::parse(uri).expect("test URI");
    let path = uri.to_file_path().expect("Stage 4 semantic source must use a file URI");
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if publication.contains_exact_source(&path, text) {
            return;
        }
        tokio::task::yield_now().await;
    }
    panic!("exact Stage 4 semantic source was not published within the 30-second yield budget: {uri}");
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
async fn keywords_and_literals_are_not_semantic_hovers() {
    let (mut client_end, server_task, publication) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    let text = "class Point {\n  x() { self }\n}\nlet flag = true\n";
    did_open(&mut client_end, &publication, uri, text).await;

    for (id, needle) in [(2, "class"), (3, "self"), (4, "let"), (5, "true")] {
        let response = hover_at(&mut client_end, id, uri, text, needle).await;
        assert!(response["result"].is_null(), "{needle} hover should be absent: {response:?}");
    }

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn phaldoc_adjacency_attaches_and_blank_line_breaks_it() {
    let (mut client_end, server_task, publication) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    let text = "class Point {\n  /// Moves the point.\n  move(_ x) { }\n\n  /// Detached, not adjacent.\n\n  reset() { }\n}\n";
    did_open(&mut client_end, &publication, uri, text).await;

    // Adjacent doc attaches.
    let response = hover_at(&mut client_end, 2, uri, text, "move(_ x)").await;
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
async fn phaldoc_adjacency_survives_member_attributes() {
    let (mut client_end, server_task, publication) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    let text = "class Point {\n  /// Private class-side movement.\n  @class\n  @private\n  move() { }\n}\n";
    did_open(&mut client_end, &publication, uri, text).await;

    let response = hover_at(&mut client_end, 2, uri, text, "move()").await;
    let value = response["result"]["contents"]["value"].as_str().expect("markup contents");
    assert!(value.contains("Private class-side movement."), "{value:?}");

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn selector_keying_gives_each_arity_its_own_doc() {
    let (mut client_end, server_task, publication) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    let text = "class Greeter {\n  /// Zero-arg greeting.\n  greet() { }\n  /// Greeting with a name.\n  greet(_ name) { }\n}\n";
    did_open(&mut client_end, &publication, uri, text).await;

    let zero = hover_at(&mut client_end, 2, uri, text, "greet()").await;
    let zero_value = zero["result"]["contents"]["value"].as_str().unwrap();
    assert!(zero_value.contains("Zero-arg greeting."), "{zero_value:?}");
    assert!(!zero_value.contains("Greeting with a name."), "{zero_value:?}");

    let one_offset = text.rfind("greet(_ name)").unwrap();
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
    workspace.write("package.ph", "");
    workspace.write("mover.ph", "class Mover {\n  /// Moves the mover by `x`.\n  move(_ x) { }\n}\n");
    let main_text = "import .mover as MoverModule\nlet m = MoverModule.Mover.new();\nm.move(1);\n";
    workspace.write("main.ph", main_text);

    let (mut client_end, server_task, publication) = spawn_server();
    initialize(&mut client_end, Some(&workspace.uri())).await;

    let main_uri = url::Url::from_file_path(workspace.root.join("main.ph")).unwrap().to_string();
    did_open(&mut client_end, &publication, &main_uri, main_text).await;

    // Hover the call site `m.move(1)` in main.ph — the doc lives in
    // mover.ph, a file never opened by the client.
    let mut next_request_id = 1;
    wait_for_workspace_symbol(&mut client_end, &mut next_request_id).await;
    let response = wait_for_hover(&mut client_end, &mut next_request_id, &main_uri, main_text, "move(1)", "Moves the mover by").await;
    let value = response["result"]["contents"]["value"].as_str().expect("markup contents");
    assert!(value.contains("Moves the mover by `x`."), "{value:?}");
    assert!(value.contains("method on Mover"), "{value:?}");

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn builtin_hover_has_kind_selector_and_native_documentation() {
    let (mut client_end, server_task, publication) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    // `ifTrue(_)` is a native `Bool` selector with no local declaration in
    // this file, so it must resolve from canonical native metadata.
    let text = "let x = true.ifTrue || { 1 };\n";
    did_open(&mut client_end, &publication, uri, text).await;

    let mut next_request_id = 1;
    let response = wait_for_hover(&mut client_end, &mut next_request_id, uri, text, "ifTrue", "ifTrue").await;
    let value = response["result"]["contents"]["value"].as_str().expect("markup contents");
    assert!(value.contains("ifTrue(_)"), "{value:?}");
    assert!(value.contains("on Bool"), "{value:?}");
    assert!(value.contains("native primitive"), "{value:?}");
    assert!(value.contains("Executes block if receiver is true."), "{value:?}");

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn receiver_qualified_hover_selects_one_of_repeated_new_members() {
    let (mut client_end, server_task, publication) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    let text = "class Counter { @constructor new() { } }\nclass Other { @constructor new() { } }\nlet c = Counter.new()\n";
    did_open(&mut client_end, &publication, uri, text).await;

    let declaration = hover_at(&mut client_end, 2, uri, text, "new()").await;
    let declaration_value = declaration["result"]["contents"]["value"].as_str().unwrap();
    assert!(declaration_value.contains("on Counter"), "{declaration_value:?}");
    assert!(!declaration_value.contains("Other"), "{declaration_value:?}");

    let call_offset = text.rfind("Counter.new()").unwrap() + "Counter.".len();
    let line = text[..call_offset].matches('\n').count();
    let character = call_offset - text[..call_offset].rfind('\n').map(|index| index + 1).unwrap_or(0);
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": line, "character": character }
            }
        }),
    )
    .await;
    let call = read_response(&mut client_end, 3).await;
    let call_value = call["result"]["contents"]["value"].as_str().unwrap();
    assert!(call_value.contains("on Counter"), "{call_value:?}");
    assert!(!call_value.contains("Other"), "{call_value:?}");

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn selector_hover_sets_range_to_the_resolved_selector_span() {
    let (mut client_end, server_task, publication) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    let text = "class Point {\n  move(_ x) { }\n}\n";
    did_open(&mut client_end, &publication, uri, text).await;

    let response = hover_at(&mut client_end, 2, uri, text, "move(_ x)").await;
    let range = &response["result"]["range"];
    assert!(!range.is_null(), "{response:#?}");
    let (line, character) = position_of(text, "move(_ x)");
    assert_eq!(range["start"]["line"].as_u64(), Some(line as u64));
    assert_eq!(range["start"]["character"].as_u64(), Some(character as u64));
    assert_eq!(range["end"]["line"].as_u64(), Some(line as u64));
    assert_eq!(range["end"]["character"].as_u64(), Some((character + "move".len()) as u64));

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn binding_hover_ranges_are_exact_and_scope_aware() {
    let (mut client_end, server_task, publication) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    let text = "class Sample {\n  method(parameter) {\n    let local = parameter\n    local\n  }\n}\n";
    did_open(&mut client_end, &publication, uri, text).await;

    let parameter_decl = text.find("parameter").unwrap();
    let local_decl = text.find("local").unwrap();
    let parameter_use = text.rfind("parameter").unwrap();
    let local_use = text.rfind("local").unwrap();
    for (id, offset, token, expected) in [
        (2, parameter_decl, "parameter", "parameter"),
        (3, local_decl, "local", "mutable binding"),
        (4, parameter_use, "parameter", "parameter"),
        (5, local_use, "local", "mutable binding"),
    ] {
        let line = text[..offset].matches('\n').count();
        let character = offset - text[..offset].rfind('\n').map(|index| index + 1).unwrap_or(0);
        write_message(
            &mut client_end,
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
        let response = read_response(&mut client_end, id).await;
        let value = response["result"]["contents"]["value"].as_str().expect("binding hover");
        assert!(value.contains(expected), "{value:?}");
        let range = &response["result"]["range"];
        assert_eq!(range["start"]["line"].as_u64(), Some(line as u64));
        assert_eq!(range["start"]["character"].as_u64(), Some(character as u64));
        assert_eq!(range["end"]["line"].as_u64(), Some(line as u64));
        assert_eq!(range["end"]["character"].as_u64(), Some((character + token.len()) as u64));
    }

    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn hover_over_a_top_level_binding_usage_surfaces_its_doc() {
    let (mut client_end, server_task, publication) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    // The usage on the last line is a bare `Expr::Var` read (not a call
    // receiver, whose enclosing `MethodCall` range would otherwise resolve
    // to *that* call's own selector first) — the case
    // `index::top_level_binding_at_offset` is meant to catch.
    let text = "/// The application's shared counter.\nlet counter = Counter.new();\nlet echo = counter;\n";
    did_open(&mut client_end, &publication, uri, text).await;

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
    let (mut client_end, server_task, publication) = spawn_server();
    initialize(&mut client_end, None).await;

    let uri = "file:///workspace/main.ph";
    let text = "let x = 1\n";
    did_open(&mut client_end, &publication, uri, text).await;

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
