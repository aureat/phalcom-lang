//! `textDocument/hover` (Stage 4, ADR-0056 §4, `docs/forge/units/U-LSP/plan.md`
//! "Stage 4").
//!
//! Three independent sources compose into one hover, per the plan:
//!
//! 1. **Keyword blurbs** ([`keyword_at_offset`] / [`keyword_blurb`]) — a
//!    closed static map covering every keyword [`phalcom_ast::token::Token`]
//!    variant plus the contextual `extends`/`on`/`catch`/`ensure`/`match`
//!    words that lex as [`phalcom_ast::token::Token::Identifier`] but are
//!    meaningful only in context. Ported (where present) verbatim from
//!    `tools/vsphalcom/src/hover.ts`'s `keywordDocs` — that file remains the
//!    source of truth for the prose and is deleted once this module is
//!    green (deferred to a follow-up, sequenced edit per the plan).
//! 2. **Selector signature + kind + defining class** — resolved by the
//!    caller (`crate::backend::Backend::hover_at`) from
//!    [`crate::index::WorkspaceIndex::definition_info`] (user classes) and
//!    the live semantic core surface (builtins), then rendered
//!    by [`render_selector_hover`].
//! 3. **Phaldoc harvest** ([`harvest_doc_for_selector`]) — a raw re-scan of
//!    the defining file's source text for `///` doc blocks, per
//!    `docs/spec/v0.2/experimental/doc-comments-phaldoc.md` §1-5. Phaldoc is
//!    lexically inert (ordinary `//` trivia, §Context), so the parser never
//!    sees it; this is a second, cheap pass over raw text, not a second
//!    parser, keying each block by the **selector** of the declaration
//!    immediately below it (§4) via a single range-containment lookup into
//!    the already-cached [`phalcom_ast::ast::Program`].
//!
//! A [`render_contract_view`] seam is kept — currently inert — so
//! U-ANNOT-CONTRACTS' `@requires`/`@ensures` rendering
//! (doc-comments-phaldoc.md §8) drops in later without reshaping this
//! module's composition path.

use phalcom_ast::ast::{ClassMember, Pattern, Program, Statement};
use phalcom_ast::lexer::Lexer;
use phalcom_ast::token::Token;

use crate::line_index::LineIndex;
use crate::selectors::class_member_selector;
use crate::semantic::{Confidence, InferredValue, ValueShape};

/// The contextual (non-keyword-token) words that carry their own hover blurb
/// only by convention, not by reserved-word status: they lex as
/// [`Token::Identifier`], not a dedicated `Token` variant
/// (error-handling.md §4, iteration.md, object-model.md's Option/Result
/// eliminator).
const CONTEXTUAL_WORDS: &[&str] = &["on", "catch", "ensure", "match"];

