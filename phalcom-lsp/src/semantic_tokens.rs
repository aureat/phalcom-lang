//! `textDocument/semanticTokens/full` (Stage 5, ADR-0056, `docs/forge/units/
//! U-LSP/plan.md` "Stage 5"): a flat, lexer-driven token-coloring pass.
//!
//! The base pass walks [`phalcom_ast::lexer::Lexer`] token-by-token and
//! classifies each [`Token`] in isolation, with no AST context — it cannot
//! by itself distinguish a class/method **definition** name from a
//! **reference** to one. What it already strictly improves on the TextMate
//! grammar it will eventually demote (DEC-VSP-C, not yet flipped): exact
//! string/comment boundaries and no false-positive keyword matches inside
//! string literals, because it is driven by the real lexer rather than
//! regexes. A second, AST-assisted pass (below) closes the definition-vs-
//! reference gap for declaration names specifically.
//!
//! Comments (including Phaldoc `///`/`//!`) are invisible to this token
//! source — `Lexer::skip_trivia` discards them before they ever reach the
//! `Iterator<Item = Spanned<Token, ..>>` stream — so comment coloring is out
//! of scope for this stage; the TextMate grammar still covers it.
//!
//! ## AST-assisted refinement
//!
//! On top of the flat lexer pass, [`tokens_for`] runs a second, best-effort
//! pass: it parses the same text (via [`phalcom_ast::parser::parse`], which
//! recovers from syntax errors rather than aborting) and walks the resulting
//! [`phalcom_ast::ast::Program`] to find every
//! `class`/method/getter/setter/constructor **declaration name**'s own span
//! ([`ClassDef::name_range`](phalcom_ast::ast::ClassDef::name_range) and its
//! siblings on [`MethodDef`](phalcom_ast::ast::MethodDef)/
//! [`GetterDef`](phalcom_ast::ast::GetterDef)/
//! [`SetterDef`](phalcom_ast::ast::SetterDef)). Constructor declarations are
//! [`MethodDef`](phalcom_ast::ast::MethodDef) nodes marked during attribute
//! expansion. Any flat-pass token
//! whose byte range exactly matches one of these declaration spans is
//! upgraded from the generic [`SemanticTokenKind::Variable`] to
//! [`SemanticTokenKind::Class`] or [`SemanticTokenKind::Method`].
//!
//! This targeted upgrade — keying off the AST's own name-only span rather
//! than re-scanning the whole declaration's source text for the first
//! identifier matching the declared name — is deliberate: a name can appear
//! earlier in a declaration incidentally (e.g. inside a default-value
//! expression), so a text-search heuristic cannot reliably tell a
//! declaration's own name token from an unrelated occurrence. Keying off the
//! parser-recorded span is exact by construction. If parsing produces no
//! usable [`Program`] at all (e.g. the document is empty), the flat pass's
//! plain-`variable` coloring is left untouched — this refinement only ever
//! upgrades, never downgrades or removes, a flat-pass token.

use phalcom_ast::ast::{ClassMember, Expr, Statement};
use phalcom_ast::lexer::Lexer;
use phalcom_ast::token::{StringSegment, Token};
use phalcom_common::range::SourceRange;
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
    SemanticTokenType::CLASS,
    SemanticTokenType::METHOD,
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
    /// A `class` declaration's own name (the AST-assisted refinement pass's
    /// upgrade of the flat pass's [`Variable`](Self::Variable) at
    /// [`ClassDef::name_range`](phalcom_ast::ast::ClassDef::name_range) —
    /// see the module doc's "AST-assisted refinement" section).
    Class,
    /// A method/getter/setter/constructor declaration's own name (the
    /// AST-assisted refinement pass's upgrade of the flat pass's
    /// [`Variable`](Self::Variable) at the declaration's own `name_range` —
    /// see the module doc's "AST-assisted refinement" section).
    Method,
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
            SemanticTokenKind::Class => 6,
            SemanticTokenKind::Method => 7,
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
        | Token::Const
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
    apply_decl_name_overrides(text, &mut raw);
    encode(text, line_index, &raw)
}

