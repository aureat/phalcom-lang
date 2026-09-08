# Raw source snapshot: language

> Captured: 2026-09-08
> Source area: normative language surface and language-program boundaries
> This immutable snapshot preserves the source excerpts used by the compiled concept articles. Program metadata is lifecycle evidence, not proof that proposed work is implemented.

--- docs/spec/current/lexical-structure.md ---
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

- **Regular:** `[a-zA-Z][a-zA-Z0-9_]*`
- **Source field:** `_[a-zA-Z][a-zA-Z0-9_]*`
- **Implementation field:** `__[a-zA-Z][a-zA-Z0-9_]*`
- **Implementation selector:** `_$[a-zA-Z][a-zA-Z0-9_]*`
- **Positional declaration marker:** standalone `_`

These are distinct token classes. Fields are receiver-local state; implementation
names are privileged. Trailing `_` has no privacy or native meaning.

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


--- docs/spec/current/syntax/expressions.md ---
# Expression Grammar

Part of the [Phalcom Language Specification](../README.md). Status: Draft 0.1.

## 1. Precedence & associativity

Every tier below desugars to a message send ([ADR-0012]); the productions just
fix how the parser groups tokens. The compiler's inliner for `and`/`or`/`if` is
a semantic concern, not a syntax one — see [Control Flow](../control-flow.md).

| Tier | Operators | Assoc |
|------|-----------|-------|
| 1 | assignment `=` `+= -= *= /= %=` | right |
| 2 | `??` (Option coalesce) | right |
| 3 | `or` | left |
| 4 | `and` | left |
| 5 | equality `== !=` | left |
| 6 | comparison `< <= > >=` (and `is` type-test) | left |
| 7 | bitwise OR `|` | left |
| 8 | bitwise XOR `^` | left |
| 9 | bitwise AND `&` | left |
| 10 | shifts `<< >>` | left |
| 11 | additive `+ -` | left |
| 12 | multiplicative `* / % ~/` | left |
| 13 | power `**` | right |
| 14 | unary prefix `- ~ ! not` | right |
| 15 | postfix `.` `?.` call `(...)` trailing-block `::` | left |
| 16 | primary | — |

> Range (`.. ...`) is a reserved-inactive binary operator ([ADR-0032]); its
> precedence slot is not yet fixed — pending U-LEX. `is`/`as` beyond the
> import-alias form (`import Foo as Bar`) are not fully specified; treated
> here only as the tier-6 type-test spelling of `is`.

```
assignment    := target ( "=" | "+=" | "-=" | "*=" | "/=" | "%=" ) assignment
               | coalesce
target        := postfix                (* IDENT, FIELD, or a "." property send *)

coalesce      := or_expr [ "??" coalesce ]

or_expr       := and_expr { "or" and_expr }

and_expr      := equality { "and" equality }

equality      := comparison { ( "==" | "!=" ) comparison }

comparison    := bit_or { ( "<" | "<=" | ">" | ">=" | "is" ) bit_or }
bit_or        := bit_xor { "|" bit_xor }
bit_xor       := bit_and { "^" bit_and }
bit_and       := shift { "&" shift }
shift         := additive { ( "<<" | ">>" ) additive }
additive      := multiplicative { ( "+" | "-" ) multiplicative }
multiplicative:= unary { ( "*" | "/" | "%" | "~/" ) unary }
unary         := ( "-" | "~" | "!" | "not" ) unary
               | power
power         := postfix [ "**" unary ]

postfix       := primary { send_tail }
send_tail     := "." [ "?" ] ( IDENT | keyword ) [ arg_list ]
               | "(" [ arg { "," arg } [ "," ] ] ")"     (* call sugar, §2 *)
               | "::" ( IDENT | "#" selector )
               | block_literal                           (* trailing block, §2 *)

primary       := literal | grouping | tuple | list | map | block
               | symbol | method_ref | "self" | "super" | IDENT | FIELD
```

Every binary and unary operator here is sugar for a message send: `a + b` is
`a.+(b)`, `a ** b` is `a.**(b)`, `a & b` is `a.&(b)`, and `~a` is `a.~()` — see §2 and [ADR-0012].

Power's right operand is `unary`, rather than `postfix`, deliberately. Therefore `2 ** -2`
groups as `2 ** (-2)`, while a prefix on the left binds outside power: `-2 ** 2` groups as
`-(2 ** 2)`. This is the Python power rule ratified by PDR-0027.

## 2. Message sends

Dot notation is the primary send syntax; a bare identifier or symbol after
`.` names the selector.

```phalcom
recv.name                       // name
recv.add(1, 2)                  // add(_,_)
recv.move(to: p, duration: 2)   // move(to,duration)
a + b                           // +(_)
recv.name = v                   // name=(put)
```

