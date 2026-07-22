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
use crate::error::{SyntaxError, SyntaxErrorKind};
use crate::lexer::Lexer;
use crate::token::{LexicalError, StringSegment, Token};
use phalcom_common::range::SourceRange;
use std::ops::Range;

/// The three pieces [`Parser::parse_class_body`] assembles a [`ClassDef`]
/// from: its members, its (currently always-empty, see that field's doc)
/// class-level attributes, and its standalone `@invariant(...)` predicates
/// (DEC-ANNOT-B).
type ClassBodyParts = (Vec<ClassMember>, Vec<Attribute>, Vec<(Expr, SourceRange)>);

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

    /// Skips any run of [`Token::Newline`] tokens (blank lines).
    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }
    }

    // ── Program / statements ─────────────────────────────────────────────────

    /// Parses a whole program, recovering from errors between top-level
    /// statements.
    ///
    /// Blank lines are ignored. Each iteration parses one top-level item (a
    /// `class` declaration or a `;`/newline-separated run of small statements);
    /// on failure the error is recorded and the parser synchronises to the next
    /// statement boundary before continuing.
    fn parse_program(&mut self) -> Program {
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
        Program { statements }
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
        if matches!(self.peek(), Token::Class | Token::At) {
            let stmt = self.parse_class()?;
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
    /// statement-introducing keyword (`class`, `let`, `return`).
    fn synchronize(&mut self) {
        loop {
            match self.peek() {
                Token::Eof | Token::RBrace => return,
                Token::Newline | Token::Semicolon => {
                    self.advance();
                    return;
                }
                Token::Class | Token::Let | Token::Const | Token::Return | Token::Import => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Parses a small (single-line) statement: `let`, `return`, or an
    /// expression statement.
    ///
    /// # Errors
    ///
    /// Propagates any error from the underlying statement parser.
    fn parse_small_statement(&mut self) -> ParserResult<Statement> {
        match self.peek() {
            Token::Let => self.parse_binding(BindingKind::Let),
            Token::Const => self.parse_binding(BindingKind::Const),
            Token::Return => self.parse_return(),
            Token::For => self.parse_for(),
            Token::Throw => self.parse_throw(),
            Token::Try => self.parse_try(),
            Token::Import => self.parse_import(),
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

    /// Parses `for (binding in iter) { body }` into a [`Statement::For`]
    /// (ADR-0035 §2, iteration.md §2, U-ITER specification §1.1).
    ///
    /// Unlike [`Self::parse_while`], which desugars to a `whileTrue` send at
    /// parse time, `for` is kept as a dedicated node: the compiler lowers it to
    /// an inlined cursor `while` over the `iterate(_)` / `iteratorValue(_)`
    /// protocol, which cannot be expressed as a single sacred send. `in` is a
    /// **contextual keyword** (DEC-ITER-B): its [`Token::In`] is consumed only
    /// here, so an identifier `in` elsewhere keeps working.
    ///
    /// # Errors
    ///
    /// Returns an error if the parentheses, the loop-variable identifier, the
    /// `in` separator, the iterable expression, or the brace body is malformed.
    fn parse_for(&mut self) -> ParserResult<Statement> {
        let start = self.cur_start();
        self.advance(); // 'for'
        self.expect(&Token::LParen, &["\"(\""])?;
        let binding = self.expect_identifier(&["loop variable"])?;
        self.expect(&Token::In, &["\"in\""])?;
        let iter = self.parse_expr()?;
        self.expect(&Token::RParen, &["\")\""])?;
        let body = match self.parse_brace_block()? {
            Expr::Block(block) => block.body,
            // `parse_brace_block` always yields an `Expr::Block`.
            _ => unreachable!("parse_brace_block must produce a block"),
        };
        let range = (start..self.prev_end).into();
        Ok(Statement::For(ForStatement { binding, iter, body, range }))
    }

    /// Parses `throw expr` — surface sugar for `expr.raise()`
    /// ([error-handling.md §1](../../../docs/spec/v0.2/error-handling.md),
    /// [ADR-0031](../../../docs/adr/accepted/0031-error-handling-surface-syntax.md) §1).
    /// The non-`Error`-literal compile check is the compiler's job
    /// (`phalcom-core/src/compiler/lib.rs`), not the parser's — this only
    /// builds the node.
    ///
    /// # Errors
    ///
    /// Returns an error if the thrown expression is malformed.
    fn parse_throw(&mut self) -> ParserResult<Statement> {
        let start = self.cur_start();
        self.advance(); // 'throw'
        let expr = self.parse_expr()?;
        let range = (start..self.prev_end).into();
        Ok(Statement::Throw { expr, range })
    }

    /// Parses `import "path" as Name` (U15, DEC-U15 A+A).
    ///
    /// `as` is [`Token::As`], a reserved word (unlike `extends`/`on`'s
    /// contextual-keyword precedent, `as` was already lexed as its own
    /// token, unused until this unit). Grammar: `import` STRING `as` IDENT.
    /// The binding is mandatory in Draft 0.1 (whole-module binding only,
    /// DEC-U15); there is no bare `import "path"` and no selective `from`
    /// form yet.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is not a string literal, `as` is
    /// missing, or the binding name is not an identifier.
    fn parse_import(&mut self) -> ParserResult<Statement> {
        let start = self.cur_start();
        self.advance(); // 'import'
        let path = match self.peek().clone() {
            Token::String(value) => {
                self.advance();
                value
            }
            _ => return Err(self.error_here(strs(&["string literal"]))),
        };
        self.expect(&Token::As, &["\"as\""])?;
        let binding = self.expect_identifier(&["identifier"])?;
        let range = (start..self.prev_end).into();
        Ok(Statement::Import(ImportStatement { path, binding, range }))
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
                args: vec![Argument {
                    label: None,
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
            args: vec![
                Argument {
                    label: None,
                    expr: class,
                    range: class_range,
                },
                Argument {
                    label: None,
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
            params: vec![param],
            body,
            expr_body: false,
            range,
        })))
    }

    /// Parses a `let`/`var` binding: `<kw> pattern (= expr)?`.
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
        let value = if self.eat(&Token::Equal) { Some(self.parse_expr()?) } else { None };
        let range = (start..self.prev_end).into();
        Ok(Statement::Let(LetBinding { kind, pattern, value, range }))
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
    fn parse_pattern(&mut self) -> ParserResult<Pattern> {
        match self.peek() {
            Token::LParen => self.parse_tuple_pattern(),
            Token::LBracket => self.parse_list_pattern(),
            _ => {
                let start = self.cur_start();
                let name = self.expect_identifier(&["identifier", "\"(\"", "\"[\""])?;
                let range = (start..self.prev_end).into();
                Ok(Pattern::Name { name, range })
            }
        }
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
    /// trailing rest sub-pattern (U9's `*name` spelling reused verbatim,
    /// messages-and-selectors.md §5 spread parity).
    ///
    /// A rest sub-pattern must be the list pattern's **last** element — the
    /// same rule [`Self::parse_param_list`] enforces for a variadic
    /// parameter — so the two `*` spellings never diverge.
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
            Token::Number(_)
                | Token::String(_)
                | Token::StringInterp(_)
                | Token::True
                | Token::False
                | Token::Identifier(_)
                | Token::NameSymbol(_)
                | Token::SelectorSymbol { .. }
                | Token::SelfKw
                | Token::Super
                | Token::LParen
                | Token::Not
                | Token::Minus
                | Token::LBrace
                | Token::If
                | Token::While
        )
    }

    /// Consumes an identifier token, returning its name.
    ///
    /// # Errors
    ///
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
    fn parse_class(&mut self) -> ParserResult<Statement> {
        let mut header_attrs = Vec::new();
        while matches!(self.peek(), Token::At) {
            header_attrs.push(self.parse_attribute()?);
            self.skip_newlines();
        }
        let start = self.cur_start();
        self.expect(&Token::Class, &["\"class\""])?;
        let name_start = self.cur_start();
        let name = self.expect_identifier(&["identifier"])?;
        let name_range = (name_start..self.prev_end).into();

        // Contextual `extends` (DEC-INH-A, U-INH): `extends` is not a reserved
        // word — it is recognised as a keyword only here, immediately after the
        // class name. Any other occurrence of `extends` remains an ordinary
        // identifier. Grammar: `class` IDENT (`extends` IDENT)? `{` … `}`.
        let superclass = if matches!(self.peek(), Token::Identifier(kw) if kw == "extends") {
            self.advance(); // 'extends'
            let sc_start = self.cur_start();
            let sc_name = self.expect_identifier(&["superclass name"])?;
            let sc_range = (sc_start..self.prev_end).into();
            Some(SuperclassRef {
                name: sc_name,
                range: sc_range,
            })
        } else {
            None
        };

        self.expect(&Token::LBrace, &["\"{\""])?;
        let (members, body_attrs, invariants) = self.parse_class_body()?;
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
            superclass,
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
            // U-ANNOT-LAYOUT §3.4: `@variant Name(labels...)` is a distinct
            // grammar production (no body, bare-colon label list) from every
            // other class member — a pending `@variant` diverts to
            // `parse_variant_decl` instead of the ordinary member parser,
            // mirroring how `@invariant` above diverts to `class_invariants`.
            // Any other attributes preceding `@variant` (unusual, but not
            // forbidden by the grammar) are attached to the resulting
            // `VariantDef` the same way `attach_attrs` would.
            let mut member = if pending_attrs.iter().any(|a| a.name == "variant") {
                self.parse_variant_decl(std::mem::take(&mut pending_attrs))?
            } else {
                self.parse_class_member()?
            };
            if !pending_attrs.is_empty() {
                self.attach_attrs(&mut member, std::mem::take(&mut pending_attrs))?;
            }
            members.push(member);
        }
        Ok((members, class_attributes, class_invariants))
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

    /// Parses a declared field (`let`/`var _name [= expr]`) at class-body
    /// position (U-ANNOT-LAYOUT §3.1).
    ///
    /// `start` is the position of the already-peeked `let`/`var` token
    /// (captured by the caller before dispatching here, matching the sibling
    /// member-parsing methods' convention). Terminated by a newline, or — with
    /// no explicit terminator consumed — the closing `}`/end-of-file, mirroring
    /// how every other [`ClassMember`] variant needs no trailing separator of
    /// its own (a method/getter/setter/constructor's `{ … }` body is
    /// self-delimiting; a field declaration has no braces, so a newline plays
    /// that role instead).
    ///
    /// # Errors
    ///
    /// Returns an error if the field name is missing, the initializer
    /// expression is malformed, or the declaration is not followed by a
    /// newline, `}`, or end-of-file.
    /// Parses a field declaration: `const`-prefixed (immutable) or bare
    /// (mutable) — ADR-0064 §3, L-2. `is_const` records whether the caller
    /// already consumed a leading `Token::Const`; a bare field has no keyword
    /// to consume at all, so [`Self::parse_class_member`] dispatches here
    /// without advancing past anything first.
    ///
    /// L-8: the field name must start with `_` followed by a letter — this is
    /// enforced here, not in the lexer or `parse_primary`, so the rule stays
    /// scoped to field-declaration position (bare `_` remains a legal binding
    /// name elsewhere).
    ///
    /// # Errors
    ///
    /// Returns an error if the field name is missing, does not start with
    /// `_` + a letter, the initializer expression is malformed, or the
    /// declaration is not followed by a newline, `}`, or end-of-file.
    fn parse_field_decl(&mut self, start: usize, is_const: bool) -> ParserResult<ClassMember> {
        let name_start = self.cur_start();
        let name = self.expect_identifier(&["field name"])?;
        let mut chars = name.chars();
        let ok = chars.next() == Some('_') && chars.next().is_some_and(|c| c.is_alphabetic());
        if !ok {
            return Err(SyntaxError {
                kind: SyntaxErrorKind::Message(format!("field name '{name}' must start with `_` followed by a letter")),
                range: name_start..self.prev_end,
            });
        }
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
            mutable: !is_const,
            is_static: false,
            default,
            attributes: Vec::new(),
            range,
        }))
    }

    /// Parses a `@variant Name(label1:, label2:, ...)` declaration
    /// (U-ANNOT-LAYOUT §3.4, `annotations-data.md` §"`@variant`") — a
    /// distinct grammar production from every other [`ClassMember`]: a
    /// capitalized name followed by a parenthesized, comma-separated list of
    /// bare `label:` tokens (no values, no types, no body). `pending` is
    /// every attribute collected before this declaration (in practice always
    /// exactly `[@variant]`, per `parse_class_body`'s dispatch) — attached
    /// verbatim to the returned [`VariantDef`].
    ///
    /// Terminated the same way [`Self::parse_field_decl`] is: a newline, or —
    /// with no explicit terminator consumed — the closing `}`/end-of-file.
    ///
    /// # Errors
    ///
    /// Returns an error if the variant name or any label is missing, a label
    /// is not followed by `:`, the argument list is unterminated, or the
    /// declaration is not followed by a newline, `}`, or end-of-file.
    fn parse_variant_decl(&mut self, pending: Vec<Attribute>) -> ParserResult<ClassMember> {
        let start = self.cur_start();
        let name = self.expect_identifier(&["variant name"])?;
        self.expect(&Token::LParen, &["\"(\""])?;
        let mut labels = Vec::new();
        if !matches!(self.peek(), Token::RParen) {
            loop {
                let label = self.expect_identifier(&["variant label"])?;
                self.expect(&Token::Colon, &["\":\""])?;
                labels.push(label);
                if !self.eat(&Token::Comma) {
                    break;
                }
            }
        }
        self.expect(&Token::RParen, &["\")\""])?;
        let range = (start..self.prev_end).into();
        match self.peek() {
            Token::Newline => {
                self.advance();
            }
            Token::RBrace | Token::Eof => {}
            _ => return Err(self.error_here(strs(&["newline", "\"}\""]))),
        }
        Ok(ClassMember::Variant(VariantDef {
            name,
            labels,
            attributes: pending,
            range,
        }))
    }

    /// Parses a single class member: a method, getter, or setter.
    ///
    /// A trailing `=` after the name marks a setter (whose parameter is always
    /// named `value`); an explicit parameter list marks a method; neither marks
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
        // a leading-`_` identifier not followed by `(`, `=>`, or `{` is the
        // bare mutable field form. Everything else falls through to the
        // ordinary method/getter/setter path below.
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
        if let Token::Identifier(name) = self.peek() {
            if name.starts_with('_') && matches!(self.peek_next(), Token::Newline | Token::RBrace | Token::Eof | Token::Equal) {
                return self.parse_field_decl(start, false);
            }
        }
        // U-INDEX (ADR-0060): a bracket subscript method (`[idx] { ... }` /
        // `[idx, put:] { ... }`) is a distinct grammar production, not
        // routed through `parse_method_name` — there is no separate name
        // token at all, the brackets themselves are the whole of this
        // member's identity. Must be checked before the `Construct`/name
        // branches below, which never expect a leading `[`.
        if matches!(self.peek(), Token::LBracket) {
            return self.parse_index_member(start);
        }
        if self.eat(&Token::Construct) {
            let name_start = self.cur_start();
            let name = self.parse_method_name()?;
            let name_range = (name_start..self.prev_end).into();
            self.expect(&Token::LParen, &["\"(\""])?;
            let params = self.parse_param_list()?;
            self.expect(&Token::RParen, &["\")\""])?;
            let body = self.parse_method_block()?;
            let range = (start..self.prev_end).into();
            return Ok(ClassMember::Method(MethodDef {
                name,
                params,
                body,
                is_static: false,
                is_constructor: true,
                attributes: Vec::new(),
                range,
                name_range,
            }));
        }
        // `class name(...)` is retained as a migration spelling for
        // `@class name(...)`; token `class` remains an expression keyword in
        // primary position, so this branch is unambiguous inside a class.
        let is_static = self.eat(&Token::Static) || self.eat(&Token::Class);
        let name_start = self.cur_start();
        let name = self.parse_method_name()?;
        let name_range = (name_start..self.prev_end).into();
        let has_equal = self.eat(&Token::Equal);
        if has_equal && name.starts_with('_') {
            let expr = self.parse_expr()?;
            self.expect(&Token::Newline, &["newline"])?;
            let range = (start..self.prev_end).into();
            return Ok(ClassMember::Getter(GetterDef {
                name,
                body: vec![Statement::Expr { expr, range }],
                is_static,
                attributes: Vec::new(),
                range,
                name_range,
            }));
        }
        let params = if self.eat(&Token::LParen) {
            let list = self.parse_param_list()?;
            self.expect(&Token::RParen, &["\")\""])?;
            Some(list)
        } else {
            None
        };
        let body = self.parse_method_block()?;
        let range = (start..self.prev_end).into();
        if has_equal {
            let param = if let Some(ref list) = params {
                if !list.is_empty() { list[0].name.clone() } else { "value".to_string() }
            } else {
                "value".to_string()
            };
            Ok(ClassMember::Setter(SetterDef {
                name,
                param,
                body,
                is_static,
                attributes: Vec::new(),
                range,
                name_range,
            }))
        } else if let Some(params) = params {
            Ok(ClassMember::Method(MethodDef {
                name,
                params,
                body,
                is_static,
                is_constructor: false,
                attributes: Vec::new(),
                range,
                name_range,
            }))
        } else {
            Ok(ClassMember::Getter(GetterDef {
                name,
                body,
                is_static,
                attributes: Vec::new(),
                range,
                name_range,
            }))
        }
    }

    /// Parses a bracket subscript method — `[idx] { ... }` / `[idx, put:] {
    /// ... }` / `[] { ... }` / `[put:] { ... }` (U-INDEX,
    /// [ADR-0060](../../docs/adr/accepted/0060-index-operator-as-real-selector.md)).
    ///
    /// `start` is the position of the already-peeked `[` (captured by the
    /// caller before dispatching here, matching the sibling member-parsing
    /// methods' convention). Reuses [`Parser::parse_param_list`] verbatim for
    /// the bracketed parameter list — the *only* structural difference from
    /// an ordinary `(params)` method is the delimiter, so `[idx, put:]`
    /// parses exactly like `(idx, put:)` would, just bracket-closed instead
    /// of paren-closed.
    ///
    /// # Errors
    ///
    /// Returns an error if the parameter list or body is malformed, or the
    /// closing `]` is missing.
    fn parse_index_member(&mut self, start: usize) -> ParserResult<ClassMember> {
        let name_start = self.cur_start();
        self.expect(&Token::LBracket, &["\"[\""])?;
        let params = self.parse_param_list()?;
        self.expect(&Token::RBracket, &["\"]\""])?;
        let name_range = (name_start..self.prev_end).into();
        let body = self.parse_method_block()?;
        let range = (start..self.prev_end).into();
        Ok(ClassMember::Index(IndexMethodDef {
            params,
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
            Token::Identifier(n) => n.clone(),
            Token::Plus => "+".to_string(),
            Token::Minus => "-".to_string(),
            Token::Asterisk => "*".to_string(),
            Token::Slash => "/".to_string(),
            Token::Percent => "%".to_string(),
            Token::EqualEqual => "==".to_string(),
            Token::BangEqual => "!=".to_string(),
            Token::Less => "<".to_string(),
            Token::LessEqual => "<=".to_string(),
            Token::Greater => ">".to_string(),
            Token::GreaterEqual => ">=".to_string(),
            Token::And => "and".to_string(),
            Token::Or => "or".to_string(),
            Token::Is => "is".to_string(),
            _ => return Err(self.error_here(strs(&["identifier", "operator"]))),
        };
        self.advance();
        if name == "new_" {
            return Err(self.error_here(strs(&["method name (new_ is reserved for native class allocator)"])));
        }
        Ok(name)
    }

    /// Parses a parenthesized parameter list: comma-separated identifiers,
    /// each optionally a labeled parameter (`name:`) or, at most once and only
    /// as the final entry, a rest parameter (`*name`, U9,
    /// `messages-and-selectors.md` §4).
    ///
    /// Shared by method and constructor parameter lists, and — since
    /// U-INDEX/ADR-0060 substitutes `[`/`]` for `(`/`)` — bracket subscript
    /// method parameter lists too (block-literal parameters are parsed by a
    /// separate ad hoc scanner in [`Parser::parse_primary`] and never reach
    /// this function, so no block-literal guard is needed here).
    ///
    /// # Errors
    ///
    /// Returns a [`SyntaxErrorKind::Message`] diagnostic (not a panic) if a
    /// rest parameter is not the list's last entry, carries a label, or
    /// follows an already-labeled parameter (mixing keyword and rest
    /// parameters would produce a selector that call sites could never
    /// exactly match, since [`crate`]-side selector encoding for a variadic
    /// method ignores labels entirely — U9 corrections §0 point 3).
    fn parse_param_list(&mut self) -> ParserResult<Vec<ParameterDef>> {
        if matches!(self.peek(), Token::RParen | Token::RBracket) {
            return Ok(Vec::new());
        }
        let mut params: Vec<ParameterDef> = Vec::new();
        let mut any_labeled = false;
        loop {
            let start = self.cur_start();
            if params.last().is_some_and(|p| p.is_rest) {
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::Message("a rest parameter (\"*name\") must be the last parameter".to_string()),
                    range: start..start,
                });
            }
            let is_rest = self.eat(&Token::Asterisk);
            let name = self.expect_identifier(&["identifier"])?;
            let label = if self.eat(&Token::Colon) { Some(name.clone()) } else { None };
            if is_rest {
                if label.is_some() {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message("a rest parameter (\"*name\") cannot have a label".to_string()),
                        range: start..self.prev_end,
                    });
                }
                if any_labeled {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message("a rest parameter cannot follow a labeled parameter".to_string()),
                        range: start..self.prev_end,
                    });
                }
            }
            any_labeled |= label.is_some();
            let range = (start..self.prev_end).into();
            params.push(ParameterDef { name, label, is_rest, range });
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        Ok(params)
    }

    /// Parses a method body: either `=> expr` (single expression) or a
    /// `{ statements }` block.
    ///
    /// The `=>` form requires a trailing newline, matching the LALRPOP
    /// `MethodBlock` rule.
    ///
    /// # Errors
    ///
    /// Returns an error if neither body form is present or the body is
    /// malformed.
    fn parse_method_block(&mut self) -> ParserResult<Vec<Statement>> {
        match self.peek() {
            Token::FatArrow => {
                self.advance();
                let stmt = self.parse_expr_statement()?;
                self.expect(&Token::Newline, &["newline"])?;
                Ok(vec![stmt])
            }
            Token::LBrace => {
                self.advance();
                let stmts = self.parse_block_statements()?;
                self.expect(&Token::RBrace, &["\"}\""])?;
                Ok(stmts)
            }
            _ => Err(self.error_here(strs(&["\"=>\"", "\"{\""]))),
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
                Token::At | Token::Class if matches!(self.peek(), Token::At) || matches!(self.peek_next(), Token::Identifier(_)) => {
                    let stmt = self.parse_class()?;
                    // `range.start` is the `class` keyword's own byte offset
                    // (set in `parse_class` before any header attribute
                    // affects it — attributes are consumed first, then
                    // `start = self.cur_start()` right before `Token::Class`
                    // is expected), so the span points at the keyword even
                    // when the nested class carries `@` attributes.
                    let Statement::Class(class_def) = &stmt else {
                        unreachable!("parse_class always returns Statement::Class")
                    };
                    let keyword_start = class_def.range.start;
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message(
                            "class.nested_declaration: class declarations are only allowed at a module's top level, not nested inside a block".to_string(),
                        ),
                        range: keyword_start..keyword_start + "class".len(),
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
            self.advance();
            let value = self.parse_assignment()?;
            let range = (start..self.prev_end).into();
            let binary = Expr::Binary(Box::new(BinaryExpr {
                op,
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
                    value,
                    range,
                })),
                Expr::Index(ix) => Expr::SetIndex(Box::new(SetIndexExpr {
                    object: ix.object,
                    args: ix.args,
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
                args: vec![Argument {
                    label: None,
                    expr: block,
                    range,
                }],
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
                        range: outer_range,
                    }))
                }
            };

            acc = Some(match acc {
                None => part,
                Some(left) => Expr::Binary(Box::new(BinaryExpr {
                    op: BinaryOp::Add,
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
            params: Vec::new(),
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
            params: vec![param],
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
        let mut left = self.parse_unary()?;
        loop {
            // `is`/`is!`/`is not`/`is! not` sit at the equality tier (prec 3)
            // but are not a `binary_op` entry — they carry affixes (a
            // contiguous `!` for strict, a `not` particle for negation) and
            // are non-chaining, so [`Parser::parse_is`] handles the whole
            // suffix itself. Gated to `min_prec <= 3` so a nested RHS
            // (`parse_binary(4)`, comparison-tier-and-above) never re-enters
            // this arm, keeping `is` from chaining through recursion.
            if min_prec <= 3 && matches!(self.peek(), Token::Is) {
                left = self.parse_is(left, start)?;
                continue;
            }
            let Some((prec, op)) = binary_op(self.peek()) else {
                break;
            };
            if prec < min_prec {
                break;
            }
            self.advance();
            let right = self.parse_binary(prec + 1)?;
            let range = (start..self.prev_end).into();
            left = Expr::Binary(Box::new(BinaryExpr { op, left, right, range }));
        }
        Ok(left)
    }

    /// Parses the `is` type-test operator suite (`is`, `is!`, `is not`,
    /// `is! not`) following `left`, desugaring into existing AST nodes —
    /// no dedicated `is` node exists.
    ///
    /// Per [is-tests.md](../../../docs/spec/v0.2/next/is-tests.md):
    /// - `x is T` desugars to the send `x.is(T)` (subclass-inclusive).
    /// - `x is! T` (a `!` **contiguous** with `is`, i.e. no whitespace
    ///   between — the same adjacency test `selectors.md §2` uses for
    ///   `#move`) desugars to `x.isExactly(T)` (live direct-class identity).
    /// - A `not` particle immediately after `is`/`is!` is **always** the
    ///   negation particle (Python's `is not` rule: it is consumed greedily
    ///   here and is never parsed as a prefix on the RHS), wrapping the base
    ///   send in `Expr::Unary(UnaryOp::Not)`.
    /// - `is` is **non-chaining**: the desugared result is `Bool`, so a
    ///   second `is` immediately following is a compile error rather than a
    ///   silently-accepted `(x is T) is U`.
    ///
    /// The RHS is parsed at the comparison tier and above
    /// (`parse_binary(4)`), matching the equality tier `is` itself occupies.
    ///
    /// # Errors
    ///
    /// Propagates any error from the RHS class expression, or returns a
    /// [`SyntaxError`] if another `is` immediately follows the result.
    fn parse_is(&mut self, left: Expr, start: usize) -> ParserResult<Expr> {
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

        let rhs = self.parse_binary(4)?;
        let range = (start..self.prev_end).into();
        let method = if strict { "isExactly" } else { "is" }.to_string();
        let base = Expr::MethodCall(Box::new(MethodCallExpr {
            object: left,
            method,
            args: vec![Argument { label: None, expr: rhs, range }],
            range,
        }));
        let result = if negate {
            Expr::Unary(Box::new(UnaryExpr {
                op: UnaryOp::Not,
                expr: base,
                range,
            }))
        } else {
            base
        };

        if matches!(self.peek(), Token::Is) {
            return Err(self.error_here(strs(&["an expression (chained `is` is not allowed — the result of `is` is a `Bool`)"])));
        }

        Ok(result)
    }

    /// Parses a prefix unary expression (`-x`, `not x`), or delegates to
    /// [`Parser::parse_call`].
    ///
    /// `not` is the sole boolean-negation prefix (`syntax/grammar.md`'s
    /// `unary := ( "-" | "not" ) unary`, `syntax/expressions.md` precedence
    /// table row 9). U-NEG retires prefix `!` (`Token::Bang`) as an
    /// expression operator — `Token::Bang` now survives only inside the
    /// lexer's `!=` (`Token::BangEqual`) disambiguation; a bare `!` in
    /// expression position is a parse error.
    ///
    /// # Errors
    ///
    /// Propagates any error from the operand expression.
    fn parse_unary(&mut self) -> ParserResult<Expr> {
        let op = match self.peek() {
            Token::Minus => UnaryOp::Negate,
            Token::Not => UnaryOp::Not,
            _ => return self.parse_call(),
        };
        let start = self.cur_start();
        self.advance();
        let expr = self.parse_unary()?;
        let range = (start..self.prev_end).into();
        Ok(Expr::Unary(Box::new(UnaryExpr { op, expr, range })))
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
        while matches!(
            self.peek(),
            Token::Dot | Token::QuestionDot | Token::LParen | Token::LBrace | Token::ColonColon | Token::LBracket
        ) {
            if self.eat(&Token::QuestionDot) {
                expr = self.parse_optional_send(expr, start)?;
            } else if self.eat(&Token::ColonColon) {
                // `::` method reference (selectors.md §3, U16-Open +
                // U16-Pinned). Ambiguity rule (LOCKED): peek the token right
                // after `::` —
                //   - a selector-form symbol (`#name(...)`) -> Pinned, the
                //     full identity is pinned at this reference site;
                //   - a bare name-form symbol (`#name`, no parens) -> reject:
                //     Pinned is callable-only-by-full-identity (Q14), there is
                //     no "pinned base name" shape;
                //   - an identifier/keyword -> Open (unchanged), the selector
                //     is rebuilt from labels at call time.
                // Uniform for both `obj::` and `Type::` — the receiver
                // expression is whatever postfix chain preceded `::`.
                let kind = match self.peek().clone() {
                    Token::SelectorSymbol { name, labels } => {
                        self.advance();
                        MethodRefKind::Pinned { name, labels }
                    }
                    Token::NameSymbol(_) => {
                        return Err(self.error_here(strs(&[
                            "a selector form `::#name(...)` (a pinned method reference requires the full selector; use `::name` for an open reference)",
                        ])));
                    }
                    _ => {
                        let name = self.parse_property_name()?;
                        MethodRefKind::Open { name }
                    }
                };
                let range = (start..self.prev_end).into();
                expr = Expr::MethodRef(Box::new(MethodRefExpr { receiver: expr, kind, range }));
            } else if self.eat(&Token::Dot) {
                let property = self.parse_property_name()?;
                if self.eat(&Token::LParen) {
                    let args = self.parse_arg_list()?;
                    self.expect(&Token::RParen, &["\")\""])?;
                    let range = (start..self.prev_end).into();
                    expr = Expr::MethodCall(Box::new(MethodCallExpr {
                        object: expr,
                        method: property,
                        args,
                        range,
                    }));
                } else {
                    let range = (start..self.prev_end).into();
                    expr = Expr::GetProperty(Box::new(GetPropertyExpr { object: expr, property, range }));
                }
            } else if self.eat(&Token::LParen) {
                let args = self.parse_arg_list()?;
                self.expect(&Token::RParen, &["\")\""])?;
                let range = (start..self.prev_end).into();
                expr = Expr::MethodCall(Box::new(MethodCallExpr {
                    object: expr,
                    method: "call".to_string(),
                    args,
                    range,
                }));
            } else if self.eat(&Token::LBracket) {
                // U-INDEX (ADR-0060): the bracket's contents are a full
                // call-shaped argument list — positional + `label:`,
                // identical grammar to `(...)` call args (`xs[i, j]`,
                // `cache[key, default: fallback]`), not a single expression.
                // Reuses `parse_arg_list` verbatim, which already
                // short-circuits on an immediately-closing delimiter
                // (`xs[]`, zero-arity).
                let args = self.parse_arg_list()?;
                self.expect(&Token::RBracket, &["\"]\""])?;
                let range = (start..self.prev_end).into();
                expr = Expr::Index(Box::new(IndexExpr { object: expr, args, range }));
            } else if matches!(self.peek(), Token::LBrace) {
                let block = self.parse_primary()?;
                let range = (start..self.prev_end).into();
                let arg = Argument {
                    label: None,
                    expr: block,
                    range: (self.prev_end..self.prev_end).into(),
                };
                match expr {
                    Expr::MethodCall(mut mc) => {
                        mc.args.push(arg);
                        mc.range = range;
                        expr = Expr::MethodCall(mc);
                    }
                    Expr::GetProperty(gp) => {
                        expr = Expr::MethodCall(Box::new(MethodCallExpr {
                            object: gp.object,
                            method: gp.property,
                            args: vec![arg],
                            range,
                        }));
                    }
                    _ => {
                        expr = Expr::MethodCall(Box::new(MethodCallExpr {
                            object: expr,
                            method: "call".to_string(),
                            args: vec![arg],
                            range,
                        }));
                    }
                }
            }
        }
        Ok(expr)
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
                range,
            }))
        };
        let range = (start..self.prev_end).into();
        let mapper = self.wrap_block_mapper(recv_name, inner, range);
        Ok(Expr::MethodCall(Box::new(MethodCallExpr {
            object,
            method: "map".to_string(),
            args: vec![Argument {
                label: None,
                expr: mapper,
                range,
            }],
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
        match self.peek().clone() {
            Token::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            Token::Class => {
                self.advance();
                Ok("class".to_string())
            }
            // `try` is a genuine reserved keyword (statement-leading, ADR-0031
            // §4) but must still resolve as an ordinary selector in message
            // position — `fiber.try(...)`/`fiber.try` (`Fiber#try`, ADR-0030)
            // predates this unit and must keep parsing.
            Token::Try => {
                self.advance();
                Ok("try".to_string())
            }
            Token::Plus => {
                self.advance();
                Ok("+".to_string())
            }
            Token::Minus => {
                self.advance();
                Ok("-".to_string())
            }
            Token::Asterisk => {
                self.advance();
                Ok("*".to_string())
            }
            Token::Slash => {
                self.advance();
                Ok("/".to_string())
            }
            Token::Percent => {
                self.advance();
                Ok("%".to_string())
            }
            Token::EqualEqual => {
                self.advance();
                Ok("==".to_string())
            }
            Token::BangEqual => {
                self.advance();
                Ok("!=".to_string())
            }
            Token::Less => {
                self.advance();
                Ok("<".to_string())
            }
            Token::LessEqual => {
                self.advance();
                Ok("<=".to_string())
            }
            Token::Greater => {
                self.advance();
                Ok(">".to_string())
            }
            Token::GreaterEqual => {
                self.advance();
                Ok(">=".to_string())
            }
            Token::And => {
                self.advance();
                Ok("and".to_string())
            }
            Token::Or => {
                self.advance();
                Ok("or".to_string())
            }
            Token::Is => {
                self.advance();
                Ok("is".to_string())
            }
            _ => Err(self.error_here(strs(&["identifier", "\"class\"", "operator"]))),
        }
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
            params: Vec::new(),
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
            params: Vec::new(),
            body: vec![Statement::Expr { expr, range }],
            expr_body: true,
            range,
        }))
    }

    /// Parses `if (cond) { ... } (else (if ... | { ... }))?` as sacred-selector
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
        self.expect(&Token::LParen, &["\"(\""])?;
        let cond = self.parse_expr()?;
        self.expect(&Token::RParen, &["\")\""])?;
        let then_arm = self.parse_brace_block()?;
        let then_range = then_arm.range();

        let mut args = vec![Argument {
            label: None,
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
            args.push(Argument {
                label: Some("ifFalse".to_string()),
                expr: else_arm,
                range: else_range,
            });
        }

        let range = (start..self.prev_end).into();
        Ok(Expr::MethodCall(Box::new(MethodCallExpr {
            object: cond,
            method: "ifTrue".to_string(),
            args,
            range,
        })))
    }

    /// Parses `while (cond) { body }` as the sacred loop send
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
        self.expect(&Token::LParen, &["\"(\""])?;
        let cond = self.parse_expr()?;
        self.expect(&Token::RParen, &["\")\""])?;
        let cond_block = Self::wrap_expr_as_block(cond);
        let body = self.parse_brace_block()?;
        let body_range = body.range();
        let range = (start..self.prev_end).into();
        Ok(Expr::MethodCall(Box::new(MethodCallExpr {
            object: cond_block,
            method: "whileTrue".to_string(),
            args: vec![Argument {
                label: None,
                expr: body,
                range: body_range,
            }],
            range,
        })))
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
            Token::Number(value) => {
                self.advance();
                Ok(Expr::Number { value, range })
            }
            Token::String(value) => {
                self.advance();
                Ok(Expr::String { value, range })
            }
            Token::StringInterp(segments) => {
                self.advance();
                self.desugar_string_interp(segments, range)
            }
            Token::NameSymbol(name) => {
                self.advance();
                Ok(Expr::Symbol(Box::new(SymbolExpr {
                    kind: SymbolLiteralKind::Name(name),
                    range,
                })))
            }
            Token::SelectorSymbol { name, labels } => {
                self.advance();
                Ok(Expr::Symbol(Box::new(SymbolExpr {
                    kind: SymbolLiteralKind::Selector { name, labels },
                    range,
                })))
            }
            Token::Identifier(value) => {
                if matches!(self.peek_next(), Token::FatArrow) {
                    let start = self.cur_start();
                    self.advance(); // identifier
                    self.advance(); // =>
                    let body_expr = self.parse_expr()?;
                    let range = (start..self.prev_end).into();
                    let stmt = Statement::Expr { expr: body_expr, range };
                    return Ok(Expr::Block(Box::new(BlockExpr {
                        params: vec![value],
                        body: vec![stmt],
                        expr_body: true,
                        range,
                    })));
                }
                self.advance();
                if value.starts_with('_') && value != "_" {
                    Ok(Expr::Field { value, range })
                } else {
                    Ok(Expr::Var { value, range })
                }
            }
            Token::SelfKw => {
                self.advance();
                Ok(Expr::SelfVar { range })
            }
            Token::Class => {
                self.advance();
                Ok(Expr::GetProperty(Box::new(GetPropertyExpr {
                    object: Expr::SelfVar { range },
                    property: "class".to_string(),
                    range,
                })))
            }
            Token::Super => {
                self.advance();
                Ok(Expr::SuperVar { range })
            }
            Token::LBracket => self.parse_list_literal(),
            Token::LParen => self.parse_paren_or_tuple(),
            Token::LBrace => {
                let start = self.cur_start();
                self.advance(); // '{'

                // Spec §6 one-token brace disambiguation: `{ IDENT : … }` is a
                // map literal; every other `{`-form stays a block and is
                // handled by the param-scan below (ADR-0032 §3.1).
                if matches!(self.peek(), Token::Identifier(_)) && matches!(self.peek_next(), Token::Colon) {
                    return self.parse_map_literal(start);
                }

                let mut params = Vec::new();
                let mut has_arrow = false;

                let mut scan_idx = self.pos;
                loop {
                    if scan_idx >= self.tokens.len() {
                        break;
                    }
                    match &self.tokens[scan_idx].token {
                        Token::Identifier(_) => {
                            scan_idx += 1;
                            if scan_idx < self.tokens.len() {
                                if matches!(self.tokens[scan_idx].token, Token::Comma) {
                                    scan_idx += 1;
                                } else if matches!(self.tokens[scan_idx].token, Token::FatArrow) {
                                    has_arrow = true;
                                    break;
                                } else {
                                    break;
                                }
                            }
                        }
                        Token::Comma => {
                            break;
                        }
                        Token::FatArrow => {
                            has_arrow = true;
                            break;
                        }
                        _ => {
                            break;
                        }
                    }
                }

                if has_arrow {
                    while !matches!(self.peek(), Token::FatArrow) {
                        let param = self.expect_identifier(&["parameter name"])?;
                        params.push(param);
                        if !self.eat(&Token::Comma) && !matches!(self.peek(), Token::FatArrow) {
                            return Err(self.error_here(strs(&["\",\"", "\"=>\""])));
                        }
                    }
                    self.expect(&Token::FatArrow, &["\"=>\""])?;
                }

                let body = self.parse_block_statements()?;
                self.expect(&Token::RBrace, &["\"}\""])?;
                let range = (start..self.prev_end).into();

                Ok(Expr::Block(Box::new(BlockExpr {
                    params,
                    body,
                    expr_body: false,
                    range,
                })))
            }
            _ => Err(self.error_here(primary_expected())),
        }
    }

    /// Parses a list literal `[e1, …, en]` and desugars it to a `List`
    /// construction chain (lexical-structure.md §4; [ADR-0029]; [ADR-0032] §1).
    ///
    /// Following the parser-level sugar precedent (`if`→`ifTrue`,
    /// `while`→`whileTrue`, `\(e)`→`toString`), the literal lowers to ordinary
    /// message sends on the landed kernel `List` rather than a dedicated
    /// [`Expr`] variant — no new AST node, bytecode, or floor primitive:
    ///
    /// - `[]`        → `List.new()`
    /// - `[a]`       → `List.new().add(a)`
    /// - `[a, b, c]` → `List.new().add(a).add(b).add(c)`
    ///
    /// Because `List#add(_)` returns `self`, the `.add` chain is one
    /// well-typed expression. The synthetic range spans the `[ … ]` so
    /// diagnostics point at the literal.
    ///
    /// [ADR-0029]: ../../../docs/adr/accepted/0029-list-literal-syntax.md
    /// [ADR-0032]: ../../../docs/adr/accepted/0032-collections-representation-and-literals.md
    ///
    /// # Errors
    ///
    /// Propagates any [`SyntaxError`] from an element expression, or from a
    /// missing closing `]`.
    fn parse_list_literal(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        self.advance(); // '['
        let elems = self.parse_comma_exprs(&Token::RBracket)?;
        self.expect(&Token::RBracket, &["\"]\""])?;
        let range: SourceRange = (start..self.prev_end).into();
        Ok(Self::list_construction_chain(elems, range))
    }

    /// Parses a parenthesised expression `(e)` or a tuple literal
    /// `(e1, …, en)` with n ≥ 2 (lexical-structure.md §7; [ADR-0032] §3.2).
    ///
    /// A single parenthesised expression stays *grouping* — `(x)` is `x`,
    /// never a one-tuple — matching prior behaviour and the LALRPOP grammar;
    /// only a top-level comma promotes the form to a tuple. This is
    /// unambiguous because unbraced arrows are single-parameter (spec §7), so
    /// `(` never begins a parameter list and no cover grammar is required.
    ///
    /// A tuple desugars to a `Tuple` construction send over an already-built
    /// `List` (so it depends on no variadic `construct`), reusing the list
    /// slice's construction chain as its argument. Per [ADR-0032] §1 `Tuple`
    /// is a native heap arm supplied by the collection-runtime unit
    /// (U-COLLTYPES); its runtime is therefore deferred, but the surface
    /// lowering is fixed here:
    ///
    /// - `(a, b)` → `Tuple.fromList(List.new().add(a).add(b))`
    ///
    /// [ADR-0032]: ../../../docs/adr/accepted/0032-collections-representation-and-literals.md
    ///
    /// # Errors
    ///
    /// Propagates any [`SyntaxError`] from an element expression, or from a
    /// missing closing `)`.
    fn parse_paren_or_tuple(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        self.advance(); // '('
        let first = self.parse_expr()?;
        if !matches!(self.peek(), Token::Comma) {
            // Grouping: behave exactly as the pre-U-COLL `(` arm did.
            self.expect(&Token::RParen, &["\")\""])?;
            return Ok(first);
        }
        // Tuple: consume the first comma, then gather any remaining elements.
        self.advance(); // ','
        let mut elems = vec![first];
        // A lone trailing comma `(a,)` still yields a tuple (spec §7).
        if !matches!(self.peek(), Token::RParen) {
            elems.extend(self.parse_comma_exprs(&Token::RParen)?);
        }
        self.expect(&Token::RParen, &["\")\""])?;
        let range: SourceRange = (start..self.prev_end).into();
        let list = Self::list_construction_chain(elems, range);
        Ok(Expr::MethodCall(Box::new(MethodCallExpr {
            object: Expr::Var {
                value: "Tuple".to_string(),
                range,
            },
            method: "fromList".to_string(),
            args: vec![Argument {
                label: None,
                expr: list,
                range,
            }],
            range,
        })))
    }

    /// Parses a map literal `{ k1: v1, …, kn: vn }` per the spec §6 brace
    /// disambiguation, where bare-identifier keys are symbols (`{a: 1}` ≡ key
    /// `#a`), mirroring labeled-argument parsing ([ADR-0032] §3.1).
    ///
    /// The disambiguation itself — one token of lookahead, `{ IDENT : }` ⇒ map
    /// and every other `{`-form ⇒ block — was the hard, shipped part of the
    /// surface unit (U-COLL). The *runtime* lowering to the native `Map` arm
    /// (U-COLLTYPES) now resolves each pair to a `Map#at(_, put:)` send —
    /// see [`Self::map_construction_chain`]. The empty map is spelled
    /// `Map()`, not `{}` (spec §6: `{}` is the empty block, so there is no
    /// empty-map literal).
    ///
    /// [ADR-0032]: ../../../docs/adr/accepted/0032-collections-representation-and-literals.md
    ///
    /// # Errors
    ///
    /// Propagates any [`SyntaxError`] from a key or value, or from a missing
    /// closing `}`.
    fn parse_map_literal(&mut self, start: usize) -> ParserResult<Expr> {
        let mut pairs: Vec<(Expr, Expr)> = Vec::new();
        loop {
            let key_start = self.cur_start();
            let key_name = self.expect_identifier(&["map key"])?;
            let key_range: SourceRange = (key_start..self.prev_end).into();
            // Bare-identifier key -> a Symbol, built via `Symbol.new("name")`
            // (ADR-0032 §3.1: `{a: 1}` ≡ key `#a`). There is no `#a` sigil in
            // the lexer yet (a separate, reserved-inactive surface — see
            // DEFERRED.md), so the desugar targets the existing interning
            // constructor directly rather than a literal token.
            let key = Expr::MethodCall(Box::new(MethodCallExpr {
                object: Expr::Var {
                    value: "Symbol".to_string(),
                    range: key_range,
                },
                method: "new".to_string(),
                args: vec![Argument {
                    label: None,
                    expr: Expr::String {
                        value: key_name,
                        range: key_range,
                    },
                    range: key_range,
                }],
                range: key_range,
            }));
            self.expect(&Token::Colon, &["\":\""])?;
            let value = self.parse_expr()?;
            pairs.push((key, value));
            if !self.eat(&Token::Comma) {
                break;
            }
            // Permit a trailing comma before the closing brace.
            if matches!(self.peek(), Token::RBrace) {
                break;
            }
        }
        self.expect(&Token::RBrace, &["\"}\""])?;
        let range: SourceRange = (start..self.prev_end).into();
        Ok(Self::map_construction_chain(pairs, range))
    }

    /// Folds `pairs` into a `Map` construction chain
    /// `Map.new().at(k1, put: v1)…​.at(kn, put: vn)`, all sharing the
    /// synthetic `range` — the map-literal analogue of
    /// [`Self::list_construction_chain`]. Because `Map#at(_, put:)` returns
    /// `self`, the result is a single expression; `{}`'s empty-map case does
    /// not exist (spec §6: `{}` is the empty block), so `pairs` is never
    /// empty at a real call site, but an empty `pairs` still yields the bare
    /// `Map.new()` receiver for robustness.
    fn map_construction_chain(pairs: Vec<(Expr, Expr)>, range: SourceRange) -> Expr {
        let mut acc = Expr::MethodCall(Box::new(MethodCallExpr {
            object: Expr::Var {
                value: "Map".to_string(),
                range,
            },
            method: "new".to_string(),
            args: Vec::new(),
            range,
        }));
        for (key, value) in pairs {
            let value_range = value.range();
            acc = Expr::MethodCall(Box::new(MethodCallExpr {
                object: acc,
                method: "at".to_string(),
                args: vec![
                    Argument {
                        label: None,
                        expr: key,
                        range: value_range,
                    },
                    Argument {
                        label: Some("put".to_string()),
                        expr: value,
                        range: value_range,
                    },
                ],
                range,
            }));
        }
        acc
    }

    /// Parses a comma-separated run of expressions up to (but not consuming)
    /// `terminator`, returning the elements in source order
    /// (lexical-structure.md §4). Shared by the list `[…]` and tuple `(…)`
    /// literal arms; a trailing comma before the terminator is permitted, and
    /// an immediate terminator yields an empty vector.
    ///
    /// The element grammar is kept intentionally pattern-compatible with the
    /// destructuring-pattern scanner (lexical-structure.md §8) so the
    /// concurrent U14 unit can share this helper rather than fork a parallel
    /// scanner: a leading `*` ([`Token::Asterisk`], spread) slot is *reserved*
    /// here. U-COLL ships no spread, so a spread element is rejected with a
    /// precise "not yet supported" diagnostic rather than silently
    /// mis-parsed (ADR-0032 §3.2; the U9 spread follow-on wires it later).
    ///
    /// # Errors
    ///
    /// Propagates any [`SyntaxError`] from an element expression, and rejects a
    /// leading-`*` spread element with a [`SyntaxErrorKind::Message`]
    /// "pending" diagnostic.
    fn parse_comma_exprs(&mut self, terminator: &Token) -> ParserResult<Vec<Expr>> {
        let mut elems = Vec::new();
        if self.peek() == terminator {
            return Ok(elems);
        }
        loop {
            if matches!(self.peek(), Token::Asterisk) {
                return Err(self.error_message_here("spread element (`*x`) in a collection literal is not yet supported"));
            }
            elems.push(self.parse_expr()?);
            if !self.eat(&Token::Comma) {
                break;
            }
            // Allow a trailing comma directly before the terminator.
            if self.peek() == terminator {
                break;
            }
        }
        Ok(elems)
    }

    /// Folds `elems` into a `List` construction chain
    /// `List.new().add(e1)…​.add(en)`, all sharing the synthetic `range`
    /// ([ADR-0029]). Because `List#add(_)` returns `self`, the result is a
    /// single expression; `[]`/`()` with no elements yields the bare
    /// `List.new()` receiver.
    ///
    /// [ADR-0029]: ../../../docs/adr/accepted/0029-list-literal-syntax.md
    fn list_construction_chain(elems: Vec<Expr>, range: SourceRange) -> Expr {
        let mut acc = Expr::MethodCall(Box::new(MethodCallExpr {
            object: Expr::Var {
                value: "List".to_string(),
                range,
            },
            method: "new".to_string(),
            args: Vec::new(),
            range,
        }));
        for elem in elems {
            let elem_range = elem.range();
            acc = Expr::MethodCall(Box::new(MethodCallExpr {
                object: acc,
                method: "add".to_string(),
                args: vec![Argument {
                    label: None,
                    expr: elem,
                    range: elem_range,
                }],
                range,
            }));
        }
        acc
    }

    /// Builds a free-form [`SyntaxErrorKind::Message`] diagnostic anchored at
    /// the current token's span. Used for surface features whose *syntax* is
    /// recognised but whose runtime lowering is deferred to a follow-on unit
    /// (e.g. a spread element in a collection literal; ADR-0032 §3.2).
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
    fn parse_arg_list(&mut self) -> ParserResult<Vec<Argument>> {
        if matches!(self.peek(), Token::RParen | Token::RBracket) {
            return Ok(Vec::new());
        }
        let mut args = Vec::new();
        loop {
            let start = self.cur_start();
            let is_labelled = matches!(self.peek(), Token::Identifier(_)) && matches!(self.peek_next(), Token::Colon);
            let label = if is_labelled {
                let lbl = self.expect_identifier(&["label"])?;
                self.expect(&Token::Colon, &["\":\""])?;
                Some(lbl)
            } else {
                None
            };
            let expr = self.parse_expr()?;
            let range = (start..self.prev_end).into();
            args.push(Argument { label, expr, range });
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        Ok(args)
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
        Token::BangEqual => (3, BinaryOp::NotEqual),
        Token::Less => (4, BinaryOp::LessThan),
        Token::LessEqual => (4, BinaryOp::LessThanOrEqual),
        Token::Greater => (4, BinaryOp::GreaterThan),
        Token::GreaterEqual => (4, BinaryOp::GreaterThanOrEqual),
        Token::Plus => (5, BinaryOp::Add),
        Token::Minus => (5, BinaryOp::Subtract),
        Token::Asterisk => (6, BinaryOp::Multiply),
        Token::Slash => (6, BinaryOp::Divide),
        Token::Percent => (6, BinaryOp::Modulo),
        _ => return None,
    })
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

    /// Returns the single statement of a program that must parse cleanly.
    fn only_statement(src: &str) -> Statement {
        let result = parse(src, 0);
        assert!(result.errors.is_empty(), "unexpected errors: {:?}", result.errors);
        assert_eq!(result.program.statements.len(), 1);
        result.program.statements.into_iter().next().unwrap()
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
        // `*rest` must be the list pattern's last element (U9 parity).
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
        let Expr::Block(block) = &call.args[0].expr else {
            panic!("expected a block thunk argument");
        };
        assert!(block.params.is_empty());
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
        let Expr::Block(thunk) = &outer.args[0].expr else {
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
        let Expr::Block(block) = &call.args[0].expr else {
            panic!("expected a mapper block");
        };
        assert_eq!(block.params.len(), 1);
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
        assert_eq!(value, &block.params[0]);
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
        let Expr::Block(block) = &call.args[0].expr else {
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
}
