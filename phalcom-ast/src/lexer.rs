//! Hand-written scanner turning Phalcom source into a [`Token`] stream.
//!
//! The [`Lexer`] is a fully hand-rolled, allocation-light scanner (no scanner
//! generator, no regex engine): it walks the source one byte at a time,
//! recognises the fixed keyword/operator/punctuation set of [`Token`], and
//! decodes identifier, string, and number literals. It implements
//! [`Iterator`], yielding a [`Spanned`] item per token with precise half-open
//! byte spans for diagnostics.
//!
//! Two behaviours are load-bearing and preserved from the previous
//! generator-based lexer so the parser and its snapshots stay stable:
//!
//! * **Newlines are tokens.** `\n` (and `\r\n`) become [`Token::Newline`]; the
//!   parser uses them as statement terminators. Spaces, tabs, form feeds, and
//!   `//` line comments are skipped as trivia.
//! * **A single end-of-file marker is injected.** After the last real token the
//!   iterator yields one [`Token::Eof`] whose span is a zero-width point at the
//!   end of the last token (not at the raw end of input), then `None`. This is
//!   what lets a file ending in trailing whitespace or a trailing newline report
//!   errors at the right place (fix F10).
//!
//! See `docs/spec/lexical-structure.md` for the target lexical grammar; this
//! scanner implements the subset the parser currently accepts.

use crate::token::{LexicalError, StringSegment, Token};

struct MultilineBoundary {
    value_end: usize,
    #[allow(dead_code)]
    close_start: usize,
    close_end: usize,
    margin: String,
}

/// A lexer result item: `Ok((start, token, end))` on success or `Err` on a
/// scan failure.
///
/// The `start`/`end` are half-open byte offsets into the source. This mirrors
/// the `(location, token, location)` triple shape expected by
/// location-threading parsers.
pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

/// A hand-written scanner over a borrowed source string.
///
/// Construct one with [`Lexer::new`] and drive it via [`Iterator`]. The scanner
/// never panics on arbitrary input: an unrecognised byte yields a
/// [`LexicalError`] and advances past the offending character so iteration
/// always terminates.
pub struct Lexer<'input> {
    /// The source text being scanned.
    input: &'input str,
    /// The source as raw bytes, for cheap positional lookahead.
    bytes: &'input [u8],
    /// The current byte offset into [`Lexer::input`].
    pos: usize,
    /// The end byte offset of the last real token emitted.
    ///
    /// The injected [`Token::Eof`] is placed at this point, so trailing trivia
    /// does not push the end-of-file marker past the meaningful source.
    last_end: usize,
    /// Whether the single injected [`Token::Eof`] has already been yielded.
    eof_emitted: bool,
    /// The last non-[`Token::Newline`] token emitted, driving newline
    /// suppression (D3).
    ///
    /// A newline that immediately follows a token which *cannot end a
    /// statement* (an operator, an opener, a separator — see
    /// [`suppresses_following_newline`]) is swallowed as trivia instead of
    /// being emitted, so a trailing-operator continuation like `1 +\n2` lexes
    /// as one expression. This is a lexer-level state machine, **not** parser
    /// ASI: the parser sees fewer newlines and its terminator logic is left
    /// untouched. See `docs/spec/lexical-structure.md` §1.
    last_significant: Option<Token>,
}

impl<'input> Lexer<'input> {
    /// Creates a scanner over `input`, positioned at the first byte.
    ///
    /// A `#!` at byte offset 0 (a Unix shebang line, e.g.
    /// `#!/usr/bin/env phalcom`) is skipped as trivia up to (not including)
    /// its terminating newline, exactly like a `//` line comment — this is
    /// the reserved sigil carve-out from selectors.md §2 that keeps a shebang
    /// from being swallowed by the `#`-symbol-literal rule. The check is only
    /// performed once, here, since `#!` has no special meaning anywhere else
    /// in the source.
    pub fn new(input: &'input str) -> Self {
        let bytes = input.as_bytes();
        let pos = if bytes.starts_with(b"#!") {
            bytes.iter().position(|&b| b == b'\n').unwrap_or(bytes.len())
        } else {
            0
        };
        Self {
            input,
            bytes,
            pos,
            last_end: pos,
            eof_emitted: false,
            last_significant: None,
        }
    }

    /// Returns the byte `n` positions ahead of the cursor, if any.
    fn peek_at(&self, n: usize) -> Option<u8> {
        self.bytes.get(self.pos + n).copied()
    }

    /// Skips spaces, tabs, form feeds, lone carriage returns, `//` line
    /// comments, and `/* … */` block comments.
    ///
    /// Newlines are intentionally *not* skipped: they are meaningful
    /// [`Token::Newline`] tokens. A `\r` is only skipped when it is not part of
    /// a `\r\n` newline. Block comments are flat (non-nesting); a newline
    /// *inside* a block comment is consumed with the comment and never leaks a
    /// [`Token::Newline`]. See `docs/spec/lexical-structure.md` and ADR-0016.
    ///
    /// # Errors
    ///
    /// Returns [`LexicalError::UnterminatedBlockComment`] if a `/*` is opened
    /// but end-of-input is reached before the closing `*/`; the carried span
    /// runs from the opening `/*` to the cursor.
    /// Scans a `//` line comment up to (not including) the terminating newline or EOF.
    fn scan_line_comment(&mut self) {
        self.pos += 2;
        while let Some(c) = self.peek_at(0) {
            if c == b'\n' || (c == b'\r' && self.peek_at(1) == Some(b'\n')) {
                break;
            }
            self.pos += 1;
        }
    }

    /// Scans a `/* … */` block comment up to and including `*/`.
    fn scan_block_comment(&mut self) -> Result<(), LexicalError> {
        let open = self.pos;
        self.pos += 2;
        loop {
            match self.peek_at(0) {
                Some(b'*') if self.peek_at(1) == Some(b'/') => {
                    self.pos += 2;
                    return Ok(());
                }
                Some(_) => self.pos += 1,
                None => {
                    return Err(LexicalError::UnterminatedBlockComment(open..self.pos));
                }
            }
        }
    }

