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
recursive-descent parser, no parser-generator — is [ADR-0016]. Individual
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
| [`lexical.md`](lexical.md) | Tokens: whitespace/newline handling, comments, identifiers, keywords, literals (numeric, string, boolean, symbol), operators and punctuation |
| [`expressions.md`](expressions.md) | The expression grammar: primary/postfix/binary forms, message sends, operator precedence and associativity |
| [`statements-and-declarations.md`](statements-and-declarations.md) | Statements, blocks, and declarations: `let`/`var`, `class`, `construct`, methods, modules, error-handling clauses |
| [`grammar.md`](grammar.md) | Consolidated appendix — every production from the three files above collected in one place |

Read `lexical.md` first; `expressions.md` and `statements-and-declarations.md`
both build on its token classes. `grammar.md` is a reference, not a tutorial —
consult it once you already know which production you're looking for.

## 4. Relationship to other docs

| Doc | Role |
|---|---|
| [`../lexical-structure.md`](../lexical-structure.md) | Prose and rationale for tokens — *why* newlines are significant, *why* `nil` has no surface keyword. This directory gives the token productions themselves. |
| [`../implementation-status.md`](../implementation-status.md) | Target-vs-built divergence for the whole spec, including syntax. Consult it before assuming a production here already parses. |

### Governing ADRs

| ADR | Syntax-relevant decision |
|---|---|
| [ADR-0016] | Hand-written lexer + recursive-descent parser; governs precedence climbing and error recovery shape |
| [ADR-0012] | Selector signature encoding — labels are part of selector identity, driving `#symbol` and method-reference grammar |
| [ADR-0014] | `let` (immutable) / `var` (mutable) binding forms |
| [ADR-0021] | No truthiness enforcement — condition positions require `Boolean`, no implicit coercion |
| [ADR-0022] | String interpolation: `\(expr)` sigil, backslash-paren form |
| [ADR-0024] | Numeric surface split: `Int` vs `Float` literal forms, `~/` integer division |
| [ADR-0025] | External/internal parameter names — labeled-argument surface syntax |
| [ADR-0027] | Modules as files, public-by-default, `import`/`as` |
| [ADR-0029] / [ADR-0032] | Collection literal syntax (`List`) and collection representation |
| [ADR-0031] | Error-handling surface syntax: `throw`, `try`/`catch`/`on`/`ensure` |
| [ADR-0035] | Iteration protocol (cursor-based) and `for`/`in` desugaring |

[ADR-0016]: ../../../adr/0016-hand-written-lexer-and-recursive-descent-parser.md
[ADR-0012]: ../../../adr/0012-selector-signature-encoding-and-dispatch.md
[ADR-0014]: ../../../adr/0014-let-and-var-bindings.md
[ADR-0021]: ../../../adr/0021-no-truthiness-enforcement.md
[ADR-0022]: ../../../adr/0022-string-interpolation-backslash-paren-sigil.md
[ADR-0024]: ../../../adr/0024-numeric-surface-split-int-float-and-division.md
[ADR-0025]: ../../../adr/0025-external-internal-parameter-names.md
[ADR-0027]: ../../../adr/0027-modules-as-files-with-public-by-default-imports.md
[ADR-0029]: ../../../adr/0029-list-literal-syntax.md
[ADR-0031]: ../../../adr/0031-error-handling-surface-syntax.md
[ADR-0032]: ../../../adr/0032-collections-representation-and-literals.md
[ADR-0035]: ../../../adr/0035-iteration-protocol-cursor.md