/// One-line blurbs for every keyword [`Token`] variant, plus
/// [`CONTEXTUAL_WORDS`], keyed by their exact surface spelling.
///
/// Entries already present in `tools/vsphalcom/src/hover.ts`'s `keywordDocs`
/// (`class`, `extends`, `super`, `self`, `static`, `try`, `catch`, `on`,
/// `ensure`, `throw`, `break`, `continue`, `match`, `return`, `while`, `for`,
/// `var`) are ported **verbatim** — that file is the source of truth for
/// this prose. The remaining keyword variants (`let`, `fn`, `true`, `false`,
/// `if`, `else`, `import`, `in`, `as`, `is`, `and`, `or`, `not`, `construct`)
/// have no `hover.ts` entry and are authored here, grounded in
/// `docs/spec/v0.2/lexical-structure.md`, `control-flow.md`, `classes.md`,
/// `modules.md`, and the U-IS/U-NEG units.
const KEYWORD_DOCS: &[(&str, &str)] = &[
    (
        "class",
        "Declares a class: a name, an optional superclass via `is` (e.g. `class Sub is Super`), fields, constructors, and methods.",
    ),
    (
        "super",
        "Refers to the receiver viewed through its superclass's method dictionary — used for `super.foo(...)` sends and `super(...)` constructor delegation.",
    ),
    (
        "self",
        "The current receiver inside a method or constructor body — the object the message was sent to.",
    ),
    (
        "static",
        "Retired class-side member syntax. Use the `@class` placement attribute on the following member.",
    ),
    (
        "try",
        "Starts a block-handler statement that is pure sugar over `Block` sends (`on(_)(_)`, `ensure(_)`, `attempt()`) — not a generic try/catch, it carries no semantics the desugaring lacks (lexical-structure.md §13).",
    ),
    (
        "catch",
        "Catch-all clause of a `try` statement, sugar for `.on(Error){ e => ... }`; a contextual keyword, reserved only inside `try`-clauses.",
    ),
    (
        "on",
        "Typed handler clause of a `try` statement (`on T e { ... }`), sugar for `.on(T){ e => ... }`; a contextual keyword, reserved only inside `try`-clauses.",
    ),
    (
        "ensure",
        "Cleanup clause of a `try` statement, sugar for `.ensure{ ... }`, always runs; a contextual keyword, reserved only inside `try`-clauses.",
    ),
    ("throw", "Prefix statement/expression that raises an `Error` value — sugar for `expr.raise()`."),
    (
        "break",
        "Loop-control keyword: leaves the enclosing `for`/`while` loop immediately (lowers to a direct jump, not a block send).",
    ),
    (
        "continue",
        "Loop-control keyword: skips to the next iteration step of the enclosing `for`/`while` loop (re-runs the cursor's `iterate(_)` step).",
    ),
    (
        "match",
        "Names the Option/Result eliminator selector (`match(some,none)` / `match(ok,err)`) that leaves Option/Result-world with a value; reserved as a keyword for pattern-matching surface syntax.",
    ),
    ("return", "Returns a value from the enclosing method/constructor/block explicitly."),
    (
        "while",
        "Loops while its condition evaluates truthy; `for` desugars to a `while` loop over the iteration protocol.",
    ),
    (
        "for",
        "Iterates a collection via the cursor protocol: `for (x in coll) { ... }` desugars to a `while` loop calling `iterate(_)`/`iteratorValue(_)`, so `break`/`continue` work as loop-control.",
    ),
    (
        "let",
        "Introduces a mutable binding; can be reassigned. `let x` with no initializer reads as `None` (there is no uninitialized-variable error).",
    ),
    (
        "const",
        "Introduces an immutable binding; must be initialized (unlike `let`), and cannot be reassigned (ADR-0064).",
    ),
    (
        "fn",
        "Reserved for a future function-literal form; not yet consumed by the parser — closures are written today as `{ ... }` block literals.",
    ),
    ("true", "The boolean literal `true`."),
    ("false", "The boolean literal `false`."),
    (
        "if",
        "Starts a conditional statement — keyword sugar for `c.ifTrue { ... }.ifNone { ... }`; compiles to the same opcodes as the message-send form (control-flow.md §1).",
    ),
    (
        "else",
        "The alternative clause of an `if` statement — part of the same `ifTrue`/`ifFalse` sugar (control-flow.md §1).",
    ),
    (
        "import",
        "Imports a module (file) into scope, optionally under an alias: `import \"./path\" as Name` (modules.md).",
    ),
    ("in", "Introduces the iterable of a `for` loop: `for (x in coll) { ... }` (iteration.md §2)."),
    ("as", "Binds an import under an alias: `import \"path\" as Name` (modules.md)."),
    (
        "is",
        "Class inheritance header keyword (`class Sub is Super`) and type-test operator (`x is Type`, `x is not Type`).",
    ),
    (
        "and",
        "Short-circuiting conjunction — sugar for `a.and { b }` (selector `and(_)`); the right-hand side is a block so it is evaluated lazily (control-flow.md §2).",
    ),
    (
        "or",
        "Short-circuiting disjunction — sugar for `a.or { b }` (selector `or(_)`); the right-hand side is a block so it is evaluated lazily (control-flow.md §2).",
    ),
    (
        "not",
        "Boolean negation, unified on the `not` keyword (U-NEG): `not b` sends the negation selector to `b`.",
    ),
    (
        "construct",
        "Introduces a named constructor definition inside a class body (`@constructor new(...) { ... }`) — Phalcom has no implicit `new`; multiple constructors are distinguished by selector, not arity (classes.md §1, ADR-0011).",
    ),
];

