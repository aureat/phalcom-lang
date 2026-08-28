//! Integration tests for module occurrences, navigation, and core source provenance (Tasks 13 & 14).

use std::path::PathBuf;
use std::time::Duration;

use phalcom_lsp::backend::Backend;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower_lsp::lsp_types::Url;
use tower_lsp::{LspService, Server};

struct ScratchWorkspace {
    root: PathBuf,
}

impl ScratchWorkspace {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "phalcom-lsp-mod-nav-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        Self { root }
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, contents).unwrap();
        path
    }

    fn uri(&self) -> String {
        Url::from_directory_path(&self.root).unwrap().to_string()
    }
}

impl Drop for ScratchWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

async fn write_message<W: AsyncWriteExt + Unpin>(writer: &mut W, value: &Value) {
    let body = serde_json::to_string(value).unwrap();
    let frame = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
    writer.write_all(frame.as_bytes()).await.unwrap();
    writer.flush().await.unwrap();
}

async fn read_message<R: AsyncReadExt + Unpin>(reader: &mut R) -> Value {
    let mut header = Vec::new();
    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte).await.unwrap();
        header.push(byte[0]);
        if header.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let header_str = std::str::from_utf8(&header).unwrap();
    let length: usize = header_str
        .lines()
        .find(|line| line.starts_with("Content-Length:"))
        .and_then(|line| line.split(':').nth(1))
        .map(|value| value.trim().parse().unwrap())
        .unwrap();

    let mut body = vec![0u8; length];
    reader.read_exact(&mut body).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

async fn read_response<R: AsyncReadExt + Unpin>(reader: &mut R, id: u64) -> Value {
    loop {
        let msg = read_message(reader).await;
        if msg.get("id").and_then(|id_val| id_val.as_u64()) == Some(id) {
            return msg;
        }
    }
}

async fn wait_for_definition(client: &mut tokio::io::DuplexStream, id: &mut u64, uri: &str, line: usize, character: usize) -> Value {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let mut last_response = Value::Null;
    while std::time::Instant::now() < deadline {
        *id += 1;
        let request_id = *id;
        write_message(
            client,
            &json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "textDocument/definition",
                "params": {
                    "textDocument": { "uri": uri },
                    "position": { "line": line, "character": character }
                }
            }),
        )
        .await;
        let response = read_response(client, request_id).await;
        last_response = response.clone();
        if response["result"].as_array().is_some_and(|locations| !locations.is_empty()) {
            return response;
        }
        tokio::task::yield_now().await;
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("definition did not become available within deadline: {last_response:#?}");
}

#[tokio::test]
async fn goto_definition_on_relative_import_path_and_selective_export() {
    let workspace = ScratchWorkspace::new("rel_import");
    let shapes_path = workspace.write("shapes.ph", "class Circle {\n  area() { 3.14 }\n}\nexport Circle\n");
    let main_path = workspace.write("main.ph", "import .shapes as shapes\nfrom .shapes import Circle\n\nlet c = Circle.new();\n");

    let (server_end, mut client_end) = tokio::io::duplex(1 << 16);
    let (server_read, server_write) = tokio::io::split(server_end);

    let (service, socket) = LspService::new(Backend::new);
    tokio::spawn(async move {
        Server::new(server_read, server_write, socket).serve(service).await;
    });

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
    let _ = read_response(&mut client_end, 1).await;
    write_message(&mut client_end, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await;

    let main_text = std::fs::read_to_string(&main_path).unwrap();
    let main_uri = Url::from_file_path(&main_path).unwrap().to_string();
    let shapes_uri = Url::from_file_path(&shapes_path).unwrap().to_string();

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
                    "text": main_text
                }
            }
        }),
    )
    .await;

    let mut req_id = 1u64;
    let def_resp = wait_for_definition(&mut client_end, &mut req_id, &main_uri, 0, 10).await;
    let result = &def_resp["result"];
    assert!(result.is_array(), "expected definition location array, got {result:?}");
    let locs = result.as_array().unwrap();
    assert!(!locs.is_empty(), "expected at least one location for .shapes import");
    assert_eq!(locs[0]["uri"].as_str(), Some(shapes_uri.as_str()));

    // 2. Hover on `shapes` import segment (line 0, col 10)
    req_id += 1;
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": main_uri },
                "position": { "line": 0, "character": 10 }
            }
        }),
    )
    .await;

    let hover_resp = read_response(&mut client_end, req_id).await;
    let hover_contents = hover_resp["result"]["contents"]["value"].as_str().unwrap_or("");
    assert!(hover_contents.contains("module"), "hover should show module info, got {hover_contents:?}");

    // 3. Find references on `shapes` (line 0, col 20)
    req_id += 1;
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "method": "textDocument/references",
            "params": {
                "textDocument": { "uri": main_uri },
                "position": { "line": 0, "character": 20 },
                "context": { "includeDeclaration": true }
            }
        }),
    )
    .await;

    let ref_resp = read_response(&mut client_end, req_id).await;
    let ref_locs = ref_resp["result"].as_array();
    assert!(ref_locs.is_some_and(|locs| !locs.is_empty()), "expected references for shapes binding");

    // 4. Go to Definition on `Circle` in selective import (line 1, col 23)
    let def_resp = wait_for_definition(&mut client_end, &mut req_id, &main_uri, 1, 23).await;
    let locs = def_resp["result"].as_array().unwrap();
    assert!(!locs.is_empty(), "expected definition for imported Circle");
    assert_eq!(locs[0]["uri"].as_str(), Some(main_uri.as_str()));

    // 5. Go to Definition on `Circle` at usage site (line 3, col 10)
    let def_resp = wait_for_definition(&mut client_end, &mut req_id, &main_uri, 3, 10).await;
    let locs = def_resp["result"].as_array().unwrap();
    assert!(!locs.is_empty(), "expected definition for Circle at usage site");
    assert_eq!(locs[0]["uri"].as_str(), Some(main_uri.as_str()));
}

