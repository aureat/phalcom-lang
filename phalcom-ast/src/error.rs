//! Parse and lex diagnostics for the Phalcom front end.
//!
//! The parser ([`crate::parse_source`]) lowers a lexer-level
//! [`crate::token::LexicalError`] and its own grammar mismatches into a single
//! [`SyntaxError`] — a [`SyntaxErrorKind`] paired with the source byte
//! [`Range`] it covers. The range feeds the CLI's span-aware renderer
//! (`phalcom-core`'s `diagnostics::print_parse`), while [`SyntaxError`]'s
//! [`Display`] produces a compact, source-free one-liner for contexts that only
//! have the error value (for example anyhow's top-level `Error: {}`
//! formatting).
//!
//! Implements fix F9: [`SyntaxError`]'s [`Display`] renders a real spanned
//! diagnostic instead of panicking, so a parse error prints cleanly and the CLI
//! exits non-zero rather than aborting.

use std::fmt::{Display, Formatter};
use std::ops::Range;
use thiserror::Error;

/// A syntax error together with the source byte range it covers.
///
/// Produced by [`crate::parse_source`] from either a grammar-level parse error
/// or a lexer-level [`crate::token::LexicalError`]. The [`kind`](Self::kind)
/// carries the human-readable message (its `#[error(..)]` text) and
/// [`range`](Self::range) locates it in the original source so a caller can
/// render a spanned diagnostic.
#[derive(Debug, Error, Eq, PartialEq, Clone)]
pub struct SyntaxError {
    /// The specific category of syntax error and its display message.
    pub kind: SyntaxErrorKind,
    /// Half-open byte range `start..end` into the source that the error spans.
    ///
    /// A zero-width range (`start == end`) denotes a point location, such as
    /// end-of-file for [`SyntaxErrorKind::UnrecognizedEof`].
    pub range: Range<usize>,
}

impl Display for SyntaxError {
    /// Formats the error as `"<message> (at bytes <start>..<end>)"`.
    ///
    /// Renders the underlying [`SyntaxErrorKind`] message followed by the byte
    /// range, so the diagnostic still carries a span when printed without the
    /// surrounding source (e.g. anyhow's top-level `Error: {}`). Span-aware
    /// callers should prefer rendering [`kind`](Self::kind) against the source
    /// via `phalcom-core`'s `diagnostics::print_parse`.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (at bytes {}..{})", self.kind, self.range.start, self.range.end)
    }
}

/// Formats a set of expected tokens into a human-readable phrase.
///
/// Returns `"Expected <t>"` for a single token and
/// `"Expected one of <t1>, <t2>, …"` for several. Used by the `#[error(..)]`
/// messages of [`SyntaxErrorKind::UnrecognizedToken`] and
/// [`SyntaxErrorKind::UnrecognizedEof`].
pub fn format_expected(expected: &[String]) -> String {
    if expected.len() == 1 {
        format!("Expected {}", expected[0])
    } else {
        let mut s = String::from("Expected one of ");
        for (i, e) in expected.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(e);
        }
        s
    }
}

/// The category of a [`SyntaxError`], carrying its display message.
///
/// Each variant's `#[error(..)]` attribute supplies the message rendered by
/// [`SyntaxError`]'s [`Display`]. Variants fall into three groups: grammar
/// mismatches ([`ExtraToken`](Self::ExtraToken),
/// [`UnrecognizedToken`](Self::UnrecognizedToken),
/// [`UnrecognizedEof`](Self::UnrecognizedEof)), lexer failures
/// ([`InvalidToken`](Self::InvalidToken),
/// [`InvalidInteger`](Self::InvalidInteger), the `Unterminated*` variants), and
/// semantic checks the compiler raises after parsing
/// ([`ReturnOutsideFunction`](Self::ReturnOutsideFunction),
/// [`SelfOutsideClass`](Self::SelfOutsideClass), and friends).
#[derive(Debug, Clone, Error, Eq, PartialEq)]
pub enum SyntaxErrorKind {
    /// Input remained after a complete program was parsed.
    #[error("Extraneous input: {token:?}")]
    ExtraToken {
        /// The unexpected trailing token's source text.
        token: String,
    },

