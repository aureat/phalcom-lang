//! Shared constants for the Phalcom REPL modules.
//!
//! Centralises language primitives — in particular the keyword list — so that
//! [`completer`](super::completer) and [`highlighter`](super::highlighter)
//! stay in sync without duplicating the token set.

/// The complete set of reserved keyword strings in the Phalcom language.
///
/// Used by both the completer (to seed identifier suggestions) and the
/// highlighter (to build the keyword-matching regex at startup).
pub const KEYWORDS: &[&str] = &[
    "class", "import", "for", "while", "if", "else", "return", "break", "continue", "true", "false", "nil", "let", "and", "or", "not", "self", "super", "in",
    "is", "as",
];
