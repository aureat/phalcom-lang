//! Hand-written recursive-descent + Pratt parser for Phalcom.
//!
//! This module replaces the previous LALRPOP-generated parser (see ADR-0016).
//! It is a two-stage front end:
//!
//! * **Scanning.** The whole source is tokenised up front via
//!   [`crate::lexer::Lexer`] into a flat vector of `(token, start, end)` triples
//!   (with any caller `offset` folded into every span). Lexer-level failures are
//!   collected as [`SyntaxError`]s rather than aborting.
//! * **Parsing.** Statements and declarations are parsed by straightforward
//!   recursive descent (`parse_program`, `parse_class`, ...), while expressions
//!   and message sends use precedence climbing (`parse_binary`) over a fixed
//!   operator-precedence table, with prefix handling for unary operators and
//!   postfix handling for `.property` member access / method calls.
//!
//! The grammar accepted here is kept at strict parity with what the old LALRPOP
//! grammar accepted (statements, `let`/`return`/expression statements, `class`
//! declarations with methods / getters / setters, and the full expression
//! grammar), plus the F10 fix: a source ending in a trailing newline (or an
//! empty / whitespace-only file) parses cleanly.
//!
//! ## Source spans
//!
//! Every AST node carries a [`SourceRange`].
//! A production spans from the start byte of its first token to the end byte of
//! its last consumed token, reproducing LALRPOP's `@L`/`@R` location semantics
//! so existing AST snapshots stay stable.
//!
//! ## Error recovery
//!
//! [`parse`] never aborts on the first error. When a top-level statement fails
//! to parse, the error is recorded and the parser *synchronises* — it skips
//! tokens up to the next statement boundary (a newline, `;`, `}`, or a
//! statement-introducing keyword) — then resumes. This lets a single call
//! surface multiple diagnostics. [`parse_source`] preserves the historical
//! single-error contract by returning the first collected error.
//!
//! See `docs/spec/lexical-structure.md` and `docs/spec/classes.md` for the
//! surface syntax realised here.

use crate::ast::*;
use crate::error::{RestParameterErrorKind, SyntaxError, SyntaxErrorKind};
use crate::lexer::Lexer;
use crate::token::{LexicalError, StringSegment, Token};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{SelectorKind, SelectorSlot};
use std::ops::Range;

/// The three pieces [`Parser::parse_class_body`] assembles a [`ClassDef`]
/// from: its members, its (currently always-empty, see that field's doc)
/// class-level attributes, and its standalone `@invariant(...)` predicates
/// (DEC-ANNOT-B).
type ClassBodyParts = (Vec<ClassMember>, Vec<Attribute>, Vec<(Expr, SourceRange)>);
type SelectorSpecSlots = (Vec<SelectorSlotSyntax>, Vec<SelectorSlotSyntax>, Option<SourceRange>, usize);

enum ProductLabelStart {
    Computed,
    ExplicitSymbol,
    BareName(String),
    BareSelector(String),
}

/// Whether a closure parser entry point accepts expression bodies or requires
/// braces. Trailing closures require braces so `|` remains unambiguous with
/// bitwise OR after a completed expression.
#[derive(Clone, Copy)]
enum ClosureBodyRequirement {
    Any,
    Braced,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TrailingTarget {
    None,
    MemberSend,
}

/// Context controlling whether variance markers (+/-) are permitted on generic binders (Spec 04 §6.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenericBinderContext {
    NominalDeclaration,
    Callable,
    Alias,
    TypeLambda,
}

/// Result of parsing a Phalcom source string with error recovery.
///
/// Carries the [`Program`] built from every statement that parsed successfully,
/// alongside the list of [`errors`](Parse::errors) recovered during parsing. An
/// empty `errors` vector means the source parsed cleanly.
#[derive(Debug)]
pub struct Parse {
    /// The parsed program. Statements that failed to parse are omitted; on a
    /// clean parse this is the complete program.
    pub program: Program,
    /// Every syntax error recovered during parsing, in the order discovered.
    pub errors: Vec<SyntaxError>,
}

/// The historical single-error parse result: `Ok(program)` or the first
/// [`SyntaxError`].
pub type ParserResult<T> = Result<T, SyntaxError>;

/// Parses `source` into a [`Program`], returning the first syntax error if any.
///
/// `offset` is added to every source span, so a snippet parsed out of a larger
/// file reports absolute byte positions. This is the entry point used by the
/// compiler and preserves the original contract: it returns `Ok` only when the
/// source parses with no errors, otherwise the first recovered error.
///
/// # Errors
///
/// Returns the first [`SyntaxError`] encountered (a lexer failure or a grammar
/// mismatch). Use [`parse`] to obtain the full list of recovered errors.
pub fn parse_source(source: &str, offset: usize) -> ParserResult<Program> {
    let result = parse(source, offset);
    match result.errors.into_iter().next() {
        Some(err) => Err(err),
        None => Ok(result.program),
    }
}

/// Parses `source` into a [`Parse`], recovering from errors to collect as many
/// diagnostics as possible.
///
/// `offset` is added to every source span. Unlike [`parse_source`], this always
/// returns a [`Parse`] whose [`program`](Parse::program) holds the
/// successfully-parsed statements and whose [`errors`](Parse::errors) lists
/// every recovered [`SyntaxError`].
pub fn parse(source: &str, offset: usize) -> Parse {
    let mut parser = Parser::new(source, offset);
    let program = parser.parse_program();
    Parse {
        program,
        errors: parser.errors,
    }
}

/// The set of terminals that may begin an expression, in the order LALRPOP
/// reported them.
///
/// Used verbatim as the `expected` list when a primary expression is required
/// but missing, keeping error snapshots stable.
fn primary_expected() -> Vec<String> {
    [
        "\"true\"",
        "\"false\"",
        "\"nil\"",
        "\"self\"",
        "\"super\"",
        "identifier",
        "string",
        "number",
        "\"(\"",
        "\"not\"",
        "\"-\"",
        "\"|\"",
        "\"{\"",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect()
}

/// Converts a slice of `&str` labels into an owned `expected` list.
fn strs(labels: &[&str]) -> Vec<String> {
    labels.iter().map(|s| (*s).to_string()).collect()
}

/// Lowers a lexer-level [`LexicalError`] into a [`SyntaxError`], applying
/// `offset` to any carried span.
fn lex_error_to_syntax(err: LexicalError, offset: usize) -> SyntaxError {
    match err {
        LexicalError::NumericLiteral(span) => SyntaxError {
            kind: SyntaxErrorKind::NumericLiteral,
            range: (span.start + offset)..(span.end + offset),
        },
        LexicalError::InvalidInteger(_) => SyntaxError {
            kind: SyntaxErrorKind::InvalidInteger,
            range: 0..0,
        },
        LexicalError::InvalidFloat(_) => SyntaxError {
            kind: SyntaxErrorKind::InvalidFloat,
            range: 0..0,
        },
        LexicalError::InvalidToken(span) => SyntaxError {
            kind: SyntaxErrorKind::InvalidToken,
            range: (span.start + offset)..(span.end + offset),
        },
        LexicalError::UnterminatedString(span) => SyntaxError {
            kind: SyntaxErrorKind::UnterminatedString,
            range: (span.start + offset)..(span.end + offset),
        },
        LexicalError::UnterminatedBlockComment(span) => SyntaxError {
            kind: SyntaxErrorKind::UnterminatedComment,
            range: (span.start + offset)..(span.end + offset),
        },
        LexicalError::InvalidEscape(span) => SyntaxError {
            kind: SyntaxErrorKind::InvalidStringEscape,
            range: (span.start + offset)..(span.end + offset),
        },
        LexicalError::UnterminatedInterpolation(span) => SyntaxError {
            kind: SyntaxErrorKind::UnterminatedInterpolation,
            range: (span.start + offset)..(span.end + offset),
        },
        LexicalError::RawNewlineInString(span) => SyntaxError {
            kind: SyntaxErrorKind::RawNewlineInString,
            range: (span.start + offset)..(span.end + offset),
        },
        LexicalError::UnterminatedMultilineString(span) => SyntaxError {
            kind: SyntaxErrorKind::UnterminatedMultilineString,
            range: (span.start + offset)..(span.end + offset),
        },
        LexicalError::InvalidMultilineStringOpening(span) => SyntaxError {
            kind: SyntaxErrorKind::InvalidMultilineStringOpening,
            range: (span.start + offset)..(span.end + offset),
        },
        LexicalError::InvalidMultilineStringIndentation(span) => SyntaxError {
            kind: SyntaxErrorKind::InvalidMultilineStringIndentation,
            range: (span.start + offset)..(span.end + offset),
        },
        LexicalError::InvalidMultilineStringLineEnding(span) => SyntaxError {
            kind: SyntaxErrorKind::InvalidMultilineStringLineEnding,
            range: (span.start + offset)..(span.end + offset),
        },
        LexicalError::Invalid => SyntaxError {
            kind: SyntaxErrorKind::InvalidToken,
            range: 0..0,
        },
    }
}

/// Lowers a [`LexicalError`] into the parser's diagnostic list, co-emitting
/// [`SyntaxErrorKind::UnrecognizedEof`] when input ended inside an open lexer mode.
///
/// Most lexical errors lower to exactly one [`SyntaxError`]. An *unterminated*
/// construct is different: it means the author has not finished typing, which is a
/// continuable state — but the grammar cannot see it. A bare `/* …` produces an
/// empty statement list, which is a **grammatically complete parse**, so the parser
/// never wants another token and never emits `UnrecognizedEof` on its own.
///
/// Co-emitting one here is what lets a consumer's completeness rule stay a single
/// question about the parser ("did any error say `UnrecognizedEof`?") instead of
/// growing a second, drifting model of the grammar. The REPL's validator is
/// unchanged by this
/// ([PDR-0006](../../docs/decisions/0006-repl-completeness-is-a-parser-signal.md) §1–2).
///
/// The obligation is over *modes*, not over today's two cases: **any** future
/// heredoc, raw string, or nested-interpolation mode must co-emit here too, or it
/// silently stops continuing in the REPL (PDR-0006 §3).
fn push_lex_error(errors: &mut Vec<SyntaxError>, err: LexicalError, offset: usize) {
    let expected_closer = match &err {
        LexicalError::UnterminatedBlockComment(_) => Some("`*/`"),
        LexicalError::UnterminatedString(_) => Some("`\"`"),
        LexicalError::UnterminatedInterpolation(_) => Some("`)`"),
        LexicalError::UnterminatedMultilineString(_) => Some("`\"\"\"`"),
        _ => None,
    };

    let lowered = lex_error_to_syntax(err, offset);
    // End-of-input is a point, not a span — the convention `SyntaxError::range`
    // documents for `UnrecognizedEof`.
    let eof_point = lowered.range.end..lowered.range.end;
    errors.push(lowered);

    if let Some(closer) = expected_closer {
        errors.push(SyntaxError {
            kind: SyntaxErrorKind::UnrecognizedEof {
                expected: vec![closer.to_string()],
            },
            range: eof_point,
        });
    }
}

/// A single scanned lexeme: a token plus its half-open byte span.
struct Lexeme {
    /// The token value.
    token: Token,
    /// Inclusive start byte offset (already offset-adjusted).
    start: usize,
    /// Exclusive end byte offset (already offset-adjusted).
    end: usize,
}

/// A recursive-descent + precedence-climbing parser over a pre-tokenised source.
///
/// Constructed by [`parse`] / [`parse_source`]; not part of the public API. The
/// token vector always ends in a single [`Token::Eof`] sentinel, so lookahead
/// never indexes out of bounds.
struct Parser<'source> {
    /// The original source, for slicing offending token text into diagnostics.
    source: &'source str,
    /// Byte offset folded into every span (see [`parse_source`]).
    offset: usize,
    /// The scanned tokens, terminated by an [`Token::Eof`] sentinel.
    tokens: Vec<Lexeme>,
    /// Index of the current lookahead token in [`Parser::tokens`].
    pos: usize,
    /// End byte offset of the most recently consumed token (LALRPOP `@R`).
    prev_end: usize,
    /// Whether expressions are being parsed inside a class declaration body.
    ///
    /// `class` is an implicit `self.class` variable only in this context. At
    /// module level it remains a bare name, which the compiler/runtime can
    /// reject as an undefined variable instead of silently binding it to
    /// `self.class`.
    in_class_body: bool,
    /// Ranges whose expression was explicitly wrapped in parentheses. The
    /// AST intentionally keeps ordinary grouping transparent, but a small
    /// amount of syntax context is needed to distinguish `(a <=> b) === c`
    /// from the forbidden unparenthesized `a <=> b === c` chain.
    parenthesized_ranges: Vec<(usize, usize)>,
    /// Disabled while parsing a refutable-header scrutinee so its following
    /// body brace cannot be consumed as a trailing closure argument.
    trailing_closures_enabled: bool,
    /// Syntax errors recovered so far, in discovery order.
    errors: Vec<SyntaxError>,
}

