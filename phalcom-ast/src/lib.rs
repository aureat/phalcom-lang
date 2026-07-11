//! Phalcom front end: lexer, tokens, AST, and a hand-written parser.
//!
//! This crate turns Phalcom source text into an [`ast::Program`] for the
//! `phalcom-core` compiler:
//!
//! * [`lexer`] — a hand-written scanner producing a [`token::Token`] stream with
//!   precise byte spans.
//! * [`parser`] — a hand-written recursive-descent + Pratt parser (see
//!   ADR-0016) with panic-mode error recovery.
//! * [`ast`] — the abstract syntax tree the compiler consumes.
//! * [`error`] — [`error::SyntaxError`], the spanned diagnostic type.
//!
//! The primary entry points are re-exported at the crate root: [`parse_source`]
//! (first-error result, used by the compiler) and [`parse`] (full error
//! recovery, returning every recovered diagnostic).

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod util;

pub use parser::{parse, parse_source, Parse, ParserResult};
