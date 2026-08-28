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
//! On top of the flat lexer pass, [`tokens_for_request`] runs a second,
//! best-effort
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
use crate::request_context::{RequestContext, SourceMatch};

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
/// literals, which have no natural fit among the standard LSP token types.
const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::KEYWORD,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::new("selector"),
    SemanticTokenType::OPERATOR,
    SemanticTokenType::CLASS,
    SemanticTokenType::METHOD,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::PROPERTY,
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
    /// A selector/symbol prefix (`Token::Hash`) or quoted symbol, emitted as
    /// a selector token. The parser owns selector-spec components following
    /// the hash and gives them their own source ranges.
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
    /// A parameter declaration or reference.
    Parameter,
    /// A field or property declaration or reference.
    Property,
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
            SemanticTokenKind::Parameter => 8,
            SemanticTokenKind::Property => 9,
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
        | Token::Where
        | Token::TypeKw
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
        | Token::From
        | Token::Export
        | Token::Expose
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

        Token::Identifier(_) | Token::FieldIdentifier(_) | Token::ImplementationFieldIdentifier(_) => Some(Variable),

        Token::ImplementationSelectorIdentifier(_) => Some(Selector),

        Token::Int { .. } | Token::Float(_) => Some(Number),

        Token::Hash | Token::QuotedSymbol(_) => Some(Selector),

        Token::Equal
        | Token::EqualEqual
        | Token::TripleEqual
        | Token::BangEqual
        | Token::Less
        | Token::LessEqual
        | Token::Greater
        | Token::GreaterEqual
        | Token::Subtype
        | Token::TypeLambdaArrow
        | Token::Spaceship
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
        | Token::DoubleAsterisk
        | Token::TripleAsterisk
        | Token::Slash
        | Token::Percent
        | Token::Power
        | Token::SlashTilde
        | Token::ShiftLeft
        | Token::ShiftRight
        | Token::Ampersand
        | Token::Pipe
        | Token::Caret
        | Token::Tilde => Some(Operator),

        // Structural punctuation, left uncolored (judgment call — see
        // `docs/forge/DEFERRED.md`).
        Token::LParen
        | Token::RParen
        | Token::LBrace
        | Token::RecordLBrace
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
        | Token::DotDotEqual
        | Token::DotDotDot
        | Token::Arrow
        | Token::Question
        | Token::At
        | Token::AtBang
        | Token::Underscore
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
    let mut attribute_marker = false;
    for item in Lexer::new(text) {
        let Ok((start, token, end)) = item else {
            // A lex error mid-recursion (e.g. an unterminated interpolation
            // body) contributes no token; diagnostics already surface it
            // separately (`crate::diagnostics`).
            continue;
        };
        match token {
            Token::StringInterp(segments) => {
                push_string_interp(&segments, offset, offset + start, offset + end, out);
            }
            Token::String(_) => out.push(RawToken {
                start: offset + start,
                end: offset + end,
                kind: SemanticTokenKind::String,
            }),
            other => {
                let kind = if attribute_marker && is_builtin_attribute_name(&other) {
                    Some(SemanticTokenKind::Keyword)
                } else {
                    classify(&other)
                };
                attribute_marker = matches!(other, Token::At);
                if let Some(kind) = kind {
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

/// Upgrades operator spellings immediately following a contiguous `#` to
/// selector tokens. The lexer intentionally emits the hash and operator as
/// separate tokens; the parser decides whether their adjacency forms a
/// symbol, so this pass mirrors that same source-boundary rule for editor
/// coloring. Whitespace-separated `# +` remains unchanged and is not a valid
/// parser symbol.
fn apply_symbol_operator_overrides(text: &str, raw: &mut Vec<RawToken>) {
    let operator_spellings = [
        "***", "**", "~/", "<<", ">>", "==", "!=", "<=", ">=", "?.", "??", "...", "+", "-", "*", "/", "%", "&", "|", "^", "~", "<", ">", "!", "?",
    ];
    let hash_ranges = raw
        .iter()
        .filter(|token| token.kind == SemanticTokenKind::Selector && text.as_bytes().get(token.start) == Some(&b'#'))
        .map(|token| (token.start, token.end))
        .collect::<Vec<_>>();
    for (_, hash_end) in hash_ranges {
        let Some(rest) = text.get(hash_end..) else { continue };
        let Some(spelling) = operator_spellings.iter().find(|spelling| rest.starts_with(**spelling)) else {
            continue;
        };
        let end = hash_end + spelling.len();
        let boundary = text
            .as_bytes()
            .get(end)
            .copied()
            .is_none_or(|byte| byte.is_ascii_whitespace() || matches!(byte, b'(' | b'[' | b'=' | b'.' | b',' | b')' | b']'));
        if !boundary {
            continue;
        }
        if let Some(token) = raw.iter_mut().find(|token| token.start == hash_end && token.end == end) {
            token.kind = SemanticTokenKind::Selector;
        } else {
            raw.push(RawToken {
                start: hash_end,
                end,
                kind: SemanticTokenKind::Selector,
            });
        }
    }
    raw.sort_by_key(|token| (token.start, token.end));
}

fn is_builtin_attribute_name(token: &Token) -> bool {
    match token {
        Token::Construct | Token::Class => true,
        Token::Identifier(name) => matches!(
            name.as_str(),
            "constructor"
                | "get"
                | "set"
                | "data"
                | "sealed"
                | "variant"
                | "invariant"
                | "requires"
                | "ensures"
                | "native"
                | "ignore"
                | "private"
                | "protected"
                | "On"
        ),
        _ => false,
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
fn push_string_interp(segments: &[StringSegment], input_offset: usize, token_start: usize, token_end: usize, out: &mut Vec<RawToken>) {
    let mut cursor = token_start;
    for segment in segments {
        let StringSegment::Expr { source, range } = segment else {
            continue;
        };
        // `range.start` is the byte offset of the expression body within the lexer input;
        // the `\(` opener itself is 2 bytes before it.
        let expr_open = input_offset + range.start;
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
        cursor = input_offset + range.end + 1;
    }
    if token_end > cursor {
        out.push(RawToken {
            start: cursor,
            end: token_end,
            kind: SemanticTokenKind::String,
        });
    }
}

/// Splits any [`RawToken`] spanning multiple lines into line-local fragments,
/// excluding line terminators (LF and CRLF) and skipping zero-length fragments.
fn line_localize(text: &str, raw: &[RawToken]) -> Vec<RawToken> {
    let mut out = Vec::with_capacity(raw.len());
    let bytes = text.as_bytes();
    for token in raw {
        if token.start >= token.end || token.start >= bytes.len() {
            continue;
        }
        let mut frag_start = token.start;
        let mut p = token.start;
        let limit = token.end.min(bytes.len());
        while p < limit {
            if bytes[p] == b'\n' || bytes[p] == b'\r' {
                if p > frag_start {
                    out.push(RawToken {
                        start: frag_start,
                        end: p,
                        kind: token.kind,
                    });
                }
                if bytes[p] == b'\r' && p + 1 < limit && bytes[p + 1] == b'\n' {
                    p += 2;
                } else {
                    p += 1;
                }
                frag_start = p;
            } else {
                p += 1;
            }
        }
        if limit > frag_start {
            out.push(RawToken {
                start: frag_start,
                end: limit,
                kind: token.kind,
            });
        }
    }
    out
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
    let localized = line_localize(text, raw);
    let mut result = Vec::with_capacity(localized.len());
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;
    for token in &localized {
        let start_pos = line_index.position(token.start);
        let length: u32 = text[token.start..token.end].chars().map(|c| c.len_utf16() as u32).sum();

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

/// Computes the full `semanticTokens/full` payload for `text`, delta-encoded
/// per the LSP wire format via `line_index`.
///
/// `line_index` must be built over the same `text` (callers pass a
/// [`crate::documents::Document`]'s cached pair, which are always rebuilt
/// together — see `documents.rs`).
/// Computes semantic tokens from one coherent document/generation pair.
pub fn tokens_for_request(request: &RequestContext) -> Vec<SemanticToken> {
    let mut raw = Vec::new();
    collect_tokens(&request.document.text, 0, &mut raw);
    apply_symbol_operator_overrides(&request.document.text, &mut raw);
    if matches!(request.source_match, SourceMatch::Exact)
        && let (Some(compiler), Some(module)) = (request.compiler.as_deref(), request.compiler_module())
        && let Some(source) = compiler.source_index.module(module)
    {
        apply_compiler_occurrence_overrides(source, &mut raw);
        apply_decl_name_overrides_program(&request.document.parse.program, &mut raw);
    } else {
        // The parse is owned by DocumentSnapshot. Never reparse live text in
        // this fallback: declaration ranges must come from the same source
        // revision as the line index and lexer input.
        apply_decl_name_overrides_program(&request.document.parse.program, &mut raw);
    }
    encode(&request.document.text, &request.document.line_index, &raw)
}

fn apply_compiler_occurrence_overrides(source: &phalcom_semantic::source_index::ModuleSourceIndex, raw: &mut [RawToken]) {
    let mut occurrences_map = std::collections::BTreeMap::new();
    for occurrence in source.occurrences.all() {
        occurrences_map.insert((occurrence.range.start, occurrence.range.end), occurrence.kind);
    }
    for token in raw.iter_mut() {
        if let Some(kind) = occurrences_map.get(&(token.start, token.end)) {
            token.kind = match kind {
                phalcom_semantic::source_index::OccurrenceKind::Parameter => SemanticTokenKind::Parameter,
                phalcom_semantic::source_index::OccurrenceKind::Binding => SemanticTokenKind::Variable,
                phalcom_semantic::source_index::OccurrenceKind::Field => SemanticTokenKind::Property,
                phalcom_semantic::source_index::OccurrenceKind::Member => SemanticTokenKind::Method,
                phalcom_semantic::source_index::OccurrenceKind::Declaration => SemanticTokenKind::Class,
                phalcom_semantic::source_index::OccurrenceKind::Module => SemanticTokenKind::Variable,
                phalcom_semantic::source_index::OccurrenceKind::Operator => SemanticTokenKind::Operator,
            };
        }
    }
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
/// [`collect_tokens`] produces; synthetic index-bracket tokens are appended
/// and the final sequence is resorted by source range.
fn apply_decl_name_overrides_program(program: &phalcom_ast::ast::Program, raw: &mut Vec<RawToken>) {
    let mut decls = Vec::new();
    collect_decl_names(&program.statements, &mut decls);
    if decls.is_empty() {
        return;
    }
    // `decls` is small in practice (one entry per declaration in the file),
    // so a linear scan per raw token is simplest and fast enough; no need
    // for a `HashMap` keyed on `(start, end)`.
    for token in raw.iter_mut() {
        if let Some(decl) = decls.iter().find(|decl| decl.range.start == token.start && decl.range.end == token.end) {
            token.kind = decl.kind;
        }
    }

    // Index declarations use their brackets as their complete name range;
    // the flat lexer intentionally leaves ordinary brackets uncolored. Emit
    // only those declaration brackets so subscript methods still have a
    // visible method token without recoloring ordinary indexing expressions.
    for decl in decls.iter().filter(|decl| decl.is_index) {
        if decl.range.end.saturating_sub(decl.range.start) >= 2 {
            raw.push(RawToken {
                start: decl.range.start,
                end: decl.range.start + 1,
                kind: decl.kind,
            });
            raw.push(RawToken {
                start: decl.range.end - 1,
                end: decl.range.end,
                kind: decl.kind,
            });
        }
    }
    raw.sort_by_key(|token| (token.start, token.end));
}

#[derive(Clone, Copy)]
struct DeclNameOverride {
    range: SourceRange,
    kind: SemanticTokenKind,
    is_index: bool,
}

/// Recursively collects every `class`/method/getter/setter/constructor
/// declaration's own name span (and [`SemanticTokenKind`] override) reachable
/// from `statements`, appending to `out`.
///
/// Descends into class bodies, `for` loop bodies, and block-expression
/// bodies — the only [`Statement`]/[`Expr`] shapes that can themselves carry
/// a nested declaration. Every other statement/expression shape is a leaf for
/// this walk's purposes and is skipped.
fn collect_decl_names(statements: &[Statement], out: &mut Vec<DeclNameOverride>) {
    for statement in statements {
        match statement {
            Statement::Class(class_def) => {
                out.push(DeclNameOverride {
                    range: class_def.name_range,
                    kind: SemanticTokenKind::Class,
                    is_index: false,
                });
                for member in &class_def.members {
                    collect_member_decl_name(member, out);
                }
            }
            Statement::TypeAlias(alias) => {
                out.push(DeclNameOverride {
                    range: alias.name_range,
                    kind: SemanticTokenKind::Class,
                    is_index: false,
                });
            }
            Statement::For(for_stmt) => collect_decl_names(&for_stmt.body, out),
            Statement::Expr { expr, .. } => collect_decl_names_in_expr(expr, out),
            Statement::Let(_)
            | Statement::Return(_)
            | Statement::Break { .. }
            | Statement::Continue { .. }
            | Statement::Throw { .. }
            | Statement::Export(_) => {}
        }
    }
}

/// Pushes `member`'s own name-span override (if it has one — a
/// [`ClassMember::Field`]/[`ClassMember::Variant`] does not) and recurses
/// into its body, mirroring [`collect_decl_names`]'s class-body traversal.
fn collect_member_decl_name(member: &ClassMember, out: &mut Vec<DeclNameOverride>) {
    match member {
        ClassMember::Method(method_def) => {
            out.push(DeclNameOverride {
                range: method_def.name_range,
                kind: SemanticTokenKind::Method,
                is_index: false,
            });
            collect_decl_names(method_def.body.statements().unwrap_or_default(), out);
        }
        ClassMember::Getter(getter_def) => {
            out.push(DeclNameOverride {
                range: getter_def.name_range,
                kind: SemanticTokenKind::Method,
                is_index: false,
            });
            collect_decl_names(getter_def.body.statements().unwrap_or_default(), out);
        }
        ClassMember::Setter(setter_def) => {
            out.push(DeclNameOverride {
                range: setter_def.name_range,
                kind: SemanticTokenKind::Method,
                is_index: false,
            });
            collect_decl_names(setter_def.body.statements().unwrap_or_default(), out);
        }
        ClassMember::Field(field_def) => {
            out.push(DeclNameOverride {
                range: field_def.name_range,
                kind: SemanticTokenKind::Property,
                is_index: false,
            });
        }
        ClassMember::Variant(_) => {}
        ClassMember::Index(index_def) => {
            out.push(DeclNameOverride {
                range: index_def.name_range,
                kind: SemanticTokenKind::Method,
                is_index: true,
            });
            collect_decl_names(&index_def.body, out);
        }
    }
}

/// Descends into an [`Expr::Block`]'s body — the only expression shape that
/// can itself carry a nested statement list (and therefore a nested
/// declaration) — for [`collect_decl_names`]'s `Statement::Expr` arm. Every
/// other [`Expr`] variant carries no nested statement list and is skipped.
fn collect_decl_names_in_expr(expr: &Expr, out: &mut Vec<DeclNameOverride>) {
    if let Expr::Block(block_expr) = expr {
        collect_decl_names(&block_expr.body, out);
    }
}