impl<'source> Parser<'source> {
    /// Scans `source` and builds a parser positioned at the first token.
    ///
    /// Lexer errors are recorded into [`Parser::errors`] up front; a defensive
    /// [`Token::Eof`] sentinel is appended if the scanner did not already
    /// produce one.
    fn new(source: &'source str, offset: usize) -> Self {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        for item in Lexer::new(source) {
            match item {
                Ok((start, token, end)) => tokens.push(Lexeme {
                    token,
                    start: start + offset,
                    end: end + offset,
                }),
                Err(err) => push_lex_error(&mut errors, err, offset),
            }
        }
        if !matches!(tokens.last(), Some(Lexeme { token: Token::Eof, .. })) {
            let end = tokens.last().map_or(offset, |l| l.end);
            tokens.push(Lexeme {
                token: Token::Eof,
                start: end,
                end,
            });
        }
        Self {
            source,
            offset,
            tokens,
            pos: 0,
            prev_end: offset,
            in_class_body: false,
            parenthesized_ranges: Vec::new(),
            trailing_closures_enabled: true,
            errors,
        }
    }

    // ── Token cursor ────────────────────────────────────────────────────────

    /// Returns the current lookahead token without consuming it.
    fn peek(&self) -> &Token {
        &self.tokens[self.pos].token
    }

    /// Returns the token immediately following the current lookahead token without consuming it.
    fn peek_next(&self) -> &Token {
        if self.pos + 1 < self.tokens.len() {
            &self.tokens[self.pos + 1].token
        } else {
            &Token::Eof
        }
    }

    /// Returns the start byte offset of the current lookahead token.
    fn cur_start(&self) -> usize {
        self.tokens[self.pos].start
    }

    /// Returns the source text of the current lookahead token (empty for
    /// [`Token::Eof`]).
    fn cur_text(&self) -> String {
        let lexeme = &self.tokens[self.pos];
        self.source[(lexeme.start - self.offset)..(lexeme.end - self.offset)].to_string()
    }

    /// Returns whether source contains a physical line break between the
    /// previously consumed token and current lookahead. Lexer D3 can suppress
    /// that newline when previous token is `>`, which is also a generic-type
    /// closer; class-member parsing still needs to recognize declaration
    /// termination before a following attribute.
    fn has_source_newline_before_current(&self) -> bool {
        let start = self.prev_end.saturating_sub(self.offset);
        let end = self.cur_start().saturating_sub(self.offset);
        self.source.get(start..end).is_some_and(|gap| gap.contains('\n'))
    }

    /// Consumes and returns the current token, advancing the cursor.
    ///
    /// The cursor never advances past the [`Token::Eof`] sentinel;
    /// [`Parser::prev_end`] is updated to the consumed token's end.
    fn advance(&mut self) -> Token {
        let lexeme = &self.tokens[self.pos];
        self.prev_end = lexeme.end;
        let token = lexeme.token.clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    /// Consumes the current token if it equals `token`, reporting `true` on a
    /// match.
    fn eat(&mut self, token: &Token) -> bool {
        if self.peek() == token {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Requires the current token to equal `token`, consuming it.
    ///
    /// # Errors
    ///
    /// Returns an [`SyntaxErrorKind::UnrecognizedToken`] carrying `expected` if
    /// the lookahead token does not match.
    fn expect(&mut self, token: &Token, expected: &[&str]) -> ParserResult<()> {
        if self.peek() == token {
            self.advance();
            Ok(())
        } else {
            Err(self.error_here(strs(expected)))
        }
    }

    /// Builds the syntax error for the current token, carrying `expected`.
    ///
    /// At [`Token::Eof`] this is [`SyntaxErrorKind::UnrecognizedEof`]; at any
    /// other token it is [`SyntaxErrorKind::UnrecognizedToken`].
    ///
    /// Routing the two apart is what makes "the input is truncated" a named,
    /// testable signal instead of a check for an empty token text (U-REPL §D7).
    /// It also reads better: `class Foo {` reported `Expected "}"`, which says
    /// nothing about *where* the parser ran out; it now reports `Unexpected end
    /// of file. Expected "}"`.
    fn error_here(&self, expected: Vec<String>) -> SyntaxError {
        let lexeme = &self.tokens[self.pos];
        let kind = if matches!(self.peek(), Token::Eof) {
            SyntaxErrorKind::UnrecognizedEof { expected }
        } else {
            SyntaxErrorKind::UnrecognizedToken {
                token: self.cur_text(),
                expected,
            }
        };
        SyntaxError {
            kind,
            range: lexeme.start..lexeme.end,
        }
    }

    /// Consumes a `>` token, handling nested `>>` token fission when needed (Spec 04 §15.5).
    fn eat_greater(&mut self) -> bool {
        if matches!(self.peek(), Token::Greater) {
            self.advance();
            true
        } else if matches!(self.peek(), Token::ShiftRight) {
            let lex = &mut self.tokens[self.pos];
            let start = lex.start;
            let mid = start + 1;
            lex.start = mid;
            lex.token = Token::Greater;
            self.prev_end = mid;
            true
        } else {
            false
        }
    }

    /// Requires the current token to be `>`, consuming it (with `>>` fission support).
    fn expect_greater(&mut self) -> ParserResult<()> {
        if self.eat_greater() {
            Ok(())
        } else {
            Err(self.error_here(vec!["\">\"".to_string()]))
        }
    }

    /// Checks if a type lambda `<...>` followed by `=>>` is immediately ahead.
    fn is_type_lambda_ahead(&self) -> bool {
        if self.peek() != &Token::Less {
            return false;
        }
        let mut depth: i32 = 0;
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i].token {
                Token::Less => depth += 1,
                Token::Greater => {
                    depth -= 1;
                    if depth == 0 {
                        return i + 1 < self.tokens.len() && matches!(self.tokens[i + 1].token, Token::TypeLambdaArrow);
                    }
                }
                Token::ShiftRight => {
                    depth -= 2;
                    if depth <= 0 {
                        return i + 1 < self.tokens.len() && matches!(self.tokens[i + 1].token, Token::TypeLambdaArrow);
                    }
                }
                Token::Newline | Token::Semicolon | Token::Eof | Token::LBrace | Token::RBrace => return false,
                _ => {}
            }
            i += 1;
        }
        false
    }

    /// Checks if a balanced angle bracket list `<...>` is ahead.
    fn is_type_arguments_ahead(&self) -> bool {
        if self.peek() != &Token::Less {
            return false;
        }
        let mut depth: i32 = 0;
        let mut i = self.pos;
        while i < self.tokens.len() {
            match &self.tokens[i].token {
                Token::Less => depth += 1,
                Token::Greater => {
                    depth -= 1;
                    if depth == 0 {
                        return true;
                    }
                }
                Token::ShiftRight => {
                    depth -= 2;
                    if depth <= 0 {
                        return true;
                    }
                }
                Token::Newline | Token::Semicolon | Token::Eof | Token::LBrace | Token::RBrace => return false,
                _ => {}
            }
            i += 1;
        }
        false
    }

    /// Skips any run of [`Token::Newline`] tokens (blank lines).
    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }
    }

    // ── Program / statements ─────────────────────────────────────────────────

    /// Parses a whole program in three distinct structural phases:
    /// 1. Module/package header attributes (`@!`)
    /// 2. Dependency preamble (`import`, `from ... import`, direct `export ... from`, `expose`)
    /// 3. Ordinary module body statements (including local `export Name`)
    fn parse_program(&mut self) -> Program {
        let mut preamble = ModulePreamble::default();

        // 1. Consume module/package header attributes (@!)
        let header_start = self.cur_start();
        match self.parse_module_metadata_header() {
            Ok(metadata) => {
                preamble.metadata = metadata;
            }
            Err(err) => {
                self.errors.push(err);
                self.synchronize();
            }
        }

        // 2. Consume dependency preamble (import, from, direct export ... from, expose)
        match self.parse_module_preamble_deps() {
            Ok(deps) => {
                preamble.dependencies = deps;
            }
            Err(err) => {
                self.errors.push(err);
                self.synchronize();
            }
        }
        preamble.range = (header_start..self.prev_end).into();

        // 3. Parse ordinary module body statements
        let mut statements = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::Eof) {
                break;
            }
            let before = self.pos;
            if let Err(err) = self.parse_top_item(&mut statements) {
                self.errors.push(err);
                self.synchronize();
                // Guarantee forward progress so recovery cannot loop forever.
                if self.pos == before && !matches!(self.peek(), Token::Eof) {
                    self.advance();
                }
            }
        }

        Program { preamble, statements }
    }

    /// Consumes leading `@!` module/package header attributes.
    fn parse_module_metadata_header(&mut self) -> ParserResult<Vec<ModuleMetadataAttribute>> {
        let mut metadata = Vec::new();
        loop {
            self.skip_newlines();
            if !matches!(self.peek(), Token::AtBang) {
                break;
            }
            let start = self.cur_start();
            self.advance(); // consume '@!'
            let name = self.expect_identifier(&["attribute name"])?;
            let arguments = if self.eat(&Token::LParen) {
                let args = self.parse_metadata_argument_list()?;
                self.expect(&Token::RParen, &["\")\""])?;
                args
            } else {
                Vec::new()
            };
            let range = (start..self.prev_end).into();
            metadata.push(ModuleMetadataAttribute { name, arguments, range });
            self.skip_newlines();
        }
        Ok(metadata)
    }

    /// Parses a parenthesized list of metadata literal arguments.
    /// If named arguments like `key: val` are present, parses them as a record argument `MetadataLiteral::Record`.
    fn parse_metadata_argument_list(&mut self) -> ParserResult<Vec<MetadataLiteral>> {
        self.skip_newlines();
        if matches!(self.peek(), Token::RParen) {
            return Ok(Vec::new());
        }
        // If first token is `ident:` (named record args for the attribute):
        if matches!(self.peek(), Token::Identifier(_)) && matches!(self.peek_next(), Token::Colon) {
            let mut fields = Vec::new();
            loop {
                self.skip_newlines();
                let key = self.expect_identifier(&["record field key"])?;
                self.expect(&Token::Colon, &["\":\""])?;
                self.skip_newlines();
                let val = self.parse_metadata_literal()?;
                fields.push((key, val));
                self.skip_newlines();
                if !self.eat(&Token::Comma) {
                    break;
                }
                if matches!(self.peek(), Token::RParen) {
                    break;
                }
            }
            return Ok(vec![MetadataLiteral::Record(fields)]);
        }

        let mut args = Vec::new();
        loop {
            self.skip_newlines();
            args.push(self.parse_metadata_literal()?);
            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                break;
            }
            if matches!(self.peek(), Token::RParen) {
                break;
            }
        }
        Ok(args)
    }

    /// Parses an inert metadata literal (unit, bool, int, float, string, symbol, tuple, record).
    fn parse_metadata_literal(&mut self) -> ParserResult<MetadataLiteral> {
        self.skip_newlines();
        match self.peek().clone() {
            Token::True => {
                self.advance();
                Ok(MetadataLiteral::Bool(true))
            }
            Token::False => {
                self.advance();
                Ok(MetadataLiteral::Bool(false))
            }
            Token::Int { digits, radix: _ } => {
                self.advance();
                Ok(MetadataLiteral::Int(digits))
            }
            Token::Float(f) => {
                self.advance();
                Ok(MetadataLiteral::Float(f))
            }
            Token::String(s) => {
                self.advance();
                Ok(MetadataLiteral::String(s))
            }
            Token::Hash => {
                self.advance(); // '#'
                let sym = self.expect_identifier(&["symbol name"])?;
                Ok(MetadataLiteral::Symbol(sym))
            }
            Token::QuotedSymbol(s) => {
                self.advance();
                Ok(MetadataLiteral::Symbol(s))
            }
            Token::LParen => {
                self.advance(); // '('
                self.skip_newlines();
                if self.eat(&Token::RParen) {
                    return Ok(MetadataLiteral::Unit);
                }
                // Could be a tuple `(a, b)` or `(key: val, ...)`
                // Check if first element is a record field `ident:`
                if matches!(self.peek(), Token::Identifier(_)) && matches!(self.peek_next(), Token::Colon) {
                    let mut fields = Vec::new();
                    loop {
                        self.skip_newlines();
                        let key = self.expect_identifier(&["record field key"])?;
                        self.expect(&Token::Colon, &["\":\""])?;
                        self.skip_newlines();
                        let val = self.parse_metadata_literal()?;
                        fields.push((key, val));
                        self.skip_newlines();
                        if !self.eat(&Token::Comma) {
                            break;
                        }
                        if matches!(self.peek(), Token::RParen) {
                            break; // trailing comma
                        }
                    }
                    self.expect(&Token::RParen, &["\")\""])?;
                    Ok(MetadataLiteral::Record(fields))
                } else {
                    let mut elements = Vec::new();
                    loop {
                        self.skip_newlines();
                        elements.push(self.parse_metadata_literal()?);
                        self.skip_newlines();
                        if !self.eat(&Token::Comma) {
                            break;
                        }
                        if matches!(self.peek(), Token::RParen) {
                            break; // trailing comma
                        }
                    }
                    self.expect(&Token::RParen, &["\")\""])?;
                    Ok(MetadataLiteral::Tuple(elements))
                }
            }
            Token::RecordLBrace => {
                self.advance(); // '#{'
                let mut fields = Vec::new();
                loop {
                    self.skip_newlines();
                    if matches!(self.peek(), Token::RBrace) {
                        break;
                    }
                    let key = self.expect_identifier(&["record field key"])?;
                    self.expect(&Token::Colon, &["\":\""])?;
                    self.skip_newlines();
                    let val = self.parse_metadata_literal()?;
                    fields.push((key, val));
                    self.skip_newlines();
                    if !self.eat(&Token::Comma) {
                        break;
                    }
                }
                self.expect(&Token::RBrace, &["\"}\""])?;
                Ok(MetadataLiteral::Record(fields))
            }
            _ => Err(self.error_here(strs(&["metadata literal (bool, number, string, symbol, tuple, record)"]))),
        }
    }

    /// Consumes preamble dependency declarations (`import`, `from`, `export ... from`, `expose`).
    fn parse_module_preamble_deps(&mut self) -> ParserResult<Vec<DependencyDecl>> {
        let mut deps = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek() {
                Token::Import => {
                    let decl = self.parse_module_import()?;
                    deps.push(DependencyDecl::Import(decl));
                }
                Token::From => {
                    let decl = self.parse_selective_import()?;
                    deps.push(DependencyDecl::Import(decl));
                }
                Token::Expose => {
                    let decl = self.parse_expose_decl()?;
                    deps.push(DependencyDecl::Expose(decl));
                }
                Token::Export => {
                    // Check if direct re-export: `export ... from .path` vs local export: `export Name`
                    // We lookahead: if after items there is `from`, it is ReExport.
                    // If not, this terminates preamble and begins body.
                    if self.is_direct_reexport_ahead() {
                        let decl = self.parse_reexport_decl()?;
                        deps.push(DependencyDecl::ReExport(decl));
                    } else {
                        break;
                    }
                }
                Token::AtBang => {
                    let start = self.cur_start();
                    self.advance();
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message(
                            "module.attribute_outside_header: @! attributes must appear at the top of the file before imports".to_string(),
                        ),
                        range: start..self.prev_end,
                    });
                }
                _ => break,
            }
            // Consume terminating newline or semicolon after preamble declaration
            match self.peek() {
                Token::Newline | Token::Semicolon => {
                    self.advance();
                }
                Token::Eof => break,
                _ => return Err(self.error_here(strs(&["newline", "\";\""]))),
            }
        }
        Ok(deps)
    }

    /// Lookahead helper to distinguish `export Item from .path` (preamble) from `export Item` (body).
    fn is_direct_reexport_ahead(&self) -> bool {
        let mut p = self.pos + 1; // skip Token::Export
        // Skip over ( items... ) or single/multiple items up to newline/semicolon/EOF
        let mut paren_depth = 0;
        while p < self.tokens.len() {
            let tok = &self.tokens[p].token;
            match tok {
                Token::LParen => paren_depth += 1,
                Token::RParen => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                }
                Token::From if paren_depth == 0 => return true,
                Token::Newline | Token::Semicolon | Token::Eof if paren_depth == 0 => return false,
                _ => {}
            }
            p += 1;
        }
        false
    }

    /// Parses an import path: absolute `geometry.point` or relative `.point`, `..units`.
    fn parse_import_path(&mut self) -> ParserResult<ImportPath> {
        let start = self.cur_start();
        let root = if matches!(self.peek(), Token::Dot | Token::DotDot | Token::DotDotDot) {
            let dots_start = self.cur_start();
            let mut dots: u16 = 0;
            while matches!(self.peek(), Token::Dot | Token::DotDot | Token::DotDotDot) {
                match self.peek() {
                    Token::Dot => {
                        dots += 1;
                        self.advance();
                    }
                    Token::DotDot => {
                        dots += 2;
                        self.advance();
                    }
                    Token::DotDotDot => {
                        dots += 3;
                        self.advance();
                    }
                    _ => unreachable!(),
                }
            }
            let dots_range = (dots_start..self.prev_end).into();
            ImportRoot::Relative { dots, range: dots_range }
        } else {
            let seg_start = self.cur_start();
            let name = self.expect_component_identifier(&["module path root"])?;
            let seg_range = (seg_start..self.prev_end).into();
            ImportRoot::Absolute(PathSegment { name, range: seg_range })
        };

        let mut segments = Vec::new();
        // If relative with 1 or more dots, the next identifier is the first segment (if present)
        if matches!(root, ImportRoot::Relative { .. }) && Self::label_name(self.peek()).is_some() {
            let seg_start = self.cur_start();
            let name = self.expect_component_identifier(&["path segment"])?;
            let seg_range = (seg_start..self.prev_end).into();
            segments.push(PathSegment { name, range: seg_range });
        }

        while self.eat(&Token::Dot) {
            let seg_start = self.cur_start();
            let name = self.expect_component_identifier(&["path segment"])?;
            let seg_range = (seg_start..self.prev_end).into();
            segments.push(PathSegment { name, range: seg_range });
        }

        let range = (start..self.prev_end).into();
        Ok(ImportPath { root, segments, range })
    }

    /// Parses whole-module import: `import path (as Alias)?`.
    fn parse_module_import(&mut self) -> ParserResult<ImportDecl> {
        let start = self.cur_start();
        self.advance(); // consume 'import'
        if matches!(self.peek(), Token::String(_)) {
            return Err(SyntaxError {
                kind: SyntaxErrorKind::Message(
                    "physical string imports `import \"...\"` have been retired; use logical imports like `import geometry.point`".to_string(),
                ),
                range: start..self.tokens[self.pos].end,
            });
        }
        let path = self.parse_import_path()?;
        let alias = if self.eat(&Token::As) {
            let alias_start = self.cur_start();
            let name = self.expect_identifier(&["import alias"])?;
            let range = (alias_start..self.prev_end).into();
            Some(ImportAlias { name, range })
        } else {
            None
        };
        let range = (start..self.prev_end).into();
        Ok(ImportDecl::Module(ModuleImportDecl { path, alias, range }))
    }

    /// Parses selective import: `from path import (Item, ...) | Item, ...`.
    fn parse_selective_import(&mut self) -> ParserResult<ImportDecl> {
        let start = self.cur_start();
        self.advance(); // consume 'from'
        let path = self.parse_import_path()?;
        self.expect(&Token::Import, &["\"import\""])?;
        let items = self.parse_import_items()?;
        let range = (start..self.prev_end).into();
        Ok(ImportDecl::Selective(SelectiveImportDecl { path, items, range }))
    }

    /// Parses import items list: flat or grouped `( ... )`.
    fn parse_import_items(&mut self) -> ParserResult<Vec<ImportItem>> {
        let mut items = Vec::new();
        if self.eat(&Token::LParen) {
            self.skip_newlines();
            while !matches!(self.peek(), Token::RParen | Token::Eof) {
                let item_start = self.cur_start();
                let name_start = self.cur_start();
                let name = self.expect_identifier(&["imported item name"])?;
                let name_range = (name_start..self.prev_end).into();
                let alias = if self.eat(&Token::As) {
                    let alias_start = self.cur_start();
                    let alias_name = self.expect_identifier(&["import alias"])?;
                    let alias_range = (alias_start..self.prev_end).into();
                    Some(ImportAlias {
                        name: alias_name,
                        range: alias_range,
                    })
                } else {
                    None
                };
                let item_range = (item_start..self.prev_end).into();
                items.push(ImportItem {
                    name,
                    name_range,
                    alias,
                    range: item_range,
                });
                self.skip_newlines();
                if !self.eat(&Token::Comma) {
                    break;
                }
                self.skip_newlines();
            }
            self.skip_newlines();
            self.expect(&Token::RParen, &["\")\""])?;
        } else {
            loop {
                let item_start = self.cur_start();
                let name_start = self.cur_start();
                let name = self.expect_identifier(&["imported item name"])?;
                let name_range = (name_start..self.prev_end).into();
                let alias = if self.eat(&Token::As) {
                    let alias_start = self.cur_start();
                    let alias_name = self.expect_identifier(&["import alias"])?;
                    let alias_range = (alias_start..self.prev_end).into();
                    Some(ImportAlias {
                        name: alias_name,
                        range: alias_range,
                    })
                } else {
                    None
                };
                let item_range = (item_start..self.prev_end).into();
                items.push(ImportItem {
                    name,
                    name_range,
                    alias,
                    range: item_range,
                });
                if !self.eat(&Token::Comma) {
                    break;
                }
            }
        }
        Ok(items)
    }

    /// Parses direct re-export: `export Item, ... from path`.
    fn parse_reexport_decl(&mut self) -> ParserResult<ReExportDecl> {
        let start = self.cur_start();
        self.advance(); // consume 'export'
        let items = self.parse_export_items()?;
        self.expect(&Token::From, &["\"from\""])?;
        let path = self.parse_import_path()?;
        let range = (start..self.prev_end).into();
        Ok(ReExportDecl { path, items, range })
    }

    /// Parses local export declaration in body: `export Item, ...`.
    fn parse_export_decl(&mut self) -> ParserResult<ExportDecl> {
        let start = self.cur_start();
        self.advance(); // consume 'export'
        let items = self.parse_export_items()?;
        let range = (start..self.prev_end).into();
        Ok(ExportDecl { items, range })
    }

    /// Parses export items list: flat or grouped `( ... )`.
    fn parse_export_items(&mut self) -> ParserResult<Vec<ExportItem>> {
        let mut items = Vec::new();
        if self.eat(&Token::LParen) {
            self.skip_newlines();
            while !matches!(self.peek(), Token::RParen | Token::Eof) {
                let item_start = self.cur_start();
                let name_start = self.cur_start();
                let local_or_remote_name = self.expect_identifier(&["exported item name"])?;
                let name_range = (name_start..self.prev_end).into();
                let alias = if self.eat(&Token::As) {
                    let alias_start = self.cur_start();
                    let alias_name = self.expect_identifier(&["export alias"])?;
                    let alias_range = (alias_start..self.prev_end).into();
                    Some(ExportAlias {
                        name: alias_name,
                        range: alias_range,
                    })
                } else {
                    None
                };
                let item_range = (item_start..self.prev_end).into();
                items.push(ExportItem {
                    local_or_remote_name,
                    name_range,
                    alias,
                    range: item_range,
                });
                self.skip_newlines();
                if !self.eat(&Token::Comma) {
                    break;
                }
                self.skip_newlines();
            }
            self.skip_newlines();
            self.expect(&Token::RParen, &["\")\""])?;
        } else {
            loop {
                let item_start = self.cur_start();
                let name_start = self.cur_start();
                let local_or_remote_name = self.expect_identifier(&["exported item name"])?;
                let name_range = (name_start..self.prev_end).into();
                let alias = if self.eat(&Token::As) {
                    let alias_start = self.cur_start();
                    let alias_name = self.expect_identifier(&["export alias"])?;
                    let alias_range = (alias_start..self.prev_end).into();
                    Some(ExportAlias {
                        name: alias_name,
                        range: alias_range,
                    })
                } else {
                    None
                };
                let item_range = (item_start..self.prev_end).into();
                items.push(ExportItem {
                    local_or_remote_name,
                    name_range,
                    alias,
                    range: item_range,
                });
                if !self.eat(&Token::Comma) {
                    break;
                }
            }
        }
        Ok(items)
    }

    /// Parses path exposure declaration: `expose .child` (package.ph only).
    fn parse_expose_decl(&mut self) -> ParserResult<ExposeDecl> {
        let start = self.cur_start();
        self.advance(); // consume 'expose'

        // Operand must be a single dot followed by single immediate child identifier
        if matches!(self.peek(), Token::DotDot | Token::DotDotDot) {
            let err_start = self.cur_start();
            self.advance();
            return Err(SyntaxError {
                kind: SyntaxErrorKind::Message("expose operand cannot ascend with `..`; must be immediate child `.child`".to_string()),
                range: err_start..self.prev_end,
            });
        }

        if !self.eat(&Token::Dot) {
            return Err(SyntaxError {
                kind: SyntaxErrorKind::Message("expose operand must be an immediate child starting with `.`, e.g. `expose .child`".to_string()),
                range: start..self.tokens[self.pos].end,
            });
        }

        let child_start = self.cur_start();
        let name = self.expect_component_identifier(&["immediate child name"])?;
        let child_range = (child_start..self.prev_end).into();

        if matches!(self.peek(), Token::Dot) {
            self.advance();
            return Err(SyntaxError {
                kind: SyntaxErrorKind::Message("expose operand must be a single immediate child segment (cannot be multi-segment like `.a.b`)".to_string()),
                range: child_start..self.tokens[self.pos].end,
            });
        }

        let child = PathSegment { name, range: child_range };
        let range = (start..self.prev_end).into();
        Ok(ExposeDecl { child, range })
    }

    /// Parses one top-level item into `out`.
    ///
    /// A `class` declaration (optionally preceded by header attributes,
    /// U-ANNOT-LAYOUT §3.3 — `Token::At` also dispatches here, since nothing
    /// else in top-level position starts with `@`) is a compound statement
    /// that must be followed by a newline terminator (never end-of-file);
    /// anything else is a run of small statements separated by `;` and
    /// terminated by a newline or end-of-file.
    ///
    /// # Errors
    ///
    /// Returns a [`SyntaxError`] if the item or its terminator is malformed.
    fn parse_top_item(&mut self, out: &mut Vec<Statement>) -> ParserResult<()> {
        let mut header_attrs = Vec::new();
        while matches!(self.peek(), Token::At) {
            header_attrs.push(self.parse_attribute()?);
            self.skip_newlines();
        }
        if !header_attrs.is_empty() || (matches!(self.peek(), Token::Class | Token::Enum) && matches!(self.peek_next(), Token::Identifier(_))) {
            let stmt = if matches!(self.peek(), Token::Enum) {
                self.parse_enum(header_attrs)?
            } else {
                self.parse_class(header_attrs)?
            };
            out.push(stmt);
            // A compound statement requires a NEWLINE terminator, not EOF.
            self.expect(&Token::Newline, &["newline"])?;
            Ok(())
        } else {
            loop {
                let stmt = self.parse_small_statement()?;
                out.push(stmt);
                if matches!(self.peek(), Token::Semicolon) {
                    self.advance();
                    if matches!(self.peek(), Token::Newline | Token::Eof) {
                        break; // trailing ';'
                    }
                } else {
                    break;
                }
            }
            match self.peek() {
                Token::Newline => {
                    self.advance();
                    Ok(())
                }
                Token::Eof => Ok(()),
                _ => Err(self.error_here(strs(&["\";\"", "newline"]))),
            }
        }
    }

    /// Skips tokens until the next plausible statement boundary.
    ///
    /// Consumes a terminating newline or `;` (so the next statement starts
    /// fresh) and stops — without consuming — at a `}`, end-of-file, or a
    /// statement-introducing keyword (`class`, `enum`, `let`, `return`).
    fn synchronize(&mut self) {
        loop {
            match self.peek() {
                Token::Eof | Token::RBrace => return,
                Token::Newline | Token::Semicolon => {
                    self.advance();
                    return;
                }
                Token::Class | Token::Enum | Token::TypeKw | Token::Let | Token::Const | Token::Return | Token::Import | Token::Export => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Parses a small (single-line) statement: `let`, `return`, `export`, or an
    /// expression statement.
    ///
    /// # Errors
    ///
    /// Propagates any error from the underlying statement parser.
    fn parse_small_statement(&mut self) -> ParserResult<Statement> {
        match self.peek() {
            Token::TypeKw => self.parse_type_alias(),
            Token::Let => self.parse_binding(BindingKind::Let),
            Token::Const => self.parse_binding(BindingKind::Const),
            Token::Return => self.parse_return(),
            Token::For => self.parse_for(),
            Token::Throw => self.parse_throw(),
            Token::Try => self.parse_try(),
            Token::Export => {
                let export_decl = self.parse_export_decl()?;
                Ok(Statement::Export(export_decl))
            }
            Token::Import | Token::From | Token::Expose => {
                let start = self.cur_start();
                let tok_name = match self.peek() {
                    Token::Import => "import",
                    Token::From => "from ... import",
                    Token::Expose => "expose",
                    _ => unreachable!(),
                };
                self.advance();
                Err(SyntaxError {
                    kind: SyntaxErrorKind::Message(format!(
                        "import.outside_preamble: static `{}` declarations must appear in the module dependency preamble at the top of the file",
                        tok_name
                    )),
                    range: start..self.prev_end,
                })
            }
            Token::AtBang => {
                let start = self.cur_start();
                self.advance();
                Err(SyntaxError {
                    kind: SyntaxErrorKind::Message(
                        "module.attribute_outside_header: @! attributes must appear at the top of the file before imports".to_string(),
                    ),
                    range: start..self.prev_end,
                })
            }
            Token::Break => {
                let start = self.cur_start();
                self.advance(); // 'break'
                Ok(Statement::Break {
                    range: (start..self.prev_end).into(),
                })
            }
            Token::Continue => {
                let start = self.cur_start();
                self.advance(); // 'continue'
                Ok(Statement::Continue {
                    range: (start..self.prev_end).into(),
                })
            }
            _ => self.parse_expr_statement(),
        }
    }

    /// Parses a transparent type alias declaration `type Name<T> where ... = Body` (Spec 04 §7.5, §13).
    fn parse_type_alias(&mut self) -> ParserResult<Statement> {
        let start = self.cur_start();
        self.expect(&Token::TypeKw, &["\"type\""])?;
        let name_start = self.cur_start();
        let name = self.expect_identifier(&["type alias name"])?;
        let name_range = (name_start..self.prev_end).into();
        let generic_parameters = if matches!(self.peek(), Token::Less) {
            self.parse_generic_parameters(GenericBinderContext::Alias)?
        } else {
            Vec::new()
        };
        let where_clause = if matches!(self.peek(), Token::Where) {
            Some(self.parse_where_clause()?)
        } else {
            None
        };
        self.expect(&Token::Equal, &["\"=\""])?;
        let body = self.parse_type_form()?;
        let range = (start..self.prev_end).into();
        Ok(Statement::TypeAlias(TypeAliasDef {
            name,
            name_range,
            generic_parameters,
            where_clause,
            body,
            range,
        }))
    }

    /// Parses `for pattern [at index] in iter, ... { body }` into a
    /// [`Statement::For`]. Parentheses are handled by the pattern grammar, so
    /// `(x, y)` is a tuple pattern rather than a header wrapper.
    /// (ADR-0035 §2, iteration.md §2, U-ITER specification §1.1).
    fn parse_for(&mut self) -> ParserResult<Statement> {
        let start = self.cur_start();
        self.advance(); // 'for'
        let mut lanes = Vec::new();
        loop {
            let lane_start = self.cur_start();
            let pattern = self.parse_pattern()?;
            let index = if matches!(self.peek(), Token::Identifier(name) if name == "at") {
                self.advance();
                let index_start = self.cur_start();
                let name = self.expect_identifier(&["iteration index name"])?;
                Some(ForIndexBinding {
                    name,
                    range: (index_start..self.prev_end).into(),
                })
            } else {
                None
            };
            self.expect(&Token::In, &["\"in\""])?;
            // The loop body starts with `{`, which must remain outside the
            // iterable expression. In particular, member-call parsing can
            // otherwise attach it as a trailing closure to `iter`.
            let iter = self.parse_expr_without_trailing_closures()?;
            lanes.push(ForLane {
                pattern,
                index,
                iter,
                range: (lane_start..self.prev_end).into(),
            });
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        let body = match self.parse_brace_block()? {
            Expr::Block(block) => block.body,
            _ => unreachable!("parse_brace_block must produce a block"),
        };
        let range = (start..self.prev_end).into();
        Ok(Statement::For(ForStatement { lanes, body, range }))
    }

    /// Parses `throw expr` — surface sugar for `expr.raise()`
    fn parse_throw(&mut self) -> ParserResult<Statement> {
        let start = self.cur_start();
        self.advance(); // 'throw'
        let expr = self.parse_expr()?;
        let range = (start..self.prev_end).into();
        Ok(Statement::Throw { expr, range })
    }

    /// Parses `try { P } (on T e { … })* (catch e { … })? (ensure { … })?`
    /// straight into the ADR-0031 §3 **nested re-wrapping** desugar over the
    /// `Block` catch protocol (error-handling.md §2):
    ///
    /// ```phalcom
    /// { { { P }.on(A) { a => HA } }.on(B) { b => HB } }.ensure { C }
    /// ```
    ///
    /// `on`/`ensure` run their receiver block **eagerly**, so a flat
    /// left-associative chain (`{P}.on(A){}.on(B){}`) would send `on(B)` to the
    /// *value* `on(A)` returns rather than a block — each successive clause
    /// must instead wrap the accumulated expression in a fresh block literal.
    /// `catch e { … }` desugars to `.on(Error) { e => … }` (catch-all, `Error`
    /// is the raisable root). This mirrors `if`/`while`'s parser-level desugar
    /// to sends (U5) rather than carrying its own `Statement` variant — the
    /// result is an ordinary [`Statement::Expr`].
    ///
    /// `on`/`catch`/`ensure` are **contextual keywords**: recognised as
    /// ordinary [`Token::Identifier`]s solely while parsing this tail
    /// (error-handling.md §2, ADR-0031 §4), so `.on()`/`.ensure()` selectors and
    /// the `Fiber>>try` message keep working everywhere else. `try` itself is a
    /// genuine reserved keyword ([`Token::Try`]); [`Self::parse_property_name`]
    /// additionally accepts it so `fiber.try(...)` still parses.
    ///
    /// # Errors
    ///
    /// Returns an error if any clause is malformed, or if the statement has
    /// neither an `on`/`catch` handler nor an `ensure` block (an empty handler
    /// set is rejected, error-handling.md §2).
    fn parse_try(&mut self) -> ParserResult<Statement> {
        let start = self.cur_start();
        self.advance(); // 'try'
        let mut acc = self.parse_brace_block()?;
        let mut has_handler = false;
        let mut has_ensure = false;
        // The very first clause's receiver is the protected block literal
        // itself — already a fresh `Expr::Block`, so it needs no wrapping.
        // Every clause *after* that sends to the **value** the previous
        // eager `.on(_)(_)`/`.ensure(_)` send just produced, not a block —
        // `on`/`ensure` run their receiver immediately, so a flat chain
        // (`{P}.on(A){}.on(B){}`) would send `on(B)` to `on(A)`'s *result*.
        // Each subsequent clause must therefore re-wrap the accumulated
        // expression in a fresh 0-param block first (ADR-0031 §3's nested
        // re-wrapping desugar — see this fn's own doc for the worked
        // example).
        let mut needs_wrap = false;

        loop {
            let is_on = matches!(self.peek(), Token::Identifier(kw) if kw == "on");
            let is_catch = matches!(self.peek(), Token::Identifier(kw) if kw == "catch");
            if is_on {
                self.advance(); // 'on'
                let class_start = self.cur_start();
                let class_name = self.expect_identifier(&["error class name"])?;
                let class_range: SourceRange = (class_start..self.prev_end).into();
                let param = self.expect_identifier(&["handler parameter"])?;
                let handler = self.parse_named_param_block(param)?;
                if needs_wrap {
                    acc = Self::wrap_expr_as_block(acc);
                }
                acc = self.wrap_on(
                    acc,
                    Expr::Var {
                        value: class_name,
                        range: class_range,
                    },
                    handler,
                    start,
                );
                has_handler = true;
                needs_wrap = true;
            } else if is_catch {
                self.advance(); // 'catch'
                let e_start = self.cur_start();
                let param = self.expect_identifier(&["handler parameter"])?;
                let handler = self.parse_named_param_block(param)?;
                let class_range: SourceRange = (e_start..e_start).into();
                if needs_wrap {
                    acc = Self::wrap_expr_as_block(acc);
                }
                acc = self.wrap_on(
                    acc,
                    Expr::Var {
                        value: "Error".to_string(),
                        range: class_range,
                    },
                    handler,
                    start,
                );
                has_handler = true;
                needs_wrap = true;
            } else {
                break;
            }
        }

        if matches!(self.peek(), Token::Identifier(kw) if kw == "ensure") {
            self.advance(); // 'ensure'
            let cleanup = self.parse_brace_block()?;
            let cleanup_range = cleanup.range();
            if needs_wrap {
                acc = Self::wrap_expr_as_block(acc);
            }
            let range = (start..self.prev_end).into();
            acc = Expr::MethodCall(Box::new(MethodCallExpr {
                object: acc,
                method: "ensure".to_string(),
                method_range: None,
                args: vec![PackItem::Positional {
                    expr: cleanup,
                    range: cleanup_range,
                }],
                range,
            }));
            has_ensure = true;
        }

        if !has_handler && !has_ensure {
            return Err(self.error_here(strs(&["\"on\"", "\"catch\"", "\"ensure\""])));
        }

        let range = (start..self.prev_end).into();
        Ok(Statement::Expr { expr: acc, range })
    }

    /// Wraps `protected` in the `.on(class, handler)` send that one `try`
    /// clause desugars to (`Self::parse_try`), spanning `start..prev_end`.
    fn wrap_on(&self, protected: Expr, class: Expr, handler: Expr, start: usize) -> Expr {
        let class_range = class.range();
        let handler_range = handler.range();
        let range = (start..self.prev_end).into();
        Expr::MethodCall(Box::new(MethodCallExpr {
            object: protected,
            method: "on".to_string(),
            method_range: None,
            args: vec![
                PackItem::Positional {
                    expr: class,
                    range: class_range,
                },
                PackItem::Positional {
                    expr: handler,
                    range: handler_range,
                },
            ],
            range,
        }))
    }

    /// Parses `{ statements }` into a 1-parameter, statement-bodied
    /// [`BlockExpr`] naming `param` — the handler-block shape `try`'s
    /// `on ClassName param { … }` / `catch param { … }` clauses lower to
    /// (`Self::parse_try`). Unlike [`Self::parse_brace_block`] (0 params), this
    /// binds the caught `Error` to `param` inside the body.
    ///
    /// # Errors
    ///
    /// Returns an error if the braces or the enclosed statements are malformed.
    fn parse_named_param_block(&mut self, param: String) -> ParserResult<Expr> {
        let start = self.cur_start();
        self.expect(&Token::LBrace, &["\"{\""])?;
        let body = self.parse_block_statements()?;
        self.expect(&Token::RBrace, &["\"}\""])?;
        let range = (start..self.prev_end).into();
        Ok(Expr::Block(Box::new(BlockExpr {
            params: ClosureParameters::fixed(vec![param]),
            body,
            expr_body: false,
            range,
        })))
    }

    /// Parses a complete type annotation / type form (Spec 04 §5).
    pub fn parse_type_annotation(&mut self) -> ParserResult<TypeAnnotation> {
        self.parse_type_form()
    }

    /// Parses a type form: type lambda or union type (Spec 04 §5.1).
    pub fn parse_type_form(&mut self) -> ParserResult<TypeAnnotation> {
        if matches!(self.peek(), Token::Less) && self.is_type_lambda_ahead() {
            self.parse_type_lambda()
        } else {
            self.parse_union_type()
        }
    }

    /// Parses a type lambda: `<T, ...> =>> Body` (Spec 04 §9).
    pub fn parse_type_lambda(&mut self) -> ParserResult<TypeAnnotation> {
        let start = self.cur_start();
        self.expect(&Token::Less, &["\"<\""])?;
        let mut parameters = Vec::new();
        while !matches!(self.peek(), Token::Greater | Token::ShiftRight | Token::Eof) {
            let param_start = self.cur_start();
            if matches!(self.peek(), Token::Plus | Token::Minus) {
                self.advance();
                self.errors.push(SyntaxError {
                    kind: SyntaxErrorKind::Message("variance not allowed on type-lambda binder".to_string()),
                    range: param_start..self.prev_end,
                });
            }
            let name_start = self.cur_start();
            let name = self.expect_identifier(&["type lambda parameter name"])?;
            let name_range = (name_start..self.prev_end).into();
            let kind = if self.eat(&Token::ColonColon) {
                self.errors.push(SyntaxError {
                    kind: SyntaxErrorKind::Message("legacy kind ascription '::' is deprecated; use ':' for kind annotations".to_string()),
                    range: param_start..self.prev_end,
                });
                Some(self.parse_kind_expression()?)
            } else if self.eat(&Token::Colon) {
                Some(self.parse_kind_expression()?)
            } else {
                None
            };
            let range = (param_start..self.prev_end).into();
            parameters.push(TypeLambdaParameter { name, name_range, kind, range });
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        self.expect_greater()?;
        self.expect(&Token::TypeLambdaArrow, &["\"=>>\""])?;
        let body = self.parse_type_form()?;
        let range = (start..self.prev_end).into();
        Ok(TypeAnnotation {
            expr: TypeAnnotationExpr::TypeLambda {
                parameters,
                body: Box::new(body),
                range,
            },
            range,
        })
    }

    /// Parses a union type `A | B | C` (Spec 04 §5.5).
    pub fn parse_union_type(&mut self) -> ParserResult<TypeAnnotation> {
        let start = self.cur_start();
        let first = self.parse_callable_type()?;
        if matches!(self.peek(), Token::Pipe) {
            let mut members = vec![first];
            while self.eat(&Token::Pipe) {
                members.push(self.parse_callable_type()?);
            }
            let range = (start..self.prev_end).into();
            Ok(TypeAnnotation {
                expr: TypeAnnotationExpr::Union { members, range },
                range,
            })
        } else {
            Ok(first)
        }
    }

    /// Parses a callable type or postfix type (Spec 04 §5.6).
    pub fn parse_callable_type(&mut self) -> ParserResult<TypeAnnotation> {
        let start = self.cur_start();
        if matches!(self.peek(), Token::LParen) {
            let paren = self.parse_paren_type()?;
            Ok(paren)
        } else {
            let atom = self.parse_postfix_type()?;
            if self.eat(&Token::Arrow) {
                let result = self.parse_type_form()?;
                let range = (start..self.prev_end).into();
                let param = TypeCallableParameter {
                    label: None,
                    ty: atom.clone(),
                    rest: false,
                    range: atom.range,
                };
                Ok(TypeAnnotation {
                    expr: TypeAnnotationExpr::Callable {
                        parameters: vec![param],
                        result: Box::new(result),
                        range,
                    },
                    range,
                })
            } else {
                Ok(atom)
            }
        }
    }

    /// Parses parenthesized, unit, tuple, or callable domain types (Spec 04 §5.3, §5.6).
    fn parse_paren_type(&mut self) -> ParserResult<TypeAnnotation> {
        let start = self.cur_start();
        self.expect(&Token::LParen, &["\"(\""])?;
        if self.eat(&Token::RParen) {
            let unit_range = (start..self.prev_end).into();
            if self.eat(&Token::Arrow) {
                let result = self.parse_type_form()?;
                let range = (start..self.prev_end).into();
                return Ok(TypeAnnotation {
                    expr: TypeAnnotationExpr::Callable {
                        parameters: Vec::new(),
                        result: Box::new(result),
                        range,
                    },
                    range,
                });
            } else {
                return Ok(TypeAnnotation {
                    expr: TypeAnnotationExpr::Unit { range: unit_range },
                    range: unit_range,
                });
            }
        }

        let mut items = Vec::new();
        let mut trailing_comma = false;
        while !matches!(self.peek(), Token::RParen | Token::Eof) {
            let item_start = self.cur_start();
            let rest = self.eat(&Token::DotDotDot);
            let label = if matches!(self.peek(), Token::Identifier(_)) && matches!(self.peek_next(), Token::Colon) {
                let lbl = self.expect_identifier(&["label"])?;
                self.advance(); // ':'
                Some(lbl)
            } else {
                None
            };
            let ty = self.parse_type_form()?;
            let item_range = (item_start..self.prev_end).into();
            items.push((label, ty, rest, item_range));

            if self.eat(&Token::Comma) {
                if matches!(self.peek(), Token::RParen) {
                    trailing_comma = true;
                    break;
                }
            } else {
                break;
            }
        }
        self.expect(&Token::RParen, &["\")\""])?;

        if self.eat(&Token::Arrow) {
            let result = self.parse_type_form()?;
            let range = (start..self.prev_end).into();
            let parameters = items
                .into_iter()
                .map(|(label, ty, rest, p_range)| TypeCallableParameter {
                    label,
                    ty,
                    rest,
                    range: p_range,
                })
                .collect();
            Ok(TypeAnnotation {
                expr: TypeAnnotationExpr::Callable {
                    parameters,
                    result: Box::new(result),
                    range,
                },
                range,
            })
        } else {
            let range = (start..self.prev_end).into();
            if items.iter().any(|(_, _, rest, _)| *rest) {
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::Message("rest parameter '...' only permitted in callable parameter list".to_string()),
                    range: start..self.prev_end,
                });
            }
            if items.len() == 1 && !trailing_comma && items[0].0.is_none() {
                let (_, inner, _, _) = items.remove(0);
                Ok(inner)
            } else {
                let elements = items
                    .into_iter()
                    .map(|(label, ty, _, el_range)| TypeTupleElement { label, ty, range: el_range })
                    .collect();
                Ok(TypeAnnotation {
                    expr: TypeAnnotationExpr::Tuple { elements, range },
                    range,
                })
            }
        }
    }

    /// Parses a postfix type with angle-bracket application `<...>` (Spec 04 §5.4).
    pub fn parse_postfix_type(&mut self) -> ParserResult<TypeAnnotation> {
        let mut atom = self.parse_type_atom()?;
        while matches!(self.peek(), Token::Less) {
            let start = atom.range.start;
            self.advance(); // '<'
            let mut arguments = Vec::new();
            while !matches!(self.peek(), Token::Greater | Token::ShiftRight | Token::Eof) {
                arguments.push(self.parse_type_form()?);
                if !self.eat(&Token::Comma) {
                    break;
                }
            }
            self.expect_greater()?;
            let range = (start..self.prev_end).into();
            atom = TypeAnnotation {
                expr: TypeAnnotationExpr::Application {
                    origin: Box::new(atom),
                    arguments,
                    range,
                },
                range,
            };
        }
        Ok(atom)
    }

    /// Parses an atomic type form (Spec 04 §5.1).
    pub fn parse_type_atom(&mut self) -> ParserResult<TypeAnnotation> {
        let start = self.cur_start();
        match self.peek() {
            Token::LParen => self.parse_paren_type(),
            Token::RecordLBrace => self.parse_record_type(),
            Token::Identifier(s) => {
                let name = s.clone();
                let root_start = self.cur_start();
                self.advance();
                let root_range = (root_start..self.prev_end).into();
                let mut members = Vec::new();
                while self.eat(&Token::Dot) {
                    let m_start = self.cur_start();
                    let m_name = self.expect_identifier(&["qualified type name"])?;
                    members.push(PathSegment {
                        name: m_name,
                        range: (m_start..self.prev_end).into(),
                    });
                }
                let range = (start..self.prev_end).into();
                if members.is_empty() {
                    match name.as_str() {
                        "Never" => {
                            return Ok(TypeAnnotation {
                                expr: TypeAnnotationExpr::Never { range },
                                range,
                            });
                        }
                        "Dynamic" => {
                            return Ok(TypeAnnotation {
                                expr: TypeAnnotationExpr::Dynamic { range },
                                range,
                            });
                        }
                        "Self" => {
                            return Ok(TypeAnnotation {
                                expr: TypeAnnotationExpr::SelfType { range },
                                range,
                            });
                        }
                        "Unknown" => {
                            self.errors.push(SyntaxError {
                                kind: SyntaxErrorKind::Message("Unknown is an analysis state, not a source type; use Dynamic or omit annotation".to_string()),
                                range: start..self.prev_end,
                            });
                        }
                        "Any" => {
                            self.errors.push(SyntaxError {
                                kind: SyntaxErrorKind::Message("Any is reserved and not yet enabled as a source type".to_string()),
                                range: start..self.prev_end,
                            });
                        }
                        _ => {}
                    }
                }
                let sym_ref = StaticSymbolRef {
                    root: name,
                    root_range,
                    members,
                    range,
                };
                Ok(TypeAnnotation {
                    expr: TypeAnnotationExpr::Reference(sym_ref),
                    range,
                })
            }
            _ => {
                let err = self.error_here(vec!["type name".to_string(), "\"(\"".to_string(), "\"#{\"".to_string()]);
                self.advance();
                let range = (start..self.prev_end).into();
                Ok(TypeAnnotation {
                    expr: TypeAnnotationExpr::Invalid {
                        message: format!("{:?}", err.kind),
                        range,
                    },
                    range,
                })
            }
        }
    }

    /// Parses a structural record type `#{ ... }` (Spec 04 §5.7).
    pub fn parse_record_type(&mut self) -> ParserResult<TypeAnnotation> {
        let start = self.cur_start();
        self.expect(&Token::RecordLBrace, &["\"#{\""])?;
        let mut fields = Vec::new();
        let mut tail = None;

        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            if self.eat(&Token::Pipe) {
                let tail_start = self.cur_start();
                let tail_name = self.expect_identifier(&["record row tail name"])?;
                tail = Some(RecordRowTail {
                    name: tail_name,
                    range: (tail_start..self.prev_end).into(),
                });
                self.eat(&Token::Comma);
                break;
            }

            let field_start = self.cur_start();
            let name = self.expect_identifier(&["field name"])?;
            self.expect(&Token::Colon, &["\":\""])?;
            let ty = self.parse_type_form()?;
            let field_range = (field_start..self.prev_end).into();
            fields.push(RecordTypeField { name, ty, range: field_range });

            if self.eat(&Token::Comma) {
                if self.eat(&Token::Pipe) {
                    let tail_start = self.cur_start();
                    let tail_name = self.expect_identifier(&["record row tail name"])?;
                    tail = Some(RecordRowTail {
                        name: tail_name,
                        range: (tail_start..self.prev_end).into(),
                    });
                    self.eat(&Token::Comma);
                    break;
                }
            } else {
                break;
            }
        }
        self.expect(&Token::RBrace, &["\"}\""])?;
        let range = (start..self.prev_end).into();
        Ok(TypeAnnotation {
            expr: TypeAnnotationExpr::Record { fields, tail, range },
            range,
        })
    }

    /// Parses a kind expression (Spec 04 §3.3): `Type`, `RecordRow`, `Kind -> Kind`, `(Kind)`.
    pub fn parse_kind_expression(&mut self) -> ParserResult<KindSyntax> {
        let start = self.cur_start();
        let atom = if self.eat(&Token::LParen) {
            let inner = self.parse_kind_expression()?;
            self.expect(&Token::RParen, &["\")\""])?;
            let range = (start..self.prev_end).into();
            KindSyntax::Grouped { inner: Box::new(inner), range }
        } else if matches!(self.peek(), Token::Identifier(s) if s == "Type") {
            self.advance();
            let range = (start..self.prev_end).into();
            KindSyntax::Type(range)
        } else if matches!(self.peek(), Token::Identifier(s) if s == "RecordRow") {
            self.advance();
            let range = (start..self.prev_end).into();
            KindSyntax::RecordRow(range)
        } else {
            let err = self.error_here(vec!["\"Type\"".to_string(), "\"RecordRow\"".to_string()]);
            self.advance();
            let range = (start..self.prev_end).into();
            return Ok(KindSyntax::Invalid {
                message: format!("{:?}", err.kind),
                range,
            });
        };

        if self.eat(&Token::Arrow) {
            let result = self.parse_kind_expression()?;
            let range = (start..self.prev_end).into();
            Ok(KindSyntax::Arrow {
                parameter: Box::new(atom),
                result: Box::new(result),
                range,
            })
        } else {
            Ok(atom)
        }
    }

    /// Parses generic parameter binders with contextual variance checks (Spec 04 §6).
    pub fn parse_generic_parameters(&mut self, context: GenericBinderContext) -> ParserResult<Vec<GenericParameterSyntax>> {
        self.expect(&Token::Less, &["\"<\""])?;
        let mut params = Vec::new();
        while !matches!(self.peek(), Token::Greater | Token::ShiftRight | Token::Eof) {
            let param_start = self.cur_start();
            let variance = if self.eat(&Token::Plus) {
                if context != GenericBinderContext::NominalDeclaration {
                    self.errors.push(SyntaxError {
                        kind: SyntaxErrorKind::Message("variance marker '+' only permitted on nominal declaration parameters".to_string()),
                        range: param_start..self.prev_end,
                    });
                }
                VarianceSyntax::Covariant
            } else if self.eat(&Token::Minus) {
                if context != GenericBinderContext::NominalDeclaration {
                    self.errors.push(SyntaxError {
                        kind: SyntaxErrorKind::Message("variance marker '-' only permitted on nominal declaration parameters".to_string()),
                        range: param_start..self.prev_end,
                    });
                }
                VarianceSyntax::Contravariant
            } else {
                VarianceSyntax::Invariant
            };

            if matches!(self.peek(), Token::Underscore) {
                let u_start = self.cur_start();
                self.advance();
                self.errors.push(SyntaxError {
                    kind: SyntaxErrorKind::Message("placeholder generic syntax 'F<_>' is rejected; declare explicit kind like 'F: Type -> Type'".to_string()),
                    range: u_start..self.prev_end,
                });
            }

            let name_start = self.cur_start();
            let name = self.expect_identifier(&["type parameter name"])?;
            let name_range = (name_start..self.prev_end).into();

            let kind = if self.eat(&Token::ColonColon) {
                self.errors.push(SyntaxError {
                    kind: SyntaxErrorKind::Message("legacy kind ascription '::' is deprecated; use ':' for kind annotations".to_string()),
                    range: param_start..self.prev_end,
                });
                Some(self.parse_kind_expression()?)
            } else if self.eat(&Token::Colon) {
                Some(self.parse_kind_expression()?)
            } else if self.eat(&Token::Subtype) {
                let inline_bound_start = self.cur_start();
                let _upper = self.parse_type_form()?;
                self.errors.push(SyntaxError {
                    kind: SyntaxErrorKind::Message(format!(
                        "inline generic constraint '<{} <: ...>' is rejected; move constraint to a 'where' clause",
                        name
                    )),
                    range: inline_bound_start..self.prev_end,
                });
                None
            } else {
                None
            };

            let range = (param_start..self.prev_end).into();
            params.push(GenericParameterSyntax {
                variance,
                name,
                name_range,
                kind,
                range,
            });

            if !self.eat(&Token::Comma) {
                break;
            }
        }
        self.expect_greater()?;
        Ok(params)
    }

    /// Parses a generic `where` clause (Spec 04 §8).
    pub fn parse_where_clause(&mut self) -> ParserResult<WhereClauseSyntax> {
        let start = self.cur_start();
        self.expect(&Token::Where, &["\"where\""])?;
        let mut constraints = Vec::new();
        while !matches!(self.peek(), Token::LBrace | Token::Equal | Token::Newline | Token::Eof) {
            let c_start = self.cur_start();
            let left = self.parse_type_form()?;
            if self.eat(&Token::Subtype) {
                let right = self.parse_type_form()?;
                let range = (c_start..self.prev_end).into();
                constraints.push(GenericConstraintSyntax::Subtype {
                    lower: left,
                    upper: right,
                    range,
                });
            } else if self.eat(&Token::EqualEqual) {
                let right = self.parse_type_form()?;
                let range = (c_start..self.prev_end).into();
                constraints.push(GenericConstraintSyntax::Equivalent { left, right, range });
            } else if self.eat(&Token::In) {
                let in_start = self.cur_start();
                if self.eat(&Token::LParen) {
                    while !matches!(self.peek(), Token::RParen | Token::Eof) {
                        let _ = self.parse_type_form()?;
                        if !self.eat(&Token::Comma) {
                            break;
                        }
                    }
                    self.expect(&Token::RParen, &["\")\""])?;
                }
                self.errors.push(SyntaxError {
                    kind: SyntaxErrorKind::Message("finite exact-set constraint 'in (...)' is not supported in generic calculus".to_string()),
                    range: in_start..self.prev_end,
                });
                let range = (c_start..self.prev_end).into();
                constraints.push(GenericConstraintSyntax::Invalid {
                    message: "unsupported finite-set constraint".to_string(),
                    range,
                });
            } else {
                let range = (c_start..self.prev_end).into();
                self.errors.push(SyntaxError {
                    kind: SyntaxErrorKind::Message("expected '<:' or '==' in generic constraint".to_string()),
                    range: c_start..self.prev_end,
                });
                constraints.push(GenericConstraintSyntax::Invalid {
                    message: "expected constraint operator".to_string(),
                    range,
                });
                break;
            }

            if !self.eat(&Token::Comma) {
                break;
            }
        }
        let range = (start..self.prev_end).into();
        Ok(WhereClauseSyntax { constraints, range })
    }

    /// Parses a `let`/`var` binding: `<kw> pattern (: Type)? (= expr)?`.
    ///
    /// `kind` records whether the `let` or `var` keyword was consumed
    /// (ADR-0014); the caller has already confirmed the current token matches.
    /// The left-hand side is a [`Pattern`] (U14, open-questions.md Q7) — a
    /// bare name or a destructuring tuple/list pattern (see
    /// [`Self::parse_pattern`]). Mutability and missing-initializer rules are
    /// enforced later by the compiler, not here.
    ///
    /// # Errors
    ///
    /// Returns an error if the pattern or the initialiser expression is
    /// malformed.
    fn parse_binding(&mut self, kind: BindingKind) -> ParserResult<Statement> {
        let start = self.cur_start();
        self.advance(); // 'let' or 'var'
        let pattern = self.parse_pattern()?;
        let annotation = if self.eat(&Token::Colon) { Some(self.parse_type_annotation()?) } else { None };
        let value = if self.eat(&Token::Equal) { Some(self.parse_expr()?) } else { None };
        let range = (start..self.prev_end).into();
        Ok(Statement::Let(LetBinding {
            kind,
            pattern,
            annotation,
            value,
            range,
        }))
    }

    /// Parses a `let`/`var` binding's left-hand side [`Pattern`] (U14,
    /// open-questions.md Q7,
    /// [ADR-0046](../../../docs/adr/accepted/0046-destructuring-bindings.md)): a bare
    /// name, a tuple pattern `(p1, …, pn)`, or a list pattern
    /// `[p1, …, pn]`/`[p1, …, pn, *rest]`. Patterns nest recursively, so
    /// `(…)`/`[…]` sub-patterns are parsed by re-entering this function.
    ///
    /// This is a distinct grammar path from the RHS tuple/list *literal*
    /// parsers ([`Self::parse_paren_or_tuple`], [`Self::parse_list_literal`])
    /// — reached only in binding-target position — so `(a, b)` never parses
    /// ambiguously between "pattern" and "literal".
    ///
    /// # Errors
    ///
    /// Returns a [`SyntaxErrorKind::Message`] diagnostic if a rest
    /// sub-pattern is not a list pattern's last element, or if the current
    /// token cannot begin a pattern.
    /// Parses a pattern, supporting or-patterns (`p1 | p2`), wildcards (`_`),
    /// tuples `(p1, ...)`, lists `[p1, ...]`, records `#{...}`, maps `{#...}`,
    /// qualified/contextual variants (`Animal::Dog*`, `Dog(...)`, `Dog(x, ..., named: y)`),
    /// and name bindings (`x`).
    fn parse_pattern(&mut self) -> ParserResult<Pattern> {
        self.parse_or_pattern()
    }

    fn parse_or_pattern(&mut self) -> ParserResult<Pattern> {
        let first = self.parse_pattern_atom()?;
        if matches!(self.peek(), Token::Pipe) {
            let mut alternatives = vec![first];
            while self.eat(&Token::Pipe) {
                self.skip_newlines();
                alternatives.push(self.parse_pattern_atom()?);
            }
            let start = alternatives.first().unwrap().range().start;
            let end = alternatives.last().unwrap().range().end;
            Ok(Pattern::Or {
                alternatives,
                range: (start..end).into(),
            })
        } else {
            Ok(first)
        }
    }

    fn parse_pattern_atom(&mut self) -> ParserResult<Pattern> {
        match self.peek() {
            Token::Underscore => {
                let start = self.cur_start();
                self.advance();
                let range = (start..self.prev_end).into();
                Ok(Pattern::Wildcard { range })
            }
            Token::LParen => self.parse_tuple_pattern(),
            Token::LBracket => self.parse_list_pattern(),
            Token::RecordLBrace => self.parse_record_pattern(),
            Token::LBrace => self.parse_map_pattern(),
            Token::Identifier(_) => self.parse_identifier_or_variant_pattern(),
            _ => {
                let start = self.cur_start();
                let name = self.expect_identifier(&["identifier", "\"(\"", "\"[\"", "\"_\""])?;
                let range = (start..self.prev_end).into();
                Ok(Pattern::Name { name, range })
            }
        }
    }

    fn parse_identifier_or_variant_pattern(&mut self) -> ParserResult<Pattern> {
        let start = self.cur_start();
        let name = self.expect_identifier(&["identifier", "pattern"])?;
        let root_range = (start..self.prev_end).into();

        let mut members = Vec::new();
        while self.eat(&Token::Dot) {
            let m_start = self.cur_start();
            let m_name = self.expect_identifier(&["qualified name"])?;
            members.push(PathSegment {
                name: m_name,
                range: (m_start..self.prev_end).into(),
            });
        }

        if self.eat(&Token::ColonColon) {
            let owner_range = (start..self.prev_end).into();
            let owner = Some(StaticSymbolRef {
                root: name,
                root_range,
                members,
                range: owner_range,
            });
            let base_start = self.cur_start();
            let base = self.expect_identifier(&["variant constructor"])?;
            let base_range = (base_start..self.prev_end).into();
            self.parse_variant_pattern_suffix(start, owner, base, base_range)
        } else if !members.is_empty() {
            let range = (start..self.prev_end).into();
            self.errors.push(SyntaxError {
                kind: SyntaxErrorKind::Message("unexpected qualified path in pattern without `::`".to_string()),
                range: start..self.prev_end,
            });
            Ok(Pattern::Name { name, range })
        } else if matches!(self.peek(), Token::Asterisk | Token::LParen) {
            self.parse_variant_pattern_suffix(start, None, name.clone(), root_range)
        } else {
            Ok(Pattern::Name {
                name,
                range: root_range,
            })
        }
    }

    fn parse_variant_pattern_suffix(
        &mut self,
        start: usize,
        owner: Option<StaticSymbolRef>,
        base: String,
        base_range: SourceRange,
    ) -> ParserResult<Pattern> {
        if self.eat(&Token::Asterisk) {
            let star_range = (self.prev_end - 1..self.prev_end).into();
            let range = (start..self.prev_end).into();
            Ok(Pattern::Variant(VariantPattern {
                owner,
                base,
                base_range,
                mode: VariantPatternMode::WholeFamily { star_range },
                range,
            }))
        } else if self.eat(&Token::LParen) {
            self.skip_newlines();
            let mut prefix = Vec::new();
            let mut suffix = Vec::new();
            let mut gap_range: Option<SourceRange> = None;

            if !matches!(self.peek(), Token::RParen) {
                loop {
                    self.skip_newlines();
                    if self.peek() == &Token::DotDotDot {
                        let dot_start = self.cur_start();
                        self.advance();
                        let d_range: SourceRange = (dot_start..self.prev_end).into();
                        if let Some(_first_gap) = gap_range {
                            self.errors.push(SyntaxError {
                                kind: SyntaxErrorKind::Message("multiple `...` selector gaps in pattern".to_string()),
                                range: dot_start..self.prev_end,
                            });
                        } else {
                            gap_range = Some(d_range);
                        }
                    } else {
                        let arg = self.parse_variant_pattern_argument()?;
                        if gap_range.is_some() {
                            suffix.push(arg);
                        } else {
                            prefix.push(arg);
                        }
                    }
                    self.skip_newlines();
                    if !self.eat(&Token::Comma) {
                        break;
                    }
                    self.skip_newlines();
                    if matches!(self.peek(), Token::RParen) {
                        break;
                    }
                }
            }
            self.expect(&Token::RParen, &["\")\""])?;
            let range = (start..self.prev_end).into();
            let mode = if let Some(gap_range) = gap_range {
                VariantPatternMode::CallablePattern {
                    prefix,
                    gap_range,
                    suffix,
                }
            } else {
                VariantPatternMode::ExactCall {
                    arguments: prefix,
                }
            };
            Ok(Pattern::Variant(VariantPattern {
                owner,
                base,
                base_range,
                mode,
                range,
            }))
        } else {
            let range = (start..self.prev_end).into();
            Ok(Pattern::Variant(VariantPattern {
                owner,
                base,
                base_range,
                mode: VariantPatternMode::Singleton,
                range,
            }))
        }
    }

    fn parse_variant_pattern_argument(&mut self) -> ParserResult<VariantPatternArgument> {
        let start = self.cur_start();
        if Self::label_name(self.peek()).is_some() && matches!(self.peek_next(), Token::Colon) {
            let label_start = self.cur_start();
            let label_str = Self::label_name(self.peek()).unwrap().to_string();
            self.advance();
            let label_range = (label_start..self.prev_end).into();
            self.expect(&Token::Colon, &["\":\""])?;
            self.skip_newlines();
            let pattern = self.parse_pattern()?;
            let range = (start..self.prev_end).into();
            Ok(VariantPatternArgument {
                label: Some(label_str),
                label_range: Some(label_range),
                pattern,
                range,
            })
        } else {
            let pattern = self.parse_pattern()?;
            let range = pattern.range();
            Ok(VariantPatternArgument {
                label: None,
                label_range: None,
                pattern,
                range,
            })
        }
    }

    fn parse_pattern_label(&mut self) -> ParserResult<String> {
        match self.peek().clone() {
            Token::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            Token::QuotedSymbol(name) => {
                self.advance();
                Ok(name)
            }
            Token::Hash => {
                self.advance();
                match self.peek().clone() {
                    Token::Identifier(name) => {
                        self.advance();
                        Ok(name)
                    }
                    Token::QuotedSymbol(name) => {
                        self.advance();
                        Ok(name)
                    }
                    _ => Err(self.error_here(strs(&["a static pattern key"]))),
                }
            }
            _ => Err(self.error_here(strs(&["a static pattern key"]))),
        }
    }

    /// Parses an open record pattern `#{field: pattern, ...}`.
    fn parse_record_pattern(&mut self) -> ParserResult<Pattern> {
        let start = self.cur_start();
        self.advance(); // `#{`
        let mut entries = Vec::new();
        if !matches!(self.peek(), Token::RBrace) {
            loop {
                let entry_start = self.cur_start();
                let label = self.parse_pattern_label()?;
                self.expect(&Token::Colon, &["\":\""])?;
                let pattern = self.parse_pattern()?;
                entries.push(RecordPatternEntry {
                    label,
                    pattern,
                    range: (entry_start..self.prev_end).into(),
                });
                if !self.eat(&Token::Comma) {
                    break;
                }
                if matches!(self.peek(), Token::RBrace) {
                    break;
                }
            }
        }
        self.expect(&Token::RBrace, &["\"}\""])?;
        Ok(Pattern::Record {
            entries,
            range: (start..self.prev_end).into(),
        })
    }

    /// Parses an open map pattern `{key: pattern, ...}`. Keys are deliberately
    /// literal/static in this first implementation.
    fn parse_map_pattern(&mut self) -> ParserResult<Pattern> {
        let start = self.cur_start();
        self.advance(); // `{`
        let mut entries = Vec::new();
        if !matches!(self.peek(), Token::RBrace) {
            loop {
                let entry_start = self.cur_start();
                let key = match self.peek().clone() {
                    Token::Hash => MapPatternKey::Symbol(self.parse_pattern_label()?),
                    Token::QuotedSymbol(name) => {
                        self.advance();
                        MapPatternKey::String(name)
                    }
                    Token::String(value) => {
                        self.advance();
                        MapPatternKey::String(value)
                    }
                    Token::Int { digits, radix } => {
                        self.advance();
                        MapPatternKey::Int { digits, radix }
                    }
                    _ => return Err(self.error_here(strs(&["a static map pattern key"]))),
                };
                self.expect(&Token::Colon, &["\":\""])?;
                let pattern = self.parse_pattern()?;
                entries.push(MapPatternEntry {
                    key,
                    pattern,
                    range: (entry_start..self.prev_end).into(),
                });
                if !self.eat(&Token::Comma) {
                    break;
                }
                if matches!(self.peek(), Token::RBrace) {
                    break;
                }
            }
        }
        self.expect(&Token::RBrace, &["\"}\""])?;
        Ok(Pattern::Map {
            entries,
            range: (start..self.prev_end).into(),
        })
    }

    /// Parses a parenthesized pattern `(p)` or a tuple pattern
    /// `(p1, …, pn)` with n ≥ 2, mirroring [`Self::parse_paren_or_tuple`]'s
    /// grouping-vs-tuple disambiguation (only a top-level comma promotes the
    /// form to a tuple pattern — `(p)` is just `p`, never a one-element tuple
    /// pattern).
    ///
    /// # Errors
    ///
    /// Propagates any [`SyntaxError`] from a sub-pattern, or from a missing
    /// closing `)`.
    fn parse_tuple_pattern(&mut self) -> ParserResult<Pattern> {
        let start = self.cur_start();
        self.advance(); // '('
        let first = self.parse_pattern()?;
        if !matches!(self.peek(), Token::Comma) {
            self.expect(&Token::RParen, &["\")\""])?;
            return Ok(first);
        }
        self.advance(); // ','
        let mut elements = vec![first];
        if !matches!(self.peek(), Token::RParen) {
            loop {
                elements.push(self.parse_pattern()?);
                if !self.eat(&Token::Comma) {
                    break;
                }
                // Allow a trailing comma directly before ')'.
                if matches!(self.peek(), Token::RParen) {
                    break;
                }
            }
        }
        self.expect(&Token::RParen, &["\")\""])?;
        let range: SourceRange = (start..self.prev_end).into();
        Ok(Pattern::Tuple { elements, range })
    }

    /// Parses a list pattern `[p1, …, pn]`, or `[p1, …, pn, *rest]` with a
    /// trailing rest sub-pattern.
    ///
    /// A rest sub-pattern must be the list pattern's **last** element. This
    /// mirrors the declaration-parameter rule.
    ///
    /// # Errors
    ///
    /// Returns a [`SyntaxErrorKind::Message`] diagnostic if `*` appears
    /// anywhere but the last element, propagates any [`SyntaxError`] from a
    /// sub-pattern, or from a missing closing `]`.
    fn parse_list_pattern(&mut self) -> ParserResult<Pattern> {
        let start = self.cur_start();
        self.advance(); // '['
        let mut elements: Vec<Pattern> = Vec::new();
        let mut rest: Option<Box<Pattern>> = None;
        if !matches!(self.peek(), Token::RBracket) {
            loop {
                let elem_start = self.cur_start();
                if rest.is_some() {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message("a rest pattern (\"*name\") must be the last element of a list pattern".to_string()),
                        range: elem_start..elem_start,
                    });
                }
                if self.eat(&Token::Asterisk) {
                    rest = Some(Box::new(self.parse_pattern()?));
                } else {
                    elements.push(self.parse_pattern()?);
                }
                if !self.eat(&Token::Comma) {
                    break;
                }
                // Allow a trailing comma directly before ']'.
                if matches!(self.peek(), Token::RBracket) {
                    break;
                }
            }
        }
        self.expect(&Token::RBracket, &["\"]\""])?;
        let range: SourceRange = (start..self.prev_end).into();
        Ok(Pattern::List { elements, rest, range })
    }

    /// Parses a `return expr?` statement.
    ///
    /// # Errors
    ///
    /// Returns an error if a present return expression is malformed.
    fn parse_return(&mut self) -> ParserResult<Statement> {
        let start = self.cur_start();
        self.advance(); // 'return'
        let value = if self.at_expr_start() { Some(self.parse_expr()?) } else { None };
        let range = (start..self.prev_end).into();
        Ok(Statement::Return(ReturnStatement { value, range }))
    }

    /// Parses a bare expression used as a statement.
    ///
    /// # Errors
    ///
    /// Propagates any error from expression parsing.
    fn parse_expr_statement(&mut self) -> ParserResult<Statement> {
        let start = self.cur_start();
        let expr = self.parse_expr()?;
        let range = (start..self.prev_end).into();
        Ok(Statement::Expr { expr, range })
    }

    /// Returns `true` if the current token can begin an expression.
    fn at_expr_start(&self) -> bool {
        matches!(
            self.peek(),
            Token::Int { .. }
                | Token::Float(_)
                | Token::String(_)
                | Token::StringInterp(_)
                | Token::True
                | Token::False
                | Token::Identifier(_)
                | Token::FieldIdentifier(_)
                | Token::ImplementationFieldIdentifier(_)
                | Token::Hash
                | Token::SelfKw
                | Token::Super
                | Token::LParen
                | Token::LBracket
                | Token::Not
                | Token::Minus
                | Token::LBrace
                | Token::Pipe
                | Token::If
                | Token::While
                // Range endpoints may be omitted; accept `..` / `..=` as
                // expression starts so constructs like `return ..` parse
                // as a Range expression with an omitted lower bound.
                | Token::DotDot
                | Token::DotDotEqual
        )
    }

    /// Consumes an identifier token, returning its name.
    ///
    /// # Errors
    /// Returns an error carrying `expected` if the current token is not an
    /// identifier.
    fn expect_identifier(&mut self, expected: &[&str]) -> ParserResult<String> {
        if let Token::Identifier(name) = self.peek().clone() {
            self.advance();
            Ok(name)
        } else {
            Err(self.error_here(strs(expected)))
        }
    }

    /// Consumes an identifier or keyword suitable as a module path component or expose operand.
    fn expect_component_identifier(&mut self, expected: &[&str]) -> ParserResult<String> {
        if let Some(name) = Self::label_name(self.peek()) {
            let s = name.to_string();
            self.advance();
            Ok(s)
        } else {
            Err(self.error_here(strs(expected)))
        }
    }

    /// Returns source text for an identifier or reserved keyword when that
    /// token occurs in a colon-marked label position. This does not admit a
    /// keyword as an ordinary variable or expression identifier.
    fn label_name(token: &Token) -> Option<&str> {
        Some(match token {
            Token::Identifier(name) => return Some(name),
            Token::Let => "let",
            Token::Const => "const",
            Token::Fn => "fn",
            Token::Class => "class",
            Token::Enum => "enum",
            Token::Match => "match",
            Token::Return => "return",
            Token::True => "true",
            Token::False => "false",
            Token::If => "if",
            Token::Else => "else",
            Token::While => "while",
            Token::For => "for",
            Token::Break => "break",
            Token::Continue => "continue",
            Token::Import => "import",
            Token::From => "from",
            Token::Export => "export",
            Token::Expose => "expose",
            Token::SelfKw => "self",
            Token::Super => "super",
            Token::In => "in",
            Token::As => "as",
            Token::Is => "is",
            Token::And => "and",
            Token::Or => "or",
            Token::Not => "not",
            Token::Static => "static",
            Token::Construct => "construct",
            Token::Throw => "throw",
            Token::Try => "try",
            Token::Where => "where",
            Token::TypeKw => "type",
            _ => return None,
        })
    }

    // ── Class declarations ───────────────────────────────────────────────────

    /// Parses an optional run of `@name(args…) class Name { members }`
    /// class-header attributes (U-ANNOT-LAYOUT §3.3, the `@construct` layout-
    /// derive tier and friends), then the class declaration itself.
    ///
    /// This is the class-header decorator position `ClassDef::attributes`'s
    /// own doc calls "reserved for a future class-level decorator" — U-ANNOT-
    /// LAYOUT is that future. A header attribute binds to the whole class,
    /// distinct from a class-*body* member attribute
    /// ([`Self::parse_class_body`]'s `pending_attrs` loop), and from the
    /// standalone `@invariant(...)` carve-out (DEC-ANNOT-B), which is
    /// body-position only. Newlines between a header attribute and the
    /// following `class` keyword (or a subsequent header attribute) are
    /// tolerated, mirroring the body-position attribute loop's own
    /// newline-tolerance.
    ///
    /// # Errors
    ///
    /// Returns an error if a header attribute, the class name, braces, or any
    /// member is malformed.
    fn parse_class(&mut self, mut header_attrs: Vec<Attribute>) -> ParserResult<Statement> {
        while matches!(self.peek(), Token::At) {
            header_attrs.push(self.parse_attribute()?);
            self.skip_newlines();
        }
        let start = self.cur_start();
        self.expect(&Token::Class, &["\"class\""])?;
        let name_start = self.cur_start();
        let name = self.expect_identifier(&["identifier"])?;
        let name_range = (name_start..self.prev_end).into();

        let generic_parameters = if matches!(self.peek(), Token::Less) {
            self.parse_generic_parameters(GenericBinderContext::NominalDeclaration)?
        } else {
            Vec::new()
        };

        // Superclass clause: `is` is Token::Is keyword (PDR-0030).
        // Grammar: `class` IDENT GENERIC_PARAMS? (`is` TYPE_FORM)? WHERE_CLAUSE? `{` … `}`.
        let superclass = if matches!(self.peek(), Token::Is) {
            self.advance(); // 'is'
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let where_clause = if matches!(self.peek(), Token::Where) {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        self.expect(&Token::LBrace, &["\"{\""])?;
        let previous_class_context = self.in_class_body;
        self.in_class_body = true;
        let body = self.parse_class_body();
        self.in_class_body = previous_class_context;
        let (members, body_attrs, invariants) = body?;
        self.expect(&Token::RBrace, &["\"}\""])?;
        let range = (start..self.prev_end).into();
        // `body_attrs` is always empty today (`parse_class_body`'s own doc:
        // no body-position grammar produces a class-level attribute); header
        // attributes are the sole real source of `ClassDef::attributes`.
        // Concatenating rather than overwriting keeps this call site correct
        // if a body-position source is ever added later.
        header_attrs.extend(body_attrs);
        Ok(Statement::Class(ClassDef {
            name,
            generic_parameters,
            superclass,
            where_clause,
            members,
            attributes: header_attrs,
            invariants,
            range,
            name_range,
        }))
    }

    /// Parses the members of a class body up to the closing `}`.
    ///
    /// Blank lines are ignored; each member is a method, getter, setter, or
    /// constructor. A member may be preceded by zero or more `@name(args…)`
    /// attributes, which are collected and attached to the member that
    /// immediately follows them (`annotations-legality-grammar.md`'s
    /// `class-member := attribute* [static] member-decl`) — except a
    /// standalone `@invariant(...)`, a one-off parse-time carve-out
    /// (DEC-ANNOT-B) that is diverted straight into the returned
    /// `invariants` list instead of binding to any member. This class-level
    /// `attributes`/`invariants` split mirrors [`ClassDef`]'s own fields; the
    /// caller assembles the final `ClassDef` from all three return values.
    ///
    /// # Errors
    ///
    /// Returns an error on a malformed member, a malformed attribute, an
    /// attribute with nothing following it to attach to (`attr.dangling`), or
    /// if end-of-file is reached before the closing brace.
    fn parse_class_body(&mut self) -> ParserResult<ClassBodyParts> {
        let mut members = Vec::new();
        // No surface grammar attaches an attribute to the class header
        // itself yet (only to a following member, or `@invariant`'s
        // standalone carve-out below) — this is always empty today, reserved
        // for a future class-level decorator (`ClassDef::attributes`'s own
        // doc).
        let class_attributes: Vec<Attribute> = Vec::new();
        let mut class_invariants = Vec::new();
        let mut pending_attrs: Vec<Attribute> = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek() {
                Token::RBrace if pending_attrs.is_empty() => break,
                Token::RBrace => return Err(self.dangling_attribute_error(&pending_attrs)),
                Token::Eof if !pending_attrs.is_empty() => return Err(self.dangling_attribute_error(&pending_attrs)),
                Token::At => {
                    let attr = self.parse_attribute()?;
                    self.skip_newlines();
                    // DEC-ANNOT-B (annotations-legality-grammar.md,
                    // U-ANNOT-CONTRACTS plan §3.1): `@invariant` alone has no
                    // following member to bind to — it stands as its own
                    // class-body item. This `if` is the *only* place any
                    // attribute skips the ordinary "binds to the next member"
                    // rule; do not generalize it to other attribute names.
                    if attr.name == "invariant" {
                        if attr.args.len() != 1 {
                            return Err(SyntaxError {
                                kind: SyntaxErrorKind::Message("@invariant expects exactly one predicate argument".to_string()),
                                range: attr.range.start..attr.range.end,
                            });
                        }
                        let predicate = attr.args.into_iter().next().unwrap();
                        class_invariants.push((predicate, attr.range));
                        continue;
                    }
                    pending_attrs.push(attr);
                    continue;
                }
                Token::Eof => return Err(self.error_here(strs(&["\"}\""]))),
                _ => {}
            }
            if let Some(variant_attr) = pending_attrs.iter().find(|a| a.name == "variant") {
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::VariantOutsideEnum,
                    range: variant_attr.range.start..variant_attr.range.end,
                });
            }
            let mut member = self.parse_class_member()?;
            if !pending_attrs.is_empty() {
                self.attach_attrs(&mut member, std::mem::take(&mut pending_attrs))?;
            }
            members.push(member);
        }
        Ok((members, class_attributes, class_invariants))
    }

    fn parse_enum(&mut self, mut header_attrs: Vec<Attribute>) -> ParserResult<Statement> {
        while matches!(self.peek(), Token::At) {
            header_attrs.push(self.parse_attribute()?);
            self.skip_newlines();
        }
        let start = self.cur_start();
        self.expect(&Token::Enum, &["\"enum\""])?;
        let name_start = self.cur_start();
        let name = self.expect_identifier(&["identifier"])?;
        let name_range = (name_start..self.prev_end).into();

        let generic_parameters = if matches!(self.peek(), Token::Less) {
            self.parse_generic_parameters(GenericBinderContext::NominalDeclaration)?
        } else {
            Vec::new()
        };

        let where_clause = if matches!(self.peek(), Token::Where) {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        self.expect(&Token::LBrace, &["\"{\""])?;
        let members = self.parse_enum_body()?;
        self.expect(&Token::RBrace, &["\"}\""])?;
        let range = (start..self.prev_end).into();

        Ok(Statement::Enum(EnumDef {
            name,
            name_range,
            generic_parameters,
            where_clause,
            members,
            attributes: header_attrs,
            range,
        }))
    }

    fn parse_enum_body(&mut self) -> ParserResult<Vec<EnumMember>> {
        let mut members = Vec::new();
        let mut pending_attrs: Vec<Attribute> = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek() {
                Token::RBrace if pending_attrs.is_empty() => break,
                Token::RBrace => return Err(self.dangling_attribute_error(&pending_attrs)),
                Token::Eof if !pending_attrs.is_empty() => return Err(self.dangling_attribute_error(&pending_attrs)),
                Token::At => {
                    let attr = self.parse_attribute()?;
                    self.skip_newlines();
                    pending_attrs.push(attr);
                    continue;
                }
                Token::Eof => return Err(self.error_here(strs(&["\"}\""]))),
                _ => {}
            }

            let member = if pending_attrs.iter().any(|a| a.name == "variant") {
                let variant = self.parse_enum_variant(std::mem::take(&mut pending_attrs))?;
                EnumMember::Variant(variant)
            } else {
                let behavior = self.parse_enum_behavior_member(std::mem::take(&mut pending_attrs))?;
                EnumMember::Behavior(behavior)
            };
            members.push(member);
        }
        Ok(members)
    }

    fn parse_enum_variant(&mut self, pending_attrs: Vec<Attribute>) -> ParserResult<VariantDecl> {
        let variant_attr = pending_attrs
            .iter()
            .find(|a| a.name == "variant")
            .cloned()
            .expect("parse_enum_variant called without variant attribute");
        let start = variant_attr.range.start;
        let variant_marker_range = variant_attr.range;
        let non_variant_attrs: Vec<Attribute> = pending_attrs.into_iter().filter(|a| a.name != "variant").collect();

        let name_start = self.cur_start();
        let name = self.expect_identifier(&["variant name"])?;
        let name_range = (name_start..self.prev_end).into();

        let payload = if matches!(self.peek(), Token::LParen) {
            let p_start = self.cur_start();
            self.advance(); // '('
            let parameters = if matches!(self.peek(), Token::RParen) {
                Vec::new()
            } else {
                self.parse_selector_params(Token::RParen)?
            };
            self.expect(&Token::RParen, &["\")\""])?;
            let p_range = (p_start..self.prev_end).into();

            for param in &parameters {
                if param.rest_mode != RestMode::None {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::VariantRestParameterUnsupported,
                        range: param.range.start..param.range.end,
                    });
                }
            }

            Some(VariantPayloadSyntax { parameters, range: p_range })
        } else {
            None
        };

        let result_annotation = if matches!(self.peek(), Token::Arrow) {
            self.advance(); // '->'
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let body = if matches!(self.peek(), Token::LBrace) {
            let b_start = self.cur_start();
            self.advance(); // '{'
            let mut members = Vec::new();
            let mut inner_pending_attrs: Vec<Attribute> = Vec::new();
            loop {
                self.skip_newlines();
                match self.peek() {
                    Token::RBrace if inner_pending_attrs.is_empty() => break,
                    Token::RBrace => return Err(self.dangling_attribute_error(&inner_pending_attrs)),
                    Token::Eof if !inner_pending_attrs.is_empty() => return Err(self.dangling_attribute_error(&inner_pending_attrs)),
                    Token::At => {
                        let attr = self.parse_attribute()?;
                        self.skip_newlines();
                        inner_pending_attrs.push(attr);
                        continue;
                    }
                    Token::Eof => return Err(self.error_here(strs(&["\"}\""]))),
                    _ => {}
                }
                let member = self.parse_enum_behavior_member(std::mem::take(&mut inner_pending_attrs))?;
                members.push(member);
            }
            self.expect(&Token::RBrace, &["\"}\""])?;
            let b_range = (b_start..self.prev_end).into();
            Some(VariantBody { members, range: b_range })
        } else {
            None
        };

        let range = (start..self.prev_end).into();
        Ok(VariantDecl {
            name,
            name_range,
            variant_marker_range,
            payload,
            result_annotation,
            body,
            attributes: non_variant_attrs,
            range,
        })
    }

    fn parse_enum_behavior_member(&mut self, pending_attrs: Vec<Attribute>) -> ParserResult<EnumBehaviorMember> {
        let start = self.cur_start();
        if matches!(self.peek(), Token::LBracket) {
            let class_member = self.parse_index_member(start)?;
            if let ClassMember::Index(mut idx) = class_member {
                idx.attributes = pending_attrs;
                return Ok(EnumBehaviorMember::Index(idx));
            } else {
                unreachable!()
            }
        }
        let is_static = false;
        let name_start = self.cur_start();
        let name = self.parse_method_name()?;
        let name_range = (name_start..self.prev_end).into();
        let has_equal = self.eat(&Token::Equal);
        if has_equal {
            self.expect(&Token::LParen, &["\"(\""])?;
            let start_put = self.cur_start();
            let put_str = self.expect_identifier(&["\"put\""])?;
            if put_str != "put" {
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::Message("setter parameter must start with \"put\"".to_string()),
                    range: start_put..self.prev_end,
                });
            }
            let local_start = self.cur_start();
            let local_name = self.expect_identifier(&["parameter name"])?;
            let local_range = (local_start..self.prev_end).into();
            let annotation = if self.eat(&Token::Colon) { Some(self.parse_type_annotation()?) } else { None };
            self.expect(&Token::RParen, &["\")\""])?;
            let param = ParameterDef {
                name: local_name,
                name_range: local_range,
                label: None,
                label_range: None,
                rest_mode: RestMode::None,
                annotation,
                range: (start_put..self.prev_end).into(),
            };
            let return_annotation = if self.eat(&Token::Arrow) { Some(self.parse_type_annotation()?) } else { None };
            let body = self.parse_member_body()?;
            let range = (start..self.prev_end).into();
            return Ok(EnumBehaviorMember::Setter(SetterDef {
                name,
                param,
                return_annotation,
                body,
                is_static,
                attributes: pending_attrs,
                range,
                name_range,
            }));
        }
        let generic_parameters = if matches!(self.peek(), Token::Less) {
            self.parse_generic_parameters(GenericBinderContext::Callable)?
        } else {
            Vec::new()
        };
        let params = if self.eat(&Token::LParen) {
            let list = self.parse_selector_params(Token::RParen)?;
            self.expect(&Token::RParen, &["\")\""])?;
            Some(list)
        } else {
            None
        };
        let return_annotation = if self.eat(&Token::Arrow) { Some(self.parse_type_annotation()?) } else { None };
        let where_clause = if matches!(self.peek(), Token::Where) {
            Some(self.parse_where_clause()?)
        } else {
            None
        };
        let body = self.parse_member_body()?;
        let range = (start..self.prev_end).into();
        if let Some(params) = params {
            Ok(EnumBehaviorMember::Method(MethodDef {
                name,
                generic_parameters,
                params,
                return_annotation,
                where_clause,
                body,
                is_static,
                is_constructor: false,
                attributes: pending_attrs,
                range,
                name_range,
            }))
        } else {
            if !generic_parameters.is_empty() {
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::Message("generic parameters not permitted on getters".to_string()),
                    range: start..self.prev_end,
                });
            }
            Ok(EnumBehaviorMember::Getter(GetterDef {
                name,
                return_annotation,
                body,
                is_static,
                attributes: pending_attrs,
                range,
                name_range,
            }))
        }
    }

    /// Parses a single `@name` or `@name(args…)` attribute.
    ///
    /// `args`, if present, is a parenthesized comma-separated list of
    /// ordinary expressions (`self.parse_expr`), exactly the grammar
    /// `annotations-legality-grammar.md` specifies — a bare identifier that
    /// isn't otherwise meaningful in this position still parses as an
    /// ordinary `Expr::Var`, so no special-casing is needed here.
    ///
    /// # Errors
    ///
    /// Returns an error if the `@` is not followed by an identifier, or the
    /// argument list is malformed.
    fn parse_attribute(&mut self) -> ParserResult<Attribute> {
        let start = self.cur_start();
        self.advance(); // '@'
        // `construct` is a reserved keyword (`Token::Construct`, not
        // `Token::Identifier`) — but it is also the exact name of the
        // U-ANNOT-LAYOUT `@construct` layout-derive attribute
        // (`annotations-construct.md`). Recognize it here the same way
        // `Self::parse_property_name` already recognizes `.class`/`.try` as
        // ordinary selector text despite being reserved words elsewhere.
        let name = if self.eat(&Token::Construct) {
            "construct".to_string()
        } else if self.eat(&Token::Class) {
            // `class` is a declaration/expression keyword elsewhere, but is
            // also the canonical placement attribute in member position.
            "class".to_string()
        } else {
            self.expect_identifier(&["attribute name"])?
        };
        let args = if self.eat(&Token::LParen) {
            let args = self.parse_attribute_arg_list()?;
            self.expect(&Token::RParen, &["\")\""])?;
            args
        } else {
            Vec::new()
        };
        let range = (start..self.prev_end).into();
        let kind = match BuiltinAttr::parse(&name) {
            Some(b) => AttrKind::Builtin(b),
            None => AttrKind::User(name.clone()),
        };
        Ok(Attribute { kind, name, args, range })
    }

    /// Parses a parenthesized, comma-separated list of attribute argument
    /// expressions (no labels — attribute arguments are always positional).
    ///
    /// # Errors
    ///
    /// Returns an error if any argument expression is malformed.
    fn parse_attribute_arg_list(&mut self) -> ParserResult<Vec<Expr>> {
        if matches!(self.peek(), Token::RParen) {
            return Ok(Vec::new());
        }
        let mut args = Vec::new();
        loop {
            args.push(self.parse_expr()?);
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        Ok(args)
    }

    /// Attaches `attrs` to `member`'s `attributes` field.
    ///
    /// Constructors are ordinary [`ClassMember::Method`] nodes, so attributes
    /// attach before attribute expansion marks and lowers constructor methods.
    /// [`ClassMember::Field`] carries an attribute list too (U-ANNOT-LAYOUT
    /// §3.1), for the not-yet-implemented `@get`/`@set` derive tier.
    fn attach_attrs(&mut self, member: &mut ClassMember, attrs: Vec<Attribute>) -> ParserResult<()> {
        match member {
            ClassMember::Method(m) => m.attributes = attrs,
            ClassMember::Getter(g) => g.attributes = attrs,
            ClassMember::Setter(s) => s.attributes = attrs,
            ClassMember::Field(f) => f.attributes = attrs,
            // Unreachable in practice — `parse_class_body` diverts a pending
            // `@variant` straight to `parse_variant_decl`, which consumes
            // every pending attribute itself before this function could ever
            // be called with a `Variant` member. Kept for match exhaustiveness
            // and future-proofing (a hypothetical attribute placed *after*
            // `@variant` in source, which the grammar does not currently
            // support attaching, would land here rather than panicking).
            ClassMember::Variant(v) => v.attributes = attrs,
            ClassMember::Index(ix) => ix.attributes = attrs,
        }
        Ok(())
    }

    /// Builds the `attr.dangling` diagnostic for one or more attributes with
    /// no following class member to bind to (end-of-file or a closing `}`
    /// reached while attributes are still pending).
    fn dangling_attribute_error(&self, pending: &[Attribute]) -> SyntaxError {
        let first = pending.first().expect("dangling_attribute_error called with no pending attributes");
        SyntaxError {
            kind: SyntaxErrorKind::Message(format!(
                "attr.dangling: `@{}` has no following method, getter, or setter to attach to",
                first.name
            )),
            range: first.range.start..first.range.end,
        }
    }

    /// Parses a field declaration: `const`-prefixed (immutable) or bare
    /// (mutable) — ADR-0064 §3, L-2. `is_const` records whether the caller
    /// already consumed a leading `Token::Const`; a bare field has no keyword
    /// to consume at all, so [`Self::parse_class_member`] dispatches here
    /// without advancing past anything first.
    ///
    /// Field eligibility is structural: lexer emits `FieldIdentifier` or
    /// `ImplementationFieldIdentifier`. Parser does not repeat prefix tests.
    ///
    /// # Errors
    ///
    /// Returns an error if the field name is missing, the initializer
    /// expression is malformed, or the
    /// declaration is not followed by a newline, `}`, or end-of-file.
    fn parse_field_decl(&mut self, start: usize, is_const: bool) -> ParserResult<ClassMember> {
        let name_start = self.cur_start();
        let name = match self.peek().clone() {
            Token::FieldIdentifier(n) | Token::ImplementationFieldIdentifier(n) => {
                self.advance();
                n
            }
            _ => return Err(self.error_here(strs(&["field name"]))),
        };
        let name_range = (name_start..self.prev_end).into();
        let annotation = if self.eat(&Token::Colon) { Some(self.parse_type_annotation()?) } else { None };
        let default = if self.eat(&Token::Equal) { Some(self.parse_expr()?) } else { None };
        let range = (start..self.prev_end).into();
        match self.peek() {
            Token::Newline => {
                self.advance();
            }
            Token::RBrace | Token::Eof => {}
            _ => return Err(self.error_here(strs(&["newline", "\"}\""]))),
        }
        Ok(ClassMember::Field(FieldDef {
            name,
            name_range,
            mutable: !is_const,
            is_static: false,
            annotation,
            default,
            attributes: Vec::new(),
            range,
        }))
    }

    /// Parses a single class member: a method, getter, or setter.
    ///
    /// A trailing `=` after the name marks a setter (whose external parameter
    /// label is always `put`); an explicit parameter list marks a method; neither marks
    /// a getter. Mirrors the LALRPOP `ClassMember` rule.
    ///
    /// # Errors
    ///
    /// Returns an error if the member name, parameter list, or body is
    /// malformed.
    fn parse_class_member(&mut self) -> ParserResult<ClassMember> {
        let start = self.cur_start();
        // Field grammar (ADR-0064 §4, U-BINDINGS §4.1), in dispatch order:
        // `const` unambiguously starts a field; `let` at field position is a
        // hard error (L-2 — there is no mutable-with-keyword field form);
        // a field token followed by a terminator or initializer is the bare
        // mutable field form. Field tokens in method-shaped positions fall
        // through and are rejected as non-selector names below.
        if matches!(self.peek(), Token::Const) {
            self.advance();
            return self.parse_field_decl(start, true);
        }
        if matches!(self.peek(), Token::Let) {
            self.advance();
            let range = start..self.prev_end;
            return Err(SyntaxError {
                kind: SyntaxErrorKind::Message("mutable fields take no keyword; write `_name` instead of `let _name`".to_string()),
                range,
            });
        }
        if matches!(self.peek(), Token::Class) && matches!(self.peek_next(), Token::Identifier(_)) {
            let range = start..self.cur_start() + 5;
            return Err(SyntaxError {
                kind: SyntaxErrorKind::Message("help: `class foo()` is legacy syntax; use `@class foo()`".to_string()),
                range,
            });
        }
        if matches!(self.peek(), Token::FieldIdentifier(_) | Token::ImplementationFieldIdentifier(_))
            && matches!(self.peek_next(), Token::Newline | Token::RBrace | Token::Eof | Token::Equal | Token::Colon)
        {
            return self.parse_field_decl(start, false);
        }
        // U-INDEX (ADR-0060): a bracket subscript method (`[_ idx] { ... }` /
        // `[_ idx]=(put value) { ... }`) is a distinct grammar production, not
        // routed through `parse_method_name` — there is no separate name
        // token at all, the brackets themselves are the whole of this
        // member's identity. Must be checked before the `Construct`/name
        // branches below, which never expect a leading `[`.
        if matches!(self.peek(), Token::LBracket) {
            return self.parse_index_member(start);
        }
        if matches!(self.peek(), Token::Construct) {
            let construct_start = self.cur_start();
            self.advance(); // consume `construct`
            let construct_end = self.prev_end;
            let range = construct_start..construct_end;

            return Err(SyntaxError {
                kind: SyntaxErrorKind::UnrecognizedToken {
                    token: "construct".to_string(),
                    expected: vec!["@constructor".to_string()],
                },
                range,
            });
        }
        if self.eat(&Token::Static) {
            return Err(SyntaxError {
                kind: SyntaxErrorKind::Message("`static` member syntax is retired; use `@class`".to_string()),
                range: start..self.prev_end,
            });
        }
        let is_static = false;
        let name_start = self.cur_start();
        let name = self.parse_method_name()?;
        let name_range = (name_start..self.prev_end).into();
        let has_equal = self.eat(&Token::Equal);
        if has_equal {
            self.expect(&Token::LParen, &["\"(\""])?;
            let start_put = self.cur_start();
            let put_str = self.expect_identifier(&["\"put\""])?;
            if put_str != "put" {
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::Message("setter parameter must start with \"put\"".to_string()),
                    range: start_put..self.prev_end,
                });
            }
            let local_start = self.cur_start();
            let local_name = self.expect_identifier(&["parameter name"])?;
            let local_range = (local_start..self.prev_end).into();
            let annotation = if self.eat(&Token::Colon) { Some(self.parse_type_annotation()?) } else { None };
            self.expect(&Token::RParen, &["\")\""])?;
            let param = ParameterDef {
                name: local_name,
                name_range: local_range,
                label: None,
                label_range: None,
                rest_mode: RestMode::None,
                annotation,
                range: (start_put..self.prev_end).into(),
            };
            let return_annotation = if self.eat(&Token::Arrow) { Some(self.parse_type_annotation()?) } else { None };
            let body = self.parse_member_body()?;
            let range = (start..self.prev_end).into();
            return Ok(ClassMember::Setter(SetterDef {
                name,
                param,
                return_annotation,
                body,
                is_static,
                attributes: Vec::new(),
                range,
                name_range,
            }));
        }
        let generic_parameters = if matches!(self.peek(), Token::Less) {
            self.parse_generic_parameters(GenericBinderContext::Callable)?
        } else {
            Vec::new()
        };
        let params = if self.eat(&Token::LParen) {
            let list = self.parse_selector_params(Token::RParen)?;
            self.expect(&Token::RParen, &["\")\""])?;
            Some(list)
        } else {
            None
        };
        let return_annotation = if self.eat(&Token::Arrow) { Some(self.parse_type_annotation()?) } else { None };
        let where_clause = if matches!(self.peek(), Token::Where) {
            Some(self.parse_where_clause()?)
        } else {
            None
        };
        let body = self.parse_member_body()?;
        let range = (start..self.prev_end).into();
        if let Some(params) = params {
            Ok(ClassMember::Method(MethodDef {
                name,
                generic_parameters,
                params,
                return_annotation,
                where_clause,
                body,
                is_static,
                is_constructor: false,
                attributes: Vec::new(),
                range,
                name_range,
            }))
        } else {
            if !generic_parameters.is_empty() {
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::Message("generic parameters not permitted on getters".to_string()),
                    range: start..self.prev_end,
                });
            }
            Ok(ClassMember::Getter(GetterDef {
                name,
                return_annotation,
                body,
                is_static,
                attributes: Vec::new(),
                range,
                name_range,
            }))
        }
    }

    /// Parses a bracket subscript method — `[_ idx] { ... }` /
    /// `[_ idx]=(put value) { ... }` / `[] { ... }` / `[]=(put value) { ... }` (U-INDEX,
    /// [ADR-0060](../../docs/adr/accepted/0060-index-operator-as-real-selector.md)).
    fn parse_index_member(&mut self, start: usize) -> ParserResult<ClassMember> {
        let name_start = self.cur_start();
        self.expect(&Token::LBracket, &["\"[\""])?;
        let params = self.parse_selector_params(Token::RBracket)?;
        if let Some(rest) = params.iter().find(|param| param.is_rest()) {
            return Err(SyntaxError {
                kind: SyntaxErrorKind::RestParameter(RestParameterErrorKind::UnsupportedInSubscript),
                range: rest.range.start..rest.range.end,
            });
        }
        self.expect(&Token::RBracket, &["\"]\""])?;
        let name_range = (name_start..self.prev_end).into();
        let accessor = if self.eat(&Token::Equal) {
            self.expect(&Token::LParen, &["\"(\""])?;
            let start_put = self.cur_start();
            let put_str = self.expect_identifier(&["\"put\""])?;
            if put_str != "put" {
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::Message("setter parameter must start with \"put\"".to_string()),
                    range: start_put..self.prev_end,
                });
            }
            let local_start = self.cur_start();
            let local_name = self.expect_identifier(&["parameter name"])?;
            let annotation = if self.eat(&Token::Colon) { Some(self.parse_type_annotation()?) } else { None };
            self.expect(&Token::RParen, &["\")\""])?;
            let put = ParameterDef {
                name: local_name,
                name_range: (local_start..self.prev_end).into(),
                label: None,
                label_range: None,
                rest_mode: RestMode::None,
                annotation,
                range: (start_put..self.prev_end).into(),
            };
            IndexAccessor::Set { put: Box::new(put) }
        } else {
            IndexAccessor::Get
        };
        let return_annotation = if self.eat(&Token::Arrow) { Some(self.parse_type_annotation()?) } else { None };
        let body = self.parse_method_block()?;
        let range = (start..self.prev_end).into();
        Ok(ClassMember::Index(IndexMethodDef {
            params,
            accessor,
            return_annotation,
            body,
            attributes: Vec::new(),
            range,
            name_range,
        }))
    }

    /// Parses a method name: an identifier or an overloadable operator token.
    ///
    /// # Errors
    ///
    /// Returns an error if the current token is neither an identifier nor an
    /// operator usable as a selector.
    fn parse_method_name(&mut self) -> ParserResult<String> {
        let name = match self.peek() {
            Token::FieldIdentifier(_) | Token::ImplementationFieldIdentifier(_) => {
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::Message("a field identifier cannot be used as a method name".to_string()),
                    range: self.tokens[self.pos].start..self.tokens[self.pos].end,
                });
            }
            Token::Identifier(n) | Token::ImplementationSelectorIdentifier(n) => n.clone(),
            Token::Plus => "+".to_string(),
            Token::Minus => "-".to_string(),
            Token::Asterisk => "*".to_string(),
            Token::DoubleAsterisk | Token::Power => "**".to_string(),
            Token::TripleAsterisk => "***".to_string(),
            Token::Slash => "/".to_string(),
            Token::SlashTilde => "~/".to_string(),
            Token::Percent => "%".to_string(),
            Token::ShiftLeft => "<<".to_string(),
            Token::ShiftRight => ">>".to_string(),
            Token::Ampersand => "&".to_string(),
            Token::Pipe => "|".to_string(),
            Token::Caret => "^".to_string(),
            Token::Tilde => "~".to_string(),
            Token::EqualEqual => "==".to_string(),
            Token::TripleEqual => "===".to_string(),
            Token::BangEqual => "!=".to_string(),
            Token::Less => "<".to_string(),
            Token::LessEqual => "<=".to_string(),
            Token::Greater => ">".to_string(),
            Token::GreaterEqual => ">=".to_string(),
            Token::Spaceship => "<=>".to_string(),
            Token::And => "and".to_string(),
            Token::Or => "or".to_string(),
            Token::Not => "not".to_string(),
            Token::Is => "is".to_string(),
            // `class` is a declaration keyword elsewhere, but is also the
            // canonical Object class getter/setter selector.
            Token::Class => "class".to_string(),
            // `try` is a statement keyword elsewhere, but remains the
            // canonical Fiber resume selector.
            Token::Try => "try".to_string(),
            // `from` is a module-syntax keyword, but remains valid as a
            // selector for existing APIs such as `Map.from(...)`.
            Token::From => "from".to_string(),
            Token::Match => "match".to_string(),
            _ => return Err(self.error_here(strs(&["identifier", "operator"]))),
        };
        self.advance();
        Ok(self.extend_selector_name(name))
    }

    /// Parses a parenthesized parameter list. Declaration labels use the
    /// no-colon `external local` form (or just `external` when both names
    /// match). The parser preserves all three rest modes; compiler scope
    /// validation decides which member kinds may use them.
    ///
    /// Shared by method and constructor parameter lists, and — since
    /// U-INDEX/ADR-0060 substitutes `[`/`]` for `(`/`)` — bracket subscript
    /// method parameter lists too (block-literal parameters are parsed by a
    /// separate ad hoc scanner in [`Parser::parse_primary`] and never reach
    /// this function, so no block-literal guard is needed here).
    ///
    /// # Errors
    ///
    /// Returns a structured rest-parameter diagnostic (not a panic) for an
    /// invalid lane ordering or combination. `*rest` is allowed before fixed
    /// labeled parameters and may be followed by one terminal `**rest`;
    /// `**rest` is terminal; `***rest` is terminal and exclusive.
    fn parse_selector_params(&mut self, end: Token) -> ParserResult<Vec<ParameterDef>> {
        self.skip_newlines();
        if self.peek() == &end {
            return Ok(Vec::new());
        }
        let mut params: Vec<ParameterDef> = Vec::new();
        let mut any_labeled = false;
        let mut positional_rest = false;
        let mut labeled_rest = false;
        let mut complete_rest = false;
        let mut labels = std::collections::HashMap::<String, SourceRange>::new();
        loop {
            self.skip_newlines();
            let start = self.cur_start();
            if labeled_rest || complete_rest {
                let kind = if labeled_rest && matches!(self.peek(), Token::DoubleAsterisk) {
                    RestParameterErrorKind::DuplicateLabeled
                } else {
                    RestParameterErrorKind::AfterTerminal
                };
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::RestParameter(kind),
                    range: start..start,
                });
            }
            let rest_mode = if self.eat(&Token::TripleAsterisk) {
                Some(RestMode::Complete)
            } else if self.eat(&Token::DoubleAsterisk) {
                Some(RestMode::Labeled)
            } else if self.eat(&Token::Asterisk) {
                Some(RestMode::Positional)
            } else {
                None
            };
            if let Some(rest_mode) = rest_mode {
                let name_start = self.cur_start();
                let name = self.expect_identifier(&["parameter name"])?;
                let name_range = (name_start..self.prev_end).into();
                if rest_mode == RestMode::Positional && any_labeled {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::RestParameter(RestParameterErrorKind::PositionalAfterLabeled),
                        range: start..self.prev_end,
                    });
                }
                let annotation = if self.eat(&Token::Colon) { Some(self.parse_type_annotation()?) } else { None };
                let range: SourceRange = (start..self.prev_end).into();
                match rest_mode {
                    RestMode::Positional if positional_rest || complete_rest => {
                        return Err(SyntaxError {
                            kind: SyntaxErrorKind::RestParameter(RestParameterErrorKind::DuplicatePositional),
                            range: range.start..range.end,
                        });
                    }
                    RestMode::Labeled if labeled_rest || complete_rest => {
                        return Err(SyntaxError {
                            kind: SyntaxErrorKind::RestParameter(RestParameterErrorKind::DuplicateLabeled),
                            range: range.start..range.end,
                        });
                    }
                    RestMode::Complete if complete_rest || positional_rest || labeled_rest => {
                        return Err(SyntaxError {
                            kind: SyntaxErrorKind::RestParameter(RestParameterErrorKind::CompleteConflict),
                            range: range.start..range.end,
                        });
                    }
                    _ => {}
                }
                positional_rest |= rest_mode == RestMode::Positional;
                labeled_rest |= rest_mode == RestMode::Labeled;
                complete_rest |= rest_mode == RestMode::Complete;
                params.push(ParameterDef {
                    name,
                    name_range,
                    label: None,
                    label_range: None,
                    rest_mode,
                    annotation,
                    range,
                });
            } else if self.eat(&Token::Underscore) {
                let name_start = self.cur_start();
                let name = self.expect_identifier(&["parameter name"])?;
                let name_range = (name_start..self.prev_end).into();
                if any_labeled || positional_rest {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::RestParameter(RestParameterErrorKind::PositionalAfterLabeledOrRest),
                        range: start..self.prev_end,
                    });
                }
                let annotation = if self.eat(&Token::Colon) { Some(self.parse_type_annotation()?) } else { None };
                let range = (start..self.prev_end).into();
                params.push(ParameterDef {
                    name,
                    name_range,
                    label: None,
                    label_range: None,
                    rest_mode: RestMode::None,
                    annotation,
                    range,
                });
            } else {
                // Reserved words remain illegal local names, but are valid
                // external labels when followed by an ordinary local name
                // (`for key`). Calls already accept the same contextual label
                // vocabulary through `label_name`.
                let first_is_identifier = matches!(self.peek(), Token::Identifier(_));
                let label_start = self.cur_start();
                let first_ident = if let Some(name) = Self::label_name(self.peek()) {
                    let name = name.to_string();
                    self.advance();
                    name
                } else {
                    return Err(self.error_here(strs(&["parameter name", "_", "*"])));
                };
                let label_range = (label_start..self.prev_end).into();
                if matches!(self.peek(), Token::Identifier(_)) {
                    let name_start = self.cur_start();
                    let local_ident = self.expect_identifier(&["parameter name"])?;
                    let name_range = (name_start..self.prev_end).into();
                    let label = first_ident;
                    any_labeled = true;
                    if labels.insert(label.clone(), label_range).is_some() {
                        return Err(SyntaxError {
                            kind: SyntaxErrorKind::Message("duplicate parameter label in selector declaration".to_string()),
                            range: label_range.start..label_range.end,
                        });
                    }
                    let annotation = if self.eat(&Token::Colon) { Some(self.parse_type_annotation()?) } else { None };
                    params.push(ParameterDef {
                        name: local_ident,
                        name_range,
                        label: Some(label),
                        label_range: Some(label_range),
                        rest_mode: RestMode::None,
                        annotation,
                        range: (start..self.prev_end).into(),
                    });
                    self.skip_newlines();
                    if !self.eat(&Token::Comma) {
                        break;
                    }
                    continue;
                }
                if !first_is_identifier {
                    return Err(self.error_here(strs(&["local parameter name after reserved label"])));
                }
                let annotation = if self.eat(&Token::Colon) { Some(self.parse_type_annotation()?) } else { None };
                any_labeled = true;
                let label = Some(first_ident.clone());
                if labels.insert(first_ident.clone(), label_range).is_some() {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message("duplicate parameter label in selector declaration".to_string()),
                        range: label_range.start..label_range.end,
                    });
                }
                params.push(ParameterDef {
                    name: first_ident,
                    name_range: label_range,
                    label,
                    label_range: Some(label_range),
                    rest_mode: RestMode::None,
                    annotation,
                    range: (start..self.prev_end).into(),
                });
            }
            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        self.skip_newlines();
        Ok(params)
    }

    /// Parses `|params| expression` or `|params| { statements }` into the
    /// existing block AST node. This is shared by ordinary primary expressions
    /// and structurally-confirmed trailing closure arguments.
    fn parse_closure_literal(&mut self, body_requirement: ClosureBodyRequirement) -> ParserResult<Expr> {
        let start = self.cur_start();
        self.expect(&Token::Pipe, &["\"|\""])?;
        self.skip_newlines();

        let mut fixed = Vec::new();
        let mut positional_rest = None;
        if !self.eat(&Token::Pipe) {
            loop {
                if self.eat(&Token::Asterisk) {
                    if positional_rest.is_some() {
                        return Err(self.error_here(strs(&["only one terminal positional rest parameter is allowed in a closure"])));
                    }
                    let rest_start = self.cur_start();
                    let rest = if self.eat(&Token::Underscore) {
                        "_".to_string()
                    } else {
                        self.expect_identifier(&["positional rest parameter name after `*`"])?
                    };
                    positional_rest = Some(ClosureParameter {
                        name: rest,
                        range: (rest_start..self.prev_end).into(),
                    });
                    self.skip_newlines();
                    if !self.eat(&Token::Pipe) {
                        return Err(self.error_here(strs(&["closure positional rest parameter must be terminal"])));
                    }
                    break;
                }
                if matches!(self.peek(), Token::DoubleAsterisk | Token::TripleAsterisk) {
                    return Err(self.error_here(strs(&["closures support only positional rest parameters written `*name`"])));
                }
                let param_start = self.cur_start();
                let param = if self.eat(&Token::Underscore) {
                    "_".to_string()
                } else {
                    self.expect_identifier(&["closure positional parameter name"])?
                };
                fixed.push(ClosureParameter {
                    name: param,
                    range: (param_start..self.prev_end).into(),
                });
                self.skip_newlines();
                if self.eat(&Token::Pipe) {
                    break;
                }
                self.expect(&Token::Comma, &["\",\"", "\"|\""])?;
                self.skip_newlines();
            }
        }
        self.skip_newlines();

        let (body, expr_body) = if self.eat(&Token::LBrace) {
            let body = self.parse_block_statements()?;
            self.expect(&Token::RBrace, &["\"}\""])?;
            (body, false)
        } else {
            if matches!(body_requirement, ClosureBodyRequirement::Braced) {
                return Err(self.error_here(strs(&["\"{\" (trailing closures require a braced body)"])));
            }
            let expr = self.parse_expr()?;
            let range = expr.range();
            (vec![Statement::Expr { expr, range }], true)
        };

        Ok(Expr::Block(Box::new(BlockExpr {
            params: ClosureParameters { fixed, positional_rest },
            body,
            expr_body,
            range: (start..self.prev_end).into(),
        })))
    }

    /// Parses a braced method body.
    ///
    /// Parses a member body, distinguishing between a braced `{ ... }` block and a declaration-only terminator.
    fn parse_member_body(&mut self) -> ParserResult<crate::ast::MemberBody> {
        match self.peek() {
            Token::LBrace => {
                self.advance();
                let stmts = self.parse_block_statements()?;
                self.expect(&Token::RBrace, &["\"}\""])?;
                Ok(crate::ast::MemberBody::Block(stmts))
            }
            Token::Newline | Token::RBrace | Token::Eof => Ok(crate::ast::MemberBody::Declaration),
            Token::At if self.has_source_newline_before_current() => Ok(crate::ast::MemberBody::Declaration),
            _ => Err(self.error_here(strs(&["\"{\"", "newline or member terminator"]))),
        }
    }

    /// # Errors
    ///
    /// Returns an error if neither body form is present or the body is
    /// malformed.
    fn parse_method_block(&mut self) -> ParserResult<Vec<Statement>> {
        match self.peek() {
            Token::LBrace => {
                self.advance();
                let stmts = self.parse_block_statements()?;
                self.expect(&Token::RBrace, &["\"}\""])?;
                Ok(stmts)
            }
            _ => Err(self.error_here(strs(&["\"{\" (method bodies must be braced)"]))),
        }
    }

    /// Parses the statements inside a `{ }` block up to the closing `}`.
    ///
    /// Small statements are separated by `;` and each group is terminated by a
    /// newline (matching the LALRPOP `BlockStatements` rule); blank lines are
    /// ignored. Class declarations are module top-level only (PDR-0001
    /// ruling 5, U-CLASSCLOSE §6) — a nested `class` still *parses* (so the
    /// error carries a real span and the surrounding block otherwise reads
    /// normally) but is rejected right after with `class.nested_declaration`.
    ///
    /// # Errors
    ///
    /// Returns an error on a malformed statement, a missing terminating
    /// newline, or a nested `class` declaration.
    fn parse_block_statements(&mut self) -> ParserResult<Vec<Statement>> {
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek() {
                Token::RBrace | Token::Eof => break,
                // `Token::At` also dispatches here — a nested class may carry
                // header attributes too (U-ANNOT-LAYOUT §3.3), same dispatch
                // shape as the top-level `parse_top_item`, but nesting itself
                // is rejected below rather than accepted.
                //
                // Synthesized `Statement::Class` nodes (`@variant`,
                // `compiler/attributes.rs`) never reach this arm — they are
                // built as Rust struct literals and handed straight to the
                // compiler, bypassing the parser entirely (U-CLASSCLOSE §1.3).
                // This ban is therefore a syntax-level check only; any future
                // desugar that synthesizes a `Statement::Class` inherits the
                // same bypass silently.
                // A declaration starts `class Name`; `class.` is an expression
                // spelling for `self.class` and must continue through small
                // statement parsing below. `@` keeps its existing decorated
                // class-declaration path.
                Token::At | Token::Class | Token::Enum if matches!(self.peek(), Token::At) || matches!(self.peek_next(), Token::Identifier(_)) => {
                    let mut header_attrs = Vec::new();
                    while matches!(self.peek(), Token::At) {
                        header_attrs.push(self.parse_attribute()?);
                        self.skip_newlines();
                    }
                    let is_enum = matches!(self.peek(), Token::Enum);
                    let stmt = if is_enum {
                        self.parse_enum(header_attrs)?
                    } else {
                        self.parse_class(header_attrs)?
                    };
                    let (keyword_start, kw_len, decl_kind) = match &stmt {
                        Statement::Class(class_def) => (class_def.range.start, "class".len(), "class"),
                        Statement::Enum(enum_def) => (enum_def.range.start, "enum".len(), "enum"),
                        _ => unreachable!(),
                    };
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message(format!(
                            "{decl_kind}.nested_declaration: {decl_kind} declarations are only allowed at a module's top level, not nested inside a block"
                        )),
                        range: keyword_start..keyword_start + kw_len,
                    });
                }
                _ => {
                    loop {
                        stmts.push(self.parse_small_statement()?);
                        if matches!(self.peek(), Token::Semicolon) {
                            self.advance();
                            if matches!(self.peek(), Token::Newline | Token::RBrace | Token::Eof) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    match self.peek() {
                        Token::Newline => {
                            self.advance();
                        }
                        Token::RBrace | Token::Eof => {}
                        _ => return Err(self.error_here(strs(&["\";\"", "newline"]))),
                    }
                }
            }
        }
        Ok(stmts)
    }

    // ── Expressions (precedence climbing) ────────────────────────────────────

    /// Parses a full expression (lowest precedence: assignment).
    ///
    /// # Errors
    ///
    /// Propagates any error from the expression grammar.
    fn parse_expr(&mut self) -> ParserResult<Expr> {
        self.parse_assignment()
    }

    /// Parses a control-header expression without allowing a following brace
    /// to attach as a trailing closure. The brace belongs to the control
    /// construct's body, even when the condition ends in a member getter or
    /// method call.
    fn parse_expr_without_trailing_closures(&mut self) -> ParserResult<Expr> {
        let previous = self.trailing_closures_enabled;
        self.trailing_closures_enabled = false;
        let result = self.parse_expr();
        self.trailing_closures_enabled = previous;
        result
    }

    /// Parses assignments and compound assignments (right-associative).
    ///
    /// A simple `=` produces an [`Expr::Assignment`], or an [`Expr::SetProperty`]
    /// when the target is a `.property` access. A compound assignment (`+=`,
    /// `-=`, ...) desugars to an assignment of a binary operation, exactly as
    /// the LALRPOP grammar did.
    ///
    /// # Errors
    ///
    /// Propagates any error from the operand expressions.
    fn parse_assignment(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        let left = self.parse_coalesce()?;

        if let Some(op) = compound_op(self.peek()) {
            let op_start = self.cur_start();
            self.advance();
            let op_range = Some((op_start..self.prev_end).into());
            let value = self.parse_assignment()?;
            let range = (start..self.prev_end).into();
            let binary = Expr::Binary(Box::new(BinaryExpr {
                op,
                op_range,
                left: left.clone(),
                right: value,
                range,
            }));
            return Ok(Expr::Assignment(Box::new(AssignmentExpr {
                name: Box::new(left),
                value: binary,
                range,
            })));
        }

        if matches!(self.peek(), Token::Equal) {
            self.advance();
            let value = self.parse_assignment()?;
            let range = (start..self.prev_end).into();
            return Ok(match left {
                Expr::GetProperty(get) => Expr::SetProperty(Box::new(SetPropertyExpr {
                    object: get.object,
                    property: get.property,
                    property_range: get.property_range,
                    value,
                    range,
                })),
                Expr::Index(ix) => Expr::SetIndex(Box::new(SetIndexExpr {
                    object: ix.object,
                    args: ix.args,
                    selector_range: ix.selector_range,
                    value,
                    range,
                })),
                other => Expr::Assignment(Box::new(AssignmentExpr {
                    name: Box::new(other),
                    value,
                    range,
                })),
            });
        }

        Ok(left)
    }

    /// Parses the non-associative Range tier between comparison and additive expressions.
    fn parse_range(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        let lower = if matches!(self.peek(), Token::DotDot | Token::DotDotEqual) {
            None
        } else {
            Some(self.parse_binary(5)?)
        };

        let upper_inclusive = match self.peek() {
            Token::DotDot => false,
            Token::DotDotEqual => true,
            _ => return Ok(lower.expect("range parser reaches this branch only after parsing an expression")),
        };
        self.advance();
        let upper = self.starts_expression().then(|| self.parse_binary(5)).transpose()?;
        if upper_inclusive && upper.is_none() {
            return Err(self.error_here(strs(&["an upper bound after `..=`"])));
        }
        if matches!(self.peek(), Token::DotDot | Token::DotDotEqual) {
            return Err(self.error_here(strs(&["end of Range expression (Range operators are non-associative)"])));
        }
        Ok(Expr::Range(Box::new(RangeExpr {
            lower,
            upper,
            upper_inclusive,
            range: (start..self.prev_end).into(),
        })))
    }

    /// Determines endpoint presence from grammar, not whitespace.
    fn starts_expression(&self) -> bool {
        match self.peek() {
            Token::If
            | Token::While
            | Token::True
            | Token::False
            | Token::Int { .. }
            | Token::Float(_)
            | Token::String(_)
            | Token::StringInterp(_)
            | Token::Hash
            | Token::QuotedSymbol(_)
            | Token::Identifier(_)
            | Token::SelfKw
            | Token::Class
            | Token::Super
            | Token::LBracket
            | Token::LParen
            | Token::RecordLBrace
            | Token::Pipe
            | Token::Plus
            | Token::Minus
            | Token::Not
            | Token::Tilde
            | Token::DotDotDot => true,
            Token::LBrace => {
                (matches!(self.peek_next(), Token::Identifier(_)) && self.tokens.get(self.pos + 2).is_some_and(|t| matches!(t.token, Token::Colon)))
                    || self.starts_computed_map_literal()
                    || matches!(self.peek_next(), Token::Asterisk | Token::DoubleAsterisk | Token::TripleAsterisk | Token::Power)
            }
            _ => false,
        }
    }

    /// Parses a `??` null-coalescing chain (right-associative), desugaring to
    /// `Option` sends.
    ///
    /// `??` binds looser than every arithmetic/comparison operator but tighter
    /// than assignment ([lexical-structure §9](../../docs/spec/lexical-structure.md)).
    /// It is **right-associative**, so `a ?? b ?? c` groups as `a ?? (b ?? c)`,
    /// and its right operand is short-circuiting. Per ADR-0007 / values-and-absence
    /// §3.4, `a ?? b` desugars directly to the message send `a.orElse { b }`
    /// (an [`Expr::MethodCall`] whose sole argument is a zero-parameter
    /// [`Expr::Block`] wrapping `b`), so no dedicated AST node is needed.
    ///
    /// # Errors
    ///
    /// Propagates any error from the operand expressions.
    fn parse_coalesce(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        let left = self.parse_binary(1)?;
        if matches!(self.peek(), Token::CoalesceQuestion) {
            self.advance(); // '??'
            // Right-associative: recurse so `a ?? b ?? c` is `a ?? (b ?? c)`.
            let right = self.parse_coalesce()?;
            let range = (start..self.prev_end).into();
            let block = self.wrap_block_thunk(right, range);
            return Ok(Expr::MethodCall(Box::new(MethodCallExpr {
                object: left,
                method: "orElse".to_string(),
                method_range: None,
                args: vec![PackItem::Positional { expr: block, range }],
                range,
            })));
        }
        Ok(left)
    }

    /// Desugars an interpolated string literal (ADR-0022) into a `+`-chain of
    /// stringified segments.
    ///
    /// Each [`StringSegment::Literal`] becomes an [`Expr::String`]; each
    /// [`StringSegment::Expr`] is re-parsed from its source slice and wrapped
    /// in a `toString` getter (`Expr::GetProperty`). The parts are folded left
    /// with binary `+` ([`BinaryOp::Add`]). If the first segment is an expression,
    /// an empty string accumulator `""` is seeded. For example, `"a \(x) b"` lowers to
    /// `("a " + x.toString) + " b"`.
    ///
    /// # Errors
    ///
    /// Returns a [`SyntaxError`] if any interpolated expression fails to parse,
    /// or if an interpolation body is empty (no expression).
    fn desugar_string_interp(&self, segments: Vec<StringSegment>, outer_range: SourceRange) -> ParserResult<Expr> {
        let starts_with_expr = matches!(segments.first(), Some(StringSegment::Expr { .. }));

        let mut acc = if starts_with_expr {
            Some(Expr::String {
                value: String::new(),
                range: outer_range,
            })
        } else {
            None
        };

        for segment in segments {
            let part = match segment {
                StringSegment::Literal(value) => Expr::String { value, range: outer_range },
                StringSegment::Expr { source, range } => {
                    let inner = self.parse_interp_expr(&source, range)?;
                    Expr::GetProperty(Box::new(GetPropertyExpr {
                        object: inner,
                        property: "toString".to_string(),
                        property_range: None,
                        range: outer_range,
                    }))
                }
            };

            acc = Some(match acc {
                None => part,
                Some(left) => Expr::Binary(Box::new(BinaryExpr {
                    op: BinaryOp::Add,
                    op_range: None,
                    left,
                    right: part,
                    range: outer_range,
                })),
            });
        }

        Ok(acc.unwrap_or(Expr::String {
            value: String::new(),
            range: outer_range,
        }))
    }

    /// Re-parses a single interpolated-expression source slice into an [`Expr`].
    ///
    /// Requires exactly one expression followed by optional trivia/newlines and EOF.
    fn parse_interp_expr(&self, source: &str, body_range: Range<usize>) -> ParserResult<Expr> {
        let absolute_start = self.offset + body_range.start;
        let absolute_range = (self.offset + body_range.start)..(self.offset + body_range.end);

        let mut parser = Parser::new(source, absolute_start);

        if let Some(err) = parser.errors.first().cloned() {
            return Err(err);
        }

        parser.skip_newlines();

        if matches!(parser.peek(), Token::Eof) {
            return Err(SyntaxError {
                kind: SyntaxErrorKind::EmptyInterpolation,
                range: absolute_range,
            });
        }

        let expr = parser.parse_expr()?;

        parser.skip_newlines();

        if !matches!(parser.peek(), Token::Eof) {
            return Err(parser.error_here(vec!["end of interpolation".to_string()]));
        }

        Ok(expr)
    }

    /// Wraps `expr` in a zero-parameter block literal `{ expr }`, used to build
    /// the thunk operand of a `??` desugar (`a.orElse { b }`).
    ///
    /// The block has [`BlockExpr::expr_body`] set so `expr` becomes its result
    /// value; `range` spans the desugared construct.
    fn wrap_block_thunk(&self, expr: Expr, range: SourceRange) -> Expr {
        Expr::Block(Box::new(BlockExpr {
            params: ClosureParameters::default(),
            body: vec![Statement::Expr { expr, range }],
            expr_body: true,
            range,
        }))
    }

    /// Wraps `body` in a single-parameter block literal `{ <param> => body }`,
    /// used to build the mapper operand of a `?.` desugar
    /// (`opt.map { x => x.m(..) }`).
    ///
    /// `param` is the synthetic receiver name that `body` references; it is not
    /// user-writable so it can never capture or shadow a source variable.
    /// `range` spans the desugared construct.
    fn wrap_block_mapper(&self, param: String, body: Expr, range: SourceRange) -> Expr {
        Expr::Block(Box::new(BlockExpr {
            params: ClosureParameters::fixed(vec![param]),
            body: vec![Statement::Expr { expr: body, range }],
            expr_body: true,
            range,
        }))
    }

    /// Parses a binary-operator expression at or above precedence `min_prec`,
    /// using precedence climbing.
    ///
    /// All binary operators here are left-associative; the recursive call uses
    /// `prec + 1` to enforce that. The resulting node's span starts at the first
    /// operand and ends at the last, reproducing LALRPOP's location semantics.
    ///
    /// # Errors
    ///
    /// Propagates any error from the operand expressions.
    fn parse_binary(&mut self, min_prec: u8) -> ParserResult<Expr> {
        let start = self.cur_start();
        let mut left = if min_prec <= 4 { self.parse_range()? } else { self.parse_unary()? };
        let mut chain_operands: Option<Vec<Expr>> = None;
        let mut chain_operators: Vec<(RelationOp, SourceRange)> = Vec::new();
        loop {
            // `is`/`is!`/`is not`/`is! not`/`is in`/`is! in`/`is not in`/`is! not in`
            // sit at the equality tier (prec 3) but are not a `binary_op` entry.
            // Gated to `min_prec <= 3` so a nested RHS (`parse_binary(4)`)
            // never re-enters these arms, keeping operators from chaining through recursion.
            if min_prec <= 3 && matches!(self.peek(), Token::Is) {
                left = self.parse_is(left, start)?;
                continue;
            }
            if min_prec <= 3 && matches!(self.peek(), Token::Not) && matches!(self.peek_next(), Token::In) {
                left = self.parse_not_in(left, start)?;
                continue;
            }
            if min_prec <= 3 && matches!(self.peek(), Token::In) {
                left = self.parse_in(left, start)?;
                continue;
            }
            let (prec, op) = if let Some((prec, op)) = binary_op(self.peek()) {
                (prec, RelationOp::Binary(op))
            } else if min_prec <= 4 {
                let Some(op) = contextual_relation_op(self.peek()) else {
                    if let Some(operands) = chain_operands.take() {
                        left = finish_chain(operands, std::mem::take(&mut chain_operators), (start..self.prev_end).into());
                    }
                    break;
                };
                (4, op)
            } else {
                break;
            };
            if prec < min_prec {
                break;
            }
            let op_start = self.cur_start();
            self.advance();
            let op_range: SourceRange = (op_start..self.prev_end).into();
            let right = self.parse_binary(prec + 1)?;
            let range = (start..self.prev_end).into();
            if let RelationOp::Binary(binary) = op.clone()
                && is_chain_relation(&binary)
            {
                if matches!(&left, Expr::Binary(expr) if matches!(&expr.op, BinaryOp::Compare)) && !self.was_parenthesized(&left) {
                    return Err(self.error_here(strs(&["parenthesize `<=>` before chaining another relation"])));
                }
                if let Some(operands) = &mut chain_operands {
                    operands.push(right);
                } else {
                    chain_operands = Some(vec![left.clone(), right]);
                }
                chain_operators.push((RelationOp::Binary(binary), op_range));
            } else if matches!(op, RelationOp::Matches | RelationOp::Understands) {
                if let Some(operands) = &mut chain_operands {
                    operands.push(right);
                } else {
                    chain_operands = Some(vec![left.clone(), right]);
                }
                chain_operators.push((op, op_range));
            } else {
                if let Some(operands) = chain_operands.take() {
                    left = finish_chain(operands, std::mem::take(&mut chain_operators), range);
                }
                let RelationOp::Binary(binary) = op else {
                    unreachable!("non-chain contextual relation handled above")
                };
                left = Expr::Binary(Box::new(BinaryExpr {
                    op: binary,
                    op_range: Some(op_range),
                    left,
                    right,
                    range,
                }));
            }
        }
        if let Some(operands) = chain_operands {
            left = finish_chain(operands, chain_operators, (start..self.prev_end).into());
        }
        Ok(left)
    }

    fn was_parenthesized(&self, expr: &Expr) -> bool {
        let range = expr.range();
        self.parenthesized_ranges.contains(&(range.start, range.end))
    }

    /// Parses `x in y`.
    fn parse_in(&mut self, left: Expr, start: usize) -> ParserResult<Expr> {
        let op_start = self.cur_start();
        self.advance(); // consume `in`
        let op_range: SourceRange = (op_start..self.prev_end).into();
        if matches!(self.peek(), Token::Not) {
            return Err(self.error_here(strs(&["an expression (did you mean `not in`? Write `x not in y`)"])));
        }
        let right = self.parse_binary(4)?;
        let range = (start..self.prev_end).into();
        if matches!(self.peek(), Token::Is) {
            return Err(self.error_here(strs(&["an expression (chained `is` is not allowed — the result of `in` is a `Bool`)"])));
        }
        if matches!(self.peek(), Token::In) || (matches!(self.peek(), Token::Not) && matches!(self.peek_next(), Token::In)) {
            return Err(self.error_here(strs(&["an expression (chained membership test is not allowed — the result is a `Bool`)"])));
        }
        Ok(Expr::Membership(Box::new(MembershipExpr {
            left,
            right,
            negated: false,
            op_range: Some(op_range),
            range,
        })))
    }

    /// Parses `x not in y`.
    fn parse_not_in(&mut self, left: Expr, start: usize) -> ParserResult<Expr> {
        let op_start = self.cur_start();
        self.advance(); // consume `not`
        self.advance(); // consume `in`
        let op_range: SourceRange = (op_start..self.prev_end).into();
        if matches!(self.peek(), Token::Not) {
            return Err(self.error_here(strs(&["an expression (did you mean `not in`? Write `x not in y`)"])));
        }
        let right = self.parse_binary(4)?;
        let range = (start..self.prev_end).into();
        if matches!(self.peek(), Token::Is) {
            return Err(self.error_here(strs(&["an expression (chained `is` is not allowed — the result of `not in` is a `Bool`)"])));
        }
        if matches!(self.peek(), Token::In) || (matches!(self.peek(), Token::Not) && matches!(self.peek_next(), Token::In)) {
            return Err(self.error_here(strs(&["an expression (chained membership test is not allowed — the result is a `Bool`)"])));
        }
        Ok(Expr::Membership(Box::new(MembershipExpr {
            left,
            right,
            negated: true,
            op_range: Some(op_range),
            range,
        })))
    }

    /// Parses the `is` type-test operator suite (`is`, `is!`, `is not`,
    /// `is! not`, `is in`, `is! in`, `is not in`, `is! not in`) following `left`.
    fn parse_is(&mut self, left: Expr, start: usize) -> ParserResult<Expr> {
        let op_start = self.cur_start();
        self.advance(); // consume `is`
        let is_end = self.prev_end;

        // Strict suffix: `!` contiguous with `is` (no whitespace). A `!`
        // preceded by whitespace is not strict, and — post-U-NEG — a bare
        // `!` elsewhere in expression position is itself a parse error, so
        // this adjacency check is unambiguous.
        let strict = matches!(self.peek(), Token::Bang) && self.cur_start() == is_end;
        if strict {
            self.advance(); // consume `!`
        }

        // `not` particle: greedy, always the negation particle here — never
        // a prefix on the RHS.
        let negate = self.eat(&Token::Not);

        if matches!(self.peek(), Token::In) {
            self.advance(); // consume `in`
            let op_range: SourceRange = (op_start..self.prev_end).into();
            if matches!(self.peek(), Token::Not) {
                return Err(self.error_here(strs(&["an expression (did you mean `is not in` or `is! not in`?)"])));
            }
            let candidates = self.parse_binary(4)?;
            let range = (start..self.prev_end).into();
            if matches!(self.peek(), Token::Is) {
                return Err(self.error_here(strs(&["an expression (chained `is` is not allowed — the result of `is in` is a `Bool`)"])));
            }
            if matches!(self.peek(), Token::In) || (matches!(self.peek(), Token::Not) && matches!(self.peek_next(), Token::In)) {
                return Err(self.error_here(strs(&["an expression (chained membership test is not allowed — the result is a `Bool`)"])));
            }
            return Ok(Expr::IsMembership(Box::new(IsMembershipExpr {
                left,
                candidates,
                strict,
                negated: negate,
                op_range: Some(op_range),
                range,
            })));
        }

        let rhs = self.parse_binary(4)?;
        let range = (start..self.prev_end).into();
        let method = if strict { "is!" } else { "is" }.to_string();
        let base = Expr::MethodCall(Box::new(MethodCallExpr {
            object: left,
            method,
            method_range: None,
            args: vec![PackItem::Positional { expr: rhs, range }],
            range,
        }));
        let result = if negate {
            Expr::Unary(Box::new(UnaryExpr {
                op: UnaryOp::Not,
                op_range: None,
                expr: base,
                range,
            }))
        } else {
            base
        };

        if matches!(self.peek(), Token::Is) {
            return Err(self.error_here(strs(&["an expression (chained `is` is not allowed — the result of `is` is a `Bool`)"])));
        }
        if matches!(self.peek(), Token::In) || (matches!(self.peek(), Token::Not) && matches!(self.peek_next(), Token::In)) {
            return Err(self.error_here(strs(&["an expression (chained membership test is not allowed — the result is a `Bool`)"])));
        }

        Ok(result)
    }

    /// Parses a prefix unary expression (`+x`, `-x`, `not x`, `~x`), or
    /// delegates to [`Parser::parse_call`].
    ///
    /// All four operators lower to bare getter sends (`+`, `-`, `not`, `~`).
    /// `not` is the sole boolean-negation prefix (`syntax/grammar.md`'s
    /// `unary := ( "+" | "-" | "not" | "~" ) unary`,
    /// `syntax/expressions.md` precedence table row 9). U-NEG retires prefix
    /// `!` (`Token::Bang`) as an expression operator — `Token::Bang` now
    /// survives only inside the lexer's `!=` (`Token::BangEqual`)
    /// disambiguation; a bare `!` in expression position is a parse error.
    ///
    /// # Errors
    ///
    /// Propagates any error from the operand expression.
    fn parse_unary(&mut self) -> ParserResult<Expr> {
        let op = match self.peek() {
            Token::Plus => UnaryOp::Plus,
            Token::Minus => UnaryOp::Minus,
            Token::Not => UnaryOp::Not,
            Token::Tilde => UnaryOp::BitNot,
            _ => return self.parse_power(),
        };
        let start = self.cur_start();
        self.advance();
        let op_range: SourceRange = (start..self.prev_end).into();
        let expr = self.parse_unary()?;
        let range = (start..self.prev_end).into();
        Ok(Expr::Unary(Box::new(UnaryExpr {
            op,
            op_range: Some(op_range),
            expr,
            range,
        })))
    }

    /// Parses power expression: `power := postfix [ "**" unary ]`
    fn parse_power(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        let left = self.parse_call()?;
        if matches!(self.peek(), Token::DoubleAsterisk | Token::Power) {
            let op_start = self.cur_start();
            self.advance(); // consume `**`
            let op_range: SourceRange = (op_start..self.prev_end).into();
            let right = self.parse_unary()?;
            let range = (start..self.prev_end).into();
            return Ok(Expr::Binary(Box::new(BinaryExpr {
                op: BinaryOp::Power,
                op_range: Some(op_range),
                left,
                right,
                range,
            })));
        }
        Ok(left)
    }

    /// Parses postfix `.property` accesses and `.method(args)` calls over a
    /// primary expression.
    ///
    /// Member access left-associates; a following `(` turns an access into a
    /// method call.
    ///
    /// # Errors
    ///
    /// Propagates any error from the primary expression, property name, or
    /// argument list.
    fn parse_call(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        let mut expr = self.parse_primary()?;
        let mut trailing_target = TrailingTarget::None;
        loop {
            if matches!(self.peek(), Token::Newline) {
                let mut next = self.pos;
                while matches!(self.tokens.get(next).map(|lexeme| &lexeme.token), Some(Token::Newline)) {
                    next += 1;
                }
                let continues_postfix = matches!(
                    self.tokens.get(next).map(|lexeme| &lexeme.token),
                    Some(Token::Dot | Token::QuestionDot | Token::ColonColon)
                );
                let continues_labelled_closure = trailing_target == TrailingTarget::MemberSend && self.starts_labelled_braced_closure_literal(next);
                if continues_postfix || continues_labelled_closure {
                    while self.pos < next {
                        self.advance();
                    }
                } else {
                    break;
                }
            }

            if trailing_target == TrailingTarget::MemberSend {
                if self.trailing_closures_enabled && matches!(self.peek(), Token::LBrace) {
                    let end_before = self.cur_start();
                    let closure = self.parse_brace_block()?;
                    expr = self.attach_trailing_arguments(
                        expr,
                        vec![PackItem::Positional {
                            expr: closure,
                            range: (end_before..self.prev_end).into(),
                        }],
                        self.prev_end,
                    )?;
                    // Keep eligibility for a following labelled closure on
                    // the same member send, including when it starts after a
                    // newline (`send { ... }\n  label: { ... }`).
                    trailing_target = TrailingTarget::MemberSend;
                    continue;
                }
                if self.trailing_closures_enabled
                    && let Some((args, end)) = self.parse_trailing_closure_arguments()?
                {
                    expr = self.attach_trailing_arguments(expr, args, end)?;
                    // Keep member-send eligibility for a following labeled
                    // trailing closure (`send || {} label: || {}`).
                    trailing_target = TrailingTarget::MemberSend;
                    continue;
                }
            }

            if matches!(self.peek(), Token::Less) && self.cur_start() == self.prev_end && self.is_type_arguments_ahead() {
                if let Some(origin) = Self::expr_to_type_annotation(&expr) {
                    self.advance(); // '<'
                    let mut arguments = Vec::new();
                    while !matches!(self.peek(), Token::Greater | Token::ShiftRight | Token::Eof) {
                        arguments.push(self.parse_type_form()?);
                        if !self.eat(&Token::Comma) {
                            break;
                        }
                    }
                    self.expect_greater()?;
                    let range = (start..self.prev_end).into();
                    expr = Expr::TypeForm(Box::new(TypeAnnotation {
                        expr: TypeAnnotationExpr::Application {
                            origin: Box::new(origin),
                            arguments,
                            range,
                        },
                        range,
                    }));
                    trailing_target = TrailingTarget::None;
                    continue;
                }
            }

            if !matches!(
                self.peek(),
                Token::Dot | Token::QuestionDot | Token::LParen | Token::ColonColon | Token::LBracket
            ) {
                break;
            }
            if self.eat(&Token::QuestionDot) {
                expr = self.parse_optional_send(expr, start)?;
                trailing_target = TrailingTarget::None;
            } else if self.eat(&Token::ColonColon) {
                expr = self.parse_associated_suffix(expr, start)?;
                trailing_target = TrailingTarget::None;
            } else if self.eat(&Token::Dot) {
                let is_method_call = matches!(self.peek_next(), Token::LParen);
                let field_kind = if is_method_call {
                    None
                } else {
                    match self.peek() {
                        Token::FieldIdentifier(_) => Some(FieldKind::Source),
                        Token::ImplementationFieldIdentifier(_) => Some(FieldKind::Implementation),
                        _ => None,
                    }
                };
                if let Some(kind) = field_kind {
                    if !matches!(expr, Expr::SelfVar { .. }) {
                        return Err(self.error_here(strs(&["a field on `self` (fields are direct receiver state; only `self._field` is valid)"])));
                    }
                    let value = self.parse_property_name()?;
                    let range = (start..self.prev_end).into();
                    expr = Expr::Field { value, kind, range };
                    trailing_target = TrailingTarget::None;
                    continue;
                }
                let property_start = self.cur_start();
                let property = self.parse_property_name()?;
                let property_range = Some((property_start..self.prev_end).into());
                if self.eat(&Token::LParen) {
                    let args = self.parse_arg_list()?;
                    self.expect(&Token::RParen, &["\")\""])?;
                    let range = (start..self.prev_end).into();
                    expr = Expr::MethodCall(Box::new(MethodCallExpr {
                        object: expr,
                        method: property,
                        method_range: property_range,
                        args,
                        range,
                    }));
                } else {
                    let range = (start..self.prev_end).into();
                    expr = Expr::GetProperty(Box::new(GetPropertyExpr {
                        object: expr,
                        property,
                        property_range,
                        range,
                    }));
                }
                trailing_target = TrailingTarget::MemberSend;
            } else if self.eat(&Token::LParen) {
                let args = self.parse_arg_list()?;
                self.expect(&Token::RParen, &["\")\""])?;
                let range = (start..self.prev_end).into();
                expr = match expr {
                    Expr::Var { value, range: name_range } => Expr::UnqualifiedCall(Box::new(UnqualifiedCallExpr {
                        name: value,
                        name_range: Some(name_range),
                        args,
                        range,
                    })),
                    Expr::ImplementationSelector { value, range: method_range } => Expr::MethodCall(Box::new(MethodCallExpr {
                        object: Expr::SelfVar { range },
                        method: value,
                        method_range: Some(method_range),
                        args,
                        range,
                    })),
                    expr => Expr::MethodCall(Box::new(MethodCallExpr {
                        object: expr,
                        method: "call".to_string(),
                        method_range: None,
                        args,
                        range,
                    })),
                };
                trailing_target = TrailingTarget::None;
            } else if self.eat(&Token::LBracket) {
                // U-INDEX (ADR-0060): the bracket's contents are a full
                // call-shaped argument list — positional + `label:`,
                // identical grammar to `(...)` call args (`xs[i, j]`,
                // `cache[key, default: fallback]`), not a single expression.
                // Reuses `parse_arg_list` verbatim, which already
                // short-circuits on an immediately-closing delimiter
                // (`xs[]`, zero-arity).
                let selector_start = self.tokens[self.pos.saturating_sub(1)].start;
                let args = self.parse_arg_list()?;
                self.expect(&Token::RBracket, &["\"]\""])?;
                let selector_range = (selector_start..self.prev_end).into();
                let range = (start..self.prev_end).into();
                expr = Expr::Index(Box::new(IndexExpr {
                    object: expr,
                    args,
                    selector_range: Some(selector_range),
                    range,
                }));
                trailing_target = TrailingTarget::None;
            }
        }
        Ok(expr)
    }

    fn parse_associated_suffix(&mut self, receiver: Expr, start: usize) -> ParserResult<Expr> {
        let first_separator_range: SourceRange = (self.tokens[self.pos.saturating_sub(1)].start..self.prev_end).into();

        if matches!(self.peek(), Token::Hash) {
            let h_start = self.cur_start();
            self.advance();
            return Err(SyntaxError {
                kind: SyntaxErrorKind::Message("associated member syntax does not use `#` after `::`".to_string()),
                range: h_start..self.prev_end,
            });
        }

        if matches!(self.peek(), Token::DotDotDot) {
            let dot_start = self.cur_start();
            self.advance();
            return Err(SyntaxError {
                kind: SyntaxErrorKind::AssociatedLegacyFamilyEllipsis,
                range: dot_start..self.prev_end,
            });
        }

        if matches!(self.peek(), Token::LBracket) {
            let bracket_start = self.cur_start();
            self.advance(); // consume '['
            let (prefix, suffix, gap_range, mut end) = self.parse_selector_spec_slots(Token::RBracket)?;
            if gap_range.is_some() {
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::AssociatedLegacyFamilyEllipsis,
                    range: bracket_start..end,
                });
            }
            let setter = if self.eat(&Token::Equal) {
                self.expect(&Token::LParen, &["\"(put)\""])?;
                let put_start = self.cur_start();
                let put = self.expect_identifier(&["\"put\""])?;
                if put != "put" {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message("setter parameter must start with \"put\"".to_string()),
                        range: put_start..self.prev_end,
                    });
                }
                self.expect(&Token::RParen, &["\")\""])?;
                end = self.prev_end;
                true
            } else {
                false
            };
            let base_range = (bracket_start..bracket_start + 1).into();
            let range = (bracket_start..end).into();
            let mut slots = prefix;
            slots.extend(suffix);
            let exact = ExactSelectorSyntax {
                base: String::new(),
                kind: if setter {
                    phalcom_common::selector::SelectorKind::SubscriptSet
                } else {
                    phalcom_common::selector::SelectorKind::SubscriptGet
                },
                slots,
                is_subscript: true,
                base_range,
                range,
            };
            let whole_range = (start..end).into();
            return Ok(Expr::AssociatedLookup(Box::new(AssociatedLookupExpr {
                receiver,
                first_separator_range,
                member: AssociatedMemberSyntax::Subscript(exact),
                range: whole_range,
            })));
        }

        if matches!(
            self.peek(),
            Token::Plus
                | Token::Minus
                | Token::Asterisk
                | Token::DoubleAsterisk
                | Token::Power
                | Token::TripleAsterisk
                | Token::Slash
                | Token::SlashTilde
                | Token::Percent
                | Token::ShiftLeft
                | Token::ShiftRight
                | Token::Ampersand
                | Token::Pipe
                | Token::Caret
                | Token::Tilde
                | Token::EqualEqual
                | Token::TripleEqual
                | Token::BangEqual
                | Token::Less
                | Token::LessEqual
                | Token::Greater
                | Token::GreaterEqual
                | Token::Spaceship
        ) {
            let base_start = self.cur_start();
            let op_name = self.parse_method_name()?;
            let base_range = (base_start..self.prev_end).into();
            let slots = if self.eat(&Token::LParen) {
                let (prefix, suffix, gap_range, _end) = self.parse_selector_spec_slots(Token::RParen)?;
                if gap_range.is_some() {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::AssociatedLegacyFamilyEllipsis,
                        range: base_start..self.prev_end,
                    });
                }
                let mut slots = prefix;
                slots.extend(suffix);
                slots
            } else if matches!(op_name.as_str(), "not" | "~") {
                Vec::new()
            } else {
                vec![SelectorSlotSyntax {
                    slot: SelectorSlot::Positional,
                    range: base_range,
                }]
            };
            let range = (base_start..self.prev_end).into();
            let exact = ExactSelectorSyntax {
                base: op_name,
                kind: SelectorKind::Method,
                slots,
                is_subscript: false,
                base_range,
                range,
            };
            let whole_range = (start..self.prev_end).into();
            return Ok(Expr::AssociatedLookup(Box::new(AssociatedLookupExpr {
                receiver,
                first_separator_range,
                member: AssociatedMemberSyntax::Operator(exact),
                range: whole_range,
            })));
        }

        let base_start = self.cur_start();
        let base = self.parse_property_name()?;
        let base_range = (base_start..self.prev_end).into();

        if matches!(self.peek(), Token::LParen) {
            let next_tok = self.peek_next();
            if matches!(next_tok, Token::Underscore) {
                let err_start = self.cur_start();
                self.advance(); // consume '('
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::AssociatedExactShapeRequiresSecondSeparator,
                    range: err_start..self.cur_start() + 1,
                });
            }
            if matches!(next_tok, Token::DotDotDot) {
                let err_start = self.cur_start();
                self.advance(); // consume '('
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::AssociatedLegacyFamilyEllipsis,
                    range: err_start..self.cur_start() + 3,
                });
            }

            self.advance(); // consume '('
            let args = self.parse_arg_list()?;
            self.expect(&Token::RParen, &["\")\""])?;
            let range = (start..self.prev_end).into();
            return Ok(Expr::AssociatedInvoke(Box::new(AssociatedInvokeExpr {
                receiver,
                first_separator_range,
                base,
                base_range,
                args,
                range,
            })));
        }

        if self.eat(&Token::ColonColon) {
            let second_separator_range: SourceRange = (self.tokens[self.pos.saturating_sub(1)].start..self.prev_end).into();
            if self.eat(&Token::Asterisk) {
                let star_range: SourceRange = (self.tokens[self.pos.saturating_sub(1)].start..self.prev_end).into();
                let member_range = (base_start..self.prev_end).into();
                let whole_range = (start..self.prev_end).into();
                return Ok(Expr::AssociatedLookup(Box::new(AssociatedLookupExpr {
                    receiver,
                    first_separator_range,
                    member: AssociatedMemberSyntax::Named(AssociatedNamedMemberSyntax {
                        base,
                        base_range,
                        mode: AssociatedNamedMode::Family {
                            second_separator_range,
                            star_range,
                        },
                        range: member_range,
                    }),
                    range: whole_range,
                })));
            }

            if self.eat(&Token::LParen) {
                let res_start = self.tokens[self.pos.saturating_sub(1)].start;
                let (prefix, suffix, gap_range, end) = self.parse_selector_spec_slots(Token::RParen)?;
                if gap_range.is_some() {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::AssociatedLegacyFamilyEllipsis,
                        range: res_start..end,
                    });
                }
                let mut slots = prefix;
                slots.extend(suffix);
                let res_range = (res_start..end).into();
                let member_range = (base_start..end).into();
                let whole_range = (start..end).into();
                return Ok(Expr::AssociatedLookup(Box::new(AssociatedLookupExpr {
                    receiver,
                    first_separator_range,
                    member: AssociatedMemberSyntax::Named(AssociatedNamedMemberSyntax {
                        base,
                        base_range,
                        mode: AssociatedNamedMode::Exact {
                            second_separator_range,
                            residual: AssociatedResidualSelectorSyntax::Method { slots, range: res_range },
                        },
                        range: member_range,
                    }),
                    range: whole_range,
                })));
            }

            if self.eat(&Token::Equal) {
                let res_start = self.tokens[self.pos.saturating_sub(1)].start;
                self.expect(&Token::LParen, &["\"(put)\""])?;
                let put_start = self.cur_start();
                let put = self.expect_identifier(&["\"put\""])?;
                if put != "put" {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message("setter parameter must start with \"put\"".to_string()),
                        range: put_start..self.prev_end,
                    });
                }
                self.expect(&Token::RParen, &["\")\""])?;
                let put_range = (put_start..put_start + 3).into();
                let res_range = (res_start..self.prev_end).into();
                let member_range = (base_start..self.prev_end).into();
                let whole_range = (start..self.prev_end).into();
                return Ok(Expr::AssociatedLookup(Box::new(AssociatedLookupExpr {
                    receiver,
                    first_separator_range,
                    member: AssociatedMemberSyntax::Named(AssociatedNamedMemberSyntax {
                        base,
                        base_range,
                        mode: AssociatedNamedMode::Exact {
                            second_separator_range,
                            residual: AssociatedResidualSelectorSyntax::Setter { put_range, range: res_range },
                        },
                        range: member_range,
                    }),
                    range: whole_range,
                })));
            }

            // Explicit getter separator: `owner::name::`
            let member_range = (base_start..self.prev_end).into();
            let whole_range = (start..self.prev_end).into();
            return Ok(Expr::AssociatedLookup(Box::new(AssociatedLookupExpr {
                receiver,
                first_separator_range,
                member: AssociatedMemberSyntax::Named(AssociatedNamedMemberSyntax {
                    base,
                    base_range,
                    mode: AssociatedNamedMode::Getter {
                        explicit_separator_range: Some(second_separator_range),
                    },
                    range: member_range,
                }),
                range: whole_range,
            })));
        }

        if matches!(self.peek(), Token::DotDotDot) {
            let dot_start = self.cur_start();
            self.advance();
            return Err(SyntaxError {
                kind: SyntaxErrorKind::AssociatedLegacyFamilyEllipsis,
                range: dot_start..self.prev_end,
            });
        }

        // Implicit getter: `owner::name`
        let member_range = (base_start..self.prev_end).into();
        let whole_range = (start..self.prev_end).into();
        Ok(Expr::AssociatedLookup(Box::new(AssociatedLookupExpr {
            receiver,
            first_separator_range,
            member: AssociatedMemberSyntax::Named(AssociatedNamedMemberSyntax {
                base,
                base_range,
                mode: AssociatedNamedMode::Getter {
                    explicit_separator_range: None,
                },
                range: member_range,
            }),
            range: whole_range,
        })))
    }

    fn skip_newlines_at(&self, mut pos: usize) -> usize {
        while matches!(self.tokens.get(pos).map(|lexeme| &lexeme.token), Some(Token::Newline)) {
            pos += 1;
        }
        pos
    }

    /// Non-consuming recognizer for exactly the closure grammar accepted by
    /// `parse_closure_literal(Braced)`.
    fn starts_braced_closure_literal(&self, pos: usize) -> bool {
        let mut pos = pos;
        if !matches!(self.tokens.get(pos).map(|lexeme| &lexeme.token), Some(Token::Pipe)) {
            return false;
        }
        pos = self.skip_newlines_at(pos + 1);
        if matches!(self.tokens.get(pos).map(|lexeme| &lexeme.token), Some(Token::Pipe)) {
            return matches!(self.tokens.get(self.skip_newlines_at(pos + 1)).map(|lexeme| &lexeme.token), Some(Token::LBrace));
        }
        loop {
            if matches!(self.tokens.get(pos).map(|lexeme| &lexeme.token), Some(Token::Asterisk)) {
                pos = self.skip_newlines_at(pos + 1);
                if !matches!(self.tokens.get(pos).map(|lexeme| &lexeme.token), Some(Token::Identifier(_) | Token::Underscore)) {
                    return false;
                }
                pos = self.skip_newlines_at(pos + 1);
                return matches!(self.tokens.get(pos).map(|lexeme| &lexeme.token), Some(Token::Pipe))
                    && matches!(self.tokens.get(self.skip_newlines_at(pos + 1)).map(|lexeme| &lexeme.token), Some(Token::LBrace));
            }
            if !matches!(self.tokens.get(pos).map(|lexeme| &lexeme.token), Some(Token::Identifier(_) | Token::Underscore)) {
                return false;
            }
            pos = self.skip_newlines_at(pos + 1);
            match self.tokens.get(pos).map(|lexeme| &lexeme.token) {
                Some(Token::Pipe) => return matches!(self.tokens.get(self.skip_newlines_at(pos + 1)).map(|lexeme| &lexeme.token), Some(Token::LBrace)),
                Some(Token::Comma) => pos = self.skip_newlines_at(pos + 1),
                _ => return false,
            }
        }
    }

    fn starts_labelled_braced_closure_literal(&self, pos: usize) -> bool {
        self.tokens.get(pos).is_some_and(|lexeme| Self::label_name(&lexeme.token).is_some())
            && matches!(self.tokens.get(pos + 1).map(|lexeme| &lexeme.token), Some(Token::Colon))
            && (self.starts_braced_closure_literal(pos + 2)
                // A bare brace is contextual zero-argument trailing-closure
                // sugar, valid here because the label already establishes a
                // member-send argument position.
                || matches!(self.tokens.get(pos + 2).map(|lexeme| &lexeme.token), Some(Token::LBrace)))
    }

    /// Parses the closure-only, unparenthesized arguments attached to an
    /// explicit member send. A positional clause may be followed by labeled
    /// clauses; subsequent positional clauses are deliberately unsupported.
    fn parse_trailing_closure_arguments(&mut self) -> ParserResult<Option<(Vec<PackItem>, usize)>> {
        let mut args = Vec::new();
        let positional = self.starts_braced_closure_literal(self.pos);
        let labelled = self.starts_labelled_braced_closure_literal(self.pos);
        if !positional && !labelled {
            return Ok(None);
        }

        let mut parse_one = |parser: &mut Self, labelled: bool| -> ParserResult<()> {
            let start = parser.cur_start();
            let label = if labelled {
                let label = Self::label_name(parser.peek()).expect("validated trailing label").to_string();
                let label_start = parser.cur_start();
                parser.advance();
                parser.expect(&Token::Colon, &["\":\""])?;
                Some(PackLabel::Static {
                    text: label,
                    range: (label_start..parser.prev_end).into(),
                })
            } else {
                None
            };
            let expr = if matches!(parser.peek(), Token::LBrace) {
                parser.parse_brace_block()?
            } else {
                parser.parse_closure_literal(ClosureBodyRequirement::Braced)?
            };
            let range = (start..parser.prev_end).into();
            args.push(match label {
                Some(label) => PackItem::Labeled { label, value: expr, range },
                None => PackItem::Positional { expr, range },
            });
            Ok(())
        };

        parse_one(self, labelled)?;
        while self.eat(&Token::Comma) {
            self.skip_newlines();
            if !self.starts_labelled_braced_closure_literal(self.pos) {
                return Err(self.error_here(strs(&["labeled trailing closure after `,`"])));
            }
            parse_one(self, true)?;
        }
        Ok(Some((args, self.prev_end)))
    }

    fn attach_trailing_arguments(&self, expr: Expr, args: Vec<PackItem>, end: usize) -> ParserResult<Expr> {
        match expr {
            Expr::GetProperty(get) => Ok(Expr::MethodCall(Box::new(MethodCallExpr {
                object: get.object,
                method: get.property,
                method_range: get.property_range,
                args,
                range: (get.range.start..end).into(),
            }))),
            Expr::MethodCall(mut call) => {
                call.args.extend(args);
                call.range.end = end;
                Ok(Expr::MethodCall(call))
            }
            _ => Err(SyntaxError {
                kind: SyntaxErrorKind::Message("internal parser error: trailing closure target is not a member send".to_string()),
                range: end..end,
            }),
        }
    }

    /// Desugars a `?.` optional send over `object`, given the enclosing
    /// expression's `start` offset.
    ///
    /// Per ADR-0007 / values-and-absence §3.4, `opt?.foo` desugars to
    /// `opt.map { x => x.foo }` and `opt?.bar(a)` to `opt.map { x => x.bar(a) }`.
    /// The mapper block's parameter is a synthetic, non-user-writable receiver
    /// name, so the inner send can never capture a source variable. Chained
    /// `?.` left-associates through [`Parser::parse_call`], so the first `None`
    /// short-circuits the rest.
    ///
    /// # Errors
    ///
    /// Propagates any error from the property name or argument list.
    fn parse_optional_send(&mut self, object: Expr, start: usize) -> ParserResult<Expr> {
        // A synthetic receiver name. The leading space is not lexable as an
        // identifier, so it cannot collide with any user-written variable.
        let recv_name = " recv".to_string();
        let property = self.parse_property_name()?;
        let inner = if self.eat(&Token::LParen) {
            let args = self.parse_arg_list()?;
            self.expect(&Token::RParen, &["\")\""])?;
            let range = (start..self.prev_end).into();
            Expr::MethodCall(Box::new(MethodCallExpr {
                object: Expr::Var {
                    value: recv_name.clone(),
                    range,
                },
                method: property,
                method_range: None,
                args,
                range,
            }))
        } else {
            let range = (start..self.prev_end).into();
            Expr::GetProperty(Box::new(GetPropertyExpr {
                object: Expr::Var {
                    value: recv_name.clone(),
                    range,
                },
                property,
                property_range: None,
                range,
            }))
        };
        let range = (start..self.prev_end).into();
        let mapper = self.wrap_block_mapper(recv_name, inner, range);
        Ok(Expr::MethodCall(Box::new(MethodCallExpr {
            object,
            method: "map".to_string(),
            method_range: None,
            args: vec![PackItem::Positional { expr: mapper, range }],
            range,
        })))
    }

    /// Parses a property name after `.`: an identifier, the `class` keyword,
    /// or an overloadable operator token.
    ///
    /// The operator arms mirror [`Parser::parse_method_name`]'s set exactly
    /// (U-ERR-FIX SUPER-OP-SYNTAX) so a `super.<operator>(...)` send parses
    /// wherever an ordinary `.<operator>(...)` send already does — e.g.
    /// `super.+(other)` super-calling an overridden `+` — closing the gap
    /// where operator methods were overridable but not super-callable.
    ///
    /// # Errors
    ///
    /// Returns an error if the token following `.` is none of the above.
    fn parse_property_name(&mut self) -> ParserResult<String> {
        let name = match self.peek().clone() {
            Token::Identifier(name)
            | Token::FieldIdentifier(name)
            | Token::ImplementationFieldIdentifier(name)
            | Token::ImplementationSelectorIdentifier(name) => name,
            Token::Class => "class".to_string(),
            Token::Enum => "enum".to_string(),
            // `try` is a genuine reserved keyword (statement-leading, ADR-0031
            // §4) but must still resolve as an ordinary selector in message
            // position — `fiber.try(...)`/`fiber.try` (`Fiber#try`, ADR-0030)
            // predates this unit and must keep parsing.
            Token::Try => "try".to_string(),
            // `from` is reserved in module preambles but remains valid in
            // message-send position (`Map.from(...)`).
            Token::From => "from".to_string(),
            Token::Match => "match".to_string(),
            Token::Plus => "+".to_string(),
            Token::Minus => "-".to_string(),
            Token::Asterisk => "*".to_string(),
            Token::Power | Token::DoubleAsterisk => "**".to_string(),
            Token::TripleAsterisk => "***".to_string(),
            Token::Slash => "/".to_string(),
            Token::SlashTilde => "~/".to_string(),
            Token::Percent => "%".to_string(),
            Token::ShiftLeft => "<<".to_string(),
            Token::ShiftRight => ">>".to_string(),
            Token::Ampersand => "&".to_string(),
            Token::Pipe => "|".to_string(),
            Token::Caret => "^".to_string(),
            Token::Tilde => "~".to_string(),
            Token::EqualEqual => "==".to_string(),
            Token::TripleEqual => "===".to_string(),
            Token::BangEqual => "!=".to_string(),
            Token::Less => "<".to_string(),
            Token::LessEqual => "<=".to_string(),
            Token::Greater => ">".to_string(),
            Token::GreaterEqual => ">=".to_string(),
            Token::Spaceship => "<=>".to_string(),
            Token::And => "and".to_string(),
            Token::Or => "or".to_string(),
            Token::Not => "not".to_string(),
            Token::Is => "is".to_string(),
            _ => return Err(self.error_here(strs(&["identifier", "\"class\"", "operator"]))),
        };
        self.advance();
        Ok(self.extend_selector_name(name))
    }

    /// Extends a selector base with syntax that is lexed as separate punctuation
    /// while preserving adjacency. This keeps `try!` and `*args` ordinary names
    /// without making those spellings expression-level identifiers.
    fn extend_selector_name(&mut self, mut name: String) -> String {
        if let Some(next) = self.tokens.get(self.pos)
            && next.start == self.prev_end
            && matches!(next.token, Token::Bang)
        {
            self.advance();
            name.push('!');
        }
        if matches!(name.as_str(), "*" | "**" | "***")
            && let Some(next) = self.tokens.get(self.pos)
            && next.start == self.prev_end
            && let Token::Identifier(suffix) = &next.token
        {
            name.push_str(suffix);
            self.advance();
        }
        name
    }

    /// Parses a `{ statements }` body into a 0-parameter, statement-bodied
    /// [`BlockExpr`] spanning `start..prev_end` after the closing brace.
    ///
    /// Shared by [`Parser::parse_if`] and [`Parser::parse_while`]: `if`/
    /// `while` are keyword sugar over sends of literal blocks
    /// (control-flow.md §1; U5-plan.md BD-U5-1 Option A), so every arm they
    /// parse becomes exactly the same [`Expr::Block`] node a bare `{ }`
    /// literal would produce.
    ///
    /// # Errors
    ///
    /// Returns an error if the braces or the enclosed statements are
    /// malformed.
    fn parse_brace_block(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        self.expect(&Token::LBrace, &["\"{\""])?;
        let body = self.parse_block_statements()?;
        self.expect(&Token::RBrace, &["\"}\""])?;
        let range = (start..self.prev_end).into();
        Ok(Expr::Block(Box::new(BlockExpr {
            params: ClosureParameters::default(),
            body,
            expr_body: false,
            range,
        })))
    }

    /// Wraps `expr` in a synthetic 0-parameter block whose single statement
    /// is `expr`, spanning `expr`'s own range. Used to give a nested `if`
    /// expression (the `else if` chain) the same block shape as an ordinary
    /// `else { ... }` arm before it is passed as a sacred-selector block
    /// argument.
    fn wrap_expr_as_block(expr: Expr) -> Expr {
        let range = expr.range();
        Expr::Block(Box::new(BlockExpr {
            params: ClosureParameters::default(),
            body: vec![Statement::Expr { expr, range }],
            expr_body: true,
            range,
        }))
    }

    /// Parses `if condition { ... } (else (if ... | { ... }))?` as sacred-selector
    /// message sends over block literals (control-flow.md §1, §3;
    /// U5-plan.md §4.1, BD-U5-1 Option A): a bare `if` desugars to
    /// `cond.ifTrue(_:)`; an `if`/`else` desugars to the paired
    /// `cond.ifTrue(_:ifFalse:)` selector — **not** the spec's illustrative
    /// `ifTrue{}.ifNone{}` `Option` chain, keeping U5 independent of U6's
    /// `Option` (see `Universe::install_primitives`' `bool_if_true_if_false`
    /// registration for why: Phalcom's selector model has no Smalltalk-style
    /// independently-worded keyword pair, so the paired arm is spelled with
    /// one positional block plus one `ifFalse:`-labeled block instead).
    /// `else if` chains by recursing and wrapping the nested `if` expression
    /// in a synthetic block ([`Self::wrap_expr_as_block`]).
    ///
    /// # Errors
    ///
    /// Returns an error if the condition, parentheses, or either arm's body
    /// is malformed.
    fn parse_if(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        self.advance(); // 'if'
        if self.eat(&Token::Let) {
            let pattern = self.parse_pattern()?;
            self.expect(&Token::Equal, &["\"=\""])?;
            let value = self.parse_expr_without_trailing_closures()?;
            let then_body = match self.parse_brace_block()? {
                Expr::Block(block) => *block,
                _ => unreachable!("parse_brace_block must produce a block"),
            };
            let else_body = if self.eat(&Token::Else) {
                Some(match self.parse_brace_block()? {
                    Expr::Block(block) => *block,
                    _ => unreachable!("parse_brace_block must produce a block"),
                })
            } else {
                None
            };
            let range = (start..self.prev_end).into();
            return Ok(Expr::IfLet(Box::new(IfLetExpr {
                pattern,
                value,
                then_body,
                else_body,
                range,
            })));
        }
        let cond = self.parse_expr_without_trailing_closures()?;
        let then_arm = self.parse_brace_block()?;
        let then_range = then_arm.range();

        let mut args = vec![PackItem::Positional {
            expr: then_arm,
            range: then_range,
        }];

        if self.eat(&Token::Else) {
            let else_arm = if matches!(self.peek(), Token::If) {
                Self::wrap_expr_as_block(self.parse_if()?)
            } else {
                self.parse_brace_block()?
            };
            let else_range = else_arm.range();
            args.push(PackItem::Labeled {
                label: PackLabel::Static {
                    text: "ifFalse".to_string(),
                    range: else_range,
                },
                value: else_arm,
                range: else_range,
            });
        }

        let range = (start..self.prev_end).into();
        Ok(Expr::MethodCall(Box::new(MethodCallExpr {
            object: cond,
            method: "ifTrue".to_string(),
            method_range: None,
            args,
            range,
        })))
    }

    /// Parses `while condition { body }` as the sacred loop send
    /// `{ cond }.whileTrue { body }` (control-flow.md §1, §3; U5-plan.md
    /// §4.1, BD-U5-1 Option A) — the receiver is itself a literal block
    /// wrapping the condition, re-evaluated each iteration.
    ///
    /// # Errors
    ///
    /// Returns an error if the condition, parentheses, or body is malformed.
    fn parse_while(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        self.advance(); // 'while'
        if self.eat(&Token::Let) {
            let pattern = self.parse_pattern()?;
            self.expect(&Token::Equal, &["\"=\""])?;
            let value = self.parse_expr_without_trailing_closures()?;
            let body = match self.parse_brace_block()? {
                Expr::Block(block) => block.body,
                _ => unreachable!("parse_brace_block must produce a block"),
            };
            let range = (start..self.prev_end).into();
            return Ok(Expr::WhileLet(Box::new(WhileLetExpr { pattern, value, body, range })));
        }
        let cond = self.parse_expr_without_trailing_closures()?;
        let cond_block = Self::wrap_expr_as_block(cond);
        let body = self.parse_brace_block()?;
        let body_range = body.range();
        let range = (start..self.prev_end).into();
        Ok(Expr::MethodCall(Box::new(MethodCallExpr {
            object: cond_block,
            method: "whileTrue".to_string(),
            method_range: None,
            args: vec![PackItem::Positional { expr: body, range: body_range }],
            range,
        })))
    }

    /// Parses a `match value { (pattern => branch)* }` pattern elimination expression.
    fn parse_match_expression(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        self.advance(); // 'match'
        let value = Box::new(self.parse_expr_without_trailing_closures()?);
        self.expect(&Token::LBrace, &["\"{\""])?;
        self.skip_newlines();
        let mut arms = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::Eof) {
            let arm_start = self.cur_start();
            let pattern = self.parse_pattern()?;
            let arrow_start = self.cur_start();
            self.expect(&Token::FatArrow, &["\"=>\""])?;
            let arrow_range = (arrow_start..self.prev_end).into();
            self.skip_newlines();
            let branch = Box::new(self.parse_match_arm_branch()?);
            let arm_range = (arm_start..self.prev_end).into();
            arms.push(MatchArm {
                pattern,
                branch,
                arrow_range,
                range: arm_range,
            });
            self.skip_newlines();
            self.eat(&Token::Semicolon);
            self.skip_newlines();
        }
        self.expect(&Token::RBrace, &["\"}\""])?;
        let range = (start..self.prev_end).into();
        Ok(Expr::Match(MatchExpr {
            value,
            arms,
            range,
        }))
    }

    fn parse_match_arm_branch(&mut self) -> ParserResult<Expr> {
        if matches!(self.peek(), Token::LBrace) {
            let next_is_map = (matches!(self.peek_next(), Token::Identifier(_) | Token::String(_) | Token::QuotedSymbol(_))
                && self.tokens.get(self.pos + 2).is_some_and(|t| matches!(t.token, Token::Colon)))
                || matches!(self.peek_next(), Token::LBracket | Token::Asterisk | Token::DoubleAsterisk | Token::TripleAsterisk | Token::Power);
            if !next_is_map {
                return self.parse_brace_block();
            }
        }
        self.parse_expr()
    }

    fn is_match_expression(&self) -> bool {
        if self.peek() != &Token::Match {
            return false;
        }
        if self.peek_next() != &Token::LParen {
            return true;
        }
        let mut depth = 0;
        let mut i = self.pos + 1;
        while i < self.tokens.len() {
            match &self.tokens[i].token {
                Token::LParen => depth += 1,
                Token::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        let mut next = i + 1;
                        while next < self.tokens.len() && matches!(self.tokens[next].token, Token::Newline) {
                            next += 1;
                        }
                        if next < self.tokens.len() && self.tokens[next].token == Token::LBrace {
                            return true;
                        }
                        return false;
                    }
                }
                Token::Colon if depth == 1 => return false,
                Token::Eof => return false,
                _ => {}
            }
            i += 1;
        }
        false
    }

    /// Parses a primary expression: a literal, variable/field, `self`/`super`,
    /// or a parenthesised expression.
    ///
    /// A parenthesised expression yields its inner expression unchanged (the
    /// parentheses do not widen the span), matching the LALRPOP grammar.
    ///
    /// # Errors
    ///
    /// Returns an [`SyntaxErrorKind::UnrecognizedToken`] listing the expression
    /// starters if the current token cannot begin a primary expression.
    fn parse_primary(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        let end = self.tokens[self.pos].end;
        let range = (start..end).into();
        match self.peek().clone() {
            Token::Match => {
                if self.is_match_expression() {
                    self.parse_match_expression()
                } else {
                    self.advance();
                    Ok(Expr::Var {
                        value: "match".to_string(),
                        range,
                    })
                }
            }
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::True => {
                self.advance();
                Ok(Expr::Boolean { value: true, range })
            }
            Token::False => {
                self.advance();
                Ok(Expr::Boolean { value: false, range })
            }
            Token::DotDotDot => {
                self.advance();
                Ok(Expr::Ellipsis { range })
            }
            Token::Int { digits, radix } => {
                self.advance();
                Ok(Expr::Int { digits, radix, range })
            }
            Token::Float(value) => {
                self.advance();
                Ok(Expr::Float { value, range })
            }
            Token::String(value) => {
                self.advance();
                Ok(Expr::String { value, range })
            }
            Token::StringInterp(segments) => {
                self.advance();
                self.desugar_string_interp(segments, range)
            }
            Token::Hash => {
                let spec = self.parse_selector_spec_after_hash()?;
                let kind = match spec {
                    SelectorSpecSyntax::Exact(exact) => exact_symbol_kind(exact),
                    SelectorSpecSyntax::Pattern(pattern) => SymbolLiteralKind::Pattern(SelectorPatternSyntax { ..pattern }),
                };
                Ok(Expr::Symbol(Box::new(SymbolExpr {
                    kind,
                    range: (start..self.prev_end).into(),
                })))
            }
            Token::QuotedSymbol(value) => {
                self.advance();
                Ok(Expr::Symbol(Box::new(SymbolExpr {
                    kind: SymbolLiteralKind::Name(value),
                    range,
                })))
            }
            Token::FieldIdentifier(value) => {
                self.advance();
                Ok(Expr::Field {
                    value,
                    kind: FieldKind::Source,
                    range,
                })
            }
            Token::ImplementationFieldIdentifier(value) => {
                self.advance();
                Ok(Expr::Field {
                    value,
                    kind: FieldKind::Implementation,
                    range,
                })
            }
            Token::Underscore => {
                self.advance();
                Ok(Expr::Var { value: "_".to_string(), range })
            }
            Token::Identifier(value) => {
                self.advance();
                Ok(Expr::Var { value, range })
            }
            Token::ImplementationSelectorIdentifier(value) => {
                self.advance();
                Ok(Expr::ImplementationSelector { value, range })
            }
            Token::SelfKw => {
                self.advance();
                Ok(Expr::SelfVar { range })
            }
            Token::Class => {
                self.advance();
                if self.in_class_body {
                    Ok(Expr::GetProperty(Box::new(GetPropertyExpr {
                        object: Expr::SelfVar { range },
                        property: "class".to_string(),
                        property_range: None,
                        range,
                    })))
                } else {
                    Ok(Expr::Var {
                        value: "class".to_string(),
                        range,
                    })
                }
            }
            Token::Super => {
                self.advance();
                Ok(Expr::SuperVar { range })
            }
            Token::LBracket => self.parse_list_literal(),
            Token::LParen => self.parse_paren_or_tuple(),
            Token::RecordLBrace => self.parse_record_literal(),
            Token::Pipe => self.parse_closure_literal(ClosureBodyRequirement::Any),
            Token::LBrace => {
                let start = self.cur_start();
                self.advance(); // '{'

                // B.3a keeps the existing block grammar intact. Association
                // forms are unambiguous; Set and empty-Map classification stay
                // behind B.3b's brace-grammar decision.
                if (matches!(self.peek(), Token::Identifier(_)) && matches!(self.peek_next(), Token::Colon))
                    || self.starts_computed_map_literal()
                    || matches!(self.peek(), Token::Asterisk | Token::DoubleAsterisk | Token::TripleAsterisk | Token::Power)
                {
                    return self.parse_map_literal(start);
                }

                let message = "bare brace block literals were removed; write `|| { ... }` for a closure";
                Err(SyntaxError {
                    kind: SyntaxErrorKind::Message(message.to_string()),
                    range: start..self.prev_end,
                })
            }
            Token::Less if self.is_type_lambda_ahead() => {
                let lambda = self.parse_type_lambda()?;
                Ok(Expr::TypeForm(Box::new(lambda)))
            }
            _ => Err(self.error_here(primary_expected())),
        }
    }

    /// Converts an expression representing a type origin into a TypeAnnotation if eligible.
    fn expr_to_type_annotation(expr: &Expr) -> Option<TypeAnnotation> {
        match expr {
            Expr::Var { value, range } => Some(TypeAnnotation {
                expr: TypeAnnotationExpr::Reference(StaticSymbolRef {
                    root: value.clone(),
                    root_range: *range,
                    members: Vec::new(),
                    range: *range,
                }),
                range: *range,
            }),
            Expr::GetProperty(gp) => {
                let parent_ann = Self::expr_to_type_annotation(&gp.object)?;
                if let TypeAnnotationExpr::Reference(mut sym) = parent_ann.expr {
                    sym.members.push(PathSegment {
                        name: gp.property.clone(),
                        range: gp.property_range.unwrap_or(gp.range),
                    });
                    let range = (parent_ann.range.start..gp.range.end).into();
                    sym.range = range;
                    Some(TypeAnnotation {
                        expr: TypeAnnotationExpr::Reference(sym),
                        range,
                    })
                } else {
                    None
                }
            }
            Expr::TypeForm(ann) => Some((**ann).clone()),
            _ => None,
        }
    }

    /// True only for `{ [key]: value }`. `{ [value] }` remains a Block until
    /// B.3b ratifies its Set-literal spelling.
    fn starts_computed_map_literal(&self) -> bool {
        if !matches!(self.peek(), Token::LBracket) {
            return false;
        }

        let mut depth = 0usize;
        for idx in self.pos..self.tokens.len() {
            match self.tokens[idx].token {
                Token::LBracket => depth += 1,
                Token::RBracket => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return self.tokens.get(idx + 1).is_some_and(|token| matches!(&token.token, Token::Colon));
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Parses a List literal into its explicit AST representation and preserves
    /// positional `*` expansion entries for compiler lowering. Ordinary Lists
    /// compile directly through `BuildList`; spread-containing Lists use the
    /// compiler-owned incremental construction path. No public mutation-chain
    /// desugaring is involved.
    ///
    /// # Errors
    ///
    /// Propagates any [`SyntaxError`] from an element expression, or from a
    /// missing closing `]`.
    fn parse_list_literal(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        self.advance(); // '['
        self.skip_newlines();
        let mut elements = Vec::new();
        if matches!(self.peek(), Token::RBracket) {
            self.advance();
            let range: SourceRange = (start..self.prev_end).into();
            return Ok(Expr::ListLiteral(Box::new(ListLiteralExpr { elements, range })));
        }

        loop {
            elements.push(self.parse_list_literal_element()?);
            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                break;
            }
            self.skip_newlines();
            if matches!(self.peek(), Token::RBracket) {
                break;
            }
        }
        self.expect(&Token::RBracket, &["\"]\""])?;
        let range: SourceRange = (start..self.prev_end).into();
        Ok(Expr::ListLiteral(Box::new(ListLiteralExpr { elements, range })))
    }

    fn parse_list_literal_element(&mut self) -> ParserResult<ListLiteralElement> {
        let start = self.cur_start();
        match self.peek() {
            Token::Asterisk => {
                self.advance();
                let expr = self.parse_expr()?;
                Ok(ListLiteralElement::Expansion {
                    expr,
                    range: (start..self.prev_end).into(),
                })
            }
            Token::DoubleAsterisk | Token::Power => {
                Err(self.error_message_here("`**` is not valid in a List literal; List has only a positional lane, use `*`"))
            }
            Token::TripleAsterisk => Err(self.error_message_here(
                "`***` is not valid in a List literal; complete expansion requires positional and labeled lanes, but List has only a positional lane",
            )),
            _ => {
                let expr = self.parse_expr()?;
                let range = expr.range();
                Ok(ListLiteralElement::Element { expr, range })
            }
        }
    }

    fn bare_product_label_name(token: &Token) -> Option<&'static str> {
        Some(match token {
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Asterisk => "*",
            Token::DoubleAsterisk | Token::Power => "**",
            Token::TripleAsterisk => "***",
            Token::Slash => "/",
            Token::SlashTilde => "~/",
            Token::Percent => "%",
            Token::ShiftLeft => "<<",
            Token::ShiftRight => ">>",
            Token::Ampersand => "&",
            Token::Pipe => "|",
            Token::Caret => "^",
            Token::Tilde => "~",
            Token::EqualEqual => "==",
            Token::TripleEqual => "===",
            Token::BangEqual => "!=",
            Token::Less => "<",
            Token::LessEqual => "<=",
            Token::Greater => ">",
            Token::GreaterEqual => ">=",
            Token::Spaceship => "<=>",
            Token::And => "and",
            Token::Or => "or",
            Token::Not => "not",
            Token::Is => "is",
            Token::Question => "?",
            _ => return None,
        })
    }

    fn looks_like_delimited_label(&self, open: &Token, close: &Token) -> bool {
        if std::mem::discriminant(self.peek()) != std::mem::discriminant(open) {
            return false;
        }
        let mut depth = 0usize;
        let mut idx = self.pos;
        while let Some(spanned) = self.tokens.get(idx) {
            let token = &spanned.token;
            if std::mem::discriminant(token) == std::mem::discriminant(open) {
                depth += 1;
            } else if std::mem::discriminant(token) == std::mem::discriminant(close) {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
                if depth == 0 {
                    return matches!(self.tokens.get(idx + 1).map(|t| &t.token), Some(Token::Colon));
                }
            } else if matches!(token, Token::LParen | Token::LBracket | Token::LBrace) {
                depth += 1;
            } else if matches!(token, Token::RParen | Token::RBracket | Token::RBrace) {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
            }
            idx += 1;
        }
        false
    }

    fn product_label_start(&self) -> Option<ProductLabelStart> {
        match self.peek() {
            Token::LBracket if self.looks_like_delimited_label(&Token::LBracket, &Token::RBracket) => Some(ProductLabelStart::Computed),
            Token::Hash if self.hash_symbol_has_label_colon() => Some(ProductLabelStart::ExplicitSymbol),
            Token::QuotedSymbol(_) if matches!(self.peek_next(), Token::Colon) => Some(ProductLabelStart::ExplicitSymbol),
            Token::Identifier(name) if matches!(self.peek_next(), Token::Colon) => Some(ProductLabelStart::BareName(name.clone())),
            Token::Identifier(name)
                if matches!(self.peek_next(), Token::Question) && matches!(self.tokens.get(self.pos + 2).map(|t| &t.token), Some(Token::Colon)) =>
            {
                Some(ProductLabelStart::BareName(format!("{name}?")))
            }
            Token::Identifier(name) if matches!(self.peek_next(), Token::LParen) && self.looks_like_delimited_label(&Token::LParen, &Token::RParen) => {
                Some(ProductLabelStart::BareSelector(name.clone()))
            }
            token if Self::label_name(token).is_some() && matches!(self.peek_next(), Token::Colon) => {
                Some(ProductLabelStart::BareName(Self::label_name(token).unwrap().to_string()))
            }
            token if Self::bare_product_label_name(token).is_some() && matches!(self.peek_next(), Token::Colon) => {
                Some(ProductLabelStart::BareName(Self::bare_product_label_name(token).unwrap().to_string()))
            }
            token
                if Self::bare_product_label_name(token).is_some()
                    && matches!(self.peek_next(), Token::LParen)
                    && self.looks_like_delimited_label(&Token::LParen, &Token::RParen) =>
            {
                Some(ProductLabelStart::BareSelector(Self::bare_product_label_name(token).unwrap().to_string()))
            }
            _ => None,
        }
    }

    fn parse_selector_label_slots(&mut self) -> ParserResult<Vec<Option<String>>> {
        let mut labels = Vec::new();
        let mut seen_label = false;
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RParen) {
                self.advance();
                break;
            }
            let positional = matches!(self.peek(), Token::Underscore);
            let slot = if positional {
                self.advance();
                "_".to_string()
            } else {
                self.expect_identifier(&["label slot"])?
            };
            if positional {
                if seen_label {
                    let range = self.tokens[self.pos.saturating_sub(1)].start..self.prev_end;
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::InvalidToken,
                        range,
                    });
                }
                labels.push(None);
            } else {
                seen_label = true;
                labels.push(Some(slot));
            }

            self.skip_newlines();
            match self.peek() {
                Token::Comma => {
                    self.advance();
                }
                Token::RParen => {
                    self.advance();
                    break;
                }
                _ => return Err(self.error_here(strs(&["\",\"", "\")\""]))),
            }
        }
        Ok(labels)
    }

    fn parse_hash_symbol(&mut self) -> ParserResult<SymbolLiteralKind> {
        match self.parse_selector_spec_after_hash()? {
            SelectorSpecSyntax::Exact(exact) => Ok(exact_symbol_kind(exact)),
            SelectorSpecSyntax::Pattern(pattern) => Ok(SymbolLiteralKind::Pattern(pattern)),
        }
    }

    fn parse_selector_spec_after_hash(&mut self) -> ParserResult<SelectorSpecSyntax> {
        self.expect(&Token::Hash, &["\"#\""])?;
        if self.prev_end != self.cur_start() {
            return Err(SyntaxError {
                kind: SyntaxErrorKind::InvalidToken,
                range: self.prev_end.saturating_sub(1)..self.prev_end,
            });
        }
        self.parse_selector_spec_body()
    }

    fn parse_selector_spec_body(&mut self) -> ParserResult<SelectorSpecSyntax> {
        let base_start = self.cur_start();

        // Bracket selectors have no named base. Keep them structural all the
        // way through normalization so `#[_]` and `#[...]` use the same common
        // selector model as ordinary index sends.
        if self.eat(&Token::LBracket) {
            let bracket_start = self.tokens[self.pos.saturating_sub(1)].start;
            let (prefix, suffix, gap_range, mut end) = self.parse_selector_spec_slots(Token::RBracket)?;
            let setter = if self.eat(&Token::Equal) {
                self.expect(&Token::LParen, &["\"(put)\""])?;
                let put = self.expect_identifier(&["\"put\""])?;
                if put != "put" {
                    return Err(self.error_here(strs(&["\"put\""])));
                }
                self.expect(&Token::RParen, &["\")\""])?;
                end = self.prev_end;
                true
            } else {
                false
            };
            let base_range = (bracket_start..bracket_start + 1).into();
            let range = (base_start..end).into();
            if let Some(gap_range) = gap_range {
                return Ok(SelectorSpecSyntax::Pattern(SelectorPatternSyntax {
                    base: String::new(),
                    kind: phalcom_common::selector::SelectorKindPattern::Exact(if setter {
                        phalcom_common::selector::SelectorKind::SubscriptSet
                    } else {
                        phalcom_common::selector::SelectorKind::SubscriptGet
                    }),
                    prefix,
                    suffix,
                    is_subscript: true,
                    gap_range,
                    base_range,
                    range,
                }));
            }
            let mut slots = prefix;
            slots.extend(suffix);
            return Ok(SelectorSpecSyntax::Exact(ExactSelectorSyntax {
                base: String::new(),
                kind: if setter {
                    phalcom_common::selector::SelectorKind::SubscriptSet
                } else {
                    phalcom_common::selector::SelectorKind::SubscriptGet
                },
                slots,
                is_subscript: true,
                base_range,
                range,
            }));
        }

        // Punctuation spellings are valid bare symbols even where the same
        // token has no expression-level meaning.
        if let Some(base) = self.parse_standalone_symbol_punctuation() {
            let range = (base_start..self.prev_end).into();
            let base_range = range;
            return Ok(SelectorSpecSyntax::Exact(ExactSelectorSyntax {
                base,
                kind: phalcom_common::selector::SelectorKind::Getter,
                slots: Vec::new(),
                is_subscript: false,
                base_range,
                range,
            }));
        }

        let base = self.parse_property_name()?;
        let base_range = (base_start..self.prev_end).into();

        if matches!(self.peek(), Token::LParen) {
            self.advance();
            let (prefix, suffix, gap_range, end) = self.parse_selector_spec_slots(Token::RParen)?;
            let range = (base_start..end).into();
            if let Some(gap_range) = gap_range {
                return Ok(SelectorSpecSyntax::Pattern(SelectorPatternSyntax {
                    base,
                    kind: phalcom_common::selector::SelectorKindPattern::Exact(phalcom_common::selector::SelectorKind::Method),
                    prefix,
                    suffix,
                    is_subscript: false,
                    gap_range,
                    base_range,
                    range,
                }));
            }
            let mut slots = prefix;
            slots.extend(suffix);
            return Ok(SelectorSpecSyntax::Exact(ExactSelectorSyntax {
                base,
                kind: phalcom_common::selector::SelectorKind::Method,
                slots,
                is_subscript: false,
                base_range,
                range,
            }));
        }

        if self.eat(&Token::Equal) {
            let marker_start = self.tokens[self.pos.saturating_sub(1)].start;
            if self.eat(&Token::DotDotDot) {
                let gap_range = (marker_start..self.prev_end).into();
                let range = (base_start..self.prev_end).into();
                return Ok(SelectorSpecSyntax::Pattern(SelectorPatternSyntax {
                    base,
                    kind: phalcom_common::selector::SelectorKindPattern::Exact(phalcom_common::selector::SelectorKind::Setter),
                    prefix: Vec::new(),
                    suffix: Vec::new(),
                    is_subscript: false,
                    gap_range,
                    base_range,
                    range,
                }));
            }
            self.expect(&Token::LParen, &["\"(put)\""])?;
            let put = self.expect_identifier(&["\"put\""])?;
            if put != "put" {
                return Err(self.error_here(strs(&["\"put\""])));
            }
            self.expect(&Token::RParen, &["\")\""])?;
            let range = (base_start..self.prev_end).into();
            return Ok(SelectorSpecSyntax::Exact(ExactSelectorSyntax {
                base,
                kind: phalcom_common::selector::SelectorKind::Setter,
                slots: Vec::new(),
                is_subscript: false,
                base_range,
                range,
            }));
        }

        if self.eat(&Token::DotDotDot) {
            let gap_range = (self.tokens[self.pos.saturating_sub(1)].start..self.prev_end).into();
            let range = (base_start..self.prev_end).into();
            return Ok(SelectorSpecSyntax::Pattern(SelectorPatternSyntax {
                base,
                kind: phalcom_common::selector::SelectorKindPattern::AnyNamed,
                prefix: Vec::new(),
                suffix: Vec::new(),
                is_subscript: false,
                gap_range,
                base_range,
                range,
            }));
        }

        let range = (base_start..self.prev_end).into();
        Ok(SelectorSpecSyntax::Exact(ExactSelectorSyntax {
            base,
            kind: phalcom_common::selector::SelectorKind::Getter,
            slots: Vec::new(),
            is_subscript: false,
            base_range,
            range,
        }))
    }

    fn parse_selector_spec_slots(&mut self, end: Token) -> ParserResult<SelectorSpecSlots> {
        let mut prefix = Vec::new();
        let mut suffix = Vec::new();
        let mut gap_range = None;
        let mut after_gap = false;
        let mut seen_label = false;
        loop {
            self.skip_newlines();
            if self.eat(&end) {
                return Ok((prefix, suffix, gap_range, self.prev_end));
            }
            let slot_start = self.cur_start();
            if self.eat(&Token::DotDotDot) {
                if gap_range.is_some() {
                    return Err(self.error_here(strs(&["one selector gap"])));
                }
                gap_range = Some((slot_start..self.prev_end).into());
                after_gap = true;
            } else {
                let slot = if self.eat(&Token::Underscore) {
                    if seen_label {
                        let range = self.tokens[self.pos.saturating_sub(1)].start..self.prev_end;
                        return Err(SyntaxError {
                            kind: SyntaxErrorKind::InvalidToken,
                            range,
                        });
                    }
                    phalcom_common::selector::SelectorSlot::Positional
                } else {
                    seen_label = true;
                    phalcom_common::selector::SelectorSlot::Label(self.expect_identifier(&["label slot"])?)
                };
                let target = if after_gap { &mut suffix } else { &mut prefix };
                target.push(SelectorSlotSyntax {
                    slot,
                    range: (slot_start..self.prev_end).into(),
                });
            }
            self.skip_newlines();
            if self.eat(&Token::Comma) {
                continue;
            }
            if self.eat(&end) {
                return Ok((prefix, suffix, gap_range, self.prev_end));
            }
            return Err(self.error_here(strs(&["\",\"", "closing selector delimiter"])));
        }
    }

    fn parse_standalone_symbol_punctuation(&mut self) -> Option<String> {
        let name = match self.peek() {
            Token::Bang => "!",
            Token::Question => "?",
            Token::QuestionDot => "?.",
            Token::CoalesceQuestion => "??",
            Token::DotDotDot => "...",
            _ => return None,
        };
        self.advance();
        Some(name.to_string())
    }

    /// Returns whether a component-token hash symbol ends immediately before
    /// a product-label colon. This lookahead keeps `#name: value` distinct
    /// from an ordinary hash expression followed by another token.
    fn hash_symbol_has_label_colon(&self) -> bool {
        if !matches!(self.peek(), Token::Hash) {
            return false;
        }
        let Some(hash) = self.tokens.get(self.pos) else { return false };
        let Some(base) = self.tokens.get(self.pos + 1) else { return false };
        if hash.end != base.start {
            return false;
        }

        let mut next = self.pos + 2;
        if matches!(
            &base.token,
            Token::Identifier(_)
                | Token::FieldIdentifier(_)
                | Token::ImplementationFieldIdentifier(_)
                | Token::ImplementationSelectorIdentifier(_)
                | Token::Underscore
        ) {
            if let Some(open) = self.tokens.get(next) {
                if matches!(open.token, Token::LParen) && base.end == open.start {
                    let mut depth = 0usize;
                    while let Some(lexeme) = self.tokens.get(next) {
                        match &lexeme.token {
                            Token::LParen => depth += 1,
                            Token::RParen => {
                                depth = depth.saturating_sub(1);
                                if depth == 0 {
                                    next += 1;
                                    break;
                                }
                            }
                            _ => {}
                        }
                        next += 1;
                    }
                }
            }
        } else {
            next = self.pos + 2;
        }
        matches!(self.tokens.get(next).map(|lexeme| &lexeme.token), Some(Token::Colon))
    }

    fn parse_product_label(&mut self) -> ParserResult<Option<ProductLabel>> {
        let Some(start_kind) = self.product_label_start() else {
            return Ok(None);
        };
        let start = self.cur_start();
        let label = match start_kind {
            ProductLabelStart::Computed => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RBracket, &["]"])?;
                self.expect(&Token::Colon, &["\":\""])?;
                ProductLabel::Computed {
                    expr: Box::new(expr),
                    range: (start..self.prev_end).into(),
                }
            }
            ProductLabelStart::ExplicitSymbol => {
                let symbol = match self.peek().clone() {
                    Token::Hash => self.parse_hash_symbol()?,
                    Token::QuotedSymbol(value) => {
                        self.advance();
                        SymbolLiteralKind::Name(value)
                    }
                    _ => unreachable!("explicit symbol label start must be a symbol token"),
                };
                self.expect(&Token::Colon, &["\":\""])?;
                ProductLabel::Static {
                    symbol,
                    syntax: ProductLabelSyntax::ExplicitSymbol,
                    range: (start..self.prev_end).into(),
                }
            }
            ProductLabelStart::BareName(name) => {
                self.advance();
                if matches!(self.peek(), Token::Question) {
                    self.advance();
                }
                self.expect(&Token::Colon, &["\":\""])?;
                ProductLabel::Static {
                    symbol: SymbolLiteralKind::Name(name),
                    syntax: ProductLabelSyntax::Bare,
                    range: (start..self.prev_end).into(),
                }
            }
            ProductLabelStart::BareSelector(name) => {
                self.advance();
                self.expect(&Token::LParen, &["("])?;
                let labels = self.parse_selector_label_slots()?;
                self.expect(&Token::Colon, &["\":\""])?;
                ProductLabel::Static {
                    symbol: SymbolLiteralKind::Selector { name, labels },
                    syntax: ProductLabelSyntax::Bare,
                    range: (start..self.prev_end).into(),
                }
            }
        };
        Ok(Some(label))
    }

    fn expansion_mode_at_cursor(&self) -> Option<ExpansionMode> {
        match self.peek() {
            Token::Asterisk => Some(ExpansionMode::Positional),
            Token::DoubleAsterisk => Some(ExpansionMode::Labeled),
            Token::TripleAsterisk => Some(ExpansionMode::Complete),
            _ => None,
        }
    }

    /// Parses one already-unambiguous Tuple contribution. `**` and labels
    /// start the labeled source phase; `***` remains positional-phase legal.
    fn parse_tuple_entry(&mut self, labeled_phase: &mut bool) -> ParserResult<TupleLiteralEntry> {
        let start = self.cur_start();
        if let Some(label) = self.parse_product_label()? {
            let value = self.parse_expr()?;
            *labeled_phase = true;
            return Ok(TupleLiteralEntry::Labeled {
                label,
                value,
                range: (start..self.prev_end).into(),
            });
        }

        if let Some(mode) = self.expansion_mode_at_cursor() {
            if *labeled_phase && !matches!(mode, ExpansionMode::Labeled) {
                return Err(self.error_message_here("positional expansion cannot follow a labeled Tuple entry"));
            }
            self.advance();
            let expr = self.parse_expr()?;
            if matches!(mode, ExpansionMode::Labeled) {
                *labeled_phase = true;
            }
            return Ok(TupleLiteralEntry::Expand {
                mode,
                expr,
                range: (start..self.prev_end).into(),
            });
        }

        if *labeled_phase {
            return Err(self.error_message_here("positional Tuple entries cannot follow labeled entries"));
        }
        let expr = self.parse_expr()?;
        let range = expr.range();
        Ok(TupleLiteralEntry::Positional { expr, range })
    }

    fn parse_paren_or_tuple(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        self.advance(); // '('
        self.skip_newlines();
        if matches!(self.peek(), Token::RParen) {
            self.advance();
            return Ok(Expr::TupleLiteral(Box::new(TupleLiteralExpr {
                entries: Vec::new(),
                range: (start..self.prev_end).into(),
            })));
        }

        let first_is_unambiguous_tuple = self.product_label_start().is_some() || self.expansion_mode_at_cursor().is_some();
        let mut entries = Vec::new();
        let mut labeled_phase = false;
        if first_is_unambiguous_tuple {
            entries.push(self.parse_tuple_entry(&mut labeled_phase)?);
            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                self.expect(&Token::RParen, &[")"])?;
                return Ok(Expr::TupleLiteral(Box::new(TupleLiteralExpr {
                    entries,
                    range: (start..self.prev_end).into(),
                })));
            }
        } else {
            let expr = self.parse_expr()?;
            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                self.skip_newlines();
                self.expect(&Token::RParen, &[")"])?;
                let expr_range = expr.range();
                self.parenthesized_ranges.push((expr_range.start, expr_range.end));
                return Ok(expr);
            }
            let range = expr.range();
            entries.push(TupleLiteralEntry::Positional { expr, range });
        }

        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RParen) {
                break;
            }
            entries.push(self.parse_tuple_entry(&mut labeled_phase)?);

            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                break;
            }
            self.skip_newlines();
            if matches!(self.peek(), Token::RParen) {
                break;
            }
        }

        self.skip_newlines();
        self.expect(&Token::RParen, &[")"])?;
        Ok(Expr::TupleLiteral(Box::new(TupleLiteralExpr {
            entries,
            range: (start..self.prev_end).into(),
        })))
    }

    fn parse_record_literal(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        self.advance(); // `#{`
        self.skip_newlines();
        if matches!(self.peek(), Token::RBrace) {
            self.advance();
            return Ok(Expr::RecordLiteral(Box::new(RecordLiteralExpr {
                entries: Vec::new(),
                range: (start..self.prev_end).into(),
            })));
        }

        let mut entries = Vec::new();
        loop {
            self.skip_newlines();
            let entry_start = self.cur_start();
            let entry = match self.peek() {
                Token::Asterisk if !matches!(self.peek_next(), Token::Colon) => {
                    return Err(self.error_message_here("`*` is not valid in a Record literal; Record expansion uses the labeled lane with `**`"));
                }
                Token::TripleAsterisk if !matches!(self.peek_next(), Token::Colon) => {
                    return Err(
                        self.error_message_here("`***` is not valid in a Record literal; Record has no positional lane, use `**` for labeled expansion")
                    );
                }
                Token::DoubleAsterisk | Token::Power if !matches!(self.peek_next(), Token::Colon) => {
                    self.advance();
                    let expr = self.parse_expr()?;
                    RecordLiteralEntry::Expansion {
                        expr,
                        range: (entry_start..self.prev_end).into(),
                    }
                }
                _ => {
                    let Some(label) = self.parse_product_label()? else {
                        return Err(self.error_here(strs(&["label"])));
                    };
                    let value = self.parse_expr()?;
                    let label_start = match &label {
                        ProductLabel::Static { range, .. } | ProductLabel::Computed { range, .. } => range.start,
                    };
                    RecordLiteralEntry::Field(RecordLiteralField {
                        label,
                        value,
                        range: (label_start..self.prev_end).into(),
                    })
                }
            };
            entries.push(entry);

            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                break;
            }
            self.skip_newlines();
            // Leave the closing brace for the final `expect` below. Consuming
            // it here would make every trailing-comma Record fail by asking
            // for a second `}`.
            if matches!(self.peek(), Token::RBrace) {
                break;
            }
        }

        self.expect(&Token::RBrace, &["\"}\""])?;
        Ok(Expr::RecordLiteral(Box::new(RecordLiteralExpr {
            entries,
            range: (start..self.prev_end).into(),
        })))
    }

    /// Parses B.3a association Map syntax without desugaring through mutation.
    ///
    /// `**mapping` is represented structurally as an [`MapLiteralEntry::Expansion`]
    /// so Spec F can lower it later. This parser does not make expansion executable.
    fn parse_map_literal(&mut self, start: usize) -> ParserResult<Expr> {
        self.skip_newlines();
        if self.eat(&Token::RBrace) {
            return Ok(Expr::MapLiteral(Box::new(MapLiteralExpr {
                entries: Vec::new(),
                range: (start..self.prev_end).into(),
            })));
        }
        let mut entries = Vec::new();
        loop {
            self.skip_newlines();
            let key_start = self.cur_start();
            if matches!(self.peek(), Token::Asterisk) {
                return Err(self.error_message_here("`*` is not valid in a Map literal; Map expansion uses the labeled lane with `**`"));
            }
            if matches!(self.peek(), Token::TripleAsterisk) {
                return Err(self.error_message_here("`***` is not valid in a Map literal; Map has no positional lane, use `**` for labeled expansion"));
            }
            if matches!(self.peek(), Token::DoubleAsterisk | Token::Power) {
                self.advance();
                let expr = self.parse_expr()?;
                entries.push(MapLiteralEntry::Expansion {
                    expr,
                    range: (key_start..self.prev_end).into(),
                });
                self.skip_newlines();
                if !self.eat(&Token::Comma) {
                    break;
                }
                self.skip_newlines();
                if matches!(self.peek(), Token::RBrace) {
                    break;
                }
                continue;
            }
            let key = if self.eat(&Token::LBracket) {
                let expr = self.parse_expr()?;
                self.expect(&Token::RBracket, &["\"]\""])?;
                MapLiteralKey::Computed {
                    expr,
                    range: (key_start..self.prev_end).into(),
                }
            } else {
                let name = self.expect_identifier(&["map key"])?;
                MapLiteralKey::BareSymbol {
                    name,
                    range: (key_start..self.prev_end).into(),
                }
            };
            self.expect(&Token::Colon, &["\":\""])?;
            self.skip_newlines();
            let value = self.parse_expr()?;
            entries.push(MapLiteralEntry::Association {
                key,
                value,
                range: (key_start..self.prev_end).into(),
            });
            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                break;
            }
            self.skip_newlines();
            // Permit a trailing comma before the closing brace.
            if matches!(self.peek(), Token::RBrace) {
                break;
            }
        }
        self.skip_newlines();
        self.expect(&Token::RBrace, &["\"}\""])?;
        Ok(Expr::MapLiteral(Box::new(MapLiteralExpr {
            entries,
            range: (start..self.prev_end).into(),
        })))
    }

    /// Builds a free-form [`SyntaxErrorKind::Message`] diagnostic anchored at
    /// the current token's span.
    fn error_message_here(&self, msg: &str) -> SyntaxError {
        let lexeme = &self.tokens[self.pos];
        SyntaxError {
            kind: SyntaxErrorKind::Message(msg.to_string()),
            range: lexeme.start..lexeme.end,
        }
    }

    /// Parses a comma-separated, optionally `label:`-prefixed argument list.
    ///
    /// Shared by every call-shaped grammar: `(...)` call args and — since
    /// U-INDEX/ADR-0060 reuses this verbatim — `[...]` subscript args too.
    /// The short-circuit below checks for either closing delimiter so a
    /// zero-arg list (`f()`, `xs[]`) parses to an empty `Vec` regardless of
    /// which one the caller is about to [`Parser::expect`].
    fn parse_arg_list(&mut self) -> ParserResult<Vec<PackItem>> {
        self.skip_newlines();
        if matches!(self.peek(), Token::RParen | Token::RBracket) {
            return Ok(Vec::new());
        }
        let mut args = Vec::new();
        let mut labeled_phase = false;
        loop {
            self.skip_newlines();
            let start = self.cur_start();
            let item = if let Some(mode) = match self.peek() {
                Token::Asterisk => Some(ExpansionMode::Positional),
                Token::DoubleAsterisk => Some(ExpansionMode::Labeled),
                Token::TripleAsterisk => Some(ExpansionMode::Complete),
                _ => None,
            } {
                if labeled_phase && mode != ExpansionMode::Labeled {
                    return Err(self.error_message_here(match mode {
                        ExpansionMode::Positional => "positional expansion cannot follow a labeled argument",
                        ExpansionMode::Complete => "complete expansion cannot follow a labeled argument",
                        ExpansionMode::Labeled => unreachable!(),
                    }));
                }
                self.advance();
                let expr = self.parse_expr()?;
                if mode == ExpansionMode::Labeled {
                    labeled_phase = true;
                }
                PackItem::Expand {
                    mode,
                    expr,
                    range: (start..self.prev_end).into(),
                }
            } else {
                let label = self.parse_product_label()?;
                if let Some(label) = label {
                    labeled_phase = true;
                    let label = match label {
                        ProductLabel::Static { symbol, range, .. } => PackLabel::Static {
                            text: symbol_text(&symbol),
                            range,
                        },
                        ProductLabel::Computed { expr, range } => PackLabel::Computed { expr, range },
                    };
                    let value = self.parse_expr()?;
                    PackItem::Labeled {
                        label,
                        value,
                        range: (start..self.prev_end).into(),
                    }
                } else {
                    if labeled_phase {
                        return Err(self.error_message_here("positional argument cannot follow a labeled argument"));
                    }
                    let expr = self.parse_expr()?;
                    PackItem::Positional {
                        expr,
                        range: (start..self.prev_end).into(),
                    }
                }
            };
            args.push(item);
            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        self.skip_newlines();
        Ok(args)
    }
}