    /// Skips spaces, tabs, form feeds, lone carriage returns, `//` line
    /// comments, and `/* … */` block comments.
    fn skip_trivia(&mut self) -> Result<(), LexicalError> {
        while let Some(b) = self.peek_at(0) {
            match b {
                b' ' | b'\t' | b'\x0c' => self.pos += 1,
                // A lone carriage return (not part of a `\r\n` newline).
                b'\r' if self.peek_at(1) != Some(b'\n') => self.pos += 1,
                b'/' if self.peek_at(1) == Some(b'/') => self.scan_line_comment(),
                b'/' if self.peek_at(1) == Some(b'*') => self.scan_block_comment()?,
                _ => return Ok(()),
            }
        }
        Ok(())
    }

    /// Scans one token starting at the cursor, advancing past it.
    ///
    /// The cursor is guaranteed to be on a non-trivia byte on entry. On an
    /// unrecognised byte the cursor is advanced past the full (possibly
    /// multi-byte) character before returning the error, so callers can resume.
    ///
    /// # Errors
    ///
    /// Returns [`LexicalError::InvalidToken`] for an unrecognised character,
    /// [`LexicalError::UnterminatedString`] for a string with no closing quote,
    /// or [`LexicalError::InvalidFloat`] if a numeric literal fails to parse.
    fn scan_token(&mut self) -> Result<Token, LexicalError> {
        let b = self.bytes[self.pos];
        match b {
            b'\n' => {
                self.pos += 1;
                Ok(Token::Newline)
            }
            b'\r' => {
                // Guaranteed `\r\n` here: a lone `\r` is skipped as trivia.
                self.pos += 2;
                Ok(Token::Newline)
            }
            b'0'..=b'9' => self.scan_number(),
            b'.' if matches!(self.peek_at(1), Some(b'0'..=b'9')) => self.scan_number(),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.scan_identifier(),
            b'#' if self.peek_at(1) == Some(b'{') => {
                self.pos += 2;
                Ok(Token::RecordLBrace)
            }
            b'#' if self.peek_at(1) == Some(b'"') => self.scan_quoted_symbol(),
            b'"' => self.scan_string_like(),
            b'#' => {
                self.pos += 1;
                Ok(Token::Hash)
            }
            _ => self.scan_operator(),
        }
    }

