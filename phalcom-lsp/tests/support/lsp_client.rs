use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, DuplexStream},
    task::JoinHandle,
    time::timeout,
};
use tower_lsp::lsp_types::Position;
use tower_lsp::{LspService, Server};

use phalcom_lsp::Backend;
use phalcom_lsp::perf::{CounterSnapshot, PerfCountersHandle};

const IO_TIMEOUT: Duration = Duration::from_secs(5);

pub struct TestLsp {
    client: DuplexStream,
    server_task: JoinHandle<()>,
    next_id: i64,
    next_version: i32,
    counters: Arc<Mutex<Option<PerfCountersHandle>>>,
}

impl TestLsp {
    pub async fn start() -> Self {
        let (server_end, client) = tokio::io::duplex(1 << 20);
        let (server_read, server_write) = tokio::io::split(server_end);

        let counters = Arc::new(Mutex::new(None));
        let counters_for_backend = counters.clone();
        let (service, socket) = LspService::new(move |client| {
            let backend = Backend::new(client);
            *counters_for_backend.lock().expect("counter capture lock poisoned") = Some(backend.perf_counters());
            backend
        });
        let server_task = tokio::spawn(async move {
            Server::new(server_read, server_write, socket).serve(service).await;
        });

        Self {
            client,
            server_task,
            next_id: 1,
            next_version: 1,
            counters,
        }
    }

    pub async fn initialize(&mut self, root_uri: Option<&str>) -> Value {
        self.initialize_with_options(root_uri, Value::Null).await
    }

    pub async fn initialize_with_options(&mut self, root_uri: Option<&str>, initialization_options: Value) -> Value {
        let workspace_folders = root_uri.map(|uri| vec![json!({ "uri": uri, "name": "test-workspace" })]);

        let response = self
            .request(
                "initialize",
                json!({
                    "processId": null,
                    "rootUri": root_uri,
                    "capabilities": {
                        "textDocument": {
                            "completion": {
                                "completionItem": { "snippetSupport": true }
                            },
                            "inlayHint": {
                                "dynamicRegistration": false,
                                "resolveSupport": { "properties": ["tooltip"] }
                            },
                            "semanticTokens": {
                                "requests": { "full": true },
                                "tokenTypes": [],
                                "tokenModifiers": [],
                                "formats": ["relative"]
                            }
                        },
                        "workspace": {
                            "workspaceFolders": true,
                            "didChangeWatchedFiles": { "dynamicRegistration": true }
                        }
                    },
                    "workspaceFolders": workspace_folders,
                    "initializationOptions": initialization_options
                }),
            )
            .await;

        self.notify("initialized", json!({})).await;
        response
    }