```
send      := postfix "." ( IDENT | keyword ) [ arg_list ]
assign_send := postfix "." IDENT "=" expr        (* property assignment, name=(put) *)
keyword   := IDENT { IDENT ":" }                  (* one or more labeled parts *)
```

A keyword is legal directly after `.`, including keywords that collide with

--- docs/spec/current/blocks.md ---
# Blocks

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

Blocks are the keystone construct. A block, a lambda, a method body, and a getter
body all **share one closure representation** — the same closure machinery, spelled
at different levels of ceremony. `Block` is a real class
([Object Model](object-model.md)). A `Method` and a `Block` are **siblings** under
the abstract `Function` root ([ADR-0006](../../adr/0006-function-as-abstract-callable-root.md));
a `Method` is **not** a `Block` — it carries a selector, holder, and receiver that a
`Block` does not (see [Functions](functions.md)).

## 1. Forms

```phalcom
n => n * 2                      // unbraced, single parameter, expression body
{ acc, n => acc + n }           // braced, any number of parameters
{ System.print("hi") }          // braced, zero parameters
{ x => ... }                    // canonical form
```

`=>` has exactly one meaning throughout the language: **"yields."** It is the same
token in a block header and in a method expression body ([Classes §Methods](classes.md)).

## 2. The unbraced form is expression-only

An unbraced arrow's body is a **single expression** — no statements, no `return`,
no brace-delimited body.

- `x => { ... }` is an arrow that **returns a block**, exactly as it reads — not
  "an arrow with a block body."
- There is no way to write a JS-looking arrow containing `return`. This makes
  non-local return (§5) safe *by construction*.

## 3. Unbraced arrows are single-parameter only

`n, x => n * 2` is **illegal**. The comma already separates call arguments and
tuple elements; giving it a third job creates a true ambiguity:

```phalcom
f(n, x => n * 2)   // two args, or one two-param lambda? unresolvable
```

The brace is what makes the multi-parameter comma safe — inside `{ }` the comma has
no other job.

## 4. Trailing block sugar

A block literal following a call's argument list is passed as the **final
argument**, filling the last declared parameter.

```phalcom
numbers.map { n => n * 2 }
numbers.fold(initial: 0, using: { acc, n => acc + n })
5.times { System.print("hi") }
file.open("data.txt") { f => f.readAll() }
```

Selector identity is unaffected: `cond.ifTrue { ... }` and `cond.ifTrue({ ... })`
are both sends of `ifTrue(_)`.

## 5. Non-local return

Every block captures the identity of the method frame in which it was created.
`return` inside a block unwinds to **that** frame and returns from the enclosing
*method*, not from the block.

```phalcom
findNegative(numbers) {
  numbers.each { n =>
    (n < 0).ifTrue { return Some(n) }   // exits findNegative
  }
  None
}
```

The last expression of a block is its value ([Classes §Implicit return](classes.md)),
so `return` is only ever needed for *early* exit.

**Escaping blocks.** A block may outlive its home frame. Give each block a **frame
token** — a frame pointer plus a generation counter. On non-local return, compare
the token against the live frame; if the generation does not match, raise
`DeadFrameError`. A cheap integer comparison converts a memory-safety hazard into a
clean runtime error. (Smalltalk raises `BlockCannotReturn` here.)

## 6. No `break` / `continue`

There is **no** `break` or `continue` inside a block. Early exit from a block is
`return`. `break`/`continue` exist only inside `while`/`for` sugar
([Control Flow](control-flow.md)), where they compile directly to jumps.

## 7. Blocks are objects

```phalcom
blk.call()      blk.call(1, 2)

--- docs/spec/current/string-interpolation.md ---
# Phalcom String Literals and Interpolation

**Status:** Accepted by [PDR-0029](../../pdr/0029-string-literals-and-interpolation-completion.md)
**Target:** Phalcom language specification
**Date:** 2026-07-22

## 1. Scope

This document defines the lexical grammar, parsing rules, evaluation semantics, diagnostics, source ranges, and conformance requirements for double-quoted string literals and string interpolation.

It supersedes any earlier rule that:

- treats an interpolation body as raw text balanced only by counting `(` and `)`;
- lowers interpolation through `String.new(expression)`;
- preserves unknown backslash escapes literally; or
- permits more than one top-level expression inside a single interpolation.

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

## 2. Source model

Phalcom source is UTF-8.

All source ranges used by diagnostics and front-end data structures are half-open UTF-8 byte ranges:

```text
start..end
```

`start` is inclusive and `end` is exclusive. A point diagnostic is represented by a zero-width range where `start == end`.

A scanner MUST advance by complete UTF-8 scalar values when it is not consuming fixed ASCII syntax. It MUST NOT split a UTF-8 scalar or report a range ending inside one.

## 3. Lexical grammar

### 3.1 String literal

A string literal is either a single-line double-quoted string or a triple-quoted multiline text block ([PDR-0034](../../pdr/0034-multiline-string-text-blocks.md)):

```text
string-literal ::= single-line-string | multiline-text-block