/// Maps a keyword [`Token`] variant to its exact surface spelling, or `None`
/// if `token` is not a keyword (or is a contextual word lexed as
/// [`Token::Identifier`] — handled separately by [`keyword_at_offset`]).
fn keyword_spelling(token: &Token) -> Option<&'static str> {
    match token {
        Token::Let => Some("let"),
        Token::Const => Some("const"),
        Token::Fn => Some("fn"),
        Token::Class => Some("class"),
        Token::Return => Some("return"),
        Token::True => Some("true"),
        Token::False => Some("false"),
        Token::If => Some("if"),
        Token::Else => Some("else"),
        Token::While => Some("while"),
        Token::For => Some("for"),
        Token::Break => Some("break"),
        Token::Continue => Some("continue"),
        Token::Import => Some("import"),
        Token::SelfKw => Some("self"),
        Token::Super => Some("super"),
        Token::In => Some("in"),
        Token::As => Some("as"),
        Token::Is => Some("is"),
        Token::And => Some("and"),
        Token::Or => Some("or"),
        Token::Not => Some("not"),
        Token::Static => Some("static"),
        Token::Construct => Some("construct"),
        Token::Throw => Some("throw"),
        Token::Try => Some("try"),
        Token::Identifier(name) => CONTEXTUAL_WORDS.iter().find(|word| **word == name.as_str()).copied(),
        _ => None,
    }
}

/// Returns the one-line blurb for keyword/contextual-word spelling `word`,
/// or `None` if `word` is not in the closed keyword-doc table.
pub fn keyword_blurb(word: &str) -> Option<&'static str> {
    KEYWORD_DOCS.iter().find(|(name, _)| *name == word).map(|(_, blurb)| *blurb)
}

/// Lexes `text` fresh and finds the keyword (or contextual word) token whose
/// span contains byte offset `offset`, returning its canonical spelling and
/// byte range.
///
/// Keywords are not represented in the AST as such (the parser consumes them
/// as grammar, not as a node), so this is a raw [`Lexer`] scan rather than a
/// walk of a cached [`Program`] — the same class of lookup
/// [`crate::index::selector_at_offset`] does for selectors, one level lower.
/// Returns `None` if the token containing `offset` is not a keyword/
/// contextual word (e.g. an ordinary identifier, a literal, or punctuation).
pub fn keyword_at_offset(text: &str, offset: usize) -> Option<(&'static str, std::ops::Range<usize>)> {
    for item in Lexer::new(text) {
        let Ok((start, token, end)) = item else {
            continue;
        };
        if start <= offset && offset < end {
            return keyword_spelling(&token).map(|spelling| (spelling, start..end));
        }
    }
    None
}

/// A harvested Phaldoc block: its summary paragraph and closed-vocabulary
/// `@tag` lines (doc-comments-phaldoc.md §2-3).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PhaldocDoc {
    /// The first blank-line/`@tag`-delimited paragraph of the block's body —
    /// the headline shown in the hover (§2).
    pub summary: String,
    /// Every `@tag payload` line (and its indented continuation lines,
    /// folded to one line), in declaration order. `@return`/`@raises` are
    /// canonicalized to `returns`/`throws` (§3); an unrecognized `@tag` is
    /// kept verbatim rather than dropped (Phaldoc is advisory-only, never an
    /// error — §3).
    pub tags: Vec<(String, String)>,
}

/// Strips a `///` doc-marker line down to its body: the leading `///` and at
/// most one following space.
fn strip_doc_marker(trimmed_line: &str) -> String {
    let rest = trimmed_line.strip_prefix("///").unwrap_or(trimmed_line);
    rest.strip_prefix(' ').unwrap_or(rest).to_string()
}

/// Canonicalizes a Phaldoc tag alias to its primary spelling
/// (`@return`→`returns`, `@raises`→`throws`; §3), leaving every other tag
/// name (recognized or not) unchanged.
fn canonical_tag_name(name: &str) -> String {
    match name {
        "return" => "returns".to_string(),
        "raises" => "throws".to_string(),
        other => other.to_string(),
    }
}

