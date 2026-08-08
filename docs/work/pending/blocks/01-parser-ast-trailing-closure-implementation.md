# Task 1 — Parser, AST Grammar, and Trailing-Closure Syntax

**Project:** Phalcom  
**Repository:** `aureat/phalcom-lang`  
**Baseline inspected:** `main` at `32580cc6599ccebd31447d69e2557bfcacdcd95f`  
**Status:** Implementation-ready  
**Primary implementation area:** `phalcom-ast/src/parser.rs`  
**Depends on:** transition-1 Tasks 1–2; collection work through the current A–C baseline  
**Must preserve:** existing `Expr::Block`, `BlockExpr`, `MethodCallExpr`, `Argument`, selector encoding, lexer `Token::Pipe`, and all current bitwise-OR semantics

---

## 1. Objective

Replace the legacy anonymous-block source forms:

```phalcom
x => x + 1

{ x => x + 1 }

{ x, y =>
  x + y
}

{ doSomething() }
```

with a single Rust-style closure literal syntax:

```phalcom
|x| x + 1

|x| { x + 1 }

|x, y| {
  x + y
}

|| {
  doSomething()
}
```

and add the ratified trailing-closure call surface:

```phalcom
numbers
  .map |value| { value * 2 }
  .filter |value| { value > 3 }

users
  .removeAll where: |user| { user.expired }
  .any where: |user| { user.expired }

result.match
  ok: |value| {
    use(value)
  },
  err: |error| {
    handle(error)
  }

predicate
  .ifTrue || {
    yes()
  }
  ifFalse: || {
    no()
  }
```

Parenthesized closure arguments remain ordinary expressions and may use expression bodies:

```phalcom
numbers.map(|value| value * 2)

result.match(
  ok: |value| use(value),
  err: |error| handle(error)
)
```

This task is strictly a parser/AST surface migration. The parser must continue producing the existing AST abstractions:

```rust
Expr::Block(Box<BlockExpr>)
Expr::MethodCall(Box<MethodCallExpr>)
Argument
```

Do **not** add any of the following:

```rust
Expr::Closure(...)
Expr::Lambda(...)
Object::Lambda(...)
Bytecode::MakeLambda
```

---

## 2. Existing repository model to preserve

The current AST already represents anonymous executable values as:

```rust
pub struct BlockExpr {
    pub params: Vec<String>,
    pub body: Vec<Statement>,
    pub expr_body: bool,
    pub range: SourceRange,
}
```

and exposes them through:

```rust
Expr::Block(Box<BlockExpr>)
```

The call AST already contains the information trailing closures need: a method base name plus an ordered `Vec<Argument>`, each with an optional label.

Therefore:

- a closure literal is not a new AST category;
- a trailing closure is not a new call-expression category;
- selector identity must continue to be derived by the existing compiler selector encoder;
- the parser must only construct ordinary `Expr::Block` and `MethodCallExpr` nodes.

Architectural target:

```text
source
  |x, y| { ... }
        │
        ▼
phalcom-ast
  Expr::Block(BlockExpr {
      params: ["x", "y"],
      body: ...,
      ...
  })
```

and:

```phalcom
users.any where: |user| {
  user.expired
}
```

must parse as:

```text
Expr::MethodCall {
    object: users,
    method: "any",
    args: [
        Argument {
            label: Some("where"),
            expr: Expr::Block(...),
        }
    ]
}
```

---

## 3. Final closure-literal grammar

Canonical zero-parameter closures:

```phalcom
|| expression
```

or:

```phalcom
|| {
  statements
}
```

Closures with parameters:

```phalcom
|x| expression

|x, y| expression

|x, y| {
  statements
}
```

Conceptual grammar:

```text
ClosureLiteral
  := "|" ClosureParameters? "|" ClosureBody

ClosureParameters
  := Identifier ("," Identifier)*

ClosureBody
  := Expression
   | "{" BlockStatements "}"
```

