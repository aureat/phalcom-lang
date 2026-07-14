# Lexical Structure

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

## 1. Statement termination

Semicolons are **optional**. Statements are newline-terminated.

**Implementation.** The lexer emits `NEWLINE` tokens; the grammar treats them as
terminators only where a statement may end. Do **not** attempt ASI in the parser —
that is how JavaScript acquired the `return\n{}` bug. A newline is **suppressed**
(not emitted as a terminator) when the previous token cannot end a statement — a
binary operator, `,`, `(`, `{`, `=>`, `.`, etc. This is a small lexer-level state
machine, not parser lookahead.

## 2. Comments

```phalcom
// line comment
/* block comment */
```

## 3. Identifiers

- **Regular:** `[a-zA-Z][a-zA-Z0-9]*`
- **Field:** `_[a-zA-Z][a-zA-Z0-9]*` — the leading underscore is significant; a
  field is private state of the declaring class (see [Classes](classes.md)).

The two are distinct token classes: a field reference is only legal inside a class
body.

- **Convention — trailing `_`:** a selector ending in `_` (e.g. `at_`, `size_`,
  `keyAt_`) is not a distinct token class, just a naming convention (Wren-style):
  it marks a native/private primitive selector — internal floor plumbing wrapped
  by a public `.ph` method of the same base name (`at_` vs. `at`) — not meant to
  be sent directly from user code (U-NATIVE-MARKER).

## 4. Literals

```phalcom
42        1_000_000       3.1415          // numbers (digit separators allowed)
"hello"   "\(name) is \(age)"            // strings, with interpolation
true  false                              // booleans
(3, 4)                                   // tuple
[1, 2, 3]                                // list
{ a: 1, b: 2 }                           // map
Set(1, 2, 3)                             // set — a send, not a literal
```

- **No surface `nil`/`null`/`undefined`.** Absence is `Option`
  ([Values & Absence](values-and-absence.md)).
- **No set literal.** `{1, 2, 3}` is ambiguous with a block (§6) and not
  resolvable by lookahead. `Set(…)` is a plain send and costs nothing.

## 5. String interpolation

Interpolation uses `\(expr)` ([ADR-0022](../../adr/0022-string-interpolation-backslash-paren-sigil.md)).
The `\(` sequence is what triggers interpolation; a literal `\(` is written `\\(`.

```phalcom
"\(name) is \(age) years old"
"a literal \\( sequence"
```

Each `\(expr)` desugars to a `toString` send and string concatenation.

## 6. Brace disambiguation

`{` in **expression position** is decided by one token of lookahead:

| `{` followed by | Construct |
|-----------------|-----------|
| `IDENT :` | Map literal |
| `IDENT ,` | Block, with parameters |
| `IDENT =>` | Block, with parameters |
| `}` | Empty block |
| anything else | Block, zero parameters, body starts with an expression |

This is LR(1) — no cover grammar, no backtracking.

- `{}` is the **empty block**. The empty map is `Map()`.
- A `{` beginning a **statement** is a parse error (it would be a no-op block
  literal). JavaScript has the mirror-image rule for object literals.

## 7. Why tuples survive

`(a, b) => a + b` is **not** in the language — unbraced arrows are single-parameter
([Blocks §3](blocks.md)). Therefore `(` never begins a parameter list, no cover
grammar is required, and `(3, 4)` is unambiguously a tuple.

## 8. Grammar note on `*`

Prefix `*` (spread/rest) is legal **only** in a call argument list, a collection
literal element, and a parameter list. Everywhere else `*` is binary
multiplication. Since binary `*` requires a left operand, the two never compete
for the same position and the grammar stays LR(1).

## 9. `Option` operators: `?.` and `??`

Two tokens desugar to `Option` sends ([Values & Absence §3.4](values-and-absence.md)):

| Token | Position | Desugars to |
|-------|----------|-------------|
| `?.` | postfix, binds like `.` | `opt?.m(a) ≡ opt.map { x => x.m(a) }` |
| `??` | binary, right-associative | `a ?? b ≡ a.orElse { b }` |

- **`?.`** is a member-access operator, so it sits at the same precedence as `.`
  and is **left-associative**; a chain `a?.b?.c` groups as `(a?.b)?.c`, each hop
  staying inside `Option` and the first `None` short-circuiting the rest.
- **`??`** is a low-precedence binary operator, **right-associative** so
  `a ?? b ?? c` groups as `a ?? (b ?? c)`. It binds looser than comparison and
  arithmetic but tighter than assignment. The right operand is only evaluated when
  the left is `None` (short-circuit).

**Lexing.** `?.` and `??` are single tokens; a lone `?` is not (yet) a token —
reserve it for a future ternary or try-operator. Both must be resolved into the
precedence table during the grammar pass ([Open Questions §8](open-questions.md)).

## 10. Symbol literals: `#`