/// Upgrades every flat-pass token in `raw` whose byte range exactly matches a
/// `class`/method/getter/setter/constructor declaration's own name span to
/// [`SemanticTokenKind::Class`]/[`SemanticTokenKind::Method`] — the
/// AST-assisted refinement pass described in the module doc.
///
/// Parses `text` via [`phalcom_ast::parser::parse`] (which recovers from
/// syntax errors rather than aborting) purely to collect declaration-name
/// spans; the flat lexer pass in `raw` is left as-is for every span this
/// walk does not find. `raw` must already be in the ascending source order
/// [`collect_tokens`] produces (this function only mutates in place, never
/// reorders).
fn apply_decl_name_overrides(text: &str, raw: &mut [RawToken]) {
    let parsed = phalcom_ast::parser::parse(text, 0);
    let mut decls = Vec::new();
    collect_decl_names(&parsed.program.statements, &mut decls);
    if decls.is_empty() {
        return;
    }
    // `decls` is small in practice (one entry per declaration in the file),
    // so a linear scan per raw token is simplest and fast enough; no need
    // for a `HashMap` keyed on `(start, end)`.
    for token in raw.iter_mut() {
        if let Some(&(_, kind)) = decls
            .iter()
            .find(|(range, _)| range.start == token.start && range.end == token.end)
        {
            token.kind = kind;
        }
    }
}

/// Recursively collects every `class`/method/getter/setter/constructor
/// declaration's own name span (and [`SemanticTokenKind`] override) reachable
/// from `statements`, appending to `out`.
///
/// Descends into class bodies, `for` loop bodies, and block-expression
/// bodies — the only [`Statement`]/[`Expr`] shapes that can themselves carry
/// a nested declaration. Every other statement/expression shape is a leaf for
/// this walk's purposes and is skipped.
fn collect_decl_names(statements: &[Statement], out: &mut Vec<(SourceRange, SemanticTokenKind)>) {
    for statement in statements {
        match statement {
            Statement::Class(class_def) => {
                out.push((class_def.name_range, SemanticTokenKind::Class));
                for member in &class_def.members {
                    collect_member_decl_name(member, out);
                }
            }
            Statement::For(for_stmt) => collect_decl_names(&for_stmt.body, out),
            Statement::Expr { expr, .. } => collect_decl_names_in_expr(expr, out),
            Statement::Let(_) | Statement::Return(_) | Statement::Break { .. } | Statement::Continue { .. } | Statement::Throw { .. } | Statement::Import(_) => {}
        }
    }
}

/// Pushes `member`'s own name-span override (if it has one — a
/// [`ClassMember::Field`]/[`ClassMember::Variant`] does not) and recurses
/// into its body, mirroring [`collect_decl_names`]'s class-body traversal.
fn collect_member_decl_name(member: &ClassMember, out: &mut Vec<(SourceRange, SemanticTokenKind)>) {
    match member {
        ClassMember::Method(method_def) => {
            out.push((method_def.name_range, SemanticTokenKind::Method));
            collect_decl_names(&method_def.body, out);
        }
        ClassMember::Getter(getter_def) => {
            out.push((getter_def.name_range, SemanticTokenKind::Method));
            collect_decl_names(&getter_def.body, out);
        }
        ClassMember::Setter(setter_def) => {
            out.push((setter_def.name_range, SemanticTokenKind::Method));
            collect_decl_names(&setter_def.body, out);
        }
        ClassMember::Field(_) | ClassMember::Variant(_) => {}
        ClassMember::Index(index_def) => {
            out.push((index_def.name_range, SemanticTokenKind::Method));
            collect_decl_names(&index_def.body, out);
        }
    }
}

