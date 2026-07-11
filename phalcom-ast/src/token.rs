//! Lexical tokens and lexer-level errors for the Phalcom front end.
//!
//! [`Token`] is the alphabet the hand-written [`crate::lexer::Lexer`] produces
//! and the hand-written [`crate::parser`] consumes. Each variant corresponds to
//! one lexeme of Phalcom's surface syntax — a keyword, an operator, a
//! punctuation mark, or a literal (identifier / string / number). The set here
//! is deliberately kept at parity with the language the parser currently
//! accepts; see `docs/spec/lexical-structure.md` for the target lexical grammar
//! this will grow into.
//!
//! A [`LexicalError`] is raised when the scanner cannot form a token at some
//! byte position (an unrecognised character, or an unterminated string). It is
//! later lowered into a [`crate::error::SyntaxError`] by [`crate::parse_source`].

use std::num::{ParseFloatError, ParseIntError};
use std::ops::Range;

/// A single lexical token of Phalcom source.
///
/// Produced by [`crate::lexer::Lexer`] and matched by [`crate::parser`]. The
/// three data-carrying variants ([`Token::Identifier`], [`Token::String`],
/// [`Token::Number`]) hold the already-decoded literal value; every other
/// variant is a fixed keyword, operator, or punctuation mark.
///
/// The [`Debug`] representation is stable and is snapshotted by the lexer tests
/// (`phalcom-ast/tests/lexer.rs`); changing a variant name changes those
/// snapshots.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /// The `let` keyword introducing an immutable binding (ADR-0014).
    Let,
    /// The `var` keyword introducing a mutable binding (ADR-0014).
    ///
    /// Distinct from [`Token::Let`] only in mutability: `var x` with no
    /// initializer reads the surface `None` value, whereas `let x` with no
    /// initializer is rejected at compile time. See
    /// `docs/spec/values-and-absence.md` §3.1.
    Var,
    /// The `fn` keyword.
    Fn,
    /// The `class` keyword introducing a class declaration.
    Class,
    /// The `return` keyword.
    Return,
    /// The `true` boolean literal keyword.
    True,
    /// The `false` boolean literal keyword.
    False,
    /// The `if` keyword.
    If,
    /// The `else` keyword.
    Else,
    /// The `while` keyword.
    While,
    /// The `for` keyword.
    For,
    /// The `break` keyword.
    Break,
    /// The `continue` keyword.
    Continue,
    /// The `import` keyword.
    Import,
    /// The `self` keyword denoting the current receiver.
    SelfKw,
    /// The `super` keyword denoting the superclass receiver.
    Super,
    /// The `in` keyword.
    In,
    /// The `as` keyword.
    As,
    /// The `is` keyword.
    Is,
    /// The `and` short-circuiting conjunction keyword.
    And,
    /// The `or` short-circuiting disjunction keyword.
    Or,
    /// The `not` keyword.
    Not,
    /// The `static` keyword marking a class-side member.
    Static,
    /// The `construct` keyword introducing an initializer definition (ADR-0011).
    Construct,

    /// An identifier lexeme, e.g. `foo` or a field name like `_bar`.
    Identifier(String),
    /// A double-quoted string literal, with the surrounding quotes stripped.
    String(String),
    /// A numeric literal decoded to an [`f64`].
    Number(f64),

    /// The `(` delimiter.
    LParen,
    /// The `)` delimiter.
    RParen,
    /// The `{` delimiter.
    LBrace,
    /// The `}` delimiter.
    RBrace,
    /// The `[` delimiter.
    LBracket,
    /// The `]` delimiter.
    RBracket,

    /// The `=` assignment operator.
    Equal,
    /// The `==` equality operator.
    EqualEqual,
    /// The `!=` inequality operator.
    BangEqual,
    /// The `<` less-than operator.
    Less,
    /// The `<=` less-than-or-equal operator.
    LessEqual,
    /// The `>` greater-than operator.
    Greater,
    /// The `>=` greater-than-or-equal operator.
    GreaterEqual,

    /// The `+=` compound-assignment operator.
    PlusEqual,
    /// The `-=` compound-assignment operator.
    MinusEqual,
    /// The `*=` compound-assignment operator.
    AsteriskEqual,
    /// The `/=` compound-assignment operator.
    SlashEqual,
    /// The `%=` compound-assignment operator.
    PercentEqual,

    /// The `;` statement separator.
    Semicolon,
    /// A newline terminator (`\n` or `\r\n`).
    Newline,

    /// The `:` punctuation mark.
    Colon,
    /// The `::` punctuation mark.
    ColonColon,
    /// The `,` separator.
    Comma,
    /// The `.` member-access / property operator.
    Dot,
    /// The `..` range punctuation mark.
    DotDot,
    /// The `...` spread punctuation mark.
    DotDotDot,
    /// The `->` arrow punctuation mark.
    Arrow,
    /// The `=>` fat-arrow used by single-expression method bodies.
    FatArrow,
    /// The `?` punctuation mark.
    ///
    /// A lone `?` is reserved for a future ternary/try operator and is not part
    /// of any current production; the parser rejects it. See
    /// `docs/spec/lexical-structure.md` §9.
    Question,
    /// The `??` null-coalescing operator (ADR-0007).
    ///
    /// A low-precedence, right-associative binary operator; `a ?? b` desugars
    /// to `a.orElse { b }`. See `docs/spec/values-and-absence.md` §3.4 and
    /// `docs/spec/lexical-structure.md` §9.
    CoalesceQuestion,
    /// The `?.` optional-send operator (ADR-0007).
    ///
    /// A postfix member-access operator binding like `.`; `opt?.m(a)` desugars
    /// to `opt.map { x => x.m(a) }`. See `docs/spec/values-and-absence.md` §3.4
    /// and `docs/spec/lexical-structure.md` §9.
    QuestionDot,
    /// The `!` logical-not / prefix operator.
    Bang,
    /// The `@` punctuation mark.
    At,

    /// The `+` addition operator.
    Plus,
    /// The `-` subtraction / negation operator.
    Minus,
    /// The `*` multiplication operator.
    Asterisk,
    /// The `/` division operator.
    Slash,
    /// The `%` modulo operator.
    Percent,

    /// The synthetic end-of-file marker injected once after the last real
    /// token.
    ///
    /// The parser treats this as a normal token: an unexpected `Eof` surfaces as
    /// a [`crate::error::SyntaxErrorKind::UnrecognizedToken`] whose text is the
    /// empty string, matching the historical LALRPOP behaviour.
    Eof,
}