    /// Scans a numeric literal per PDR-0026, producing either [`Token::Int`] or [`Token::Float`].
    ///
    /// Malformed literals return a single [`LexicalError::NumericLiteral`] with a span over
    /// the full lexeme.
    fn scan_number(&mut self) -> Result<Token, LexicalError> {
        let start = self.pos;
        let res = self.scan_number_body(start);
        if res.is_err() {
            // Consume remaining contiguous alphanumeric/underscore chars to form complete lexeme span
            while matches!(self.peek_at(0), Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'.')) {
                // If dot, only consume if followed by alpha/digit/underscore or end
                self.pos += 1;
            }
            return Err(LexicalError::NumericLiteral(start..self.pos));
        }
        res
    }

    fn scan_number_body(&mut self, start: usize) -> Result<Token, LexicalError> {
        let first_byte = self.bytes[self.pos];

        // 1. Radix Prefixes: 0b, 0o, 0x
        if first_byte == b'0' && matches!(self.peek_at(1), Some(b'b' | b'B' | b'o' | b'O' | b'x' | b'X')) {
            let prefix_char = self.bytes[self.pos + 1].to_ascii_lowercase();
            let radix = match prefix_char {
                b'b' => 2,
                b'o' => 8,
                b'x' => 16,
                _ => unreachable!(),
            };
            self.pos += 2; // skip 0b/0o/0x

            // Allow one underscore immediately after base prefix (`0x_FF`)
            if self.peek_at(0) == Some(b'_') {
                self.pos += 1;
            }

            let digits = self.scan_radix_digits(radix)?;
            if digits.is_empty() {
                return Err(LexicalError::NumericLiteral(start..self.pos));
            }
            return Ok(Token::Int { digits, radix });
        }

        // 2. Leading Dot Float (`.5`)
        if first_byte == b'.' {
            self.pos += 1; // skip '.'
            let frac = self.scan_decimal_digits()?;
            if frac.is_empty() {
                return Err(LexicalError::NumericLiteral(start..self.pos));
            }
            let mut exponent = None;
            if matches!(self.peek_at(0), Some(b'e' | b'E')) {
                exponent = Some(self.scan_exponent()?);
            }
            let raw_str = format!("0.{}", frac);
            let val = parse_float_val(&raw_str, exponent)?;
            return Ok(Token::Float(val));
        }

        // 3. Leading Zero Decimal check (`0123` rejected, zero-only forms like `0`, `0_0` valid)
        if first_byte == b'0' {
            // Check if followed by digits or underscore
            let mut peek_idx = 1;
            while matches!(self.peek_at(peek_idx), Some(b'0'..=b'9' | b'_')) {
                peek_idx += 1;
            }
            let run = &self.input[self.pos..self.pos + peek_idx];
            let cleaned = run.replace('_', "");
            // If run contains digits other than '0' after leading zero, e.g., '0123'
            if cleaned.len() > 1 && cleaned.chars().any(|c| c != '0') {
                return Err(LexicalError::NumericLiteral(start..self.pos + peek_idx));
            }
        }

        // 4. Decimal Int / Float
        let int_part = self.scan_decimal_digits()?;
        if int_part.is_empty() {
            return Err(LexicalError::NumericLiteral(start..self.pos));
        }

        // A trailing decimal point is not a Float literal. Preserve `5.foo`,
        // `5.-`, `5.+`, and `5..6` as ordinary member/operator/range tokenization,
        // but reject a numeric candidate ending at `5.` atomically.
        if self.peek_at(0) == Some(b'.')
            && !matches!(
                self.peek_at(1),
                Some(
                    b'0'..=b'9'
                    | b'a'..=b'z'
                    | b'A'..=b'Z'
                    | b'_'
                    | b'.'
                    | b'+'
                    | b'-'
                    | b'*'
                    | b'/'
                    | b'~'
                    | b'%'
                    | b'<'
                    | b'='
                    | b'!'
                    | b'>'
                    | b'&'
                    | b'|'
                    | b'^'
                    | b'?'
                    | b'@'
                    | b':'
                    | b'#'
                    | b'[',
                )
            )
        {
            return Err(LexicalError::NumericLiteral(start..self.pos + 1));
        }

        let has_dot = self.peek_at(0) == Some(b'.') && matches!(self.peek_at(1), Some(b'0'..=b'9'));
        let mut frac_part = None;
        if has_dot {
            self.pos += 1; // consume '.'
            let frac = self.scan_decimal_digits()?;
            if frac.is_empty() {
                return Err(LexicalError::NumericLiteral(start..self.pos));
            }
            frac_part = Some(frac);
        }

        let has_exp = matches!(self.peek_at(0), Some(b'e' | b'E'));
        let mut exponent = None;
        if has_exp {
            exponent = Some(self.scan_exponent()?);
        }

        if has_dot || has_exp {
            let raw_str = if let Some(frac) = frac_part {
                format!("{}.{}", int_part, frac)
            } else {
                int_part
            };
            let val = parse_float_val(&raw_str, exponent)?;
            Ok(Token::Float(val))
        } else {
            Ok(Token::Int { digits: int_part, radix: 10 })
        }
    }

    /// Scans decimal digits with single underscore separators between digits.
    fn scan_decimal_digits(&mut self) -> Result<String, LexicalError> {
        let mut digits = String::new();
        while let Some(b) = self.peek_at(0) {
            if b.is_ascii_digit() {
                digits.push(b as char);
                self.pos += 1;
                if self.peek_at(0) == Some(b'_') {
                    if self.peek_at(1).is_some_and(|next| next.is_ascii_digit()) {
                        self.pos += 1; // skip valid '_'
                    } else {
                        // Malformed separator (e.g., doubled `__`, trailing `_`, or before `.`)
                        return Err(LexicalError::NumericLiteral(self.pos..self.pos + 1));
                    }
                }
            } else {
                break;
            }
        }
        Ok(digits)
    }

    /// Scans radix digits for binary (2), octal (8), or hex (16).
    fn scan_radix_digits(&mut self, radix: u32) -> Result<String, LexicalError> {
        let mut digits = String::new();
        while let Some(b) = self.peek_at(0) {
            let valid = match radix {
                2 => matches!(b, b'0' | b'1'),
                8 => matches!(b, b'0'..=b'7'),
                16 => b.is_ascii_hexdigit(),
                _ => false,
            };
            if valid {
                digits.push((b as char).to_ascii_lowercase());
                self.pos += 1;
                if self.peek_at(0) == Some(b'_') {
                    let next_valid = self.peek_at(1).is_some_and(|nb| match radix {
                        2 => matches!(nb, b'0' | b'1'),
                        8 => matches!(nb, b'0'..=b'7'),
                        16 => nb.is_ascii_hexdigit(),
                        _ => false,
                    });
                    if next_valid {
                        self.pos += 1; // skip valid '_'
                    } else {
                        return Err(LexicalError::NumericLiteral(self.pos..self.pos + 1));
                    }
                }
            } else {
                break;
            }
        }
        Ok(digits)
    }

    /// Scans decimal exponent `e[+-]?digits`.
    fn scan_exponent(&mut self) -> Result<String, LexicalError> {
        let mut exp = String::new();
        // Skip 'e' or 'E'
        self.pos += 1;
        if matches!(self.peek_at(0), Some(b'+' | b'-')) {
            exp.push(self.bytes[self.pos] as char);
            self.pos += 1;
        }
        let digits = self.scan_decimal_digits()?;
        if digits.is_empty() {
            return Err(LexicalError::NumericLiteral(self.pos..self.pos));
        }
        exp.push_str(&digits);
        Ok(exp)
    }

    /// Scans an identifier (`[A-Za-z_][A-Za-z0-9_]*`), resolving keywords.
    ///
    /// Field names (a leading `_`) also lex as [`Token::Identifier`]; the parser
    /// decides between a variable and a field reference.
    fn scan_identifier(&mut self) -> Result<Token, LexicalError> {
        let start = self.pos;
        if self.peek_at(0) != Some(b'_') {
            while matches!(self.peek_at(0), Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')) {
                self.pos += 1;
            }
            let slice = &self.input[start..self.pos];
            return Ok(match slice {
                "let" => Token::Let,
                "const" => Token::Const,
                "fn" => Token::Fn,
                "class" => Token::Class,
                "return" => Token::Return,
                "true" => Token::True,
                "false" => Token::False,
                "if" => Token::If,
                "else" => Token::Else,
                "while" => Token::While,
                "for" => Token::For,
                "break" => Token::Break,
                "continue" => Token::Continue,
                "import" => Token::Import,
                "from" => Token::From,
                "export" => Token::Export,
                "expose" => Token::Expose,
                "self" => Token::SelfKw,
                "super" => Token::Super,
                "in" => Token::In,
                "as" => Token::As,
                "is" => Token::Is,
                "and" => Token::And,
                "or" => Token::Or,
                "not" => Token::Not,
                "static" => Token::Static,
                "construct" => Token::Construct,
                "throw" => Token::Throw,
                "try" => Token::Try,
                _ => Token::Identifier(slice.to_string()),
            });
        }

        // Starts with '_'
        if self.peek_at(1) == Some(b'_') {
            if matches!(self.peek_at(2), Some(b'a'..=b'z' | b'A'..=b'Z')) {
                self.pos += 2;
                while matches!(self.peek_at(0), Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')) {
                    self.pos += 1;
                }
                let slice = &self.input[start..self.pos];
                if slice.len() >= 4 && slice.ends_with("__") {
                    return Ok(Token::Identifier(slice.to_string()));
                }
                return Ok(Token::ImplementationFieldIdentifier(slice.to_string()));
            }
        } else if self.peek_at(1) == Some(b'$') {
            if matches!(self.peek_at(2), Some(b'a'..=b'z' | b'A'..=b'Z')) {
                self.pos += 2;
                while matches!(self.peek_at(0), Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')) {
                    self.pos += 1;
                }
                let slice = &self.input[start..self.pos];
                return Ok(Token::ImplementationSelectorIdentifier(slice.to_string()));
            }
        } else if matches!(self.peek_at(1), Some(b'a'..=b'z' | b'A'..=b'Z')) {
            self.pos += 1;
            while matches!(self.peek_at(0), Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')) {
                self.pos += 1;
            }
            let slice = &self.input[start..self.pos];
            return Ok(Token::FieldIdentifier(slice.to_string()));
        } else if !matches!(self.peek_at(1), Some(b'0'..=b'9' | b'_' | b'$')) {
            // A standalone `_` positional-slot marker.
            self.pos += 1;
            return Ok(Token::Underscore);
        }

        // Malformed/reserved sequence starting with '_'
        while matches!(self.peek_at(0), Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$')) {
            self.pos += 1;
        }
        Err(LexicalError::InvalidToken(start..self.pos))
    }

    /// Scans a double-quoted string literal, stripping the surrounding quotes.
    ///
    /// A string with no `\(…)` interpolation lexes to [`Token::String`], at
    /// parity with the previous `"[^"]*"` lexeme (any byte except `"` accepted
    /// in the body, including newlines). A string containing at least one
    /// `\(expr)` interpolation lexes to [`Token::StringInterp`] carrying ordered
    /// [`StringSegment`]s (ADR-0022, `docs/spec/lexical-structure.md` §5):
    ///
    /// * `\(` … `)` delimits an interpolated expression (balanced parentheses).
    /// * `\\` is a literal backslash, so `\\(` is a literal `\(` (no
    ///   interpolation).
    /// * Any other `\x` is left verbatim as a literal `\` then `x`, preserving
    ///   the pre-interpolation behaviour where `"\n"` was two literal
    ///   characters.
    ///
    /// # Errors
    ///
    /// Returns [`LexicalError::UnterminatedString`] if end-of-input is reached
    /// before the closing quote — including inside an unterminated `\(…`
    /// interpolation.
    /// Dispatches between single-line string and triple-quoted multiline text block.
    fn scan_string_like(&mut self) -> Result<Token, LexicalError> {
        if self.peek_at(0) == Some(b'"') && self.peek_at(1) == Some(b'"') && self.peek_at(2) == Some(b'"') {
            self.scan_multiline_string()
        } else {
            self.scan_string()
        }
    }

    /// Scans an interpolation body `\(…)` balancing parentheses while respecting
    /// nested lexical modes (nested strings, line comments, and block comments).
    fn scan_interpolation_body(&mut self, interpolation_open: usize) -> Result<StringSegment, LexicalError> {
        let body_start = self.pos;
        let mut depth = 1usize;

        loop {
            match self.peek_at(0) {
                None => {
                    return Err(LexicalError::UnterminatedInterpolation(interpolation_open..self.pos));
                }
                Some(b'"') => {
                    // Consume nested string or multiline string using centralized dispatcher.
                    let _ = self.scan_string_like()?;
                }
                Some(b'/') if self.peek_at(1) == Some(b'/') => {
                    self.scan_line_comment();
                }
                Some(b'/') if self.peek_at(1) == Some(b'*') => {
                    self.scan_block_comment()?;
                }
                Some(b'(') => {
                    depth += 1;
                    self.pos += 1;
                }
                Some(b')') => {
                    depth -= 1;
                    if depth == 0 {
                        let body_end = self.pos;
                        let source = self.input[body_start..body_end].to_string();
                        self.pos += 1;
                        return Ok(StringSegment::Expr {
                            source,
                            range: body_start..body_end,
                        });
                    }
                    self.pos += 1;
                }
                Some(_) => {
                    self.pos += self.char_len_at(self.pos);
                }
            }
        }
    }

    /// Scans the structural opening line of a multiline text block `"""`.
    fn scan_multiline_opening(&mut self, open: usize) -> Result<usize, LexicalError> {
        self.pos += 3;
        while let Some(b) = self.peek_at(0) {
            if b == b' ' || b == b'\t' {
                self.pos += 1;
            } else {
                break;
            }
        }

        match self.peek_at(0) {
            Some(b'\n') => {
                self.pos += 1;
                Ok(self.pos)
            }
            Some(b'\r') if self.peek_at(1) == Some(b'\n') => {
                self.pos += 2;
                Ok(self.pos)
            }
            None => Err(LexicalError::UnterminatedMultilineString(open..self.pos)),
            Some(_) => {
                let start = self.pos;
                let len = self.char_len_at(self.pos);
                self.pos += len;
                Err(LexicalError::InvalidMultilineStringOpening(start..self.pos))
            }
        }
    }

    /// Tests whether the line starting at `line_start` is an isolated closing line:
    /// `<margin>"""<optional spaces/tabs><newline | EOF>`
    fn multiline_close_at_line_start(&self, line_start: usize) -> Option<(usize, usize, String)> {
        let mut p = line_start;
        while p < self.bytes.len() && (self.bytes[p] == b' ' || self.bytes[p] == b'\t') {
            p += 1;
        }

        if p + 2 < self.bytes.len() && self.bytes[p] == b'"' && self.bytes[p + 1] == b'"' && self.bytes[p + 2] == b'"' {
            let quote_start = p;
            let quote_end = p + 3;
            let mut after = quote_end;
            while after < self.bytes.len() && (self.bytes[after] == b' ' || self.bytes[after] == b'\t') {
                after += 1;
            }
            if after == self.bytes.len() || self.bytes[after] == b'\n' || (self.bytes[after] == b'\r' && self.bytes.get(after + 1) == Some(&b'\n')) {
                let margin = self.input[line_start..quote_start].to_string();
                return Some((quote_start, quote_end, margin));
            }
        }
        None
    }

    /// Discovers the multiline boundary and closing margin.
    fn discover_multiline_boundary(&mut self, open: usize, body_start: usize) -> Result<MultilineBoundary, LexicalError> {
        let mut line_start = body_start;

        loop {
            if let Some((close_start, close_end, margin)) = self.multiline_close_at_line_start(line_start) {
                // Compute value_end by stripping the structural newline before the closing line
                let value_end = if line_start == body_start {
                    body_start
                } else if line_start >= 2 && &self.bytes[line_start - 2..line_start] == b"\r\n" {
                    line_start - 2
                } else if line_start >= 1 && self.bytes[line_start - 1] == b'\n' {
                    line_start - 1
                } else {
                    line_start
                };

                return Ok(MultilineBoundary {
                    value_end,
                    close_start,
                    close_end,
                    margin,
                });
            }

            // Advance through this physical line
            self.pos = line_start;
            let mut hit_newline = false;
            while self.pos < self.bytes.len() {
                match self.peek_at(0) {
                    None => break,
                    Some(b'\n') => {
                        self.pos += 1;
                        line_start = self.pos;
                        hit_newline = true;
                        break;
                    }
                    Some(b'\r') if self.peek_at(1) == Some(b'\n') => {
                        self.pos += 2;
                        line_start = self.pos;
                        hit_newline = true;
                        break;
                    }
                    Some(b'\r') => {
                        let start = self.pos;
                        self.pos += 1;
                        return Err(LexicalError::InvalidMultilineStringLineEnding(start..self.pos));
                    }
                    Some(b'\\') => {
                        let esc_start = self.pos;
                        match self.peek_at(1) {
                            Some(b'(') => {
                                self.pos += 2;
                                self.scan_interpolation_body(esc_start)?;
                                if let Some(last_nl) = self.bytes[line_start..self.pos].iter().rposition(|&b| b == b'\n') {
                                    line_start = line_start + last_nl + 1;
                                    hit_newline = true;
                                    break;
                                }
                            }
                            Some(b'"' | b'\\' | b'n' | b't' | b'r') => {
                                self.pos += 2;
                            }
                            Some(_) => {
                                let next_len = self.char_len_at(self.pos + 1);
                                let end = self.pos + 1 + next_len;
                                self.pos = end;
                                return Err(LexicalError::InvalidEscape(esc_start..end));
                            }
                            None => {
                                self.pos += 1;
                            }
                        }
                    }
                    Some(_) => {
                        self.pos += self.char_len_at(self.pos);
                    }
                }
            }

            if !hit_newline {
                // Reached EOF without closing delimiter
                return Err(LexicalError::UnterminatedMultilineString(open..self.input.len()));
            }
        }
    }

    /// Validates that every nonblank physical line begins with the exact margin prefix.
    fn validate_multiline_margin(&self, body_start: usize, value_end: usize, margin: &str) -> Result<(), LexicalError> {
        let mut cur = body_start;
        while cur < value_end {
            let line_start = cur;
            let mut line_end = cur;
            while line_end < value_end && self.bytes[line_end] != b'\n' && self.bytes[line_end] != b'\r' {
                line_end += 1;
            }

            let line_slice = &self.bytes[line_start..line_end];
            let is_blank = line_slice.iter().all(|&b| b == b' ' || b == b'\t');

            if !is_blank && !margin.is_empty() && !line_slice.starts_with(margin.as_bytes()) {
                // Report narrow indentation error
                let mut mismatch_end = line_start;
                while mismatch_end < line_end && (self.bytes[mismatch_end] == b' ' || self.bytes[mismatch_end] == b'\t') {
                    mismatch_end += 1;
                }
                if mismatch_end == line_start {
                    mismatch_end += self.char_len_at(line_start);
                }
                return Err(LexicalError::InvalidMultilineStringIndentation(line_start..mismatch_end));
            }

            cur = line_end;
            if cur < value_end && self.bytes[cur] == b'\r' && self.bytes.get(cur + 1) == Some(&b'\n') {
                cur += 2;
            } else if cur < value_end && (self.bytes[cur] == b'\n' || self.bytes[cur] == b'\r') {
                cur += 1;
            }
        }
        Ok(())
    }

    /// Decodes the semantic content of the multiline text block.
    fn decode_multiline_body(&mut self, body_start: usize, value_end: usize, margin: &str) -> Result<Token, LexicalError> {
        let mut segments: Vec<StringSegment> = Vec::new();
        let mut literal = String::new();
        let mut interpolated = false;

        self.pos = body_start;
        let mut at_line_start = true;

        while self.pos < value_end {
            if at_line_start {
                // Check if current physical line is blank
                let mut p = self.pos;
                while p < value_end && self.bytes[p] != b'\n' && self.bytes[p] != b'\r' {
                    p += 1;
                }
                let line_bytes = &self.bytes[self.pos..p];
                let is_blank = line_bytes.iter().all(|&b| b == b' ' || b == b'\t');

                if is_blank {
                    // Discard all blank-line whitespace
                    self.pos = p;
                } else {
                    // Consume exact margin bytes
                    self.pos += margin.len();
                }
                at_line_start = false;
                if self.pos >= value_end {
                    break;
                }
            }

            match self.peek_at(0) {
                None => break,
                Some(b'\n') => {
                    literal.push('\n');
                    self.pos += 1;
                    at_line_start = true;
                }
                Some(b'\r') if self.peek_at(1) == Some(b'\n') => {
                    literal.push('\n');
                    self.pos += 2;
                    at_line_start = true;
                }
                Some(b'\\') => {
                    let esc_start = self.pos;
                    match self.peek_at(1) {
                        Some(b'(') => {
                            interpolated = true;
                            if !literal.is_empty() {
                                segments.push(StringSegment::Literal(std::mem::take(&mut literal)));
                            }
                            let before_body = self.pos;
                            self.pos += 2;
                            let expr_seg = self.scan_interpolation_body(esc_start)?;
                            segments.push(expr_seg);
                            if self.bytes[before_body..self.pos].contains(&b'\n') {
                                at_line_start = false;
                            }
                        }
                        Some(b'"') => {
                            literal.push('"');
                            self.pos += 2;
                        }
                        Some(b'\\') => {
                            literal.push('\\');
                            self.pos += 2;
                        }
                        Some(b'n') => {
                            literal.push('\n');
                            self.pos += 2;
                        }
                        Some(b't') => {
                            literal.push('\t');
                            self.pos += 2;
                        }
                        Some(b'r') => {
                            literal.push('\r');
                            self.pos += 2;
                        }
                        Some(_) => {
                            let next_len = self.char_len_at(self.pos + 1);
                            let end = self.pos + 1 + next_len;
                            self.pos = end;
                            return Err(LexicalError::InvalidEscape(esc_start..end));
                        }
                        None => {
                            self.pos += 1;
                        }
                    }
                }
                Some(_) => {
                    let len = self.char_len_at(self.pos);
                    literal.push_str(&self.input[self.pos..self.pos + len]);
                    self.pos += len;
                }
            }
        }

        if !interpolated {
            return Ok(Token::String(literal));
        }

        if !literal.is_empty() {
            segments.push(StringSegment::Literal(literal));
        }

        Ok(Token::StringInterp(segments))
    }

    /// Scans a multiline text block `""" ... """`.
    fn scan_multiline_string(&mut self) -> Result<Token, LexicalError> {
        let open = self.pos;
        let body_start = self.scan_multiline_opening(open)?;

        self.pos = body_start;
        let boundary = self.discover_multiline_boundary(open, body_start)?;

        self.validate_multiline_margin(body_start, boundary.value_end, &boundary.margin)?;

        self.pos = body_start;
        let token = self.decode_multiline_body(body_start, boundary.value_end, &boundary.margin)?;

        self.pos = boundary.close_end;
        Ok(token)
    }

    /// Scans a double-quoted string literal, stripping the surrounding quotes.
    ///
    /// Double-quoted string literals decode conventional escapes (`\"`, `\\`, `\n`,
    /// `\t`, `\r`), open interpolation on `\(`, reject unknown escapes with
    /// [`LexicalError::InvalidEscape`], and reject physical newlines with
    /// [`LexicalError::RawNewlineInString`].
    fn scan_string(&mut self) -> Result<Token, LexicalError> {
        let open = self.pos;
        self.pos += 1;
        let mut segments: Vec<StringSegment> = Vec::new();
        let mut literal = String::new();
        let mut interpolated = false;
        loop {
            match self.peek_at(0) {
                None => return Err(LexicalError::UnterminatedString(open..self.pos)),
                Some(b'"') => {
                    self.pos += 1;
                    if !interpolated {
                        return Ok(Token::String(literal));
                    }
                    if !literal.is_empty() {
                        segments.push(StringSegment::Literal(std::mem::take(&mut literal)));
                    }
                    return Ok(Token::StringInterp(segments));
                }
                Some(b'\n') => {
                    let start = self.pos;
                    self.pos += 1;
                    return Err(LexicalError::RawNewlineInString(start..self.pos));
                }
                Some(b'\r') if self.peek_at(1) == Some(b'\n') => {
                    let start = self.pos;
                    self.pos += 2;
                    return Err(LexicalError::RawNewlineInString(start..self.pos));
                }
                Some(b'\r') => {
                    let start = self.pos;
                    self.pos += 1;
                    return Err(LexicalError::RawNewlineInString(start..self.pos));
                }
                Some(b'\\') => {
                    let esc_start = self.pos;
                    match self.peek_at(1) {
                        Some(b'(') => {
                            interpolated = true;
                            if !literal.is_empty() {
                                segments.push(StringSegment::Literal(std::mem::take(&mut literal)));
                            }
                            self.pos += 2;
                            let expr_seg = self.scan_interpolation_body(esc_start)?;
                            segments.push(expr_seg);
                        }
                        Some(b'"') => {
                            literal.push('"');
                            self.pos += 2;
                        }
                        Some(b'\\') => {
                            literal.push('\\');
                            self.pos += 2;
                        }
                        Some(b'n') => {
                            literal.push('\n');
                            self.pos += 2;
                        }
                        Some(b't') => {
                            literal.push('\t');
                            self.pos += 2;
                        }
                        Some(b'r') => {
                            literal.push('\r');
                            self.pos += 2;
                        }
                        Some(_) => {
                            let next_len = self.char_len_at(self.pos + 1);
                            let end = self.pos + 1 + next_len;
                            self.pos = end;
                            return Err(LexicalError::InvalidEscape(esc_start..end));
                        }
                        None => {
                            self.pos += 1;
                        }
                    }
                }
                Some(_) => {
                    let len = self.char_len_at(self.pos);
                    literal.push_str(&self.input[self.pos..self.pos + len]);
                    self.pos += len;
                }
            }
        }
    }

    /// Scans a quoted symbol literal `#"..."`.
    fn scan_quoted_symbol(&mut self) -> Result<Token, LexicalError> {
        let open = self.pos;
        self.pos += 2;
        let mut literal = String::new();
        loop {
            match self.peek_at(0) {
                None => return Err(LexicalError::UnterminatedString(open..self.pos)),
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(Token::QuotedSymbol(literal));
                }
                Some(b'\n') => {
                    let start = self.pos;
                    self.pos += 1;
                    return Err(LexicalError::RawNewlineInString(start..self.pos));
                }
                Some(b'\r') if self.peek_at(1) == Some(b'\n') => {
                    let start = self.pos;
                    self.pos += 2;
                    return Err(LexicalError::RawNewlineInString(start..self.pos));
                }
                Some(b'\r') => {
                    let start = self.pos;
                    self.pos += 1;
                    return Err(LexicalError::RawNewlineInString(start..self.pos));
                }
                Some(b'\\') => {
                    let esc_start = self.pos;
                    match self.peek_at(1) {
                        Some(b'"') => {
                            literal.push('"');
                            self.pos += 2;
                        }
                        Some(b'\\') => {
                            literal.push('\\');
                            self.pos += 2;
                        }
                        Some(b'n') => {
                            literal.push('\n');
                            self.pos += 2;
                        }
                        Some(b't') => {
                            literal.push('\t');
                            self.pos += 2;
                        }
                        Some(b'r') => {
                            literal.push('\r');
                            self.pos += 2;
                        }
                        Some(_) => {
                            let next_len = self.char_len_at(self.pos + 1);
                            let end = self.pos + 1 + next_len;
                            self.pos = end;
                            return Err(LexicalError::InvalidEscape(esc_start..end));
                        }
                        None => {
                            self.pos += 1;
                        }
                    }
                }
                Some(_) => {
                    let len = self.char_len_at(self.pos);
                    literal.push_str(&self.input[self.pos..self.pos + len]);
                    self.pos += len;
                }
            }
        }
    }

    /// Returns the UTF-8 byte length of the character starting at byte `at`.
    ///
    /// Used to advance the cursor by whole characters inside a string body so
    /// multi-byte scalars are never split.
    fn char_len_at(&self, at: usize) -> usize {
        self.input[at..].chars().next().map_or(1, char::len_utf8)
    }

    /// Scans an operator or punctuation token using maximal munch.
    ///
    /// Multi-character operators (`==`, `+=`, `->`, `...`, ...) take priority
    /// over their single-character prefixes.
    ///
    /// # Errors
    ///
    /// Returns [`LexicalError::InvalidToken`] for any byte that does not begin a
    /// known operator or punctuation mark, after advancing past the full
    /// character.
    fn scan_operator(&mut self) -> Result<Token, LexicalError> {
        let b = self.bytes[self.pos];
        let next = self.peek_at(1);
        let (len, token) = match b {
            b'+' if next == Some(b'=') => (2, Token::PlusEqual),
            b'+' => (1, Token::Plus),
            b'-' if next == Some(b'=') => (2, Token::MinusEqual),
            b'-' if next == Some(b'>') => (2, Token::Arrow),
            b'-' => (1, Token::Minus),
            b'*' if next == Some(b'=') => (2, Token::AsteriskEqual),
            b'*' if next == Some(b'*') && self.peek_at(2) == Some(b'*') => (3, Token::TripleAsterisk),
            b'*' if next == Some(b'*') => (2, Token::DoubleAsterisk),
            b'*' => (1, Token::Asterisk),
            b'/' if next == Some(b'=') => (2, Token::SlashEqual),
            b'/' => (1, Token::Slash),
            b'~' if next == Some(b'/') => (2, Token::SlashTilde),
            b'%' if next == Some(b'=') => (2, Token::PercentEqual),
            b'%' => (1, Token::Percent),
            b'<' if next == Some(b'<') => (2, Token::ShiftLeft),
            b'=' if next == Some(b'=') => (2, Token::EqualEqual),
            b'=' => (1, Token::Equal),
            b'!' if next == Some(b'=') => (2, Token::BangEqual),
            b'!' => (1, Token::Bang),
            b'<' if next == Some(b'=') => (2, Token::LessEqual),
            b'<' => (1, Token::Less),
            b'>' if next == Some(b'>') => (2, Token::ShiftRight),
            b'>' if next == Some(b'=') => (2, Token::GreaterEqual),
            b'>' => (1, Token::Greater),
            b':' if next == Some(b':') => (2, Token::ColonColon),
            b':' => (1, Token::Colon),
            b'.' if next == Some(b'.') && self.peek_at(2) == Some(b'.') => (3, Token::DotDotDot),
            b'.' if next == Some(b'.') && self.peek_at(2) == Some(b'=') => (3, Token::DotDotEqual),
            b'.' if next == Some(b'.') => (2, Token::DotDot),
            b'.' => (1, Token::Dot),
            b'(' => (1, Token::LParen),
            b')' => (1, Token::RParen),
            b'{' => (1, Token::LBrace),
            b'#' if next == Some(b'{') => (2, Token::RecordLBrace),
            b'#' => (1, Token::Hash),
            b'}' => (1, Token::RBrace),
            b'[' => (1, Token::LBracket),
            b']' => (1, Token::RBracket),
            b';' => (1, Token::Semicolon),
            b',' => (1, Token::Comma),
            // `Option` operators (ADR-0007). Multi-char `??` and `?.` take
            // priority over a lone `?`; see `docs/spec/lexical-structure.md` §9.
            b'?' if next == Some(b'?') => (2, Token::CoalesceQuestion),
            b'?' if next == Some(b'.') => (2, Token::QuestionDot),
            b'?' => (1, Token::Question),
            b'@' if next == Some(b'!') => (2, Token::AtBang),
            b'@' => (1, Token::At),
            b'&' => (1, Token::Ampersand),
            b'|' => (1, Token::Pipe),
            b'^' => (1, Token::Caret),
            b'~' => (1, Token::Tilde),
            _ => {
                // Unknown character: advance past the whole UTF-8 scalar so the
                // iterator makes progress, and report its exact byte span.
                let ch_len = self.input[self.pos..].chars().next().map_or(1, char::len_utf8);
                let span = self.pos..self.pos + ch_len;
                self.pos += ch_len;
                return Err(LexicalError::InvalidToken(span));
            }
        };
        self.pos += len;
        Ok(token)
    }
}

