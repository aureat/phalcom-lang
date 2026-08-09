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
    /// `as` is [`Token::As`], a reserved word (unlike `on`'s
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
            Token::Int { .. }
                | Token::Float(_)
                | Token::String(_)
                | Token::StringInterp(_)
                | Token::True
                | Token::False
                | Token::Identifier(_)
                | Token::FieldIdentifier(_)
                | Token::ImplementationFieldIdentifier(_)
                | Token::NameSymbol(_)
                | Token::SelectorSymbol { .. }
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

        // Superclass clause: `is` is Token::Is keyword (PDR-0030).
        // Grammar: `class` IDENT (`is` IDENT)? `{` … `}`.
        let superclass = if matches!(self.peek(), Token::Is) {
            self.advance(); // 'is'
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
        let name = match self.peek().clone() {
            Token::FieldIdentifier(n) | Token::ImplementationFieldIdentifier(n) => {
                self.advance();
                n
            }
            _ => return Err(self.error_here(strs(&["field name"]))),
        };
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
            && matches!(self.peek_next(), Token::Newline | Token::RBrace | Token::Eof | Token::Equal)
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

            // DEPRECATED:
            // `constructor` syntax removed in favor of `@constructor` attribute
            //
            // let name_start = self.cur_start();
            // let name = self.parse_method_name()?;
            // let name_range = (name_start..self.prev_end).into();
            // self.expect(&Token::LParen, &["\"(\""])?;
            // let params = self.parse_param_list()?;
            // self.expect(&Token::RParen, &["\")\""])?;
            // let body = self.parse_method_block()?;
            // let range = (start..self.prev_end).into();
            // return Ok(ClassMember::Method(MethodDef {
            //     name,
            //     params,
            //     body,
            //     is_static: false,
            //     is_constructor: true,
            //     attributes: Vec::new(),
            //     range,
            //     name_range,
            // }));
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
            if matches!(self.peek(), Token::Colon) {
                let err_start = self.cur_start();
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::Message(
                        "parameter declaration labels no longer use `:`; write `label local`, or `label` when the external and local names are identical"
                            .to_string(),
                    ),
                    range: start_put..err_start,
                });
            }
            let local_name = self.expect_identifier(&["parameter name"])?;
            self.expect(&Token::RParen, &["\")\""])?;
            let param = ParameterDef {
                name: local_name,
                label: None,
                rest_mode: RestMode::None,
                range: (start_put..self.prev_end).into(),
            };
            let body = self.parse_method_block()?;
            let range = (start..self.prev_end).into();
            return Ok(ClassMember::Setter(SetterDef {
                name,
                param,
                body,
                is_static,
                attributes: Vec::new(),
                range,
                name_range,
            }));
        }
        let params = if self.eat(&Token::LParen) {
            let list = self.parse_selector_params(Token::RParen)?;
            self.expect(&Token::RParen, &["\")\""])?;
            Some(list)
        } else {
            None
        };
        let body = self.parse_method_block()?;
        let range = (start..self.prev_end).into();
        if let Some(params) = params {
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

    /// Parses a bracket subscript method — `[_ idx] { ... }` /
    /// `[_ idx]=(put value) { ... }` / `[] { ... }` / `[]=(put value) { ... }` (U-INDEX,
    /// [ADR-0060](../../docs/adr/accepted/0060-index-operator-as-real-selector.md)).
    ///
    /// `start` is the position of the already-peeked `[` (captured by the
    /// caller before dispatching here, matching the sibling member-parsing
    /// methods' convention). Reuses [`Parser::parse_param_list`] verbatim for
    /// the bracketed parameter list — the *only* structural difference from
    /// an ordinary `(params)` method is the delimiter; the assignment value
    /// follows the brackets as `=(put value)`
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
        let params = self.parse_selector_params(Token::RBracket)?;
        if let Some(rest) = params.iter().find(|param| param.is_rest()) {
            return Err(SyntaxError {
                kind: SyntaxErrorKind::Message("rest parameters are not supported in subscript declarations".to_string()),
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
            if matches!(self.peek(), Token::Colon) {
                let err_start = self.cur_start();
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::Message(
                        "parameter declaration labels no longer use `:`; write `label local`, or `label` when the external and local names are identical"
                            .to_string(),
                    ),
                    range: start_put..err_start,
                });
            }
            let local_name = self.expect_identifier(&["parameter name"])?;
            self.expect(&Token::RParen, &["\")\""])?;
            let put = ParameterDef {
                name: local_name,
                label: None,
                rest_mode: RestMode::None,
                range: (start_put..self.prev_end).into(),
            };
            IndexAccessor::Set { put }
        } else {
            IndexAccessor::Get
        };
        let body = self.parse_method_block()?;
        let range = (start..self.prev_end).into();
        Ok(ClassMember::Index(IndexMethodDef {
            params,
            accessor,
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
        Ok(name)
    }

    /// Parses a parenthesized parameter list. Declaration labels use the
    /// no-colon `external local` form (or just `external` when both names
    /// match). F.1 preserves all three parsed rest modes; pre-F.3 compilation
    /// executes only final positional rest under the transitional U9 rules.
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
    fn parse_selector_params(&mut self, end: Token) -> ParserResult<Vec<ParameterDef>> {
        self.skip_newlines();
        if self.peek() == &end {
            return Ok(Vec::new());
        }
        let mut params: Vec<ParameterDef> = Vec::new();
        let mut any_labeled = false;
        let mut labels = std::collections::HashMap::<String, SourceRange>::new();
        loop {
            self.skip_newlines();
            let start = self.cur_start();
            if params.last().is_some_and(ParameterDef::is_rest) {
                return Err(SyntaxError {
                    kind: SyntaxErrorKind::Message("a rest parameter (\"*name\") must be the last parameter".to_string()),
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
                let name = self.expect_identifier(&["parameter name"])?;
                if any_labeled {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message("a rest parameter cannot follow a labeled parameter".to_string()),
                        range: start..self.prev_end,
                    });
                }
                let range = (start..self.prev_end).into();
                params.push(ParameterDef {
                    name,
                    label: None,
                    rest_mode,
                    range,
                });
            } else if self.eat(&Token::Underscore) {
                let name = self.expect_identifier(&["parameter name"])?;
                if any_labeled {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message("positional parameters must precede labeled parameters".to_string()),
                        range: start..self.prev_end,
                    });
                }
                let range = (start..self.prev_end).into();
                params.push(ParameterDef {
                    name,
                    label: None,
                    rest_mode: RestMode::None,
                    range,
                });
            } else {
                // Reserved words remain illegal local names, but are valid
                // external labels when followed by an ordinary local name
                // (`for key`). Calls already accept the same contextual label
                // vocabulary through `label_name`.
                let first_is_identifier = matches!(self.peek(), Token::Identifier(_));
                let first_ident = if let Some(name) = Self::label_name(self.peek()) {
                    let name = name.to_string();
                    self.advance();
                    name
                } else {
                    return Err(self.error_here(strs(&["parameter name", "_", "*"])));
                };
                if matches!(self.peek(), Token::Colon) {
                    let err_start = self.cur_start();
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message(
                            "parameter declaration labels no longer use `:`; write `label local`, or `label` when the external and local names are identical"
                                .to_string(),
                        ),
                        range: start..err_start,
                    });
                }
                let (name, label) = if matches!(self.peek(), Token::Identifier(_)) {
                    let local_ident = self.expect_identifier(&["parameter name"])?;
                    (local_ident, Some(first_ident))
                } else if !first_is_identifier {
                    return Err(self.error_here(strs(&["local parameter name after reserved label"])));
                } else {
                    (first_ident.clone(), Some(first_ident))
                };
                any_labeled = true;
                let range = (start..self.prev_end).into();
                if labels.insert(label.clone().expect("labeled parameter has a label"), range).is_some() {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message("duplicate parameter label in selector declaration".to_string()),
                        range: range.start..range.end,
                    });
                }
                params.push(ParameterDef {
                    name,
                    label,
                    rest_mode: RestMode::None,
                    range,
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

        let mut params = Vec::new();
        if !self.eat(&Token::Pipe) {
            loop {
                let param = if self.eat(&Token::Underscore) {
                    "_".to_string()
                } else {
                    self.expect_identifier(&["closure parameter name"])?
                };
                params.push(param);
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
            params,
            body,
            expr_body,
            range: (start..self.prev_end).into(),
        })))
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
        let left = self.parse_range()?;

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

    /// Parses the non-associative Range tier immediately above assignment.
    /// Endpoint expressions use the existing non-assignment tier, leaving this
    /// layer reversible when a full precedence table is ratified.
    fn parse_range(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        let lower = if matches!(self.peek(), Token::DotDot | Token::DotDotEqual) {
            None
        } else {
            Some(self.parse_coalesce()?)
        };

        let upper_inclusive = match self.peek() {
            Token::DotDot => false,
            Token::DotDotEqual => true,
            _ => return Ok(lower.expect("range parser reaches this branch only after parsing an expression")),
        };
        self.advance();
        let upper = self.starts_expression().then(|| self.parse_coalesce()).transpose()?;
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
        matches!(
            self.peek(),
            Token::If
                | Token::While
                | Token::True
                | Token::False
                | Token::Int { .. }
                | Token::Float(_)
                | Token::String(_)
                | Token::StringInterp(_)
                | Token::NameSymbol(_)
                | Token::SelectorSymbol { .. }
                | Token::QuotedSymbol(_)
                | Token::Identifier(_)
                | Token::SelfKw
                | Token::Class
                | Token::Super
                | Token::LBracket
                | Token::LParen
                | Token::RecordLBrace
                | Token::LBrace
                | Token::Pipe
                | Token::Minus
                | Token::Not
                | Token::Tilde
        )
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
            args: vec![PackItem::Positional { expr: rhs, range }],
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
            Token::Tilde => UnaryOp::BitNot,
            _ => return self.parse_power(),
        };
        let start = self.cur_start();
        self.advance();
        let expr = self.parse_unary()?;
        let range = (start..self.prev_end).into();
        Ok(Expr::Unary(Box::new(UnaryExpr { op, expr, range })))
    }

    /// Parses power expression: `power := postfix [ "**" unary ]`
    fn parse_power(&mut self) -> ParserResult<Expr> {
        let start = self.cur_start();
        let left = self.parse_call()?;
        if matches!(self.peek(), Token::DoubleAsterisk | Token::Power) {
            self.advance(); // consume `**`
            let right = self.parse_unary()?;
            let range = (start..self.prev_end).into();
            return Ok(Expr::Binary(Box::new(BinaryExpr {
                op: BinaryOp::Power,
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
        let mut trailing_target = false;
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
                let continues_labelled_closure = trailing_target && self.starts_labelled_braced_closure_literal(next);
                if continues_postfix || continues_labelled_closure {
                    while self.pos < next {
                        self.advance();
                    }
                } else {
                    break;
                }
            }

            if trailing_target {
                if let Some((args, end)) = self.parse_trailing_closure_arguments()? {
                    expr = self.attach_trailing_arguments(expr, args, end)?;
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
                trailing_target = false;
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
                trailing_target = false;
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
                    trailing_target = false;
                    continue;
                }
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
                trailing_target = true;
            } else if self.eat(&Token::LParen) {
                let args = self.parse_arg_list()?;
                self.expect(&Token::RParen, &["\")\""])?;
                let range = (start..self.prev_end).into();
                expr = match expr {
                    Expr::Var { value, .. } => Expr::UnqualifiedCall(Box::new(UnqualifiedCallExpr { name: value, args, range })),
                    Expr::ImplementationSelector { value, .. } => Expr::MethodCall(Box::new(MethodCallExpr {
                        object: Expr::SelfVar { range },
                        method: value,
                        args,
                        range,
                    })),
                    expr => Expr::MethodCall(Box::new(MethodCallExpr {
                        object: expr,
                        method: "call".to_string(),
                        args,
                        range,
                    })),
                };
                trailing_target = true;
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
                trailing_target = false;
            }
        }
        Ok(expr)
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
            && self.starts_braced_closure_literal(pos + 2)
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
            let expr = parser.parse_closure_literal(ClosureBodyRequirement::Braced)?;
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
        match self.peek().clone() {
            Token::Identifier(name)
            | Token::FieldIdentifier(name)
            | Token::ImplementationFieldIdentifier(name)
            | Token::ImplementationSelectorIdentifier(name) => {
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
            Token::Power => {
                self.advance();
                Ok("**".to_string())
            }
            Token::Slash => {
                self.advance();
                Ok("/".to_string())
            }
            Token::SlashTilde => {
                self.advance();
                Ok("~/".to_string())
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
            args: vec![PackItem::Positional { expr: body, range: body_range }],
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
                if matches!(self.peek_next(), Token::FatArrow) {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message("anonymous `=>` closures were removed; write `|x| expression`".to_string()),
                        range: range.start..range.end,
                    });
                }
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
                    || matches!(self.peek(), Token::DoubleAsterisk)
                {
                    return self.parse_map_literal(start);
                }

                let message = if matches!(self.peek(), Token::Identifier(_) | Token::Underscore) && matches!(self.peek_next(), Token::FatArrow) {
                    "brace block literals were removed; write `|x| { ... }`"
                } else {
                    "bare brace block literals were removed; write `|| { ... }` for a closure"
                };
                Err(SyntaxError {
                    kind: SyntaxErrorKind::Message(message.to_string()),
                    range: (start..self.prev_end).into(),
                })
            }
            _ => Err(self.error_here(primary_expected())),
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
        let elements = elems
            .into_iter()
            .map(|expr| {
                let r = expr.range();
                ListLiteralElement::Element { expr, range: r }
            })
            .collect();
        Ok(Expr::ListLiteral(Box::new(ListLiteralExpr { elements, range })))
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
            Token::BangEqual => "!=",
            Token::Less => "<",
            Token::LessEqual => "<=",
            Token::Greater => ">",
            Token::GreaterEqual => ">=",
            Token::And => "and",
            Token::Or => "or",
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
            Token::NameSymbol(_) | Token::SelectorSymbol { .. } | Token::QuotedSymbol(_) if matches!(self.peek_next(), Token::Colon) => {
                Some(ProductLabelStart::ExplicitSymbol)
            }
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
            let slot = self.expect_identifier(&["label slot"])?;
            if slot == "_" {
                if seen_label {
                    return Err(self.error_here(strs(&["label slot"])));
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
                    Token::NameSymbol(name) => {
                        self.advance();
                        SymbolLiteralKind::Name(name)
                    }
                    Token::SelectorSymbol { name, labels } => {
                        self.advance();
                        SymbolLiteralKind::Selector { name, labels }
                    }
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
            let first_is_expansion = self.expansion_mode_at_cursor().is_some();
            entries.push(self.parse_tuple_entry(&mut labeled_phase)?);
            self.skip_newlines();
            if first_is_expansion && !matches!(self.peek(), Token::Comma) {
                return Err(self.error_here(strs(&["\",\""])));
            }
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
                return Ok(expr);
            }
            let range = expr.range();
            entries.push(TupleLiteralEntry::Positional { expr, range });
        }

        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RParen) {
                self.advance();
                break;
            }
            entries.push(self.parse_tuple_entry(&mut labeled_phase)?);

            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                break;
            }
            self.skip_newlines();
            if matches!(self.peek(), Token::RParen) {
                self.advance();
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
        if matches!(self.peek(), Token::RBrace) {
            self.advance();
            return Ok(Expr::RecordLiteral(Box::new(RecordLiteralExpr {
                fields: Vec::new(),
                range: (start..self.prev_end).into(),
            })));
        }

        let mut fields = Vec::new();
        loop {
            let Some(label) = self.parse_product_label()? else {
                return Err(self.error_here(strs(&["label"])));
            };
            let value = self.parse_expr()?;
            let label_start = match &label {
                ProductLabel::Static { range, .. } | ProductLabel::Computed { range, .. } => range.start,
            };
            fields.push(RecordLiteralField {
                label,
                value,
                range: (label_start..self.prev_end).into(),
            });

            if !self.eat(&Token::Comma) {
                break;
            }
            if matches!(self.peek(), Token::RBrace) {
                self.advance();
                break;
            }
        }

        self.expect(&Token::RBrace, &["\"}\""])?;
        Ok(Expr::RecordLiteral(Box::new(RecordLiteralExpr {
            fields,
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
            if self.eat(&Token::DoubleAsterisk) {
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
        self.skip_newlines();
        if self.peek() == terminator {
            return Ok(elems);
        }
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::Asterisk) {
                return Err(self.error_message_here("spread element (`*x`) in a collection literal is not yet supported"));
            }
            elems.push(self.parse_expr()?);
            self.skip_newlines();
            if !self.eat(&Token::Comma) {
                break;
            }
            self.skip_newlines();
            // Allow a trailing comma directly before the terminator.
            if self.peek() == terminator {
                break;
            }
        }
        self.skip_newlines();
        Ok(elems)
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
        let Expr::Block(block) = positional_pack_expr(&call.args[0]) else {
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
        assert_eq!(block.params, ["x", "y"]);
        assert!(block.expr_body);
        assert_eq!(block.body.len(), 1);

        let Statement::Let(binding) = only_statement("const f = || {\n  1\n}") else {
            panic!("expected binding");
        };
        let Expr::Block(block) = binding.value.expect("expected value") else {
            panic!("expected closure block");
        };
        assert!(block.params.is_empty());
        assert!(!block.expr_body);
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
        let Expr::TupleLiteral(tuple) = expr else {
            panic!("expected tuple literal")
        };
        assert!(matches!(tuple.entries.as_slice(), [
            TupleLiteralEntry::Positional { .. },
            TupleLiteralEntry::Expand { mode: ExpansionMode::Positional, .. },
            TupleLiteralEntry::Expand { mode: ExpansionMode::Complete, .. },
            TupleLiteralEntry::Labeled { .. },
            TupleLiteralEntry::Expand { mode: ExpansionMode::Labeled, .. },
        ]));

        let Statement::Expr { expr, .. } = only_statement("(***first, x, ***second, label: y)") else {
            panic!("expected tuple expression")
        };
        let Expr::TupleLiteral(tuple) = expr else {
            panic!("expected tuple literal")
        };
        assert_eq!(tuple.entries.len(), 4, "*** must not start the labeled phase");
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
