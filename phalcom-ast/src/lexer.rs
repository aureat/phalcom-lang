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

use crate::token::{LexicalError, Token};

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
}

impl<'input> Lexer<'input> {
    /// Creates a scanner over `input`, positioned at the first byte.
    pub fn new(input: &'input str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            pos: 0,
            last_end: 0,
            eof_emitted: false,
        }
    }

    /// Returns the byte `n` positions ahead of the cursor, if any.
    fn peek_at(&self, n: usize) -> Option<u8> {
        self.bytes.get(self.pos + n).copied()
    }

    /// Skips spaces, tabs, form feeds, lone carriage returns, and `//` line
    /// comments.
    ///
    /// Newlines are intentionally *not* skipped: they are meaningful
    /// [`Token::Newline`] tokens. A `\r` is only skipped when it is not part of
    /// a `\r\n` newline.
    fn skip_trivia(&mut self) {
        while let Some(b) = self.peek_at(0) {
            match b {
                b' ' | b'\t' | b'\x0c' => self.pos += 1,
                // A lone carriage return (not part of a `\r\n` newline).
                b'\r' if self.peek_at(1) != Some(b'\n') => self.pos += 1,
                // Line comment: consume `//` and everything up to (not
                // including) the terminating newline.
                b'/' if self.peek_at(1) == Some(b'/') => {
                    self.pos += 2;
                    while let Some(c) = self.peek_at(0) {
                        if c == b'\n' {
                            break;
                        }
                        self.pos += 1;
                    }
                }
                _ => return,
            }
        }
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
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => Ok(self.scan_identifier()),
            b'"' => self.scan_string(),
            _ => self.scan_operator(),
        }
    }

    /// Scans a numeric literal (`[0-9]+(\.[0-9]+)?`) and decodes it to [`f64`].
    ///
    /// A `.` is only consumed as a decimal point when it is followed by a digit,
    /// so `1..2` and `3.method` tokenise correctly.
    ///
    /// # Errors
    ///
    /// Returns [`LexicalError::InvalidFloat`] if the matched slice fails to
    /// parse as an [`f64`] (not reachable for the matched grammar, but surfaced
    /// rather than panicking).
    fn scan_number(&mut self) -> Result<Token, LexicalError> {
        let start = self.pos;
        while matches!(self.peek_at(0), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if self.peek_at(0) == Some(b'.') && matches!(self.peek_at(1), Some(b'0'..=b'9')) {
            self.pos += 1;
            while matches!(self.peek_at(0), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        let slice = &self.input[start..self.pos];
        Ok(Token::Number(slice.parse::<f64>()?))
    }

    /// Scans an identifier (`[A-Za-z_][A-Za-z0-9_]*`), resolving keywords.
    ///
    /// Field names (a leading `_`) also lex as [`Token::Identifier`]; the parser
    /// decides between a variable and a field reference.
    fn scan_identifier(&mut self) -> Token {
        let start = self.pos;
        while matches!(
            self.peek_at(0),
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
        ) {
            self.pos += 1;
        }
        let slice = &self.input[start..self.pos];
        match slice {
            "let" => Token::Let,
            "var" => Token::Var,
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
            "self" => Token::SelfKw,
            "super" => Token::Super,
            "in" => Token::In,
            "as" => Token::As,
            "is" => Token::Is,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            "static" => Token::Static,
            _ => Token::Identifier(slice.to_string()),
        }
    }

    /// Scans a double-quoted string literal, stripping the surrounding quotes.
    ///
    /// Any byte except `"` is accepted in the body (including newlines), at
    /// parity with the previous `"[^"]*"` lexeme.
    ///
    /// # Errors
    ///
    /// Returns [`LexicalError::UnterminatedString`] if end-of-input is reached
    /// before the closing quote.
    fn scan_string(&mut self) -> Result<Token, LexicalError> {
        let open = self.pos;
        self.pos += 1;
        let content_start = self.pos;
        while let Some(b) = self.peek_at(0) {
            if b == b'"' {
                let value = self.input[content_start..self.pos].to_string();
                self.pos += 1;
                return Ok(Token::String(value));
            }
            self.pos += 1;
        }
        Err(LexicalError::UnterminatedString(open..self.pos))
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
            b'*' => (1, Token::Asterisk),
            b'/' if next == Some(b'=') => (2, Token::SlashEqual),
            b'/' => (1, Token::Slash),
            b'%' if next == Some(b'=') => (2, Token::PercentEqual),
            b'%' => (1, Token::Percent),
            b'=' if next == Some(b'=') => (2, Token::EqualEqual),
            b'=' if next == Some(b'>') => (2, Token::FatArrow),
            b'=' => (1, Token::Equal),
            b'!' if next == Some(b'=') => (2, Token::BangEqual),
            b'!' => (1, Token::Bang),
            b'<' if next == Some(b'=') => (2, Token::LessEqual),
            b'<' => (1, Token::Less),
            b'>' if next == Some(b'=') => (2, Token::GreaterEqual),
            b'>' => (1, Token::Greater),
            b':' if next == Some(b':') => (2, Token::ColonColon),
            b':' => (1, Token::Colon),
            b'.' if next == Some(b'.') && self.peek_at(2) == Some(b'.') => (3, Token::DotDotDot),
            b'.' if next == Some(b'.') => (2, Token::DotDot),
            b'.' => (1, Token::Dot),
            b'(' => (1, Token::LParen),
            b')' => (1, Token::RParen),
            b'{' => (1, Token::LBrace),
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
            b'@' => (1, Token::At),
            _ => {
                // Unknown character: advance past the whole UTF-8 scalar so the
                // iterator makes progress, and report its exact byte span.
                let ch_len = self.input[self.pos..]
                    .chars()
                    .next()
                    .map_or(1, char::len_utf8);
                let span = self.pos..self.pos + ch_len;
                self.pos += ch_len;
                return Err(LexicalError::InvalidToken(span));
            }
        };
        self.pos += len;
        Ok(token)
    }
}

impl Iterator for Lexer<'_> {
    type Item = Spanned<Token, usize, LexicalError>;

    /// Yields the next token, or the single injected [`Token::Eof`] once the
    /// source is exhausted, then `None`.
    fn next(&mut self) -> Option<Self::Item> {
        self.skip_trivia();

        if self.pos >= self.bytes.len() {
            if self.eof_emitted {
                return None;
            }
            self.eof_emitted = true;
            return Some(Ok((self.last_end, Token::Eof, self.last_end)));
        }

        let start = self.pos;
        match self.scan_token() {
            Ok(token) => {
                let end = self.pos;
                self.last_end = end;
                Some(Ok((start, token, end)))
            }
            Err(err) => Some(Err(err)),
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
        assert_eq!(
            toks,
            vec![(Token::Identifier("x".to_string()), 0, 1), (Token::Eof, 1, 1)]
        );
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
            vec![Token::Number(1.0), Token::Newline, Token::Number(2.0), Token::Eof]
        );
    }

    #[test]
    fn maximal_munch_prefers_longer_operators() {
        let kinds: Vec<Token> = spans("a += 1 .. 2 ... 3 -> 4")
            .into_iter()
            .map(|(t, _, _)| t)
            .collect();
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
            vec![Token::Number(1.0), Token::DotDot, Token::Number(2.0), Token::Eof]
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
