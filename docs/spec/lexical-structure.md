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

## 4. Literals

```phalcom
42        1_000_000       3.1415          // numbers (digit separators allowed)
"hello"   "{name} is {age}"              // strings, with interpolation
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

Interpolation uses `{expr}`. `\{` escapes a literal brace.

```phalcom
"{name} is {age} years old"
"a literal \{ brace"
```

Each `{expr}` desugars to a `toString` send and string concatenation.

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
</content>
