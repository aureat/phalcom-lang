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

`**` is one `BINARY-OP` token. The lexer must take the longest match, so `**` never becomes two
adjacent `*` tokens. It is power, not a compound assignment; `**=` is not a token.

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
| `const` | | `break` | | `is` |
| `continue` | | `true` | | `false` |
| `self` | | `return` | | `super` |
| `import` | | `throw` | | `as` |
| `if` | | `else` | | |

```
KEYWORD :=
    "let" | "const" | "class" | "self" | "super"
  | "if" | "else" | "while" | "for" | "in" | "break" | "continue" | "return"
  | "and" | "or" | "not" | "is" | "true" | "false" | "import" | "as" | "throw"
```

### 5.1 Reserved names and migration spellings

`construct` and `constructor` are reserved names. User declarations may not
define either name as a declaration, selector family, or attribute class.

`construct` and `static` are retired declaration spellings, not canonical
keywords. Parsers may recognize their legacy positions long enough for the
compiler to emit non-fatal migration hints: `@constructor
new(...) { ... }` →
`@constructor`, `static ...` → `@class`. A method-shaped `class name(...) { ... }`
form receives the same `@class` hint.

### 5.2 Contextual keywords

Reserved only in a specific grammatical position; elsewhere they lex as
ordinary `IDENT` and may be used as identifiers.

| Word | Reserved position |
|---|---|
| `try`, `catch`, `on`, `ensure` | error-handling clauses ([ADR-0031]) |

### 5.3 Reserved-inactive

`fn` is reserved but not currently bound to any production — it lexes as a
keyword token so it cannot be used as an identifier, but no grammar rule
consumes it yet.

### 5.4 No `nil`

There is no `nil`, `null`, or `none` keyword in Phalcom. Absence is
represented by the `None` value of the abstract `Option` type; there is no
lexical token for it ([ADR-0007]).

## 6. Numeric literals

Numeric syntax is specified by [Numeric literals](../../library/numbers/numeric-literals.md), ratified by
PDR-0026. It supplies exact radix `Int` literals, decimal exponent `Float` literals,
separator rules, and atomic malformed-literal diagnostics.

```
INT   := DEC-INT | BIN-INT | OCT-INT | HEX-INT
FLOAT := DEC-DIGITS "." DEC-DIGITS [ EXPONENT ]
       | "." DEC-DIGITS [ EXPONENT ]
       | DEC-DIGITS EXPONENT
```

`5.` is invalid: trailing-dot floats are excluded so `5.toString` stays a dot send and
`5..2` stays a range. `0x_FF_A0_00`, `.25`, and `6.02e-23` are valid.

## 7. String literals

```
STRING := "\"" { STRING-SEGMENT } "\""
```

Double-quoted strings do not span physical lines. A raw `NEWLINE` is invalid
inside the literal; use `\n` or `\r\n` escapes for embedded line breaks. Multiline string literal
syntax is deferred by [PDR-0029](../../../pdr/0029-string-literals-and-interpolation-completion.md).

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
fire). The defined escapes inside a string are:

| Escape | Meaning |
|---|---|
| `\\` | literal backslash |
| `\"` | literal quotation mark |
| `\n` | line feed |
| `\t` | horizontal tab |
| `\r` | carriage return |
| `\(` | begins an interpolation segment; `\\(` for a literal `\(` |

Every other escape is an error. The stable diagnostic codes are
`string.invalid_escape`, `string.interpolation.unterminated`,
`string.interpolation.empty`, and `string.raw_newline`.

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
#&                       // binary operator selector
#~                       // nullary operator selector
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
| Bitwise | `&` `|` `^` `~` `<<` `>>` (PDR-0020) |
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
| Prefix | `-` (negate), `~` (bitwise not), `!` / `not` (boolean not) |

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
    "let" | "const" | "class" | "self" | "super"
  | "if" | "else" | "while" | "for" | "in" | "break" | "continue" | "return"
  | "and" | "or" | "not" | "is" | "true" | "false" | "import" | "as" | "throw"

INT   := DEC-INT | BIN-INT | OCT-INT | HEX-INT
FLOAT := DEC-DIGITS "." DEC-DIGITS [ EXPONENT ]
       | "." DEC-DIGITS [ EXPONENT ]
       | DEC-DIGITS EXPONENT
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
    "+" | "-" | "*" | "/" | "%" | "~/" | "**"
  | "&" | "|" | "^" | "<<" | ">>"
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
