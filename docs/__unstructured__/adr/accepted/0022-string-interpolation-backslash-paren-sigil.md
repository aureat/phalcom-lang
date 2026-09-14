# 22. String interpolation uses the `\(expr)` sigil

- Status: Accepted
- Date: 2026-07-12
- Amended: 2026-07-15 — **the stringification target is now `expr.toString`, not
  `String.new(expr)`.** The sigil decision is untouched. This is the revisit this ADR
  itself pre-authorised ("when U-CORE-4 lands a real content `toString`, the desugar
  target can be revisited"); U-CORE-4 landed in `2061795`. See
  [§Amendment](#amendment-2026-07-15--stringification-target-is-now-tostring). Prompted by
  DEFERRED CB-1.
- Related: `docs/spec/current/lexical-structure.md` §5 (string interpolation);
  `docs/spec/current/units/U/lex-lexical-delta.md` §D4; `docs/forge/open-questions.md`
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

**Stringification target.** _Superseded 2026-07-15 — see [§Amendment](#amendment-2026-07-15--stringification-target-is-now-tostring).
The paragraph below records the original 2026-07-12 reasoning and its explicit revisit
trigger, which has since fired._

> The spec's illustrative desugar wrote
> `a.toString + " x " + b.toString`. On the current substrate `toString` is the
> `Object` default (it returns the class name, not the value's content) and
> content stringification is blocked on U-CORE-4 (DEFERRED #19). The working
> content-stringify primitive today is `String.new(_)`, which renders any value
> via `Value::to_string`. The desugar therefore wraps each interpolated
> expression as `String.new(expr)` rather than `expr.toString`; when U-CORE-4
> lands a real content `toString`, the desugar target can be revisited. The shape
> is otherwise exactly the spec's: literal runs become `String` literals,
> expression runs become a stringify send, and the whole is folded left with `+`.

The shape is unchanged and is exactly the spec's: literal runs become `String` literals,
expression runs become a stringify send, and the whole is folded left with `+`. Only the
*target* of that send moved — to `expr.toString`, which is what the spec's illustrative
desugar wrote in the first place.

## Consequences

- Interpolation is unambiguous against block/map braces: `"{ … }"` is a plain
  string, `"\( … )"` interpolates. No `{`/`}` escaping is ever required inside a
  string.
- The `\` sigil reuses the reader's existing "backslash means special" intuition;
  the only new escape a user must learn is `\\(` for a literal `\(`.
- Because the desugar is a parser concern, the compiler and the object model are
  untouched — interpolation is pure front-end sugar over `+` and a stringify send.
- ~~The desugar depends on `String.new(_)` rather than `toString`; this couples the
  interpolation output to that primitive until U-CORE-4 provides a content
  `toString`. Tracked as a follow-up, not a blocker.~~ **Discharged 2026-07-15** — the
  follow-up came due and was done; the desugar now sends `toString`. It was *not*
  harmless while it stood: see §Amendment.
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

## Amendment (2026-07-15) — stringification target is now `toString`

**The sigil decision is unchanged. Only the desugar's target moved**, from
`String.new(expr)` to `expr.toString`. This is the revisit the original "Stringification
target" section explicitly authorised: *"when U-CORE-4 lands a real content `toString`, the
desugar target can be revisited."* U-CORE-4 landed (`2061795`).

**Why it could not stay.** `String.new(_)` renders via `Value::to_string`, a native
renderer that never sends a message. It hardcodes `Str`/`List`/`Map`/`Set`/`Tuple`/
`Range`/`None`/`Some` and falls through to `Value::to_debug` for everything else —
including a plain instance, a class, a metaclass. So a user's `toString` **override was
silently bypassed by interpolation**, and `System.print` (routed through
`Value::to_display_string` by U-ERR-FIX's BUG-PRINT-TOSTRING, `dd2e178`) disagreed with
`"\(…)"` for exactly the objects [ADR-0015](0015-object-default-tostring.md) governs:

```phalcom
class Secret { toString => "<redacted>" }
let p = Secret.new()
System.print(p)      // <redacted>
System.print("\(p)") // <Secret instance>   <- the override, bypassed
```

**The security dimension** (why this outranked its size). ADR-0015's default `toString`
is redaction-safe — a `SecretKey` renders as `<SecretKey>`, never its contents — and that
property did not survive interpolation, by far the most common stringify site. The leak was
*bounded*, not absent: `to_debug` prints no field contents today, so nothing sensitive
escaped. But that safety rested on an unrelated implementation detail. Enriching `to_debug`
to dump slots — an ordinary debug convenience someone will eventually want — would have
silently converted every interpolation in the codebase into a field-disclosure bug. Fixed
before the temptation, not after.

**What made it affordable.** Interpolation was the last consumer needing
`Map`/`Set`/`Tuple`/`Range` to answer `toString`, and those four were the *only* classes
with none — everything else already had one (native on `Object`/`Number`/`Symbol`/`List`,
derived for `String`/`Bool`/`Option`/`Result`). They are now derived in `core.ph` over the
existing floor (`22cc756`), so **no floor amendment was required**: ADR-0019's default
answer to "add a primitive" holds, and each mirrors `Value::to_string`'s native rendering
exactly.

**Implementation.** `desugar_string_interp` emits `Expr::GetProperty { property: "toString" }`
— a **getter** send, not a zero-arg `MethodCall`: `toString` is bound as
`SignatureKind::Getter`, and `toString()` would encode a different selector that misses it.
`String.new(_)` is untouched and still renders natively; it is simply no longer the
interpolation path. Guard: `tests/lang/strings/string_interp_sends_tostring.ph`.

**Known gap, deliberately left (DEFERRED CB-6).** `Value::to_string`'s recursion still never
sends. `List#toString` is native (`list_to_string`) and recurses through it, so an override
*nested in a `List`* is still bypassed — and `List` is now the odd one out, since the four
new `.ph` collection `toString`s recurse by sending:

```
"\(m)"   -> {k: <redacted>}      Map#toString is .ph, sends
"\([p])" -> [<Secret instance>]  List#toString is native, does not
```

That is this same defect one level down. Closing it means either rewriting `list_to_string`
in `.ph` (which would strand a floor binding) or making the native renderer's recursion
send — neither is in this amendment's scope.