single-line-string ::= `"` string-part* `"`
multiline-text-block ::= `"""` hspace* newline multiline-body margin `"""`

string-part    ::= literal-character
                 | escape-sequence
                 | interpolation

escape-sequence ::= `\"`
                  | `\\`
                  | `\n`
                  | `\t`
                  | `\r`

interpolation  ::= `\(` interpolation-expression `)`
```

`literal-character` is any Unicode scalar value other than `"` or `\`.

Physical LF and CRLF are invalid inside a single-line double-quoted string; multiline text blocks (`"""`) are used for strings spanning physical lines.

### 3.2 Escape values

The supported escapes decode as follows:

| Source | String value |
|---|---|
| `\"` | U+0022 QUOTATION MARK |
| `\\` | U+005C REVERSE SOLIDUS |
| `\n` | U+000A LINE FEED |
| `\t` | U+0009 CHARACTER TABULATION |
| `\r` | U+000D CARRIAGE RETURN |

`\(` is not a character escape. It opens an interpolation.

No other escape is valid. An unknown escape such as `\q` is a syntax error. It MUST NOT be preserved as two literal characters.

A backslash is processed left-to-right and consumes its escape partner. Therefore:

```phalcom
"\\("
```

contains the literal two-character text `\(` and does not open an interpolation.

Likewise:

```phalcom
"\\\(value)"
```

contains one literal backslash followed by one interpolation.

### 3.3 Quotes

--- docs/spec/current/control-flow.md ---
# Control Flow

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

## 1. Sugar

`if`, `else`, `while`, `for` are **keyword sugar** over message sends. They exist
because they are what a newcomer expects (Invariant 6), and because both spellings
compile to identical opcodes — nothing is lost.

```phalcom
if (c) { ... } else { ... }          // === c.ifTrue { ... }.ifNone { ... }
while (c) { ... }                    // === { c }.whileTrue { ... }
for (x in xs) { ... }                // iteration protocol (see iteration.md)
```

`for` lowers to the **cursor iteration protocol**
([iteration.md](iteration.md)) — a `while` loop over `iterate(_)`/`iteratorValue(_)`,
**not** `xs.each { … }` — so `break`/`continue` work as loop control. `.each` is the
full-traversal combinator over the same protocol.

## 2. `and` / `or` / `??` short-circuit

A short-circuiting operator **cannot** be an eager send — message arguments
evaluate before the send. Smalltalk's answer, which Phalcom adopts: the right-hand
side is a **block**.

```phalcom
a and b     // a.and { b }      -> selector and(_)
a or  b     // a.or  { b }      -> selector or(_)
a ?? b      // a.orElse { b }   -> Option
```

Laziness falls out of the object model for free. `and` and `or` are ordinary
methods on `Bool` and can be overridden.

## 3. The inliner — load-bearing

When the compiler sees a send of a **sacred selector** whose block arguments are
**literal blocks at the call site**, it emits jump opcodes instead of a send.

Sacred selectors: `ifTrue(_)`, `ifFalse(_)`, `ifTrue(_)ifFalse(_)`, `and(_)`,
`or(_)`, `whileTrue(_)`, `repeat(_)`. (Comma form throughout, consistent with
[Selectors, Symbols & References §1](selectors.md#1-selector-identity); the
paired form `ifTrue(_)ifFalse(_)` is two block-typed positional slots, not
labels — see [Open Questions](open-questions.md) item on §7 #2 of that doc.)

The inlined code is guarded by a receiver type check that **deoptimizes to a real
send** if the receiver is not the expected `Bool` / `Block`. Result: zero closure
allocation and zero call frames on the common path, full genericity when someone
actually needs it.

This must land **early** (Invariant 5). If blocks are slow, users learn to avoid
them and every other decision in the spec unravels.
</content>

--- docs/implementation/LANG001-language-surface/PROGRAM.md ---
---
id: LANG001
category: LANG
kind: completion-and-correction
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# LANG001 — language surface

This program owns source-language syntax, lexical behavior, blocks, migration
notes, and historical language-surface implementation records.

--- docs/implementation/LANG003-language-semantics/PROGRAM.md ---
---
id: LANG003
category: LANG
kind: semantics
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# LANG003 — language semantics

This program owns the repository-grounded language-semantics implementation and
handoff records.