fn symbol_text(symbol: &SymbolLiteralKind) -> String {
    match symbol {
        SymbolLiteralKind::Name(text) => text.clone(),
        SymbolLiteralKind::Selector { name, labels } => {
            let slots = labels
                .iter()
                .map(|label| label.clone().unwrap_or_else(|| "_".to_string()))
                .collect::<Vec<_>>()
                .join(",");
            format!("{name}({slots})")
        }
        SymbolLiteralKind::Subscript { labels, setter } => {
            let slots = labels
                .iter()
                .map(|label| label.clone().unwrap_or_else(|| "_".to_string()))
                .collect::<Vec<_>>()
                .join(",");
            if *setter { format!("[{slots}]=(put)") } else { format!("[{slots}]") }
        }
        SymbolLiteralKind::Pattern(pattern) => pattern.normalize().map(|p| p.encode()).unwrap_or_else(|_| pattern.base.clone()),
    }
}

fn exact_symbol_kind(spec: ExactSelectorSyntax) -> SymbolLiteralKind {
    if spec.is_subscript {
        return SymbolLiteralKind::Subscript {
            labels: spec
                .slots
                .into_iter()
                .map(|slot| match slot.slot {
                    phalcom_common::selector::SelectorSlot::Positional => None,
                    phalcom_common::selector::SelectorSlot::Label(label) => Some(label),
                })
                .collect(),
            setter: matches!(spec.kind, phalcom_common::selector::SelectorKind::SubscriptSet),
        };
    }
    match spec.kind {
        phalcom_common::selector::SelectorKind::Getter => SymbolLiteralKind::Name(spec.base),
        phalcom_common::selector::SelectorKind::Setter => SymbolLiteralKind::Selector {
            name: format!("{}=", spec.base),
            labels: vec![Some("put".to_string())],
        },
        _ => SymbolLiteralKind::Selector {
            name: spec.base,
            labels: spec
                .slots
                .into_iter()
                .map(|slot| match slot.slot {
                    phalcom_common::selector::SelectorSlot::Positional => None,
                    phalcom_common::selector::SelectorSlot::Label(label) => Some(label),
                })
                .collect(),
        },
    }
}