/// Returns the count of leading ASCII/Unicode whitespace bytes-as-chars in
/// `line` — the indentation depth [`parse_tags`] compares a candidate
/// continuation line's indentation against (doc-comments-phaldoc.md §3's
/// "indented continuation lines").
fn indent_width(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// Parses the `@tag payload` lines of a doc block's body (everything after
/// the summary paragraph), folding each tag's **indented** continuation
/// lines into its payload.
///
/// Per doc-comments-phaldoc.md §3, "everything after the tag on that line
/// (and any following **indented** continuation lines) is the tag's
/// payload" — a following non-blank, non-`@` line only folds into the
/// current tag when it is indented *strictly more* than the tag line
/// itself (`lines` here is already marker-stripped by
/// [`strip_doc_marker`], so the comparison is against the doc-body
/// indentation, not the `///` column). A same-or-lesser-indented line ends
/// the tag's payload without being consumed — it is simply skipped (no tag
/// vocabulary, no continuation), consistent with Phaldoc's "advisory only,
/// never an error" stance (§3).
fn parse_tags(lines: &[String]) -> Vec<(String, String)> {
    let mut tags = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let tag_indent = indent_width(&lines[i]);
        let trimmed = lines[i].trim_start();
        if let Some(rest) = trimmed.strip_prefix('@') {
            let mut parts = rest.splitn(2, char::is_whitespace);
            let raw_name = parts.next().unwrap_or("").to_string();
            let mut payload = parts.next().unwrap_or("").trim().to_string();
            i += 1;
            while i < lines.len() && !lines[i].trim().is_empty() && !lines[i].trim_start().starts_with('@') && indent_width(&lines[i]) > tag_indent {
                if !payload.is_empty() {
                    payload.push(' ');
                }
                payload.push_str(lines[i].trim());
                i += 1;
            }
            tags.push((canonical_tag_name(&raw_name), payload));
        } else {
            i += 1;
        }
    }
    tags
}

/// Splits a doc block's already-marker-stripped body lines into a
/// [`PhaldocDoc`]: the summary paragraph (every line up to the first blank
/// line or `@tag` line) and the parsed `@tag` lines after it (§2-3).
fn parse_doc_block(lines: &[String]) -> PhaldocDoc {
    let mut idx = 0;
    let mut summary_lines = Vec::new();
    while idx < lines.len() {
        let line = &lines[idx];
        if line.trim().is_empty() || line.trim_start().starts_with('@') {
            break;
        }
        summary_lines.push(line.trim().to_string());
        idx += 1;
    }
    let summary = summary_lines.join(" ").trim().to_string();
    while idx < lines.len() && lines[idx].trim().is_empty() {
        idx += 1;
    }
    let tags = parse_tags(&lines[idx..]);
    PhaldocDoc { summary, tags }
}

/// Returns the byte-offset [`SourceRange`](phalcom_common::range::SourceRange)
/// of a [`ClassMember`] declaration.
fn member_range(member: &ClassMember) -> phalcom_common::range::SourceRange {
    match member {
        ClassMember::Method(m) => m.range,
        ClassMember::Getter(g) => g.range,
        ClassMember::Setter(s) => s.range,
        ClassMember::Field(f) => f.range,
        // See `selectors::class_member_selector`'s doc for why a `@variant`
        // arm is handled uniformly alongside the other member kinds here.
        ClassMember::Variant(v) => v.range,
        ClassMember::Index(ix) => ix.range,
    }
}

/// Finds the selector of the `ClassMember` declaration that starts on 0-based
/// line `line` of `program`'s source, or `None` if no member starts there.
///
/// This is the "declaration adjacent to this `///` block" resolution
/// doc-comments-phaldoc.md §4/§5 needs: a single range-containment lookup
/// into the already-cached [`Program`], not a second parse.
fn member_selector_at_line(program: &Program, line: usize, line_index: &LineIndex) -> Option<String> {
    for statement in &program.statements {
        if let Statement::Class(class_def) = statement {
            for member in &class_def.members {
                let range = member_range(member);
                if line_index.position(range.start).line as usize == line {
                    return Some(class_member_selector(member));
                }
            }
        }
    }
    None
}