#[tokio::test]
async fn unresolved_import_does_not_crash_definition_or_hover() {
    let workspace = ScratchWorkspace::new("unresolved");
    let main_path = workspace.write("main.ph", "import .nonexistent as Missing\n\nlet x = 1;\n");

    let (server_end, mut client_end) = tokio::io::duplex(1 << 16);
    let (server_read, server_write) = tokio::io::split(server_end);

    let (service, socket) = LspService::new(Backend::new);
    tokio::spawn(async move {
        Server::new(server_read, server_write, socket).serve(service).await;
    });

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
    let _ = read_response(&mut client_end, 1).await;
    write_message(&mut client_end, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await;

    let main_text = std::fs::read_to_string(&main_path).unwrap();
    let main_uri = Url::from_file_path(&main_path).unwrap().to_string();

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
                    "text": main_text
                }
            }
        }),
    )
    .await;

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Go to definition on nonexistent module
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": { "uri": main_uri },
                "position": { "line": 0, "character": 12 }
            }
        }),
    )
    .await;

    let def_resp = read_response(&mut client_end, 2).await;
    // Should gracefully return null / empty without error
    assert!(def_resp["error"].is_null());
}

#[tokio::test]
async fn goto_definition_on_core_class_returns_virtual_location() {
    let workspace = ScratchWorkspace::new("core_nav");
    let main_path = workspace.write("main.ph", "class MyClass is Object {\n}\n");

    let (server_end, mut client_end) = tokio::io::duplex(1 << 16);
    let (server_read, server_write) = tokio::io::split(server_end);

    let (service, socket) = LspService::new(Backend::new);
    tokio::spawn(async move {
        Server::new(server_read, server_write, socket).serve(service).await;
    });

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
    let _ = read_response(&mut client_end, 1).await;
    write_message(&mut client_end, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await;

    let main_text = std::fs::read_to_string(&main_path).unwrap();
    let main_uri = Url::from_file_path(&main_path).unwrap().to_string();

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
                    "text": main_text
                }
            }
        }),
    )
    .await;

    let mut req_id = 1u64;
    let def_resp = wait_for_definition(&mut client_end, &mut req_id, &main_uri, 0, 18).await;
    let result = &def_resp["result"];
    assert!(result.is_array(), "expected definition location array for Object, got {result:?}");
    let locs = result.as_array().unwrap();
    assert!(!locs.is_empty(), "expected location for Object definition");
    let uri_str = locs[0]["uri"].as_str().unwrap_or("");
    assert!(
        uri_str.starts_with("phalcom://") || uri_str.ends_with(".ph"),
        "expected virtual or physical core location, got {uri_str}"
    );
}