### 3.1 Lexer rule

Do **not** add `Token::DoublePipe`.

The existing lexer already emits `|` as `Token::Pipe`. Zero-arity closure syntax is therefore:

```text
Pipe Pipe
```

at parser level.

This avoids creating a lexer/operator conflict with the already-overloadable `|` token.

### 3.2 Parameter scope

This implementation supports only the current block-parameter semantic category:

```phalcom
|x, y| ...
```

Do not add:

```phalcom
|x: Int| ...
|label x| ...
|*xs| ...
|(x, y)| ...
|x = default| ...
```

Do not implement type annotations, destructuring, rest parameters, defaults, or named parameters.

Prefer no trailing comma initially:

```phalcom
|x, y|    // valid
|x, y,|   // invalid for now
```

---

## 4. Closure body construction

### 4.1 Expression body

For:

```phalcom
|x, y| x + y
```

construct the existing block representation with `expr_body = true`:

```rust
let expr = self.parse_expr()?;
let expr_range = expr.range();

BlockExpr {
    params,
    body: vec![
        Statement::Expr {
            expr,
            range: expr_range,
        }
    ],
    expr_body: true,
    range: whole_closure_range,
}
```

### 4.2 Braced body

For:

```phalcom
|x, y| {
  x + y
}
```

construct:

```rust
BlockExpr {
    params,
    body: self.parse_block_statements()?,
    expr_body: false,
    range: whole_closure_range,
}
```

No parser-side or compiler-side implicit-return mechanism is needed. Existing block value semantics already use the final expression.

### 4.3 Header newlines

Parser-local newline skipping may be allowed while a closure header is structurally incomplete:

```phalcom
|x,
 y| {
  ...
}
```

and:

```phalcom
|x, y|
{
  ...
}
```

Canonical formatting remains:

```phalcom
|x, y| {
  ...
}
```

Do not globally suppress newline tokens after `Pipe` in the lexer. `Pipe` remains both a closure delimiter and bitwise operator.

---

## 5. General closure expressions vs trailing closures

Keep two concepts distinct:

1. closure literals are general expressions;
2. trailing closures are method-send syntax that omits argument-list parentheses.

Once `parse_primary()` recognizes `Token::Pipe`, ordinary parenthesized calls should work through the existing `parse_arg_list()` path:

```phalcom
items.map(|x| x + 1)

items.map(indexed: |i, x| {
  ...
})

result.match(
  ok: |value| use(value),
  err: |error| handle(error)
)
```

Do not introduce a trailing-closure AST.

---

## 6. Mandatory `|` / bitwise-OR disambiguation

This is the highest-risk grammar constraint.

`Token::Pipe` already represents:

```phalcom
a | b
```

and maps to:

```rust
BinaryOp::BitOr
```

A naïve postfix loop that treats every `Pipe` after a completed expression as a possible trailing closure will break valid bitwise expressions.

For example:

```phalcom
flags | mask | other
```

must remain a bitwise expression and must never be interpreted as:

```text
flags <trailing closure |mask| other>
```

### 6.1 Ratified disambiguation rule

An **unparenthesized trailing closure must have a braced body**.

Valid:

```phalcom
items.map |x| {
  x + 1
}
```

Valid:

```phalcom
flag.ifTrue || {
  work()
}
```

Not a trailing-closure form:

```phalcom
items.map |x| x + 1
```

Users must write either:

```phalcom
items.map(|x| x + 1)
```

or:

```phalcom
items.map |x| {
  x + 1
}
```

This gives the parser a structural recognizer:

```text
"|" params? "|" [newline]* "{"
```

before it commits to trailing-closure parsing.

Do not consume a `Pipe` in postfix/trailing-call parsing unless the braced-closure lookahead has already succeeded.

---

## 7. Closure parser structure

Do not duplicate closure parsing between primary expressions and trailing calls.

Recommended design:

