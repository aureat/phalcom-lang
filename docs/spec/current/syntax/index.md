# Syntax

Part of the [Phalcom Language Specification](../README.md). Status: Draft 0.1.

## 1. Purpose

This directory is the consolidated **normative grammar** for Phalcom's surface
syntax. The topic docs one level up — [Lexical Structure](../lexical-structure.md),
[Messages & Selectors](../messages-and-selectors.md), [Selectors, Symbols &
References](../selectors.md), [Classes](../classes.md), [Control Flow](../control-flow.md),
[Iteration](../iteration.md), [Error Handling](../error-handling.md), and the rest —
remain authoritative on *semantics and rationale*: why a form exists, what it
desugars to, what invariant it protects. The four files in this directory give the
**productions**: the formal grammar a parser is built from, with only the minimal
examples needed to read them.

This describes the **v0.2 target** surface grammar, not necessarily what the
current tree accepts today. Where the implemented parser diverges from a
production here — a form not yet wired up, an older shape still active — that
divergence is tracked centrally in [Implementation Status](../implementation-status.md)
rather than being restated file by file.

The governing decision for the parsing strategy itself — hand-written lexer,
recursive-descent parser, no parser-generator — is [TDR-0014]. Individual
productions cite their own governing ADR inline where one exists.

## 2. Notation

All grammar fences in this directory (and its siblings) use one small metagrammar:

| Symbol | Meaning |
|---|---|
| `:=` | defines a production |
| \| | alternation |
| `[ x ]` | `x` is optional |
| `{ x }` | zero or more repetitions of `x` |
| `( x )` | grouping |
| `"lit"` | a literal terminal or keyword |
| UPPERCASE | a lexical token class (`IDENT`, `INT`, `FLOAT`, `STRING`, `NEWLINE`, `EOF`, ...) |
| `(* ... *)` | a comment inside the grammar, not part of the language |

Productions are given in plain fenced blocks (no language tag); source examples
are given in ` ```phalcom ` fenced blocks with aligned `//` comments.

## 3. Reading order

| File | Covers |
|---|---|
| [`lexical.md`](lexical.md) | Tokens: whitespace/newline handling, comments, identifiers, keywords, string/boolean/symbol literals, operators and punctuation |
| [TDR-0070](../../../decisions/accepted/0070-numeric-literals.md) | Normative numeric literal grammar: radices, separators, exponent floats, boundaries, diagnostics |
| [`expressions.md`](expressions.md) | The expression grammar: primary/postfix/binary forms, message sends, operator precedence and associativity |
| [`statements-and-declarations.md`](statements-and-declarations.md) | Statements, blocks, and declarations: `let`/`var`, `class`, `trait`, methods, modules, error-handling clauses |
| [`grammar.md`](grammar.md) | Consolidated appendix — every production from the three files above collected in one place |

Read `lexical.md` first; `expressions.md` and `statements-and-declarations.md`
both build on its token classes. `grammar.md` is a reference, not a tutorial —
consult it once you already know which production you're looking for.

## 4. Relationship to other docs

| Doc | Role |
|---|---|
| [`../lexical-structure.md`](../lexical-structure.md) | Prose and rationale for tokens — *why* newlines are significant, *why* `nil` has no surface keyword. This directory gives the token productions themselves. |
| [`../implementation-status.md`](../implementation-status.md) | Target-vs-built divergence for the whole spec, including syntax. Consult it before assuming a production here already parses. |
| [`../../extensions/traits.md`](../../extensions/traits.md) | Effective semantics for first-class non-storage trait contracts and defaults |

### Governing ADRs

| ADR | Syntax-relevant decision |
|---|---|
| [TDR-0014] | Hand-written lexer + recursive-descent parser; governs precedence climbing and error recovery shape |
| [TDR-0011] | Selector signature encoding — labels are part of selector identity, driving `#symbol` and callable-reference grammar |
| [TDR-0053] | `let` (mutable) / `const` (immutable) binding forms; unkeyworded mutable fields — supersedes [ADR-0014] |
| [TDR-0019] | No truthiness enforcement — condition positions require `Boolean`, no implicit coercion |
| [TDR-0020] | String interpolation: `\(expr)` sigil, backslash-paren form |
| [TDR-0074] | String escape, diagnostic, lowering, and range completion; multiline literals deferred |
| [TDR-0022] | Numeric surface split: `Int` vs `Float` literal forms, `~/` integer division |
| [TDR-0023] | External/internal parameter names — labeled-argument surface syntax |
| [TDR-0024] | Modules as files, public-by-default, `import`/`as` |
| [TDR-0026] / [TDR-0029] | Collection literal syntax (`List`) and collection representation |
| [TDR-0028] | Error-handling surface syntax: `throw`, `try`/`catch`/`on`/`ensure` |
| [TDR-0030] | Iteration protocol (cursor-based) and `for`/`in` desugaring |

[TDR-0014]: ../../../adr/0016-hand-written-lexer-and-recursive-descent-parser.md
[TDR-0011]: ../../../adr/0012-selector-signature-encoding-and-dispatch.md
[ADR-0014]: ../../../adr/0014-let-and-var-bindings.md
[TDR-0019]: ../../../adr/0021-no-truthiness-enforcement.md
[TDR-0020]: ../../../adr/0022-string-interpolation-backslash-paren-sigil.md
[TDR-0074]: ../../../pdr/0029-string-literals-and-interpolation-completion.md
[TDR-0022]: ../../../adr/0024-numeric-surface-split-int-float-and-division.md
[TDR-0023]: ../../../adr/0025-external-internal-parameter-names.md
[TDR-0024]: ../../../adr/0027-modules-as-files-with-public-by-default-imports.md
[TDR-0026]: ../../../adr/0029-list-literal-syntax.md
[TDR-0028]: ../../../adr/0031-error-handling-surface-syntax.md
[TDR-0029]: ../../../adr/0032-collections-representation-and-literals.md
[TDR-0030]: ../../../adr/0035-iteration-protocol-cursor.md
