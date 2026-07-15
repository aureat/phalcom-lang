# Lexical Grammar

Part of the [Phalcom Language Specification](../README.md). Status: Draft 0.1.

## 1. Source and encoding

Source files are UTF-8. Identifiers, keywords, and operators are restricted to
ASCII; non-ASCII bytes may only occur inside comments and string literals. The
lexer injects a single synthetic `EOF` token at the end of the token stream, so
every production that consumes tokens can rely on a terminator existing.

```
SOURCE := { TOKEN } EOF
```

## 2. Whitespace and newlines

Spaces, tabs, and form feed are trivia: they separate tokens but never appear
in the token stream themselves.

```
TRIVIA  := " " | "\t" | "\f"
NEWLINE := "\n" | "\r" "\n"
```

`NEWLINE` is significant — it is the statement terminator (see
[`statements-and-declarations.md`](statements-and-declarations.md)). Full rationale
for this design is in [`../lexical-structure.md#1`](../lexical-structure.md).

### 2.1 Newline suppression

The lexer swallows a `NEWLINE` — emits nothing, effectively treating the line
break as trivia — whenever the *previous* token cannot end a statement. This
is a one-sided lexer state machine keyed only on the previous token, not
parser-level automatic semicolon insertion.

A `NEWLINE` is suppressed immediately after any of:

```
SUPPRESS-AFTER :=
    BINARY-OP | ASSIGN-OP | "and" | "or" | "not"
  | "," | "(" | "{" | "[" | "." | "::" | ":" | "=>" | "->" | "??" | "?."
```

Every other position emits a real `NEWLINE` token.

## 3. Comments

```
LINE-COMMENT  := "//" { any-char-but-NEWLINE }
BLOCK-COMMENT := "/*" { any-char } "*/"
```

A line comment ends at the next `NEWLINE`; the newline itself is not consumed
by the comment and survives as its own token (subject to the suppression rule
in §2.1). Block comments do not nest — the first `*/` closes the comment.

## 4. Identifiers and fields

```
IDENT := ALPHA { ALPHA | DIGIT }
FIELD := "_" { ALPHA | DIGIT }
ALPHA := "A".."Z" | "a".."z"
DIGIT := "0".."9"
```

`FIELD` (a leading underscore) denotes a private field when used inside a
class body, and a module-private top-level name when used at module scope
([ADR-0027]). Capitalizing a class name's initial letter is a project
convention enforced by style, not a distinct token class — `Person` and
`person` both lex as `IDENT`.

## 5. Keywords

| Keyword | | Keyword | | Keyword |
|---|---|---|---|---|
| `let` | | `while` | | `and` |
| `var` | | `for` | | `or` |
| `class` | | `in` | | `not` |
| `construct` | | `break` | | `is` |
| `static` | | `continue` | | `true` |
| `self` | | `return` | | `false` |
| `super` | | `import` | | `throw` |
| `if` | | `as` | | |
| `else` | | | | |

```
KEYWORD :=
    "let" | "var" | "class" | "self" | "super"
  | "if" | "else" | "while" | "for" | "in" | "break" | "continue" | "return"
  | "and" | "or" | "not" | "is" | "true" | "false" | "import" | "as" | "throw"
```

### 5.1 Contextual keywords

Reserved only in a specific grammatical position; elsewhere they lex as
ordinary `IDENT` and may be used as identifiers.

| Word | Reserved position |
|---|---|
| `extends` | class header (see [`statements-and-declarations.md`](statements-and-declarations.md)) |
| `try`, `catch`, `on`, `ensure` | error-handling clauses ([ADR-0031]) |

### 5.2 Reserved-inactive

`fn` is reserved but not currently bound to any production — it lexes as a
keyword token so it cannot be used as an identifier, but no grammar rule
consumes it yet.

### 5.3 No `nil`

There is no `nil`, `null`, or `none` keyword in Phalcom. Absence is
represented by the `None` value of the abstract `Option` type; there is no
lexical token for it ([ADR-0007]).

## 6. Numeric literals

Numeric surface syntax splits `Int` (exact, auto-promoting on overflow) and
`Float` on the presence of a decimal point ([ADR-0024]).

```
INT      := DIGIT { DIGIT | "_" }
FLOAT    := DIGIT { DIGIT | "_" } "." DIGIT { DIGIT | "_" }
```