/// Returns whether a newline immediately following `prev` should be suppressed.
///
/// The rule (D3, `docs/spec/lexical-structure.md` §1) is one-sided: it keys on
/// the **previous** significant token only. Suppression fires exactly when
/// `prev` *cannot end a statement* — an arithmetic/comparison/logical/assignment
/// operator, an [`Option`](Token::CoalesceQuestion) operator, an opener or
/// separator (`(`, `{`, `[`, `,`, `.`, `::`, `:`), or an arrow (`->`, `=>`).
/// After any token that *can* end a statement — an identifier, a literal, a
/// closer (`)`, `}`, `]`), `self`/`super`/`true`/`false`, or a
/// `return`/`break`/`continue` keyword — the newline is preserved so it still
/// terminates the statement.
///
/// This is deliberately scoped to operators and punctuation: statement
/// keywords such as `let`/`if`/`class` are **not** suppressors. Leading-operator
/// continuation (`foo\n.bar`) is intentionally unsupported — the rule never
/// inspects the *next* token.
fn suppresses_following_newline(prev: &Token) -> bool {
    matches!(
        prev,
        // Arithmetic.
        Token::Plus
            | Token::Minus
            | Token::Asterisk
            | Token::DoubleAsterisk
            | Token::TripleAsterisk
            | Token::Power
            | Token::Slash
            | Token::SlashTilde
            | Token::Percent
            // Comparison.
            | Token::EqualEqual
            | Token::BangEqual
            | Token::Less
            | Token::LessEqual
            | Token::Greater
            | Token::GreaterEqual
            // Logical keywords.
            | Token::And
            | Token::Or
            | Token::Not
            // Assignment.
            | Token::Equal
            | Token::PlusEqual
            | Token::MinusEqual
            | Token::AsteriskEqual
            | Token::SlashEqual
            | Token::PercentEqual
            // `Option` operators (ADR-0007).
            | Token::CoalesceQuestion
            | Token::QuestionDot
            // Openers and separators.
            | Token::Comma
            | Token::LParen
            | Token::LBrace
            | Token::RecordLBrace
            | Token::LBracket
            | Token::Dot
            | Token::ColonColon
            | Token::Colon
            // Arrows.
            | Token::Arrow
    )
}

