//! `textDocument/semanticTokens/full` (Stage 5, ADR-0056, `docs/forge/units/
//! U-LSP/plan.md` "Stage 5"): a flat, lexer-driven token-coloring pass.
//!
//! This is deliberately the **flat** pass only: it walks
//! [`phalcom_ast::lexer::Lexer`] token-by-token and classifies each
//! [`Token`] in isolation, with no AST context. It cannot yet distinguish a
//! class/method **definition** name from a **reference** to one (that would
//! need the cached [`phalcom_ast::parser::Parse`] tree, an optional
//! AST-assisted refinement pass left to a later commit within this stage —
//! see the module's design note in the U-LSP plan). What it already
//! strictly improves on the TextMate grammar it will eventually demote
//! (DEC-VSP-C, not yet flipped): exact string/comment boundaries and no
//! false-positive keyword matches inside string literals, because it is
//! driven by the real lexer rather than regexes.
//!
//! Comments (including Phaldoc `///`/`//!`) are invisible to this token
//! source — `Lexer::skip_trivia` discards them before they ever reach the
//! `Iterator<Item = Spanned<Token, ..>>` stream — so comment coloring is out
//! of scope for this stage; the TextMate grammar still covers it.

use phalcom_ast::lexer::Lexer;
use phalcom_ast::token::{StringSegment, Token};
use tower_lsp::lsp_types::{SemanticToken, SemanticTokenType, SemanticTokensLegend};

use crate::line_index::LineIndex;

/// The semantic token types this server declares, in legend order.
///
/// The index of a [`SemanticTokenKind`] variant in this array *is* the
/// wire-format `token_type` index [`encode`] writes into each
/// [`SemanticToken`] — the two must stay in lock-step, which is why
/// [`SemanticTokenKind::legend_index`] is the only place that indexes into
/// it.
///
/// `"selector"` is a server-defined custom token type (legal per the LSP
/// spec — `SemanticTokensLegend.token_types` is server-declared, not
/// restricted to the standard set) for Phalcom's `#name`/`#sel(...)` symbol
/// literals ([`Token::NameSymbol`], [`Token::SelectorSymbol`]), which have no
/// natural fit among the standard LSP token types.
const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::KEYWORD,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::new("selector"),
    SemanticTokenType::OPERATOR,
];

/// A classified token's semantic kind, corresponding 1:1 to a slot in
/// [`TOKEN_TYPES`].
///
/// No [`SemanticTokenModifier`](tower_lsp::lsp_types::SemanticTokenModifier)s
/// are emitted in this first cut (empty legend) — see the module-level
/// judgment call recorded in `docs/forge/DEFERRED.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SemanticTokenKind {
    /// A reserved keyword (`let`, `if`, `class`, `self`, `and`, …).
    Keyword,
    /// An [`Token::Identifier`] — a variable, field, class, or method name.
    /// The flat pass cannot yet distinguish these roles from each other.
    Variable,
    /// A string literal (whole [`Token::String`], or one literal run /
    /// delimiter span of a [`Token::StringInterp`]).
    String,
    /// A [`Token::Number`] literal.
    Number,
    /// A `#name` or `#sel(...)` symbol literal ([`Token::NameSymbol`],
    /// [`Token::SelectorSymbol`]), emitted as **one** token spanning the
    /// whole literal — never split into a `#`-punctuation token plus an
    /// identifier.
    Selector,
    /// A binary/unary/compound-assignment operator (`+`, `==`, `+=`, `??`,
    /// …). Structural punctuation (parens, braces, brackets, comma, dot,
    /// colon, arrows) is deliberately left uncolored — see the module's
    /// recorded judgment call.
    Operator,
}

impl SemanticTokenKind {
    /// The index of this kind's [`SemanticTokenType`] in [`TOKEN_TYPES`],
    /// i.e. the wire-format `token_type` [`encode`] writes.
    fn legend_index(self) -> u32 {
        match self {
            SemanticTokenKind::Keyword => 0,
            SemanticTokenKind::Variable => 1,
            SemanticTokenKind::String => 2,
            SemanticTokenKind::Number => 3,
            SemanticTokenKind::Selector => 4,
            SemanticTokenKind::Operator => 5,
        }
    }
}