/// Finds the bound name of the top-level `let`/`var` [`Statement::Let`]
/// binding that starts on 0-based line `line` of `program`'s source, or
/// `None` if no top-level binding starts there.
///
/// The doc-comments-phaldoc.md §5 placement-legality table lists `let`/`var`
/// among the items a `///` block above it documents, alongside class members
/// — this is [`member_selector_at_line`]'s counterpart for that case. Only a
/// non-destructuring [`Pattern::Name`] target has a single stable name to key
/// the doc under; a destructuring `let (a, b) = …` binds several names at
/// once and has no single adjacency target, so it is not matched here.
///
/// Bindings are matched only at the top level (`program.statements`
/// directly) — a `let` inside a class member's body is a **local** binding,
/// out of Phaldoc's placement-legality scope (§5 only names top-of-file/
/// class-body/class-member positions), so no recursion into member bodies
/// is done here.
fn top_level_binding_name_at_line(program: &Program, line: usize, line_index: &LineIndex) -> Option<String> {
    for statement in &program.statements {
        if let Statement::Let(binding) = statement {
            if line_index.position(binding.range.start).line as usize == line {
                if let Pattern::Name { name, .. } = &binding.pattern {
                    return Some(name.clone());
                }
            }
        }
    }
    None
}

/// Scans `text` (the raw, un-lexed source) for `///` doc blocks and returns
/// the one whose target selector is exactly `selector`, if any.
///
/// Implements doc-comments-phaldoc.md §1-5:
/// - `//!` lines are inner docs and are never parsed as an outer `///` block.
/// - A `///` run's target is the selector of the `ClassMember` declaration
///   (`member_selector_at_line`), or else the bound name of a top-level
///   `let`/`var` binding (`top_level_binding_name_at_line`), on the next
///   non-blank, non-`///` line (a single range-containment lookup into the
///   cached `Program`) — adjacency. A blank line immediately after the block
///   breaks adjacency (a dangling doc documents nothing, §5).
/// - A first body line of the literal shape `selector: <sel>` (§4.4) pins the
///   block's target explicitly, overriding adjacency resolution entirely
///   (the block need not even be immediately followed by a declaration).
///
/// This is Stage 4's "docs key to the selector, not the name" rule (§4):
/// `foo()` and `foo(_)` immediately above one another each resolve
/// independently, since each is looked up by its own exact selector string.
pub fn harvest_doc_for_selector(text: &str, program: &Program, line_index: &LineIndex, selector: &str) -> Option<PhaldocDoc> {
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with("//!") {
            i += 1;
            continue;
        }
        if trimmed.starts_with("///") {
            let mut body_lines: Vec<String> = Vec::new();
            let mut j = i;
            while j < lines.len() {
                let t = lines[j].trim();
                if !t.starts_with("///") {
                    break;
                }
                body_lines.push(strip_doc_marker(t));
                j += 1;
            }

            let pinned = body_lines
                .first()
                .and_then(|l| l.trim().strip_prefix("selector:"))
                .map(|s| s.trim().to_string());

            let target = if let Some(pin) = pinned.clone() {
                Some(pin)
            } else if j < lines.len() && !lines[j].trim().is_empty() {
                member_selector_at_line(program, j, line_index).or_else(|| top_level_binding_name_at_line(program, j, line_index))
            } else {
                // Either the block is at EOF (no following line) or the
                // following line is blank — a dangling doc (§5).
                None
            };

            if target.as_deref() == Some(selector) {
                let content_lines: &[String] = if pinned.is_some() { &body_lines[1..] } else { &body_lines[..] };
                return Some(parse_doc_block(content_lines));
            }

            i = j;
            continue;
        }
        i += 1;
    }
    None
}

/// The contract-view seam (plan's explicit "must not preclude").
///
/// Currently **inert** — always returns `None` — so U-ANNOT-CONTRACTS'
/// `@requires`/`@ensures` rendering (doc-comments-phaldoc.md §8 harvest
/// order) can drop in later by giving this a real body, without reshaping
/// [`render_selector_hover`]'s composition path. Do not implement contract
/// rendering here; U-ANNOT-CONTRACTS is a separate, already-in-progress
/// unit.
pub fn render_contract_view(selector: &str) -> Option<String> {
    let _ = selector;
    None
}