    pub async fn open(&mut self, uri: &str, text: &str) {
        let version = self.bump_version();
        self.notify(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": "phalcom",
                    "version": version,
                    "text": text
                }
            }),
        )
        .await;
    }

    pub async fn change(&mut self, uri: &str, text: &str) {
        let version = self.bump_version();
        self.notify(
            "textDocument/didChange",
            json!({
                "textDocument": { "uri": uri, "version": version },
                "contentChanges": [{ "text": text }]
            }),
        )
        .await;
    }

    pub async fn completion(&mut self, uri: &str, position: Position) -> Value {
        self.request(
            "textDocument/completion",
            json!({
                "textDocument": { "uri": uri },
                "position": position
            }),
        )
        .await
    }

    pub async fn hover(&mut self, uri: &str, position: Position) -> Value {
        self.request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": uri },
                "position": position
            }),
        )
        .await
    }

    pub async fn inlay_hints(&mut self, uri: &str, end_line: u32) -> Value {
        self.request(
            "textDocument/inlayHint",
            json!({
                "textDocument": { "uri": uri },
                "range": {
                    "start": { "line": 0, "character": 0 },
                    "end": { "line": end_line, "character": 0 }
                }
            }),
        )
        .await
    }

    pub async fn semantic_tokens_full(&mut self, uri: &str) -> Value {
        self.request("textDocument/semanticTokens/full", json!({ "textDocument": { "uri": uri } }))
            .await
    }

    pub fn counter_snapshot(&self) -> CounterSnapshot {
        self.counters
            .lock()
            .expect("counter capture lock poisoned")
            .as_ref()
            .expect("backend counter handle captured during start")
            .snapshot()
    }

    pub async fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;

        self.write_message(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        }))
        .await;

        self.read_response(id).await
    }

    pub async fn notify(&mut self, method: &str, params: Value) {
        self.write_message(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        }))
        .await;
    }

    pub async fn finish(self) {
        let Self { client, server_task, .. } = self;
        drop(client);
        let _ = timeout(IO_TIMEOUT, server_task).await;
    }

    fn bump_version(&mut self) -> i32 {
        let version = self.next_version;
        self.next_version += 1;
        version
    }

    async fn write_message(&mut self, value: &Value) {
        let body = serde_json::to_string(value).expect("serialize JSON-RPC");
        let header = format!("Content-Length: {}\r\n\r\n", body.len());

        timeout(IO_TIMEOUT, self.client.write_all(header.as_bytes()))
            .await
            .expect("timed out writing LSP header")
            .expect("write LSP header");
        timeout(IO_TIMEOUT, self.client.write_all(body.as_bytes()))
            .await
            .expect("timed out writing LSP body")
            .expect("write LSP body");
    }

    async fn read_response(&mut self, id: i64) -> Value {
        for _ in 0..64 {
            let message = self.read_message().await;
            if message.get("id").and_then(Value::as_i64) == Some(id) && message.get("method").is_none() {
                return message;
            }
            if let Some(req_id) = message.get("id") {
                if message.get("method").is_some() {
                    let resp = json!({ "jsonrpc": "2.0", "id": req_id, "result": null });
                    let body = serde_json::to_string(&resp).expect("serialize JSON-RPC response");
                    let header = format!("Content-Length: {}\r\n\r\n", body.len());
                    let _ = self.client.write_all(header.as_bytes()).await;
                    let _ = self.client.write_all(body.as_bytes()).await;
                }
            }
        }
        panic!("did not observe response id {id} within message budget");
    }

    async fn read_message(&mut self) -> Value {
        timeout(IO_TIMEOUT, async {
            let mut header = Vec::new();

            loop {
                let mut byte = [0u8; 1];
                self.client.read_exact(&mut byte).await.expect("read LSP header");
                header.push(byte[0]);

                if header.ends_with(b"\r\n\r\n") {
                    break;
                }

                assert!(header.len() < 64 * 1024, "LSP header exceeded sanity limit");
            }

            let header = String::from_utf8(header).expect("UTF-8 LSP header");
            let content_length: usize = header
                .lines()
                .find_map(|line| line.strip_prefix("Content-Length: "))
                .expect("Content-Length header")
                .trim()
                .parse()
                .expect("numeric Content-Length");

            let mut body = vec![0u8; content_length];
            self.client.read_exact(&mut body).await.expect("read LSP body");

            serde_json::from_slice(&body).expect("valid JSON-RPC response")
        })
        .await
        .expect("timed out waiting for LSP message")
    }
}

pub fn completion_labels(response: &Value) -> Vec<String> {
    let result = &response["result"];
    let items = if let Some(items) = result.as_array() {
        items
    } else {
        result["items"]
            .as_array()
            .unwrap_or_else(|| panic!("completion result is neither array nor CompletionList: {response:#?}"))
    };

    items.iter().filter_map(|item| item["label"].as_str().map(str::to_owned)).collect()
}

pub fn hint_labels(response: &Value) -> Vec<String> {
    response["result"]
        .as_array()
        .unwrap_or_else(|| panic!("inlayHint result is not an array: {response:#?}"))
        .iter()
        .filter_map(|hint| match &hint["label"] {
            Value::String(s) => Some(s.clone()),
            Value::Array(parts) => Some(parts.iter().filter_map(|part| part["value"].as_str()).collect::<Vec<_>>().join("")),
            _ => None,
        })
        .collect()
}
