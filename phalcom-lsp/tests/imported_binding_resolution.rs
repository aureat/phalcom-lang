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
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "phalcom-lsp-imported-binding-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        Self {
            root: root.canonicalize().unwrap(),
        }
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
    let header = std::str::from_utf8(&header).unwrap();
    let length: usize = header
        .lines()
        .find(|line| line.starts_with("Content-Length:"))
        .and_then(|line| line.split(':').nth(1))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

async fn read_response<R: AsyncReadExt + Unpin>(reader: &mut R, id: u64) -> Value {
    loop {
        let message = read_message(reader).await;
        if message.get("id").and_then(Value::as_u64) == Some(id) {
            return message;
        }
    }
}

async fn wait_for_definition(client: &mut tokio::io::DuplexStream, id: &mut u64, uri: &str, line: usize, character: usize) -> Value {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let mut last = Value::Null;
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
        last = response.clone();
        if response["result"].as_array().is_some_and(|locations| !locations.is_empty()) {
            return response;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("definition did not become available: {last:#?}");
}

#[tokio::test]
async fn imported_binding_definition_crosses_module_boundary_at_declaration_and_use() {
    let workspace = ScratchWorkspace::new();
    let shapes_path = workspace.write("shapes.ph", "class Circle {\n  area() -> Float { 3.14 }\n}\nexport Circle\n");
    let main_path = workspace.write("main.ph", "from .shapes import Circle\n\nlet circle = Circle\n");

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

    let mut request_id = 1;

    let imported_name = wait_for_definition(&mut client_end, &mut request_id, &main_uri, 0, 22).await;
    let imported_locations = imported_name["result"].as_array().expect("definition locations");
    assert_eq!(
        imported_locations[0]["uri"].as_str(),
        Some(shapes_uri.as_str()),
        "the selective import declaration must resolve to the exported declaration's module"
    );

    let usage = wait_for_definition(&mut client_end, &mut request_id, &main_uri, 2, 15).await;
    let usage_locations = usage["result"].as_array().expect("definition locations");
    assert_eq!(
        usage_locations[0]["uri"].as_str(),
        Some(shapes_uri.as_str()),
        "the imported binding use must preserve the same canonical declaration target"
    );
}