Full semantics in [Selectors, Symbols & References §2](selectors.md#2-symbol-literals-);
this section covers only the token-level rules.

`#` introduces a **name symbol** (`#move`) or a **selector symbol**
(`#move(_,to,duration)`), lexed as a **single atomic token**:

```rust
#[regex(r"#[a-zA-Z_][a-zA-Z0-9_]*(\([^)]*\))?", callback = canon_selector)]
```

with a separate branch for operator selectors (`#+`, `#==`, `#[]`).

**Whitespace-adjacency / ASI rule.** Outside the parens, `#`, the name, and `(`
must be contiguous — no whitespace. `#move (a, b)` lexes as the name symbol
`#move` followed by a parenthesized expression, *not* a selector symbol; this is
what prevents `#move` on one line from greedily eating a parenthesized
expression that starts the next line (the same ASI hazard §1 avoids for
newlines). Inside the parens, whitespace is free and stripped at intern time —
canonicalization happens at intern time, so `#move(_, to, duration)` and
`#move(_,to,duration)` intern to the same `Symbol`. A malformed body (interior
positionals, e.g. `#move(to,_)`) is a **lex-time error** with a precise span.

**Shebang special case.** `#!/usr/bin/env phalcom` is special-cased: `#!` is
skipped **only at byte offset 0** of the source file. Everywhere else `#!`
would otherwise ambiguously start a symbol token; the offset-0 restriction
removes the ambiguity without a lookahead.

`#` is reserved exclusively for symbols — JS-style private-field `#x` syntax is
**not adopted** (see [Selectors §5](selectors.md#5-field-visibility)); `@` (§12
below) owns attributes/decorators, so `#` and `@` never compete.

## 11. Method references: `::`

Full semantics in [Selectors, Symbols & References §3](selectors.md#3-method-references-).

`::` is a **postfix token**, `receiver::name` (Open family) or
`receiver::#name(_,...)` (Pinned family). The grammar is LR(1)-clean: after
lexing `::`, the parser peeks one token — `#` selects the Pinned form, anything
else is the Open form. No backtracking, no cover grammar.

## 12. Attribute token: `@`

Full semantics in [Selectors, Symbols & References §4](selectors.md#4-attributes-).

`@` prefixes a field or class declaration to mark a derived-accessor attribute
(`@construct`, `@get`, `@set`; planned, not yet grammar-specified beyond the
token). Attributes desugar to ordinary method-table entries at compile time —
no new dispatch machinery. `@` is reserved for this role only and does not
overlap with `#` (symbols) or `::` (method references).

## 13. Error-handling keywords

`throw`, `try`, `catch`, `on`, and `ensure` are the error-handling keywords
([Error Handling](error-handling.md); [ADR-0031](../../adr/0031-error-handling-surface-syntax.md)).

- `throw expr` is a prefix statement/expression; `expr` must evaluate to an
  [`Error`](values-and-absence.md). Sugar for `expr.raise()`.
- `try` / `on` / `catch` / `ensure` form the block-handler statement — pure sugar
  over the `Block` sends `on(_)(_)`, `ensure(_)`, and `attempt()`, no token
  carrying semantics the desugaring lacks. `on T e { … }` is a typed handler
  (`.on(T){ e => … }`); `catch e { … }` is the catch-all (`.on(Error){ e => … }`);
  `ensure { … }` is the cleanup (`.ensure{ … }`).
- `on`, `catch`, and `ensure` are **contextual keywords** — reserved only as
  `try`-clauses — so the `.on()`/`.ensure()` selectors and the `Fiber>>try` message
  keep working; `try` is reserved at statement-leading position.

## 14. Subscript indexing syntax: `[]` / `[]=`

Full semantics in [ADR-0060](../../adr/accepted/0060-index-operator-as-real-selector.md)
(supersedes the retired [ADR-0055](../../adr/retired/0055-index-syntax-sugar-over-at-selectors.md),
which lowered `[]` straight to `at`/`at(_,put:)` — that lowering no longer
occurs).

**Call site.** `expr[args...]` and `expr[args...] = value` are **not** sugar
for an `at`/`at(_,put:)` send — they compile to a direct send against a
dedicated bracket selector, exactly the way `expr == other` compiles to a
`==(_)` send. `args` is a full call-shaped argument list — positional and/or
`label:` — identical grammar to a call's `(...)`, so it generalizes past a
single index for free:

| Source | Selector sent |
|---|---|
| `expr[idx]` | `[_]` |
| `expr[idx] = value` | `[_,put]` |
| `expr[i, j]` | `[_,_]` |
| `expr[key, default: fallback]` | `[_,default]` |
| `expr[]` | `[]` |
| `expr[] = value` | `[put]` |

A collection opts into `[]`/`[]=` explicitly — implementing `at(_)` does not
automatically make a class indexable (collection-protocol.md §2 governs
`at`; this is a separate, independently-overridable selector).

**Definition site.** `[]` joins the operator-selector name set already
occupied by `==`, `+`, `<`, etc. — a class defines a bracket method with the
params living *inside* the brackets, substituting `[`/`]` for a method's
ordinary `(`/`)`:

```
class Example {
  [idx] { ... }           // read, one positional arg — selector [_]
  [idx, put:] { ... }     // write — selector [_,put]
  [] { ... }              // zero-arity read — selector []
  [put:] { ... }          // zero-arity write — selector [put]
}
```

**Postfix newline boundary.** The `[` is recognized as a postfix operator
only when it immediately follows a completed primary/postfix chain on the
**same line** — the same newline-termination rule `.`/`(`/`::`/`{` already
follow. A `[` at the start of a new line is always parsed as a fresh list
literal, not an index (avoiding a JavaScript-style ASI hazard).
</content>
