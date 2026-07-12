# 22. String interpolation uses the `\(expr)` sigil

- Status: Accepted
- Date: 2026-07-12
- Related: `docs/spec/v0.2/lexical-structure.md` §5 (string interpolation);
  `docs/spec/v0.2/units/U/lex-lexical-delta.md` §D4; `docs/forge/open-questions.md`
  item 5 (Q5); `docs/forge/PHASE2-INDEX.md` §4 DEC-F;
  [ADR-0016](0016-hand-written-lexer-and-recursive-descent-parser.md)
  (hand-written scanner); `phalcom-ast/src/lexer.rs` (`scan_string`),
  `phalcom-ast/src/token.rs` (`Token::StringInterp` / `StringSegment`),
  `phalcom-ast/src/parser.rs` (interpolation desugar)

## Context

Phalcom string literals grow an interpolation form so `"Hello \(name)!"`
renders the value of `name` inline. The lexeme that marks an interpolated
expression inside a string was open decision **Q5 / DEC-F**: the U-LEX spec (§2
D4) and `open-questions.md` item 5 recorded three candidate sigils, none
ratified:

- **(a) `{expr}`** — no sigil noise, but the opening brace visually collides
  with block and map-literal braces (`{ … }` is already a block literal), and it
  forces a `\{` escape for a literal brace inside every string.
- **(b) `${expr}`** — shell/JS familiar and unambiguous, but adds a two-character
  sigil and a `$` that has no other role in the grammar.
- **(c) `\(expr)`** — Swift-style. `\` already reads as "escape / special" inside
  a string, and `(` … `)` are self-delimiting.

The spec's own default and the architect recommendation (DEC-F) was **(a)
`{expr}`**; the staged pending fixture was written in the `{name}` form.

## Decision

Adopt **(c) `\(expr)`** (Swift-style). The user overrode the architect's
`{expr}` recommendation and ratified `\(expr)` over both `{expr}` and `${expr}`.

Concretely, in a double-quoted string literal:

- `\(` … `)` delimits an interpolated expression. The expression text between the
  balanced parentheses is re-parsed and its value stringified at the
  interpolation site.
- `\\(` is the escape for a **literal** backslash-then-paren `\(` (the leading
  `\\` is an escaped backslash), so a string can contain a literal `\(` without
  triggering interpolation.
- `\\` is a literal backslash. Any other `\x` sequence is left verbatim (a
  literal backslash followed by `x`), preserving the pre-interpolation string
  behaviour where `"\n"` was a two-character literal.
- A string with no `\(` still lexes to the plain [`Token::String`] path — the
  interpolation machinery is inert unless an interpolation actually appears.

The lexer emits an interpolated string as a single
`Token::StringInterp(Vec<StringSegment>)` lexeme (ordered literal runs and
expression runs); the parser desugars it in place to a `+`-chain of string
segments, so no interpolation node reaches the compiler
([ADR-0016](0016-hand-written-lexer-and-recursive-descent-parser.md)'s
front-end split). This mirrors how `if`/`while`/`??`/`?.` already desugar in the
parser rather than adding a compiler-visible AST node.

**Stringification target.** The spec's illustrative desugar wrote
`a.toString + " x " + b.toString`. On the current substrate `toString` is the
`Object` default (it returns the class name, not the value's content) and
content stringification is blocked on U-CORE-4 (DEFERRED #19). The working
content-stringify primitive today is `String.new(_)`, which renders any value
via `Value::to_string`. The desugar therefore wraps each interpolated
expression as `String.new(expr)` rather than `expr.toString`; when U-CORE-4
lands a real content `toString`, the desugar target can be revisited. The shape
is otherwise exactly the spec's: literal runs become `String` literals,
expression runs become a stringify send, and the whole is folded left with `+`.

## Consequences

- Interpolation is unambiguous against block/map braces: `"{ … }"` is a plain
  string, `"\( … )"` interpolates. No `{`/`}` escaping is ever required inside a
  string.
- The `\` sigil reuses the reader's existing "backslash means special" intuition;
  the only new escape a user must learn is `\\(` for a literal `\(`.
- Because the desugar is a parser concern, the compiler and the object model are
  untouched — interpolation is pure front-end sugar over `+` and `String.new`.
- The desugar depends on `String.new(_)` rather than `toString`; this couples the
  interpolation output to that primitive until U-CORE-4 provides a content
  `toString`. Tracked as a follow-up, not a blocker.
- Balanced-paren scanning does not understand a string literal nested inside an
  interpolation expression (`"\(f(")"))"`), so an unbalanced `)` inside a nested
  string would mis-terminate the expression. This edge is accepted for v1 and
  noted in DEFERRED.

## Alternatives considered

- **`{expr}` (the architect/spec default).** Rejected by the user: the opening
  brace collides with block and map-literal braces and forces a `\{` escape in
  every string that wants a literal brace.
- **`${expr}`.** Familiar from shells and JavaScript template literals, but
  introduces a `$` sigil with no other grammar role; `\(expr)` reuses the
  existing escape character instead.
- **A compiler-visible `StringInterp` AST node** (defer desugar to the
  compiler). Rejected: the compiler is outside U-LEX's write-set, and the
  established idiom (`if`/`while`/`??`/`?.`) desugars surface sugar in the parser
  over existing nodes. A dedicated node would buy nothing here.
