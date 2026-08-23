//! `phalcom-lsp` binary entry point: thin stdio wiring for the language
//! server.
//!
//! Constructs a [`phalcom_lsp::Backend`] and serves it over stdin/stdout via
//! `tower-lsp`'s JSON-RPC transport — the standard way editors (VS Code via
//! `vscode-languageclient`, etc.) launch an LSP server as a child process.
//! All server logic lives in the `phalcom_lsp` library crate; this binary
//! contains no logic of its own beyond transport wiring.

use phalcom_lsp::Backend;
use tower_lsp::{LspService, Server};

/// Wires stdin/stdout to a fresh [`Backend`] and serves it until the client
/// disconnects.
#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new)
        .custom_method("phalcom/sourceText", Backend::source_text)
        .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