/// Renders a keyword/contextual-word hover: the bolded spelling, tagged
/// `_(keyword)_`, followed by its blurb — mirrors `hover.ts`'s
/// `provideHover` keyword branch.
pub fn render_keyword_hover(word: &str, blurb: &str) -> String {
    format!("**{word}** _(keyword)_\n\n{blurb}")
}

/// One place a selector is declared/known, as [`render_selector_hover`]
/// needs it: the class it is declared on and its dispatch kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorSite {
    /// The name of the class the member is declared on.
    pub class: String,
    /// The member's dispatch kind (method/getter/setter/construct/...).
    pub kind: crate::index::MemberKind,
}

/// Renders a member kind as the lowercase word `render_selector_hover` shows
/// (`"method"`, `"getter"`, `"setter"`, `"class method"`, `"construct"`).
fn kind_word(kind: crate::index::MemberKind) -> &'static str {
    use crate::index::MemberKind;
    match kind {
        MemberKind::Getter => "getter",
        MemberKind::Setter => "setter",
        MemberKind::Method => "method",
        MemberKind::StaticMethod => "class method",
        MemberKind::Construct => "construct",
    }
}

/// Composes the selector-hover markdown from whichever of its independent
/// sources are present: `sites` (the classes/kinds `selector` is declared or
/// listed on — user-class [`crate::index::DefinitionInfo`]s or builtin
/// live semantic core members, whichever the caller resolved),
/// the harvested `phaldoc` summary/tags, and the (currently inert)
/// [`render_contract_view`] seam.
///
/// Returns `None` if `sites` is empty and `phaldoc` is `None` — the "nothing
/// to show" case (mirrors `hover.ts`'s `matches.length === 0 && !summary`
/// guard), so the caller degrades to no hover rather than an empty one.
pub fn render_selector_hover(selector: &str, sites: &[SelectorSite], phaldoc: Option<&PhaldocDoc>) -> Option<String> {
    render_selector_hover_with_value(selector, sites, phaldoc, None)
}