`1` lexes as `INT`; `1.0` lexes as `FLOAT`. Digit-group separators (`_`) are
permitted only directly between two digits — a leading, trailing, or doubled
`_` is not part of either production.

`.` is read as a decimal point only when the next character is a digit; in
every other position it lexes as the member-access operator. This is why
`1..2` lexes as `INT ".." INT` (range) rather than a malformed float followed
by `.2`.

> **Unresolved / not yet specified:** hexadecimal, octal, binary, and
> exponent (scientific-notation) numeric forms have no production. See
> [`../implementation-status.md`](../implementation-status.md).

```phalcom
let count = 1_000_000   // Int, digit-separated
let ratio = 0.5          // Float
let range = 1..10        // Int .. Int, not "1." followed by ".10"
```

## 7. String literals

```
STRING := "\"" { STRING-SEGMENT } "\""
```

Double-quoted; a string literal may span multiple physical lines (an embedded
`NEWLINE` character is part of the string content, not a token).

### 7.1 Interpolation ([ADR-0022])

A string containing one or more `\(expr)` interpolations is an *interpolated
string*. Each `\(expr)` desugars to a `toString` send on the evaluated
expression, concatenated with the surrounding literal segments.

```
STRING-SEGMENT   := LITERAL-SEGMENT | INTERP-SEGMENT
LITERAL-SEGMENT  := { STRING-CHAR }        (* non-empty run of literal chars *)
INTERP-SEGMENT   := "\(" expr ")"          (* expr per expressions.md *)
```

`\\(` lexes as a literal `\(` (the backslash is escaped, so the sigil does not
fire). The two escapes with defined meaning inside a string are:

| Escape | Meaning |
|---|---|
| `\\` | literal backslash |
| `\(` | begins an interpolation segment; `\\(` for a literal `\(` |

> **Unresolved:** the full escape set (`\n`, `\t`, `\"`, unicode escapes, etc.)
> beyond `\\` and `\(` is not yet specified. See
> [`../implementation-status.md`](../implementation-status.md).

```phalcom
let name = "Alice"
let msg  = "Hello, \(name)!"      // interpolated: literal + expr + literal
let lit  = "price: \\(tax)"       // literal backslash-paren, not interpolated
```

## 8. Boolean literals

```
BOOL := "true" | "false"
```

Lexed as the keywords `true` and `false` (§5); there is no separate boolean
token class.

## 9. Symbol literals (`#`)

Symbol literals name a message selector or a bare name as a first-class
value ([ADR-0012]; semantics in [`../selectors.md#2-symbol-literals-`](../selectors.md)).

```
SYMBOL           := NAME-SYMBOL | SELECTOR-SYMBOL
NAME-SYMBOL      := "#" IDENT
SELECTOR-SYMBOL  := "#" SELECTOR-BODY
SELECTOR-BODY    := IDENT "(" SELECTOR-ARGS ")" | OPERATOR-CHARS | "[" "]"
SELECTOR-ARGS    := SELECTOR-ARG { "," SELECTOR-ARG }
SELECTOR-ARG     := "_" | IDENT
```

A symbol is a single atomic token: `#` must be immediately adjacent to the
name or selector body, with no intervening trivia — `# move` is **not** a
selector symbol (it lexes as `#` — a lex error — followed by `move`).
Adjacency is likewise required before the parenthesized argument list:
`#move (a,b)` is **not** a selector symbol; the space breaks it.

Whitespace *inside* the parens is stripped and the argument list is
canonicalized at intern time, so `#move(_,to,duration)` and
`#move(_, to, duration)` intern to the same symbol. Positional slots (`_`)
must precede all labeled slots within the argument list — `#move(to,_)` is a
lex error, not merely unconventional.

```phalcom
#move                    // name symbol
#move(_,to,duration)     // selector symbol, one positional + two labels
#+                       // operator selector symbol
#[]                      // index-selector symbol
```

A `#!` at byte offset 0 of the source file (a shebang line) is skipped by the
lexer and does not produce a `SYMBOL` token; `#!` anywhere else lexes normally.

## 10. Method-reference token (`::`)

`::` is a postfix token producing a first-class reference to a method
([`../selectors.md#3-method-references-`](../selectors.md)). What follows the
token selects the form: a `#` immediately after `::` yields the **Pinned**
form (bound to a specific selector symbol); anything else yields the **Open**
form (late-bound, resolved by lookup at the call site).

```
METHOD-REF := "::" [ "#" ]   (* target production: expressions.md *)
```