```rust
enum ClosureBodyRequirement {
    Any,
    Braced,
}

fn parse_closure_literal(
    &mut self,
    body_requirement: ClosureBodyRequirement,
) -> ParserResult<Expr>
```

with:

- `parse_primary()` using `ClosureBodyRequirement::Any`;
- trailing-call parsing using `ClosureBodyRequirement::Braced`.

Equivalent factoring into header/body helpers is acceptable if the grammar remains single-sourced.

Also add a non-consuming lookahead:

```rust
fn starts_braced_closure_literal(&self, pos: usize) -> bool
```

The recognizer must accept exactly the same parameter-header grammar as the real parser.

Conceptually:

```rust
fn starts_braced_closure_literal_at(&self, mut i: usize) -> bool {
    if !matches!(token(i), Token::Pipe) {
        return false;
    }
    i += 1;

    // ||
    if matches!(token(i), Token::Pipe) {
        i += 1;
        i = skip_newlines_at(i);
        return matches!(token(i), Token::LBrace);
    }

    // |x, y, z|
    loop {
        i = skip_newlines_at(i);

        if !is_closure_parameter_name(token(i)) {
            return false;
        }
        i += 1;

        i = skip_newlines_at(i);

        match token(i) {
            Token::Comma => i += 1,
            Token::Pipe => {
                i += 1;
                break;
            }
            _ => return false,
        }
    }

    i = skip_newlines_at(i);
    matches!(token(i), Token::LBrace)
}
```

The recognizer must not be more permissive than the parser.

---

## 8. Concrete parser changes

Primary file:

```text
phalcom-ast/src/parser.rs
```

### 8.1 `starts_expression()`

Add `Token::Pipe` to the set of valid expression-start tokens.

This is required not only for direct expressions, but for parser branches that ask whether an expression follows a delimiter.

Also update expected-primary diagnostics so `|` is recognized as a legal primary-expression starter.

### 8.2 `parse_primary()`

Add:

```rust
Token::Pipe => {
    self.parse_closure_literal(ClosureBodyRequirement::Any)
}
```

before the unexpected-primary fallback.

A `Pipe` must only become a closure delimiter when parsing an expression start or an explicitly confirmed trailing closure. The Pratt/binary parser must continue treating `Pipe` after a completed left operand as bitwise OR.

### 8.3 Remove legacy unbraced arrow closures

Current parsing includes a special case approximately like:

```rust
Token::Identifier(value) if peek_next == FatArrow => {
    // construct Expr::Block
}
```

Remove that anonymous-closure behavior.

After migration:

```phalcom
x => x + 1
```

is invalid as a closure.

However, `Token::FatArrow` must remain in the lexer and parser because method/getter expression bodies still use syntax such as:

```phalcom
size => _size
```

Do not mechanically replace member expression bodies.

### 8.4 Remove bare `{ ... }` block literals from primary-expression position

The current `Token::LBrace` primary branch performs two jobs:

1. classify Map associations;
2. otherwise construct a block literal, optionally using an internal `=>` parameter header.

Keep the currently landed B.3a Map-association classification:

```phalcom
{ key: value }

{ [computedKey]: value }

{ **mapping, key: value }
```

according to the current reservation behavior.

Delete the fallback that turns an arbitrary brace expression into `Expr::Block`.

After this task:

```phalcom
{ doSomething() }
```

is not a closure.

Use:

```phalcom
|| {
  doSomething()
}
```

Likewise:

```phalcom
{ x => x + 1 }
```

must no longer create a block.

Use:

```phalcom
|x| {
  x + 1
}
```

### 8.5 Do not implement B.3b in this task

The removal of bare block expressions resolves the parser conflict that currently blocks future:

```phalcom
{}         // empty Map
{a, b, c}  // Set
```

but this task must not decide or implement those forms.

If a brace expression is neither a currently valid B.3a Map form nor another already-supported brace construct, it should currently be a syntax error.

The B.3b documentation unblock belongs to Task 3.

