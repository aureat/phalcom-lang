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
        "\"true\"", "\"false\"", "\"nil\"", "\"self\"", "\"super\"", "identifier", "string",
        "number", "\"(\"", "\"!\"", "\"-\"", "\"{\"",
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
        LexicalError::Invalid => SyntaxError {
            kind: SyntaxErrorKind::InvalidToken,
            range: 0..0,
        },
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
                Err(err) => errors.push(lex_error_to_syntax(err, offset)),
            }
        }
        if !matches!(
            tokens.last(),
            Some(Lexeme {
                token: Token::Eof,
                ..
            })
        ) {
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

    /// Builds an [`SyntaxErrorKind::UnrecognizedToken`] at the current token.
    fn error_here(&self, expected: Vec<String>) -> SyntaxError {
        let lexeme = &self.tokens[self.pos];
        SyntaxError {
            kind: SyntaxErrorKind::UnrecognizedToken {
                token: self.cur_text(),
                expected,
            },
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
    /// A `class` declaration is a compound statement that must be followed by a
    /// newline terminator (never end-of-file); anything else is a run of small
    /// statements separated by `;` and terminated by a newline or end-of-file.
    ///
    /// # Errors
    ///
    /// Returns a [`SyntaxError`] if the item or its terminator is malformed.
    fn parse_top_item(&mut self, out: &mut Vec<Statement>) -> ParserResult<()> {
        if matches!(self.peek(), Token::Class) {
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
                Token::Class | Token::Let | Token::Var | Token::Return => return,
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
            Token::Var => self.parse_binding(BindingKind::Var),
            Token::Return => self.parse_return(),
            _ => self.parse_expr_statement(),
        }
    }

    /// Parses a `let`/`var` binding: `<kw> name (= expr)?`.
    ///
    /// `kind` records whether the `let` or `var` keyword was consumed
    /// (ADR-0014); the caller has already confirmed the current token matches.
    /// Mutability and missing-initializer rules are enforced later by the
    /// compiler, not here.
    ///
    /// # Errors
    ///
    /// Returns an error if the binding name is missing or the initialiser
    /// expression is malformed.
    fn parse_binding(&mut self, kind: BindingKind) -> ParserResult<Statement> {
        let start = self.cur_start();
        self.advance(); // 'let' or 'var'
        let name = self.expect_identifier(&["identifier"])?;
        let value = if self.eat(&Token::Equal) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        let range = (start..self.prev_end).into();
        Ok(Statement::Let(LetBinding {
            kind,
            name,
            value,
            range,
        }))
    }

    /// Parses a `return expr?` statement.
    ///
    /// # Errors
    ///
    /// Returns an error if a present return expression is malformed.
    fn parse_return(&mut self) -> ParserResult<Statement> {
        let start = self.cur_start();
        self.advance(); // 'return'
        let value = if self.at_expr_start() {
            Some(self.parse_expr()?)
        } else {
            None
        };
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
                | Token::True
                | Token::False
                | Token::Identifier(_)
                | Token::SelfKw
                | Token::Super
                | Token::LParen
                | Token::Bang
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

    /// Parses a `class Name { members }` declaration.
    ///
    /// # Errors
    ///
    /// Returns an error if the name, braces, or any member is malformed.
    fn parse_class(&mut self) -> ParserResult<Statement> {
        let start = self.cur_start();
        self.advance(); // 'class'
        let name = self.expect_identifier(&["identifier"])?;
        self.expect(&Token::LBrace, &["\"{\""])?;
        let members = self.parse_class_body()?;
        self.expect(&Token::RBrace, &["\"}\""])?;
        let range = (start..self.prev_end).into();
        Ok(Statement::Class(ClassDef {
            name,
            members,
            range,
        }))
    }

    /// Parses the members of a class body up to the closing `}`.
    ///
    /// Blank lines are ignored; each member is a method, getter, or setter.
    ///
    /// # Errors
    ///
    /// Returns an error on a malformed member or if end-of-file is reached
    /// before the closing brace.
    fn parse_class_body(&mut self) -> ParserResult<Vec<ClassMember>> {
        let mut members = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek() {
                Token::RBrace => break,
                Token::Eof => return Err(self.error_here(strs(&["\"}\""]))),
                _ => {}
            }
            members.push(self.parse_class_member()?);
        }
        Ok(members)
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
        if self.eat(&Token::Construct) {
            let name = self.parse_method_name()?;
            self.expect(&Token::LParen, &["\"(\""])?;
            let params = self.parse_param_list()?;
            self.expect(&Token::RParen, &["\")\""])?;
            let body = self.parse_method_block()?;
            let range = (start..self.prev_end).into();
            return Ok(ClassMember::Construct(ConstructDef {
                name,
                params,
                body,
                range,
            }));
        }
        let is_static = self.eat(&Token::Static);
        let name = self.parse_method_name()?;
        let has_equal = self.eat(&Token::Equal);
        if has_equal && name.starts_with('_') {
            let expr = self.parse_expr()?;
            self.expect(&Token::Newline, &["newline"])?;
            let range = (start..self.prev_end).into();
            return Ok(ClassMember::Getter(GetterDef {
                name,
                body: vec![Statement::Expr { expr, range }],
                is_static,
                range,
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
                if !list.is_empty() {
                    list[0].name.clone()
                } else {
                    "value".to_string()
                }
            } else {
                "value".to_string()
            };
            Ok(ClassMember::Setter(SetterDef {
                name,
                param,
                body,
                is_static,
                range,
            }))
        } else if let Some(params) = params {
            Ok(ClassMember::Method(MethodDef {
                name,
                params,
                body,
                is_static,
                range,
            }))
        } else {
            Ok(ClassMember::Getter(GetterDef {
                name,
                body,
                is_static,
                range,
            }))
        }
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
        Ok(name)
    }

    /// Parses a parenthesized parameter list: comma-separated identifiers,
    /// each optionally a labeled parameter (`name:`) or, at most once and only
    /// as the final entry, a rest parameter (`*name`, U9,
    /// `messages-and-selectors.md` §4).
    ///
    /// Shared by method and constructor parameter lists (block-literal
    /// parameters are parsed by a separate ad hoc scanner in
    /// [`Parser::parse_primary`] and never reach this function, so no
    /// block-literal guard is needed here).
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
        if matches!(self.peek(), Token::RParen) {
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
            let label = if self.eat(&Token::Colon) {
                Some(name.clone())
            } else {
                None
            };
            if is_rest {
                if label.is_some() {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message("a rest parameter (\"*name\") cannot have a label".to_string()),
                        range: start..self.prev_end,
                    });
                }
                if any_labeled {
                    return Err(SyntaxError {
                        kind: SyntaxErrorKind::Message(
                            "a rest parameter cannot follow a labeled parameter".to_string(),
                        ),
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
    /// ignored and nested `class` declarations are permitted.
    ///
    /// # Errors
    ///
    /// Returns an error on a malformed statement or a missing terminating
    /// newline.
    fn parse_block_statements(&mut self) -> ParserResult<Vec<Statement>> {
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek() {
                Token::RBrace | Token::Eof => break,
                Token::Class => {
                    stmts.push(self.parse_class()?);
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
    /// [`StringSegment::Expr`] is re-parsed from its source slice (with this
    /// parser's [`offset`](Self::offset) folded onto the segment's start so
    /// spans stay absolute) and wrapped as `String.new(expr)` — the working
    /// content-stringify send while a real content `toString` is blocked on
    /// U-CORE-4 (see ADR-0022). The parts are folded left with binary `+`
    /// ([`BinaryOp::Add`]), so `"a \(x) b"` lowers to
    /// `"a " + String.new(x) + " b"`.
    ///
    /// # Errors
    ///
    /// Returns a [`SyntaxError`] if any interpolated expression fails to parse,
    /// or if an interpolation body is empty (no expression).
    fn desugar_string_interp(
        &self,
        segments: Vec<StringSegment>,
        range: SourceRange,
    ) -> ParserResult<Expr> {
        let mut acc: Option<Expr> = None;
        for segment in segments {
            let part = match segment {
                StringSegment::Literal(value) => Expr::String { value, range },
                StringSegment::Expr { source, start } => {
                    let inner = self.parse_interp_expr(&source, start, range)?;
                    Expr::MethodCall(Box::new(MethodCallExpr {
                        object: Expr::Var {
                            value: "String".to_string(),
                            range,
                        },
                        method: "new".to_string(),
                        args: vec![Argument {
                            label: None,
                            expr: inner,
                            range,
                        }],
                        range,
                    }))
                }
            };
            acc = Some(match acc {
                None => part,
                Some(left) => Expr::Binary(Box::new(BinaryExpr {
                    op: BinaryOp::Add,
                    left,
                    right: part,
                    range,
                })),
            });
        }
        // An interpolated string always has at least one segment (the lexer only
        // emits `StringInterp` once it has seen an interpolation), so `acc` is
        // populated; fall back to the empty string for total robustness.
        Ok(acc.unwrap_or(Expr::String {
            value: String::new(),
            range,
        }))
    }

    /// Re-parses a single interpolated-expression source slice into an [`Expr`].
    ///
    /// `start` is the byte offset of `source` within the lexer input; this
    /// parser's [`offset`](Self::offset) is folded on so the re-parsed spans are
    /// absolute. The slice must contain exactly one expression.
    ///
    /// # Errors
    ///
    /// Returns a [`SyntaxError`] if the slice does not parse as a single
    /// expression (a parse failure, or an empty / non-expression body).
    fn parse_interp_expr(
        &self,
        source: &str,
        start: usize,
        range: SourceRange,
    ) -> ParserResult<Expr> {
        let program = parse_source(source, self.offset + start)?;
        match program.statements.into_iter().next() {
            Some(Statement::Expr { expr, .. }) => Ok(expr),
            _ => Err(SyntaxError {
                kind: SyntaxErrorKind::UnrecognizedToken {
                    token: String::new(),
                    expected: primary_expected(),
                },
                range: range.start..range.end,
            }),
        }
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
        while let Some((prec, op)) = binary_op(self.peek()) {
            if prec < min_prec {
                break;
            }
            self.advance();
            let right = self.parse_binary(prec + 1)?;
            let range = (start..self.prev_end).into();
            left = Expr::Binary(Box::new(BinaryExpr {
                op,
                left,
                right,
                range,
            }));
        }
        Ok(left)
    }

    /// Parses a prefix unary expression (`-x`, `!x`), or delegates to
    /// [`Parser::parse_call`].
    ///
    /// # Errors
    ///
    /// Propagates any error from the operand expression.
    fn parse_unary(&mut self) -> ParserResult<Expr> {
        let op = match self.peek() {
            Token::Minus => UnaryOp::Negate,
            Token::Bang => UnaryOp::Not,
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
            Token::Dot | Token::QuestionDot | Token::LParen | Token::LBrace
        ) {
            if self.eat(&Token::QuestionDot) {
                expr = self.parse_optional_send(expr, start)?;
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
                    expr = Expr::GetProperty(Box::new(GetPropertyExpr {
                        object: expr,
                        property,
                        range,
                    }));
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

    /// Parses a property name after `.`: an identifier or the `class` keyword.
    ///
    /// # Errors
    ///
    /// Returns an error if the token following `.` is neither.
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
            _ => Err(self.error_here(strs(&["identifier", "\"class\""]))),
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
                Ok(Expr::Boolean {
                    value: false,
                    range,
                })
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
            Token::Identifier(value) => {
                if matches!(self.peek_next(), Token::FatArrow) {
                    let start = self.cur_start();
                    self.advance(); // identifier
                    self.advance(); // =>
                    let body_expr = self.parse_expr()?;
                    let range = (start..self.prev_end).into();
                    let stmt = Statement::Expr {
                        expr: body_expr,
                        range,
                    };
                    return Ok(Expr::Block(Box::new(BlockExpr {
                        params: vec![value],
                        body: vec![stmt],
                        expr_body: true,
                        range,
                    })));
                }
                self.advance();
                if value.starts_with('_') {
                    Ok(Expr::Field { value, range })
                } else {
                    Ok(Expr::Var { value, range })
                }
            }
            Token::SelfKw => {
                self.advance();
                Ok(Expr::SelfVar { range })
            }
            Token::Super => {
                self.advance();
                Ok(Expr::SuperVar { range })
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen, &["\")\""])?;
                Ok(expr)
            }
            Token::LBrace => {
                let start = self.cur_start();
                self.advance(); // '{'
                
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

    fn parse_arg_list(&mut self) -> ParserResult<Vec<Argument>> {
        if matches!(self.peek(), Token::RParen) {
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
        assert!(
            result.errors.len() >= 2,
            "expected at least two recovered errors, got {:?}",
            result.errors
        );
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
    fn let_binding_records_immutable_kind() {
        let Statement::Let(binding) = only_statement("let x = 1") else {
            panic!("expected a let binding");
        };
        assert_eq!(binding.kind, BindingKind::Let);
        assert_eq!(binding.name, "x");
        assert!(binding.value.is_some());
    }

    #[test]
    fn var_binding_records_mutable_kind() {
        let Statement::Let(binding) = only_statement("var x") else {
            panic!("expected a var binding");
        };
        assert_eq!(binding.kind, BindingKind::Var);
        assert_eq!(binding.name, "x");
        // `var x` has no initializer; the compiler surfaces this as `None`.
        assert!(binding.value.is_none());
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