    /// The lexer could not form a valid token at this position.
    #[error("Invalid token")]
    InvalidToken,

    /// A class initializer (`init`) attempted to return a value.
    #[error(r#"init() should not return a value"#)]
    ReturnInInitializer,

    /// A `return` statement appeared outside of any function or method body.
    #[error(r#""return" used outside function"#)]
    ReturnOutsideFunction,

    /// `super` was used outside of a class body.
    #[error(r#""super" used outside class"#)]
    SuperOutsideClass,

    /// `super` was used in a class that has no superclass.
    #[error(r#""super" used in class without a superclass"#)]
    SuperWithoutSuperclass,

    /// `self` was used outside of a class body.
    #[error(r#""self" used outside class"#)]
    SelfOutsideClass,

    /// A token appeared where no grammar rule could accept it.
    #[error("Unexpected input: {token:?}")]
    UnexpectedInput {
        /// The offending token's source text.
        token: String,
    },

    /// Input ended while the parser still expected more tokens.
    #[error("Unexpected end of file. {}", format_expected(expected))]
    UnrecognizedEof {
        /// The tokens that would have been accepted at end-of-file.
        expected: Vec<String>,
    },

    /// A token was found where a different set of tokens was expected.
    #[error("{}", format_expected(expected))]
    UnrecognizedToken {
        /// The offending token's source text.
        token: String,
        /// The tokens that would have been accepted in its place.
        expected: Vec<String>,
    },

    /// An integer literal could not be parsed.
    #[error("Invalid integer")]
    InvalidInteger,

    /// A floating-point literal could not be parsed.
    #[error("Invalid integer")]
    InvalidFloat,

    /// A string literal was not closed before end-of-input.
    #[error("Unterminated string")]
    UnterminatedString,

    /// A block comment was not closed before end-of-input.
    #[error("Unterminated comment")]
    UnterminatedComment,

    /// An invalid escape sequence was encountered in a string literal.
    #[error("Invalid string escape")]
    InvalidStringEscape,

    /// An interpolation `\(` was not closed before end-of-input.
    #[error("Unterminated string interpolation")]
    UnterminatedInterpolation,

    /// String interpolation contains no expression.
    #[error("String interpolation requires an expression")]
    EmptyInterpolation,

    /// A unescaped physical newline occurred inside a double-quoted string.
    #[error("Raw newline is not allowed in a string literal")]
    RawNewlineInString,

    /// A syntax error with no more specific classification.
    #[error("Unknown error")]
    Unknown,

    /// A pre-formatted, free-form diagnostic message.
    #[error("{0}")]
    Message(String),
}

impl SyntaxErrorKind {
    /// Returns the stable diagnostic code for this error category.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::ExtraToken { .. } => "syntax.extra_token",
            Self::InvalidToken => "syntax.invalid_token",
            Self::ReturnInInitializer => "syntax.block.return_in_initializer",
            Self::ReturnOutsideFunction => "syntax.block.return_outside_function",
            Self::SuperOutsideClass => "syntax.class.super_outside_class",
            Self::SuperWithoutSuperclass => "syntax.class.super_without_superclass",
            Self::SelfOutsideClass => "syntax.class.self_outside_class",
            Self::UnexpectedInput { .. } => "syntax.unexpected_input",
            Self::UnrecognizedEof { .. } => "syntax.unrecognized_eof",
            Self::UnrecognizedToken { .. } => "syntax.unrecognized_token",
            Self::InvalidInteger => "number.invalid_integer",
            Self::InvalidFloat => "number.invalid_float",
            Self::UnterminatedString => "string.unterminated",
            Self::UnterminatedComment => "comment.unterminated",
            Self::InvalidStringEscape => "string.invalid_escape",
            Self::UnterminatedInterpolation => "string.interpolation.unterminated",
            Self::EmptyInterpolation => "string.interpolation.empty",
            Self::RawNewlineInString => "string.raw_newline",
            Self::Unknown => "syntax.unknown",
            Self::Message(_) => "syntax.message",
        }
    }
}

impl SyntaxError {
    /// Returns the stable diagnostic code for this syntax error.
    pub const fn code(&self) -> &'static str {
        self.kind.code()
    }
}
