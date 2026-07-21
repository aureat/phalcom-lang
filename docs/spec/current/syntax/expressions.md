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
recv.name = v                   // name=(_)
```

```
send      := postfix "." ( IDENT | keyword ) [ arg_list ]
assign_send := postfix "." IDENT "=" expr        (* property assignment, name=(_) *)
keyword   := IDENT { IDENT ":" }                  (* one or more labeled parts *)
```

A keyword is legal directly after `.`, including keywords that collide with
reserved words elsewhere (`recv.class`, `recv.new(...)`) — dot position
disambiguates.

**Argument list.**

```
arg_list := "(" [ arg { "," arg } [ "," ] ] ")"
arg      := [ IDENT ":" ] [ "*" ] expr
```

An `arg` is a positional expression, a labeled `label: expr` (the label joins
the selector — [Messages & Selectors §2]), or a spread `*expr` that splices a
collection's elements into the argument list ([Messages & Selectors §5]).

**Call sugar.** A non-`.` callee applied to an argument list sends `call(...)`:

```phalcom
blk(1, 2)          // ≡ blk.call(1, 2)
callee(args)        // any callable value, not just blocks
```

**Trailing block.** A block literal immediately following a call's argument
list is passed as the call's final positional argument:

```phalcom
recv.m { ... }        // ≡ recv.m({ ... }) — both send m(_)
```

See [Blocks §4](../blocks.md#4) for the full trailing-block rules, including
interaction with zero-arg calls (`recv.m { ... }` when `m` takes no other
arguments).

## 3. Option operators `?.` and `??`

```phalcom
opt?.m(a)     // ≡ opt.map { x => x.m(a) }
a ?? b        // ≡ a.orElse { b }
```

`?.` is a postfix chain member (tier 14): left-associative, and a `None`
anywhere in a `?.` chain short-circuits the rest of the chain without
evaluating further sends. `??` (tier 2) is right-associative: looser than
comparison and arithmetic, tighter than assignment, so `a ?? b == c` parses
as `a ?? (b == c)` and `x = a ?? b` parses as `x = (a ?? b)`.

See [Lexical Structure §9](../lexical-structure.md#9) and [ADR-0007].

## 4. Primary expressions

```
primary := literal | grouping | tuple | list | map | block
         | symbol | method_ref | "self" | "super" | IDENT | FIELD
```

**Literals.** `INT`, `FLOAT` ([ADR-0024]), `STRING` with `\( )` interpolation
([ADR-0022]), `true` / `false`. There is no `nil` literal — absence is
expressed with `Option`, i.e. `None` ([ADR-0007]; [Values & Absence](../values-and-absence.md)).

**Grouping vs. tuple.** A trailing comma disambiguates:

```
grouping := "(" expr ")"
tuple    := "(" ")"
          | "(" expr "," [ expr { "," expr } [ "," ] ] ")"
```

```phalcom
(1 + 2)        // grouping — an Int
(1,)           // one-element tuple
(1, 2, 3)      // three-element tuple
()             // empty tuple
```

`(a,)` — the trailing comma is required for a one-element tuple, since `(a)`
is plain grouping ([ADR-0032]).

**List** ([ADR-0029]).

```
list := "[" [ expr { "," expr } [ "," ] ] "]"
```

```phalcom
[]                 // empty list
[1, 2, 3]
[1, 2, 3,]         // trailing comma allowed
```

A list literal is construction only — there is no subscript sugar; element
access is the message `at(_)`. A list literal is itself a `primary`.

**Map** ([ADR-0032]).

```
map       := "{" map_entry { "," map_entry } [ "," ] "}"
map_entry := ( IDENT | expr ) ":" expr
```

```phalcom
{ x: 1, y: 2 }        // bare-identifier keys are symbols: #x, #y
{ (k): v }            // parenthesized expression key
```

`{}` alone is the **empty block**, not an empty map — the empty map is
constructed with `Map.new()`. A set literal `#{...}` is reserved-inactive;
use `Set(...)` or `Set.new()` instead.

**Brace disambiguation.** One token of lookahead after `{` in expression
position picks the production (full rule: [Lexical Structure §6](../lexical-structure.md#6)):

| Lookahead after `{` | Parses as |
|---|---|
| `IDENT :` | map |
| `IDENT ,` | block with params |
| `IDENT =>` | block with params |
| `}` | empty block |
| anything else | zero-param block, expression body |

**Block literals.**

```
block          := "{" [ block_params "=>" ] statements "}"
block_params   := IDENT { "," IDENT }
unbraced_block := IDENT "=>" expr
```

```phalcom
{ }                        // zero-param, empty body
{ System.print("hi") }     // zero-param block
{ x => x * 2 }             // one param, braced
{ acc, n => acc + n }      // multi-param, braced — requires braces
x => x * 2                 // unbraced single-param, expression body only
```

`=>` means "yields" in every position it appears. The unbraced arrow form is
single-parameter and expression-body only — it cannot contain statements or a
brace-delimited body. See [Blocks](../blocks.md) for the full rationale
(comma ambiguity, non-local return).

**Symbol / method-reference / self / super.**

```
symbol     := "#" ( IDENT | selector )
method_ref := postfix "::" ( IDENT | "#" selector )
```

`#name` / `#sel` builds a `Symbol` ([Selectors §2](../selectors.md#2)).
`recv::name` and `recv::#sel` build an Open or Pinned method reference — the
bound/unbound method-reference family ([Selectors §3](../selectors.md#3)).
`self` is the current receiver; `super.m(a)` is a super-send — lookup begins
in the superclass of the method's holder, not the receiver's class.

**Spread `*`.** `*expr` is legal only in call arguments, collection element
lists, and parameter lists. Everywhere else `*` is the multiply operator (tier
12) — see [Lexical Structure §8](../lexical-structure.md#8).

## 5. No cascades

Phalcom has no cascade syntax. `;` is only a statement separator, never a
message-chaining operator.

[ADR-0007]: ../../../adr/0007-option-as-abstract-with-some-none.md
[ADR-0012]: ../../../adr/0012-selector-signature-encoding-and-dispatch.md
[ADR-0022]: ../../../adr/0022-string-interpolation-backslash-paren-sigil.md
[ADR-0024]: ../../../adr/0024-numeric-surface-split-int-float-and-division.md
[ADR-0025]: ../../../adr/0025-external-internal-parameter-names.md
[ADR-0029]: ../../../adr/0029-list-literal-syntax.md
[ADR-0032]: ../../../adr/0032-collections-representation-and-literals.md
