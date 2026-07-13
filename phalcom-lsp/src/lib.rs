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
//! - [`backend`] — the [`tower_lsp::LanguageServer`] trait implementation,
//!   exported as [`Backend`].

#![warn(missing_docs)]

pub mod backend;
pub mod diagnostics;
pub mod documents;
pub mod line_index;

pub use backend::Backend;