fn parse_float_val(raw_str: &str, exponent: Option<String>) -> Result<f64, LexicalError> {
    let full_str = if let Some(exp) = exponent {
        format!("{}e{}", raw_str, exp)
    } else {
        raw_str.to_string()
    };
    full_str.parse::<f64>().map_err(LexicalError::InvalidFloat)
}

impl Iterator for Lexer<'_> {
    type Item = Spanned<Token, usize, LexicalError>;

    /// Yields the next token, or the single injected [`Token::Eof`] once the
    /// source is exhausted, then `None`.
    ///
    /// Newlines that follow a token which cannot end a statement are suppressed
    /// here (D3) via `suppresses_following_newline`, so a trailing-operator
    /// continuation spans multiple physical lines as one logical construct.
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Err(err) = self.skip_trivia() {
                return Some(Err(err));
            }

            if self.pos >= self.bytes.len() {
                if self.eof_emitted {
                    return None;
                }
                self.eof_emitted = true;
                return Some(Ok((self.last_end, Token::Eof, self.last_end)));
            }

            let start = self.pos;
            match self.scan_token() {
                Ok(Token::Newline) if self.last_significant.as_ref().is_some_and(suppresses_following_newline) => {
                    // Swallow the newline as trivia and scan the next token; the
                    // previous significant token cannot end a statement.
                    continue;
                }
                Ok(token) => {
                    let end = self.pos;
                    self.last_end = end;
                    if token != Token::Newline {
                        self.last_significant = Some(token.clone());
                    }
                    return Some(Ok((start, token, end)));
                }
                Err(err) => return Some(Err(err)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Collects `(token, start, end)` triples, panicking on the first lex error.
    fn spans(src: &str) -> Vec<(Token, usize, usize)> {
        Lexer::new(src)
            .map(|item| {
                let (start, token, end) = item.expect("should lex");
                (token, start, end)
            })
            .collect()
    }

    #[test]
    fn injects_a_single_eof_at_end_of_input() {
        let toks = spans("x");
        assert_eq!(toks, vec![(Token::Identifier("x".to_string()), 0, 1), (Token::Eof, 1, 1)]);
    }

    #[test]
    fn eof_sits_at_end_of_last_real_token_not_trailing_whitespace() {
        // "let = " — the `=` ends at byte 5; the trailing space is skipped and
        // the injected EOF is a zero-width point at byte 5 (fix F10 relies on
        // this precise placement).
        let toks = spans("let = ");
        assert_eq!(toks.last(), Some(&(Token::Eof, 5, 5)));
    }

    #[test]
    fn newlines_are_tokens_and_spaces_are_trivia() {
        let toks = spans("a\n b");
        assert_eq!(
            toks,
            vec![
                (Token::Identifier("a".to_string()), 0, 1),
                (Token::Newline, 1, 2),
                (Token::Identifier("b".to_string()), 3, 4),
                (Token::Eof, 4, 4),
            ]
        );
    }

    #[test]
    fn line_comment_is_skipped_but_its_newline_survives() {
        let toks = spans("1 // c\n2");
        let kinds: Vec<Token> = toks.into_iter().map(|(t, _, _)| t).collect();
        assert_eq!(
            kinds,
            vec![
                Token::Int {
                    digits: "1".to_string(),
                    radix: 10
                },
                Token::Newline,
                Token::Int {
                    digits: "2".to_string(),
                    radix: 10
                },
                Token::Eof,
            ]
        );
    }

    #[test]
    fn maximal_munch_prefers_longer_operators() {
        let kinds: Vec<Token> = spans("a += 1 .. 2 ... 3 -> 4").into_iter().map(|(t, _, _)| t).collect();
        assert_eq!(kinds[1], Token::PlusEqual);
        assert!(kinds.contains(&Token::DotDot));
        assert!(kinds.contains(&Token::DotDotDot));
        assert!(kinds.contains(&Token::Arrow));
    }

    #[test]
    fn number_dot_dot_is_not_a_decimal() {
        // `1..2` must be Number, DotDot, Number — never `1.` then `.2`.
        let kinds: Vec<Token> = spans("1..2").into_iter().map(|(t, _, _)| t).collect();
        assert_eq!(
            kinds,
            vec![
                Token::Int {
                    digits: "1".to_string(),
                    radix: 10
                },
                Token::DotDot,
                Token::Int {
                    digits: "2".to_string(),
                    radix: 10
                },
                Token::Eof,
            ]
        );
    }

    #[test]
    fn unterminated_string_is_an_error_not_a_panic() {
        let mut lexer = Lexer::new("\"oops");
        let first = lexer.next().expect("one item");
        assert!(matches!(first, Err(LexicalError::UnterminatedString(_))));
    }

    #[test]
    fn invalid_character_is_reported_with_its_span() {
        // `€` is a 3-byte UTF-8 scalar; the error span must cover the whole
        // character and the iterator must make progress past it.
        let items: Vec<_> = Lexer::new("€x").collect();
        assert!(matches!(
            items[0],
            Err(LexicalError::InvalidToken(ref span)) if span.start == 0 && span.end == 3
        ));
    }
}