/// Descends into an [`Expr::Block`]'s body — the only expression shape that
/// can itself carry a nested statement list (and therefore a nested
/// declaration) — for [`collect_decl_names`]'s `Statement::Expr` arm. Every
/// other [`Expr`] variant carries no nested statement list and is skipped.
fn collect_decl_names_in_expr(expr: &Expr, out: &mut Vec<(SourceRange, SemanticTokenKind)>) {
    if let Expr::Block(block_expr) = expr {
        collect_decl_names(&block_expr.body, out);
    }
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

    /// Like [`kinds`], but also runs the AST-assisted declaration-name
    /// refinement pass ([`apply_decl_name_overrides`]) before flattening —
    /// for asserting the `class`/`method` upgrade, not just the flat pass.
    fn kinds_with_decl_overrides(text: &str) -> Vec<SemanticTokenKind> {
        let mut raw = Vec::new();
        collect_tokens(text, 0, &mut raw);
        apply_decl_name_overrides(text, &mut raw);
        raw.into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn class_declaration_name_is_class_kind() {
        use SemanticTokenKind::{Class, Keyword};
        assert_eq!(
            kinds_with_decl_overrides("class Foo {\n}\n"),
            vec![Keyword, Class]
        );
    }

    #[test]
    fn method_declaration_name_is_method_kind() {
        use SemanticTokenKind::{Class, Keyword, Method, Variable};
        // `class Foo { bar(x) { return x } }` — `bar` is the method name
        // (Method), `x` in the parameter list and body is an ordinary
        // Variable, untouched by the refinement pass.
        assert_eq!(
            kinds_with_decl_overrides("class Foo {\n  bar(x) {\n    return x\n  }\n}\n"),
            vec![Keyword, Class, Method, Variable, Keyword, Variable]
        );
    }

    #[test]
    fn getter_and_construct_names_are_method_kind() {
        use SemanticTokenKind::{Class, Keyword, Method, Number};
        // `construct new()` and a bare-body getter `greeting { return 1 }` —
        // both `new` and `greeting` are declaration names, upgraded to
        // Method.
        assert_eq!(
            kinds_with_decl_overrides(
                "class Foo {\n  construct new() {\n  }\n  greeting {\n    return 1\n  }\n}\n"
            ),
            vec![Keyword, Class, Keyword, Method, Method, Keyword, Number]
        );
    }

    #[test]
    fn setter_name_is_method_kind() {
        use SemanticTokenKind::{Class, Keyword, Method, Operator, Variable};
        // `greeting = (v) { }` — a setter: `greeting` is the declaration
        // name (Method); `v` is an ordinary parameter binding, untouched.
        assert_eq!(
            kinds_with_decl_overrides("class Foo {\n  greeting = (v) {\n  }\n}\n"),
            vec![Keyword, Class, Method, Operator, Variable]
        );
    }

    #[test]
    fn variable_reference_to_a_class_name_is_not_upgraded() {
        // Only the declaration's own `name_range` is upgraded — an ordinary
        // reference to the class name elsewhere in the file (e.g. as a
        // superclass) stays a plain Variable, since it isn't the
        // *declaring* occurrence. `extends` itself is a contextual keyword
        // (DEC-INH-A) lexed as an ordinary identifier, so it also stays a
        // plain Variable here.
        use SemanticTokenKind::{Class, Keyword, Variable};
        assert_eq!(
            kinds_with_decl_overrides("class Foo {\n}\nclass Bar extends Foo {\n}\n"),
            vec![Keyword, Class, Keyword, Class, Variable, Variable]
        );
    }

    #[test]
    fn legend_index_matches_token_types_order() {
        assert_eq!(TOKEN_TYPES.len(), 8);
        assert_eq!(SemanticTokenKind::Keyword.legend_index(), 0);
        assert_eq!(SemanticTokenKind::Operator.legend_index(), 5);
        assert_eq!(SemanticTokenKind::Class.legend_index(), 6);
        assert_eq!(SemanticTokenKind::Method.legend_index(), 7);
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
