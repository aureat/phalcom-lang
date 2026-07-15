# Statements & Declarations

Part of the [Phalcom Language Specification](../README.md). Status: Draft 0.1.

## 1. Programs & statement termination

```
program     := { top_item } EOF
top_item    := class_decl | import_decl | statement
statement   := binding | return | expr_statement
             | if_stmt | while_stmt | for_stmt | break | continue
             | throw | try_stmt
```

A statement is terminated by `NEWLINE` or `;`. `;` separates multiple small
statements written on one line; both are optional at end of input. There is
**no parser-level ASI** — newline significance is a lexical concern, resolved
by the lexer before the parser ever sees a `NEWLINE` token
([Lexical Structure §1](../lexical-structure.md#1)).

A `{` at statement-start is a parse error: it reads as a (no-op) block
literal, not a compound statement opener, so `class`/`if`/`while`/`for` bodies
always require their own keyword to introduce the block.

## 2. Bindings — `let` / `var` ([ADR-0014])

```
binding := ("let" | "var") IDENT [ "=" expr ]
```

```phalcom
let name = "Ada"   // immutable — reassigning name is a compile error
var count = 0      // mutable
var seen           // None ([ADR-0007]) — no initializer required for var
```

`let` bindings are immutable; reassignment is an error. `var` bindings are
mutable. `var x` with no initializer reads as `None` ([ADR-0007]); `let x`
with no initializer is rejected — a `let` must be given its one value up
front.

## 3. `return`

```
return := "return" [ expr ]
```

Early exit from the enclosing method. A method's (and a block's) value is
otherwise its **last expression** — implicit return, see
[Classes §4](../classes.md#4-implicit-return). Inside a block, `return` is
**non-local**: it returns from the enclosing method, not just the block
([Blocks §5](../blocks.md#5)).

## 4. Expression statements

```
expr_statement := expr
```

Assignments (`x = v`, `obj.prop = v` — the latter desugaring to a send of
`prop=(_)`) and message sends are both expression statements; neither needs
extra statement-level syntax.

## 5. Class declarations

```
class_decl  := "class" IDENT [ "extends" IDENT ] "{" { member } "}"

member      := { attribute } member_body
attribute   := "@" IDENT [ "(" [ arg_list_body ] ")" ]

member_body := method_decl | getter_decl | setter_decl
             | field_init

method_decl    := method_name param_list method_body
getter_decl    := IDENT method_body
setter_decl    := IDENT "=" param_list method_body
field_init     := FIELD "=" expr

method_name    := IDENT | operator
operator       := "+" | "-" | "*" | "/" | "%"
                | "==" | "!=" | "<" | "<=" | ">" | ">="
                | "and" | "or" | "is"

method_body    := "=>" expr | block
```

```phalcom
class Point extends Shape {
  @constructor
  new(x:, y:) { _x = x; _y = y }

  x => _x                    // getter — selector `x`, distinct from `x()`
  y=(value) { _y = value }   // setter — selector `y=(_)`

  +(other) => Point.new(x: _x + other.x, y: _y + other.y)

  @static
  origin => Point.new(x: 0, y: 0)
}
```

`extends` is a **contextual keyword**, valid only in the class header; when
absent, the superclass defaults to `Object`
([Object Model §5](../object-model.md#5)). Single inheritance only; a class
naming itself or a supertype ancestor (a cycle) as its own superclass is
rejected. See [Classes](../classes.md) and [Object Model](../object-model.md)
for the semantics.

- `attribute` covers `@constructor` / `@static` / `@get` / `@set` and the contract
  forms ([Selectors, Symbols & References §4](../selectors.md#4-attributes-)).
  **It is load-bearing**: `member` has no `static` or `construct` slot of its own —
  class-side placement and constructor-ness are carried by `@static` and
  `@constructor`, which desugar into ordinary `method_decl`s before the rest of
  compilation
  ([ADR-0063](../../../adr/accepted/0063-constructors-are-ordinary-class-side-methods.md)).
- `@static` puts the member on the metaclass rather than the instance side.
- **Constructor**: `@constructor` on a `method_decl` allocates a fresh instance via
  `new_`, runs the body with `self` bound to it, and returns the instance implicitly.
  Multiple constructors on one class are distinguished by **selector**, not arity, and
  the name need not be `new` ([Classes §1](../classes.md#1-constructors)).
- **Method**: `method_name` may be an ordinary identifier or one of the
  operator spellings above — operators are ordinary methods with ordinary
  dispatch.
- **Getter**: no parameter list. `name` and `name()` are different selectors.
- **Setter**: `IDENT "=" param_list method_body` — selector `name=(_)`. If the
  parameter list is empty the parameter defaults to `value`.
- **Field initializer**: `FIELD "=" expr`, a `_`-prefixed field init. Carrying
  `@classField` puts the storage on the class object rather than on instances, per
  declaring class ([Classes §2.1](../classes.md)) — the `static` keyword that once
  marked this is gone. Instance fields carry no declaration syntax at all — they are
  implicitly declared by assignment inside a method, and reading a field never
  assigned anywhere in the class is a compile error
  ([Classes §2](../classes.md#2-fields)).
- `method_body := "=>" expr | block` — `=>` is general expression-body sugar,
  not limited to getters ([Classes §3](../classes.md#3-methods-accessors-operators)).

## 6. Parameters ([ADR-0025])

```
param_list := "(" [ param { "," param } ] ")"
param      := positional | labeled | rest

positional := IDENT
labeled    := IDENT ":"
            | IDENT IDENT ":"
rest       := "*" IDENT
```

```phalcom
@constructor new(name:, age:) { ... }     // labeled: label == binding
method move(to target:) { ... }           // labeled: external `to:`, body sees `target`
method sum(*values) { ... }               // rest: collects trailing positionals into a List
```

- **Positional**: bare `IDENT`.
- **Labeled**: `IDENT ":"` binds the label as its own name in the body; the
  two-identifier form `IDENT IDENT ":"` splits **external label** from
  **internal binding** — callers pass `to:`, the body reads `target`. The
  label, not the internal name, is part of selector identity
  ([Messages & Selectors](../messages-and-selectors.md)).
- **Rest**: `"*" IDENT` collects trailing positional arguments into a `List`.
  Must be the last parameter; at most one per list; a labeled parameter
  cannot be variadic (positional-only).
- No default-argument syntax is specified. This is a deliberate gap, not an
  oversight — see [Selectors, Symbols & References §7](../selectors.md#7)
  for the open design space.

## 7. Control-flow statements (surface sugar)

```
if_stmt    := "if" "(" expr ")" block [ "else" ( block | if_stmt ) ]
while_stmt := "while" "(" expr ")" block
for_stmt   := "for" "(" IDENT "in" expr ")" block
```

```phalcom
if (n > 0) { positive() } else if (n < 0) { negative() } else { zero() }
while (queue.isEmpty.not) { process(queue.next) }
for (item in items) { print(item) }
```

`if`/`else`, `while`, and `for` are keyword sugar over message sends and
desugared loops, not primitive control constructs — see
[Control Flow](../control-flow.md) for what each compiles to.
`for` specifically lowers to the **cursor iteration protocol**
([ADR-0035], [Iteration](../iteration.md)), not a full-traversal combinator.

`break` and `continue` are loop-control keywords, valid only inside `for` or
`while` bodies; they compile directly to jumps. Blocks have no `break` —
non-local exit from a block uses `return` instead
([Blocks §6](../blocks.md#6)).

`and`, `or`, `not`, and `??` short-circuit via block arguments rather than
being primitive operators — see
[Control Flow §2](../control-flow.md#2-and--or--short-circuit).

## 8. Error-handling statements ([ADR-0031])

```
throw       := "throw" expr

try_stmt    := "try" block { on_clause } [ catch_clause ] [ ensure_clause ]
on_clause   := "on" IDENT IDENT block
catch_clause := "catch" IDENT block
ensure_clause := "ensure" block
```

```phalcom
try {
  risky()
} on IOError e {
  log(e)
} catch e {
  fallback(e)
} ensure {
  cleanup()
}
```

`throw expr` requires `expr` to be an `Error`; it is sugar for
`expr.raise()`. `on`, `catch`, and `ensure` are contextual keywords, valid
only as clauses of a `try` statement:

- `on T e { ... }` ≡ `.on(T) { e => ... }`
- `catch e { ... }` ≡ `.on(Error) { e => ... }`
- `ensure { ... }` ≡ `.ensure { ... }`

See [Error Handling](../error-handling.md) for the underlying handler-chain
semantics.

## 9. Modules & imports ([ADR-0027])

```
import_decl := "import" IDENT
             | "import" IDENT { "," IDENT } "from" IDENT
             | "import" IDENT "as" IDENT
```

```phalcom
import http                       // qualified — http.Client
import Client, Request from http  // selective
import http as net                // aliased — net.Client
```

A file is a module. Top-level names are public unless `_`-prefixed; there is
no separate export list.