/// Renders selector hover with a shared semantic return/value section.
pub fn render_selector_hover_with_value(
    selector: &str,
    sites: &[SelectorSite],
    phaldoc: Option<&PhaldocDoc>,
    inferred: Option<&InferredValue>,
) -> Option<String> {
    if sites.is_empty() && phaldoc.is_none() {
        return None;
    }

    let mut sections = Vec::new();

    if sites.is_empty() {
        // A purely local, Phaldoc-only hover still gets a label so it never
        // renders as an unlabelled floating paragraph (mirrors `hover.ts`).
        sections.push(format!("**{selector}**"));
    } else {
        let classes: Vec<&str> = sites.iter().map(|s| s.class.as_str()).collect();
        let kind = kind_word(sites[0].kind);
        sections.push(format!("`{selector}` — {kind} on {}", classes.join(", ")));
    }

    if let Some(doc) = phaldoc {
        if !doc.summary.is_empty() {
            sections.push(doc.summary.clone());
        }
        if !doc.tags.is_empty() {
            let tag_lines: Vec<String> = doc.tags.iter().map(|(tag, payload)| format!("- **@{tag}** {payload}")).collect();
            sections.push(tag_lines.join("\n"));
        }
    }

    if let Some(value) = inferred.filter(|value| !matches!(value.shape, ValueShape::Unknown) && value.confidence != Confidence::Heuristic) {
        sections.push(format!(
            "**Observed return:** `{}`\n\nConfidence: {}",
            crate::semantic::render_value_shape(&value.shape),
            crate::semantic::confidence_name(value.confidence)
        ));
    }

    if let Some(contract_view) = render_contract_view(selector) {
        sections.push(contract_view);
    }

    Some(sections.join("\n\n---\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::line_index::LineIndex;
    use phalcom_ast::parser::parse;

    fn parsed(src: &str) -> (Program, LineIndex) {
        let parse_result = parse(src, 0);
        assert!(parse_result.errors.is_empty(), "{:?}", parse_result.errors);
        (parse_result.program, LineIndex::new(src))
    }

    #[test]
    fn keyword_blurb_covers_every_token_keyword_and_contextual_word() {
        let all = [
            "let",
            "const",
            "fn",
            "class",
            "return",
            "true",
            "false",
            "if",
            "else",
            "while",
            "for",
            "break",
            "continue",
            "import",
            "self",
            "super",
            "in",
            "as",
            "is",
            "and",
            "or",
            "not",
            "static",
            "construct",
            "throw",
            "try",
            "on",
            "catch",
            "ensure",
            "match",
        ];
        for word in all {
            assert!(keyword_blurb(word).is_some(), "missing blurb for {word:?}");
        }
    }

    #[test]
    fn keyword_at_offset_finds_self_keyword() {
        let src = "class Point {\n  x() { self }\n}\n";
        let offset = src.find("self").unwrap();
        let (word, range) = keyword_at_offset(src, offset).expect("self is a keyword");
        assert_eq!(word, "self");
        assert_eq!(&src[range], "self");
    }

    #[test]
    fn keyword_at_offset_finds_is_keyword() {
        let src = "class Dog is Animal {\n}\n";
        let offset = src.find("is").unwrap();
        let (word, _) = keyword_at_offset(src, offset).expect("is is a keyword");
        assert_eq!(word, "is");
    }

    #[test]
    fn keyword_at_offset_none_for_plain_identifier() {
        let src = "let x = 1\n";
        let offset = src.find('x').unwrap();
        assert_eq!(keyword_at_offset(src, offset), None);
    }

    #[test]
    fn phaldoc_adjacency_attaches_to_the_immediately_following_method() {
        let src = "class Point {\n  /// Moves the point.\n  move(_ x) { }\n}\n";
        let (program, line_index) = parsed(src);
        let doc = harvest_doc_for_selector(src, &program, &line_index, "move(_)").expect("adjacent doc must attach");
        assert_eq!(doc.summary, "Moves the point.");
    }

    #[test]
    fn phaldoc_blank_line_breaks_adjacency() {
        let src = "class Point {\n  /// Moves the point.\n\n  move(_ x) { }\n}\n";
        let (program, line_index) = parsed(src);
        assert_eq!(harvest_doc_for_selector(src, &program, &line_index, "move(_)"), None);
    }

    #[test]
    fn phaldoc_is_selector_keyed_not_name_keyed() {
        let src = "class Point {\n  /// Zero-arity reset.\n  reset() { }\n  /// Reset with a value.\n  reset(_ x) { }\n}\n";
        let (program, line_index) = parsed(src);
        let zero = harvest_doc_for_selector(src, &program, &line_index, "reset()").expect("reset() has its own doc");
        let one = harvest_doc_for_selector(src, &program, &line_index, "reset(_)").expect("reset(_) has its own doc");
        assert_eq!(zero.summary, "Zero-arity reset.");
        assert_eq!(one.summary, "Reset with a value.");
        assert_ne!(zero.summary, one.summary);
    }

    #[test]
    fn phaldoc_detached_pin_overrides_adjacency() {
        let src = "class Point {\n  /// selector: move(_,to)\n  /// A detached doc.\n\n  move(_ x, to) { }\n}\n";
        let (program, line_index) = parsed(src);
        let doc = harvest_doc_for_selector(src, &program, &line_index, "move(_,to)").expect("pinned selector overrides adjacency");
        assert_eq!(doc.summary, "A detached doc.");
    }

    #[test]
    fn phaldoc_parses_tags() {
        let src = "class Point {\n  /// Moves the point.\n  ///\n  /// @param x how far to move\n  /// @returns the new position\n  move(_ x) { }\n}\n";
        let (program, line_index) = parsed(src);
        let doc = harvest_doc_for_selector(src, &program, &line_index, "move(_)").unwrap();
        assert_eq!(doc.summary, "Moves the point.");
        assert_eq!(
            doc.tags,
            vec![
                ("param".to_string(), "x how far to move".to_string()),
                ("returns".to_string(), "the new position".to_string()),
            ]
        );
    }

    #[test]
    fn phaldoc_attaches_to_a_top_level_let_binding() {
        let src = "/// The application's shared counter.\nlet counter = Counter.new();\n";
        let (program, line_index) = parsed(src);
        let doc = harvest_doc_for_selector(src, &program, &line_index, "counter").expect("doc above a top-level let must attach to its bound name");
        assert_eq!(doc.summary, "The application's shared counter.");
    }

    #[test]
    fn phaldoc_attaches_to_a_top_level_const_binding() {
        let src = "/// A fixed running total.\nconst total = 0;\n";
        let (program, line_index) = parsed(src);
        let doc = harvest_doc_for_selector(src, &program, &line_index, "total").expect("doc above a top-level const must attach to its bound name");
        assert_eq!(doc.summary, "A fixed running total.");
    }

    #[test]
    fn phaldoc_indented_continuation_line_folds_into_the_tag() {
        let src = "class Point {\n  /// Moves the point.\n  ///\n  /// @param x how far to move,\n  ///   continued on an indented line\n  move(_ x) { }\n}\n";
        let (program, line_index) = parsed(src);
        let doc = harvest_doc_for_selector(src, &program, &line_index, "move(_)").unwrap();
        assert_eq!(
            doc.tags,
            vec![("param".to_string(), "x how far to move, continued on an indented line".to_string())]
        );
    }

    #[test]
    fn phaldoc_non_indented_following_line_does_not_fold_into_the_tag() {
        // The second `///` line sits at the *same* indentation as the `@param`
        // line's own body text — per §3 it is not a continuation, so it must
        // not fold into the tag's payload.
        let src = "class Point {\n  /// Moves the point.\n  ///\n  /// @param x how far to move\n  /// not a continuation\n  move(_ x) { }\n}\n";
        let (program, line_index) = parsed(src);
        let doc = harvest_doc_for_selector(src, &program, &line_index, "move(_)").unwrap();
        assert_eq!(doc.tags, vec![("param".to_string(), "x how far to move".to_string())]);
    }

    #[test]
    fn phaldoc_return_and_raises_aliases_canonicalize() {
        let src = "class Point {\n  /// Summary.\n  ///\n  /// @return a value\n  /// @raises Error sometimes\n  move() { }\n}\n";
        let (program, line_index) = parsed(src);
        let doc = harvest_doc_for_selector(src, &program, &line_index, "move()").unwrap();
        assert_eq!(doc.tags[0].0, "returns");
        assert_eq!(doc.tags[1].0, "throws");
    }

    #[test]
    fn render_selector_hover_for_user_class_member() {
        let sites = vec![SelectorSite {
            class: "Point".to_string(),
            kind: crate::index::MemberKind::Method,
        }];
        let doc = PhaldocDoc {
            summary: "Moves the point.".to_string(),
            tags: vec![],
        };
        let rendered = render_selector_hover("move(_)", &sites, Some(&doc)).unwrap();
        assert!(rendered.contains("move(_)"));
        assert!(rendered.contains("method on Point"));
        assert!(rendered.contains("Moves the point."));
    }

    #[test]
    fn render_selector_hover_for_builtin_has_no_phaldoc_section() {
        let sites = vec![SelectorSite {
            class: "Bool".to_string(),
            kind: crate::index::MemberKind::Method,
        }];
        let rendered = render_selector_hover("ifTrue(_)", &sites, None).unwrap();
        assert!(rendered.contains("ifTrue(_)"));
        assert!(rendered.contains("method on Bool"));
        assert_eq!(rendered.matches("---").count(), 0);
    }

    #[test]
    fn render_selector_hover_none_when_nothing_resolves() {
        assert_eq!(render_selector_hover("mystery(_)", &[], None), None);
    }

    #[test]
    fn render_selector_hover_includes_shared_return_summary() {
        let value = InferredValue::flow(
            ValueShape::Instance(crate::semantic::ClassId::new(crate::semantic::ModuleId::new("phalcom://core"), "String")),
            (0..0).into(),
        );
        let sites = vec![SelectorSite {
            class: "Factory".to_string(),
            kind: crate::index::MemberKind::Method,
        }];
        let rendered = render_selector_hover_with_value("make()", &sites, None, Some(&value)).unwrap();
        assert!(rendered.contains("**Observed return:** `String`"));
        assert!(rendered.contains("Confidence: flow"));
    }
}
