//! In-process Language Server Protocol server for Phalcom.
//!
//! Implements [ADR-0056](../../docs/adr/0056-phalcom-lsp-architecture.md):
//! an editor-intelligence server that embeds `phalcom-ast` directly (no
//! subprocess, no CLI shelling) over `tower-lsp`. Deliberately **VM-free** —
//! this crate depends on [`phalcom_ast`] and [`phalcom_common`] only, never
//! `phalcom-core`, so it starts instantly and cannot be destabilized by
//! runtime/heap churn (ADR-0056 §2).
//!
//! Grows capability-by-capability per
//! `docs/forge/units/U-LSP/plan.md`'s staged plan. This crate currently
//! implements **Stage 1**: live, multi-error diagnostics on
//! `textDocument/didOpen`/`didChange`/`didClose`, superseding the
//! subprocess path's single-error, save-only `phalcom check`.
//!
//! # Modules
//!
//! - [`line_index`] — the standing correctness hotspot: byte offset ↔
//!   `(line, UTF-16 character)` position mapping.
//! - [`diagnostics`] — [`phalcom_ast::error::SyntaxError`] → LSP
//!   `Diagnostic`.
//! - [`documents`] — the open-document store (text + cached parse + cached
//!   [`line_index::LineIndex`]).
//! - [`core_table`] — the embedded builtin core-class selector table
//!   (`core-table.json`), consulted by completion for builtin receivers.
//! - [`completion`] — receiver-aware [`textDocument/completion`] (Stage 3):
//!   the pluggable [`completion::ReceiverResolver`] plus snippet rendering.
//! - [`semantic_tokens`] — flat, lexer-driven [`textDocument/semanticTokens/
//!   full`] (Stage 5): token classification and LSP delta-encoding.
//! - [`backend`] — the [`tower_lsp::LanguageServer`] trait implementation,
//!   exported as [`Backend`].
//!
//! [`textDocument/completion`]: tower_lsp::LanguageServer::completion
//! [`textDocument/semanticTokens/full`]: tower_lsp::LanguageServer::semantic_tokens_full

#![warn(missing_docs)]

pub mod backend;
pub mod completion;
pub mod core_table;
pub mod diagnostics;
pub mod documents;
pub mod index;
pub mod line_index;
pub mod selectors;
pub mod semantic_tokens;

pub use backend::Backend;
