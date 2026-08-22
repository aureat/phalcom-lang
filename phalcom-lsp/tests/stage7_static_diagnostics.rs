//! Stage 7 integration tests: static semantic diagnostics published through LSP JSON-RPC.

use phalcom_lsp::Backend;
use serde_json::{Value, json};
use std::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{Duration, timeout};
use tower_lsp::{LspService, Server};

async fn write_message(w: &mut (impl AsyncWriteExt + Unpin), value: &Value) {
    let body = serde_json::to_string(value).unwrap();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    w.write_all(header.as_bytes()).await.unwrap();
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

async fn read_until_matching(r: &mut (impl AsyncReadExt + Unpin), predicate: impl Fn(&Value) -> bool) -> Value {
    let mut observed = Vec::new();
    timeout(Duration::from_secs(30), async {
        for _ in 0..64 {
            let msg = read_message(r).await;
            if predicate(&msg) {
                return Some(msg);
            }
            observed.push(msg);
        }
        None
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for matching message; observed: {observed:#?}"))
    .unwrap_or_else(|| panic!("did not observe matching message within the read budget; observed: {observed:#?}"))
}

#[tokio::test]
async fn test_static_mismatch_publishes_typecheck_diagnostics() {
    let (server_end, mut client_end) = tokio::io::duplex(1 << 16);
    let (server_read, server_write) = tokio::io::split(server_end);

    let (service, socket) = LspService::new(Backend::new);
    let server_task = tokio::spawn(async move {
        Server::new(server_read, server_write, socket).serve(service).await;
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
    let _ = read_message(&mut client_end).await;

    // initialized
    write_message(&mut client_end, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await;

    // Open file with static type mismatch
    let uri = "file:///tmp/typecheck_test.ph";
    let source = "let x: Int = \"string\"\n";

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
                    "text": source
                }
            }
        }),
    )
    .await;

    let diag_msg = read_until_matching(&mut client_end, |msg| {
        if msg.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics") {
            let diags = msg["params"]["diagnostics"].as_array();
            diags.is_some_and(|diagnostics| !diagnostics.is_empty())
        } else {
            false
        }
    })
    .await;

    let diags = diag_msg["params"]["diagnostics"].as_array().expect("diagnostics array");
    assert!(!diags.is_empty(), "expected typecheck diagnostics");
    let typecheck = diags
        .iter()
        .find(|diagnostic| diagnostic["source"] == json!("phalcom-typecheck"))
        .unwrap_or_else(|| panic!("expected diagnostic with source = 'phalcom-typecheck', got: {diags:?}"));
    assert_eq!(
        typecheck["code"],
        json!("type.binding.initializer_mismatch"),
        "semantic diagnostic code must survive LSP adaptation"
    );

    // Recovery can retain a valid first statement. Publish both syntax and
    // semantic diagnostics for the same recovered source revision.
    let recovered_source = "let x: Int = \"string\"\nlet =\n";
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": uri, "version": 2 },
                "contentChanges": [{ "text": recovered_source }]
            }
        }),
    )
    .await;

    let coexist_msg = read_until_matching(&mut client_end, |msg| {
        let Some(diags) = msg["params"]["diagnostics"].as_array() else {
            return false;
        };
        diags.iter().any(|d| d["source"] == json!("phalcom")) && diags.iter().any(|d| d["source"] == json!("phalcom-typecheck"))
    })
    .await;
    let coexist = coexist_msg["params"]["diagnostics"].as_array().unwrap();
    assert!(coexist.iter().any(|d| d["source"] == json!("phalcom")));
    assert!(coexist.iter().any(|d| d["source"] == json!("phalcom-typecheck")));

    // A syntax-invalid replacement must not reuse type diagnostics from the
    // previous source revision while the worker catches up.
    let invalid_source = "let x: Int =\n";
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": uri, "version": 3 },
                "contentChanges": [{ "text": invalid_source }]
            }
        }),
    )
    .await;

    let invalid_msg = read_until_matching(&mut client_end, |msg| {
        if msg["params"]["version"] != json!(3) {
            return false;
        }
        let Some(diags) = msg["params"]["diagnostics"].as_array() else {
            return false;
        };
        diags.iter().any(|d| d["source"] == json!("phalcom"))
    })
    .await;
    let invalid_diags = invalid_msg["params"]["diagnostics"].as_array().unwrap();
    assert!(invalid_diags.iter().all(|d| d["source"] != json!("phalcom-typecheck")));

    // Fix the mismatch and verify diagnostics clear
    let fixed_source = "let x: Int = 42\n";
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": uri, "version": 4 },
                "contentChanges": [{ "text": fixed_source }]
            }
        }),
    )
    .await;

    // First empty publication is the immediate syntax pass for version 4.
    // The second proves the static worker published the clean generation.
    let _ = read_until_matching(&mut client_end, |msg| {
        if msg.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics") {
            let diags = msg["params"]["diagnostics"].as_array();
            diags.is_some_and(Vec::is_empty) && msg["params"]["version"] == json!(4)
        } else {
            false
        }
    })
    .await;
    let cleared_msg = read_until_matching(&mut client_end, |msg| {
        msg.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
            && msg["params"]["diagnostics"].as_array().is_some_and(Vec::is_empty)
            && msg["params"]["version"] == json!(4)
    })
    .await;

    let diags = cleared_msg["params"]["diagnostics"].as_array().unwrap();
    assert!(diags.is_empty(), "expected diagnostics to be cleared on fix");

    // Closing the transport terminates the in-process server task.
    drop(client_end);
    let _ = server_task.await;
}