/// Builds the [`SemanticTokensLegend`] this server advertises at
/// `initialize` time (`capabilities.semantic_tokens_provider.legend`).
///
/// No token modifiers are declared for this first cut (see the module doc).
pub fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TOKEN_TYPES.to_vec(),
        token_modifiers: Vec::new(),
    }
}

/// Classifies one non-string [`Token`], returning its [`SemanticTokenKind`]
/// or `None` if the token is deliberately left uncolored (structural
/// punctuation, [`Token::Newline`], [`Token::Eof`]).
///
/// [`Token::String`] and [`Token::StringInterp`] are **not** handled here —
/// callers must intercept them before reaching this function, since
/// [`Token::StringInterp`] expands to more than one emitted range (see
/// [`push_string_interp`]).
fn classify(token: &Token) -> Option<SemanticTokenKind> {
    use SemanticTokenKind::{Keyword, Number, Operator, Selector, Variable};
    match token {
        Token::Let
        | Token::Var
        | Token::Fn
        | Token::Class
        | Token::Return
        | Token::True
        | Token::False
        | Token::If
        | Token::Else
        | Token::While
        | Token::For
        | Token::Break
        | Token::Continue
        | Token::Import
        | Token::SelfKw
        | Token::Super
        | Token::In
        | Token::As
        | Token::Is
        | Token::And
        | Token::Or
        | Token::Not
        | Token::Static
        | Token::Construct
        | Token::Throw
        | Token::Try => Some(Keyword),

        Token::Identifier(_) => Some(Variable),

        Token::Number(_) => Some(Number),

        Token::NameSymbol(_) | Token::SelectorSymbol { .. } => Some(Selector),

        Token::Equal
        | Token::EqualEqual
        | Token::BangEqual
        | Token::Less
        | Token::LessEqual
        | Token::Greater
        | Token::GreaterEqual
        | Token::PlusEqual
        | Token::MinusEqual
        | Token::AsteriskEqual
        | Token::SlashEqual
        | Token::PercentEqual
        | Token::CoalesceQuestion
        | Token::QuestionDot
        | Token::Bang
        | Token::Plus
        | Token::Minus
        | Token::Asterisk
        | Token::Slash
        | Token::Percent => Some(Operator),

        // Structural punctuation, left uncolored (judgment call — see
        // `docs/forge/DEFERRED.md`).
        Token::LParen
        | Token::RParen
        | Token::LBrace
        | Token::RBrace
        | Token::LBracket
        | Token::RBracket
        | Token::Semicolon
        | Token::Newline
        | Token::Colon
        | Token::ColonColon
        | Token::Comma
        | Token::Dot
        | Token::DotDot
        | Token::DotDotDot
        | Token::Arrow
        | Token::FatArrow
        | Token::Question
        | Token::At
        | Token::Eof => None,

        // Handled by the caller before `classify` is reached.
        Token::String(_) | Token::StringInterp(_) => None,
    }
}

/// One classified token, in absolute byte-offset coordinates into the
/// top-level document text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RawToken {
    /// Half-open byte-range start, absolute into the document text.
    start: usize,
    /// Half-open byte-range end, absolute into the document text.
    end: usize,
    /// The classified kind.
    kind: SemanticTokenKind,
}

/// Lexes `text` and appends every classified [`RawToken`] to `out`, in
/// ascending source order.
///
/// `offset` is added to every byte position the lexer reports, so this can
/// be called recursively on a `\(expr)` interpolation body (whose own
/// [`Lexer`] runs over just `expr`'s source, position 0) and still produce
/// absolute offsets into the original document.
fn collect_tokens(text: &str, offset: usize, out: &mut Vec<RawToken>) {
    for item in Lexer::new(text) {
        let Ok((start, token, end)) = item else {
            // A lex error mid-recursion (e.g. an unterminated interpolation
            // body) contributes no token; diagnostics already surface it
            // separately (`crate::diagnostics`).
            continue;
        };
        match token {
            Token::StringInterp(segments) => {
                push_string_interp(&segments, offset + start, offset + end, out);
            }
            Token::String(_) => out.push(RawToken {
                start: offset + start,
                end: offset + end,
                kind: SemanticTokenKind::String,
            }),
            other => {
                if let Some(kind) = classify(&other) {
                    out.push(RawToken {
                        start: offset + start,
                        end: offset + end,
                        kind,
                    });
                }
            }
        }
    }
}