### 8.6 Preserve explicit brace-body parsers

Do not remove:

```rust
parse_brace_block()
parse_block_statements()
```

Parser-known control constructs still use braces and may synthesize `Expr::Block`.

For example:

```phalcom
if (condition) {
  ...
}
```

remains valid.

The important distinction is:

```text
{ ... }          as a primary expression      => no closure

if (...) { ... } parser-known control body    => may lower to Expr::Block

|x| { ... }      explicit closure literal     => Expr::Block
```

---

## 9. Refactor postfix/call parsing

The current `parse_call()` has a large postfix loop and a special `Token::LBrace` branch for legacy trailing blocks.

That path can currently:

- convert a `GetProperty` to a `MethodCall`;
- append a block to an existing `MethodCall`;
- turn an arbitrary expression into `.call(block)`.

Delete the legacy `LBrace` trailing-block path.

Replace it with explicit trailing-closure recognition and argument attachment.

### 9.1 Do not infer eligibility solely from AST shape

Use parser-local state to track whether the current expression still represents an explicit member-send head eligible for trailing arguments.

This matters because some syntax desugars immediately. In particular, `?.` lowers into an `Option#map` call containing a synthesized block. If eligibility is inferred from `Expr::MethodCall`, then:

```phalcom
obj?.foo |x| {
  ...
}
```

could accidentally attach the closure to the synthetic outer `map`, silently changing semantics.

Recommended parser state:

```rust
enum TrailingTarget {
    None,
    MemberSend,
}
```

or an equivalent precisely named boolean.

Set `MemberSend` after an ordinary explicit member getter/call produced by `.`.

Preserve eligibility after parenthesized arguments are attached to that same send.

Set `None` after:

- optional-send desugaring;
- method references;
- index expressions;
- generic callable invocation;
- other postfix forms whose trailing-call semantics are not explicitly designed.

Do not accidentally expand trailing closure semantics because two AST forms look alike.

### 9.2 Attach trailing arguments through one helper

Provide:

```rust
fn attach_trailing_arguments(
    expr: Expr,
    args: Vec<Argument>,
    end: usize,
) -> ParserResult<Expr>
```

For:

```rust
Expr::GetProperty(gp)
```

produce:

```rust
Expr::MethodCall(Box::new(MethodCallExpr {
    object: gp.object,
    method: gp.property,
    args,
    range: gp.range.start..end,
}))
```

For:

```rust
Expr::MethodCall(mut call)
```

append the arguments and extend `call.range.end`.

Anything else reaching this helper is an internal parser invariant error.

Do not lower trailing closures through `.call`.

Trailing closure syntax in this task is **send syntax**.

First-class function invocation remains:

```phalcom
fn(|x| ...)
fn.call(|x| ...)
```

---

## 10. Reuse the ordinary argument-label grammar

Do not create a second definition of an argument label.

`parse_arg_list()` already uses the parser's label helper before `:`. Extract or reuse that exact path for trailing clauses.

This matters because labels may not be equivalent to a naïve `Token::Identifier` check. Existing design examples include labels such as:

```phalcom
true:
false:
```

These forms must describe identical selector identity:

```phalcom
result.match(
  ok: |v| { ... },
  err: |e| { ... }
)
```

and:

```phalcom
result.match
  ok: |v| { ... },
  err: |e| { ... }
```

Store ordinary:

```rust
Argument {
    label: Option<String>,
    expr,
    range,
}
```

Do not encode selectors in the parser.

---

## 11. Supported trailing argument shapes

### 11.1 One positional trailing closure

```phalcom
items.map |item| {
  item.name
}
```

Labels:

```text
[None]
```

### 11.2 Existing parenthesized arguments plus a trailing closure

Support:

```phalcom
items.fold(initial) |acc, item| {
  ...
}
```

when the selector shape is otherwise valid.

The trailing block is appended to the existing `MethodCallExpr.args`.

### 11.3 One labeled trailing closure