/// Maps a token to its binary-operator precedence and [`BinaryOp`], or `None`.
///
/// Higher numbers bind tighter: `or` (1), `and` (2), equality (3), comparison
/// (4), additive (5), multiplicative (6).
fn binary_op(token: &Token) -> Option<(u8, BinaryOp)> {
    Some(match token {
        Token::Or => (1, BinaryOp::Or),
        Token::And => (2, BinaryOp::And),
        Token::EqualEqual => (3, BinaryOp::Equal),
        Token::TripleEqual => (3, BinaryOp::Same),
        Token::BangEqual => (3, BinaryOp::NotEqual),
        Token::Less => (4, BinaryOp::LessThan),
        Token::LessEqual => (4, BinaryOp::LessThanOrEqual),
        Token::Greater => (4, BinaryOp::GreaterThan),
        Token::GreaterEqual => (4, BinaryOp::GreaterThanOrEqual),
        Token::Spaceship => (4, BinaryOp::Compare),
        Token::Plus => (5, BinaryOp::Add),
        Token::Minus => (5, BinaryOp::Subtract),
        Token::Asterisk => (6, BinaryOp::Multiply),
        Token::DoubleAsterisk | Token::Power => (6, BinaryOp::Power),
        Token::Slash => (6, BinaryOp::Divide),
        Token::SlashTilde => (6, BinaryOp::IntegerDivide),
        Token::Percent => (6, BinaryOp::Modulo),
        Token::ShiftLeft => (5, BinaryOp::ShiftLeft),
        Token::ShiftRight => (5, BinaryOp::ShiftRight),
        Token::Ampersand => (4, BinaryOp::BitAnd),
        Token::Caret => (3, BinaryOp::BitXor),
        Token::Pipe => (2, BinaryOp::BitOr),
        _ => return None,
    })
}