/// Expands one [`Token::StringInterp`] into its constituent ranges:
/// alternating `string`-colored literal/delimiter runs and recursively
/// classified `\(expr)` bodies.
///
/// [`StringSegment::Literal`] runs don't carry their own byte span (escape
/// decoding changes their length relative to the source), so the literal
/// **gaps** between consecutive [`StringSegment::Expr`] sub-ranges (and
/// between the string's opening/closing quotes and the first/last
/// interpolation) are colored `string` instead; each `Expr` body is
/// recursed into via [`collect_tokens`] so its own tokens classify as their
/// own kinds (e.g. `"a \(x + 1) b"` yields `string`, `x`, `+`, `1`,
/// `string` — not one giant `string` token).
fn push_string_interp(
    segments: &[StringSegment],
    token_start: usize,
    token_end: usize,
    out: &mut Vec<RawToken>,
) {
    let mut cursor = token_start;
    for segment in segments {
        let StringSegment::Expr { source, start } = segment else {
            continue;
        };
        // `start` is the absolute byte offset of the expression body (right
        // after the `\(`); the `\(` opener itself is 2 bytes before it.
        let expr_open = *start;
        let backslash_paren = expr_open.saturating_sub(2);
        if backslash_paren > cursor {
            out.push(RawToken {
                start: cursor,
                end: backslash_paren,
                kind: SemanticTokenKind::String,
            });
        }
        collect_tokens(source, expr_open, out);
        // Past the expression body and its closing `)`.
        cursor = expr_open + source.len() + 1;
    }
    if token_end > cursor {
        out.push(RawToken {
            start: cursor,
            end: token_end,
            kind: SemanticTokenKind::String,
        });
    }
}

/// Computes the full `semanticTokens/full` payload for `text`, delta-encoded
/// per the LSP wire format via `line_index`.
///
/// `line_index` must be built over the same `text` (callers pass a
/// [`crate::documents::Document`]'s cached pair, which are always rebuilt
/// together — see `documents.rs`).
pub fn tokens_for(text: &str, line_index: &LineIndex) -> Vec<SemanticToken> {
    let mut raw = Vec::new();
    collect_tokens(text, 0, &mut raw);
    encode(text, line_index, &raw)
}