```phalcom
obj::#move        // Pinned form
obj::move         // Open form
```

## 11. Attribute token (`@`)

`@` is a prefix token attaching a declaration-level attribute
([`../selectors.md#4-attributes-`](../selectors.md)). Planned, not yet built.

```
ATTRIBUTE := "@" IDENT
```

```phalcom
@construct
@get
@set
```

## 12. Operators and punctuation

| Group | Tokens |
|---|---|
| Arithmetic | `+` `-` `*` `/` `%` `~/` (`~/` is integer division, [ADR-0024]) |
| Comparison | `==` `!=` `<` `<=` `>` `>=` |
| Assignment | `=`, compound `+=` `-=` `*=` `/=` `%=` |
| Option | `??` `?.` |
| Member | `.` |
| Range | `..` `...` (reserved-inactive: `a..b` inclusive, `a...b` exclusive, [ADR-0032]) |
| Spread / rest | prefix `*` |
| Label / map | `:` |
| Arrow | `=>` (block/expression body); `->` reserved-inactive |
| Delimiters | `(` `)` `{` `}` `[` `]` |
| Separators | `,` `;` `NEWLINE` |
| Prefix | `-` (negate), `!` (not) |

A lone `?` is not a token — it is reserved but currently unassigned to any
production; `??` and `?.` are each lexed as a single two-character token, not
as `?` followed by `?`/`.`.

## 13. Token grammar

The complete set of terminal productions defined above, collected for
reference:

```
EOF          := (* synthetic end-of-input marker *)
NEWLINE      := "\n" | "\r" "\n"
TRIVIA       := " " | "\t" | "\f"

LINE-COMMENT  := "//" { any-char-but-NEWLINE }
BLOCK-COMMENT := "/*" { any-char } "*/"

ALPHA := "A".."Z" | "a".."z"
DIGIT := "0".."9"
IDENT := ALPHA { ALPHA | DIGIT }
FIELD := "_" { ALPHA | DIGIT }

KEYWORD :=
    "let" | "var" | "class" | "self" | "super"
  | "if" | "else" | "while" | "for" | "in" | "break" | "continue" | "return"
  | "and" | "or" | "not" | "is" | "true" | "false" | "import" | "as" | "throw"

INT   := DIGIT { DIGIT | "_" }
FLOAT := DIGIT { DIGIT | "_" } "." DIGIT { DIGIT | "_" }
BOOL  := "true" | "false"

STRING           := "\"" { STRING-SEGMENT } "\""
STRING-SEGMENT   := LITERAL-SEGMENT | INTERP-SEGMENT
LITERAL-SEGMENT  := { STRING-CHAR }
INTERP-SEGMENT   := "\(" expr ")"

SYMBOL           := NAME-SYMBOL | SELECTOR-SYMBOL
NAME-SYMBOL      := "#" IDENT
SELECTOR-SYMBOL  := "#" SELECTOR-BODY
SELECTOR-BODY    := IDENT "(" SELECTOR-ARGS ")" | OPERATOR-CHARS | "[" "]"
SELECTOR-ARGS    := SELECTOR-ARG { "," SELECTOR-ARG }
SELECTOR-ARG     := "_" | IDENT

METHOD-REF  := "::" [ "#" ]
ATTRIBUTE   := "@" IDENT

BINARY-OP  :=
    "+" | "-" | "*" | "/" | "%" | "~/"
  | "==" | "!=" | "<" | "<=" | ">" | ">="
  | ".." | "..."
ASSIGN-OP  := "=" | "+=" | "-=" | "*=" | "/=" | "%="
OPTION-OP  := "??" | "?."
PUNCT      :=
    "." | "::" | ":" | "=>" | "->"
  | "(" | ")" | "{" | "}" | "[" | "]"
  | "," | ";"
```

[ADR-0007]: ../../../adr/0007-option-as-abstract-with-some-none.md
[ADR-0012]: ../../../adr/0012-selector-signature-encoding-and-dispatch.md
[ADR-0022]: ../../../adr/0022-string-interpolation-backslash-paren-sigil.md
[ADR-0024]: ../../../adr/0024-numeric-surface-split-int-float-and-division.md
[ADR-0027]: ../../../adr/0027-modules-as-files-with-public-by-default-imports.md
[ADR-0031]: ../../../adr/0031-error-handling-surface-syntax.md
[ADR-0032]: ../../../adr/0032-collections-representation-and-literals.md