fn contextual_relation_op(token: &Token) -> Option<RelationOp> {
    match token {
        Token::Identifier(name) if name == "matches" => Some(RelationOp::Matches),
        Token::Identifier(name) if name == "understands" => Some(RelationOp::Understands),
        _ => None,
    }
}

fn finish_chain(operands: Vec<Expr>, mut operators: Vec<(RelationOp, SourceRange)>, range: SourceRange) -> Expr {
    if operands.len() == 2 && operators.len() == 1 {
        let (op, op_range) = operators.remove(0);
        if let RelationOp::Binary(binary) = op {
            let mut it = operands.into_iter();
            let left = it.next().unwrap();
            let right = it.next().unwrap();
            return Expr::Binary(Box::new(BinaryExpr {
                op: binary,
                op_range: Some(op_range),
                left,
                right,
                range,
            }));
        }
        operators.push((op, op_range));
    }
    Expr::ComparisonChain(Box::new(ComparisonChainExpr {
        operands,
        operators: operators.into_iter().map(|(op, _)| op).collect(),
        range,
    }))
}

fn is_chain_relation(op: &BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Equal
            | BinaryOp::Same
            | BinaryOp::NotEqual
            | BinaryOp::LessThan
            | BinaryOp::LessThanOrEqual
            | BinaryOp::GreaterThan
            | BinaryOp::GreaterThanOrEqual
    )
}