#[tokio::test]
async fn test_project_edit_rechecks_importer_without_polluting_unrelated_module() {
    let root = std::env::temp_dir().join(format!(
        "phalcom_lsp_project_diagnostics_{}_{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    let source_root = root.join("src");
    fs::create_dir_all(&source_root).unwrap();
    fs::write(
        root.join("project.toml"),
        "[project]\nname = \"lsp-diagnostics\"\nnamespace = \"lsp_diagnostics\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(source_root.join("package.ph"), "").unwrap();
    let provider_source = "class User {}\nclass Admin {}\nexport (User, Admin)\n";
    let importer_source = "from .provider import (User, Admin)\nlet x: User = Admin()\n";
    let unrelated_source = "let value: Int = 1\n";
    let provider_path = source_root.join("provider.ph");
    let importer_path = source_root.join("importer.ph");
    let unrelated_path = source_root.join("unrelated.ph");
    fs::write(&provider_path, provider_source).unwrap();
    fs::write(&importer_path, importer_source).unwrap();
    fs::write(&unrelated_path, unrelated_source).unwrap();

    let root_uri = tower_lsp::lsp_types::Url::from_directory_path(root.canonicalize().unwrap())
        .unwrap()
        .to_string();
    let provider_uri = tower_lsp::lsp_types::Url::from_file_path(provider_path.canonicalize().unwrap())
        .unwrap()
        .to_string();
    let importer_uri = tower_lsp::lsp_types::Url::from_file_path(importer_path.canonicalize().unwrap())
        .unwrap()
        .to_string();
    let unrelated_uri = tower_lsp::lsp_types::Url::from_file_path(unrelated_path.canonicalize().unwrap())
        .unwrap()
        .to_string();

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
            "params": { "processId": null, "rootUri": root_uri, "capabilities": {} }
        }),
    )
    .await;
    let _ = read_message(&mut client_end).await;
    write_message(&mut client_end, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })).await;

    for (uri, text) in [
        (&importer_uri, importer_source),
        (&unrelated_uri, unrelated_source),
        (&provider_uri, provider_source),
    ] {
        write_message(
            &mut client_end,
            &json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": { "uri": uri, "languageId": "phalcom", "version": 1, "text": text }
                }
            }),
        )
        .await;
    }

    let mismatch = read_until_matching(&mut client_end, |msg| {
        msg["method"] == json!("textDocument/publishDiagnostics")
            && msg["params"]["uri"] == json!(importer_uri)
            && msg["params"]["diagnostics"].as_array().is_some_and(|diagnostics| {
                diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic["code"] == json!("type.binding.initializer_mismatch"))
            })
    })
    .await;
    assert!(
        mismatch["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic["source"] == json!("phalcom-typecheck"))
    );

    let changed_provider = "class User {}\nclass Admin is User {}\nexport (User, Admin)\n";
    write_message(
        &mut client_end,
        &json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": provider_uri, "version": 2 },
                "contentChanges": [{ "text": changed_provider }]
            }
        }),
    )
    .await;

    let importer_update = read_until_matching(&mut client_end, |msg| {
        msg["method"] == json!("textDocument/publishDiagnostics")
            && msg["params"]["uri"] == json!(importer_uri)
            && msg["params"]["version"] == json!(1)
            && msg["params"]["diagnostics"].as_array().is_some_and(Vec::is_empty)
    })
    .await;
    assert!(
        importer_update["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .all(|diagnostic| diagnostic["code"] != json!("type.binding.initializer_mismatch")),
        "dependent mismatch must clear after exported hierarchy changes"
    );

    let unrelated_update = read_until_matching(&mut client_end, |msg| {
        msg["method"] == json!("textDocument/publishDiagnostics") && msg["params"]["uri"] == json!(unrelated_uri) && msg["params"]["version"] == json!(1)
    })
    .await;
    assert!(
        unrelated_update["params"]["diagnostics"].as_array().is_some_and(Vec::is_empty),
        "unrelated module must remain diagnostic-free"
    );

    drop(client_end);
    let _ = server_task.await;
    let _ = fs::remove_dir_all(root);
}