/// Encodes classified, absolute-offset [`RawToken`]s into the LSP
/// delta-encoded [`SemanticToken`] wire format.
///
/// Each token's `delta_line`/`delta_start` are relative to the **previous**
/// token in `raw` (zero for the first), per the LSP spec; `raw` must already
/// be in ascending source order ([`collect_tokens`] preserves this — it
/// never reorders the lexer's own output, including its recursive
/// interpolation expansion).
fn encode(text: &str, line_index: &LineIndex, raw: &[RawToken]) -> Vec<SemanticToken> {
    let mut result = Vec::with_capacity(raw.len());
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;
    for token in raw {
        let start_pos = line_index.position(token.start);
        let length: u32 = text[token.start..token.end]
            .chars()
            .map(|c| c.len_utf16() as u32)
            .sum();

        let delta_line = start_pos.line - prev_line;
        let delta_start = if delta_line == 0 {
            start_pos.character - prev_start
        } else {
            start_pos.character
        };

        result.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type: token.kind.legend_index(),
            token_modifiers_bitset: 0,
        });

        prev_line = start_pos.line;
        prev_start = start_pos.character;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Classifies `text` end-to-end (lex → recurse interpolations → drop
    /// position info) into a flat sequence of [`SemanticTokenKind`]s, for
    /// table-driven assertions independent of delta-encoding.
    fn kinds(text: &str) -> Vec<SemanticTokenKind> {
        let mut raw = Vec::new();
        collect_tokens(text, 0, &mut raw);
        raw.into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn keyword_identifier_number_string_operator() {
        use SemanticTokenKind::{Keyword, Number, Operator, String, Variable};
        assert_eq!(
            kinds("let x = 1 + \"a\""),
            vec![Keyword, Variable, Operator, Number, Operator, String]
        );
    }

    #[test]
    fn punctuation_and_newline_are_uncolored() {
        assert_eq!(kinds("f(x, y)\n"), vec![
            SemanticTokenKind::Variable,
            SemanticTokenKind::Variable,
            SemanticTokenKind::Variable,
        ]);
    }

    #[test]
    fn bare_name_symbol_is_one_selector_token() {
        assert_eq!(kinds("#move"), vec![SemanticTokenKind::Selector]);
    }

    #[test]
    fn selector_symbol_with_labels_is_one_selector_token() {
        assert_eq!(
            kinds("#move(_,to,duration)"),
            vec![SemanticTokenKind::Selector]
        );
    }

    #[test]
    fn string_interpolation_recurses_into_expression_body() {
        use SemanticTokenKind::{Number, Operator, String, Variable};
        assert_eq!(
            kinds(r#""a \(x + 1) b""#),
            vec![String, Variable, Operator, Number, String]
        );
    }

    #[test]
    fn string_interpolation_with_no_literal_text_still_colors_quotes() {
        // No literal run at all — the opening and closing quote each still
        // form their own zero-literal `string` gap token around the single
        // interpolation.
        use SemanticTokenKind::{String, Variable};
        assert_eq!(kinds(r#""\(x)""#), vec![String, Variable, String]);
    }

    #[test]
    fn plain_string_with_no_interpolation_is_one_token() {
        assert_eq!(kinds(r#""hello""#), vec![SemanticTokenKind::String]);
    }

    #[test]
    fn legend_index_matches_token_types_order() {
        assert_eq!(TOKEN_TYPES.len(), 6);
        assert_eq!(SemanticTokenKind::Keyword.legend_index(), 0);
        assert_eq!(SemanticTokenKind::Operator.legend_index(), 5);
    }

    #[test]
    fn encode_produces_deltas_relative_to_previous_token() {
        // "let x = 1\ny" -> classified tokens: let(0,3) x(4,5) =(6,7) 1(8,9)
        // y(10,11) (Newline at 9,10 is uncolored and contributes no token).
        let text = "let x = 1\ny";
        let line_index = LineIndex::new(text);
        let mut raw = Vec::new();
        collect_tokens(text, 0, &mut raw);
        let tokens = encode(text, &line_index, &raw);
        assert_eq!(tokens.len(), 5);

        // let: line 0, char 0, length 3
        assert_eq!(tokens[0].delta_line, 0);
        assert_eq!(tokens[0].delta_start, 0);
        assert_eq!(tokens[0].length, 3);

        // x: same line, char 4 -> delta_start = 4 - 0 = 4
        assert_eq!(tokens[1].delta_line, 0);
        assert_eq!(tokens[1].delta_start, 4);
        assert_eq!(tokens[1].length, 1);

        // =: same line, char 6 -> delta_start = 6 - 4 = 2
        assert_eq!(tokens[2].delta_line, 0);
        assert_eq!(tokens[2].delta_start, 2);
        assert_eq!(tokens[2].length, 1);

        // 1: same line, char 8 -> delta_start = 8 - 6 = 2
        assert_eq!(tokens[3].delta_line, 0);
        assert_eq!(tokens[3].delta_start, 2);
        assert_eq!(tokens[3].length, 1);

        // y: next line -> delta_line = 1, delta_start = absolute char (0)
        assert_eq!(tokens[4].delta_line, 1);
        assert_eq!(tokens[4].delta_start, 0);
        assert_eq!(tokens[4].length, 1);
    }

    #[test]
    fn tokens_for_matches_manual_encode() {
        let text = "let x = 1\n";
        let line_index = LineIndex::new(text);
        let tokens = tokens_for(text, &line_index);
        assert_eq!(tokens.len(), 4); // let, x, =, 1 (Newline is uncolored)
    }
}