/// Maps a compound-assignment token (`+=`, `-=`, ...) to the [`BinaryOp`] it
/// desugars to, or `None`.
fn compound_op(token: &Token) -> Option<BinaryOp> {
    Some(match token {
        Token::PlusEqual => BinaryOp::Add,
        Token::MinusEqual => BinaryOp::Subtract,
        Token::AsteriskEqual => BinaryOp::Multiply,
        Token::SlashEqual => BinaryOp::Divide,
        Token::PercentEqual => BinaryOp::Modulo,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source_parses_to_empty_program() {
        let result = parse("", 0);
        assert!(result.errors.is_empty());
        assert!(result.program.statements.is_empty());
    }

    #[test]
    fn trailing_newline_is_accepted() {
        // F10: a file ending in a newline must parse cleanly.
        assert!(parse_source("let x = 1\n", 0).is_ok());
    }

    #[test]
    fn class_keyword_is_valid_selector_in_class_members() {
        let source =
            "@native\nclass Object {\n  @native\n  class -> Dynamic\n  @native\n  class=(put value: Dynamic) -> Dynamic\n  @native\n  try() -> Dynamic\n}\n";
        let program = parse_source(source, 0).expect("class selectors must parse");
        let Statement::Class(class) = program.statements.into_iter().next().expect("class statement") else {
            panic!("expected class declaration");
        };
        assert!(matches!(&class.members[0], ClassMember::Getter(getter) if getter.name == "class"));
        assert!(matches!(&class.members[1], ClassMember::Setter(setter) if setter.name == "class"));
        assert!(matches!(&class.members[2], ClassMember::Method(method) if method.name == "try"));
    }

    #[test]
    fn generic_return_declaration_accepts_next_member_attribute() {
        let source = "class Holder {\n  value -> Option<Int>\n  @native\n  next -> Int\n}\n";
        let program = parse_source(source, 0).expect("generic declaration must terminate before next attribute");
        let Statement::Class(class) = program.statements.into_iter().next().expect("class statement") else {
            panic!("expected class declaration");
        };
        assert_eq!(class.members.len(), 2);
    }

    #[test]
    fn recovers_and_reports_multiple_errors() {
        // Two broken top-level statements must both be reported, not just the
        // first — error recovery synchronises between them. Each line must end
        // in a token that *can* end a statement (a number here) so the
        // separating newline is not suppressed by D3's continuation rule
        // (`lexer::suppresses_following_newline`); a line ending in an operator
        // would legitimately continue onto the next.
        let result = parse("let 9\nlet 9\n", 0);
        assert!(result.errors.len() >= 2, "expected at least two recovered errors, got {:?}", result.errors);
    }

    #[test]
    fn offset_is_applied_to_spans() {
        let result = parse("x", 100);
        let stmt = &result.program.statements[0];
        let Statement::Expr { range, .. } = stmt else {
            panic!("expected an expression statement");
        };
        assert_eq!(range.start, 100);
        assert_eq!(range.end, 101);
    }

    #[test]
    fn parse_error_exits_with_first_error() {
        let err = parse_source("let = )", 0).unwrap_err();
        assert_eq!(err.range, 4..5);
    }

    #[test]
    fn new_operators_and_expression_ellipsis_parse() {
        let Statement::Expr {
            expr: Expr::Binary(binary), ..
        } = only_statement("1 === 1")
        else {
            panic!("expected exact-sameness binary expression");
        };
        assert!(matches!(binary.op, BinaryOp::Same));

        let Statement::Expr {
            expr: Expr::Binary(binary), ..
        } = only_statement("a <=> b")
        else {
            panic!("expected spaceship binary expression");
        };
        assert!(matches!(binary.op, BinaryOp::Compare));

        let Statement::Expr {
            expr: Expr::Ellipsis { .. }, ..
        } = only_statement("...")
        else {
            panic!("expected expression ellipsis");
        };
    }

    #[test]
    fn control_headers_accept_grouped_and_unwrapped_conditions() {
        assert!(parse("if value { value }", 0).errors.is_empty());
        assert!(parse("if (value or other) { value }", 0).errors.is_empty());
        assert!(parse("if value.isEmpty { value }", 0).errors.is_empty());
        assert!(parse("while value { break }", 0).errors.is_empty());
        assert!(parse("while (value and other) { break }", 0).errors.is_empty());
        assert!(parse("while value.isEmpty { break }", 0).errors.is_empty());
    }

    #[test]
    fn for_lanes_preserve_tuple_patterns_and_contextual_at() {
        let Statement::For(for_statement) = only_statement("for (x, y) at i in pairs, z in values { x }") else {
            panic!("expected for statement");
        };
        assert_eq!(for_statement.lanes.len(), 2);
        assert!(matches!(for_statement.lanes[0].pattern, Pattern::Tuple { .. }));
        assert_eq!(for_statement.lanes[0].index.as_ref().map(|index| index.name.as_str()), Some("i"));
        assert!(matches!(&for_statement.lanes[1].pattern, Pattern::Name { name, .. } if name == "z"));
    }

    #[test]
    fn contextual_relations_form_explicit_comparison_chains() {
        let Statement::Expr {
            expr: Expr::ComparisonChain(chain),
            ..
        } = only_statement("a < b <= c matches pattern")
        else {
            panic!("expected comparison chain");
        };
        assert_eq!(chain.operands.len(), 4);
        assert!(matches!(chain.operators[0], RelationOp::Binary(BinaryOp::LessThan)));
        assert!(matches!(chain.operators[1], RelationOp::Binary(BinaryOp::LessThanOrEqual)));
        assert!(matches!(chain.operators[2], RelationOp::Matches));
    }

    #[test]
    fn if_let_and_while_let_are_explicit_nodes() {
        let Statement::Expr { expr: Expr::IfLet(if_let), .. } = only_statement("if let Some(value) = option { value } else { None }") else {
            panic!("expected if let expression");
        };
        assert!(matches!(if_let.pattern, Pattern::Variant(VariantPattern { ref base, .. }) if base == "Some"));

        let Statement::Expr {
            expr: Expr::WhileLet(while_let),
            ..
        } = only_statement("while let Some(value) = next() { value }")
        else {
            panic!("expected while let expression");
        };
        assert!(matches!(while_let.pattern, Pattern::Variant(VariantPattern { ref base, .. }) if base == "Some"));
    }

    /// Returns the single statement of a program that must parse cleanly.
    fn only_statement(src: &str) -> Statement {
        let result = parse(src, 0);
        assert!(result.errors.is_empty(), "unexpected errors: {:?}", result.errors);
        assert_eq!(result.program.statements.len(), 1);
        result.program.statements.into_iter().next().unwrap()
    }

    fn positional_pack_expr(item: &PackItem) -> &Expr {
        match item {
            PackItem::Positional { expr, .. } => expr,
            other => panic!("expected positional pack item, got {other:?}"),
        }
    }

    fn static_pack_label(item: &PackItem) -> Option<&str> {
        match item {
            PackItem::Positional { .. } => None,
            PackItem::Labeled {
                label: PackLabel::Static { text, .. },
                ..
            } => Some(text),
            other => panic!("expected static pack item, got {other:?}"),
        }
    }

    fn pack_item_range(item: &PackItem) -> SourceRange {
        match item {
            PackItem::Positional { range, .. } | PackItem::Labeled { range, .. } | PackItem::Expand { range, .. } => *range,
        }
    }

    #[test]
    fn let_binding_records_mutable_kind() {
        let Statement::Let(binding) = only_statement("let x = 1") else {
            panic!("expected a let binding");
        };
        assert_eq!(binding.kind, BindingKind::Let);
        assert!(matches!(&binding.pattern, Pattern::Name { name, .. } if name == "x"));
        assert!(binding.value.is_some());
    }

    #[test]
    fn const_binding_records_immutable_kind() {
        let Statement::Let(binding) = only_statement("const x = 1") else {
            panic!("expected a const binding");
        };
        assert_eq!(binding.kind, BindingKind::Const);
        assert!(matches!(&binding.pattern, Pattern::Name { name, .. } if name == "x"));
        assert!(binding.value.is_some());
    }

    #[test]
    fn tuple_pattern_parses_two_element_binding() {
        // U14, open-questions.md Q7: `let (a, b) = point` — a tuple pattern.
        let Statement::Let(binding) = only_statement("let (a, b) = point") else {
            panic!("expected a let binding");
        };
        let Pattern::Tuple { elements, .. } = &binding.pattern else {
            panic!("expected a tuple pattern, got {:?}", binding.pattern);
        };
        assert_eq!(elements.len(), 2);
        assert!(matches!(&elements[0], Pattern::Name { name, .. } if name == "a"));
        assert!(matches!(&elements[1], Pattern::Name { name, .. } if name == "b"));
    }

    #[test]
    fn paren_pattern_with_no_comma_is_grouping_not_a_one_tuple() {
        // Mirrors the RHS literal's grouping-vs-tuple rule: `(x)` is `x`.
        let Statement::Let(binding) = only_statement("let (x) = point") else {
            panic!("expected a let binding");
        };
        assert!(matches!(&binding.pattern, Pattern::Name { name, .. } if name == "x"));
    }

    #[test]
    fn list_pattern_with_rest_parses() {
        // U14: `let [first, *rest] = list`.
        let Statement::Let(binding) = only_statement("let [first, *rest] = list") else {
            panic!("expected a let binding");
        };
        let Pattern::List { elements, rest, .. } = &binding.pattern else {
            panic!("expected a list pattern, got {:?}", binding.pattern);
        };
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], Pattern::Name { name, .. } if name == "first"));
        let rest = rest.as_deref().expect("expected a rest sub-pattern");
        assert!(matches!(rest, Pattern::Name { name, .. } if name == "rest"));
    }

    #[test]
    fn list_literal_parses_positional_expansion() {
        let Statement::Expr { expr, .. } = only_statement("[first, *middle, last,]") else {
            panic!("expected a List literal expression");
        };
        let Expr::ListLiteral(list) = expr else {
            panic!("expected List literal, got {expr:?}");
        };
        assert_eq!(list.elements.len(), 3);
        assert!(matches!(&list.elements[0], ListLiteralElement::Element { .. }));
        assert!(matches!(&list.elements[1], ListLiteralElement::Expansion { .. }));
        assert!(matches!(&list.elements[2], ListLiteralElement::Element { .. }));
    }

    #[test]
    fn list_literal_rejects_labeled_expansion_operators() {
        for (source, range, message) in [
            (
                "[**source]",
                1..3,
                "`**` is not valid in a List literal; List has only a positional lane, use `*`",
            ),
            (
                "[***source]",
                1..4,
                "`***` is not valid in a List literal; complete expansion requires positional and labeled lanes, but List has only a positional lane",
            ),
        ] {
            let error = parse_source(source, 0).expect_err("invalid List expansion should fail during parsing");
            assert_eq!(error.range, range);
            assert!(error.to_string().contains(message), "unexpected diagnostic: {error}");
        }
    }

    #[test]
    fn nested_tuple_pattern_parses() {
        // U14: `let ((a, b), c) = …` — patterns nest recursively.
        let Statement::Let(binding) = only_statement("let ((a, b), c) = pair") else {
            panic!("expected a let binding");
        };
        let Pattern::Tuple { elements, .. } = &binding.pattern else {
            panic!("expected a tuple pattern, got {:?}", binding.pattern);
        };
        assert_eq!(elements.len(), 2);
        assert!(matches!(&elements[0], Pattern::Tuple { elements, .. } if elements.len() == 2));
        assert!(matches!(&elements[1], Pattern::Name { name, .. } if name == "c"));
    }

    #[test]
    fn interior_rest_pattern_is_a_parse_error() {
        // `*rest` must be the list pattern's last element.
        let result = parse("let [*rest, last] = xs", 0);
        assert!(!result.errors.is_empty(), "expected a parse error for an interior rest pattern");
    }

    #[test]
    fn surface_nil_parses_as_a_plain_variable() {
        // ADR-0007: `nil` is no longer a keyword/literal — it lexes as an
        // ordinary identifier, which the compiler later rejects as undefined.
        let Statement::Expr { expr, .. } = only_statement("nil") else {
            panic!("expected an expression statement");
        };
        let Expr::Var { value, .. } = expr else {
            panic!("expected `nil` to parse as a variable reference, got {expr:?}");
        };
        assert_eq!(value, "nil");
    }

    #[test]
    fn coalesce_desugars_to_or_else_over_a_block() {
        // `a ?? b` ≡ `a.orElse { b }` (ADR-0007, values-and-absence §3.4).
        let Statement::Expr { expr, .. } = only_statement("a ?? b") else {
            panic!("expected an expression statement");
        };
        let Expr::MethodCall(call) = expr else {
            panic!("expected a method call, got {expr:?}");
        };
        assert_eq!(call.method, "orElse");
        assert!(matches!(call.object, Expr::Var { .. }));
        assert_eq!(call.args.len(), 1);
        let Expr::Block(block) = positional_pack_expr(&call.args[0]) else {
            panic!("expected a block thunk argument");
        };
        assert!(block.params.fixed.is_empty());
        assert!(block.params.positional_rest.is_none());
        assert!(block.expr_body);
    }

    #[test]
    fn coalesce_is_right_associative() {
        // `a ?? b ?? c` ≡ `a ?? (b ?? c)`: the block thunk holds another
        // `orElse` send.
        let Statement::Expr { expr, .. } = only_statement("a ?? b ?? c") else {
            panic!("expected an expression statement");
        };
        let Expr::MethodCall(outer) = expr else {
            panic!("expected an outer method call");
        };
        assert_eq!(outer.method, "orElse");
        let Expr::Block(thunk) = positional_pack_expr(&outer.args[0]) else {
            panic!("expected a block thunk");
        };
        let Statement::Expr { expr: inner, .. } = &thunk.body[0] else {
            panic!("expected an expression in the thunk body");
        };
        let Expr::MethodCall(inner_call) = inner else {
            panic!("expected the thunk to hold a nested orElse send, got {inner:?}");
        };
        assert_eq!(inner_call.method, "orElse");
    }

    #[test]
    fn optional_property_desugars_to_map_over_a_getter() {
        // `opt?.foo` ≡ `opt.map { <recv> => <recv>.foo }` (values-and-absence §3.4).
        let Statement::Expr { expr, .. } = only_statement("opt?.foo") else {
            panic!("expected an expression statement");
        };
        let Expr::MethodCall(call) = expr else {
            panic!("expected a map send, got {expr:?}");
        };
        assert_eq!(call.method, "map");
        let Expr::Block(block) = positional_pack_expr(&call.args[0]) else {
            panic!("expected a mapper block");
        };
        assert_eq!(block.params.fixed.len(), 1);
        let Statement::Expr { expr: body, .. } = &block.body[0] else {
            panic!("expected an expression body");
        };
        let Expr::GetProperty(get) = body else {
            panic!("expected a property access in the mapper, got {body:?}");
        };
        assert_eq!(get.property, "foo");
        // The mapper's receiver is the synthetic block parameter, not a source
        // variable, so it cannot collide with user code.
        let Expr::Var { value, .. } = &get.object else {
            panic!("expected the synthetic receiver variable");
        };
        assert_eq!(value, &block.params.fixed[0].name);
    }

    #[test]
    fn optional_call_desugars_to_map_over_a_send() {
        // `opt?.bar(baz)` ≡ `opt.map { <recv> => <recv>.bar(baz) }`.
        let Statement::Expr { expr, .. } = only_statement("opt?.bar(baz)") else {
            panic!("expected an expression statement");
        };
        let Expr::MethodCall(call) = expr else {
            panic!("expected a map send");
        };
        assert_eq!(call.method, "map");
        let Expr::Block(block) = positional_pack_expr(&call.args[0]) else {
            panic!("expected a mapper block");
        };
        let Statement::Expr { expr: body, .. } = &block.body[0] else {
            panic!("expected an expression body");
        };
        let Expr::MethodCall(inner) = body else {
            panic!("expected an inner send, got {body:?}");
        };
        assert_eq!(inner.method, "bar");
        assert_eq!(inner.args.len(), 1);
    }

    #[test]
    fn optional_chain_left_associates() {
        // `a?.b?.c` ≡ `(a?.b)?.c`: the outer map's receiver is itself a map send.
        let Statement::Expr { expr, .. } = only_statement("a?.b?.c") else {
            panic!("expected an expression statement");
        };
        let Expr::MethodCall(outer) = expr else {
            panic!("expected an outer map send");
        };
        assert_eq!(outer.method, "map");
        let Expr::MethodCall(receiver) = &outer.object else {
            panic!("expected the receiver to be a nested map send, got {:?}", outer.object);
        };
        assert_eq!(receiver.method, "map");
    }

    #[test]
    fn power_associativity_and_precedence() {
        // `2 ** 3 ** 2` parses as `2 ** (3 ** 2)` (right-associative)
        let Statement::Expr { expr, .. } = only_statement("2 ** 3 ** 2") else {
            panic!()
        };
        let Expr::Binary(b1) = expr else { panic!() };
        assert!(matches!(b1.op, BinaryOp::Power));
        let Expr::Binary(ref b2) = b1.right else { panic!() };
        assert!(matches!(b2.op, BinaryOp::Power));

        // `-2 ** 2` parses as `-(2 ** 2)` (unary prefix `-` has lower precedence than `**`)
        let Statement::Expr { expr: e2, .. } = only_statement("-2 ** 2") else {
            panic!()
        };
        let Expr::Unary(u1) = e2 else { panic!() };
        assert!(matches!(u1.expr, Expr::Binary(_)));

        // `2 ** -2` parses as `2 ** (-2)`
        let Statement::Expr { expr: e3, .. } = only_statement("2 ** -2") else {
            panic!()
        };
        let Expr::Binary(b3) = e3 else { panic!() };
        assert!(matches!(b3.right, Expr::Unary(_)));
    }

    #[test]
    fn map_expansion_parses_into_reserved_entry() {
        let Statement::Expr { expr, .. } = only_statement("{ **mapping, name: value }") else {
            panic!("expected an expression statement");
        };
        let Expr::MapLiteral(map) = expr else {
            panic!("expected a Map literal, got {expr:?}");
        };
        assert_eq!(map.entries.len(), 2);
        assert!(matches!(map.entries[0], MapLiteralEntry::Expansion { .. }));
        assert!(matches!(map.entries[1], MapLiteralEntry::Association { .. }));
    }

    #[test]
    fn pipe_closures_preserve_existing_block_ast_shape() {
        let Statement::Let(binding) = only_statement("const f = |x, y| x + y") else {
            panic!("expected binding");
        };
        let Expr::Block(block) = binding.value.expect("expected value") else {
            panic!("expected closure block");
        };
        assert_eq!(block.params.fixed.iter().map(|param| param.name.as_str()).collect::<Vec<_>>(), ["x", "y"]);
        assert!(block.params.positional_rest.is_none());
        assert!(block.expr_body);
        assert_eq!(block.body.len(), 1);

        let Statement::Let(binding) = only_statement("const f = || {\n  1\n}") else {
            panic!("expected binding");
        };
        let Expr::Block(block) = binding.value.expect("expected value") else {
            panic!("expected closure block");
        };
        assert!(block.params.fixed.is_empty());
        assert!(block.params.positional_rest.is_none());
        assert!(!block.expr_body);
    }

    #[test]
    fn closure_positional_rest_is_structured_and_terminal() {
        let Statement::Let(binding) = only_statement("const f = |head, *tail| tail") else {
            panic!("expected binding");
        };
        let Expr::Block(block) = binding.value.expect("expected closure") else {
            panic!("expected closure block");
        };
        assert_eq!(block.params.fixed.iter().map(|param| param.name.as_str()).collect::<Vec<_>>(), ["head"]);
        assert_eq!(block.params.positional_rest.as_ref().map(|param| param.name.as_str()), Some("tail"));

        for source in ["|**labels| labels", "|*rest, next| rest", "|*rest, *other| rest"] {
            assert!(!parse(source, 0).errors.is_empty(), "invalid closure rest accepted: {source}");
        }
    }

    #[test]
    fn bare_brace_trailing_closure_is_synthesized_only_for_member_sends() {
        let Statement::Expr { expr, .. } = only_statement("items.map { value }") else {
            panic!("expected expression statement");
        };
        let Expr::MethodCall(call) = expr else {
            panic!("expected method call");
        };
        assert!(matches!(&call.args[0], PackItem::Positional { expr: Expr::Block(_), .. }));

        assert!(!parse("items[0] { value }", 0).errors.is_empty());
        assert!(!parse("items::map { value }", 0).errors.is_empty());
        assert!(parse("items.map({ key: value })", 0).errors.is_empty());
    }

    #[test]
    fn bare_labelled_trailing_closure_can_follow_bare_closure() {
        let Statement::Expr { expr, .. } = only_statement("self.ifTrue { \"true\" } ifFalse: { \"false\" }") else {
            panic!("expected expression statement");
        };
        let Expr::MethodCall(call) = expr else {
            panic!("expected method call");
        };
        assert_eq!(call.method, "ifTrue");
        assert_eq!(call.args.iter().map(static_pack_label).collect::<Vec<_>>(), [None, Some("ifFalse")]);
        assert!(matches!(&call.args[0], PackItem::Positional { expr: Expr::Block(_), .. }));
        assert!(matches!(&call.args[1], PackItem::Labeled { value: Expr::Block(_), .. }));
    }

    #[test]
    fn trailing_closures_attach_as_ordinary_method_arguments() {
        let Statement::Expr { expr, .. } = only_statement("items.any where: |item| {\n  item.valid\n}") else {
            panic!("expected expression statement");
        };
        let Expr::MethodCall(call) = expr else {
            panic!("expected method call");
        };
        assert_eq!(call.method, "any");
        assert_eq!(call.args.len(), 1);
        assert_eq!(static_pack_label(&call.args[0]), Some("where"));
        assert!(matches!(&call.args[0], PackItem::Labeled { value: Expr::Block(_), .. }));

        let Statement::Expr { expr, .. } = only_statement("result.match\n  ok: |v| { v },\n  err: |e| { e }") else {
            panic!("expected expression statement");
        };
        let Expr::MethodCall(call) = expr else {
            panic!("expected method call");
        };
        assert_eq!(call.method, "match");
        assert_eq!(call.args.iter().map(static_pack_label).collect::<Vec<_>>(), [Some("ok"), Some("err")]);
    }

    #[test]
    fn labelled_trailing_closure_can_follow_bare_closure_on_newline() {
        let parsed = parse("const x = 5.ifTrue { \"a\" }\n  ifFalse: || { \"b\" }", 0);
        assert!(parsed.errors.is_empty(), "unexpected parse errors: {:?}", parsed.errors);
    }

    #[test]
    fn trailing_closure_chains_across_newlines_without_capturing_bitwise_or() {
        let Statement::Expr { expr, .. } = only_statement("numbers\n  .map |value| { value * 2 }\n  .filter |value| { value > 10 }") else {
            panic!("expected expression statement");
        };
        let Expr::MethodCall(filter) = expr else {
            panic!("expected filter call");
        };
        assert_eq!(filter.method, "filter");
        assert!(matches!(filter.object, Expr::MethodCall(_)));

        let Statement::Expr { expr, .. } = only_statement("obj.flags | mask | other") else {
            panic!("expected expression statement");
        };
        let Expr::Binary(binary) = expr else {
            panic!("expected bitwise binary expression");
        };
        assert!(matches!(binary.op, BinaryOp::BitOr));
        assert!(matches!(binary.left, Expr::Binary(_)));
    }

    #[test]
    fn tuple_entries_share_one_pack_aware_parser_in_every_position() {
        let Statement::Expr { expr, .. } = only_statement("(1, *xs, ***pack, label: 2, **tail)") else {
            panic!("expected tuple expression")
        };
        let Expr::TupleLiteral(tuple) = expr else { panic!("expected tuple literal") };
        assert!(matches!(
            tuple.entries.as_slice(),
            [
                TupleLiteralEntry::Positional { .. },
                TupleLiteralEntry::Expand {
                    mode: ExpansionMode::Positional,
                    ..
                },
                TupleLiteralEntry::Expand {
                    mode: ExpansionMode::Complete,
                    ..
                },
                TupleLiteralEntry::Labeled { .. },
                TupleLiteralEntry::Expand {
                    mode: ExpansionMode::Labeled,
                    ..
                },
            ]
        ));

        let Statement::Expr { expr, .. } = only_statement("(***first, x, ***second, label: y)") else {
            panic!("expected tuple expression")
        };
        let Expr::TupleLiteral(tuple) = expr else { panic!("expected tuple literal") };
        assert_eq!(tuple.entries.len(), 4, "*** must not start the labeled phase");
    }

    #[test]
    fn tuple_pack_entries_accept_trailing_commas_without_consuming_the_closer_twice() {
        for source in ["(*xs,)", "(**labels,)", "(tag: 1,)"] {
            let result = parse(source, 0);
            assert!(result.errors.is_empty(), "{source} must parse: {:?}", result.errors);
        }
    }

    #[test]
    fn pure_tuple_expansions_are_tuples_with_or_without_trailing_commas() {
        for (source, mode) in [
            ("(*args)", ExpansionMode::Positional),
            ("(*args,)", ExpansionMode::Positional),
            ("(**args)", ExpansionMode::Labeled),
            ("(**args,)", ExpansionMode::Labeled),
            ("(***args)", ExpansionMode::Complete),
            ("(***args,)", ExpansionMode::Complete),
        ] {
            let Statement::Expr {
                expr: Expr::TupleLiteral(tuple),
                ..
            } = only_statement(source)
            else {
                panic!("{source} must produce a Tuple literal");
            };
            assert!(matches!(
                tuple.entries.as_slice(),
                [TupleLiteralEntry::Expand { mode: actual, .. }] if *actual == mode
            ));
        }

        let Statement::Expr { expr, .. } = only_statement("(value)") else {
            panic!("expected grouped expression");
        };
        assert!(matches!(expr, Expr::Var { value, .. } if value == "value"));

        let Statement::Expr {
            expr: Expr::TupleLiteral(tuple),
            ..
        } = only_statement("(value,)")
        else {
            panic!("expected singleton Tuple literal");
        };
        assert!(matches!(
            tuple.entries.as_slice(),
            [TupleLiteralEntry::Positional {
                expr: Expr::Var { value, .. }, ..
            }] if value == "value"
        ));
    }

    #[test]
    fn tuple_pack_source_phase_rejects_positionals_after_labels() {
        for source in ["(label: 1, 2)", "(**labels, *xs)", "(label: 1, ***pack)"] {
            let result = parse(source, 0);
            assert!(!result.errors.is_empty(), "{source} must be rejected after labeled phase begins");
        }
    }

    #[test]
    fn declaration_labels_and_subscript_rest_are_rejected() {
        assert!(!parse("class C { f(x first, x second) {} }", 0).errors.is_empty());
        for source in ["class C { [*indices] {} }", "class C { [**labels] {} }", "class C { [***pack] {} }"] {
            assert!(!parse(source, 0).errors.is_empty(), "{source} must reject subscript rest");
        }
    }

    #[test]
    fn trailing_closure_ranges_and_mixed_labels_match_parenthesized_calls() {
        let source = "predicate\n  .ifTrue || { 1 }\n  ifFalse: || { 2 }";
        let Statement::Expr { expr, .. } = only_statement(source) else {
            panic!("expected expression statement");
        };
        let Expr::MethodCall(call) = expr else {
            panic!("expected method call");
        };
        assert_eq!(call.method, "ifTrue");
        assert_eq!(call.args.iter().map(static_pack_label).collect::<Vec<_>>(), [None, Some("ifFalse")]);
        assert_eq!(pack_item_range(&call.args[1]).start, source.find("ifFalse").unwrap());
        assert_eq!(call.range.end, source.len());

        let Statement::Expr { expr, .. } = only_statement("items.map(|x| x + 1)") else {
            panic!("expected expression statement");
        };
        let Expr::MethodCall(call) = expr else {
            panic!("expected method call");
        };
        assert_eq!(call.method, "map");
        assert!(matches!(positional_pack_expr(&call.args[0]), Expr::Block(block) if block.expr_body));

        let Statement::Return(return_statement) = only_statement("return || { 1 }.on(Error) |e| { e }") else {
            panic!("expected return statement");
        };
        let Some(expr) = return_statement.value else {
            panic!("expected return value");
        };
        let Expr::MethodCall(call) = expr else {
            panic!("expected method call");
        };
        assert_eq!(call.method, "on");
        assert_eq!(call.args.len(), 2);
        assert!(matches!(positional_pack_expr(&call.args[1]), Expr::Block(_)));
    }
}