```phalcom
items.any where: |item| {
  item.valid
}
```

Labels:

```text
[Some("where")]
```

### 11.4 Multiple labeled closures

```phalcom
result.match
  ok: |value| {
    ...
  },
  err: |error| {
    ...
  }
```

### 11.5 Positional followed by labeled closure

```phalcom
predicate
  .ifTrue || {
    ...
  }
  ifFalse: || {
    ...
  }
```

Labels:

```text
[None, Some("ifFalse")]
```

### 11.6 Separators

Between trailing closure arguments:

- comma allows continuation on the same or next line;
- newline allows a following **labeled** trailing closure without a comma;
- same-line adjacent arguments require a comma.

Valid:

```phalcom
foo
  first: |x| { ... },
  second: |y| { ... }
```

Also valid:

```phalcom
foo
  first: |x| { ... }
  second: |y| { ... }
```

Do not accept:

```phalcom
foo first: |x| { ... } second: |y| { ... }
```

without a comma.

When newline is the separator, only continue if lookahead confirms:

```text
<label> ":" <braced closure>
```

Otherwise the newline terminates the expression normally.

---

## 12. Leading-dot/postfix continuation

Canonical examples require:

```phalcom
numbers
  .map |value| {
    ...
  }
  .where |value| {
    ...
  }
```

and:

```phalcom
users
  .removeAll where: |user| {
    ...
  }
  .any where: |user| {
    ...
  }
```

The current parser preserves the newline after the base expression and `parse_call()` does not consume it before `.`. This task must add a **narrow parser-level continuation exception**.

### 12.1 Required rule

Inside `parse_call()`, when the current token is `Newline`, look past consecutive newlines.

If the next token is a postfix token that cannot reasonably begin a new statement, consume those newlines and continue the same expression.

At minimum support:

```text
.
?.
::
```

Do not automatically extend this rule to:

```text
(
[
|
```

because those tokens introduce real statement-boundary ambiguity.

For a labeled trailing closure after a newline, consume the newline only when lookahead verifies:

```text
label ":" "|" ... "|" "{"
```

For a positional `|...| { ... }` closure, rely on explicit member-send/trailing-target state rather than global newline attachment.

Do not change the lexer to globally suppress all newlines after identifiers.

### 12.2 Required lexical-documentation clarification

The lexer may continue emitting newline tokens. Update comments/specification to state the policy conceptually as:

```text
The lexer retains the newline. The parser recognizes a narrow leading-postfix
continuation for member-chain punctuation and structurally confirmed trailing
closure clauses.
```

---

## 13. Source ranges

Do not preserve the range bug from the legacy trailing-block path.

For:

```phalcom
where: |user| {
  user.expired
}
```

the `Argument.range` must begin at `where` and end at the closure's closing `}`.

For an unlabeled trailing closure:

```phalcom
|value| {
  ...
}
```

the `Argument.range` begins at the opening `|`.

`BlockExpr.range` begins at the opening `|` and ends at the expression-body end or closing `}`.

`MethodCallExpr.range` must extend through the last trailing argument.

---

## 14. Diagnostics

Add migration-oriented diagnostics where they can be recognized cheaply.

### 14.1 Old arrow closure

For expression-position:

```phalcom
x => x + 1
```

prefer:

```text
anonymous `=>` closures were removed; write `|x| expression`
```

### 14.2 Old braced parameter closure

For:

```phalcom
{ x => ... }
```

prefer:

```text
brace block literals were removed; write `|x| { ... }`
```

### 14.3 Old zero-argument brace closure

For an expression-position brace that is not a valid current Map literal, prefer:

```text
bare brace block literals were removed; write `|| { ... }` for a closure
```

Phrase this as a statement about the **current** grammar; do not foreclose future B.3b Set syntax.

### 14.4 Unbraced trailing expression closure

If recovery can support it without complicating binary parsing, detect:

```phalcom
items.map |x| x + 1
```

and suggest:

```text
a trailing closure must use a braced body; write
`.map |x| { x + 1 }` or `.map(|x| x + 1)`
```

The diagnostic is optional; the braced-only grammar rule is mandatory.

---

## 15. Parser and AST tests

Tests must assert AST shape, not merely successful parsing.

### 15.1 Expression-body closure

Input:

```phalcom
const f = |x, y| x + y
```

Assert:

```text
Statement::Let
  value = Expr::Block
    params = ["x", "y"]
    expr_body = true
    body.len() = 1
```

### 15.2 Braced closure

Input:

```phalcom
const f = |x, y| {
  x + y
}
```

Assert:

```text
Expr::Block
  params = ["x", "y"]
  expr_body = false
```

### 15.3 Zero-parameter closure

```phalcom
const f = || {
  1
}
```

Assert zero parameters.

### 15.4 Labeled trailing closure

```phalcom
items.any where: |item| {
  item.valid
}
```

Assert:

```text
MethodCall.method = "any"
args.len() = 1
args[0].label = Some("where")
args[0].expr = Expr::Block
```

### 15.5 Multiple labeled closures

```phalcom
result.match
  ok: |v| { v },
  err: |e| { e }
```

Assert labels in order:

```text
["ok", "err"]
```

### 15.6 Positional plus labeled

```phalcom
predicate
  .ifTrue || { 1 }
  ifFalse: || { 2 }
```

Assert:

```text
method = "ifTrue"
labels = [None, Some("ifFalse")]
```

---

## 16. Mandatory ambiguity regression tests

### 16.1 Bitwise OR remains bitwise

These must remain `Expr::Binary(BitOr)` chains:

```phalcom
a | b

a | b | c

obj.flags | mask | other
```

The final example must not become a trailing call.

### 16.2 Structurally complete trailing closure wins

```phalcom
obj.map |value| {
  value
}
```

must become a `map(_)` send with one `Expr::Block` argument.

### 16.3 Closure as ordinary primary on operator RHS

Where ordinary precedence permits it, closure literals remain primary expressions.

A representative syntax regression is:

```phalcom
something | (|x| x)
```

Runtime type validity is out of scope; parser correctness is the requirement.

---

## 17. Newline and chaining tests

The following must parse as one expression:

```phalcom
numbers
  .map |value| {
    value * 2
  }
  .filter |value| {
    value > 10
  }
```

Also:

```phalcom
users
  .removeAll where: |user| {
    user.expired
  }
  .any where: |user| {
    user.expired
  }
```

And:

```phalcom
result.match
  ok: |v| {
    v
  },
  err: |e| {
    e
  }
```

Negative boundary:

```phalcom
foo
|x| {
  x
}
```

When `foo` is already a complete independent expression and no member-send/trailing context exists, the closure on the next line must **not** silently attach to `foo`.

This is why newline consumption before positional trailing closures must not be global.

---

## 18. Parenthesized-equivalence tests

For each important trailing shape, compare semantic AST shape with its parenthesized equivalent.

### Positional

```phalcom
items.map |x| {
  x
}
```

vs:

```phalcom
items.map(|x| {
  x
})
```

### Labeled

```phalcom
items.any where: |x| {
  x
}
```

vs:

```phalcom
items.any(where: |x| {
  x
})
```

### Multiple labeled

```phalcom
result.match
  ok: |v| { v },
  err: |e| { e }
```

vs:

```phalcom
result.match(
  ok: |v| { v },
  err: |e| { e }
)
```

Do not require byte-for-byte range equality.

Require identical:

- method base name;
- argument count;
- argument order;
- argument labels;
- `Expr::Block` parameter/body shape.

---

## 19. Lexer tests

No lexer grammar extension is expected.

Add explicit tests showing:

```phalcom
|| {}
```

lexes as:

```text
Pipe
Pipe
LBrace
RBrace
```

and:

```phalcom
|x, y|
```

uses ordinary existing tokens.