/// A lexer-level failure to form a token at a specific byte range.
///
/// Raised by [`crate::lexer::Lexer`] and lowered into a
/// [`crate::error::SyntaxError`] by [`crate::parse_source`]. The `Default`
/// variant ([`LexicalError::Invalid`]) exists only so the type can be used in
/// generic positions that require `Default`; it is never produced by the
/// scanner.
#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    /// An integer literal could not be parsed into its numeric value.
    InvalidInteger(ParseIntError),
    /// A floating-point literal could not be parsed into its numeric value.
    InvalidFloat(ParseFloatError),
    /// No valid token could be scanned starting at the given byte range.
    InvalidToken(Range<usize>),
    /// A string literal was opened but never closed before end-of-input.
    UnterminatedString(Range<usize>),
    /// A block comment `/* … */` was opened but never closed before
    /// end-of-input.
    ///
    /// The range spans from the opening `/*` to the current cursor (end of
    /// input). Lowered to [`crate::error::SyntaxErrorKind::UnterminatedComment`]
    /// by [`crate::parse_source`]. See `docs/spec/lexical-structure.md` and
    /// ADR-0016 (the hand-written scanner treats block comments as trivia).
    UnterminatedBlockComment(Range<usize>),

    /// Placeholder default; never produced by the scanner.
    #[default]
    Invalid,
}

impl From<ParseIntError> for LexicalError {
    /// Wraps an integer-parse failure as [`LexicalError::InvalidInteger`].
    fn from(err: ParseIntError) -> Self {
        LexicalError::InvalidInteger(err)
    }
}

impl From<ParseFloatError> for LexicalError {
    /// Wraps a float-parse failure as [`LexicalError::InvalidFloat`].
    fn from(err: ParseFloatError) -> Self {
        LexicalError::InvalidFloat(err)
    }
}