Retain existing bitwise-`|` lexer snapshots.

Do not:

- create `Token::DoublePipe`;
- reinterpret `||` as logical OR;
- change overloadable `|` method parsing.

Phalcom's logical OR remains the existing `or` keyword.

---

## 20. Implementation phases for this task

### Phase A — New closure primary

Implement:

- `Token::Pipe` in expression-start recognition;
- `parse_closure_literal`;
- zero, one, and multiple parameters;
- expression body;
- braced body;
- closure source ranges;
- AST shape tests.

During local TDD, legacy syntax may temporarily coexist if necessary.

### Phase B — Trailing closure and postfix continuation

Implement:

- braced trailing-closure lookahead;
- parser-local trailing-target state;
- leading-dot/postfix continuation;
- positional trailing closure;
- labeled trailing closure;
- multiple labeled closures;
- positional + labeled closures;
- range-preserving argument attachment;
- conversion from `GetProperty` to `MethodCall`;
- extension of existing `MethodCallExpr.args`;
- bitwise ambiguity regressions;
- parenthesized-equivalence tests.

### Phase C — Remove old parser forms after executable migration

Task 2 owns source migration. Once executable source has been migrated, remove:

- `Identifier FatArrow` anonymous-closure parsing;
- `LBrace` fallback to `BlockExpr`;
- legacy `LBrace` trailing-call branch;
- `parse_block_params()` if no remaining consumer exists;
- any helper used exclusively by retired syntax.

Then add migration diagnostics.

---

## 21. Acceptance criteria for Task 1

The parser work is complete only when all of the following hold.

### Closure syntax parses

```phalcom
const a = || 1
const b = |x| x + 1
const c = |x, y| x + y

const d = || {
  1
}

const e = |x, y| {
  x + y
}
```

### Ordinary closure arguments parse

```phalcom
items.map(|x| x + 1)
items.map(indexed: |i, x| { ... })
```

### Trailing closure parses

```phalcom
items.map |x| {
  x + 1
}
```

as `map(_)`.

### Labeled trailing closure parses

```phalcom
items.any where: |x| {
  predicate(x)
}
```

as `any(where:)`.

### Multiple labeled closures parse as one send

```phalcom
result.match
  ok: |v| {
    ...
  },
  err: |e| {
    ...
  }
```

### Mixed positional/labeled closure preserves selector shape

```phalcom
predicate
  .ifTrue || {
    ...
  }
  ifFalse: || {
    ...
  }
```

matches the parenthesized equivalent.

### Chaining works across newlines

```phalcom
numbers
  .map |x| {
    x
  }
  .filter |x| {
    x.valid
  }
```

is one expression.

### Bitwise semantics remain intact

```phalcom
a | b
a | b | c
obj.flags | mask | other
```

remain bitwise expressions.

### Old anonymous syntax no longer constructs closures

```phalcom
x => x + 1
{ x => x + 1 }
```

are rejected as anonymous closure forms.

### Existing B.3a Map syntax still parses

Currently landed association Map forms must remain unchanged.

---

## 22. Explicit non-goals

Do not implement:

- function/closure type annotations;
- `->` function-type syntax;
- parameter type annotations;
- closure destructuring;
- rest parameters;
- named/default closure parameters;
- async closures;
- capture modifiers;
- capture-by-value syntax;
- implicit `self`;
- automatic trailing closures after `?.`;
- arbitrary non-closure unparenthesized arguments;
- Set literals;
- empty-Map `{}`;
- generalized whitespace-sensitive `|` lexing;
- runtime closure changes.

---

## 23. Handoff to Task 2

Task 2 must migrate executable `.ph` source before the old parser forms are permanently removed from the branch.

Parser implementation is correct only if it preserves the architectural invariant:

```text
closure source
    ↓
Expr::Block
```

and:

```text
trailing closure send
    ↓
ordinary MethodCallExpr + Argument
```

Everything below those AST abstractions must remain unchanged.
